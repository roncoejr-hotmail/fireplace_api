use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinState {
    High,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinStatus {
    pub pin: u32,
    pub state: PinState,
    pub last_toggled: Option<String>,
}

pub struct GpioController {
    pin_states: HashMap<u32, PinState>,
}

impl GpioController {
    pub fn new() -> Self {
        Self {
            pin_states: HashMap::new(),
        }
    }

    /// Toggle a GPIO pin using the gpio command
    pub async fn toggle_pin(&mut self, pin: u32) -> crate::error::Result<()> {
        tracing::debug!("Attempting to toggle GPIO pin {}", pin);

        // Read current state
        let current_state = self.read_gpio_pin(pin)?;
        
        tracing::debug!("GPIO pin {} current state: {:?}", pin, current_state);
        
        // Determine new state (toggle)
        let new_state = match current_state {
            PinState::High => {
                tracing::debug!("Setting GPIO pin {} to LOW", pin);
                self.write_gpio_pin(pin, false)?;
                PinState::Low
            },
            _ => {
                tracing::debug!("Setting GPIO pin {} to HIGH", pin);
                self.write_gpio_pin(pin, true)?;
                PinState::High
            }
        };

        tracing::debug!("GPIO pin {} new state: {:?}", pin, new_state);
        self.pin_states.insert(pin, new_state.clone());
        tracing::info!("GPIO Pin {} toggled to {:?}", pin, new_state);
        Ok(())
    }

    /// Set a GPIO pin to a specific state
    pub async fn set_pin(&mut self, pin: u32, logical_on: bool, active_low: bool) -> crate::error::Result<()> {
        let physical_high = if active_low { !logical_on } else { logical_on };
        let state = if physical_high { PinState::High } else { PinState::Low };
        tracing::debug!(
            "Setting GPIO pin {} to {:?} (logical_on={}, active_low={}, physical_high={})",
            pin,
            state,
            logical_on,
            active_low,
            physical_high
        );
        
        self.write_gpio_pin(pin, physical_high)?;
        self.pin_states.insert(pin, state.clone());
        tracing::info!("GPIO Pin {} set to {:?}", pin, state);
        Ok(())
    }

    /// Get the current state of a pin
    pub fn get_pin_state(&self, pin: u32) -> PinState {
        self.pin_states
            .get(&pin)
            .cloned()
            .unwrap_or(PinState::Unknown)
    }

    /// Get all pin states
    pub fn get_all_pin_states(&self) -> Vec<PinStatus> {
        self.pin_states
            .iter()
            .map(|(pin, state)| PinStatus {
                pin: *pin,
                state: state.clone(),
                last_toggled: Some(chrono::Local::now().to_rfc3339()),
            })
            .collect()
    }

    // Private helper methods for gpio command execution
    
    /// Write to a GPIO pin using the gpio command
    fn write_gpio_pin(&self, pin: u32, high: bool) -> crate::error::Result<()> {
        
        tracing::info!("Writing GPIO pin {} to {}", pin, if high { "HIGH" } else { "LOW"});
        
        // Set pin mode to output
        tracing::debug!("Setting GPIO pin {} mode to output", pin);
        let mode_result = Command::new("gpio")
            .args(&["mode", &pin.to_string(), "out"])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpio mode command: {}", e);
                crate::error::ApiError::GpioError(format!("gpio mode command failed: {}", e))
            })?;
        
        if !mode_result.status.success() {
            let stderr = String::from_utf8_lossy(&mode_result.stderr);
            tracing::error!("Failed to set GPIO mode for pin {}: {}", pin, stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to set GPIO mode: {}", stderr)));
        }
        
        // Write the pin state
        let value = if high { "1" } else { "0" };
        
        tracing::debug!("Writing GPIO pin {} value {}", pin, value);
        let write_result = Command::new("gpio")
            .args(&["write", &pin.to_string(), value])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpio write command: {}", e);
                crate::error::ApiError::GpioError(format!("gpio write command failed: {}", e))
            })?;
        
        if !write_result.status.success() {
            let stderr = String::from_utf8_lossy(&write_result.stderr);
            tracing::error!("Failed to write GPIO pin {}: {}", pin, stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to write GPIO: {}", stderr)));
        }

        tracing::info!("GPIO pin {} set to {}", pin, value);
        Ok(())
    }

    /// Read from a GPIO pin using the gpio command
    fn read_gpio_pin(&self, pin: u32) -> crate::error::Result<PinState> {
        tracing::debug!("Reading GPIO pin {}", pin);
        
        // Set pin mode to input
        let mode_result = Command::new("gpio")
            .args(&["mode", &pin.to_string(), "in"])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpio mode command: {}", e);
                crate::error::ApiError::GpioError(format!("gpio mode command failed: {}", e))
            })?;
        
        if !mode_result.status.success() {
            let stderr = String::from_utf8_lossy(&mode_result.stderr);
            tracing::error!("Failed to set GPIO mode for pin {}: {}", pin, stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to set GPIO mode: {}", stderr)));
        }
        
        // Read the pin state
        tracing::debug!("Reading GPIO pin {} value", pin);
        let read_result = Command::new("gpio")
            .args(&["read", &pin.to_string()])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpio read command: {}", e);
                crate::error::ApiError::GpioError(format!("gpio read command failed: {}", e))
            })?;
        
        if !read_result.status.success() {
            let stderr = String::from_utf8_lossy(&read_result.stderr);
            tracing::error!("Failed to read GPIO pin {}: {}", pin, stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to read GPIO: {}", stderr)));
        }
        
        let state_str = String::from_utf8_lossy(&read_result.stdout).trim().to_string();
        let state = if state_str == "1" {
            PinState::High
        } else {
            PinState::Low
        };
        
        tracing::info!("GPIO pin {} read state: {:?}", pin, state);
        Ok(state)
    }
}
