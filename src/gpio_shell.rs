use std::process::Command;
use log::{debug, error};

/// Toggle a GPIO pin using the gpio command (compatible with RPi.GPIO behavior)
pub fn toggle_pin(pin: u32, state: bool) -> Result<bool, String> {
    debug!("GPIO Shell: Attempting to toggle pin {} to {}", pin, if state { "HIGH" } else { "LOW" });
    
    // Set pin mode to output
    let mode_output = Command::new("gpio")
        .args(&["mode", &pin.to_string(), "out"])
        .output();
    
    match mode_output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Failed to set GPIO mode: {}", stderr);
                return Err(format!("Failed to set GPIO mode: {}", stderr));
            }
            debug!("GPIO pin {} mode set to output", pin);
        }
        Err(e) => {
            error!("Failed to execute gpio mode command: {}", e);
            return Err(format!("Failed to execute gpio command: {}", e));
        }
    }
    
    // Write the pin state
    let write_value = if state { "1" } else { "0" };
    let write_output = Command::new("gpio")
        .args(&["write", &pin.to_string(), write_value])
        .output();
    
    match write_output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Failed to write GPIO pin: {}", stderr);
                return Err(format!("Failed to write GPIO pin: {}", stderr));
            }
            debug!("GPIO pin {} set to {}", pin, write_value);
            Ok(state)
        }
        Err(e) => {
            error!("Failed to execute gpio write command: {}", e);
            Err(format!("Failed to write GPIO pin: {}", e))
        }
    }
}

/// Read the current state of a GPIO pin
pub fn read_pin(pin: u32) -> Result<bool, String> {
    debug!("GPIO Shell: Reading state of pin {}", pin);
    
    // Set pin mode to input first
    let mode_output = Command::new("gpio")
        .args(&["mode", &pin.to_string(), "in"])
        .output();
    
    if let Err(e) = mode_output {
        error!("Failed to set GPIO mode to input: {}", e);
        return Err(format!("Failed to set GPIO mode: {}", e));
    }
    
    // Read the pin state
    let read_output = Command::new("gpio")
        .args(&["read", &pin.to_string()])
        .output();
    
    match read_output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Failed to read GPIO pin: {}", stderr);
                return Err(format!("Failed to read GPIO pin: {}", stderr));
            }
            
            let state_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let state = state_str.parse::<u32>().unwrap_or(0) != 0;
            debug!("GPIO pin {} state: {}", pin, state);
            Ok(state)
        }
        Err(e) => {
            error!("Failed to execute gpio read command: {}", e);
            Err(format!("Failed to read GPIO pin: {}", e))
        }
    }
}
