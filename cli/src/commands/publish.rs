use crate::api::{ApiClient, PublishRequest};
use crate::auth::AuthManager;
use crate::config::ConfigManager;
use crate::utils::error::{CarpError, CarpResult};
use crate::utils::manifest::AgentManifest;
use colored::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Execute the publish command
pub async fn execute(
    manifest_path: Option<String>,
    yes: bool,
    dry_run: bool,
    verbose: bool,
) -> CarpResult<()> {
    // Ensure user is authenticated
    if !dry_run {
        AuthManager::ensure_authenticated().await?;
    }

    // Find and load manifest
    let manifest_path = find_manifest(manifest_path)?;
    let manifest = AgentManifest::load(&manifest_path)?;

    if verbose {
        println!("Loaded manifest from {}", manifest_path.display());
    }

    // Validate manifest
    manifest.validate()?;

    println!(
        "Publishing {} v{}...",
        manifest.name.blue().bold(),
        manifest.version
    );

    // Show what will be published
    display_publish_info(&manifest, verbose);

    // Confirm publication unless --yes is specified
    if !yes && !dry_run {
        if !confirm_publish(&manifest)? {
            println!("Publication cancelled.");
            return Ok(());
        }
    }

    if dry_run {
        println!(
            "{} Dry run completed. No files were published.",
            "✓".green().bold()
        );
        return Ok(());
    }

    // Package the agent
    if verbose {
        println!("Packaging agent files...");
    }
    let package_content = package_agent(&manifest, &manifest_path)?;

    // Create publish request
    let request = PublishRequest {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        readme: load_readme(&manifest_path)?,
        homepage: manifest.homepage.clone(),
        repository: manifest.repository.clone(),
        license: manifest.license.clone(),
        tags: manifest.tags.clone(),
    };

    // Publish to registry
    let config = ConfigManager::load()?;
    let client = ApiClient::new(&config)?;

    println!("Uploading to registry...");
    let response = client.publish(request, package_content).await?;

    if response.success {
        println!(
            "{} Successfully published {} v{}",
            "✓".green().bold(),
            manifest.name.blue().bold(),
            manifest.version
        );

        if let Some(agent) = response.agent {
            println!("View at: https://carp.refcell.org/agents/{}", agent.name);
        }
    } else {
        return Err(CarpError::Api {
            status: 400,
            message: response.message,
        });
    }

    Ok(())
}

/// Find the manifest file
fn find_manifest(path: Option<String>) -> CarpResult<PathBuf> {
    if let Some(path) = path {
        let manifest_path = PathBuf::from(path);
        if !manifest_path.exists() {
            return Err(CarpError::ManifestError(format!(
                "Manifest file not found: {}",
                manifest_path.display()
            )));
        }
        return Ok(manifest_path);
    }

    // Look for common manifest file names
    let candidates = ["Carp.toml", "carp.toml", "agent.toml"];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(CarpError::ManifestError(
        "No manifest file found. Create a Carp.toml file or specify --manifest.".to_string(),
    ))
}

/// Display information about what will be published
fn display_publish_info(manifest: &AgentManifest, verbose: bool) {
    println!("\n{}", "Package Information:".bold());
    println!("  Name: {}", manifest.name.blue());
    println!("  Version: {}", manifest.version);
    println!("  Description: {}", manifest.description);
    println!("  Author: {}", manifest.author.green());

    if let Some(license) = &manifest.license {
        println!("  License: {}", license);
    }

    if !manifest.tags.is_empty() {
        print!("  Tags: ");
        for (i, tag) in manifest.tags.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", tag.yellow());
        }
        println!();
    }

    if verbose {
        println!("\n{}", "Files to include:".bold());
        for file in &manifest.files {
            println!("  • {}", file);
        }
    }

    println!();
}

/// Confirm publication with the user
fn confirm_publish(manifest: &AgentManifest) -> CarpResult<bool> {
    print!(
        "Publish {} v{} to the registry? [y/N]: ",
        manifest.name, manifest.version
    );
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

/// Package the agent into a ZIP file
fn package_agent(manifest: &AgentManifest, manifest_path: &Path) -> CarpResult<Vec<u8>> {
    let base_dir = manifest_path
        .parent()
        .ok_or_else(|| CarpError::FileSystem("Invalid manifest path".to_string()))?;

    let mut zip_data = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut zip_data);
        let mut zip = ZipWriter::new(cursor);

        // Add manifest file
        add_file_to_zip(&mut zip, "Carp.toml", manifest_path)?;

        // Add specified files
        for file_pattern in &manifest.files {
            let file_path = base_dir.join(file_pattern);

            if file_path.is_file() {
                add_file_to_zip(&mut zip, file_pattern, &file_path)?;
            } else if file_path.is_dir() {
                add_directory_to_zip(&mut zip, file_pattern, &file_path)?;
            } else {
                eprintln!("{} File not found: {}", "Warning:".yellow(), file_pattern);
            }
        }

        zip.finish()?;
    }

    Ok(zip_data)
}

/// Add a single file to the ZIP archive
fn add_file_to_zip(
    zip: &mut ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
    name: &str,
    path: &Path,
) -> CarpResult<()> {
    let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

    zip.start_file(name, options)?;
    let content = fs::read(path)?;
    zip.write_all(&content)?;

    Ok(())
}

/// Add a directory recursively to the ZIP archive
fn add_directory_to_zip(
    zip: &mut ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
    prefix: &str,
    dir_path: &Path,
) -> CarpResult<()> {
    let walker = walkdir::WalkDir::new(dir_path);

    for entry in walker {
        let entry = entry.map_err(|e| CarpError::FileSystem(format!("Walk error: {}", e)))?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(dir_path)
            .map_err(|e| CarpError::FileSystem(format!("Path strip error: {}", e)))?;

        if path.is_file() {
            let zip_path = format!("{}/{}", prefix, relative_path.display());
            add_file_to_zip(zip, &zip_path, path)?;
        }
    }

    Ok(())
}

/// Load README file if it exists
fn load_readme(manifest_path: &Path) -> CarpResult<Option<String>> {
    let base_dir = manifest_path
        .parent()
        .ok_or_else(|| CarpError::FileSystem("Invalid manifest path".to_string()))?;

    let readme_candidates = ["README.md", "readme.md", "README.txt", "readme.txt"];

    for candidate in &readme_candidates {
        let readme_path = base_dir.join(candidate);
        if readme_path.exists() {
            let content = fs::read_to_string(readme_path)?;
            return Ok(Some(content));
        }
    }

    Ok(None)
}
