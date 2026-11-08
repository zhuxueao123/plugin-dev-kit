#!/usr/bin/env bash
set -euo pipefail
KIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_CODE="${1:?Usage: plugin-build.sh <code> [project-root]}"
shift
PLUGIN_ROOT="$PWD"
if [[ "${1:-}" == "--root" ]]; then
  [[ $# -eq 2 ]] || { echo "--root requires one project directory" >&2; exit 2; }
  PLUGIN_ROOT="$2"
elif [[ $# -eq 1 ]]; then
  PLUGIN_ROOT="$1"
elif [[ $# -ne 0 ]]; then
  echo "Usage: plugin-build.sh <code> [--root <project-directory>]" >&2
  exit 2
fi
"${KIT_DIR}/asapflow.sh" plugin validate --root "${PLUGIN_ROOT}" --code "${PLUGIN_CODE}"
"${KIT_DIR}/asapflow.sh" plugin pack --root "${PLUGIN_ROOT}" --code "${PLUGIN_CODE}"
