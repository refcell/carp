/// Authentication and Authorization tests
/// These tests cover user authentication, token validation, and permission checks

use carp_api::{
    auth::{AuthService, AuthUser},
    db::Database,
    models::{AuthRequest, UserProfile},
    utils::{config::Config, ApiError},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// JWT claims structure for testing
#[derive(Debug, Serialize, Deserialize)]
struct TestClaims {
    sub: String,
    username: String,
    email: String,
    scopes: Vec<String>,
    exp: usize,
    iat: usize,
}

// Test utilities for authentication
mod auth_test_utils {
    use super::*;
    use mockito::ServerGuard;
    use serde_json::json;
    use std::env;

    pub struct AuthTestContext {
        pub mock_server: ServerGuard,
        pub config: Arc<Config>,
        pub auth_service: Arc<AuthService>,
    }

    impl AuthTestContext {
        pub async fn new() -> Self {
            let mut server = mockito::Server::new_async().await;
            let db_url = server.url();

            // Set test environment variables
            env::set_var("SUPABASE_URL", &db_url);
            env::set_var("SUPABASE_ANON_KEY", "test-anon-key");
            env::set_var("SUPABASE_SERVICE_KEY", "test-service-key");
            env::set_var("JWT_SECRET", "test-jwt-secret-key-for-authentication-testing-only");
            env::set_var("SERVER_HOST", "localhost");
            env::set_var("SERVER_PORT", "3000");
            env::set_var("UPLOAD_MAX_FILE_SIZE", "10485760");
            env::set_var("UPLOAD_STORAGE_BUCKET", "test-bucket");

            let config = Arc::new(Config::from_env().expect("Failed to load config"));
            let db = Database::new().expect("Failed to create database");
            let auth_service = Arc::new(AuthService::new(db, config.clone()));

            Self {
                mock_server: server,
                config,
                auth_service,
            }
        }

        pub fn create_test_jwt(&self, user_id: Uuid, username: &str, scopes: Vec<String>) -> String {
            let now = Utc::now().timestamp() as usize;
            let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;

            let claims = TestClaims {
                sub: user_id.to_string(),
                username: username.to_string(),
                email: format!("{}@test.com", username),
                scopes,
                exp,
                iat: now,
            };

            let header = Header::new(Algorithm::HS256);
            encode(
                &header,
                &claims,
                &EncodingKey::from_secret(self.config.jwt.secret.as_ref()),
            )
            .expect("Failed to create test JWT")
        }

        pub fn create_expired_jwt(&self, user_id: Uuid, username: &str) -> String {
            let past_time = (Utc::now() - Duration::hours(2)).timestamp() as usize;

            let claims = TestClaims {
                sub: user_id.to_string(),
                username: username.to_string(),
                email: format!("{}@test.com", username),
                scopes: vec!["read".to_string()],
                exp: past_time, // Expired
                iat: past_time,
            };

            let header = Header::new(Algorithm::HS256);
            encode(
                &header,
                &claims,
                &EncodingKey::from_secret(self.config.jwt.secret.as_ref()),
            )
            .expect("Failed to create expired JWT")
        }

        pub fn create_invalid_jwt(&self) -> String {
            "invalid.jwt.token".to_string()
        }

        pub fn mock_user_lookup(&mut self, username: &str, password_hash: &str, user_id: &str) -> mockito::Mock {
            self.mock_server
                .mock("GET", "/rest/v1/users")
                .match_query(mockito::Matcher::AllOf(vec![
                    mockito::Matcher::UrlEncoded("select".to_string(), "*".to_string()),
                    mockito::Matcher::UrlEncoded("username".to_string(), format!("eq.{}", username)),
                ]))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(
                    json!([{
                        "id": user_id,
                        "username": username,
                        "email": format!("{}@test.com", username),
                        "password_hash": password_hash,
                        "created_at": "2024-01-01T00:00:00Z"
                    }])
                    .to_string(),
                )
                .create()
        }

        pub fn mock_user_not_found(&mut self, username: &str) -> mockito::Mock {
            self.mock_server
                .mock("GET", "/rest/v1/users")
                .match_query(mockito::Matcher::AllOf(vec![
                    mockito::Matcher::UrlEncoded("select".to_string(), "*".to_string()),
                    mockito::Matcher::UrlEncoded("username".to_string(), format!("eq.{}", username)),
                ]))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("[]")
                .create()
        }

        pub fn mock_profile_lookup(&mut self, user_id: &str, username: &str) -> mockito::Mock {
            self.mock_server
                .mock("GET", "/rest/v1/users")
                .match_query(mockito::Matcher::AllOf(vec![
                    mockito::Matcher::UrlEncoded("select".to_string(), "id,username,email,created_at".to_string()),
                    mockito::Matcher::UrlEncoded("id".to_string(), format!("eq.{}", user_id)),
                ]))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(
                    json!([{
                        "id": user_id,
                        "username": username,
                        "email": format!("{}@test.com", username),
                        "created_at": "2024-01-01T00:00:00Z"
                    }])
                    .to_string(),
                )
                .create()
        }
    }
}

// Test JWT token creation and validation
#[tokio::test]
async fn test_jwt_token_creation() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();
    let username = "testuser";
    let scopes = vec!["read".to_string(), "write".to_string()];

    let token = ctx.create_test_jwt(user_id, username, scopes.clone());

    // Verify token can be decoded
    let validation = Validation::new(Algorithm::HS256);
    let decoded = decode::<TestClaims>(
        &token,
        &DecodingKey::from_secret(ctx.config.jwt.secret.as_ref()),
        &validation,
    );

    assert!(decoded.is_ok());
    let claims = decoded.unwrap().claims;
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.username, username);
    assert_eq!(claims.scopes, scopes);
}

// Test JWT token expiration
#[tokio::test]
async fn test_jwt_token_expiration() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();
    let username = "testuser";

    let expired_token = ctx.create_expired_jwt(user_id, username);

    // Verify expired token is rejected
    let validation = Validation::new(Algorithm::HS256);
    let decoded = decode::<TestClaims>(
        &expired_token,
        &DecodingKey::from_secret(ctx.config.jwt.secret.as_ref()),
        &validation,
    );

    assert!(decoded.is_err());
}

// Test invalid JWT token format
#[tokio::test]
async fn test_invalid_jwt_token() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let invalid_token = ctx.create_invalid_jwt();

    let validation = Validation::new(Algorithm::HS256);
    let decoded = decode::<TestClaims>(
        &invalid_token,
        &DecodingKey::from_secret(ctx.config.jwt.secret.as_ref()),
        &validation,
    );

    assert!(decoded.is_err());
}

// Test user authentication with valid credentials
#[tokio::test]
async fn test_user_authentication_success() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();
    let username = "validuser";
    let password = "validpassword";

    // Hash the password for comparison
    let password_hash = argon2::hash_encoded(
        password.as_bytes(),
        b"somesalt",
        &argon2::Config::default(),
    )
    .expect("Failed to hash password");

    let _mock = ctx.mock_user_lookup(username, &password_hash, &user_id.to_string());

    let result = ctx
        .auth_service
        .authenticate_user(username, password)
        .await;

    assert!(result.is_ok());
    let (token, expires_at) = result.unwrap();
    assert!(!token.is_empty());
    assert!(expires_at > Utc::now());
}

// Test user authentication with invalid username
#[tokio::test]
async fn test_user_authentication_invalid_username() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let username = "nonexistent";
    let password = "password";

    let _mock = ctx.mock_user_not_found(username);

    let result = ctx
        .auth_service
        .authenticate_user(username, password)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError { error, .. } => {
            assert_eq!(error, "AuthenticationError");
        }
    }
}

// Test user authentication with invalid password
#[tokio::test]
async fn test_user_authentication_invalid_password() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();
    let username = "validuser";
    let correct_password = "correctpassword";
    let wrong_password = "wrongpassword";

    let password_hash = argon2::hash_encoded(
        correct_password.as_bytes(),
        b"somesalt",
        &argon2::Config::default(),
    )
    .expect("Failed to hash password");

    let _mock = ctx.mock_user_lookup(username, &password_hash, &user_id.to_string());

    let result = ctx
        .auth_service
        .authenticate_user(username, wrong_password)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError { error, .. } => {
            assert_eq!(error, "AuthenticationError");
        }
    }
}

// Test user profile retrieval
#[tokio::test]
async fn test_get_user_profile() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();
    let username = "profileuser";

    let _mock = ctx.mock_profile_lookup(&user_id.to_string(), username);

    let result = ctx.auth_service.get_user_profile(user_id).await;

    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.id, user_id);
    assert_eq!(profile.username, username);
    assert_eq!(profile.email, format!("{}@test.com", username));
}

// Test user profile retrieval for nonexistent user
#[tokio::test]
async fn test_get_user_profile_not_found() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();

    // Mock returning empty array for nonexistent user
    let _mock = ctx
        .mock_server
        .mock("GET", "/rest/v1/users")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create();

    let result = ctx.auth_service.get_user_profile(user_id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError { error, .. } => {
            assert_eq!(error, "NotFoundError");
        }
    }
}

// Test scope validation
#[tokio::test]
async fn test_scope_validation() {
    let auth_user = AuthUser {
        user_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    // User should have read scope
    assert!(auth_user.scopes.contains(&"read".to_string()));

    // User should have write scope
    assert!(auth_user.scopes.contains(&"write".to_string()));

    // User should not have admin scope
    assert!(!auth_user.scopes.contains(&"admin".to_string()));
}

// Test scope-based authorization
#[tokio::test]
async fn test_scope_based_authorization() {
    let read_only_user = AuthUser {
        user_id: Uuid::new_v4(),
        username: "readonly".to_string(),
        email: "readonly@example.com".to_string(),
        scopes: vec!["read".to_string()],
    };

    let read_write_user = AuthUser {
        user_id: Uuid::new_v4(),
        username: "readwrite".to_string(),
        email: "readwrite@example.com".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    // Simulate authorization check for write operation
    let write_required = "write";

    // Read-only user should not be authorized for write operations
    assert!(!read_only_user.scopes.contains(&write_required.to_string()));

    // Read-write user should be authorized for write operations
    assert!(read_write_user.scopes.contains(&write_required.to_string()));
}

// Test password hashing and verification
#[test]
fn test_password_hashing() {
    let password = "testpassword123";
    let salt = b"testsalt";

    // Hash the password
    let hash1 = argon2::hash_encoded(password.as_bytes(), salt, &argon2::Config::default())
        .expect("Failed to hash password");

    let hash2 = argon2::hash_encoded(password.as_bytes(), salt, &argon2::Config::default())
        .expect("Failed to hash password");

    // Same password and salt should produce same hash
    assert_eq!(hash1, hash2);

    // Verify password against hash
    let verification_result = argon2::verify_encoded(&hash1, password.as_bytes());
    assert!(verification_result.is_ok());
    assert!(verification_result.unwrap());

    // Wrong password should fail verification
    let wrong_password = "wrongpassword";
    let wrong_verification = argon2::verify_encoded(&hash1, wrong_password.as_bytes());
    assert!(wrong_verification.is_ok());
    assert!(!wrong_verification.unwrap());
}

// Test different salt produces different hash
#[test]
fn test_password_salt_randomness() {
    let password = "testpassword123";
    let salt1 = b"salt1234";
    let salt2 = b"salt5678";

    let hash1 = argon2::hash_encoded(password.as_bytes(), salt1, &argon2::Config::default())
        .expect("Failed to hash password");

    let hash2 = argon2::hash_encoded(password.as_bytes(), salt2, &argon2::Config::default())
        .expect("Failed to hash password");

    // Different salts should produce different hashes
    assert_ne!(hash1, hash2);

    // Both should verify correctly with their respective passwords
    assert!(argon2::verify_encoded(&hash1, password.as_bytes()).unwrap());
    assert!(argon2::verify_encoded(&hash2, password.as_bytes()).unwrap());
}

// Test AuthUser serialization/deserialization
#[test]
fn test_auth_user_serialization() {
    let auth_user = AuthUser {
        user_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    // This tests that AuthUser can be serialized for session storage, etc.
    let json = serde_json::to_string(&auth_user);
    assert!(json.is_ok());

    if let Ok(json_str) = json {
        let deserialized: Result<AuthUser, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_user = deserialized.unwrap();
        assert_eq!(auth_user.user_id, deserialized_user.user_id);
        assert_eq!(auth_user.username, deserialized_user.username);
        assert_eq!(auth_user.email, deserialized_user.email);
        assert_eq!(auth_user.scopes, deserialized_user.scopes);
    }
}

// Test authentication request validation
#[test]
fn test_auth_request_validation() {
    use validator::Validate;

    // Valid request
    let valid_request = AuthRequest {
        username: "validuser".to_string(),
        password: "validpassword".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Empty username
    let invalid_username = AuthRequest {
        username: "".to_string(),
        password: "validpassword".to_string(),
    };
    assert!(invalid_username.validate().is_err());

    // Empty password
    let invalid_password = AuthRequest {
        username: "validuser".to_string(),
        password: "".to_string(),
    };
    assert!(invalid_password.validate().is_err());

    // Both empty
    let both_empty = AuthRequest {
        username: "".to_string(),
        password: "".to_string(),
    };
    assert!(both_empty.validate().is_err());
}

// Test JWT with different algorithms (security test)
#[test]
fn test_jwt_algorithm_security() {
    let secret = "test-secret-key";
    let user_id = Uuid::new_v4();

    // Create token with HS256
    let claims = TestClaims {
        sub: user_id.to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        scopes: vec!["read".to_string()],
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to create token");

    // Should validate with HS256
    let validation_hs256 = Validation::new(Algorithm::HS256);
    let result_hs256 = decode::<TestClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation_hs256,
    );
    assert!(result_hs256.is_ok());

    // Should NOT validate with different algorithm
    let validation_hs512 = Validation::new(Algorithm::HS512);
    let result_hs512 = decode::<TestClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation_hs512,
    );
    assert!(result_hs512.is_err());
}

// Test concurrent authentication attempts
#[tokio::test]
async fn test_concurrent_authentication() {
    let mut ctx = auth_test_utils::AuthTestContext::new().await;
    let user_id = Uuid::new_v4();
    let username = "concurrentuser";
    let password = "password123";

    let password_hash = argon2::hash_encoded(
        password.as_bytes(),
        b"testsalt",
        &argon2::Config::default(),
    )
    .expect("Failed to hash password");

    // Mock multiple successful responses
    let _mock = ctx
        .mock_server
        .mock("GET", "/rest/v1/users")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!([{
                "id": user_id.to_string(),
                "username": username,
                "email": format!("{}@test.com", username),
                "password_hash": password_hash,
                "created_at": "2024-01-01T00:00:00Z"
            }])
            .to_string(),
        )
        .expect(3) // Expect 3 calls
        .create();

    // Run multiple authentication attempts concurrently
    let auth_service = ctx.auth_service.clone();
    let futures = (0..3).map(|_| {
        let auth_service = auth_service.clone();
        tokio::spawn(async move {
            auth_service
                .authenticate_user(username, password)
                .await
        })
    });

    let results = futures::future::join_all(futures).await;

    // All authentication attempts should succeed
    for result in results {
        let auth_result = result.expect("Task should complete");
        assert!(auth_result.is_ok());
        let (token, expires_at) = auth_result.unwrap();
        assert!(!token.is_empty());
        assert!(expires_at > Utc::now());
    }
}