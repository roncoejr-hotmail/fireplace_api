# Fireplace API - Raspberry Pi Setup Guide

This guide walks you through deploying the Fireplace API server on a Raspberry Pi with proper configuration handling.

## Configuration File Priority

The server loads configuration in this order (first found wins):

1. **Environment Variable**: `FIREPLACE_API_CONFIG=/path/to/config.toml`
2. **System Directory**: `/etc/fireplace-api/family_room.toml` (recommended)
3. **Current Directory**: `./config/family_room.toml`
4. **Built-in Defaults**: If no config files found, uses hardcoded defaults

## Setup Steps

### 1. Copy Binary and Config Files to Pi

```bash
# On your development machine
scp target/release/fireplace_api pi@192.168.1.X:/home/pi/
scp config/family_room.toml pi@192.168.1.X:/home/pi/

# Or if you're on the Pi, build there
cargo build --release
```

### 2. Create System Configuration Directory (Recommended)

```bash
# SSH to Raspberry Pi
ssh pi@192.168.1.X

# Create the config directory
sudo mkdir -p /etc/fireplace-api

# Copy your config file
sudo cp ~/family_room.toml /etc/fireplace-api/
sudo chown root:root /etc/fireplace-api/family_room.toml
sudo chmod 644 /etc/fireplace-api/family_room.toml

# Verify it works
./fireplace_api
# Should log: Configuration loaded from /etc/fireplace-api/family_room.toml
```

### 3. Install as Systemd Service (Optional but Recommended)

Create `/etc/systemd/system/fireplace-api.service`:

```ini
[Unit]
Description=Fireplace API Server
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi
ExecStart=/home/pi/fireplace_api
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Optional: Set environment variables
# Environment="RUST_LOG=fireplace_api=debug"
# Environment="FIREPLACE_API_CONFIG=/etc/fireplace-api/family_room.toml"

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable fireplace-api
sudo systemctl start fireplace-api

# Check status
sudo systemctl status fireplace-api

# View logs
sudo journalctl -u fireplace-api -f
```

### 4. Enable GPIO Access

If you have a Raspberry Pi 3B and want to control actual GPIO pins:

1. **Uncomment rppal in Cargo.toml** (if commented):
   ```toml
   rppal = "0.14"
   ```

2. **Update src/gpio.rs** to use actual GPIO instead of mock:
   ```rust
   pub async fn toggle_pin(&mut self, pin: u32) -> crate::error::Result<()> {
       use rppal::gpio::Gpio;
       
       let gpio = Gpio::new()?;
       let mut gpio_pin = gpio.get(pin)?.into_output();
       gpio_pin.toggle();
       
       let new_state = match gpio_pin.read() {
           rppal::gpio::Level::High => PinState::High,
           rppal::gpio::Level::Low => PinState::Low,
       };
       
       self.pin_states.insert(pin, new_state.clone());
       tracing::info!("GPIO Pin {} toggled to {:?}", pin, new_state);
       Ok(())
   }
   ```

3. **Add GPIO permissions**:
   ```bash
   # Make pi user able to access GPIO without sudo
   sudo usermod -aG gpio pi
   
   # Log out and back in for group changes to take effect
   exit
   ssh pi@192.168.1.X
   ```

4. **Rebuild and redeploy**:
   ```bash
   cargo build --release
   sudo systemctl restart fireplace-api
   ```

## Configuration File Format

Create `family_room.toml` (or `master_bedroom.toml`):

```toml
[room]
name = "family_room"
device_ip = "192.168.1.100"  # Optional: IP of device if needed

[pins]
fireplace = 17
fireplace_fan = 27
lights = 22
secondary_device = 23

[safety]
max_pulse_duration_ms = 5000
require_confirmation = false
```

## Testing the Deployment

### From the Pi directly:

```bash
# Test health endpoint
curl http://localhost:8090/health

# Test legacy endpoint
curl "http://localhost:8090/?cmdType=toggle&cmdAction=ON&v_ACTION=on&m_PIN=17"

# Test modern endpoint
curl -X POST http://localhost:8090/api/v1/fireplace/control \
  -H "Content-Type: application/json" \
  -d '{"action":"ON","device":"fireplace"}'
```

### From another machine:

```bash
# Replace 192.168.1.X with your Pi's IP
curl http://192.168.1.X:8090/health

# Test status
curl http://192.168.1.X:8090/api/v1/gpio/status
```

## Troubleshooting

### "Failed to load config: No such file or directory"

This means the config file wasn't found in any of these locations:
1. `$FIREPLACE_API_CONFIG` environment variable
2. `/etc/fireplace-api/family_room.toml`
3. `./config/family_room.toml`

**Solution**: Place your config file in one of these locations, or set the environment variable.

### Port 8090 Already in Use

```bash
# Find what's using the port
sudo lsof -i :8090

# Kill the process if needed
sudo kill -9 <PID>

# Or stop the systemd service
sudo systemctl stop fireplace-api
```

### Permission Denied on GPIO

If you get GPIO permission errors:

```bash
# Check your user is in gpio group
groups pi

# If not, add it
sudo usermod -aG gpio pi

# Check GPIO permissions
ls -la /sys/class/gpio/
# Should be readable/writable by gpio group
```

### Not Listening on Port 8090

Check the logs:

```bash
# If using systemd
sudo journalctl -u fireplace-api -n 50

# If running directly
RUST_LOG=fireplace_api=debug ./fireplace_api
```

## Environment Variables

- `RUST_LOG`: Set logging level (e.g., `fireplace_api=debug`)
- `FIREPLACE_API_CONFIG`: Full path to config file

Example:
```bash
export RUST_LOG=fireplace_api=debug
export FIREPLACE_API_CONFIG=/etc/fireplace-api/family_room.toml
./fireplace_api
```

## Next Steps

1. **Configure your React UI** to point to your Pi's IP on port 8090
2. **Test endpoint connections** from your development machine
3. **Monitor logs** during initial testing for any issues
4. **Set up HTTPS** (optional but recommended for security) using a reverse proxy like nginx

## Support

For issues or questions, check:
- `/var/log/syslog` for systemd service logs
- `journalctl` for service logs
- Run with `RUST_LOG=debug` for detailed logging
