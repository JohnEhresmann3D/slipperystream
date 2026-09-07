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

## 2026-09-07 — Lightweight native/web foundation

- **Owner direction:** Web-first, not web-only. Preserve lightweight native desktop
  publishing and incremental upgrades. No engine/language migration. Rust is the
  demonstrated browser gameplay path; native Lua 5.4 remains supported. This resolves
  the earlier scripting blocker for Rust browser games, not browser Lua itself.
- **Source correction:** Current WASM backend is WebGL2 only, not mixed WebGPU fallback.
- **Reliability:** Shared captured-press/release input policy, per-tick edge consumption,
  focus cancellation, explicit pause/single-step clock, bounded total accumulator and
  uncapped elapsed diagnostics. No controller tuning or collision-solver replacement.
- **Reuse:** Scene/atlas/animation registry extracted into sme_core with compatibility
  facades; bounded byte parsers for scenes, atlases, animation and collision. Thin
  GameHost native/web runner and typed bounded FIFO EventQueue added. Overflow and
  dispatch boundaries are explicit; no global callbacks or networking.
- **Assets:** Native scene/atlas/animation reload stages and validates candidates and
  textures before replacing live state. Fallible bounded image decoding, same-path
  texture refresh, collision construction validation and duration-overflow rejection.
- **Telemetry:** Owner requested eventual encrypted information export, then explicitly
  deferred implementation. Receiver/privacy/key-management design remains open. No
  collector, encryption implementation, credentials or data transmission was added.
- **Dependencies:** Only existing pollster/wasm-bindgen-futures versions gained direct
  target-specific platform edges. No new external crate/version for this foundation.
- **Verification:** 122 engine workspace tests after game separation; native build,
  clippy/fmt, Grim WASM build/browser rendering smoke, native startup and transactional
  reload failure/recovery tests. Reports are ignored local artifacts, not human playtests.
- **Next:** Shared browser asset fetch/cache, authored web demo, GPU recovery, CI and
  export tooling. Native Steam services remain optional future work.

## 2026-09-07 — Owner-authorized repository separation and publication

- Owner authorized pushing reusable engine updates to the existing public origin and
  creating a separate **private** game repository. Game source, music and design files
  are excluded from the engine commit; games reference an exact engine Git revision.
- Previously uncommitted mixed-workspace game logs were preserved in the private game's
  history archive and ignored local split backup, rather than exposing them publicly.
  All previously committed engine state entries above remain preserved.
- Source assets are copied and hash-verified before legacy paths are archived locally.
  No force push, public game release, audio regeneration or save-data migration.
