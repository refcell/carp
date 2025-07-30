use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use uuid::Uuid;
use vercel_runtime::Request;

// Re-export common dependencies that auth clients need
pub use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
pub use reqwest;
pub use sha2::{Digest, Sha256};

/// User context extracted from authenticated requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub auth_method: AuthMethod,
    pub scopes: Vec<String>,
    pub metadata: UserMetadata,
}

/// Authentication method used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey { key_id: Uuid },
    JwtToken { provider: String },
}

/// Additional user metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    pub email: Option<String>,
    pub github_username: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

/// JWT claims structure for Supabase tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseJwtClaims {
    pub sub: String, // user ID
    pub aud: String, // audience
    pub exp: i64,    // expiration timestamp
    pub iat: i64,    // issued at timestamp
    pub iss: String, // issuer
    pub email: Option<String>,
    pub phone: Option<String>,
    pub app_metadata: Option<serde_json::Value>,
    pub user_metadata: Option<serde_json::Value>,
    pub role: Option<String>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub supabase_url: String,
    pub supabase_service_role_key: String,
    pub supabase_jwt_secret: String,
    pub debug_mode: bool,
}

impl AuthConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            supabase_url: env::var("SUPABASE_URL").unwrap_or_default(),
            supabase_service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default(),
            supabase_jwt_secret: env::var("SUPABASE_JWT_SECRET").unwrap_or_default(),
            debug_mode: env::var("DEBUG_AUTH").unwrap_or_default() == "true",
        }
    }

    /// Check if running in development mode (no database configured)
    pub fn is_development(&self) -> bool {
        self.supabase_url.is_empty() || self.supabase_service_role_key.is_empty()
    }
}

/// Extract bearer token from request headers
pub fn extract_bearer_token(req: &Request) -> Option<String> {
    let headers = req.headers();

    // Try Authorization header first
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(key_str) = api_key_header.to_str() {
            return Some(key_str.to_string());
        }
    }

    None
}

/// Hash an API key using SHA-256
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validate a Supabase JWT token and extract user information
pub async fn validate_jwt_token(
    token: &str,
    config: &AuthConfig,
) -> Result<SupabaseJwtClaims, ApiError> {
    // For development/testing, allow mock JWT tokens
    if config.supabase_jwt_secret.is_empty() {
        if config.debug_mode {
            eprintln!("DEBUG: Using mock JWT token in development mode");
        }

        // Create a mock claim for development - use a fixed UUID for consistency
        return Ok(SupabaseJwtClaims {
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(), // Fixed dev UUID
            aud: "authenticated".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
            iss: "supabase".to_string(),
            email: Some("dev@example.com".to_string()),
            phone: None,
            app_metadata: None,
            user_metadata: Some(json!({
                "github_username": "dev-user"
            })),
            role: Some("authenticated".to_string()),
        });
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated"]);
    validation.validate_exp = true;

    let decoding_key = DecodingKey::from_secret(config.supabase_jwt_secret.as_bytes());

    let token_data =
        decode::<SupabaseJwtClaims>(token, &decoding_key, &validation).map_err(|e| {
            if config.debug_mode {
                eprintln!("DEBUG: JWT validation failed: {e}");
            }
            ApiError {
                error: "invalid_jwt".to_string(),
                message: format!("Invalid JWT token: {e}"),
                details: Some(json!({
                    "token_format_expected": "Valid Supabase JWT token",
                    "common_causes": [
                        "Token expired",
                        "Invalid signature",
                        "Wrong audience",
                        "Malformed token structure"
                    ]
                })),
            }
        })?;

    // Additional expiration check (belt and suspenders)
    let now = Utc::now().timestamp();
    if token_data.claims.exp < now {
        return Err(ApiError {
            error: "expired_jwt".to_string(),
            message: "JWT token has expired".to_string(),
            details: Some(json!({
                "expired_at": token_data.claims.exp,
                "current_time": now,
                "expired_seconds_ago": now - token_data.claims.exp
            })),
        });
    }

    Ok(token_data.claims)
}

/// Authenticate using JWT token (for frontend/web UI)
pub async fn authenticate_jwt(
    token: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, ApiError> {
    let jwt_claims = validate_jwt_token(token, config).await?;

    // Parse user ID from JWT claims
    let user_id = Uuid::parse_str(&jwt_claims.sub).map_err(|e| ApiError {
        error: "invalid_jwt_user_id".to_string(),
        message: format!("Invalid user ID format in JWT token: {e}"),
        details: Some(json!({
            "provided_user_id": jwt_claims.sub,
            "expected_format": "UUID v4 format (xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx)"
        })),
    })?;

    // Extract metadata
    let github_username = jwt_claims
        .user_metadata
        .as_ref()
        .and_then(|meta| meta.get("github_username"))
        .and_then(|username| username.as_str())
        .map(|s| s.to_string());

    // JWT tokens get specific scopes for frontend operations
    let scopes = vec![
        "read".to_string(),
        "api_key_create".to_string(),
        "api_key_manage".to_string(),
    ];

    Ok(AuthenticatedUser {
        user_id,
        auth_method: AuthMethod::JwtToken {
            provider: "supabase".to_string(),
        },
        scopes,
        metadata: UserMetadata {
            email: jwt_claims.email,
            github_username,
            created_at: Some(Utc::now()), // In production, this would come from the database
        },
    })
}

/// Authenticate using API key (for CLI/API)
pub async fn authenticate_api_key(
    api_key: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, ApiError> {
    let key_hash = hash_api_key(api_key);

    if config.is_development() {
        if config.debug_mode {
            eprintln!("DEBUG: Using mock API key authentication in development mode");
        }

        // Return mock user for development - use consistent UUIDs
        return Ok(AuthenticatedUser {
            user_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            auth_method: AuthMethod::ApiKey {
                key_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            },
            scopes: vec![
                "read".to_string(),
                "write".to_string(),
                "upload".to_string(),
                "publish".to_string(),
                "admin".to_string(),
            ],
            metadata: UserMetadata {
                email: Some("dev@example.com".to_string()),
                github_username: Some("dev-user".to_string()),
                created_at: Some(Utc::now()),
            },
        });
    }

    let client = reqwest::Client::new();

    // Verify API key using the database function
    let response = client
        .post(format!(
            "{}/rest/v1/rpc/validate_api_key",
            config.supabase_url
        ))
        .header("apikey", &config.supabase_service_role_key)
        .header(
            "Authorization",
            format!("Bearer {}", config.supabase_service_role_key),
        )
        .header("Content-Type", "application/json")
        .json(&json!({ "api_key_hash": key_hash }))
        .send()
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: format!("Failed to verify API key: {e}"),
            details: None,
        })?;

    if !response.status().is_success() {
        return Err(ApiError {
            error: "invalid_api_key".to_string(),
            message: "Invalid or expired API key".to_string(),
            details: None,
        });
    }

    let verification_result: serde_json::Value = response.json().await.map_err(|e| ApiError {
        error: "parse_error".to_string(),
        message: format!("Failed to parse verification response: {e}"),
        details: None,
    })?;

    // Extract user info from verification result
    if let Some(result) = verification_result.as_array().and_then(|arr| arr.first()) {
        if let (Some(user_id), Some(key_id), Some(is_valid)) = (
            result.get("user_id").and_then(|v| v.as_str()),
            result.get("key_id").and_then(|v| v.as_str()),
            result.get("is_valid").and_then(|v| v.as_bool()),
        ) {
            if is_valid {
                let scopes = result
                    .get("scopes")
                    .and_then(|s| s.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["read".to_string()]);

                // Extract additional metadata if available
                let email = result
                    .get("user_email")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let github_username = result
                    .get("github_username")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                return Ok(AuthenticatedUser {
                    user_id: Uuid::parse_str(user_id).map_err(|_| ApiError {
                        error: "invalid_user_id".to_string(),
                        message: "Invalid user ID format".to_string(),
                        details: None,
                    })?,
                    auth_method: AuthMethod::ApiKey {
                        key_id: Uuid::parse_str(key_id).map_err(|_| ApiError {
                            error: "invalid_key_id".to_string(),
                            message: "Invalid key ID format".to_string(),
                            details: None,
                        })?,
                    },
                    scopes,
                    metadata: UserMetadata {
                        email,
                        github_username,
                        created_at: None, // Would be populated from database in production
                    },
                });
            }
        }
    }

    Err(ApiError {
        error: "invalid_api_key".to_string(),
        message: "Invalid or expired API key".to_string(),
        details: None,
    })
}

/// Ensure user exists in database (for JWT authentication)
/// This synchronizes GitHub OAuth users with our user table
pub async fn sync_jwt_user(user: &AuthenticatedUser, config: &AuthConfig) -> Result<(), ApiError> {
    if config.is_development() {
        return Ok(()); // Skip in development
    }

    let client = reqwest::Client::new();

    // Check if user exists, create if not
    let user_data = json!({
        "id": user.user_id,
        "email": user.metadata.email,
        "github_username": user.metadata.github_username,
        "created_at": user.metadata.created_at.unwrap_or_else(Utc::now)
    });

    let _response = client
        .post(format!("{}/rest/v1/users", config.supabase_url))
        .header("apikey", &config.supabase_service_role_key)
        .header(
            "Authorization",
            format!("Bearer {}", config.supabase_service_role_key),
        )
        .header("Content-Type", "application/json")
        .header("Prefer", "resolution=merge-duplicates")
        .json(&user_data)
        .send()
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: format!("Failed to sync user: {e}"),
            details: None,
        })?;

    Ok(())
}

/// Check if user has required scope
pub fn check_scope(user: &AuthenticatedUser, required_scope: &str) -> bool {
    user.scopes.contains(&required_scope.to_string()) || user.scopes.contains(&"admin".to_string())
}

/// Determine token type based on content heuristics
pub fn guess_token_type(token: &str) -> TokenType {
    // API keys have a specific format: carp_xxxxxxxx_xxxxxxxx_xxxxxxxx
    if token.starts_with("carp_") && token.matches('_').count() == 3 {
        TokenType::ApiKey
    } else if token.contains('.') && token.len() > 100 {
        // JWTs typically have dots and are longer
        TokenType::Jwt
    } else {
        // Default to JWT for ambiguous cases
        TokenType::Jwt
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    ApiKey,
    Jwt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_token_type() {
        // Test API key detection
        assert_eq!(
            guess_token_type("carp_abc12345_def67890_ghi09876"),
            TokenType::ApiKey
        );

        // Test JWT detection (simplified)
        assert_eq!(
            guess_token_type("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
            TokenType::Jwt
        );

        // Test unknown token defaults to JWT
        assert_eq!(guess_token_type("some_random_token"), TokenType::Jwt);
    }

    #[test]
    fn test_check_scope() {
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

        assert!(check_scope(&user, "read"));
        assert!(check_scope(&user, "write"));
        assert!(!check_scope(&user, "admin"));

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

        assert!(check_scope(&admin_user, "read"));
        assert!(check_scope(&admin_user, "write"));
        assert!(check_scope(&admin_user, "admin"));
    }
}
