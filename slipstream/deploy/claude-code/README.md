# Deploying Slipstream to Claude Code

Claude Code is the one host where Slipstream gets real mechanical (Tier A)
enforcement for free, via hooks. Everything else operates at Tier B, same as
other hosts.

## Mapping

| Slipstream artifact | Claude Code location |
|---|---|
| Persona (`packs/{pack}/personas/{name}.md`) | `.claude/agents/{name}.md` — frontmatter maps to the agent's `name`/`description`; the prose body becomes the agent's system prompt, prefixed with a pointer to its constitution stack |
| Base constitution | Referenced from `CLAUDE.md`: "Read and follow `slipstream/core/constitution/BASE_CONSTITUTION.md`; it is supreme." |
| Role constitutions | Referenced from each agent file's body |
| Skill (`core/skills/{name}.md`, `packs/{pack}/skills/{name}.md`) | `.claude/skills/{name}/SKILL.md` — the file content is already in SKILL.md convention; copy it into a directory named for the skill |
| Manifest | `MANIFEST.md` at repo root; `CLAUDE.md` instructs Claude to read it at session start |
| Tier-A enforcement | `.claude/settings.json` hooks — see `hooks.example.json` |

## Steps

1. Copy `MANIFEST.template.md` → `MANIFEST.md` at the repo root and fill it
   in. Map capabilities to Claude Code's actual tools (`web_research ->
   WebSearch`, `codebase_search -> Grep/Glob`, etc.).
2. Add to `CLAUDE.md`:

   ```
   Read MANIFEST.md at session start. It lists which Slipstream personas,
   constitutions, and skills are active. The base constitution at
   slipstream/core/constitution/BASE_CONSTITUTION.md is supreme.
   ```

3. For each active persona, create `.claude/agents/{name}.md` from the
   persona file (body first, then a line pointing at its role constitution).
4. For each active skill, create `.claude/skills/{name}/SKILL.md` from the
   skill file.
5. **Promote what you can to Tier A.** `hooks.example.json` shows the
   pattern: a persona's `prohibits` entries that correspond to bash/write
   patterns become PreToolUse deny hooks; `hitl_required` entries become
   confirmation prompts. Copy the relevant blocks into `.claude/settings.json`
   and adapt the patterns to your project. Record which hooks you enabled in
   the manifest's *Deployment* section.

## The Tier Line, Honestly

Hooks can enforce what is expressible as a tool-call pattern: "no `git push
--force`", "nothing writes to `secrets/`", "confirm before `rm -rf`". They
cannot enforce "don't override the designer on feasibility" — that stays
Tier B, carried by the persona and constitution prose. Don't relabel Tier B
as enforced just because the same file also ships hooks.
