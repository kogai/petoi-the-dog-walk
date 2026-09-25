#!/usr/bin/env bash
# Runs every check that CI runs. See docs/rules/static-analysis.md.
set -euo pipefail
cd "$(dirname "$0")/.."

run() {
  echo "==> $*"
  "$@"
}

run scripts/check-public.sh
run uv lock --check
run uv run --frozen ruff format --check
run uv run --frozen ruff check
run uv run --frozen mypy
HYPOTHESIS_PROFILE=ci run uv run --frozen pytest --cov "$@"
