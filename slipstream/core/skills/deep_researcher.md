---
name: deep_researcher
description: "Research methodology to apply before proposing designs, decisions, or recommendations — establishes what is true before arguing about what to do."
capabilities_needed: [web_research, file_read]
allows: [propose_findings, flag_gaps]
prohibits: [make_final_decisions, lock_scope]
---

# Deep Researcher

## Purpose

Produce a defensible picture of reality before anyone designs against it. The
output of this skill is *findings with confidence levels*, never a decision.

## Use When

- A design, plan, or recommendation is about to be made and the underlying
  facts have not been checked in this project.
- Someone (human or persona) states a load-bearing claim you cannot verify
  from what you've already read.

## Procedure

1. **Frame the question.** Write down, in one or two sentences, what decision
   this research serves. Research without a consuming decision is procrastination.
2. **Inventory what the project already knows.** Read local artifacts first
   (`file_read`): prior decisions, specs, state files. Half of most research
   questions are already answered locally and contradicting a recorded answer
   by accident is worse than not researching at all.
3. **Go outside only for what's still open** (`web_research`). Prefer primary
   sources. Note the date of everything — a stale source stated confidently is
   how research goes wrong.
4. **Triangulate anything that matters.** A claim that changes the decision
   needs two independent sources or an explicit "single-source, unverified"
   label.
5. **Write findings, not conclusions.** For each finding: the claim, the
   source(s), your confidence (verified / corroborated / single-source /
   inferred), and what it means for the framing question.
6. **Flag the gaps.** List what you could not establish and what it would take
   to establish it. An honest gap list is a deliverable, not an apology.

## Quality Gate

- Every load-bearing claim carries a confidence label.
- Local project artifacts were read before external search.
- The output contains zero recommendations phrased as decisions. If asked
  "so what should we do," the answer routes to the persona that owns the
  decision.
