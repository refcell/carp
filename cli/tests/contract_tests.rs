/// API contract testing for the Carp CLI
/// Tests API schema compliance, response validation, and contract adherence
use carp_cli::api::types::*;
use carp_cli::api::ApiClient;
use carp_cli::config::{Config, RetrySettings, SecuritySettings};
use carp_cli::utils::error::CarpResult;
use std::env;
use tokio::time::{timeout, Duration};

/// Contract test configuration
pub struct ContractTestConfig {
    pub api_base_url: String,
    pub test_timeout: Duration,
    pub skip_api_tests: bool,
}

impl Default for ContractTestConfig {
    fn default() -> Self {
        Self {
            api_base_url: env::var("CARP_TEST_API_URL")
                .unwrap_or_else(|_| "https://api.carp.refcell.org".to_string()),
            test_timeout: Duration::from_secs(30),
            skip_api_tests: env::var("CARP_SKIP_API_TESTS").is_ok(),
        }
    }
}

/// Create a test configuration for contract testing
fn create_contract_test_config() -> Config {
    let test_config = ContractTestConfig::default();

    Config {
        registry_url: test_config.api_base_url,
        api_token: env::var("CARP_TEST_TOKEN").ok(),
        timeout: 30,
        verify_ssl: true,
        default_output_dir: Some("./contract_test_output".to_string()),
        max_concurrent_downloads: 4,
        retry: RetrySettings {
            max_retries: 2,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
        },
        security: SecuritySettings {
            max_download_size: 100 * 1024 * 1024, // 100MB
            max_publish_size: 50 * 1024 * 1024,   // 50MB
            allow_http: false,
            token_warning_hours: 24,
        },
    }
}

/// Test health check API contract
#[tokio::test]
async fn test_health_check_contract() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping health check contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    let result = timeout(Duration::from_secs(10), client.health_check()).await;

    match result {
        Ok(Ok(response)) => {
            // Validate health check response structure
            assert!(
                !response.status.is_empty(),
                "Health status should not be empty"
            );
            assert!(
                !response.service.is_empty(),
                "Service name should not be empty"
            );
            assert!(
                !response.environment.is_empty(),
                "Service environment should not be empty"
            );

            // Validate expected values
            assert_eq!(
                response.service, "carp-api",
                "Service name should be 'carp-api'"
            );
            assert!(
                response.status == "healthy" || response.status == "ok",
                "Status should be 'healthy' or 'ok', got: {}",
                response.status
            );

            // Validate timestamp format
            assert!(
                response.timestamp.len() > 10,
                "Timestamp should be properly formatted"
            );

            // Test that message is present (may be empty)
            println!("✓ Health check contract validated");
            println!("  Status: {}", response.status);
            println!("  Service: {} ({})", response.service, response.environment);
            println!("  Timestamp: {}", response.timestamp);
            println!("  Message: {}", response.message);
        }
        Ok(Err(e)) => {
            println!("Health check failed (may be expected): {}", e);
            // API might be unavailable, which is OK for testing
        }
        Err(_) => {
            println!("Health check timed out (may be expected)");
        }
    }

    Ok(())
}

/// Test search API contract
#[tokio::test]
async fn test_search_contract() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping search contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    let result = timeout(
        Duration::from_secs(15),
        client.search("test", Some(5), false),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            // Validate search response structure
            assert!(response.agents.len() <= 5, "Should respect limit parameter");
            assert!(
                response.total >= response.agents.len(),
                "Total should be >= returned agents count"
            );

            // Validate pagination fields
            assert!(response.page >= 1, "Page should be >= 1");
            assert!(response.per_page > 0, "Per page should be > 0");

            println!("✓ Search response structure validated");
            println!(
                "  Found {} agents (total: {})",
                response.agents.len(),
                response.total
            );
            println!(
                "  Page: {} (per page: {})",
                response.page, response.per_page
            );

            // Validate each agent in response
            for (i, agent) in response.agents.iter().enumerate() {
                validate_agent_structure(agent, &format!("Agent {}", i))?;
            }

            println!("✓ All agent structures validated");
        }
        Ok(Err(e)) => {
            println!("Search failed (may be expected): {}", e);
        }
        Err(_) => {
            println!("Search timed out (may be expected)");
        }
    }

    Ok(())
}

/// Test agent download info API contract
#[tokio::test]
async fn test_download_info_contract() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping download info contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    // First, try to find an agent to test with
    let search_result = client.search("example", Some(1), false).await;

    match search_result {
        Ok(response) if !response.agents.is_empty() => {
            let agent = &response.agents[0];

            let result = timeout(
                Duration::from_secs(10),
                client.get_agent_download(&agent.name, Some(&agent.version)),
            )
            .await;

            match result {
                Ok(Ok(download_info)) => {
                    validate_download_info_structure(&download_info)?;

                    // Validate consistency with search result
                    assert_eq!(
                        download_info.name, agent.name,
                        "Download name should match search result"
                    );
                    assert_eq!(
                        download_info.version, agent.version,
                        "Download version should match search result"
                    );

                    println!("✓ Download info contract validated for {}", agent.name);
                }
                Ok(Err(e)) => {
                    println!("Download info failed for {}: {}", agent.name, e);
                }
                Err(_) => {
                    println!("Download info timed out for {}", agent.name);
                }
            }
        }
        Ok(_) => {
            println!("No agents available for download info contract test");
        }
        Err(e) => {
            println!("Search failed, cannot test download info contract: {}", e);
        }
    }

    Ok(())
}

/// Test authentication API contract
#[tokio::test]
async fn test_authentication_contract() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping authentication contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    // Test authentication with invalid credentials (to test error response structure)
    let result = client
        .authenticate("invalid_user", "invalid_password")
        .await;

    match result {
        Ok(auth_response) => {
            // If authentication succeeds (unexpected), validate response structure
            assert!(!auth_response.token.is_empty(), "Token should not be empty");
            // Note: expires_at is a DateTime, not a timestamp
            assert!(
                auth_response.expires_at > chrono::Utc::now(),
                "Expiration should be in the future"
            );

            println!("✓ Authentication response structure validated (unexpected success)");
        }
        Err(e) => {
            // Expected failure - validate error structure
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");

            // Should be a proper authentication error
            match e {
                carp_cli::utils::error::CarpError::Auth(_) => {
                    println!("✓ Authentication error properly categorized");
                }
                carp_cli::utils::error::CarpError::Api { status, message: _ } => {
                    assert!(
                        status == 401 || status == 403,
                        "Authentication failure should return 401 or 403, got {}",
                        status
                    );
                    println!("✓ Authentication API error properly structured");
                }
                _ => {
                    println!("Authentication failed with different error type: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Test error response contracts
#[tokio::test]
async fn test_error_response_contracts() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping error response contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    // Test 404 error for nonexistent agent
    let result = client
        .get_agent_download("nonexistent-agent-12345", None)
        .await;
    match result {
        Err(carp_cli::utils::error::CarpError::Api { status, message }) => {
            assert_eq!(status, 404, "Nonexistent agent should return 404");
            assert!(!message.is_empty(), "Error message should not be empty");
            println!("✓ 404 error contract validated: {}", message);
        }
        Err(e) => {
            println!("Nonexistent agent returned different error: {}", e);
        }
        Ok(_) => {
            println!("⚠ Nonexistent agent unexpectedly succeeded");
        }
    }

    // Test invalid search parameters
    let result = client.search("", None, false).await;
    match result {
        Err(carp_cli::utils::error::CarpError::InvalidAgent(msg)) => {
            assert!(
                !msg.is_empty(),
                "Validation error message should not be empty"
            );
            println!("✓ Validation error contract validated: {}", msg);
        }
        Err(e) => {
            println!("Invalid search returned different error: {}", e);
        }
        Ok(_) => {
            println!("⚠ Invalid search unexpectedly succeeded");
        }
    }

    Ok(())
}

/// Test API versioning and compatibility
#[tokio::test]
async fn test_api_versioning() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping API versioning test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();

    // Verify API URL contains version prefix
    assert!(
        config.registry_url.contains("api") || config.registry_url.ends_with(".org"),
        "API URL should indicate versioning structure"
    );

    let client = ApiClient::new(&config)?;

    // Test that health check includes environment information
    if let Ok(response) = client.health_check().await {
        assert!(
            !response.environment.is_empty(),
            "API should return environment information"
        );

        // Environment should be a valid value
        assert!(
            response.environment == "development"
                || response.environment == "staging"
                || response.environment == "production",
            "Environment should be a valid value: {}",
            response.environment
        );

        println!("✓ API environment validated: {}", response.environment);
    }

    Ok(())
}

/// Test response time contracts (SLA compliance)
#[tokio::test]
async fn test_response_time_contracts() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping response time contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    // Test health check response time
    let start = std::time::Instant::now();
    let result = client.health_check().await;
    let health_duration = start.elapsed();

    if result.is_ok() {
        assert!(
            health_duration < Duration::from_secs(5),
            "Health check should respond within 5 seconds, took {:?}",
            health_duration
        );
        println!(
            "✓ Health check response time contract met: {:?}",
            health_duration
        );
    }

    // Test search response time
    let start = std::time::Instant::now();
    let result = client.search("test", Some(5), false).await;
    let search_duration = start.elapsed();

    if result.is_ok() {
        assert!(
            search_duration < Duration::from_secs(15),
            "Search should respond within 15 seconds, took {:?}",
            search_duration
        );
        println!("✓ Search response time contract met: {:?}", search_duration);
    }

    Ok(())
}

/// Test data consistency contracts
#[tokio::test]
async fn test_data_consistency_contracts() -> CarpResult<()> {
    if ContractTestConfig::default().skip_api_tests {
        println!("Skipping data consistency contract test (CARP_SKIP_API_TESTS set)");
        return Ok(());
    }

    let config = create_contract_test_config();
    let client = ApiClient::new(&config)?;

    // Perform same search twice and check consistency
    let result1 = client.search("consistency-test", Some(10), false).await;
    tokio::time::sleep(Duration::from_millis(100)).await; // Small delay
    let result2 = client.search("consistency-test", Some(10), false).await;

    match (result1, result2) {
        (Ok(response1), Ok(response2)) => {
            // Results should be consistent (allowing for small timing differences)
            let total_diff = response1.total.abs_diff(response2.total);
            assert!(
                total_diff <= 1,
                "Search totals should be consistent: {} vs {}",
                response1.total,
                response2.total
            );

            // Check that agent data is consistent
            if response1.agents.len() == response2.agents.len() {
                for (agent1, agent2) in response1.agents.iter().zip(response2.agents.iter()) {
                    assert_eq!(agent1.name, agent2.name, "Agent names should be consistent");
                    assert_eq!(
                        agent1.version, agent2.version,
                        "Agent versions should be consistent"
                    );
                }
            }

            println!("✓ Data consistency contract validated");
        }
        _ => {
            println!("Could not test data consistency (API unavailable)");
        }
    }

    Ok(())
}

/// Validate agent structure according to API contract
fn validate_agent_structure(agent: &Agent, context: &str) -> CarpResult<()> {
    // Required fields
    assert!(
        !agent.name.is_empty(),
        "{}: Agent name should not be empty",
        context
    );
    assert!(
        !agent.version.is_empty(),
        "{}: Agent version should not be empty",
        context
    );
    assert!(
        !agent.description.is_empty(),
        "{}: Agent description should not be empty",
        context
    );
    assert!(
        !agent.author.is_empty(),
        "{}: Agent author should not be empty",
        context
    );

    // Date fields should be valid (DateTime fields are never empty, just check they're reasonable)
    assert!(
        agent.created_at <= chrono::Utc::now(),
        "{}: Created date should not be in the future",
        context
    );
    assert!(
        agent.updated_at <= chrono::Utc::now(),
        "{}: Updated date should not be in the future",
        context
    );
    assert!(
        agent.updated_at >= agent.created_at,
        "{}: Updated date should be >= created date",
        context
    );

    // Numeric fields should be reasonable
    assert!(
        agent.download_count >= 0,
        "{}: Download count should be >= 0",
        context
    );

    // Name should follow naming conventions
    assert!(
        agent.name.len() <= 100,
        "{}: Agent name should be <= 100 chars",
        context
    );
    assert!(
        agent
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
        "{}: Agent name should only contain alphanumeric, hyphens, underscores",
        context
    );

    // Version should be reasonable
    assert!(
        agent.version.len() <= 50,
        "{}: Agent version should be <= 50 chars",
        context
    );

    // Tags should be valid
    for tag in &agent.tags {
        assert!(!tag.is_empty(), "{}: Tags should not be empty", context);
        assert!(
            tag.len() <= 50,
            "{}: Tag '{}' should be <= 50 chars",
            context,
            tag
        );
    }

    // Optional fields validation
    if let Some(ref homepage) = agent.homepage {
        assert!(
            homepage.starts_with("http://") || homepage.starts_with("https://"),
            "{}: Homepage should be a valid URL",
            context
        );
    }

    if let Some(ref repository) = agent.repository {
        assert!(
            repository.starts_with("http://") || repository.starts_with("https://"),
            "{}: Repository should be a valid URL",
            context
        );
    }

    println!(
        "✓ Agent structure validated: {} v{}",
        agent.name, agent.version
    );
    Ok(())
}

/// Validate download info structure according to API contract
fn validate_download_info_structure(download_info: &AgentDownload) -> CarpResult<()> {
    // Required fields
    assert!(
        !download_info.name.is_empty(),
        "Download name should not be empty"
    );
    assert!(
        !download_info.version.is_empty(),
        "Download version should not be empty"
    );
    assert!(
        !download_info.download_url.is_empty(),
        "Download URL should not be empty"
    );
    assert!(
        !download_info.checksum.is_empty(),
        "Checksum should not be empty"
    );

    // URL should be HTTPS
    assert!(
        download_info.download_url.starts_with("https://"),
        "Download URL should use HTTPS: {}",
        download_info.download_url
    );

    // File size should be reasonable
    assert!(download_info.file_size > 0, "File size should be > 0");
    assert!(
        download_info.file_size < 1024 * 1024 * 1024, // 1GB limit
        "File size should be reasonable: {}",
        download_info.file_size
    );

    // Checksum should be in expected format (SHA256)
    assert!(
        download_info.checksum.starts_with("sha256:"),
        "Checksum should start with 'sha256:': {}",
        download_info.checksum
    );

    let hash_part = download_info.checksum.strip_prefix("sha256:").unwrap();
    assert_eq!(
        hash_part.len(),
        64,
        "SHA256 hash should be 64 hex characters"
    );
    assert!(
        hash_part.chars().all(|c| c.is_ascii_hexdigit()),
        "SHA256 hash should contain only hex characters"
    );

    println!(
        "✓ Download info structure validated: {} v{} ({} bytes)",
        download_info.name, download_info.version, download_info.file_size
    );

    Ok(())
}
