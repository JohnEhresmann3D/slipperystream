# Project Manifest — {project name}

<!--
Copy this file to your project as MANIFEST.md and fill it in.
This file is the single source of truth for which Slipstream library
contents are ACTIVE in this project. The library holds far more than any
one project uses; without this file, Claude has to guess what's live.
Read this file at the start of every session.
-->

## Active Personas

<!-- One line per active persona: name and the pack it comes from.
     Only personas listed here may be adopted in this project. -->

- {persona_name} (packs/{pack})
- {persona_name} (packs/{pack})

## Constitution Stack

<!-- The base constitution is always first and always active.
     Role constitutions are active only for the personas listed above. -->

- core/constitution/BASE_CONSTITUTION.md
- packs/{pack}/constitution/roles/*.md (for active personas only)

## Active Skills

<!-- All of core/skills/ is active by default. List pack skills explicitly. -->

- core/skills/* (default)
- packs/{pack}/skills/{skill_name}.md

## Capability Map

<!-- Map each capability the active skills need (see core/capabilities.md)
     to what is actually connected in THIS project. Map to "not available"
     honestly — skills degrade explicitly rather than pretending. -->

- web_research -> {e.g., WebSearch (Claude Code) | web search (claude.ai) | not available}
- file_read -> {binding}
- file_write -> {binding | not available}
- codebase_search -> {binding | not available}
- issue_tracking -> {e.g., Linear MCP | not available}
- design_lookup -> {e.g., Figma MCP | not available}
- data_query -> {binding | not available}
- communication -> {binding | not available}

## Deployment

- Host: {Claude Project | Claude Code | API}
- Tier-A hooks enabled: {e.g., credential_guard, destructive_bash_guard | none — Tier B only}

## Project State

<!-- Optional but recommended: where this project logs decisions and prior
     human rejections, so base constitution §5 (sticky rejections) has
     something to re-read. -->

- Decision log: {path, e.g., docs/decisions.md | none}
- State file: {path, e.g., STATE.md | none}
