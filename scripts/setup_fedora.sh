#!/usr/bin/env bash
set -euo pipefail

sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  libxdo-devel \
  gcc \
  gcc-c++ \
  make

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js n’est pas installé. Installe Node.js LTS puis relance ce script." >&2
  exit 1
fi

npm install

echo
echo "Prérequis installés. Lance maintenant :"
echo "  npm run tauri dev"
