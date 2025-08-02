use crate::api::ApiClient;
use crate::config::ConfigManager;
use crate::utils::error::{CarpError, CarpResult};
use colored::*;
use inquire::{InquireError, Select, Text};
use std::fs;
use std::path::PathBuf;

/// Execute the pull command
pub async fn execute(
    agent: Option<String>,
    output: Option<String>,
    force: bool,
    verbose: bool,
) -> CarpResult<()> {
    let config = ConfigManager::load_with_env_checks()?;
    let client = ApiClient::new(&config)?;

    // If no agent specified, show interactive selection
    let agent_spec = match agent {
        Some(spec) => spec,
        None => {
            if verbose {
                println!("Fetching available agents for selection...");
            }
            interactive_agent_selection(&client).await?
        }
    };

    let (name, version) = parse_agent_spec(&agent_spec)?;

    if verbose {
        println!(
            "Pulling agent '{}'{}...",
            name,
            version.map(|v| format!(" version {v}")).unwrap_or_default()
        );
    }

    // Get agent definition directly from search API
    let agent_info = get_agent_definition(&client, &name, version).await?;

    if verbose {
        println!(
            "Found {} v{} by {}",
            agent_info.name, agent_info.version, agent_info.author
        );
    }

    // Determine output file path
    let output_path = determine_output_file(&name, output, &config).await?;

    // Check if file exists and handle force flag
    if output_path.exists() && !force {
        return Err(CarpError::FileSystem(format!(
            "File '{}' already exists. Use --force to overwrite.",
            output_path.display()
        )));
    }

    // Create the agent definition content
    let agent_content = create_agent_definition_file(&agent_info)?;

    // Ensure the parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the agent definition file
    fs::write(&output_path, agent_content)?;

    println!(
        "{} Successfully pulled {} v{} to {}",
        "✓".green().bold(),
        agent_info.name.blue().bold(),
        agent_info.version,
        output_path.display().to_string().cyan()
    );

    // Show usage instructions
    println!("\nTo use this agent:");
    println!(
        "  # The agent definition is now available at {}",
        output_path.display()
    );
    println!("  # You can reference this agent in your code or agent orchestration system");

    Ok(())
}

/// Parse agent specification (name or name@version)
fn parse_agent_spec(spec: &str) -> CarpResult<(String, Option<&str>)> {
    if let Some(at_pos) = spec.find('@') {
        let name = &spec[..at_pos];
        let version = &spec[at_pos + 1..];

        if name.is_empty() || version.is_empty() {
            return Err(CarpError::InvalidAgent(
                "Invalid agent specification. Use 'name' or 'name@version'.".to_string(),
            ));
        }

        Ok((name.to_string(), Some(version)))
    } else {
        Ok((spec.to_string(), None))
    }
}

/// Get agent definition directly from search API
async fn get_agent_definition(
    client: &ApiClient,
    name: &str,
    version: Option<&str>,
) -> CarpResult<crate::api::types::Agent> {
    // Search for the specific agent
    let response = client.search(name, Some(1000), true).await?;

    // Find the agent with matching name and version
    let target_version = version.unwrap_or("latest");

    if target_version == "latest" {
        // Find the latest version (versions are sorted in descending order from search)
        response
            .agents
            .into_iter()
            .find(|agent| agent.name == name)
            .ok_or_else(|| CarpError::Api {
                status: 404,
                message: format!("Agent '{name}' not found"),
            })
    } else {
        // Find exact version match
        response
            .agents
            .into_iter()
            .find(|agent| agent.name == name && agent.version == target_version)
            .ok_or_else(|| CarpError::Api {
                status: 404,
                message: format!("Agent '{name}' version '{target_version}' not found"),
            })
    }
}

/// Determine the output file path for the agent definition
async fn determine_output_file(
    name: &str,
    output: Option<String>,
    config: &crate::config::Config,
) -> CarpResult<PathBuf> {
    if let Some(output_path) = output {
        let path = expand_tilde(&output_path);

        // If the path is a directory (or will be a directory), append the agent name as filename
        if path.is_dir() || output_path.ends_with('/') || output_path.ends_with('\\') {
            return Ok(path.join(format!("{name}.md")));
        }

        return Ok(path);
    }

    // Get default agents directory
    let default_agents_dir = get_default_agents_dir(config)?;

    // Ask user where to place the file
    let prompt_text = format!("Where would you like to save the '{name}' agent definition?");

    let default_path = default_agents_dir.join(format!("{name}.md"));

    let file_path = Text::new(&prompt_text)
        .with_default(&default_path.to_string_lossy())
        .with_help_message("Enter the full path where you want to save the agent definition file")
        .prompt()
        .map_err(|e| match e {
            InquireError::OperationCanceled => CarpError::Api {
                status: 0,
                message: "Operation cancelled by user.".to_string(),
            },
            _ => CarpError::Api {
                status: 500,
                message: format!("Input error: {e}"),
            },
        })?;

    let path = expand_tilde(&file_path);

    // If the path is a directory (or will be a directory), append the agent name as filename
    if path.is_dir() || file_path.ends_with('/') || file_path.ends_with('\\') {
        Ok(path.join(format!("{name}.md")))
    } else {
        Ok(path)
    }
}

/// Expand tilde (~) in file paths
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(stripped)
        } else {
            PathBuf::from(path)
        }
    } else if path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}

/// Get the default agents directory
fn get_default_agents_dir(config: &crate::config::Config) -> CarpResult<PathBuf> {
    if let Some(default_dir) = &config.default_output_dir {
        return Ok(PathBuf::from(default_dir));
    }

    // Use ~/.config/carp/agents/ as default
    let config_dir = dirs::config_dir()
        .ok_or_else(|| CarpError::Config("Unable to find config directory".to_string()))?;

    let agents_dir = config_dir.join("carp").join("agents");
    Ok(agents_dir)
}

/// Create agent definition file content
fn create_agent_definition_file(agent: &crate::api::types::Agent) -> CarpResult<String> {
    let mut content = String::new();

    // Add YAML frontmatter
    content.push_str("---\n");
    content.push_str(&format!("name: {}\n", agent.name));
    content.push_str(&format!("version: {}\n", agent.version));
    content.push_str(&format!("description: {}\n", agent.description));
    content.push_str(&format!("author: {}\n", agent.author));

    if let Some(homepage) = &agent.homepage {
        content.push_str(&format!("homepage: {homepage}\n"));
    }

    if let Some(repository) = &agent.repository {
        content.push_str(&format!("repository: {repository}\n"));
    }

    if let Some(license) = &agent.license {
        content.push_str(&format!("license: {license}\n"));
    }

    if !agent.tags.is_empty() {
        content.push_str("tags:\n");
        for tag in &agent.tags {
            content.push_str(&format!("  - {tag}\n"));
        }
    }

    content.push_str(&format!(
        "created_at: {}\n",
        agent.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    content.push_str(&format!(
        "updated_at: {}\n",
        agent.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    content.push_str(&format!("download_count: {}\n", agent.download_count));
    content.push_str("---\n\n");

    // Add title
    content.push_str(&format!("# {} Agent\n\n", agent.name));

    // Add description
    content.push_str(&format!("{}\n\n", agent.description));

    // Add metadata section
    content.push_str("## Metadata\n\n");
    content.push_str(&format!("- **Version**: {}\n", agent.version));
    content.push_str(&format!("- **Author**: {}\n", agent.author));
    content.push_str(&format!("- **Downloads**: {}\n", agent.download_count));
    content.push_str(&format!(
        "- **Created**: {}\n",
        agent.created_at.format("%Y-%m-%d %H:%M UTC")
    ));
    content.push_str(&format!(
        "- **Updated**: {}\n",
        agent.updated_at.format("%Y-%m-%d %H:%M UTC")
    ));

    if !agent.tags.is_empty() {
        content.push_str(&format!("- **Tags**: {}\n", agent.tags.join(", ")));
    }

    if let Some(homepage) = &agent.homepage {
        content.push_str(&format!("- **Homepage**: {homepage}\n"));
    }

    if let Some(repository) = &agent.repository {
        content.push_str(&format!("- **Repository**: {repository}\n"));
    }

    if let Some(license) = &agent.license {
        content.push_str(&format!("- **License**: {license}\n"));
    }

    // Add README if available
    if let Some(readme) = &agent.readme {
        if !readme.trim().is_empty() {
            content.push_str("\n## README\n\n");
            content.push_str(readme);
            content.push('\n');
        }
    }

    Ok(content)
}

/// Interactive agent selection using inquire
async fn interactive_agent_selection(client: &ApiClient) -> CarpResult<String> {
    // Step 1: Get unique agent names
    let agent_names = get_unique_agent_names(client).await?;

    if agent_names.is_empty() {
        return Err(CarpError::Api {
            status: 404,
            message: "No agents found in the registry.".to_string(),
        });
    }

    println!(
        "{} {} unique agents available:",
        "Found".green().bold(),
        agent_names.len()
    );

    // Step 2: Let user select agent name
    let selected_agent = Select::new("Select an agent:", agent_names.clone())
        .with_page_size(15)
        .with_help_message("↑/↓ to navigate • Enter to select • Ctrl+C to cancel")
        .prompt()
        .map_err(|e| match e {
            InquireError::OperationCanceled => CarpError::Api {
                status: 0,
                message: "Operation cancelled by user.".to_string(),
            },
            _ => CarpError::Api {
                status: 500,
                message: format!("Selection error: {e}"),
            },
        })?;

    // Step 3: Get versions for selected agent
    let versions = get_agent_versions(client, &selected_agent).await?;

    if versions.is_empty() {
        return Err(CarpError::Api {
            status: 404,
            message: format!("No versions found for agent '{selected_agent}'."),
        });
    }

    println!(
        "\n{} {} versions available for {}:",
        "Found".green().bold(),
        versions.len(),
        selected_agent.blue().bold()
    );

    // Step 4: Let user select version
    let selected_version = if versions.len() == 1 {
        versions[0].clone()
    } else {
        Select::new(
            &format!("Select a version for {}:", selected_agent.blue().bold()),
            versions.clone(),
        )
        .with_page_size(15)
        .with_help_message("↑/↓ to navigate • Enter to select • Ctrl+C to cancel")
        .prompt()
        .map_err(|e| match e {
            InquireError::OperationCanceled => CarpError::Api {
                status: 0,
                message: "Operation cancelled by user.".to_string(),
            },
            _ => CarpError::Api {
                status: 500,
                message: format!("Selection error: {e}"),
            },
        })?
    };

    println!(
        "\n{} Selected: {} v{}",
        "✓".green().bold(),
        selected_agent.blue().bold(),
        selected_version
    );

    // Step 5: Get and display agent definition
    if let Ok(agent_info) =
        get_agent_definition(client, &selected_agent, Some(&selected_version)).await
    {
        display_agent_definition(&agent_info);
    }

    Ok(format!("{selected_agent}@{selected_version}"))
}

/// Get unique agent names from the registry
async fn get_unique_agent_names(client: &ApiClient) -> CarpResult<Vec<String>> {
    let response = client.search("", Some(1000), false).await?;

    let mut unique_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for agent in response.agents {
        unique_names.insert(agent.name);
    }

    let mut names: Vec<String> = unique_names.into_iter().collect();
    names.sort();
    Ok(names)
}

/// Get versions for a specific agent
async fn get_agent_versions(client: &ApiClient, agent_name: &str) -> CarpResult<Vec<String>> {
    let response = client.search(agent_name, Some(1000), true).await?;

    let mut versions: Vec<String> = response
        .agents
        .into_iter()
        .filter(|agent| agent.name == agent_name)
        .map(|agent| agent.version)
        .collect();

    // Sort versions in descending order (latest first)
    versions.sort_by(|a, b| {
        // Simple lexicographic comparison for now - could be improved with proper semver
        b.cmp(a)
    });

    Ok(versions)
}

/// Display agent definition information
fn display_agent_definition(agent: &crate::api::types::Agent) {
    println!("\n{}", "Agent Definition:".bold().underline());
    println!("  {}: {}", "Name".bold(), agent.name.blue());
    println!("  {}: {}", "Version".bold(), agent.version);
    println!("  {}: {}", "Author".bold(), agent.author.green());
    println!("  {}: {}", "Description".bold(), agent.description);

    if let Some(homepage) = &agent.homepage {
        println!("  {}: {}", "Homepage".bold(), homepage.cyan());
    }

    if let Some(repository) = &agent.repository {
        println!("  {}: {}", "Repository".bold(), repository.cyan());
    }

    if let Some(license) = &agent.license {
        println!("  {}: {}", "License".bold(), license);
    }

    if !agent.tags.is_empty() {
        print!("  {}: ", "Tags".bold());
        for (i, tag) in agent.tags.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", tag.yellow());
        }
        println!();
    }

    println!(
        "  {}: {}",
        "Downloads".bold(),
        agent.download_count.to_string().cyan()
    );
    println!(
        "  {}: {}",
        "Created".bold(),
        agent.created_at.format("%Y-%m-%d %H:%M UTC")
    );
    println!(
        "  {}: {}",
        "Updated".bold(),
        agent.updated_at.format("%Y-%m-%d %H:%M UTC")
    );

    if let Some(readme) = &agent.readme {
        if !readme.trim().is_empty() {
            println!("\n{}", "README:".bold().underline());
            println!("{readme}");
        }
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent_spec() {
        let (name, version) = parse_agent_spec("test-agent").unwrap();
        assert_eq!(name, "test-agent");
        assert!(version.is_none());

        let (name, version) = parse_agent_spec("test-agent@1.0.0").unwrap();
        assert_eq!(name, "test-agent");
        assert_eq!(version, Some("1.0.0"));

        assert!(parse_agent_spec("@1.0.0").is_err());
        assert!(parse_agent_spec("test-agent@").is_err());
    }

    #[test]
    fn test_expand_tilde() {
        // Test tilde expansion for home directory paths
        let expanded = expand_tilde("~/test/path");
        if let Some(home_dir) = dirs::home_dir() {
            assert_eq!(expanded, home_dir.join("test/path"));
        }

        // Test just tilde
        let expanded = expand_tilde("~");
        if let Some(home_dir) = dirs::home_dir() {
            assert_eq!(expanded, home_dir);
        }

        // Test absolute paths (no tilde)
        let expanded = expand_tilde("/absolute/path");
        assert_eq!(expanded, PathBuf::from("/absolute/path"));

        // Test relative paths (no tilde)
        let expanded = expand_tilde("relative/path");
        assert_eq!(expanded, PathBuf::from("relative/path"));
    }

    #[test]
    fn test_directory_path_handling() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test case 1: Existing directory should append agent name
        let existing_dir = temp_path.join("existing");
        fs::create_dir(&existing_dir).unwrap();

        // Mock the logic from determine_output_file for directory handling
        let agent_name = "test-agent";
        let file_path = existing_dir.to_string_lossy().to_string();
        let path = expand_tilde(&file_path);

        let result = if path.is_dir() || file_path.ends_with('/') || file_path.ends_with('\\') {
            path.join(format!("{agent_name}.md"))
        } else {
            path
        };

        assert_eq!(result, existing_dir.join("test-agent.md"));

        // Test case 2: Path ending with '/' should append agent name
        let dir_with_slash = format!("{}/", temp_path.join("nonexistent").to_string_lossy());
        let path = expand_tilde(&dir_with_slash);

        let result =
            if path.is_dir() || dir_with_slash.ends_with('/') || dir_with_slash.ends_with('\\') {
                path.join(format!("{agent_name}.md"))
            } else {
                path
            };

        assert_eq!(result, temp_path.join("nonexistent").join("test-agent.md"));

        // Test case 3: Regular file path should be returned as-is
        let file_path = temp_path.join("agent.md").to_string_lossy().to_string();
        let path = expand_tilde(&file_path);

        let result = if path.is_dir() || file_path.ends_with('/') || file_path.ends_with('\\') {
            path.join(format!("{agent_name}.md"))
        } else {
            path
        };

        assert_eq!(result, temp_path.join("agent.md"));

        // Test case 4: Tilde expansion with directory should work
        if let Some(home_dir) = dirs::home_dir() {
            let tilde_path = "~/.claude/agents/";
            let path = expand_tilde(tilde_path);

            let result = if path.is_dir() || tilde_path.ends_with('/') || tilde_path.ends_with('\\')
            {
                path.join(format!("{agent_name}.md"))
            } else {
                path
            };

            assert_eq!(result, home_dir.join(".claude/agents/test-agent.md"));
        }
    }
}
