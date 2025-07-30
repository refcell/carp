use crate::api::ApiClient;
use crate::config::ConfigManager;
use crate::utils::error::CarpResult;
use colored::*;

/// Execute the list command to show all available agents
pub async fn execute(verbose: bool) -> CarpResult<()> {
    if verbose {
        println!("Fetching all available agents...");
    }

    let config = ConfigManager::load_with_env_checks()?;
    let client = ApiClient::new(&config)?;

    // Use search with empty query to get all agents
    let response = client.search("", Some(1000), false).await?;

    if response.agents.is_empty() {
        println!("{}", "No agents found in the registry.".yellow());
        return Ok(());
    }

    println!(
        "{} {} agents available:\n",
        "Found".green().bold(),
        response.total
    );

    for agent in &response.agents {
        println!("{} {}", agent.name.bold().blue(), agent.version.dimmed());
        println!("  {}", agent.description);
        println!(
            "  by {} • {} views",
            agent.author.green(),
            agent.download_count.to_string().cyan()
        );

        if !agent.tags.is_empty() {
            print!("  tags: ");
            for (i, tag) in agent.tags.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", tag.yellow());
            }
            println!();
        }

        if verbose {
            println!("  created: {}", agent.created_at.format("%Y-%m-%d"));
            if let Some(homepage) = &agent.homepage {
                println!("  homepage: {}", homepage.blue().underline());
            }
            if let Some(repository) = &agent.repository {
                println!("  repository: {}", repository.blue().underline());
            }
        }

        println!();
    }

    if response.total > response.agents.len() {
        println!(
            "{} Showing {} of {} agents. Some agents may be hidden due to API limits.",
            "Note:".yellow().bold(),
            response.agents.len(),
            response.total
        );
    }

    Ok(())
}
