# Slipstream

A markdown-native persona, constitution, and skill system for governing
Claude's behavior inside a project. **It is read, not run** — drop it into a
Claude Project, a Claude Code repo, or an API system prompt. It is not a
runtime, and it does not execute.

Slipstream gives a Claude-powered project three things:

1. **Identity** — who Claude is acting as, with a real point of view, not a
   job description. → `packs/{pack}/personas/`
2. **Authority** — what that identity may decide, and when it must stop and
   ask a human. → `core/constitution/` + `packs/{pack}/constitution/roles/`
3. **Capability** — reusable procedures portable across projects regardless
   of what tools are connected. → `core/skills/` + `packs/{pack}/skills/`

## The Honesty Rule

Every feature is classified by how it *actually* constrains behavior:

- **Tier A — platform-enforced.** Mechanically blocked (Claude Code hooks,
  or your own code on the raw API). Shipped wherever the host offers it.
- **Tier B — injected-once.** Identity and constraints present every turn,
  followed as instruction. Strong, and the backbone of everything here.
- **Tier C — protocol-dependent.** Multi-step self-administration across many
  turns with no outside check. Treated as a design smell and rewritten, not
  elaborated.

Nothing in this repository claims Tier A where the host can't back it.

## Getting Started

**The fast path:** give Claude `SETUP.md` and say "set this up." It detects
your host and tools, asks 3–5 questions, and generates the manifest, persona
wiring, and hooks itself. That's the intended onboarding; the steps below are
the manual equivalent.

1. Copy `MANIFEST.template.md` into your project as `MANIFEST.md` and fill
   it in: active personas, constitution stack, capability map, host.
2. Follow the deployment guide for your host:
   - `deploy/claude-project/` — claude.ai Projects (Tier B only)
   - `deploy/claude-code/` — Claude Code (Tier B + Tier A hooks)
   - `deploy/api/engine/` — raw API (optional reference runtime; not
     installed by default)
3. Need a role that doesn't exist? Use the factory: `factory/persona_author.md`
   and `factory/constitution_author.md` mint new ones, calibrated against the
   gamedev pack as few-shot exemplars.

## Layout

```
core/         constitution (supreme, domain-neutral), universal skills,
              capability vocabulary
factory/      skills that mint new personas, role constitutions, and skills
packs/        domain packs — gamedev ships as the worked example, not the product
deploy/       per-host deployment guides; api/engine is optional and out-of-band
MANIFEST.template.md   per-project activation file
```

## Design Notes

- **Personas are prose-first.** Frontmatter is what a platform can key off;
  the body is what conditions the model. A two-sentence body produces a
  generic assistant wearing a name tag — the quality bar lives in
  `factory/persona_author.md`.
- **Skills declare capabilities, not tools** (`core/capabilities.md`). The
  manifest maps capabilities to whatever is actually connected, so skills
  survive tool changes unchanged.
- **The gamedev pack is the factory's exemplar set**, not the framework.
  A new non-gamedev project takes all of `core/` and none of any pack.

Full rationale, format definitions, and the list of what v3 cut from v2 (and
why): `../SLIPSTREAM_TECHNICAL_SPEC_v3.md`.
