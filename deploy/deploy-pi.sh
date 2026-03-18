#!/usr/bin/env bash
# Deploy opcua-howick (full OPC UA + HTTP) to Pi 5.
# Wraps the mise deploy:pi5 task — prefer using that directly:
#
#   PI5_HOST=pi@100.x.x.x mise run deploy:pi5
#
# This script exists for cases where mise is not installed on the deploy host.
#
# Usage: ./deploy/deploy-pi.sh [user@host]
#   Default host: pi@howick-pi5.local

set -euo pipefail

PI5_HOST="${1:-pi@howick-pi5.local}"
TARGET="aarch64-unknown-linux-gnu"
BINARY="opcua-howick"

echo "=== opcua-howick Pi 5 deploy ==="
echo "Target: $PI5_HOST"
echo ""

# Prefer mise
if command -v mise &>/dev/null; then
  PI5_HOST="$PI5_HOST" mise run deploy:pi5
  exit 0
fi

# Fallback: direct cross invocation
if ! command -v cross &>/dev/null; then
  echo "ERROR: neither mise nor cross is installed."
  echo "Install mise: https://mise.jdx.dev"
  echo "Or: cargo install cross --git https://github.com/cross-rs/cross"
  exit 1
fi

echo "Building for Pi 5 (aarch64)..."
cross build --release --bin "$BINARY" --target "$TARGET"

BINARY_PATH="target/$TARGET/release/$BINARY"
echo "Deploying to $PI5_HOST..."
scp "$BINARY_PATH" "$PI5_HOST:~/$BINARY.new"
scp deploy/opcua-howick.service "$PI5_HOST:~/opcua-howick.service"
scp opcua-howick.pi5.toml "$PI5_HOST:~/opcua-howick.pi5.toml"

ssh "$PI5_HOST" << 'REMOTE'
  set -e
  mv ~/opcua-howick.new ~/opcua-howick
  chmod +x ~/opcua-howick
  if [ ! -f /etc/systemd/system/opcua-howick.service ]; then
    sudo mv ~/opcua-howick.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable opcua-howick
    echo "Service installed and enabled"
  fi
  if [ ! -f ~/opcua-howick.pi5.toml ]; then
    echo "opcua-howick.pi5.toml missing — edit it before restarting"
  fi
  sudo systemctl restart opcua-howick
  sleep 2
  sudo systemctl status opcua-howick --no-pager
REMOTE

echo ""
echo "=== Deploy complete ==="
echo "OPC UA: opc.tcp://$(echo $PI5_HOST | cut -d@ -f2):4840/"
echo "HTTP:   http://$(echo $PI5_HOST | cut -d@ -f2):4841/status"
echo ""
echo "Verify with: mise run status:pi5"
