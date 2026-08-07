#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/setup_fedora.sh
bash scripts/build_linux.sh
bash scripts/install_rpm.sh
