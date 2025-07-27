use crate::{
    db::Database,
    models::{ApiTokenValidation, UserProfile},
    utils::{ApiError, ApiResult, Config},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

/// Authenticated user context
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub scopes: Vec<String>,
}

/// Authentication service
pub struct AuthService {
    db: Database,
    config: Arc<Config>,
    argon2: Argon2<'static>,
}

impl AuthService {
    pub fn new(db: Database, config: Arc<Config>) -> Self {
        Self {
            db,
            config,
            argon2: Argon2::default(),
        }
    }

    /// Authenticate user with username/password and return JWT token
    pub async fn authenticate_user(&self, username: &str, password: &str) -> ApiResult<(String, DateTime<Utc>)> {
        // Query user from Supabase auth
        let user_query = self.db
            .client()
            .from("profiles")
            .select("user_id,username,password_hash")
            .eq("username", username)
            .single()
            .execute()
            .await?;

        if user_query.status() != 200 {
            return Err(ApiError::authentication_error("Invalid credentials"));
        }

        let user_data: serde_json::Value = user_query.json().await?;
        let user_id_str = user_data["user_id"]
            .as_str()
            .ok_or_else(|| ApiError::authentication_error("Invalid user data"))?;
        let user_id = Uuid::parse_str(user_id_str)?;
        
        let stored_hash = user_data["password_hash"]
            .as_str()
            .ok_or_else(|| ApiError::authentication_error("Invalid credentials"))?;

        // Verify password
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|_| ApiError::authentication_error("Invalid credentials"))?;
        
        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::authentication_error("Invalid credentials"))?;

        // Generate JWT token
        let expires_at = Utc::now() + Duration::hours(self.config.jwt.expiration_hours as i64);
        let claims = Claims {
            sub: user_id.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            iss: "carp-api".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt.secret.as_ref()),
        ).map_err(|_| ApiError::internal_error("Failed to generate token"))?;

        Ok((token, expires_at))
    }

    /// Validate JWT token and return user information
    pub fn validate_jwt_token(&self, token: &str) -> ApiResult<Uuid> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt.secret.as_ref()),
            &validation,
        ).map_err(|_| ApiError::authentication_error("Invalid token"))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        Ok(user_id)
    }

    /// Validate API token and return user information
    pub async fn validate_api_token(&self, token: &str) -> ApiResult<ApiTokenValidation> {
        // Hash the token for database lookup
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        // Call the database function to validate the token
        let validation_query = self.db
            .rpc_with_params("validate_api_token", serde_json::json!({
                "token_hash": token_hash
            }))
            .execute()
            .await?;

        if validation_query.status() != 200 {
            return Err(ApiError::authentication_error("Invalid API token"));
        }

        let validation_result: Vec<ApiTokenValidation> = validation_query.json().await?;
        validation_result
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::authentication_error("Invalid API token"))
    }

    /// Get user profile by user ID
    pub async fn get_user_profile(&self, user_id: Uuid) -> ApiResult<UserProfile> {
        let profile_query = self.db
            .client()
            .from("profiles")
            .select("*")
            .eq("user_id", user_id.to_string())
            .single()
            .execute()
            .await?;

        if profile_query.status() != 200 {
            return Err(ApiError::not_found_error("User profile not found"));
        }

        let profile: UserProfile = profile_query.json().await?;
        Ok(profile)
    }

    /// Hash password for storage
    pub fn hash_password(&self, password: &str) -> ApiResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| ApiError::internal_error("Failed to hash password"))?;
        Ok(password_hash.to_string())
    }
}

/// Extract authentication from request headers
pub fn extract_auth_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })
}

/// Authentication middleware
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    
    if let Some(token) = extract_auth_token(headers) {
        // Try JWT token first
        if let Ok(user_id) = auth_service.validate_jwt_token(&token) {
            let auth_user = AuthUser {
                user_id,
                scopes: vec!["read".to_string(), "write".to_string()],
            };
            request.extensions_mut().insert(auth_user);
        } 
        // Try API token
        else if let Ok(validation) = auth_service.validate_api_token(&token).await {
            let auth_user = AuthUser {
                user_id: validation.user_id,
                scopes: validation.scopes,
            };
            request.extensions_mut().insert(auth_user);
        }
    }

    Ok(next.run(request).await)
}

/// Required authentication middleware (returns 401 if no valid auth)
pub async fn require_auth(
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if let Some(auth_user) = request.extensions().get::<AuthUser>().cloned() {
        // Insert as Extension for handlers to use
        request.extensions_mut().insert(auth_user);
        Ok(next.run(request).await)
    } else {
        Err(ApiError::authentication_error("Authentication required"))
    }
}

/// Extract authenticated user from request extensions
pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}