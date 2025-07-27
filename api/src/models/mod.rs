use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Agent model matching the CLI expectations
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Database agent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbAgent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: String,
    pub author_name: Option<String>,
    pub current_version: String,
    pub tags: Vec<String>,
    pub keywords: Option<Vec<String>>,
    pub download_count: i64,
    pub view_count: Option<i32>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub readme: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Search response matching CLI expectations
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub agents: Vec<Agent>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

/// Agent download information
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDownload {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
}

/// Request for publishing an agent
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PublishRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 50))]
    pub version: String,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 10000))]
    pub readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 100))]
    pub license: Option<String>,
    #[validate(length(max = 10))]
    pub tags: Vec<String>,
}

/// Response from publishing an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<Agent>,
}

/// Authentication request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 1, max = 100))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// User profile information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub github_username: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// API token validation result
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTokenValidation {
    pub user_id: Uuid,
    pub scopes: Vec<String>,
}

/// Search query parameters
#[derive(Debug, Deserialize, Validate)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(default)]
    pub exact: bool,
    #[serde(default)]
    pub tags: Option<String>, // Comma-separated tags
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub sort: Option<String>, // relevance, downloads, created_at, updated_at
}

impl From<DbAgent> for Agent {
    fn from(db_agent: DbAgent) -> Self {
        Self {
            name: db_agent.name,
            version: db_agent.current_version,
            description: db_agent.description,
            author: db_agent.author_name.unwrap_or_else(|| "Unknown".to_string()),
            created_at: db_agent.created_at,
            updated_at: db_agent.updated_at,
            download_count: db_agent.download_count as u64,
            tags: db_agent.tags,
            readme: db_agent.readme,
            homepage: db_agent.homepage,
            repository: db_agent.repository,
            license: db_agent.license,
        }
    }
}