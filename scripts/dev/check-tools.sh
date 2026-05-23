#!/usr/bin/env bash
set -euo pipefail

required_tools=(
  cargo
  rustfmt
  cargo-clippy
  swift
  swiftlint
  swiftformat
)

missing=()
for tool in "${required_tools[@]}"; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    missing+=("$tool")
  fi
done

if (( ${#missing[@]} > 0 )); then
  printf 'Missing required developer tools:\n' >&2
  printf '  - %s\n' "${missing[@]}" >&2
  exit 1
fi

printf 'All required developer tools are installed.\n'
