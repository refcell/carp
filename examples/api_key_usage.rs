// Example Rust code for working with API keys securely
// This demonstrates the proper patterns for generating, storing, and validating API keys

use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Represents an API key with its components
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub full_key: String,   // The complete key (never store this!)
    pub key_hash: String,   // SHA-256 hash for database storage
    pub key_prefix: String, // First 8+ chars for identification
}

/// Generate a new API key with proper format and security
/// Compatible with both original and enhanced API key formats
pub fn generate_api_key() -> ApiKey {
    // Generate random bytes for the key
    let mut rng = rand::thread_rng();

    // Generate 3 parts of 8 characters each (similar to original format but more secure)
    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    let part1: String = (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect();
    let part2: String = (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect();
    let part3: String = (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect();

    // Create the full key with format compatible with original migration
    let full_key = format!("carp_{}_{}_{}", part1, part2, part3);

    // Generate SHA-256 hash for database storage
    let mut hasher = Sha256::new();
    hasher.update(full_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    // Extract prefix for identification (first 13 chars to match original format)
    let key_prefix = full_key.chars().take(13).collect::<String>();

    ApiKey {
        full_key,
        key_hash,
        key_prefix,
    }
}

/// Hash an existing API key for verification
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Database operations for API keys (using postgrest client)
pub struct ApiKeyManager {
    client: postgrest::Postgrest,
}

impl ApiKeyManager {
    pub fn new(supabase_url: &str, supabase_key: &str) -> Self {
        let client = postgrest::Postgrest::new(supabase_url).insert_header("apikey", supabase_key);

        Self { client }
    }

    /// Create a new API key for a user
    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        name: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        auth_token: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let api_key = generate_api_key();

        let payload = serde_json::json!({
            "user_id": user_id,
            "name": name,
            "key_hash": api_key.key_hash,
            "key_prefix": api_key.key_prefix,
            "expires_at": expires_at,
        });

        let response = self
            .client
            .from("api_keys")
            .auth(auth_token)
            .insert(payload.to_string())
            .execute()
            .await?;

        if response.status().is_success() {
            // Return the full key (only time it's ever exposed) and prefix
            Ok((api_key.full_key, api_key.key_prefix))
        } else {
            Err(format!("Failed to create API key: {}", response.status()).into())
        }
    }

    /// Verify an API key and return user info if valid
    /// Works with both original validate_api_key and enhanced verify_api_key functions
    pub async fn verify_api_key(
        &self,
        key: &str,
    ) -> Result<Option<ApiKeyInfo>, Box<dyn std::error::Error>> {
        let key_hash = hash_api_key(key);

        // Try the enhanced function first, fall back to original if not available
        let mut response = self
            .client
            .rpc(
                "verify_api_key",
                format!(r#"{{"key_hash_param": "{}"}}"#, key_hash),
            )
            .execute()
            .await;

        // If enhanced function doesn't exist, try the original function
        if response.is_err() || !response.as_ref().unwrap().status().is_success() {
            response = self
                .client
                .rpc(
                    "validate_api_key",
                    format!(r#"{{"api_key_hash": "{}"}}"#, key_hash),
                )
                .execute()
                .await;
        }

        if let Ok(resp) = response {
            if resp.status().is_success() {
                let body = resp.text().await?;
                let results: Vec<ApiKeyInfo> = serde_json::from_str(&body)?;

                if let Some(info) = results.first() {
                    if info.is_valid {
                        // Update last_used_at timestamp
                        let _ = self.update_last_used(&key_hash).await;
                        return Ok(Some(info.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Update the last_used_at timestamp for an API key
    async fn update_last_used(&self, key_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .client
            .rpc(
                "update_api_key_last_used",
                format!(r#"{{"key_hash_param": "{}"}}"#, key_hash),
            )
            .execute()
            .await?;

        Ok(())
    }

    /// List API keys for a user (excluding sensitive data)
    pub async fn list_user_api_keys(
        &self,
        auth_token: &str,
    ) -> Result<Vec<ApiKeyListItem>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .from("api_keys")
            .auth(auth_token)
            .select("id,name,key_prefix,is_active,last_used_at,expires_at,created_at")
            .order("created_at.desc")
            .execute()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            let keys: Vec<ApiKeyListItem> = serde_json::from_str(&body)?;
            Ok(keys)
        } else {
            Err(format!("Failed to list API keys: {}", response.status()).into())
        }
    }

    /// Deactivate an API key
    pub async fn deactivate_api_key(
        &self,
        key_id: Uuid,
        auth_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "is_active": false
        });

        let response = self
            .client
            .from("api_keys")
            .auth(auth_token)
            .eq("id", key_id.to_string())
            .update(payload.to_string())
            .execute()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to deactivate API key: {}", response.status()).into())
        }
    }
}

/// Information returned when verifying an API key
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiKeyInfo {
    pub user_id: Option<Uuid>,
    pub key_id: Option<Uuid>,
    pub scopes: Option<Vec<String>>,
    pub is_valid: bool,
}

/// API key information for listing (excludes sensitive data)
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiKeyListItem {
    pub id: Uuid,
    pub name: Option<String>,
    pub key_prefix: String,
    pub is_active: bool,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Middleware for API key authentication
pub async fn authenticate_api_key(
    key: &str,
    api_key_manager: &ApiKeyManager,
) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
    if let Some(key_info) = api_key_manager.verify_api_key(key).await? {
        Ok(key_info.user_id)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();

        // Test key format (compatible with original migration format)
        assert!(key.full_key.starts_with("carp_"));
        assert_eq!(key.full_key.len(), 32); // "carp_" + 8 + "_" + 8 + "_" + 8 = 32 chars

        // Test prefix
        assert!(key.key_prefix.starts_with("carp_"));
        assert_eq!(key.key_prefix.len(), 13); // First 13 chars

        // Test hash
        assert_eq!(key.key_hash.len(), 64); // SHA-256 hex string

        // Verify hash is correct
        let expected_hash = hash_api_key(&key.full_key);
        assert_eq!(key.key_hash, expected_hash);
    }

    #[test]
    fn test_hash_consistency() {
        let key = "carp_test1234_test5678_test9012";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_multiple_keys_unique() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();

        assert_ne!(key1.full_key, key2.full_key);
        assert_ne!(key1.key_hash, key2.key_hash);
        assert_ne!(key1.key_prefix, key2.key_prefix);
    }
}

// Example usage in a web handler
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the API key manager
    let api_key_manager = ApiKeyManager::new("https://your-project.supabase.co", "your-anon-key");

    // Example: Create a new API key for a user
    let user_id = Uuid::new_v4();
    let (full_key, prefix) = api_key_manager
        .create_api_key(
            user_id,
            Some("My CLI Key".to_string()),
            None, // No expiration
            "user-auth-token",
        )
        .await?;

    println!("Generated API key: {}", full_key);
    println!("Key prefix: {}", prefix);
    println!("⚠️  Store this key securely - it won't be shown again!");

    // Example: Verify the API key
    if let Some(user_id) = authenticate_api_key(&full_key, &api_key_manager).await? {
        println!("API key is valid for user: {}", user_id);
    } else {
        println!("Invalid API key");
    }

    Ok(())
}
