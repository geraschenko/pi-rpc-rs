#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/presubmit.sh [--release]

Runs checks expected before committing:
  - treefmt --ci
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo nextest run --all-targets --all-features

With --release, also runs:
  - cargo package
USAGE
}

release=false
for arg in "$@"; do
  case "$arg" in
    --release)
      release=true
      ;;
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

if [[ "$release" == true ]]; then
  run cargo package
fi
