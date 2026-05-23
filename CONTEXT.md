# LaunchKick Context

## Domain language

- **Launcher** — the user-facing command surface. On Darwin it is shown as a floating panel; behavior belongs in the Rust core unless it is UI, hotkey, rendering, or IPC plumbing.
- **Core** — the Rust process that owns launcher behavior and computation.
- **Platform client** — a native UI process that renders the launcher and communicates with the core. The current platform client is the Darwin client.
- **IPC contract** — the newline-delimited JSON message vocabulary exchanged between the core and a platform client.
- **Core session** — the core-owned launcher behavior for handling client messages and producing server messages. It is independent of process spawning and stdio transport.
- **Client message** — a message sent from a platform client to the core. Current variants: `app::list` and `app::launch`.
- **Server message** — a message sent from the core to a platform client. Current variants: `app::list` and `app::launch::result`.
- **Application discovery/launch** — core-owned behavior for discovering launchable applications and launching a selected application.
