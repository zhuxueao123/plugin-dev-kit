#!/usr/bin/env bash
set -euo pipefail

KIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_CODE="${1:-}"
if [[ -z "${PLUGIN_CODE}" ]]; then
  echo "Usage: ./plugin-build.sh <plugin-code> [--root <plugin-repository>]" >&2
  exit 2
fi
shift

PLUGIN_ROOT="${ASAPFLOW_PLUGIN_ROOT:-}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      [[ $# -ge 2 ]] || { echo "--root requires a directory." >&2; exit 2; }
      PLUGIN_ROOT="$2"
      shift 2
      ;;
    *)
      # Keep the former second positional argument compatible.
      if [[ -z "${PLUGIN_ROOT}" ]]; then
        PLUGIN_ROOT="$1"
        shift
      else
        echo "Unknown argument: $1" >&2
        exit 2
      fi
      ;;
  esac
done

if [[ -z "${PLUGIN_ROOT}" ]]; then
  for candidate in "${KIT_DIR}/plugin-workspace" "${KIT_DIR}/plugin-workspace/examples"; do
    if [[ -f "${candidate}/backend/${PLUGIN_CODE}/manifest.json" || -f "${candidate}/frontend/src/pages/${PLUGIN_CODE}/manifest.json" ]]; then
      PLUGIN_ROOT="${KIT_DIR}"
      PLUGIN_LOCATION="${candidate}"
      break
    fi
  done
else
  [[ -d "${PLUGIN_ROOT}" ]] || { echo "Plugin repository does not exist: ${PLUGIN_ROOT}" >&2; exit 2; }
  PLUGIN_ROOT="$(cd "${PLUGIN_ROOT}" && pwd)"
  PLUGIN_LOCATION="${PLUGIN_ROOT}"
fi
if [[ -z "${PLUGIN_ROOT}" ]]; then
  echo "Plugin '${PLUGIN_CODE}' was not found. Set ASAPFLOW_PLUGIN_ROOT or pass --root <plugin-repository>." >&2
  exit 2
fi

echo "Plugin repository: ${PLUGIN_LOCATION}"
"${KIT_DIR}/asapflow.sh" plugin validate --root "${PLUGIN_ROOT}" --code "${PLUGIN_CODE}"
"${KIT_DIR}/asapflow.sh" plugin pack --root "${PLUGIN_ROOT}" --code "${PLUGIN_CODE}"
