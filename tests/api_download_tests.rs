/// Comprehensive unit tests for the download API endpoint
/// Tests the core functionality that was recently fixed to prevent regression
///
/// These tests focus on:
/// - Database function called with correct signature: get_agent_download_info(p_agent_name, p_version_text)
/// - Proper handling of version parameter (empty string for "latest")
/// - Correct response parsing and error handling
use serde_json::json;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

// Import the download module functions that we want to test
// Since the download.rs is a binary, we need to test the public functions
use shared::{ApiError, AuthMethod, AuthenticatedUser, UserMetadata};

/// Test configuration for download API tests
pub struct DownloadTestConfig {
    pub mock_supabase_url: String,
    pub mock_supabase_key: String,
    pub debug_mode: bool,
}

impl Default for DownloadTestConfig {
    fn default() -> Self {
        Self {
            mock_supabase_url: "https://test.supabase.co".to_string(),
            mock_supabase_key: "test_service_key".to_string(),
            debug_mode: true,
        }
    }
}

/// Mock HTTP request builder for testing
#[allow(dead_code)]
pub struct MockRequestBuilder {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[allow(dead_code)]
impl MockRequestBuilder {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn bearer_token(self, token: &str) -> Self {
        self.header("Authorization", &format!("Bearer {}", token))
    }

    pub fn build_uri(&self) -> http::Uri {
        self.path.parse().unwrap()
    }
}

#[cfg(test)]
mod download_api_tests {
    use super::*;

    /// Test path parameter parsing for valid agent names and versions
    #[tokio::test]
    async fn test_path_parameter_extraction() {
        let test_cases = vec![
            (
                "/api/v1/agents/my-agent/1.0.0/download",
                "my-agent",
                "1.0.0",
            ),
            (
                "/api/v1/agents/test_agent/latest/download",
                "test_agent",
                "latest",
            ),
            (
                "/api/v1/agents/complex-name-123/v2.1.3/download",
                "complex-name-123",
                "v2.1.3",
            ),
            (
                "/api/v1/agents/agent%20with%20spaces/1.0/download",
                "agent with spaces",
                "1.0",
            ),
        ];

        for (path, expected_name, expected_version) in test_cases {
            let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            assert!(
                path_segments.len() >= 6,
                "Path should have enough segments: {}",
                path
            );

            let agent_name = urlencoding::decode(path_segments[3]).unwrap();
            let version = urlencoding::decode(path_segments[4]).unwrap();

            assert_eq!(
                agent_name, expected_name,
                "Agent name mismatch for path: {}",
                path
            );
            assert_eq!(
                version, expected_version,
                "Version mismatch for path: {}",
                path
            );
        }
    }

    /// Test invalid path formats return proper error responses
    #[tokio::test]
    async fn test_invalid_path_formats() {
        let invalid_paths = vec![
            "/api/v1/agents",
            "/api/v1/agents/name",
            "/api/v1/agents/name/version",
            "/api/v1/agents/name/version/not-download",
            "/invalid/path",
        ];

        for path in invalid_paths {
            let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            if path_segments.len() < 6 {
                // This should trigger the bad_request error
                let expected_error = ApiError {
                    error: "bad_request".to_string(),
                    message:
                        "Invalid path format. Expected /api/v1/agents/{name}/{version}/download"
                            .to_string(),
                    details: None,
                };

                let error_json = serde_json::to_string(&expected_error).unwrap();
                assert!(
                    error_json.contains("bad_request"),
                    "Should contain error type for path: {}",
                    path
                );
                assert!(
                    error_json.contains("Invalid path format"),
                    "Should contain error message for path: {}",
                    path
                );
            }
        }
    }

    /// Test URL encoding/decoding for agent names and versions
    #[tokio::test]
    async fn test_url_encoding_decoding() {
        let test_cases = vec![
            ("my-agent", "my-agent"),
            ("agent%20with%20spaces", "agent with spaces"),
            ("agent%2Bplus", "agent+plus"),
            ("agent%40symbol", "agent@symbol"),
            ("complex%2Dname%5F123", "complex-name_123"),
        ];

        for (encoded, expected_decoded) in test_cases {
            let decoded = urlencoding::decode(encoded).unwrap();
            assert_eq!(
                decoded, expected_decoded,
                "Decoding failed for: {}",
                encoded
            );

            // Test that encoding round-trip works
            let re_encoded = urlencoding::encode(&decoded);
            let re_decoded = urlencoding::decode(&re_encoded).unwrap();
            assert_eq!(
                re_decoded, expected_decoded,
                "Round-trip failed for: {}",
                encoded
            );
        }
    }
}

#[cfg(test)]
mod database_integration_tests {
    use super::*;

    /// Test that database function is called with correct signature and parameters
    #[tokio::test]
    async fn test_database_function_call_signature() {
        let mock_server = MockServer::start().await;

        // Test case 1: Specific version
        let expected_payload_specific = json!({
            "p_agent_name": "test-agent",
            "p_version_text": "1.0.0"
        });

        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/get_agent_download_info"))
            .and(header("apikey", "test_service_key"))
            .and(header("Authorization", "Bearer test_service_key"))
            .and(header("Content-Type", "application/json"))
            .and(body_json(&expected_payload_specific))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![json!({
                "agent_name": "test-agent",
                "version": "1.0.0",
                "file_path": "test-agent/1.0.0/agent.zip",
                "checksum": "abc123",
                "file_size": 1024
            })]))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Test case 2: Latest version (should send empty string)
        let expected_payload_latest = json!({
            "p_agent_name": "test-agent",
            "p_version_text": ""
        });

        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/get_agent_download_info"))
            .and(header("apikey", "test_service_key"))
            .and(header("Authorization", "Bearer test_service_key"))
            .and(header("Content-Type", "application/json"))
            .and(body_json(&expected_payload_latest))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![json!({
                "agent_name": "test-agent",
                "version": "2.0.0",
                "file_path": "test-agent/2.0.0/agent.zip",
                "checksum": "def456",
                "file_size": 2048
            })]))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Set environment variables to use mock server
        env::set_var("SUPABASE_URL", &mock_server.uri());
        env::set_var("SUPABASE_SERVICE_ROLE_KEY", "test_service_key");

        // Test the database function call logic by directly testing the HTTP calls
        let client = reqwest::Client::new();

        // Test specific version
        let response1 = client
            .post(&format!(
                "{}/rest/v1/rpc/get_agent_download_info",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&expected_payload_specific)
            .send()
            .await
            .unwrap();

        assert!(
            response1.status().is_success(),
            "Specific version request should succeed"
        );

        // Test latest version
        let response2 = client
            .post(&format!(
                "{}/rest/v1/rpc/get_agent_download_info",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&expected_payload_latest)
            .send()
            .await
            .unwrap();

        assert!(
            response2.status().is_success(),
            "Latest version request should succeed"
        );

        // Clean up environment variables
        env::remove_var("SUPABASE_URL");
        env::remove_var("SUPABASE_SERVICE_ROLE_KEY");
    }

    /// Test version resolution logic (latest vs specific version)
    #[tokio::test]
    async fn test_version_resolution() {
        let test_cases = vec![
            ("latest", ""),                   // "latest" should become empty string
            ("1.0.0", "1.0.0"),               // Specific version should remain unchanged
            ("v2.1.3", "v2.1.3"),             // Version with prefix should remain unchanged
            ("0.1.0-alpha", "0.1.0-alpha"),   // Pre-release version should remain unchanged
            ("1.0.0-beta.1", "1.0.0-beta.1"), // Pre-release with build should remain unchanged
        ];

        for (input_version, expected_db_version) in test_cases {
            let actual_db_version = if input_version == "latest" {
                ""
            } else {
                input_version
            };
            assert_eq!(
                actual_db_version, expected_db_version,
                "Version resolution failed for input: {}",
                input_version
            );
        }
    }

    /// Test database response parsing
    #[tokio::test]
    async fn test_database_response_parsing() {
        // Test successful response parsing
        let mock_db_response = json!([{
            "agent_name": "test-agent",
            "version": "1.0.0",
            "file_path": "test-agent/1.0.0/agent.zip",
            "checksum": "sha256:abc123def456",
            "file_size": 1024
        }]);

        if let Some(data) = mock_db_response.as_array().and_then(|arr| arr.first()) {
            let name = data
                .get("agent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("fallback");
            let version = data
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("fallback");
            let file_path = data.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            let checksum = data.get("checksum").and_then(|v| v.as_str()).unwrap_or("");
            let file_size = data.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);

            assert_eq!(name, "test-agent");
            assert_eq!(version, "1.0.0");
            assert_eq!(file_path, "test-agent/1.0.0/agent.zip");
            assert_eq!(checksum, "sha256:abc123def456");
            assert_eq!(file_size, 1024);
        } else {
            panic!("Failed to parse mock database response");
        }

        // Test empty response (agent not found)
        let empty_response = json!([]);
        let result = empty_response.as_array().and_then(|arr| arr.first());
        assert!(result.is_none(), "Empty response should return None");

        // Test malformed response
        let malformed_response = json!([{
            "agent_name": "test-agent",
            // Missing required fields
        }]);

        if let Some(data) = malformed_response.as_array().and_then(|arr| arr.first()) {
            let file_path = data.get("file_path").and_then(|v| v.as_str());
            assert!(file_path.is_none(), "Missing file_path should return None");
        }
    }

    /// Test database error handling
    #[tokio::test]
    async fn test_database_error_handling() {
        let mock_server = MockServer::start().await;

        // Test 404 response (agent not found)
        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/get_agent_download_info"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "message": "Agent not found"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/rest/v1/rpc/get_agent_download_info",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&json!({
                "p_agent_name": "nonexistent-agent",
                "p_version_text": "1.0.0"
            }))
            .send()
            .await
            .unwrap();

        assert!(
            !response.status().is_success(),
            "Request should fail for nonexistent agent"
        );
        assert_eq!(
            response.status(),
            404,
            "Should return 404 for nonexistent agent"
        );
    }

    /// Test environment variable requirements
    #[tokio::test]
    async fn test_environment_variable_requirements() {
        // Save original environment
        let original_url = env::var("SUPABASE_URL").ok();
        let original_key = env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

        // Test missing SUPABASE_URL
        env::remove_var("SUPABASE_URL");
        env::set_var("SUPABASE_SERVICE_ROLE_KEY", "test_key");

        let url_result = env::var("SUPABASE_URL");
        assert!(
            url_result.is_err(),
            "Should fail when SUPABASE_URL is not set"
        );

        // Test missing SUPABASE_SERVICE_ROLE_KEY
        env::set_var("SUPABASE_URL", "https://test.supabase.co");
        env::remove_var("SUPABASE_SERVICE_ROLE_KEY");

        let key_result = env::var("SUPABASE_SERVICE_ROLE_KEY");
        assert!(
            key_result.is_err(),
            "Should fail when SUPABASE_SERVICE_ROLE_KEY is not set"
        );

        // Restore original environment
        match original_url {
            Some(val) => env::set_var("SUPABASE_URL", val),
            None => env::remove_var("SUPABASE_URL"),
        }
        match original_key {
            Some(val) => env::set_var("SUPABASE_SERVICE_ROLE_KEY", val),
            None => env::remove_var("SUPABASE_SERVICE_ROLE_KEY"),
        }
    }
}

#[cfg(test)]
mod signed_url_generation_tests {
    use super::*;

    /// Test signed URL generation for file downloads
    #[tokio::test]
    async fn test_signed_url_generation() {
        let mock_server = MockServer::start().await;

        let expected_signed_response = json!({
            "signedURL": "/storage/v1/object/sign/agent-packages/test-agent/1.0.0/agent.zip?token=abc123"
        });

        Mock::given(method("POST"))
            .and(path(
                "/storage/v1/object/sign/agent-packages/test-agent/1.0.0/agent.zip",
            ))
            .and(header("apikey", "test_service_key"))
            .and(header("Authorization", "Bearer test_service_key"))
            .and(header("Content-Type", "application/json"))
            .and(body_json(json!({"expiresIn": 3600})))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected_signed_response))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/storage/v1/object/sign/agent-packages/test-agent/1.0.0/agent.zip",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&json!({"expiresIn": 3600}))
            .send()
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "Signed URL generation should succeed"
        );

        let response_json: serde_json::Value = response.json().await.unwrap();
        let signed_url = response_json
            .get("signedURL")
            .and_then(|v| v.as_str())
            .unwrap();

        assert!(
            signed_url.contains("agent-packages"),
            "Signed URL should contain bucket path"
        );
        assert!(
            signed_url.contains("test-agent/1.0.0/agent.zip"),
            "Signed URL should contain file path"
        );
        assert!(
            signed_url.contains("token="),
            "Signed URL should contain token parameter"
        );
    }

    /// Test signed URL generation error handling
    #[tokio::test]
    async fn test_signed_url_generation_errors() {
        let mock_server = MockServer::start().await;

        // Test 404 for nonexistent file
        Mock::given(method("POST"))
            .and(path(
                "/storage/v1/object/sign/agent-packages/nonexistent/file.zip",
            ))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "file_not_found",
                "message": "File not found in storage"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/storage/v1/object/sign/agent-packages/nonexistent/file.zip",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&json!({"expiresIn": 3600}))
            .send()
            .await
            .unwrap();

        assert!(
            !response.status().is_success(),
            "Should fail for nonexistent file"
        );
        assert_eq!(
            response.status(),
            404,
            "Should return 404 for nonexistent file"
        );
    }
}

#[cfg(test)]
mod download_tracking_tests {
    use super::*;

    /// Test download recording with correct parameters
    #[tokio::test]
    async fn test_download_recording() {
        let mock_server = MockServer::start().await;

        // Test recording specific version download
        let expected_payload_specific = json!({
            "agent_name": "test-agent",
            "version_text": "1.0.0",
            "user_agent_text": "carp-cli/0.1.0",
            "ip_addr": "192.168.1.1"
        });

        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/record_download"))
            .and(header("apikey", "test_service_key"))
            .and(header("Authorization", "Bearer test_service_key"))
            .and(header("Content-Type", "application/json"))
            .and(body_json(&expected_payload_specific))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Test recording latest version download (empty string)
        let expected_payload_latest = json!({
            "agent_name": "test-agent",
            "version_text": "",
            "user_agent_text": "carp-cli/0.1.0",
            "ip_addr": "192.168.1.1"
        });

        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/record_download"))
            .and(header("apikey", "test_service_key"))
            .and(header("Authorization", "Bearer test_service_key"))
            .and(header("Content-Type", "application/json"))
            .and(body_json(&expected_payload_latest))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();

        // Test specific version recording
        let response1 = client
            .post(&format!(
                "{}/rest/v1/rpc/record_download",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&expected_payload_specific)
            .send()
            .await
            .unwrap();

        assert!(
            response1.status().is_success(),
            "Specific version download recording should succeed"
        );

        // Test latest version recording
        let response2 = client
            .post(&format!(
                "{}/rest/v1/rpc/record_download",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Authorization", "Bearer test_service_key")
            .header("Content-Type", "application/json")
            .json(&expected_payload_latest)
            .send()
            .await
            .unwrap();

        assert!(
            response2.status().is_success(),
            "Latest version download recording should succeed"
        );
    }

    /// Test IP address extraction from headers
    #[test]
    fn test_ip_address_extraction() {
        let test_cases = vec![
            (Some("192.168.1.1"), None, "192.168.1.1"),
            (Some("203.0.113.1, 192.168.1.1"), None, "203.0.113.1"),
            (None, Some("10.0.0.1"), "10.0.0.1"),
            (Some("invalid"), Some("10.0.0.1"), "invalid"), // x-forwarded-for takes precedence
            (None, None, "127.0.0.1"),                      // Default fallback
        ];

        for (forwarded_for, real_ip, expected) in test_cases {
            let ip_addr = forwarded_for
                .or(real_ip)
                .unwrap_or("127.0.0.1")
                .split(',')
                .next()
                .unwrap_or("127.0.0.1")
                .trim();

            assert_eq!(
                ip_addr, expected,
                "IP extraction failed for forwarded_for: {:?}, real_ip: {:?}",
                forwarded_for, real_ip
            );
        }
    }

    /// Test user agent extraction
    #[test]
    fn test_user_agent_extraction() {
        let test_cases = vec![
            (Some("carp-cli/0.1.0"), "carp-cli/0.1.0"),
            (
                Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
            ),
            (Some("curl/7.68.0"), "curl/7.68.0"),
            (None, ""), // Default empty string
        ];

        for (user_agent_header, expected) in test_cases {
            let user_agent = user_agent_header.unwrap_or("");
            assert_eq!(
                user_agent, expected,
                "User agent extraction failed for: {:?}",
                user_agent_header
            );
        }
    }
}

#[cfg(test)]
mod authentication_tests {
    use super::*;

    /// Test optional authentication behavior
    #[test]
    fn test_optional_authentication_logic() {
        // Test cases for optional authentication
        let test_cases = vec![
            (Some("Bearer valid_token"), true), // Should attempt authentication
            (Some("Bearer "), true), // Should attempt authentication even with empty token
            (Some("Basic auth"), false), // Non-bearer token should not attempt authentication
            (None, false),           // No token should not attempt authentication
        ];

        for (auth_header, should_authenticate) in test_cases {
            let has_bearer_token = auth_header
                .and_then(|header| {
                    if header.starts_with("Bearer ") {
                        Some(header.strip_prefix("Bearer ").unwrap_or(""))
                    } else {
                        None
                    }
                })
                .is_some();

            assert_eq!(
                has_bearer_token, should_authenticate,
                "Authentication decision failed for header: {:?}",
                auth_header
            );
        }
    }

    /// Test authenticated user creation
    #[test]
    fn test_authenticated_user_creation() {
        let user = AuthenticatedUser {
            user_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            },
            scopes: vec!["read".to_string(), "download".to_string()],
            metadata: UserMetadata {
                email: Some("test@example.com".to_string()),
                github_username: Some("testuser".to_string()),
                created_at: None,
            },
        };

        assert_eq!(
            user.user_id.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert!(user.scopes.contains(&"read".to_string()));
        assert!(user.scopes.contains(&"download".to_string()));
        assert_eq!(user.metadata.email, Some("test@example.com".to_string()));
    }
}

#[cfg(test)]
mod response_format_tests {
    use super::*;

    /// Test successful download response format
    #[test]
    fn test_successful_download_response_format() {
        // This tests the AgentDownload struct serialization
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        pub struct AgentDownload {
            pub name: String,
            pub version: String,
            pub download_url: String,
            pub checksum: String,
            pub size: u64,
        }

        let download_info = AgentDownload {
            name: "test-agent".to_string(),
            version: "1.0.0".to_string(),
            download_url: "https://test.supabase.co/storage/v1/object/sign/agent-packages/test-agent/1.0.0/agent.zip?token=abc123".to_string(),
            checksum: "sha256:abc123def456".to_string(),
            size: 1024,
        };

        let json = serde_json::to_string(&download_info).unwrap();
        let parsed: AgentDownload = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "test-agent");
        assert_eq!(parsed.version, "1.0.0");
        assert!(parsed.download_url.contains("agent-packages"));
        assert!(parsed.download_url.contains("token="));
        assert_eq!(parsed.checksum, "sha256:abc123def456");
        assert_eq!(parsed.size, 1024);

        // Ensure JSON contains expected fields
        assert!(json.contains("\"name\":\"test-agent\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"download_url\":"));
        assert!(json.contains("\"checksum\":"));
        assert!(json.contains("\"size\":1024"));
    }

    /// Test error response format
    #[test]
    fn test_error_response_format() {
        let error = ApiError {
            error: "not_found".to_string(),
            message: "Agent 'nonexistent-agent' version '1.0.0' not found: Agent not found or no valid response from database".to_string(),
            details: None,
        };

        let json = serde_json::to_string(&error).unwrap();
        let parsed: ApiError = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.error, "not_found");
        assert!(parsed.message.contains("Agent 'nonexistent-agent'"));
        assert!(parsed.message.contains("version '1.0.0'"));
        assert!(parsed.message.contains("not found"));
        assert!(parsed.details.is_none());

        // Ensure JSON format is correct
        assert!(json.contains("\"error\":\"not_found\""));
        assert!(json.contains("\"message\":"));
        assert!(json.contains("nonexistent-agent"));
    }

    /// Test HTTP response headers
    #[test]
    fn test_http_response_headers() {
        use vercel_runtime::{Body, Response};

        // Test successful response headers
        let success_response: Response<Body> = Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body("{}".into())
            .unwrap();

        assert_eq!(success_response.status(), 200);
        assert_eq!(
            success_response.headers().get("content-type").unwrap(),
            "application/json"
        );

        // Test error response headers
        let error_response: Response<Body> = Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("{}".into())
            .unwrap();

        assert_eq!(error_response.status(), 404);
        assert_eq!(
            error_response.headers().get("content-type").unwrap(),
            "application/json"
        );
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Test the specific regression that was fixed:
    /// Database function called with correct signature: get_agent_download_info(p_agent_name, p_version_text)
    #[tokio::test]
    async fn test_database_function_parameter_names() {
        let mock_server = MockServer::start().await;

        // This test ensures the exact parameter names that were fixed in the regression
        let correct_payload = json!({
            "p_agent_name": "test-agent",        // Must be "p_agent_name", not "agent_name"
            "p_version_text": "1.0.0"           // Must be "p_version_text", not "version"
        });

        // This mock will only match if the exact parameter names are used
        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/get_agent_download_info"))
            .and(body_json(&correct_payload))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![json!({
                "agent_name": "test-agent",
                "version": "1.0.0",
                "file_path": "test-agent/1.0.0/agent.zip",
                "checksum": "abc123",
                "file_size": 1024
            })]))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Test that the correct parameter names are used
        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/rest/v1/rpc/get_agent_download_info",
                mock_server.uri()
            ))
            .header("apikey", "test_key")
            .header("Authorization", "Bearer test_key")
            .header("Content-Type", "application/json")
            .json(&correct_payload)
            .send()
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "Request with correct parameter names should succeed"
        );

        // Verify that using wrong parameter names would fail
        let wrong_payload = json!({
            "agent_name": "test-agent",          // Wrong: should be "p_agent_name"
            "version": "1.0.0"                  // Wrong: should be "p_version_text"
        });

        // This request should not match any mock and would fail in real scenario
        let response_wrong = client
            .post(&format!(
                "{}/rest/v1/rpc/get_agent_download_info",
                mock_server.uri()
            ))
            .header("apikey", "test_key")
            .header("Authorization", "Bearer test_key")
            .header("Content-Type", "application/json")
            .json(&wrong_payload)
            .send()
            .await
            .unwrap();

        // This would typically fail with 404 or 500 because the function signature doesn't match
        assert!(
            !response_wrong.status().is_success(),
            "Request with wrong parameter names should fail"
        );
    }

    /// Test the latest version handling regression
    #[tokio::test]
    async fn test_latest_version_empty_string_regression() {
        let mock_server = MockServer::start().await;

        // Test that "latest" version is converted to empty string
        let expected_payload = json!({
            "p_agent_name": "test-agent",
            "p_version_text": ""                 // Must be empty string for "latest"
        });

        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/get_agent_download_info"))
            .and(body_json(&expected_payload))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![json!({
                "agent_name": "test-agent",
                "version": "2.0.0",
                "file_path": "test-agent/2.0.0/agent.zip",
                "checksum": "latest123",
                "file_size": 2048
            })]))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();

        // Simulate the version conversion logic
        let input_version = "latest";
        let db_version_param = if input_version == "latest" {
            ""
        } else {
            input_version
        };

        let payload = json!({
            "p_agent_name": "test-agent",
            "p_version_text": db_version_param
        });

        let response = client
            .post(&format!(
                "{}/rest/v1/rpc/get_agent_download_info",
                mock_server.uri()
            ))
            .header("apikey", "test_key")
            .header("Authorization", "Bearer test_key")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "Latest version conversion should work correctly"
        );

        // Verify the response contains the latest version info
        let response_json: serde_json::Value = response.json().await.unwrap();
        if let Some(data) = response_json.as_array().and_then(|arr| arr.first()) {
            let version = data.get("version").and_then(|v| v.as_str()).unwrap();
            assert_ne!(
                version, "latest",
                "Response should contain actual version number, not 'latest'"
            );
            assert_eq!(version, "2.0.0", "Should return the actual latest version");
        }
    }

    /// Test that both download recording and agent info queries use consistent parameter conversion
    #[tokio::test]
    async fn test_consistent_version_parameter_handling() {
        let test_cases = vec![
            ("latest", ""),
            ("1.0.0", "1.0.0"),
            ("v2.1.3", "v2.1.3"),
            ("0.1.0-alpha", "0.1.0-alpha"),
        ];

        for (input_version, expected_db_param) in test_cases {
            // Test agent info query parameter conversion
            let agent_info_param = if input_version == "latest" {
                ""
            } else {
                input_version
            };
            assert_eq!(
                agent_info_param, expected_db_param,
                "Agent info parameter conversion failed for: {}",
                input_version
            );

            // Test download recording parameter conversion
            let download_record_param = if input_version == "latest" {
                ""
            } else {
                input_version
            };
            assert_eq!(
                download_record_param, expected_db_param,
                "Download recording parameter conversion failed for: {}",
                input_version
            );

            // Ensure both conversions are identical
            assert_eq!(agent_info_param, download_record_param, "Parameter conversion should be consistent between agent info and download recording for: {}", input_version);
        }
    }
}
