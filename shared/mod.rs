//! Shared authentication and middleware for Carp API
//!
//! This module provides centralized authentication logic that can be used
//! across all Vercel serverless functions in the API.
//!
//! ## Architecture
//!
//! The authentication system supports two distinct methods:
//!
//! 1. **JWT Authentication**: For frontend/web UI operations
//!    - Uses Supabase JWT tokens from GitHub OAuth
//!    - Required for API key creation and management
//!    - Provides scopes: `read`, `api_key_create`, `api_key_manage`
//!
//! 2. **API Key Authentication**: For CLI/programmatic access
//!    - Uses API keys created through the web interface
//!    - Required for agent upload, publish, and other API operations
//!    - Provides scopes based on key configuration: `read`, `write`, `upload`, `publish`, etc.
//!
//! ## Usage
//!
//! ### For JWT-only endpoints (API key management):
//!
//! ```rust
//! use shared::middleware::jwt_middleware;
//!
//! pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
//!     let user = jwt_middleware(&req).await?;
//!     // ... rest of handler
//! }
//! ```
//!
//! ### For API key-only endpoints (agent operations):
//!
//! ```rust
//! use shared::middleware::{api_key_middleware, require_scope};
//!
//! pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
//!     let user = api_key_middleware(&req).await?;
//!     require_scope(&user, "upload")?;
//!     // ... rest of handler
//! }
//! ```

pub mod auth;
pub mod middleware;

// Re-export commonly used types and functions
pub use auth::{
    authenticate_api_key, authenticate_jwt, check_scope, extract_bearer_token, guess_token_type,
    hash_api_key, validate_jwt_token, ApiError, AuthConfig, AuthMethod, AuthenticatedUser,
    SupabaseJwtClaims, TokenType, UserMetadata,
};

pub use middleware::{api_key_middleware, jwt_middleware, require_scope, AuthStrategy};
