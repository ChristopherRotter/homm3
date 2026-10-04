# E04a design notes (ephemeral)

The working detail behind [ADR-0013](../adr/0013-heroes-in-battle.md): field lists, formulas and
sizes that guide the implementation. The ADR holds the binding decisions; this note is a handover.

**Pruning rule:** the task that implements a section deletes it in the same change (the code
documentation, the rule catalog and the pack docs then hold the detail). Where the implementation
deviates, the code wins; change the ADR only if a decision changes. Delete this file when E04a closes.
Section numbers follow the ADR. Survey references: `docs/rules/surveys/`.

## Batch 1a results (experiment, SoD; evaluated 2026-10-03)

Results: `experiments/results/E04a-batch-1a.md`; predictions: `experiments/specs/E04a-batch-1a.toml`.
Played with the Complete edition's `Heroes3.exe` through the HD Mod 5.6 R9 launcher with HD+ on;
none of HD+'s fixes (https://www.sites.google.com/site/heroes3hd/eng/original-game-bug-fixing,
checked 2026-10-03) touches these battles. Recorded in OPEN-QUESTIONS (Q21, Q23), the quirks ledger
(Q-0004 to Q-0006) and the catalog (BAT-DMG-002, BAT-DMG-009, BAT-WAR-002, BAT-TRN-005). What each
task has to do with them:

- **T6** (done): quirk Q-0005 (T6a, BAT-DMG-002, R1–R3 as fixtures); the hero's damage
  bonuses enter as single-precision rates of their summed basis points (T6b, BAT-HER-004).
- **T7a** (done): R5 → BAT-WAR-005, R6/R7 → BAT-WAR-006 and HER-BAT-005, Q-0004 →
  BAT-WAR-002.
- **T7b:** a commanded First Aid Tent got no turn while no ally was hurt (R6); its Wait/Defend
  and the heal amount with the skill were not observable (the Golems destroyed it) — retry in
  batch 1b with troops hurt before the tent's first turn.
- **T10** (T3b put the values into the packs): Elixir of Life on Trolls shows 54 hit points: +25 % of the creature's base (40 → 50)
  plus the parts' flat +4 (Ring of Vitality, Ring of Life, Vial of Lifeblood); regeneration restores
  all 54 (R8: survey quirk S-04 not reproduced, no ledger entry). Creature specialties at hero
  level 4: +1 attack, +1 defense, +1 speed for 10/12 and 12/12 creatures (ceil of 5 % per step),
  Catherine and Sorsha for Swordsmen and Crusaders, Roland for Crusaders only (B4–B6; quirk Q-0006,
  already in the packs as the executable has it).
- Not observed: the wiki's Armorer example "2002 → 2001" is not explained by Q-0005 alone; Q22
  (jousting) was not staged.

## 3. Artifacts in the derivation (T10)

T4b1 implemented the effect sources (`h3_heroes` `derive.rs`, `effects`: skills at their levels,
worn artifacts per copy with a combination's parts, `needs_skill`, the specialty; HER-BAT-001)
and the primary skills and Tactics from them. Left for T10:

- **Which effects count:** each equipped artifact type counts once, except `primary_skills`
  (per copy, done); a skill specialty grows only the skill's own values, not the artifacts' (as
  VCMI).
- **Pending** (no battle effect in E04a): see `packs/README.md`, "What the harvested artifacts
  leave out"; Necromancy goes with the Necromancy skill (E08–E10), not T9.

## 4. `BattleHero` (read by T5–T7, T9, T10)

T4b1 implemented the types (`h3_contracts` `hero.rs`, docs there) and `derive_battle_hero`
(HER-BAT-001/002). Deviations from the ADR-time sketch: `BallistaControl.double_damage_chance_bp`
(basis points, as in content), `TentControl.heal_bonus_bp: u32` (the increase over the tent's
own maximum, as the content's `first_aid` effect; a specialist's growth exceeds `u16`),
`CatapultControl` an empty `#[non_exhaustive]` struct (cannot be built before E04c),
`BattleEffect` uses the content's `CreatureSelector` (one creature: a creature and its upgrades
become several effects) and `StatOp { Add, PercentBp }`. What the tasks still have to do:

- **Validation (T4b2), user decision Q31:** the battle checks only soundness — references of the
  ruleset, `effects.len() <= MAX_BATTLE_EFFECTS`, `reduction_bp` and `surrender_discount_bp`
  and the Ballista's chance at most 10 000, Tactics and school levels at most 3, the spell
  book's spells of the ruleset and not creature abilities; it does not check the primary-skill
  limits (any `u8` is accepted).
- **The derivation fills the rest:** `effects` from artifacts and specialties other than morale
  and luck (T10; morale and luck since T5b, HER-BAT-003; at most 64 in all), the tent's control
  (T7b; the Ballista's since T7a, HER-BAT-005), surrender discount (T9), magic (E04b); damage
  bonuses since T6b (HER-BAT-004, applied by BAT-HER-004). Sums that can exceed the field types saturate; `surrender_discount_bp` is
  clamped to 10 000 like `reduction_bp`.

## 5. `BattleSetup`, `BattleOutcome`, `BattleResult` (T9)

T4b2 implemented `BattleSetup.heroes` (replaces `led_by_hero`; Q-0003 reads hero presence;
BAT-HER-001). The battle keeps a small per-side record (`h3_battle` `hero.rs`, `HeroRecord`:
hero type, primary skills, Tactics level, mana, maximum mana); each later task adds the fields
its rules read, together with the scenario/replay format field (`h3_pack::HeroEntry`) and a
state-hash bump. Left for T9:

- `BattleSetup.funds: [u32; 2]`: gold the side's owner can spend on a surrender (ignored for a side
  without a hero). `war_machines`, `formations`, `terrain` and `obstacles` stay. `war_machines` is
  filled from `h3_heroes::war_machines`, which returns the Ballista, Ammo Cart and First Aid Tent
  of the hero's war-machine slots; the **Catapult** is left out outside sieges, and the battle
  keeps rejecting it until E04c (it then belongs to the attacker of a siege, controlled through
  `MachineControl.catapult`).
- `BattleResult` gains `Retreat(side)` and `Surrender(side)` (the side that left; `winner()` is
  the other side) next to `Victory(side)` and `Draw`.
- `BattleOutcome` gains:
  - `stacks: [Vec<StackRecord>; 2]`, **every** stack of both sides in stack order:
    `StackRecord { origin: StackOrigin, creature: CreatureTypeId, start: u32, end: u32 }`,
    `StackOrigin` ∈ {army slot (u8), war machine} (E04b adds summoned and clone). `end` is the
    count alive when the battle ended, whatever the result;
  - `surrender_cost: Option<u32>` (gold to be paid by the side that surrendered);
  - `heroes: [Option<HeroOutcome>; 2]` with `HeroOutcome { mana: u16, spells_cast: Vec<SpellId> }`
    (E04b fills the spells; Eagle Eye needs them);
  - `rounds: u16` (the last round started; the tactics phase is round 0).
  `survivors` and `war_machines` stay and are derived from `stacks` (army stacks and machines with
  `end > 0`). The outcome reports facts; the consequences — a retreating side loses its army, a
  surrendering side keeps it, war machines follow the hero's artifacts, the loser's artifacts are
  captured, experience and Necromancy — are applied by the orchestrator under T9's rules.
- **Seam stability:** this fixes every field E04a and E04b need. Later epics add only enum
  variants (`BattleEffect`, `StackOrigin`) or, for E04c, two optional fields whose names this ADR
  reserves: `BattleSetup.siege` and `BattleOutcome.siege` (contents defined by E04c's ADR), plus
  a special-battlefield kind next to `terrain`. Existing fields keep their meaning.

## 6. The hero's effects in the battle state (T5, T10)

- **Battle-long stat effects are applied once at setup.** T5a implemented the hero's attack and
  defense (BAT-HER-002, `Stack::new`: creature, native terrain, hero; war machines included).
  T10 adds the `StackStat` effects after them (percentages of the creature's own value, computed
  before any addition). Traits such as regeneration or no range penalty become stack traits. The
  pool (ADR-0012) keeps what changes during a battle; its evaluation starts from these bases.
- **Morale and luck effects become side modifiers:** done in T5b (BAT-HER-003, `h3_battle`
  `morale.rs`): both heroes' changes summed and their caps combined per side and stat, so 4 per
  side and `MODIFIER_CAPACITY` 28 (the sketch's per-hero keys would have needed 36 for the same
  result). E04b adds stack bearers and timed expiry and re-derives the capacity (it may keep
  per-stack spell effects in per-stack arrays if that is cheaper; its decision).
- The battle keeps a small per-side hero record (present, tactics level, damage bonuses, machine
  control, retreat and surrender allowed, an offer made this round; E04b: mana, cast this round).

## 7. Phases and commands (T7, T8, T9)

- **Tactics phase** (round 0, after placement in the side's formation, BAT-SET-001): the side whose
  Tactics level exceeds the enemy's gets a zone of 1 + 2 × difference columns (3/5/7) from its
  edge; equal levels cancel. Only that side's army stacks are activated, in round-1 order and
  cycling; commands: `Move { to }` with the footprint, occupancy and obstacle rules of an ordinary
  move (a walking stack needs a path; flying stacks fly), the speed limit replaced by the zone;
  `Wait` (skip to the next stack); and the new `EndTactics`. War machines are not moved. Events
  `TacticsStarted`, `TacticsEnded`; `RoundStarted(1)` follows. Like the battle itself the phase has
  no limit in the rules; the sim's step limit truncates (E01), and E05 handles the action balance.
  The exact zone geometry and whether placement needs a path are batch-1 questions (OQ-1, OQ-2).
- **Retreat** (`Retreat`): by the side whose stack is active or whose tactics phase runs; legal if
  the side has a hero and no `NoRetreat` effect is in the battle. Ends the battle with
  `Retreat(side)`.
- **Surrender** (`Surrender`, then `AcceptSurrender` or `RejectSurrender` by the opponent):
  legal like Retreat, and only if the enemy has a hero, the side has not offered yet in this round
  (the tactics phase counts as round 0) and its `funds` cover the price. The price is a new BAT
  rule: Σ over the side's living army stacks (war machines excluded) of count × the creature's
  gold cost, × a ruleset factor in percent (D-1: 50 or 100), × (10 000 − discount) / 10 000,
  computed in `u64` and rounded down (D-2 checks the combination). If the ruleset parameter
  `surrender_needs_consent` is set (default per D-5; the wiki says the opponent may refuse), the
  battle enters a pending state in which the only legal commands are the opponent's answer; a
  rejection resumes where the offer was made.
- **Hero-run war machines:** the Ballista is done (T7a, BAT-WAR-006). With `MachineControl.tent`
  the side commands its First Aid Tent (new `Heal { target }`, T7b); otherwise it acts
  automatically as in E03. Wait and Defend stay legal for it (OQ-4).
- Actors are validated as in ADR-0004: the answer to a surrender comes from the side it was offered
  to, every other command from the side whose stack is active or whose tactics phase runs.

## 8. Visibility (T7–T9, T11)

T4b2: `Observation.heroes` shows both sides the public part (`HeroView`: hero type, primary
skills, mana, maximum mana); the Tactics level stays private (hygiene property test). The
encoding still has only the two "led by a hero" flags; T11 adds hero features. Still to do:

- Public in the observation of both sides: the war machines and who controls them, the
  tactics phase and its zone, whether each side may retreat and surrender (a `NoRetreat` effect
  changes both sides' options, so it is public), a surrender offer and its price (accepting that an
  offer reveals that the offering side can pay).
- Private to the hero's own side: `tactics` level, the damage bonuses, `surrender_discount_bp`,
  school levels, the `effects` list other than the public consequences above (stat effects show in
  the stacks' public stats), the spell book, `funds`.
- E04b's hidden battlefield objects (Land Mine, Quicksand) use the same mechanism: state with an
  owner side, projected away for the other viewer, events with `Visibility::Side` (ADR-0004 §6).

## 9. Encodings (T11)

- E04a extends the dense action encoding (v5): `EndTactics`, `Retreat`, `Surrender`, the two
  answers, and `Heal { target }` over the stack slots; tactics placement reuses the `Move`
  indices and the commanded Ballista reuses `Shoot`. Observation encoding v4 adds per-side hero
  features (the public part for the opponent). C ABI v3.
- E04b's casting (spells × targets) does not fit a flat dense encoding; E04b designs a factorized
  one together with E05 (BACKLOG). E04a does not pre-empt it.
