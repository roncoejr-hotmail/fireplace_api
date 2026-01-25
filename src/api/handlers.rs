use axum::{
    extract::{Query, State, Json},
    http::StatusCode,
};
use chrono::Local;
use crate::{
    api::models::*,
    error::{ApiError, Result},
    state::AppState,
};

/// Handle legacy GPIO endpoint (backward compatible)
pub async fn handle_legacy_gpio(
    Query(req): Query<LegacyGpioRequest>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse>> {
    tracing::debug!("Legacy GPIO request: {:?}", req);

    // Validate command type
    if req.cmd_type.to_lowercase() != "toggle" {
        return Err(ApiError::InvalidCommand);
    }

    // Validate action
    let action_upper = req.cmd_action.to_uppercase();
    if action_upper != "ON" && action_upper != "OFF" {
        return Err(ApiError::InvalidAction);
    }

    // Get the GPIO pin and execute the command
    let pin = req.m_pin;
    let mut gpio = state.gpio_controller.lock().await;
    
    // Use explicit ON/OFF instead of toggle
    // "ON" means set HIGH, "OFF" means set LOW
    let set_high = action_upper == "ON";
    gpio.set_pin(pin, set_high).await?;

    let device_name = state.config.get_pin_name(pin);

    Ok(Json(ApiResponse {
        success: true,
        action: action_upper,
        pin,
        device: device_name,
        timestamp: Local::now().to_rfc3339(),
    }))
}

/// Handle modern fireplace control endpoint
pub async fn handle_fireplace_control(
    State(state): State<AppState>,
    Json(req): Json<FireplaceControlRequest>,
) -> Result<Json<ApiResponse>> {
    tracing::debug!("Fireplace control request: {:?}", req);

    // Determine which PIN to control
    let pin = match req.device.to_lowercase().as_str() {
        "fireplace" => state.config.pins.fireplace,
        "fan" => state.config.pins.fireplace_fan,
        _ => return Err(ApiError::InvalidPin),
    };

    // Validate action
    let action_upper = req.action.to_uppercase();
    if action_upper != "ON" && action_upper != "OFF" {
        return Err(ApiError::InvalidAction);
    }

    // Execute with explicit ON/OFF instead of toggle
    // "ON" means set HIGH, "OFF" means set LOW
    let mut gpio = state.gpio_controller.lock().await;
    let set_high = action_upper == "ON";
    gpio.set_pin(pin, set_high).await?;

    Ok(Json(ApiResponse {
        success: true,
        action: action_upper,
        pin,
        device: Some(req.device),
        timestamp: Local::now().to_rfc3339(),
    }))
}

/// Get status of all GPIO pins
pub async fn handle_gpio_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>> {
    let gpio = state.gpio_controller.lock().await;
    let pins = gpio.get_all_pin_states();

    Ok(Json(StatusResponse {
        room: state.config.room.name.clone(),
        pins,
    }))
}

/// Get current configuration
pub async fn handle_get_config(
    State(state): State<AppState>,
) -> Result<Json<ConfigResponse>> {
    let config = &state.config;

    Ok(Json(ConfigResponse {
        room: config.room.name.clone(),
        pins: serde_json::to_value(&config.pins)
            .map_err(|_| ApiError::InternalError)?,
        safety: serde_json::to_value(&config.safety)
            .map_err(|_| ApiError::InternalError)?,
    }))
}

/// Reload configuration from file
pub async fn handle_reload_config(
    State(_state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // In a real implementation, this would reload from the config file
    // For now, just acknowledge the request
    tracing::info!("Configuration reload requested");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Configuration reload requested",
            "timestamp": Local::now().to_rfc3339(),
        })),
    ))
}

/// Health check endpoint
pub async fn handle_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime_ms: 0, // Could track actual uptime
    })
}
