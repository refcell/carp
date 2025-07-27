use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use std::collections::HashMap;

// Simple health check handler
async fn health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "carp-api",
        "environment": "serverless",
        "message": "API is being deployed - full functionality coming soon",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

// Placeholder search handler
async fn search() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "agents": [],
        "total": 0,
        "page": 1,
        "per_page": 20,
        "message": "Search functionality coming soon"
    })))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize simple logging
    println!("Starting Carp API Serverless Handler");

    // Create router with basic routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/agents/search", get(search));

    // For serverless, we don't start a server but export the router
    // Vercel will handle the HTTP layer
    
    // This is just a placeholder main function
    // The actual serverless handler will be called by Vercel runtime
    Ok(())
}

// Export handler for Vercel (this is the actual entry point)
pub async fn handler(
    req: http::Request<axum::body::Body>,
) -> Result<http::Response<axum::body::Body>, Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/agents/search", get(search));
    
    Ok(app.oneshot(req).await?)
}