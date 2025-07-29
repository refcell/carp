# CLI Production Integration - Phase 2 Implementation Documentation

## Overview

This document provides comprehensive information about the Phase 2 CLI production integration implementation for the QA engineer who will create tests in Phase 3. The implementation has successfully transformed the CLI from a mock-based system to a production-ready tool with enhanced security, error handling, and configuration management.

## Implementation Summary

### Major Changes Completed

1. **Mock Dependencies Removal**: All mock dependencies have been removed from production code paths while maintaining mock usage only in test environments.

2. **Enhanced API Client**: The API client now includes:
   - Production-grade error handling with proper error categorization
   - Retry logic with exponential backoff
   - Timeout handling and connection management
   - Input validation and sanitization
   - Security checks for URLs and downloads

3. **Enhanced Configuration System**: The configuration system now supports:
   - Environment variable overrides
   - Secure credential management
   - CI/CD environment detection
   - Production security settings
   - Comprehensive validation

4. **Updated Commands**: All commands have been updated for production API usage:
   - Search command with enhanced validation
   - Pull command with secure download handling
   - Healthcheck command with proper error reporting
   - Publish command (disabled pending security hardening)

5. **Integration Test Framework**: Comprehensive integration tests for real API testing.

## Key Files and Architecture

### Core API Client (`/Users/andreasbigger/carp/cli/src/api/client.rs`)

**Key Features:**
- `RetryConfig` struct for configurable retry behavior
- `ApiClient` with production-grade HTTP client configuration
- Comprehensive input validation for all API methods
- Security enforcement (HTTPS-only downloads)
- Proper error handling with detailed error types

**Important Methods:**
- `new()` - Creates client with default configuration
- `with_retry_config()` - Creates client with custom retry settings
- `search()` - Search agents with validation and retry logic
- `get_agent_download()` - Get download info with URL encoding
- `download_agent()` - Secure agent download with size limits
- `authenticate()` - User authentication (no retries for security)
- `health_check()` - API health check with minimal retry

### Enhanced Configuration (`/Users/andreasbigger/carp/cli/src/config/settings.rs`)

**Key Features:**
- `Config` struct with production settings
- `RetrySettings` for configurable retry behavior
- `SecuritySettings` for security constraints
- Environment variable override support
- Secure credential management methods
- CI/CD environment detection

**Important Methods:**
- `load_with_env_checks()` - Load config with environment validation
- `apply_env_overrides()` - Apply environment variable overrides
- `validate_config()` - Comprehensive configuration validation
- `set_api_token_secure()` - Secure token management
- `export_template()` - Generate deployment templates

### Integration Tests (`/Users/andreasbigger/carp/cli/tests/integration_tests.rs`)

**Test Categories:**
- Health check tests
- Search functionality tests
- Input validation tests
- Authentication tests
- Concurrent request tests
- Retry mechanism tests
- Performance benchmarks
- Error handling tests
- Security feature tests

## Environment Variables

The CLI now supports the following environment variables for configuration:

| Variable | Description | Default |
|----------|-------------|---------|
| `CARP_REGISTRY_URL` | API base URL | `https://api.carp.refcell.org` |
| `CARP_API_TOKEN` | Authentication token | None |
| `CARP_TIMEOUT` | Request timeout in seconds | `30` |
| `CARP_VERIFY_SSL` | SSL certificate verification | `true` |
| `CARP_OUTPUT_DIR` | Default output directory | None |
| `CARP_ALLOW_HTTP` | Allow HTTP URLs (insecure) | `false` |

### Test-Specific Environment Variables

| Variable | Description |
|----------|-------------|
| `CARP_TEST_API_URL` | Test API URL |
| `CARP_TEST_TOKEN` | Test authentication token |
| `CARP_SKIP_AUTH_TESTS` | Skip authentication tests |

## Security Features

### Input Validation
- Agent names: alphanumeric + hyphens/underscores, max 100 chars
- Versions: semantic version format, max 50 chars
- URLs: proper format validation and HTTPS enforcement
- File sizes: configurable limits (100MB download, 50MB upload)

### Security Settings
- HTTPS-only enforcement (configurable)
- SSL certificate verification
- Download size limits
- Secure file permissions on config files (600)
- Protection against directory traversal attacks
- JWT token format validation

### Error Handling
- Categorized error types: `CarpError` enum
- Proper error propagation with context
- Sanitized error messages (no sensitive data exposure)
- Graceful degradation for network issues

## Testing Strategy for QA Engineer

### Unit Tests (Already Implemented)
- Configuration loading and validation
- Agent manifest parsing
- Error handling edge cases
- Input validation functions

### Integration Tests (Framework Provided)
- Real API endpoint testing
- Authentication flow testing
- Command-line interface testing
- Error scenario testing
- Performance benchmarking

### Test Environments

#### Local Testing
```bash
# Run unit tests
cargo test --lib

# Run integration tests (requires network)
CARP_SKIP_AUTH_TESTS=1 cargo test --test integration_tests

# Build and test CLI
cargo build --release
./target/release/carp healthcheck --verbose
```

#### CI/CD Testing
```bash
# Set environment variables
export CARP_REGISTRY_URL="https://staging-api.carp.refcell.org"
export CARP_VERIFY_SSL="true"
export CARP_SKIP_AUTH_TESTS="1"

# Run tests
cargo test --all --verbose
```

### Recommended Test Cases for Phase 3

#### Command Testing
1. **Health Check Command**
   - Basic health check functionality
   - Verbose output validation
   - Error handling for API unavailability
   - Response format validation

2. **Search Command**
   - Basic search functionality
   - Pagination and limits
   - Exact match vs fuzzy search
   - Input validation (empty queries, special characters)
   - Large result sets handling

3. **Pull Command**
   - Agent download functionality
   - Version specification (latest vs specific)
   - Output directory handling
   - Force overwrite functionality
   - Checksum verification
   - Archive extraction (ZIP, tar.gz)
   - Security validation (path traversal protection)

4. **Publish Command**
   - Currently disabled - test error handling
   - Dry run functionality
   - Manifest validation
   - File packaging

#### Configuration Testing
1. **Environment Variable Overrides**
   - Test all supported environment variables
   - Validation of invalid values
   - Security settings enforcement

2. **Configuration File Handling**
   - Default config creation
   - Config file validation
   - Secure file permissions
   - Template generation

#### Security Testing
1. **Input Validation**
   - Malformed agent names and versions
   - SQL injection attempts
   - Path traversal attempts
   - Large input handling

2. **Network Security**
   - HTTPS enforcement
   - SSL certificate validation
   - URL validation
   - Download size limits

3. **Authentication**
   - Token format validation
   - Token expiry handling
   - Invalid credentials handling

#### Error Handling Testing
1. **Network Errors**
   - Connection timeouts
   - DNS resolution failures
   - HTTP error codes (4xx, 5xx)
   - Retry mechanism validation

2. **API Errors**
   - Malformed responses
   - Missing data fields
   - Rate limiting responses
   - Server errors

#### Performance Testing
1. **Response Times**
   - Health check performance
   - Search query performance
   - Download speed validation

2. **Concurrent Operations**
   - Multiple simultaneous requests
   - Resource usage monitoring
   - Memory leak detection

#### Platform Testing
1. **Cross-Platform Compatibility**
   - macOS, Linux, Windows
   - Different shell environments
   - Path handling across platforms

2. **Dependency Management**
   - Missing dependencies handling
   - Version compatibility

## Known Limitations and Considerations

### Current Limitations
1. **Publishing Disabled**: Publishing is currently disabled pending backend security hardening
2. **Mock Server Limitations**: Some unit tests use mock servers with limited functionality
3. **Download Size Limits**: Hardcoded in some places due to API client design limitations

### Security Considerations
1. **Token Storage**: API tokens are stored in plaintext config files with restricted permissions
2. **HTTPS Enforcement**: Can be disabled for development but not recommended for production
3. **Input Validation**: Additional validation may be needed based on API specification updates

### Performance Considerations
1. **Retry Logic**: May increase response times for failing requests
2. **Concurrent Downloads**: Limited to configured maximum (default: 4)
3. **Memory Usage**: Large downloads are loaded into memory

## Troubleshooting Guide

### Common Issues

#### Connection Errors
```
Error: HTTP error: error trying to connect: dns error: failed to lookup address information
```
**Solution**: Check network connectivity and DNS resolution

#### Authentication Errors
```
Error: Authentication error: No API token configured. Please login first.
```
**Solution**: Set `CARP_API_TOKEN` environment variable or authenticate via CLI

#### Configuration Errors
```
Error: Configuration error: Registry URL must use HTTPS for security
```
**Solution**: Use HTTPS URL or set `allow_http=true` in config

#### SSL Certificate Errors
```
Error: HTTP error: error trying to connect: invalid peer certificate: UnknownIssuer
```
**Solution**: Set `CARP_VERIFY_SSL=false` or fix certificate issues

### Debug Commands

```bash
# Enable verbose logging
./target/release/carp healthcheck --verbose

# Check configuration
CARP_REGISTRY_URL=https://api.example.com ./target/release/carp healthcheck

# Test with different timeout
CARP_TIMEOUT=60 ./target/release/carp search test
```

## Files Modified/Created

### Modified Files
- `/Users/andreasbigger/carp/cli/src/api/client.rs` - Enhanced API client
- `/Users/andreasbigger/carp/cli/src/config/settings.rs` - Enhanced configuration
- `/Users/andreasbigger/carp/cli/src/config/mod.rs` - Module exports
- `/Users/andreasbigger/carp/cli/src/commands/search.rs` - Updated for production
- `/Users/andreasbigger/carp/cli/src/commands/pull.rs` - Updated for production
- `/Users/andreasbigger/carp/cli/src/commands/healthcheck.rs` - Updated for production
- `/Users/andreasbigger/carp/cli/src/commands/publish.rs` - Updated with security notice
- `/Users/andreasbigger/carp/cli/Cargo.toml` - Added dependencies

### Created Files
- `/Users/andreasbigger/carp/cli/tests/integration_tests.rs` - Integration test framework

## Next Steps for QA Engineer

1. **Review Implementation**: Study the code changes and understand the architecture
2. **Set Up Test Environment**: Configure test environment with appropriate environment variables
3. **Run Existing Tests**: Execute unit and integration tests to establish baseline
4. **Design Test Plan**: Create comprehensive test plan based on recommendations above
5. **Implement Additional Tests**: Add missing test coverage areas
6. **Performance Testing**: Establish performance benchmarks
7. **Security Testing**: Conduct security validation tests
8. **Documentation**: Update test documentation and procedures

## Build and Deployment

### Build Commands
```bash
# Build CLI
just build-native

# Run tests
just tests

# Format code
just fmt-native-fix

# Lint code
just lint-native
```

### Dependencies Added
- `urlencoding = "2.1"` - URL encoding for API parameters
- `futures = "0.3"` (dev) - For integration tests

The implementation is now ready for comprehensive QA testing and provides a solid foundation for production use with proper security, error handling, and configuration management.