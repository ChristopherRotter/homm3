# E04a design notes (ephemeral)

The working detail behind [ADR-0013](../adr/0013-heroes-in-battle.md): field lists, formulas and
sizes that guide the implementation. The ADR holds the binding decisions; this note is a handover.

**Pruning rule:** the task that implements a section deletes it in the same change (the code
documentation, the rule catalog and the pack docs then hold the detail). Where the implementation
deviates, the code wins; change the ADR only if a decision changes. Delete this file when E04a closes.
Section numbers follow the ADR. Survey references: `docs/rules/surveys/`.

## 2. `HeroDetails`, validation and the translation (T4)

- `h3-heroes`: domain, logic crate (`no_std`, 100 % coverage), depends on kernel, content and
  contracts only; `arch.toml` entry; rule catalog `docs/rules/catalog/heroes.toml`, prefix `HER`.
  In E04a it contains `HeroDetails`, its validation and the translation, and **no commands and no
  map or battle state**:
  - `HeroDetails { hero_type, level, experience, primary: [u8; 4], secondary: Vec<(SkillId, u8)>,
    artifacts: equipped slots + backpack, spell_book: Option<Vec<SpellId>>, mana }`.
  - `validate(&HeroDetails, &Ruleset) -> Result<(), DetailsError>` (loadout validation: levels,
    skill count and levels, slot compatibility, combination locks, one artifact per slot, the
    ruleset's slot set — 18 slots before SoD, 19 with it).
  - `derive_battle_hero(&HeroDetails, &Ruleset) -> BattleHero` and
    `war_machines(&HeroDetails, &Ruleset) -> Vec<CreatureTypeId>` (from the war-machine slots).
- `h3-battle` never depends on `h3-heroes`; it reads only `BattleHero`.
- `h3-content` owns the definitions and the content-side effect vocabulary (§3), including the
  creature upgrade relation that selectors need (T3 adds it).

## 3. Effect vocabulary in content (T3, T10)

- A closed enum `HeroEffect<C>` in `h3-content` (generic over creature references, like `Ability`,
  ADR-0012), referenced by artifacts, by each level of a secondary skill and by specialties.
  Parameters are Tier 1 data; the compiler validates them. New behaviour is a new variant plus the
  rule that reads it (ADR-0006).
- Effects state *how* a value is computed (fixed, per hero level, per hero level over creature
  level, "only if the hero has skill X", "on own side / enemy side / both"); the translation (§4)
  turns them into resolved battle effects. Effects that matter only on the adventure map are not
  given variants before their epic; T3 lists which artifacts and specialties carry such uncovered
  effects instead of inventing placeholders.
- **Units:** percentages are basis points (1/100 %) in content and in the seam. Every skill and
  specialty product is then an exact integer.
- The initial E04a variants follow the survey's effect kinds (`E04-artifacts.md` §4,
  `E04-heroes.md` §3, `E04-skills.md` §1): primary skills, morale and luck (with the affected side),
  morale and luck caps, creature stats (attack, defense, speed, hit points, damage; fixed or
  scaled), damage percentages (ranged, melee, reduction), skill-value bonuses (Bow artifacts,
  Necromancy artifacts), the skill specialty, regeneration, no range penalty, no retreat, war
  machine, and the battle-action skills (tactics, artillery, first aid; ballistics for E04c). T3
  fixes the exact list.

## 4. `BattleHero` (T4; read by T5–T7, T10)

```rust
pub struct BattleHero {
    pub hero_type: HeroTypeId,       // identity for observation and replays; no rule reads it
    pub attack: u8,                  // effective primary skills: base + artifacts + specialty,
    pub defense: u8,                 //   within the ruleset's limits (checked by both sides)
    pub power: u8,                   // E04b reads power; knowledge only explains max mana
    pub knowledge: u8,
    pub tactics: u8,                 // Tactics level 0..=3 (the battle compares both sides')
    pub damage: DamageBonuses,       // basis points, specialty and artifacts included
    pub war_machines: MachineControl,
    pub surrender_discount_bp: u16,  // Diplomacy and artifacts, 0..=10_000
    pub effects: Vec<BattleEffect>,  // everything else, resolved; at most MAX_BATTLE_EFFECTS = 64
    pub magic: BattleMagic,          // E04b
}
pub struct DamageBonuses { pub ranged_bp: u16, pub melee_bp: u16, pub reduction_bp: u16 }
pub struct MachineControl {          // all derived from skill level, specialty and artifacts
    pub ballista: Option<BallistaControl>,   // Some = the side commands its Ballista (Artillery)
    pub tent: Option<TentControl>,           // Some = the side commands its First Aid Tent
    pub catapult: Option<CatapultControl>,   // reserved for E04c (Ballistics); None in E04a
}
pub struct BallistaControl { pub shots: u8, pub double_damage_percent: u8 }
pub struct TentControl { pub max_heal: u16 }  // heals 1..=max_heal (installation.md §7.5, Q29)
pub struct BattleMagic { pub spell_book: Option<Vec<SpellId>>, pub mana: u16, pub max_mana: u16, pub schools: [u8; 4] }
```

- **Hybrid shape:** values the battle reads by name, once per side, on the hot path or for action
  legality are typed fields; variable, selective effects are the `effects` list. Adding an
  artifact or specialty kind adds a `BattleEffect` variant, not a field. War-machine control is
  carried as derived numbers, not skill levels, so specialties and artifacts that change shots,
  chances or heals need no channel of their own.
- **Units and ranges:** `ranged_bp` and `melee_bp` may exceed 10 000 (an Archery specialist at a
  high level); `reduction_bp` and `surrender_discount_bp` are at most 10 000; skill and school
  levels at most 3; `max_mana` is informative (mana may exceed it, e.g. after a Mana Vortex).
  The ruleset carries the primary-skill limits; the derivation stays within them and the battle
  rejects values outside them.
- **How the damage fields enter the damage formula** (survey `E04-skills.md` §3): `ranged_bp`
  (the attacker's side) is added to the bonus sum of shots, the Ballista's included; `melee_bp`
  to the bonus sum of melee attacks and retaliations, a shooter's melee included; never both.
  `reduction_bp` (the defender's side) is one multiplicative factor `(10 000 − r) / 10 000` on every
  physical strike against that side's stacks. BAT-DMG-002 stays exact-integer until Q23 is
  answered.
- `BattleEffect` (closed enum in contracts, selectors resolved):
  `Morale { amount: i8, affects }`, `Luck { amount: i8, affects }`,
  `MoraleAtMost { value: i8, affects }`, `LuckAtMost { value: i8, affects }`,
  `StackStat { creatures, stat, op, amount: i16 }` with `stat` ∈ {attack, defense, speed, hit
  points, minimum damage, maximum damage} and `op` ∈ {add, percent of the creature's own value
  (basis points, rounded up)}, `Regeneration { creatures }`, `NoRangePenalty { creatures }`,
  `NoRetreat` (neither side may retreat or surrender), plus what T10 adds from the survey list.
  `affects` ∈ {own side, enemy side, both}; `creatures` is `All`, a `CreatureFlag` trait (living,
  undead, dragon, shooter, …) or up to four creature types (a creature and its upgrades).
- **E04b fields are present and validated from E04a on** (ranges, spell ids of the ruleset);
  E04a's battle carries them and ignores them.
- The battle validates a `BattleHero` like the rest of the setup (`SetupError`).

## 5. `BattleSetup`, `BattleOutcome`, `BattleResult` (T4, T9)

- `BattleSetup.heroes: [Option<BattleHero>; 2]` **replaces `led_by_hero`**. "Led by a hero" is
  `heroes[side].is_some()`; quirk Q-0003 reads it.
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

- **Battle-long stat effects are applied once at setup.** A stack's *base* stats (ADR-0012 §3)
  become: the creature's values, then the native-terrain bonus, then the hero's attack and defense,
  then `StackStat` effects (percentages of the creature's own value, computed before any
  addition). They apply to every stack of the side, war machines included (VCMI; to be confirmed
  for the Ballista in batch 1). Traits such as regeneration or no range penalty become stack traits.
  The pool (ADR-0012) keeps what changes during a battle; its evaluation starts from these bases.
- **Morale and luck effects become side modifiers.** The battle sums a hero's effects per affected
  side, stat and operation at setup and inserts one modifier per (bearer side, stat, operation,
  source hero): sources are distinct keys for "own hero add", "own hero cap", "enemy hero add",
  "enemy hero cap", so an `Add` and an `AtMost` from the same hero never replace each other. That is
  at most 8 per bearer side, 16 in all: `MODIFIER_CAPACITY` grows from 20 to 36 (sized by
  construction and tested). E04b adds stack bearers and timed expiry and re-derives the capacity
  (it may keep per-stack spell effects in per-stack arrays if that is cheaper; its decision).
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
- **Hero-run war machines:** with `MachineControl.ballista` the side commands its Ballista
  (ordinary `Shoot`, `shots` shots, `double_damage_percent`), with `tent` its First Aid Tent (new
  `Heal { target }`); otherwise they act automatically as in E03. Wait and Defend stay legal for
  them (OQ-4).
- Actors are validated as in ADR-0004: the answer to a surrender comes from the side it was offered
  to, every other command from the side whose stack is active or whose tactics phase runs.

## 8. Visibility (T4, T11)

- Public in the observation of both sides: `hero_type`, attack, defense, power, knowledge, mana
  and max mana (the original shows these in battle), the war machines and who controls them, the
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
