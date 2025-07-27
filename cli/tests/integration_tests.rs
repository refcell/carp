/// Integration tests for the CLI tool
/// These tests verify CLI functionality against mock API servers

use mockito::{Mock, ServerGuard};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::process::Command;

// Test utilities module
mod test_utils {
    use super::*;
    use carp_cli::{
        api::{client::ApiClient, types::*},
        config::Config,
    };

    pub struct TestContext {
        pub temp_dir: TempDir,
        pub config: Config,
        pub mock_server: ServerGuard,
        pub mock_base_url: String,
    }

    impl TestContext {
        pub async fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let mut server = mockito::Server::new_async().await;
            let mock_base_url = server.url();

            let config = Config {
                registry_url: mock_base_url.clone(),
                api_token: Some("test-token".to_string()),
                timeout: 30,
                verify_ssl: false,
                default_output_dir: Some(temp_dir.path().to_path_buf()),
            };

            Self {
                temp_dir,
                config,
                mock_server: server,
                mock_base_url,
            }
        }

        pub fn create_api_client(&self) -> ApiClient {
            ApiClient::new(&self.config).expect("Failed to create API client")
        }

        pub fn get_temp_path(&self, filename: &str) -> PathBuf {
            self.temp_dir.path().join(filename)
        }
    }
}

// Test CLI API client search functionality
#[tokio::test]
async fn test_api_client_search() {
    let mut ctx = test_utils::TestContext::new().await;

    // Mock the search endpoint
    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("q".to_string(), "test".to_string()),
            mockito::Matcher::UrlEncoded("limit".to_string(), "10".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [
                    {
                        "name": "test-agent",
                        "version": "1.0.0",
                        "description": "A test agent",
                        "author": "testuser",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z",
                        "download_count": 42,
                        "tags": ["test", "ai"],
                        "readme": "# Test Agent",
                        "homepage": "https://example.com",
                        "repository": "https://github.com/test/agent",
                        "license": "MIT"
                    }
                ],
                "total": 1,
                "page": 1,
                "per_page": 10
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.search("test", Some(10), false).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.agents.len(), 1);
    assert_eq!(response.agents[0].name, "test-agent");
    assert_eq!(response.agents[0].version, "1.0.0");
    assert_eq!(response.total, 1);
}

// Test CLI API client search with no results
#[tokio::test]
async fn test_api_client_search_no_results() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .match_query(mockito::Matcher::UrlEncoded(
            "q".to_string(),
            "nonexistent".to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [],
                "total": 0,
                "page": 1,
                "per_page": 10
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.search("nonexistent", None, false).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.agents.len(), 0);
    assert_eq!(response.total, 0);
}

// Test CLI API client authentication
#[tokio::test]
async fn test_api_client_authentication() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("POST", "/api/v1/auth/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test",
                "expires_at": "2024-12-31T23:59:59Z"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.authenticate("testuser", "password").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.token, "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test");
}

// Test CLI API client authentication failure
#[tokio::test]
async fn test_api_client_authentication_failure() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("POST", "/api/v1/auth/login")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": "AuthenticationError",
                "message": "Invalid credentials"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.authenticate("baduser", "badpassword").await;

    assert!(result.is_err());
    match result {
        Err(carp_cli::utils::error::CarpError::Api { status, message }) => {
            assert_eq!(status, 401);
            assert!(message.contains("Invalid credentials"));
        }
        _ => panic!("Expected API error"),
    }
}

// Test CLI API client download functionality
#[tokio::test]
async fn test_api_client_get_download_info() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/test-agent/latest/download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "name": "test-agent",
                "version": "1.0.0",
                "download_url": "https://storage.example.com/test-agent-1.0.0.zip",
                "checksum": "sha256:abcdef123456",
                "size": 1024
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.get_agent_download("test-agent", None).await;

    assert!(result.is_ok());
    let download = result.unwrap();
    assert_eq!(download.name, "test-agent");
    assert_eq!(download.version, "1.0.0");
    assert_eq!(download.size, 1024);
}

// Test CLI API client download with specific version
#[tokio::test]
async fn test_api_client_get_download_specific_version() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/test-agent/2.0.0/download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "name": "test-agent",
                "version": "2.0.0",
                "download_url": "https://storage.example.com/test-agent-2.0.0.zip",
                "checksum": "sha256:fedcba654321",
                "size": 2048
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.get_agent_download("test-agent", Some("2.0.0")).await;

    assert!(result.is_ok());
    let download = result.unwrap();
    assert_eq!(download.name, "test-agent");
    assert_eq!(download.version, "2.0.0");
    assert_eq!(download.size, 2048);
}

// Test CLI API client agent file download
#[tokio::test]
async fn test_api_client_download_agent_file() {
    let mut ctx = test_utils::TestContext::new().await;

    let test_content = b"test zip file content";
    let _mock = ctx
        .mock_server
        .mock("GET", "/download/test-agent.zip")
        .with_status(200)
        .with_header("content-type", "application/zip")
        .with_body(test_content)
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let download_url = format!("{}/download/test-agent.zip", ctx.mock_base_url);
    let result = client.download_agent(&download_url).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes.as_ref(), test_content);
}

// Test CLI API client publish functionality
#[tokio::test]
async fn test_api_client_publish() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("POST", "/api/v1/agents/publish")
        .match_header("authorization", "Bearer test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "success": true,
                "message": "Agent published successfully",
                "agent": {
                    "name": "new-agent",
                    "version": "1.0.0",
                    "description": "A new agent",
                    "author": "testuser",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "download_count": 0,
                    "tags": ["new", "test"],
                    "readme": null,
                    "homepage": null,
                    "repository": null,
                    "license": null
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let publish_request = carp_cli::api::types::PublishRequest {
        name: "new-agent".to_string(),
        version: "1.0.0".to_string(),
        description: "A new agent".to_string(),
        readme: None,
        homepage: None,
        repository: None,
        license: None,
        tags: vec!["new".to_string(), "test".to_string()],
    };

    let test_content = b"fake zip content".to_vec();
    let result = client.publish(publish_request, test_content).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.message, "Agent published successfully");
    assert!(response.agent.is_some());
}

// Test CLI API client publish without authentication
#[tokio::test]
async fn test_api_client_publish_no_auth() {
    let mut ctx = test_utils::TestContext::new().await;
    // Remove the token to simulate no authentication
    ctx.config.api_token = None;

    let client = ctx.create_api_client();
    let publish_request = carp_cli::api::types::PublishRequest {
        name: "new-agent".to_string(),
        version: "1.0.0".to_string(),
        description: "A new agent".to_string(),
        readme: None,
        homepage: None,
        repository: None,
        license: None,
        tags: vec![],
    };

    let test_content = b"fake zip content".to_vec();
    let result = client.publish(publish_request, test_content).await;

    assert!(result.is_err());
    match result {
        Err(carp_cli::utils::error::CarpError::Auth(message)) => {
            assert!(message.contains("No API token"));
        }
        _ => panic!("Expected auth error"),
    }
}

// Test error handling for API errors
#[tokio::test]
async fn test_api_client_error_handling() {
    let mut ctx = test_utils::TestContext::new().await;

    // Mock server error
    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": "InternalError",
                "message": "Database connection failed"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.search("test", None, false).await;

    assert!(result.is_err());
    match result {
        Err(carp_cli::utils::error::CarpError::Api { status, message }) => {
            assert_eq!(status, 500);
            assert!(message.contains("Database connection failed"));
        }
        _ => panic!("Expected API error"),
    }
}

// Test network timeout handling
#[tokio::test]
async fn test_api_client_timeout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create a config with very short timeout
    let config = carp_cli::config::Config {
        registry_url: "http://127.0.0.1:9999".to_string(), // Non-existent server
        api_token: None,
        timeout: 1, // 1 second timeout
        verify_ssl: false,
        default_output_dir: Some(temp_dir.path().to_path_buf()),
    };

    let client = carp_cli::api::client::ApiClient::new(&config).expect("Failed to create client");
    let result = client.search("test", None, false).await;

    assert!(result.is_err());
    // Should be a network/timeout error
    match result {
        Err(carp_cli::utils::error::CarpError::Network(_)) => {
            // Expected
        }
        Err(carp_cli::utils::error::CarpError::Request(_)) => {
            // Also acceptable for timeout
        }
        _ => panic!("Expected network or request error, got: {:?}", result),
    }
}

// Test configuration loading and validation
#[tokio::test]
async fn test_config_validation() {
    use carp_cli::config::Config;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test valid config
    let valid_config = Config {
        registry_url: "https://api.example.com".to_string(),
        api_token: Some("valid-token".to_string()),
        timeout: 30,
        verify_ssl: true,
        default_output_dir: Some(temp_dir.path().to_path_buf()),
    };

    let client_result = carp_cli::api::client::ApiClient::new(&valid_config);
    assert!(client_result.is_ok());

    // Test config with invalid timeout
    let invalid_timeout_config = Config {
        registry_url: "https://api.example.com".to_string(),
        api_token: Some("valid-token".to_string()),
        timeout: 0, // Invalid timeout
        verify_ssl: true,
        default_output_dir: Some(temp_dir.path().to_path_buf()),
    };

    // This should still create a client but use a default timeout
    let client_result = carp_cli::api::client::ApiClient::new(&invalid_timeout_config);
    assert!(client_result.is_ok());
}

// Test JSON parsing errors
#[tokio::test]
async fn test_json_parsing_errors() {
    let mut ctx = test_utils::TestContext::new().await;

    // Mock endpoint returning invalid JSON
    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("invalid json response")
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.search("test", None, false).await;

    assert!(result.is_err());
    match result {
        Err(carp_cli::utils::error::CarpError::Json(_)) => {
            // Expected JSON parsing error
        }
        _ => panic!("Expected JSON parsing error, got: {:?}", result),
    }
}

// Test user-agent header
#[tokio::test]
async fn test_user_agent_header() {
    let mut ctx = test_utils::TestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .match_header("user-agent", mockito::Matcher::Regex(r"carp-cli/\d+\.\d+\.\d+".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [],
                "total": 0,
                "page": 1,
                "per_page": 10
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = ctx.create_api_client();
    let result = client.search("test", None, false).await;

    assert!(result.is_ok());
}

// Test SSL verification settings
#[tokio::test]
async fn test_ssl_verification() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test with SSL verification disabled
    let config = carp_cli::config::Config {
        registry_url: "https://self-signed.badssl.com".to_string(),
        api_token: None,
        timeout: 30,
        verify_ssl: false, // Disabled SSL verification
        default_output_dir: Some(temp_dir.path().to_path_buf()),
    };

    let client_result = carp_cli::api::client::ApiClient::new(&config);
    assert!(client_result.is_ok());

    // Test with SSL verification enabled (should work for valid certs)
    let secure_config = carp_cli::config::Config {
        registry_url: "https://httpbin.org".to_string(),
        api_token: None,
        timeout: 30,
        verify_ssl: true, // Enabled SSL verification
        default_output_dir: Some(temp_dir.path().to_path_buf()),
    };

    let secure_client_result = carp_cli::api::client::ApiClient::new(&secure_config);
    assert!(secure_client_result.is_ok());
}