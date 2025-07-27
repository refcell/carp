# Carp API Backend

A Rust-based REST API backend for the Carp agent registry, built with Axum and integrated with Supabase.

## Features

- **Agent Management**: Publish, search, and download agents
- **Authentication**: JWT and API token-based authentication
- **File Storage**: Secure file uploads via Supabase Storage
- **Rate Limiting**: Built-in request rate limiting
- **CORS Support**: Configurable cross-origin resource sharing
- **Comprehensive Logging**: Structured logging with tracing
- **Input Validation**: Request validation and sanitization

## API Endpoints

### Authentication

- `POST /api/v1/auth/login` - Authenticate with username/password
- `GET /api/v1/auth/me` - Get current user profile (requires auth)

### Agents

- `GET /api/v1/agents/search` - Search for agents with filtering and pagination
- `GET /api/v1/agents/{name}/{version}/download` - Get download information for an agent
- `POST /api/v1/agents/publish` - Publish a new agent or version (requires auth)

### Health

- `GET /health` - Health check endpoint

## Environment Variables

Create a `.env` file in the `/api` directory with the following variables:

```env
# Server Configuration
HOST=0.0.0.0
PORT=3001
CORS_ORIGINS=http://localhost:5173,https://carp.refcell.org

# Supabase Configuration (required)
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
SUPABASE_JWT_SECRET=your-jwt-secret

# JWT Configuration
JWT_SECRET=your-jwt-secret-key
JWT_EXPIRATION_HOURS=24

# Upload Configuration
MAX_FILE_SIZE=104857600  # 100MB in bytes

# Rate Limiting
RATE_LIMIT_RPM=60        # Requests per minute
RATE_LIMIT_BURST=10      # Burst size

# Logging
RUST_LOG=info            # debug, info, warn, error
```

## Development Setup

1. **Prerequisites**:
   - Rust 1.82+ installed
   - Access to a Supabase project with the schema installed
   - Environment variables configured

2. **Install dependencies**:
   ```bash
   cd api && cargo build
   ```

3. **Run the development server**:
   ```bash
   just dev-api
   # or
   cd api && RUST_LOG=debug cargo run
   ```

4. **Run tests**:
   ```bash
   just test-api
   # or
   cd api && cargo test
   ```

## Build Commands

Using the project's Justfile:

```bash
# Build the API
just build-api

# Run tests
just test-api

# Lint code
just lint-api

# Format code
just fmt-api

# Check everything (lint + test + build)
just check-api

# Run development server
just dev-api
```

## API Usage Examples

### Authentication

```bash
# Login
curl -X POST http://localhost:3001/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "your-username", "password": "your-password"}'

# Response
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_at": "2024-01-15T12:00:00Z"
}
```

### Search Agents

```bash
# Basic search
curl "http://localhost:3001/api/v1/agents/search?q=web&limit=10"

# Search with filters
curl "http://localhost:3001/api/v1/agents/search?q=web&tags=automation,scraping&author=johndoe&sort=downloads"

# Response
{
  "agents": [
    {
      "name": "web-scraper",
      "version": "1.0.0",
      "description": "A web scraping agent",
      "author": "johndoe",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z",
      "download_count": 150,
      "tags": ["web", "scraping", "automation"]
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 10
}
```

### Download Agent

```bash
# Get download information
curl "http://localhost:3001/api/v1/agents/web-scraper/1.0.0/download"

# Response
{
  "name": "web-scraper",
  "version": "1.0.0",
  "download_url": "https://supabase-storage-url/agent-packages/...",
  "checksum": "sha256-hash",
  "size": 1024000
}
```

### Publish Agent

```bash
# Publish an agent (requires authentication)
curl -X POST http://localhost:3001/api/v1/agents/publish \
  -H "Authorization: Bearer your-jwt-token" \
  -F 'metadata={"name":"my-agent","version":"1.0.0","description":"My awesome agent","tags":["automation"]}' \
  -F 'content=@agent.zip'

# Response
{
  "success": true,
  "message": "Agent published successfully",
  "agent": {
    "name": "my-agent",
    "version": "1.0.0",
    "description": "My awesome agent",
    "author": "your-username",
    "created_at": "2024-01-15T12:00:00Z",
    "updated_at": "2024-01-15T12:00:00Z",
    "download_count": 0,
    "tags": ["automation"]
  }
}
```

## Architecture

### Components

- **Handlers**: HTTP request handlers for each endpoint
- **Auth**: Authentication and authorization logic
- **Database**: Supabase integration and query builders
- **Models**: Data structures and validation
- **Middleware**: Cross-cutting concerns (CORS, auth, logging)
- **Utils**: Configuration, error handling, and utilities

### Security Features

- JWT-based authentication
- API token validation via database
- Request size limits
- Input validation and sanitization
- CORS configuration
- Rate limiting (configurable)
- Secure file upload handling

### Database Integration

The API integrates with a Supabase PostgreSQL database using:

- **PostgREST**: For direct SQL queries
- **Row Level Security (RLS)**: Database-level access control
- **Database Functions**: For complex operations
- **Storage API**: For file upload/download

## Error Handling

The API returns structured error responses:

```json
{
  "error": "ValidationError",
  "message": "Invalid request: field 'name' is required",
  "details": {
    "field": "name",
    "code": "required"
  }
}
```

Common error types:
- `ValidationError` (400) - Invalid input data
- `AuthenticationError` (401) - Missing or invalid authentication
- `AuthorizationError` (403) - Insufficient permissions
- `NotFoundError` (404) - Resource not found
- `ConflictError` (409) - Resource already exists
- `RateLimitError` (429) - Too many requests
- `PayloadTooLarge` (413) - File too large
- `InternalError` (500) - Server error

## Deployment

### Production Build

```bash
# Build release binary
cargo build --release

# Binary will be at target/release/carp-api
```

### Docker (example)

```dockerfile
FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/carp-api /usr/local/bin/carp-api
EXPOSE 3001
CMD ["carp-api"]
```

### Environment Setup

Ensure all required environment variables are set in production:
- Database credentials (Supabase)
- JWT secrets
- CORS origins
- File upload limits
- Rate limiting configuration

## Performance Considerations

- Uses connection pooling for database access
- Implements request rate limiting
- Efficient file streaming for uploads/downloads
- Structured logging for observability
- Built on async Rust for high concurrency

## Contributing

1. Follow the existing code style (use `just fmt-api`)
2. Add tests for new functionality
3. Ensure all checks pass (`just check-api`)
4. Update documentation as needed

## License

See the main project LICENSE file.