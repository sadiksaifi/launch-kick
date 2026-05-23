ui_package := "client/darwin"
ui_product := "launch-kick"
ui_binary := ".build/launch-kick"
ui_profile_file := ".build/launch-kick.profile"
core_manifest := "core/Cargo.toml"
core_release_binary := "core/target/release/launchkick-core"

# List available commands.
default:
    just --list

# Build everything for development.
build: build-core build-ui

# Build everything with release optimizations for production-like local runs.
build-release: build-core-release build-ui-release

# Run the launcher prototype in development mode.
run: ensure-ui
    cargo run --manifest-path {{core_manifest}}

# Run the launcher prototype with release optimizations.
run-release: build-release
    {{core_release_binary}}

# Check that Rust and Swift compile.
check:
    cargo check --manifest-path {{core_manifest}}
    swift build --package-path {{ui_package}} --scratch-path .build/swift --product {{ui_product}}

# Run Rust tests.
test:
    cargo test --manifest-path {{core_manifest}}

# Format Rust code.
fmt:
    cargo fmt --manifest-path {{core_manifest}}

# Build the Rust core for development.
build-core:
    cargo build --manifest-path {{core_manifest}}

# Build the Rust core with release optimizations.
build-core-release:
    cargo build --release --manifest-path {{core_manifest}}

# Build the Darwin UI for development and copy it to the path used by the core.
build-ui:
    mkdir -p .build
    swift build --package-path {{ui_package}} --scratch-path .build/swift --product {{ui_product}}
    cp .build/swift/debug/{{ui_product}} {{ui_binary}}
    printf 'debug\n' > {{ui_profile_file}}

# Build the Darwin UI with release optimizations and copy it to the path used by the core.
build-ui-release:
    mkdir -p .build
    swift build --configuration release --package-path {{ui_package}} --scratch-path .build/swift --product {{ui_product}}
    cp .build/swift/release/{{ui_product}} {{ui_binary}}
    printf 'release\n' > {{ui_profile_file}}

# Rebuild the development UI only when the copied binary is missing, stale, or from another profile.
ensure-ui:
    mkdir -p .build
    if [ ! -x {{ui_binary}} ] || [ ! -f {{ui_profile_file}} ] || [ "$(cat {{ui_profile_file}})" != "debug" ] || [ {{ui_package}}/Package.swift -nt {{ui_binary}} ] || find {{ui_package}}/Sources -name '*.swift' -newer {{ui_binary}} | grep -q .; then just build-ui; fi

# Rebuild the release UI only when the copied binary is missing, stale, or from another profile.
ensure-ui-release:
    mkdir -p .build
    if [ ! -x {{ui_binary}} ] || [ ! -f {{ui_profile_file}} ] || [ "$(cat {{ui_profile_file}})" != "release" ] || [ {{ui_package}}/Package.swift -nt {{ui_binary}} ] || find {{ui_package}}/Sources -name '*.swift' -newer {{ui_binary}} | grep -q .; then just build-ui-release; fi

# Show current launcher process resource usage in a human-readable table.
resources:
    #!/usr/bin/env bash
    set -euo pipefail
    ps -axo pid=,ppid=,%cpu=,%mem=,rss=,vsz=,etime=,command= | awk '
    function human_kb(kb) {
        kb += 0
        if (kb >= 1048576) return sprintf("%.1f GB", kb / 1048576)
        if (kb >= 1024) return sprintf("%.1f MB", kb / 1024)
        return sprintf("%d KB", kb)
    }
    function process_name(command) {
        if (command ~ /[.]build\/launch-kick/) return "macOS UI"
        if (command ~ /launchkick-core/) return "Rust core"
        return "Launcher"
    }
    function build_profile(command,   profile) {
        if (command ~ /target\/release\/launchkick-core/) return "release"
        if (command ~ /target\/debug\/launchkick-core/) return "debug"
        if (command ~ /[.]build\/launch-kick/) {
            if ((getline profile < ".build/launch-kick.profile") > 0) {
                close(".build/launch-kick.profile")
                return profile
            }
        }
        return "unknown"
    }
    BEGIN {
        border = "+------------+----------+---------+---------+---------+-----------+-----------+-----------+"
        print border
        printf "| %-10s | %-8s | %7s | %7s | %7s | %9s | %9s | %9s |\n", "Process", "Build", "PID", "CPU", "Mem", "Real Mem", "Virtual", "Running"
        print border
    }
    {
        command = ""
        for (i = 8; i <= NF; i++) command = command (i == 8 ? "" : " ") $i
        if (command !~ /(^|\/)launchkick-core( |$)/ && command !~ /\/[.]build\/launch-kick( |$)/) next

        pid = $1
        cpu = $3 "%"
        mem = $4 "%"
        rss = human_kb($5)
        virtual = human_kb($6)
        elapsed = $7

        printf "| %-10s | %-8s | %7s | %7s | %7s | %9s | %9s | %9s |\n", process_name(command), build_profile(command), pid, cpu, mem, rss, virtual, elapsed
        count++
    }
    END {
        if (count == 0) printf "| %-87s |\n", "No LaunchKick processes are currently running."
        print border
    }'

# Remove Darwin UI build artifacts.
clean-ui:
    rm -rf .build client/darwin/.build

# Remove Rust build artifacts.
clean-core:
    rm -rf core/target

# Remove all build artifacts.
clean: clean-ui clean-core
