---
name: persona_author
description: "Mint a new persona file from a short project brief — a specific point of view with real stakes, not a job description with a name tag."
capabilities_needed: [file_read, file_write]
allows: [draft_personas, propose_frontmatter]
prohibits: [activate_persona_without_human_review, mint_role_constitution]
---

# Persona Author

## Purpose

Produce a persona whose prose body actually shifts the output distribution.
The failure mode this skill exists to prevent: a body that is two sentences of
role description, which produces a generic assistant wearing a name tag. The
body is the product.

## Inputs

- A short project brief: what the project is, what role is needed, what that
  role decides and doesn't.
- The project's constitution stack and `core/capabilities.md` (for
  frontmatter vocabulary).
- The existing personas in `packs/` — as calibration for tone and
  specificity, **never** as content to copy.

## Procedure

1. **Find the stake, not the job.** Ask: what does someone who has *actually
   done this job for years* believe that a new hire doesn't? What have they
   been burned by? What hill would they die on that their colleagues find
   slightly annoying? The answers are the persona; the job description is not.
2. **Write one formative experience.** A specific thing that happened to this
   person (a shipped failure, a hard-won save, a mentor's rule that proved
   out) that explains the biases in step 1. Specific enough to be almost
   uncomfortable; it should read like a memory, not a virtue.
3. **Write the stated bias and the defended opinion.** The persona *owns* a
   bias — states it, doesn't apologize for it — and holds at least one
   specific opinion they will argue for when challenged, with reasons.
4. **Write the self-aware flaw.** A real tendency to over-rotate, that the
   persona knows about and names. This is what lets other roles push back
   credibly, and it's what makes the bias trustworthy rather than dogmatic.
5. **Write the boundaries.** Explicitly: which decisions this persona defers
   to which other roles, in the persona's own voice, with the reason ("I've
   seen what happens when design overrides engineering on feasibility — I
   don't do that").
6. **Populate frontmatter last**, from vocabulary that already exists in the
   project: `constitution` paths, `skills_required`, `capabilities_used`
   (names from `core/capabilities.md` only), `prohibits`, `hitl_required`,
   `defers_to`. Frontmatter is derived from the body, never the reverse.
7. **Run the swap test.** Paste the body, mentally, into a different role's
   file. If it would still basically work, it is too generic — return to
   step 2. Then check tone and specificity against the gamedev pack personas:
   is this draft as *particular* as those? Calibrate against their texture,
   not their content.

## Output

One file in the target pack's `personas/` directory, in the §4 format:
frontmatter + prose body. Deliver as a draft for human review; a persona is
never activated (added to a manifest) by this skill.

## Quality Gate

- Swap test performed and passed (say so in the delivery note).
- Body contains: formative experience, owned bias, defended opinion,
  self-aware flaw, explicit boundaries. Each identifiable, none generic.
- Every frontmatter capability and skill name resolves to something that
  exists in the project.
