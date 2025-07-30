use crate::api::{ApiClient, UploadAgentRequest};
use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::utils::error::{CarpError, CarpResult};
use colored::*;
use inquire::Select;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Agent file information extracted from agent definition files
#[derive(Debug, Clone)]
pub struct AgentFile {
    pub path: PathBuf,
    pub name: String,
    pub description: String,
    pub display_name: String,
}

/// Execute the upload command
pub async fn execute(
    directory: Option<String>,
    api_key: Option<String>,
    verbose: bool,
) -> CarpResult<()> {
    // Ensure user is authenticated (either via API key parameter or stored configuration)
    AuthManager::ensure_authenticated(api_key.as_deref()).await?;

    // Expand directory path, defaulting to ~/.claude/agents/
    let dir_path = expand_directory_path(directory)?;

    if verbose {
        println!("Scanning directory: {}", dir_path.display());
    }

    // Scan directory for agent files
    let agent_files = scan_agent_files(&dir_path, verbose)?;

    if agent_files.is_empty() {
        println!(
            "{} No agent files found in {}",
            "Warning:".yellow().bold(),
            dir_path.display()
        );
        println!(
            "Looking for .md files with YAML frontmatter containing name and description fields."
        );
        return Ok(());
    }

    if verbose {
        println!("Found {} agent files", agent_files.len());
    }

    // Use inquire to prompt user for agent selection
    let selected_agent = select_agent(agent_files)?;

    if verbose {
        println!("Selected agent: {}", selected_agent.name);
    }

    // Read and parse the selected agent file
    let agent_content = fs::read_to_string(&selected_agent.path)?;

    // Upload the agent
    upload_agent(&selected_agent, agent_content, api_key, verbose).await?;

    println!(
        "{} Successfully uploaded agent '{}'",
        "✓".green().bold(),
        selected_agent.name.blue().bold()
    );

    Ok(())
}

/// Expand directory path, handling tilde expansion
fn expand_directory_path(directory: Option<String>) -> CarpResult<PathBuf> {
    let dir_str = directory.unwrap_or_else(|| "~/.claude/agents/".to_string());

    let expanded_path = if let Some(stripped) = dir_str.strip_prefix('~') {
        if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(dir_str.strip_prefix("~/").unwrap_or(stripped))
        } else {
            return Err(CarpError::FileSystem(
                "Unable to determine home directory".to_string(),
            ));
        }
    } else {
        PathBuf::from(dir_str)
    };

    if !expanded_path.exists() {
        return Err(CarpError::FileSystem(format!(
            "Directory does not exist: {}",
            expanded_path.display()
        )));
    }

    if !expanded_path.is_dir() {
        return Err(CarpError::FileSystem(format!(
            "Path is not a directory: {}",
            expanded_path.display()
        )));
    }

    Ok(expanded_path)
}

/// Scan directory recursively for agent definition files
fn scan_agent_files(dir_path: &Path, verbose: bool) -> CarpResult<Vec<AgentFile>> {
    let mut agents = Vec::new();

    if verbose {
        println!("Scanning for agent files recursively...");
    }

    for entry in WalkDir::new(dir_path).follow_links(false) {
        let entry =
            entry.map_err(|e| CarpError::FileSystem(format!("Error scanning directory: {e}")))?;

        let path = entry.path();
        if path.is_file() {
            // Check if it's a markdown file
            if let Some(extension) = path.extension() {
                if extension == "md" {
                    if let Ok(agent) = parse_agent_file(path, verbose) {
                        agents.push(agent);
                    }
                }
            }
        }
    }

    // Sort by name for consistent ordering
    agents.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(agents)
}

/// Parse an agent file to extract name and description from YAML frontmatter
fn parse_agent_file(path: &Path, verbose: bool) -> CarpResult<AgentFile> {
    let content = fs::read_to_string(path)?;

    // Check if file starts with YAML frontmatter
    if !content.starts_with("---") {
        return Err(CarpError::ManifestError(
            "Agent file does not contain YAML frontmatter".to_string(),
        ));
    }

    // Find the end of the frontmatter
    let lines: Vec<&str> = content.lines().collect();
    let mut frontmatter_end = None;

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            frontmatter_end = Some(i);
            break;
        }
    }

    let frontmatter_end = frontmatter_end.ok_or_else(|| {
        CarpError::ManifestError("Invalid YAML frontmatter: missing closing ---".to_string())
    })?;

    // Extract frontmatter content
    let frontmatter_lines = &lines[1..frontmatter_end];
    let frontmatter_content = frontmatter_lines.join("\n");

    // Parse YAML frontmatter
    let frontmatter: serde_json::Value = serde_yaml::from_str(&frontmatter_content)
        .map_err(|e| CarpError::ManifestError(format!("Invalid YAML frontmatter: {e}")))?;

    // Extract name and description
    let name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CarpError::ManifestError("Missing 'name' field in frontmatter".to_string()))?
        .to_string();

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CarpError::ManifestError("Missing 'description' field in frontmatter".to_string())
        })?
        .to_string();

    // Create display name for selection
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let display_name = format!("{name} ({file_name})");

    if verbose {
        println!(
            "  Found agent: {} - {}",
            name,
            description.chars().take(60).collect::<String>()
        );
    }

    Ok(AgentFile {
        path: path.to_path_buf(),
        name,
        description,
        display_name,
    })
}

/// Use inquire to prompt user for agent selection
fn select_agent(agents: Vec<AgentFile>) -> CarpResult<AgentFile> {
    let options: Vec<String> = agents.iter().map(|a| a.display_name.clone()).collect();

    let selection = Select::new("Select an agent to upload:", options)
        .prompt()
        .map_err(|e| CarpError::Other(format!("Selection cancelled: {e}")))?;

    // Find the selected agent
    agents
        .into_iter()
        .find(|a| a.display_name == selection)
        .ok_or_else(|| CarpError::Other("Selected agent not found".to_string()))
}

/// Upload the selected agent to the registry
async fn upload_agent(
    agent: &AgentFile,
    content: String,
    api_key: Option<String>,
    verbose: bool,
) -> CarpResult<()> {
    if verbose {
        println!("Preparing to upload agent '{}'...", agent.name);
    }

    // Create upload request
    let request = UploadAgentRequest {
        name: agent.name.clone(),
        description: agent.description.clone(),
        content,
        version: Some("1.0.0".to_string()), // Default version for uploaded agents
        tags: vec!["claude-agent".to_string()], // Default tag for uploaded agents
        homepage: None,
        repository: None,
        license: Some("MIT".to_string()), // Default license
    };

    // Upload to registry
    let config = ConfigManager::load_with_env_checks()?;
    let client = ApiClient::new(&config)?.with_api_key(api_key);

    if verbose {
        println!("Uploading to registry...");
    }

    let response = client.upload(request).await?;

    if !response.success {
        if let Some(validation_errors) = &response.validation_errors {
            println!("{} Validation errors:", "Error:".red().bold());
            for error in validation_errors {
                println!("  {}: {}", error.field.yellow(), error.message);
            }
        }
        return Err(CarpError::Api {
            status: 400,
            message: response.message,
        });
    }

    if verbose {
        if let Some(agent_info) = response.agent {
            println!(
                "View at: https://carp.refcell.org/agents/{}",
                agent_info.name
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_agent_file() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file_path = temp_dir.path().join("test-agent.md");

        let content = r#"---
name: test-agent
description: A test agent for unit testing
color: blue
---

# Test Agent

This is a test agent for unit testing purposes.

## Usage

This agent helps with testing.
"#;

        fs::write(&agent_file_path, content).unwrap();

        let result = parse_agent_file(&agent_file_path, false);
        assert!(result.is_ok());

        let agent = result.unwrap();
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.description, "A test agent for unit testing");
    }

    #[test]
    fn test_parse_agent_file_missing_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file_path = temp_dir.path().join("invalid-agent.md");

        let content = r#"# Invalid Agent

This file doesn't have YAML frontmatter.
"#;

        fs::write(&agent_file_path, content).unwrap();

        let result = parse_agent_file(&agent_file_path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_agent_file_missing_name() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file_path = temp_dir.path().join("incomplete-agent.md");

        let content = r#"---
description: Missing name field
---

# Incomplete Agent
"#;

        fs::write(&agent_file_path, content).unwrap();

        let result = parse_agent_file(&agent_file_path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_expand_directory_path() {
        // Test relative path
        let result = expand_directory_path(Some(".".to_string()));
        assert!(result.is_ok());

        // Test non-existent directory
        let result = expand_directory_path(Some("/non/existent/path".to_string()));
        assert!(result.is_err());
    }
}
