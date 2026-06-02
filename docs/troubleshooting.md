# Nebula Mesh Troubleshooting

## Broker does not start
- Ensure Rust toolchain is installed (`rustup` + stable toolchain).
- Ensure port `24800` is not in use and allowed in firewall.

## Pairing fails
- Confirm all clients use the same `NEBULA_PAIR_CODE`.
- Regenerate pairing code in broker state storage for production.

## macOS capture/inject not working
- Grant `Accessibility` and `Input Monitoring` permissions to the app.
- Restart the app after permission change.

## Edge switching does not trigger
- Verify `config/layout.example.toml` edges are symmetrical.
- Increase deadzone in edge detector for high DPI monitors.

## Clipboard loop or stale text
- Keep debounce around 200-300ms.
- Drop updates from own `origin` token.

## Reconnect takes too long
- Confirm heartbeat (`PING`/`PONG`) is active.
- Add exponential backoff with max interval under 2s for MVP target.
