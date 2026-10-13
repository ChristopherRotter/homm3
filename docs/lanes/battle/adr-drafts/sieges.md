# ADR draft — Sieges at the seam

- Status: **accepted** by the user at BAT-Q40 (2026-10-13); numbered as an ADR with the seam's
  `shared/` pull request (T5's first step)
- Context: the survey `docs/rules/surveys/E04c-sieges.md`; the code reading
  `docs/formats/installation.md` §7.27; ADR-0013 point 5 (the reserved `siege` fields), ADR-0022
  (the battlefield, derived before the seam)

## Decision

1. **`BattleSetup.siege: Option<Siege>`** (`h3-contracts`), `Some` for every battle at a town,
   with or without a fort (the original's: no surrender or retreat for its defender, +500
   experience for the attacker, the town's terrain; survey D-7). Public. `None` elsewhere; every
   existing constructor takes the default.
2. **`Siege` carries derived facts, not the town's buildings.** The town context (E09; until then
   whoever starts a battle at a town) derives them from the town's buildings and the map, as the
   battlefield is derived (ADR-0022):
   - `faction: ContentKey`: the town's faction; the battle part holds a siege record for it (the
     moat's hexes and damage, the Fortress's second moat row, the Tower's mines instead of a moat,
     the turrets' look) and the picture follows it;
   - `fort: FortLevel` (`None`, `Fort`, `Citadel`, `Castle`): the walls and the gate from a Fort,
     the keep and the moat from a Citadel, the towers from a Castle;
   - `moat: Moat` (`Absent`, `Harmless`, `Damaging`): the map's format decides (survey S-29: a map
     before SoD has a harmless moat and no Stronghold moat), so the battle needs no map version;
   - `structures: u8`: the town's buildings as the turrets count them (the keep's count is 5 +
     structures, a tower's half of it rounded up: a battle rule, T7);
   - `escape_tunnel: bool`: its defender may retreat;
   - `morale: i8`, `luck: i8`: the town's bonuses to the defender's stacks (Tavern, Brotherhood
     of the Sword, Fountain of Fortune), side modifiers like a hero's.
3. **The defending hero's town bonuses** (the Grails, Blood Obelisk, Glyphs of Fear, Brimstone
   Stormclouds) are applied to its `BattleHero` before the seam, like its other bonuses (ADR-0013
   point 6). The battle never restores them; the orchestrator keeps the hero's own skills and, as
   the original, sets the mana after the battle to the smaller of the mana left and the mana before.
4. **No outcome field.** The original writes nothing about the walls back to the town; a capture
   follows the result and is the adventure lane's. ADR-0013 point 5's reserved `siege` outcome field
   is dropped.
5. **Walls, gate, turrets and moat are battle state**, public and in the observation (their hit
   points, the gate's state), added with T5–T7 with their encodings, the wire and the replays; the
   walls' hexes and parts come from the battle part's siege record. Natural obstacles stay before
   the seam (BAT-GEO-003): none in a siege on a SoD map, the usual ones for the pre-SoD Stronghold
   (survey S-29), drawn by whoever derives the siege.

## Consequences

- One `shared/` pull request adds `Siege` and its enums (T5's first step), `cross-lane` for the
  struct literals as T2a was; the battle rejects a `Siege` whose faction has no siege record
  (`SetupError`).
- The town context's derivation (E09) gets a rule list from the survey's §1; until E09, scenarios
  and the CLI give `Siege` directly.
- Experiments need towns on the experiment maps (the writer, `cross-lane`).
