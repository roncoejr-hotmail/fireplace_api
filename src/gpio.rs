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
    pub async fn set_pin(&mut self, pin: u32, high: bool) -> crate::error::Result<()> {
        let state = if high { PinState::High } else { PinState::Low };
        tracing::debug!("Setting GPIO pin {} to {:?}", pin, state);
        
        self.write_gpio_pin(pin, high)?;
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
    
    /// Write to a GPIO pin using the gpioset command with BCM numbering
    fn write_gpio_pin(&self, pin: u32, high: bool) -> crate::error::Result<()> {
        // Convert physical pin to BCM GPIO number
        // Physical 37 = GPIO26, Physical 38 = GPIO20
        let bcm_pin = self.physical_to_bcm(pin)?;
        
        tracing::info!("Writing GPIO: Physical pin {} = BCM GPIO {}, value: {}", 
            pin, bcm_pin, if high { "HIGH" } else { "LOW"});
        
        // Use gpioset to toggle the pin with a pulse
        // Format: gpioset -c gpiochip0 -t 200ms,0 <pin>=<0|1>
        // The -t flag: set value, wait 200ms, toggle (to opposite), then exit (0 period)
        let value = if high { "1" } else { "0" };
        let gpio_arg = format!("{}={}", bcm_pin, value);
        
        let args = vec!["-c", "gpiochip0", "-t", "200ms,0", &gpio_arg];
        
        tracing::debug!("Executing: gpioset {}", args.join(" "));
        let write_result = Command::new("gpioset")
            .args(&args)
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpioset command: {}", e);
                crate::error::ApiError::GpioError(format!("gpioset command failed: {}", e))
            })?;

        let write_stderr = String::from_utf8_lossy(&write_result.stderr);
        let write_stdout = String::from_utf8_lossy(&write_result.stdout);
        tracing::debug!("gpioset - status: {}, stdout: {}, stderr: {}", 
            write_result.status, write_stdout, write_stderr);

        if !write_result.status.success() {
            tracing::error!("Failed to write GPIO {}: {} {}", bcm_pin, write_stdout, write_stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to write: {}", write_stderr)));
        }

        tracing::info!("GPIO pin {} (BCM {}) pulsed to {}", pin, bcm_pin, value);
        Ok(())
    }

    /// Read from a GPIO pin using the gpioget command
    fn read_gpio_pin(&self, pin: u32) -> crate::error::Result<PinState> {
        // Convert physical pin to BCM GPIO number
        let bcm_pin = self.physical_to_bcm(pin)?;
        
        tracing::debug!("Reading GPIO: Physical pin {} = BCM GPIO {}", pin, bcm_pin);
        
        // Use gpioget to read the pin state
        // Format: gpioget -c gpiochip0 <pin>
        tracing::debug!("Executing: gpioget -c gpiochip0 {}", bcm_pin);
        let read_result = Command::new("gpioget")
            .args(&["-c", "gpiochip0", &bcm_pin.to_string()])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute gpioget command: {}", e);
                crate::error::ApiError::GpioError(format!("gpioget command failed: {}", e))
            })?;

        let read_stderr = String::from_utf8_lossy(&read_result.stderr);
        let read_stdout = String::from_utf8_lossy(&read_result.stdout);
        tracing::debug!("gpioget - status: {}, stdout: '{}', stderr: '{}'", 
            read_result.status, read_stdout, read_stderr);

        if !read_result.status.success() {
            tracing::error!("Failed to read GPIO {}: {} {}", bcm_pin, read_stdout, read_stderr);
            return Err(crate::error::ApiError::GpioError(format!("Failed to read: {}", read_stderr)));
        }

        let state_str = String::from_utf8_lossy(&read_result.stdout).trim().to_string();
        tracing::debug!("GPIO {} (physical {}) raw read value: '{}'", bcm_pin, pin, state_str);
        
        let state = if state_str == "1" {
            PinState::High
        } else {
            PinState::Low
        };

        tracing::info!("GPIO pin {} (BCM {}) read state: {:?}", pin, bcm_pin, state);
        Ok(state)
    }

    /// Convert physical pin number to BCM GPIO number
    /// Physical pins 37-40 = GPIO26, GPIO20, GPIO21, GPIO16
    fn physical_to_bcm(&self, physical_pin: u32) -> crate::error::Result<u32> {
        let bcm = match physical_pin {
            37 => 26,  // Physical 37 = GPIO26
            38 => 20,  // Physical 38 = GPIO20
            22 => 25,  // Physical 22 = GPIO25
            23 => 24,  // Physical 23 = GPIO24
            _ => {
                tracing::warn!("Unknown physical pin {}, attempting to use as BCM", physical_pin);
                physical_pin
            }
        };
        tracing::debug!("Physical pin {} maps to BCM GPIO {}", physical_pin, bcm);
        Ok(bcm)
    }
}
