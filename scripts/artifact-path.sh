#!/usr/bin/env bash
# Print the path of the native release binary built by `mise run build`.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$(ls -t "$root"/target/*/release/slp 2>/dev/null | head -n 1)"
[[ -x "$binary" ]] || { echo "no release binary; run: mise run build" >&2; exit 1; }
echo "$binary"
