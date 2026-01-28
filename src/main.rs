mod api;
mod config;
mod error;
mod gpio;
mod state;
mod hap_server;

use axum::{
    Router,
    routing::get,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fireplace_api=debug".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Fireplace API Server");

    // Load configuration with fallback logic for master bedroom by default
    // Override via FIREPLACE_API_CONFIG env var if needed
    let config = config::Config::load_with_fallback("master_bedroom");

    // Create application state
    let state = state::AppState {
        config: Arc::new(config),
        gpio_controller: Arc::new(tokio::sync::Mutex::new(
            gpio::GpioController::new(),
        )),
    };

    // Build router with both legacy and modern endpoints
    let app = Router::new()
        // Legacy endpoint (backward compatible with Python API)
        .route("/", get(api::handlers::handle_legacy_gpio))
        
        // Health check
        .route("/health", get(api::handlers::handle_health))
        
        // Modern RESTful endpoints
        .route("/api/v1/fireplace/control", axum::routing::post(api::handlers::handle_fireplace_control))
        .route("/api/v1/gpio/status", get(api::handlers::handle_gpio_status))
        .route("/api/v1/config", get(api::handlers::handle_get_config))
        .route("/api/v1/config/reload", axum::routing::post(api::handlers::handle_reload_config))
        
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    // Clone state for HAP server
    let hap_gpio = Arc::clone(&state.gpio_controller);
    let hap_config = Arc::clone(&state.config);

    // Spawn HomeKit Accessory Protocol server in background
    tokio::spawn(async move {
        tracing::info!("Launching HomeKit Accessory Protocol server...");
        if let Err(e) = hap_server::start_hap_server(hap_config, hap_gpio).await {
            tracing::error!("HAP server error: {}", e);
        }
    });

    // Start REST API server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8090")
        .await
        .expect("Failed to bind to port 8090");

    tracing::info!("Server listening on http://0.0.0.0:8090");
    tracing::info!("Legacy endpoint: GET /?cmdType=toggle&cmdAction=ON&v_ACTION=on&m_PIN=37&m_pulsePIN=0&m_monPIN=0&n_CYCLE=0");
    tracing::info!("Modern endpoint: POST /api/v1/fireplace/control");
    tracing::info!("Health check: GET /health");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
