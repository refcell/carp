# Carp 🐟

**Claude Agent Registry Portal** - A modern registry for Claude AI agents, similar to crates.io for Rust packages.

[![Website](https://img.shields.io/badge/Website-carp.refcell.org-blue)](https://carp.refcell.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> ⚠️ **Under Active Development**: This project is under active development and may break. APIs and features are subject to change.

## Overview

Carp is an open-source registry that allows developers to publish, discover, and pull Claude AI agents. The platform consists of a modern web interface, backend API, and a Rust CLI tool for seamless agent management.

## Features

- 🔍 **Agent Discovery**: Browse and search through a curated collection of Claude agents
- 📦 **Package Management**: Use the `carp` CLI tool to pull agents from the registry
- 🚀 **Publishing**: Easily publish your own agents to share with the community
- 🌐 **Modern Web Interface**: Clean, responsive design with greenish-blue theme
- 🔐 **Authentication**: Secure user accounts and agent publishing

## CLI Tool

The `carp` command-line tool provides a seamless experience for working with Claude agents:

### Installation

```bash
# Install from crates.io (coming soon)
cargo install carp-cli

# Or build from source
git clone https://github.com/refcell/carp
cd carp
cargo build --release
```

### Usage

```bash
# Search for agents in the registry
carp search <query>

# Pull an agent from the registry
carp pull <agent-name>

# Publish your agent to the registry
carp publish

# Create a new agent template
carp new <agent-name>
```

## Technology Stack

- **Frontend**: React + TypeScript with Tailwind CSS
- **Backend**: Rust with modern web frameworks (planned)
- **CLI**: Rust with Clap for command parsing
- **Database**: PostgreSQL with Supabase
- **Deployment**: Modern cloud infrastructure

## Development

This project follows Rust conventions and includes comprehensive tooling:

```bash
# Build the project
just build

# Run tests
just test

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
