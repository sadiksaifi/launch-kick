# LaunchKick

Native Raycast-style command launcher with a Rust core and platform-specific native clients.

## Current structure

- `core/` — Rust core process using serde for the IPC contract and command/launcher behavior.
- `client/darwin/` — SwiftPM AppKit macOS client.
- `client/win/` — Windows client placeholder for future support.
- `client/linux/` — Linux client placeholder for future support.
- Platform clients communicate with the Rust core over newline-delimited JSON IPC.
- LaunchKick is expected to grow beyond application launching into a general command surface. Do not encode architecture assumptions that app launching is the only feature.

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
- Keep command behavior, launcher behavior, ranking/filtering, action execution, and other product computation in Rust core.
- Treat macOS as the active client while preserving seams for future Windows and Linux clients.
- Do not add extra backend variants unless explicitly requested.
- Do not commit generated build directories: `.build/`, `client/darwin/.build/`, or `core/target/`.

## Architecture guardrails

- Design for a growing Raycast-style command surface, not just app launching. Prefer domain names like command, action, result, source, or session when the behavior is broader than applications.
- Keep the Core session as the module that owns product policy for client messages: request handling, server-message decisions, result semantics, errors, and future command behavior.
- Keep platform clients as adapters at the IPC seam. They may own local UI state such as visibility and selection, but command computation and action semantics belong in the core.
- Do not introduce pass-through modules or seams just to make tests easy. A seam should represent real variation; test-only adapters are not enough by themselves.
- Keep feature implementations deep: put parsing, traversal, ranking, error policy, sorting, fallback behavior, and side-effect handling behind small interfaces with tests at those interfaces.
- Keep transport concerns separate from product behavior. Stdio/newline-delimited JSON framing should stay separate from process spawning and separate from Core session behavior.
- Treat the IPC contract as a real seam between the core and platform clients. When changing message vocabulary, update Rust and Swift codecs plus shared fixtures/tests together.
- When adding a new feature, update `CONTEXT.md` with durable domain language. Avoid listing short-lived implementation details there.
