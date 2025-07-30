# API Test Scripts

Simple bash scripts to test the serverless API endpoints using curl.

## Setup

1. Update the `API_BASE_URL` variable in each script to point to your production API
2. Set environment variables for authentication tokens and parameters

## Environment Variables

### Authentication
- `JWT_TOKEN` - JWT token for authenticated endpoints
- `API_KEY` - API key for API key authenticated endpoints
- `USERNAME` - Username for login
- `PASSWORD` - Password for login

### Parameters
- `KEY_ID` - UUID of API key for update/delete operations
- `AGENT_FILE` - Path to agent file for upload (default: agent.tar.gz)
- `SEARCH_QUERY` - Search query for agent search (default: test)
- `AGENT_NAME` - Agent name for download (default: test-agent)
- `AGENT_VERSION` - Agent version for download (default: 1.0.0)

## Usage

### Basic Health Check
```bash
./test_health.sh
```

### Authentication Flow
```bash
# Login to get JWT token
USERNAME=myuser PASSWORD=mypass ./test_login.sh

# Use the JWT token from login response
JWT_TOKEN=eyJ... ./test_list_api_keys.sh
JWT_TOKEN=eyJ... ./test_create_api_key.sh
```

### API Key Management
```bash
# Update an API key
API_KEY=ak_... KEY_ID=uuid-here ./test_update_api_key.sh

# Delete an API key
API_KEY=ak_... KEY_ID=uuid-here ./test_delete_api_key.sh
```

### Agent Operations
```bash
# Upload an agent
API_KEY=ak_... AGENT_FILE=./my-agent.tar.gz ./test_upload_agent.sh

# Publish an agent
API_KEY=ak_... ./test_publish_agent.sh

# Search for agents
SEARCH_QUERY=myquery ./test_search_agents.sh

# Download a specific agent
AGENT_NAME=my-agent AGENT_VERSION=2.0.0 ./test_download_agent.sh
```

## Scripts

- `test_health.sh` - Test health check endpoint
- `test_login.sh` - Test user login
- `test_list_api_keys.sh` - List user's API keys (requires JWT)
- `test_create_api_key.sh` - Create new API key (requires JWT)
- `test_update_api_key.sh` - Update API key (requires API key)
- `test_delete_api_key.sh` - Delete API key (requires API key)
- `test_upload_agent.sh` - Upload agent (requires API key)
- `test_publish_agent.sh` - Publish agent (requires API key)
- `test_search_agents.sh` - Search agents (public)
- `test_download_agent.sh` - Download agent (public)

## Notes

- All scripts include basic HTTP status code checking
- Scripts exit with code 1 on failure, 0 on success
- Response bodies are displayed for debugging
- Update the `API_BASE_URL` variable in each script before use