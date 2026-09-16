#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXE_PATH="${SCRIPT_DIR}/asapflow"

if [[ ! -x "${EXE_PATH}" ]]; then
  echo "未找到 asapflow 可执行文件，请将 asapflow.sh 与 asapflow 放在同一目录。" >&2
  exit 1
fi

HAS_OUTPUT=0
for arg in "$@"; do
  if [[ "${arg}" == "--output" ]]; then
    HAS_OUTPUT=1
    break
  fi
done

if [[ ${HAS_OUTPUT} -eq 1 ]]; then
  exec "${EXE_PATH}" "$@"
fi

TMP_OUTPUT="$(mktemp "${TMPDIR:-/tmp}/asapflow-output.XXXXXX.json")"
cleanup() {
  rm -f "${TMP_OUTPUT}"
}
trap cleanup EXIT

"${EXE_PATH}" --output "${TMP_OUTPUT}" "$@"
cat "${TMP_OUTPUT}"
