# Deploying Slipstream to a Claude Project

A Claude Project (claude.ai) reads Slipstream as project knowledge. There is
no mechanical (Tier A) enforcement available on this host — everything here
operates at Tier B: identity and constraints stated in every session, followed
as instruction. That is the honest ceiling; nothing in these files implies a
guarantee the platform can't back.

## Steps

1. **Create the manifest.** Copy `MANIFEST.template.md` to `MANIFEST.md`,
   fill it in, and add it to the project knowledge. Under *Capability Map*,
   map capabilities to what claude.ai actually has (e.g., `web_research ->
   web search`, `file_write -> not available`) — skills will degrade
   honestly where a capability is unmapped.
2. **Add the constitution stack** to project knowledge:
   - `core/constitution/BASE_CONSTITUTION.md`
   - The role constitution for each active persona.
3. **Add the active personas** (`packs/{pack}/personas/*.md` for the personas
   the manifest lists).
4. **Add the active skills** — all of `core/skills/`, plus any pack skills
   the manifest lists.
5. **Set the project's custom instructions** to something like:

   > Read MANIFEST.md first. Act as the persona it activates (or ask which,
   > if several are active), under the constitution stack it lists. Apply
   > skills from the library when their "Use When" conditions hold.

## What to Expect

- Persona identity, constitution constraints, and skill procedures all work —
  they are Tier B by design.
- `hitl_required` and `prohibits` lists are honored as instruction, not
  enforced by the platform. For work where that distinction matters, use
  Claude Code (hooks) or the API engine instead.
