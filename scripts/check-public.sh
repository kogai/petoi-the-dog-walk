#!/usr/bin/env bash
# Fails if tracked files contain local-environment details. See docs/rules/public-repo.md.
# A line containing "public-ok" is allowed (explain why in the PR).
set -euo pipefail
cd "$(dirname "$0")/.."

patterns=(
  '/Users/[A-Za-z0-9._-]+'                                   # macOS home path
  '/home/[A-Za-z0-9._-]+/'                                   # Linux home path
  '/dev/(cu|tty)\.[A-Za-z0-9]'                               # concrete serial port name
  '\b(10|127)\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\b'         # IPv4 (private / loopback)
  '\b192\.168\.[0-9]{1,3}\.[0-9]{1,3}\b'
  '\b172\.(1[6-9]|2[0-9]|3[01])\.[0-9]{1,3}\.[0-9]{1,3}\b'
  '\b([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}\b'                   # MAC address
  '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'           # email address
  '(sk|ts)-[A-Za-z0-9]{20,}'                                 # API-key-like token
)
allowed_emails='noreply@anthropic\.com|@users\.noreply\.github\.com'

mapfile -d '' files < <(git ls-files -z --cached --others --exclude-standard -- ':!pnpm-lock.yaml')
existing=()
for f in "${files[@]}"; do [[ -f "$f" ]] && existing+=("$f"); done

found=0
for p in "${patterns[@]}"; do
  # Decide on the output, not the exit status: grep exits 1 when a batch has no match.
  matches=$(printf '%s\0' "${existing[@]}" | xargs -0 grep -nIE -- "$p" 2>/dev/null \
    | grep -v 'public-ok' | grep -vE "$allowed_emails" || true)
  if [[ -n "$matches" ]]; then
    echo "$matches"
    found=1
  fi
done

if [[ $found -ne 0 ]]; then
  echo "check-public: local-environment details found (docs/rules/public-repo.md)" >&2
  exit 1
fi
echo "check-public: ok"
