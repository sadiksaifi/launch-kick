# LaunchKick Context

## Domain language

- **Launcher** — the user-facing command surface. On Darwin it is shown as a floating panel; behavior belongs in the Rust core unless it is UI, hotkey, rendering, or IPC plumbing.
- **Core** — the Rust process that owns launcher behavior and computation.
- **Platform client** — a native UI process that renders the launcher and communicates with the core. The current platform client is the Darwin client.
- **IPC contract** — the newline-delimited JSON message vocabulary exchanged between the core and a platform client.
- **Core session** — the core-owned launcher behavior for handling client messages and producing server messages. It is independent of process spawning and stdio transport.
- **Client message** — a message sent from a platform client to the core. Current variant: `input` with `text`.
- **Server message** — a message sent from the core to a platform client. Current variant: `result` with `value`.
- **Calculator** — the prototype computation used by the core session to evaluate launcher input.
