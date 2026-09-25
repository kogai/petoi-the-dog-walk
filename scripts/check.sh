#!/usr/bin/env bash
# Runs every check that CI runs. See docs/rules/static-analysis.md.
set -euo pipefail
cd "$(dirname "$0")/.."

run() {
  echo "==> $*"
  "$@"
}

run scripts/check-public.sh
run pnpm install --frozen-lockfile --silent
run pnpm exec biome format .
run pnpm exec eslint .
run pnpm exec tsc
run pnpm exec depcruise src scripts --config .dependency-cruiser.cjs
run pnpm exec vitest run --project default --coverage "$@"
