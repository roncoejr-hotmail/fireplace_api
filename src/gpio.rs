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
        
        // Use gpioset with --chip and --hold-period (gpioset v2.x)
        // Spawn without waiting since --hold-period will auto-exit after the pulse
        // Use short 50ms pulse to minimize chance of "device busy" errors
        // Syntax: gpioset --chip gpiochip0 --hold-period 50ms PIN=VALUE
        let value = if high { "1" } else { "0" };
        let pin_value = format!("{}={}", pin, value);
        
        tracing::debug!("Executing: gpioset --chip gpiochip0 --hold-period 50ms {}", pin_value);
        Command::new("gpioset")
            .args(&["--chip", "gpiochip0", "--hold-period", "50ms", &pin_value])
            .spawn()
            .map_err(|e| {
                tracing::error!("Failed to execute gpioset command: {}", e);
                crate::error::ApiError::GpioError(format!("gpioset command failed: {}", e))
            })?;

        tracing::info!("GPIO pin {} pulse initiated to {}", pin, value);
        Ok(())
    }

    /// Read from a GPIO pin using gpioget command
    fn read_gpio_pin(&self, pin: u32) -> crate::error::Result<PinState> {
        tracing::debug!("Reading GPIO pin {}", pin);
        
        // Use gpioget with --chip (gpioget v2.x)
        // Syntax: gpioget --chip gpiochip0 PIN
        let pin_str = pin.to_string();
        tracing::debug!("Executing: gpioget --chip gpiochip0 {}", pin);
        let read_result = Command::new("gpioget")
            .args(&["--chip", "gpiochip0", &pin_str])
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
