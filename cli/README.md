# Carp CLI

Command-line tool for the Claude Agent Registry Portal (Carp).

## Features

- **Search Agents**: Find agents in the registry with powerful search functionality
- **List Agents**: Browse all available agents in the registry
- **Pull Agents**: Download and extract agents locally for use
- **Upload Agents**: Upload your agents to the registry
- **Health Checks**: Verify API connectivity and status
- **Authentication**: Manage API keys for registry access

## Installation

### From Source

```bash
git clone https://github.com/refcell/carp
cd carp
just build-cli
```

The binary will be available at `cli/target/release/carp`.

### From Crates.io

```bash
cargo install carp-cli
```

## Usage

### Health Check

```bash
# Check API connectivity and status
carp healthcheck
```

### Authentication

```bash
# Login with API key
carp auth login

# Check authentication status
carp auth status

# Logout (clear stored API key)
carp auth logout
```

You can also provide API keys via:
- Command line: `--api-key YOUR_KEY`
- Environment variable: `CARP_API_KEY=YOUR_KEY`
- Global flags work with all commands for authentication

### List All Agents

```bash
# List all available agents
carp list
```

### Search for Agents

```bash
# Basic search
carp search "text processing"

# Limit results  
carp search "claude" --limit 5

# Exact match only
carp search "my-agent" --exact

# Search with verbose output
carp search "claude" --verbose
```

### Pull an Agent

```bash
# Interactive selection (shows available agents)
carp pull

# Pull specific agent (latest version)
carp pull agent-name

# Pull specific version
carp pull agent-name@1.2.0

# Pull to specific directory
carp pull agent-name --output ./my-agents/

# Force overwrite existing directory
carp pull agent-name --force

# Pull with verbose output
carp pull agent-name --verbose
```

### Upload an Agent

```bash
# Upload from current directory (requires Carp.toml)
carp upload

# Upload from specific directory
carp upload --directory ./path/to/agent

# Upload with API key (if not configured)
carp upload --api-key YOUR_API_KEY

# Upload with verbose output
carp upload --verbose
```

## Configuration

Configuration is stored in `~/.config/carp/config.toml`:

```toml
api_key = "your-api-key"
```

### Authentication Methods

1. **Config file** (persistent): `~/.config/carp/config.toml`
2. **Environment variable**: `export CARP_API_KEY="your-api-key"`
3. **Command line flag**: `--api-key YOUR_API_KEY` (works with any command)

### Global Options

All commands support these global options:
- `--verbose`: Enable detailed output
- `--quiet`: Suppress all output except errors
- `--api-key`: Provide API key for authentication

## Agent Manifest (Carp.toml)

```toml
name = "my-agent"
version = "1.0.0"
description = "A Claude AI agent that does amazing things"
author = "Your Name <your.email@example.com>"
license = "MIT"
tags = ["claude", "ai", "automation"]
files = ["README.md", "agent.py"]
main = "agent.py"
```

## Development

### Building

```bash
# Build CLI only
just build-cli

# Or from CLI directory
cd cli && cargo build --release
```

### Testing

```bash
# Test CLI only  
just test-cli

# Or run specific tests
cargo nextest run --package carp-cli
```

### Linting & Formatting

```bash
# Lint (treats warnings as errors)
just lint-cli

# Format code
just fmt-cli

# Format with nightly rustfmt
cargo +nightly fmt
```

### All Checks

```bash
# Run full test suite
just tests

# Or workspace-wide checks
just build && just lint && just test
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
│   ├── healthcheck.rs  # API health check
│   ├── list.rs         # List all agents
│   ├── search.rs       # Agent search functionality
│   ├── pull.rs         # Agent download and extraction
│   └── upload.rs       # Agent upload functionality
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
- Agent manifest validation errors
- API rate limiting and server errors
- ZIP extraction and path traversal protection

## Contributing

This CLI tool follows Rust best practices:

- MSRV: 1.82
- No warnings policy (clippy warnings treated as errors)
- Comprehensive error handling with `anyhow`
- Security-first design with input validation
- Type-safe APIs and strong typing
- Performance-conscious implementation

## License

MIT License - see [LICENSE](../LICENSE) for details.
