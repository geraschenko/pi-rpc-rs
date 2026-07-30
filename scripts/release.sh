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

VERSION=$(toml get Cargo.toml package.version --raw)

TEST_AGENT_DIR=""
cleanup() {
  if [[ -n "$TEST_AGENT_DIR" ]]; then
    rm -rf "$TEST_AGENT_DIR"
    TEST_AGENT_DIR=""
  fi
}
trap cleanup EXIT

prepare_test_agent_dir() {
  local source_agent_dir="${PI_CODING_AGENT_DIR:-$HOME/.pi/agent}"
  local source_auth_path="$source_agent_dir/auth.json"
  local expires_ms
  local minimum_expires_ms=$(( $(date +%s) * 1000 + 60 * 60 * 1000 ))

  if ! expires_ms=$(jq -er '."openai-codex" | select(.type == "oauth") | .expires' "$source_auth_path"); then
    echo "no openai-codex OAuth credential found in $source_auth_path" >&2
    return 1
  fi
  if [[ ! "$expires_ms" =~ ^[0-9]+$ ]] || (( expires_ms < minimum_expires_ms )); then
    echo "openai-codex OAuth credential must remain valid for at least one hour" >&2
    echo "refresh it before running the release; this script will not refresh it" >&2
    return 1
  fi

  TEST_AGENT_DIR=$(mktemp -d)
  install -m 600 /dev/null "$TEST_AGENT_DIR/auth.json"
  jq '{"openai-codex": .["openai-codex"]}' "$source_auth_path" > "$TEST_AGENT_DIR/auth.json"
}

run scripts/presubmit.sh
prepare_test_agent_dir
run env PI_CODING_AGENT_DIR="$TEST_AGENT_DIR" cargo nextest run --all-targets --all-features --run-ignored all --test-threads 1
cleanup
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
