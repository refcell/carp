# Carp CLI Test Suite Documentation

## Overview

This directory contains comprehensive test suites for the Carp CLI, covering security, performance, API contracts, regression prevention, and end-to-end functionality.

## Test Suites

### Unit Tests (`src/`)
Located within the source code modules, these test individual components in isolation.

```bash
# Run all unit tests
cargo test --lib

# Run unit tests with verbose output
cargo test --lib -- --nocapture
```

### Integration Tests (`integration_tests.rs`)
Test the API client integration with real or mock API endpoints.

```bash
# Run with authentication tests skipped
CARP_SKIP_AUTH_TESTS=1 cargo test --test integration_tests

# Run with test API URL
CARP_TEST_API_URL=https://staging.api.carp.refcell.org cargo test --test integration_tests
```

**Key Tests:**
- Health check functionality
- Search API integration
- Agent download info retrieval
- Input validation
- Authentication flows
- Concurrent request handling
- Retry mechanism validation
- Performance benchmarks
- Error handling scenarios

### Security Tests (`security_tests.rs`)
Comprehensive security validation including input sanitization, authentication security, and attack prevention.

```bash
# Run with network tests skipped
CARP_SKIP_NETWORK_TESTS=1 cargo test --test security_tests
```

**Key Tests:**
- Input validation (SQL injection, XSS, path traversal)
- Agent name and version validation
- URL validation and HTTPS enforcement
- Authentication bypass prevention
- Configuration security
- Concurrent request security
- Memory exhaustion protection
- Error message sanitization
- Timeout enforcement
- SSL/TLS validation

### Performance Tests (`performance_tests.rs`)
Load testing, response time validation, and resource usage monitoring.

```bash
# Run with load tests skipped
CARP_SKIP_LOAD_TESTS=1 cargo test --test performance_tests

# Run with full load testing (CI environments)
cargo test --test performance_tests -- --test-threads=1
```

**Key Tests:**
- Health check performance
- Search performance
- Concurrent request performance
- Sustained load testing
- Memory usage monitoring
- Retry mechanism performance
- Download performance simulation
- JSON parsing performance

**Performance Requirements:**
- 95th percentile response time < 500ms
- Support for 20+ concurrent requests
- No memory leaks in sustained operations
- Proper retry timing with exponential backoff

### Contract Tests (`contract_tests.rs`)
API schema validation and contract adherence testing.

```bash
# Run with API tests skipped (uses mock validation)
CARP_SKIP_API_TESTS=1 cargo test --test contract_tests
```

**Key Tests:**
- Health check response structure
- Search response schema validation
- Agent data structure validation
- Download info contract validation
- Authentication response structure
- Error response contracts
- API versioning validation
- Response time SLA compliance
- Data consistency validation

### Regression Tests (`regression_tests.rs`)
Prevention of previously identified bugs and edge cases.

```bash
# Run with full regression suite skipped
CARP_SKIP_REGRESSION_TESTS=1 cargo test --test regression_tests
```

**Key Tests:**
- Empty search query validation
- Whitespace query handling
- Zero/large search limits
- Invalid agent name patterns
- Timeout handling improvements
- Retry loop prevention
- Malformed JSON handling
- Concurrent request isolation
- Token handling edge cases
- URL encoding correctness
- Configuration validation
- Error message sanitization
- Memory leak prevention
- Client state consistency

### End-to-End Tests (`e2e_tests.rs`)
Full CLI workflow simulation using mock servers.

```bash
# Run E2E tests (requires compiled CLI binary)
cargo test --test e2e_tests
```

**Key Tests:**
- CLI search command functionality
- Pull command with version specification
- New agent creation
- Publish command validation
- Help and version output
- Verbose/quiet mode behavior
- Network error handling
- Command-line argument parsing

## Environment Configuration

### Required Environment Variables
```bash
# Test API endpoint (optional, defaults to production API)
export CARP_TEST_API_URL="https://api.carp.refcell.org"

# Test authentication token (optional)
export CARP_TEST_TOKEN="your-test-token"
```

### Test Control Flags
```bash
# Skip authentication-dependent tests
export CARP_SKIP_AUTH_TESTS=1

# Skip network-dependent tests
export CARP_SKIP_NETWORK_TESTS=1

# Skip intensive load tests
export CARP_SKIP_LOAD_TESTS=1

# Skip live API tests
export CARP_SKIP_API_TESTS=1

# Skip full regression test suite
export CARP_SKIP_REGRESSION_TESTS=1
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Comprehensive Testing

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: 1.82
        components: clippy, rustfmt
    
    - name: Build CLI
      run: cargo build --release
    
    - name: Unit Tests
      run: cargo test --lib
    
    - name: Integration Tests
      run: CARP_SKIP_AUTH_TESTS=1 cargo test --test integration_tests
      env:
        CARP_TEST_API_URL: ${{ secrets.TEST_API_URL }}
    
    - name: Security Tests
      run: CARP_SKIP_NETWORK_TESTS=1 cargo test --test security_tests
    
    - name: Performance Tests
      run: CARP_SKIP_LOAD_TESTS=1 cargo test --test performance_tests
    
    - name: Contract Tests
      run: CARP_SKIP_API_TESTS=1 cargo test --test contract_tests
    
    - name: Regression Tests
      run: CARP_SKIP_REGRESSION_TESTS=1 cargo test --test regression_tests
    
    - name: Code Quality
      run: |
        cargo clippy -- -D warnings
        cargo fmt -- --check
```

### Quality Gates
- **Unit Tests**: 100% pass rate required
- **Integration Tests**: 100% pass rate required
- **Security Tests**: 90%+ pass rate required
- **Performance Tests**: Meet SLA requirements (95th percentile < 500ms)
- **Contract Tests**: 100% schema compliance required
- **Code Quality**: Zero clippy warnings, proper formatting

## Test Data Management

### Mock Data
Tests use controlled mock data to ensure consistent, repeatable results:
- Mock API responses for offline testing
- Predefined test agents and search results
- Controlled error scenarios
- Network timeout simulations

### Test Cleanup
Tests are designed to be isolated and clean up after themselves:
- Temporary directories automatically removed
- No persistent state between tests
- Mock servers properly shut down
- Environment variables scoped to test runs

## Performance Benchmarking

### Metrics Collected
- Response time percentiles (50th, 95th, 99th)
- Success/failure rates
- Concurrent request handling
- Memory usage patterns
- Network timeout behavior
- Retry mechanism effectiveness

### Performance Thresholds
```rust
// Response time requirements
const MAX_HEALTH_CHECK_TIME: Duration = Duration::from_millis(500);
const MAX_SEARCH_TIME: Duration = Duration::from_secs(10);
const MAX_DOWNLOAD_INFO_TIME: Duration = Duration::from_secs(5);

// Throughput requirements
const MIN_CONCURRENT_REQUESTS: usize = 20;
const MIN_SUCCESS_RATE: f64 = 0.8;
```

## Security Testing

### Attack Vectors Tested
- SQL injection attempts
- Cross-site scripting (XSS)
- Path traversal attacks
- JNDI injection
- Authentication bypasses
- DoS via large inputs
- Memory exhaustion
- Information disclosure

### Security Validation
- Input sanitization effectiveness
- HTTPS enforcement
- Certificate validation
- Token handling security
- Error message sanitization
- Timeout protection
- Concurrent request limits

## Troubleshooting

### Common Issues

**Tests fail with network errors:**
```bash
# Skip network-dependent tests
CARP_SKIP_NETWORK_TESTS=1 cargo test
```

**Tests fail due to API unavailability:**
```bash
# Skip API-dependent tests
CARP_SKIP_API_TESTS=1 cargo test
```

**Performance tests are too strict:**
```bash
# Skip intensive performance tests
CARP_SKIP_LOAD_TESTS=1 cargo test --test performance_tests
```

**E2E tests fail:**
```bash
# Ensure CLI binary is built
cargo build --release

# Check binary location
ls -la target/release/carp
```

### Debug Mode
```bash
# Run tests with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

## Contributing

### Adding New Tests
1. Choose appropriate test suite based on test type
2. Follow existing naming conventions (`test_feature_description`)
3. Include proper error handling and cleanup
4. Add environment variable controls for CI/CD flexibility
5. Document test purpose and expected behavior

### Test Guidelines
- Tests should be deterministic and repeatable
- Use descriptive assertion messages
- Handle network failures gracefully
- Clean up resources properly
- Validate both success and failure scenarios
- Include performance benchmarks where appropriate

### Code Coverage
```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage
```

## Test Results Interpretation

### Success Criteria
- All unit tests pass (100%)
- Integration tests pass with API connectivity (90%+)
- Security tests validate input handling (90%+)
- Performance tests meet SLA requirements
- Contract tests confirm API compliance (100%)
- Regression tests prevent known issues (90%+)

### Expected Failures
Some tests may fail in certain environments:
- Network tests without internet connectivity
- API tests without valid endpoints
- Authentication tests without valid tokens
- Load tests in resource-constrained environments

These failures are expected and controlled via environment variables.

---

For more information, see the comprehensive QA report: `/Users/andreasbigger/carp/CLI_PHASE3_QA_REPORT.md`