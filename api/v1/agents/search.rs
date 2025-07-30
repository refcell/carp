use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

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

/// Search results from the API
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub agents: Vec<Agent>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Extract query parameters for search functionality
    let query = req.uri().query().unwrap_or("");
    let search_params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let search_query = search_params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit = search_params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);
    let page = search_params
        .get("page")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    let exact = search_params.get("exact").is_some();

    // Search agents in database
    let agents = search_agents_in_db(search_query, limit, page, exact).await?;
    let total = get_total_agent_count(search_query, exact).await?;

    let response_body = SearchResponse {
        agents,
        total,
        page,
        per_page: limit,
    };

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&response_body)?.into())?;

    Ok(response)
}

async fn search_agents_in_db(
    query: &str,
    limit: usize,
    page: usize,
    exact: bool,
) -> Result<Vec<Agent>, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_SERVICE_ROLE_KEY",
        ));
    }

    // Create Supabase client
    let client = postgrest::Postgrest::new(format!("{}/rest/v1", supabase_url))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", supabase_key));

    // Calculate offset for pagination
    let offset = (page - 1) * limit;

    // Build query based on search parameters
    let mut query_builder = client
        .from("agents")
        .select("name,version,description,author,created_at,updated_at,download_count,tags,readme,homepage,repository,license");

    // Apply search filter if query is provided
    if !query.is_empty() {
        if exact {
            // Exact match on name
            query_builder = query_builder.eq("name", query);
        } else {
            // Text search across name, description, and tags
            // Using PostgreSQL full-text search or ILIKE for partial matches
            query_builder = query_builder.or(format!(
                "name.ilike.%{}%,description.ilike.%{}%,tags.cs.{{\"{}\"}}",
                query, query, query
            ));
        }
    }

    // Apply pagination
    query_builder = query_builder
        .range(offset, offset + limit - 1)
        .order("download_count.desc,updated_at.desc");

    // Execute query
    let response = query_builder
        .execute()
        .await
        .map_err(|e| Error::from(format!("Database query failed: {}", e)))?;

    let body = response
        .text()
        .await
        .map_err(|e| Error::from(format!("Failed to read response: {}", e)))?;

    // Parse response as Vec<Agent>
    let agents: Vec<Agent> = serde_json::from_str(&body)
        .map_err(|e| Error::from(format!("Failed to parse agents: {}", e)))?;

    Ok(agents)
}

async fn get_total_agent_count(query: &str, exact: bool) -> Result<usize, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_SERVICE_ROLE_KEY",
        ));
    }

    // Create Supabase client
    let client = postgrest::Postgrest::new(format!("{}/rest/v1", supabase_url))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", supabase_key));

    // Build count query using PostgreSQL COUNT function
    let mut query_builder = client.from("agents").select("count(*)").single(); // Return single row with count

    // Apply same search filter as main query
    if !query.is_empty() {
        if exact {
            query_builder = query_builder.eq("name", query);
        } else {
            query_builder = query_builder.or(format!(
                "name.ilike.%{}%,description.ilike.%{}%,tags.cs.{{\"{}\"}}",
                query, query, query
            ));
        }
    }

    // Execute count query
    let response = query_builder
        .execute()
        .await
        .map_err(|e| Error::from(format!("Database count query failed: {}", e)))?;

    let body = response
        .text()
        .await
        .map_err(|e| Error::from(format!("Failed to read count response: {}", e)))?;

    // Parse count result
    #[derive(Deserialize)]
    struct CountResult {
        count: i64,
    }

    let count_result: CountResult = serde_json::from_str(&body)
        .map_err(|e| Error::from(format!("Failed to parse count: {}", e)))?;

    let count = count_result.count.max(0) as usize;

    Ok(count)
}

fn create_mock_agents(query: &str) -> Vec<Agent> {
    if query.is_empty() {
        vec![]
    } else {
        // Return filtered results based on query
        vec![Agent {
            name: format!("{}-agent", query),
            version: "1.0.0".to_string(),
            description: format!("Agent for {}", query),
            author: format!("{}-author", query),
            created_at: Utc::now() - chrono::Duration::days(7),
            updated_at: Utc::now() - chrono::Duration::days(1),
            download_count: 42,
            tags: vec![query.to_string()],
            readme: None,
            homepage: None,
            repository: None,
            license: Some("MIT".to_string()),
        }]
    }
}
