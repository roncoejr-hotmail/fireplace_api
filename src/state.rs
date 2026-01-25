use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::config::Config>,
    pub gpio_controller: Arc<Mutex<crate::gpio::GpioController>>,
}
