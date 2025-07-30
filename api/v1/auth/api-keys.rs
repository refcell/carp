use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use uuid::Uuid;
use vercel_runtime::{run, Body, Error, Request, Response};

// Since this is a Vercel serverless function, include auth functions directly
use sha2::{Digest, Sha256};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

/// User context extracted from authenticated API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub key_id: Uuid,
    pub scopes: Vec<String>,
}

/// JWT claims structure for Supabase tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseJwtClaims {
    pub sub: String,  // user ID
    pub aud: String,  // audience
    pub exp: i64,     // expiration timestamp
    pub iat: i64,     // issued at timestamp
    pub iss: String,  // issuer
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

/// Extract bearer token from request headers (API key or JWT token)
fn extract_bearer_token(req: &Request) -> Option<String> {
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
fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validate a Supabase JWT token and extract user information
async fn validate_jwt_token(token: &str) -> Result<SupabaseJwtClaims, ApiError> {
    let jwt_secret = env::var("SUPABASE_JWT_SECRET").unwrap_or_default();
    
    // For development/testing, allow mock JWT tokens
    if jwt_secret.is_empty() {
        // Create a mock claim for development
        return Ok(SupabaseJwtClaims {
            sub: Uuid::new_v4().to_string(),
            aud: "authenticated".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
            iss: "supabase".to_string(),
            email: Some("dev@example.com".to_string()),
            phone: None,
            app_metadata: None,
            user_metadata: None,
            role: Some("authenticated".to_string()),
        });
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated"]);
    
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    
    let token_data = decode::<SupabaseJwtClaims>(token, &decoding_key, &validation)
        .map_err(|e| ApiError {
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
        })?;

    // Check if token is expired
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

/// Authenticate a request using API key
async fn authenticate_request(req: &Request) -> Result<AuthenticatedUser, ApiError> {
    let api_key = extract_bearer_token(req).ok_or_else(|| ApiError {
        error: "missing_api_key".to_string(),
        message: "API key is required".to_string(),
        details: None,
    })?;

    let key_hash = hash_api_key(&api_key);
    
    // Get database credentials
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock user for development
        return Ok(AuthenticatedUser {
            user_id: Uuid::new_v4(),
            key_id: Uuid::new_v4(),
            scopes: vec!["read".to_string(), "write".to_string(), "admin".to_string()],
        });
    }

    let client = reqwest::Client::new();
    
    // Verify API key using the database function
    let response = client
        .post(format!("{supabase_url}/rest/v1/rpc/verify_api_key"))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .header("Content-Type", "application/json")
        .json(&json!({ "key_hash_param": key_hash }))
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

                return Ok(AuthenticatedUser {
                    user_id: Uuid::parse_str(user_id).map_err(|_| ApiError {
                        error: "invalid_user_id".to_string(),
                        message: "Invalid user ID format".to_string(),
                        details: None,
                    })?,
                    key_id: Uuid::parse_str(key_id).map_err(|_| ApiError {
                        error: "invalid_key_id".to_string(),
                        message: "Invalid key ID format".to_string(),
                        details: None,
                    })?,
                    scopes,
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

/// Bootstrap authenticate a request using either API key or JWT token
/// This allows initial API key creation using Supabase JWT tokens
async fn bootstrap_authenticate_request(req: &Request) -> Result<AuthenticatedUser, ApiError> {
    // Extract token once to avoid duplicate processing
    let token = extract_bearer_token(req).ok_or_else(|| ApiError {
        error: "missing_authentication".to_string(),
        message: "Authentication required: provide either a valid API key or Supabase JWT token".to_string(),
        details: Some(json!({
            "accepted_auth_methods": ["api_key", "jwt_token"],
            "header_formats": [
                "Authorization: Bearer <api_key>",
                "Authorization: Bearer <jwt_token>",
                "X-API-Key: <api_key>"
            ]
        })),
    })?;

    // First try to authenticate with API key (existing method)
    match authenticate_request(req).await {
        Ok(user) => {
            // Successfully authenticated with API key
            return Ok(user);
        }
        Err(api_key_error) => {
            // Log the API key authentication failure for debugging
            if env::var("DEBUG_AUTH").unwrap_or_default() == "true" {
                eprintln!("API key authentication failed: {:?}", api_key_error);
            }
            // Continue to try JWT authentication instead of failing here
        }
    }

    // Validate JWT token with better error context
    let jwt_claims = match validate_jwt_token(&token).await {
        Ok(claims) => claims,
        Err(jwt_error) => {
            return Err(ApiError {
                error: "authentication_failed".to_string(),
                message: "Neither API key nor JWT token authentication succeeded".to_string(),
                details: Some(json!({
                    "jwt_error": jwt_error.message,
                    "help": "Ensure you're using a valid Supabase JWT token for initial API key creation, or a valid API key for subsequent operations"
                })),
            });
        }
    };
    
    // Parse user ID from JWT claims with better error handling
    let user_id = Uuid::parse_str(&jwt_claims.sub).map_err(|e| ApiError {
        error: "invalid_jwt_user_id".to_string(),
        message: format!("Invalid user ID format in JWT token: {}", e),
        details: Some(json!({
            "provided_user_id": jwt_claims.sub,
            "expected_format": "UUID v4 format (xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx)"
        })),
    })?;

    // For JWT-based authentication, we create a synthetic AuthenticatedUser
    // with bootstrap scopes that allow API key creation
    Ok(AuthenticatedUser {
        user_id,
        key_id: Uuid::new_v4(), // Synthetic key ID for JWT authentication
        scopes: vec!["bootstrap".to_string(), "read".to_string(), "write".to_string()],
    })
}

/// Check if user has required scope
fn check_scope(user: &AuthenticatedUser, required_scope: &str) -> bool {
    user.scopes.contains(&required_scope.to_string()) || user.scopes.contains(&"admin".to_string())
}

/// Create a 403 forbidden error response
fn forbidden_error(message: &str) -> Response<Body> {
    Response::builder()
        .status(403)
        .header("content-type", "application/json")
        .body(json!({
            "error": "Forbidden",
            "message": message
        }).to_string().into())
        .unwrap()
}

/// API key information (without the actual key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new API key
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response when creating a new API key
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub key: String, // Only returned once during creation
    pub info: ApiKeyInfo,
}

/// Request to update an API key
#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Route based on HTTP method and use appropriate authentication
    match req.method().as_str() {
        "POST" => {
            // For creating API keys, use bootstrap authentication (accepts both API key and JWT)
            let authenticated_user = match bootstrap_authenticate_request(&req).await {
                Ok(user) => user,
                Err(auth_error) => {
                    return Ok(Response::builder()
                        .status(401)
                        .header("content-type", "application/json")
                        .body(serde_json::to_string(&auth_error)?.into())?);
                }
            };
            create_api_key(&req, &authenticated_user).await
        }
        "GET" | "PUT" | "PATCH" | "DELETE" => {
            // For all other operations, use regular API key authentication
            let authenticated_user = match authenticate_request(&req).await {
                Ok(user) => user,
                Err(auth_error) => {
                    return Ok(Response::builder()
                        .status(401)
                        .header("content-type", "application/json")
                        .body(serde_json::to_string(&auth_error)?.into())?);
                }
            };
            
            match req.method().as_str() {
                "GET" => list_api_keys(&authenticated_user).await,
                "PUT" | "PATCH" => update_api_key(&req, &authenticated_user).await,
                "DELETE" => delete_api_key(&req, &authenticated_user).await,
                _ => unreachable!(), // We already matched these methods above
            }
        }
        _ => {
            let error = ApiError {
                error: "method_not_allowed".to_string(),
                message: "Method not allowed".to_string(),
                details: None,
            };
            Ok(Response::builder()
                .status(405)
                .header("content-type", "application/json")
                .header("allow", "GET, POST, PUT, PATCH, DELETE")
                .body(serde_json::to_string(&error)?.into())?)
        }
    }
}

async fn list_api_keys(
    authenticated_user: &AuthenticatedUser,
) -> Result<Response<Body>, Error> {
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock data for development
        let mock_keys = vec![
            ApiKeyInfo {
                id: Uuid::new_v4(),
                name: "Development Key".to_string(),
                prefix: "carp_dev".to_string(),
                scopes: vec!["read".to_string(), "write".to_string()],
                is_active: true,
                last_used_at: Some(Utc::now()),
                expires_at: None,
                created_at: Utc::now(),
            }
        ];
        
        return Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&mock_keys)?.into())?);
    }

    let client = reqwest::Client::new();
    
    // Query user's API keys
    let response = client
        .get(&format!("{}/rest/v1/api_keys", supabase_url))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .header("Content-Type", "application/json")
        .query(&[("user_id", format!("eq.{}", authenticated_user.user_id))])
        .query(&[("select", "id,name,prefix,scopes,is_active,last_used_at,expires_at,created_at")])
        .send()
        .await?;

    if !response.status().is_success() {
        let error = ApiError {
            error: "database_error".to_string(),
            message: "Failed to retrieve API keys".to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    let body = response.text().await?;
    let api_keys: Vec<ApiKeyInfo> = serde_json::from_str(&body)
        .map_err(|_| Error::from("Failed to parse API keys response"))?;

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&api_keys)?.into())?)
}

async fn create_api_key(
    req: &Request,
    authenticated_user: &AuthenticatedUser,
) -> Result<Response<Body>, Error> {
    // Parse request body
    let body_bytes = req.body();
    let body_str = std::str::from_utf8(body_bytes)
        .map_err(|_| Error::from("Invalid UTF-8 in request body"))?;

    let create_request: CreateApiKeyRequest = match serde_json::from_str(body_str) {
        Ok(req) => req,
        Err(e) => {
            let error = ApiError {
                error: "bad_request".to_string(),
                message: format!("Invalid JSON in request body: {}", e),
                details: None,
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?);
        }
    };

    // Validate scopes
    let valid_scopes = ["read", "write", "upload", "publish", "delete", "admin"];
    for scope in &create_request.scopes {
        if !valid_scopes.contains(&scope.as_str()) {
            let error = ApiError {
                error: "invalid_scope".to_string(),
                message: format!("Invalid scope: {}. Valid scopes are: {}", scope, valid_scopes.join(", ")),
                details: None,
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?);
        }
    }

    // Generate new API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    let prefix = api_key.chars().take(12).collect::<String>(); // "carp_xxxxxxxx"
    
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock response for development
        let mock_info = ApiKeyInfo {
            id: Uuid::new_v4(),
            name: create_request.name,
            prefix,
            scopes: create_request.scopes,
            is_active: true,
            last_used_at: None,
            expires_at: create_request.expires_at,
            created_at: Utc::now(),
        };
        
        let response = CreateApiKeyResponse {
            key: api_key,
            info: mock_info,
        };
        
        return Ok(Response::builder()
            .status(201)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&response)?.into())?);
    }

    let client = reqwest::Client::new();
    
    // Insert new API key into database
    let insert_data = json!({
        "user_id": authenticated_user.user_id,
        "name": create_request.name,
        "key_hash": key_hash,
        "key_prefix": prefix,
        "scopes": create_request.scopes,
        "expires_at": create_request.expires_at
    });

    let response = client
        .post(&format!("{}/rest/v1/api_keys", supabase_url))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&insert_data)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        let error = ApiError {
            error: "database_error".to_string(),
            message: format!("Failed to create API key: {}", error_text),
            details: None,
        };
        return Ok(Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    let body = response.text().await?;
    let created_keys: Vec<ApiKeyInfo> = serde_json::from_str(&body)
        .map_err(|_| Error::from("Failed to parse created API key response"))?;

    if let Some(key_info) = created_keys.first() {
        let response = CreateApiKeyResponse {
            key: api_key,
            info: key_info.clone(),
        };
        
        Ok(Response::builder()
            .status(201)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&response)?.into())?)
    } else {
        let error = ApiError {
            error: "creation_failed".to_string(),
            message: "API key creation failed".to_string(),
            details: None,
        };
        Ok(Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?)
    }
}

async fn update_api_key(
    req: &Request,
    authenticated_user: &AuthenticatedUser,
) -> Result<Response<Body>, Error> {
    // Extract key ID from query parameters
    let query = req.uri().query().unwrap_or("");
    let query_params: std::collections::HashMap<String, String> = 
        url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    
    let key_id = match query_params.get("id") {
        Some(id) => match Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => {
                let error = ApiError {
                    error: "invalid_id".to_string(),
                    message: "Invalid API key ID format".to_string(),
                    details: None,
                };
                return Ok(Response::builder()
                    .status(400)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&error)?.into())?);
            }
        },
        None => {
            let error = ApiError {
                error: "missing_id".to_string(),
                message: "API key ID is required in query parameters".to_string(),
                details: None,
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?);
        }
    };

    // Parse request body
    let body_bytes = req.body();
    let body_str = std::str::from_utf8(body_bytes)
        .map_err(|_| Error::from("Invalid UTF-8 in request body"))?;

    let update_request: UpdateApiKeyRequest = match serde_json::from_str(body_str) {
        Ok(req) => req,
        Err(e) => {
            let error = ApiError {
                error: "bad_request".to_string(),
                message: format!("Invalid JSON in request body: {}", e),
                details: None,
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?);
        }
    };

    // TODO: Implement API key update logic
    let error = ApiError {
        error: "not_implemented".to_string(),
        message: "API key update not yet implemented".to_string(),
        details: None,
    };
    Ok(Response::builder()
        .status(501)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&error)?.into())?)
}

async fn delete_api_key(
    req: &Request,
    authenticated_user: &AuthenticatedUser,
) -> Result<Response<Body>, Error> {
    // Extract key ID from query parameters
    let query = req.uri().query().unwrap_or("");
    let query_params: std::collections::HashMap<String, String> = 
        url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    
    let key_id = match query_params.get("id") {
        Some(id) => match Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => {
                let error = ApiError {
                    error: "invalid_id".to_string(),
                    message: "Invalid API key ID format".to_string(),
                    details: None,
                };
                return Ok(Response::builder()
                    .status(400)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&error)?.into())?);
            }
        },
        None => {
            let error = ApiError {
                error: "missing_id".to_string(),
                message: "API key ID is required in query parameters".to_string(),
                details: None,
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?);
        }
    };

    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return success for development
        return Ok(Response::builder()
            .status(204)
            .body("".into())?);
    }

    let client = reqwest::Client::new();
    
    // Delete the API key (only if owned by the user)
    let response = client
        .delete(&format!("{}/rest/v1/api_keys", supabase_url))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .query(&[("id", format!("eq.{}", key_id))])
        .query(&[("user_id", format!("eq.{}", authenticated_user.user_id))])
        .send()
        .await?;

    if response.status().is_success() {
        Ok(Response::builder()
            .status(204)
            .body("".into())?)
    } else {
        let error = ApiError {
            error: "deletion_failed".to_string(),
            message: "Failed to delete API key or key not found".to_string(),
            details: None,
        };
        Ok(Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?)
    }
}

/// Generate a new API key with the format "carp_xxxxxxxx_xxxxxxxx_xxxxxxxx"
fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let part1: String = (0..8).map(|_| {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        chars[rng.gen_range(0..chars.len())] as char
    }).collect();
    
    let part2: String = (0..8).map(|_| {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        chars[rng.gen_range(0..chars.len())] as char
    }).collect();
    
    let part3: String = (0..8).map(|_| {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        chars[rng.gen_range(0..chars.len())] as char
    }).collect();
    
    format!("carp_{}_{}_{}",part1, part2, part3)
}

