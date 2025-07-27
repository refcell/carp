# Carp CLI

Command-line tool for the Claude Agent Registry Portal (Carp).

## Features

- **Search Agents**: Find agents in the registry with powerful search functionality
- **Pull Agents**: Download and extract agents locally for use
- **Publish Agents**: Package and upload your agents to the registry
- **Create Templates**: Generate new agent projects with built-in templates

## Installation

### From Source

```bash
git clone https://github.com/refcell/carp
cd carp
just build-cli
```

The binary will be available at `cli/target/release/carp`.

### From Crates.io (Coming Soon)

```bash
cargo install carp-cli
```

## Usage

### Search for Agents

```bash
# Basic search
carp search "text processing"

# Limit results
carp search "claude" --limit 5

# Exact match only
carp search "my-agent" --exact
```

### Pull an Agent

```bash
# Pull latest version
carp pull agent-name

# Pull specific version
carp pull agent-name@1.2.0

# Pull to specific directory
carp pull agent-name --output ./my-agents/

# Force overwrite existing directory
carp pull agent-name --force
```

### Create a New Agent

```bash
# Create basic agent template
carp new my-agent

# Create with specific template
carp new my-agent --template python

# Create in specific directory
carp new my-agent --path ./projects/my-agent

# Available templates: basic, advanced, python
```

### Publish an Agent

```bash
# Publish from current directory (requires Carp.toml)
carp publish

# Publish with specific manifest
carp publish --manifest ./path/to/Carp.toml

# Dry run (validate without publishing)
carp publish --dry-run

# Skip confirmation prompts
carp publish --yes
```

## Configuration

Configuration is stored in `~/.config/carp/config.toml`:

```toml
registry_url = "https://api.carp.refcell.org"
api_token = "your-api-token"
timeout = 30
verify_ssl = true
default_output_dir = "/path/to/agents"
```

## Agent Manifest (Carp.toml)

```toml
name = "my-agent"
version = "1.0.0"
description = "A Claude AI agent that does amazing things"
author = "Your Name <your.email@example.com>"
license = "MIT"
homepage = "https://github.com/username/my-agent"
repository = "https://github.com/username/my-agent"
tags = ["claude", "ai", "automation"]

files = [
    "README.md",
    "agent.py",
    "config.toml",
    "src/"
]

main = "agent.py"

[dependencies]
# Optional: dependencies on other agents
# other-agent = "1.0.0"
```

## Development

### Building

```bash
just build-cli
```

### Testing

```bash
just test-cli
```

### Linting

```bash
just lint-cli
```

### Formatting

```bash
just fmt-cli
```

## Security

The Carp CLI includes several security features:

- **Secure Config Storage**: API tokens stored with restricted file permissions (600)
- **Path Traversal Protection**: ZIP extraction validates paths to prevent directory traversal
- **HTTPS by Default**: All network requests use HTTPS
- **URL Validation**: Registry URLs are validated for format and security
- **Input Validation**: All user inputs are validated and sanitized

## Architecture

The CLI is built with a modular architecture:

```
src/
├── main.rs              # CLI entry point and argument parsing
├── lib.rs              # Library exports
├── commands/           # Command implementations
│   ├── search.rs       # Agent search functionality
│   ├── pull.rs         # Agent download and extraction
│   ├── publish.rs      # Agent packaging and upload
│   └── new.rs          # Template generation
├── config/             # Configuration management
├── api/                # HTTP client for registry API
├── auth/               # Authentication handling
└── utils/              # Shared utilities and error handling
```

## Error Handling

The CLI provides comprehensive error handling with user-friendly messages:

- Network connectivity issues
- Authentication failures
- File system errors
- Manifest validation errors
- API rate limiting and server errors

## Contributing

This CLI tool follows Rust best practices:

- MSRV: 1.82
- No warnings policy (clippy warnings treated as errors)
- Comprehensive error handling
- Security-first design
- Full test coverage (coming soon)

## License

MIT License - see [LICENSE](../LICENSE) for details.