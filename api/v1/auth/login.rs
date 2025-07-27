use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
// use std::collections::HashMap; // Not used in this file
// use std::env; // Not used in this file
use vercel_runtime::{run, Body, Error, Request, Response};

/// Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Parse request body
    let body = req.body();
    let auth_request: AuthRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(_) => {
            let error = ApiError {
                error: "bad_request".to_string(),
                message: "Invalid JSON in request body".to_string(),
                details: None,
            };
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())?);
        }
    };

    // For now, implement basic authentication (in production, use proper auth)
    let valid_credentials = authenticate_user(&auth_request.username, &auth_request.password).await;

    if !valid_credentials {
        let error = ApiError {
            error: "unauthorized".to_string(),
            message: "Invalid username or password".to_string(),
            details: None,
        };
        return Ok(Response::builder()
            .status(401)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&error)?.into())?);
    }

    // Generate JWT token (simplified for now)
    let token = generate_jwt_token(&auth_request.username)?;
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    let response = AuthResponse { token, expires_at };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&response)?.into())?)
}

async fn authenticate_user(username: &str, password: &str) -> bool {
    // In production, this would check against Supabase
    // For now, accept any non-empty credentials
    !username.is_empty() && !password.is_empty()
}

fn generate_jwt_token(username: &str) -> Result<String, Error> {
    // Simplified JWT generation - in production use proper JWT library
    let token_data = json!({
        "username": username,
        "exp": (Utc::now() + chrono::Duration::hours(24)).timestamp()
    });

    // For now, return a simple base64 encoded token
    Ok(format!("jwt_{}", base64::encode(token_data.to_string())))
}

// Base64 encoding helper (simplified)
mod base64 {
    pub fn encode(input: String) -> String {
        // Simplified base64 encoding
        input.chars().map(|c| ((c as u8) + 1) as char).collect()
    }
}
