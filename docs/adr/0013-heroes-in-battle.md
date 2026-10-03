# ADR-0013: Heroes in battle — entities, the `BattleHero` seam and the hero's battle rules

- Status: Proposed (E04a T1, 2026-10-03) — work on E04a T4 and later waits for the user's review
- Date: 2026-10-03
- Deciders: agent session (E04a T1), to be accepted by the user

## Context

E04a lets a hero lead a battle without magic; E04b adds battle magic and E04c sieges (OPEN-QUESTIONS
Q27). The user fixed the direction in the refinement: heroes in battle and on the adventure map are
distinct entities, there is no "god" hero type and no type named plain `Hero`; a domain crate
`h3-heroes` holds only the hero's lasting details and the pure translation to the battle's own seam
type; all derived values are computed before the seam (03-domain, "Seams"). This ADR fixes the
names, the crate boundaries, the shape of `BattleHero` for E04a *and* E04b, the changes to
`BattleSetup`, `BattleOutcome` and `BattleResult`, the effect vocabulary, the modifier capacity,
the new battle phases and commands, visibility and the encoding policy, so that the seam changes
once (epic E04a, "Open design points for ADR-0013").

Inputs from the T1 survey (`docs/rules/surveys/`):

- The executable holds the effect *numbers*: per-level skill values, the specialty table, the
  artifact primary-skill table (`docs/formats/installation.md` §7). Conditional behaviour (what
  Tactics or Artillery let a side do) is code, i.e. our rules.
- Specialty terms are fractional: skill value × (1 + 0.05 × hero level) gives e.g. 17.5 % or
  0.25 % steps. All of them are whole **basis points** (1/100 %).
- Many effects reach the *other* side or both (Spirit of Oppression, Hourglass of the Evil Hour,
  Shackles of War, enemy-morale artifacts); creature-selective effects need selectors (a creature
  and its upgrade, dragons, living creatures, shooters).
- The outcome must carry per-stack casualties (experience; Necromancy is computed per stack in
  SoD), the way the battle ended (retreat, surrender) and the surrender price.
- Tactics, retreat and surrender are decisions inside the battle; surrender's legality depends on
  a price that changes with casualties, so the battle must compute it.

## Decision

### 1. Entities and names

| Entity | Owns | Lives in | Type |
|---|---|---|---|
| Hero type (definition) | class, sex, specialty, starting skills, spell, army | content, Tier 1 data | `h3_content::HeroType` (handle `HeroTypeId`); also `HeroClass`, `SecondarySkill`, `ArtifactType`, spell rows (E04a T3) |
| Hero details | the hero's lasting state: type, level, experience, base primary skills, secondary skills, artifacts, spell book, mana | Heroes context | `h3_heroes::HeroDetails` |
| Hero on the map | owner, position, movement, boat, vision | Adventure context (E08) | its own type, named by E08 |
| Battle hero | what a battle needs, all derived | contract | `h3_contracts::BattleHero` (in-battle state stays private to `h3-battle`) |

- **No type is named `Hero`.** Contexts meet through ids and explicit pure translations; a
  context never imports another context's type. "Hero" in prose means the person across contexts.
- **Identity:** a cross-context `HeroId` handle is introduced when a store of heroes exists (E08/E10).
  E04a has none: the battle refers to "the hero of a side", and `BattleHero.hero_type` identifies
  the person for observation and replays only (no rule reads it).
- **The army is not part of `HeroDetails`.** Armies exist without heroes (garrisons, neutral
  stacks) and keep using `h3_contracts::Army`; the orchestrator pairs a hero with an army.
- The name `HeroDetails` is kept (provisional name from the refinement, no better one found).

### 2. Crates and dependencies

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

### 3. Effect vocabulary in content (Tier 3)

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

### 4. `BattleHero` (contract, designed for E04a and E04b)

```rust
pub struct BattleHero {
    pub hero_type: HeroTypeId,      // identity for observation and replays; no rule reads it
    pub attack: u8,                 // effective primary skills: base + artifacts + specialty,
    pub defense: u8,                //   clamped by the derivation to the ruleset's limits
    pub power: u8,                  // E04b reads power; knowledge only bounds mana
    pub knowledge: u8,
    pub skills: BattleSkills,       // levels 0..=3 that unlock or scale battle actions
    pub damage: DamageBonuses,      // basis points, specialty and artifacts included
    pub surrender_discount_bp: u16, // Diplomacy and artifacts (the battle prices a surrender)
    pub effects: Vec<BattleEffect>, // everything else, resolved (closed enum, bounded length)
    pub magic: BattleMagic,         // E04b: spell book, mana, max mana, school levels
}
pub struct BattleSkills { pub tactics: u8, pub artillery: u8, pub first_aid: u8, pub ballistics: u8 }
pub struct DamageBonuses { pub ranged_bp: u16, pub melee_bp: u16, pub reduction_bp: u16 }
pub struct BattleMagic { pub spell_book: Option<Vec<SpellId>>, pub mana: u16, pub max_mana: u16, pub schools: [u8; 4] }
```

- **Hybrid shape:** values the battle reads by name, once per side, on the hot path or for action
  legality are typed fields; variable, selective effects are the `effects` list. Adding an
  artifact or specialty kind adds a `BattleEffect` variant, not a field.
- `BattleEffect` (closed enum in contracts, selectors already resolved):
  `Morale { amount, affects }`, `Luck { amount, affects }`, `MoraleAtMost { value, affects }`,
  `LuckAtMost { value, affects }`, `StackStat { creatures, stat, op, amount }` (op: add, or percent
  of the creature's base value), `Regeneration { creatures }`, `NoRangePenalty { creatures }`,
  `NoRetreat` (neither side may retreat or surrender), plus what T10 adds from the survey list.
  `affects` ∈ {own side, enemy side, both}. `creatures` is `All`, a trait (living, undead, dragon,
  shooter — the `CreatureFlag`s) or up to four explicit creature types (a creature and its
  upgrades). E04b adds spell variants (opening spells, immunities, resistance, spell damage,
  duration, blocked magic levels) without touching the fields.
- **E04b fields are present and validated from E04a on** (ranges, spell ids of the ruleset,
  `mana ≤ max_mana`); E04a's battle carries them and ignores them. So E04b changes no seam type,
  only adds `BattleEffect` variants.
- The battle validates a `BattleHero` like the rest of the setup (`SetupError`): levels ≤ 3,
  bounded `effects` length (`MAX_BATTLE_EFFECTS`), creature ids of this ruleset.

### 5. `BattleSetup`, `BattleOutcome`, `BattleResult`

- `BattleSetup.heroes: [Option<BattleHero>; 2]` **replaces `led_by_hero`**. "Led by a hero" is
  `heroes[side].is_some()`; quirk Q-0003 reads it.
- `BattleSetup.funds: [u32; 2]`: gold the side's owner can spend on a surrender (0 without a
  hero). `war_machines`, `formations`, `terrain` and `obstacles` stay; `war_machines` is filled
  from `h3_heroes::war_machines` before the seam.
- **Reserved for E04c:** one optional `siege` field in the setup and one siege result in the
  outcome, and a special-battlefield kind; E04c's own ADR defines their contents. Nothing else in
  the setup changes for sieges.
- `BattleResult` gains `Retreat(side)` and `Surrender(side)` (the side that left; `winner()` is
  the other side) next to `Victory(side)` and `Draw`.
- `BattleOutcome` gains:
  - `stacks: [Vec<StackRecord>; 2]` with `origin` (army slot, war machine; E04b: summoned,
    clone), creature type, count at the start, count at the end — the casualties per stack that
    experience and Necromancy need (both are applied by the orchestrator later, not by the battle);
  - `surrender_cost: Option<u32>` (gold paid);
  - `heroes: [Option<HeroOutcome>; 2]` with `mana` left and `spells_cast` (E04b fills them;
    Eagle Eye needs them);
  - `rounds: u16`.
  `survivors` and `war_machines` stay as they are. Hero fate (killed, fled, surrendered) and
  artifact capture follow from `result` and hero presence and are the orchestrator's business.

### 6. The hero's effects in the battle state

- **Battle-long stat effects are applied once at setup** into each stack's base stats (hero
  attack and defense, `StackStat`, specialties), after the native-terrain bonus; traits such as
  regeneration or no range penalty become stack traits. The hot path keeps reading plain fields.
  ADR-0012's modifier pool keeps what can change during a battle.
- **Morale and luck effects become side modifiers** in the pool (ADR-0012), one per source and
  stat: the own hero's morale and luck, the enemy hero's morale and luck, and the caps (`AtMost`).
  At most 6 per side, so `MODIFIER_CAPACITY` grows from 20 to 32 (sized by construction, tested).
  E04b adds stack bearers and timed expiry and re-derives the capacity (it may keep per-stack
  spell effects in per-stack arrays if that is cheaper; its decision).
- The battle keeps a small per-side hero record (present, skill levels, damage bonuses, may
  retreat, may surrender, controls Ballista/Tent; E04b: mana, cast this round). Damage rules read
  `ranged_bp`/`melee_bp` of the attacker's side and `reduction_bp` of the defender's side
  (BAT-DMG-002 stays exact-integer until Q23 is answered).

### 7. Phases and commands

- **Tactics phase** (before round 1; "round 0"): the side whose Tactics level exceeds the
  enemy's gets a zone of 1 + 2 × difference columns (3/5/7) from its edge; equal levels cancel.
  Only that side's army stacks are activated, in round-1 order and cycling; commands: `Move { to }`
  to any reachable hex of the zone (the stack's speed does not limit it), `Wait` (skip to the next
  stack) and the new `EndTactics`. War machines are not moved. Events `TacticsStarted`,
  `TacticsEnded`. (Survey `E04-battle.md` §1; open points OQ-1/OQ-2 go to batch 1.)
- **Retreat** (`Retreat`): by the side whose stack is active or whose tactics phase runs; legal
  if the side has a hero and no `NoRetreat` effect is in the battle. Ends the battle with
  `Retreat(side)`.
- **Surrender** (`Surrender`, `AcceptSurrender`, `RejectSurrender`): legal like Retreat and only
  if the enemy has a hero and the side's `funds` cover the price. The battle computes the price
  (a new BAT rule: gold cost of the living army creatures × a ruleset factor × (1 − discount); the
  factor is D-1 in batch 1). If the ruleset parameter `surrender_needs_consent` is set (default
  per D-5; the wiki says the opponent may refuse), the battle enters a pending state in which the
  only legal commands are the opponent's answer; a rejected offer resumes the active stack, and a
  side may offer once per round.
- **Hero-run war machines:** with Artillery the side commands its Ballista (ordinary `Shoot`, with
  the skill's shots and double-damage chance), with First Aid its tent (new `Heal { target }`);
  without the skill they act automatically as in E03. Wait and Defend stay legal for them (OQ-4).
- Actors are validated as in ADR-0004: the answer to a surrender comes from the side it was
  offered to, every other command from the side whose stack is active.

### 8. Visibility

- Public in the observation of both sides: `hero_type`, attack, defense, power, knowledge, mana
  and max mana (the original shows these on right-click in battle), the war machines, the tactics
  phase and its zone, a surrender offer and its price.
- Private to the hero's own side: skill levels, damage bonuses, the `effects` list (their visible
  consequences are on the stacks, whose stats are public), the spell book, `funds`.
- E04b's hidden battlefield objects (Land Mine, Quicksand) use the same mechanism: state with an
  owner side, projected away for the other viewer, events with `Visibility::Side` (ADR-0004 §6).

### 9. Encodings

- E04a extends the dense action encoding (v5): `EndTactics`, `Retreat`, `Surrender`, the two
  answers, and `Heal { target }` over the stack slots; tactics placement reuses the `Move`
  indices and the commanded Ballista reuses `Shoot`. Observation encoding v4 adds per-side hero
  features (the public part for the opponent). C ABI v3.
- E04b's casting (spells × targets) does not fit a flat dense encoding; E04b designs a factorized
  one together with E05 (BACKLOG). E04a does not pre-empt it.

### 10. Rules and catalog

- Battle rules for heroes keep the `BAT-` prefix (new areas such as `BAT-TAC`, `BAT-FLE` for retreat
  and surrender, `BAT-HER` for hero effects); derivation rules that encode original behaviour (primary
  totals and caps, specialty formulas, skill tables) are `HER-` rules in `heroes.toml`.
- Each rule cites its sources; values from the executable tables are `original-data`.

## Consequences

- `BattleSetup`, `BattleOutcome` and `BattleResult` change once for E04a and E04b; E04c adds its
  reserved siege fields. Every caller (sim, CLI scenarios and replays, FFI, tests) changes in E04a
  T4/T9/T11; replays get a hero table; state hash v9; ruleset hash v9 (content types, T3).
- The battle stays ignorant of hero management: no level, class, specialty or artifact logic in
  `h3-battle`; changing a skill's numbers is a data change; a new kind of effect is one variant in
  content, one in contracts and one rule.
- Setup does a little more work; the hot path is unchanged (plain fields, one pool lookup per
  materialisation). State grows by two small hero records and 12 pool entries; T4 measures it.
- Enforced by: `check-arch` (no `h3-battle` → `h3-heroes` edge; `h3-heroes` is a logic crate),
  property tests (hero isolation: a hero changes the other side only through modelled effects;
  observation hygiene for the private fields; conservation of creatures with casualties
  accounted), the rule catalog, `SetupError` tests for every new validation.

## Alternatives considered

- **One hero type for all contexts** (or passing `HeroDetails` across the seam): the battle would
  depend on hero management; rejected by the user (Q27).
- **Typed fields only:** the struct would grow with every artifact kind and the seam would change
  each time. **Effects list only:** the hot path would search the list for Archery and the like,
  and action legality would depend on list contents. The hybrid keeps both small.
- **Hero attack and defense as pool modifiers:** more pool entries and recomputation for values
  that never change during a battle; rejected (setup-time application, §6).
- **Percent units:** whole percents cannot express 17.5 % or 0.25 % steps; floats are banned in
  logic crates (ADR-0003). Basis points are exact for every known term.
- **Tactics with a `Place { stack, hex }` command:** an action per stack and hex (about 3 500 new
  actions) for little gain; cycling activations reuse `Move`.
- **Surrender without the battle computing the price:** the action mask could not tell whether
  surrender is affordable; the orchestrator would have to undo a battle end. Rejected; this
  revises the epic's non-goal "surrender cost: the orchestrator computes it" (the orchestrator
  still deducts the gold).
- **Surrender without consent** (HotA): simpler, but the wiki says the SoD opponent may refuse;
  kept as a ruleset parameter until batch 1 answers D-5.

## Revision history

- 2026-10-03: Proposed (E04a T1).
