mod auth;
mod db;
mod handlers;
mod middleware;
mod models;
mod state;
mod utils;

use auth::{auth_middleware, require_auth, AuthService};
use axum::{
    extract::DefaultBodyLimit,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use db::Database;
use handlers::{agents, auth as auth_handlers};
use middleware::{cors_layer, health_check, request_id_layer, trace_layer, validate_request_size};
use state::AppState;
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tower::ServiceBuilder;
use tracing::info;
use utils::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "carp_api=debug,tower_http=debug".into()),
        )
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    info!("Starting Carp API server with config: {:?}", config);

    // Initialize database connection
    let db = Database::new()?;
    info!("Connected to database");

    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new(db.clone(), config.clone()));

    // Build our application with routes
    let app = create_app(db, auth_service, config.clone()).await?;

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on {}", addr);

    // Run the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn create_app(
    db: Database, 
    auth_service: Arc<AuthService>,
    config: Arc<Config>
) -> anyhow::Result<Router> {
    let state = AppState {
        db,
        auth_service: auth_service.clone(),
        config: config.clone(),
    };
    // Create rate limiter (commented out due to compilation complexity)
    // let rate_limiter = middleware::create_rate_limiter(&config);

    // Create public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/agents/search", get(agents::search_agents))
        .route("/api/v1/agents/:name/:version/download", get(agents::get_agent_download))
        .route("/api/v1/auth/login", post(auth_handlers::login));

    // Create protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/v1/agents/publish", post(agents::publish_agent))
        .route("/api/v1/auth/me", get(auth_handlers::me))
        .layer(from_fn(require_auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            ServiceBuilder::new()
                // Request tracing
                .layer(trace_layer())
                // Request ID
                .layer(request_id_layer())
                // CORS
                .layer(cors_layer(&state.config))
                // Authentication middleware
                .layer(from_fn_with_state(
                    state.auth_service.clone(),
                    auth_middleware,
                ))
                // Request size validation
                .layer(from_fn_with_state(
                    state.config.clone(),
                    validate_request_size,
                ))
                // Body size limit
                .layer(DefaultBodyLimit::max(state.config.upload.max_file_size as usize))
                // Rate limiting (commented out)
                // .layer(rate_limiter)
        )
        .with_state(state);

    Ok(app)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down");
        }
        _ = terminate => {
            info!("Received terminate signal, shutting down");
        }
    }
}