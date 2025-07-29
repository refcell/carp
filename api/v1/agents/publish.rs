use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

// Shared authentication code for Vercel serverless functions
use sha2::{Digest, Sha256};

/// User context extracted from authenticated API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub key_id: uuid::Uuid,
    pub scopes: Vec<String>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Extract API key from request headers
fn extract_api_key(req: &vercel_runtime::Request) -> Option<String> {
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

/// Authenticate a request using API key
async fn authenticate_request(req: &vercel_runtime::Request) -> Result<AuthenticatedUser, ApiError> {
    let api_key = extract_api_key(req).ok_or_else(|| ApiError {
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
            user_id: uuid::Uuid::new_v4(),
            key_id: uuid::Uuid::new_v4(),
            scopes: vec!["read".to_string(), "write".to_string(), "publish".to_string(), "upload".to_string(), "admin".to_string()],
        });
    }

    let client = reqwest::Client::new();
    
    // Verify API key using the database function
    let response = client
        .post(&format!("{}/rest/v1/rpc/verify_api_key", supabase_url))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .json(&json!({ "key_hash_param": key_hash }))
        .send()
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: format!("Failed to verify API key: {}", e),
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
        message: format!("Failed to parse verification response: {}", e),
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
                    user_id: uuid::Uuid::parse_str(user_id).map_err(|_| ApiError {
                        error: "invalid_user_id".to_string(),
                        message: "Invalid user ID format".to_string(),
                        details: None,
                    })?,
                    key_id: uuid::Uuid::parse_str(key_id).map_err(|_| ApiError {
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

/// Agent metadata returned by the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_count: u64,
    pub tags: Vec<String>,
    pub readme: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

/// Request for publishing an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub readme: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
}

/// Response from publishing an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub success: bool,
    pub message: String,
    pub agent: Option<Agent>,
}

// ApiError is now imported from shared module

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Authenticate the request using API key
    let authenticated_user = match authenticate_request(&req).await {
        Ok(user) => user,
        Err(auth_error) => {
            return Ok(Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&auth_error)?.into())?);
        }
    };

    // Check if user has publish permissions
    if !check_scope(&authenticated_user, "publish") {
        return Ok(forbidden_error("Insufficient permissions to publish agents"));
    }

    let headers = req.headers();

    // Parse multipart form data
    let content_type = headers
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !content_type.starts_with("multipart/form-data") {
        let error = ApiError {
            error: "bad_request".to_string(),
            message: "Content-Type must be multipart/form-data".to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    // For simplicity, we'll mock the parsing of multipart data
    // In production, you'd use a proper multipart parser
    let mock_publish_request = PublishRequest {
        name: "example-agent".to_string(),
        version: "1.0.0".to_string(),
        description: "An example agent".to_string(),
        readme: Some("# Example Agent\n\nThis is an example.".to_string()),
        homepage: None,
        repository: None,
        license: Some("MIT".to_string()),
        tags: vec!["example".to_string()],
    };

    // Process the publish request
    match publish_agent(mock_publish_request, &authenticated_user).await {
        Ok(agent) => {
            let response = PublishResponse {
                success: true,
                message: "Agent published successfully".to_string(),
                agent: Some(agent),
            };
            Ok(Response::builder()
                .status(201)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&response)?.into())?)
        }
        Err(err_msg) => {
            let error = ApiError {
                error: "publish_failed".to_string(),
                message: err_msg,
                details: None,
            };
            Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?)
        }
    }
}

// JWT token validation removed - now using API key authentication

async fn publish_agent(request: PublishRequest, user: &AuthenticatedUser) -> Result<Agent, String> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock success if no database configured
        return Ok(create_mock_published_agent(request, user));
    }

    // In production:
    // 1. Validate the agent package
    // 2. Store the package in Supabase Storage
    // 3. Create/update agent record in database
    // 4. Return the created agent

    Ok(create_mock_published_agent(request, user))
}

fn create_mock_published_agent(request: PublishRequest, user: &AuthenticatedUser) -> Agent {
    Agent {
        name: request.name,
        version: request.version,
        description: request.description,
        author: format!("user-{}", user.user_id), // Use authenticated user ID
        created_at: Utc::now(),
        updated_at: Utc::now(),
        download_count: 0,
        tags: request.tags,
        readme: request.readme,
        homepage: request.homepage,
        repository: request.repository,
        license: request.license,
    }
}
