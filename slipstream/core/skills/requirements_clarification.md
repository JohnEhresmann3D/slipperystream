---
name: requirements_clarification
description: "Surface the questions hiding inside a request before work begins — resolve ambiguity by asking, assuming-with-labels, or narrowing, never by silently guessing."
capabilities_needed: [file_read]
allows: [ask_questions, propose_interpretations, record_assumptions]
prohibits: [invent_requirements, treat_assumptions_as_confirmed]
---

# Requirements Clarification

## Purpose

Most failed work isn't executed badly — it answers a different question than
the one asked. This skill is the checkpoint between receiving a request and
committing effort to an interpretation of it.

## Use When

- A request could reasonably mean two or more different things.
- A request specifies a solution but not the problem ("add a cache" — for
  what pain?).
- Partway through work, you notice you've been assuming something the
  requester never said.

## Procedure

1. **Restate the request** as you understand it, including the implied parts
   you'd otherwise fill in silently.
2. **List the genuine ambiguities** — points where different answers lead to
   materially different work. Skip trivia; asking ten questions to avoid
   thinking is as bad as asking none.
3. **Sort each ambiguity into one of three bins:**
   - **Ask** — the answer changes the outcome and only the requester knows it.
   - **Assume and label** — a sensible default exists; state it in the output
     ("assuming X; say the word if not") and proceed.
   - **Narrow** — deliver the unambiguous subset now, defer the rest.
4. **Batch the asks.** One round of well-chosen questions beats a drip of
   interruptions. If the human is unavailable, convert asks into labeled
   assumptions and flag them prominently in the deliverable.
5. **Record what was resolved** where the project keeps decisions, so the same
   ambiguity isn't re-litigated next session.

## Quality Gate

- No requirement in the final understanding was invented — each traces to the
  request, an answer, or a labeled assumption.
- Labeled assumptions are visible in the deliverable itself, not buried in a
  process note.
