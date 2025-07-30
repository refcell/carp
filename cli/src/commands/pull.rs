use crate::api::ApiClient;
use crate::config::ConfigManager;
use crate::utils::error::{CarpError, CarpResult};
use colored::*;
use inquire::{InquireError, Select};
use std::fs;
use std::path::{Path, PathBuf};

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

    // Get download information
    let download_info = client.get_agent_download(&name, version).await?;

    if verbose {
        println!(
            "Found {} v{} ({} bytes)",
            download_info.name, download_info.version, download_info.file_size
        );
    }

    // Determine output directory
    let output_dir = determine_output_dir(&name, output, &config)?;

    // Check if directory exists and handle force flag
    if output_dir.exists() && !force {
        return Err(CarpError::FileSystem(format!(
            "Directory '{}' already exists. Use --force to overwrite.",
            output_dir.display()
        )));
    }

    if output_dir.exists() && force {
        if verbose {
            println!("Removing existing directory...");
        }
        fs::remove_dir_all(&output_dir)?;
    }

    // Download the agent
    println!("Downloading {}...", download_info.name.blue().bold());
    let content = client.download_agent(&download_info.download_url).await?;

    // Verify checksum if available
    if !download_info.checksum.is_empty() {
        if verbose {
            println!("Verifying checksum...");
        }
        verify_checksum(&content, &download_info.checksum)?;
    }

    // Extract the agent
    if verbose {
        println!("Extracting to {}...", output_dir.display());
    }
    extract_agent(&content, &output_dir, &download_info.content_type)?;

    println!(
        "{} Successfully pulled {} v{} to {}",
        "✓".green().bold(),
        download_info.name.blue().bold(),
        download_info.version,
        output_dir.display().to_string().cyan()
    );

    // Show usage instructions
    println!("\nTo use this agent:");
    println!("  cd {}", output_dir.display());
    println!("  # Follow the README.md for specific usage instructions");

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

/// Determine the output directory for the agent
fn determine_output_dir(
    name: &str,
    output: Option<String>,
    config: &crate::config::Config,
) -> CarpResult<PathBuf> {
    if let Some(output_path) = output {
        return Ok(PathBuf::from(output_path));
    }

    if let Some(default_dir) = &config.default_output_dir {
        return Ok(PathBuf::from(default_dir).join(name));
    }

    // Default to current directory
    Ok(PathBuf::from(name))
}

/// Verify the checksum of downloaded content
fn verify_checksum(content: &[u8], expected: &str) -> CarpResult<()> {
    use sha2::{Digest, Sha256};

    // Parse expected checksum (remove "sha256-" prefix if present)
    let expected_hash = if let Some(hash) = expected.strip_prefix("sha256-") {
        hash
    } else {
        expected
    };

    let mut hasher = Sha256::new();
    hasher.update(content);
    let computed = format!("{:x}", hasher.finalize());

    if computed != expected_hash {
        return Err(CarpError::Network(format!(
            "Checksum verification failed. Expected: {expected_hash}, Computed: {computed}. The downloaded file may be corrupted."
        )));
    }

    Ok(())
}

/// Extract agent content to the specified directory
fn extract_agent(content: &[u8], output_dir: &Path, content_type: &str) -> CarpResult<()> {
    // Create output directory
    fs::create_dir_all(output_dir)?;

    match content_type {
        "application/zip" => extract_zip(content, output_dir),
        "application/gzip" | "application/x-gzip" => extract_gzip(content, output_dir),
        _ => {
            // Try to detect format by checking magic bytes
            if content.starts_with(&[0x50, 0x4b, 0x03, 0x04])
                || content.starts_with(&[0x50, 0x4b, 0x05, 0x06])
            {
                extract_zip(content, output_dir)
            } else if content.starts_with(&[0x1f, 0x8b]) {
                extract_gzip(content, output_dir)
            } else {
                Err(CarpError::FileSystem(format!(
                    "Unsupported content type: {content_type}. Expected ZIP or GZIP format."
                )))
            }
        }
    }
}

/// Extract ZIP archive content
fn extract_zip(content: &[u8], output_dir: &Path) -> CarpResult<()> {
    use std::io::Cursor;

    let reader = Cursor::new(content);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| CarpError::FileSystem(format!("Failed to read ZIP archive: {e}")))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| CarpError::FileSystem(format!("Failed to read ZIP entry: {e}")))?;

        // Security: Validate file path to prevent directory traversal attacks
        let file_name = file.name();
        if file_name.contains("..") || file_name.starts_with('/') || file_name.contains('\0') {
            return Err(CarpError::FileSystem(format!(
                "Unsafe file path in archive: {file_name}"
            )));
        }

        let file_path = output_dir.join(file_name);

        // Additional security: Ensure the resolved path is still within output_dir
        let canonical_output = output_dir.canonicalize()?;
        let canonical_file = file_path.canonicalize().unwrap_or_else(|_| {
            file_path
                .parent()
                .unwrap_or(output_dir)
                .join(file_path.file_name().unwrap_or_default())
        });

        if !canonical_file.starts_with(&canonical_output) {
            return Err(CarpError::FileSystem(format!(
                "File path outside target directory: {file_name}"
            )));
        }

        if file.is_dir() {
            fs::create_dir_all(&file_path)?;
        } else {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = fs::File::create(&file_path)?;
            std::io::copy(&mut file, &mut output_file)?;

            // Set safe permissions on extracted files
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = output_file.metadata()?.permissions();
                perms.set_mode(0o644); // Owner read/write, group/other read
                fs::set_permissions(&file_path, perms)?;
            }
        }
    }

    Ok(())
}

/// Extract GZIP archive content (assumes it's a tar.gz)
fn extract_gzip(content: &[u8], output_dir: &Path) -> CarpResult<()> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let reader = Cursor::new(content);
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| CarpError::FileSystem(format!("Failed to read tar entries: {e}")))?
    {
        let mut entry =
            entry.map_err(|e| CarpError::FileSystem(format!("Failed to read tar entry: {e}")))?;

        let path = entry
            .path()
            .map_err(|e| CarpError::FileSystem(format!("Failed to get entry path: {e}")))?;
        let file_path = output_dir.join(&path);

        // Security: Validate file path to prevent directory traversal attacks
        let path_str = path.to_string_lossy();
        if path_str.contains("..") || path_str.starts_with('/') || path_str.contains('\0') {
            return Err(CarpError::FileSystem(format!(
                "Unsafe file path in archive: {path_str}"
            )));
        }

        // Additional security: Ensure the resolved path is still within output_dir
        let canonical_output = output_dir.canonicalize()?;
        let canonical_file = file_path.canonicalize().unwrap_or_else(|_| {
            file_path
                .parent()
                .unwrap_or(output_dir)
                .join(file_path.file_name().unwrap_or_default())
        });

        if !canonical_file.starts_with(&canonical_output) {
            return Err(CarpError::FileSystem(format!(
                "File path outside target directory: {path_str}"
            )));
        }

        // Extract the entry
        entry
            .unpack(&file_path)
            .map_err(|e| CarpError::FileSystem(format!("Failed to extract file: {e}")))?;

        // Set safe permissions on extracted files
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if file_path.is_file() {
                let mut perms = fs::metadata(&file_path)?.permissions();
                perms.set_mode(0o644); // Owner read/write, group/other read
                fs::set_permissions(&file_path, perms)?;
            }
        }
    }

    Ok(())
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
            message: format!("No versions found for agent '{}'.", selected_agent),
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
            versions.clone()
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
    if let Ok(agent_info) = get_agent_info(client, &selected_agent, &selected_version).await {
        display_agent_definition(&agent_info);
    }

    Ok(format!("{}@{}", selected_agent, selected_version))
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
    
    let mut versions: Vec<String> = response.agents
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

/// Get detailed agent information
async fn get_agent_info(client: &ApiClient, name: &str, version: &str) -> CarpResult<crate::api::types::Agent> {
    let response = client.search(name, Some(1000), true).await?;
    
    response.agents
        .into_iter()
        .find(|agent| agent.name == name && agent.version == version)
        .ok_or_else(|| CarpError::Api {
            status: 404,
            message: format!("Agent '{}' version '{}' not found", name, version),
        })
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
    
    println!("  {}: {}", "Downloads".bold(), agent.download_count.to_string().cyan());
    println!("  {}: {}", "Created".bold(), agent.created_at.format("%Y-%m-%d %H:%M UTC"));
    println!("  {}: {}", "Updated".bold(), agent.updated_at.format("%Y-%m-%d %H:%M UTC"));
    
    if let Some(readme) = &agent.readme {
        if !readme.trim().is_empty() {
            println!("\n{}", "README:".bold().underline());
            println!("{}", readme);
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
}
