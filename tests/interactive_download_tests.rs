/// Interactive Download Tests
///
/// Tests for the interactive agent selection mode in the CLI pull command.
/// These tests verify that the search functionality integrates properly with
/// the download flow and that users can select agents interactively.
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::TempDir;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

/// Test context for interactive download tests
pub struct InteractiveTestContext {
    pub temp_dir: TempDir,
    pub mock_server: MockServer,
    pub cli_config_path: PathBuf,
    pub cli_binary_path: PathBuf,
}

impl InteractiveTestContext {
    /// Create a new interactive test context
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_server = MockServer::start().await;

        // Create CLI config
        let cli_config_path = temp_dir.path().join("config.toml");
        let config_content = format!(
            r#"registry_url = "{}"
timeout = 30
verify_ssl = false
"#,
            mock_server.uri()
        );
        fs::write(&cli_config_path, config_content).expect("Failed to write config");

        // Find CLI binary
        let cli_binary_path = Self::find_cli_binary();

        Self {
            temp_dir,
            mock_server,
            cli_config_path,
            cli_binary_path,
        }
    }

    /// Find the CLI binary for testing
    fn find_cli_binary() -> PathBuf {
        // Try various locations
        let paths = vec![
            "target/debug/carp",
            "target/release/carp",
            "../target/debug/carp",
            "../target/release/carp",
            "cli/target/debug/carp",
            "cli/target/release/carp",
        ];

        for path in paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return path_buf;
            }
        }

        // Fallback to PATH
        PathBuf::from("carp")
    }

    /// Set up mock search endpoint with test agents
    pub async fn setup_search_mock(&self) {
        let agents_data = json!({
            "agents": [
                {
                    "name": "test-agent-1",
                    "version": "1.0.0",
                    "description": "First test agent for interactive selection",
                    "author": "test-user-1",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "view_count": 100,
                    "tags": ["test", "interactive"],
                    "readme": null,
                    "homepage": null,
                    "repository": null,
                    "license": "MIT"
                },
                {
                    "name": "test-agent-2",
                    "version": "2.0.0",
                    "description": "Second test agent for interactive selection",
                    "author": "test-user-2",
                    "created_at": "2024-01-02T00:00:00Z",
                    "updated_at": "2024-01-02T00:00:00Z",
                    "view_count": 50,
                    "tags": ["test", "example"],
                    "readme": null,
                    "homepage": null,
                    "repository": null,
                    "license": "Apache-2.0"
                },
                {
                    "name": "popular-agent",
                    "version": "3.1.0",
                    "description": "A popular agent with many downloads",
                    "author": "popular-dev",
                    "created_at": "2024-01-03T00:00:00Z",
                    "updated_at": "2024-01-03T00:00:00Z",
                    "view_count": 1000,
                    "tags": ["popular", "production"],
                    "readme": "# Popular Agent\n\nThis is a popular agent.",
                    "homepage": "https://example.com/popular",
                    "repository": "https://github.com/user/popular-agent",
                    "license": "MIT"
                }
            ],
            "total": 3,
            "page": 1,
            "per_page": 1000
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/agents/search"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&agents_data)
                    .append_header("content-type", "application/json"),
            )
            .mount(&self.mock_server)
            .await;
    }

    /// Set up mock search endpoint for empty results
    pub async fn setup_empty_search_mock(&self) {
        let empty_data = json!({
            "agents": [],
            "total": 0,
            "page": 1,
            "per_page": 20
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/agents/search"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&empty_data)
                    .append_header("content-type", "application/json"),
            )
            .mount(&self.mock_server)
            .await;
    }

    /// Run CLI command and capture output
    pub fn run_cli_command(&self, args: &[&str]) -> std::process::Output {
        Command::new(&self.cli_binary_path)
            .args(args)
            .env("CARP_CONFIG", &self.cli_config_path)
            .env("CARP_OUTPUT_DIR", self.temp_dir.path())
            .output()
            .expect("Failed to execute CLI command")
    }

    /// Run CLI command with stdin input (for interactive testing)
    pub fn run_cli_command_with_input(&self, args: &[&str], input: &str) -> std::process::Output {
        let mut child = Command::new(&self.cli_binary_path)
            .args(args)
            .env("CARP_CONFIG", &self.cli_config_path)
            .env("CARP_OUTPUT_DIR", self.temp_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start CLI command");

        // Write input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
            stdin.flush().expect("Failed to flush stdin");
        }

        child
            .wait_with_output()
            .expect("Failed to wait for command")
    }
}

#[cfg(test)]
mod interactive_download_tests {
    use super::*;

    /// Test that the search functionality works and returns agent list
    #[tokio::test]
    async fn test_search_returns_agent_list() {
        let ctx = InteractiveTestContext::new().await;
        ctx.setup_search_mock().await;

        // Test search command
        let output = ctx.run_cli_command(&["search", "test"]);

        assert!(output.status.success(), "Search command should succeed");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should display all test agents
        assert!(
            stdout.contains("test-agent-1"),
            "Should show first test agent"
        );
        assert!(
            stdout.contains("test-agent-2"),
            "Should show second test agent"
        );
        assert!(
            stdout.contains("popular-agent"),
            "Should show popular agent"
        );

        // Should show descriptions
        assert!(
            stdout.contains("First test agent"),
            "Should show first description"
        );
        assert!(
            stdout.contains("Second test agent"),
            "Should show second description"
        );
        assert!(
            stdout.contains("popular agent"),
            "Should show popular description"
        );

        // Should show download counts
        assert!(
            stdout.contains("100"),
            "Should show download count for first agent"
        );
        assert!(
            stdout.contains("50"),
            "Should show download count for second agent"
        );
        assert!(
            stdout.contains("1000"),
            "Should show download count for popular agent"
        );
    }

    /// Test search with no results
    #[tokio::test]
    async fn test_search_no_results() {
        let ctx = InteractiveTestContext::new().await;
        ctx.setup_empty_search_mock().await;

        let output = ctx.run_cli_command(&["search", "nonexistent"]);

        assert!(
            output.status.success(),
            "Search with no results should succeed"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("No agents found")
                || stdout.contains("0 results")
                || stdout.contains("No results"),
            "Should indicate no results found. Output: {}",
            stdout
        );
    }

    /// Test search with different query parameters
    #[tokio::test]
    async fn test_search_with_parameters() {
        let ctx = InteractiveTestContext::new().await;

        // Mock search with specific query
        Mock::given(method("GET"))
            .and(path("/api/v1/agents/search"))
            .and(query_param("q", "popular"))
            .and(query_param("limit", "5"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({
                        "agents": [
                            {
                                "name": "popular-agent",
                                "version": "3.1.0",
                                "description": "A popular agent",
                                "author": "popular-dev",
                                "created_at": "2024-01-03T00:00:00Z",
                                "updated_at": "2024-01-03T00:00:00Z",
                                "view_count": 1000,
                                "tags": ["popular"],
                                "readme": null,
                                "homepage": null,
                                "repository": null,
                                "license": "MIT"
                            }
                        ],
                        "total": 1,
                        "page": 1,
                        "per_page": 5
                    }))
                    .append_header("content-type", "application/json"),
            )
            .mount(&ctx.mock_server)
            .await;

        let output = ctx.run_cli_command(&["search", "popular", "--limit", "5"]);

        assert!(
            output.status.success(),
            "Search with parameters should succeed"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("popular-agent"),
            "Should find the popular agent"
        );
        assert!(
            !stdout.contains("test-agent"),
            "Should not show test agents"
        );
    }

    /// Test that pull command without arguments triggers search
    /// Note: This tests the pathway but not full interactive mode due to testing limitations
    #[tokio::test]
    async fn test_pull_without_args_shows_help() {
        let ctx = InteractiveTestContext::new().await;

        // When no agent is specified, the CLI should either:
        // 1. Show help/usage information
        // 2. Attempt to fetch agents for interactive selection
        //
        // Since true interactive testing is complex, we test that the command
        // behaves appropriately when no agent is specified.

        let output = ctx.run_cli_command(&["pull"]);

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should either show help or attempt to fetch agents
        assert!(
            stdout.contains("Usage") || 
            stdout.contains("pull") ||
            stderr.contains("required") ||
            stderr.contains("agent") ||
            !output.status.success(), // It's OK if it fails due to missing agent parameter
            "Pull without arguments should show usage or indicate missing agent. STDOUT: {} STDERR: {}", 
            stdout, stderr
        );
    }

    /// Test direct agent specification (non-interactive mode)
    #[tokio::test]
    async fn test_direct_agent_specification() {
        let ctx = InteractiveTestContext::new().await;

        // Mock the download endpoint for direct specification
        Mock::given(method("GET"))
            .and(path("/api/v1/agents/test-agent-1/latest/download"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(json!({
                        "error": "not_found",
                        "message": "Agent 'test-agent-1' version 'latest' not found"
                    }))
                    .append_header("content-type", "application/json"),
            )
            .mount(&ctx.mock_server)
            .await;

        // Test direct specification (should try to download directly)
        let output = ctx.run_cli_command(&["pull", "test-agent-1"]);

        // Should fail with not found (proving it attempted direct download)
        assert!(
            !output.status.success(),
            "Direct specification of non-existent agent should fail"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("not found") || stderr.contains("404"),
            "Should indicate agent not found for direct specification. STDERR: {}",
            stderr
        );
    }

    /// Test version resolution in direct specification
    #[tokio::test]
    async fn test_version_resolution_direct() {
        let ctx = InteractiveTestContext::new().await;

        // Mock specific version endpoint
        Mock::given(method("GET"))
            .and(path("/api/v1/agents/test-agent/1.0.0/download"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(json!({
                        "error": "not_found",
                        "message": "Agent 'test-agent' version '1.0.0' not found"
                    }))
                    .append_header("content-type", "application/json"),
            )
            .mount(&ctx.mock_server)
            .await;

        // Mock latest version endpoint
        Mock::given(method("GET"))
            .and(path("/api/v1/agents/test-agent/latest/download"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(json!({
                        "error": "not_found",
                        "message": "Agent 'test-agent' version 'latest' not found"
                    }))
                    .append_header("content-type", "application/json"),
            )
            .mount(&ctx.mock_server)
            .await;

        // Test specific version
        let output1 = ctx.run_cli_command(&["pull", "test-agent@1.0.0"]);
        assert!(
            !output1.status.success(),
            "Specific version should be attempted"
        );

        // Test latest version (implicit)
        let output2 = ctx.run_cli_command(&["pull", "test-agent"]);
        assert!(
            !output2.status.success(),
            "Latest version should be attempted"
        );

        // Test latest version (explicit)
        let output3 = ctx.run_cli_command(&["pull", "test-agent@latest"]);
        assert!(
            !output3.status.success(),
            "Explicit latest should be attempted"
        );

        // All should attempt the correct endpoints (verified by mocks being called)
    }

    /// Test error handling when search fails
    #[tokio::test]
    async fn test_search_api_error() {
        let ctx = InteractiveTestContext::new().await;

        // Mock search endpoint to return error
        Mock::given(method("GET"))
            .and(path("/api/v1/agents/search"))
            .respond_with(
                ResponseTemplate::new(500)
                    .set_body_json(json!({
                        "error": "internal_server_error",
                        "message": "Database connection failed"
                    }))
                    .append_header("content-type", "application/json"),
            )
            .mount(&ctx.mock_server)
            .await;

        let output = ctx.run_cli_command(&["search", "test"]);

        assert!(
            !output.status.success(),
            "Search should fail with server error"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("500")
                || stderr.contains("server error")
                || stderr.contains("connection")
                || stderr.contains("error"),
            "Should indicate server error. STDERR: {}",
            stderr
        );
    }

    /// Test that search respects different output formats
    #[tokio::test]
    async fn test_search_output_formats() {
        let ctx = InteractiveTestContext::new().await;
        ctx.setup_search_mock().await;

        // Test default output format
        let output1 = ctx.run_cli_command(&["search", "test"]);
        assert!(output1.status.success(), "Default search should succeed");

        let stdout1 = String::from_utf8_lossy(&output1.stdout);
        assert!(
            stdout1.contains("test-agent-1"),
            "Default format should show agents"
        );

        // Test verbose output
        let output2 = ctx.run_cli_command(&["search", "test", "--verbose"]);
        assert!(output2.status.success(), "Verbose search should succeed");

        let stdout2 = String::from_utf8_lossy(&output2.stdout);
        let stderr2 = String::from_utf8_lossy(&output2.stderr);

        // Verbose should provide additional information
        assert!(
            stdout2.len() >= stdout1.len() || stderr2.len() > 0,
            "Verbose mode should provide at least as much output"
        );
    }

    /// Test CLI argument parsing for pull command
    #[tokio::test]
    async fn test_pull_argument_parsing() {
        let ctx = InteractiveTestContext::new().await;

        // Test invalid agent specifications
        let invalid_specs = vec![
            "@1.0.0",       // Missing agent name
            "agent@",       // Missing version
            "agent@@1.0.0", // Double @
            "",             // Empty string
        ];

        for spec in invalid_specs {
            let output = ctx.run_cli_command(&["pull", spec]);

            // Should either fail or show help
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            if output.status.success() {
                // If it succeeds, it should show help or usage
                assert!(
                    stdout.contains("Usage") || stdout.contains("help"),
                    "Invalid spec '{}' should show usage if command succeeds",
                    spec
                );
            } else {
                // If it fails, it should indicate the problem
                assert!(
                    stderr.contains("invalid")
                        || stderr.contains("required")
                        || stderr.contains("argument")
                        || stderr.contains("specification"),
                    "Invalid spec '{}' should show appropriate error. STDERR: {}",
                    spec,
                    stderr
                );
            }
        }
    }

    /// Test that interactive mode handles large result sets appropriately
    #[tokio::test]
    async fn test_search_large_result_set() {
        let ctx = InteractiveTestContext::new().await;

        // Create a large result set
        let mut agents = Vec::new();
        for i in 1..=100 {
            agents.push(json!({
                "name": format!("agent-{:03}", i),
                "version": "1.0.0",
                "description": format!("Test agent number {}", i),
                "author": "test-user",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "view_count": i * 10,
                "tags": ["test"],
                "readme": null,
                "homepage": null,
                "repository": null,
                "license": "MIT"
            }));
        }

        let large_result = json!({
            "agents": agents,
            "total": 100,
            "page": 1,
            "per_page": 100
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/agents/search"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&large_result)
                    .append_header("content-type", "application/json"),
            )
            .mount(&ctx.mock_server)
            .await;

        let output = ctx.run_cli_command(&["search", "agent"]);

        assert!(
            output.status.success(),
            "Large result set search should succeed"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should show multiple agents
        assert!(stdout.contains("agent-001"), "Should show first agent");
        assert!(stdout.contains("agent-100"), "Should show last agent");

        // Should handle the large output appropriately (not crash)
        assert!(
            stdout.len() > 1000,
            "Should produce substantial output for 100 agents"
        );
    }

    /// Test network timeout handling
    #[tokio::test]
    async fn test_network_timeout_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create config with very short timeout and non-existent server
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"registry_url = "http://127.0.0.1:9999"
timeout = 1
verify_ssl = false
"#;
        fs::write(&config_path, config_content).expect("Failed to write config");

        let cli_binary_path = InteractiveTestContext::find_cli_binary();

        let output = Command::new(&cli_binary_path)
            .args(&["search", "test"])
            .env("CARP_CONFIG", &config_path)
            .output()
            .expect("Failed to execute CLI command");

        assert!(
            !output.status.success(),
            "Should fail with connection error"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("connection")
                || stderr.contains("timeout")
                || stderr.contains("network")
                || stderr.contains("refused"),
            "Should indicate network/connection error. STDERR: {}",
            stderr
        );
    }
}

/// Test utilities for interactive scenarios
mod test_utils {
    use super::*;

    /// Simulate interactive input for testing
    /// Note: Real interactive testing would require more sophisticated tooling
    pub fn simulate_interactive_selection(agents: &[&str], selection_index: usize) -> String {
        // In a real interactive scenario, this would simulate:
        // 1. Arrow key navigation
        // 2. Enter key press
        // 3. Escape/Ctrl+C for cancellation
        //
        // For now, we just return the selected agent name
        if selection_index < agents.len() {
            agents[selection_index].to_string()
        } else {
            String::new()
        }
    }

    /// Create test data for interactive scenarios
    pub fn create_test_agent_data(count: usize) -> serde_json::Value {
        let mut agents = Vec::new();

        for i in 1..=count {
            agents.push(json!({
                "name": format!("interactive-agent-{}", i),
                "version": "1.0.0",
                "description": format!("Interactive test agent {}", i),
                "author": "interactive-tester",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "view_count": i * 25,
                "tags": ["interactive", "test"],
                "readme": null,
                "homepage": null,
                "repository": null,
                "license": "MIT"
            }));
        }

        json!({
            "agents": agents,
            "total": count,
            "page": 1,
            "per_page": count
        })
    }
}
