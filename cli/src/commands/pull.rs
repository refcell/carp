use crate::api::ApiClient;
use crate::config::ConfigManager;
use crate::utils::error::{CarpError, CarpResult};
use colored::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Execute the pull command
pub async fn execute(agent: String, output: Option<String>, force: bool, verbose: bool) -> CarpResult<()> {
    let (name, version) = parse_agent_spec(&agent)?;
    
    if verbose {
        println!("Pulling agent '{}'{}...", name, 
                version.map(|v| format!(" version {}", v)).unwrap_or_default());
    }
    
    let config = ConfigManager::load()?;
    let client = ApiClient::new(&config)?;
    
    // Get download information
    let download_info = client.get_agent_download(&name, version).await?;
    
    if verbose {
        println!("Found {} v{} ({} bytes)", 
                download_info.name, 
                download_info.version,
                download_info.size);
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
    extract_agent(&content, &output_dir)?;
    
    println!("{} Successfully pulled {} v{} to {}", 
            "✓".green().bold(),
            download_info.name.blue().bold(), 
            download_info.version,
            output_dir.display().to_string().cyan());
    
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
                "Invalid agent specification. Use 'name' or 'name@version'.".to_string()
            ));
        }
        
        Ok((name.to_string(), Some(version)))
    } else {
        Ok((spec.to_string(), None))
    }
}

/// Determine the output directory for the agent
fn determine_output_dir(name: &str, output: Option<String>, config: &crate::config::Config) -> CarpResult<PathBuf> {
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
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let computed = format!("{:x}", hasher.finish());
    
    if computed != expected {
        return Err(CarpError::Network(
            "Checksum verification failed. The downloaded file may be corrupted.".to_string()
        ));
    }
    
    Ok(())
}

/// Extract agent content to the specified directory
fn extract_agent(content: &[u8], output_dir: &Path) -> CarpResult<()> {
    use std::io::Cursor;
    
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // For now, assume the content is a ZIP file
    let reader = Cursor::new(content);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| CarpError::FileSystem(format!("Failed to read ZIP archive: {}", e)))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| CarpError::FileSystem(format!("Failed to read ZIP entry: {}", e)))?;
        
        // Security: Validate file path to prevent directory traversal attacks
        let file_name = file.name();
        if file_name.contains("..") || file_name.starts_with('/') || file_name.contains('\0') {
            return Err(CarpError::FileSystem(format!(
                "Unsafe file path in archive: {}", file_name
            )));
        }
        
        let file_path = output_dir.join(file_name);
        
        // Additional security: Ensure the resolved path is still within output_dir
        let canonical_output = output_dir.canonicalize()?;
        let canonical_file = file_path.canonicalize().unwrap_or_else(|_| {
            file_path.parent().unwrap_or(output_dir).join(
                file_path.file_name().unwrap_or_default()
            )
        });
        
        if !canonical_file.starts_with(&canonical_output) {
            return Err(CarpError::FileSystem(format!(
                "File path outside target directory: {}", file_name
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