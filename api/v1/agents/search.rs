use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

/// Database agent structure (matches actual DB schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbAgent {
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
    pub readme: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

/// Agent metadata returned by the API (matches expected client schema)
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

impl From<DbAgent> for Agent {
    fn from(db_agent: DbAgent) -> Self {
        Agent {
            name: db_agent.name,
            version: db_agent.version,
            description: db_agent.description,
            author: db_agent.author_name.unwrap_or_else(|| "Unknown".to_string()),
            created_at: db_agent.created_at,
            updated_at: db_agent.updated_at,
            download_count: db_agent.download_count,
            tags: db_agent.tags.unwrap_or_else(Vec::new),
            readme: db_agent.readme,
            homepage: db_agent.homepage,
            repository: db_agent.repository,
            license: db_agent.license,
        }
    }
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
    // For public search operations, use anon key for proper public access
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_ANON_KEY",
        ));
    }

    // Create Supabase client for public read access (search endpoint should be public)
    // Use only apikey header, no Authorization Bearer token needed for public reads
    let client = postgrest::Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key);

    // Calculate offset for pagination
    let offset = (page - 1) * limit;

    // Build query based on search parameters
    // Note: Using actual database column names
    let mut query_builder = client
        .from("agents")
        .select("name,current_version,description,author_name,created_at,updated_at,download_count,tags,readme,homepage,repository,license");

    // Apply search filter if query is provided
    if !query.is_empty() {
        if exact {
            // Exact match on name
            query_builder = query_builder.eq("name", query);
        } else {
            // Text search across name and description using proper PostgREST syntax
            query_builder = query_builder
                .or(format!("name.ilike.*{query}*,description.ilike.*{query}*"));
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
        .map_err(|e| Error::from(format!("Database query failed: {e}")))?;

    let body = response
        .text()
        .await
        .map_err(|e| Error::from(format!("Failed to read response: {e}")))?;

    // Parse response as Vec<DbAgent> then convert to Vec<Agent>
    let db_agents: Vec<DbAgent> = serde_json::from_str(&body)
        .map_err(|e| Error::from(format!("Failed to parse agents: {e}")))?;

    let agents: Vec<Agent> = db_agents.into_iter().map(Agent::from).collect();

    Ok(agents)
}

async fn get_total_agent_count(query: &str, exact: bool) -> Result<usize, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    // For public search operations, use anon key for proper public access
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();

    if supabase_url.is_empty() || supabase_key.is_empty() {
        return Err(Error::from(
            "Database not configured - missing SUPABASE_URL or SUPABASE_ANON_KEY",
        ));
    }

    // Create Supabase client for public read access (search endpoint should be public)
    // Use only apikey header, no Authorization Bearer token needed for public reads
    let client = postgrest::Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key);

    // Build count query using PostgREST's exact_count feature
    let mut query_builder = client.from("agents").select("id").exact_count();

    // Apply same search filter as main query
    if !query.is_empty() {
        if exact {
            query_builder = query_builder.eq("name", query);
        } else {
            // Use proper PostgREST text search syntax
            query_builder = query_builder
                .or(format!("name.ilike.*{query}*,description.ilike.*{query}*"));
        }
    }

    // Execute count query
    let response = query_builder
        .execute()
        .await
        .map_err(|e| Error::from(format!("Database count query failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(Error::from(format!(
            "Database query failed with status {status}: {error_text}"
        )));
    }

    // PostgREST returns the count in the Content-Range header when using exact_count
    if let Some(content_range) = response.headers().get("content-range") {
        if let Ok(range_str) = content_range.to_str() {
            // Parse the content-range header to get total count
            // Format: "0-4/5" where 5 is the total count, or "*/0" if no records
            if let Some(total_str) = range_str.split('/').nth(1) {
                if let Ok(count) = total_str.parse::<usize>() {
                    return Ok(count);
                }
            }
        }
    }

    // Fallback to 0 if count parsing fails
    Ok(0)
}

