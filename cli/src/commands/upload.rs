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

/// Selection result from agent selection prompt
#[derive(Debug)]
enum AgentSelection {
    Single(AgentFile),
    All(Vec<AgentFile>),
}

/// Execute the upload command
pub async fn execute(
    directory: Option<String>,
    api_key: Option<String>,
    verbose: bool,
) -> CarpResult<()> {
    // Load config first to get stored API key
    let config = ConfigManager::load_with_env_checks()?;
    
    if verbose {
        println!("DEBUG: Runtime API key present: {}", api_key.is_some());
        println!("DEBUG: Stored API key present: {}", config.api_key.is_some());
    }
    
    // Use runtime API key if provided, otherwise use stored API key
    let effective_api_key = api_key.as_deref().or(config.api_key.as_deref());
    
    if verbose {
        println!("DEBUG: Effective API key present: {}", effective_api_key.is_some());
    }
    
    // Ensure user is authenticated (either via API key parameter or stored configuration)
    AuthManager::ensure_authenticated(effective_api_key).await?;

    // Get directory path - either provided, prompted for, or use default
    let dir_path = get_directory_path(directory, verbose)?;

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

    // Use inquire to prompt user for agent selection (including "All" option)
    let selection = select_agents(agent_files.clone())?;

    match selection {
        AgentSelection::Single(agent) => {
            if verbose {
                println!("Selected agent: {}", agent.name);
            }

            // Read and parse the selected agent file
            let agent_content = fs::read_to_string(&agent.path)?;

            // Upload the agent
            upload_agent(&agent, agent_content, effective_api_key, verbose, &config).await?;

            println!(
                "{} Successfully uploaded agent '{}'",
                "✓".green().bold(),
                agent.name.blue().bold()
            );
        }
        AgentSelection::All(agents) => {
            if verbose {
                println!("Uploading all {} agents", agents.len());
            }

            let mut successful = 0;
            let mut failed = 0;

            for agent in agents {
                println!(
                    "{} Uploading agent '{}'...",
                    "⟳".blue().bold(),
                    agent.name.blue().bold()
                );

                match fs::read_to_string(&agent.path) {
                    Ok(agent_content) => {
                        match upload_agent(&agent, agent_content, effective_api_key, verbose, &config).await {
                            Ok(_) => {
                                println!(
                                    "{} Successfully uploaded agent '{}'",
                                    "✓".green().bold(),
                                    agent.name.blue().bold()
                                );
                                successful += 1;
                            }
                            Err(e) => {
                                println!(
                                    "{} Failed to upload agent '{}': {}",
                                    "✗".red().bold(),
                                    agent.name.red().bold(),
                                    e
                                );
                                failed += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to read agent '{}': {}",
                            "✗".red().bold(),
                            agent.name.red().bold(),
                            e
                        );
                        failed += 1;
                    }
                }
            }

            println!(
                "\n{} Upload complete: {} successful, {} failed",
                "✓".green().bold(),
                successful.to_string().green().bold(),
                if failed > 0 { failed.to_string().red().bold() } else { failed.to_string().green().bold() }
            );
        }
    }

    Ok(())
}

/// Get directory path from user input, prompt, or default
fn get_directory_path(directory: Option<String>, verbose: bool) -> CarpResult<PathBuf> {
    let dir_path = if let Some(dir) = directory {
        // Directory provided via command line
        expand_directory_path(Some(dir))?
    } else {
        // Prompt user for directory
        let default_dir = "~/.claude/agents/";
        let prompt_text = format!("Enter directory to scan for agents (default: {}):", default_dir);
        
        let input = inquire::Text::new(&prompt_text)
            .with_default(default_dir)
            .prompt()
            .map_err(|e| CarpError::Other(format!("Input cancelled: {e}")))?;
        
        let input = if input.trim().is_empty() {
            default_dir.to_string()
        } else {
            input
        };
        
        expand_directory_path(Some(input))?
    };
    
    if verbose {
        println!("Using directory: {}", dir_path.display());
    }
    
    Ok(dir_path)
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
                    match parse_agent_file(path, verbose) {
                        Ok(agent) => {
                            agents.push(agent);
                        }
                        Err(e) => {
                            if verbose {
                                println!(
                                    "  {} Skipping {}: {}",
                                    "⚠".yellow(),
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by name for consistent ordering
    agents.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(agents)
}

/// Extract a field from YAML as a string, handling various data types
fn extract_field_as_string(frontmatter: &serde_json::Value, field: &str) -> Option<String> {
    frontmatter.get(field).and_then(|v| match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Array(arr) => {
            // Join array elements as comma-separated string
            Some(
                arr.iter()
                    .filter_map(|item| match item {
                        serde_json::Value::String(s) => Some(s.clone()),
                        _ => Some(item.to_string()),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        serde_json::Value::Object(_) => {
            // Convert object to string representation
            Some(v.to_string())
        }
        serde_json::Value::Null => None,
    })
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

    // Find the end of the frontmatter with more flexible boundary detection
    let lines: Vec<&str> = content.lines().collect();
    let mut frontmatter_end = None;

    for (i, line) in lines.iter().enumerate().skip(1) {
        let trimmed = line.trim();
        // Accept "---" or "..." as valid YAML document endings
        if trimmed == "---" || trimmed == "..." {
            frontmatter_end = Some(i);
            break;
        }
    }

    let frontmatter_end = frontmatter_end.ok_or_else(|| {
        if verbose {
            eprintln!("Could not find closing frontmatter boundary in {}", path.display());
            eprintln!("Looking for '---' or '...' after opening '---'");
        }
        CarpError::ManifestError("Invalid YAML frontmatter: missing closing --- or ...".to_string())
    })?;

    // Extract frontmatter content
    let frontmatter_lines = &lines[1..frontmatter_end];
    let frontmatter_content = frontmatter_lines.join("\n");

    // Parse YAML frontmatter with better error handling
    let frontmatter: serde_json::Value = serde_yaml::from_str(&frontmatter_content)
        .map_err(|e| {
            if verbose {
                eprintln!("YAML parsing failed for {}: {}", path.display(), e);
                eprintln!("Frontmatter content:\n{}", frontmatter_content);
            }
            CarpError::ManifestError(format!("Invalid YAML frontmatter: {e}"))
        })?;

    // Extract name and description with more flexible handling
    let name = extract_field_as_string(&frontmatter, "name")
        .ok_or_else(|| CarpError::ManifestError("Missing 'name' field in frontmatter".to_string()))?;

    let description = extract_field_as_string(&frontmatter, "description")
        .ok_or_else(|| {
            CarpError::ManifestError("Missing 'description' field in frontmatter".to_string())
        })?;

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

/// Use inquire to prompt user for agent selection (single or all)
fn select_agents(agents: Vec<AgentFile>) -> CarpResult<AgentSelection> {
    if agents.is_empty() {
        return Err(CarpError::Other("No agents found".to_string()));
    }

    let mut options = vec!["📦 All agents".to_string()];
    options.extend(agents.iter().map(|a| a.display_name.clone()));

    let selection = Select::new("Select agents to upload:", options)
        .prompt()
        .map_err(|e| CarpError::Other(format!("Selection cancelled: {e}")))?;

    if selection == "📦 All agents" {
        Ok(AgentSelection::All(agents))
    } else {
        // Find the selected agent
        let selected_agent = agents
            .into_iter()
            .find(|a| a.display_name == selection)
            .ok_or_else(|| CarpError::Other("Selected agent not found".to_string()))?;
        
        Ok(AgentSelection::Single(selected_agent))
    }
}

/// Upload the selected agent to the registry
async fn upload_agent(
    agent: &AgentFile,
    content: String,
    api_key: Option<&str>,
    verbose: bool,
    config: &crate::config::Config,
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
    let client = ApiClient::new(&config)?.with_api_key(api_key.map(|s| s.to_string()));

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
