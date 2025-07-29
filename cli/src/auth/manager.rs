use crate::config::ConfigManager;
use crate::utils::error::{CarpError, CarpResult};
use colored::*;

/// Authentication manager for handling login/logout
pub struct AuthManager;

impl AuthManager {
    /// Set API key for authentication
    pub async fn set_api_key() -> CarpResult<()> {
        println!("{}", "Set API Key for Carp Registry".bold().green());
        println!("Enter your API key (input will be hidden):");

        let api_key = rpassword::prompt_password("API Key: ")?;

        if api_key.trim().is_empty() {
            return Err(CarpError::Auth("API key cannot be empty".to_string()));
        }

        println!("Validating API key...");

        // Validate the API key format
        ConfigManager::set_api_key_secure(api_key)?;
        
        println!("{}", "API key saved successfully!".green().bold());
        println!("You can now use authenticated commands.");
        Ok(())
    }

    /// Legacy login method (deprecated)
    #[deprecated(note = "Use set_api_key instead. Username/password authentication is deprecated.")]
    pub async fn login() -> CarpResult<()> {
        println!("{}", "Username/password login is deprecated.".yellow().bold());
        println!("Please use API key authentication instead:");
        println!("  Run: carp auth set-api-key");
        println!("  Or: set CARP_API_KEY environment variable");
        println!("  Or: use --api-key command line option");
        
        Err(CarpError::Auth("Please use API key authentication instead of username/password".to_string()))
    }

    /// Logout by clearing the stored API key
    pub async fn logout() -> CarpResult<()> {
        ConfigManager::clear_api_key()?;
        println!("{}", "Successfully logged out!".green().bold());
        println!("API key has been removed from configuration.");
        Ok(())
    }

    /// Check if user is currently authenticated
    pub async fn check_auth() -> CarpResult<bool> {
        let config = ConfigManager::load()?;
        Ok(config.api_key.is_some())
    }

    /// Check if user is currently authenticated, considering runtime API key
    pub async fn check_auth_with_key(api_key: Option<&str>) -> CarpResult<bool> {
        if api_key.is_some() {
            return Ok(true);
        }
        Self::check_auth().await
    }

    /// Get current authentication status
    pub async fn status() -> CarpResult<()> {
        Self::status_with_key(None).await
    }

    /// Get current authentication status, considering runtime API key
    pub async fn status_with_key(runtime_api_key: Option<&str>) -> CarpResult<()> {
        let config = ConfigManager::load()?;
        
        // Determine which API key to use (runtime takes precedence)
        let api_key = runtime_api_key.or(config.api_key.as_deref());
        
        if api_key.is_some() {
            println!("{}", "Authenticated".green().bold());
            println!("Registry: {}", config.registry_url);
            
            if let Some(key) = api_key {
                // Show only first and last few characters for security
                let masked_key = if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len()-4..])
                } else {
                    "****".to_string()
                };
                
                let source = if runtime_api_key.is_some() {
                    "command line/environment"
                } else {
                    "config file"
                };
                println!("API Key: {} (masked, from {})", masked_key, source);
            }

            println!("Status: {}", "Ready to use authenticated commands".green());
        } else {
            println!("{}", "Not authenticated".red().bold());
            println!("Authenticate using one of these methods:");
            println!("  1. Run: carp auth set-api-key");
            println!("  2. Set CARP_API_KEY environment variable");
            println!("  3. Use --api-key command line option");
        }
        Ok(())
    }

    /// Ensure user is authenticated, prompt to login if not
    pub async fn ensure_authenticated(api_key: Option<&str>) -> CarpResult<()> {
        if !Self::check_auth_with_key(api_key).await? {
            println!("{}", "Authentication required.".yellow().bold());
            println!("You can authenticate by:");
            println!("  1. Setting CARP_API_KEY environment variable");
            println!("  2. Using --api-key command line option");
            println!("  3. Running 'carp login' to store API key in config");
            return Err(CarpError::Auth("No API key provided. Please authenticate to continue.".to_string()));
        }
        Ok(())
    }
}
