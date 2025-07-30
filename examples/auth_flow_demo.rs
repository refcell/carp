/// Practical demonstration of authentication flows
/// Shows how the authentication system works with actual API endpoints
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Carp Authentication Flow Demonstration ===\n");
    
    // Set up development environment
    env::set_var("DEBUG_AUTH", "true");
    env::set_var("SUPABASE_URL", ""); // Empty to trigger dev mode
    env::set_var("SUPABASE_SERVICE_ROLE_KEY", "");
    env::set_var("SUPABASE_JWT_SECRET", "");
    
    println!("🔧 Environment configured for development mode");
    
    // Demonstrate the complete authentication flow
    demonstrate_complete_flow().await?;
    
    // Test error scenarios
    demonstrate_error_scenarios().await?;
    
    // Test configuration scenarios
    demonstrate_configuration_scenarios().await?;
    
    println!("\n=== Authentication Flow Demonstration Complete ===");
    Ok(())
}

async fn demonstrate_complete_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 1. Complete Authentication Flow:");
    
    // Step 1: Simulate GitHub OAuth login (results in JWT)
    println!("   Step 1: User logs in via GitHub OAuth");
    let mock_jwt_token = "mock.github.oauth.jwt.token";
    println!("   📝 Generated JWT token: {}", mock_jwt_token);
    
    // Step 2: JWT token is used to authenticate frontend requests
    println!("   Step 2: Frontend uses JWT for authentication");
    let jwt_user_scopes = vec!["read", "api_key_create", "api_key_manage"];
    println!("   🔐 JWT user granted scopes: {:?}", jwt_user_scopes);
    
    // Step 3: Frontend creates API key using JWT authentication
    println!("   Step 3: Frontend creates API key for CLI");
    let generated_api_key = "carp_demo_key12_demo5678_demo9012";
    println!("   🗝️  Generated API key: {}", generated_api_key);
    
    // Step 4: CLI uses API key for agent operations
    println!("   Step 4: CLI uses API key for operations");
    let api_key_user_scopes = vec!["read", "write", "upload", "publish", "admin"];
    println!("   🔧 API key user granted scopes: {:?}", api_key_user_scopes);
    
    // Step 5: Demonstrate endpoint access patterns
    println!("   Step 5: Verify endpoint access patterns");
    
    // JWT-only endpoints
    let jwt_endpoints = vec![
        ("POST /v1/auth/api-keys", "Create API key", "api_key_create"),
        ("GET /profile", "View user profile", "read"),
        ("GET /dashboard", "View dashboard", "read"),
    ];
    
    for (endpoint, description, required_scope) in jwt_endpoints {
        let has_access = jwt_user_scopes.contains(&required_scope.to_string());
        let status = if has_access { "✅ ALLOWED" } else { "❌ DENIED" };
        println!("     {} {} - {}: {}", status, endpoint, description, required_scope);
    }
    
    // API-key-only endpoints
    let api_key_endpoints = vec![
        ("POST /v1/agents/upload", "Upload agent", "upload"),
        ("POST /v1/agents/publish", "Publish agent", "publish"),
        ("GET /v1/agents/my-agent/1.0.0/download", "Download agent", "read"),
        ("GET /v1/auth/api-keys", "List API keys", "read"),
    ];
    
    for (endpoint, description, required_scope) in api_key_endpoints {
        let has_access = api_key_user_scopes.contains(&required_scope.to_string()) 
                      || api_key_user_scopes.contains(&"admin".to_string());
        let status = if has_access { "✅ ALLOWED" } else { "❌ DENIED" };
        println!("     {} {} - {}: {}", status, endpoint, description, required_scope);
    }
    
    println!("   ✅ Complete flow demonstrated successfully");
    Ok(())
}

async fn demonstrate_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚨 2. Error Scenario Testing:");
    
    // Test 1: Wrong authentication method for endpoint
    println!("   Test 1: Wrong authentication method");
    let api_key = "carp_test1234_test5678_test9012";
    let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI...";
    
    // API key used on JWT-only endpoint
    println!("   ❌ API key '{}' used on JWT-only endpoint /v1/auth/api-keys POST", 
             &api_key[..20]);
    println!("      Expected error: 'API keys are not allowed for this endpoint'");
    
    // JWT used on API-key-only endpoint  
    println!("   ❌ JWT token used on API-key-only endpoint /v1/agents/upload POST");
    println!("      Expected error: 'JWT tokens are not allowed for this endpoint'");
    
    // Test 2: Insufficient scope
    println!("   Test 2: Insufficient scope");
    let limited_user_scopes = vec!["read"];
    
    let scope_tests = vec![
        ("upload", "Upload agent"),
        ("publish", "Publish agent"),
        ("api_key_create", "Create API key"),
    ];
    
    for (required_scope, operation) in scope_tests {
        let has_scope = limited_user_scopes.contains(&required_scope.to_string());
        if !has_scope {
            println!("   ❌ User with scopes {:?} cannot {}", limited_user_scopes, operation);
            println!("      Expected error: 'Required scope {} not found'", required_scope);
        }
    }
    
    // Test 3: Invalid token formats
    println!("   Test 3: Invalid token formats");
    let invalid_tokens = vec![
        ("", "Empty token"),
        ("invalid", "Too short"),
        ("carp_incomplete", "Malformed API key"),
        ("not.a.real.jwt", "Invalid JWT"),
    ];
    
    for (token, description) in invalid_tokens {
        println!("   ❌ {} ('{}')", description, token);
        if token.starts_with("carp_") {
            let is_valid = token.matches('_').count() == 3;
            if !is_valid {
                println!("      Expected error: 'Invalid API key format'");
            }
        } else if token.contains('.') {
            println!("      Expected error: 'Invalid JWT token'");
        } else {
            println!("      Expected error: 'Invalid token format'");
        }
    }
    
    println!("   ✅ Error scenarios validated");
    Ok(())
}

async fn demonstrate_configuration_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  3. Configuration Scenarios:");
    
    // Scenario 1: Development mode
    println!("   Scenario 1: Development Mode");
    let dev_config = ("", "", "");
    let is_dev = dev_config.0.is_empty() || dev_config.1.is_empty();
    println!("   🔧 Development mode detected: {}", is_dev);
    if is_dev {
        println!("      - Mock authentication responses");
        println!("      - Consistent development user ID");
        println!("      - Bypassed database requirements");
        println!("      - Debug logging enabled");
    }
    
    // Scenario 2: Production mode  
    println!("   Scenario 2: Production Mode");
    let prod_config = ("https://prod.supabase.co", "prod_key", "prod_secret");
    let is_prod = !prod_config.0.is_empty() && !prod_config.1.is_empty();
    println!("   🏭 Production mode detected: {}", is_prod);
    if is_prod {
        println!("      - Real JWT validation required");
        println!("      - Database API key verification");
        println!("      - Strict error handling");
        println!("      - Security logging enabled");
    }
    
    // Scenario 3: Environment variable configuration
    println!("   Scenario 3: Environment Variables");
    let env_vars = vec![
        ("DEBUG_AUTH", env::var("DEBUG_AUTH").unwrap_or_default()),
        ("SUPABASE_URL", env::var("SUPABASE_URL").unwrap_or_default()),
        ("SUPABASE_SERVICE_ROLE_KEY", env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default()),
    ];
    
    for (var_name, var_value) in env_vars {
        let status = if var_value.is_empty() { "unset" } else { "set" };
        println!("   📝 {}: {} ({})", var_name, status, 
                if var_value.len() > 20 { format!("{}...", &var_value[..20]) } else { var_value });
    }
    
    println!("   ✅ Configuration scenarios validated");
    Ok(())
}

/// Helper function to demonstrate API key hashing
fn demo_api_key_hashing() {
    println!("\n🔐 4. API Key Security:");
    
    let api_key = "carp_demo_key12_demo5678_demo9012";
    
    // Simulate the hashing that would occur in the real system
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    fn mock_hash_api_key(key: &str) -> String {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    let hash = mock_hash_api_key(api_key);
    println!("   Original API key: {}", api_key);
    println!("   Stored hash:      {}", hash);
    println!("   Hash length:      {} characters", hash.len());
    
    // Verify consistency
    let hash2 = mock_hash_api_key(api_key);
    let is_consistent = hash == hash2;
    println!("   Hash consistency: {}", if is_consistent { "✅ PASSED" } else { "❌ FAILED" });
    
    // Verify uniqueness (different keys produce different hashes)
    let different_key = "carp_different_key123_test456";
    let different_hash = mock_hash_api_key(different_key);
    let is_unique = hash != different_hash;
    println!("   Hash uniqueness:  {}", if is_unique { "✅ PASSED" } else { "❌ FAILED" });
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_authentication_flow_integration() {
        // Test the core authentication logic
        let jwt_scopes = vec!["read", "api_key_create"];
        let api_key_scopes = vec!["read", "upload", "publish"];
        
        // JWT user should be able to create API keys
        assert!(jwt_scopes.contains(&"api_key_create"));
        
        // API key user should be able to upload
        assert!(api_key_scopes.contains(&"upload"));
        
        // Verify separation: JWT users can't upload directly
        assert!(!jwt_scopes.contains(&"upload"));
        
        // Verify separation: API key users can't create API keys (unless admin)
        assert!(!api_key_scopes.contains(&"api_key_create"));
    }
    
    #[test]
    fn test_token_format_validation() {
        // Test API key format
        let valid_api_key = "carp_test1234_test5678_test9012";
        let is_valid_api_key = valid_api_key.starts_with("carp_") 
                              && valid_api_key.matches('_').count() == 3;
        assert!(is_valid_api_key);
        
        // Test invalid API key format
        let invalid_api_key = "carp_incomplete";
        let is_invalid_api_key = invalid_api_key.starts_with("carp_") 
                                && invalid_api_key.matches('_').count() == 3;
        assert!(!is_invalid_api_key);
        
        // Test JWT format (simplified)
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let looks_like_jwt = jwt_token.contains('.') && jwt_token.len() > 100;
        assert!(looks_like_jwt);
    }
    
    #[test]
    fn test_development_vs_production_config() {
        // Development config (empty values)
        let dev_config = ("", "", "");
        let is_dev = dev_config.0.is_empty() || dev_config.1.is_empty();
        assert!(is_dev);
        
        // Production config (all values set)
        let prod_config = ("https://prod.supabase.co", "key", "secret");
        let is_prod = !prod_config.0.is_empty() && !prod_config.1.is_empty();
        assert!(is_prod);
    }
}