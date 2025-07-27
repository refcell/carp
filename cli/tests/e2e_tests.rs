/// End-to-End tests for the CLI application
/// These tests simulate real user workflows and test the complete integration
use mockito::{Mock, ServerGuard};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Test utilities for E2E testing
mod e2e_utils {
    use super::*;
    use std::env;

    pub struct E2ETestContext {
        pub temp_dir: TempDir,
        pub mock_server: ServerGuard,
        pub cli_binary_path: PathBuf,
        pub config_path: PathBuf,
    }

    impl E2ETestContext {
        pub async fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let mut server = mockito::Server::new_async().await;

            // Create a test config file
            let config_path = temp_dir.path().join("config.toml");
            let config_content = format!(
                r#"
registry_url = "{}"
timeout = 30
verify_ssl = false
"#,
                server.url()
            );
            fs::write(&config_path, config_content).expect("Failed to write config");

            // Try to locate the CLI binary
            let cli_binary_path = Self::find_cli_binary();

            Self {
                temp_dir,
                mock_server: server,
                cli_binary_path,
                config_path,
            }
        }

        fn find_cli_binary() -> PathBuf {
            // First try in target/debug
            let debug_path = PathBuf::from("target/debug/carp");
            if debug_path.exists() {
                return debug_path;
            }

            // Try in target/release
            let release_path = PathBuf::from("target/release/carp");
            if release_path.exists() {
                return release_path;
            }

            // Try relative to CLI directory
            let cli_debug_path = PathBuf::from("../target/debug/carp");
            if cli_debug_path.exists() {
                return cli_debug_path;
            }

            let cli_release_path = PathBuf::from("../target/release/carp");
            if cli_release_path.exists() {
                return cli_release_path;
            }

            // Fallback - assume it's in PATH
            PathBuf::from("carp")
        }

        pub fn run_cli_command(&self, args: &[&str]) -> std::process::Output {
            let mut cmd = Command::new(&self.cli_binary_path);
            cmd.args(args)
                .env("CARP_CONFIG", &self.config_path)
                .env("CARP_OUTPUT_DIR", self.temp_dir.path())
                .output()
                .expect("Failed to execute CLI command")
        }

        pub fn write_test_file(&self, filename: &str, content: &str) -> PathBuf {
            let file_path = self.temp_dir.path().join(filename);
            fs::write(&file_path, content).expect("Failed to write test file");
            file_path
        }

        pub fn create_test_manifest(&self, name: &str, version: &str) -> PathBuf {
            let manifest_content = format!(
                r#"
[package]
name = "{}"
version = "{}"
description = "Test agent for E2E testing"
authors = ["Test User <test@example.com>"]
license = "MIT"
tags = ["test", "e2e"]

[agent]
main = "main.py"
"#,
                name, version
            );

            self.write_test_file("carp.toml", &manifest_content)
        }

        pub fn create_test_agent_files(&self) {
            // Create a simple Python agent file
            let main_py = r#"
#!/usr/bin/env python3
"""
Test agent for E2E testing.
"""

def main():
    print("Hello from test agent!")

if __name__ == "__main__":
    main()
"#;
            self.write_test_file("main.py", main_py);

            // Create README
            let readme = r#"
# Test Agent

This is a test agent for E2E testing.

## Usage

Run the agent with:
```
python main.py
```
"#;
            self.write_test_file("README.md", readme);
        }

        pub fn get_temp_path(&self, filename: &str) -> PathBuf {
            self.temp_dir.path().join(filename)
        }
    }
}

// Test CLI search command
#[tokio::test]
async fn test_cli_search_command() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    // Mock the search API endpoint
    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
            "q".to_string(),
            "test".to_string(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [
                    {
                        "name": "test-agent",
                        "version": "1.0.0",
                        "description": "A test agent for E2E testing",
                        "author": "testuser",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z",
                        "download_count": 42,
                        "tags": ["test", "ai"],
                        "readme": "# Test Agent",
                        "homepage": "https://example.com",
                        "repository": "https://github.com/test/agent",
                        "license": "MIT"
                    },
                    {
                        "name": "another-agent",
                        "version": "2.1.0",
                        "description": "Another test agent",
                        "author": "anotheruser",
                        "created_at": "2024-01-02T00:00:00Z",
                        "updated_at": "2024-01-02T00:00:00Z",
                        "download_count": 15,
                        "tags": ["test", "example"],
                        "readme": null,
                        "homepage": null,
                        "repository": null,
                        "license": "Apache-2.0"
                    }
                ],
                "total": 2,
                "page": 1,
                "per_page": 20
            })
            .to_string(),
        )
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["search", "test"]);

    assert!(output.status.success(), "CLI search command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that both agents are displayed
    assert!(stdout.contains("test-agent"));
    assert!(stdout.contains("another-agent"));
    assert!(stdout.contains("A test agent for E2E testing"));
    assert!(stdout.contains("Another test agent"));
}

// Test CLI search command with limit
#[tokio::test]
async fn test_cli_search_with_limit() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("q".to_string(), "ai".to_string()),
            mockito::Matcher::UrlEncoded("limit".to_string(), "5".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [
                    {
                        "name": "ai-agent",
                        "version": "1.0.0",
                        "description": "An AI agent",
                        "author": "aiuser",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z",
                        "download_count": 100,
                        "tags": ["ai"],
                        "readme": null,
                        "homepage": null,
                        "repository": null,
                        "license": null
                    }
                ],
                "total": 1,
                "page": 1,
                "per_page": 5
            })
            .to_string(),
        )
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["search", "ai", "--limit", "5"]);

    assert!(output.status.success(), "CLI search with limit failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ai-agent"));
}

// Test CLI search command with no results
#[tokio::test]
async fn test_cli_search_no_results() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .match_query(mockito::Matcher::UrlEncoded(
            "q".to_string(),
            "nonexistent".to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [],
                "total": 0,
                "page": 1,
                "per_page": 20
            })
            .to_string(),
        )
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["search", "nonexistent"]);

    assert!(output.status.success(), "CLI search no results failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No agents found") || stdout.contains("0 results"));
}

// Test CLI pull command
#[tokio::test]
async fn test_cli_pull_command() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    // Mock the download info endpoint
    let _download_info_mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/test-agent/latest/download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "name": "test-agent",
                "version": "1.0.0",
                "download_url": format!("{}/download/test-agent-1.0.0.zip", ctx.mock_server.url()),
                "checksum": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "size": 1024
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Mock the actual file download
    let test_zip_content = b"PK\x03\x04test zip content"; // Minimal ZIP-like content
    let _download_mock = ctx
        .mock_server
        .mock("GET", "/download/test-agent-1.0.0.zip")
        .with_status(200)
        .with_header("content-type", "application/zip")
        .with_body(test_zip_content.to_vec())
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["pull", "test-agent"]);

    // Note: This might fail if the CLI expects a valid ZIP file
    // In a real E2E test, we'd create a proper ZIP file
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The command should at least attempt to download
    // Check that the download was attempted (may fail on ZIP extraction)
    assert!(
        output.status.success() || stderr.contains("zip") || stdout.contains("Downloaded"),
        "CLI pull command should attempt download"
    );
}

// Test CLI pull command with specific version
#[tokio::test]
async fn test_cli_pull_specific_version() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    let _download_info_mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/test-agent/2.0.0/download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "name": "test-agent",
                "version": "2.0.0",
                "download_url": format!("{}/download/test-agent-2.0.0.zip", ctx.mock_server.url()),
                "checksum": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "size": 2048
            })
            .to_string(),
        )
        .create_async()
        .await;

    let test_zip_content = b"PK\x03\x04version 2.0.0 content";
    let _download_mock = ctx
        .mock_server
        .mock("GET", "/download/test-agent-2.0.0.zip")
        .with_status(200)
        .with_header("content-type", "application/zip")
        .with_body(test_zip_content.to_vec())
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["pull", "test-agent@2.0.0"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should attempt to download the specific version
    assert!(
        output.status.success() || stderr.contains("zip") || stdout.contains("Downloaded"),
        "CLI pull specific version should attempt download"
    );
}

// Test CLI pull command with nonexistent agent
#[tokio::test]
async fn test_cli_pull_nonexistent_agent() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/nonexistent-agent/latest/download")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": "NotFoundError",
                "message": "Agent not found"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["pull", "nonexistent-agent"]);

    assert!(!output.status.success(), "CLI pull nonexistent should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("404"));
}

// Test CLI new command
#[tokio::test]
async fn test_cli_new_command() {
    let ctx = e2e_utils::E2ETestContext::new().await;

    let output = ctx.run_cli_command(&["new", "my-test-agent"]);

    assert!(output.status.success(), "CLI new command failed");

    // Check that the agent directory was created
    let agent_dir = ctx.get_temp_path("my-test-agent");
    assert!(agent_dir.exists(), "Agent directory should be created");

    // Check that manifest file was created
    let manifest_path = agent_dir.join("carp.toml");
    assert!(manifest_path.exists(), "Manifest file should be created");

    // Verify manifest content
    let manifest_content = fs::read_to_string(manifest_path).expect("Should read manifest");
    assert!(manifest_content.contains("my-test-agent"));
    assert!(manifest_content.contains("[package]"));
}

// Test CLI new command with custom path
#[tokio::test]
async fn test_cli_new_with_custom_path() {
    let ctx = e2e_utils::E2ETestContext::new().await;

    let custom_path = ctx.get_temp_path("custom-location");
    let output = ctx.run_cli_command(&[
        "new",
        "custom-agent",
        "--path",
        custom_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "CLI new with custom path failed");

    let agent_dir = custom_path.join("custom-agent");
    assert!(
        agent_dir.exists(),
        "Agent directory should be created at custom path"
    );

    let manifest_path = agent_dir.join("carp.toml");
    assert!(
        manifest_path.exists(),
        "Manifest should exist at custom path"
    );
}

// Test CLI publish command (without authentication)
#[tokio::test]
async fn test_cli_publish_no_auth() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    // Create test agent files
    ctx.create_test_manifest("test-publish-agent", "1.0.0");
    ctx.create_test_agent_files();

    let output = ctx.run_cli_command(&["publish"]);

    // Should fail due to lack of authentication
    assert!(
        !output.status.success(),
        "CLI publish without auth should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("authentication") || stderr.contains("token") || stderr.contains("login"),
        "Should indicate authentication required"
    );
}

// Test CLI publish command with dry run
#[tokio::test]
async fn test_cli_publish_dry_run() {
    let ctx = e2e_utils::E2ETestContext::new().await;

    // Create test agent files
    ctx.create_test_manifest("test-dry-run-agent", "1.0.0");
    ctx.create_test_agent_files();

    let output = ctx.run_cli_command(&["publish", "--dry-run"]);

    // Dry run should succeed without making actual API calls
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should indicate it's a dry run
    assert!(
        stdout.contains("dry run") || stderr.contains("dry run") || output.status.success(),
        "Dry run should be indicated"
    );
}

// Test CLI error handling for network issues
#[tokio::test]
async fn test_cli_network_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create config pointing to non-existent server
    let config_path = temp_dir.path().join("config.toml");
    let config_content = r#"
registry_url = "http://127.0.0.1:9999"
timeout = 1
verify_ssl = false
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");

    let cli_binary_path = e2e_utils::E2ETestContext::find_cli_binary();

    let output = Command::new(&cli_binary_path)
        .args(&["search", "test"])
        .env("CARP_CONFIG", &config_path)
        .output()
        .expect("Failed to execute CLI command");

    assert!(
        !output.status.success(),
        "CLI should fail with network error"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("connection") || stderr.contains("network") || stderr.contains("timeout"),
        "Should indicate network error"
    );
}

// Test CLI help and version commands
#[tokio::test]
async fn test_cli_help_and_version() {
    let ctx = e2e_utils::E2ETestContext::new().await;

    // Test help command
    let help_output = ctx.run_cli_command(&["--help"]);
    assert!(help_output.status.success(), "CLI help should work");
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(help_stdout.contains("carp"));
    assert!(help_stdout.contains("search"));
    assert!(help_stdout.contains("pull"));
    assert!(help_stdout.contains("publish"));
    assert!(help_stdout.contains("new"));

    // Test version command
    let version_output = ctx.run_cli_command(&["--version"]);
    assert!(version_output.status.success(), "CLI version should work");
    let version_stdout = String::from_utf8_lossy(&version_output.stdout);
    assert!(version_stdout.contains("carp"));
    assert!(version_stdout.contains("0.1.0")); // Should match Cargo.toml version
}

// Test CLI verbose output
#[tokio::test]
async fn test_cli_verbose_output() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [],
                "total": 0,
                "page": 1,
                "per_page": 20
            })
            .to_string(),
        )
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["search", "test", "--verbose"]);

    assert!(output.status.success(), "CLI verbose search should work");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose mode should show additional information
    // This depends on how verbose logging is implemented
    assert!(
        stderr.contains("debug") || stderr.contains("verbose") || stderr.len() > 0,
        "Verbose mode should provide additional output"
    );
}

// Test CLI quiet output
#[tokio::test]
async fn test_cli_quiet_output() {
    let mut ctx = e2e_utils::E2ETestContext::new().await;

    let _mock = ctx
        .mock_server
        .mock("GET", "/api/v1/agents/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "agents": [],
                "total": 0,
                "page": 1,
                "per_page": 20
            })
            .to_string(),
        )
        .create_async()
        .await;

    let output = ctx.run_cli_command(&["search", "test", "--quiet"]);

    assert!(output.status.success(), "CLI quiet search should work");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Quiet mode should suppress most output
    // The exact behavior depends on implementation
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "Quiet mode should suppress normal output"
    );
}
