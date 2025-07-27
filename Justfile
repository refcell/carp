set positional-arguments

# default recipe to display help information
default:
  @just --list

# Spin up the dev server for the site.
dev:
  @echo "Starting development server..."
  # Install bun if not already installed
  @if ! command -v bun &> /dev/null; then \
    echo "Bun is not installed. Installing..."; \
    curl -fsSL https://bun.sh/install | bash; \
  else \
    echo "Bun is already installed."; \
  fi
  @cd site && bun install && bun run dev

# Build the CLI tool
build-cli:
  @echo "Building Carp CLI..."
  @cd cli && cargo build --release

# Build the API backend
build-api:
  @echo "Building Carp API..."
  @cd api && cargo build --release

# Build everything
build-all: build-cli build-api

# Build the CLI tool (alias)
b: build-all

# Build CLI in native mode
build-native: build-all

# Run CLI tests
test-cli:
  @echo "Running CLI tests..."
  @cd cli && cargo nextest run

# Run API tests
test-api:
  @echo "Running API tests..."
  @cd api && cargo nextest run

# Run all tests (including CLI and API)
t: test-cli test-api tests

# Run all tests
tests: test-cli test-api

# Lint the CLI code
lint-cli:
  @echo "Linting CLI code..."
  @cd cli && cargo clippy -- -D warnings

# Lint the API code
lint-api:
  @echo "Linting API code..."
  @cd api && cargo clippy -- -D warnings

# Lint all code
lint-all: lint-cli lint-api

# Lint all code (alias)
l: lint-all

# Lint in native mode
lint-native: lint-all

# Format CLI code
fmt-cli:
  @echo "Formatting CLI code..."
  @cd cli && cargo +nightly fmt

# Format API code
fmt-api:
  @echo "Formatting API code..."
  @cd api && cargo +nightly fmt

# Format all code
fmt-all: fmt-cli fmt-api

# Format all code (alias)
f: fmt-all

# Format and fix in native mode
fmt-native-fix: fmt-all

# Install the CLI tool locally
install-cli:
  @echo "Installing Carp CLI locally..."
  @cd cli && cargo install --path .

# Clean CLI build artifacts
clean-cli:
  @echo "Cleaning CLI build artifacts..."
  @cd cli && cargo clean

# Clean API build artifacts
clean-api:
  @echo "Cleaning API build artifacts..."
  @cd api && cargo clean

# Clean all build artifacts
clean-all: clean-cli clean-api

# Run CLI documentation tests
test-docs-cli:
  @echo "Testing CLI documentation..."
  @cd cli && cargo test --doc

# Test all documentation
test-docs: test-docs-cli

# Check CLI for issues (combines lint, test, and build)
check-cli: lint-cli test-cli build-cli

# Check API for issues (combines lint, test, and build)
check-api: lint-api test-api build-api

# Check everything
check: check-cli check-api

# Prepare CLI for release
release-cli: lint-cli test-cli
  @echo "Building CLI for release..."
  @cd cli && cargo build --release
  @echo "CLI built successfully at cli/target/release/carp"

# Show CLI version info
version-cli:
  @cd cli && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="carp-cli") | "carp-cli \(.version)"'

# Run the API server in development mode
dev-api:
  @echo "Starting API server in development mode..."
  @cd api && RUST_LOG=debug cargo run

# Install the API server locally
install-api:
  @echo "Installing Carp API locally..."
  @cd api && cargo install --path .

# Show API version info
version-api:
  @cd api && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="carp-api") | "carp-api \(.version)"'

# Install Supabase CLI if not present and apply migrations
migrate-db:
  @echo "Checking for Supabase CLI..."
  @if ! command -v supabase &> /dev/null; then \
    echo "Supabase CLI is not installed. Installing via Homebrew..."; \
    brew install supabase/tap/supabase; \
  else \
    echo "Supabase CLI is already installed."; \
  fi
  @echo "Applying database migrations..."
  @cd site/supabase && supabase db push --linked
