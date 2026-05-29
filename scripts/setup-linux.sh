#!/usr/bin/env bash
# Installs the system libraries required to build PromptFlow STT (Tauri v2) on
# Debian/Ubuntu. Tauri needs WebKitGTK; the `cpal` audio crate needs ALSA.
# Safe to re-run.
set -euo pipefail

if ! command -v apt-get >/dev/null 2>&1; then
  echo "This script targets Debian/Ubuntu (apt-get). For other distros see"
  echo "https://tauri.app/start/prerequisites/"
  exit 1
fi

sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev \
  build-essential \
  curl \
  wget \
  file

echo "✓ System dependencies installed. Next: 'npm install' then 'npm run tauri dev'."
