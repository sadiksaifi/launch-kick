#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
Usage: scripts/dev/stage-swift-client.sh <debug|release> <package-path> <product> <scratch-path> <staged-binary> <staged-profile> [--if-needed]
USAGE
}

profile="${1:-}"
swift_package="${2:-}"
swift_product="${3:-}"
swift_build_scratch="${4:-}"
staged_binary="${5:-}"
staged_profile="${6:-}"
mode="${7:-}"

if [[ "$profile" != "debug" && "$profile" != "release" ]]; then
  usage
  exit 2
fi

if [[ -z "$swift_package" || -z "$swift_product" || -z "$swift_build_scratch" || -z "$staged_binary" || -z "$staged_profile" ]]; then
  usage
  exit 2
fi

if [[ -n "$mode" && "$mode" != "--if-needed" ]]; then
  usage
  exit 2
fi

binary_configuration="debug"

if [[ "$profile" == "release" ]]; then
  binary_configuration="release"
fi

needs_stage() {
  [[ ! -x "$staged_binary" ]] && return 0
  [[ ! -f "$staged_profile" ]] && return 0
  [[ "$(cat "$staged_profile")" != "$profile" ]] && return 0
  [[ "$swift_package/Package.swift" -nt "$staged_binary" ]] && return 0
  find "$swift_package/Sources" -name '*.swift' -newer "$staged_binary" | grep -q . && return 0
  return 1
}

mkdir -p "$(dirname "$staged_binary")"

if [[ "$mode" == "--if-needed" ]] && ! needs_stage; then
  exit 0
fi

if [[ "$profile" == "release" ]]; then
  swift build --configuration release \
    --package-path "$swift_package" \
    --scratch-path "$swift_build_scratch" \
    --product "$swift_product"
else
  swift build \
    --package-path "$swift_package" \
    --scratch-path "$swift_build_scratch" \
    --product "$swift_product"
fi

cp "$swift_build_scratch/$binary_configuration/$swift_product" "$staged_binary"
printf '%s\n' "$profile" > "$staged_profile"
