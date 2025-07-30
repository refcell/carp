/// Integration tests for complete authentication flows
/// Tests the full cycle: GitHub OAuth -> JWT -> API key creation -> CLI operations
use chrono::{DateTime, Utc};
use serde_json::json;
use std::env;
use uuid::Uuid;

use shared::{
    authenticate_api_key, authenticate_jwt, hash_api_key, sync_jwt_user, ApiError, AuthConfig,
    AuthMethod, AuthenticatedUser, SupabaseJwtClaims, UserMetadata,
};

/// Mock HTTP client for testing database operations
pub struct MockHttpClient {
    pub should_succeed: bool,
    pub mock_responses: std::collections::HashMap<String, serde_json::Value>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            should_succeed: true,
            mock_responses: std::collections::HashMap::new(),
        }
    }

    pub fn with_mock_response(mut self, url: &str, response: serde_json::Value) -> Self {
        self.mock_responses.insert(url.to_string(), response);
        self
    }

    pub fn should_fail(mut self) -> Self {
        self.should_succeed = false;
        self
    }
}

/// Integration test configuration
pub struct IntegrationTestConfig {
    pub supabase_url: String,
    pub supabase_key: String,
    pub jwt_secret: String,
    pub github_user_id: String,
    pub github_username: String,
    pub github_email: String,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            supabase_url: "https://test.supabase.co".to_string(),
            supabase_key: "test_service_key".to_string(),
            jwt_secret: "test_jwt_secret_for_integration_tests".to_string(),
            github_user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            github_username: "integration_test_user".to_string(),
            github_email: "integration@test.com".to_string(),
        }
    }
}

impl IntegrationTestConfig {
    fn to_auth_config(&self) -> AuthConfig {
        AuthConfig {
            supabase_url: self.supabase_url.clone(),
            supabase_service_role_key: self.supabase_key.clone(),
            supabase_jwt_secret: self.jwt_secret.clone(),
            debug_mode: true,
        }
    }

    fn to_dev_auth_config(&self) -> AuthConfig {
        AuthConfig {
            supabase_url: "".to_string(),
            supabase_service_role_key: "".to_string(),
            supabase_jwt_secret: "".to_string(),
            debug_mode: true,
        }
    }

    fn create_mock_jwt_claims(&self) -> SupabaseJwtClaims {
        SupabaseJwtClaims {
            sub: self.github_user_id.clone(),
            aud: "authenticated".to_string(),
            exp: (Utc::now().timestamp() + 3600), // 1 hour from now
            iat: Utc::now().timestamp(),
            iss: "supabase".to_string(),
            email: Some(self.github_email.clone()),
            phone: None,
            app_metadata: Some(json!({ "provider": "github" })),
            user_metadata: Some(json!({
                "github_username": self.github_username,
                "avatar_url": "https://github.com/avatars/integration_test_user"
            })),
            role: Some("authenticated".to_string()),
        }
    }
}

#[cfg(test)]
mod integration_flow_tests {
    use super::*;

    /// Test the complete authentication flow in development mode
    #[tokio::test]
    async fn test_complete_auth_flow_development() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Step 1: User logs in via GitHub OAuth (simulated by JWT authentication)
        let mock_jwt_token = "mock.github.oauth.jwt.token";
        let jwt_result = authenticate_jwt(mock_jwt_token, &auth_config).await;
        
        assert!(jwt_result.is_ok(), "GitHub OAuth JWT should authenticate successfully");
        
        let jwt_user = jwt_result.unwrap();
        assert_eq!(jwt_user.user_id.to_string(), test_config.github_user_id);
        assert!(matches!(jwt_user.auth_method, AuthMethod::JwtToken { .. }));
        assert!(jwt_user.scopes.contains(&"api_key_create".to_string()));
        assert_eq!(jwt_user.metadata.email, Some("dev@example.com".to_string()));

        // Step 2: Frontend uses JWT to create an API key
        // In development mode, we simulate the API key creation
        let generated_api_key = "carp_dev_test12_test5678_test9012";
        let api_key_hash = hash_api_key(generated_api_key);
        
        assert!(!api_key_hash.is_empty());
        assert_ne!(api_key_hash, generated_api_key);

        // Step 3: CLI uses the API key for agent operations
        let api_key_result = authenticate_api_key(generated_api_key, &auth_config).await;
        
        assert!(api_key_result.is_ok(), "Generated API key should authenticate successfully");
        
        let api_key_user = api_key_result.unwrap();
        assert_eq!(api_key_user.user_id, jwt_user.user_id); // Same user
        assert!(matches!(api_key_user.auth_method, AuthMethod::ApiKey { .. }));
        assert!(api_key_user.scopes.contains(&"upload".to_string()));
        assert!(api_key_user.scopes.contains(&"publish".to_string()));

        // Step 4: Verify authentication method separation
        assert!(shared::check_scope(&jwt_user, "api_key_create"));
        assert!(!shared::check_scope(&jwt_user, "upload"));

        assert!(shared::check_scope(&api_key_user, "upload"));
        assert!(!shared::check_scope(&api_key_user, "api_key_create"));
    }

    /// Test user synchronization between GitHub OAuth and local database
    #[tokio::test]
    async fn test_jwt_user_synchronization() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config(); // Dev mode skips sync

        // Create a user from JWT authentication
        let jwt_user = AuthenticatedUser {
            user_id: Uuid::parse_str(&test_config.github_user_id).unwrap(),
            auth_method: AuthMethod::JwtToken {
                provider: "supabase".to_string(),
            },
            scopes: vec![
                "read".to_string(),
                "api_key_create".to_string(),
            ],
            metadata: UserMetadata {
                email: Some(test_config.github_email.clone()),
                github_username: Some(test_config.github_username.clone()),
                created_at: Some(Utc::now()),
            },
        };

        // Test user synchronization (should succeed in dev mode)
        let sync_result = sync_jwt_user(&jwt_user, &auth_config).await;
        assert!(sync_result.is_ok(), "User sync should succeed in dev mode");

        // Verify user data is consistent
        assert_eq!(jwt_user.user_id.to_string(), test_config.github_user_id);
        assert_eq!(jwt_user.metadata.email, Some(test_config.github_email));
        assert_eq!(jwt_user.metadata.github_username, Some(test_config.github_username));
    }

    /// Test API key lifecycle management
    #[tokio::test]
    async fn test_api_key_lifecycle() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Step 1: Create multiple API keys for the same user
        let api_keys = vec![
            "carp_key1_test1234_test5678",
            "carp_key2_test1234_test5678", 
            "carp_key3_test1234_test5678",
        ];

        let mut authenticated_users = Vec::new();

        for api_key in &api_keys {
            let result = authenticate_api_key(api_key, &auth_config).await;
            assert!(result.is_ok(), "API key '{}' should authenticate", api_key);
            
            let user = result.unwrap();
            authenticated_users.push(user);
        }

        // Step 2: Verify all API keys belong to the same user
        let first_user_id = authenticated_users[0].user_id;
        for user in &authenticated_users {
            assert_eq!(user.user_id, first_user_id, "All API keys should belong to same user");
            assert!(matches!(user.auth_method, AuthMethod::ApiKey { .. }));
        }

        // Step 3: Verify different API keys have different key IDs
        let mut key_ids = std::collections::HashSet::new();
        for user in &authenticated_users {
            if let AuthMethod::ApiKey { key_id } = user.auth_method {
                key_ids.insert(key_id);
            }
        }
        
        // In development mode, all keys use the same mock key ID
        // In production, each would have a unique key ID
        if auth_config.is_development() {
            assert_eq!(key_ids.len(), 1, "Dev mode uses same mock key ID");
        }

        // Step 4: Test API key hashing consistency
        for api_key in &api_keys {
            let hash1 = hash_api_key(api_key);
            let hash2 = hash_api_key(api_key);
            assert_eq!(hash1, hash2, "API key hashing should be deterministic");
        }
    }

    /// Test authentication error scenarios in integration context
    #[tokio::test]
    async fn test_integration_error_scenarios() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_auth_config(); // Production-like config

        // Test 1: Invalid JWT token
        let invalid_jwt = "invalid.jwt.token";
        let jwt_result = authenticate_jwt(invalid_jwt, &auth_config).await;
        assert!(jwt_result.is_err(), "Invalid JWT should fail");
        
        if let Err(error) = jwt_result {
            assert_eq!(error.error, "invalid_jwt");
            assert!(error.message.contains("Invalid JWT token"));
        }

        // Test 2: Expired JWT claims (if we had real JWT parsing)
        // This would require proper JWT token generation and expiration testing

        // Test 3: API key authentication without database (production mode)
        let api_key = "carp_test1234_test5678_test9012";
        let api_result = authenticate_api_key(api_key, &auth_config).await;
        
        // In production mode without mock database, this should fail
        if !auth_config.is_development() {
            assert!(api_result.is_err(), "API key should fail without database");
        }

        // Test 4: Malformed tokens
        let malformed_tokens = vec!["", "malformed", "carp_incomplete"];
        
        for token in malformed_tokens {
            let jwt_result = authenticate_jwt(token, &auth_config).await;
            let api_result = authenticate_api_key(token, &auth_config).await;
            
            if !auth_config.is_development() {
                assert!(jwt_result.is_err(), "Malformed token '{}' should fail JWT", token);
                assert!(api_result.is_err(), "Malformed token '{}' should fail API key", token);
            }
        }
    }

    /// Test concurrent authentication requests
    #[tokio::test]
    async fn test_concurrent_authentication() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Test concurrent JWT authentications
        let jwt_tasks: Vec<_> = (0..10)
            .map(|i| {
                let config = auth_config.clone();
                let token = format!("mock.jwt.token.{}", i);
                tokio::spawn(async move { authenticate_jwt(&token, &config).await })
            })
            .collect();

        let jwt_results = futures::future::join_all(jwt_tasks).await;
        for result in jwt_results {
            let auth_result = result.unwrap();
            assert!(auth_result.is_ok(), "Concurrent JWT auth should succeed");
        }

        // Test concurrent API key authentications
        let api_key_tasks: Vec<_> = (0..10)
            .map(|i| {
                let config = auth_config.clone();
                let key = format!("carp_test{:04}_test5678_test9012", i);
                tokio::spawn(async move { authenticate_api_key(&key, &config).await })
            })
            .collect();

        let api_results = futures::future::join_all(api_key_tasks).await;
        for result in api_results {
            let auth_result = result.unwrap();
            assert!(auth_result.is_ok(), "Concurrent API key auth should succeed");
        }
    }

    /// Test authentication with different scope combinations
    #[tokio::test]
    async fn test_scope_based_authentication() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Test JWT authentication (frontend scopes)
        let jwt_result = authenticate_jwt("mock.jwt.token", &auth_config).await;
        assert!(jwt_result.is_ok());
        
        let jwt_user = jwt_result.unwrap();
        let expected_jwt_scopes = vec!["read", "api_key_create", "api_key_manage"];
        
        for scope in expected_jwt_scopes {
            assert!(
                shared::check_scope(&jwt_user, scope),
                "JWT user should have scope: {}",
                scope
            );
        }

        // JWT users should NOT have CLI scopes
        let cli_only_scopes = vec!["upload", "publish", "write"];
        for scope in cli_only_scopes {
            assert!(
                !shared::check_scope(&jwt_user, scope),
                "JWT user should NOT have CLI scope: {}",
                scope
            );
        }

        // Test API key authentication (CLI scopes)
        let api_result = authenticate_api_key("carp_test_key_123", &auth_config).await;
        assert!(api_result.is_ok());
        
        let api_user = api_result.unwrap();
        let expected_api_scopes = vec!["read", "write", "upload", "publish", "admin"];
        
        for scope in expected_api_scopes {
            assert!(
                shared::check_scope(&api_user, scope),
                "API key user should have scope: {}",
                scope
            );
        }

        // In dev mode, API key users get admin scope, so they have all scopes
        // This is different from production where scopes would be more restricted
        assert!(shared::check_scope(&api_user, "api_key_create"));
    }

    /// Test metadata preservation across authentication methods
    #[tokio::test]
    async fn test_metadata_consistency() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Authenticate via JWT
        let jwt_result = authenticate_jwt("mock.jwt.token", &auth_config).await;
        assert!(jwt_result.is_ok());
        let jwt_user = jwt_result.unwrap();

        // Authenticate via API key
        let api_result = authenticate_api_key("carp_test_key_123", &auth_config).await;
        assert!(api_result.is_ok());
        let api_user = api_result.unwrap();

        // In development mode, both should represent the same user
        assert_eq!(jwt_user.user_id, api_user.user_id);
        
        // Metadata should be consistent
        assert_eq!(jwt_user.metadata.email, api_user.metadata.email);
        assert_eq!(jwt_user.metadata.github_username, api_user.metadata.github_username);

        // Authentication methods should be different
        assert!(matches!(jwt_user.auth_method, AuthMethod::JwtToken { .. }));
        assert!(matches!(api_user.auth_method, AuthMethod::ApiKey { .. }));
    }

    /// Test rate limiting and security scenarios
    #[tokio::test]
    async fn test_security_scenarios() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Test 1: Very long tokens (potential DoS)
        let long_token = "a".repeat(10000);
        let jwt_result = authenticate_jwt(&long_token, &auth_config).await;
        
        // Should handle gracefully (in dev mode, it succeeds)
        if auth_config.is_development() {
            assert!(jwt_result.is_ok(), "Dev mode should handle long tokens");
        }

        // Test 2: Special characters in tokens
        let special_tokens = vec![
            "token\n\nwith\nnewlines",
            "token\x00with\x00nulls",
            "token with spaces",
            "токен_с_unicode",
        ];

        for token in special_tokens {
            let jwt_result = authenticate_jwt(token, &auth_config).await;
            let api_result = authenticate_api_key(token, &auth_config).await;
            
            // These should be handled gracefully
            if auth_config.is_development() {
                // Dev mode is permissive for testing
                assert!(jwt_result.is_ok(), "Dev mode should handle special token: {}", token);
            } else {
                // Production mode should be more strict
                // (This would require proper validation in the actual implementation)
            }
        }

        // Test 3: Token type confusion attacks
        let jwt_looking_api_key = "carp_eyJhbGci_eyJzdWIi_eyJpc3Mi"; // Looks like JWT but is API key format
        let api_looking_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.carp_fake_jwt_token.signature";

        assert_eq!(shared::guess_token_type(jwt_looking_api_key), shared::TokenType::ApiKey);
        assert_eq!(shared::guess_token_type(api_looking_jwt), shared::TokenType::Jwt);
    }
}

#[cfg(test)]
mod production_simulation_tests {
    use super::*;

    /// Test authentication behavior that simulates production environment
    #[tokio::test]
    async fn test_production_like_environment() {
        let test_config = IntegrationTestConfig::default();
        let prod_config = test_config.to_auth_config(); // Non-empty config = production-like

        // In production-like environment (with non-empty config), authentication should fail
        // without proper database/JWT validation
        
        let invalid_jwt = "invalid.jwt.token";
        let jwt_result = authenticate_jwt(invalid_jwt, &prod_config).await;
        assert!(jwt_result.is_err(), "Invalid JWT should fail in production");

        let api_key = "carp_test1234_test5678_test9012";
        let api_result = authenticate_api_key(api_key, &prod_config).await;
        assert!(api_result.is_err(), "API key should fail without database in production");

        // Verify error messages are appropriate
        if let Err(jwt_error) = jwt_result {
            assert_eq!(jwt_error.error, "invalid_jwt");
            assert!(jwt_error.message.contains("Invalid JWT token"));
        }

        if let Err(api_error) = api_result {
            assert_eq!(api_error.error, "database_error");
            assert!(api_error.message.contains("Failed to verify API key"));
        }
    }

    /// Test configuration validation
    #[test]
    fn test_auth_config_validation() {
        let test_config = IntegrationTestConfig::default();
        
        // Test development mode detection
        let dev_config = test_config.to_dev_auth_config();
        assert!(dev_config.is_development());
        
        let prod_config = test_config.to_auth_config();
        assert!(!prod_config.is_development());
        
        // Test config from environment
        let original_debug = env::var("DEBUG_AUTH").ok();
        env::set_var("DEBUG_AUTH", "true");
        
        let env_config = AuthConfig::from_env();
        assert!(env_config.debug_mode);
        
        // Restore environment
        match original_debug {
            Some(val) => env::set_var("DEBUG_AUTH", val),
            None => env::remove_var("DEBUG_AUTH"),
        }
    }

    /// Test error propagation and handling
    #[tokio::test]
    async fn test_error_propagation() {
        let test_config = IntegrationTestConfig::default();
        let prod_config = test_config.to_auth_config();

        // Test that authentication errors contain detailed information
        let result = authenticate_jwt("malformed.jwt", &prod_config).await;
        
        if let Err(error) = result {
            // Error should have structured information
            assert!(!error.error.is_empty());
            assert!(!error.message.is_empty());
            
            // Error details should provide debugging information
            if let Some(details) = error.details {
                assert!(details.is_object());
            }
        }
    }
}

// Helper function to run all integration tests
#[cfg(test)]
pub async fn run_integration_test_suite() {
    println!("Running authentication integration test suite...");
    
    // This would be called by a test runner to execute all integration tests
    // In practice, cargo test handles this automatically
}

// Performance benchmarking (if needed)
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_authentication_performance() {
        let test_config = IntegrationTestConfig::default();
        let auth_config = test_config.to_dev_auth_config();

        // Benchmark JWT authentication
        let start = Instant::now();
        let iterations = 100;
        
        for i in 0..iterations {
            let token = format!("mock.jwt.token.{}", i);
            let result = authenticate_jwt(&token, &auth_config).await;
            assert!(result.is_ok());
        }
        
        let jwt_duration = start.elapsed();
        let jwt_avg = jwt_duration / iterations;
        
        println!("JWT auth average time: {:?}", jwt_avg);
        
        // Benchmark API key authentication
        let start = Instant::now();
        
        for i in 0..iterations {
            let key = format!("carp_test{:04}_test5678_test9012", i);
            let result = authenticate_api_key(&key, &auth_config).await;
            assert!(result.is_ok());
        }
        
        let api_duration = start.elapsed();
        let api_avg = api_duration / iterations;
        
        println!("API key auth average time: {:?}", api_avg);
        
        // Both should be reasonably fast (under 1ms each in dev mode)
        assert!(jwt_avg.as_millis() < 10, "JWT auth should be fast");
        assert!(api_avg.as_millis() < 10, "API key auth should be fast");
    }
}