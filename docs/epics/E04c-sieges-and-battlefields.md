# E04c — Sieges and special battlefields

Status: **in progress** (refined 2026-10-13, BAT-Q37; increment 1: T1 done, waiting for the user's review of the ADR draft, BAT-Q38; then T2) · Follows E04b (closed 2026-10-12) · Part of the former E04 (OPEN-QUESTIONS Q27): E04a heroes in battle, E04b battle magic, **E04c** sieges and special battlefields (this one).

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

- [x] **T1 Battlefield survey and ADR draft** — done 2026-10-13: the code reading
  (`installation.md` §7.26: the setup 0x4643B0, morale and luck by field, the native bonus, the
  mastery raise, the obstacle choice, the boats' hexes), the survey `docs/rules/surveys/E04c-battlefields.md`
  (the twelve battlefields, the sources compared, disputes D-1 to D-4), the ADR draft
  `docs/lanes/battle/adr-drafts/battlefields.md`; the review's questions BAT-Q38.
- [ ] **T2 The battlefields** (refined below; two pull requests: the seam in a `shared/` one first).
- [ ] T3 Experiments for the battlefields (D-1, D-2, D-4; the `.h3m` writer needs the special
  ground objects, the adventure lane's `h3-map`).
- [ ] T4 Siege survey and seam (the siege input on the setup and outcome: fort level, walls, gate,
  towers, moat, the town's faction; an ADR amendment or its own ADR).
- [ ] T5 Walls, gate and moat (the layout, movement and stops, shooting at and through walls, the
  moat's damage, the Tower's mines).
- [ ] T6 The Catapult, Ballistics and Earthquake (S-08).
- [ ] T7 Arrow towers (S-06, S-07; Artillery's control).
- [ ] T8 The siege's other rules (no retreat or surrender for the besieged, the spell cost's town
  armies, Teleport's and the summon's siege limits, the outcome: walls, capture).
- [ ] T9 Workload and measurements; T10 closure (the independent review, as E04b's T11).

### T2 — The battlefields (refined 2026-10-13; after the review of the ADR draft)

- **T2a, the seam** (`shared/battlefield-seam`, merged first): `BattleSetup.battlefield`
  (`Option<ContentKey>`) and `.anti_magic` in `h3-contracts`, documented and defaulted (every
  constructor outside the battle lane uses the default); BACKLOG's E04c row `in-progress`.
- **T2b, the battle** (`battle/e04c-t2`, `cross-lane` for `h3-protocol`): the battle part's
  `[[battlefield]]` (pack format v36) and the obstacle types' battlefields; the harvester reads the executable's two
  obstacle tables (types and fields) and gap-fills the battlefields' effects; packs regenerated
  (`base` shore, Magic Plains, Cursed Ground, deck, boats; `sod` the seven); the rules: morale and
  luck, the native bonus, the magic limits and the anti-magic garrison, the mastery raise, the
  generator's type choice and the boats' hexes; setup errors; the observation, its encoding and
  the wire; scenarios and the CLI take a battlefield; catalog rules BAT-TER-002 onwards with tests;
  golden replays and the bench re-recorded with the reason.
- Acceptance: every effect of the survey's §1 has a rule with its sources and a test; D-1 and D-2
  follow the user's answer to BAT-Q38; battles without a battlefield replay unchanged apart from
  the hashes; `verify --with-original-data` green.

## Design

The ADR draft [`docs/lanes/battle/adr-drafts/battlefields.md`](../lanes/battle/adr-drafts/battlefields.md)
(numbered when the user accepts it). ADR-0012 (modifiers), ADR-0013 (the hero seam, point 5) and
ADR-0014 (magic) still apply.

## Exit criteria

1. The battlefield ADR accepted; the siege seam accepted at its increment.
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
