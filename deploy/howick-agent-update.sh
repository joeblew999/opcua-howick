#!/usr/bin/env bash
# Auto-update howick-agent on Pi Zero 2W from GitHub Releases.
# Run by systemd timer (howick-agent-update.timer) every hour.
# Also callable manually: sudo /usr/local/bin/howick-agent-update.sh

set -euo pipefail

REPO="joeblew999/opcua-howick"
BINARY="howick-agent"
TARGET="aarch64-unknown-linux-gnu"
INSTALL_PATH="/home/pi/howick-agent"
SERVICE="howick-agent"
VERSION_FILE="/home/pi/.howick-agent-version"

echo "[howick-agent-update] checking for updates..."

# Get latest release tag from GitHub
LATEST=$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
  echo "[howick-agent-update] could not reach GitHub — skipping"
  exit 0
fi

CURRENT=$(cat "$VERSION_FILE" 2>/dev/null || echo "none")

if [ "$LATEST" = "$CURRENT" ]; then
  echo "[howick-agent-update] already at $LATEST — nothing to do"
  exit 0
fi

echo "[howick-agent-update] updating $CURRENT → $LATEST"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TARGET}"

curl -fsSL "$DOWNLOAD_URL" -o "${INSTALL_PATH}.new"
chmod +x "${INSTALL_PATH}.new"
mv "${INSTALL_PATH}.new" "${INSTALL_PATH}"

echo "$LATEST" > "$VERSION_FILE"

systemctl restart "$SERVICE"
echo "[howick-agent-update] updated to $LATEST and restarted $SERVICE"
