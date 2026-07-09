---
name: skill_author
description: "Mint a new skill file — a reusable procedure declared against the capability vocabulary, portable across projects with different tool sets."
capabilities_needed: [file_read, file_write]
allows: [draft_skills, propose_capability_additions]
prohibits: [hardcode_tool_names, add_capabilities_without_updating_vocabulary]
---

# Skill Author

## Purpose

Produce a skill another project could adopt unchanged. The portability rule
that makes this possible: a skill declares **capabilities**, never tools. The
moment a skill body says "use the Linear MCP" instead of "using
`issue_tracking`," it stops being a library skill and becomes project config.

## Use When

- A procedure has been worked out in one project and is worth keeping.
- A persona's `skills_required` names a skill that doesn't exist yet.

## Procedure

1. **Decide where it lives.** Universal (any domain could use it) →
   `core/skills/`. Domain-specific → `packs/{pack}/skills/`. When unsure,
   pack — promoting to core later is cheap; a domain assumption in core
   contaminates every project.
2. **Write the frontmatter** per §6 of the spec: `name`, `description`
   (one sentence, starts with what the skill produces), `capabilities_needed`
   (names from `core/capabilities.md` only), `allows`, `prohibits`.
   If a needed capability isn't in the vocabulary, propose the vocabulary
   addition first — as an ability, not a product name.
3. **Write the body:** Purpose (including the failure mode the skill
   prevents), Use When (concrete triggers, not "when appropriate"), Procedure
   (numbered steps, each producing something inspectable), Quality Gate
   (checks a reader could apply to the skill's output).
4. **Sweep for tool names.** Search the draft for anything that names a
   product, platform, or binary. Each hit either becomes a capability
   reference or moves to a deployment README.
5. **Sweep for Tier C** (spec §2). Any step requiring the model to faithfully
   self-administer state across many turns with no outside check ("track your
   remaining budget," "remember to re-verify every N steps") is a design
   smell. Rewrite it as a single-turn check, an artifact the model re-reads,
   or a constitution clause — or cut it.

## Output

One skill file, delivered as a draft for human review, plus (if needed) a
proposed diff to `core/capabilities.md`.

## Quality Gate

- Zero concrete tool names in the body or frontmatter.
- Zero Tier-C steps.
- Every `capabilities_needed` entry exists in the vocabulary.
- Procedure steps each yield an inspectable output.
