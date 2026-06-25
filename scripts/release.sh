#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/release.sh

Runs release checks, publishes the crate to crates.io, and tags the release.
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

VERSION=$(
  python3 - <<'PY'
import tomllib
from pathlib import Path

with Path("Cargo.toml").open("rb") as f:
    print(tomllib.load(f)["package"]["version"])
PY
)

run scripts/presubmit.sh
run cargo nextest run --all-targets --all-features --run-ignored all
run cargo package

printf '\n==> cargo package --list\n'
cargo package --list

printf '\nPublish pi-rpc-rs %s with the packaged files listed above? [y/N] ' "$VERSION"
read -r answer
case "$answer" in
  y|Y|yes|YES)
    ;;
  *)
    echo "aborting release"
    exit 1
    ;;
esac

run cargo publish
run git tag "v$VERSION"
run git push origin "v$VERSION"
