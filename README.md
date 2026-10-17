# Kanso

A local-first markdown notes app built with [GPUI](https://www.gpui.rs) through
[gpui-kit](https://github.com/longbridge/gpui-kit). It imports an Inkdrop database, serves
Inkdrop's local HTTP API on `127.0.0.1:19840` so existing Inkdrop MCP setups keep working, and
syncs through Cloudflare.

Status: early. See [docs/PLAN.md](docs/PLAN.md) for the design and milestones.

## Running the editor spike

```bash
cargo run --release -p kanso-app -- path/to/note.md --files path/to/attachments
```

Add `--bench` to print edit-to-frame latencies and quit, or `--no-preview` to hide the preview.
