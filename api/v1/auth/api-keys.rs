use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use uuid::Uuid;
use vercel_runtime::{run, Body, Error, Request, Response};

// Use shared authentication module
use shared::{
    api_key_middleware, jwt_middleware, require_scope,
    ApiError, AuthenticatedUser, AuthMethod
};








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
    // Route based on HTTP method and use appropriate authentication strategy
    match req.method().as_str() {
        "POST" => {
            // For creating API keys, use JWT authentication only (frontend users)
            let authenticated_user = match jwt_middleware(&req).await {
                Ok(user) => user,
                Err(error_response) => return Ok(error_response),
            };
            
            // Ensure user has API key creation scope
            if let Err(error_response) = require_scope(&authenticated_user, "api_key_create") {
                return Ok(error_response);
            }
            
            create_api_key(&req, &authenticated_user).await
        }
        "GET" | "PUT" | "PATCH" | "DELETE" => {
            // For API key management operations, use API key authentication
            let authenticated_user = match api_key_middleware(&req).await {
                Ok(user) => user,
                Err(error_response) => return Ok(error_response),
            };
            
            // Ensure user has API key management scope
            if let Err(error_response) = require_scope(&authenticated_user, "api_key_manage") {
                return Ok(error_response);
            }

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

async fn list_api_keys(authenticated_user: &AuthenticatedUser) -> Result<Response<Body>, Error> {
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock data for development
        let mock_keys = vec![ApiKeyInfo {
            id: Uuid::new_v4(),
            name: "Development Key".to_string(),
            prefix: "carp_dev".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
            is_active: true,
            last_used_at: Some(Utc::now()),
            expires_at: None,
            created_at: Utc::now(),
        }];

        return Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&mock_keys)?.into())?);
    }

    let client = reqwest::Client::new();

    // Query user's API keys
    let response = client
        .get(format!("{supabase_url}/rest/v1/api_keys"))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .header("Content-Type", "application/json")
        .query(&[("user_id", format!("eq.{}", authenticated_user.user_id))])
        .query(&[(
            "select",
            "id,name,prefix,scopes,is_active,last_used_at,expires_at,created_at",
        )])
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
                message: format!("Invalid JSON in request body: {e}"),
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
                message: format!(
                    "Invalid scope: {}. Valid scopes are: {}",
                    scope,
                    valid_scopes.join(", ")
                ),
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
    let key_hash = shared::hash_api_key(&api_key);
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
        .post(format!("{supabase_url}/rest/v1/api_keys"))
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
            message: format!("Failed to create API key: {error_text}"),
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
    _authenticated_user: &AuthenticatedUser,
) -> Result<Response<Body>, Error> {
    // Extract key ID from query parameters
    let query = req.uri().query().unwrap_or("");
    let query_params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    let _key_id = match query_params.get("id") {
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

    let _update_request: UpdateApiKeyRequest = match serde_json::from_str(body_str) {
        Ok(req) => req,
        Err(e) => {
            let error = ApiError {
                error: "bad_request".to_string(),
                message: format!("Invalid JSON in request body: {e}"),
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
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    // Extract key ID from authenticated user context
    let key_id = match &authenticated_user.auth_method {
        AuthMethod::ApiKey { key_id } => *key_id,
        _ => {
            let error = ApiError {
                error: "invalid_auth_method".to_string(),
                message: "API key management requires API key authentication".to_string(),
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
        return Ok(Response::builder().status(204).body("".into())?);
    }

    let client = reqwest::Client::new();

    // Delete the API key (only if owned by the user)
    let response = client
        .delete(format!("{supabase_url}/rest/v1/api_keys"))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .query(&[("id", format!("eq.{key_id}"))])
        .query(&[("user_id", format!("eq.{}", authenticated_user.user_id))])
        .send()
        .await?;

    if response.status().is_success() {
        Ok(Response::builder().status(204).body("".into())?)
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

    let part1: String = (0..8)
        .map(|_| {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();

    let part2: String = (0..8)
        .map(|_| {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();

    let part3: String = (0..8)
        .map(|_| {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();

    format!("carp_{part1}_{part2}_{part3}")
}
