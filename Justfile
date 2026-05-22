ui_package := "client/darwin"
ui_product := "launch-kick"
ui_binary := ".build/launch-kick"
core_manifest := "core/Cargo.toml"

# List available commands.
default:
    just --list

# Build everything.
build: build-core build-ui

# Run the launcher prototype.
run: ensure-ui
    cargo run --manifest-path {{core_manifest}}

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

# Build the Rust core.
build-core:
    cargo build --manifest-path {{core_manifest}}

# Build the Darwin UI and copy it to the path used by the core.
build-ui:
    mkdir -p .build
    swift build --package-path {{ui_package}} --scratch-path .build/swift --product {{ui_product}}
    cp .build/swift/debug/{{ui_product}} {{ui_binary}}

# Rebuild the UI only when the copied binary is missing or stale.
ensure-ui:
    mkdir -p .build
    if [ ! -x {{ui_binary}} ] || [ {{ui_package}}/Package.swift -nt {{ui_binary}} ] || find {{ui_package}}/Sources -name '*.swift' -newer {{ui_binary}} | grep -q .; then just build-ui; fi

# Remove Darwin UI build artifacts.
clean-ui:
    rm -rf .build client/darwin/.build

# Remove Rust build artifacts.
clean-core:
    rm -rf core/target

# Remove all build artifacts.
clean: clean-ui clean-core
