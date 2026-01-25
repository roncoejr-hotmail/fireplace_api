use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    /// Toggle a GPIO pin (simulated for non-Pi systems)
    pub async fn toggle_pin(&mut self, pin: u32) -> crate::error::Result<()> {
        // On a real Raspberry Pi, this would use rppal:
        // use rppal::gpio::Gpio;
        // let gpio = Gpio::new()?;
        // let mut pin = gpio.get(pin)?.into_output();
        // pin.toggle();

        // For simulation, just toggle the state
        let current_state = self.pin_states.get(&pin).cloned().unwrap_or(PinState::Low);
        let new_state = match current_state {
            PinState::High => PinState::Low,
            PinState::Low => PinState::High,
            PinState::Unknown => PinState::High,
        };

        self.pin_states.insert(pin, new_state.clone());
        tracing::info!("GPIO Pin {} toggled to {:?}", pin, new_state);
        Ok(())
    }

    /// Set a GPIO pin to a specific state
    pub async fn set_pin(&mut self, pin: u32, high: bool) -> crate::error::Result<()> {
        let state = if high { PinState::High } else { PinState::Low };
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
}
