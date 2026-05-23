#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
Usage: scripts/dev/resources.sh <rust-debug-binary> <rust-release-binary> <swift-staged-binary> <swift-staged-profile>
USAGE
}

rust_debug_binary="${1:-}"
rust_release_binary="${2:-}"
swift_staged_binary="${3:-}"
swift_staged_profile="${4:-}"

if [[ -z "$rust_debug_binary" || -z "$rust_release_binary" || -z "$swift_staged_binary" || -z "$swift_staged_profile" ]]; then
  usage
  exit 2
fi

ps -axo pid=,ppid=,%cpu=,%mem=,rss=,vsz=,etime=,command= | awk \
  -v rust_debug_binary="$rust_debug_binary" \
  -v rust_release_binary="$rust_release_binary" \
  -v swift_staged_binary="$swift_staged_binary" \
  -v swift_staged_profile="$swift_staged_profile" '
function human_kb(kb) {
    kb += 0
    if (kb >= 1048576) return sprintf("%.1f GB", kb / 1048576)
    if (kb >= 1024) return sprintf("%.1f MB", kb / 1024)
    return sprintf("%d KB", kb)
}
function is_rust_core(command) {
    return index(command, rust_debug_binary) || index(command, rust_release_binary)
}
function is_swift_client(command) {
    return index(command, swift_staged_binary)
}
function process_name(command) {
    if (is_swift_client(command)) return "Swift Darwin"
    if (is_rust_core(command)) return "Rust core"
    return "Launcher"
}
function build_profile(command,   profile) {
    if (index(command, rust_release_binary)) return "release"
    if (index(command, rust_debug_binary)) return "debug"
    if (is_swift_client(command)) {
        if ((getline profile < swift_staged_profile) > 0) {
            close(swift_staged_profile)
            return profile
        }
    }
    return "unknown"
}
BEGIN {
    border = "+--------------+----------+---------+---------+---------+-----------+-----------+-----------+"
    print border
    printf "| %-12s | %-8s | %7s | %7s | %7s | %9s | %9s | %9s |\n", "Process", "Build", "PID", "CPU", "Mem", "Real Mem", "Virtual", "Running"
    print border
}
{
    command = ""
    for (i = 8; i <= NF; i++) command = command (i == 8 ? "" : " ") $i
    if (command ~ /scripts\/dev\/resources[.]sh/ || command ~ / awk /) next
    if (!is_rust_core(command) && !is_swift_client(command)) next

    pid = $1
    cpu = $3 "%"
    mem = $4 "%"
    rss = human_kb($5)
    virtual = human_kb($6)
    elapsed = $7

    printf "| %-12s | %-8s | %7s | %7s | %7s | %9s | %9s | %9s |\n", process_name(command), build_profile(command), pid, cpu, mem, rss, virtual, elapsed
    count++
}
END {
    if (count == 0) printf "| %-91s |\n", "No LaunchKick processes are running."
    print border
}'
