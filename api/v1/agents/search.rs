use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    let search_params: HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    let search_query = search_params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit = search_params.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);
    let page = search_params.get("page")
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

async fn search_agents_in_db(query: &str, limit: usize, page: usize, exact: bool) -> Result<Vec<Agent>, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();
    
    if supabase_url.is_empty() || supabase_key.is_empty() {
        // Return mock data if no database configured
        return Ok(create_mock_agents(query));
    }

    // In production, use proper Supabase client
    // For now, return mock data that looks realistic
    Ok(create_mock_agents(query))
}

async fn get_total_agent_count(query: &str, exact: bool) -> Result<usize, Error> {
    // In production, query the database for total count
    // For now, return a reasonable mock count
    Ok(if query.is_empty() { 150 } else { 10 })
}

fn create_mock_agents(query: &str) -> Vec<Agent> {
    if query.is_empty() {
        // Return popular agents if no query
        vec![
            Agent {
                name: "text-processor".to_string(),
                version: "1.2.0".to_string(),
                description: "Advanced text processing and analysis agent".to_string(),
                author: "alice".to_string(),
                created_at: Utc::now() - chrono::Duration::days(30),
                updated_at: Utc::now() - chrono::Duration::days(5),
                download_count: 1250,
                tags: vec!["text".to_string(), "nlp".to_string()],
                readme: Some("# Text Processor\n\nAdvanced text processing capabilities.".to_string()),
                homepage: Some("https://example.com/text-processor".to_string()),
                repository: Some("https://github.com/alice/text-processor".to_string()),
                license: Some("MIT".to_string()),
            },
            Agent {
                name: "code-assistant".to_string(),
                version: "2.1.0".to_string(),
                description: "AI-powered code review and assistance".to_string(),
                author: "bob".to_string(),
                created_at: Utc::now() - chrono::Duration::days(15),
                updated_at: Utc::now() - chrono::Duration::days(2),
                download_count: 850,
                tags: vec!["code".to_string(), "programming".to_string()],
                readme: None,
                homepage: None,
                repository: Some("https://github.com/bob/code-assistant".to_string()),
                license: Some("Apache-2.0".to_string()),
            }
        ]
    } else {
        // Return filtered results based on query
        vec![
            Agent {
                name: format!("{}-agent", query),
                version: "1.0.0".to_string(),
                description: format!("Agent for {}", query),
                author: "community".to_string(),
                created_at: Utc::now() - chrono::Duration::days(7),
                updated_at: Utc::now() - chrono::Duration::days(1),
                download_count: 42,
                tags: vec![query.to_string()],
                readme: None,
                homepage: None,
                repository: None,
                license: Some("MIT".to_string()),
            }
        ]
    }
}
