#!/usr/bin/env bash
# Runs every check that CI runs. See docs/rules/static-analysis.md.
# Tools come from mise.toml: run `mise install`, then `mise run check` (or this script inside a
# shell where mise is activated). Takes no arguments.
set -euo pipefail
cd "$(dirname "$0")/.."

run() {
  echo "==> $*"
  "$@"
}

# The tools must be the versions pinned in mise.toml, so local results match CI.
pinned() { sed -nE "s/^$1 = .*version = \"([^\"]+)\".*/\1/p; s/^\"?$1\"? = \"([^\"]+)\"/\1/p" mise.toml | head -n 1; }
want_rust=$(pinned rust)
want_cov=$(pinned 'cargo:cargo-llvm-cov')
have_rust=$(rustc --version 2>/dev/null | awk '{print $2}' || true)
have_cov=$(cargo llvm-cov --version 2>/dev/null | awk '{print $2}' || true)
if [[ -z "$want_rust" || -z "$want_cov" ]]; then
  echo "check: cannot read tool versions from mise.toml" >&2
  exit 1
fi
if [[ "$have_rust" != "$want_rust" || "$have_cov" != "$want_cov" ]]; then
  echo "check: need rustc $want_rust and cargo-llvm-cov $want_cov (mise.toml);" \
    "found rustc ${have_rust:-none} and cargo-llvm-cov ${have_cov:-none}." >&2
  echo "check: run 'mise install' and then 'mise run check'." >&2
  exit 1
fi

# CI also runs this in public-check.yml; it is repeated here so that one local command covers all.
run scripts/check-public.sh
run cargo fmt --all --check
run cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" run cargo doc --locked --workspace --no-deps
run cargo xtask deps
run cargo xtask profiles
run cargo xtask lints
# Coverage floor applies to product crates; tooling (xtask, walk-experiments) is tested but not counted.
run cargo test --locked --package xtask --package walk-experiments
# Runs the default test suite (no live-* features) and enforces the coverage floor.
run cargo llvm-cov --locked --workspace --exclude xtask --exclude walk-experiments --fail-under-lines 80
