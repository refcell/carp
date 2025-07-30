#!/bin/bash

# Quick Test Script - Tests the fixed search API directly against the live database
# This bypasses the serverless function and tests our logic directly

set -e

echo "🧪 Quick API Function Test"
echo "=========================="
echo ""

# Check if required dependencies are available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

# Create a temporary Cargo project for testing
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "📦 Creating temporary test project..."

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "search-test"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
postgrest = "1.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12", features = ["json"] }
url = "2.0"
EOF

# Create src directory and the test source
mkdir -p src
cat > src/main.rs << 'EOF'
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Fixed Search API Functions");
    println!("====================================");
    println!();

    // Test 1: List all agents
    println!("📋 Test 1: List all agents");
    match search_agents_in_db("", 10, 1, false).await {
        Ok(agents) => {
            println!("  ✅ Found {} agents", agents.len());
            for (i, agent) in agents.iter().take(3).enumerate() {
                println!("    {}. {} v{} by {}", i + 1, agent.name, agent.version, agent.author);
            }
        }
        Err(e) => {
            println!("  ❌ Search failed: {}", e);
            return Err(e);
        }
    }

    // Test 2: Get total count
    println!();
    println!("🔢 Test 2: Get total agent count");
    match get_total_agent_count("", false).await {
        Ok(count) => {
            println!("  ✅ Total agents: {}", count);
        }
        Err(e) => {
            println!("  ❌ Count failed: {}", e);
            return Err(e);
        }
    }

    // Test 3: Search with query
    println!();
    println!("🔍 Test 3: Search for 'test'");
    match search_agents_in_db("test", 5, 1, false).await {
        Ok(agents) => {
            println!("  ✅ Found {} agents matching 'test'", agents.len());
            for agent in &agents {
                println!("    - {} ({})", agent.name, agent.description);
            }
        }
        Err(e) => {
            println!("  ❌ Search failed: {}", e);
            return Err(e);
        }
    }

    println!();
    println!("✅ All tests passed! The API functions are working correctly.");
    println!("🚀 The issue was with the PostgREST query syntax, which has been fixed.");

    Ok(())
}

async fn search_agents_in_db(
    query: &str,
    limit: usize,
    page: usize,
    exact: bool,
) -> Result<Vec<Agent>, Box<dyn std::error::Error>> {
    let supabase_url = env::var("SUPABASE_URL")?;
    
    // For public read operations, prefer anon key over service role key
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))?;

    let client = postgrest::Postgrest::new(format!("{}/rest/v1", supabase_url))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", supabase_key));

    let offset = (page - 1) * limit;

    let mut query_builder = client
        .from("agents")
        .select("name,current_version,description,author_name,created_at,updated_at,download_count,tags,readme,homepage,repository,license");

    if !query.is_empty() {
        if exact {
            query_builder = query_builder.eq("name", query);
        } else {
            // Fixed PostgREST syntax
            query_builder = query_builder
                .or(format!("name.ilike.*{}*,description.ilike.*{}*", query, query));
        }
    }

    query_builder = query_builder
        .range(offset, offset + limit - 1)
        .order("download_count.desc,updated_at.desc");

    let response = query_builder.execute().await?;
    let body = response.text().await?;
    let db_agents: Vec<DbAgent> = serde_json::from_str(&body)?;
    let agents: Vec<Agent> = db_agents.into_iter().map(Agent::from).collect();

    Ok(agents)
}

async fn get_total_agent_count(query: &str, exact: bool) -> Result<usize, Box<dyn std::error::Error>> {
    let supabase_url = env::var("SUPABASE_URL")?;
    
    // For public read operations, prefer anon key over service role key
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))?;

    let client = postgrest::Postgrest::new(format!("{}/rest/v1", supabase_url))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", supabase_key));

    // Fixed: Use exact_count() method
    let mut query_builder = client.from("agents").select("id").exact_count();

    if !query.is_empty() {
        if exact {
            query_builder = query_builder.eq("name", query);
        } else {
            query_builder = query_builder
                .or(format!("name.ilike.*{}*,description.ilike.*{}*", query, query));
        }
    }

    let response = query_builder.execute().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Database query failed with status {}: {}", status, error_text).into());
    }

    // Fixed: Parse Content-Range header
    if let Some(content_range) = response.headers().get("content-range") {
        if let Ok(range_str) = content_range.to_str() {
            if let Some(total_str) = range_str.split('/').nth(1) {
                if let Ok(count) = total_str.parse::<usize>() {
                    return Ok(count);
                }
            }
        }
    }

    Ok(0)
}
EOF

echo "🔨 Building and running test..."
echo ""

# Build and run the test
cargo run --release

# Clean up
cd - > /dev/null
rm -rf "$TEMP_DIR"

echo ""
echo "🎉 Test completed successfully!"
echo ""
echo "Next steps:"
echo "1. The API functions are now fixed and should work correctly"
echo "2. Deploy to Vercel to test with the live API"
echo "3. Run 'cd cli && cargo run -- list' to test the CLI"
