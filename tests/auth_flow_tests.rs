/// Comprehensive authentication flow testing
/// Tests both JWT and API key authentication flows
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

// Import the shared authentication module
use shared::{
    authenticate_api_key, authenticate_jwt, extract_bearer_token, guess_token_type, hash_api_key,
    validate_jwt_token, ApiError, AuthConfig, AuthMethod, AuthenticatedUser, SupabaseJwtClaims,
    TokenType, UserMetadata,
};

/// Test configuration for authentication tests
pub struct AuthTestConfig {
    pub mock_supabase_url: String,
    pub mock_supabase_key: String,
    pub mock_jwt_secret: String,
    pub debug_mode: bool,
}

impl Default for AuthTestConfig {
    fn default() -> Self {
        Self {
            mock_supabase_url: "https://test.supabase.co".to_string(),
            mock_supabase_key: "test_service_key".to_string(),
            mock_jwt_secret: "test_jwt_secret_for_development_only".to_string(),
            debug_mode: true,
        }
    }
}

impl AuthTestConfig {
    /// Create development auth config for testing
    fn to_auth_config(&self) -> AuthConfig {
        AuthConfig {
            supabase_url: self.mock_supabase_url.clone(),
            supabase_service_role_key: self.mock_supabase_key.clone(),
            supabase_jwt_secret: self.mock_jwt_secret.clone(),
            debug_mode: self.debug_mode,
        }
    }

    /// Create empty config to trigger development mode
    fn to_dev_auth_config(&self) -> AuthConfig {
        AuthConfig {
            supabase_url: "".to_string(), // Empty to trigger dev mode
            supabase_service_role_key: "".to_string(),
            supabase_jwt_secret: "".to_string(),
            debug_mode: self.debug_mode,
        }
    }
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    // JWT Authentication Tests
    #[tokio::test]
    async fn test_jwt_authentication_development_mode() {
        let config = AuthTestConfig::default().to_dev_auth_config();
        let mock_jwt_token = "mock.jwt.token";

        let result = authenticate_jwt(mock_jwt_token, &config).await;
        assert!(
            result.is_ok(),
            "JWT authentication should succeed in dev mode"
        );

        let user = result.unwrap();
        assert_eq!(
            user.user_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert!(matches!(
            user.auth_method,
            AuthMethod::JwtToken { provider } if provider == "supabase"
        ));
        assert!(user.scopes.contains(&"read".to_string()));
        assert!(user.scopes.contains(&"api_key_create".to_string()));
        assert_eq!(user.metadata.email, Some("dev@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_jwt_token_validation_development_mode() {
        let config = AuthTestConfig::default().to_dev_auth_config();
        let mock_jwt_token = "any.jwt.token";

        let result = validate_jwt_token(mock_jwt_token, &config).await;
        assert!(result.is_ok(), "JWT validation should succeed in dev mode");

        let claims = result.unwrap();
        assert_eq!(claims.sub, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(claims.aud, "authenticated");
        assert_eq!(claims.email, Some("dev@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_jwt_authentication_with_real_config() {
        let config = AuthTestConfig::default().to_auth_config();
        let invalid_jwt = "invalid.jwt.token";

        let result = authenticate_jwt(invalid_jwt, &config).await;
        assert!(
            result.is_err(),
            "JWT authentication should fail with invalid token"
        );

        let error = result.unwrap_err();
        assert_eq!(error.error, "invalid_jwt");
        assert!(error.message.contains("Invalid JWT token"));
    }

    // API Key Authentication Tests
    #[tokio::test]
    async fn test_api_key_authentication_development_mode() {
        let config = AuthTestConfig::default().to_dev_auth_config();
        let mock_api_key = "carp_test1234_test5678_test9012";

        let result = authenticate_api_key(mock_api_key, &config).await;
        assert!(
            result.is_ok(),
            "API key authentication should succeed in dev mode"
        );

        let user = result.unwrap();
        assert_eq!(
            user.user_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert!(matches!(
            user.auth_method,
            AuthMethod::ApiKey { key_id } if key_id == Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap()
        ));
        assert!(user.scopes.contains(&"read".to_string()));
        assert!(user.scopes.contains(&"write".to_string()));
        assert!(user.scopes.contains(&"upload".to_string()));
        assert!(user.scopes.contains(&"publish".to_string()));
        assert!(user.scopes.contains(&"admin".to_string()));
    }

    #[tokio::test]
    async fn test_api_key_hash_consistency() {
        let api_key = "carp_test1234_test5678_test9012";
        let hash1 = hash_api_key(api_key);
        let hash2 = hash_api_key(api_key);

        assert_eq!(hash1, hash2, "API key hashing should be consistent");
        assert!(!hash1.is_empty(), "Hash should not be empty");
        assert_ne!(hash1, api_key, "Hash should be different from original key");
    }

    // Token Type Detection Tests
    #[test]
    fn test_token_type_detection_api_key() {
        let api_key = "carp_abc12345_def67890_ghi09876";
        assert_eq!(guess_token_type(api_key), TokenType::ApiKey);
    }

    #[test]
    fn test_token_type_detection_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert_eq!(guess_token_type(jwt), TokenType::Jwt);
    }

    #[test]
    fn test_token_type_detection_ambiguous() {
        let ambiguous_token = "some_random_token";
        assert_eq!(guess_token_type(ambiguous_token), TokenType::Jwt); // Defaults to JWT
    }

    #[test]
    fn test_token_type_detection_malformed_api_key() {
        let malformed_api_key = "carp_only_two_parts";
        assert_eq!(guess_token_type(malformed_api_key), TokenType::Jwt); // Should default to JWT
    }

    // Scope Checking Tests
    #[test]
    fn test_scope_checking_basic_user() {
        let user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::new_v4(),
            },
            scopes: vec!["read".to_string(), "write".to_string()],
            metadata: UserMetadata {
                email: None,
                github_username: None,
                created_at: None,
            },
        };

        assert!(shared::check_scope(&user, "read"));
        assert!(shared::check_scope(&user, "write"));
        assert!(!shared::check_scope(&user, "admin"));
        assert!(!shared::check_scope(&user, "publish"));
    }

    #[test]
    fn test_scope_checking_admin_user() {
        let admin_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::new_v4(),
            },
            scopes: vec!["admin".to_string()],
            metadata: UserMetadata {
                email: None,
                github_username: None,
                created_at: None,
            },
        };

        // Admin should have access to all scopes
        assert!(shared::check_scope(&admin_user, "read"));
        assert!(shared::check_scope(&admin_user, "write"));
        assert!(shared::check_scope(&admin_user, "admin"));
        assert!(shared::check_scope(&admin_user, "publish"));
        assert!(shared::check_scope(&admin_user, "upload"));
    }

    // Authentication Error Scenarios
    #[tokio::test]
    async fn test_jwt_authentication_with_empty_token() {
        let config = AuthTestConfig::default().to_auth_config();
        let empty_token = "";

        let result = authenticate_jwt(empty_token, &config).await;
        assert!(result.is_err(), "Empty JWT should fail authentication");
    }

    #[tokio::test]
    async fn test_api_key_authentication_with_invalid_key() {
        let config = AuthTestConfig::default().to_auth_config();
        let invalid_key = "invalid_api_key_format";

        // In production mode with real config, this should fail
        // We'll test this when not in development mode
        if !config.is_development() {
            let result = authenticate_api_key(invalid_key, &config).await;
            assert!(
                result.is_err(),
                "Invalid API key should fail authentication"
            );
        }
    }

    // Bearer Token Extraction Tests (Mock Request Testing)
    #[test]
    fn test_bearer_token_extraction_formats() {
        // Note: These tests would require proper HTTP request mocking
        // For now, we test the logic components that don't require full HTTP requests

        // Test API key hash generation for various formats
        let api_keys = vec![
            "carp_test1234_test5678_test9012",
            "carp_abcdefgh_ijklmnop_qrstuvwx",
            "carp_12345678_87654321_11111111",
        ];

        for key in api_keys {
            let hash = hash_api_key(key);
            assert!(
                !hash.is_empty(),
                "Hash should not be empty for key: {}",
                key
            );
            assert_ne!(hash, key, "Hash should differ from original key: {}", key);

            // Verify token type detection
            assert_eq!(guess_token_type(key), TokenType::ApiKey);
        }
    }

    // Authentication Config Tests
    #[test]
    fn test_auth_config_development_detection() {
        let dev_config = AuthTestConfig::default().to_dev_auth_config();
        assert!(
            dev_config.is_development(),
            "Should detect development mode"
        );

        let prod_config = AuthTestConfig::default().to_auth_config();
        assert!(
            !prod_config.is_development(),
            "Should detect production mode"
        );
    }

    #[test]
    fn test_auth_config_from_env() {
        // Save original environment
        let original_url = env::var("SUPABASE_URL").ok();
        let original_key = env::var("SUPABASE_SERVICE_ROLE_KEY").ok();
        let original_secret = env::var("SUPABASE_JWT_SECRET").ok();
        let original_debug = env::var("DEBUG_AUTH").ok();

        // Set test environment variables
        env::set_var("SUPABASE_URL", "https://test-env.supabase.co");
        env::set_var("SUPABASE_SERVICE_ROLE_KEY", "env_service_key");
        env::set_var("SUPABASE_JWT_SECRET", "env_jwt_secret");
        env::set_var("DEBUG_AUTH", "true");

        let config = AuthConfig::from_env();
        assert_eq!(config.supabase_url, "https://test-env.supabase.co");
        assert_eq!(config.supabase_service_role_key, "env_service_key");
        assert_eq!(config.supabase_jwt_secret, "env_jwt_secret");
        assert!(config.debug_mode);

        // Restore original environment
        match original_url {
            Some(val) => env::set_var("SUPABASE_URL", val),
            None => env::remove_var("SUPABASE_URL"),
        }
        match original_key {
            Some(val) => env::set_var("SUPABASE_SERVICE_ROLE_KEY", val),
            None => env::remove_var("SUPABASE_SERVICE_ROLE_KEY"),
        }
        match original_secret {
            Some(val) => env::set_var("SUPABASE_JWT_SECRET", val),
            None => env::remove_var("SUPABASE_JWT_SECRET"),
        }
        match original_debug {
            Some(val) => env::set_var("DEBUG_AUTH", val),
            None => env::remove_var("DEBUG_AUTH"),
        }
    }

    // Test JWT Claims Structure
    #[test]
    fn test_supabase_jwt_claims_serialization() {
        let claims = SupabaseJwtClaims {
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            aud: "authenticated".to_string(),
            exp: (Utc::now().timestamp() + 3600),
            iat: Utc::now().timestamp(),
            iss: "supabase".to_string(),
            email: Some("test@example.com".to_string()),
            phone: None,
            app_metadata: Some(json!({"provider": "github"})),
            user_metadata: Some(json!({"github_username": "testuser"})),
            role: Some("authenticated".to_string()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&claims);
        assert!(serialized.is_ok(), "JWT claims should serialize correctly");

        // Test deserialization
        let json_str = serialized.unwrap();
        let deserialized: Result<SupabaseJwtClaims, _> = serde_json::from_str(&json_str);
        assert!(
            deserialized.is_ok(),
            "JWT claims should deserialize correctly"
        );

        let deserialized_claims = deserialized.unwrap();
        assert_eq!(deserialized_claims.sub, claims.sub);
        assert_eq!(deserialized_claims.email, claims.email);
    }

    // Error Response Structure Tests
    #[test]
    fn test_api_error_serialization() {
        let error = ApiError {
            error: "test_error".to_string(),
            message: "This is a test error message".to_string(),
            details: Some(json!({
                "code": 400,
                "additional_info": "Test details"
            })),
        };

        let serialized = serde_json::to_string(&error);
        assert!(serialized.is_ok(), "API error should serialize correctly");

        let json_str = serialized.unwrap();
        let deserialized: Result<ApiError, _> = serde_json::from_str(&json_str);
        assert!(
            deserialized.is_ok(),
            "API error should deserialize correctly"
        );

        let deserialized_error = deserialized.unwrap();
        assert_eq!(deserialized_error.error, "test_error");
        assert_eq!(deserialized_error.message, "This is a test error message");
        assert!(deserialized_error.details.is_some());
    }
}

#[cfg(test)]
mod middleware_tests {
    use super::*;
    use shared::{authenticate_request, AuthStrategy};
    // Note: Full middleware tests would require proper HTTP request mocking
    // These tests focus on the core logic that can be tested without full HTTP infrastructure

    #[test]
    fn test_auth_strategy_debug() {
        // Test that auth strategies can be formatted for debugging
        let jwt_only = AuthStrategy::JwtOnly;
        let api_key_only = AuthStrategy::ApiKeyOnly;

        let jwt_debug = format!("{:?}", jwt_only);
        let api_key_debug = format!("{:?}", api_key_only);

        assert!(jwt_debug.contains("JwtOnly"));
        assert!(api_key_debug.contains("ApiKeyOnly"));
    }

    // Integration test for middleware logic (without HTTP layer)
    #[tokio::test]
    async fn test_middleware_token_type_validation() {
        // Test that JWT-only strategy would reject API keys
        let api_key = "carp_test1234_test5678_test9012";
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI...";

        // Verify token type detection works as expected for middleware
        assert_eq!(guess_token_type(api_key), TokenType::ApiKey);
        assert_eq!(guess_token_type(jwt_token), TokenType::Jwt);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Test complete authentication flow in development mode
    #[tokio::test]
    async fn test_complete_jwt_to_api_key_flow_dev_mode() {
        let config = AuthTestConfig::default().to_dev_auth_config();

        // Step 1: Authenticate with JWT (simulating frontend login)
        let jwt_token = "mock.jwt.token";
        let jwt_user = authenticate_jwt(jwt_token, &config).await;
        assert!(jwt_user.is_ok(), "JWT authentication should succeed");

        let jwt_user = jwt_user.unwrap();
        assert!(jwt_user.scopes.contains(&"api_key_create".to_string()));

        // Step 2: Use JWT-authenticated user to simulate API key creation
        // (In a real test, this would make an HTTP request to the API key creation endpoint)
        let mock_api_key = "carp_generated_key123_test456_mock789";

        // Step 3: Use the generated API key for agent operations
        let api_key_user = authenticate_api_key(mock_api_key, &config).await;
        assert!(
            api_key_user.is_ok(),
            "API key authentication should succeed"
        );

        let api_key_user = api_key_user.unwrap();
        assert!(api_key_user.scopes.contains(&"upload".to_string()));
        assert!(api_key_user.scopes.contains(&"publish".to_string()));

        // Verify the users have the same ID (same actual user, different auth methods)
        assert_eq!(jwt_user.user_id, api_key_user.user_id);

        // Verify different authentication methods
        assert!(matches!(jwt_user.auth_method, AuthMethod::JwtToken { .. }));
        assert!(matches!(
            api_key_user.auth_method,
            AuthMethod::ApiKey { .. }
        ));
    }

    // Test authentication separation
    #[test]
    fn test_authentication_method_separation() {
        // JWT tokens should not be mistaken for API keys
        let jwt_tokens = vec![
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI...",
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
            "bearer.token.with.dots",
        ];

        for token in jwt_tokens {
            assert_eq!(
                guess_token_type(token),
                TokenType::Jwt,
                "Token should be detected as JWT: {}",
                token
            );
        }

        // API keys should not be mistaken for JWT tokens
        let api_keys = vec![
            "carp_test1234_test5678_test9012",
            "carp_abcdefgh_ijklmnop_qrstuvwx",
            "carp_11111111_22222222_33333333",
        ];

        for key in api_keys {
            assert_eq!(
                guess_token_type(key),
                TokenType::ApiKey,
                "Token should be detected as API key: {}",
                key
            );
        }
    }

    // Test error handling scenarios
    #[tokio::test]
    async fn test_authentication_error_handling() {
        let config = AuthTestConfig::default().to_auth_config(); // Use production mode config

        // Test various invalid tokens
        let invalid_tokens = vec![
            "",
            "invalid",
            "bearer",
            "carp_invalid",
            "carp_only_two_parts",
            "not.a.valid.jwt.token",
        ];

        for token in invalid_tokens {
            // Try JWT authentication
            let jwt_result = authenticate_jwt(token, &config).await;
            if token.is_empty() {
                assert!(jwt_result.is_err(), "Empty token should fail JWT auth");
            } else {
                // Most invalid tokens will fail JWT parsing
                assert!(
                    jwt_result.is_err(),
                    "Invalid token '{}' should fail JWT auth",
                    token
                );
            }

            // Try API key authentication (in production mode, these would fail database lookup)
            // In development mode, they would succeed, so we only test production mode
            if !config.is_development() {
                let api_key_result = authenticate_api_key(token, &config).await;
                assert!(
                    api_key_result.is_err(),
                    "Invalid token '{}' should fail API key auth",
                    token
                );
            }
        }
    }
}
