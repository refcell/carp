# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

- Build workspace: `just b` or `just build-native` 
- Lint: `just l` or `just lint-native`
- Format: `just f` or `just fmt-native-fix`
- Run all tests: `just t` or `just tests`
- Run specific test: `cargo nextest run --package [package-name] --test [test-name]`
- Run single test: `cargo nextest run --package [package-name] --test [test-name] -- [test_function_name]`
- Documentation: `just test-docs`

## Code Style

- MSRV: 1.82
- Format with nightly rustfmt: `cargo +nightly fmt`
- Imports: organized by crate, reordered automatically
- Error handling: use proper error types, prefer `Result<T, E>` over panics
- Naming: follow Rust conventions (snake_case for variables/functions, CamelCase for types)
- Prefer type-safe APIs and strong typing
- Documentation: rustdoc for public APIs, clear comments for complex logic
- Tests: write unit and integration tests for all functionality
- Performance: be mindful of allocations and copying, prefer references where appropriate
- No warnings policy: all clippy warnings are treated as errors (-D warnings)

## Project Architecture

Carp is a Claude Agent Registry Portal, similar to crates.io for Rust packages. The system consists of three main components:

### 1. CLI Tool (`cli/`)
- **Purpose**: Command-line interface for agent management (`carp` command)
- **Key operations**: search, pull, upload, authentication, health checks
- **Authentication**: API key-based (stored in `~/.config/carp/config.toml`)
- **Agent format**: Uses `Carp.toml` manifest files for agent metadata
- **Main modules**:
  - `commands/`: All CLI subcommands (healthcheck, list, pull, search, upload)
  - `api/`: HTTP client for registry API
  - `auth/`: API key management and authentication
  - `config/`: Configuration file handling

### 2. Serverless API (`api/`)
- **Purpose**: Vercel serverless functions providing REST API
- **Database**: PostgreSQL via Supabase
- **Authentication**: Dual system - JWT for web UI, API keys for CLI
- **Key endpoints**:
  - `v1/agents/search`: Public agent search
  - `v1/agents/[name]/[version]/download`: Agent download
  - `v1/agents/upload`: Authenticated agent upload
  - `v1/auth/api-keys`: API key management (JWT-only)
- **Shared code**: `shared/` module provides authentication middleware

### 3. Web Frontend (`site/`)
- **Purpose**: React-based web interface for browsing agents
- **Tech stack**: React + TypeScript + Tailwind CSS + Vite
- **Features**: Agent discovery, search, trending agents, user authentication
- **Database**: Uses Supabase client for PostgreSQL access
- **Authentication**: GitHub OAuth via Supabase

## Development Workflow

The project uses a monorepo structure with workspace dependencies:

- **CLI** (`cli/`): Standalone Rust binary, publishes to crates.io as `carp-cli`
- **API** (`./`): Root workspace with serverless functions as individual binaries
- **Site** (`site/`): Node.js project with separate package.json

Common development tasks:
- `just build-cli`: Build CLI tool only
- `just build-api`: Build API functions only
- `just dev`: Start frontend development server
- `just install-cli`: Install CLI locally for testing

## Agent Format

Agents use a `Carp.toml` manifest file with metadata:
```toml
name = "agent-name"
version = "1.0.0"
description = "Agent description"
author = "Author <email>"
license = "MIT"
tags = ["claude", "ai"]
files = ["README.md", "agent.py"]
main = "agent.py"
```

## Testing Strategy

- **Unit tests**: Within each module using `cargo test`
- **Integration tests**: In `tests/` directory using `cargo nextest`
- **E2E tests**: Full workflow tests including API and CLI interaction
- **Contract tests**: API endpoint validation
- **Performance tests**: Load testing for API endpoints

Use `just tests` to run the full test suite across all components.