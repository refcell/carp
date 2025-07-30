use crate::auth::{
    authenticate_api_key, authenticate_jwt, extract_bearer_token, guess_token_type, sync_jwt_user,
    ApiError, AuthConfig, AuthenticatedUser, TokenType,
};
use serde_json::json;
use vercel_runtime::{Body, Request, Response};

/// Authentication strategy for different endpoints
#[derive(Debug, Clone, Copy)]
pub enum AuthStrategy {
    /// Only allow JWT tokens (for frontend endpoints like API key creation)
    JwtOnly,
    /// Only allow API keys (for CLI/API endpoints)
    ApiKeyOnly,
    /// Allow both JWT and API keys (deprecated - avoid using)
    #[deprecated(note = "Use specific authentication strategies instead")]
    Flexible,
}

/// Authenticate a request based on the specified strategy
pub async fn authenticate_request(
    req: &Request,
    strategy: AuthStrategy,
) -> Result<AuthenticatedUser, Response<Body>> {
    let config = AuthConfig::from_env();

    let token = extract_bearer_token(req).ok_or_else(|| {
        create_auth_error(
            401,
            &ApiError {
                error: "missing_authentication".to_string(),
                message: match strategy {
                    AuthStrategy::JwtOnly => {
                        "JWT authentication required. Please login through the web interface."
                            .to_string()
                    }
                    AuthStrategy::ApiKeyOnly => {
                        "API key authentication required. Create an API key through the web interface or use an existing one.".to_string()
                    }
                    #[allow(deprecated)]
                    AuthStrategy::Flexible => {
                        "Authentication required: provide either a valid API key or JWT token"
                            .to_string()
                    }
                },
                details: Some(json!({
                    "strategy": format!("{:?}", strategy),
                    "accepted_methods": match strategy {
                        AuthStrategy::JwtOnly => vec!["jwt_token"],
                        AuthStrategy::ApiKeyOnly => vec!["api_key"],
                        #[allow(deprecated)]
                        AuthStrategy::Flexible => vec!["jwt_token", "api_key"],
                    },
                    "header_formats": match strategy {
                        AuthStrategy::JwtOnly => vec!["Authorization: Bearer <jwt_token>"],
                        AuthStrategy::ApiKeyOnly => vec![
                            "Authorization: Bearer <api_key>",
                            "X-API-Key: <api_key>"
                        ],
                        #[allow(deprecated)]
                        AuthStrategy::Flexible => vec![
                            "Authorization: Bearer <jwt_token>",
                            "Authorization: Bearer <api_key>",
                            "X-API-Key: <api_key>"
                        ],
                    }
                })),
            },
        )
    })?;

    // Authenticate based on strategy
    let user = match strategy {
        AuthStrategy::JwtOnly => authenticate_jwt_only(&token, &config).await?,
        AuthStrategy::ApiKeyOnly => authenticate_api_key_only(&token, &config).await?,
        #[allow(deprecated)]
        AuthStrategy::Flexible => authenticate_flexible(&token, &config).await?,
    };

    // For JWT authentication, ensure user is synced in database
    if matches!(user.auth_method, crate::auth::AuthMethod::JwtToken { .. }) {
        if let Err(sync_error) = sync_jwt_user(&user, &config).await {
            if config.debug_mode {
                eprintln!("DEBUG: User sync failed (non-fatal): {:?}", sync_error);
            }
            // Don't fail authentication for sync errors, just log them
        }
    }

    Ok(user)
}

/// Authenticate using JWT only
async fn authenticate_jwt_only(
    token: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, Response<Body>> {
    // Reject obvious API keys
    if guess_token_type(token) == TokenType::ApiKey {
        return Err(create_auth_error(
            401,
            &ApiError {
                error: "invalid_auth_method".to_string(),
                message: "API keys are not allowed for this endpoint. Please use JWT authentication through the web interface.".to_string(),
                details: Some(json!({
                    "received_token_type": "api_key",
                    "expected_token_type": "jwt_token",
                    "help": "Login through the web interface to get a valid JWT token"
                })),
            },
        ));
    }

    authenticate_jwt(token, config)
        .await
        .map_err(|e| create_auth_error(401, &e))
}

/// Authenticate using API key only
async fn authenticate_api_key_only(
    token: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, Response<Body>> {
    // Reject obvious JWTs
    if guess_token_type(token) == TokenType::Jwt {
        return Err(create_auth_error(
            401,
            &ApiError {
                error: "invalid_auth_method".to_string(),
                message: "JWT tokens are not allowed for this endpoint. Please use API key authentication.".to_string(),
                details: Some(json!({
                    "received_token_type": "jwt_token",
                    "expected_token_type": "api_key",
                    "help": "Create an API key through the web interface at /profile"
                })),
            },
        ));
    }

    authenticate_api_key(token, config)
        .await
        .map_err(|e| create_auth_error(401, &e))
}

/// Flexible authentication (deprecated)
#[allow(deprecated)]
async fn authenticate_flexible(
    token: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, Response<Body>> {
    // Try to determine token type and authenticate accordingly
    match guess_token_type(token) {
        TokenType::ApiKey => authenticate_api_key(token, config)
            .await
            .map_err(|e| create_auth_error(401, &e)),
        TokenType::Jwt => authenticate_jwt(token, config)
            .await
            .map_err(|e| create_auth_error(401, &e)),
    }
}

/// Check if user has required scope, returning error response if not
pub fn require_scope(
    user: &AuthenticatedUser,
    required_scope: &str,
) -> Result<(), Response<Body>> {
    if !crate::auth::check_scope(user, required_scope) {
        return Err(create_auth_error(
            403,
            &ApiError {
                error: "insufficient_scope".to_string(),
                message: format!("Required scope '{}' not found in user permissions", required_scope),
                details: Some(json!({
                    "required_scope": required_scope,
                    "user_scopes": user.scopes,
                    "auth_method": format!("{:?}", user.auth_method)
                })),
            },
        ));
    }
    Ok(())
}

/// Create a standardized authentication error response
fn create_auth_error(status: u16, error: &ApiError) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("WWW-Authenticate", "Bearer")
        .body(
            serde_json::to_string(error)
                .unwrap_or_else(|_| r#"{"error":"serialization_error","message":"Failed to serialize error response"}"#.to_string())
                .into(),
        )
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body("Internal server error".into())
                .unwrap()
        })
}

/// Middleware for endpoints that require JWT authentication (frontend operations)
pub async fn jwt_middleware(req: &Request) -> Result<AuthenticatedUser, Response<Body>> {
    authenticate_request(req, AuthStrategy::JwtOnly).await
}

/// Middleware for endpoints that require API key authentication (CLI/API operations)
pub async fn api_key_middleware(req: &Request) -> Result<AuthenticatedUser, Response<Body>> {
    authenticate_request(req, AuthStrategy::ApiKeyOnly).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use vercel_runtime::Body;

    fn create_mock_request_with_auth(auth_value: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), format!("Bearer {}", auth_value));
        
        // This is a simplified mock - in real tests you'd use proper HTTP request builders
        // For now, we'll just test the core logic
        todo!("Implement proper request mocking for tests")
    }

    #[tokio::test]
    async fn test_jwt_only_rejects_api_key() {
        // Test that JWT-only authentication rejects API keys
        let api_key = "carp_test1234_test5678_test9012";
        let config = AuthConfig::from_env();
        
        let result = authenticate_jwt_only(api_key, &config).await;
        assert!(result.is_err());
        
        if let Err(response) = result {
            assert_eq!(response.status(), 401);
        }
    }

    #[tokio::test]
    async fn test_api_key_only_rejects_jwt() {
        // Test that API key-only authentication rejects JWTs
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let config = AuthConfig::from_env();
        
        let result = authenticate_api_key_only(jwt, &config).await;
        assert!(result.is_err());
        
        if let Err(response) = result {
            assert_eq!(response.status(), 401);
        }
    }
}