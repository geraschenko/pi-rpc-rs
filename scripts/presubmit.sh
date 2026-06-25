#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/presubmit.sh

Runs checks expected before committing:
  - treefmt --ci
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo nextest run --all-targets --all-features
  - cargo doc --no-deps
USAGE
}

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      usage >&2
      exit 2
      ;;
  esac
done

cd "$(git rev-parse --show-toplevel)"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

run treefmt --ci
run cargo clippy --all-targets --all-features -- -D warnings
run cargo nextest run --all-targets --all-features
run cargo doc --no-deps
