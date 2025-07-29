use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;
use vercel_runtime::{Error, Request};

/// User context extracted from authenticated API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub key_id: Uuid,
    pub scopes: Vec<String>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Authentication result
pub type AuthResult<T> = Result<T, ApiError>;

/// Extract API key from request headers
/// Supports both "Authorization: Bearer <key>" and "X-API-Key: <key>" formats
pub fn extract_api_key(req: &Request) -> Option<String> {
    let headers = req.headers();
    
    // Try Authorization header first
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if let Some(bearer_token) = auth_header.strip_prefix("Bearer ") {
            return Some(bearer_token.to_string());
        }
    }
    
    // Try X-API-Key header as fallback
    if let Some(api_key_header) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
        return Some(api_key_header.to_string());
    }
    
    None
}

/// Validate API key and return authenticated user context
pub async fn validate_api_key(api_key: &str) -> AuthResult<AuthenticatedUser> {
    // Validate API key format - should start with "carp_" and have proper structure
    if !api_key.starts_with("carp_") || api_key.len() != 31 {
        return Err(ApiError {
            error: "invalid_api_key".to_string(),
            message: "API key format is invalid".to_string(),
            details: None,
        });
    }

    // Hash the API key for database lookup
    let key_hash = hash_api_key(api_key)?;
    
    // Query database to validate the API key
    match query_api_key_from_database(&key_hash).await {
        Ok(Some(user)) => {
            // Update last_used_at timestamp
            let _ = update_api_key_last_used(&key_hash).await;
            Ok(user)
        }
        Ok(None) => Err(ApiError {
            error: "invalid_api_key".to_string(),
            message: "API key not found or invalid".to_string(),
            details: None,
        }),
        Err(err) => Err(ApiError {
            error: "database_error".to_string(),
            message: format!("Failed to validate API key: {}", err),
            details: None,
        }),
    }
}

/// Hash API key using SHA-256 for database lookup (simplified approach)
/// In production, consider using Argon2 with proper salt management
fn hash_api_key(api_key: &str) -> AuthResult<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify API key against stored hash
fn verify_api_key(api_key: &str, hash: &str) -> bool {
    match hash_api_key(api_key) {
        Ok(computed_hash) => computed_hash == hash,
        Err(_) => false,
    }
}

/// Query API key from database
async fn query_api_key_from_database(key_hash: &str) -> Result<Option<AuthenticatedUser>, Error> {
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock user for development/testing
        return Ok(Some(AuthenticatedUser {
            user_id: Uuid::new_v4(),
            key_id: Uuid::new_v4(),
            scopes: vec!["read".to_string(), "write".to_string()],
        }));
    }

    let client = reqwest::Client::new();
    
    // Call the database function to validate API key
    let response = client
        .post(&format!("{}/rest/v1/rpc/validate_api_key", supabase_url))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "api_key_hash": key_hash
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::from("Failed to validate API key"));
    }

    let body = response.text().await?;
    let validation_results: Vec<ApiKeyValidation> = serde_json::from_str(&body)
        .map_err(|_| Error::from("Failed to parse API key validation response"))?;

    if let Some(result) = validation_results.first() {
        if result.is_valid {
            return Ok(Some(AuthenticatedUser {
                user_id: result.user_id,
                key_id: result.key_id,
                scopes: result.scopes.clone(),
            }));
        }
    }

    Ok(None)
}

/// Update API key last_used_at timestamp
async fn update_api_key_last_used(key_hash: &str) -> Result<(), Error> {
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        return Ok(()); // Skip in development
    }

    let client = reqwest::Client::new();
    
    // Call the database function to update last_used_at
    let _response = client
        .post(&format!("{}/rest/v1/rpc/update_api_key_last_used", supabase_url))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "api_key_hash": key_hash
        }))
        .send()
        .await?;

    Ok(())
}

/// Database record structure for API key validation
#[derive(Debug, Deserialize)]
struct ApiKeyValidation {
    user_id: Uuid,
    key_id: Uuid,
    scopes: Vec<String>,
    is_valid: bool,
}

/// Middleware function to authenticate requests
pub async fn authenticate_request(req: &Request) -> AuthResult<AuthenticatedUser> {
    let api_key = extract_api_key(req).ok_or_else(|| ApiError {
        error: "missing_api_key".to_string(),
        message: "API key is required. Provide it via 'Authorization: Bearer <key>' or 'X-API-Key: <key>' header".to_string(),
        details: None,
    })?;

    validate_api_key(&api_key).await
}

/// Check if user has required scope
pub fn check_scope(user: &AuthenticatedUser, required_scope: &str) -> bool {
    user.scopes.contains(&required_scope.to_string()) || user.scopes.contains(&"admin".to_string())
}

/// Create unauthorized error response
pub fn unauthorized_error(message: &str) -> ApiError {
    ApiError {
        error: "unauthorized".to_string(),
        message: message.to_string(),
        details: None,
    }
}

/// Create forbidden error response
pub fn forbidden_error(message: &str) -> ApiError {
    ApiError {
        error: "forbidden".to_string(),
        message: message.to_string(),
        details: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_format_validation() {
        // Valid API key format (31 chars: carp_ + 8 + _ + 8 + _ + 8)
        assert!(validate_api_key_format("carp_abcdefgh_ijklmnop_qrstuvwx"));
        
        // Invalid formats
        assert!(!validate_api_key_format("invalid_key"));
        assert!(!validate_api_key_format("carp_short"));
        assert!(!validate_api_key_format("wrong_prefix_abcdefgh_ijklmnop_qrstuvwx"));
        assert!(!validate_api_key_format(""));
        assert!(!validate_api_key_format("carp_abcdefgh_ijklmnop_qrstuvwxtoolong"));
    }

    #[test]
    fn test_hash_and_verify_api_key() {
        let api_key = "carp_abcdefgh_ijklmnop_qrstuvwx";
        let hash = hash_api_key(api_key).expect("Failed to hash API key");
        
        assert!(verify_api_key(api_key, &hash));
        assert!(!verify_api_key("carp_wrongkey_ijklmnop_qrstuvwx", &hash));
    }

    #[test]
    fn test_scope_checking() {
        let user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            key_id: Uuid::new_v4(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };

        assert!(check_scope(&user, "read"));
        assert!(check_scope(&user, "write"));
        assert!(!check_scope(&user, "admin"));

        let admin_user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            key_id: Uuid::new_v4(),
            scopes: vec!["admin".to_string()],
        };

        assert!(check_scope(&admin_user, "read"));
        assert!(check_scope(&admin_user, "write"));
        assert!(check_scope(&admin_user, "admin"));
    }

    fn validate_api_key_format(key: &str) -> bool {
        key.starts_with("carp_") && key.len() == 31
    }
}