set shell := ["bash", "-euo", "pipefail", "-c"]

rust_manifest := "core/Cargo.toml"
rust_debug_binary := "core/target/debug/launchkick-core"
rust_release_binary := "core/target/release/launchkick-core"

swift_package := "client/darwin"
swift_product := "launch-kick"
swift_build_scratch := ".build/swift"
swift_test_scratch := ".build/swift-test"
swift_staged_binary := ".build/launch-kick"
swift_staged_profile := ".build/launch-kick.profile"

swiftlint_config := ".swiftlint.yml"
swiftformat_config := ".swiftformat"

# List available recipes.
default: list

# List available recipes.
list:
    just --list

# Verify required developer tools are installed.
doctor:
    scripts/dev/check-tools.sh

# Run the full local verification suite.
ci: fmt-check lint check test

# Build the Rust core and stage the Swift Darwin client for development.
build: rust-build swift-stage

# Build the Rust core and stage the Swift Darwin client with release optimizations.
build-release: rust-build-release swift-stage-release

# Run the launcher in development mode.
run: swift-stage-if-needed
    cargo run --manifest-path {{rust_manifest}}

# Run the launcher with release optimizations.
run-release: build-release
    {{rust_release_binary}}

# Check Rust and Swift compilation.
check: rust-check swift-check

# Run Rust and Swift tests.
test: rust-test swift-test

# Run Rust and Swift linters.
lint: clippy swiftlint

# Format Rust and Swift code.
fmt: rustfmt swiftformat

# Check Rust and Swift formatting without writing changes.
fmt-check: rustfmt-check swiftformat-check

# Build the Rust core for development.
rust-build:
    cargo build --manifest-path {{rust_manifest}}

# Build the Rust core with release optimizations.
rust-build-release:
    cargo build --release --manifest-path {{rust_manifest}}

# Check Rust compilation across all targets.
rust-check:
    cargo check --manifest-path {{rust_manifest}} --all-targets

# Run Rust tests.
rust-test:
    cargo test --manifest-path {{rust_manifest}}

# Run Clippy across all Rust targets with warnings denied.
clippy:
    cargo clippy --manifest-path {{rust_manifest}} --all-targets -- -D warnings

# Format Rust code with rustfmt.
rustfmt:
    cargo fmt --manifest-path {{rust_manifest}}

# Check Rust formatting without writing changes.
rustfmt-check:
    cargo fmt --manifest-path {{rust_manifest}} -- --check

# Build the Swift Darwin client for development.
swift-build:
    swift build --package-path {{swift_package}} --scratch-path {{swift_build_scratch}} --product {{swift_product}}

# Build the Swift Darwin client with release optimizations.
swift-build-release:
    swift build --configuration release --package-path {{swift_package}} --scratch-path {{swift_build_scratch}} --product {{swift_product}}

# Build and stage the Swift Darwin client binary used by the Rust core.
swift-stage:
    scripts/dev/stage-swift-client.sh debug {{swift_package}} {{swift_product}} {{swift_build_scratch}} {{swift_staged_binary}} {{swift_staged_profile}}

# Build and stage the release Swift Darwin client binary used by the Rust core.
swift-stage-release:
    scripts/dev/stage-swift-client.sh release {{swift_package}} {{swift_product}} {{swift_build_scratch}} {{swift_staged_binary}} {{swift_staged_profile}}

# Build-check the Swift Darwin client.
swift-check: swift-build

# Run Swift Darwin client tests.
swift-test:
    swift test --package-path {{swift_package}} --scratch-path {{swift_test_scratch}}

# Run SwiftLint for the Darwin client.
swiftlint:
    swiftlint lint --config {{swiftlint_config}}

# Format Swift code with SwiftFormat.
swiftformat:
    swiftformat {{swift_package}} --config {{swiftformat_config}}

# Check Swift formatting without writing changes.
swiftformat-check:
    swiftformat {{swift_package}} --config {{swiftformat_config}} --lint

# Stage the development Swift Darwin client only when the staged binary is missing or stale.
swift-stage-if-needed:
    scripts/dev/stage-swift-client.sh debug {{swift_package}} {{swift_product}} {{swift_build_scratch}} {{swift_staged_binary}} {{swift_staged_profile}} --if-needed

# Stage the release Swift Darwin client only when the staged binary is missing or stale.
swift-stage-release-if-needed:
    scripts/dev/stage-swift-client.sh release {{swift_package}} {{swift_product}} {{swift_build_scratch}} {{swift_staged_binary}} {{swift_staged_profile}} --if-needed

# Show current launcher process resource usage in a human-readable table.
resources:
    scripts/dev/resources.sh {{rust_debug_binary}} {{rust_release_binary}} {{swift_staged_binary}} {{swift_staged_profile}}

# Remove Rust build artifacts.
clean-rust:
    rm -rf core/target

# Remove Swift and staged launcher build artifacts.
clean-swift:
    rm -rf .build client/darwin/.build

# Remove all build artifacts.
clean: clean-rust clean-swift
