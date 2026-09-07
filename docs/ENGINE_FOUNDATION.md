# Lightweight engine foundations — implemented pass

2026-09-07. Web-first, not web-only. This implements the first reliability/reuse pass
from [WEB_FIRST_ENGINE_REVIEW.md](WEB_FIRST_ENGINE_REVIEW.md), plus typed events.
It is not completion of the entire publishing/platform roadmap.

## Fixed input and time semantics

- `InputState::handle_key(key, pressed, ui_captured)` is the shared host policy:
  captured presses do not enter gameplay; releases always release held keys.
- Call `end_tick()` after **each** fixed update, before any rendering that might fail.
  Zero-tick frames leave pending edges intact. `end_frame()` remains a compatibility alias.
- Use `cancel_all()` on blur. Use `take_just_pressed()` for host shortcuts that must
  work while simulation is paused; do not consume gameplay presses this way accidentally.
- `TimeState::begin_paused_frame(single_step)` discards paused simulation debt, optionally
  schedules exactly one tick, and does not count ticks that the game never executes.
- `reset_elapsed()` rebases focus/resume transitions. The total accumulator is bounded
  even when a caller stops draining it. Invalid public timing settings are normalized.
- `real_dt` and FPS now describe actual elapsed time, not the capped simulation delta.
  This is a deliberate diagnostic correction; code requiring capped time should use
  the fixed tick path rather than feeding `real_dt` to gameplay.

The sandbox and Grim Delivery use the corrected input policy. Sandbox pause cancels
pending presses rather than replaying them on resume. Grim Delivery retains its existing
running-on-blur behavior but cancels held input. Movement constants and collision
resolution were not retuned.

## Small typed event system

`crates/sme_core/src/events.rs` provides `EventQueue<E>` with no new dependencies:

```rust
use sme_core::events::EventQueue;

#[derive(Debug)]
enum GameEvent { MusicChanged(&'static str), CheckpointReached(u32) }

let mut events = EventQueue::new(64);
// Producers handle Result: overflow returns the unsent event, never silently evicts.
if let Err(event) = events.send(GameEvent::MusicChanged("exploration")) {
    eprintln!("Queue full; rejected {event:?}");
}

// Host-owned boundary after simulation, before presentation/audio:
for event in events.drain() {
    // audio.observe(&event);
    // achievements.observe(&event); // optional explicit fan-out
}
```

The snippet illustrates the queue, not a provided achievement/audio service API.

Contract:
- Owned, typed payloads; FIFO order; bounded event count, not arbitrary payload bytes.
- Single owner. The host drains once and explicitly fans out to observers. Two separate
  drains are competing consumers, **not** broadcast subscriptions.
- No global bus, dynamic string lookup, callback reentrancy, worker thread or serialization.
- Consumers that create more events use a separate next-boundary queue, avoiding
  unbounded same-tick feedback loops. Dropping a drain discards its unread remainder.
- Host decides scheduling and overflow policy. Critical gameplay changes should not rely
  on an unchecked best-effort audio queue. Lifecycle `clear()` explicitly cancels a batch.

Standalone games define their own event enums (for example music-selection messages)
and dispatch them to their audio backend. Engine code does not know about individual
games' cues or encounters. The queue's FIFO, overflow, lifecycle cancellation and explicit
fan-out behavior have unit coverage. No telemetry subscriber was added.

## Shared native/browser runner

`crates/sme_platform/src/app.rs` exposes `GameHost` and `run::<Host>(PlatformConfig)`.
Grim Delivery demonstrates the host contract instead of duplicating window creation,
async-init handoff and event-loop setup. Standalone game consumers use the same API.

The runner owns startup and redraw scheduling. The game owns update, rendering and
input/event routing. Pending focus and actual size are reconciled after async startup;
input during loading is not replayed. Native uses blocking initialization, browser uses
`spawn_local`/`spawn_app`. Only existing workspace versions of `pollster` and
`wasm-bindgen-futures` became direct target-specific platform dependencies; no new
external crate/version was introduced. Cached manifests identify pollster 0.4.0
(zesterer/pollster, Apache-2.0/MIT) and wasm-bindgen-futures 0.4.58
(wasm-bindgen project, MIT OR Apache-2.0). No fresh upstream maintenance audit or
version upgrade was performed. Offline checks succeeded using the existing cache.

This is deliberately a thin runner, not a general scene/render framework. The native
Lua sandbox still has its specialized runner. GPU-init failures still follow existing
panic/error-shell behavior; device-loss recovery and native mobile lifecycle recreation
remain future work.

## Reusable authored-data modules

Existing native sandbox scene/atlas/animation registry modules now live in `sme_core`.
Compatibility re-exports keep sandbox imports intact. Shared APIs:

- `scene::parse_scene(&[u8])`
- `atlas::parse_atlas(&[u8])`
- `animation::parse_animation(&[u8])`
- `collision::parse_collision(&[u8])`
- `animation_registry::AnimationRegistry::add_file(...)`

These parsers have no I/O; disk/embedded/fetched bytes get the same validation. JSON
inputs are capped at 16 MiB; native path wrappers retain contextual file errors.
The limit applies when parsing; filesystem wrappers currently read before checking it,
so it is not a bounded file-acquisition or hostile-upload sandbox.

`CollisionGrid::try_from_file` validates programmatic construction too; legacy
`from_file` is a checked/panicking convenience for trusted valid data. Animation duration
conversion rejects overflow. Existing on-disk schema versions remain unchanged.

A shared byte parser is not yet a browser fetch/cache system. A complete atlas-backed
browser authoring demo, relative-URL asset loader, export manifest and package CLI remain
next work. Grim Delivery still demonstrates its existing code-authored level.

## Last-known-good reload

- `MultiAtlasRegistry::with_replacement` builds a candidate without mutating the live
  registry. Duplicate atlas keys are rejected instead of leaving stale indexed sprite IDs.
- Sandbox scene/atlas/animation reload validates dependencies/references and stages
  decoded/uploaded textures before committing registries, scene state and texture cache.
- Missing declared files, duplicate IDs, missing animation/sprite references and malformed
  PNGs return errors without replacing the working content.
- Successful reload replaces texture bytes even at unchanged paths. Removed authored
  textures are retired from the cache; generated debug textures are preserved.
- `Texture::try_from_bytes` bounds encoded bytes/decoded image allocation, checks device
  dimension limits and returns decoding errors. Existing trusted convenience constructors
  remain available. GPU OOM/device loss is not converted into a texture validation error.

Reload is transactional per resource/scene operation, not across every file changed in
one manual reload command. Watchers still monitor metadata; a PNG-only edit requires a
manual reload or a metadata change. GPU pixel identity after reload is not automatically
asserted in this pass.

## Telemetry — planned, NOT implemented

Owner request: eventually package selected engine/game information and send it out,
**with encryption**. Owner explicitly deferred implementation; events are the priority.

Future work must establish:
- Explicit event/metric schema, collection controls, privacy/consent and retention policy.
- Bounded batching, nonblocking transport and limited retry/backoff behavior on both targets.
- Receiver/endpoints and the required encryption boundary (transport, payload, storage).
- Receiver authentication/key management; do not embed a shared secret in browser games.

These are open design requirements, not approved crypto/configuration choices. No metrics
collector, identifiers, credentials, network exporter, encryption library, external
endpoint or automatic event forwarding was added. The event queue is local only.

## Verification

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python scripts/verify_engine.py --browser
# Requires native window/GPU; uses temporary copies, never edits original assets:
python scripts/smoke_engine_reload.py
```

Results for this pass:
- 122 engine workspace tests passed after standalone game separation.
- Native build/lint/fmt and Grim WASM check/release/browser rendering smoke are covered
  by `verify_engine.py`. Browser automation is not a human playtest or a device matrix.
- Native sandbox and Grim Delivery survived startup; Grim Delivery logged first presentation.
  These are brief smoke checks, not native gameplay completion or human playtests.
- Native reload test rejected missing sprite references, corrupt PNG and missing animation
  dependency, then recovered via valid reloads without panic or missing-sprite render warnings.
  Uses disposable fixture copies and reports log/process evidence, not pixel comparison.
- Standalone game code, audio, design documents and game-specific verification belong
  in their separate repository, not this engine distribution.

Reports: `.slipstream/engine-verification/`, `.slipstream/foundation-reload/`.
No telemetry transmission, Steam integration or scripting migration is implemented.
