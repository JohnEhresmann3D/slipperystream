# Project State — Saturday Morning Engine

Decision and rejection log for Slipstream base constitution §5 (sticky
rejections). Append new entries; do not rewrite history.

## Decisions

- 2026-07-09 — **Slipstream installed** (full setup, gamedev pack). Producer,
  game_designer, gameplay_engineer personas wired into `.claude/agents/`.
  They layer over existing domain agents rather than replacing them.
- 2026-07-09 — **Strategic direction: 2D + web publishing first.** Keep the
  proven 2D engine as the primary target. Ship web publishing and better
  asset/ease-of-use tooling now. 3D (PS1/PS2 retro look) is deferred, not
  cancelled. First concrete deliverable: a browser build of the current engine.
  (Owner decision; producer/engineer to sequence.)

- 2026-07-09 — **Grim Delivery ported in as first game case.** Copied
  `crates/grim_delivery` (48h-jam prototype: Paperboy-as-Grim-Reaper) from
  `D:\Development\Testing_Slipstream\slipperystream` (engine there was at the
  identical commit 9db328c — no engine divergence). GDD at
  `docs/planning/grim_delivery_gdd.md`. Pure Rust, no Lua, no external assets
  (vertex-colored quads, one draw call) — deliberately skips the Lua bridge
  because the v0.1 intent API can't express spawning/projectiles/HUD (fork's
  2026-07-08 decision, carried over). This makes it the ideal first candidate
  for the web (wasm) build. All 10 crate tests pass; workspace tests, clippy,
  fmt clean.

- 2026-07-09 — **Grim Delivery moved to `examples/grim_delivery`** (owner
  direction: examples dir, no commit yet).
- 2026-07-09 — **Web build path implemented.** `sme_core` now uses
  `web_time::Instant`; `GpuContext` gained `new_async` with
  BROWSER_WEBGPU/GL backends + WebGL2 downlevel limits on wasm;
  `create_window` attaches the canvas on web; grim_delivery has a wasm entry
  (spawn_app + user-event async init). `egui-winit` default features disabled
  workspace-wide (arboard doesn't build on wasm; loses egui clipboard).
  Everything compiles for `wasm32-unknown-unknown`; native: 11/11 test suites,
  clippy, fmt clean. Docs: `docs/setup/web_build.md`. Release bundle built
  with trunk (6.3 MB wasm — wasm-opt/size profile is future work) and verified
  serving (index + wasm HTTP 200). In-browser runtime behavior not yet
  human-verified.

## Rejections

- (none yet)

## Open Questions

- **Scripting for web.** mlua (vendored C Lua) does not compile to
  `wasm32-unknown-unknown`, which wgpu/winit require for web. Options:
  (a) switch to a pure-Rust scripting VM (e.g. Rhai, or a Rust Lua like
  piccolo) for one codebase across native+web; (b) keep Lua and ship web
  without scripting for now; (c) dual-target. Owner asked for a recommendation
  — pending write-up. Blocks the web deliverable.
