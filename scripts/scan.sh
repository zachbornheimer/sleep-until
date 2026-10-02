#!/usr/bin/env bash
# Scan the working tree for secrets, then prove the scanner can fail.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
probe="$(mktemp -d)"
trap 'rm -rf "$probe"' EXIT

gitleaks dir "$root" --no-banner --redact --exit-code 1

# Synthetic finding: the scanner must reject it, or a green scan means nothing.
printf 'token = "ghp_%s"\n' "R8sT2uVwXyZ4aBcD6eFgH8iJkL0mNoPqRs1T" >"$probe/synthetic.env"
if gitleaks dir "$probe" --no-banner --redact --exit-code 1 >/dev/null 2>&1; then
  echo "scan self-test failed: gitleaks missed a synthetic secret" >&2
  exit 1
fi
echo "scan ok (self-test caught the synthetic secret)"
