use anyhow::anyhow;
use postgrest::Postgrest;
use serde_json::json;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    match get_database_health().await {
        Ok(agent_count) => {
            let response_body = json!({
                "status": "healthy",
                "service": "carp-api",
                "environment": "serverless",
                "message": "API is running on Vercel with database connectivity",
                "agent_count": agent_count,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            let response = Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(response_body.to_string().into())?;

            Ok(response)
        }
        Err(err) => {
            let response_body = json!({
                "status": "unhealthy",
                "service": "carp-api",
                "environment": "serverless",
                "message": "Database connection failed",
                "error": err.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            let response = Response::builder()
                .status(503)
                .header("content-type", "application/json")
                .body(response_body.to_string().into())?;

            Ok(response)
        }
    }
}

async fn get_database_health() -> Result<i64, anyhow::Error> {
    // Get database connection details from environment
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| anyhow!("SUPABASE_URL environment variable not set"))?;
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
        .map_err(|_| anyhow!("SUPABASE_SERVICE_ROLE_KEY environment variable not set"))?;

    if supabase_url.is_empty() {
        return Err(anyhow!("SUPABASE_URL is empty"));
    }
    if supabase_key.is_empty() {
        return Err(anyhow!("SUPABASE_SERVICE_ROLE_KEY is empty"));
    }

    // Create Postgrest client
    let client = Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {supabase_key}"));

    // Query the agents table to get the exact count using PostgREST's exact_count feature
    let response = client
        .from("agents")
        .select("id")
        .exact_count()
        .execute()
        .await
        .map_err(|e| anyhow!("Failed to execute database query: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!(
            "Database query failed with status {}: {}",
            status,
            error_text
        ));
    }

    // PostgREST returns the count in the Content-Range header when using exact_count
    // Format: "0-N/total" where total is the exact count
    if let Some(content_range) = response.headers().get("content-range") {
        if let Ok(range_str) = content_range.to_str() {
            // Parse the content-range header to get total count
            // Format: "0-4/5" where 5 is the total count, or "*/0" if no records
            if let Some(total_str) = range_str.split('/').nth(1) {
                if let Ok(count) = total_str.parse::<i64>() {
                    return Ok(count);
                }
            }
        }
    }

    // Fallback: if Content-Range header parsing fails,
    // do a simple query to check if database is accessible
    // This at least verifies database connectivity
    let test_response = client
        .from("agents")
        .select("id")
        .limit(1)
        .execute()
        .await
        .map_err(|e| anyhow!("Failed to execute fallback database query: {}", e))?;

    if !test_response.status().is_success() {
        let status = test_response.status();
        let error_text = test_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!(
            "Fallback database query failed with status {}: {}",
            status,
            error_text
        ));
    }

    // If we reach here, the database is accessible but we couldn't get exact count
    // Return -1 to indicate "unknown count but database is healthy"
    Ok(-1)
}
