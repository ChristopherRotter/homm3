# E04c — Sieges and special battlefields

Status: **in progress** (refined 2026-10-13, BAT-Q37; increment 1, T1 and T2, merged (#164, #170, #171); increment 2 closed: T3 (#179), T4 (#180), BAT-Q39 and BAT-Q40 answered; increment 3, T5: the seam ADR-0023 (#183), towns on the experiment maps (#184), the reading (BAT-Q41, BAT-Q42), the battle's walls, gate and moat (#187), the siege batch 3c run by
an agent (T5e): increment 3 done (#190); increment 4, T6, refined 2026-10-14 (BAT-Q43)) · Follows E04b (closed 2026-10-12) · Part of the former E04 (OPEN-QUESTIONS Q27): E04a heroes in battle, E04b battle magic, **E04c** sieges and special battlefields (this one).

## Goal

Battles where the original fights them: the special battlefields (the shore, the nine magic
fields, a ship's deck, two boats) and the anti-magic garrison, then sieges (walls, gate, moat,
towers, the Catapult and Ballistics, Earthquake, shooting at and through walls, the town's
defenders). The battle keeps the architecture of E01–E04b: pure logic, validated commands,
events with visibility, observations, determinism, at least 100 k steps/s/core on the `full`
scenario.

## Decisions at the refinement (2026-10-13, user, OPEN-QUESTIONS BAT-Q37)

1. **The battlefields first.** Increment 1 surveys and builds the special battlefields; sieges come
   in later increments, cut at each checkpoint (the town input waits for E09's refinement anyway).
2. **The seam is reviewed first.** T1 writes the survey and an ADR draft, then stops for the
   user's review in the session; T2 builds on what the user accepts.
3. **The wiki** is read this session (the user's approval; the global HANDOFF, "Sources").

Unchanged: one epic in increments with a user checkpoint each, only the current increment refined
(as E04b, Q41); Earthquake belongs here (Q27); a quirk is confirmed for SoD before it gets a flag
(Q30); experiment maps are for SoD (Q28); a code reading ranks as an observation and a reading at
risk gets a confirmation row (`docs/04-rules-and-sources.md` §2); token efficiency beats speed.

## Starting point (from E04b)

- `BattleSetup.terrain` gives the native bonus (BAT-TER-001) and the obstacles' terrain
  (BAT-GEO-004, VCMI's generator); nothing knows a special battlefield. ADR-0013 point 5 reserved a
  battlefield kind and the siege fields.
- The magic limits (`MagicLimits`: the Orb of Inhibition, Recanter's Cloak) and the hero's mastery
  per spell from the seam (`h3_heroes::spell_masteries`) are where the fields' magic joins; the
  readings of §7.21 to §7.24 already name every place the original reads the field.
- Packs hold 78 obstacle types (VCMI's, gap-filled); the 47 that only special battlefields use are
  missing. The client lane reads the executable's obstacle and background tables
  (`E17-design.md` §5).
- Leads in the battle queue's inbox ("E04c" bullets of the E04b closure and the review) and
  `E04-battle.md` §7 (siege inputs); quirks S-06, S-07, S-08 (towers, Earthquake), S-22 (underground
  battles, confirmed by the reading).

## Non-goals

The derivation of the battlefield from the adventure map (the adventure lane's `h3-game`, with
the survey's §4); battlefield pictures and their variants (the client lane, CLT-Q4); creature
banks' fixed layouts; HotA's fields (Cracked Ice, Dunes, Fields of Glory). AI (E05/E06).

## Tasks (outline; refine at the start of each increment; keep status current)

**Increment 1** (BAT-Q37): T1, then T2 after the user's review.

- [x] **T1 Battlefield survey and ADR draft** — done 2026-10-13 (record: [`E04c-log.md`](E04c-log.md)): the code reading
  (`installation.md` §7.26: the setup 0x4643B0, morale and luck by field, the native bonus, the
  mastery raise, the obstacle choice, the boats' hexes), the survey `docs/rules/surveys/E04c-battlefields.md`
  (the twelve battlefields, the sources compared, disputes D-1 to D-4), the ADR draft
  `docs/lanes/battle/adr-drafts/battlefields.md`, accepted as ADR-0022; the review's questions BAT-Q38.
- [x] **T2 The battlefields** — built 2026-10-13, adjusted to BAT-Q38 (ADR-0022; D-1 and D-2 as the code, S-28 a quirk candidate): T2a `shared/battlefield-seam`, T2b `battle/e04c-t2`; BAT-TER-002 to BAT-TER-006, BAT-GEO-004 amended; the obstacle types harvested from the executable; record: [`E04c-log.md`](E04c-log.md).
**Increment 2** (the user, 2026-10-13: T3's preparation, then T4; the map writer's change as a
`cross-lane` pull request from battle; an agent plays the batch on the capture harness).

- [x] **T3 Experiments for the battlefields** — done 2026-10-13 (record: [`E04c-log.md`](E04c-log.md)): the writer's
  special grounds and coast flag (`h3-map`, `cross-lane`), `[[map.ground]]` and `--only`; batches 3a and
  3b run by an agent: D-1, D-2 and D-4 as the code, the defender's tile decides; S-28 not a
  quirk (BAT-Q39).
- [x] **T4 Siege survey and seam** — done 2026-10-13 (record: [`E04c-log.md`](E04c-log.md)): the code reading
  `installation.md` §7.27 (the town and fort level, the moat, the turrets, the war machines, the
  defending hero's and the stacks' town bonuses, the controls, the experience, the siege's
  obstacles), the survey `docs/rules/surveys/E04c-sieges.md` (D-5 to D-7, S-29), the ADR draft
  `docs/lanes/battle/adr-drafts/sieges.md`, accepted at BAT-Q40 as ADR-0023.
**Increment 3** (the user at BAT-Q40, 2026-10-13: T5, starting with the seam's `shared/` pull
request and towns on the experiment maps; siege batches run by an agent).

- [x] **T5 Walls, gate and moat** — done 2026-10-14 (record: [`E04c-log.md`](E04c-log.md)): T5a (#183),
  T5b (#184), T5c and T5d (#187); T5e the batch 3c by an agent: every rule as the code, D-8 and D-10
  confirmed; BAT-SIE-003 to 008 `verified`.
**Increment 4** (the user, 2026-10-14: the next sensible increment, built and confirmed at its
checkpoint; BAT-Q43 at its start).

- [ ] T6 The Catapult, Ballistics and Earthquake (S-08): refined below.
- [ ] T7 Arrow towers (S-06, S-07; Artillery's control).
- [ ] T8 The siege's other rules (no retreat or surrender for the besieged, the spell cost's town
  armies, Teleport's and the summon's siege limits, the outcome: walls, capture).
- [ ] T9 Workload and measurements; T10 closure (the independent review, as E04b's T11).

### T5 — Walls, gate and moat (refined 2026-10-14, increment 3)

- **T5a, the seam** (`shared/siege-seam`, #183, merged first): `BattleSetup.siege` (`Siege`,
  `FortLevel`, `Moat`), ADR-0023 (BAT-Q40, BAT-Q41: the faction by handle, the turrets' count by
  the code, the moat by the game's version), BAT-STUB-2 until the battle reads it.
- **T5b, towns on the experiment maps** (`battle/e04c-t5b`, `cross-lane` for `h3-map`): the
  writer's town (class 98, buildings, garrison, owner), `[encounter.opponent.town]` with a visiting
  hero.
- **T5c, the reading** (`installation.md` §7.28; the survey's D-8 to D-10; BAT-Q42: the code).
- **T5d, the battle** (`battle/e04c-t5`, stacked on T5a; `cross-lane` for `h3-protocol`): the
  battle part's siege records per faction (the moat's damage and rows, the Tower's mines) and the
  siege's geometry (wall, gate and moat hexes, harvested from the executable); setup errors; the
  state (the gate) and its observation, encodings and wire; the walls' hexes, the gate's passage,
  opening and closing; the moat's stop, damage on entering and on acting in place, and −3
  defense; the Tower's hidden mines; the shot's wall test; scenarios, replays and the CLI; catalog
  rules BAT-SIE-001 onwards; golden replays and the bench with the reason. The walls' hit points
  join with T6, which damages them (ADR-0023 point 5). **Done:** BAT-SIE-001 to BAT-SIE-008
  (the log has the record).
- **T5e, a siege batch run by an agent** on the capture harness: the walk into the moat (stop,
  damage, defense), the gate, the Tower's mines; D-8's and D-10's confirmation rows. **Done:**
  batch 3c (`experiments/agent/siege.py`, hot seat), every value as the code and the engine.
- Acceptance: every rule of §7.28 has a catalog rule with its sources and a test; battles without
  a siege replay unchanged apart from the hashes; the Tower's mines never reach the attacker's
  observation; `verify --with-original-data` green.

### T6 — The Catapult, Ballistics and Earthquake (refined 2026-10-14, increment 4)

- **T6a, the seam** (`shared/catapult-seam`, #191, merged first): `CatapultControl { level }`, the
  hero's Ballistics level; `HeroEffect::Ballistics`; ADR-0023 point 6 (BAT-Q43).
- **T6b, the reading** (`installation.md` §7.29; the survey's D-11 to D-15, S-08 confirmed): the
  Catapult's stack and turn (0x473C00), the wall shot (0x445D30, 0x445EC0), the parts' damage
  (0x465990), Earthquake (0x5A7C80). **BAT-Q43 (the user): the code on D-11 to D-15; S-08's fix
  keeps the absent parts in Earthquake's draw and harms no stack; the Cyclops' wall shot in T6.**
- **T6c, the battle** (`battle/e04c-t6`, stacked on T6a; `cross-lane` for `h3-protocol`): the
  battle part's data (the targets, their kinds and wall hexes, the miss and automatic lists, the
  Catapult's hex, the Ballistics table, the parts' hit points per town, harvested from the
  executable, `BALLIST.TXT` and `WALLS.TXT`); the Catapult creature harvested (a siege machine) and
  the Cyclops' wall shots (gap-fill); the Ballistics skill's effect; Earthquake's effect; the
  battle: the Catapult set up for an attacker with a hero at a fort, first in the round's order,
  automatic without Ballistics, commanded with it; the wall-shot command (the Catapult, an
  attacker's Cyclops); the shot (damage, hit, miss, re-aim), the parts' hit points, a destroyed
  wall's hex, a destroyed gate (passage, moat, the shot's line); Earthquake (a siege with a fort,
  either side; the code's draw, absent parts soak hits: BAT-QUIRK-22); observation, encodings, the
  wire, replays; catalog rules from BAT-SIE-009; golden replay and the bench with the reason. The
  turrets' part (a destroyed tower's stack) joins T7.
- **T6d, a Catapult batch run by an agent** on the capture harness: the automatic aim (D-13),
  a destroyed Mid-Upper Wall's hex (D-11), the gate hit and destroyed (passage, the drawbridge's
  hex no longer moat), Earthquake's hits, a miss (D-12) as far as one shows.
- Acceptance: every rule of §7.29 has a catalog rule with its sources and a test; battles without
  a siege replay unchanged apart from the hashes; walls and the gate agree with movement, the
  legal moves and the shot's line; `verify --with-original-data` green.

## Design

[ADR-0022](../adr/0022-battlefields.md) (accepted at BAT-Q38), [ADR-0023](../adr/0023-sieges.md)
(accepted at BAT-Q40, BAT-Q41). ADR-0012 (modifiers), ADR-0013 (the
hero seam, point 5) and ADR-0014 (magic) still apply.

## Exit criteria

1. The battlefield ADR accepted (done: ADR-0022); the siege seam accepted at its increment.
2. Every battlefield and siege rule has catalog rules with labelled sources and tests; disputed
   ones have an observation fixture (SoD) or an OPEN-QUESTIONS entry with its experiment.
3. Confirmed quirks are in the ledger with flags; S-06, S-07 and S-08 classified by the user.
4. Earlier invariants still property-tested, plus: hidden siege objects (the Tower's mines) never
   reach the other side's observation; walls and the moat agree with movement and the legal moves.
5. Golden replays with a special battlefield and a siege; at least 100 k steps/s/core on the
   `full` scenario; `bench-check` re-baselined with the reason.
6. At least one E04c experiment batch played by the user and recorded.
7. `cargo xtask verify` green on Linux and Windows, including `--with-original-data`.

## Risks

- **The siege's breadth** (walls, towers, moat, Catapult, the town's defenders) and its town input,
  which E09 has not refined. Mitigation: battlefields first; the siege seam surveyed on its own.
- **The seam reaches three lanes** (adventure derives the battlefield, server maps it, client draws
  it). Mitigation: the ADR draft first, one `shared/` pull request, notes in each lane's queue.
- **Readings at risk** (D-1, D-2): the code against VCMI and the wiki. Mitigation: the game's own
  texts agree with the code; confirmation rows in T3.
