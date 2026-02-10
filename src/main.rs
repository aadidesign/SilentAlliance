//! SilentAlliance - Anonymous, Privacy-First Social Discussion Platform
//!
//! This is the main entry point for the SilentAlliance API server.
//! It initializes all services, establishes database connections, and starts the HTTP server.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use silent_alliance::{
    api::create_router,
    config::Settings,
    infrastructure::{
        database::DatabasePool,
        cache::RedisPool,
        crypto::CryptoService,
        storage::StorageService,
    },
    AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber with JSON formatting for production
    init_tracing();

    info!("Starting SilentAlliance API Server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let settings = match Settings::load() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let db_pool = match DatabasePool::new(&settings.database).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };
    info!("Database connection pool established");

    // Run database migrations
    if let Err(e) = db_pool.run_migrations().await {
        error!("Failed to run database migrations: {}", e);
        std::process::exit(1);
    }
    info!("Database migrations completed");

    // Initialize Redis connection pool
    let redis_pool = match RedisPool::new(&settings.redis).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };
    info!("Redis connection pool established");

    // Initialize cryptographic service
    let crypto_service = match CryptoService::new(&settings.crypto) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to initialize crypto service: {}", e);
            std::process::exit(1);
        }
    };
    info!("Cryptographic service initialized");

    // Initialize storage service
    let storage_service = match StorageService::new(&settings.storage).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to initialize storage service: {}", e);
            std::process::exit(1);
        }
    };
    info!("Storage service initialized");

    // Create shared application state
    let app_state = Arc::new(AppState {
        db: db_pool,
        redis: redis_pool,
        crypto: crypto_service,
        storage: storage_service,
        settings: settings.clone(),
    });

    // Create the router with all routes and middleware
    let app = create_router(app_state.clone());

    // Bind to the configured host and port
    let host: std::net::IpAddr = settings
        .server
        .host
        .parse()
        .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    let addr = SocketAddr::from((host, settings.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on http://{}", addr);
    info!("API documentation available at http://{}/api/docs", addr);

    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Initialize the tracing subscriber for structured logging
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,silent_alliance=debug,tower_http=debug,sqlx=warn"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Use JSON formatting in production, pretty formatting in development
    if std::env::var("RUST_ENV").unwrap_or_default() == "production" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer.json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer.pretty())
            .init();
    }
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            warn!("Received terminate signal, initiating graceful shutdown...");
        }
    }
}
