# Web (wasm) Build

The reusable engine crates and `grim_delivery` compile to
`wasm32-unknown-unknown` and run in the browser via **WebGL2** through wgpu.
The current source explicitly selects GL on wasm; automatic WebGPU fallback is
not enabled. Standalone game projects live outside this workspace and pin an engine
Git revision. Run `python scripts/verify_engine.py --browser` from the repository
root for build/lint/tests and a local-only browser rendering smoke.

## Prerequisites

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

## Build and serve

```powershell
cd examples/grim_delivery
trunk serve          # dev server at http://127.0.0.1:8080, rebuilds on change
trunk build --release  # static bundle in examples/grim_delivery/dist/
```

The `dist/` output is plain static files — host it anywhere (itch.io zip,
GitHub Pages, any static host).

## How the web path works

| Concern | Native | Web (wasm32) |
|---|---|---|
| GPU init | `GpuContext::new` (blocking, pollster) | `GpuContext::new_async` awaited in a spawned future; finished `GameApp` returns to the loop as a winit user event |
| Backends | DX12 / Vulkan | `GL` (WebGL2), using `downlevel_webgl2_defaults` limits |
| Clock | `std::time::Instant` | `web_time::Instant` (performance.now) — swapped in `sme_core::time` for both targets |
| Event loop | `run_app` (blocks) | `spawn_app` (returns; browser drives via requestAnimationFrame) |
| Canvas | n/a | `sme_platform::window::create_window` attaches winit's canvas to `#sme-container` (or `<body>`) and focuses it |
| Logging | env_logger | console_log + console_error_panic_hook |

## Known constraints

- `egui-winit` runs with `default-features = false` workspace-wide: its
  default `clipboard` feature (arboard) does not compile on wasm32. Cost:
  no copy/paste in egui text fields.
- `sme_game` (the engine sandbox binary) is **not** web-buildable yet: mlua's
  vendored C Lua does not compile to wasm32. See STATE.md open question on
  web scripting. `grim_delivery` is pure Rust and unaffected.
- Filesystem asset loading / hot reload paths in `sme_game` would need an
  HTTP fetch layer on web; `grim_delivery` embeds everything, so it needs none.
