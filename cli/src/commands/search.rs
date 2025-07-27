use crate::api::ApiClient;
use crate::config::ConfigManager;
use crate::utils::error::CarpResult;
use colored::*;

/// Execute the search command
pub async fn execute(query: String, limit: Option<usize>, exact: bool, verbose: bool) -> CarpResult<()> {
    if verbose {
        println!("Searching for agents matching '{}'...", query);
    }
    
    let config = ConfigManager::load()?;
    let client = ApiClient::new(&config)?;
    
    let response = client.search(&query, limit, exact).await?;
    
    if response.agents.is_empty() {
        println!("{}", "No agents found matching your search.".yellow());
        return Ok(());
    }
    
    println!("{} {} agents found:\n", "Found".green().bold(), response.total);
    
    let agents_count = response.agents.len();
    for agent in &response.agents {
        println!("{} {}", agent.name.bold().blue(), agent.version.dimmed());
        println!("  {}", agent.description);
        println!("  by {} • {} downloads", agent.author.green(), agent.download_count.to_string().cyan());
        
        if !agent.tags.is_empty() {
            print!("  tags: ");
            for (i, tag) in agent.tags.iter().enumerate() {
                if i > 0 { print!(", "); }
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
    
    if response.total > agents_count {
        println!("Showing {} of {} results. Use --limit to see more.", 
                agents_count, response.total);
    }
    
    Ok(())
}