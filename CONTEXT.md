# LaunchKick Context

## Domain language

- **Launcher** — the user-facing command surface. On Darwin it is shown as a floating panel; behavior belongs in the Rust core unless it is UI, hotkey, rendering, or IPC plumbing.
- **Core** — the Rust process that owns launcher behavior and computation.
- **Platform client** — a native UI process that renders the launcher and communicates with the core. Darwin is the active platform client; Windows and Linux are reserved platform seams.
- **IPC contract** — the newline-delimited JSON message vocabulary exchanged between the core and a platform client.
- **Core session** — the core-owned product-policy module for handling client messages and producing server messages. It owns query handling, result production, action execution semantics, and error-to-message decisions. It is independent of process spawning and stdio transport.
- **Client message** — a message sent from a platform client to the core. Durable categories are query/update intent and execute intent.
- **Server message** — a message sent from the core to a platform client. Durable categories are renderable launcher results and action execution results.
- **Query** — the current user text used by the core to compute and rank launcher results.
- **Launcher result** — a renderable item returned by the core. A result has an opaque ID, title, optional subtitle/icon hint, source, and available actions.
- **Command source** — a core implementation that contributes launcher results for a query. Applications are modeled as a command source.
- **Command source registry** — the core module that combines registered command sources for a query while preserving source-owned result production and action semantics.
- **Application command source** — the core module that adapts application discovery/launch into launcher results and app-open action bindings.
- **Action** — an operation available on a launcher result, such as opening an application. The platform client sends execute intent; the core owns action semantics.
- **Action binding** — the core-owned association between a launcher result action ID and the source-specific action execution behavior. Launcher session state stores bindings but does not inspect source-specific action details.
- **Action execution** — core-owned behavior for running an action and returning success or failure to the platform client.
- **Launcher session state** — the core-owned memory of the current query, visible launcher results, known launcher results, and action bindings that can be executed.
- **Launcher interaction** — the Darwin platform client policy module that translates user intents and core events into local launcher state changes plus UI/IPC effects. It keeps AppKit as an adapter for rendering and effect execution.
- **Application discovery/launch** — a source-specific core implementation for discovering launchable macOS applications and executing the app-open action.
- **Application discovery snapshot cache** — the core-owned in-memory snapshot of discovered applications. It serves query-time ranking from memory, returns the current snapshot immediately, and may refresh discovery in the background for later queries.
- **Darwin IPC adapter** — the Darwin platform client adapter that maps launcher-domain IPC intents/events to the IPC contract while delegating `FileHandle` IO and NDJSON stream semantics to the Darwin IPC stream.
- **Darwin IPC stream** — the Darwin platform client module that owns `FileHandle` IO and NDJSON line stream semantics. It does not own IPC contract message vocabulary or launcher interaction policy.
- **Platform client process lifecycle** — the core runtime concern for spawning, wiring stdio for, and waiting on the active platform client process. It is separate from IPC transport and Core session behavior.
