use crate::utils::error::{CarpError, CarpResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration structure for the Carp CLI
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// Registry API base URL
    pub registry_url: String,
    /// User API key for authentication
    pub api_key: Option<String>,
    /// Legacy API token field (deprecated, use api_key instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    /// Default timeout for API requests in seconds
    pub timeout: u64,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    /// Default output directory for pulled agents
    pub default_output_dir: Option<String>,
    /// Maximum number of concurrent downloads
    #[serde(default = "default_max_concurrent_downloads")]
    pub max_concurrent_downloads: u32,
    /// Request retry configuration
    #[serde(default)]
    pub retry: RetrySettings,
    /// Security settings
    #[serde(default)]
    pub security: SecuritySettings,
}

/// Retry configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySettings {
    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,
    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
}

/// Security configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Maximum download size in bytes
    #[serde(default = "default_max_download_size")]
    pub max_download_size: u64,
    /// Maximum publish size in bytes
    #[serde(default = "default_max_publish_size")]
    pub max_publish_size: u64,
    /// Whether to allow HTTP URLs (insecure)
    #[serde(default)]
    pub allow_http: bool,
    /// Token expiry warning threshold in hours
    #[serde(default = "default_token_warning_hours")]
    pub token_warning_hours: u64,
}

// Default value functions
fn default_max_concurrent_downloads() -> u32 {
    4
}
fn default_max_retries() -> u32 {
    3
}
fn default_initial_delay_ms() -> u64 {
    100
}
fn default_max_delay_ms() -> u64 {
    5000
}
fn default_backoff_multiplier() -> f64 {
    2.0
}
fn default_max_download_size() -> u64 {
    100 * 1024 * 1024
} // 100MB
fn default_max_publish_size() -> u64 {
    50 * 1024 * 1024
} // 50MB
fn default_token_warning_hours() -> u64 {
    24
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay_ms: default_initial_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            backoff_multiplier: default_backoff_multiplier(),
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            max_download_size: default_max_download_size(),
            max_publish_size: default_max_publish_size(),
            allow_http: false,
            token_warning_hours: default_token_warning_hours(),
        }
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("registry_url", &self.registry_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field("api_token", &self.api_token.as_ref().map(|_| "***"))
            .field("timeout", &self.timeout)
            .field("verify_ssl", &self.verify_ssl)
            .field("default_output_dir", &self.default_output_dir)
            .field("max_concurrent_downloads", &self.max_concurrent_downloads)
            .field("retry", &self.retry)
            .field("security", &self.security)
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry_url: "https://api.carp.refcell.org".to_string(),
            api_key: None,
            api_token: None,
            timeout: 30,
            verify_ssl: true,
            default_output_dir: None,
            max_concurrent_downloads: default_max_concurrent_downloads(),
            retry: RetrySettings::default(),
            security: SecuritySettings::default(),
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

        let mut config = if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .map_err(|e| CarpError::Config(format!("Failed to read config file: {e}")))?;

            toml::from_str::<Config>(&contents)?
        } else {
            let default_config = Config::default();
            Self::save(&default_config)?;
            default_config
        };

        // Override with environment variables if present
        Self::apply_env_overrides(&mut config)?;

        // Handle backward compatibility: migrate api_token to api_key
        Self::migrate_legacy_token(&mut config)?;

        // Validate configuration
        Self::validate_config(&config)?;

        Ok(config)
    }

    /// Migrate legacy api_token to api_key for backward compatibility
    fn migrate_legacy_token(config: &mut Config) -> CarpResult<()> {
        // If we have an api_token but no api_key, migrate it
        if config.api_key.is_none() && config.api_token.is_some() {
            config.api_key = config.api_token.take();
            // Save the migrated config
            if let Err(e) = Self::save(config) {
                eprintln!("Warning: Failed to save migrated configuration: {e}");
            } else {
                eprintln!("Info: Migrated api_token to api_key in configuration file.");
            }
        }
        Ok(())
    }

    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(config: &mut Config) -> CarpResult<()> {
        // Registry URL
        if let Ok(url) = std::env::var("CARP_REGISTRY_URL") {
            config.registry_url = url;
        }

        // API Key (new environment variable)
        if let Ok(api_key) = std::env::var("CARP_API_KEY") {
            config.api_key = Some(api_key);
        }
        // API Token (legacy environment variable for backward compatibility)
        else if let Ok(api_token) = std::env::var("CARP_API_TOKEN") {
            eprintln!("Warning: CARP_API_TOKEN is deprecated. Please use CARP_API_KEY instead.");
            config.api_key = Some(api_token);
        }

        // Timeout
        if let Ok(timeout_str) = std::env::var("CARP_TIMEOUT") {
            config.timeout = timeout_str
                .parse()
                .map_err(|_| CarpError::Config("Invalid CARP_TIMEOUT value".to_string()))?;
        }

        // SSL Verification
        if let Ok(verify_ssl_str) = std::env::var("CARP_VERIFY_SSL") {
            config.verify_ssl = verify_ssl_str
                .parse()
                .map_err(|_| CarpError::Config("Invalid CARP_VERIFY_SSL value".to_string()))?;
        }

        // Output Directory
        if let Ok(output_dir) = std::env::var("CARP_OUTPUT_DIR") {
            config.default_output_dir = Some(output_dir);
        }

        // Allow HTTP (for development/testing)
        if let Ok(allow_http_str) = std::env::var("CARP_ALLOW_HTTP") {
            config.security.allow_http = allow_http_str
                .parse()
                .map_err(|_| CarpError::Config("Invalid CARP_ALLOW_HTTP value".to_string()))?;
        }

        Ok(())
    }

    /// Validate the complete configuration
    fn validate_config(config: &Config) -> CarpResult<()> {
        // Validate registry URL
        Self::validate_registry_url(&config.registry_url)?;

        // Security checks
        if !config.security.allow_http && !config.registry_url.starts_with("https://") {
            return Err(CarpError::Config(
                "Registry URL must use HTTPS for security. Set allow_http=true in config to override.".to_string()
            ));
        }

        // Validate timeout
        if config.timeout == 0 || config.timeout > 300 {
            return Err(CarpError::Config(
                "Timeout must be between 1 and 300 seconds".to_string(),
            ));
        }

        // Validate retry settings
        if config.retry.max_retries > 10 {
            return Err(CarpError::Config(
                "Maximum retries cannot exceed 10".to_string(),
            ));
        }

        if config.retry.initial_delay_ms > 60000 {
            return Err(CarpError::Config(
                "Initial retry delay cannot exceed 60 seconds".to_string(),
            ));
        }

        if config.retry.max_delay_ms > 300000 {
            return Err(CarpError::Config(
                "Maximum retry delay cannot exceed 5 minutes".to_string(),
            ));
        }

        // Validate security settings
        if config.security.max_download_size > 1024 * 1024 * 1024 {
            // 1GB
            return Err(CarpError::Config(
                "Maximum download size cannot exceed 1GB".to_string(),
            ));
        }

        if config.security.max_publish_size > 200 * 1024 * 1024 {
            // 200MB
            return Err(CarpError::Config(
                "Maximum publish size cannot exceed 200MB".to_string(),
            ));
        }

        // Warn about insecure settings
        if !config.verify_ssl {
            eprintln!("Warning: SSL verification is disabled. This is insecure and not recommended for production use.");
        }

        if config.security.allow_http {
            eprintln!("Warning: HTTP URLs are allowed. This is insecure and not recommended for production use.");
        }

        Ok(())
    }

    /// Validate registry URL format and security
    fn validate_registry_url(url: &str) -> CarpResult<()> {
        // Basic URL validation
        if url.is_empty() {
            return Err(CarpError::Config(
                "Registry URL cannot be empty".to_string(),
            ));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(CarpError::Config(
                "Registry URL must start with http:// or https://".to_string(),
            ));
        }

        // Parse URL to validate format
        if url.parse::<reqwest::Url>().is_err() {
            return Err(CarpError::Config("Invalid registry URL format".to_string()));
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save(config: &Config) -> CarpResult<()> {
        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(config)
            .map_err(|e| CarpError::Config(format!("Failed to serialize config: {e}")))?;

        fs::write(&config_path, contents)
            .map_err(|e| CarpError::Config(format!("Failed to write config file: {e}")))?;

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

    /// Update the API key in the config
    pub fn set_api_key(api_key: String) -> CarpResult<()> {
        let mut config = Self::load()?;
        config.api_key = Some(api_key);
        config.api_token = None; // Clear legacy token
        Self::save(&config)
    }

    /// Clear the API key from the config
    pub fn clear_api_key() -> CarpResult<()> {
        let mut config = Self::load()?;
        config.api_key = None;
        config.api_token = None; // Also clear legacy token
        Self::save(&config)
    }

    /// Legacy method for backward compatibility
    #[deprecated(note = "Use set_api_key instead")]
    pub fn set_api_token(token: String) -> CarpResult<()> {
        Self::set_api_key(token)
    }

    /// Legacy method for backward compatibility
    #[deprecated(note = "Use clear_api_key instead")]
    pub fn clear_api_token() -> CarpResult<()> {
        Self::clear_api_key()
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

    /// Get configuration with runtime environment checks
    pub fn load_with_env_checks() -> CarpResult<Config> {
        let config = Self::load()?;

        // Check for common CI/CD environment variables and adjust settings
        if Self::is_ci_environment() {
            eprintln!("Detected CI/CD environment. Using stricter security settings.");
        }

        // Validate API key if present
        if let Some(api_key) = &config.api_key {
            Self::validate_api_key(api_key)?;
        }

        Ok(config)
    }

    /// Check if running in a CI/CD environment
    fn is_ci_environment() -> bool {
        std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GITLAB_CI").is_ok()
            || std::env::var("JENKINS_URL").is_ok()
            || std::env::var("BUILDKITE").is_ok()
    }

    /// Validate API key format and basic security checks
    pub fn validate_api_key(api_key: &str) -> CarpResult<()> {
        if api_key.is_empty() {
            return Err(CarpError::Auth("Empty API key".to_string()));
        }

        // Basic API key format validation
        if api_key.len() < 8 {
            return Err(CarpError::Auth(
                "API key too short (minimum 8 characters)".to_string(),
            ));
        }

        // Check for potentially unsafe characters
        if api_key.contains(['\n', '\r', '\t', ' ']) {
            return Err(CarpError::Auth(
                "API key contains invalid characters".to_string(),
            ));
        }

        // Warn about potentially insecure keys
        if api_key.starts_with("test_") || api_key.starts_with("dev_") {
            eprintln!("Warning: API key appears to be for development/testing. Ensure you're using a production key for live environments.");
        }

        Ok(())
    }

    /// Securely update API key with validation
    pub fn set_api_key_secure(api_key: String) -> CarpResult<()> {
        // Validate API key format
        Self::validate_api_key(&api_key)?;

        let mut config = Self::load()?;
        config.api_key = Some(api_key);
        config.api_token = None; // Clear legacy token
        Self::save(&config)?;

        println!("API key updated successfully.");
        Ok(())
    }

    /// Legacy method for backward compatibility
    #[deprecated(note = "Use set_api_key_secure instead")]
    pub fn set_api_token_secure(token: String) -> CarpResult<()> {
        Self::set_api_key_secure(token)
    }

    /// Export configuration template for deployment
    pub fn export_template() -> CarpResult<String> {
        let template_config = Config {
            registry_url: "${CARP_REGISTRY_URL:-https://api.carp.refcell.org}".to_string(),
            api_key: None,   // Never include API keys in templates
            api_token: None, // Never include legacy tokens in templates
            timeout: 30,
            verify_ssl: true,
            default_output_dir: Some("${CARP_OUTPUT_DIR:-./agents}".to_string()),
            max_concurrent_downloads: 4,
            retry: RetrySettings::default(),
            security: SecuritySettings::default(),
        };

        let template = toml::to_string_pretty(&template_config)
            .map_err(|e| CarpError::Config(format!("Failed to generate template: {e}")))?;

        Ok(format!(
            "# Carp CLI Configuration Template\n# Environment variables will be substituted at runtime\n# Copy this file to ~/.config/carp/config.toml and customize as needed\n# Set CARP_API_KEY environment variable or add api_key field for authentication\n\n{template}"
        ))
    }

    /// Validate configuration file without loading sensitive data
    pub fn validate_config_file(path: &PathBuf) -> CarpResult<()> {
        if !path.exists() {
            return Err(CarpError::Config(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| CarpError::Config(format!("Failed to read config file: {e}")))?;

        // Parse without loading into full config to check syntax
        let _: toml::Value = toml::from_str(&contents)
            .map_err(|e| CarpError::Config(format!("Invalid TOML syntax: {e}")))?;

        println!("Configuration file syntax is valid.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
