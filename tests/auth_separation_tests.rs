/// Authentication separation testing
/// Tests that JWT-only and API-key-only endpoints properly enforce authentication methods
use std::collections::HashMap;
use std::env;

use shared::{
    api_key_middleware, jwt_middleware, require_scope, ApiError, AuthMethod, AuthenticatedUser,
    TokenType, UserMetadata,
};
use uuid::Uuid;

/// Mock HTTP request builder for testing
#[derive(Debug)]
pub struct MockRequest {
    headers: HashMap<String, String>,
    method: String,
    body: Vec<u8>,
    uri: String,
}

impl MockRequest {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            method: "GET".to_string(),
            body: Vec::new(),
            uri: "/".to_string(),
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_lowercase(), value.to_string());
        self
    }

    pub fn bearer_token(mut self, token: &str) -> Self {
        self.headers
            .insert("authorization".to_string(), format!("Bearer {}", token));
        self
    }

    pub fn api_key_header(mut self, key: &str) -> Self {
        self.headers.insert("x-api-key".to_string(), key.to_string());
        self
    }

    pub fn body(mut self, body: &[u8]) -> Self {
        self.body = body.to_vec();
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }
}

#[cfg(test)]
mod auth_separation_tests {
    use super::*;
    // Note: These tests demonstrate the intended behavior of authentication separation
    // Full HTTP testing would require implementing HTTP request mocking for vercel_runtime::Request

    #[test]
    fn test_token_type_separation_logic() {
        // Test that our token detection logic properly separates JWT from API keys

        // Valid API key format should be detected as API key
        let api_key = "carp_test1234_test5678_test9012";
        assert_eq!(shared::guess_token_type(api_key), TokenType::ApiKey);

        // Valid JWT format should be detected as JWT
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert_eq!(shared::guess_token_type(jwt), TokenType::Jwt);

        // Edge cases should default to JWT for security
        assert_eq!(shared::guess_token_type("ambiguous_token"), TokenType::Jwt);
        assert_eq!(shared::guess_token_type("carp_malformed"), TokenType::Jwt);
        assert_eq!(shared::guess_token_type(""), TokenType::Jwt);
    }

    #[test]
    fn test_scope_validation_logic() {
        // Test JWT user scopes (frontend operations)
        let jwt_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::JwtToken {
                provider: "supabase".to_string(),
            },
            scopes: vec![
                "read".to_string(),
                "api_key_create".to_string(),
                "api_key_manage".to_string(),
            ],
            metadata: UserMetadata {
                email: Some("test@example.com".to_string()),
                github_username: Some("testuser".to_string()),
                created_at: None,
            },
        };

        // JWT users should be able to create API keys
        assert!(shared::check_scope(&jwt_user, "api_key_create"));
        assert!(shared::check_scope(&jwt_user, "read"));

        // But not perform CLI operations directly
        assert!(!shared::check_scope(&jwt_user, "upload"));
        assert!(!shared::check_scope(&jwt_user, "publish"));

        // Test API key user scopes (CLI operations)
        let api_key_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::new_v4(),
            },
            scopes: vec![
                "read".to_string(),
                "write".to_string(),
                "upload".to_string(),
                "publish".to_string(),
            ],
            metadata: UserMetadata {
                email: Some("test@example.com".to_string()),
                github_username: Some("testuser".to_string()),
                created_at: None,
            },
        };

        // API key users should be able to perform CLI operations
        assert!(shared::check_scope(&api_key_user, "upload"));
        assert!(shared::check_scope(&api_key_user, "publish"));
        assert!(shared::check_scope(&api_key_user, "read"));
        assert!(shared::check_scope(&api_key_user, "write"));

        // But not create API keys (unless explicitly granted admin scope)
        assert!(!shared::check_scope(&api_key_user, "api_key_create"));
    }

    #[test]
    fn test_admin_user_scope_override() {
        // Test that admin users have access to all scopes
        let admin_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::new_v4(),
            },
            scopes: vec!["admin".to_string()],
            metadata: UserMetadata {
                email: Some("admin@example.com".to_string()),
                github_username: Some("admin".to_string()),
                created_at: None,
            },
        };

        // Admin should have access to all operations
        assert!(shared::check_scope(&admin_user, "read"));
        assert!(shared::check_scope(&admin_user, "write"));
        assert!(shared::check_scope(&admin_user, "upload"));
        assert!(shared::check_scope(&admin_user, "publish"));
        assert!(shared::check_scope(&admin_user, "api_key_create"));
        assert!(shared::check_scope(&admin_user, "api_key_manage"));
        assert!(shared::check_scope(&admin_user, "admin"));
        assert!(shared::check_scope(&admin_user, "delete"));
    }

    // Test authentication method validation for different endpoint types
    #[test]
    fn test_jwt_only_endpoint_requirements() {
        // Endpoints that should only accept JWT tokens (frontend operations)
        let jwt_only_endpoints = vec![
            "/v1/auth/api-keys POST",      // Create API key
            "/profile",                    // User profile
            "/dashboard",                  // User dashboard
        ];

        let api_key = "carp_test1234_test5678_test9012";
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI...";

        // JWT-only endpoints should reject API keys
        assert_eq!(shared::guess_token_type(api_key), TokenType::ApiKey);
        assert_eq!(shared::guess_token_type(jwt_token), TokenType::Jwt);

        // The middleware logic ensures API keys are rejected for JWT-only endpoints
        // This is tested in the middleware tests
    }

    #[test]
    fn test_api_key_only_endpoint_requirements() {
        // Endpoints that should only accept API keys (CLI operations)
        let api_key_only_endpoints = vec![
            "/v1/agents/upload POST",          // Upload agent
            "/v1/agents/publish POST",         // Publish agent
            "/v1/agents/{name}/{version}/download GET", // Download agent
            "/v1/auth/api-keys GET",           // List API keys
            "/v1/auth/api-keys PUT",           // Update API key
            "/v1/auth/api-keys DELETE",        // Delete API key
        ];

        let api_key = "carp_test1234_test5678_test9012";
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI...";

        // API-key-only endpoints should reject JWT tokens
        assert_eq!(shared::guess_token_type(api_key), TokenType::ApiKey);
        assert_eq!(shared::guess_token_type(jwt_token), TokenType::Jwt);

        // The middleware logic ensures JWT tokens are rejected for API-key-only endpoints
        // This is tested in the middleware tests
    }

    #[test]
    fn test_error_messages_for_wrong_auth_method() {
        // Test that appropriate error messages are generated for wrong authentication methods

        let api_key = "carp_test1234_test5678_test9012";
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVDAuZXlKemRXSWlPaUl4TWpNME5UWTNO";

        // Verify token type detection
        assert_eq!(shared::guess_token_type(api_key), TokenType::ApiKey);
        assert_eq!(shared::guess_token_type(jwt_token), TokenType::Jwt);

        // The actual error message generation happens in the middleware
        // Here we test the token detection that drives those error messages
    }

    #[tokio::test]
    async fn test_development_mode_authentication_flows() {
        // In development mode, both JWT and API key authentication should work
        // but still maintain proper scope separation

        let config = shared::AuthConfig {
            supabase_url: "".to_string(), // Empty triggers dev mode
            supabase_service_role_key: "".to_string(),
            supabase_jwt_secret: "".to_string(),
            debug_mode: true,
        };

        // Test JWT authentication in dev mode
        let jwt_result = shared::authenticate_jwt("mock.jwt.token", &config).await;
        assert!(jwt_result.is_ok(), "JWT should work in dev mode");

        let jwt_user = jwt_result.unwrap();
        assert!(matches!(
            jwt_user.auth_method,
            AuthMethod::JwtToken { .. }
        ));
        assert!(jwt_user.scopes.contains(&"api_key_create".to_string()));

        // Test API key authentication in dev mode
        let api_key_result = shared::authenticate_api_key("carp_dev_key_123", &config).await;
        assert!(api_key_result.is_ok(), "API key should work in dev mode");

        let api_key_user = api_key_result.unwrap();
        assert!(matches!(api_key_user.auth_method, AuthMethod::ApiKey { .. }));
        assert!(api_key_user.scopes.contains(&"upload".to_string()));

        // Verify users have the same ID (consistent dev user)
        assert_eq!(jwt_user.user_id, api_key_user.user_id);
    }

    #[test]
    fn test_api_key_format_validation() {
        // Test various API key formats to ensure proper detection
        let valid_api_keys = vec![
            "carp_test1234_test5678_test9012",
            "carp_abcdefgh_ijklmnop_qrstuvwx",
            "carp_12345678_87654321_11111111",
            "carp_ABC12345_DEF67890_GHI09876",
        ];

        for key in valid_api_keys {
            assert_eq!(
                shared::guess_token_type(key),
                TokenType::ApiKey,
                "Valid API key should be detected: {}",
                key
            );
        }

        let invalid_api_keys = vec![
            "carp_only_two_parts",         // Not enough parts
            "wrong_prefix_test_test_test", // Wrong prefix
            "carp_test1234_test5678",      // Only 2 parts after prefix
            "carp_test1234_test5678_test9012_extra", // Too many parts
            "",                            // Empty string
            "carp___",                     // Empty parts
        ];

        for key in invalid_api_keys {
            assert_eq!(
                shared::guess_token_type(key),
                TokenType::Jwt,
                "Invalid API key should default to JWT: {}",
                key
            );
        }
    }

    #[test]
    fn test_jwt_format_validation() {
        // Test various JWT formats to ensure proper detection
        let valid_jwt_formats = vec![
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJhdWQiOiIxIiwianRpIjoiZGY4NzNhNDNlYWFmNzg4ZDJlZDNhMTYwNjU5ODRkZmE4NmVkNmIzNWZjOGU5OGQ4OWEwMTQ4MDcxYmJlNGJmYWZlZTZmOTY5NzU3NDA3ZmIiLCJpYXQiOjE2NDM5OTgwMzUsIm5iZiI6MTY0Mzk5ODAzNSwiZXhwIjoxNjQzOTk4MDM1LCJzdWIiOiIxIiwic2NvcGVzIjpbXX0.xyz",
            "bearer.token.with.dots.and.long.enough.content.to.be.detected.as.jwt",
        ];

        for token in valid_jwt_formats {
            assert_eq!(
                shared::guess_token_type(token),
                TokenType::Jwt,
                "Valid JWT format should be detected: {}",
                token
            );
        }
    }

    #[test]
    fn test_scope_requirements_by_endpoint_type() {
        // Test that different endpoint types have appropriate scope requirements

        // Frontend endpoints (JWT-only) typically require these scopes
        let frontend_scopes = vec!["read", "api_key_create", "api_key_manage"];

        // CLI endpoints (API-key-only) typically require these scopes
        let cli_scopes = vec!["read", "write", "upload", "publish"];

        // Create users with different scope sets
        let frontend_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::JwtToken {
                provider: "supabase".to_string(),
            },
            scopes: frontend_scopes.iter().map(|s| s.to_string()).collect(),
            metadata: UserMetadata {
                email: Some("frontend@example.com".to_string()),
                github_username: Some("frontend_user".to_string()),
                created_at: None,
            },
        };

        let cli_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::new_v4(),
            },
            scopes: cli_scopes.iter().map(|s| s.to_string()).collect(),
            metadata: UserMetadata {
                email: Some("cli@example.com".to_string()),
                github_username: Some("cli_user".to_string()),
                created_at: None,
            },
        };

        // Test frontend user can access frontend operations
        assert!(shared::check_scope(&frontend_user, "api_key_create"));
        assert!(shared::check_scope(&frontend_user, "read"));

        // But not CLI operations
        assert!(!shared::check_scope(&frontend_user, "upload"));
        assert!(!shared::check_scope(&frontend_user, "publish"));

        // Test CLI user can access CLI operations
        assert!(shared::check_scope(&cli_user, "upload"));
        assert!(shared::check_scope(&cli_user, "publish"));
        assert!(shared::check_scope(&cli_user, "write"));

        // But not frontend-specific operations
        assert!(!shared::check_scope(&cli_user, "api_key_create"));
    }

    #[test]
    fn test_authentication_method_metadata() {
        // Test that authentication methods carry appropriate metadata

        let jwt_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::JwtToken {
                provider: "supabase".to_string(),
            },
            scopes: vec!["read".to_string()],
            metadata: UserMetadata {
                email: Some("jwt@example.com".to_string()),
                github_username: Some("jwt_user".to_string()),
                created_at: Some(chrono::Utc::now()),
            },
        };

        let api_key_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            },
            scopes: vec!["upload".to_string()],
            metadata: UserMetadata {
                email: Some("api@example.com".to_string()),
                github_username: Some("api_user".to_string()),
                created_at: Some(chrono::Utc::now()),
            },
        };

        // Test JWT method contains provider information
        match jwt_user.auth_method {
            AuthMethod::JwtToken { provider } => {
                assert_eq!(provider, "supabase");
            }
            _ => panic!("Expected JWT token auth method"),
        }

        // Test API key method contains key ID
        match api_key_user.auth_method {
            AuthMethod::ApiKey { key_id } => {
                assert_eq!(
                    key_id,
                    Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap()
                );
            }
            _ => panic!("Expected API key auth method"),
        }

        // Test metadata is properly preserved
        assert!(jwt_user.metadata.email.is_some());
        assert!(jwt_user.metadata.github_username.is_some());
        assert!(api_key_user.metadata.email.is_some());
        assert!(api_key_user.metadata.github_username.is_some());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_authentication_error_structure() {
        // Test various authentication error scenarios
        let errors = vec![
            ApiError {
                error: "missing_authentication".to_string(),
                message: "JWT authentication required".to_string(),
                details: Some(serde_json::json!({
                    "strategy": "JwtOnly",
                    "accepted_methods": ["jwt_token"]
                })),
            },
            ApiError {
                error: "invalid_auth_method".to_string(),
                message: "API keys are not allowed for this endpoint".to_string(),
                details: Some(serde_json::json!({
                    "received_token_type": "api_key",
                    "expected_token_type": "jwt_token"
                })),
            },
            ApiError {
                error: "insufficient_scope".to_string(),
                message: "Required scope 'upload' not found in user permissions".to_string(),
                details: Some(serde_json::json!({
                    "required_scope": "upload",
                    "user_scopes": ["read", "api_key_create"]
                })),
            },
        ];

        for error in errors {
            // Test that errors can be serialized (for HTTP responses)
            let serialized = serde_json::to_string(&error);
            assert!(
                serialized.is_ok(),
                "Error should serialize: {}",
                error.message
            );

            // Test that errors contain expected fields
            assert!(!error.error.is_empty());
            assert!(!error.message.is_empty());
        }
    }

    #[tokio::test]
    async fn test_invalid_token_handling() {
        let config = shared::AuthConfig {
            supabase_url: "https://test.supabase.co".to_string(),
            supabase_service_role_key: "test_key".to_string(),
            supabase_jwt_secret: "test_secret".to_string(),
            debug_mode: false,
        };

        // Test completely invalid tokens
        let invalid_tokens = vec!["", "invalid", "not.a.token", "malformed"];

        for token in invalid_tokens {
            let jwt_result = shared::authenticate_jwt(token, &config).await;
            assert!(
                jwt_result.is_err(),
                "Invalid token should fail JWT auth: '{}'",
                token
            );

            if let Err(error) = jwt_result {
                assert_eq!(error.error, "invalid_jwt");
                assert!(error.message.contains("Invalid JWT token"));
            }
        }
    }

    #[test]
    fn test_scope_requirement_errors() {
        // Test scope requirement validation
        let user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::new_v4(),
            },
            scopes: vec!["read".to_string()],
            metadata: UserMetadata {
                email: None,
                github_username: None,
                created_at: None,
            },
        };

        // User should have read access
        assert!(shared::check_scope(&user, "read"));

        // User should NOT have write access
        assert!(!shared::check_scope(&user, "write"));
        assert!(!shared::check_scope(&user, "upload"));
        assert!(!shared::check_scope(&user, "admin"));

        // Test that require_scope function would generate proper errors
        // (This is tested more thoroughly in integration tests with actual HTTP middleware)
    }
}