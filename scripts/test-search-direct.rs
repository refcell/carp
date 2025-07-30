#!/usr/bin/env rust-script

//! Direct API Test Script
//!
//! Tests the search API functions directly without needing a server.
//! Run with: cargo run --bin test-search-direct

use std::collections::HashMap;
use std::env;

// Mock the vercel_runtime types for testing
mod mock_vercel {
    use std::collections::HashMap;

    pub struct Request {
        query: String,
    }

    impl Request {
        pub fn new(query: &str) -> Self {
            Self { query: query.to_string() }
        }

        pub fn uri(&self) -> MockUri {
            MockUri { query: &self.query }
        }
    }

    pub struct MockUri<'a> {
        query: &'a str,
    }

    impl<'a> MockUri<'a> {
        pub fn query(&self) -> Option<&str> {
            if self.query.is_empty() { None } else { Some(self.query) }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Carp Search API Functions Directly");
    println!("=============================================");
    println!();

    // Test 1: Empty search (list all)
    println!("📋 Test 1: List all agents (empty search)");
    test_search_scenario("", 20, 1, false).await?;

    // Test 2: Search with query
    println!("🔍 Test 2: Search for agents");
    test_search_scenario("q=test", 10, 1, false).await?;

    // Test 3: Search with limit
    println!("📄 Test 3: Search with limit");
    test_search_scenario("limit=5", 5, 1, false).await?;

    // Test 4: Exact search
    println!("🎯 Test 4: Exact search");
    test_search_scenario("q=example&exact=true", 20, 1, true).await?;

    println!("✅ All direct tests completed!");
    Ok(())
}

async fn test_search_scenario(
    query_string: &str,
    expected_limit: usize,
    expected_page: usize,
    expected_exact: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse query parameters
    let search_params: HashMap<String, String> = if query_string.is_empty() {
        HashMap::new()
    } else {
        url::form_urlencoded::parse(query_string.as_bytes())
            .into_owned()
            .collect()
    };

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

    println!("  Query: '{}', Limit: {}, Page: {}, Exact: {}", search_query, limit, page, exact);

    // Test the search function directly
    match search_agents_in_db(search_query, limit, page, exact).await {
        Ok(agents) => {
            println!("  ✅ Found {} agents", agents.len());
            for (i, agent) in agents.iter().take(3).enumerate() {
                println!("    {}. {} v{} by {}", i + 1, agent.name, agent.version, agent.author);
            }
            if agents.len() > 3 {
                println!("    ... and {} more", agents.len() - 3);
            }
        }
        Err(e) => {
            println!("  ❌ Search failed: {}", e);
            return Err(e.into());
        }
    }

    // Test the count function
    match get_total_agent_count(search_query, exact).await {
        Ok(count) => {
            println!("  ✅ Total count: {}", count);
        }
        Err(e) => {
            println!("  ❌ Count failed: {}", e);
            return Err(e.into());
        }
    }

    println!();
    Ok(())
}

// Include the actual search functions from the API
// (This would normally be imported, but for a standalone script we'll inline them)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

async fn search_agents_in_db(
    query: &str,
    limit: usize,
    page: usize,
    exact: bool,
) -> Result<Vec<Agent>, Box<dyn std::error::Error>> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL")?;
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY")?;

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
            // Text search across name and description using proper PostgREST syntax
            query_builder = query_builder
                .or(format!("name.ilike.*{}*,description.ilike.*{}*", query, query));
        }
    }

    // Apply pagination
    query_builder = query_builder
        .range(offset, offset + limit - 1)
        .order("download_count.desc,updated_at.desc");

    // Execute query
    let response = query_builder.execute().await?;

    let body = response.text().await?;

    // Parse response as Vec<Agent>
    let agents: Vec<Agent> = serde_json::from_str(&body)?;

    Ok(agents)
}

async fn get_total_agent_count(query: &str, exact: bool) -> Result<usize, Box<dyn std::error::Error>> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL")?;
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY")?;

    // Create Supabase client
    let client = postgrest::Postgrest::new(format!("{}/rest/v1", supabase_url))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", supabase_key));

    // Build count query using PostgREST's exact_count feature
    let mut query_builder = client.from("agents").select("id").exact_count();

    // Apply same search filter as main query
    if !query.is_empty() {
        if exact {
            query_builder = query_builder.eq("name", query);
        } else {
            // Use proper PostgREST text search syntax
            query_builder = query_builder
                .or(format!("name.ilike.*{}*,description.ilike.*{}*", query, query));
        }
    }

    // Execute count query
    let response = query_builder.execute().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Database query failed with status {}: {}", status, error_text).into());
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
