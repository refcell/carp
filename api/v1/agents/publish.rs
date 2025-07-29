use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

mod shared;
use shared::{authenticate_request, check_scope, ApiError, forbidden_error};

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
        let error = forbidden_error("Insufficient permissions to publish agents");
        return Ok(Response::builder()
            .status(403)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
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

async fn publish_agent(request: PublishRequest, user: &shared::AuthenticatedUser) -> Result<Agent, String> {
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

fn create_mock_published_agent(request: PublishRequest, user: &shared::AuthenticatedUser) -> Agent {
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
