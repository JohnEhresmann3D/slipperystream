# Optional Reference Runtime (Raw API Deployments)

**Not installed by default. Nothing in `core/`, `factory/`, or `packs/`
depends on this directory, references it, or assumes it is running.**

## What This Is

If you drive Slipstream personas through the raw Anthropic API, *you* own the
agent loop — which means Tier-A enforcement is genuinely achievable: your code
sits between the model and every tool call and can actually block, gate, and
log. This directory is the home for a reference runtime for that one use
case: the agent loop, circuit breaker, HITL gate manager, and audit trail
from the v2.x architecture live here going forward, as optional code a
caller may adopt.

## What This Is Not

This is not part of the framework. The v2.x mistake — documented in the spec
(§2, §10) — was shipping this machinery as if it enforced anything inside
hosts that never execute it. A Python circuit breaker that no platform runs
is Tier C dressed as Tier A. If you are deploying to a Claude Project or to
Claude Code, close this directory; it has nothing for you. (Claude Code users:
hooks give you Tier A natively — see `deploy/claude-code/`.)

## Responsibilities of a Conforming Runtime

If you build (or port the v2.x code into) a runtime here, it should:

1. **Load the manifest** and inject the active persona, its constitution
   stack, and active skills into the system prompt.
2. **Enforce `prohibits` mechanically** — inspect each tool call the model
   requests and refuse those matching a persona's prohibited patterns, before
   execution.
3. **Gate `hitl_required` mechanically** — pause the loop and require an
   out-of-band human approval before executing matching actions, and persist
   rejections so they stay sticky across sessions (base constitution §5 gets
   real state here).
4. **Audit** — log every tool call, block, and human decision.

Everything the constitution states behaviorally, a runtime here may
additionally enforce mechanically. The prose remains the source of truth; the
code is a check on it, not a replacement for it.
