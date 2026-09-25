#!/usr/bin/env bash
# Looks for secrets and local-environment details. See docs/rules/public-repo.md.
# It catches only some leaks; always read the diff too.
#
#   scripts/check-public.sh                  tracked + new (not ignored) files in the working tree
#   scripts/check-public.sh --staged         lines added in the index (run before committing)
#   scripts/check-public.sh --range A..B     lines added and commit messages in a commit range (CI)
#
# A line whose content contains "public-ok" is allowed (explain why in the PR).
# Written for bash 3.2 (macOS default) and POSIX/BSD tools: no mapfile, no \b, no awk intervals.
set -euo pipefail
cd "$(dirname "$0")/.."
git rev-parse --is-inside-work-tree >/dev/null # fail loudly outside a repository

patterns=(
  '/(Users|home)/[A-Za-z0-9._-]+'                                   # macOS / Linux home path
  '[A-Za-z]:\\Users\\'                                              # Windows home path
  '/dev/(cu\.|tty\.|ttyUSB|ttyACM)[A-Za-z0-9]'                      # concrete serial port name
  '(^|[^0-9.])10\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}([^0-9]|$)'    # private IPv4
  '(^|[^0-9.])192\.168\.[0-9]{1,3}\.[0-9]{1,3}([^0-9]|$)'
  '(^|[^0-9.])172\.(1[6-9]|2[0-9]|3[01])\.[0-9]{1,3}\.[0-9]{1,3}([^0-9]|$)'
  '([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}'                           # MAC address
  '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'                  # email address
  '(^|[^A-Za-z0-9])(sk|ts)-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|AKIA[0-9A-Z]{16}'
)
email_pattern='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
allowed_emails='noreply@anthropic\.com|[A-Za-z0-9._-]+@users\.noreply\.github\.com|git@github\.com'

tab=$(printf '\t')
stream=$(mktemp)
scan=$(mktemp)
filtered=$(mktemp)
trap 'rm -f "$stream" "$scan" "$filtered"' EXIT

# Each emitter writes "label<TAB>content" lines.
emit_files() {
  local existing=() f
  while IFS= read -r -d '' f; do
    if [[ -f "$f" ]]; then existing+=("$f"); fi
  done < <(git ls-files -z --cached --others --exclude-standard -- ':!Cargo.lock')
  if [[ ${#existing[@]} -eq 0 ]]; then
    echo "check-public: no files found; is this the repository root?" >&2
    exit 2
  fi
  # -H file name, -n line number, -I skip binary; '' matches every line. Status 2 = error.
  grep -nHI -e '' -- "${existing[@]}" | sed -E "s/^([^:]*:[0-9]+):/\1${tab}/"
}

emit_added_lines() { # $1 label; stdin: unified diff
  { grep -E '^\+' || true; } | { grep -vE '^\+\+\+ ' || true; } | sed -E "s/^\+/$1${tab}/"
}

case "${1:-}" in
  "") emit_files >"$stream" ;;
  --staged) git diff --cached -U0 --no-color | emit_added_lines "staged" >"$stream" ;;
  --range)
    if [[ -z "${2:-}" ]]; then
      echo "usage: $0 --range A..B" >&2
      exit 2
    fi
    {
      git log --no-color --format='%B' "$2" | sed -E "s/^/commit-message${tab}/"
      git log -p -U0 --no-color --format='' "$2" | emit_added_lines "history"
    } >"$stream"
    ;;
  *)
    echo "usage: $0 [--staged | --range A..B]" >&2
    exit 2
    ;;
esac

# Drop lines whose content (after the tab) says public-ok. The label (file name) is not checked.
grep -vE "${tab}.*public-ok" "$stream" >"$scan" || true
# For the email check, remove allowed addresses first so they cannot hide another one on the line.
sed -E "s/${allowed_emails}//g" "$scan" >"$filtered"

found=0
for p in "${patterns[@]}"; do
  input="$scan"
  if [[ "$p" == "$email_pattern" ]]; then input="$filtered"; fi
  status=0
  matches=$(grep -E -- "$p" "$input") || status=$?
  if [[ $status -gt 1 ]]; then
    echo "check-public: grep failed on pattern: $p" >&2
    exit 2
  fi
  if [[ -n "$matches" ]]; then
    echo "$matches"
    found=1
  fi
done

if [[ $found -ne 0 ]]; then
  echo "check-public: possible secrets or local-environment details found (docs/rules/public-repo.md)" >&2
  exit 1
fi
echo "check-public: ok"
