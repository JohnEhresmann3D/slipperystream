---
name: planning_and_scoping
description: "Turn a goal into a bounded, sequenced plan with explicit non-goals, risks, and a definition of done — before any execution begins."
capabilities_needed: [file_read]
allows: [propose_plan, sequence_work, flag_risks]
prohibits: [begin_execution_before_plan_accepted, expand_scope_without_signoff]
---

# Planning and Scoping

## Purpose

Convert "we want X" into a plan someone could disagree with — concrete enough
that its wrongness would be visible, small enough that its end is defined.

## Use When

- Starting any piece of work expected to span more than one sitting or touch
  more than one artifact.
- Work in progress has drifted and needs re-anchoring (base constitution §4).

## Procedure

1. **State the goal in the requester's words**, then in your own. If the two
   differ, resolve that first — see `requirements_clarification`.
2. **Write the non-goals.** What this work will *not* do is the half of scope
   that prevents drift. A plan without non-goals is a wish.
3. **Define done.** One or more checks that a third party could run to confirm
   completion. "Done when it works" is not a definition.
4. **Decompose into steps that each produce something inspectable.** A step
   whose output can't be looked at can't be verified, and strings of them are
   where plans silently fail.
5. **Order by risk, not convenience.** Front-load the step most likely to
   invalidate the plan, so failure happens cheaply.
6. **Name the risks and their tripwires.** For each real risk: what early
   signal would indicate it's materializing, and what the fallback is.
7. **Present the plan for acceptance before executing.** Scope changes after
   acceptance go back through this step — they are not absorbed silently.

## Quality Gate

- Non-goals section exists and is non-empty.
- Definition of done is checkable by someone other than the author.
- The riskiest step is scheduled early, and the plan says why it's the riskiest.
