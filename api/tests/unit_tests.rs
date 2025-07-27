/// Unit tests for API components
/// These tests focus on individual functions and modules without external dependencies

use carp_api::{
    auth::AuthUser,
    models::{Agent, AgentDownload, AuthRequest, DbAgent, PublishRequest, SearchQuery, SearchResponse, UserProfile},
    utils::ApiError,
};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

// Test authentication request validation
#[test]
fn test_auth_request_validation() {
    // Valid request
    let valid_request = AuthRequest {
        username: "testuser".to_string(),
        password: "validpassword".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Empty username
    let invalid_request = AuthRequest {
        username: "".to_string(),
        password: "validpassword".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Empty password
    let invalid_request = AuthRequest {
        username: "testuser".to_string(),
        password: "".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Both empty
    let invalid_request = AuthRequest {
        username: "".to_string(),
        password: "".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

// Test publish request validation
#[test]
fn test_publish_request_validation() {
    // Valid request
    let valid_request = PublishRequest {
        name: "valid-agent".to_string(),
        version: "1.0.0".to_string(),
        description: "A valid description".to_string(),
        readme: Some("# README".to_string()),
        homepage: Some("https://example.com".to_string()),
        repository: Some("https://github.com/user/repo".to_string()),
        license: Some("MIT".to_string()),
        tags: vec!["ai".to_string(), "assistant".to_string()],
    };
    assert!(valid_request.validate().is_ok());

    // Empty name
    let invalid_request = PublishRequest {
        name: "".to_string(),
        version: "1.0.0".to_string(),
        description: "Description".to_string(),
        readme: None,
        homepage: None,
        repository: None,
        license: None,
        tags: vec![],
    };
    assert!(invalid_request.validate().is_err());

    // Empty version
    let invalid_request = PublishRequest {
        name: "valid-agent".to_string(),
        version: "".to_string(),
        description: "Description".to_string(),
        readme: None,
        homepage: None,
        repository: None,
        license: None,
        tags: vec![],
    };
    assert!(invalid_request.validate().is_err());

    // Empty description
    let invalid_request = PublishRequest {
        name: "valid-agent".to_string(),
        version: "1.0.0".to_string(),
        description: "".to_string(),
        readme: None,
        homepage: None,
        repository: None,
        license: None,
        tags: vec![],
    };
    assert!(invalid_request.validate().is_err());
}

// Test search query validation
#[test]
fn test_search_query_validation() {
    // Valid query with all parameters
    let valid_query = SearchQuery {
        q: "test query".to_string(),
        limit: Some(20),
        page: Some(1),
        exact: false,
        tags: Some("ai,assistant".to_string()),
        author: Some("testuser".to_string()),
        sort: Some("relevance".to_string()),
    };
    assert!(valid_query.validate().is_ok());

    // Valid query with minimal parameters
    let minimal_query = SearchQuery {
        q: "".to_string(),
        limit: None,
        page: None,
        exact: false,
        tags: None,
        author: None,
        sort: None,
    };
    assert!(minimal_query.validate().is_ok());

    // Valid query with only search term
    let search_only_query = SearchQuery {
        q: "search term".to_string(),
        limit: None,
        page: None,
        exact: false,
        tags: None,
        author: None,
        sort: None,
    };
    assert!(search_only_query.validate().is_ok());
}

// Test API error creation and serialization
#[test]
fn test_api_error_creation() {
    let validation_error = ApiError::validation_error("Invalid input");
    assert_eq!(validation_error.error, "ValidationError");
    assert_eq!(validation_error.message, "Invalid input");
    assert!(validation_error.details.is_none());

    let not_found_error = ApiError::not_found_error("Resource not found");
    assert_eq!(not_found_error.error, "NotFoundError");
    assert_eq!(not_found_error.message, "Resource not found");

    let internal_error = ApiError::internal_error("Something went wrong");
    assert_eq!(internal_error.error, "InternalError");
    assert_eq!(internal_error.message, "Something went wrong");

    let auth_error = ApiError::authorization_error("Unauthorized access");
    assert_eq!(auth_error.error, "AuthorizationError");
    assert_eq!(auth_error.message, "Unauthorized access");

    let conflict_error = ApiError::conflict_error("Resource already exists");
    assert_eq!(conflict_error.error, "ConflictError");
    assert_eq!(conflict_error.message, "Resource already exists");

    let payload_error = ApiError::payload_too_large();
    assert_eq!(payload_error.error, "PayloadTooLarge");
    assert_eq!(payload_error.message, "Request payload too large");
}

// Test API error serialization
#[test]
fn test_api_error_serialization() {
    let error = ApiError::validation_error("Test validation error");
    let serialized = serde_json::to_string(&error).expect("Should serialize");
    let deserialized: ApiError = serde_json::from_str(&serialized).expect("Should deserialize");
    
    assert_eq!(error.error, deserialized.error);
    assert_eq!(error.message, deserialized.message);
    assert_eq!(error.details, deserialized.details);
}

// Test model conversion from DbAgent to Agent
#[test]
fn test_db_agent_to_agent_conversion() {
    let now = Utc::now();
    let db_agent = DbAgent {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        name: "test-agent".to_string(),
        current_version: "2.1.0".to_string(),
        description: "A test agent for validation".to_string(),
        author_name: Some("test-author".to_string()),
        created_at: now,
        updated_at: now,
        download_count: 150,
        tags: vec!["test".to_string(), "validation".to_string()],
        keywords: None,
        view_count: None,
        is_public: true,
        readme: Some("# Test Agent\nThis is a test agent.".to_string()),
        homepage: Some("https://example.com/test-agent".to_string()),
        repository: Some("https://github.com/test/agent".to_string()),
        license: Some("Apache-2.0".to_string()),
    };

    let agent: Agent = Agent::from(db_agent.clone());

    assert_eq!(agent.name, db_agent.name);
    assert_eq!(agent.version, db_agent.current_version);
    assert_eq!(agent.description, db_agent.description);
    assert_eq!(agent.author, db_agent.author_name.unwrap());
    assert_eq!(agent.created_at, db_agent.created_at);
    assert_eq!(agent.updated_at, db_agent.updated_at);
    assert_eq!(agent.download_count, db_agent.download_count as u64);
    assert_eq!(agent.tags, db_agent.tags);
    assert_eq!(agent.readme, db_agent.readme);
    assert_eq!(agent.homepage, db_agent.homepage);
    assert_eq!(agent.repository, db_agent.repository);
    assert_eq!(agent.license, db_agent.license);
}

// Test agent serialization/deserialization
#[test]
fn test_agent_serialization() {
    let now = Utc::now();
    let agent = Agent {
        name: "test-agent".to_string(),
        version: "1.0.0".to_string(),
        description: "Test agent description".to_string(),
        author: "testuser".to_string(),
        created_at: now,
        updated_at: now,
        download_count: 42,
        tags: vec!["ai".to_string(), "test".to_string()],
        readme: Some("# README".to_string()),
        homepage: Some("https://example.com".to_string()),
        repository: Some("https://github.com/test/repo".to_string()),
        license: Some("MIT".to_string()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&agent).expect("Should serialize to JSON");
    let deserialized: Agent = serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(agent.name, deserialized.name);
    assert_eq!(agent.version, deserialized.version);
    assert_eq!(agent.description, deserialized.description);
    assert_eq!(agent.author, deserialized.author);
    assert_eq!(agent.download_count, deserialized.download_count);
    assert_eq!(agent.tags, deserialized.tags);
    assert_eq!(agent.readme, deserialized.readme);
    assert_eq!(agent.homepage, deserialized.homepage);
    assert_eq!(agent.repository, deserialized.repository);
    assert_eq!(agent.license, deserialized.license);
}

// Test search response structure
#[test]
fn test_search_response_structure() {
    let now = Utc::now();
    let agents = vec![
        Agent {
            name: "agent-1".to_string(),
            version: "1.0.0".to_string(),
            description: "First test agent".to_string(),
            author: "author1".to_string(),
            created_at: now,
            updated_at: now,
            download_count: 10,
            tags: vec!["test".to_string()],
            readme: None,
            homepage: None,
            repository: None,
            license: None,
        },
        Agent {
            name: "agent-2".to_string(),
            version: "2.0.0".to_string(),
            description: "Second test agent".to_string(),
            author: "author2".to_string(),
            created_at: now,
            updated_at: now,
            download_count: 25,
            tags: vec!["test".to_string(), "demo".to_string()],
            readme: Some("# Agent 2".to_string()),
            homepage: Some("https://example.com".to_string()),
            repository: Some("https://github.com/test/agent2".to_string()),
            license: Some("MIT".to_string()),
        },
    ];

    let search_response = SearchResponse {
        agents: agents.clone(),
        total: 100,
        page: 2,
        per_page: 20,
    };

    assert_eq!(search_response.agents.len(), 2);
    assert_eq!(search_response.total, 100);
    assert_eq!(search_response.page, 2);
    assert_eq!(search_response.per_page, 20);

    // Test serialization
    let json = serde_json::to_string(&search_response).expect("Should serialize");
    let deserialized: SearchResponse = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(search_response.agents.len(), deserialized.agents.len());
    assert_eq!(search_response.total, deserialized.total);
    assert_eq!(search_response.page, deserialized.page);
    assert_eq!(search_response.per_page, deserialized.per_page);
}

// Test agent download structure
#[test]
fn test_agent_download_structure() {
    let download = AgentDownload {
        name: "test-agent".to_string(),
        version: "1.0.0".to_string(),
        download_url: "https://storage.example.com/agent.zip".to_string(),
        checksum: "sha256:abcdef123456".to_string(),
        size: 1024,
    };

    assert_eq!(download.name, "test-agent");
    assert_eq!(download.version, "1.0.0");
    assert_eq!(download.download_url, "https://storage.example.com/agent.zip");
    assert_eq!(download.checksum, "sha256:abcdef123456");
    assert_eq!(download.size, 1024);

    // Test serialization
    let json = serde_json::to_string(&download).expect("Should serialize");
    let deserialized: AgentDownload = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(download.name, deserialized.name);
    assert_eq!(download.version, deserialized.version);
    assert_eq!(download.download_url, deserialized.download_url);
    assert_eq!(download.checksum, deserialized.checksum);
    assert_eq!(download.size, deserialized.size);
}

// Test user profile structure
#[test]
fn test_user_profile_structure() {
    let profile = UserProfile {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        display_name: Some("Test User".to_string()),
        email: Some("test@example.com".to_string()),
        avatar_url: None,
        github_username: None,
        created_at: Utc::now(),
    };

    assert_eq!(profile.username, "testuser");
    assert_eq!(profile.email, Some("test@example.com".to_string()));

    // Test serialization
    let json = serde_json::to_string(&profile).expect("Should serialize");
    let deserialized: UserProfile = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(profile.id, deserialized.id);
    assert_eq!(profile.username, deserialized.username);
    assert_eq!(profile.email, deserialized.email);
    assert_eq!(profile.created_at, deserialized.created_at);
}

// Test auth user structure
#[test]
fn test_auth_user_structure() {
    let auth_user = AuthUser {
        user_id: Uuid::new_v4(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    assert_eq!(auth_user.scopes.len(), 2);
    assert!(auth_user.scopes.contains(&"read".to_string()));
    assert!(auth_user.scopes.contains(&"write".to_string()));
}

// Test checksum calculation
#[test]
fn test_checksum_calculation() {
    use sha2::{Digest, Sha256};

    let content = b"test content for checksum";
    let mut hasher = Sha256::new();
    hasher.update(content);
    let checksum = format!("{:x}", hasher.finalize());

    // Verify the checksum is the expected length (64 chars for SHA256)
    assert_eq!(checksum.len(), 64);
    
    // Verify consistency
    let mut hasher2 = Sha256::new();
    hasher2.update(content);
    let checksum2 = format!("{:x}", hasher2.finalize());
    assert_eq!(checksum, checksum2);

    // Verify different content produces different checksum
    let different_content = b"different test content";
    let mut hasher3 = Sha256::new();
    hasher3.update(different_content);
    let checksum3 = format!("{:x}", hasher3.finalize());
    assert_ne!(checksum, checksum3);
}

// Test pagination calculations
#[test]
fn test_pagination_logic() {
    // Test limit capping (should be between 1 and 100)
    let limit = Some(150u32).unwrap_or(20).min(100).max(1);
    assert_eq!(limit, 100);

    let limit = Some(0u32).unwrap_or(20).min(100).max(1);
    assert_eq!(limit, 1);

    let limit = Some(50u32).unwrap_or(20).min(100).max(1);
    assert_eq!(limit, 50);

    let limit = None.unwrap_or(20u32).min(100).max(1);
    assert_eq!(limit, 20);

    // Test page minimum (should be at least 1)
    let page = Some(0u32).unwrap_or(1).max(1);
    assert_eq!(page, 1);

    let page = Some(5u32).unwrap_or(1).max(1);
    assert_eq!(page, 5);

    let page = None.unwrap_or(1u32).max(1);
    assert_eq!(page, 1);
}

// Test tag parsing
#[test]
fn test_tag_parsing() {
    // Test comma-separated tags
    let tags_str = "ai,assistant,productivity,automation";
    let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    
    assert_eq!(tags.len(), 4);
    assert_eq!(tags[0], "ai");
    assert_eq!(tags[1], "assistant");
    assert_eq!(tags[2], "productivity");
    assert_eq!(tags[3], "automation");

    // Test tags with spaces
    let tags_str = "ai, assistant , productivity,  automation  ";
    let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    
    assert_eq!(tags.len(), 4);
    assert_eq!(tags[0], "ai");
    assert_eq!(tags[1], "assistant");
    assert_eq!(tags[2], "productivity");
    assert_eq!(tags[3], "automation");

    // Test empty string
    let tags_str = "";
    let tags: Vec<String> = if tags_str.is_empty() {
        Vec::new()
    } else {
        tags_str.split(',').map(|s| s.trim().to_string()).collect()
    };
    
    assert_eq!(tags.len(), 0);

    // Test single tag
    let tags_str = "single-tag";
    let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0], "single-tag");
}

// Test URL construction
#[test]
fn test_url_construction() {
    let base_url = "https://api.example.com";
    let user_id = "user123";
    let agent_name = "test-agent";
    let version = "1.0.0";
    let filename = "agent.zip";

    let file_path = format!("{}/{}/{}/{}", user_id, agent_name, version, filename);
    assert_eq!(file_path, "user123/test-agent/1.0.0/agent.zip");

    let storage_bucket = "agent-storage";
    let storage_url = format!("{}/object/public/{}/{}", base_url, storage_bucket, file_path);
    assert_eq!(storage_url, "https://api.example.com/object/public/agent-storage/user123/test-agent/1.0.0/agent.zip");

    let upload_url = format!("{}/object/{}/{}", base_url, storage_bucket, file_path);
    assert_eq!(upload_url, "https://api.example.com/object/agent-storage/user123/test-agent/1.0.0/agent.zip");
}