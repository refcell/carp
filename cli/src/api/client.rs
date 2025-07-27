use crate::api::types::*;
use crate::config::Config;
use crate::utils::error::{CarpError, CarpResult};
use reqwest::{Client, ClientBuilder, Response};
use std::time::Duration;

/// HTTP client for interacting with the Carp registry API
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_token: Option<String>,
}

impl ApiClient {
    /// Create a new API client from configuration
    pub fn new(config: &Config) -> CarpResult<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent(format!("carp-cli/{}", env!("CARGO_PKG_VERSION")))
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()?;

        Ok(Self {
            client,
            base_url: config.registry_url.clone(),
            api_token: config.api_token.clone(),
        })
    }

    /// Search for agents in the registry
    pub async fn search(&self, query: &str, limit: Option<usize>, exact: bool) -> CarpResult<SearchResponse> {
        let url = format!("{}/api/v1/agents/search", self.base_url);
        let mut params = vec![("q", query)];

        let limit_str;
        if let Some(limit) = limit {
            limit_str = limit.to_string();
            params.push(("limit", &limit_str));
        }

        if exact {
            params.push(("exact", "true"));
        }

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get download information for a specific agent
    pub async fn get_agent_download(&self, name: &str, version: Option<&str>) -> CarpResult<AgentDownload> {
        let version = version.unwrap_or("latest");
        let url = format!("{}/api/v1/agents/{}/{}/download", self.base_url, name, version);

        println!("Sending GET request to: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Download agent content
    pub async fn download_agent(&self, download_url: &str) -> CarpResult<bytes::Bytes> {
        let response = self.client
            .get(download_url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CarpError::Api {
                status: response.status().as_u16(),
                message: "Failed to download agent".to_string(),
            });
        }

        let bytes = response.bytes().await?;
        Ok(bytes)
    }

    /// Publish an agent to the registry
    pub async fn publish(&self, request: PublishRequest, content: Vec<u8>) -> CarpResult<PublishResponse> {
        let token = self.api_token.as_ref()
            .ok_or_else(|| CarpError::Auth("No API token configured. Please login first.".to_string()))?;

        let url = format!("{}/api/v1/agents/publish", self.base_url);

        // Create multipart form with metadata and content
        let form = reqwest::multipart::Form::new()
            .text("metadata", serde_json::to_string(&request)?)
            .part("content", reqwest::multipart::Part::bytes(content)
                .file_name("agent.zip")
                .mime_str("application/zip")?);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .multipart(form)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Authenticate with the registry
    pub async fn authenticate(&self, username: &str, password: &str) -> CarpResult<AuthResponse> {
        let url = format!("{}/api/v1/auth/login", self.base_url);
        let request = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response, parsing JSON or error
    async fn handle_response<T>(&self, response: Response) -> CarpResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&text).map_err(CarpError::from)
        } else {
            // Try to parse as API error, fallback to generic error
            match serde_json::from_str::<ApiError>(&text) {
                Ok(api_error) => Err(CarpError::Api {
                    status: status.as_u16(),
                    message: api_error.message,
                }),
                Err(_) => Err(CarpError::Api {
                    status: status.as_u16(),
                    message: text,
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_search_request() {
        let mut server = Server::new_async().await;
        let config = Config {
            registry_url: server.url(),
            api_token: None,
            timeout: 30,
            verify_ssl: true,
            default_output_dir: None,
        };

        let _m = server.mock("GET", "/api/v1/agents/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"agents": [], "total": 0, "page": 1, "per_page": 10}"#)
            .create_async()
            .await;

        let client = ApiClient::new(&config).unwrap();
        let result = client.search("test", Some(10), false).await;

        match &result {
            Ok(_) => (),
            Err(e) => println!("Test error: {:?}", e),
        }
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.agents.len(), 0);
        assert_eq!(response.total, 0);
    }
}
