#!/usr/bin/env bash
set -euo pipefail
KIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
case "$(uname -s):$(uname -m)" in
  Darwin:arm64) CLI="${KIT_DIR}/bin/macos/asapflow/asapflow" ;;
  Linux:x86_64) CLI="${KIT_DIR}/bin/linux-x86_64/asapflow/asapflow" ;;
  *) echo "Unsupported platform. Supported: macOS Apple Silicon, Linux x86_64." >&2; exit 1 ;;
esac
exec "${CLI}" "$@"
