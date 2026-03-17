#!/usr/bin/env bash
# Auto-update opcua-howick on Pi 5 from GitHub Releases.
# Run by systemd timer (opcua-howick-update.timer) every hour.
# Also callable manually: sudo /usr/local/bin/opcua-howick-update.sh

set -euo pipefail

REPO="joeblew999/opcua-howick"
BINARY="opcua-howick"
TARGET="aarch64-unknown-linux-gnu"
INSTALL_PATH="/home/pi/opcua-howick"
SERVICE="opcua-howick"
VERSION_FILE="/home/pi/.opcua-howick-version"

echo "[opcua-howick-update] checking for updates..."

# Get latest release tag from GitHub
LATEST=$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
  echo "[opcua-howick-update] could not reach GitHub — skipping"
  exit 0
fi

CURRENT=$(cat "$VERSION_FILE" 2>/dev/null || echo "none")

if [ "$LATEST" = "$CURRENT" ]; then
  echo "[opcua-howick-update] already at $LATEST — nothing to do"
  exit 0
fi

echo "[opcua-howick-update] updating $CURRENT → $LATEST"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TARGET}"

curl -fsSL "$DOWNLOAD_URL" -o "${INSTALL_PATH}.new"
chmod +x "${INSTALL_PATH}.new"
mv "${INSTALL_PATH}.new" "${INSTALL_PATH}"

echo "$LATEST" > "$VERSION_FILE"

systemctl restart "$SERVICE"
echo "[opcua-howick-update] updated to $LATEST and restarted $SERVICE"
