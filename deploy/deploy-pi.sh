#!/usr/bin/env bash
# Deploy opcua-howick to a factory Raspberry Pi
# Usage: ./deploy/deploy-pi.sh [pi-hostname]
#
# Prerequisites:
#   cargo install cross
#   SSH key auth set up to the Pi

set -euo pipefail

PI_HOST="${1:-factory-pi.local}"
PI_USER="pi"
BINARY="opcua-howick"
TARGET="aarch64-unknown-linux-gnu"

echo "Building for Raspberry Pi 5 (aarch64)..."
cross build --release --target "$TARGET"

BINARY_PATH="target/$TARGET/release/$BINARY"

echo "Deploying to $PI_USER@$PI_HOST..."
scp "$BINARY_PATH" "$PI_USER@$PI_HOST:~/$BINARY.new"
scp deploy/opcua-howick.service "$PI_USER@$PI_HOST:~/opcua-howick.service"
scp config.toml "$PI_USER@$PI_HOST:~/config.toml.example"

ssh "$PI_USER@$PI_HOST" << 'REMOTE'
  # Swap binary atomically
  mv ~/opcua-howick.new ~/opcua-howick
  chmod +x ~/opcua-howick

  # Install systemd service (first time only)
  if [ ! -f /etc/systemd/system/opcua-howick.service ]; then
    sudo mv ~/opcua-howick.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable opcua-howick
    echo "Service installed and enabled"
  fi

  # Restart
  sudo systemctl restart opcua-howick
  sleep 2
  sudo systemctl status opcua-howick --no-pager
REMOTE

echo "Done. opcua-howick running on $PI_HOST"
echo "OPC UA endpoint: opc.tcp://$PI_HOST:4840/"
