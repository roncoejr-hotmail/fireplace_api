# GPIO Implementation Fix - Using gpio Command

## Problem
The rppal Rust GPIO library wasn't controlling the pins despite compiling successfully and logging correct pin numbers. The hardware worked fine with the Python implementation using `RPi.GPIO`.

## Solution
Switched from rppal library to using the system `gpio` command, which:
1. Uses the same underlying control as the working Python implementation
2. Doesn't require special library initialization
3. Works reliably on Raspberry Pi with WiringPi installed
4. Matches the Python code's behavior (setup pin → set HIGH/LOW)

## Key Changes

### src/gpio.rs
- Removed rppal library dependency
- Replaced with `std::process::Command` to execute `gpio` system commands
- Implemented read/write functions that call:
  - `gpio mode <pin> out/in` - Set pin direction
  - `gpio write <pin> 0/1` - Write pin state
  - `gpio read <pin>` - Read pin state

### Cargo.toml
- Removed `rppal = "0.14"` dependency
- Removed `lazy_static = "1.4"` dependency
- No new dependencies needed!

## How It Works

Before toggling a pin:
1. **Read current state**: `gpio read <pin>` → returns 0 or 1
2. **Set mode**: `gpio mode <pin> out`
3. **Write new state**: `gpio write <pin> <value>`

This mirrors the Python code's approach:
```python
GPIO.setup(m_PIN, GPIO.OUT)  # ← our `gpio mode <pin> out`
GPIO.output(m_PIN, GPIO.HIGH)  # ← our `gpio write <pin> 1`
```

## Deployment

### On Raspberry Pi:

1. **Transfer the binary** (from your dev machine):
   ```bash
   scp target/release/fireplace_api pi@<your-pi-ip>:~/
   ```

2. **Deploy** (on the Pi):
   ```bash
   # Option 1: Use the deploy script
   bash ~/deploy_fireplace_api.sh ~/fireplace_api

   # Option 2: Manual installation
   sudo cp ~/fireplace_api /usr/local/bin/
   sudo chmod +x /usr/local/bin/fireplace_api
   sudo systemctl restart fireplace_api
   ```

3. **Verify**:
   ```bash
   sudo systemctl status fireplace_api
   # or test directly with sudo:
   sudo /usr/local/bin/fireplace_api
   ```

## Testing

### Before restarting the service, test locally:
```bash
# Make a test request (replace IP with your Pi's address)
curl "http://localhost:8090/api/v1/fireplace/control?action=toggle&pin=37"

# Check service logs
sudo journalctl -u fireplace_api -f
```

### Expected output in logs:
```
DEBUG Attempting to toggle GPIO pin 37
DEBUG GPIO pin 37 current state: Low
DEBUG Setting GPIO pin 37 to HIGH
INFO GPIO Pin 37 toggled to High
```

## Troubleshooting

### "gpio: command not found"
Install WiringPi GPIO tool:
```bash
sudo apt-get update
sudo apt-get install wiringpi
```

### Permission denied errors
Make sure the service runs with `User=root`:
```bash
sudo cat /etc/systemd/system/fireplace_api.service | grep User
```

### Pins still not activating
Check GPIO directly:
```bash
# Manual test
sudo gpio mode 37 out
sudo gpio write 37 1  # Should activate
sudo gpio write 37 0  # Should deactivate
```

## Hardware Notes

- **Physical pins**: 37 (fireplace), 38 (fan) - using GPIO.BOARD numbering
- **Relay activation**: LOW=ON, HIGH=OFF (or vice versa depending on relay configuration)
- **GPIO command**: Uses WiringPi numbering internally (converted from BOARD)

## Why This Works Better

1. **Proven**: Same approach as working Python code
2. **Simple**: No library-specific initialization issues
3. **Reliable**: Uses WiringPi, the standard on Raspberry Pi
4. **Debuggable**: Can test pins manually with `gpio` commands
5. **No dependencies**: Only needs the `gpio` command-line tool
