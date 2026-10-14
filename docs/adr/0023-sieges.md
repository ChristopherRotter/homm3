# ADR-0023: Sieges — the town on the setup as derived facts

- Status: Accepted (user, 2026-10-13, BAT-Q40 and BAT-Q41; drafted at E04c T4); point 6 amended
  2026-10-14 (user, BAT-Q43; E04c T6); point 7 added 2026-10-14 (user, BAT-Q46; E04c T7d)
- Date: 2026-10-13
- Deciders: user; battle lane session (E04c T4, T5)
- Working detail: the survey [`docs/rules/surveys/E04c-sieges.md`](../rules/surveys/E04c-sieges.md),
  the code reading `docs/formats/installation.md` §7.27; the epic `docs/epics/E04c-sieges-and-battlefields.md`

## Context

E04c's sieges need the town a battle is fought at. ADR-0013 point 5 reserved optional `siege`
fields on the setup and the outcome; ADR-0022 left the siege's field to its increment. In the
original (0x462A20 onwards) every battle at a town is "at a town", with or without a fort: the
town decides the controls, the experience, the defenders' morale and luck and the defending
hero's bonuses; the fort level adds the walls, the gate, the moat and the turrets; the game's
version (RoE, AB or SoD rules: the original's `mapKind`) decides whether the moat exists and does
damage (survey S-29). The town's buildings
belong to the town context (E09, not refined), which will derive the facts the battle needs, as
the adventure lane derives the battlefield (ADR-0022).

## Decision

1. **`BattleSetup.siege: Option<Siege>`** (`h3-contracts`): `Some` for every battle at a town,
   with or without a fort (the original's: no surrender, no retreat without the Escape Tunnel for
   its defender, +500 experience for the attacker, the town's terrain; survey D-7). Public. `None`
   elsewhere; every existing constructor takes `None`.
2. **`Siege` carries derived facts, not the town's buildings.** Whoever starts a battle at a town
   derives them (the town context, E09; until then scenarios and the CLI give them):
   - `faction: FactionId`: the town's faction, a handle of the battle's ruleset (BAT-Q41); the
     battle part holds a siege record per faction (the walls' parts, the moat's hexes and damage,
     the Fortress's second moat row, the Tower's mines instead of a moat, the turrets);
   - `fort: FortLevel` (`None`, `Fort`, `Citadel`, `Castle`): the walls and the gate from a Fort,
     the keep and the moat from a Citadel, the towers from a Castle;
   - `moat: Moat` (`Absent`, `Harmless`, `Damaging`): the game's version decides (under RoE or
     AB rules a moat is harmless and a Stronghold has none; BAT-Q41), so the battle needs no
     version; a Tower's land mines take its moat's place and the value decides their damage;
   - `structures: u8`: the town's buildings as the original counts them for its turrets
     (0x5BF4F0, BAT-Q41); the turrets' counts from it are a battle rule (T7);
   - `escape_tunnel: bool`: the town's defender may retreat;
   - `morale: i8`, `luck: i8`: the town's bonuses to the defender's stacks (the Tavern, the
     Brotherhood of the Sword, the Fountain of Fortune), side modifiers like a hero's.
3. **The defending hero's town bonuses** (the Grails, Blood Obelisk, Glyphs of Fear, Brimstone
   Stormclouds) are applied to its `BattleHero` before the seam, like its other bonuses (ADR-0013
   point 6). The battle never restores them; the orchestrator keeps the hero's own skills and, as
   the original, sets the mana after the battle to the smaller of the mana left and the mana before.
4. **No outcome field.** The original writes nothing about the walls back to the town; a capture
   follows the result and is the adventure lane's. ADR-0013 point 5's reserved `siege` outcome
   field is dropped.
5. **Walls, gate, turrets and moat are battle state**, public and in the observation (their hit
   points, the gate's state), added with E04c T5–T7 with their encodings, the wire and the
   replays; the walls' hexes and parts come from the battle part's siege record. Natural
   obstacles stay before the seam (BAT-GEO-003): none in a siege under SoD rules, the usual ones
   for a Stronghold under RoE or AB rules (survey S-29), drawn by whoever derives the siege.
6. **The Catapult** (amendment, E04c T6, BAT-Q43): the battle sets the attacker's Catapult up
   itself in a siege with a fort when the attacker has a hero (the original gives every hero
   one); the setup's `war_machines` never list it. A side commands it when its hero's
   `war_machines.catapult` is `Some(CatapultControl { level })`, the hero's Ballistics level 1–3
   (`HeroEffect::Ballistics`, derived in `h3-heroes`); without it the Catapult shoots on its own.
   The Ballistics table (the original's `BALLIST.TXT`, levels 0–3) and the walls' hit points
   (`WALLS.TXT`) are battle part data, since the uncontrolled Catapult (level 0) and the Cyclops
   read the same table. Unlike the Ballista's control (ADR-0013 point 4: derived numbers) the
   seam carries the level.
7. **The game's difficulty and the human sides** (amendment, E04c T7d, BAT-Q46): the battle makes
   some choices for a side as the original's computer player does (an automatic turret's target:
   0x41E190, whose desperate mode needs the difficulty above Easy or a human defender,
   `installation.md` §7.31; later the automatic Ballista, T7e). The setup carries the two facts
   those choices read, as the game has them: `BattleSetup.difficulty: u8` (0 Easy to 4
   Impossible) and `BattleSetup.human: [bool; 2]` (whether each side's player is human; false for
   a side without a player). Both are public and battle state. They do not decide who commands a
   side: every side's player commands its stacks as before. The defaults (0, no human) are a
   battle between two computer players on Easy; the game orchestrator sets them from the game
   (the adventure lane), scenarios and replays name them.

## Consequences

- One `shared/` pull request adds `Siege` and its enums, `cross-lane` for the struct literals (as
  ADR-0022's); the battle reads the field from T5 on (BAT-STUB-2 until then) and rejects a `Siege`
  whose faction has no siege record (`SetupError`).
- The town context's derivation (E09) gets a rule list from the survey's §1; how a game gets its
  version is the adventure lane's (the original probably takes it from the map's format: an
  experiment candidate).
- Versions with T5: pack format (the siege records), state hash, replay format, observation
  encoding, C ABI and the wire; golden replays with a siege. Battles without a town play as before.
- Experiments need towns on the experiment maps (`h3-map`'s writer, `cross-lane`).
- Point 7: one `shared/` pull request adds the two fields (`cross-lane` for the struct literals);
  the battle keeps them from E04c T7d on, with the observation, its encoding, the wire, the
  scenario and replay formats.

## Alternatives considered

- The town's buildings and the game's version on the setup: the battle would need the town
  context's building ids and the versions, and would derive what the town context knows.
- The faction by key (`ContentKey`, as ADR-0022's battlefield): factions are core content, and the
  seam already names them by handle (`FactionGroup`; BAT-Q41).
- The defending hero's town bonuses as siege fields the battle applies: ADR-0013 point 6 derives
  hero bonuses before the seam, and the battle would have to undo them.
- A siege outcome (the walls' state): the original keeps nothing of it.
- Point 7: a derived flag per side ("the computer's full tactics": difficulty above Easy or a
  human player) instead of the two facts. Both routines read in T7d test exactly that, but the
  computer's other choices (spells, retreats) read the difficulty in other ways, and the two
  facts are what the game knows.
