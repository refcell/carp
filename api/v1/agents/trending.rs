use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

/// Optimized agent structure for trending endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    #[serde(rename = "current_version")]
    pub version: String,
    pub description: String,
    #[serde(rename = "author_name")]
    pub author_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_count: u64,
    pub tags: Option<Vec<String>>,
}

/// Trending agents response
#[derive(Debug, Serialize, Deserialize)]
pub struct TrendingAgentsResponse {
    pub agents: Vec<Agent>,
    pub cached_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Parse limit parameter (default 10, max 50)
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> = 
        url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10)
        .min(50); // Cap at 50 to prevent abuse

    // Fetch trending agents optimized query
    let agents = get_trending_agents(limit).await?;

    let response_body = TrendingAgentsResponse {
        agents,
        cached_at: chrono::Utc::now(),
    };

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("Cache-Control", "public, max-age=300") // Cache for 5 minutes (materialized view allows longer cache)
        .body(serde_json::to_string(&response_body)?.into())?;

    Ok(response)
}

async fn get_trending_agents(limit: usize) -> Result<Vec<Agent>, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_ANON_KEY",
        ));
    }

    let client = postgrest::Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key);

    // Try materialized view first for optimal performance
    let response = client
        .from("trending_agents_mv")
        .select("name,current_version,description,author_name,created_at,updated_at,download_count,tags")
        .order("trending_score.desc") // Uses idx_trending_agents_mv_score index
        .limit(limit)
        .execute()
        .await;

    let response = match response {
        Ok(resp) if resp.status().is_success() => resp,
        _ => {
            // Fallback to regular agents table if materialized view fails
            client
                .from("agents")
                .select("name,current_version,description,author_name,created_at,updated_at,download_count,tags")
                .eq("is_public", "true")
                .gte("download_count", "1")
                .order("download_count.desc,updated_at.desc")
                .limit(limit)
                .execute()
                .await
                .map_err(|e| Error::from(format!("Database query failed: {e}")))?
        }
    };

    let body = response
        .text()
        .await
        .map_err(|e| Error::from(format!("Failed to read response: {e}")))?;

    let agents: Vec<Agent> = serde_json::from_str(&body)
        .map_err(|e| Error::from(format!("Failed to parse agents: {e}")))?;

    Ok(agents)
}