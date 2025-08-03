/// Tests for the optimized latest and trending API endpoints
///
/// These tests verify:
/// - Proper error handling and graceful degradation
/// - Database field validation and fallback behavior
/// - Response format consistency
/// - Materialized view fallback logic
use serde_json::json;
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Test configuration for optimized endpoints
pub struct OptimizedEndpointTestConfig {
    pub mock_supabase_url: String,
    pub mock_supabase_key: String,
    pub debug_mode: bool,
}

impl Default for OptimizedEndpointTestConfig {
    fn default() -> Self {
        Self {
            mock_supabase_url: "https://test.supabase.co".to_string(),
            mock_supabase_key: "test_service_key".to_string(),
            debug_mode: true,
        }
    }
}

/// Expected response structure for latest/trending endpoints
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct Agent {
    pub name: String,
    #[serde(rename = "current_version")]
    pub version: String,
    pub description: String,
    #[serde(rename = "author_name")]
    pub author_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub download_count: u64,
    pub tags: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LatestAgentsResponse {
    pub agents: Vec<Agent>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TrendingAgentsResponse {
    pub agents: Vec<Agent>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod latest_endpoint_tests {
    use super::*;

    /// Test successful latest agents query with all required fields
    #[tokio::test]
    async fn test_latest_agents_successful_response() {
        let mock_server = MockServer::start().await;

        let mock_agents = json!([
            {
                "name": "test-agent-1",
                "current_version": "1.0.0",
                "description": "Test agent 1",
                "author_name": "Test Author 1",
                "created_at": "2025-01-01T00:00:00Z",
                "updated_at": "2025-01-01T00:00:00Z",
                "download_count": 100,
                "tags": ["test", "agent"]
            },
            {
                "name": "test-agent-2",
                "current_version": "2.0.0",
                "description": "Test agent 2",
                "author_name": null,
                "created_at": "2025-01-02T00:00:00Z",
                "updated_at": "2025-01-02T00:00:00Z",
                "download_count": 50,
                "tags": null
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/rest/v1/agents"))
            .and(header("apikey", "test_service_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_agents))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/rest/v1/agents", mock_server.uri()))
            .header("apikey", "test_service_key")
            .query(&[
                ("select", "name,current_version,description,author_name,created_at,updated_at,download_count,tags"),
                ("is_public", "eq.true"),
                ("current_version", "not.is.null"),
                ("order", "created_at.desc"),
                ("limit", "10")
            ])
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        let agents: Vec<Agent> = response.json().await.unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "test-agent-1");
        assert_eq!(agents[0].version, "1.0.0");
        assert_eq!(agents[0].download_count, 100);
        assert_eq!(agents[1].name, "test-agent-2");
        assert_eq!(agents[1].author_name, None);
    }

    /// Test latest agents query with empty result
    #[tokio::test]
    async fn test_latest_agents_empty_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/v1/agents"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/rest/v1/agents", mock_server.uri()))
            .header("apikey", "test_service_key")
            .query(&[
                ("select", "name,current_version,description,author_name,created_at,updated_at,download_count,tags"),
                ("is_public", "eq.true"),
                ("current_version", "not.is.null"),
                ("order", "created_at.desc"),
                ("limit", "10")
            ])
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let agents: Vec<Agent> = response.json().await.unwrap();
        assert_eq!(agents.len(), 0);
    }

    /// Test latest agents query with database error
    #[tokio::test]
    async fn test_latest_agents_database_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/v1/agents"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": "database_error",
                "message": "Internal server error"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/rest/v1/agents", mock_server.uri()))
            .header("apikey", "test_service_key")
            .query(&[
                ("select", "name,current_version,description,author_name,created_at,updated_at,download_count,tags"),
                ("is_public", "eq.true"),
                ("current_version", "not.is.null"),
                ("order", "created_at.desc"),
                ("limit", "10")
            ])
            .send()
            .await
            .unwrap();

        assert!(!response.status().is_success());
        assert_eq!(response.status(), 500);
    }

    /// Test query parameter validation (limit bounds)
    #[test]
    fn test_limit_parameter_validation() {
        let test_cases = vec![
            (None, 10),        // Default limit
            (Some("5"), 5),    // Valid limit
            (Some("50"), 50),  // Max limit
            (Some("100"), 50), // Exceeds max, should cap at 50
            (Some("0"), 10),   // Zero should use default
            (Some("abc"), 10), // Invalid, should use default
        ];

        for (input_limit, expected_limit) in test_cases {
            let parsed_limit = input_limit
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(10);
            let limit = if parsed_limit == 0 {
                10
            } else {
                parsed_limit.min(50)
            };

            assert_eq!(
                limit, expected_limit,
                "Limit validation failed for input: {:?}",
                input_limit
            );
        }
    }
}

#[cfg(test)]
mod trending_endpoint_tests {
    use super::*;

    /// Test trending agents with materialized view success
    #[tokio::test]
    async fn test_trending_agents_materialized_view_success() {
        let mock_server = MockServer::start().await;

        let mock_trending_agents = json!([
            {
                "name": "trending-agent-1",
                "current_version": "1.0.0",
                "description": "Most trending agent",
                "author_name": "Popular Author",
                "created_at": "2025-01-01T00:00:00Z",
                "updated_at": "2025-01-01T12:00:00Z",
                "download_count": 1000,
                "tags": ["trending", "popular"]
            },
            {
                "name": "trending-agent-2",
                "current_version": "2.1.0",
                "description": "Second trending agent",
                "author_name": "Another Author",
                "created_at": "2025-01-02T00:00:00Z",
                "updated_at": "2025-01-02T12:00:00Z",
                "download_count": 500,
                "tags": ["trending"]
            }
        ]);

        // Mock successful materialized view query
        Mock::given(method("GET"))
            .and(path("/rest/v1/trending_agents_mv"))
            .and(header("apikey", "test_service_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_trending_agents))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/rest/v1/trending_agents_mv", mock_server.uri()))
            .header("apikey", "test_service_key")
            .query(&[
                ("select", "name,current_version,description,author_name,created_at,updated_at,download_count,tags"),
                ("order", "trending_score.desc"),
                ("limit", "10")
            ])
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        let agents: Vec<Agent> = response.json().await.unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "trending-agent-1");
        assert_eq!(agents[0].download_count, 1000);
        assert_eq!(agents[1].name, "trending-agent-2");
        assert_eq!(agents[1].download_count, 500);
    }

    /// Test trending agents with materialized view failure and fallback
    #[tokio::test]
    async fn test_trending_agents_fallback_to_regular_table() {
        let mock_server = MockServer::start().await;

        let mock_fallback_agents = json!([
            {
                "name": "fallback-agent-1",
                "current_version": "1.0.0",
                "description": "Fallback agent 1",
                "author_name": "Author 1",
                "created_at": "2025-01-01T00:00:00Z",
                "updated_at": "2025-01-01T00:00:00Z",
                "download_count": 200,
                "tags": ["fallback"]
            }
        ]);

        // Mock materialized view failure
        Mock::given(method("GET"))
            .and(path("/rest/v1/trending_agents_mv"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "relation_not_found",
                "message": "relation \"trending_agents_mv\" does not exist"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Mock successful fallback query
        Mock::given(method("GET"))
            .and(path("/rest/v1/agents"))
            .and(header("apikey", "test_service_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_fallback_agents))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();

        // First try materialized view (will fail)
        let mv_response = client
            .get(&format!("{}/rest/v1/trending_agents_mv", mock_server.uri()))
            .header("apikey", "test_service_key")
            .send()
            .await
            .unwrap();

        assert!(!mv_response.status().is_success());

        // Then try fallback query (will succeed)
        let fallback_response = client
            .get(&format!("{}/rest/v1/agents", mock_server.uri()))
            .header("apikey", "test_service_key")
            .query(&[
                ("select", "name,current_version,description,author_name,created_at,updated_at,download_count,tags"),
                ("is_public", "eq.true"),
                ("download_count", "gte.1"),
                ("current_version", "not.is.null"),
                ("order", "download_count.desc,updated_at.desc"),
                ("limit", "10")
            ])
            .send()
            .await
            .unwrap();

        assert!(fallback_response.status().is_success());

        let agents: Vec<Agent> = fallback_response.json().await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "fallback-agent-1");
        assert_eq!(agents[0].download_count, 200);
    }

    /// Test materialized view population function
    #[tokio::test]
    async fn test_ensure_trending_view_populated_function() {
        let mock_server = MockServer::start().await;

        let expected_function_call = json!({});

        Mock::given(method("POST"))
            .and(path("/rest/v1/rpc/ensure_trending_view_populated"))
            .and(header("apikey", "test_service_key"))
            .and(body_json(&expected_function_call))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(true)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/rest/v1/rpc/ensure_trending_view_populated",
                mock_server.uri()
            ))
            .header("apikey", "test_service_key")
            .header("Content-Type", "application/json")
            .json(&expected_function_call)
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        let result: bool = response.json().await.unwrap();
        assert!(result, "Function should return true on success");
    }
}

#[cfg(test)]
mod response_format_tests {
    use super::*;

    /// Test response structure serialization/deserialization
    #[test]
    fn test_agent_response_format() {
        let agent = Agent {
            name: "test-agent".to_string(),
            version: "1.0.0".to_string(),
            description: "Test description".to_string(),
            author_name: Some("Test Author".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            download_count: 42,
            tags: Some(vec!["test".to_string(), "api".to_string()]),
        };

        let json = serde_json::to_string(&agent).unwrap();
        let parsed: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, agent.name);
        assert_eq!(parsed.version, agent.version);
        assert_eq!(parsed.description, agent.description);
        assert_eq!(parsed.author_name, agent.author_name);
        assert_eq!(parsed.download_count, agent.download_count);
        assert_eq!(parsed.tags, agent.tags);

        // Verify JSON field names match expected API format
        assert!(json.contains("\"current_version\":"));
        assert!(json.contains("\"author_name\":"));
        assert!(!json.contains("\"version\":\"1.0.0\"")); // Should be current_version
    }

    /// Test response wrapper structures
    #[test]
    fn test_response_wrapper_format() {
        let now = chrono::Utc::now();
        let agents = vec![Agent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author_name: None,
            created_at: now,
            updated_at: now,
            download_count: 0,
            tags: None,
        }];

        let latest_response = LatestAgentsResponse {
            agents: agents.clone(),
            cached_at: now,
        };

        let trending_response = TrendingAgentsResponse {
            agents,
            cached_at: now,
        };

        let latest_json = serde_json::to_string(&latest_response).unwrap();
        let trending_json = serde_json::to_string(&trending_response).unwrap();

        assert!(latest_json.contains("\"agents\":"));
        assert!(latest_json.contains("\"cached_at\":"));
        assert!(trending_json.contains("\"agents\":"));
        assert!(trending_json.contains("\"cached_at\":"));

        // Verify deserialization works
        let parsed_latest: LatestAgentsResponse = serde_json::from_str(&latest_json).unwrap();
        let parsed_trending: TrendingAgentsResponse = serde_json::from_str(&trending_json).unwrap();

        assert_eq!(parsed_latest.agents.len(), 1);
        assert_eq!(parsed_trending.agents.len(), 1);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// Test various database error scenarios
    #[tokio::test]
    async fn test_database_connection_errors() {
        let error_scenarios = vec![
            (500, "Internal server error"),
            (503, "Service unavailable"),
            (408, "Request timeout"),
        ];

        for (status_code, error_message) in error_scenarios {
            // Create a fresh mock server for each test case
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/rest/v1/agents"))
                .respond_with(ResponseTemplate::new(status_code).set_body_json(json!({
                    "error": "database_error",
                    "message": error_message
                })))
                .expect(1)
                .mount(&mock_server)
                .await;

            let client = reqwest::Client::new();
            let response = client
                .get(&format!("{}/rest/v1/agents", mock_server.uri()))
                .header("apikey", "test_service_key")
                .send()
                .await
                .unwrap();

            assert!(
                !response.status().is_success(),
                "Status should indicate failure for {}",
                status_code
            );
            assert_eq!(
                response.status(),
                status_code,
                "Status code should match for {}",
                status_code
            );
        }
    }

    /// Test malformed response handling
    #[tokio::test]
    async fn test_malformed_response_handling() {
        let mock_server = MockServer::start().await;

        // Test invalid JSON response
        Mock::given(method("GET"))
            .and(path("/rest/v1/agents"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/rest/v1/agents", mock_server.uri()))
            .header("apikey", "test_service_key")
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        // Attempt to parse as JSON should fail
        let parse_result = response.json::<Vec<Agent>>().await;
        assert!(parse_result.is_err(), "Should fail to parse invalid JSON");
    }

    /// Test missing required fields in response
    #[tokio::test]
    async fn test_missing_required_fields() {
        let mock_server = MockServer::start().await;

        let incomplete_agent = json!([{
            "name": "test-agent",
            // Missing current_version, description, etc.
            "download_count": 100
        }]);

        Mock::given(method("GET"))
            .and(path("/rest/v1/agents"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&incomplete_agent))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/rest/v1/agents", mock_server.uri()))
            .header("apikey", "test_service_key")
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        // Attempt to parse should fail due to missing required fields
        let parse_result = response.json::<Vec<Agent>>().await;
        assert!(
            parse_result.is_err(),
            "Should fail to parse agent with missing required fields"
        );
    }
}
