#!/usr/bin/env bash
# Run cargo for this machine's native CPU, even when the Rust toolchain itself runs under
# emulation (a Rosetta-installed rustup defaults to x86_64 and yields slower, larger-RSS binaries).
set -euo pipefail

host_os="$(uname -s)"
if [[ "$host_os" == Darwin ]]; then
  # `uname -m` reports x86_64 for any process running under Rosetta; the sysctl does not.
  if [[ "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" == 1 ]]; then
    triple=aarch64-apple-darwin
  else
    triple=x86_64-apple-darwin
  fi
else
  case "$(uname -m)" in
    x86_64) triple=x86_64-unknown-linux-gnu ;;
    aarch64 | arm64) triple=aarch64-unknown-linux-gnu ;;
    *) echo "unsupported CPU: $(uname -m)" >&2; exit 1 ;;
  esac
fi

rustup target add "$triple" >/dev/null
exec cargo "$@" --target "$triple"
