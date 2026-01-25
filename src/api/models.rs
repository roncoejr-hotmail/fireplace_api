use serde::{Deserialize, Serialize};

// Legacy request model for backward compatibility
#[derive(Debug, Deserialize)]
pub struct LegacyGpioRequest {
    #[serde(rename = "cmdType")]
    pub cmd_type: String,
    
    #[serde(rename = "cmdAction")]
    pub cmd_action: String,
    
    #[serde(rename = "v_ACTION")]
    pub v_action: String,
    
    #[serde(rename = "m_PIN")]
    pub m_pin: u32,
    
    #[serde(rename = "m_pulsePIN")]
    pub m_pulse_pin: Option<u32>,
    
    #[serde(rename = "m_monPIN")]
    pub m_mon_pin: Option<u32>,
    
    #[serde(rename = "n_CYCLE")]
    pub n_cycle: Option<u32>,
}

// Modern request model
#[derive(Debug, Deserialize)]
pub struct FireplaceControlRequest {
    pub action: String,      // ON or OFF
    pub device: String,      // fireplace or fan
    pub room: Option<String>, // optional room identifier
}

// Unified response model
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub action: String,
    pub pin: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub room: String,
    pub pins: Vec<crate::gpio::PinStatus>,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub room: String,
    pub pins: serde_json::Value,
    pub safety: serde_json::Value,
}
