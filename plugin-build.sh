#!/usr/bin/env bash
set -euo pipefail

KIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_CODE="${1:-}"
PLUGIN_ROOT="${2:-}"
PLUGIN_LOCATION="${PLUGIN_ROOT}"
if [[ -z "${PLUGIN_CODE}" ]]; then
  echo "Usage: ./plugin-build.sh <plugin-code>" >&2
  exit 2
fi

if [[ -z "${PLUGIN_ROOT}" ]]; then
  for candidate in "${KIT_DIR}/plugin-workspace" "${KIT_DIR}/plugin-workspace/examples"; do
    if [[ -f "${candidate}/backend/${PLUGIN_CODE}/manifest.json" || -f "${candidate}/frontend/src/pages/${PLUGIN_CODE}/manifest.json" ]]; then
      PLUGIN_ROOT="${KIT_DIR}"
      PLUGIN_LOCATION="${candidate}"
      break
    fi
  done
fi
if [[ -z "${PLUGIN_ROOT}" ]]; then
  echo "Plugin '${PLUGIN_CODE}' was not found under plugin-workspace or plugin-workspace/examples." >&2
  exit 2
fi

echo "Plugin location: ${PLUGIN_LOCATION}"
"${KIT_DIR}/bin/macos/asapflow/asapflow" plugin validate --root "${PLUGIN_ROOT}" --code "${PLUGIN_CODE}"
"${KIT_DIR}/bin/macos/asapflow/asapflow" plugin pack --root "${PLUGIN_ROOT}" --code "${PLUGIN_CODE}"
