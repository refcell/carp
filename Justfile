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

# Build the CLI tool (alias)
b: build-cli

# Build CLI in native mode
build-native: build-cli

# Run CLI tests
test-cli:
  @echo "Running CLI tests..."
  @cd cli && cargo nextest run

# Run all tests (including CLI)
t: test-cli tests

# Run all tests
tests: test-cli

# Lint the CLI code
lint-cli:
  @echo "Linting CLI code..."
  @cd cli && cargo clippy -- -D warnings

# Lint all code (alias)
l: lint-cli

# Lint in native mode
lint-native: lint-cli

# Format CLI code
fmt-cli:
  @echo "Formatting CLI code..."
  @cd cli && cargo +nightly fmt

# Format all code (alias)
f: fmt-cli

# Format and fix in native mode
fmt-native-fix: fmt-cli

# Install the CLI tool locally
install-cli:
  @echo "Installing Carp CLI locally..."
  @cd cli && cargo install --path .

# Clean CLI build artifacts
clean-cli:
  @echo "Cleaning CLI build artifacts..."
  @cd cli && cargo clean

# Run CLI documentation tests
test-docs-cli:
  @echo "Testing CLI documentation..."
  @cd cli && cargo test --doc

# Test all documentation
test-docs: test-docs-cli

# Check CLI for issues (combines lint, test, and build)
check-cli: lint-cli test-cli build-cli

# Check everything
check: check-cli

# Prepare CLI for release
release-cli: lint-cli test-cli
  @echo "Building CLI for release..."
  @cd cli && cargo build --release
  @echo "CLI built successfully at cli/target/release/carp"

# Show CLI version info
version-cli:
  @cd cli && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="carp-cli") | "carp-cli \(.version)"'
