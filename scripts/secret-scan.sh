#!/usr/bin/env bash
set -euo pipefail

# Conservative repository scan. Documentation and test fixtures may contain
# deliberately fake examples; implementation/configuration files must not.
patterns='(AKIA[0-9A-Z]{16}|-----BEGIN (RSA|OPENSSH|EC|DSA) PRIVATE KEY-----|(^|[[:space:]])(api[_-]?key|secret|token|password)[[:space:]]*[:=][[:space:]]*["'"'][^"'"']{8,}["'"'])'

matches="$(git grep -nIE "$patterns" -- \
  ':!docs/**' ':!tests/**' ':!**/*.md' ':!**/*.json' ':!**/*.lock' \
  ':!**/locales/**' || true)"

if [[ -n "$matches" ]]; then
  printf '%s\n' "$matches" >&2
  echo "secret-scan: potential credential material found" >&2
  exit 1
fi

echo "secret-scan: no credential patterns found"
