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
- **T7** (done): R5 → BAT-WAR-005, R6/R7 → BAT-WAR-006/007 and HER-BAT-005, Q-0004 →
  BAT-WAR-002. Still to observe (batch 1b, R7): the commanded tent's Wait/Defend, target choice and
  heal amounts (R6's Golems destroyed it) — stage troops hurt before the tent's first turn.
- **T10** (T3b put the values into the packs): Elixir of Life on Trolls shows 54 hit points: +25 % of the creature's base (40 → 50)
  plus the parts' flat +4 (Ring of Vitality, Ring of Life, Vial of Lifeblood); regeneration restores
  all 54 (R8: survey quirk S-04 not reproduced, no ledger entry). Creature specialties at hero
  level 4: +1 attack, +1 defense, +1 speed for 10/12 and 12/12 creatures (ceil of 5 % per step),
  Catherine and Sorsha for Swordsmen and Crusaders, Roland for Crusaders only (B4–B6; quirk Q-0006,
  already in the packs as the executable has it).
- Not observed: the wiki's Armorer example "2002 → 2001" is not explained by Q-0005 alone; Q22
  (jousting) was not staged.

## Batch 1b results (experiment, SoD; evaluated 2026-10-04)

Results: `experiments/results/E04a-batch-1b.md` (the user's commit `4610112`); predictions:
`experiments/specs/E04a-batch-1b.toml`. Played with the Complete edition's `Heroes3.exe` through the
HD Mod 5.6 R9 launcher, plain HD. The user's decisions on the results: OPEN-QUESTIONS Q32. What
each task has to do with them:

- **T7** (done; observation sources added to BAT-WAR-007 and BAT-RNG-002): R7 — the player chooses
  the patient, the hurt Ballista is not one, Wait (the tent acts again later in the round) and
  Defend are available, eight heals at Basic between 6 and 50 (one capped at the 41 missing hit
  points). The automatic Ballista has the range penalty (7–11 at long range).
- **T8** (done): R2 and R3 are observation sources of BAT-TAC-001/002; the Tactics-formation
  toggle is not modelled (agent decision, BAT-TAC-001).
- **T9** (done): R4 and B6 are observation sources of BAT-FLE-001/002 and HER-BAT-007; the
  outcome (BAT-END-003) carries the facts that R5 (experience: killed war machines give none,
  Learning multiplies the whole sum including the +500, the loser's war machines are not
  captured) and R6 (Necromancy rounded down per killed stack) need; those rules are E08–E10's
  (BACKLOG inbox, 2026-10-04).
- **T10** (R1, R8, B7):
  - Angelic Alliance as the packs model it (artifacts D1): the five SoD factions count as one
    alignment, Conflux and neutral creatures stay separate (R1 in battle: −1 "Troops of three
    alignments" with all four, 0 without the Pixies or without the Peasants); VCMI (every non-evil
    faction, +1) and the wiki's "they cancel the bonus" are not the original. The hero screen shows
    no alignment morale; only the battle does.
  - War machines get the hit-point artifacts (R8: Ballista 254, Ammo Cart 104, Tent 79 with +4)
    but not the speed artifacts (speed stays 0) (artifacts open 1; VCMI gives both).
  - The Golden Bow removes the Ballista's range penalty (R8: 23, 17, 20; open 2, as VCMI).
  - Xeron's Devils get +1 speed (B7: 23/23/12; H-2, as the packs have it).
- Not staged: D8 (Cloak of the Undead King, E08–E10), Q22.

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
- **The derivation fills the rest:** `effects` from artifacts and specialties other than morale,
  luck and no-retreat (T10; morale and luck since T5b, HER-BAT-003; `NoRetreat` since T9a,
  HER-BAT-006; at most 64 in all), magic (E04b); damage bonuses since T6b (HER-BAT-004, applied
  by BAT-HER-004), war-machine control since T7 (HER-BAT-005, applied by BAT-WAR-006/007), the
  surrender discount since T9b (HER-BAT-007, applied by BAT-FLE-002). Sums that can exceed the
  field types saturate; `surrender_discount_bp` is clamped to 10 000 like `reduction_bp`.

## 5. `BattleSetup`, `BattleOutcome`, `BattleResult` (done)

T4b2 implemented `BattleSetup.heroes`, T9b `funds`, T9a/T9b the results `Retreat`/`Surrender`, T9c
the outcome (`StackRecord`, `StackOrigin`, `HeroOutcome`, `surrender_cost`, `rounds`;
BAT-END-003). `war_machines` comes from `h3_heroes::war_machines` (Ballista, Ammo Cart, First Aid
Tent); the **Catapult** is left out outside sieges, and the battle keeps rejecting it until E04c
(it then belongs to the attacker of a siege, controlled through `MachineControl.catapult`).
**Seam stability** (ADR-0013 point 5): later epics add only enum variants (`BattleEffect`,
`StackOrigin`: summoned stacks and clones in E04b) or, for E04c, the reserved optional fields
`BattleSetup.siege` and `BattleOutcome.siege` and a special-battlefield kind next to `terrain`.

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

## 7. Phases and commands (done)

The tactics phase is implemented (T8: BAT-TAC-001/002, `h3_battle` `tactics.rs`), Retreat and the
results `Retreat(side)`/`Surrender(side)` too (T9a: BAT-FLE-001, `Observation.may_retreat`), and
Surrender with its price (T9b: BAT-FLE-002, `flee.rs`; the price's resource and factor are
battle parameters, `BattleParams.surrender`).

- Actors are validated as in ADR-0004: every command comes from the side whose stack is active or
  whose tactics phase runs.

## 8. Visibility (T9, T11)

T4b2: `Observation.heroes` shows both sides the public part (`HeroView`: hero type, primary
skills, mana, maximum mana); the Tactics level stays private (hygiene property test). The
encoding still has only the two "led by a hero" flags; T11 adds hero features. T8:
`Observation.tactics` shows both sides the phase and its zone (the encoding shows round 0 until
T11). Still to do:

- Public in the observation of both sides: the war machines and who controls them, and whether
  surrender is possible at all (both heroes present, no `NoRetreat`
  effect; public because it changes both sides' options; T9b left it and the own price out of
  `Observation`, whose `observe` has no ruleset: T11 decides with the encoding; the action mask
  already shows the own side's option). Whether a side can afford its surrender, and its price, show only to that side (its action mask).
- Private to the hero's own side: `tactics` level, the damage bonuses, `surrender_discount_bp`,
  school levels, the `effects` list other than the public consequences above (stat effects show in
  the stacks' public stats), the spell book, `funds`.
- E04b's hidden battlefield objects (Land Mine, Quicksand) use the same mechanism: state with an
  owner side, projected away for the other viewer, events with `Visibility::Side` (ADR-0004 §6).

## 9. Encodings (T11)

- T7b already appended `Heal { target }` over the stack slots (action encoding v5, C ABI v3:
  every legal command must be encodable); the commanded Ballista reuses `Shoot`. T8 appended
  `EndTactics` (action encoding v6, C ABI v4; placements reuse the `Move` indices, passing reuses
  Wait); T9 appended `Retreat` (v7, C ABI v5, with result codes for retreats and surrenders) and
  `Surrender` (v8, C ABI v6); each bump also bumps the C ABI (ADR-0011). Observation encoding v4
  adds per-side hero features (the public part for the opponent) in T11.
- E04b's casting (spells × targets) does not fit a flat dense encoding; E04b designs a factorized
  one together with E05 (BACKLOG). E04a does not pre-empt it.
