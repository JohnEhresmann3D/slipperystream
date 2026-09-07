# Lightweight, web-first engine roadmap

Updated 2026-09-07 after separating standalone game development into its own repository.
This public document covers reusable engine architecture only. See
[ENGINE_FOUNDATION.md](ENGINE_FOUNDATION.md) for implemented APIs and verification.

## Direction

**Web-first, not web-only.** Keep Rust/wgpu/winit, a lightweight baseline and native
Windows builds. Preserve future desktop storefront integrations without making them
mandatory dependencies of gameplay or browser packages. Native build success does not
establish Steam readiness or macOS/Linux compatibility; those need separate work.

Separate three products:
1. Engine: portable simulation, authored data and platform services.
2. Creator workflow: templates, validation, testing and web export.
3. Future game portal: discovery, moderation and isolated hosting.

Version engine APIs/data formats. Standalone games should pin a Git revision and upgrade
intentionally, rather than copying the engine or following a moving branch.

## Implemented foundation

- Fixed-step input edge consumption and shared key-capture/release policy.
- Explicit paused/single-step clock behavior, bounded debt and actual elapsed diagnostics.
- Transactional candidate validation and texture staging for native asset reload.
- Portable scene/atlas/animation/collision byte parsers and reusable registries.
- A thin native/browser app runner, demonstrated by Grim Delivery.
- Local bounded typed event queues with explicit overflow and dispatch ownership.

No ECS/editor rewrite or scripting-language migration was required.

## Next priorities

1. Browser asset fetch/cache and a small authored scene/atlas demo, including nested-path
   hosting and missing/corrupt resource handling. Parsers are portable; fetching is not yet
   a shared service.
2. Fallible GPU startup, visible startup errors, device/context-loss recovery and tests.
3. Project template/export manifest, pinned toolchain and automated native/browser CI.
4. Reusable audio/SFX, action mapping and deliberately scoped storage/UI interfaces when
   more than one game demonstrates their concrete requirements. Keep these optional.
5. Measure cold first-playable latency, compressed transfer size, decoded asset memory,
   draw calls, and p50/p95/p99 frame times on declared real hardware/browser targets.
   Reuse scratch buffers and add view culling before redesigning the renderer.

The collision solver remains bounded-speed axis-separable resolution, not arbitrary
continuous collision detection. Shared Lua remains native-only; Rust is the demonstrated
browser gameplay path. A future browser scripting requirement needs a bounded VM spike,
not an automatic language migration.

## Events and future telemetry

Gameplay events should remain local and deterministic. Audio and other services consume
messages at host-owned boundaries. A telemetry observer is a separate future capability,
not an automatic forwarding of every event.

Eventually package selected information and send it out **encrypted**. Implementation
is explicitly deferred. Receiver, privacy/consent, schemas, retry budgets, encryption
boundary and key management remain open requirements. No collector or exporter exists.

## Future publishing safety

A portal must treat uploaded games as untrusted executable content. WASM does not isolate
accompanying JavaScript or make source builds safe. Before open submissions, design
separate cookieless content origins, sandboxed frames, capability-limited host messaging,
validated packages and disposable resource-limited build workers without production
secrets. Rust build scripts/procedural macros execute during compilation.

Start with curated game packages. Public submissions need moderation/reporting,
provenance/credits and abuse handling before rollout. These are future design requirements,
not implemented security controls. No public portal or storefront integration is shipped.
