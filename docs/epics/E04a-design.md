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
- **T10** (done): R1, R8 and B7 are fixtures of HER-BAT-008 to 010 and BAT-HER-005 to 007
  (`h3-pack`, `harvested_hero_effects_reproduce_the_experiments`). The hero screen shows no
  alignment morale; only the battle does (R1).
- Not staged: D8 (Cloak of the Undead King, E08–E10), Q22.

## 4. `BattleHero` (done in E04a; magic E04b)

T4b1 implemented the types (`h3_contracts` `hero.rs`, docs there) and `derive_battle_hero`
(HER-BAT-001/002). Deviations from the ADR-time sketch: `BallistaControl.double_damage_chance_bp`
(basis points, as in content), `TentControl.heal_bonus_bp: u32` (the increase over the tent's
own maximum, as the content's `first_aid` effect; a specialist's growth exceeds `u16`),
`CatapultControl` an empty `#[non_exhaustive]` struct (cannot be built before E04c),
`BattleEffect` uses the content's `CreatureSelector` (one creature: a creature and its upgrades
become several effects), `StatOp { Add, PercentBp, PercentBpDown }` (T10a) and `FactionGroup`, a
fixed array of up to 16 factions (T10c, Q36).

- **Validation (T4b2), user decision Q31:** the battle checks only soundness — references of the
  ruleset, `effects.len() <= MAX_BATTLE_EFFECTS`, `reduction_bp` and `surrender_discount_bp`
  and the Ballista's chance at most 10 000, Tactics and school levels at most 3, the spell
  book's spells of the ruleset and not creature abilities; it does not check the primary-skill
  limits (any `u8` is accepted).
- **The derivation** (HER-BAT-001 to 010) fills every field except the magic (E04b): sums that
  can exceed the field types saturate; `surrender_discount_bp` is clamped to 10 000 like
  `reduction_bp`; more than 64 effects is an error.

## 5. `BattleSetup`, `BattleOutcome`, `BattleResult` (done)

T4b2 implemented `BattleSetup.heroes`, T9b `funds`, T9a/T9b the results `Retreat`/`Surrender`, T9c
the outcome (`StackRecord`, `StackOrigin`, `HeroOutcome`, `surrender_cost`, `rounds`;
BAT-END-003). `war_machines` comes from `h3_heroes::war_machines` (Ballista, Ammo Cart, First Aid
Tent); the **Catapult** is left out outside sieges, and the battle keeps rejecting it until E04c
(it then belongs to the attacker of a siege, controlled through `MachineControl.catapult`).
**Seam stability** (ADR-0013 point 5): later epics add only enum variants (`BattleEffect`,
`StackOrigin`: summoned stacks and clones in E04b) or, for E04c, the reserved optional fields
`BattleSetup.siege` and `BattleOutcome.siege` and a special-battlefield kind next to `terrain`.

## 6. The hero's effects in the battle state (done; read by E04b)

- **Battle-long stat effects are applied once at setup** (BAT-HER-002, BAT-HER-005, BAT-HER-006,
  `Stack::new`; the alignment groups BAT-HER-007 in `morale::setup`). The pool (ADR-0012) keeps
  what changes during a battle; its evaluation starts from these bases.
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

## 8. Visibility (done)

T11a (Q37): `observe` takes the ruleset. Public: `HeroView` (hero type, primary skills, mana, the
war machines the hero commands), `Observation.tactics`, `may_retreat` and `surrender_possible`.
Private to the own side: `Observation.own_hero` (`OwnHeroView`: Tactics level, damage bonuses,
how the machines work, surrender discount, funds, surrender price). E04b adds its private parts
(school levels, spell book) there. E04b's hidden battlefield objects (Land Mine, Quicksand) use
the same mechanism: state with an owner side, projected away for the other viewer, events with
`Visibility::Side` (ADR-0004 §6).

## 9. Encodings (T11)

- T7b already appended `Heal { target }` over the stack slots (action encoding v5, C ABI v3:
  every legal command must be encodable); the commanded Ballista reuses `Shoot`. T8 appended
  `EndTactics` (action encoding v6, C ABI v4; placements reuse the `Move` indices, passing reuses
  Wait); T9 appended `Retreat` (v7, C ABI v5, with result codes for retreats and surrenders) and
  `Surrender` (v8, C ABI v6); each bump also bumps the C ABI (ADR-0011). Observation encoding v4
  adds per-side hero features (the public part for the opponent) in T11.
- E04b's casting (spells × targets) does not fit a flat dense encoding; E04b designs a factorized
  one together with E05 (BACKLOG). E04a does not pre-empt it.
