# HomeKit Integration Guide

This fireplace API now includes **native HomeKit support** via the HomeKit Accessory Protocol (HAP). You can control your fireplace and fan directly from the Apple Home app, Siri, and automation routines—no Homebridge required.

## What Gets Exposed

Two HomeKit accessories appear in your Home app:
- **Fireplace** (shown as a lightbulb/switch)
- **Fireplace Fan** (shown as a lightbulb/switch)

Both are simple on/off controls that directly drive the GPIO pins with proper active-low handling.

## Initial Setup

### 1. Start the Server

On the Raspberry Pi:
```bash
./target/release/fireplace_api --config config/master_bedroom.toml
```

Or with environment variable:
```bash
FIREPLACE_API_CONFIG=config/master_bedroom.toml ./target/release/fireplace_api
```

### 2. Note the HomeKit PIN

When the server starts, you'll see:
```
HomeKit PIN: 123-45-678
HomeKit Device Name: master_bedroom Fireplace Control
HAP server ready on port 5353
Add to HomeKit using PIN: 123-45-678
```

**IMPORTANT:** The default PIN is `123-45-678`. This is hardcoded for simplicity. For production, you should generate a random PIN or allow it to be configured.

### 3. Add to Home App

On your iPhone/iPad:
1. Open the **Home** app
2. Tap **+** (top right) → **Add Accessory**
3. Select **More options...**
4. You should see "**master_bedroom Fireplace Control**" appear
5. Tap it and select **Add Anyway** (it will warn about uncertified accessory)
6. Enter PIN: **123-45-678**
7. Choose a room (e.g., "Master Bedroom")
8. Name the accessories (default: "Fireplace" and "Fireplace Fan")
9. Tap **Done**

### 4. Control

- **Siri:** "Hey Siri, turn on the fireplace"
- **Home App:** Tap the fireplace tile to toggle
- **Automations:** Create schedules, scenes, etc.

## Technical Details

### Ports
- REST API: `8090` (existing)
- HomeKit HAP: `5353` (mDNS/Bonjour)

Both servers run in parallel. REST API remains fully functional.

### Persistent Storage

HomeKit pairing data is stored in:
```
./homekit_data/
```

This directory contains cryptographic keys and pairing records. **Do not delete** or you'll need to re-pair all devices.

### GPIO Handling

The HAP callbacks use the same `GpioController::set_pin()` method as the REST API, so:
- Active-low relays work correctly
- All logging/tracing is consistent
- Same error handling applies

### Icons

HomeKit doesn't support custom "fireplace" icons natively. Accessories appear as lightbulbs/switches in the Home app. Third-party apps (Eve, Home+) may allow custom icons.

## Troubleshooting

**Accessory doesn't appear in Home app:**
- Ensure iPhone and Pi are on the same network
- Check Pi firewall allows port 5353 (mDNS)
- Restart the server and check logs for HAP errors

**"Unable to Add Accessory":**
- Verify PIN is correct: `123-45-678`
- Check server logs for pairing errors
- Delete `homekit_data/` and restart to reset pairing

**Controls don't work:**
- Check GPIO permissions (`gpio` command needs proper access)
- Review server logs for GPIO errors
- Test REST API to verify GPIO layer works

**Want to change the PIN:**
Edit [src/hap_server.rs](src/hap_server.rs#L19):
```rust
let pin = Pin::new([1, 2, 3, 4, 5, 6, 7, 8])?;  // Change these 8 digits
```

Format: First 3 digits, middle 2, last 3 (displayed as XXX-XX-XXX).

## Security Notes

- Default PIN is **not secure** for production
- HAP uses encrypted communication after pairing
- Only paired devices can control accessories
- Pairing keys stored in `homekit_data/` should be protected
- For multi-room setup, consider unique PINs and MAC addresses per Pi

## Next Steps

- Set up automations (e.g., "Turn off fireplace at 11 PM")
- Add to scenes (e.g., "Movie Night" turns on fireplace + dims lights)
- Use Siri shortcuts for voice control
- Monitor logs: `tail -f fireplace_api.log` (if using systemd/logging)
