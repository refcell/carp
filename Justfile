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
