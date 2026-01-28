use hap::{
    accessory::{lightbulb::LightbulbAccessory, AccessoryCategory, AccessoryInformation},
    server::{IpServer, Server},
    storage::{FileStorage, Storage},
    Config, MacAddress, Pin,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{gpio::GpioController, config::Config as AppConfig};

/// Start the HomeKit Accessory Protocol server
/// Exposes fireplace and fan as HomeKit lightbulb accessories
pub async fn start_hap_server(
    config: Arc<AppConfig>,
    gpio_controller: Arc<Mutex<GpioController>>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting HomeKit Accessory Protocol (HAP) server");

    // Create storage for HomeKit pairing data
    let storage = FileStorage::new("homekit_data")?;
    
    // Generate a unique PIN for HomeKit pairing (8 digits, format: XXX-XX-XXX)
    let pin = Pin::new([1, 2, 3, 4, 5, 6, 7, 8])?;
    
    // Create unique MAC address for this HomeKit bridge
    let mac_addr = MacAddress::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

    // Configure HAP server
    let hap_config = Config {
        pin,
        name: format!("{} Fireplace Control", config.room.name),
        device_id: mac_addr,
        category: AccessoryCategory::Lightbulb,
        ..Default::default()
    };

    tracing::info!("HomeKit PIN: {}", hap_config.pin);
    tracing::info!("HomeKit Device Name: {}", hap_config.name);

    // Create fireplace accessory (as a lightbulb)
    let fireplace_info = AccessoryInformation {
        name: "Fireplace".into(),
        manufacturer: "Custom".into(),
        model: "GPIO-Fireplace-v1".into(),
        serial_number: format!("FP-{}", config.room.name).into(),
        firmware_revision: "1.0.0".into(),
        ..Default::default()
    };

    let gpio_clone = Arc::clone(&gpio_controller);
    let config_clone = Arc::clone(&config);
    let fireplace_pin = config.pins.fireplace;
    let active_low = config.pins.active_low;

    let mut fireplace = LightbulbAccessory::new(1, fireplace_info)?;
    
    // Set up on/off callback for fireplace
    fireplace.lightbulb.on.on_update(move |current: &bool, new: &bool| {
        if current != new {
            let gpio = gpio_clone.clone();
            let pin = fireplace_pin;
            let logical_on = *new;
            let active_low_flag = active_low;
            
            tokio::spawn(async move {
                let mut gpio_lock = gpio.lock().await;
                if let Err(e) = gpio_lock.set_pin(pin, logical_on, active_low_flag).await {
                    tracing::error!("HAP: Failed to control fireplace GPIO: {}", e);
                } else {
                    tracing::info!("HAP: Fireplace turned {}", if logical_on { "ON" } else { "OFF" });
                }
            });
        }
        Ok(())
    });

    // Create fan accessory (as a lightbulb)
    let fan_info = AccessoryInformation {
        name: "Fireplace Fan".into(),
        manufacturer: "Custom".into(),
        model: "GPIO-Fan-v1".into(),
        serial_number: format!("FAN-{}", config.room.name).into(),
        firmware_revision: "1.0.0".into(),
        ..Default::default()
    };

    let gpio_clone2 = Arc::clone(&gpio_controller);
    let config_clone2 = Arc::clone(&config);
    let fan_pin = config.pins.fireplace_fan;
    let active_low2 = config.pins.active_low;

    let mut fan = LightbulbAccessory::new(2, fan_info)?;
    
    // Set up on/off callback for fan
    fan.lightbulb.on.on_update(move |current: &bool, new: &bool| {
        if current != new {
            let gpio = gpio_clone2.clone();
            let pin = fan_pin;
            let logical_on = *new;
            let active_low_flag = active_low2;
            
            tokio::spawn(async move {
                let mut gpio_lock = gpio.lock().await;
                if let Err(e) = gpio_lock.set_pin(pin, logical_on, active_low_flag).await {
                    tracing::error!("HAP: Failed to control fan GPIO: {}", e);
                } else {
                    tracing::info!("HAP: Fan turned {}", if logical_on { "ON" } else { "OFF" });
                }
            });
        }
        Ok(())
    });

    // Create and start the HAP server
    let server = IpServer::new(hap_config, storage).await?;
    server.add_accessory(fireplace).await?;
    server.add_accessory(fan).await?;

    tracing::info!("HAP server ready on port 5353");
    tracing::info!("Add to HomeKit using PIN: {}", pin);
    tracing::info!("Open Home app → Add Accessory → Enter PIN manually");

    // Run the server (blocks)
    server.run().await?;

    Ok(())
}
