use crate::utils::error::{CarpError, CarpResult};
use crate::utils::manifest::AgentManifest;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Execute the new command to create an agent template
pub async fn execute(
    name: String,
    path: Option<String>,
    template: Option<String>,
    verbose: bool,
) -> CarpResult<()> {
    // Validate agent name
    validate_agent_name(&name)?;

    // Determine target directory
    let target_dir = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&name));

    if target_dir.exists() {
        return Err(CarpError::FileSystem(format!(
            "Directory '{}' already exists. Choose a different path or remove the existing directory.",
            target_dir.display()
        )));
    }

    let template_type = template.as_deref().unwrap_or("basic");

    if verbose {
        println!(
            "Creating new agent '{}' using '{}' template...",
            name, template_type
        );
    }

    // Create directory structure
    create_directory_structure(&target_dir, &name, template_type, verbose).await?;

    println!(
        "{} Successfully created agent '{}'",
        "✓".green().bold(),
        name.blue().bold()
    );
    println!("Directory: {}", target_dir.display().to_string().cyan());
    println!("\nNext steps:");
    println!("  cd {}", target_dir.display());
    println!("  # Edit the Carp.toml file with your agent details");
    println!("  # Implement your agent logic in agent.py");
    println!("  # Test locally, then run 'carp publish' when ready");

    Ok(())
}

/// Validate the agent name
fn validate_agent_name(name: &str) -> CarpResult<()> {
    if name.is_empty() {
        return Err(CarpError::InvalidAgent(
            "Agent name cannot be empty".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CarpError::InvalidAgent(
            "Agent name can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        ));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(CarpError::InvalidAgent(
            "Agent name cannot start or end with a hyphen".to_string(),
        ));
    }

    if name.len() > 50 {
        return Err(CarpError::InvalidAgent(
            "Agent name cannot be longer than 50 characters".to_string(),
        ));
    }

    Ok(())
}

/// Create the directory structure and files for the new agent
async fn create_directory_structure(
    target_dir: &Path,
    name: &str,
    template_type: &str,
    verbose: bool,
) -> CarpResult<()> {
    // Create main directory
    fs::create_dir_all(target_dir)?;

    match template_type {
        "basic" => create_basic_template(target_dir, name, verbose).await?,
        "advanced" => create_advanced_template(target_dir, name, verbose).await?,
        "python" => create_python_template(target_dir, name, verbose).await?,
        _ => {
            return Err(CarpError::InvalidAgent(format!(
                "Unknown template type '{}'. Available: basic, advanced, python",
                template_type
            )));
        }
    }

    Ok(())
}

/// Create a basic agent template
async fn create_basic_template(target_dir: &Path, name: &str, verbose: bool) -> CarpResult<()> {
    if verbose {
        println!("Creating basic template structure...");
    }

    // Create manifest
    let manifest = AgentManifest::template(name);
    manifest.save(target_dir.join("Carp.toml"))?;

    // Create README.md
    let readme_content = format!(
        r#"# {}

A Claude AI agent created with Carp.

## Description

TODO: Describe what your agent does and how to use it.

## Usage

TODO: Provide usage instructions for your agent.

## Configuration

TODO: Document any configuration options.

## License

MIT
"#,
        name
    );

    fs::write(target_dir.join("README.md"), readme_content)?;

    // Create basic agent script
    let agent_content = r#"#!/usr/bin/env python3
"""
Basic Claude AI Agent Template

This is a template for creating Claude AI agents.
Customize this file to implement your agent's specific functionality.
"""

import json
import sys
from typing import Dict, Any

class Agent:
    """Basic Claude AI Agent"""
    
    def __init__(self, config: Dict[str, Any] = None):
        """Initialize the agent with optional configuration."""
        self.config = config or {}
        self.name = self.config.get('name', 'Basic Agent')
        self.version = self.config.get('version', '0.1.0')
    
    def process(self, input_data: str) -> str:
        """Process input and return output."""
        # TODO: Implement your agent logic here
        return f"Hello from {self.name}! You said: {input_data}"
    
    def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle a structured request."""
        try:
            input_data = request.get('input', '')
            result = self.process(input_data)
            
            return {
                'success': True,
                'result': result,
                'agent': {
                    'name': self.name,
                    'version': self.version
                }
            }
        except Exception as e:
            return {
                'success': False,
                'error': str(e),
                'agent': {
                    'name': self.name,
                    'version': self.version
                }
            }

def main():
    """Main entry point for the agent."""
    # Load configuration if available
    config = {}
    try:
        with open('config.toml', 'r') as f:
            # Basic TOML parsing (you might want to use a proper TOML library)
            pass
    except FileNotFoundError:
        pass
    
    agent = Agent(config)
    
    if len(sys.argv) > 1:
        # Command line input
        input_data = ' '.join(sys.argv[1:])
        result = agent.process(input_data)
        print(result)
    else:
        # Interactive mode or JSON input
        try:
            line = input()
            request = json.loads(line)
            response = agent.handle_request(request)
            print(json.dumps(response))
        except (EOFError, KeyboardInterrupt):
            pass
        except json.JSONDecodeError:
            # Treat as plain text input
            result = agent.process(line)
            print(result)

if __name__ == '__main__':
    main()
"#;

    fs::write(target_dir.join("agent.py"), agent_content)?;

    // Create basic config file
    let config_content = r#"# Configuration for your Claude AI agent
# Customize these settings as needed

[agent]
name = "My Agent"
version = "0.1.0"
debug = false

[settings]
# Add your agent-specific settings here
timeout = 30
max_retries = 3
"#;

    fs::write(target_dir.join("config.toml"), config_content)?;

    // Create .gitignore
    let gitignore_content = r#"# Python
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
env/
venv/
ENV/
env.bak/
venv.bak/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Agent specific
.carp/
*.tmp
"#;

    fs::write(target_dir.join(".gitignore"), gitignore_content)?;

    Ok(())
}

/// Create an advanced agent template with more structure
async fn create_advanced_template(target_dir: &Path, name: &str, verbose: bool) -> CarpResult<()> {
    if verbose {
        println!("Creating advanced template structure...");
    }

    // Create basic template first
    create_basic_template(target_dir, name, verbose).await?;

    // Add additional directories and files
    fs::create_dir_all(target_dir.join("src"))?;
    fs::create_dir_all(target_dir.join("tests"))?;
    fs::create_dir_all(target_dir.join("docs"))?;

    // Create a more sophisticated agent structure
    let main_agent_content = r#"#!/usr/bin/env python3
"""
Advanced Claude AI Agent Template

This template provides a more structured approach to building Claude AI agents
with proper error handling, logging, and modular design.
"""

import logging
import json
import sys
from pathlib import Path
from typing import Dict, Any, Optional

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent / "src"))

from agent_core import AgentCore
from config_manager import ConfigManager

class AdvancedAgent(AgentCore):
    """Advanced Claude AI Agent with enhanced capabilities."""
    
    def __init__(self, config_path: Optional[str] = None):
        """Initialize the advanced agent."""
        self.config_manager = ConfigManager(config_path or "config.toml")
        config = self.config_manager.load_config()
        
        super().__init__(config)
        
        # Set up logging
        log_level = getattr(logging, config.get('log_level', 'INFO').upper())
        logging.basicConfig(level=log_level)
        self.logger = logging.getLogger(__name__)
        
        self.logger.info(f"Initialized {self.name} v{self.version}")
    
    def process(self, input_data: str) -> str:
        """Process input with enhanced error handling and logging."""
        self.logger.debug(f"Processing input: {input_data[:100]}...")
        
        try:
            # Your custom processing logic here
            result = f"Advanced processing from {self.name}: {input_data}"
            
            self.logger.info("Processing completed successfully")
            return result
            
        except Exception as e:
            self.logger.error(f"Processing failed: {e}")
            raise

def main():
    """Main entry point with argument parsing."""
    import argparse
    
    parser = argparse.ArgumentParser(description=f'{name} - Advanced Claude AI Agent')
    parser.add_argument('--config', '-c', help='Configuration file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose logging')
    parser.add_argument('input', nargs='*', help='Input text to process')
    
    args = parser.parse_args()
    
    # Override log level if verbose
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    agent = AdvancedAgent(args.config)
    
    if args.input:
        # Command line input
        input_text = ' '.join(args.input)
        result = agent.process(input_text)
        print(result)
    else:
        # Interactive or JSON mode
        try:
            for line in sys.stdin:
                line = line.strip()
                if not line:
                    continue
                    
                try:
                    # Try to parse as JSON
                    request = json.loads(line)
                    response = agent.handle_request(request)
                    print(json.dumps(response))
                except json.JSONDecodeError:
                    # Treat as plain text
                    result = agent.process(line)
                    print(result)
                    
        except (EOFError, KeyboardInterrupt):
            pass

if __name__ == '__main__':
    main()
"#;

    fs::write(target_dir.join("agent.py"), main_agent_content)?;

    // Create core agent module
    let core_content = r#""""
Core agent functionality
"""

from typing import Dict, Any
import json

class AgentCore:
    """Base class for Claude AI agents."""
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize core agent functionality."""
        self.config = config
        self.name = config.get('name', 'Agent')
        self.version = config.get('version', '0.1.0')
    
    def process(self, input_data: str) -> str:
        """Override this method to implement agent logic."""
        raise NotImplementedError("Subclasses must implement process()")
    
    def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle structured JSON requests."""
        try:
            input_data = request.get('input', '')
            result = self.process(input_data)
            
            return {
                'success': True,
                'result': result,
                'agent': {
                    'name': self.name,
                    'version': self.version
                }
            }
        except Exception as e:
            return {
                'success': False,
                'error': str(e),
                'agent': {
                    'name': self.name,
                    'version': self.version
                }
            }
"#;

    fs::write(target_dir.join("src/agent_core.py"), core_content)?;

    // Create config manager
    let config_manager_content = r#""""
Configuration management utilities
"""

import toml
from typing import Dict, Any
from pathlib import Path

class ConfigManager:
    """Manages agent configuration."""
    
    def __init__(self, config_path: str):
        """Initialize with config file path."""
        self.config_path = Path(config_path)
    
    def load_config(self) -> Dict[str, Any]:
        """Load configuration from file."""
        if not self.config_path.exists():
            return self._default_config()
        
        try:
            with open(self.config_path, 'r') as f:
                return toml.load(f)
        except Exception as e:
            print(f"Warning: Failed to load config: {e}")
            return self._default_config()
    
    def _default_config(self) -> Dict[str, Any]:
        """Return default configuration."""
        return {
            'name': 'Advanced Agent',
            'version': '0.1.0',
            'log_level': 'INFO',
            'timeout': 30,
            'max_retries': 3
        }
"#;

    fs::write(
        target_dir.join("src/config_manager.py"),
        config_manager_content,
    )?;

    // Create requirements.txt
    let requirements_content = r#"toml>=0.10.0
"#;

    fs::write(target_dir.join("requirements.txt"), requirements_content)?;

    Ok(())
}

/// Create a Python-specific template
async fn create_python_template(target_dir: &Path, name: &str, verbose: bool) -> CarpResult<()> {
    if verbose {
        println!("Creating Python template structure...");
    }

    // Create advanced template and add Python-specific features
    create_advanced_template(target_dir, name, verbose).await?;

    // Add setup.py for proper Python packaging
    let setup_content = format!(
        r#"from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="{name}",
    version="0.1.0",
    author="Your Name",
    author_email="your.email@example.com",
    description="A Claude AI agent created with Carp",
    long_description=long_description,
    long_description_content_type="text/markdown",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.8",
    install_requires=[
        "toml>=0.10.0",
    ],
    entry_points={{
        "console_scripts": [
            "{name}=agent:main",
        ],
    }},
)
"#,
        name = name
    );

    fs::write(target_dir.join("setup.py"), setup_content)?;

    // Add basic test
    let test_content = r#"import unittest
from src.agent_core import AgentCore

class TestAgent(unittest.TestCase):
    def setUp(self):
        self.config = {
            'name': 'Test Agent',
            'version': '0.1.0'
        }
    
    def test_agent_initialization(self):
        agent = AgentCore(self.config)
        self.assertEqual(agent.name, 'Test Agent')
        self.assertEqual(agent.version, '0.1.0')

if __name__ == '__main__':
    unittest.main()
"#;

    fs::write(target_dir.join("tests/test_agent.py"), test_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_agent_name() {
        assert!(validate_agent_name("valid-name").is_ok());
        assert!(validate_agent_name("valid_name").is_ok());
        assert!(validate_agent_name("valid123").is_ok());

        assert!(validate_agent_name("").is_err());
        assert!(validate_agent_name("-invalid").is_err());
        assert!(validate_agent_name("invalid-").is_err());
        assert!(validate_agent_name("invalid name").is_err());
        assert!(validate_agent_name("invalid@name").is_err());
    }
}
