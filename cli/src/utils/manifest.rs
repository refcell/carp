use crate::utils::error::{CarpError, CarpResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Agent manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    /// Agent name
    pub name: String,
    /// Agent version (semver)
    pub version: String,
    /// Short description
    pub description: String,
    /// Author information
    pub author: String,
    /// License identifier
    pub license: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// List of files to include in the package
    pub files: Vec<String>,
    /// Entry point script or configuration
    pub main: Option<String>,
    /// Dependencies on other agents
    pub dependencies: Option<std::collections::HashMap<String, String>>,
}

impl AgentManifest {
    /// Load manifest from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> CarpResult<Self> {
        let contents = fs::read_to_string(&path)
            .map_err(|e| CarpError::ManifestError(format!("Failed to read manifest: {e}")))?;

        let manifest: AgentManifest = toml::from_str(&contents)
            .map_err(|e| CarpError::ManifestError(format!("Failed to parse manifest: {e}")))?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Save manifest to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> CarpResult<()> {
        self.validate()?;

        let contents = toml::to_string_pretty(self).map_err(|e| {
            CarpError::ManifestError(format!("Failed to serialize manifest: {e}"))
        })?;

        fs::write(&path, contents)
            .map_err(|e| CarpError::ManifestError(format!("Failed to write manifest: {e}")))?;

        Ok(())
    }

    /// Validate the manifest
    pub fn validate(&self) -> CarpResult<()> {
        if self.name.is_empty() {
            return Err(CarpError::ManifestError(
                "Agent name cannot be empty".to_string(),
            ));
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(CarpError::ManifestError(
                "Agent name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        if self.version.is_empty() {
            return Err(CarpError::ManifestError(
                "Version cannot be empty".to_string(),
            ));
        }

        // Basic semver validation
        if !self
            .version
            .split('.')
            .all(|part| part.chars().all(|c| c.is_numeric()))
        {
            return Err(CarpError::ManifestError(
                "Version must be in semver format (e.g., 1.0.0)".to_string(),
            ));
        }

        if self.description.is_empty() {
            return Err(CarpError::ManifestError(
                "Description cannot be empty".to_string(),
            ));
        }

        if self.author.is_empty() {
            return Err(CarpError::ManifestError(
                "Author cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a default manifest template
    pub fn template(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: format!("A Claude AI agent named {name}"),
            author: "Your Name <your.email@example.com>".to_string(),
            license: Some("MIT".to_string()),
            homepage: None,
            repository: None,
            tags: vec!["claude".to_string(), "ai".to_string()],
            files: vec![
                "README.md".to_string(),
                "agent.py".to_string(),
                "config.toml".to_string(),
            ],
            main: Some("agent.py".to_string()),
            dependencies: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let mut manifest = AgentManifest::template("test-agent");
        assert!(manifest.validate().is_ok());

        // Test empty name
        manifest.name = "".to_string();
        assert!(manifest.validate().is_err());

        // Test invalid name
        manifest.name = "test agent!".to_string();
        assert!(manifest.validate().is_err());

        // Test invalid version
        manifest.name = "test-agent".to_string();
        manifest.version = "invalid".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = AgentManifest::template("test-agent");
        let toml_str = toml::to_string(&manifest).unwrap();
        let deserialized: AgentManifest = toml::from_str(&toml_str).unwrap();

        assert_eq!(manifest.name, deserialized.name);
        assert_eq!(manifest.version, deserialized.version);
        assert_eq!(manifest.description, deserialized.description);
    }
}
