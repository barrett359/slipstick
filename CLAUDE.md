# Slipstick agent interface

Use the provider-neutral Rust CLI described in [docs/agent-cli.md](docs/agent-cli.md).
Discover capabilities and schemas before editing:

```sh
cargo run --quiet -- agent capabilities
cargo run --quiet -- agent schema fleet
```

Do design work in a draft. Never edit or replace `data/fleet.json` directly.
Use `--input FILE` or `--input -` for JSON and parse only stdout as the result
envelope. A live-fleet commit must name each selected entity and use `--apply`.
