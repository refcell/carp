use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

/// Optimized agent structure for trending endpoint
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
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(serde_json::to_string(&response_body)?.into())?;

    Ok(response)
}

async fn get_trending_agents(limit: usize) -> Result<Vec<Agent>, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();

    eprintln!("[DEBUG] Trending - SUPABASE_URL present: {}", !supabase_url.is_empty());
    eprintln!("[DEBUG] Trending - SUPABASE_KEY present: {}", !supabase_key.is_empty());

    if supabase_url.is_empty() || supabase_key.is_empty() {
        eprintln!("[ERROR] Trending - Database not configured");
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_ANON_KEY",
        ));
    }

    let client = postgrest::Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", &supabase_key));

    // Try to ensure the materialized view is populated if we have service role key
    if env::var("SUPABASE_SERVICE_ROLE_KEY").is_ok() {
        let _ = client
            .rpc("ensure_trending_view_populated", "{}")
            .execute()
            .await; // Ignore errors, will fall back to regular query if needed
    }

    // Try materialized view first for optimal performance
    let response = client
        .from("trending_agents_mv")
        .select("name,description,created_at,updated_at,tags,view_count")
        .order("view_count.desc") // Order by view count as fallback
        .limit(limit)
        .execute()
        .await;

    let response = match response {
        Ok(resp) if resp.status().is_success() => {
            // Check if the response has content
            let body_check = resp.text().await.unwrap_or_default();
            if body_check.is_empty() || body_check == "[]" {
                // Materialized view is empty, fall back to regular query
                None
            } else {
                // Return the successful response by re-executing the query
                // since we consumed the body above
                Some(
                    client
                        .from("trending_agents_mv")
                        .select("name,description,created_at,updated_at,tags,view_count")
                        .order("view_count.desc")
                        .limit(limit)
                        .execute()
                        .await
                        .map_err(|e| Error::from(format!("Materialized view query failed: {e}")))?
                )
            }
        }
        Ok(_) | Err(_) => None, // Failed or unsuccessful status, use fallback
    };

    let response = match response {
        Some(resp) => resp,
        None => {
            // Fallback to regular agents table if materialized view fails or is empty
            eprintln!("Falling back to regular agents table for trending query");
            client
                .from("agents")
                .select("name,description,created_at,updated_at,tags,view_count")
                .eq("is_public", "true")
                .gte("view_count", "1")
                .order("view_count.desc,updated_at.desc")
                .limit(limit)
                .execute()
                .await
                .map_err(|e| Error::from(format!("Fallback database query failed: {e}")))?
        }
    };

    if !response.status().is_success() {
        return Err(Error::from(format!(
            "Database query failed with status: {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| Error::from(format!("Failed to read response: {e}")))?;

    // Return empty list if no data
    if body.is_empty() || body == "[]" {
        eprintln!("[DEBUG] Trending - Empty response from database");
        return Ok(Vec::new());
    }

    eprintln!("[DEBUG] Trending - Response body length: {}", body.len());
    eprintln!("[DEBUG] Trending - Response preview: {}", body.chars().take(200).collect::<String>());

    let agents: Vec<Agent> = serde_json::from_str(&body).map_err(|e| {
        eprintln!("[ERROR] Failed to parse trending agents response: {}", body);
        eprintln!("[ERROR] Trending - Parse error: {}", e);
        Error::from(format!("Failed to parse agents: {e}"))
    })?;

    eprintln!("[DEBUG] Trending - Successfully parsed {} agents", agents.len());
    Ok(agents)
}
