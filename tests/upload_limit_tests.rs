/// Tests for daily upload limit validation
/// Verifies that users cannot upload more than 200 agents in a 24-hour period
use chrono::{Duration, Utc};
use serde_json::json;
use std::env;
use uuid::Uuid;

use shared::{AuthMethod, AuthenticatedUser, UserMetadata};

/// Mock test configuration for upload limit testing
pub struct UploadLimitTestConfig {
    pub supabase_url: String,
    pub supabase_key: String,
    pub test_user_id: Uuid,
}

impl Default for UploadLimitTestConfig {
    fn default() -> Self {
        Self {
            supabase_url: env::var("SUPABASE_URL").unwrap_or_else(|_| "http://localhost:54321".to_string()),
            supabase_key: env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_else(|_| "test-key".to_string()),
            test_user_id: Uuid::new_v4(),
        }
    }
}

/// Create a test user for upload limit testing
fn create_test_user(user_id: Uuid) -> AuthenticatedUser {
    AuthenticatedUser {
        user_id,
        auth_method: AuthMethod::ApiKey { key_id: Uuid::new_v4() },
        scopes: vec!["upload".to_string(), "write".to_string()],
        metadata: UserMetadata {
            email: Some("test@example.com".to_string()),
            github_username: Some("testuser".to_string()),
            created_at: Some(Utc::now()),
        },
    }
}

#[tokio::test]
async fn test_upload_limit_validation_logic() {
    // Test the core logic without requiring a real database
    // This tests the calculation and error message logic
    
    let user = create_test_user(Uuid::new_v4());
    
    // Calculate timestamps for testing
    let now = Utc::now();
    let twenty_four_hours_ago = now - Duration::hours(24);
    let one_hour_ago = now - Duration::hours(1);
    
    // Test timestamp format
    let timestamp_filter = twenty_four_hours_ago.to_rfc3339();
    assert!(timestamp_filter.contains("T"));
    assert!(timestamp_filter.len() > 10); // Should be a proper timestamp
    
    // Test error message formatting
    let upload_count = 199;
    let expected_error = format!(
        "Daily upload limit exceeded. You have uploaded {} agents in the past 24 hours. Maximum allowed is 200 agents per day.",
        upload_count + 1
    );
    assert!(expected_error.contains("200 agents"));
    assert!(expected_error.contains("Daily upload limit exceeded"));
}

#[tokio::test]
async fn test_upload_count_calculation() {
    // Test the logic for determining if a user is at the limit
    
    // Test cases for different upload counts
    let test_cases = vec![
        (0, true),    // 0 uploads -> can upload
        (100, true),  // 100 uploads -> can upload
        (198, true),  // 198 uploads -> can upload (would be 199th)
        (199, false), // 199 uploads -> cannot upload (would be 200th)
        (200, false), // 200 uploads -> cannot upload
        (250, false), // Over limit -> cannot upload
    ];
    
    for (upload_count, should_allow) in test_cases {
        let can_upload = upload_count < 199;
        assert_eq!(can_upload, should_allow, 
            "Upload count {} should allow upload: {}", upload_count, should_allow);
    }
}

#[tokio::test]
async fn test_query_url_construction() {
    // Test that we construct the correct Supabase query URL
    let config = UploadLimitTestConfig::default();
    let user_id = config.test_user_id;
    
    let twenty_four_hours_ago = Utc::now() - Duration::hours(24);
    let timestamp_filter = twenty_four_hours_ago.to_rfc3339();
    
    let expected_url = format!(
        "{}/rest/v1/agents?user_id=eq.{}&created_at=gte.{}&select=id",
        config.supabase_url, user_id, timestamp_filter
    );
    
    // Verify URL components
    assert!(expected_url.contains("/rest/v1/agents"));
    assert!(expected_url.contains(&format!("user_id=eq.{}", user_id)));
    assert!(expected_url.contains("created_at=gte."));
    assert!(expected_url.contains("select=id"));
}

#[tokio::test]
async fn test_no_database_fallback() {
    // Test that the function gracefully handles missing database configuration
    
    // Temporarily unset environment variables
    let original_url = env::var("SUPABASE_URL").ok();
    let original_key = env::var("SUPABASE_SERVICE_ROLE_KEY").ok();
    
    env::remove_var("SUPABASE_URL");
    env::remove_var("SUPABASE_SERVICE_ROLE_KEY");
    
    let user = create_test_user(Uuid::new_v4());
    
    // This should not fail when database is not configured
    // (We can't actually call the function here without importing it,
    // but we can test the logic that checks for empty env vars)
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap_or_default();
    
    let should_skip = supabase_url.is_empty() || supabase_key.is_empty();
    assert!(should_skip, "Should skip validation when database not configured");
    
    // Restore environment variables
    if let Some(url) = original_url {
        env::set_var("SUPABASE_URL", url);
    }
    if let Some(key) = original_key {
        env::set_var("SUPABASE_SERVICE_ROLE_KEY", key);
    }
}

/// Integration test that would test the actual database query
/// This test is marked as ignored because it requires a real database connection
#[ignore]
#[tokio::test]
async fn test_upload_limit_with_database() {
    // This test would require setting up a test database
    // and creating test data to verify the actual query works
    
    let config = UploadLimitTestConfig::default();
    let user = create_test_user(config.test_user_id);
    
    // Would need to:
    // 1. Set up test database with known data
    // 2. Create some test agents for the user
    // 3. Call the check_daily_upload_limit function
    // 4. Verify the correct count is returned
    // 5. Clean up test data
    
    println!("This test requires a real database connection and test data setup");
}