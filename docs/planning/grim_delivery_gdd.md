# GRIM DELIVERY
### Game Design Doc — 48hr Jam Prototype

---

## 1. High Concept

You play **Death**, riding a bike (or pale horse) down a suburban street, delivering **death notices** to houses instead of newspapers. Hit the right house, cleanly collect a soul. Hit the wrong house, and you've just marked an innocent person for death by clerical error.

**Pitch line:** *Paperboy, but you're the Grim Reaper and the paper route is your job description.*

---

## 2. Core Pillars

1. **Arcade tightness** — the bike-riding, throwing, and dodging must feel good in the first 10 seconds. This is the make-or-break of the whole prototype.
2. **One twist, well executed** — the "wrong address" mechanic is our single point of differentiation from vanilla Paperboy. Don't dilute it with more systems.
3. **Dark comedy tone, light execution** — morbid premise, silly/slapstick presentation. Never gross, always absurd.

---

## 3. Core Loop

1. Player rides down a scrolling street (auto-scroll or player-controlled speed — see Open Questions).
2. Clipboard shows the day's delivery list: names + addresses marked for death.
3. Player throws death notices at the correct houses while dodging obstacles.
4. Correct hit → soul collected, quota ticks up, satisfying FX/sound.
5. Wrong hit → innocent person marked, comedic "oops" beat, possible consequence (see 4.3).
6. Reach end of street before losing all "misses" / running out of route → level cleared.
7. Score tallied: souls collected, wrong deliveries, style/combo bonus.

---

## 4. Mechanics

### 4.1 Movement & Throwing
- Bike moves left/right across 3–5 lanes of a scrolling street; forward motion is automatic or speed-controlled by player (pick one, see Open Questions).
- Single throw button, aimed by player position/lane (Paperboy-style: your lane determines your throw arc, no free-aim needed for scope reasons).
- Optional: hold-to-charge throw for a stronger/farther notice — **stretch goal, cut first if behind schedule.**

### 4.2 The Quota
- Each level has a target number of correct deliveries.
- HUD shows quota progress (e.g. "3 / 7 souls collected").
- Missing the quota by the end of the route = fail state (or partial score, softer for a jam — recommend **no hard fail**, just a worse score, so playtesters always see the whole level).

### 4.3 Wrong Addresses (the twist)
- Not every house on the clipboard is legit — some addresses are outdated/misfiled (bureaucracy joke).
- Houses give a **light environmental tell** so skilled players can learn to read them:
  - Correct target: subtle visual cue (porch light off, curtains drawn, a black cat on the step — pick 1 consistent tell for scope).
  - Wrong target: opposite/absence of the tell.
- Hitting a wrong house:
  - MVP version: just a negative score hit + a funny sound/VFX (person runs out yelling). **Keep it here for 48 hours.**
  - Stretch: that NPC starts chasing you on their own bike for the rest of the level, adding a dodge-obstacle. Fun, but cut if time is short — it needs new AI.

### 4.4 Obstacles
Reskinned Paperboy classics, cheap to reuse across levels:
- Kids on bikes, sprinklers, dogs, parked cars pulling out.
- Reskin flavor: a few obstacles get a death-themed coat of paint (a hearse idling, a black cat crossing) — cosmetic only, doesn't need new logic.
- Colliding with an obstacle = lose control briefly / drop your current notice, NOT instant fail. Keep the game forgiving.

### 4.5 Scoring
- Base points per correct delivery.
- Bonus for combo streaks (consecutive correct hits).
- Penalty for wrong deliveries.
- Style bonus ideas (stretch, only if time allows): trick shots, long-distance throws, near-miss obstacle dodges.

---

## 5. Level Structure (MVP)

- **1 street type**, reused with increasing density/difficulty (more houses, tighter obstacle spacing, more decoy addresses).
- Recommend **3 short levels** (60–90 sec each) rather than 1 long level — easier to tune difficulty curve and gives playtesters a sense of progression in a jam demo.
- End-of-route summary screen: souls collected / quota, wrong deliveries, score, maybe a one-line flavor text from Death's manager (ties to the bureaucracy joke without needing a new scene).

---

## 6. Tone & Presentation

- **Visual style:** flat, bright, slightly retro suburban-Americana (green lawns, identical houses) with Death as an incongruously cartoony skeleton-on-a-bike. Contrast is the joke — cheerful street, morbid job.
- **Audio:** upbeat, jingly delivery-boy whistle music undercut by occasional ominous sting on a correct soul collection. Wrong delivery = slide-whistle/cartoon "oops" sound.
- **UI voice:** clipboard and HUD text written like corporate delivery-app copy ("Route 12B — 7 stops remaining") for the bureaucracy flavor, without needing extra content.

---

## 7. Scope Guardrails for 48 Hours

**Must-have (cuts the game if missing):**
- Lane-based bike movement + throw
- 1 street loop, reusable across levels
- Correct/wrong house distinction with 1 consistent visual tell
- Quota + score HUD
- 3 short levels
- End screen

**Should-have (cut second):**
- 2–3 obstacle types with basic collision-slowdown
- Combo scoring
- Simple win/lose framing per level

**Nice-to-have (cut first if behind):**
- Chasing angry NPC after a wrong delivery
- Charge-throw mechanic
- Style bonuses / trick scoring
- Manager flavor-text between levels

---

## 8. Open Questions for the Team

1. **Auto-scroll vs. player-controlled speed** — auto-scroll is simpler to tune and keeps pace consistent for a jam demo; player speed control adds a skill dimension but costs more tuning time. *Recommend auto-scroll for 48 hours.*
2. **Fail state severity** — hard fail on missed quota vs. always-let-them-finish-with-a-score. *Recommend the latter for smoother playtesting.*
3. **Perspective** — top-down (classic Paperboy) vs. side-scrolling. Top-down matches the reference most closely and is easier to read lanes/houses; side view could look punchier but adds art complexity. *Recommend top-down.*
4. **Engine/tooling** — whichever the team is fastest in; nothing here requires anything exotic (2D, minimal physics, simple collision).

---

## 9. Reference

- *Paperboy* (1985, Atari) — core route/throw/dodge loop.
- Tone reference: darkly comic bureaucratic-afterlife humor (think *The Good Place*'s admin gags, or DMV-style absurdity applied to Death's job).

---

*Doc owner: [you]. Feedback and scope changes should route back through this doc before implementation to keep the 48-hour build honest.*
