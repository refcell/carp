use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// Agent download information
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDownload {
    pub agent_id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub download_url: String,
    pub file_size: u64,
    pub checksum: String,
    pub content_type: String,
    pub definition: serde_json::Value,
}

/// Request for publishing an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub readme: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
}

/// Response from publishing an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub success: bool,
    pub message: String,
    pub agent: Option<Agent>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Request for uploading an agent via JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadAgentRequest {
    pub name: String,
    pub description: String,
    pub content: String,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

/// Response from uploading an agent
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadAgentResponse {
    pub success: bool,
    pub message: String,
    pub agent: Option<Agent>,
    pub validation_errors: Option<Vec<ValidationError>>,
}

/// Validation error details
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub environment: String,
    pub message: String,
    pub agent_count: Option<i64>,
    pub timestamp: String,
    pub error: Option<String>,
}
