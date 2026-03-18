#!/usr/bin/env bash
# Deploy howick-frama (minimal binary) to Pi Zero 2W.
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
BINARY="howick-frama"

echo "=== howick-frama Pi Zero 2W deploy ==="
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

echo "Building howick-frama for Pi Zero 2W (aarch64)..."
cross build --release --bin "$BINARY" --target "$TARGET"

BINARY_PATH="target/$TARGET/release/$BINARY"
echo "Deploying to $ZERO_HOST..."
scp "$BINARY_PATH" "$ZERO_HOST:~/$BINARY.new"
scp deploy/howick-frama.service "$ZERO_HOST:~/howick-frama.service"
scp howick-frama.pi-zero.toml "$ZERO_HOST:~/howick-frama.pi-zero.toml"

ssh "$ZERO_HOST" << 'REMOTE'
  set -e
  mv ~/howick-frama.new ~/howick-frama
  chmod +x ~/howick-frama
  if [ ! -f /etc/systemd/system/howick-frama.service ]; then
    sudo mv ~/howick-frama.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable howick-frama
    echo "Service installed and enabled"
  fi
  if [ ! -f ~/howick-frama.pi-zero.toml ]; then
    echo "howick-frama.pi-zero.toml missing — edit it before restarting"
    echo "Set: machine_input_dir = /mnt/usb_share"
  fi
  sudo systemctl restart howick-frama
  sleep 2
  sudo systemctl status howick-frama --no-pager
REMOTE

echo ""
echo "=== Deploy complete ==="
echo "Logs: mise run logs:pi-zero"
echo "Status: mise run status:pi-zero"
