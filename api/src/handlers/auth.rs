use crate::{
    models::{AuthRequest, AuthResponse, UserProfile},
    utils::{ApiError, ApiResult},
};
use axum::{
    extract::State,
    Extension, Json,
};
use validator::Validate;

/// Handle user login
pub async fn login(
    State(state): State<crate::AppState>,
    Json(request): Json<AuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let auth_service = &state.auth_service;
    // Validate input
    request.validate()
        .map_err(|e| ApiError::validation_error(format!("Invalid request: {}", e)))?;

    // Authenticate user
    let (token, expires_at) = auth_service
        .authenticate_user(&request.username, &request.password)
        .await?;

    Ok(Json(AuthResponse {
        token,
        expires_at,
    }))
}

/// Get current user profile
pub async fn me(
    State(state): State<crate::AppState>,
    Extension(auth_user): Extension<crate::auth::AuthUser>,
) -> ApiResult<Json<UserProfile>> {
    let auth_service = &state.auth_service;

    let profile = auth_service
        .get_user_profile(auth_user.user_id)
        .await?;

    Ok(Json(profile))
}