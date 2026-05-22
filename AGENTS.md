# LaunchKick

Native launcher application with a Rust core and platform-specific native clients.

## Current structure

- `core/` — Rust core process using serde for the IPC contract and launcher behavior.
- `client/darwin/` — SwiftPM AppKit macOS client.
- `client/win/` — Windows client placeholder for future support.
- `client/linux/` — Linux client placeholder for future support.
- Platform clients communicate with the Rust core over newline-delimited JSON IPC.

## Platform direction

- Current development targets macOS only.
- Keep the architecture ready for future Windows and Linux clients.
- Do not implement Windows or Linux behavior unless explicitly requested.

## Commands

- `just run` — build the macOS client if needed, then run the Rust core.
- `just build-ui` — build the current macOS UI binary.
- `just build-core` — build the Rust core.
- `just check` — verify Rust core and macOS client build.
- `just fmt` — format Rust code.
- `just clean-ui` — remove UI build artifacts.
- `just clean-core` — remove Rust target artifacts.
- `just clean` — clean both UI and core artifacts.

## Working rules

- Keep platform clients thin: UI, hotkeys, rendering, IPC.
- Keep launcher behavior and computation in Rust core.
- Treat macOS as the active client while preserving seams for future Windows and Linux clients.
- Do not add extra backend variants unless explicitly requested.
- Do not commit generated build directories: `.build/`, `client/darwin/.build/`, or `core/target/`.
