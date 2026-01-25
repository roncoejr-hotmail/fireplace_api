#!/bin/bash
# Deploy script for fireplace_api binary to Raspberry Pi
# Run this on the Pi: bash ~/deploy_fireplace_api.sh

echo "Deploying fireplace_api with GPIO command support..."

# Stop running server if it exists
if pgrep -x "fireplace_api" > /dev/null; then
    echo "Stopping existing fireplace_api server..."
    sudo pkill -f fireplace_api
    sleep 1
fi

# Copy binary (ensure you've transferred it first!)
# This assumes the binary is in the current directory or a known location
BINARY_PATH="${1:-./_dev/fireplace-api/target/release/fireplace_api}"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Please ensure fireplace_api binary is available"
    exit 1
fi

echo "Installing binary to /usr/local/bin/fireplace_api..."
sudo cp "$BINARY_PATH" /usr/local/bin/fireplace_api
sudo chmod +x /usr/local/bin/fireplace_api

# Create systemd service if it doesn't exist
if [ ! -f /etc/systemd/system/fireplace_api.service ]; then
    echo "Creating systemd service..."
    sudo tee /etc/systemd/system/fireplace_api.service > /dev/null << EOF
[Unit]
Description=Fireplace API Server
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/fireplace_api
Restart=on-failure
RestartSec=10
Environment="RUST_LOG=fireplace_api=debug"

[Install]
WantedBy=multi-user.target
EOF
fi

# Enable and start service
echo "Starting fireplace_api service..."
sudo systemctl daemon-reload
sudo systemctl enable fireplace_api
sudo systemctl start fireplace_api

echo "Checking service status..."
sudo systemctl status fireplace_api

echo "Deployment complete!"
echo "Service logs: sudo journalctl -u fireplace_api -f"
