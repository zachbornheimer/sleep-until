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
if command -v rustc >/dev/null 2>&1; then
  host="$(rustc -vV | sed -n 's/^host: //p')"
  if [[ "$(uname -s)" == Darwin && "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" == 1 && "$host" != aarch64-* ]]; then
    echo "rustc targets $host on an Apple Silicon Mac (emulated; roughly doubles memory). Install a native rustup (brew install rustup) and run: rustup set default-host aarch64-apple-darwin" >&2
    missing=1
  fi
fi
install_bin="${SLP_INSTALL_ROOT:-$HOME/.local}/bin"
case ":$PATH:" in
  *":$install_bin:"*) ;;
  *) echo "warning: $install_bin is not on PATH; slp will not be found" >&2 ;;
esac
if ((missing == 0)); then
  echo "slp environment ok"
fi
exit "$missing"
