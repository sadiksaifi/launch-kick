# LaunchKick

LaunchKick is a native launcher application with a Rust core and a platform-specific native client.

Current development targets macOS. The codebase is structured so Windows and Linux clients can be added later without moving launcher behavior out of the Rust core.

```bash
just run
```

## Structure

```txt
core/            Rust core and launcher behavior
client/darwin/   SwiftPM AppKit macOS client
client/win/      Windows client placeholder
client/linux/    Linux client placeholder
```

Platform clients communicate with the Rust core over newline-delimited JSON IPC.
