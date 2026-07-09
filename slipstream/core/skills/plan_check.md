---
name: plan_check
description: "Forced-perspective review of a plan or design in a single pass — critic, minimalist, skeptic, and reverser — replacing the multi-agent pow-wow at a fraction of the cost."
capabilities_needed: [file_read]
allows: [challenge_plan, recommend_revisions]
prohibits: [approve_own_plan_without_running_this, rewrite_plan_during_review]
---

# Plan Check

## Purpose

The benefit of convening five personas to argue about a plan, obtained inside
one response: four deliberately hostile reads of the same plan, each from a
stance the author didn't hold while writing it. This skill exists because the
mandatory multi-persona pow-wow was cut (spec §10) — this is its replacement,
and running it is not optional where the pow-wow used to be.

## Use When

- A plan from `planning_and_scoping` is about to be presented for acceptance.
- Any decision that would be expensive to reverse is about to be recommended.

## Procedure

Read the plan once, then produce four short, separate passes. Each pass must
find something or explicitly state it looked and found nothing — silence is
not a pass.

1. **Critic** — attack the reasoning. Where does the plan assert instead of
   argue? Which step depends on a claim nobody verified? What's the strongest
   version of the case against this plan?
2. **Minimalist** — attack the size. What could be deleted and still meet the
   definition of done? Which step exists because it's conventional rather than
   necessary? What's the 20% that delivers the 80%?
3. **Skeptic** — attack the assumptions. List every "this will probably work"
   hiding in the plan. For the top two, what happens if they're false, and
   would we find out early or late?
4. **Reverser** — argue for the opposite. Sketch the case for *not* doing
   this, or doing the inverse. If the reversal case is embarrassingly weak,
   say so — that's signal the plan is sound, and it must be earned, not assumed.

Then close with a **verdict**: proceed as-is, proceed with named revisions, or
rework. Revisions go back to the plan's author (which may be you, in a
different persona) — the reviewer does not rewrite the plan mid-review.

## Quality Gate

- All four passes are present and none is a rubber stamp ("looks good" with no
  evidence of the stance being applied).
- The verdict names specific revisions or states clearly why none are needed.
