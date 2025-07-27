use serde::{Deserialize, Serialize};
// use serde_json::json; // Not used in this file
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

/// Agent download information
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDownload {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Extract path parameters
    let path = req.uri().path();
    let path_segments: Vec<&str> = path.split('/').collect();
    
    // Expected format: /api/v1/agents/{name}/{version}/download
    if path_segments.len() < 6 {
        let error = ApiError {
            error: "bad_request".to_string(),
            message: "Invalid path format. Expected /api/v1/agents/{name}/{version}/download".to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    let agent_name = path_segments[4];
    let version = path_segments[5];

    // Get agent download info from database
    match get_agent_download_info(agent_name, version).await {
        Ok(download_info) => {
            Ok(Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&download_info)?.into())?)
        }
        Err(_) => {
            let error = ApiError {
                error: "not_found".to_string(),
                message: format!("Agent '{}' version '{}' not found", agent_name, version),
                details: None,
            };
            Ok(Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?)
        }
    }
}

async fn get_agent_download_info(name: &str, version: &str) -> Result<AgentDownload, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();
    
    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock data if no database configured
        return Ok(create_mock_download_info(name, version));
    }

    // In production, query Supabase for the agent and get download URL from storage
    // For now, return mock data
    Ok(create_mock_download_info(name, version))
}

fn create_mock_download_info(name: &str, version: &str) -> AgentDownload {
    // Mock download URL - in production this would be a Supabase Storage URL
    let download_url = format!("https://mock-storage.supabase.co/storage/v1/object/public/agent-packages/{}/{}/agent.zip", name, version);
    
    // Mock checksum - in production this would be stored in the database
    let checksum = format!("sha256:{}", mock_sha256(format!("{}-{}", name, version)));
    
    AgentDownload {
        name: name.to_string(),
        version: version.to_string(),
        download_url,
        checksum,
        size: 1024 * 50, // Mock size: 50KB
    }
}

fn mock_sha256(input: String) -> String {
    // Mock SHA256 - in production use proper hashing
    format!("{:064x}", input.chars().map(|c| c as u64).sum::<u64>())
}