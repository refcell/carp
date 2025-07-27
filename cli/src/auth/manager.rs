use crate::api::ApiClient;
use crate::config::{Config, ConfigManager};
use crate::utils::error::{CarpError, CarpResult};
use colored::*;
use std::io::{self, Write};

/// Authentication manager for handling login/logout
pub struct AuthManager;

impl AuthManager {
    /// Prompt user for credentials and authenticate
    pub async fn login() -> CarpResult<()> {
        let config = ConfigManager::load()?;
        let client = ApiClient::new(&config)?;

        println!("{}", "Login to Carp Registry".bold().green());
        println!("Enter your credentials:");

        print!("Username: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        let username = username.trim();

        let password = rpassword::prompt_password("Password: ")?;

        println!("Authenticating...");

        match client.authenticate(username, &password).await {
            Ok(auth_response) => {
                ConfigManager::set_api_token(auth_response.token)?;
                println!("{}", "Successfully logged in!".green().bold());
                println!("Token expires: {}", auth_response.expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
                Ok(())
            }
            Err(e) => {
                println!("{} {}", "Login failed:".red().bold(), e);
                Err(e)
            }
        }
    }

    /// Logout by clearing the stored API token
    pub async fn logout() -> CarpResult<()> {
        ConfigManager::clear_api_token()?;
        println!("{}", "Successfully logged out!".green().bold());
        Ok(())
    }

    /// Check if user is currently authenticated
    pub async fn check_auth() -> CarpResult<bool> {
        let config = ConfigManager::load()?;
        Ok(config.api_token.is_some())
    }

    /// Get current authentication status
    pub async fn status() -> CarpResult<()> {
        if Self::check_auth().await? {
            println!("{}", "Authenticated".green().bold());
            
            let config = ConfigManager::load()?;
            println!("Registry: {}", config.registry_url);
            
            // Try to validate token by making a test request
            let client = ApiClient::new(&config)?;
            match client.search("", Some(1), false).await {
                Ok(_) => println!("Token: {}", "Valid".green()),
                Err(_) => {
                    println!("Token: {}", "Invalid or expired".red());
                    println!("Run 'carp login' to authenticate again.");
                }
            }
        } else {
            println!("{}", "Not authenticated".red().bold());
            println!("Run 'carp login' to authenticate.");
        }
        Ok(())
    }

    /// Ensure user is authenticated, prompt to login if not
    pub async fn ensure_authenticated() -> CarpResult<()> {
        if !Self::check_auth().await? {
            println!("{}", "Authentication required.".yellow().bold());
            Self::login().await
        } else {
            Ok(())
        }
    }
}