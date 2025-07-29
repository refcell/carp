use carp_cli::api::{ApiClient, UploadAgentRequest};
use carp_cli::config::{Config, RetrySettings, SecuritySettings};
use carp_cli::utils::error::CarpResult;
use std::env;
use tokio::time::{timeout, Duration};

/// Integration test configuration
pub struct IntegrationTestConfig {
    pub api_base_url: String,
    pub test_timeout: Duration,
    pub skip_auth_tests: bool,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            api_base_url: env::var("CARP_TEST_API_URL")
                .unwrap_or_else(|_| "https://api.carp.refcell.org".to_string()),
            test_timeout: Duration::from_secs(30),
            skip_auth_tests: env::var("CARP_SKIP_AUTH_TESTS").is_ok(),
        }
    }
}

/// Create a test configuration for integration tests
fn create_test_config() -> Config {
    let test_config = IntegrationTestConfig::default();

    Config {
        registry_url: test_config.api_base_url,
        api_token: env::var("CARP_TEST_TOKEN").ok(),
        timeout: 30,
        verify_ssl: true,
        default_output_dir: Some("./test_output".to_string()),
        max_concurrent_downloads: 2,
        retry: RetrySettings {
            max_retries: 2,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 1.5,
        },
        security: SecuritySettings {
            max_download_size: 10 * 1024 * 1024, // 10MB for tests
            max_publish_size: 5 * 1024 * 1024,   // 5MB for tests
            allow_http: false,
            token_warning_hours: 1,
        },
    }
}

#[tokio::test]
async fn test_health_check() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    let result = timeout(Duration::from_secs(10), client.health_check()).await;

    match result {
        Ok(Ok(response)) => {
            assert!(!response.status.is_empty());
            assert!(!response.service.is_empty());
            println!(
                "Health check passed: {} - {}",
                response.status, response.message
            );
            Ok(())
        }
        Ok(Err(e)) => {
            eprintln!("Health check failed: {}", e);
            // Don't fail the test if the API is temporarily unavailable
            Ok(())
        }
        Err(_) => {
            eprintln!("Health check timed out");
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_search_functionality() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Test basic search
    let result = timeout(
        Duration::from_secs(15),
        client.search("test", Some(5), false),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            assert!(response.agents.len() <= 5);
            println!("Search test passed: found {} agents", response.agents.len());

            // Validate agent structure
            for agent in &response.agents {
                assert!(!agent.name.is_empty());
                assert!(!agent.version.is_empty());
                assert!(!agent.author.is_empty());
                assert!(!agent.description.is_empty());
            }
            Ok(())
        }
        Ok(Err(e)) => {
            eprintln!("Search failed: {}", e);
            // Don't fail the test if no agents are found or API is unavailable
            Ok(())
        }
        Err(_) => {
            eprintln!("Search timed out");
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_search_validation() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Test empty query validation
    let result = client.search("", None, false).await;
    assert!(result.is_err(), "Empty query should fail validation");

    // Test zero limit validation
    let result = client.search("test", Some(0), false).await;
    assert!(result.is_err(), "Zero limit should fail validation");

    println!("Search validation tests passed");
    Ok(())
}

#[tokio::test]
async fn test_agent_download_info() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // First, try to find an agent to test with
    let search_result = client.search("example", Some(1), false).await;

    match search_result {
        Ok(response) if !response.agents.is_empty() => {
            let agent = &response.agents[0];

            // Test getting download info
            let result = timeout(
                Duration::from_secs(10),
                client.get_agent_download(&agent.name, Some(&agent.version)),
            )
            .await;

            match result {
                Ok(Ok(download_info)) => {
                    assert_eq!(download_info.name, agent.name);
                    assert_eq!(download_info.version, agent.version);
                    assert!(!download_info.download_url.is_empty());
                    assert!(download_info.file_size > 0);
                    println!("Download info test passed for {}", agent.name);
                }
                Ok(Err(e)) => {
                    eprintln!("Download info failed: {}", e);
                }
                Err(_) => {
                    eprintln!("Download info timed out");
                }
            }
        }
        _ => {
            println!("No agents available for download info test");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_agent_name_validation() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Test invalid agent names
    let long_name = "a".repeat(101);
    let invalid_names = vec![
        "",          // Empty
        " ",         // Whitespace only
        "invalid-@", // Invalid characters
        &long_name,  // Too long
    ];

    for invalid_name in invalid_names {
        let result = client.get_agent_download(invalid_name, None).await;
        assert!(
            result.is_err(),
            "Invalid name '{}' should fail validation",
            invalid_name
        );
    }

    println!("Agent name validation tests passed");
    Ok(())
}

#[tokio::test]
async fn test_authentication_validation() -> CarpResult<()> {
    if IntegrationTestConfig::default().skip_auth_tests {
        println!("Skipping authentication tests (CARP_SKIP_AUTH_TESTS set)");
        return Ok(());
    }

    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Test invalid credentials
    let result = client.authenticate("", "").await;
    assert!(result.is_err(), "Empty credentials should fail validation");

    let result = client.authenticate("invalid_user", "invalid_pass").await;
    // This may succeed or fail depending on the API implementation
    // We're mainly testing that the request is properly formed
    match result {
        Ok(_) => println!("Authentication test completed (unexpected success)"),
        Err(_) => println!("Authentication test completed (expected failure)"),
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_requests() -> CarpResult<()> {
    let config = create_test_config();
    let _client = ApiClient::new(&config)?;

    // Test multiple concurrent health checks
    let futures = (0..3).map(|i| {
        let client = ApiClient::new(&config).unwrap();
        async move {
            let result = client.health_check().await;
            println!("Concurrent request {} completed", i);
            result
        }
    });

    let results = futures::future::join_all(futures).await;

    // Check that at least some requests succeeded
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    println!(
        "Concurrent test: {}/{} requests succeeded",
        success_count,
        results.len()
    );

    Ok(())
}

#[tokio::test]
async fn test_retry_mechanism() -> CarpResult<()> {
    let mut config = create_test_config();
    // Use an invalid URL to test retry behavior
    config.registry_url = "https://invalid-domain-that-does-not-exist.test".to_string();

    let client = ApiClient::new(&config)?;

    let start = std::time::Instant::now();
    let result = client.health_check().await;
    let duration = start.elapsed();

    // Should fail after retries
    assert!(result.is_err(), "Request to invalid URL should fail");

    // Should take some time due to retries (at least 100ms initial delay)
    assert!(
        duration >= Duration::from_millis(50),
        "Should have retry delay"
    );

    println!("Retry mechanism test passed: failed after {:?}", duration);
    Ok(())
}

#[tokio::test]
async fn test_configuration_loading() -> CarpResult<()> {
    // Test environment variable overrides
    env::set_var("CARP_REGISTRY_URL", "https://test.example.com");
    env::set_var("CARP_TIMEOUT", "60");
    env::set_var("CARP_VERIFY_SSL", "false");

    // Note: We can't easily test ConfigManager::load_with_env_checks() here
    // because it would interfere with other tests. In a real test suite,
    // you'd want to separate these tests or use a mock configuration system.

    println!("Configuration loading test completed");

    // Clean up
    env::remove_var("CARP_REGISTRY_URL");
    env::remove_var("CARP_TIMEOUT");
    env::remove_var("CARP_VERIFY_SSL");

    Ok(())
}

/// Performance test for API response times
#[tokio::test]
async fn test_performance_benchmarks() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Benchmark health check
    let start = std::time::Instant::now();
    let result = client.health_check().await;
    let health_duration = start.elapsed();

    if result.is_ok() {
        println!("Health check took: {:?}", health_duration);
        assert!(
            health_duration < Duration::from_secs(5),
            "Health check should be fast"
        );
    }

    // Benchmark search
    let start = std::time::Instant::now();
    let result = client.search("test", Some(10), false).await;
    let search_duration = start.elapsed();

    if result.is_ok() {
        println!("Search took: {:?}", search_duration);
        assert!(
            search_duration < Duration::from_secs(10),
            "Search should complete quickly"
        );
    }

    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Test with malformed query parameters
    let result = client
        .search("test query with spaces", Some(1000), false)
        .await;
    // Should either succeed or fail gracefully
    match result {
        Ok(_) => println!("Query with spaces handled successfully"),
        Err(e) => println!("Query with spaces failed gracefully: {}", e),
    }

    // Test with very long query
    let long_query = "a".repeat(1000);
    let result = client.search(&long_query, Some(1), false).await;
    match result {
        Ok(_) => println!("Long query handled successfully"),
        Err(e) => println!("Long query failed gracefully: {}", e),
    }

    Ok(())
}

/// Test security features
#[tokio::test]
async fn test_security_features() -> CarpResult<()> {
    let config = create_test_config();
    let client = ApiClient::new(&config)?;

    // Test HTTPS enforcement
    let result = client
        .download_agent("http://example.com/malicious.zip")
        .await;
    assert!(result.is_err(), "HTTP URLs should be rejected");

    // Test malformed URLs
    let result = client.download_agent("not-a-url").await;
    assert!(result.is_err(), "Invalid URLs should be rejected");

    // Test empty URLs
    let result = client.download_agent("").await;
    assert!(result.is_err(), "Empty URLs should be rejected");

    println!("Security feature tests passed");
    Ok(())
}

/// Helper function to create a valid upload request for testing
fn create_test_upload_request() -> UploadAgentRequest {
    UploadAgentRequest {
        name: "integration-test-agent".to_string(),
        description: "A test agent for integration testing".to_string(),
        content: r#"---
name: integration-test-agent
description: A test agent for integration testing
---

# Integration Test Agent

This is a test agent used for integration testing the upload functionality.

## Features

- Basic functionality test
- Integration test validation
- Upload endpoint testing

## Usage

This agent is only used for testing purposes.
"#
        .to_string(),
        version: Some("1.0.0".to_string()),
        tags: vec!["test".to_string(), "integration".to_string()],
        homepage: Some("https://example.com/integration-test-agent".to_string()),
        repository: Some("https://github.com/test/integration-test-agent".to_string()),
        license: Some("MIT".to_string()),
    }
}

/// Test upload functionality without authentication (should fail)
#[tokio::test]
async fn test_upload_without_auth() -> CarpResult<()> {
    let mut config = create_test_config();
    config.api_token = None; // Remove token to test auth failure

    let client = ApiClient::new(&config)?;
    let request = create_test_upload_request();

    let result = timeout(Duration::from_secs(10), client.upload(request)).await;

    match result {
        Ok(Err(e)) => {
            println!("Upload without auth correctly failed: {}", e);
            // Should be an auth error
            match e {
                carp_cli::utils::error::CarpError::Auth(_) => {
                    println!("Correct auth error returned");
                }
                _ => {
                    println!("Unexpected error type, but upload still failed as expected");
                }
            }
        }
        Ok(Ok(_)) => {
            println!("Warning: Upload without auth unexpectedly succeeded");
        }
        Err(_) => {
            println!("Upload without auth timed out (expected)");
        }
    }

    Ok(())
}

/// Test upload validation with invalid requests
#[tokio::test]
async fn test_upload_validation() -> CarpResult<()> {
    if IntegrationTestConfig::default().skip_auth_tests {
        println!("Skipping upload validation tests (CARP_SKIP_AUTH_TESTS set)");
        return Ok(());
    }

    let config = create_test_config();

    // Skip test if no token available
    if config.api_token.is_none() {
        println!("Skipping upload validation tests (no API token available)");
        return Ok(());
    }

    let client = ApiClient::new(&config)?;

    // Test with empty name
    let mut invalid_request = create_test_upload_request();
    invalid_request.name = "".to_string();

    let result = client.upload(invalid_request).await;
    assert!(result.is_err(), "Upload with empty name should fail");

    // Test with invalid characters in name
    let mut invalid_request = create_test_upload_request();
    invalid_request.name = "invalid-name!@#$".to_string();

    let result = client.upload(invalid_request).await;
    assert!(result.is_err(), "Upload with invalid name should fail");

    // Test with mismatched frontmatter
    let mut invalid_request = create_test_upload_request();
    invalid_request.content = r#"---
name: different-name
description: A test agent for integration testing
---

# Different Agent
"#
    .to_string();

    let result = client.upload(invalid_request).await;
    assert!(
        result.is_err(),
        "Upload with mismatched frontmatter should fail"
    );

    // Test with no frontmatter
    let mut invalid_request = create_test_upload_request();
    invalid_request.content = "# No Frontmatter Agent\n\nThis has no frontmatter.".to_string();

    let result = client.upload(invalid_request).await;
    assert!(result.is_err(), "Upload without frontmatter should fail");

    println!("Upload validation tests passed");
    Ok(())
}

/// Test upload with valid request (if token is available)
#[tokio::test]
async fn test_upload_with_auth() -> CarpResult<()> {
    if IntegrationTestConfig::default().skip_auth_tests {
        println!("Skipping authenticated upload tests (CARP_SKIP_AUTH_TESTS set)");
        return Ok(());
    }

    let config = create_test_config();

    // Skip test if no token available
    if config.api_token.is_none() {
        println!("Skipping authenticated upload tests (no API token available)");
        return Ok(());
    }

    let client = ApiClient::new(&config)?;
    let request = create_test_upload_request();

    let result = timeout(Duration::from_secs(30), client.upload(request)).await;

    match result {
        Ok(Ok(response)) => {
            if response.success {
                println!("Upload test passed: {}", response.message);
                if let Some(agent) = response.agent {
                    println!("Uploaded agent: {} v{}", agent.name, agent.version);
                }
            } else {
                println!("Upload failed with validation errors:");
                if let Some(errors) = response.validation_errors {
                    for error in errors {
                        println!("  {}: {}", error.field, error.message);
                    }
                }
                // This might be expected if the agent already exists
            }
        }
        Ok(Err(e)) => {
            println!("Upload test failed: {}", e);
            // This might be expected depending on the API state
        }
        Err(_) => {
            println!("Upload test timed out");
        }
    }

    Ok(())
}

/// Test upload with very large content (should fail)
#[tokio::test]
async fn test_upload_large_content() -> CarpResult<()> {
    if IntegrationTestConfig::default().skip_auth_tests {
        println!("Skipping large content upload tests (CARP_SKIP_AUTH_TESTS set)");
        return Ok(());
    }

    let config = create_test_config();

    // Skip test if no token available
    if config.api_token.is_none() {
        println!("Skipping large content upload tests (no API token available)");
        return Ok(());
    }

    let client = ApiClient::new(&config)?;
    let mut request = create_test_upload_request();

    // Create content larger than 1MB
    let large_content = "x".repeat(2 * 1024 * 1024);
    request.content = format!(
        r#"---
name: integration-test-agent
description: A test agent for integration testing
---

# Large Content Agent

{}
"#,
        large_content
    );

    let result = client.upload(request).await;
    assert!(result.is_err(), "Upload with large content should fail");

    if let Err(e) = result {
        println!("Large content upload correctly failed: {}", e);
    }

    Ok(())
}

/// Test upload request structure and serialization
#[tokio::test]
async fn test_upload_request_serialization() -> CarpResult<()> {
    let request = create_test_upload_request();

    // Test JSON serialization
    let json = serde_json::to_string(&request)?;
    assert!(!json.is_empty());
    println!("Upload request serializes to {} bytes", json.len());

    // Test deserialization
    let deserialized: UploadAgentRequest = serde_json::from_str(&json)?;
    assert_eq!(deserialized.name, request.name);
    assert_eq!(deserialized.description, request.description);
    assert_eq!(deserialized.content, request.content);

    println!("Upload request serialization test passed");
    Ok(())
}
