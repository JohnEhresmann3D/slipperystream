---
name: balance_tuning
description: "Adjust game balance values methodically — one hypothesis per change, measured against stated player-experience goals, with every change reversible and logged."
capabilities_needed: [file_read, file_write, data_query]
allows: [propose_value_changes, run_tuning_passes, report_results]
prohibits: [change_difficulty_defaults_post_launch, tune_monetization_adjacent_values, batch_unrelated_changes]
---

# Balance Tuning

## Purpose

Change numbers without lying to yourself about why. The failure mode this
skill prevents: a dozen values nudged in one pass, the game feels different,
and nobody can say which change did it — so the knowledge evaporates and the
next tuning pass starts from superstition.

## Use When

- Playtest feedback or telemetry indicates a system is too easy, too hard,
  too dominant, or too ignorable.
- A new mechanic needs initial values grounded in something better than vibes.

## Procedure

1. **State the experience target first**, in player terms, from the design
   intent: "a first-time player should fail this encounter about once," not
   "reduce boss HP." The number serves the sentence; get the sentence from
   the game_designer if it isn't written down.
2. **Establish the baseline.** What do telemetry (`data_query`) or playtest
   notes say the current values produce? If neither exists, say so — tuning
   without a baseline is guessing, sometimes necessary, never silent.
3. **One hypothesis per change.** Each change gets a written line: the value,
   old → new, and the predicted effect on the experience target. Multiple
   values may move together only when they serve one hypothesis (e.g., a
   damage curve reshaped as a unit).
4. **Predict before you measure.** Write the expected observable outcome
   before looking at results. Post-hoc "that's about what I expected" is how
   tuning knowledge stays superstition.
5. **Measure against the target**, not against "feels better" — unless
   feels-better is the stated target, in which case name whose hands it must
   feel better in.
6. **Log every change** where the project keeps decisions: hypothesis,
   values, result, kept-or-reverted. Reverted changes are the most valuable
   entries; they are the map of what doesn't work.

## Quality Gate

- Every changed value traces to a written hypothesis with a prediction made
  *before* measurement.
- The experience target came from design intent, not from the tuner's taste.
- The change log entry exists and includes reverted attempts.
- Nothing touched is on the role constitution's escalation list (difficulty
  defaults post-launch, monetization-adjacent values) without a human's go.
