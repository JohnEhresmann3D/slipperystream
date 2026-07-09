# Slipstream Setup — Instructions for Claude

You have been given this file because a human wants Slipstream set up in
their project. Your job: run a short interview, then generate everything —
they should answer a handful of questions and be done. Do not make them read
deployment docs; that is what this file replaces.

## Ground Rules

- **Detect before asking.** Anything you can establish yourself (host
  platform, available tools, project type from the repo contents), establish
  yourself. Only ask what you genuinely can't know.
- **Batch the questions.** One round, 3–5 questions, with a recommended
  default marked on each. A second round only if an answer opens a real fork.
- **Defaults over ceremony.** Every question they skip gets the default.
  A working minimal setup now beats a complete one abandoned halfway.

## Phase 1 — Detect (no questions yet)

1. **Host:** Are you running in Claude Code (you can write files and run
   commands), a Claude Project (knowledge + chat only), or via API
   orchestration? This decides everything downstream.
2. **Tools:** Inventory what's actually connected (built-in tools, MCP
   servers). Draft the capability map yourself from `core/capabilities.md`
   — map each capability to a real tool you can see, or "not available."
   The human confirms; they don't author it.
3. **Project:** If there's a repo or knowledge base, skim it. Language,
   domain, maturity. If the domain matches an existing pack (`packs/`),
   you'll propose it in Phase 2.

## Phase 2 — Interview (one batch)

Ask, with your detected recommendation stated inline on each:

1. **What is this project, in a sentence or two?** (Skip if Phase 1 made it
   obvious — state your inference and ask only for correction.)
2. **What roles do you want Claude to play?** Propose concretely: matching
   pack personas if a pack fits, otherwise the 2–3 roles the factory should
   mint. Never ask this open-ended; always propose a slate to react to.
3. **Where should decisions and rejections be logged?** Default:
   `STATE.md` at project root. This gives the base constitution's sticky
   rejections (§5) something to re-read.
4. **[Claude Code only] Enable Tier-A hooks?** Default: yes —
   `credential_guard` + `destructive_bash_guard` from
   `deploy/claude-code/hooks.example.json`, patterns adapted to this repo.
5. **Anything Claude must never do in this project?** Free text; becomes
   manifest-level prohibits and, where expressible as a tool pattern, hooks.

## Phase 3 — Generate

From the answers, produce (write directly if you can write files; otherwise
output each as a copy-paste block with its destination stated):

1. **`MANIFEST.md`** — from `MANIFEST.template.md`, fully filled: active
   personas, constitution stack, active skills, the confirmed capability
   map, host, hooks enabled, state file location.
2. **Personas** — for pack personas, wire them in as-is. For new roles, run
   `factory/persona_author.md` (and `constitution_author.md` for new role
   types), calibrating against `packs/gamedev/`. Deliver drafts and say
   plainly: *persona bodies deserve human review — read these.*
3. **Host wiring:**
   - *Claude Code:* `.claude/agents/{name}.md` per persona (body + pointer
     to its constitution stack), `.claude/skills/{name}/SKILL.md` per active
     skill, the manifest-read block appended to `CLAUDE.md`, hooks merged
     into `.claude/settings.json` if accepted.
   - *Claude Project:* the ordered list of files to add to project
     knowledge, plus the custom-instructions block from
     `deploy/claude-project/README.md`.
   - *API:* point them at `deploy/api/engine/README.md`; generate the
     manifest and persona files only.
4. **`STATE.md`** (or their chosen path) — created with headers:
   `## Decisions`, `## Rejections`, `## Open Questions`.

## Phase 4 — Verify and Hand Off

- Re-read every generated file; check each persona's `skills_required` and
  `capabilities_used` resolve to files and vocabulary entries that exist.
- Demonstrate, don't describe: adopt one active persona and answer a small
  real question from their project in that persona's voice, so they see the
  difference immediately.
- Close with the two-line usage summary: *"Sessions start by reading
  MANIFEST.md. Ask for a persona by name; log decisions to STATE.md."*

## What Not to Do

- Don't install packs the project doesn't need (a non-gamedev project takes
  `core/` and nothing from `packs/`).
- Don't claim hooks enforce anything on hosts without hooks. Tier B is
  stated instruction — say so honestly if asked.
- Don't run the interview if a filled `MANIFEST.md` already exists — offer
  to update it instead.
