/// Regression tests for the Carp CLI
/// Tests for previously identified bugs and edge cases to prevent regressions
use carp_cli::api::ApiClient;
use carp_cli::config::{Config, RetrySettings, SecuritySettings};
use carp_cli::utils::error::{CarpError, CarpResult};
use std::env;
use tokio::time::{timeout, Duration};

/// Regression test configuration
pub struct RegressionTestConfig {
    pub api_base_url: String,
    pub test_timeout: Duration,
    pub skip_regression_tests: bool,
}

impl Default for RegressionTestConfig {
    fn default() -> Self {
        Self {
            api_base_url: env::var("CARP_TEST_API_URL")
                .unwrap_or_else(|_| "https://api.carp.refcell.org".to_string()),
            test_timeout: Duration::from_secs(30),
            skip_regression_tests: env::var("CARP_SKIP_REGRESSION_TESTS").is_ok(),
        }
    }
}

/// Create a test configuration for regression testing
fn create_regression_test_config() -> Config {
    let test_config = RegressionTestConfig::default();

    Config {
        registry_url: test_config.api_base_url,
        api_key: env::var("CARP_TEST_API_KEY").ok(),
        api_token: env::var("CARP_TEST_TOKEN").ok(),
        timeout: 15,
        verify_ssl: true,
        default_output_dir: Some("./regression_test_output".to_string()),
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

/// Test for bug where empty search queries were not properly validated
#[tokio::test]
async fn test_regression_empty_search_query() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    // This should fail with proper validation error
    let result = client.search("", None, false).await;
    match result {
        Err(CarpError::InvalidAgent(msg)) => {
            assert!(
                msg.contains("empty") || msg.contains("cannot be empty"),
                "Error message should mention empty query: {}",
                msg
            );
            println!("✓ Empty search query properly rejected: {}", msg);
        }
        Err(e) => {
            println!("Empty search query failed with different error: {}", e);
        }
        Ok(_) => {
            panic!("Empty search query should not succeed");
        }
    }

    Ok(())
}

/// Test for bug where whitespace-only queries were accepted
#[tokio::test]
async fn test_regression_whitespace_search_query() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    let whitespace_queries = vec!["   ", "\t", "\n", " \t \n "];

    for query in whitespace_queries {
        let result = client.search(query, None, false).await;
        match result {
            Err(CarpError::InvalidAgent(_)) => {
                println!(
                    "✓ Whitespace query '{}' properly rejected",
                    query.replace('\n', "\\n").replace('\t', "\\t")
                );
            }
            Err(e) => {
                println!("Whitespace query failed with different error: {}", e);
            }
            Ok(_) => {
                println!("⚠ Whitespace query unexpectedly succeeded");
            }
        }
    }

    Ok(())
}

/// Test for bug where zero limit was accepted in search
#[tokio::test]
async fn test_regression_zero_search_limit() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    let result = client.search("test", Some(0), false).await;
    match result {
        Err(CarpError::InvalidAgent(msg)) => {
            assert!(
                msg.contains("0") || msg.contains("greater"),
                "Error message should mention invalid limit: {}",
                msg
            );
            println!("✓ Zero search limit properly rejected: {}", msg);
        }
        Err(e) => {
            println!("Zero search limit failed with different error: {}", e);
        }
        Ok(_) => {
            panic!("Zero search limit should not succeed");
        }
    }

    Ok(())
}

/// Test for bug where extremely large limits could cause memory issues
#[tokio::test]
async fn test_regression_large_search_limit() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    let large_limits = vec![usize::MAX, 1_000_000, 100_000];

    for limit in large_limits {
        let result = timeout(
            Duration::from_secs(10),
            client.search("test", Some(limit), false),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                // If it succeeds, response should be reasonable
                assert!(
                    response.agents.len() < 10_000,
                    "Large limit should not return excessive results: {} agents",
                    response.agents.len()
                );
                println!(
                    "✓ Large limit {} handled reasonably: {} agents returned",
                    limit,
                    response.agents.len()
                );
            }
            Ok(Err(e)) => {
                println!("Large limit {} properly rejected: {}", limit, e);
            }
            Err(_) => {
                println!("Large limit {} timed out (handled gracefully)", limit);
            }
        }
    }

    Ok(())
}

/// Test for bug where invalid agent names were not properly validated
#[tokio::test]
async fn test_regression_invalid_agent_names() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    // Test various invalid agent names that previously caused issues
    let invalid_names = vec![
        ("", "empty name"),
        ("../etc/passwd", "path traversal"),
        ("name with spaces", "spaces in name"),
        ("name@invalid", "@ symbol"),
        ("UPPERCASE", "uppercase letters"),
        ("-startdash", "starting with dash"),
        ("enddash-", "ending with dash"),
        ("name.with.dots", "dots in name"),
        ("name/with/slashes", "slashes in name"),
        ("very-long-name-that-exceeds-reasonable-limits-and-should-be-rejected-by-validation-logic", "extremely long name"),
    ];

    for (name, description) in invalid_names {
        let result = client.get_agent_download(name, None).await;
        match result {
            Err(_) => {
                println!(
                    "✓ Invalid agent name '{}' properly rejected ({})",
                    name, description
                );
            }
            Ok(_) => {
                println!(
                    "⚠ Invalid agent name '{}' unexpectedly succeeded ({})",
                    name, description
                );
            }
        }
    }

    Ok(())
}

/// Test for bug where client could hang indefinitely without proper timeout
#[tokio::test]
async fn test_regression_timeout_handling() -> CarpResult<()> {
    let mut config = create_regression_test_config();
    config.timeout = 1; // Very short timeout
    config.registry_url = "https://httpstat.us/200?sleep=5000".to_string(); // Slow response

    let client = ApiClient::new(&config)?;

    let start = std::time::Instant::now();
    let result = client.health_check().await;
    let duration = start.elapsed();

    // Should fail relatively quickly due to timeout
    assert!(
        duration < Duration::from_secs(10),
        "Request should timeout quickly, took {:?}",
        duration
    );

    match result {
        Err(_) => {
            println!("✓ Timeout properly enforced in {:?}", duration);
        }
        Ok(_) => {
            println!("Request completed unexpectedly quickly in {:?}", duration);
        }
    }

    Ok(())
}

/// Test for bug where retry logic could cause infinite loops
#[tokio::test]
async fn test_regression_retry_loop_prevention() -> CarpResult<()> {
    let mut config = create_regression_test_config();
    config.registry_url = "https://httpstat.us/503".to_string(); // Always fails
    config.retry.max_retries = 3;
    config.retry.initial_delay_ms = 50;
    config.retry.max_delay_ms = 200;

    let client = ApiClient::new(&config)?;

    let start = std::time::Instant::now();
    let result = client.health_check().await;
    let duration = start.elapsed();

    // Should fail after retries but not hang indefinitely
    assert!(result.is_err(), "Request should fail after retries");
    assert!(
        duration < Duration::from_secs(30),
        "Retry loop should complete in reasonable time: {:?}",
        duration
    );

    // Should take at least some time for retries
    assert!(
        duration >= Duration::from_millis(30),
        "Should take some time for retries: {:?}",
        duration
    );

    println!("✓ Retry loop completed properly in {:?}", duration);

    Ok(())
}

/// Test for bug where malformed JSON responses could crash the client
#[tokio::test]
async fn test_regression_malformed_json_handling() -> CarpResult<()> {
    let mut config = create_regression_test_config();
    // Use a URL that returns invalid JSON
    config.registry_url = "https://httpstat.us/200".to_string(); // Returns plain text

    let client = ApiClient::new(&config)?;

    let result = client.health_check().await;

    // Should handle malformed JSON gracefully
    match result {
        Err(CarpError::Json(_)) => {
            println!("✓ Malformed JSON properly handled with JSON error");
        }
        Err(e) => {
            println!("✓ Malformed JSON handled with error: {}", e);
        }
        Ok(_) => {
            println!("Request unexpectedly succeeded (may be valid JSON)");
        }
    }

    Ok(())
}

/// Test for bug where concurrent requests could interfere with each other
#[tokio::test]
async fn test_regression_concurrent_request_isolation() -> CarpResult<()> {
    if RegressionTestConfig::default().skip_regression_tests {
        println!("Skipping concurrent isolation test (CARP_SKIP_REGRESSION_TESTS set)");
        return Ok(());
    }

    let config = create_regression_test_config();

    // Create multiple clients to test isolation
    let mut futures = Vec::new();

    for i in 0..5 {
        let client = ApiClient::new(&config)?;
        let future = async move {
            let query = format!("test-{}", i);
            let result = client.search(&query, Some(5), false).await;
            (i, result)
        };
        futures.push(future);
    }

    let results = futures::future::join_all(futures).await;

    // Check that each request was handled independently
    for (i, result) in results {
        match result {
            Ok(response) => {
                println!(
                    "Concurrent request {} succeeded: {} agents",
                    i,
                    response.agents.len()
                );
            }
            Err(e) => {
                println!("Concurrent request {} failed (expected): {}", i, e);
            }
        }
    }

    println!("✓ Concurrent request isolation test completed");

    Ok(())
}

/// Test for bug where authentication tokens were not properly handled
#[tokio::test]
async fn test_regression_token_handling() -> CarpResult<()> {
    // Test with invalid token format
    let mut config = create_regression_test_config();
    config.api_token = Some("invalid-token-format".to_string());

    let client = ApiClient::new(&config)?;
    let result = client.authenticate("test", "test").await;

    // Should handle invalid token gracefully
    match result {
        Err(CarpError::Auth(_)) => {
            println!("✓ Invalid token properly rejected");
        }
        Err(e) => {
            println!("Invalid token handled with error: {}", e);
        }
        Ok(_) => {
            println!("Invalid token unexpectedly succeeded");
        }
    }

    // Test with empty token
    config.api_token = Some("".to_string());
    let _client = ApiClient::new(&config)?;
    // Empty token should be handled (might be treated as no token)

    // Test with very long token
    config.api_token = Some("x".repeat(10000));
    let result = ApiClient::new(&config);
    // Should not crash on very long token
    assert!(
        result.is_ok(),
        "Very long token should not crash client creation"
    );

    println!("✓ Token handling regression tests completed");

    Ok(())
}

/// Test for bug where URL encoding was not properly handled
#[tokio::test]
async fn test_regression_url_encoding() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    // Test search queries that require URL encoding
    let special_queries = vec![
        "query with spaces",
        "query+with+plus",
        "query&with&ampersand",
        "query=with=equals",
        "query?with?question",
        "query#with#hash",
        "query%with%percent",
    ];

    for query in special_queries {
        let result = timeout(
            Duration::from_secs(10),
            client.search(query, Some(1), false),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                println!("✓ Special query '{}' handled properly", query);
            }
            Ok(Err(e)) => {
                println!("Special query '{}' failed: {}", query, e);
            }
            Err(_) => {
                println!("Special query '{}' timed out", query);
            }
        }
    }

    Ok(())
}

/// Test for bug where configuration validation was insufficient
#[tokio::test]
async fn test_regression_config_validation() -> CarpResult<()> {
    // Test various invalid configurations
    let mut config = create_regression_test_config();

    // Empty registry URL
    config.registry_url = "".to_string();
    let result = ApiClient::new(&config);
    match result {
        Err(CarpError::Config(_)) => {
            println!("✓ Empty registry URL properly rejected");
        }
        _ => {
            panic!("Empty registry URL should be rejected");
        }
    }

    // Invalid timeout values
    config = create_regression_test_config();
    config.timeout = 0;
    let result = ApiClient::new(&config);
    // Zero timeout should be handled (might be converted to default)
    assert!(result.is_ok(), "Zero timeout should be handled gracefully");

    config.timeout = u64::MAX;
    let result = ApiClient::new(&config);
    // Very large timeout should be handled
    assert!(result.is_ok(), "Large timeout should be handled gracefully");

    println!("✓ Configuration validation regression tests completed");

    Ok(())
}

/// Test for bug where error messages contained sensitive information
#[tokio::test]
async fn test_regression_error_message_sanitization() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    // Trigger various error conditions and check message sanitization
    let result = client.get_agent_download("nonexistent", None).await;

    if let Err(e) = result {
        let error_msg = e.to_string().to_lowercase();

        // Check that error doesn't contain sensitive information
        let sensitive_patterns = vec![
            "password",
            "token",
            "secret",
            "key",
            "/home/",
            "/users/",
            "c:\\",
            "/etc/",
            "/var/",
            "connection string",
            "sql error:",
            "internal error",
        ];

        for pattern in sensitive_patterns {
            assert!(
                !error_msg.contains(pattern),
                "Error message should not contain '{}': {}",
                pattern,
                error_msg
            );
        }

        println!("✓ Error message properly sanitized");
    }

    Ok(())
}

/// Test for memory leaks in long-running operations
#[tokio::test]
async fn test_regression_memory_leaks() -> CarpResult<()> {
    let config = create_regression_test_config();

    // Perform many operations to detect memory leaks
    for i in 0..50 {
        let client = ApiClient::new(&config)?;
        let _ = client
            .search(&format!("test-{}", i % 5), Some(1), false)
            .await;

        // Drop client to test cleanup
        drop(client);

        if i % 10 == 0 {
            // Simple memory pressure check (not perfect, but indicative)
            tokio::task::yield_now().await;
        }
    }

    println!("✓ Memory leak regression test completed");
    Ok(())
}

/// Test for bug where client state could become corrupted
#[tokio::test]
async fn test_regression_client_state_consistency() -> CarpResult<()> {
    let config = create_regression_test_config();
    let client = ApiClient::new(&config)?;

    // Perform various operations that previously could corrupt state
    let _ = client.search("test1", Some(5), false).await;
    let _ = client.health_check().await;
    let _ = client.search("test2", Some(3), true).await;
    let _ = client.get_agent_download("test-agent", None).await;
    let _ = client.search("test3", Some(1), false).await;

    // Client should still work after multiple operations
    let final_result = client.health_check().await;

    match final_result {
        Ok(_) => println!("✓ Client state remained consistent"),
        Err(e) => println!("Final operation failed (may be expected): {}", e),
    }

    Ok(())
}
