use anyhow::{anyhow, Result as AnyhowResult};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

// Use shared authentication module
use shared::{
    api_key_middleware, check_scope, extract_bearer_token,
    ApiError, AuthenticatedUser
};




/// Optional authentication for downloads - allows both authenticated and unauthenticated access
async fn optional_authenticate(req: &Request) -> Option<AuthenticatedUser> {
    // Only attempt authentication if a token is provided
    if extract_bearer_token(req).is_some() {
        api_key_middleware(req).await.ok()
    } else {
        None
    }
}

/// Agent download information
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDownload {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
}

// ApiError is now imported from shared module

/// Supabase storage response for signed URLs
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedUrlResponse {
    #[serde(rename = "signedURL")]
    pub signed_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Optional authentication - if API key is provided, validate it
    // This allows both authenticated and unauthenticated access
    let authenticated_user = optional_authenticate(&req).await;

    // Extract path parameters from URL path
    let path = req.uri().path();
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Expected format: api/v1/agents/{name}/{version}/download
    if path_segments.len() < 6 {
        let error = ApiError {
            error: "bad_request".to_string(),
            message: "Invalid path format. Expected /api/v1/agents/{name}/{version}/download"
                .to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    let agent_name = urlencoding::decode(path_segments[3])
        .map_err(|_| Error::from("Invalid agent name encoding"))?;
    let version = urlencoding::decode(path_segments[4])
        .map_err(|_| Error::from("Invalid version encoding"))?;

    // Get agent download info from database
    match get_agent_download_info(&agent_name, &version, &req, authenticated_user.as_ref()).await {
        Ok(download_info) => Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&download_info)?.into())?),
        Err(e) => {
            let error = ApiError {
                error: "not_found".to_string(),
                message: format!(
                    "Agent '{}' version '{}' not found: {}",
                    agent_name, version, e
                ),
                details: None,
            };
            Ok(Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?)
        }
    }
}

async fn get_agent_download_info(
    name: &str,
    version: &str,
    req: &Request,
    authenticated_user: Option<&AuthenticatedUser>,
) -> AnyhowResult<AgentDownload> {
    // Get database connection parameters
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| anyhow!("SUPABASE_URL environment variable not set"))?;
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
        .map_err(|_| anyhow!("SUPABASE_SERVICE_ROLE_KEY environment variable not set"))?;

    let client = reqwest::Client::new();

    // Query the database for agent information
    let agent_info = query_agent_info(
        &client,
        &supabase_url,
        &supabase_key,
        name,
        version,
        authenticated_user,
    )
    .await?;

    // Generate signed URL for download
    let download_url =
        generate_signed_url(&client, &supabase_url, &supabase_key, &agent_info.file_path).await?;

    // Record the download
    record_download(&client, &supabase_url, &supabase_key, name, version, req).await?;

    Ok(AgentDownload {
        name: agent_info.name,
        version: agent_info.version,
        download_url,
        checksum: agent_info.checksum,
        size: agent_info.file_size,
    })
}

#[derive(Debug)]
struct AgentInfo {
    name: String,
    version: String,
    file_path: String,
    checksum: String,
    file_size: u64,
}

async fn query_agent_info(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
    name: &str,
    version: &str,
    authenticated_user: Option<&AuthenticatedUser>,
) -> AnyhowResult<AgentInfo> {
    let url = format!("{}/rest/v1/rpc/get_agent_download_info", supabase_url);

    let payload = json!({
        "p_agent_name": name,
        "p_version_text": if version == "latest" { "" } else { version },
        "p_user_id": authenticated_user.map(|u| u.user_id.to_string())
    });

    let response = client
        .post(&url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow!("Database query failed: {}", error_text));
    }

    let result: serde_json::Value = response.json().await?;

    // Parse the result from the database function
    if let Some(data) = result.as_array().and_then(|arr| arr.first()) {
        // Check if the agent is private and user has access
        let is_public = data
            .get("is_public")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let owner_id = data.get("user_id").and_then(|v| v.as_str());

        if !is_public {
            match authenticated_user {
                Some(user) => {
                    let user_id_str = user.user_id.to_string();
                    if Some(user_id_str.as_str()) != owner_id && !check_scope(user, "admin") {
                        return Err(anyhow!("Access denied: This agent is private"));
                    }
                }
                None => {
                    return Err(anyhow!("Authentication required: This agent is private"));
                }
            }
        }
        Ok(AgentInfo {
            name: data
                .get("agent_name")
                .and_then(|v| v.as_str())
                .unwrap_or(name)
                .to_string(),
            version: data
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or(version)
                .to_string(),
            file_path: data
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing file_path in database response"))?
                .to_string(),
            checksum: data
                .get("checksum")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            file_size: data.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    } else {
        Err(anyhow!(
            "Agent not found or no valid response from database"
        ))
    }
}

async fn generate_signed_url(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
    file_path: &str,
) -> AnyhowResult<String> {
    let url = format!(
        "{}/storage/v1/object/sign/agent-packages/{}",
        supabase_url, file_path
    );

    let payload = json!({
        "expiresIn": 3600 // 1 hour expiration
    });

    let response = client
        .post(&url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow!("Failed to generate signed URL: {}", error_text));
    }

    let signed_response: SignedUrlResponse = response.json().await?;
    Ok(format!("{}{}", supabase_url, signed_response.signed_url))
}

async fn record_download(
    client: &reqwest::Client,
    supabase_url: &str,
    supabase_key: &str,
    name: &str,
    version: &str,
    req: &Request,
) -> AnyhowResult<()> {
    let url = format!("{}/rest/v1/rpc/record_download", supabase_url);

    // Extract user agent and IP from request headers
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let ip_addr = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1")
        .split(',')
        .next()
        .unwrap_or("127.0.0.1")
        .trim()
        .to_string();

    let payload = json!({
        "agent_name": name,
        "version_text": if version == "latest" { "" } else { version },
        "user_agent_text": user_agent,
        "ip_addr": ip_addr
    });

    let response = client
        .post(&url)
        .header("apikey", supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        // Don't fail the entire request if download tracking fails
        eprintln!("Warning: Failed to record download: {}", error_text);
    }

    Ok(())
}
