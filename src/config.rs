use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub room: RoomConfig,
    pub pins: PinConfig,
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub name: String,
    pub device_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinConfig {
    pub fireplace: u32,
    pub fireplace_fan: u32,
    #[serde(default)]
    pub lights: Option<u32>,
    #[serde(default)]
    pub secondary_device: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_pulse_duration_ms: u32,
    pub require_confirmation: bool,
}

impl Config {
    pub fn load(path: &str) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::ApiError::ConfigError(format!("Failed to read config: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| crate::error::ApiError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    pub fn default() -> Self {
        Self {
            room: RoomConfig {
                name: "family_room".to_string(),
                device_ip: Some("127.0.0.1".to_string()),
            },
            pins: PinConfig {
                fireplace: 17,
                fireplace_fan: 27,
                lights: Some(22),
                secondary_device: Some(23),
            },
            safety: SafetyConfig {
                max_pulse_duration_ms: 5000,
                require_confirmation: false,
            },
        }
    }

    pub fn get_pin_name(&self, pin: u32) -> Option<String> {
        if pin == self.pins.fireplace {
            Some("fireplace".to_string())
        } else if pin == self.pins.fireplace_fan {
            Some("fireplace_fan".to_string())
        } else if let Some(lights_pin) = self.pins.lights {
            if pin == lights_pin {
                return Some("lights".to_string());
            }
            None
        } else if let Some(secondary_pin) = self.pins.secondary_device {
            if pin == secondary_pin {
                return Some("secondary_device".to_string());
            }
            None
        } else {
            None
        }
    }
}
