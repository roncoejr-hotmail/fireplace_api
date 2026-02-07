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
    
    /// Write to a GPIO pin using gpioset command
    fn write_gpio_pin(&self, pin: u32, high: bool) -> crate::error::Result<()> {
        tracing::info!("Writing GPIO pin {} to {}", pin, if high { "HIGH" } else { "LOW"});
        
        // Use gpioset to drive the pin
        // Syntax: gpioset --mode=exit --chip=gpiochip0 PIN=VALUE
        let value = if high { "1" } else { "0" };
        let pin_str = pin.to_string();
        let pin_value = format!("{}={}", pin, value);
        
        tracing::debug!("Executing: gpioset --mode=exit --chip=gpiochip0 {}", pin_value);
        let write_result = Command::new("gpioset")
            .args(&["--mode=exit", "--chip=gpiochip0", &pin_value])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpioset command: {}", e);
                crate::error::ApiError::GpioError(format!("gpioset command failed: {}", e))
            })?;

        if !write_result.status.success() {
            let stderr = String::from_utf8_lossy(&write_result.stderr);
            let stdout = String::from_utf8_lossy(&write_result.stdout);
            tracing::error!("Failed to write GPIO pin {}: {} {}", pin, stdout, stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to write: {}", stderr)));
        }

        tracing::info!("GPIO pin {} set to {}", pin, value);
        Ok(())
    }

    /// Read from a GPIO pin using gpioget command
    fn read_gpio_pin(&self, pin: u32) -> crate::error::Result<PinState> {
        tracing::debug!("Reading GPIO pin {}", pin);
        
        // Use gpioget to read the pin state
        // Syntax: gpioget --chip=gpiochip0 PIN
        let pin_str = pin.to_string();
        tracing::debug!("Executing: gpioget --chip=gpiochip0 {}", pin);
        let read_result = Command::new("gpioget")
            .args(&["--chip=gpiochip0", &pin_str])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpioget command: {}", e);
                crate::error::ApiError::GpioError(format!("gpioget command failed: {}", e))
            })?;

        if !read_result.status.success() {
            let stderr = String::from_utf8_lossy(&read_result.stderr);
            tracing::error!("Failed to read GPIO pin {}: {}", pin, stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to read: {}", stderr)));
        }

        let output = String::from_utf8_lossy(&read_result.stdout).trim().to_string();
        tracing::debug!("GPIO {} raw output: '{}'", pin, output);
        
        let state = if output == "1" {
            PinState::High
        } else if output == "0" {
            PinState::Low
        } else {
            PinState::Unknown
        };
        
        tracing::info!("GPIO pin {} read state: {:?}", pin, state);
        Ok(state)
    }
}
