use crate::utils::{ApiError, Config};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
// use tower_governor::{
//     governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
// };

// /// Create rate limiting middleware (commented out for compatibility)
// pub fn create_rate_limiter(config: &Config) -> GovernorLayer<SmartIpKeyExtractor> {
//     let governor_conf = Box::new(
//         GovernorConfigBuilder::default()
//             .per_minute(config.rate_limit.requests_per_minute)
//             .burst_size(config.rate_limit.burst_size)
//             .finish()
//             .unwrap(),
//     );

//     GovernorLayer {
//         config: Arc::new(governor_conf),
//     }
// }

/// Validate content type for file uploads
pub async fn validate_content_type(
    State(_config): State<Arc<Config>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Only validate content type for POST/PUT requests to upload endpoints
    let method = request.method();
    let path = request.uri().path();

    if (method == "POST" || method == "PUT") && path.contains("/publish") {
        if let Some(content_type) = headers.get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            
            // Check if it's multipart/form-data (for file uploads)
            if content_type_str.starts_with("multipart/form-data") {
                // Allow multipart uploads
                return Ok(next.run(request).await);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Validate request size
pub async fn validate_request_size(
    State(config): State<Arc<Config>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Check content-length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                if length > config.upload.max_file_size {
                    return Err(ApiError::payload_too_large());
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// CORS middleware configuration
pub fn cors_layer(config: &Config) -> tower_http::cors::CorsLayer {
    use tower_http::cors::CorsLayer;

    CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_origin(
            config
                .server
                .cors_origins
                .iter()
                .map(|origin| origin.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_credentials(true)
}

/// Request ID middleware
pub fn request_id_layer() -> tower_http::request_id::SetRequestIdLayer<tower_http::request_id::MakeRequestUuid> {
    tower_http::request_id::SetRequestIdLayer::x_request_id(tower_http::request_id::MakeRequestUuid)
}

/// Tracing middleware
pub fn trace_layer() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO))
}

/// Health check handler
pub async fn health_check() -> Result<Response<Body>, StatusCode> {
    let health_response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(health_response.to_string()))
        .unwrap())
}