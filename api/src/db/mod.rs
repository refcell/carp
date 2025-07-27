use anyhow::Result;
use postgrest::Postgrest;
use std::env;

/// Database client wrapper for Supabase PostgreSQL
#[derive(Clone)]
pub struct Database {
    client: Postgrest,
    supabase_url: String,
    supabase_key: String,
}

impl Database {
    /// Create a new database connection
    pub fn new() -> Result<Self> {
        let supabase_url = env::var("SUPABASE_URL")
            .map_err(|_| anyhow::anyhow!("SUPABASE_URL environment variable is required"))?;
        
        let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY environment variable is required"))?;

        let client = Postgrest::new(format!("{}/rest/v1", supabase_url))
            .insert_header("apikey", &supabase_key)
            .insert_header("Authorization", format!("Bearer {}", supabase_key));

        Ok(Self {
            client,
            supabase_url,
            supabase_key,
        })
    }

    /// Get a reference to the PostgREST client
    pub fn client(&self) -> &Postgrest {
        &self.client
    }

    /// Create a new client with user authentication
    pub fn with_user_auth(&self, token: &str) -> Postgrest {
        Postgrest::new(format!("{}/rest/v1", self.supabase_url))
            .insert_header("apikey", &self.supabase_key)
            .insert_header("Authorization", format!("Bearer {}", token))
    }

    /// Execute a PostgreSQL function with the service role
    pub fn rpc(&self, function_name: &str) -> postgrest::Builder {
        self.client.rpc(function_name, "{}")
    }

    /// Execute a PostgreSQL function with parameters
    pub fn rpc_with_params(&self, function_name: &str, params: serde_json::Value) -> postgrest::Builder {
        self.client.rpc(function_name, &params.to_string())
    }

    /// Get storage client for file operations
    pub fn storage_url(&self) -> String {
        format!("{}/storage/v1", self.supabase_url)
    }

    /// Get the service role key for storage operations
    pub fn service_key(&self) -> &str {
        &self.supabase_key
    }
}