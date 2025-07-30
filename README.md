# Carp 🐟

**Claude Agent Registry Portal** - A modern registry for Claude AI agents, similar to crates.io for Rust packages.

[![Website](https://img.shields.io/badge/Website-carp.refcell.org-blue)](https://carp.refcell.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> ⚠️ **Under Active Development**: This project is under active development and may break. APIs and features are subject to change.

## Overview

Carp is an open-source registry for discovering, pulling, and publishing Claude AI agents. The platform includes a web interface, serverless REST API, and Rust CLI tool for agent management.

## Features

- 🔍 **Agent Discovery**: Browse and search through Claude agents
- 📦 **CLI Management**: Pull agents from the registry with the `carp` CLI
- ⬆️ **Agent Upload**: Upload agents to share with the community
- 🌐 **Web Interface**: Modern React-based frontend
- 🔐 **API Authentication**: Secure API key-based authentication

## CLI Tool

### Installation

```bash
# Install from crates.io
cargo install carp-cli

# Or build from source
git clone https://github.com/refcell/carp
cd carp/cli
cargo build --release
```

### Usage

```bash
# Check API health
carp healthcheck

# List all available agents
carp list

# Search for agents
carp search <query>

# Pull an agent (interactive selection if no name provided)
carp pull [agent-name[@version]]

# Upload agents from directory
carp upload --directory ~/.claude/agents/

# Authentication
carp auth set-api-key
carp auth status
carp auth logout
```

## Technology Stack

- **Frontend**: React + TypeScript with Tailwind CSS
- **Backend**: Serverless Rust API on Vercel
- **CLI**: Rust with Clap for command parsing
- **Database**: PostgreSQL with Supabase
- **Authentication**: API key-based with secure storage

## Development

This project follows Rust conventions with comprehensive tooling:

```bash
# Build workspace
just build

# Run tests
just tests

# Lint and format
just lint
just fmt
```

## Contributing

We welcome contributions! This is an open-source project under the MIT license. Feel free to:

- Report bugs and request features
- Submit pull requests
- Improve documentation
- Share your agents with the community

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Visit [carp.refcell.org](https://carp.refcell.org) to explore the registry and discover amazing Claude agents!
