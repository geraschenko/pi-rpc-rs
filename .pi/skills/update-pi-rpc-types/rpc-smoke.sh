#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <setup|before|after|compare> <old-version> <new-version>" >&2
  exit 2
}

[[ $# -eq 3 ]] || usage
phase=$1
old_version=${2#v}
new_version=${3#v}
case "$phase" in
  setup|before|after|compare) ;;
  *) usage ;;
esac
[[ "$old_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || usage
[[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || usage

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

smoke_dir="/tmp/pi-rpc-update-${old_version}-${new_version}"
old_pi_binary="$smoke_dir/pi-old"
new_pi_binary="$smoke_dir/pi-new"

write_pi_wrapper() {
  local output_path=$1
  local version=$2
  cat >"$output_path" <<EOF
#!/bin/sh
exec npx --yes --package=@earendil-works/pi-coding-agent@${version} -- pi "\$@"
EOF
  chmod +x "$output_path"
}

verify_pi_wrapper() {
  local pi_binary=$1
  local expected_version=$2
  local actual_version
  actual_version=$("$pi_binary" --version)
  if [[ "$actual_version" != "$expected_version" ]]; then
    echo "Expected $pi_binary to report $expected_version, got $actual_version" >&2
    return 1
  fi
}

setup_smoke() {
  mkdir -p "$smoke_dir"
  write_pi_wrapper "$old_pi_binary" "$old_version"
  write_pi_wrapper "$new_pi_binary" "$new_version"
  verify_pi_wrapper "$old_pi_binary" "$old_version"
  verify_pi_wrapper "$new_pi_binary" "$new_version"
}

ensure_setup() {
  if [[ ! -x "$old_pi_binary" || ! -x "$new_pi_binary" ]]; then
    setup_smoke
    return
  fi
  verify_pi_wrapper "$old_pi_binary" "$old_version"
  verify_pi_wrapper "$new_pi_binary" "$new_version"
}

capture() {
  local capture_phase=$1
  local pi_binary=$2
  local output_prefix="$smoke_dir/$capture_phase"

  printf 'old_pi=%s\nnew_pi=%s\ncrate=%s\ncommit=%s\n' \
    "$old_version" \
    "$new_version" \
    "$(awk -F '"' '/^version = / { print $2; exit }' Cargo.toml)" \
    "$(git rev-parse HEAD)" \
    >"$output_prefix.metadata"

  cargo run --quiet --bin pi-rpc-debug -- \
    --pi-binary "$pi_binary" \
    --jsonl \
    --prompt "Use the bash tool to run ls, then stop" \
    >"$output_prefix.raw.jsonl" \
    2>"$output_prefix.stderr"

  jq -e . "$output_prefix.raw.jsonl" >/dev/null
}

normalize_capture() {
  local input_path=$1
  local output_path=$2

  jq -c '
    def field_paths:
      [paths(scalars)
       | map(if type == "number" then "[]" else . end)
       | join(".")]
      | unique;
    if .record == "event" then
      {
        record,
        classification,
        eventType: .value.type,
        messageRole: .value.message.role?,
        assistantEventType: .value.assistantMessageEvent.type?,
        assistantEventReason: .value.assistantMessageEvent.reason?,
        toolName: (.value.toolName? // .value.assistantMessageEvent.toolName?),
        stopReason: (.value.stopReason? // .value.message.stopReason?),
        error: (.value.error? // .value.errorMessage? // .value.message.errorMessage?),
        fields: (.value | field_paths)
      }
    else
      {record, fields: (.value | field_paths)}
    end
  ' "$input_path" \
    | uniq \
    >"$output_path"
}

compare_captures() {
  local before_raw="$smoke_dir/before.raw.jsonl"
  local after_raw="$smoke_dir/after.raw.jsonl"
  local before_normalized="$smoke_dir/before.normalized.jsonl"
  local after_normalized="$smoke_dir/after.normalized.jsonl"

  if [[ ! -f "$before_raw" || ! -f "$after_raw" ]]; then
    echo "Both $before_raw and $after_raw are required" >&2
    return 1
  fi

  jq -e . "$before_raw" >/dev/null
  jq -e . "$after_raw" >/dev/null
  normalize_capture "$before_raw" "$before_normalized"
  normalize_capture "$after_raw" "$after_normalized"

  diff -u \
    <(jq . "$before_normalized") \
    <(jq . "$after_normalized") \
    >"$smoke_dir/normalized.diff" \
    || true
  cat "$smoke_dir/normalized.diff"
}

case "$phase" in
  setup)
    setup_smoke
    ;;
  before)
    ensure_setup
    capture before "$old_pi_binary"
    ;;
  after)
    ensure_setup
    capture after "$new_pi_binary"
    ;;
  compare)
    compare_captures
    ;;
esac

printf 'Smoke artifacts: %s\n' "$smoke_dir" >&2
