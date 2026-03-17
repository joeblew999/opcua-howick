#!/usr/bin/env bash
# Deploy opcua-howick to a factory Raspberry Pi.
# Wraps the mise deploy:pi task — prefer using that directly:
#
#   PI_HOST=pi@192.168.1.100 mise run deploy:pi
#
# This script exists for cases where mise is not installed on the deploy host.
#
# Usage: ./deploy/deploy-pi.sh [user@host]
#   Default host: pi@factory-pi.local

set -euo pipefail

PI_HOST="${1:-pi@factory-pi.local}"
TARGET="aarch64-unknown-linux-gnu"
BINARY="opcua-howick"

echo "=== opcua-howick factory deploy ==="
echo "Target: $PI_HOST"
echo ""

# Build
echo "Building for Raspberry Pi 5 (aarch64)..."
if command -v mise &>/dev/null; then
  PI_HOST="$PI_HOST" mise run deploy:pi
  exit 0
fi

# Fallback: direct cross invocation
if ! command -v cross &>/dev/null; then
  echo "ERROR: neither mise nor cross is installed."
  echo "Install mise: https://mise.jdx.dev"
  echo "Or: cargo install cross --git https://github.com/cross-rs/cross"
  exit 1
fi

cross build --release --target "$TARGET"

BINARY_PATH="target/$TARGET/release/$BINARY"
echo "Deploying to $PI_HOST..."
scp "$BINARY_PATH" "$PI_HOST:~/$BINARY.new"
scp deploy/opcua-howick.service "$PI_HOST:~/opcua-howick.service"
scp config.toml "$PI_HOST:~/config.toml.example"

ssh "$PI_HOST" << 'REMOTE'
  set -e
  mv ~/opcua-howick.new ~/opcua-howick
  chmod +x ~/opcua-howick
  if [ ! -f /etc/systemd/system/opcua-howick.service ]; then
    sudo mv ~/opcua-howick.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable opcua-howick
    echo "Service installed and enabled"
  fi
  if [ ! -f ~/config.toml ]; then
    cp ~/config.toml.example ~/config.toml
    echo "config.toml created from example — edit before restarting"
  fi
  sudo systemctl restart opcua-howick
  sleep 2
  sudo systemctl status opcua-howick --no-pager
REMOTE

echo ""
echo "=== Deploy complete ==="
echo "OPC UA: opc.tcp://$(echo $PI_HOST | cut -d@ -f2):4840/"
echo "HTTP:   http://$(echo $PI_HOST | cut -d@ -f2):4841/status"
echo ""
echo "Verify with: mise run status:pi"
