use crate::api::ApiClient;
use crate::config::ConfigManager;
use crate::utils::error::CarpResult;
use colored::*;

/// Execute the healthcheck command
pub async fn execute(verbose: bool) -> CarpResult<()> {
    if verbose {
        println!("Checking API health...");
    }

    let config = ConfigManager::load()?;
    let client = ApiClient::new(&config)?;

    let response = client.health_check().await?;

    // Print health status with color coding
    let status_display = match response.status.as_str() {
        "healthy" => response.status.green().bold(),
        "unhealthy" => response.status.red().bold(),
        _ => response.status.yellow().bold(),
    };

    println!("{} {}", "Status:".bold(), status_display);
    println!("{} {}", "Service:".bold(), response.service);
    println!("{} {}", "Environment:".bold(), response.environment);
    println!("{} {}", "Message:".bold(), response.message);

    if let Some(agent_count) = response.agent_count {
        if agent_count >= 0 {
            println!(
                "{} {}",
                "Agent Count:".bold(),
                agent_count.to_string().cyan()
            );
        } else {
            println!(
                "{} {}",
                "Agent Count:".bold(),
                "unknown (database accessible)".yellow()
            );
        }
    }

    if verbose {
        println!("{} {}", "Timestamp:".bold(), response.timestamp.dimmed());
        println!(
            "{} {}",
            "Registry URL:".bold(),
            config.registry_url.blue().underline()
        );
    }

    if let Some(error) = response.error {
        println!("{} {}", "Error:".red().bold(), error);
    }

    // Set exit code based on health status
    if response.status != "healthy" {
        std::process::exit(1);
    }

    Ok(())
}
