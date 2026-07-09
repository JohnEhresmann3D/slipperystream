---
name: security_review
description: "Review a change or design for security impact — secrets, trust boundaries, injection, authorization, and blast radius — and report findings with severity."
capabilities_needed: [file_read, codebase_search]
allows: [report_findings, assign_severity, recommend_mitigations]
prohibits: [approve_release, weaken_protections, handle_live_credentials]
---

# Security Review

## Purpose

A structured pass over a change or design asking one question five ways:
*what could someone make this do that its author didn't intend?* Output is
findings with severities and recommended mitigations — never a sign-off, which
belongs to a human (base constitution §2.3).

## Use When

- A change touches authentication, authorization, secrets, user input
  handling, or anything network-facing.
- A new dependency, integration, or externally reachable surface is added.
- A human asks for one.

## Procedure

1. **Map the trust boundaries** in the changed area: where does data cross
   from less-trusted to more-trusted context? Every crossing is a review site.
2. **Secrets sweep** (`codebase_search`): credentials, tokens, or keys in
   code, config, logs, error messages, or test fixtures.
3. **Input handling:** for each externally influenced input, trace where it
   goes. Look for injection (query, command, path, template), unvalidated
   deserialization, and length/size assumptions.
4. **Authorization, not just authentication:** for each action, who *can*
   trigger it versus who *should*. Check the deny path, not just the allow path.
5. **Blast radius:** if this component is compromised anyway, what does it
   reach? Flag anything holding more privilege than its job requires.
6. **Report:** each finding gets a location, an attack sketch (one or two
   sentences of how it's abused), a severity (critical / high / medium / low /
   informational), and a recommended mitigation.

## Quality Gate

- Every finding names a location and an abuse path — "this looks unsafe" with
  neither is a hunch, not a finding.
- The report explicitly states what was *not* reviewed, so absence of findings
  isn't mistaken for assurance.
- No live secret values appear in the report itself.
