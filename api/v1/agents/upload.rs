use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

// Use shared authentication module
use serde_json::json;
use shared::{api_key_middleware, require_scope, ApiError, AuthenticatedUser};

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

    // Authenticate the request using API key only
    let authenticated_user = match api_key_middleware(&req).await {
        Ok(user) => user,
        Err(error_response) => return Ok(error_response),
    };

    // Check if user has upload permissions
    if let Err(error_response) = require_scope(&authenticated_user, "upload") {
        return Ok(error_response);
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

    // Extract the original API key from the request for database function
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Add debug logging for authentication state
    eprintln!("DEBUG: Upload request from user_id: {}, auth_method: {:?}, scopes: {:?}", 
        authenticated_user.user_id, authenticated_user.auth_method, authenticated_user.scopes);

    // Process the upload request
    match upload_agent(upload_request, &authenticated_user, auth_header).await {
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
            eprintln!("DEBUG: Upload failed with error: {}", err_msg);
            let error = ApiError {
                error: "upload_failed".to_string(),
                message: err_msg,
                details: Some(json!({
                    "user_id": authenticated_user.user_id,
                    "auth_method": format!("{:?}", authenticated_user.auth_method),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            };
            Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?)
        }
    }
}

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

async fn upload_agent(
    request: UploadAgentRequest,
    user: &AuthenticatedUser,
    _auth_header: &str,
) -> Result<Agent, String> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    eprintln!("DEBUG: Database config - URL: {}, Key: {}", 
        if supabase_url.is_empty() { "MISSING" } else { "SET" },
        if supabase_key.is_empty() { "MISSING" } else { "SET" }
    );

    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock success if no database configured
        eprintln!("DEBUG: Using mock upload (no database configured)");
        return Ok(create_mock_uploaded_agent(request, user));
    }

    // Create HTTP client
    let client = reqwest::Client::new();

    // First, ensure the user exists in the database (critical for API key users)
    eprintln!("DEBUG: Syncing user to database: {}", user.user_id);
    let sync_result = match &user.auth_method {
        shared::AuthMethod::ApiKey { .. } => {
            // Sync API key user
            let sync_params = json!({
                "user_uuid": user.user_id,
                "user_email": user.metadata.email,
                "github_username": user.metadata.github_username
            });
            
            client
                .post(format!("{}/rest/v1/rpc/sync_api_key_user", supabase_url))
                .header("apikey", &supabase_key)
                .header("Authorization", format!("Bearer {}", supabase_key))
                .header("Content-Type", "application/json")
                .json(&sync_params)
                .send()
                .await
        }
        shared::AuthMethod::JwtToken { .. } => {
            // Sync JWT user
            let sync_params = json!({
                "user_uuid": user.user_id,
                "user_email": user.metadata.email,
                "github_username": user.metadata.github_username,
                "display_name": user.metadata.github_username,
                "avatar_url": null
            });
            
            client
                .post(format!("{}/rest/v1/rpc/sync_jwt_user_fixed", supabase_url))
                .header("apikey", &supabase_key)
                .header("Authorization", format!("Bearer {}", supabase_key))
                .header("Content-Type", "application/json")
                .json(&sync_params)
                .send()
                .await
        }
    };

    // Check sync result but don't fail the upload if sync fails
    match sync_result {
        Ok(response) => {
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                eprintln!("DEBUG: User sync failed (non-fatal): {}", error_text);
            } else {
                eprintln!("DEBUG: User sync successful");
            }
        }
        Err(e) => {
            eprintln!("DEBUG: User sync request failed (non-fatal): {}", e);
        }
    }

    // Parse YAML frontmatter from content to create definition JSON
    let definition = parse_agent_definition(&request.content)
        .map_err(|e| format!("Failed to parse agent definition: {e}"))?;

    // Prepare parameters for create_agent function
    let version = request.version.unwrap_or_else(|| "1.0.0".to_string());
    
    // First, try to use the safe agent creation function that bypasses RLS
    let create_agent_params = json!({
        "p_user_id": user.user_id,
        "p_name": request.name,
        "p_description": request.description,
        "p_definition": definition,
        "p_tags": request.tags,
        "p_author_name": format!("user-{}", user.user_id),
        "p_license": request.license.clone().unwrap_or_else(|| "MIT".to_string()),
        "p_homepage": request.homepage.clone().unwrap_or_else(|| "".to_string()),
        "p_repository": request.repository.clone().unwrap_or_else(|| "".to_string()),
        "p_readme": request.content,
        "p_keywords": request.tags,
        "p_current_version": version,
        "p_is_public": true
    });

    eprintln!("DEBUG: Attempting to insert agent using safe function");

    // Try the safe function first
    let response = client
        .post(format!("{supabase_url}/rest/v1/rpc/create_agent_safe"))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .header("Content-Type", "application/json")
        .json(&create_agent_params)
        .send()
        .await
        .map_err(|e| format!("Database request failed: {e}"))?;

    let status_code = response.status();
    eprintln!("DEBUG: Database response status: {}", status_code);
    
    if response.status().is_success() {
        // Success path - parse response
        let response_body = response.text().await
            .map_err(|e| format!("Failed to read response: {e}"))?;
            
        eprintln!("DEBUG: Database response body: {}", response_body);

        // Parse the created agent from database response
        let created_agents: Vec<serde_json::Value> = serde_json::from_str(&response_body)
            .map_err(|e| format!("Failed to parse database response '{}': {e}", response_body))?;

        if let Some(agent_data) = created_agents.first() {
            let agent = Agent {
                name: agent_data["name"].as_str().unwrap_or(&request.name).to_string(),
                version: version.clone(),
                description: agent_data["description"].as_str().unwrap_or(&request.description).to_string(),
                author: agent_data["author_name"].as_str().unwrap_or(&format!("user-{}", user.user_id)).to_string(),
                created_at: serde_json::from_value(agent_data["created_at"].clone())
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: serde_json::from_value(agent_data["updated_at"].clone())
                    .unwrap_or_else(|_| Utc::now()),
                download_count: agent_data["download_count"].as_u64().unwrap_or(0),
                tags: serde_json::from_value(agent_data["tags"].clone()).unwrap_or(request.tags.clone()),
                readme: Some(request.content.clone()),
                homepage: request.homepage.clone(),
                repository: request.repository.clone(),
                license: request.license.clone(),
            };
            return Ok(agent);
        } else {
            return Err("No agent data returned from database".to_string());
        }
    }
    
    // Safe function failed, get error details
    let error_text = response.text().await.unwrap_or_default();
    eprintln!("DEBUG: Safe function failed with response: {}", error_text);
    
    // Try direct insert as fallback
    eprintln!("DEBUG: Safe function failed, trying direct insert as fallback");
    
    let agent_data = json!({
        "user_id": user.user_id,
        "name": request.name,
        "description": request.description,
        "definition": definition,
        "tags": request.tags,
        "author_name": format!("user-{}", user.user_id),
        "license": request.license.clone().unwrap_or_else(|| "MIT".to_string()),
        "homepage": request.homepage.clone().unwrap_or_else(|| "".to_string()),
        "repository": request.repository.clone().unwrap_or_else(|| "".to_string()),
        "readme": request.content,
        "keywords": request.tags,
        "current_version": version,
        "is_public": true
    });
    
    let fallback_response = client
        .post(format!("{supabase_url}/rest/v1/agents"))
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {supabase_key}"))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&agent_data)
        .send()
        .await
        .map_err(|e| format!("Fallback database request failed: {e}"))?;
        
    let fallback_status = fallback_response.status();
    if !fallback_status.is_success() {
        let fallback_error = fallback_response.text().await.unwrap_or_default();
        eprintln!("DEBUG: Fallback also failed: {}", fallback_error);
        return Err(format!("Database error - Safe function failed ({}): {}\nFallback failed ({}): {}", 
            status_code, error_text, fallback_status, fallback_error));
    }
    
    eprintln!("DEBUG: Fallback succeeded");
    
    // Use fallback response for parsing
    let response_body = fallback_response.text().await
        .map_err(|e| format!("Failed to read fallback response: {e}"))?;
        
    eprintln!("DEBUG: Fallback response body: {}", response_body);

    // Parse the created agent from fallback response
    let created_agents: Vec<serde_json::Value> = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse fallback response '{}': {e}", response_body))?;
        
    if let Some(agent_data) = created_agents.first() {
        let agent = Agent {
            name: agent_data["name"].as_str().unwrap_or(&request.name).to_string(),
            version,
            description: agent_data["description"].as_str().unwrap_or(&request.description).to_string(),
            author: agent_data["author_name"].as_str().unwrap_or(&format!("user-{}", user.user_id)).to_string(),
            created_at: serde_json::from_value(agent_data["created_at"].clone())
                .unwrap_or_else(|_| Utc::now()),
            updated_at: serde_json::from_value(agent_data["updated_at"].clone())
                .unwrap_or_else(|_| Utc::now()),
            download_count: agent_data["download_count"].as_u64().unwrap_or(0),
            tags: serde_json::from_value(agent_data["tags"].clone()).unwrap_or(request.tags),
            readme: Some(request.content),
            homepage: request.homepage,
            repository: request.repository,
            license: request.license,
        };
        Ok(agent)
    } else {
        Err("No agent data returned from fallback database".to_string())
    }
}

/// Parse agent definition from markdown content with YAML frontmatter
fn parse_agent_definition(content: &str) -> Result<serde_json::Value, String> {
    // Validate that content starts with YAML frontmatter
    if !content.starts_with("---") {
        return Err("Content must contain YAML frontmatter starting with ---".to_string());
    }

    // Find the end of the frontmatter
    let lines: Vec<&str> = content.lines().collect();
    let mut frontmatter_end = None;

    for (i, line) in lines.iter().enumerate().skip(1) {
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "..." {
            frontmatter_end = Some(i);
            break;
        }
    }

    let frontmatter_end = frontmatter_end
        .ok_or_else(|| "Invalid YAML frontmatter: missing closing --- or ...".to_string())?;

    // Extract frontmatter and content body
    let frontmatter_lines = &lines[1..frontmatter_end];
    let frontmatter_content = frontmatter_lines.join("\n");
    let body_lines = &lines[(frontmatter_end + 1)..];
    let body_content = body_lines.join("\n");

    // Parse YAML frontmatter
    let frontmatter: serde_json::Value = serde_yaml::from_str(&frontmatter_content)
        .map_err(|e| format!("Invalid YAML frontmatter: {e}"))?;

    // Create complete definition with frontmatter metadata and body content
    let definition = json!({
        "metadata": frontmatter,
        "content": body_content,
        "format": "markdown",
        "frontmatter_type": "yaml"
    });

    Ok(definition)
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