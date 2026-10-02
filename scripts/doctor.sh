#!/usr/bin/env bash
# Report every missing prerequisite, then fail if any were missing.
set -euo pipefail

missing=0
for tool in cargo rustc rustup gitleaks lefthook; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "missing: $tool (run: mise install)" >&2
    missing=1
  fi
done
install_bin="${SLP_INSTALL_ROOT:-$HOME/.local}/bin"
case ":$PATH:" in
  *":$install_bin:"*) ;;
  *) echo "warning: $install_bin is not on PATH; slp will not be found" >&2 ;;
esac
((missing == 0)) && echo "slp environment ok"
exit "$missing"
