use crate::{
    models::{
        Agent, AgentDownload, DbAgent, PublishRequest, PublishResponse, SearchQuery, SearchResponse,
    },
    utils::{ApiError, ApiResult},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    Extension, Json,
};
use bytes::Bytes;
use serde_json::json;
use sha2::{Digest, Sha256};
use validator::Validate;

/// Search for agents
pub async fn search_agents(
    State(state): State<crate::AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<SearchResponse>> {
    let db = &state.db;
    // Validate query parameters
    query.validate()
        .map_err(|e| ApiError::validation_error(format!("Invalid query parameters: {}", e)))?;

    let limit = query.limit.unwrap_or(20).min(100).max(1); // Default 20, max 100, min 1
    let page = query.page.unwrap_or(1).max(1); // Default 1, min 1
    
    // Parse tags filter
    let tags_filter: Vec<String> = query.tags
        .as_ref()
        .map(|tags| tags.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Build search parameters
    let search_params = json!({
        "search_query": query.q,
        "tags_filter": tags_filter,
        "author_filter": query.author.unwrap_or_default(),
        "sort_by": query.sort.unwrap_or_else(|| "relevance".to_string()),
        "sort_order": "desc",
        "page_num": page,
        "page_size": limit
    });

    // Call the search function
    let search_query = db
        .rpc_with_params("search_agents", search_params)
        .execute()
        .await?;

    if search_query.status() != 200 {
        return Err(ApiError::internal_error("Search failed"));
    }

    let search_results: Vec<serde_json::Value> = search_query.json().await?;
    
    let mut agents = Vec::new();
    let mut total = 0;

    for result in search_results {
        // Extract agent data and convert to expected format
        let db_agent: DbAgent = serde_json::from_value(result.clone())
            .map_err(|_| ApiError::internal_error("Failed to parse search results"))?;
        
        agents.push(Agent::from(db_agent));
        
        // Get total count from first result
        if total == 0 {
            total = result["total_count"]
                .as_u64()
                .unwrap_or(0) as usize;
        }
    }

    Ok(Json(SearchResponse {
        agents,
        total,
        page,
        per_page: limit,
    }))
}

/// Get agent download information
pub async fn get_agent_download(
    State(state): State<crate::AppState>,
    Path((name, version)): Path<(String, String)>,
) -> ApiResult<Json<AgentDownload>> {
    let db = &state.db;
    let config = &state.config;
    // Handle "latest" version
    let version_str = if version == "latest" {
        // Get the latest version for this agent
        let agent_query = db
            .client()
            .from("agents")
            .select("current_version")
            .eq("name", &name)
            .eq("is_public", "true")
            .single()
            .execute()
            .await?;

        if agent_query.status() != 200 {
            return Err(ApiError::not_found_error("Agent not found"));
        }

        let agent_data: serde_json::Value = agent_query.json().await?;
        agent_data["current_version"]
            .as_str()
            .unwrap_or(&version)
            .to_string()
    } else {
        version
    };

    // First get the agent ID
    let agent_query = db
        .client()
        .from("agents")
        .select("id")
        .eq("name", &name)
        .eq("is_public", "true")
        .single()
        .execute()
        .await
        .map_err(|_| ApiError::not_found_error("Agent not found"))?;

    if agent_query.status() != 200 {
        return Err(ApiError::not_found_error("Agent not found"));
    }

    let agent_data: serde_json::Value = agent_query.json().await
        .map_err(|_| ApiError::internal_error("Failed to parse agent data"))?;
    let agent_id = agent_data["id"]
        .as_str()
        .ok_or_else(|| ApiError::internal_error("Invalid agent data"))?;

    // Then get the version information
    let version_response = db
        .client()
        .from("agent_versions")
        .select("id,version,package_size,checksum")
        .eq("agent_id", agent_id)
        .eq("version", &version_str)
        .single()
        .execute()
        .await
        .map_err(|_| ApiError::not_found_error("Agent version not found"))?;

    if version_response.status() != 200 {
        return Err(ApiError::not_found_error("Agent version not found"));
    }

    let version_data: serde_json::Value = version_response.json().await
        .map_err(|_| ApiError::internal_error("Failed to parse version data"))?;
    let version_id = version_data["id"]
        .as_str()
        .ok_or_else(|| ApiError::internal_error("Invalid version data"))?;

    // Get package information
    let package_query = db
        .client()
        .from("agent_packages")
        .select("file_name,file_path,file_size,checksum")
        .eq("version_id", version_id)
        .single()
        .execute()
        .await?;

    if package_query.status() != 200 {
        return Err(ApiError::not_found_error("Package not found"));
    }

    let package_data: serde_json::Value = package_query.json().await?;

    // Record the download
    let _ = db
        .rpc_with_params("record_download", json!({
            "agent_name": name,
            "version_text": version_str,
            "user_agent_text": "", // Could extract from headers
            "ip_addr": null
        }))
        .execute()
        .await;

    // Build download URL
    let file_path = package_data["file_path"]
        .as_str()
        .ok_or_else(|| ApiError::internal_error("Invalid package data"))?;
    
    let download_url = format!(
        "{}/object/public/{}/{}",
        db.storage_url(),
        config.upload.storage_bucket,
        file_path
    );

    Ok(Json(AgentDownload {
        name,
        version: version_str,
        download_url,
        checksum: package_data["checksum"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        size: package_data["file_size"]
            .as_u64()
            .unwrap_or(0),
    }))
}

/// Publish a new agent or version
pub async fn publish_agent(
    State(state): State<crate::AppState>,
    Extension(auth_user): Extension<crate::auth::AuthUser>,
    mut multipart: Multipart,
) -> ApiResult<Json<PublishResponse>> {
    let db = &state.db;
    let config = &state.config;

    // Check write permissions
    if !auth_user.scopes.contains(&"write".to_string()) {
        return Err(ApiError::authorization_error("Write permission required"));
    }

    let mut metadata: Option<PublishRequest> = None;
    let mut content: Option<Bytes> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await
        .map_err(|_| ApiError::validation_error("Invalid multipart data"))? 
    {
        let name = field.name()
            .ok_or_else(|| ApiError::validation_error("Missing field name"))?;

        match name {
            "metadata" => {
                let data = field.bytes().await
                    .map_err(|_| ApiError::validation_error("Failed to read metadata"))?;
                let metadata_str = String::from_utf8(data.to_vec())
                    .map_err(|_| ApiError::validation_error("Invalid metadata encoding"))?;
                metadata = Some(serde_json::from_str(&metadata_str)?);
            }
            "content" => {
                let data = field.bytes().await
                    .map_err(|_| ApiError::validation_error("Failed to read content"))?;
                
                if data.len() > config.upload.max_file_size as usize {
                    return Err(ApiError::payload_too_large());
                }
                
                content = Some(data);
            }
            _ => {} // Ignore unknown fields
        }
    }

    let metadata = metadata
        .ok_or_else(|| ApiError::validation_error("Missing metadata"))?;
    let content = content
        .ok_or_else(|| ApiError::validation_error("Missing content"))?;

    // Validate metadata
    metadata.validate()
        .map_err(|e| ApiError::validation_error(format!("Invalid metadata: {}", e)))?;

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let checksum = format!("{:x}", hasher.finalize());

    // Check if agent exists, create if not
    let agent_exists_query = db
        .client()
        .from("agents")
        .select("id")
        .eq("name", &metadata.name)
        .eq("user_id", auth_user.user_id.to_string())
        .execute()
        .await?;

    let _agent_id = if agent_exists_query.status() == 200 {
        let existing_agents: Vec<serde_json::Value> = agent_exists_query.json().await?;
        if existing_agents.is_empty() {
            // Create new agent
            let create_result = db
                .rpc_with_params("create_agent", json!({
                    "agent_name": metadata.name,
                    "description": metadata.description,
                    "author_name": "",
                    "tags": metadata.tags,
                    "keywords": Vec::<String>::new(),
                    "license": metadata.license.unwrap_or_default(),
                    "homepage": metadata.homepage.unwrap_or_default(),
                    "repository": metadata.repository.unwrap_or_default(),
                    "readme": metadata.readme.unwrap_or_default(),
                    "is_public": true
                }))
                .execute()
                .await?;

            let create_response: serde_json::Value = create_result.json().await?;
            if !create_response["success"].as_bool().unwrap_or(false) {
                return Err(ApiError::conflict_error(
                    create_response["error"]
                        .as_str()
                        .unwrap_or("Failed to create agent")
                ));
            }

            create_response["agent_id"]
                .as_str()
                .ok_or_else(|| ApiError::internal_error("Invalid create response"))?
                .to_string()
        } else {
            existing_agents[0]["id"]
                .as_str()
                .ok_or_else(|| ApiError::internal_error("Invalid agent data"))?
                .to_string()
        }
    } else {
        return Err(ApiError::internal_error("Failed to check agent existence"));
    };

    // Upload file to storage
    let file_path = format!(
        "{}/{}/{}/{}",
        auth_user.user_id,
        metadata.name,
        metadata.version,
        "agent.zip"
    );

    let storage_client = reqwest::Client::new();
    let upload_url = format!(
        "{}/object/{}/{}",
        db.storage_url(),
        config.upload.storage_bucket,
        file_path
    );

    let upload_response = storage_client
        .post(&upload_url)
        .header("Authorization", format!("Bearer {}", db.service_key()))
        .header("Content-Type", "application/zip")
        .body(content.to_vec())
        .send()
        .await?;

    if !upload_response.status().is_success() {
        return Err(ApiError::internal_error("Failed to upload file"));
    }

    // Publish the version
    let publish_result = db
        .rpc_with_params("publish_agent_version", json!({
            "agent_name": metadata.name,
            "version": metadata.version,
            "description": metadata.description,
            "changelog": "",
            "definition_data": json!({}),
            "package_data": json!({
                "file_name": "agent.zip",
                "file_size": content.len(),
                "checksum": checksum,
                "content_type": "application/zip"
            })
        }))
        .execute()
        .await?;

    let publish_response: serde_json::Value = publish_result.json().await?;
    if !publish_response["success"].as_bool().unwrap_or(false) {
        return Err(ApiError::conflict_error(
            publish_response["error"]
                .as_str()
                .unwrap_or("Failed to publish version")
        ));
    }

    // Get the published agent for response
    let agent_query = db
        .client()
        .from("agents")
        .select("*")
        .eq("name", &metadata.name)
        .eq("user_id", auth_user.user_id.to_string())
        .single()
        .execute()
        .await?;

    let agent_data: Option<Agent> = if agent_query.status() == 200 {
        let db_agent: DbAgent = agent_query.json().await?;
        Some(Agent::from(db_agent))
    } else {
        None
    };

    Ok(Json(PublishResponse {
        success: true,
        message: "Agent published successfully".to_string(),
        agent: agent_data,
    }))
}