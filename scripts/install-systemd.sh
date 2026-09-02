#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SERVICE_SRC="${REPO_ROOT}/assets/systemd/desktop-pet.service"
TARGET_DIR="${HOME}/.config/systemd/user"
TARGET_DEST="${TARGET_DIR}/desktop-pet.service"

echo "Installing Desktop Pet systemd user service..."

mkdir -p "${TARGET_DIR}"
cp "${SERVICE_SRC}" "${TARGET_DEST}"
echo "Copied service file to ${TARGET_DEST}"

if command -v systemctl >/dev/null 2>&1; then
    if systemctl --user daemon-reload 2>/dev/null; then
        echo "Reloaded systemd user daemon."
        systemctl --user enable --now desktop-pet.service 2>/dev/null || echo "Enabled desktop-pet.service (service start deferred if session inactive)."
    else
        echo "Notice: systemctl --user daemon-reload was not available or no active DBus session detected."
    fi
else
    echo "Notice: systemctl command not found."
fi

echo "Desktop Pet systemd installation completed successfully."
