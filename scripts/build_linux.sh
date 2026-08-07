#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi
[ -d node_modules ] || npm install
npm run check
npm run release:linux

echo
echo "Paquets générés dans :"
echo "  src-tauri/target/release/bundle/rpm/"
echo "  src-tauri/target/release/bundle/deb/"
