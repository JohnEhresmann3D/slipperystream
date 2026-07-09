# Project Manifest — Saturday Morning Engine (SlipperyStream)

<!-- Single source of truth for which Slipstream library contents are ACTIVE.
     Read this file at the start of every session. -->

## Active Personas

- producer (packs/gamedev)
- game_designer (packs/gamedev)
- gameplay_engineer (packs/gamedev)

<!-- These governance personas layer OVER the existing domain agents in
     .claude/agents/ (engine-architect, rendering-engineer, etc.). The
     Slipstream personas own decision authority, scope, and boundaries;
     the domain agents remain the specialist executors. When they overlap
     (producer vs orchestrator-technical-producer), the Slipstream persona's
     constitution and defers_to/hitl_required lists govern. -->

## Constitution Stack

- slipstream/core/constitution/BASE_CONSTITUTION.md  (supreme, always active)
- slipstream/packs/gamedev/constitution/roles/producer_constitution.md
- slipstream/packs/gamedev/constitution/roles/game_designer_constitution.md
- slipstream/packs/gamedev/constitution/roles/gameplay_engineer_constitution.md

## Active Skills

Read directly from the repo (Slipstream is "read, not run"; not duplicated
into .claude/skills to avoid colliding with the existing engine-* skills):

- slipstream/core/skills/deep_researcher.md
- slipstream/core/skills/planning_and_scoping.md
- slipstream/core/skills/plan_check.md
- slipstream/core/skills/requirements_clarification.md
- slipstream/core/skills/security_review.md
- slipstream/packs/gamedev/skills/balance_tuning.md

## Capability Map

- web_research   -> WebSearch / WebFetch (Claude Code)
- file_read      -> Read
- file_write     -> Write / Edit
- codebase_search -> Grep / Glob
- code_execution -> Bash / PowerShell
- version_control -> git via shell, gh CLI
- issue_tracking -> not available
- design_lookup  -> not available
- data_query     -> not available
- communication  -> not available

## Deployment

- Host: Claude Code
- Tier-A hooks enabled: credential_guard, destructive_bash_guard
  (in .claude/settings.json). The prototype_guard example hook is NOT
  enabled — no prototypes/ directory exists yet; revisit when one does.
- Tier line: all persona boundaries (defers_to, prohibits like
  override_design_intent) are Tier B — carried by prose, not enforced by hooks.

## Project State

- Decision log: STATE.md
- State file: STATE.md
