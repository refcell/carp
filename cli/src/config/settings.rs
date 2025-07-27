use crate::utils::error::{CarpError, CarpResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration structure for the Carp CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Registry API base URL
    pub registry_url: String,
    /// User API token for authentication
    pub api_token: Option<String>,
    /// Default timeout for API requests in seconds
    pub timeout: u64,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    /// Default output directory for pulled agents
    pub default_output_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry_url: "https://api.carp.refcell.org".to_string(),
            api_token: None,
            timeout: 30,
            verify_ssl: true,
            default_output_dir: None,
        }
    }
}

/// Configuration manager for loading and saving config
pub struct ConfigManager;

impl ConfigManager {
    /// Get the path to the config file
    pub fn config_path() -> CarpResult<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CarpError::Config("Unable to find config directory".to_string()))?;
        
        let carp_dir = config_dir.join("carp");
        if !carp_dir.exists() {
            fs::create_dir_all(&carp_dir)?;
        }
        
        Ok(carp_dir.join("config.toml"))
    }
    
    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> CarpResult<Config> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let default_config = Config::default();
            Self::save(&default_config)?;
            return Ok(default_config);
        }
        
        let contents = fs::read_to_string(&config_path)
            .map_err(|e| CarpError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&contents)?;
        
        // Validate registry URL
        Self::validate_registry_url(&config.registry_url)?;
        
        // Ensure HTTPS for security
        if !config.registry_url.starts_with("https://") {
            eprintln!("Warning: Registry URL is not using HTTPS. This is insecure.");
        }
        
        Ok(config)
    }
    
    /// Validate registry URL format and security
    fn validate_registry_url(url: &str) -> CarpResult<()> {
        // Basic URL validation
        if url.is_empty() {
            return Err(CarpError::Config("Registry URL cannot be empty".to_string()));
        }
        
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(CarpError::Config(
                "Registry URL must start with http:// or https://".to_string()
            ));
        }
        
        // Parse URL to validate format
        if let Err(_) = url.parse::<reqwest::Url>() {
            return Err(CarpError::Config(
                "Invalid registry URL format".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save(config: &Config) -> CarpResult<()> {
        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(config)
            .map_err(|e| CarpError::Config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&config_path, contents)
            .map_err(|e| CarpError::Config(format!("Failed to write config file: {}", e)))?;
        
        // Set restrictive permissions on config file (600 - owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&config_path, perms)?;
        }
        
        Ok(())
    }
    
    /// Update the API token in the config
    pub fn set_api_token(token: String) -> CarpResult<()> {
        let mut config = Self::load()?;
        config.api_token = Some(token);
        Self::save(&config)
    }
    
    /// Clear the API token from the config
    pub fn clear_api_token() -> CarpResult<()> {
        let mut config = Self::load()?;
        config.api_token = None;
        Self::save(&config)
    }
    
    /// Get the cache directory for storing downloaded agents
    pub fn cache_dir() -> CarpResult<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| CarpError::Config("Unable to find cache directory".to_string()))?;
        
        let carp_cache = cache_dir.join("carp");
        if !carp_cache.exists() {
            fs::create_dir_all(&carp_cache)?;
        }
        
        Ok(carp_cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.registry_url, "https://api.carp.refcell.org");
        assert!(config.api_token.is_none());
        assert_eq!(config.timeout, 30);
        assert!(config.verify_ssl);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.registry_url, deserialized.registry_url);
        assert_eq!(config.timeout, deserialized.timeout);
    }
}