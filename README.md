# LaunchKick

LaunchKick is a native Raycast-style command launcher with a Rust core and platform-specific native clients.

macOS/Darwin is the active platform. Windows and Linux directories are reserved platform seams; keep launcher behavior in the Rust core so new platform clients can stay thin.

```bash
just run
```

## Structure

```txt
core/            Rust core, command sources, launcher session state, and IPC contract
client/darwin/   SwiftPM AppKit macOS client for UI, hotkeys, rendering, and IPC
client/win/      Reserved Windows platform client seam
client/linux/    Reserved Linux platform client seam
ipc/fixtures/    Shared Rust/Swift IPC contract fixtures
```

Platform clients communicate with the Rust core over newline-delimited JSON IPC. The IPC vocabulary is launcher-oriented: `launcher::query`, `launcher::execute`, `launcher::results`, and `launcher::action::result`.
