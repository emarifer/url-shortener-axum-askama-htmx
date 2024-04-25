use std::sync::Arc;

use anyhow::Result;
use axum::{routing::get, Router};
use sqlx::{PgPool, Pool, Postgres};
use tokio::sync::RwLock;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handler;
mod model;
mod service;

/// This struct represents the application state,
/// holding a database connection pool
struct AppState {
    db: Pool<Postgres>,
}

/// This function serves as the entry point for running the Axum web server.
/// It takes a PostgreSQL connection pool (`PgPool`) as input,
/// sets up the application state,
/// creates the API routes using the provided application state,
/// binds the server to a specific port,
/// and starts serving incoming connections.
pub async fn serve(db: PgPool) -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shorturl_rs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router…");

    // Set up the application state with the provided database connection pool
    let app_state = AppState { db };

    // Create the API routes using the application state

    let app = api_router(Arc::new(RwLock::new(app_state)));
    let port = 8086_u16;

    // Bind the server to the specified address and port
    let address = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("🚀 router initialized, now listening on port {}", port);

    // Start serving incoming connections
    axum::serve(address, app.into_make_service()).await?;

    Ok(())
}

/// This function defines the API routes for the application.
/// It takes the application state as input and sets up
/// the routes for handling different HTTP methods and endpoints.
fn api_router(app_state: Arc<RwLock<AppState>>) -> Router {
    // Get the current directory for serving assets
    let assets_path = std::env::current_dir().unwrap();

    Router::new()
        .route("/", get(handler::app).post(handler::post_url))
        .route("/:url", get(handler::get_url))
        .route("/404", get(handler::handler_404))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())), // Serve static assets
        )
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
}
