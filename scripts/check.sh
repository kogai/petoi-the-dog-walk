#!/usr/bin/env bash
# Runs every check that CI runs. See docs/rules/static-analysis.md.
# Needs: the toolchain in rust-toolchain.toml (rustup installs it) and cargo-llvm-cov
# (`cargo install cargo-llvm-cov --version 0.9.1 --locked`). Takes no arguments.
set -euo pipefail
cd "$(dirname "$0")/.."

run() {
  echo "==> $*"
  "$@"
}

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "cargo-llvm-cov is missing: cargo install cargo-llvm-cov --version 0.9.1 --locked" >&2
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
