#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
RPM=$(find src-tauri/target/release/bundle/rpm -maxdepth 1 -type f -name '*.rpm' | sort | tail -n1)
if [ -z "${RPM:-}" ]; then
  echo "Aucun paquet RPM. Lance d'abord : bash scripts/build_linux.sh" >&2
  exit 1
fi
sudo dnf install -y "$RPM"
echo "Libris est installé dans le menu des applications."
