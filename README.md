# Nebula Mesh MVP

Custom LAN-first input mesh with:
- Rust WebSocket broker (`broker`)
- Cross-platform client skeleton (`client`)
- Shared protocol crate (`protocol`)

## About

Nebula Mesh is a local-first input mesh project focused on reliable cross-device control for a high-performance desk setup:
- Gaming PC as the primary control head for lowest-latency transitions.
- Always-on Windows laptop as the mesh hub (broker, session routing, config).
- macOS + Windows clients with edge-drag cursor handoff and clipboard sync.

Built by Kushagra Agarwal to create a practical multi-device workflow foundation that can later evolve into a broader home-lab automation stack.

## Current implementation status

- Broker: `HELLO`, `PAIR` (6-digit code), `PING`/`PONG`, session route table, active controller/target routing.
- Protocol: mouse move/button/scroll, key, edge enter/exit, layout, clipboard text, ack, error.
- Client transport: connect/reconnect loop, heartbeat, message send/receive pipeline.
- Edge switching: deadzone-based edge detection hooks + enter/exit payloads.
- Clipboard sync: text payload emission with debounce and loop-prevention placeholder.
- Windows module: cursor delta capture via `GetCursorPos`, input injection via `SendInput`.
- macOS module: integration scaffold for capture/inject with suppression windows.

## Project layout

```text
nebula-mesh/
  broker/
  client/
  protocol/
  config/layout.example.toml
  docs/troubleshooting.md
  docs/windows-service.md
```

## Run plan

1. Install Rust toolchain (`rustup`) on development machine(s).
2. Start broker:
   - `cargo run -p broker`
3. Start clients (each machine):
   - `NEBULA_BROKER=ws://<ideapad-ip>:24800 NEBULA_DEVICE_ID=<device-id> NEBULA_PAIR_CODE=123456 cargo run -p client`

## Environment variables

- `NEBULA_BROKER`: broker websocket URL.
- `NEBULA_DEVICE_ID`: stable per-machine id (`gaming-pc`, `ideapad`, `mac-air`, `mac-pro`).
- `NEBULA_PAIR_CODE`: pairing code for session token negotiation.
- `NEBULA_RECONNECT_MS`: reconnect sleep in milliseconds (default `800`).

## Notes

- This MVP currently favors fast iteration and protocol plumbing over full native parity.
- For production: replace static pairing code, add TLS (`wss`), and harden native capture loops.

## CI Artifacts

GitHub Actions builds release binaries for Windows and macOS on:
- pushes to `main`
- pull requests targeting `main`
- tags like `v0.1.0`

Download compiled binaries from the workflow run artifacts:
- `nebula-Windows-release`
- `nebula-macOS-release`
