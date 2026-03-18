#!/usr/bin/env bash
# Deploy howick-agent (minimal binary) to Pi Zero 2W.
# Wraps the mise deploy:pi-zero task — prefer using that directly:
#
#   ZERO_HOST=pi@100.x.x.x mise run deploy:pi-zero
#
# This script exists for cases where mise is not installed on the deploy host.
#
# Usage: ./deploy/deploy-pi-zero.sh [user@host]
#   Default host: pi@howick-pi-zero.local
#
# See docs/customer/06-pi-zero-setup.md for first-time setup (USB gadget mode, Tailscale, etc.)

set -euo pipefail

ZERO_HOST="${1:-pi@howick-pi-zero.local}"
TARGET="aarch64-unknown-linux-gnu"
BINARY="howick-agent"

echo "=== howick-agent Pi Zero 2W deploy ==="
echo "Target: $ZERO_HOST"
echo ""

# Prefer mise
if command -v mise &>/dev/null; then
  ZERO_HOST="$ZERO_HOST" mise run deploy:pi-zero
  exit 0
fi

# Fallback: direct cross invocation
if ! command -v cross &>/dev/null; then
  echo "ERROR: neither mise nor cross is installed."
  echo "Install mise: https://mise.jdx.dev"
  echo "Or: cargo install cross --git https://github.com/cross-rs/cross"
  exit 1
fi

echo "Building howick-agent for Pi Zero 2W (aarch64)..."
cross build --release --bin "$BINARY" --target "$TARGET"

BINARY_PATH="target/$TARGET/release/$BINARY"
echo "Deploying to $ZERO_HOST..."
scp "$BINARY_PATH" "$ZERO_HOST:~/$BINARY.new"
scp deploy/howick-agent.service "$ZERO_HOST:~/howick-agent.service"
scp howick-agent.pi-zero.toml "$ZERO_HOST:~/howick-agent.pi-zero.toml"

ssh "$ZERO_HOST" << 'REMOTE'
  set -e
  mv ~/howick-agent.new ~/howick-agent
  chmod +x ~/howick-agent
  if [ ! -f /etc/systemd/system/howick-agent.service ]; then
    sudo mv ~/howick-agent.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable howick-agent
    echo "Service installed and enabled"
  fi
  if [ ! -f ~/howick-agent.pi-zero.toml ]; then
    echo "howick-agent.pi-zero.toml missing — edit it before restarting"
    echo "Set: machine_input_dir = /mnt/usb_share"
  fi
  sudo systemctl restart howick-agent
  sleep 2
  sudo systemctl status howick-agent --no-pager
REMOTE

echo ""
echo "=== Deploy complete ==="
echo "Logs: mise run logs:pi-zero"
echo "Status: mise run status:pi-zero"
