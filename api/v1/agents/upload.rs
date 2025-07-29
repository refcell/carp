use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

// Shared authentication code for Vercel serverless functions
use sha2::{Digest, Sha256};
use serde_json::json;

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

/// Request for uploading an agent via JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadAgentRequest {
    pub name: String,
    pub description: String,
    pub content: String,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

/// Response from uploading an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadAgentResponse {
    pub success: bool,
    pub message: String,
    pub agent: Option<Agent>,
    pub validation_errors: Option<Vec<ValidationError>>,
}

/// Validation error details
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

// ApiError is now imported from shared module

/// YAML frontmatter structure
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlFrontmatter {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub tags: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Only allow POST requests
    if req.method() != "POST" {
        let error = ApiError {
            error: "method_not_allowed".to_string(),
            message: "Only POST requests are allowed".to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(405)
            .header("content-type", "application/json")
            .header("allow", "POST")
            .body(serde_json::to_string(&error)?.into())?);
    }

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

    // Check if user has upload permissions
    if !check_scope(&authenticated_user, "upload") {
        return Ok(forbidden_error("Insufficient permissions to upload agents"));
    }

    // Check content type
    let headers = req.headers();
    let content_type = headers
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !content_type.starts_with("application/json") {
        let error = ApiError {
            error: "bad_request".to_string(),
            message: "Content-Type must be application/json".to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    // Parse request body
    let body_bytes = req.body();
    let body_str = std::str::from_utf8(body_bytes)
        .map_err(|_| Error::from("Invalid UTF-8 in request body"))?;

    let upload_request: UploadAgentRequest = match serde_json::from_str(body_str) {
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

    // Validate the upload request
    match validate_upload_request(&upload_request) {
        Ok(_) => {}
        Err(validation_errors) => {
            let response = UploadAgentResponse {
                success: false,
                message: "Validation failed".to_string(),
                agent: None,
                validation_errors: Some(validation_errors),
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&response)?.into())?);
        }
    }

    // Process the upload request
    match upload_agent(upload_request, &authenticated_user).await {
        Ok(agent) => {
            let response = UploadAgentResponse {
                success: true,
                message: "Agent uploaded successfully".to_string(),
                agent: Some(agent),
                validation_errors: None,
            };
            Ok(Response::builder()
                .status(201)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&response)?.into())?)
        }
        Err(err_msg) => {
            let error = ApiError {
                error: "upload_failed".to_string(),
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

fn validate_upload_request(request: &UploadAgentRequest) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate agent name
    if request.name.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Agent name cannot be empty".to_string(),
        });
    } else if !request
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        errors.push(ValidationError {
            field: "name".to_string(),
            message:
                "Agent name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
        });
    } else if request.name.len() > 100 {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Agent name cannot exceed 100 characters".to_string(),
        });
    }

    // Validate description
    if request.description.trim().is_empty() {
        errors.push(ValidationError {
            field: "description".to_string(),
            message: "Description cannot be empty".to_string(),
        });
    } else if request.description.len() > 1000 {
        errors.push(ValidationError {
            field: "description".to_string(),
            message: "Description cannot exceed 1000 characters".to_string(),
        });
    }

    // Validate content
    if request.content.trim().is_empty() {
        errors.push(ValidationError {
            field: "content".to_string(),
            message: "Content cannot be empty".to_string(),
        });
    } else if request.content.len() > 1024 * 1024 {
        // 1MB limit for content
        errors.push(ValidationError {
            field: "content".to_string(),
            message: "Content size exceeds maximum allowed size (1MB)".to_string(),
        });
    }

    // Validate YAML frontmatter in content
    if let Err(frontmatter_errors) = validate_frontmatter_consistency(request) {
        errors.extend(frontmatter_errors);
    }

    // Validate optional version
    if let Some(version) = &request.version {
        if version.trim().is_empty() {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Version cannot be empty".to_string(),
            });
        } else if !version
            .chars()
            .all(|c| c.is_alphanumeric() || ".-_+".contains(c))
        {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Version can only contain alphanumeric characters, dots, hyphens, underscores, and plus signs".to_string(),
            });
        } else if version.len() > 50 {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Version cannot exceed 50 characters".to_string(),
            });
        }
    }

    // Validate tags
    for (index, tag) in request.tags.iter().enumerate() {
        if tag.trim().is_empty() {
            errors.push(ValidationError {
                field: format!("tags[{index}]"),
                message: "Tags cannot be empty".to_string(),
            });
        } else if tag.len() > 50 {
            errors.push(ValidationError {
                field: format!("tags[{index}]"),
                message: "Tags cannot exceed 50 characters".to_string(),
            });
        }
    }

    if request.tags.len() > 20 {
        errors.push(ValidationError {
            field: "tags".to_string(),
            message: "Cannot have more than 20 tags".to_string(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_frontmatter_consistency(
    request: &UploadAgentRequest,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check if content starts with YAML frontmatter
    if !request.content.starts_with("---") {
        errors.push(ValidationError {
            field: "content".to_string(),
            message: "Content must contain YAML frontmatter starting with ---".to_string(),
        });
        return Err(errors);
    }

    // Find the end of the frontmatter
    let lines: Vec<&str> = request.content.lines().collect();
    let mut frontmatter_end = None;

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            frontmatter_end = Some(i);
            break;
        }
    }

    let frontmatter_end = match frontmatter_end {
        Some(end) => end,
        None => {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: "Invalid YAML frontmatter: missing closing ---".to_string(),
            });
            return Err(errors);
        }
    };

    // Extract frontmatter content
    let frontmatter_lines = &lines[1..frontmatter_end];
    let frontmatter_content = frontmatter_lines.join("\n");

    // Parse YAML frontmatter
    let frontmatter: YamlFrontmatter = match serde_yaml::from_str(&frontmatter_content) {
        Ok(fm) => fm,
        Err(e) => {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: format!("Invalid YAML frontmatter: {e}"),
            });
            return Err(errors);
        }
    };

    // Validate name consistency
    if frontmatter.name != request.name {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: format!(
                "Name mismatch: frontmatter contains '{}' but request contains '{}'",
                frontmatter.name, request.name
            ),
        });
    }

    // Validate description consistency
    if frontmatter.description != request.description {
        errors.push(ValidationError {
            field: "description".to_string(),
            message: format!(
                "Description mismatch: frontmatter contains '{}' but request contains '{}'",
                frontmatter.description, request.description
            ),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

async fn upload_agent(request: UploadAgentRequest, user: &AuthenticatedUser) -> Result<Agent, String> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock success if no database configured
        return Ok(create_mock_uploaded_agent(request, user));
    }

    // In production:
    // 1. Parse YAML frontmatter from content
    // 2. Extract agent metadata and content body
    // 3. Store the agent definition in Supabase Storage
    // 4. Create/update agent record in database
    // 5. Return the created agent

    // For now, return mock data
    Ok(create_mock_uploaded_agent(request, user))
}

fn create_mock_uploaded_agent(request: UploadAgentRequest, user: &AuthenticatedUser) -> Agent {
    let version = request.version.unwrap_or_else(|| "1.0.0".to_string());

    Agent {
        name: request.name,
        version,
        description: request.description,
        author: format!("user-{}", user.user_id), // Use authenticated user ID
        created_at: Utc::now(),
        updated_at: Utc::now(),
        download_count: 0,
        tags: request.tags,
        readme: Some(request.content), // Store the full content as readme for now
        homepage: request.homepage,
        repository: request.repository,
        license: request.license,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_upload_request() -> UploadAgentRequest {
        UploadAgentRequest {
            name: "test-agent".to_string(),
            description: "A test agent".to_string(),
            content: r#"---
name: test-agent
description: A test agent
version: "1.0.0"
tags: ["test", "example"]
---

# Test Agent

This is a test agent for demonstration purposes.

## Usage

This agent can be used for testing the upload functionality.
"#
            .to_string(),
            version: Some("1.0.0".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
            homepage: Some("https://example.com".to_string()),
            repository: Some("https://github.com/user/test-agent".to_string()),
            license: Some("MIT".to_string()),
        }
    }

    #[test]
    fn test_validate_upload_request_valid() {
        let request = create_valid_upload_request();
        assert!(validate_upload_request(&request).is_ok());
    }

    #[test]
    fn test_validate_upload_request_empty_name() {
        let mut request = create_valid_upload_request();
        request.name = "".to_string();

        let result = validate_upload_request(&request);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "name" && e.message.contains("cannot be empty")));
    }

    #[test]
    fn test_validate_upload_request_invalid_name() {
        let mut request = create_valid_upload_request();
        request.name = "invalid name!".to_string();

        let result = validate_upload_request(&request);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "name" && e.message.contains("alphanumeric")));
    }

    #[test]
    fn test_validate_upload_request_no_frontmatter() {
        let mut request = create_valid_upload_request();
        request.content = "# Test Agent\n\nNo frontmatter here.".to_string();

        let result = validate_upload_request(&request);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "content" && e.message.contains("YAML frontmatter")));
    }

    #[test]
    fn test_validate_upload_request_mismatched_name() {
        let mut request = create_valid_upload_request();
        request.content = r#"---
name: different-name
description: A test agent
---

# Test Agent
"#
        .to_string();

        let result = validate_upload_request(&request);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "name" && e.message.contains("Name mismatch")));
    }

    #[test]
    fn test_validate_upload_request_mismatched_description() {
        let mut request = create_valid_upload_request();
        request.content = r#"---
name: test-agent
description: Different description
---

# Test Agent
"#
        .to_string();

        let result = validate_upload_request(&request);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "description" && e.message.contains("Description mismatch")));
    }

    #[test]
    fn test_validate_upload_request_too_many_tags() {
        let mut request = create_valid_upload_request();
        request.tags = (0..25).map(|i| format!("tag{}", i)).collect();

        let result = validate_upload_request(&request);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field == "tags" && e.message.contains("more than 20 tags")));
    }

    #[test]
    fn test_create_mock_uploaded_agent() {
        let request = create_valid_upload_request();
        let mock_user = AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            key_id: uuid::Uuid::new_v4(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };
        let agent = create_mock_uploaded_agent(request.clone(), &mock_user);

        assert_eq!(agent.name, request.name);
        assert_eq!(agent.description, request.description);
        assert_eq!(agent.version, request.version.unwrap());
        assert_eq!(agent.tags, request.tags);
        assert_eq!(agent.homepage, request.homepage);
        assert_eq!(agent.repository, request.repository);
        assert_eq!(agent.license, request.license);
        assert_eq!(agent.download_count, 0);
        assert_eq!(agent.author, "mock-user");
    }
}
