use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

/// Optimized agent structure for latest/trending endpoints - minimal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    #[serde(default = "default_version")]
    pub current_version: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub download_count: u64,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub view_count: u64,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Latest agents response
#[derive(Debug, Serialize, Deserialize)]
pub struct LatestAgentsResponse {
    pub agents: Vec<Agent>,
    pub cached_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Handle CORS preflight
    if req.method() == "OPTIONS" {
        return Ok(Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .body(Body::Empty)?)
    }

    // Parse limit parameter (default 10, max 50)
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .min(50); // Cap at 50 to prevent abuse

    // Fetch latest agents optimized query
    let agents = get_latest_agents(limit).await?;

    let response_body = LatestAgentsResponse {
        agents,
        cached_at: chrono::Utc::now(),
    };

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("Cache-Control", "public, max-age=60") // Cache for 1 minute
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(serde_json::to_string(&response_body)?.into())?;

    Ok(response)
}

async fn get_latest_agents(limit: usize) -> Result<Vec<Agent>, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();

    eprintln!("[DEBUG] SUPABASE_URL present: {}", !supabase_url.is_empty());
    eprintln!("[DEBUG] SUPABASE_KEY present: {}", !supabase_key.is_empty());
    eprintln!("[DEBUG] URL prefix: {}", supabase_url.chars().take(30).collect::<String>());

    if supabase_url.is_empty() || supabase_key.is_empty() {
        eprintln!("[ERROR] Database not configured - missing environment variables");
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_ANON_KEY",
        ));
    }

    let client = postgrest::Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", &supabase_key));

    // Optimized query: Only fetch what we need, use existing optimal index
    // Handle potential missing fields gracefully
    eprintln!("[DEBUG] Executing query on agents table with limit: {}", limit);
    
    // First try a simple query to verify connection
    let test_response = client
        .from("agents")
        .select("name")
        .limit(1)
        .execute()
        .await;
    
    match test_response {
        Ok(resp) => {
            eprintln!("[DEBUG] Test query status: {}", resp.status());
            let body = resp.text().await.unwrap_or_default();
            eprintln!("[DEBUG] Test query response: {}", body);
        }
        Err(e) => eprintln!("[ERROR] Test query failed: {}", e),
    }
    
    let response = client
        .from("agents")
        .select("name,description,created_at,updated_at,tags,view_count")
        .eq("is_public", "true")
        .order("created_at.desc") // Uses idx_agents_public_created index
        .limit(limit)
        .execute()
        .await
        .map_err(|e| {
            eprintln!("[ERROR] Database query failed: {}", e);
            Error::from(format!("Database query failed: {e}"))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        eprintln!("[ERROR] Database query failed with status: {}", status);
        eprintln!("[ERROR] Error response: {}", error_body);
        return Err(Error::from(format!(
            "Database query failed with status: {} - {}",
            status, error_body
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| Error::from(format!("Failed to read response: {e}")))?;

    // Return empty list if no data
    if body.is_empty() || body == "[]" {
        eprintln!("[DEBUG] Empty response from database");
        return Ok(Vec::new());
    }

    eprintln!("[DEBUG] Response body length: {}", body.len());
    eprintln!("[DEBUG] Response preview: {}", body.chars().take(200).collect::<String>());

    let agents: Vec<Agent> = serde_json::from_str(&body).map_err(|e| {
        eprintln!("[ERROR] Failed to parse agents response: {}", body);
        eprintln!("[ERROR] Parse error: {}", e);
        Error::from(format!("Failed to parse agents: {e}"))
    })?;

    eprintln!("[DEBUG] Successfully parsed {} agents", agents.len());
    Ok(agents)
}
