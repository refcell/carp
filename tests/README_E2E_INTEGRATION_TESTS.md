# End-to-End Download Integration Tests

This directory contains comprehensive end-to-end integration tests for the Carp download pipeline. These tests verify that the complete flow from CLI command to database function works correctly and serve as regression tests for the recent database function signature fix.

## Overview

The download pipeline involves multiple components:

1. **CLI Client** (`carp pull` command)
2. **Registry API** (download endpoint)
3. **Database Functions** (`get_agent_download_info`, `record_download`)
4. **File Storage** (Supabase storage with signed URLs)

These tests ensure all components work together correctly and catch integration issues that unit tests might miss.

## Recent Fix Validation

The primary purpose of these tests is to validate the recent fix to the database function signature mismatch:

- **Problem**: The database function `get_agent_download_info` expected parameters named `p_agent_name` and `p_version_text`
- **Fix**: Updated the API code to send the correct parameter names
- **Validation**: These tests ensure the fix works and prevent regression

## Test Structure

### Core Test Files

- **`e2e_download_integration_tests.rs`** - Main E2E test suite covering the complete download flow
- **`interactive_download_tests.rs`** - Tests for interactive agent selection and search functionality
- **`api_download_tests.rs`** - Unit tests for API components (existing)
- **`test_runner.rs`** - Test runner utility for executing and reporting on all tests

### Test Coverage

#### Database Function Integration
- ✅ Correct parameter names (`p_agent_name`, `p_version_text`)
- ✅ Version resolution ("latest" → empty string parameter)
- ✅ Response parsing and error handling
- ✅ Download recording with proper parameters

#### CLI Integration
- ✅ Direct agent specification (`carp pull agent@version`)
- ✅ Latest version downloading (`carp pull agent`)
- ✅ Interactive agent selection mode
- ✅ Error handling for non-existent agents
- ✅ Complex agent names and version strings
- ✅ Force overwrite functionality

#### API Endpoint Integration
- ✅ Download endpoint parameter handling
- ✅ Signed URL generation
- ✅ File download and extraction
- ✅ Checksum verification
- ✅ Authentication handling (optional for downloads)

#### Error Scenarios
- ✅ Non-existent agents
- ✅ Non-existent versions
- ✅ Network timeouts and connection errors
- ✅ Server errors (5xx responses)
- ✅ Malformed responses

#### Search Integration
- ✅ Agent search functionality
- ✅ Large result sets
- ✅ Empty search results
- ✅ Search with filters and parameters

## Running the Tests

### Prerequisites

1. **Rust Environment**: Ensure you have Rust 1.82+ installed
2. **CLI Binary**: The tests require the `carp` CLI binary to be built
3. **Dependencies**: All test dependencies should be installed automatically

### Quick Start

```bash
# Run all E2E integration tests
cargo test --test e2e_download_integration_tests

# Run interactive download tests
cargo test --test interactive_download_tests

# Run all tests with verbose output
cargo test --test e2e_download_integration_tests -- --nocapture
```

### Using the Test Runner

The test runner provides a comprehensive way to run all tests and generate reports:

```bash
# Run all tests with the test runner
cargo run --bin test_runner --features test-runner

# Run with verbose output
cargo run --bin test_runner --features test-runner -- --verbose

# Set custom timeout (default: 300 seconds)
cargo run --bin test_runner --features test-runner -- --timeout 600

# Continue after first failure
cargo run --bin test_runner --features test-runner -- --no-fail-fast

# Show help
cargo run --bin test_runner --features test-runner -- --help
```

### Running Specific Tests

```bash
# Run a specific test function
cargo test test_complete_download_flow_specific_version --test e2e_download_integration_tests

# Run all tests matching a pattern
cargo test database_function --test e2e_download_integration_tests

# Run tests for version resolution
cargo test version_resolution --test e2e_download_integration_tests
```

## Test Architecture

### Mock Server Setup

The tests use `wiremock` to create mock servers that simulate:

1. **Registry API Server** - Mock endpoints for agent downloads and search
2. **Supabase Mock Server** - Mock database functions and storage endpoints

### Test Data

The tests create realistic test agents with:
- Valid ZIP file content (minimal but functional)
- Proper manifest files (`Carp.toml`)
- README and main executable files
- Realistic metadata (versions, descriptions, authors)
- Proper checksums for content verification

### Environment Isolation

Each test runs in isolation with:
- Temporary directories for downloads
- Separate configuration files
- Mock servers on different ports
- Clean environment variables

## Key Test Scenarios

### 1. Database Function Parameter Fix

```rust
#[tokio::test]
async fn test_database_function_parameter_names() {
    // Ensures the database function is called with:
    // - "p_agent_name" (not "agent_name")
    // - "p_version_text" (not "version")
}
```

### 2. Version Resolution

```rust
#[tokio::test]
async fn test_latest_version_parameter_conversion() {
    // Ensures "latest" version becomes empty string parameter
    // for the database function
}
```

### 3. Complete Download Flow

```rust
#[tokio::test]
async fn test_complete_download_flow_specific_version() {
    // Tests the entire pipeline:
    // CLI -> API -> Database -> Storage -> File Extraction
}
```

### 4. Error Handling

```rust
#[tokio::test]
async fn test_download_nonexistent_agent() {
    // Ensures proper error messages for missing agents
}
```

## Mock Server Configuration

The tests set up comprehensive mocks for:

### Database Functions
```json
{
  "p_agent_name": "test-agent",
  "p_version_text": "1.0.0"  // or "" for latest
}
```

### API Endpoints
```
GET /api/v1/agents/{name}/{version}/download
GET /api/v1/agents/search
POST /rest/v1/rpc/get_agent_download_info
POST /rest/v1/rpc/record_download
POST /storage/v1/object/sign/agent-packages/{path}
```

### File Downloads
- Proper ZIP file content with realistic structure
- Correct content-type headers
- Proper content-length headers
- Valid checksums for verification

## Debugging Test Failures

### Enable Debug Output

```bash
# Run with debug output
RUST_LOG=debug cargo test --test e2e_download_integration_tests -- --nocapture

# Use the test runner with verbose mode
cargo run --bin test_runner --features test-runner -- --verbose
```

### Check Mock Server Logs

The tests include debug output for:
- CLI command execution
- Mock server requests and responses
- File system operations
- Environment variables

### Common Issues

1. **CLI Binary Not Found**
   - Ensure `cargo build --bin carp` succeeds
   - Check that the binary exists in `target/debug/carp`

2. **Mock Server Port Conflicts**
   - Tests use random ports, but conflicts can still occur
   - Run tests sequentially: `cargo test -- --test-threads=1`

3. **Timeout Issues**
   - Increase timeout: `--timeout 600`
   - Check for deadlocks in mock server setup

4. **ZIP File Issues**
   - The tests create minimal valid ZIP files
   - Check that extraction is working correctly

## Test Reports

The test runner generates detailed reports:

```bash
# Run tests and generate report
cargo run --bin test_runner --features test-runner

# Report is saved as test_report.md
cat test_report.md
```

Report includes:
- Test summary (pass/fail counts)
- Execution time
- Coverage details
- Failure analysis
- Recommendations

## CI/CD Integration

For continuous integration, add these tests to your pipeline:

```yaml
# Example GitHub Actions workflow
- name: Run E2E Integration Tests
  run: |
    cargo build --bin carp
    cargo run --bin test_runner --features test-runner
```

## Contributing

When adding new tests:

1. **Follow the Pattern**: Use the existing test structure
2. **Add Mock Setup**: Ensure proper mock server configuration
3. **Test Both Success and Failure**: Cover positive and negative cases
4. **Update Documentation**: Update this README with new test scenarios
5. **Validate Regression Protection**: Ensure tests catch the specific issues they're designed for

### Adding New Test Scenarios

1. Create test function in appropriate module
2. Set up necessary mocks
3. Execute the scenario
4. Validate expected behavior
5. Add to test runner if needed

## Troubleshooting

### Environment Issues

```bash
# Validate environment
cargo run --bin test_runner --features test-runner -- --help

# Check CLI binary
cargo build --bin carp
./target/debug/carp --help
```

### Network Issues

```bash
# Check if ports are available
netstat -tlnp | grep :0  # Should show available random ports

# Run with specific timeout
cargo test --test e2e_download_integration_tests -- --timeout 120
```

### File System Issues

```bash
# Check temp directory permissions
ls -la /tmp/

# Clean up previous test artifacts
rm -rf /tmp/carp_test_*
```

## Performance Considerations

- Tests use mock servers to avoid network dependencies
- Temporary directories are cleaned up automatically
- ZIP files are minimal to reduce test execution time
- Concurrent test execution is supported but can be disabled

## Security Considerations

- Tests run in isolated environments
- No real API keys or credentials are used
- All network traffic goes to mock servers
- Temporary files are cleaned up properly
- No actual file uploads or downloads to external services

## Future Enhancements

- [ ] Add performance benchmarking tests
- [ ] Add tests for concurrent downloads
- [ ] Add tests for large file handling
- [ ] Add tests for network interruption recovery
- [ ] Add integration with real (staging) database
- [ ] Add visual test reports with charts

---

**Note**: These tests are critical for ensuring the stability of the download pipeline. They should be run before any deployment and whenever changes are made to the download-related code.
