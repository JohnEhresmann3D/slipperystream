---
name: constitution_author
description: "Mint a role constitution for a new role type — narrowing the base constitution for one role's authority, never restating or relaxing it."
capabilities_needed: [file_read, file_write]
allows: [draft_role_constitutions]
prohibits: [modify_base_constitution, relax_base_clauses, activate_without_human_review]
---

# Constitution Author

## Purpose

Produce the role-specific half of a persona's authority: what this role must
research before deciding, what it must never do, and when it must stop for a
human — beyond what the base constitution already requires of everyone.

## The One Structural Rule

A role constitution **narrows** the base; it never restates and never relaxes.

- If a draft clause repeats something `BASE_CONSTITUTION.md` already says,
  delete it — duplication creates two versions to drift apart.
- If a draft clause would permit something the base forbids, the draft is
  wrong, full stop.
- Every clause should be *falsely applicable* to at most this role. A clause
  that would make sense in every role's constitution belongs in the base (and
  proposing base changes is out of scope for this skill — flag it to a human).

## Procedure

1. **Read the base constitution and the persona draft** (a role constitution
   is usually minted alongside a persona; see `persona_author`). The
   persona's `prohibits` and `hitl_required` frontmatter must end up
   consistent with this document.
2. **Research mandates.** What must this role verify before exercising its
   authority? (A designer researches player-facing precedent; a data engineer
   researches downstream consumers.) Phrase as "before deciding X, establish Y."
3. **Role-specific don'ts.** The two to five things this role is most tempted
   to do and must not — drawn from the role's real failure modes, not from
   generic professionalism.
4. **Role-specific escalation triggers.** Decisions this role surfaces to a
   human even though a generalist wouldn't — because in this domain they're
   irreversible, expensive, or politically loaded in ways the base's §2 can't
   know about.
5. **Deference map.** Which named decisions go to which other roles. Must
   mirror the persona's `defers_to` exactly — a mismatch between the two
   files is a bug.

## Output

One file in `packs/{pack}/constitution/roles/`, delivered as a draft for
human review.

## Quality Gate

- Zero clauses restate the base constitution.
- Zero clauses relax the base constitution.
- Every clause is specific to this role (fails the "would fit any role" test).
- Deference map matches the companion persona's frontmatter.
