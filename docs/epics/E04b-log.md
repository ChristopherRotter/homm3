# E04b log: done tasks in full

The full record of E04b's finished tasks (what was built, refinements, agent decisions marked
"change freely", measurements). **Not start-of-session reading:** the epic file keeps one line per
done task; read the matching section here when a task builds on an earlier one or a decision is
questioned. Rules live in the catalog, quirks in the ledger, decisions in ADR-0014; this log only
records how and why the tasks went.

## Done tasks

### T1 — Magic survey and ADR-0014 (2026-10-05, one session with the refinement)

- **Survey** `docs/rules/surveys/E04b-magic.md`, just in time (Q41): casting by a hero, effect kinds
  and targeting, durations and end conditions, counters, the resolution order, stacks created in
  battle, hidden obstacles, creature magic, quirks S-09 to S-21 by task, disputes M-1 to M-6,
  batch 2a candidates X-1 to X-6. Sources: VCMI 1.7.5 (cloned in the sandbox), the wiki (fetched
  with the user's approval through a summarising tool; it quotes at most about 125 characters and
  several pages are thin, so resistance stacking and the duration countdown stay open), the RoE
  manual pages on the wiki, original data.
- **Original data:** the spell table's flag word decoded by matching the 81 values against known
  behaviour and VCMI (`docs/formats/installation.md` §7.6; bit `0x1000` unclear, dispute M-1); the
  AB and SoD `SPTRAITS.TXT` differ only in row 57. Code reading for Q42 (§7.7): 21 stack records
  per side, Clone and Summon Elemental refused at 20 stacks, the elemental-type rule. Tools:
  `capstone` and `pefile` in the sandbox, `skipdata` sweep of `.text` (HANDOFF trap).
- **ADR-0014** proposed, revised once and accepted (Q42); ADR-0004 point 8 amended (growable lists
  in battle). Design note `docs/epics/E04b-design.md` with proposals for T2a–T2d, T3, T5, T6.
- **User decisions:** a hero's cast is optional, at most once per round if permitted; a counter
  spell replaces its pair's effect, the later wins; timed effects in one growable list per battle;
  21 slots per side with the limit of 20; commands may resolve unexpectedly for the limited
  observer (Q42).
- **Agent decisions (change freely):** rule area `BAT-MAG`; the confidence labels of the decoded
  flags; the candidate list for batch 2a; the proposals in the design note (sizes, names).

### T2a — Content and the hero's magic (2026-10-05)

- **Split:** the user confirmed T2a–T2d and asked for T2a + T2b in one session with a checkpoint
  after T2b (then T2c + T2d).
- **Content** (`h3_content::spell`, pack format v4): `SpellMagic` per spell: `target` (none,
  stack, hex), `area` (`inner`, radius by mastery), `chain` (stacks by mastery, factor),
  conditions (`requires`/`excludes` creature flags), `effects` (closed kinds; `damage` only).
  Validation: adventure spells have none, targeting needs effects, area only with a hex target,
  chain only with a stack target, limits (radius 16, 16 stacks, four effects). At most four spell
  schools (`MAX_SPELL_SCHOOLS`): `BattleMagic.schools` is indexed by the school's handle (its doc
  said "air, earth, fire, water", which the data order did not follow). Hero effects gain a fifth
  reference type (schools) and the kinds `school_mastery`, `spell_damage` (scope every spell, a
  school or a spell: `SpellScope`), `spell_book`, and `spell_specialty` a `bp_per_level`. Ruleset
  hash v14.
- **Harvest:** the executable's spell flag word is read (`HeroData::spell_flags`); the category
  bits are checked against `SPTRAITS.TXT`'s groups; the 13 damage spells get `target` and the
  `damage` effect from bits `0x10`/`0x80`/`0x200`; gap-fill `[[spells.targeting]]` adds areas,
  Chain Lightning and the conditions (VCMI offensive.json, the game's own spell texts agree);
  Sorcery from the executable's value table; the four magic skills, the Spell Book and the Orbs
  from the gap-fill file (38 artifacts still pending for E04b); spell specialists 3 % per level,
  Ciele and Luna a fixed `spell_damage` (wiki, SoD argument). Survey §11.
- **Seam** (additions, ADR-0014 point 10): `BattleEffect::SpellDamage { scope, bp }` and
  `SpellSpecialty { spell, bp }`; the battle validates them (unknown school or spell) and reads
  nothing yet; scenario and replay files cannot hold them yet (`HeroNotWritable`, joins in T2b).
- **Derivation:** HER-BAT-011 (spell book from a `spell_book` effect: the known spells; highest
  mastery per school; mana passed through; `max_mana` stays 0) and HER-BAT-012 (spell damage per
  scope summed, grown by a skill specialty; specialty `bp_per_level` × level).
- **Agent decisions (change freely):** no `flags` list in the pack: the flag word's targeting
  and damage bits are expressed by `target` and `effects` (the design note proposed both; T3 and
  T4 add the flags they read, mass at Expert and mind); the spell book as a hero effect on its
  artifact; Luna's Fire Wall bonus harvested now (data only, read in T5); the damage scope
  summing per scope as VCMI's bonus sums.
- **Verification:** golden replays re-recorded (only the ruleset hash changed; same battles).

### T2b — Casting in battle (2026-10-05)

- **Content:** at most 128 spells (`MAX_SPELLS`; spell books are bitsets); `SpellMagic.side`
  (any, enemy, own; the harvester writes `enemy` for single-target spells that harm, VCMI's smart
  targeting); battle parameter `hero_casts_per_round` (Tier 2, 1 in `base` from the gap-fill
  file). Ruleset hash v15.
- **Battle** (`h3_battle::magic`): `SpellSet`, `SpellBonuses` (every spell, per school, at most
  four single spells: `HeroError::TooManySpellBonuses`), the hero record's magic (book,
  masteries, bonuses, spells cast); `HeroView.casts` public, `OwnHeroView` gains the book,
  masteries and bonuses. `Command::Cast { spell, target: CastTarget }` (none, stack, hex; the
  second target joins with Teleport and Sacrifice); errors `NotInSpellBook`, `NotACombatSpell`,
  `AlreadyCast`, `NotCastable` (stub STUB-0001: only damage to one stack so far),
  `NotEnoughMana`, `InvalidSpellTarget`; events `SpellCast`, `SpellDamage`. Casts are not in
  `legal_commands`: `castable_spells` and `cast_targets` list them (the factorized action's
  heads). Rounds reset the casts; the outcome lists the distinct spells cast by handle (the
  contract's doc no longer says "in order"). State hash v18; `BattleState` 2 192 → 2 448 bytes
  (the hero record 40 → 168 bytes).
- **Rules:** BAT-MAG-001 (when a hero casts), BAT-MAG-002 (mastery and cost), BAT-MAG-003
  (targets), BAT-MAG-004 (damage); BAT-HER-001 and BAT-END-003 updated.
- **Formats:** scenario and replay heroes hold the spell book, the masteries and the spell
  effects (`PackError::HeroNotWritable` removed; `UnknownSpell`, `UnknownSchool`); the CLI's
  replay text `cast <spell handle> none|stack <id>|hex <hex>`; replay format v14. The sim's
  `encode` has no index for casts (T2c adds the factorized action), so the action encoding and
  the C ABI are unchanged.
- **Tests:** rule tests `tests/magic.rs`; the property tests cast in generated games (the lists
  agree with `apply` both ways; mana spent only by casts, at most the per-round limit; a cast keeps
  the turn; the book, masteries and bonuses stay private).
- **Agent decisions (change freely):** M-4 as VCMI (casting during a commanded war machine's
  turn; X-3); the damage computed exactly in basis points with truncation after each factor
  (VCMI rounds the specialty term to whole percent first; X-1); the damage computed during
  validation (the plan carries it); the spells cast as a set by handle (Eagle Eye needs the set);
  four single-spell bonuses per hero.
- **Verification:** golden replays re-recorded (state hash v18, replay format v14; the same
  commands).


### T2c — The sim (2026-10-06)

- **Checkpoint:** the user confirmed T2c + T2d with batch 2a and kept T2b's three decisions
  (Q44). The baseline verify of `84d877b` (T2b) was green in full this session (coverage,
  determinism, bench-check −1.1 %, original data).
- **Stack slots:** `SIDE_STACK_SLOTS` = 21, `STACK_CAPACITY` = 42 (ADR-0014 point 6; a compile-time
  check that army slots and war machines fit). The creation limit of 20 moves to T6 with its
  readers. `BattleState` 2 448 → 2 624 bytes.
- **Action encoding v9** (`h3_sim::action`): `Action { entry, spell, target, second }`; the dense
  entries grow to 42 stack slots and gain `HERO_CAST` and `CREATURE_CAST` (never legal until T7);
  heads: spell by handle (`SPELL_COUNT` = `MAX_SPELLS` = 128), targets `0` none, `1 + h` hex,
  `256 + s` stack (`TARGET_COUNT` = 298). Masks per head conditioned on the earlier choices
  (`legal_mask`, `spell_mask`, `target_mask`, `second_mask`); `BattleEnv::step` takes an `Action`.
  `END_TACTICS`, `RETREAT`, `SURRENDER` are public.
- **C ABI v8:** `H3Action` by value in `h3_env_step`, `h3_spell_count`, `h3_target_count`,
  `h3_env_spell_mask`, `h3_env_target_mask`, `h3_env_second_mask`; `include/h3.h`; the C smoke
  test plays the full scenario choosing every head from its mask (ADR-0011 amended).
- **Observation encoding v5:** the hero block gains its casts this round (public), the four
  school masteries and the spell book as 128 features (private); 42 stack slots.
- **Workload:** both heroes of `full` cast (Brea: the four damage spells and Ward, Basic Storm,
  power 2, 40 mana; Corwin: Spark, Arc, Blight, Advanced Stone, power 3, 40 mana). The random agent
  (`rollout::choose`) treats the hero's cast as one more entry, then takes a uniform castable spell
  and a uniform target; scenarios without spell books draw exactly as before (`duel-s1` and
  `skirmish-s7` unchanged). `full-s3` re-recorded: 12 casts (only Spark is castable until T2d).
- **Measurements** (this sandbox, 2 cores): `h3 bench --games 20` 11 100 steps instead of 13 744
  (spells shorten the battles), 457 942 318 Ir instead of 509 946 255: ≈ 41.3 k Ir per step
  instead of ≈ 37.1 k (+11 %: the agent asks for the castable spells every step); `bench-check`
  re-baselined for that reason. `h3 bench` full 300 games ≈ 148–150 k steps/s, the T2b build in the
  same sandbox ≈ 138–160 k (noise); after the change `check_spell` borrows the hero record
  instead of copying its 168 bytes. Criterion: `clone BattleState` ≈ 300 ns, `BattleEnv::fork` ≈
  1.23 µs (the T2b build in the same sandbox, measured under load, ≈ 400 ns and 1.46 µs: this
  sandbox's wall clock is too noisy to compare; E04a's 183 ns came from another machine).
- **Tests:** layout, round trips, the non-cast heads, an exhaustive cast check (every spell of the
  ruleset and every target: the masks allow exactly what `apply` accepts, and allowed tuples
  round-trip), truncated masks, the observation's hero blocks, the ABI's casting path and error
  codes.
- **Agent decisions (change freely):** after a legal non-cast entry each head allows exactly 0
  (one sampling rule for every entry); a non-cast entry with a non-zero head does not decode;
  `CREATURE_CAST` reserved now so T7 does not move the entries; the spell book in the observation
  as one feature per spell handle.

### T2d — Area, chain and battle-wide damage; immunities and resistance (2026-10-06)

- **Code reading** (`docs/formats/installation.md` §7.8, survey §12): the damage order, the
  immunity and resistance test, Chain Lightning's jumps and the area handler from the SoD
  executable; settles M-1, M-2, M-6 and X-1's rounding, most of X-6. Where VCMI differs (the
  target's reductions first, truncation after every factor, chain by hex distance), the original
  wins.
- **User decisions (Q44):** `BattleEffect::SpellSpecialty` carries the per-level rate and the
  hero level (contract change); Chain Lightning by screen geometry (hex centres 44 × 42 pixels,
  the two-hex shift, the code's tie order, no sprite offset: Q43), a pixel-vs-hex case in batch
  2a; magic resistance single precision under Q-0005.
- **Content:** abilities `spell_level_immunity`, `school_immunity`, `spell_immunity`,
  `spell_vulnerability`, `spell_damage_reduction`, `magic_resistance`, `magic_resistance_aura`
  (codes 22–28; `Ability` gains school and spell references); hero effect `magic_resistance`
  (code 25); the ruleset resolves abilities after spells and schools; ruleset hash v16. The
  harvester reads Resistance's value table (skill 26), concatenates a creature's ability entries
  (one per topic, with its sources) and checks `fire_immune` against `school_immunity` fire; the
  gap-fill file gains the creatures' magic abilities and the three resistance artifacts.
- **Heroes:** HER-BAT-012 (specialty with level), HER-BAT-013 (one `MagicResistance` effect per
  source, the skill grown by a Resistance specialty).
- **Battle:** `magic` rewritten after the code: BAT-MAG-003 (stack, hex and battle-wide targets),
  BAT-MAG-004 (the original's order), BAT-MAG-005 (immunities), -006 (Chain Lightning), -007
  (area and battle-wide spells, the aimed stack's roll first), -008 (resistance; event
  `SpellResisted`); `h3_kernel::arith::round_single`. STUB-0001 removed: every damage spell is
  castable. State hash v19, replay format v15.
- **Tests:** `tests/spells.rs` (immunities and reductions, resistance chances exact and single,
  the aimed stack's fizzle, areas and battle-wide spells, the chain's screen order and skipped
  resisting stacks, the hex target list); `the_casters_bonuses_apply_in_the_originals_order`;
  property tests cast the new spells; harvester and pack tests for the new kinds.
- **Measurements:** `BattleState` 2 624 → 2 640 bytes (the hero's resistance). `h3 bench --games
  20` 8 955 steps instead of 11 100 (area and battle-wide spells end battles sooner), 365 502 439
  Ir: ≈ 40.8 k Ir per step (T2c ≈ 41.3 k); `bench-check` re-baselined for that reason. Golden
  replays re-recorded (`duel-s1`, `skirmish-s7`: the same commands; `full-s3`: 9 casts).
- **Agent decisions (change freely):** a hex target must be standable, its stack (if any)
  reachable, and the area must reach a stack (VCMI; the original's validation not read); S-18
  cannot occur from a hero's cast (power
  ≥ 1 gives at least 20 before the Golems' reduction) and stays for creature casters (T7);
  Titan's Lightning Bolt's elemental entries wait for its castability (T8, BACKLOG).

### Batch 2a compiled (2026-10-06)

- **Tool:** experiment specs gain `spells` (`SPTRAITS.TXT` names, ids below 70, a worn spell book
  required); `--list spells`; spell specialties print by name. Mana comes from knowledge (a map's
  hero starts with full mana).
- **Spec** `experiments/specs/E04b-batch-2a.toml`, template `experiments/results/E04b-batch-2a.md`,
  map `experiments/out/E04b-batch-2a.h3m` (7 encounters, 7 Red and 6 Blue heroes): R1 Chain
  Lightning screen vs hex distance (Q44: B 88 px / 2 hexes left of A, C 84 px / 2 hexes beneath;
  every hex model picks B, the screen picks C), R2 its tie order (the code: army slot; VCMI: the
  smaller column), R3 aiming Implosion at a Ballista, Fireball at an empty hex and at an immune
  Efreeti, then six Fireballs centred on Battle Dwarves (the whole spell fizzles 40 %), R4 Alagar
  at level 1 with Basic Sorcery (119 / 115 / 17 / 230 against VCMI's 118 / 116 / 16 / 231), R5
  an Orb with Sorcery (173 and 25 against 172 and 24), R6 costs with Mages, Pegasi, Familiars
  and a cast in the Ballista's turn, R7 Q40.
- **Not staged:** Q-0005's single-precision resistance (one point of a chance), Q43, the E04a
  candidates (1)–(4). A detail found while predicting: the original grows a Sorcery specialist's
  rate in single precision (`f32((0.05 × level + 1) × 0.15)`), slightly above the decimal value,
  while HER-BAT-004 grows it in whole basis points before Q-0005 converts it, slightly below; the
  two can differ by one damage point when the decimal product is a whole number (level 10, value
  200: 245 against our 244). Recorded for the Q-0005 review with batch 2a's results.

### T2e — Batch 2a folded in (2026-10-06)

- **Seam:** the user chose T2e alone (fold batch 2a in, close increment 1) before increment 2.
- **Results** (`experiments/results/E04b-batch-2a.md`, evaluation in the survey §13): confirmed
  Chain Lightning's screen geometry and tie order (R1, R2; Q44), the damage order and rounding
  (R4, R5: 119/115/17/230, 173/25), the immunities (R3, R4), the cast in a commanded Ballista's
  turn (R6; X-3), the Familiars' 20 % rounded down (R6, for T7) and speed ties by army slot
  (R7e). Refuted: Chain Lightning at an own stack (R1 round 2: not allowed), Fireball at an
  empty or an immune stack's hex (R3b/c: allowed), the area spell's fizzle when its centre stack
  resists (R3: Gnolls struck by all six casts, Dwarves resisting three), the tactics hand-over
  (R7b/c). Open: R6 round 3 cost 5 with the Pegasus dead (VCMI 3; T7).
- **Code re-read** (`installation.md` §7.8): the aimed roll at 0x5A02EA belongs to spells that
  take a target stack (0x59E060), not to area spells; the cursor 0x59F700 and the target test
  0x5A39C0 (side columns refused; a target stack needs `f > 0`; any other hex accepted; Chain
  Lightning refuses own stacks at 0x59F7BF); flag `0x20` = mass from Advanced (§7.6, S-10).
- **Battle:** BAT-MAG-003 (`aimable`: side, immunity and a share above 0; hex spells any
  standable hex; `has_target` simplified), BAT-MAG-007 (no aimed roll for hex spells), the
  resistance split into `share` and `aura` (the aura's abilities checked before the geometry),
  `SpellDamage { value, caster, damage, kills }`; BAT-TAC-002 (`tactics::placed`/`skip` with the
  ACTED/WAITED flags in round 0; Wait emits `Waited` when the stack waits). No version changes
  (the state already hashes the flags; events are not encoded).
- **Content:** the gap-fill file's spell targeting gains `side` (only for a stack spell whose
  side the flags and the polarity do not give); Chain Lightning `side = "enemy"`; packs
  regenerated (`base` one line; the ruleset hash value changes, not its version).
- **Tests:** spells (aimed stack vs area centre, any hex, the immune centre spared, no target
  that always resists, Chain Lightning only at enemies, the event's amounts), tactics (R7's
  sequence: waits, passes, new rounds, placements), harvester (`side` from the gap-fill file and
  its errors).
- **Measurements:** `h3 bench --games 20` 8 993 steps (T2d 8 955), 374 856 140 Ir: ≈ 41.7 k Ir
  per step (T2d ≈ 40.8 k, +2.1 %: `castable_spells` now checks each stack spell's target for a
  share above 0); `bench-check` re-baselined for that reason. `full-s3` re-recorded (the
  tactics hand-over and the hex targets change the random agent's draws: 7 casts, 378 commands
  instead of 9 and 776); `duel-s1` and `skirmish-s7` unchanged.
- **Agent decisions (change freely):** a hex spell aimed where it strikes nobody is legal and
  listed in the cast mask (faithful; ADR-0014 lists exactly what `apply` accepts; pruning useless
  casts is the agents' business, E05); Wait in the tactics phase emits `Waited` only when the
  stack starts waiting.
- **Not done:** the Sorcery specialist's growth in single precision turned out to concern every
  skill specialty and to need a contract change: OPEN-QUESTIONS Q45 for the checkpoint.

### T3a — The effect list and the value spells (2026-10-06)

- **Seam:** increment 2 is T3 in three parts, then T4 (Q46); the user chose T3a alone, with a
  checkpoint after it to review the effect list and its measurements. Q45 decided: option (b),
  in T3c.
- **Survey** (`installation.md` §7.9, survey §14): the executable's timed-spell code decides the
  rules: duration = power + 0x4E4DB0 (Collar +1, Ring +2, Cape +3 of Conjuring, Ring of the Magi
  +50 with its parts); a cast changes the stack's record (Weakness takes at most the attack,
  Stone Skin, Prayer and Haste add) and the end gives it back; Bloodlust, Precision, Bless,
  Curse, Slow, Mirth, Sorrow, Fortune and Misfortune are read while active; durations fall for
  every stack at the start of a round (M-3 settled); a dead stack loses its spells; Disrupting
  Ray lowers the defense at every cast (at least to 0) and is never ended (S-17 narrowed to
  Berserk and Bind); only Bless/Curse and Haste/Slow end each other; a re-cast keeps the first
  value and the longer duration. The user chose the executable over VCMI on both points and
  amended ADR-0014 point 5 (Q47).
- **Content (pack format v5):** `SpellEffect::Timed(TimedChange)` with 14 changes,
  `SpellMagic.mind`, `.mass_from`, `.duration`, `.cumulative`, `Spell.counters` (resolved after
  the pack's spells are registered); hero effect `spell_duration`; HER-BAT-014 (one
  `BattleEffect::SpellDuration`, the sum, each artifact type once, parts included). The harvester
  derives target, side, mass, `mind` and the war-machine exclusion from the flag word and takes
  the changes, counters and Disrupting Ray's permanence from `[[spells.timed]]` (source
  `exe-timed-spells`, an `original-data` source that names a `docs/` file); the Conjuring
  artifacts and the Ring of the Magi leave the pending list. Ruleset hash v17.
- **Battle:** `h3_battle::effects`: the effect list (one entry per stack and spell, sorted, stack
  runs found by binary search, 32 entries reserved at the start and on every fork), each entry
  with rounds (or permanent), the first value and what it added to attack, defense and speed;
  `SpellValues` on each stack recomputed from its entries; `Stack::attack_now`, `defense_now`,
  `speed_now` (movement, turn order, tactics hand-over); Bless/Curse fix the base damage; morale
  and luck: the side's sum clamped, then the spells, then caps and minimums; effects fall at the
  round start and end with `EffectEnded`; dead stacks lose them at the end of the command. Mind
  immunity (BAT-MAG-005), mass versions (BAT-MAG-003), BAT-MAG-009 to 011. Attack and defense are
  signed in the damage formula (below 0 after Disrupting Ray and the end of Stone Skin). State
  hash v20.
- **Sim and ABI:** observation encoding v6 (attack, defense, speed with the effects; seven effect
  values and eight effect slots per stack; 54 features per stack), C ABI v9 (ADR-0011 amended);
  the action encoding is unchanged (mass versions use the target `none`).
- **Workload:** the synthetic pack gains eight timed spells (every change, mass versions, a mind
  spell, counters, a permanent cumulative spell) and the Storm Orb `spell_duration`; in `full`
  Brea knows Hasten, Drag and Fury (Basic Storm), Corwin Zeal, Gloom, Boon, Bane and Corrode with
  Expert Stone and a duration bonus of 2. All three golden replays re-recorded (the ruleset
  hash; `full-s3`: 7 casts, three of them timed).
- **Measurements** (this sandbox, 2 cores): `BattleState` 2 640 → 3 336 bytes (16 bytes of
  materialised spell values per stack slot, 672; the list's header, 24); a fork allocates once
  (512 bytes reserved). Criterion: `clone BattleState` ≈ 264 ns, `BattleEnv::fork` ≈ 1.44 µs
  (T2c in this kind of sandbox ≈ 300 ns and 1.23 µs; noisy). `h3 bench` full ≈ 179 k steps/s.
  Engine overhead without timed spells: the T2e scenario on the new engine 375 486 125 Ir for the
  same 8 993 steps (+0.17 %). New workload: 10 162 steps, 451 356 012 Ir (≈ 44.4 k Ir per step:
  longer battles and larger spell books for `castable_spells`); `bench-check` re-baselined for
  that reason.
- **Tests:** `tests/timed.rs` (durations and the bonus, counters and the coexisting pairs,
  re-cast, Weakness and Disrupting Ray at 0 and a negative defense, mass versions and mind
  immunity, Prayer on a war machine, death, Bloodlust/Precision/Bless in damage, Slow's rounding
  exact and single, morale and luck stages, public and hashed effects); the property tests cast
  every timed test spell and check the list (sorted, living stacks, values equal to a
  recompute); content, pack, harvester, derivation and observation tests.
- **Agent decisions (change freely):** a duration of at least 1 round (power 0 does not occur in
  the original); several `speed_percent` effects: the strongest; several `maximum_damage` or
  `minimum_damage` effects add, and a maximum wins over a minimum (the original checks Bless
  first); a dead stack's effects end without an event; the Defend bonus of a defense below 0 is
  the minimum; the observation shows a stack's first eight effects.
- **Not done (later tasks):** the specialties in timed spells (T3c; `SpellSpecialty` stays
  damage-only); creature casters' re-casts (T7); Magic Mirror on negative timed spells (T4).

### T3b1a — Staged physical damage and the protective spells (2026-10-06)

- **Seam:** the user had the original files staged from the repo folder through the device link
  (Q49 answered) and chose: read all of survey §15, then build Shield, Air Shield, the four
  Protections, Slayer and Counterstrike, checkpoint before Dispel and Cure (T3b1b).
- **Code reading** (`installation.md` §7.10, survey §15 results): what the five spells store;
  physical damage in three truncated stages (0x443F40); Slayer's king levels (king 1 at any
  mastery); retaliations reset before the durations fall; Dispel and Cure read for T3b1b.
- **Damage formula (Q50, the user: stage it now):** BAT-DMG-002 under quirk Q-0005 follows the
  executable: integer bonuses each truncated in the code's order (jousting, luck, attack
  advantage, hate, Offense/Archery, death blow, double damage), then the defense and penalty
  factor, then the protection factor (Shield or Air Shield and Armorer), each truncated, at
  least 1; the code's doubles (0.05, 0.025, 0.3, 3.0) and single-precision skill and spell
  factors in an exact x87 emulation (`h3_kernel::arith::X87`, 53 bits; cross-checked against
  IEEE doubles by a property test). Strict-fixed keeps exact decimals rounded once
  (`damage::Strike` replaces `Factors`). Agent decision: the x87 precision is Windows' default
  of 53 bits (the game never sets it); batch 2b tells it from 64 bits.
- **Content (pack format v6):** five timed changes (`melee_protection`, `ranged_protection`,
  `school_protection`, `slayer`, `retaliations`); `protection_rank` on spell schools (the
  original's check order earth, air, fire, water); quirk Q-0007 (`quirk.battle.first_protection`,
  S-19). Harvested from the gap-fill (source `exe-protection-spells`); ruleset hash v18.
- **Battle:** BAT-MAG-012 to 015. `SpellValues` gains the melee, ranged and school protections
  and Slayer (bonus and the first cast's mastery; `ActiveEffect.mastery`); Counterstrike adds
  its value at the cast and at every round start from the effect list, before the countdown;
  the Protections join spell damage after the creature step (`magic::protected`). State hash
  v21.
- **Workload:** the synthetic pack gains Aegis (melee and ranged protection), Hallow (storm
  protection), Smiting (Slayer; the Brute becomes king 1) and Riposte (Counterstrike); in `full`
  Brea knows Smiting and Riposte, Corwin Aegis and Hallow. Golden replays re-recorded.
- **Measurements** (this sandbox): the old workload (af5022f's packs) on the new engine 461.2 M
  Ir for 10 273 steps against 451.4 M for 10 162 (+1.1 % per step; the trajectories differ with
  the new damage; the staged damage costs ≈ 4 M Ir: `X87::times`, `ratio`). New workload 10 907
  steps, 490.2 M Ir, `bench-check` re-baselined. `BattleState` 3 336 → 3 672 bytes (`Stack` 60 →
  68); clone ≈ 340 → 397 ns, fork ≈ 1.10 → 1.28 µs (both engines measured in this session).
  `h3 bench` full ≈ 155–170 k steps/s.
- **Tests:** `tests/protect.rs` (Shield and Air Shield in melee and shots, the Protections in
  three profiles, Slayer by mastery and king level, Counterstrike now, at round starts, on
  re-cast and with unlimited retaliations); `tests/rules.rs` (batch 1a R1–R3 staged, the stages'
  truncations, the profile flag); `arith.rs`/`num.rs` (X87 vectors and the double cross-check).
- **Not done:** Dispel and Cure (T3b1b); Fire Shield and Frenzy (T3b2); the specialties
  (T3c); petrification's ×0.5 in stage 3 and the Psychic and Magic Elementals' halvings in stage
  2 (T7 and BACKLOG); the Blind and Paralyze retaliation factors (T4, T7).

### T3b1b — Dispel and Cure (2026-10-07)

- **Seam:** the user chose T3b1b and T3b2 for this session (checkpoint before T3c) and
  classified Counterstrike's extra round as fix-optional (Q-0008, `quirk.battle.
  counterstrike_extra_round`: without it the round start gives no extra retaliations for an
  effect that ends at it).
- **Code reading** (`installation.md` §7.10, survey §15 point 7 settled): Dispel's test
  (0x5A8090 for spell 35) returns 1 before the immunity, resistance and aura tests, so no stack
  is immune to Dispel and none resists it (the first reading had "then the usual immunity
  tests"); the Sphere is checked on the controlling hero (0x442690). Cure ends its 14 spells,
  then heals.
- **Content (pack format v7):** effect kinds `dispel` and `heal`; `any_side` (a mastery from
  which the side is any), `needs_effect`, `ignores_immunity`; `dispels`/`spares` on the spell
  (resolved like `counters`; not both, a dispel effect needed). Harvested from a new gap-fill
  list `spells.other` (source `exe-dispel-cure`); the harvester's stack-spell targeting is shared
  with the timed spells (a neutral spell names its side). Ruleset hash v19. Agent decision
  (change freely): a permanent effect is never dispelled, standing for the original's end
  routine refusing Disrupting Ray; Cure's list is explicit, Dispel's an exception list.
- **Battle:** BAT-MAG-016 (`effects::dispel`), BAT-MAG-017 (heal in `magic::affect`, event
  `SpellHealed`); `affect` applies damage, dispel, heal, then timed changes; `reachable` and
  `aimable` take the mastery (side by mastery, a target with an effect); `affects` and
  `affect_chance` honour `ignores_immunity`. No state change (state hash v21 stays).
- **Workload:** the synthetic pack gains Purge (Dispel, stone; Corwin casts it mass at Expert)
  and Mend (Cure, storm; Brea); golden replays re-recorded (duel and skirmish: ruleset hash
  only).
- **Measurements** (this sandbox): the old workload (e12e312's packs with the new flag on) on the
  new engine: 10 907 steps as before, 491.9 M Ir against the baseline's 490.2 M (+0.35 %). New
  workload 10 613 steps, 483.2 M Ir, `bench-check` re-baselined. `BattleState` 3 672 bytes
  (unchanged). `h3 bench` full ≈ 136 k steps/s.
- **Tests:** `tests/dispel.rs` (Dispel by mastery, spares and permanence, immunity and
  resistance ignored, castable at Expert without effects; Cure's order, cap, healthy target,
  mass), `tests/protect.rs` (Counterstrike in both profiles), content and pack tests for the new
  fields.
- **Moved out:** the Sphere of Permanence to T8 (one hero effect with the Pendants, a seam
  addition); Expert Dispel's removal of the spell obstacles to T5; Magic Mirror, hypnotized
  stacks to T4; Dispel Helpful Spells to T7 (BACKLOG).

### T3b2 — Fire Shield and Frenzy (2026-10-07)

- **Seam:** the user's (T3b1b and T3b2 in one session, checkpoint before T3c).
- **Code reading** (`installation.md` §7.11): Fire Shield's reflection is computed inside the
  physical damage routine from stage 1, capped at the target's hit points, passed through the
  spell-damage modifiers with the target's hero as the caster; shots drop it; only the attacked
  stack reflects, a multi-headed attack sums; it lands before the retaliation. Frenzy: 1 or 2
  turns of the stack, counted at its own turn start (once per round); attack + own defense ×
  factor as the struck stack sees it (S-16 confirmed), defense 0 when struck. The Behemoths'
  reduction truncates the share taken, so the defense left rounds up: the user chose to follow
  it under Q-0005 and keep Q19 in strict-fixed (BAT-DMG-006).
- **Content (pack format v8):** timed changes `fire_shield` and `frenzy`, duration `own_turn`;
  quirk Q-0009 (`quirk.battle.frenzy_reduced_defense`, S-16). Harvested from the gap-fill
  (source `exe-fire-shield-frenzy`).
- **Battle:** BAT-MAG-018 (`magic::reflection`, `combat::reflect`, event `Reflected`;
  `Strike::stage_one`; `spell_damage` takes the spell and the caster's bonuses instead of a
  cast), BAT-MAG-019 (`damage::frenzied`, `effects::own_turn` called at a stack's first
  activation of the round, `count_down` skips own-turn effects), BAT-DMG-006 (`reduced_defense`
  with the executable's rounding under Q-0005). `SpellValues` gains `fire_shield` and `frenzy`:
  state hash v22, `Stack` 68 → 72 bytes, `BattleState` 3 672 → 3 840 bytes.
- **Workload:** the synthetic pack gains Blaze (Fire Shield, Brea) and Rage (Frenzy, Corwin);
  golden replays re-recorded (the state hash changes every checkpoint).
- **bench-check:** the new binary hit a valgrind 3.22 limitation (no symbols read from an
  lld-linked executable whose two writable segments the kernel merges: the RELRO end fell on a
  page boundary), so the gate measured 0. `xtask bench` now links the measured `h3` with `-z
  norelro` (`cargo rustc`, the final link only). Agent decision (change freely).
- **Measurements** (this sandbox): the old workload (9431440's packs) on the new engine 10 613
  steps as before, 485.7 M Ir against 483.2 M (+0.53 %: the larger stack, the Frenzy and Fire
  Shield checks per strike). New workload 9 141 steps, 426.3 M Ir; `bench-check` re-baselined.
  Clone ≈ 513 → 598 ns, fork ≈ 2.20 → 2.00 µs (criterion, both engines in this sandbox, which
  is noisy: the fork moved the other way); `h3 bench` full ≈ 114 k steps/s.
- **Tests:** `tests/rage.rs` (the reflection's base, cap, retaliations, shots, fire immunity,
  breath and multi-headed attacks, the caster's bonuses and the striker's reductions; Frenzy's
  attack and defense by mastery, rounding, S-16 in both profiles, the own-turn countdown in
  play), `tests/abilities.rs` (BAT-DMG-006 in both roundings).
- **Left:** the Efreet Sultans' own shield (T7), a clone's health of 1 for the cap (T6), a
  hypnotized stack's controller (T4); two quirk candidates for the user (BACKLOG).

### T3c1 — The spell specialists, quirk Q-0010 (2026-10-07)

- **Seam:** the user classified Fire Shield's base as fix-optional and kept the breath's second
  stack as the rule in every profile (considered, no flag), and cut T3c in three: the
  specialists (T3c1) and Q45 option (b) (T3c2) this session, a checkpoint before batch 2b (T3c3).
- **Q-0010** (`31fbd01`): `quirk.battle.fire_shield_first_stage`; strict-fixed reflects from the
  hit points the strike took. The same commit fixes the x87 property test's helper (a
  difference below 2^-12 panicked a coverage run; now a unit test).
- **Code reading** (`installation.md` §7.12): 0x4E5FF0 switches on the spell: Bloodlust,
  Precision, Weakness, Stone Skin, Prayer, Haste by the stack's level (3, 3, 2, 2, 1, 1, 0),
  Slayer (4, 3, 2, 1, 0, 0, 0), Disrupting Ray 2, Fortune 3 − v, Magic Arrow and Fire Wall as
  known, else the scaled formula. Its callers: the cast of the seven timed spells (caster,
  target), Disrupting Ray, Cure, Bless in stage 1 after Offense or Archery and Slayer's attack
  (the striker's side's hero, the striker's level). VCMI agrees except Bless (an additive
  premium).
- **Content (pack format v9):** `SpellMagic.specialist` (`SpecialistBonus`: creature level,
  fixed, total), harvested from the gap-fill (`[[spells.specialist]]`, sources
  `exe-spell-specialist`, `vcmi-spell-specialist`); ruleset hash v20.
- **Battle:** BAT-MAG-020 (`magic::Specialist`, `specialist_bonus`): at the cast for a spell
  with a rule of its own (`TimedCast.own`), Cure's heal, the damage spells (as before), Slayer
  and Bless at the strike (`combat::specialist_terms`, `Strike::blessing`). `WideUint` gains
  `saturating_add`. No state change.
- **Agent decisions (change freely):** Slayer's bonus is added at the strike, not stored at the
  cast (as the original; only a hypnotized stack would differ); in strict-fixed a Bless
  specialist's bonus is a whole amount rounded up, as BAT-MAG-004's, added to the exact first
  stage before the one rounding; a timed spell without a rule of its own gets no scaled bonus at
  the cast (the original's call sites); levels above 7 read the table's last value.
- **Workload:** Brea is a level-6 Hasten specialist, Corwin a level-1 Boon (Bless) specialist;
  golden replays re-recorded. The old workload (`31fbd01`'s packs) on the new engine: 9 141
  steps, 426.55 M Ir against 426.33 M (+0.05 %); the new one 8 818 steps, 416.84 M Ir;
  `bench-check` re-baselined. `h3 bench` full ≈ 100–130 k steps/s (noisy sandbox).
- **Tests:** `tests/specialist.rs` (the creature-level table through a cast and on levels 1–7,
  another spell's specialty, Mirth without a rule, Weakness before the cut, Disrupting Ray to 0,
  Fortune's total, Slayer and Bless at the strike in both roundings, Cure by level); the pack
  and harvester tests for `specialist`.
- **Left:** a hypnotized stack's controller as the hero (T4), the raising spells' and
  Hypnotize's specialist values (T6, T4).

### T3c2 — Q45 option (b): the hero's rates in the original's single precision (2026-10-07)

- **Code reading** (`installation.md` §7.4 confirmed): each skill getter grows its value as
  `f32(r × (f32(level) × 0.05f + 1))` in x87 arithmetic; Archery's getter then adds the three
  bows (artifacts 60, 61, 62; only with the skill), Resistance's the Garniture, Surcoat and
  Boots (57, 58, 59; also without the skill), each sum stored as `f32`, in id order.
- **Contract (Q45 (b), agent's shape):** `h3_contracts::Rate { bp: u32, single:
  Option<NonZeroU32> }`, `single` the original's value as an f32 bit pattern where it differs
  from the single-precision value of `bp`; `DamageBonuses { ranged, melee, reduction }`,
  `SpellDamage { scope, rate }`, `MagicResistance { rate }`, `TentControl { heal_bonus }`. A
  numerator over 2^40 (Q45's wording) made `BattleState` 352 bytes larger; the bit pattern
  costs 64. The surrender discount keeps basis points (no Diplomacy specialist exists).
- **Derivation:** `Growth`, `RateSum` (HER-BAT-004/005/012/013): under Q-0005 the grown values
  and the artifacts' sums in the original's order (artifacts by type for every rate source
  now, so the `SpellDamage` effects of artifacts follow the types' order); `h3-heroes` depends
  on `h3-kernel` (`arch.toml`).
- **Battle:** `magic::hero_rate`, `add_rates`; the strike's hero bonus and Armorer
  (`Strike::hero_bonus`, `hero_reduction`), Sorcery and the Orbs, magic resistance, and the
  tent's largest heal under Q-0005 `⌊max × (1 + bonus)⌋` in x87 (BAT-WAR-007: Rion at level 4
  with Expert First Aid heals at most 114, exact 115). `Plan::Cast` is boxed (the larger
  `SpellBonuses` tripped clippy's `large_enum_variant`). `X87::from_f32_bits`, `to_f32_bits`,
  `fixed`, `to_fixed`.
- **Formats:** scenario and replay files gain optional `ranged_single`, `melee_single`,
  `reduction_single`, the tent's and the effects' `single`; replay format v16; state hash v23
  (`BattleState` 3 840 → 3 904 bytes). The workload is unchanged (8 818 steps); 419.80 M Ir
  against the T3c1 baseline's 416.84 M (+0.71 %, within the gate; not re-baselined).
- **Also:** the coverage run of `54f9a65` found `Strike::stage_one`'s staged branch unexecuted
  (the new golden replays no longer reach it); a test profile with Q-0005 and Q-0010 covers it.
- **Tests:** `tests/derive.rs` (an Archery specialist with the loop and the eye, Q45's
  Logistics example, a First Aid specialist at level 23, normalised rates), `tests/rules.rs`
  (Q45's 245 against 244), `tests/magic.rs` (the same through a cast), `tests/war.rs` (the
  tent's 114), `tests/heroes.rs` (a reduction above 1 rejected), `arith.rs` (bit patterns,
  round trip property), the pack's scenario round trip with single values.

### T3c3 — Batch 2b compiled (2026-10-07)

- **Seam (Q52):** batch 2b now, its predictions from the finished T3 engine; the user plays it
  while T4 proceeds; the results are folded in at a later checkpoint.
- **Spec** `experiments/specs/E04b-batch-2b.toml`, template `experiments/results/E04b-batch-2b.md`,
  map `experiments/out/E04b-batch-2b.h3m` (6 encounters, 6 Red and 6 Blue heroes): R1 4 and 8
  Angels against one defense point more (53 bits 195 and 390, 64 bits 194 and 389; Q50); R2
  Counterstrike at power 1 cast in round 2, retaliations in rounds 2–4 (faithful: two in round 3,
  Q-0008); R3 10 Naga Queens (no retaliation) on Fire-Shielded Peasants at a 20-point defense
  deficit (150 dealt; reflection 60 from stage 1, Q-0010, against 30); R4 Shield with Basic
  Armorer on Angels at equal attack and defense (150: 121 in one product against 120 truncated
  per factor or added; 50: 40 against 39) and Armorer alone (200: 189 with Q-0005 against 190);
  R5 Bloodlust then Weakness (both listed, Q47) and Haste then Slow (Slow ends Haste); R6
  Mephala at level 10 with Expert Armorer under Angels at the attack cap (800: 619 with the
  single-precision growth against 620; 200: 154 against 155; levels 9 and 11 tabulated).
- **Checked against the engine:** R1, R4 and R6 through `damage::Strike` and the derivation's
  growth (bit pattern `0x3e666667`) in a scratch test (not committed); the 64-bit alternative
  from exact rational arithmetic.
- **Design notes for the player:** the stacks whose defense matters never Defend before they
  are struck (the Angels act first, or the target moves instead of defending); Mephala's
  level-ups are the user's choice and cannot change R6 (the attack bonus is capped).

### T4a — The spell defences: Anti-Magic and Magic Mirror (2026-10-07)

- **Seam (Q52):** T4 in three parts by mechanism; T4a with T3c3 this session, a checkpoint after.
- **Code reading** (`installation.md` §7.13): Anti-Magic stores v (4, 4, 5, 6) and ends the
  active negative spells below level v that are not creature abilities (Sorrow and Berserk, level
  4, survive a Basic one; Disrupting Ray never ends); while active, 0x5A8090 refuses every spell
  below level v whatever its polarity, after Dispel's own test. Magic Mirror stores its chance;
  the redirect lives where a player's cast is entered (0x59EC50): a single-target cast at a
  living enemy stack, Dispel excepted, goes with `random(1, 100) ≤ chance` to a uniformly drawn
  living stack of the caster's side (arrow towers excepted, war machines not), which rolls with
  the polarity tests skipped; the first target never rolls. Also found: a stack with Magic Mirror
  cannot be hypnotized (T4c); a clone cannot receive Anti-Magic (T6); the computer player's
  casts skip the redirect (the original's AI only). VCMI differs on Anti-Magic's dispel (every
  negative and neutral effect) and leaves war machines out of the redirect.
- **Content (pack format v10):** timed changes `spell_immunity` and `magic_mirror`; the gap-fill
  gives them to Anti-Magic and Magic Mirror (sources `exe-anti-magic-mirror`,
  `vcmi-spells-defences`); `ignores_immunity` (Dispel) also passes Anti-Magic and the mirror.
  The synthetic pack gains Veil (an Anti-Magic) and Glass (a Magic Mirror).
- **Battle:** BAT-MAG-021 (`magic::kept_off`, `affects_stack` in every reach test: stack spells,
  areas, mass and battle-wide spells, Chain Lightning's jumps; `effects::apply` ends the
  negative effects below the level at the first cast, through `end_where`, which Dispel now
  shares); BAT-MAG-022 (`magic::mirror` in `apply_cast`, `SpellRedirected`; the new stack takes
  the spell through `affects_stack` and the resistance roll). `SpellValues` gains
  `spell_immunity` and `mirror`; state hash v24.
- **Agent decisions (change freely):** the chance is drawn only when the stack has a mirror (the
  original draws for every cast at an enemy stack; draws are ours, ADR-0003); Bloodlust's and
  Precision's inline bonuses are `u8` now (saturating at 255), so the two new values keep the
  stack record at 72 bytes and `BattleState` at 3 904.
- **Workload:** Brea casts Glass, Corwin Veil; golden replays re-recorded. The old workload
  (`8c90804`'s synthetic pack) on the new engine: 421.11 M Ir against T3c2's 419.80 M (+0.31 %;
  +1.02 % on the T3c1 baseline); the new one 10 070 steps (was 8 818), 472.93 M Ir;
  `bench-check` re-baselined. `h3 bench` full ≈ 190 k steps/s (noisy sandbox).
- **Tests:** `tests/defences.rs` (the cast's dispel by level and polarity, own and enemy spells
  kept off, areas and battle-wide spells passing over, Dispel ending it, a Basic one keeping
  level 1, abilities never kept off; the redirect of damage, timed and chain spells, no redirect
  for areas, own targets and Dispel, an immune or resisting new stack, the chance and the draw
  among living stacks).

### T4b — Stacks that lose actions: Forgetfulness and Blind (2026-10-07)

- **Seam (Q53):** the user chose T4b + T4c in one session (the agent had proposed T4b alone), a
  checkpoint after; the quirk classes asked after the reading.
- **Code reading** (`installation.md` §7.14, for T4b and T4c): Forgetfulness stores the mastery;
  the base damage counts `max(1, n / 2)` creatures of a forgetful shooter on every strike (S-09
  confirmed); from Advanced no shooting; flag `0x20` makes it mass from Advanced (S-10
  confirmed). Blind stores its factor; the battle loop starts a blind stack's turn and passes it;
  any damage ends it (0x444090); the retaliation of a stack blind at the attack takes the factor
  in damage stage 2, none at 0. Absolute immunities: Forgetfulness only shooters, Blind never
  undead or Troglodytes; Air and Storm Elementals are immune to Blind.
- **User (Q53):** S-09 the rule in every profile (no flag); S-10 fix-optional (Q-0011,
  `quirk.battle.early_mass_version`); S-11 the engine's pathfinding, unconfirmed; S-13
  fix-optional (T4c).
- **Content (pack format v11):** timed changes `forgetfulness` and `blind`; `ends = "damaged"`;
  `fixed_mass` (the mass version's mastery without Q-0011); the gap-fill's timed entries take
  `requires`, `excludes`, `ends`, `fixed_mass`; Blind immunities for Troglodytes and Air and
  Storm Elementals (source `exe-forgetfulness-blind`). Ruleset hash v21. The synthetic pack gains
  Haze (a Forgetfulness) and Dazzle (a Blind).
- **Battle:** BAT-MAG-023 (`combat::striking_count`, `can_shoot`, `magic::target_at` with the
  quirk), BAT-MAG-024 (`effects::damaged` after every strike, reflection and spell damage,
  `effects::blind_share` before a melee attack, `Strike::retaliation_percent` in stage 2 and the
  exact formula, no retaliation at 0); BAT-TRN-001 and BAT-DMG-002 amended. `SpellValues`
  gains `status` (`effects::Status`: forgetful, no shooting, blind, ended by damage) in the
  record's spare byte; `StackFlags::STARTED` (the original's +0x4F0) passes a blind stack and
  guards the turn-start rules (regeneration, own-turn effects) instead of `WAITED`; event
  `TurnPassed`. State hash v25; `BattleState` stays 3 904 bytes.
- **Workload:** Brea casts Haze, Corwin Dazzle; golden replays re-recorded. The old workload
  (`7812d9d`'s synthetic pack) on the new engine: 472.54 M Ir against T4a's 472.93 M (−0.08 %,
  10 070 steps); the new one 10 682 steps, 494.80 M Ir; `bench-check` re-baselined.
- **Tests:** `tests/actions.rs` (the halved count by shot, melee and retaliation, one creature
  in full; no shooting from Advanced, a Basic cast keeping its mastery; the targets and the mass
  version with and without Q-0011; a blind stack passing its turns, acting after a shot ends
  its blindness in the same round, Blind running out; the retaliation at 50 %, 25 % and none,
  exact and staged; spell damage ending it, the undead spared).

### T4c — Control: Berserk and Hypnotize (2026-10-07)

- **Seam (Q53):** with T4b; read in the same pass (`installation.md` §7.14); S-11 the engine's
  pathfinding (unconfirmed), S-13 fix-optional (Q-0012).
- **Content (pack format v12):** timed changes `berserk` and `hypnotize`; `duration = "fixed"`
  with `fixed_rounds` (Berserk's 255, S-17 as data in every profile: an agent decision to
  confirm); `ends = "own_attack"`; `health_limit`; `refuses` (timed changes whose active effects
  keep a stack from being a target: Hypnotize's `magic_mirror`); the gap-fill's timed entries
  take an area and a side (Berserk: an area timed spell, flag `0x80`, either side) and these
  fields (source `exe-berserk-hypnotize`). Ruleset hash v22. The synthetic pack gains Rampage (a
  Berserk) and Charm (a Hypnotize).
- **Battle:** BAT-MAG-025 (`control::berserk`: the nearest of every other living stack, by hex
  distance for a shooter, else by the steps to its nearest attack position over the whole field;
  a draw among the nearest; shoot, walk and attack, walk towards it, or pass;
  `turn::berserk_turn` with good morale, `turn::berserk_now` after a cast that makes the active
  stack berserk; `effects::attacked` ends it after its own melee attack or shot). BAT-MAG-026
  (`Stack::controller`, `Stack::is_hostile_to`: legality, validation, `side_to_act`, blocking,
  secondary targets, the war machines' targets and patients use them; the controller's hero in
  BAT-HER-004, BAT-MAG-008/018/020; `magic::admits` for the health limit and the refused
  changes, also for a Magic Mirror's new target; `magic::skips_hypnotized` for mass versions,
  Expert Cure under quirk Q-0012). `Status` gains berserk, hypnotized, ended by its attack. No
  state-hash change (the status byte holds them; v25 since T4b).
- **Performance:** the old workload (`5196b7d`'s synthetic pack) first cost +2.84 % on the new
  engine: `magic::reachable` was no longer inlined into its 300 k calls per workload and the aura
  recomputed the stack's controller per neighbour. `#[inline(always)]` on `reachable`, the
  controller hoisted, and a fast path in `is_hostile_to` (neither stack berserk nor hypnotized:
  the sides) bring it to 498.14 M Ir against 494.80 M (+0.68 %). The new workload: 11 368
  steps, 535.05 M Ir; `bench-check` re-baselined; golden replays re-recorded. `h3 bench` full
  about 240 k steps/s.
- **Agent decisions (change freely):** the draw among the nearest only when there are several;
  where a walker beyond its speed stops (the lowest hex at its speed on a shortest path) and a
  flyer (the position within its speed nearest to the goal); a hypnotized stack's morale, luck,
  turn-order tie-break, battle end and outcome by its owner (the original's statistics and
  tie-break; the battle end not read); the observation's "viewer to act" is the controller's.
- **Tests:** `tests/control.rs` (Berserk's area on both sides by mastery, 255 rounds, the
  counter pair; the automatic attack on the nearest stack, own stacks attacking it, its
  retaliation not ending it; the shooter, the draw among equals; walking towards a far target,
  a flyer, passing; a cast on the active stack; good morale; Hypnotize's limit, the wounded
  stack, the mirror; a hypnotized stack's turn and the controller's damage bonus; the battle
  going on; mass versions and Expert Cure with and without Q-0012, a single Cure; the
  controller's resistance).

### T5a — Visible spell obstacles and Teleport (2026-10-07)

- **Seam (Q54):** T5 in three parts, T5a + T5b this session, a checkpoint before T5c; S-17 and
  the T4c agent decisions confirmed.
- **Reading** (`installation.md` §7.15): the battle's object list and hex flags, Force Field's
  and Fire Wall's handlers and descriptors, their target test, the triggers when a stack enters
  a hex (0x46A570: Fire Wall 0x59FCD0, Land Mine 0x59FB40, Quicksand 0x469E50) and where they
  are called (each step of a walk, a flyer's landing, after an action without a move), the round
  countdown, Teleport's handler and destination test, Remove Obstacle's mastery table (T5b),
  Expert Dispel's clearing.
- **Content (pack format v13):** effect kinds `obstacle` (`wall`, `fire`) with the spell's
  `obstacle` shape (steps from the target hex as the attacker casts it, hexes by mastery,
  mirrored, separate, rounds) and `teleport`; `clears_obstacles` (Dispel: Expert). Gap-fill
  `[[spells.obstacle]]` and Teleport as `[[spells.other]]` (source `exe-obstacle-spells`); the
  harvester writes the war-machine exclusion of the other kinds' flag word (Teleport). Ruleset
  hash v23. The synthetic pack gains Bulwark (a Force Field), Scorch (a Fire Wall) and Blink (a
  Teleport); Purge clears obstacles at Expert; Brea casts Scorch and Blink, Corwin Bulwark.
- **Battle:** `h3_battle::obstacles` (the list in placement order, reserved like the effects, with
  the blocked and burning hexes as sets; `place`, `count_down` after the effects at the round
  start, `clear`, `travel` (a move: a walker's path entered hex by hex, a flyer's landing; fast
  path without Fire Walls), `stand` (after defend, wait, a shot, a melee attack from where it
  stands)); `movement::path` (BAT-MOV-003); `magic::burn` (BAT-MAG-028: `f > 0`, no roll, the
  owner's hero's bonuses); `Command::Cast.second`, `second_targets`, `CastTarget::hex`; events
  `ObstaclePlaced`, `ObstacleRemoved`, `ObstacleTriggered`, `Teleported`, `SpellCast.second`;
  the observation's `spell_obstacles` (the flat sim encoding unchanged, BACKLOG). State hash v26;
  `BattleState` 3 992 bytes.
- **Sim and formats:** action encoding v10 (the second head: Teleport's destination), C ABI v10,
  replay format v17 (`cast <spell> stack <s> hex <h>`); random rollouts draw a second target only
  when there are several.
- **Performance:** the old workload (`2c4ca11`'s synthetic pack) on the new engine: 538.14 M Ir
  against 535.05 M (+0.58 %; at first +0.98 %: `second_targets` listed every hex for spells
  without a second target, and `travel` and `stand` now split a small inline test from their
  out-of-line Fire Wall paths); the rest is `legal_commands` (+1.5 M, a larger `Command`) and
  inlining. The new workload: 10 776 steps, 569.60 M Ir; `bench-check` re-baselined; golden
  replays re-recorded and a fourth one added (`full-s3-a362`: Bulwark, Scorch twice, Blink).
- **Agent decisions (change freely):** a walker's path among equally short ones (BAT-MOV-003;
  the original's pathfinder not read, BACKLOG); no burn once the battle is over after a melee
  attack from where the stack stands (the original checks the end only after a shot); Teleport
  takes the head hex of a two-hex stack's position, not the original's click adjustment; the flat
  observation has no obstacle features; the spell obstacle list reserves its room with the first
  obstacle and a fork copies it only when it has any, so a fork without obstacles still costs one
  allocation (ADR-0014 point 5's aim; `clone BattleState` about 387 ns here).
- **Q55 (open):** the anchor hexes of natural obstacles (`installation.md` §7.15) and T5b's seam
  change wait for an experiment (the user, 2026-10-07).
- **Tests:** `tests/obstacles.rs` (Force Field's shape by mastery and side, blocking, two rounds,
  events, observation; free hexes for a cast; Fire Wall's mirrored shape, one obstacle per hex,
  values; burns of a walker per hex entered, a two-hex walker once per hex, a flyer only where it
  lands, the moves split at a burn; death on the way, no attack, the battle's end; burns after
  defend, wait, a shot, an attack in place, not after a walk and attack; immunity, reduction, no
  resistance roll, the caster's bonus; a two-hex stack on two burning hexes; berserk walkers;
  Teleport's targets, destinations, no burn, refused second targets; Expert Dispel; the path);
  the property tests' books gain the four spells and check every second target; sim, CLI, pack
  and content tests for the second head, the text form, format v13 and validation.

### Batch 2b folded in (2026-10-07)

- **Seam (Q56):** the batch 2b fold-in and batch 2c, a checkpoint after; T5c next.
- **Results** (`experiments/results/E04b-batch-2b.md`, SoD 4.0 Complete, plain HD): every
  encounter as the engine's faithful prediction (T3c3). R1 195 and 390: the x87 at 53 bits
  (Q50; 64 bits gives 194, 389). R2: both Wolf Rider attacks struck back in rounds 2 and 3,
  Counterstrike no longer listed in round 3, only the first in round 4 (Q-0008). R3: 150 dealt,
  "The Fire Shield does 60 damage.", the top Naga Queen at 50 (Q-0010; of the damage dealt: 30).
  R4: 121, 40 (Shield and Basic Armorer in one product) and 189 (Armorer alone, Q-0005). R5:
  Bloodlust and Weakness both listed, attack 4; Slow ended Haste, speed 3 (Q47). R6: Mephala
  at level 10 (attack 2, defense 5), 619 and 154 (Q45 (b)). The retaliations' kills (5, 5, 6, 5,
  5) fit the damage range.
- **Play notes:** in R1, R4 and R6 the Angels (speed 12) could not reach the Peasants in round
  1; the user moved or waited first, which changes none of the values (the struck stacks never
  defended). Batch 2c's spec checks reach against speed.
- **Recorded:** observation sources on BAT-DMG-002, BAT-HER-004, BAT-MAG-009 to 012, 015, 018
  and HER-BAT-004 (now verified); the quirks ledger's Q-0005, Q-0008 and Q-0010; fixtures
  `the_staged_damage_reproduces_batch_2b` (R1, R3, R4, R6 through `damage::Strike`) and
  Mephala's and Octavia's Armorer from the harvested packs (`harvested_magic_derives`); the
  mechanism tests cite R2, R3 and R5. No behaviour or version changes.

### Batch 2c compiled (2026-10-07)

- **Seam (Q56):** Q55's anchors with T5a's and T4c's candidates; E04a's (1)–(4) and T4a's wait.
- **Spec** `experiments/specs/E04b-batch-2c.toml`, template `experiments/results/E04b-batch-2c.md`,
  map `experiments/out/E04b-batch-2c.h3m` (8 encounters, 8 Red and 8 Blue heroes; dirt, where 6
  of the 16 usual obstacle types leave their anchor free: obstacle_5, 6, 8, 13, 14, 15). A1–A3
  (A3 only if needed): a Phoenix's hero tries Remove Obstacle's cursor on the free hexes beside
  each obstacle, Fire Wall's on the hex found (X) and on its down-left neighbour, lands on X and
  casts Remove Obstacle on X in round 3 (ours: one free hex per such obstacle takes it, Fire Wall
  refused, the landing allowed, the whole obstacle removed; VCMI: no free hex takes it). F1: Fire
  Wall (20 per burn) on 50 Pikemen: entering, Wait, Defend (48, 46, 44); Blue's Force Field
  (ours up-right, VCMI up-left) and Fire Wall (up-left). F2: both sides' Pikemen walk two rows
  straight up past a Fire Wall on their up-right (Red) or up-left (Blue) neighbour (ours: Red
  burnt, Blue not; checked in a scratch test, not committed). B1: Berserk on the middle of three
  Blue stacks, two equally near (ours draws; up to three tries). H1: Hypnotize (limit 60) on 2
  Swordsmen before and after a 30-point Magic Arrow (ours refused both times, VCMI allowed after),
  then Expert Cure (ours: Hypnotize stays, Q-0012). C1: Basic Cure on a hypnotized Swordsman
  (ends it: the control for H1).
- **Design notes for the player:** the battlefield's obstacles come from the map tile, so every
  step has a fallback when an obstacle is in the way; F1's and F2's armies mix two alignments
  (morale 0: no extra turn adds a burn); every move is within the stack's speed (the batch 2b
  note); the hex grid and the movement shadow on; a spell's cursor is tried and cancelled.

### T5c1 — Quicksand and hidden obstacles (2026-10-07)

- **Seam (BAT-Q1):** T5c in two parts, T5c1 alone this session, a checkpoint before T5c2 (Land
  Mine). After the reading the user made the natives' sight (BAT-QUIRK-1) and an attack from a
  seen pit (BAT-QUIRK-2) fix-optional quirks; ADR-0014 point 7 amended in `shared/adr-0014-natives`.
- **Reading** (`installation.md` §7.16): the natives' byte per stack and per side (set once at the
  battle's start), Quicksand's handler (pits by mastery from a table, a sampler without
  repetition, the free-hex test with corpses), the walker's stops and reveal, the stop array of
  the pathfinder (flyers and Devils none), the move and attack skipping its attack. Correction:
  0x469E50, called when a hex is entered, is the moat's damage, not a Quicksand test (§7.15).
- **Content (pack format v14):** obstacle kind `pit` (hidden: `ObstacleKind::is_hidden`), a
  random shape (`random`, target none; `rounds` 0 = the whole battle, now the default); quirk
  flags `quirk.battle.natives_sight_at_start` and `quirk.battle.attack_from_seen_pit` in both
  profiles; ruleset hash v24. Gap-fill Quicksand (source `exe-quicksand`; the harvester writes no
  target for a random shape). The synthetic pack gains Mire (a Quicksand); Corwin casts it.
- **Battle:** `SpellObstacle::revealed`; `SpellObstacles` keeps the pit hexes and the sides'
  sight (natives at the start, live sight when BAT-QUIRK-1 is off); `StackTraits::NATIVE`;
  `obstacles::{sees, seen_by, seen_pits, random_hexes}`; `Reach::stops_at` and walks that never
  pass a seen pit (`movement::reachable_within`, `path`, now searched within the stack's speed);
  `travel` returns whether the stack arrived and stops a walker at the first pit it enters
  (`stuck`: reveal, `ObstacleRevealed`, `ObstacleTriggered`); obstacle events carry the
  visibility of BAT-MAG-031; the observation lists what the viewer sees; Fire Wall's and Force
  Field's target test ignores what the caster's side does not see (Q-0002); legality and
  validation of attacks from seen pits by BAT-QUIRK-2 (`CommandError::Unreachable` when fixed);
  the berserk turn the same. Rules BAT-MAG-031/032; BAT-MAG-027, 030 and BAT-MOV-003 updated.
  State hash v27; `BattleState` 4 032 bytes (+40). Replay format, action and observation
  encodings, C ABI unchanged.
- **Performance:** the old workload (no Mire) on the new engine: 568.44 M Ir against 569.60 M
  (−0.20 %; at first +1.3 %: `movement::fits` fell out of line and the target test built its
  hex set through the visibility filter, now inlined and only with pits). The new workload:
  11 424 steps against 10 776 (pits stop walkers), 610.18 M Ir (+7.13 %, +1.05 % per step: walks
  through pits compute their path; the path search now stops at the stack's speed instead of
  the whole field); `bench-check` re-baselined; golden replays re-recorded and a fifth one added
  (`full-s3-a5`: Mire's pits stop Brea's walkers, four revealed).
- **Agent decisions (change freely):** a hypnotized walker's stops and reveals use its
  controller's view (the original's its owner's, Q-0002); a Fire Wall or Force Field may go over
  a hidden pit its caster does not see (both objects stay; the original refuses the hex); the
  pit's trigger event follows `Moved` (the original reveals before moving); a hex with a corpse
  takes no pit (the original's +0x1C test; the engine's dead stacks keep their hexes); the
  natives' sight ignores special battlefield 2, where the original has no natives (E04c).
- **Tests:** `tests/pits.rs` (counts by mastery, free hexes, no free hex, the whole battle;
  observations and projected events; an unseen pit stops and reveals, a stack leaves a pit, paths
  around a seen pit, its own pit not revealed, flyers ignore pits; a two-hex walker's tail; an
  attack from a seen pit with and without BAT-QUIRK-2; a move and attack into an unseen pit; the
  natives' sight with and without BAT-QUIRK-1, Expert Dispel's events; Fire Wall over a hidden
  pit; a hypnotized walker); the property tests' books gain Quicksand, the battlefield a terrain
  (natives), and a new property checks that what a side does not see changes neither its
  observation nor its legal commands, castable spells and targets; pack, content and harvester
  tests for format v14.

### Batch 2c folded in (2026-10-07)

- **Seam:** the user pushed the results to `battle/e04b-t5c1` during T5c1 and let the agent fold
  them in on the same branch ("at your discretion").
- **Results** (`experiments/results/E04b-batch-2c.md`, SoD 4.0 Complete, plain HD): A2 confirms
  the anchor reading of Q55 (the free hex left of a 3×1 rock takes Remove Obstacle, refuses Fire
  Wall as either hex, takes the Phoenix, and Remove Obstacle there removes the whole rock; the
  5×1 obstacles, absolute, refuse it); on A1's battlefield the spell was refused at once ("That
  spell will not affect anything."), as for absolute obstacles only; A3 not needed. F1: 48, 46,
  44 (entering, Wait, Defend; 20 per burn), the defender's Force Field up-right (not mirrored),
  its Fire Wall up-left, Red's wall gone in round 3. F2: Red burnt (8), Blue not (10): the
  original's walker passes the up-right neighbour, as BAT-MOV-003. B1: lower, upper, lower (a
  random choice). H1: refused, allowed; Magic Arrow 30, 2 Swordsmen with the top at 5; refused
  at 40 of 70; Hypnotize listed after Expert Cure (Q-0012), the healed top at 35; Red in rounds 2
  and 3 (first noted Blue for round 3, a mix-up the user corrected: BAT-Q2, answered).
  C1: Basic Cure allowed, Hypnotize ended, Blue in round 2.
- **Recorded:** observation sources on BAT-MAG-025 to 028 and BAT-MOV-003; the ledger's Q-0012;
  Q55 updated (the reading confirmed; the seam options wait for the user); BAT-Q2 answered. No
  behaviour or version changes.

### T5c2 — Land Mine, and the Q55 survey (2026-10-07)

- **Seam (BAT-Q3):** after PR #4 was merged, T5c2 with the Q55 community survey, a checkpoint
  after (Q55's class and seam option, then T5b).
- **Reading** (`installation.md` §7.17): the spell book's refusal while the other side's natives
  byte is set (no cast, no cost; no immunity test in that byte, against the wiki), the handler
  (mines by mastery from the table 0x642224, Quicksand's sampler, one object per mine with the
  cast's value, flag `0x8`), the trigger after Fire Wall's (spares the owner's stacks by the real
  side, revealed mines and stacks with `f ≤ 0`; Fire Wall's damage with spell 11; the mine
  removed; the Phoenix rebirth check 0x469440 as after Fire Wall, T7), and the walker, which
  ignores the trigger's result.
- **Content (pack format v15):** obstacle kind `mine` (hidden, `ObstacleKind::has_value`); the
  gap-fill's Land Mine (source `exe-land-mine`); the synthetic pack's Snare (a Land Mine), which
  Brea casts. Ruleset hash scheme unchanged (v24).
- **Battle:** `SpellObstacles` keeps the mine hexes; `obstacles::may_place` (no mines while the
  other side's natives would see them: at the start under BAT-QUIRK-1, living natives without
  it) in the cast's target tests; entering a hex (`enter`, for walkers, flyers' landing and the
  stand after an action without a move) burns, then sets off a mine its side does not see and
  whose spell hurts it (`mine_under`, `magic::hurts`: Fire Wall's `f > 0`), which is revealed,
  triggered, does Fire Wall's damage (`magic::burn`) and is removed, all public; a mine that does
  not go off changes nothing, not even how the move is reported (a split `Moved` would give it
  away). Rule BAT-MAG-033; BAT-MAG-027 and 031 updated. State hash v27 unchanged (the mine set
  follows from the list); `BattleState` 4 064 bytes (+32). Replay format, action and observation
  encodings, C ABI unchanged.
- **Performance:** the old workload (no Snare) on the new engine: 609.73 M Ir against 610.18 M
  (−0.07 %). The new workload: 10 878 steps against 11 424 (mines kill), 584.17 M Ir (−4.26 %,
  +0.54 % per step); `bench-check` re-baselined; golden replays re-recorded; `full-s3-a246`
  replaces `full-s3-a5`, which no longer cast Mire (two Snares, eight mines go off, Mire's pits
  stop walkers 14 times).
- **Agent decisions (change freely):** the cast test follows BAT-QUIRK-1 (strict-fixed: living
  natives at the cast) rather than a quirk of its own; with live sight a mine the stack's side
  sees does not go off (the original's revealed mine); a mine's events are public and follow its
  `Moved`; a mine-immune stack (or the owner's) crossing a mine reports a single move.
- **Tests:** `tests/mines.rs` (mines by mastery with the value, hidden, the whole battle; the
  refusal with and without BAT-QUIRK-1 and the caster's own natives; an enemy walker sets one off
  and walks on, the battle ends on a mine; a move and attack still attacks; own and immune stacks
  cross without a trace; a flyer's landing; a stack standing on a mine; Fire Wall then mine on
  one hex; a two-hex walker once; a hypnotized stack by its owner's side; living natives' sight);
  the property tests' books gain Land Mine, the hidden-obstacle property places mines too and
  checks that the acting side's non-cast commands report the same with and without what it does
  not see (unless a hidden obstacle shows itself); pack and content tests for format v15. The C
  smoke test's `second == 0` check, stale since T5a's Teleport (the game merely never chose it),
  now accepts any second target from its mask.
- **Q55 survey:** nobody describes the anchors: the wiki's pages, HotA's changelog (1.7.0 fixed
  Remove Obstacle's highlighting of invisible Quicksand), VCMI's battle bug list (#1219), df2.ru's
  spell and reference threads; heroesworld.ru's old threads redirect, the web archive is blocked,
  Celestial Heavens refuses the fetch. Recorded in OPEN-QUESTIONS Q55 with the recommendation
  fix-optional and seam option (a).
- **Review** (an independent sub-agent on the diff against §7.17 and BAT-MAG-031/033): no code
  problem; two tests tightened (Fire Wall's burn before the mine on one hex, a comment); found
  that Quicksand's and Land Mine's random hexes avoid the other side's hidden obstacles, a small
  leak the original has (T5c1 kept it): OPEN-QUESTIONS BAT-Q4 for the user.

### T5b — Remove Obstacle, the obstacles as pieces, BAT-QUIRK-3 and BAT-QUIRK-4 (2026-10-07)

- **Seam (BAT-Q5, Q55, BAT-Q4):** after PR #5 was merged, the user chose to finish T5 in one
  battle pull request labelled `cross-lane` (the contract cannot compile without the battle
  crates that use it); the anchors fix-optional with seam option (a), the draws fix-optional. The
  user then left for some hours: the session went on unattended (the original files were staged
  before the computer went offline).
- **Seam change:** `BattleSetup.obstacles: Vec<h3_contracts::Obstacle>` with `Obstacle { hexes,
  anchor: Option<u8> }` (ADR-0014 point 10 and ADR-0004 amended: a change of a field's type, not
  an addition); `generate_obstacles` returns the pieces in the order placed, each usual one with
  its anchor; scenario and replay files carry `{ hexes, anchor }` (`h3_pack::ObstacleEntry`;
  replay format v18, `[[setup.obstacles]]` after the plain values).
- **Battle:** `h3_battle::natural` keeps the blocked hexes, the removable pieces by value (at most
  16 of at most 8 hexes: no allocation on a fork; `SetupError::InvalidAnchor`, `TooManyObstacles`)
  and the original's hex records as far as they matter: the anchors that belong to their pieces
  (BAT-QUIRK-3) and the hexes a removal cleared. A hex leads to the piece listed last among those
  touching it; removing a piece clears its blocked hexes and, with the quirk, its anchor hex (the
  original's 0x466B30, re-read for this: bits `0x3F` and the index on the descriptor's hexes,
  `0x1` and the index on the anchor). The anchors keep Fire Wall, Force Field and the random
  draws off (`obstacles::free`, `random_hexes`). Remove Obstacle (BAT-MAG-034, effect
  `remove_obstacle`, `removes` by obstacle kind, pack format v16): the natural piece on the hex,
  else the spell obstacles there that the caster sees and the mastery removes; the Expert
  cursor's leak fixed under Q-0002. BAT-QUIRK-4: Quicksand's and Land Mine's draws avoid every
  spell obstacle (faithful) or those the caster sees (strict-fixed, overlap allowed). Event
  `NaturalObstacleRemoved` (public); the observation gains the removable pieces and the anchors.
- **Content:** synthetic Uproot (Corwin's, in the `full` scenario); the harvester's gap-fill kind
  `[[spells.remove]]` and source `exe-remove-obstacle` (base: Remove Obstacle, target from flag
  `0x100`); profiles with `quirk.battle.obstacle_anchors` and `quirk.battle.draws_avoid_hidden`; a
  random obstacle shape must be `separate` (else a full field would place an empty obstacle).
- **Versions:** pack format v16, ruleset hash v25, state hash v28, replay format v18; action and
  observation encodings, C ABI unchanged (the flat observation has no obstacle features, E05/T9).
  `BattleState` 4 304 bytes (+240).
- **Performance:** the old workload (no Uproot) on the new code: 585.81 M Ir against 584.17 M
  (+0.28 %, same 10 878 steps). The new workload (Corwin knows Uproot): 10 867 steps, 580.60 M Ir
  (−0.61 %); `bench-check` re-baselined. A first version tested a remove spell's castability over
  every hex (+9.6 %); `obstacles::any_removable` tests the obstacles directly.
- **Replays:** all re-recorded; a sixth, `full-s3-a138`, where Corwin's Uproot takes away a hex of
  Brea's Scorch.
- **Agent decisions (change freely):** a hex shared by several pieces in a hand-written setup
  follows setup order as the placement order; with several spell obstacles on one hex (only the
  other side's unseen ones can overlap), Expert Remove Obstacle takes each it sees there; the
  cast's public target hex may show where the caster's own hidden obstacle was (as the original);
  Uproot only in the `full` scenario (no natural obstacles there; the duel has them but no
  heroes).
- **Tests:** `tests/remove.rs` (a whole stone removed, the cliff never; anchors with and without
  BAT-QUIRK-3 for Remove Obstacle, Fire Wall, Force Field, Quicksand's draw and a stack standing
  there; the hex placed last and the clearing; a piece blocking nothing; spell obstacles by
  mastery and sight, an overlapped hex; the draws with and without BAT-QUIRK-4; setup
  validation); the generator keeps the anchors; the property games get anchored pieces and Remove
  Obstacle, and the hidden-obstacle property applies casts too (it had silently left them out:
  the inner target list shadowed the legal commands); pack, content, harvester and sim tests.
- **Review** (an independent sub-agent on the diff): the property test's shadowed list; a region
  never executed (`any_removable` for a piece without hexes, also caught by the coverage gate);
  the clearing after a removal, which the model had re-derived from the remaining pieces (fixed
  after re-reading 0x466B30); generator anchors in a side column with more than one side column
  (`placement` now asks `is_standable`); a random shape that is not separate (rejected); the
  public cast target (documented); the observation's allocation (now by value); the per-kind
  table sized by `ObstacleKind::COUNT`. All fixed.

### T6a — Resurrection, Animate Dead and Sacrifice (2026-10-08)

- **Seam (the user, 2026-10-08):** T6 in two parts this session, T6a raising then T6b created
  stacks, one pull request each. The user questioned ADR-0014's new `StackOrigin` variants:
  nothing created in battle needs to cross the seam (only army stacks and war machines enter a
  battle; the orchestrator needs records only to update the army). ADR-0014 points 6 and 10
  amended in `shared/adr-0014-created-stacks` (PR #7): summoned stacks and clones are left out of
  the outcome; Pit Lords' demons are T7's question.
- **Code reading** (`installation.md` §7.18): the corpse lists of the hex records (at most 14,
  last added on top), the finders 0x5A3CC0/0x5A3F50 (a living stack on the hex, else the first
  qualifying corpse from the top of a free hex), 0x5A8090's cases for 38–40 (fewer creatures than
  at the start, the hit points reach one creature's health; the victim living, not undead, not
  summoned, own by controller and real side), the raise 0x5A7560 (capped at the start count;
  temporary below Advanced into the counter +0x54; the dead bit cleared, no other flag), the
  death 0x444120 (every spell ended; sacrificed and summoned stacks leave no corpse), Sacrifice
  0x5A199C (`(the table's health + effect[mastery] + power) × creatures`); found while reading
  for T6b, the end of the battle 0x4774E2: the counter subtracted (never lowered during the
  battle, unlike VCMI's), and one creature kept in the last stack when none is left, the total
  counting war machines and summoned stacks (the S-20 bug).
- **Quirks (the user):** BAT-QUIRK-5 fix-optional: Sacrifice tests its target through
  Resurrection's routine (its level 4, conditions and hit points at the hero's Earth mastery);
  S-15 is explained by it (the Dispel half was settled in T3b1b). BAT-QUIRK-6 (S-20) exploit-fix:
  first the orchestrator's (VCMI's fix); after the end-of-battle reading the user chose the
  original's rule counting only army stacks: the last living army stack keeps one creature.
- **Battle:** `h3_battle::raise`: `BattleState.raised` (per stack: temporary creatures, the
  corpse's order, `NO_CORPSE`; beside the stacks, so the stacks the hot paths walk stay small),
  marked after every command (`mark_corpses`); raise targets by `raise_test`/`raisable` (a living
  stack, or a dead one with free unblocked hexes that lies on top among the qualifying corpses on
  one of its hexes), the victim (`victim`, a stack as the second target); `SpellRaised`,
  `Sacrificed` events; the outcome's end counts without the temporary creatures. A cast that
  kills the active stack now ends its turn (also the hero's own area and battle-wide damage,
  which left a dead stack active before: BAT-MAG-001).
- **Content:** effect kinds `raise` and `sacrifice`, `temporary_below`, `tests_as` (pack format
  v17); gap-fill `[[spells.other]]` with `requires`, `excludes`, `temporary_below`, `tests_as`;
  the base pack's Resurrection, Animate Dead and Sacrifice; synthetic Revive (Brea without Stone
  magic: temporary; Corwin at Expert) and Offer (Brea); profiles with
  `quirk.battle.sacrifice_tests_as_resurrection`.
- **Versions:** pack format v17, ruleset hash v26, state hash v29; replay format, action and
  observation encodings, C ABI unchanged. `BattleState` 4 560 bytes (+256).
- **Performance:** the old workload on the new code: 588.06 M Ir against 580.60 M (+1.28 %, the
  same 10 867 steps): about half a codegen swing in the geometry hot path (`reachable_within`
  +6.1 M, the footprint iterator −3.5 M; the same functions), the rest the raise checks in
  `castable_spells` (+2.0 M) and the corpse marking (after every command and activation). A
  first version kept the two new fields in `Stack` (+1.61 % before the marking) and tested the
  raise kind per stack. The new workload (Revive and Offer in the books): 10 913 steps, 600.64 M
  Ir (+3.45 %); `bench-check` re-baselined.
- **Replays:** all re-recorded; `full-s3-a199` (Blink, Scorch, Snare's mines going off) replaces
  `full-s3-a362`, `full-s3-a325` (Uproot takes away Scorch, mines, Mire's stops) replaces
  `full-s3-a246` and `full-s3-a138`; new `full-s3-a52` (Offer; Corwin raises a dead stack) and
  `full-s3-a149` (Brea's temporary raise, an attacker victory).
- **Agent decisions (change freely):** the target may not be its own victim (the original's
  code does not compare them; the second cursor was not read); the 14-corpse limit of a hex is
  not modelled (a stack dying on a hex with 14 corpses leaves none there); the corpses of one
  command are ordered by stack id (they cannot share a hex); a raised stack keeps its round flags,
  shots and retaliations (the original clears only its dead bit); a sacrifice counts the victim's
  temporary creatures too (the original's count +0x4C), so a Basic raise sacrificed becomes
  permanent hit points.
- **Tests:** `tests/raise.rs` (temporary and permanent raises and the outcome; the health test;
  a dead stack back without effects and with its flags; corpses: the top one, a living stack on
  the hex, a two-hex corpse; Animate Dead; Sacrifice: the victims, no corpse, the active stack;
  BAT-QUIRK-5 with the hit points and the Gold Dragons' immunity; a battle-wide cast killing the
  active stack); the property games cast the three spells, conservation and the outcome allow
  for raises; pack, content, harvester and sim tests.
- **Review** (an independent sub-agent on the diff): an uncovered arm (`CastTarget::stack`); the
  corpse order when automatic turns follow within one command (now marked before every
  activation and after every automatic berserk action); the raise path skipped the battle's end
  test (now shared with the other casts, `reach_stacks`); stale docs of the second target (the
  sim's head, the command, the event); missing rule tests (Anti-Magic, corpses that do not count,
  a wall over a corpse, the specialist, BAT-QUIRK-5 on a dead target, a raised stack's turn); the
  sim's head-by-head test assumed only Teleport had second targets. All fixed; the counter's
  lowering (VCMI lowers it) was settled by the end-of-battle reading.

### T6b — Clone and Summon Elemental (2026-10-08)

- **Code reading** (`installation.md` §7.19): the stack creation 0x47A100 (the first of records
  0–19 empty or holding a dead summoned stack; set up as at the battle's start, so it acts in the
  round it appears); Clone's target test (case 0x5A841A: no clone, none made, level + 1 ≤
  effect[mastery]) and place search 0x5A6C70 (rings 1–10, rays per direction from the position
  and a two-hex stack's other hex, three skips; 0x446C40 for the fit), its rounds (power + the
  duration bonus, counted down at each round's start, 0x447120), death by any damage (0x444090)
  and with its original (0x444120); Summon Elemental 0x5A7080 (effect[mastery] × power, the
  first column from the side's edge with a fitting row, a sampler's row); the winner test
  0x4658B0 (a living stack neither summoned nor a war machine: a side left with only created
  stacks does not win, also against a retreat); the experience 0x46A350 leaves created stacks
  out, Necromancy 0x4775E1 counts them. Found on the way: 0x43D730 clears the town of creatures
  112–115 unless a game setting is on (their native terrain; left for BAT-TER-001).
- **Quirk:** BAT-QUIRK-7 (exploit-fix, agent; to confirm at the checkpoint): 0x446C40 counts
  the original's hexes as free, so around a crowded two-hex original the clone may get the
  original's head (the creation fails and the cast flags the original as summoned with the
  clone's rounds: it vanishes and is lost from the army) or share a hex with it. The engine takes
  the original's hexes. S-21 cannot occur in SoD (one clone per stack, a dying original kills
  it).
- **Battle:** `h3_battle::created`: `BattleState.created` (per stack the clone link and the
  clone's rounds; per side the summoned creature and, computed at setup, a stack of each creature
  its hero's book summons, as `Stack::new` makes it); traits `SUMMONED` and `CLONE`;
  `CREATED_SLOT`; `clonable`, `may_summon` (the side's stacks, dead ones included, below
  `side_stack_limit`), `clone_place`, `summon`, `add` (a dead created stack's slot, else a new
  id); `died` (after damage, a death stare or a sacrifice: a dead clone frees its original, a
  dead original takes its clone); the round's countdown; `StackCreated`, `Vanished`; corpses,
  sacrifice victims, the winner test (`check_end`: `Draw` when the side left has only created
  stacks) and the outcome leave created stacks out; `apply_damage` kills a whole clone.
- **Content:** effect kinds `clone` and `summon`, `Spell::summons`, at most four summon spells,
  `BattleParams::side_stack_limit` (≤ `MAX_SIDE_STACKS` = 21) (pack format v18); gap-fill
  `[[spells.summon]]` (spell, original creature id) and the battle's `side_stack_limit`; the base
  pack's Clone and Fire, Earth, Water and Air Elemental; synthetic Copy (Brea, Basic: level 1) and
  Conjure (Corwin, Expert: 12 Shades), a new creature Shade.
- **Seam:** `BattleResult::Draw`'s doc and ADR-0014 point 6 say that a side left with only
  created stacks does not win (a second commit in PR #7). BAT-Q7 asks about Necromancy counting
  killed created stacks and a retreat or surrender against only created stacks (the contract's
  `Retreat`/`Surrender` imply a winner; the original names none).
- **Versions:** pack format v18, ruleset hash v27, state hash v30, observation encoding v7 (per
  stack: created, clone, rounds left), C ABI v11; replay format and action encoding unchanged.
  `BattleState` 5 288 bytes (+728: the summon templates, 576).
- **Performance:** the old workload on the new code first measured +2.20 % (613.86 M Ir, the same
  10 913 steps): `reachable_within` +9.9 M (a codegen swing again, through the footprint's hex
  iterator in `movement::fits`) and the summon test in `castable_spells` (+2.4 M, `no_target`).
  `movement::fits` now tests a footprint's head and tail directly: 532.53 M Ir (−11.3 %). The new
  workload (Copy and Conjure in the books): 11 750 steps, 548.40 M Ir; `bench-check`
  re-baselined.
- **Replays:** all re-recorded; new agent seeds as the books changed: `full-s3-a1396` (Blink,
  Scorch, Snare's mines and Mire's pits), `full-s3-a300` (Conjure; Uproot takes away Scorch;
  mines and pits), `full-s3-a1476` (Offer; Revive at Expert), `full-s3-a1436` (a temporary raise,
  an attacker victory with 14 such creatures), `full-s3-a728` (Conjure; Copy of 30, the clone
  vanishes; a defender victory with Shades alive) replace a199, a325, a52, a149.
- **Agent decisions (change freely):** a summon count of 0 creates nothing (the original would
  create an empty stack; a hero's power is at least 1); without a free hex nothing happens and
  the mana is spent (the original shows a message); a created stack's morale and luck are
  materialised as its side's stacks' at its creation; the clone copies the original's statistics
  (equal to a fresh stack of its creature and side: the original sets it up afresh) and gets its
  creature's full shots.
- **Tests:** `tests/created.rs` (Clone's targets by level, immunity and side, the copy, its
  rounds, the outcome; the place along the rays for both sides and a two-hex stack; a crowded
  two-hex stack (BAT-QUIRK-7) and no place; the clone's end by rounds, damage, its original's
  death; Summon Elemental's count, column, hero bonuses, binding, a second stack, the defender's
  column 15 and 14; the limit with dead stacks and the reused slot; a side with only created
  stacks does not win; created stacks are no victims, a sacrificed original takes its clone); the
  property games cast Copy and the two summons (limit 12 in `lively`), with new invariants (no
  hex shared by living stacks, clone rounds, the side limit, no corpse) and the outcome without
  created stacks; content, pack, harvester, sim and C ABI tests.
- **Review** (an independent sub-agent on the diff): two T6 items left from T3b2 and T4a (no
  Anti-Magic on a clone; a clone's Fire Shield reflects a share of one hit point, in either
  profile); target lists fixed before damage struck a clone that had vanished with its original
  (secondary strikes, area and battle-wide spells), and a draining or shielded clone that killed
  its own original could act on after vanishing (now skipped); a clone spell aimed at the enemy
  could overfill that side (content now wants side own); uncovered lines (a summon's impossible
  cases, a duplicate summon template); stale docs (stack ids, `base_count`, the turn order's tie,
  `damaged`, the target docs, §7.7, S-21). All fixed; the surrender price with created stacks
  went to BAT-Q7.

### Batch 2d compiled (2026-10-08)

- **Seam:** the user cut increment 5 as batch 2d alone, with T6's candidates only (the older
  waiting ones, E04a's (1)–(4) and T4a's to T5b's, stay in `docs/experiments/README.md`); T7 is cut
  at the next checkpoint.
- **Spec** `experiments/specs/E04b-batch-2d.toml`, template `experiments/results/E04b-batch-2d.md`,
  map `experiments/out/E04b-batch-2d.h3m` (8 encounters, 8 Red and 8 Blue heroes; grass, so the
  elementals' native terrain shows). CL1: Clone on both sides in the open (the code: up-right for
  the attacker, up-left for the defender; VCMI: right, left), both clones act in round 1, the
  Archer clone's 12 shots after the original fired, Magic Arrow (30) kills the whole clone, the
  defender's clone gone at the start of round 3; Red's surrender price 800 → 1 300 with the clone
  (the engine; VCMI 800). CL2 (BAT-QUIRK-7): Griffins and Wolf Riders in row 0 at their edges,
  blocked below and at the second hex beside the head by a Cavalier and Marksmen (Red) or Goblins
  and Hobgoblins (Blue), the hex beside the head free: the code's ring 2 takes the head as the
  clone's position, so no clone appears and the originals vanish at the start of round 3 (the
  engine's fixed search, and VCMI: a clone in row 2). RA1: Resurrection (290) and Sacrifice refused
  on a wounded full stack, dead Titans (300) and dead Gold Dragons (immune to level 4), allowed on
  Pikemen; Sacrifice's victim the target itself (the engine refuses; not read). RA4: the later of
  two corpses on a hex is raised; a two-hex corpse whose other hex a living stack holds is refused.
  RA2/RA3: 10 Swordsmen → 5 → 8 (3 temporary) → 2: the army keeps 1 (the code, the engine) or none
  with a Ballista (the code, S-20; the engine keeps 1; VCMI 2 in both). NE: Necromancy from 10
  Pikemen, 6 Earth Elementals and a clone of 10 Pikemen at 30 %: the code 7, VCMI 3, the engine's
  outcome 3; experience 600 (created stacks left out); Air Elementals in an army 9/9/7 (the code)
  or 10/10/8 (the engine) on grass. SU: Air Elemental with 7 one-Peasant stacks, an Ammo Cart and a
  First Aid Tent, Blue's Death Ripple and Implosion: E1 in row 3 or 7 of column 1, later summons at
  random rows of column 1 then 2 (VCMI: always the hex nearest the top left), E3 in E1's record acts
  before E2, 12 casts until the 20-stack limit, still refused after another dies; ends a (both
  heroes lose), b (Blue retreats: the code names no winner) and c (Blue surrenders) by reloading.
- **Predictions** were computed in a scratch test (not committed) on the harvested SoD packs
  without natural obstacles: `BattleState::start` with derived heroes, commands as in the
  procedures; for CL2 a scratch copy of `created::clone_place` with the original's hexes free gave
  the code's place. Found on the way: an army of four alignments lost a blocker's turn to bad
  morale, so CL2's Blue army is all Stronghold.
- **Not staged:** a clone killed by 0 damage (VCMI 1.7.5 spares a clone from a 0-damage hit, the
  original kills it; no hero spell deals 0 in SoD); the elementals' native terrain with the game
  setting that keeps it (0x43D730, not read).

### T7 cut and T7a1 — the strike's spells (2026-10-08)

- **Cut** (the user, increment 6): T7a in two parts (T7a1 the strike's spells, T7a2 the new
  statuses), batch 2d folded in at the checkpoint after T7a2; T7b (casting creatures) and T7c
  (defences, auras, mana) refined then. The code reading confirmed the split.
- **Code reading** (`installation.md` §7.20): the melee attack 0x441CB0 (retaliation unless
  petrified or the strike answers "no retaliation"; the second strike unless the attacker is blind,
  petrified or paralysed), the strike 0x441610 (damage, then 0x440500 puts the striker's spell in
  the target's pending spell, 0x468990 casts the pending spells as a creature at mastery 0 with
  power 3, then 0x440BC0's striker effects, then Fire Shield), the table of 0x440500 (chances,
  living targets, Bind's lists, Acid Breath without a test), the test 0x5A8640 (0x5A8090 without
  its side test, a second draw), 0x440BC0 (Thunderbird, flies, Rust Dragon), the cast routine's
  handlers for spells 70–80 and their cases in 0x4448F0, 0x444510 and the round start 0x447120
  (Poison's tenths kept after it ends; Age, Disease, Bind's release), 0x44A4D0's absolute cases
  (Stone Gaze spares Troglodytes, Poison the non-living and Gargoyles). Nothing in the shot routine.
- **Content:** abilities `spell_after_strike` (spell, chance, mastery, rounds, living, unless
  active; several per creature for different spells) and `spell_damage_after_strike` (spell,
  chance, per creature), the compiler checks their spells' effects; `dispels_positive` on a
  dispel effect (pack format v19); gap-fill: Unicorns and War Unicorns Blind 20 %, Black and
  Dread Knights and Mummies Curse 25 %, Thunderbirds Lightning Bolt 20 % × 10, Serpent Flies
  Dispel Helpful Spells, Dragon Flies that and Weakness at Advanced unless active (source
  `exe-after-strike`); the harvester accepts effect entries on ability spells; the synthetic
  Brute strikes with Glare (a Dispel Helpful), Dazzle, Bane and Spark.
- **Battle:** `h3_battle::after_strike` (BAT-MAG-040 to BAT-MAG-042): after the strikes of a melee
  attack and before life drain, in ability order; `StrikeSpell` event; `affects_stack`,
  `resists` and `spell_damage` shared with the hero's casts; a turn-stopping spell (Blind) stops
  the retaliation, a blinded attacker loses its second strike; `dispel_ends` honours
  `dispels_positive`.
- **Measurements:** the new binary on the old workload −0.11 % Ir; the new workload (Brutes'
  spells: +7.2 % steps) +8.52 %, re-baselined. Golden replays re-recorded (ruleset hash) with new
  agent seeds showing T5–T6's features and the strike spells: a385, a1059, a124, a300, a301.
- **Versions:** pack format v19, ruleset hash v28; state hash v30, observation encoding v7, C ABI
  v11, replay format and action encoding unchanged.

### T7a2 — the strike's statuses (2026-10-08)

- **Split** (proposed to the user with T7a1's PR): Stone Gaze, Paralyze, Disease, Age and Poison
  here; Bind and Acid Breath (new mechanics: binder lists, a permanent defense loss and the Rust
  Dragon's extra damage) in T7a3. The code reading is §7.20's (T7a1).
- **Content:** timed changes `petrify`, `paralyze`, `poison`, `age`; `restores_health` (Cure);
  the gap-fill's timed entries may name ability spells and give their effect values (Disease 2,
  Paralyze 25: Blind's Advanced share, which the original reads); Stone Gaze ends with damage,
  Poison needs a living creature; Troglodytes immune to Stone Gaze, Gargoyles to Poison; strike
  abilities for Zombies (Disease 20 %), Ghost Dragons (Age 20 %), Wyvern Monarchs (Poison 30 %),
  Medusas and Basilisks (Stone Gaze 20 %), Scorpicores (Paralyze 20 %), source
  `exe-strike-statuses` (pack format v20). The synthetic Brute strikes with five more spells and
  Mend restores health; the new synthetic spells come last (no handle moves).
- **Battle:** `Status::BLIND` became `NO_TURNS` (Blind, Stone Gaze, Paralyze) and `PETRIFIED` took
  the status byte's last bit; `retaliation_share` (Blind's or Paralyze's, the smaller);
  `stops_turns`; a petrified target halves stage 3 (`Strike::petrified`, exact: a seventh factor)
  and does not retaliate while petrified (the strike ends it first, as 0x444090 does);
  `h3_battle::vitality` (per stack the start hit points and Poison's tenths, beside the stacks:
  the hit points per creature materialised in `effects::recompute`, Poison's tenth at a new
  effect and at each round start where it lasts, `Poisoned` event, Cure's restore, a clone from
  the start value, a reused slot afresh); Q-0005 covers the x87 formula (9 hit points poisoned and
  aged: 4, exact 5).
- **Measurements:** the new binary on T7a1's workload +0.09 % Ir; the new workload has 20.7 %
  fewer steps (stacks lose turns, battles end sooner) and 11.8 % fewer instructions,
  re-baselined. Replays re-recorded with new agent seeds: a213, a804, a196, a61, a514.
- **Versions:** pack format v20, ruleset hash v29 (`restores_health`, the new timed changes),
  state hash v31 (the vitality table), observation encoding v7 (the effects list and the hit
  points already show it), C ABI v11, replay format v18.

### T7a3 — Bind, Acid Breath and the Rust Dragon's extra damage (2026-10-08)

- **Code reading** (§7.20, extended): the binder release 0x43EE10 runs on every move of a binder
  (the move handler 0x445990: walk, flight, a Devil's teleporting move), at its death (0x444120)
  and on Teleport (0x5A1B88), which also ends the teleported stack's own Bind (0x5A1B91); the
  pathfinder reads Bind's duration, not the lists; the Rust Dragon case 0x44149F (T alive, the
  draw, a roll × creatures / 2 if above 0). Not reproduced: a binder listed on a stack its strike
  killed (BAT-Q8, recommendation: a harmless bug, not reproduced).
- **Content:** timed change `bind`; `spell_after_strike` gains `defended`; ability
  `damage_after_strike`; the gap-fill's timed entries may ignore immunity; Bind (255 rounds) for
  the Dendroids, Acid Breath (effect 3, permanent, cumulative) and the extra damage (20 %, 50 %)
  for the Rust Dragon, source `exe-bind-acid` (pack format v21). The synthetic Brute strikes with
  Tangle (a Bind, 25 %), Etch (an Acid Breath of 1, 30 %, `defended`) and the extra damage.
- **Battle:** a spell after a strike that ignores immunity is cast without any test (also on war
  machines); `h3_battle::binds` keeps the bound stacks (materialised in `effects::recompute`, a
  dead stack never bound) and each stack's binders beside the stacks; a bound stack's movement
  range is 0 (`movement::range`; a berserk one picks only neighbours); `release` on
  `obstacles::travel`, Teleport and the death hook `created::died` (which now takes the ruleset;
  a vanishing clone releases too); `StrikeDamage` event (BAT-MAG-047).
- **Measurements:** the new binary on T7a2's workload +0.30 % Ir (the per-move and per-death
  checks); the new workload has 7.1 % fewer steps (bound stacks do not move, battles end
  sooner) and 6.45 % fewer instructions, re-baselined. Replays re-recorded with new agent seeds:
  a471, a53, a6, a126, a235.
- **Versions:** pack format v21, ruleset hash v30 (`defended`, `damage_after_strike`, `bind`),
  state hash v32 (the binds table), observation encoding v7 (the effects list shows Bind), C ABI
  v11, replay format v18.

### T7a checkpoint, the review and the T7b reading (2026-10-08)

- **Seam** (the user): close T7a (the QUEUE's inbox, an independent review with fixes, the global
  HANDOFF's battle row), then read the casting creatures and propose the T7b/T7c cut, and stop.
  Batch 2d's results are not committed yet (the results file is still the template), so
  BAT-QUIRK-7, BAT-Q7 and the clone's shots wait. Decided: BAT-Q8 not reproduced; Q-0005's
  extension to Poison and Age confirmed.
- **Inbox:** T7a2's docs commit (5751e65) had dropped the QUEUE's whole "Inbox" section; restored
  from b1df5ad, with T7a's closed items marked and its loose ends added.
- **Review** (an independent sub-agent on T7a's diff against §7.20): (1) a stack that died aged
  kept its halved hit points per creature, so a raise counted twice the creatures (fixed:
  `effects::remove_dead` recomputes the stacks whose effects it drops, as the original's death
  0x444120 ends every spell; Poison's tenths stay); (2) a bound Harpy flew back from the
  Dendroids it attacked (survey S-14, below); (3) the observation lacks the binder lists (inbox,
  T9); (4) the order of a creature's strike abilities is the pack's (documented on the ability;
  the harvested packs follow the original); (5) test gaps (a reused slot's vitality, a vanishing
  binding clone, Magic Mirror on a strike's spell, Disease below 2, a bound shooter) filled;
  (6) stale docs, and §7.20's Poison formula (`f32(H) × factor`, the product not stored, 0x444F65).
- **S-14 read** (`installation.md` §7.20): the action handler returns Harpies (0x4789F8) unless
  dead, blind, petrified or paralysed, not testing Bind; the flight's 0x43EE10 also empties the
  moving stack's own binder lists without ending its Bind, so the original's bound Harpy stays
  bound whatever its binders do. Community: known only as a bug (wiki; fixed in HotA 1.3.3; VCMI
  does not return a bound stack). **User (BAT-Q9): no flag; a bound stack does not return.** A
  blinded, petrified or paralysed Harpy stays as well (BAT-ATK-004).
- **Golden replays:** hashes re-recorded with the same seeds; the command logs and results are
  unchanged (a dead stack's recomputed values enter the state hash). `bench-check` passes
  without a new baseline.
- **T7b reading** (`installation.md` §7.21): the creature's cast action (case 10 → 0x448260: no
  spell named, casts left −1, a switch by creature), the cast routine's mastery and power
  arguments (2 and 6 for the Ogre Mage, the four AB elementals and the Master Genie; 2 and 5 ×
  creatures for the Faerie Dragon), the Faerie Dragon's weighted draw at its activation, the
  Master Genie's random useful spell (flag `0x800`, 0x447A80), the Archangel's raise (100 ×
  creatures) and the Pit Lord's Demons (0x447330, 0x5A7390), the Enchanter's per-side counter
  and weighted table (S-12 confirmed: the first stack decides), the hero's spell cost with
  Pegasi and Mages read from the armies (batch 2a R6 round 3 explained) and the Familiars'
  channel. Proposed cut in the epic (T7b1–T7b4, T7c), to confirm.

### Batch 2d folded in (2026-10-08)

- **Seam:** the user committed the results on a new branch while the T7a checkpoint ran
  (`battle/e04b-batch-2d-results`), rebased onto `master` after that checkpoint merged; the
  fold-in on the same branch.
- **Results** (`experiments/results/E04b-batch-2d.md`, SoD 4.0 Complete, plain HD; SU's rows
  counted from 1, confirmed by the user): as the code reading (`installation.md` §7.19) in every
  encounter but the elementals' native terrain. CL1: the clones up-right (attacker) and up-left
  (defender), both act in round 1, the Archer clone 12 shots then 11, Magic Arrow 30 kills all 10,
  the defender's clone gone at round 3 without a corpse. CL2: no clone, the Griffins and Wolf
  Riders themselves gone at round 3, the Griffins lost from the army (BAT-QUIRK-7 as read).
  RA1: every cursor as predicted (BAT-QUIRK-5 faithful; Sacrifice refuses the target as its own
  victim). RA4: the later corpse raised (1 Swordsman), the two-hex corpse refused until the
  Archers left. RA2: 5 (top 25), 8, 2, the army keeps 1. RA3: only the Ballista (S-20); the
  hero still moves and loses its next battle at once. NE: 7 Skeletons (created stacks counted),
  experience 600; Archangels 31/31/19, Pixies 3/3/8, **Air Elementals 10/10/8 and Earth
  Elementals 11/11/5: natives**, as the engine (the reading of 0x43D730 predicted 9/9/7). SU:
  E1 in row 7, then random free rows of column 1, then column 2; E3 in E1's record acts first; 12
  casts to the limit (refused with the "already summoned" message), still refused after a death;
  a, b and c all leave Red's hero gone, c pays Red's player 300. **Surrender prices 800 → 800
  (CL1) and 910 → 910 (SU): created stacks left out**, war machines counted; the engine had 1 300
  and 1 160. SU a's casualty window lists 2 Air Elementals of at least 4 killed (a reused
  record's losses seem lost; an experiment candidate).
- **Community search (the user's condition for BAT-QUIRK-7):** heroes3wog.net "Did you know?
  #136" (2022) and pikabu.ru "Знаете ли Вы всё о… «Клоне»" (2018) describe the vanishing
  original, the latter also the clone overlapping its original (invisible, untargetable) and
  Necromancy from clones as a bug HotA fixed; HD+ lists a Clone fix.
- **Decisions (the user):** the surrender price follows the original; BAT-Q7 (b) for Necromancy
  (the outcome reports created stacks' kills: a seam addition, T6c) and (a) for a retreat or
  surrender against only created stacks (documented, no contract change); BAT-QUIRK-7
  fix-optional (known); the agent split off BAT-QUIRK-8 (the overlap, exploit-fix).
- **Battle:** `flee::surrender_price` leaves created stacks out (BAT-FLE-002); test
  `the_surrender_price_leaves_created_stacks_out`. Replays and versions unchanged.
- **Recorded:** observation sources on BAT-TER-001, BAT-END-001, BAT-FLE-002, BAT-END-003 and
  BAT-MAG-035 to 039; the ledger's BAT-QUIRK-5/6 (observed), BAT-QUIRK-7 (fix-optional, the flag
  queued) and BAT-QUIRK-8; BAT-Q7 answered; `installation.md` §7.19's notes; T6c queued.

### T7b1: the creature's cast (2026-10-08)

- **Seam** (the user, BAT-Q10, at the start of increment 7): T7b1 alone, with a checkpoint
  after; T6c goes with T7b3 in increment 8 (both change `BattleOutcome`: one `shared/` pull
  request); the command is a new `CreatureCast { target }` that names no spell, as the original's
  action 10 (ADR-0014 points 3 and 9 amended in the shared PR #18, merged first).
- **Code reading** (`installation.md` §7.21, "Who may cast, and at what"): 0x4476C0 and the cursor
  0x4766EA: casts left, magic allowed for the controller's side (0x41F890: not Cursed Ground, not
  an anti-magic garrison, no Orb of Inhibition), a living stack on the hex the spell can affect
  (0x5A3C80), only another stack of the acting side and **never the caster itself**; 0x448260
  casts through 0x59FE30(spell, hex, 1, −1, 2, 6); the dispatcher rolls good morale after it
  (0x464920) and does not re-enter the caster's hexes (no 0x441F70); a created stack gets its
  creature's casts (0x47A100).
- **Content:** ability `spell_caster` (spell, mastery, power; pack format v22, ruleset hash v31);
  the compiler accepts a spell with damage, timed, dispel or heal effects only, no health limit
  (the health limit reads the hero), aimed at a stack at that mastery in every profile. Harvested
  from the gap-fill (`exe-creature-cast`): Ogre Mage Bloodlust, Storm/Ice/Energy/Magma Elementals
  the Protections from Air/Water/Fire/Earth, Advanced, power 6. The synthetic Wisp casts Hasten
  twice a battle.
- **Battle:** `h3_battle::casters` (casts left beside the stacks, set at the start and when a
  stack is created; `stack_spell`, `creature_cast_spell`, `creature_cast_targets`, `plan`);
  `magic::plan_creature_cast` and `creature_target` (another stack of `first_target` at the
  caster's mastery), `Cast::by` (the casting stack: no payment, `StackCast` event, no resistance
  roll for the aimed stack); `Command::CreatureCast`, `CommandError::CannotCast`; the turn ends
  as `Acted` (good morale may follow, no `obstacles::stand`); observation `casts_left`; state
  hash v33. BAT-MAG-048; BAT-MOR-003 gains the cast.
- **Sim and ABI:** entry 1395 legal: spell head the stack's spell only, target head its targets,
  second head only 0 (action encoding v11); the random agent draws it as one more entry, then a
  uniform target; observation encoding v8 (casts left), C ABI v12; replay text
  `creature-cast stack <s>` (replay format v19). Fixed: the cast heads encoded hex 255 as the
  first stack's index (`target_index` now refuses hexes beyond the field's limit).
- **Measurements:** the new binary on T7a3's workload +0.42 % Ir after making the random agent's
  check cheap (the active stack's casts first; the targets' buffer only for a caster; +2.11 %
  before); the new workload has 2.7 % more steps (9 526, Wisps cast Hasten) and +6.98 % Ir
  (hasted stacks reach further), re-baselined. Replays re-recorded with new agent seeds keeping
  T5–T7a's features and showing the Wisps' casts: a51, a168, a42, a225, a123.
- **Tests:** `tests/casters.rs` (the cast and its turn end, the refusals, a hypnotized caster, no
  resistance roll, good morale after a cast, a clone's casts), the property test's legality and
  casts-left checks, the sim's head-by-head test, the C ABI's head-by-head test.

### T7b2a: the Faerie Dragon (2026-10-08)

- **Seam and decisions** (the user, BAT-Q11, at the start of increment 8): T7b2 in two parts with
  one checkpoint after both (T7b2a Faerie Dragon, T7b2b Master Genie; T6c with T7b3 becomes
  increment 9); the drawn spell is private to the dragon's controller until it casts (ADR-0014
  point 8 amended, shared PR #22); the Genie's spell head allows only 0 and the observation
  counts its candidates (point 9, for T7b2b); S-18 is not reproducible in SoD (no flag); a quirk
  found in the reading, BAT-QUIRK-9, fix-optional.
- **Code reading** (`installation.md` §7.21, "The Faerie Dragon, re-read for T7b2a"): the draw
  (0x447510) at every activation (0x465330) with casts left, again after Wait and good morale, not
  in the tactics phase; `rand() mod 100` over 0x63B850 in table order; a combat-log line for a
  local human controller; +0x4E0 read only for the active stack. The target (0x4476C0 case 7,
  the table 0x447A00 decoded for every caster): a stack on the hex that 0x5A3C80 accepts (its
  polarity test only: Chain Lightning and the area spells may hit own stacks); the cursor
  (0x4766FC) any stack but the caster, an empty hex never (0x47684B → 0x4473D0, the Archangel's
  and Pit Lord's). The cast: 0x59FE30(spell, hex, 1, −1, 2, 5 × creatures). Magic Mirror: the
  creature caster's button goes through 0x59EC50 (action 10), and 0x59FE30 skips the test of
  caster kind 1 at 0x5A02CB, the redirected stack included: BAT-QUIRK-9.
- **Content:** abilities `random_spell_caster` (mastery, power per creature) and `drawn_spell`
  (spell, weight), one entry per spell (pack format v23; the compiler: they need each other,
  exclude `spell_caster`, a drawn spell takes a stack or hex at the caster's mastery and places no
  obstacle; ruleset hash v32). Harvested from the gap-fill (`exe-faerie-dragon`): Ice Bolt 22,
  Lightning Bolt 22, Fireball 21, Magic Arrow 10, Frost Ring 10, Chain Lightning 5, Meteor Shower
  5, Inferno 5, Advanced, 5 per creature. Quirk flag 14 `quirk.battle.creature_mirror_untested`
  (faithful on, strict-fixed off). The synthetic Drake (Spark 5, Cinders 3, Arc 2, Advanced, 2
  per creature, three casts) replaces the defender's last Brute stack in `full.toml`.
- **Battle:** `casters::activated` (from `activate_next`, the good-morale turn and a berserk
  stack that keeps its turn) draws with the battle's RNG into `Casts.drawn` (the active stack's,
  forgotten at any other activation; state hash v34) and emits `SpellDrawn` to the controller;
  `casters::stack_cast` gives the drawn spell, power per creature × creatures; `magic::drawn_target`
  and `affectable` (polarity side, immunities, Anti-Magic, share and aura above zero, never the
  caster; a hex spell at a hex of such a stack), `magic::creature_targets` (hexes from the
  stacks' footprints); `mirror` returns whether it sent the spell away, and `reach_stacks` tests
  and rolls a creature's redirected target unless BAT-QUIRK-9 is on. Observation `drawn` (only
  for the controller of the active stack).
- **Sim and ABI:** observation encoding v9 (a 12th global: the drawn spell's handle + 1 for its
  controller), C ABI v13; the spell head shows the drawn spell, the target head may hold hexes
  (layout unchanged, action encoding v11).
- **Fixed on the way:** a random game on `master` failed `hidden_obstacles_do_not_show`: a raised
  native gave its side sight of a hidden mine again (BAT-MAG-031 without BAT-QUIRK-1), which the
  property did not allow for; it now skips a command after which the viewer sees an obstacle it
  did not see (the engine was right; the case is kept in the proptest regressions).
- **Measurements:** the new binary on T7b1's workload +0.25 % Ir. The new workload (a Drake
  stack): 9 835 steps (+3.2 %), +8.63 % Ir (the drawn spells' targets, `affectable`, about 1.1 %;
  the rest the flying two-hex caster's play), re-baselined. Replays re-recorded with new agent
  seeds keeping T5–T7b1's features and showing the Drakes' casts: a759, a804, a1522, a1432, a155.
- **Tests:** `tests/casters.rs` (the draw by weight and its visibility, no casts and forgetting,
  the targets of a negative and a neutral drawn spell, an area at a stack's hex hitting the
  caster, power per creature and S-18's golems, a hypnotized dragon, the draws after good morale
  and Wait, BAT-QUIRK-9 in both profiles); the hidden-state property covers the drawn spell; the
  content's list rules; the sim's drawn-spell global.

### T7b2b: the Master Genie (2026-10-08)

- **Decisions** (the user, BAT-Q12): whether a hero carries a spell book becomes public (the
  Genie's test reads it and its draw counts would reveal it; ADR-0014 point 8 amended, shared PR
  #24; the spells stay private); the test's Slayer case (dead enemies count, king level 1 only)
  is the rule in every profile (ledger BAT-QUIRK-10).
- **Code reading** (`installation.md` §7.21, "The Master Genie, re-read for T7b2b"): 0x4476C0's
  table 0x447A00 decoded; case 1: a stack the acting side owns (the cursor), not the caster, for
  which 0x447A80 accepts a spell of 10–69 with flag `0x800` (21 spells); 0x447EE0 draws
  `random(1, n)` among them in spell order and casts at mastery 2, power 6. 0x447A80: not active,
  0x5A3C80, then the cases (byte table 0x447E8C): Shield and Fire Shield an enemy that cannot
  shoot, Air Shield one that can (alive, not blind, petrified or paralysed, not 147/148), the
  Protections, Anti-Magic and Magic Mirror an enemy hero's spell book, Cure a wounded top
  creature, Bloodlust a stack that cannot shoot, Precision one that can (0x4428F0), Prayer not
  acted this round, Slayer an enemy record with king 1, dead or alive.
- **Content:** ability `helpful_spell_caster` (mastery, power) with `drawn_spell` entries (the
  compiler: a helpful caster's drawn spells take a stack; the three caster kinds exclude each
  other), the spell field `useful` (`Usefulness`: always, enemy_melee, enemy_shooter,
  enemy_spell_book, wounded, melee, shooting, not_acted, enemy_king; pack format v24, ruleset
  hash v33); the gap-fill writes `useful` for 14 spells (`spells.useful`, source `exe-useful`) and
  gives the Master Genie its 21 spells with weight 1 (`exe-genie`). The synthetic Djinn (Hasten,
  Fury, Boon, Aegis, Hallow, Smiting, Mend) replaces the attacker's last Spearman stack in
  `full.toml`; five synthetic spells got `useful`.
- **Battle:** the public `CastSpell` (`Named`, `AtResolution`) for `stack_spell` and
  `creature_cast_spell`; `casters::useful` (with `combat::can_shoot` and the hero view's
  `spell_book`), `useful_spells`, `candidates`; `magic::helpful_target`, `plan_creature_cast`
  takes the first useful spell and `Cast::draws`, `draw_useful` draws by weight in `apply_cast`
  before the cast is reported; `HeroView.spell_book` (public; state hash v35); observation
  `cast_candidates`.
- **Sim and ABI:** the Genie's spell head allows only 0 (`spell_head`), decode checks it;
  observation encoding v10 (the hero block's 12th public feature, the stack's 36th value), C ABI
  v14.
- **Measurements:** the new binary on T7b2a's workload +0.01 % Ir. The new workload (a Djinn
  stack): 8 838 steps (−10.1 %), −6.42 % Ir (+4.1 % per step; `useful` and its `can_shoot` about
  0.3 %), re-baselined. Replays re-recorded with new agent seeds showing T5–T7b2a's features and
  the Djinns' casts: a1914, a1535, a1886, a463, a1825.
- **Tests:** `tests/casters.rs` (the draw among the useful spells and its counts for the
  controller only, no target without a useful spell, every usefulness case, the enemy's shooters,
  war machines, spell book and kings); the content's list and compile rules; the sim's spell head
  0; the pack's unknown usefulness; the hero view's book flag.

### Increment 9 start and T6c: the created stacks' follow-ups (2026-10-08)

- **Seam** (the user): increment 9 = T6c with T7b3 as planned, one checkpoint after both; one
  shared PR for the outcome (#26, `shared/battle-outcome-raised`), then T6c and T7b3 as stacked
  battle PRs.
- **Code reading** (`installation.md` §7.21, "The raisers and the end of the battle"): the Pit
  Lord's Demons are an ordinary army stack in the corpse's army slot; Necromancy (0x4775E1) and
  the experience (0x46A350) count per record, `start − count`; a reused record starts afresh; the
  death 0x444120 keeps the dying stack's count but zeroes an original's clone; Clone on its
  original flags the stack on the position's hex (0x4E7170) `0x400000`, not `0x800000`.
- **Decisions** (the user, BAT-Q13): the Demons as `StackOrigin::Raised(slot)`; created stacks as
  records with `StackOrigin::Created` (exact per-record Necromancy; the experience leaves them out
  by origin), instead of BAT-Q7 (b)'s sum by creature; a reused record's lost losses the rule, no
  flag (ledger BAT-QUIRK-11). ADR-0014 points 6, 9 and 10 amended in #26.
- **Content:** quirk flag 15 `quirk.battle.clone_on_its_original` (BAT-QUIRK-7; faithful on,
  strict-fixed off); packs regenerated (the base and synthetic profiles).
- **Battle:** `created::clone_place` returns a `ClonePlace`; with the flag the original's hexes are
  free and a place on them turns the original (`created::turn`: `SUMMONED` without `CLONE`, the
  clone's rounds, linked to itself; event `TurnedSummoned`); a place whose front hex is the
  original's is skipped in every profile (BAT-QUIRK-8). `Created.vanished` keeps the creatures a
  stack vanished with when its rounds ran out (state hash v36); `BattleState::outcome` reports
  every stack by origin, `end` with the vanished creatures, BAT-QUIRK-6's keep-one over army
  stacks only.
- **Tests:** `tests/created.rs` (BAT-QUIRK-7 both ways, BAT-QUIRK-8 under the flag, the outcome
  per created stack with a reused slot and a clone gone with its original, the vanished count);
  the property `outcomes_account_for_every_stack` covers every stack by origin. Golden replays:
  hashes re-recorded (same commands and results).

### T7b3: the raisers (2026-10-08)

- **Decisions** (the user, BAT-Q13, at the increment's start): the Demons as
  `StackOrigin::Raised(slot)` (shared PR #26). Agent decisions: the Archangel and the Pit Lord keep
  the sim's spell head at 0 (ADR-0014 point 9, in #26); the raise without a free record, which
  crashes the original, is refused (ledger BAT-QUIRK-12, exploit-fix); a raised stack takes the
  turn order's tie-break of a created stack (`Stack::turn_slot`), as the original's new record.
- **Code reading** (`installation.md` §7.21, "The raisers and the end of the battle"): 0x4473D0
  and 0x4476C0's case 0 (no test of the side's records; the cursor never offers the caster),
  0x5A3CC0 with its third argument 1, 0x447330's two counts, 0x5A3E40's corpse search, 0x5A7390.
- **Content:** abilities `stack_raiser` (spell, hit points per creature) and `corpse_raiser`
  (creature, hit points per creature; one-hex, at most `MAX_RAISED_CREATURES` = 1 in a ruleset),
  pack format v25, ruleset hash v34; the gap-fill's `exe-raisers` (Archangel: Resurrection, 100;
  Pit Lord: Demons, 50). The synthetic Seraph (two Seraphs, the attacker's slot 4) and Fiends (six,
  the defender's slot 4, raising Spearmen) in `full.toml`.
- **Battle:** `CastSpell::Raise`; `casters::Raiser`, `raiser`, `raise_target`, `corpse_count`,
  `raise`, `CreatureAction` (`plan`), `Plan::CreatureRaise`; `raise::creature_test`,
  `corpse_raisable` and the shared `on_top`; `created::has_slot`, `raise_corpse`; the side's
  templates (`TEMPLATES` = 5) hold the raised creature when either army has a corpse raiser;
  `StackTraits::RAISED`, `Stack::turn_slot`; events `CreatureRaise`, `RoseFromCorpse`; the outcome's
  `StackOrigin::Raised`, its survivors sorted by slot. The summon templates no longer carry
  `SUMMONED` (Summon Elemental sets it), and a clone of a raised stack is no raised stack.
- **Measurements:** `BattleState` 5 888 → 6 200 bytes (two template stacks of 72 bytes per side
  and T6c's 168 bytes of vanished counts). T6c's and T7b3's code on the old workload +0.17 % Ir
  (8 838 steps, unchanged). The new workload (a Seraph and Fiends): 8 152 steps (−7.8 %), +2.54 %
  Ir (the raises themselves about 1 %; the rest the armies), re-baselined. Replays re-recorded
  with new agent seeds showing the raises with T5–T7b2's features: a755, a1253, a709, a631, a373.
- **Tests:** `tests/raisers.rs` (the Archangel's targets, health test and permanent raise, a
  corpse; the Pit Lord's corpse search, count, slot, outcome, a hypnotized raiser, Demons from
  Demons, the missing slot); the content's raiser checks; the properties: a reused slot taken by a
  raised stack, the outcome's `Raised` records, one survivor per slot.

### T7b4: the Enchanter (2026-10-09)

- **Decisions** (the user, BAT-Q14): increment 10 is T7b4 alone; survey S-12 fix-optional (ledger
  BAT-QUIRK-13, `quirk.battle.periodic_first_stack`, faithful on); every battle starts with the
  counters ready (BAT-QUIRK-14, exploit-fix: the original's counters carry over between battles).
  Agent decisions: the draw uses the battle's random generator (the original's `rand()`, as the
  Faerie Dragon's); a periodic caster's drawn spells take no target and do no damage (the compiler
  checks), so a cast at an activation ends no battle; the activation casts before a berserk
  stack's automatic turn and at every re-activation (Wait, good morale), as the original's
  0x465330; the observation shows each side's count (public: every cast is).
- **Code reading** (`installation.md` §7.21, "The Enchanter, re-read for T7b4"): 0x465330's
  creature switch (0x465503), 0x465080's skipped activations (0x465283), round start 0x475ED0 and
  its callers (no call before a battle's first round), the battle manager made once per session
  (0x4EDCF5, constructor 0x462760, which leaves +0x132A0 alone), 0x447FE0's two loops (the aimed
  side 0x448042, the first record 0x44807C, `rand()` 0x4480B8, the cast 0x448250), 0x5A8090's
  dead-stack refusal. Eight address-map entries.
- **Content:** ability `periodic_spell_caster` (rounds, mastery, power) with `drawn_spell`
  entries, excluding the other caster kinds; pack format v26, ruleset hash v35; quirk flag 16;
  the gap-fill's `exe-enchanter` (Haste 15, Air Shield 10, Slow 10, Stone Skin 15, Bloodlust 5,
  Bless 15, Cure 10, Weakness 4; every 3 rounds at Expert with power 3). The synthetic Chanters
  (five, the defender's slot 5, replacing his second Wisps) in `full.toml`.
- **Battle:** `Casts.periodic` (per side, 255 at the start, +1 at every round start,
  `casters::round_started`), `casters::periodic` (at the activation in `turn::begins`, after good
  morale and in a berserk stack's good-morale loop), `counts`, `periodic_spells`;
  `magic::periodic_cast` (`Cast.automatic`: no cast used); event `PeriodicCast`;
  `StackTraits::PERIODIC_CASTER`; `BattleState::periodic_rounds`; `Observation.periodic_rounds`.
  State hash v37, observation encoding v11 (two globals), C ABI v15.
- **Measurements:** the code on the old workload +0.31 % Ir with unchanged steps (8 152), nearly
  all a codegen swing in `movement::reachable_within` (untouched), the activation's test about
  45 k Ir; the new workload (Chanters): 7 494 steps (−8.1 %), +4.57 % Ir (mass Haste and Slow change
  the walkers' reach; the morale recomputations after mass spells), re-baselined. `BattleState`
  stays 6 200 bytes. Replays re-recorded with new agent seeds showing the Chanters' casts with
  T5–T7b3's features: a753, a38, a402, a1732, a1006.
- **Tests:** `tests/enchanters.rs` (ready at the start, the mass version for three rounds, once in
  three round starts per side, nothing counted, the quirk on and off, a dead first stack, a
  hypnotized and a berserk Enchanter); the content's periodic-caster checks; the observation's
  globals; the property test explains an enemy stack healed by the other side's periodic Cure.
- **Review** (independent, no defect in the rule's logic): a coverage gap (a periodic caster
  whose ability comes first never reached `periodic`'s other abilities: the test `luster` lists it
  last), a stack that lost its turns during its own action no longer casts at a good-morale turn
  (`Status::NO_TURNS`; the good-morale grant itself is in the QUEUE's inbox), the berserk and the
  hypnotized tests made stronger; the property test that wants every random game to end now
  assumes it (a petrifying striker and a First Aid Tent can keep a battle going for ever).

### T7c1: the mana creatures (2026-10-09)

- **Decisions** (the user, BAT-Q15): increment 11 is T7c1 alone (T7c2, the creature defences,
  next); the spell cost reads the armies, so a dead Pegasus still counts (ledger BAT-QUIRK-15, the
  rule, no flag); the Familiars' loop that walks both sides' records up to one side's count
  (BAT-QUIRK-16) always fixed. Agent decisions: the changes as data (`spell_cost` with `affects`:
  the largest increase and the largest reduction, once each, the original's +2 and −2 as data);
  a spell with cost 0 stays free (the original names Titan's Lightning Bolt, the only hero spell
  with cost 0); the channel as a percent (20: `cost div 5`), the largest among the side's living
  stacks; the cost change computed once at the battle's start from `BattleSetup`'s armies
  (`HeroMagic.cost_change`), as they never change in a battle; the round's wait phase kept in the
  state (`BattleState.wait_phase`, the original's +0x13DE4), set when a waited stack is activated,
  so a stack created later in that phase drains nothing either; the drain and the periodic cast
  share one activation hook (`turn::activation`, which skips a stack that lost its turns); the
  events public (mana is). No observation change: mana is shown, and the armies' creatures,
  dead ones included, are in the stack features.
- **Code reading** (`installation.md` §7.21, "The mana creatures, re-read for T7c1"): 0x4E5240
  (no change without an enemy army; a cast passes one, 0x59FFCF), 0x44AB80 (types only), the
  cast's payment 0x59FFDF and 0x4D92D0; the 35 handlers' common tail 0x5A2058 and the channel
  0x5A2170 (caster kind 0, the paid cost at least 5, the Familiars' creatures by controller, the
  loop bound 0x5A21B2, no cap); the activation's Wraith case 0x465548 and the wait phase (0x4652A7
  sets +0x13DE4 and clears every record's waited bit, 0x4760AB clears it). Ten map entries,
  five struct fields. VCMI 1.7.5 differs: living stacks for the cost, the drain once per round.
- **Content:** abilities `spell_cost` (amount, affects), `mana_channel` (percent),
  `mana_drain` (amount); pack format v27, ruleset hash v36; the gap-fill's `exe-mana-creatures`
  (Pegasus and Silver Pegasus +2 enemies, Mage and Arch Mage −2 allies, Familiar 20, Wraith 2).
- **Battle:** `h3_battle::mana` (`cost_change`, `cost`, `channel`, `drain`); `apply_cast` split
  (`take_effect`), the channel after a hero's cast's effect, before the end check; events
  `ManaChanneled`, `ManaDrained`; `StackTraits::MANA_DRAIN`; state hash v38.
- **Measurements:** the code on the old workload +0.18 % Ir, steps unchanged (7 494); the new
  workload (the synthetic Brutes, Djinns, Wisps and Fiends with the abilities) 7 943 steps
  (+6.0 %), +4.50 % Ir, −1.4 % per step; re-baselined. `BattleState` stays 6 200 bytes. Replays
  re-recorded with new agent seeds showing T5 to T7c1's features: a1159, a416, a255, a48, a1023.
- **Tests:** `tests/mana.rs` (the armies' changes, at least 1, a free spell, the refused cast; a
  dead Pegasus; the channel: share, threshold, largest, after the effect, own side, no hero; a
  hypnotized Familiar beyond the other side's count; the drain: activations, the wait phase, at
  least 0, no hero, a hypnotized Wraith, good morale, lost turns); `h3-pack`'s
  `harvested_mana.rs` replays batch 2a R6 on the harvested packs (20 → 21 → 16 → 11, Blue
  10 → 2); the property test explains mana by casts, channels and drains.
- **Review** (independent, no defect in the logic): tests made stronger (two enemy Pegasi
  count once; Familiars killed by the cast while the battle goes on; the channel after an
  obstacle spell; a blind, a bad-morale, a berserk Wraith and the tactics phase through
  commands), the test Wraith's ability after another (the drain's search), the channel's event
  reports the mana actually gained, `turn`'s docs. A created Pegasus cannot be tested: a clone's
  original is in its army and no summon makes Pegasi (the change is computed from the armies
  at the start).

### T7c2a: the creature defences (2026-10-09)

- **Decisions** (the user, BAT-Q16): T7c2 in two parts, T7c2a (the Faerie Dragons' mirror, the
  Efreet Sultans' shield, the elementals' damage factors) in increment 12, the Phoenix's rebirth
  (T7c2b) next; T7c1's own calls confirmed. Agent decisions: the two creature spells as one
  ability, `innate_spell` (a spell with a timed `fire_shield` or `magic_mirror` change, and a
  value), read only where the original's getters read the creature (the mirror's chance, the
  shield's factor), so Hypnotize, Dispel and the Master Genie's usefulness see the spell only, as
  in the original; "the larger of the spell's value and the creature's" for both (the original
  takes the larger for the dragon and the spell's while active for the Sultan; no pack value of
  either spell is below 20, so they never differ); the halves as `damage_penalty` (a creature)
  and `damage_penalty_by_flag` (a flag: mind-immune targets), the largest applying, in stage 2
  right after the defense factor (exact: one more factor). No new state, event or encoding.
- **Code reading** (`installation.md` §7.21, "The creature defences, re-read for T7c2a"):
  0x4487F0 (creature 134: the larger of the spell's value and spell 36's table value 20; only the
  cast's entry and the AI ask it; Hypnotize reads the duration, 0x5A83A6), 0x443130 and 0x422440
  (creature 53: 0.2 as an `f32` while the spell's duration is 0; the same reflection path with
  the Sultan's hero's spell bonuses), 0x443B90 (creature 120 against flag `0x400`, 121 against
  121 and 83, × 0.5 as a double right after the defense factor). A first look at the Phoenix's
  rebirth 0x469440 for T7c2b (its callers, the count: start div 5 plus a 1-in-5 draw per creature
  left over). One map entry (0x445A60). VCMI 1.7.5: `MAGIC_MIRROR` 20, `FIRE_SHIELD` 20, no halves.
- **Content:** abilities `innate_spell` (spell, value), `damage_penalty` (creature, percent),
  `damage_penalty_by_flag` (flag, percent); pack format v28, ruleset hash v37; the gap-fill's
  `exe-creature-defences` (Faerie Dragon Magic Mirror 20, Efreet Sultan Fire Shield 20, Psychic
  Elemental half against `mind_immune`, Magic Elemental half against Magic Elementals and Black
  Dragons); packs re-harvested.
- **Battle:** `magic::innate`, `magic::mirror_chance` (BAT-MAG-057), `magic::reflection` with an
  innate shield (BAT-MAG-058), `StackTraits::INNATE_SPELL` (the strike's cheap test), the strike's
  `creature_penalty_percent` (BAT-DMG-010); catalog BAT-MAG-018, BAT-MAG-022 and BAT-DMG-002
  updated. State hash unchanged (v38).
- **Measurements:** the code on the old workload +0.01 % Ir, steps unchanged (7 943); the new
  workload (the synthetic Drakes mirror, the Fiends reflect, the Djinns halve against two-hex
  stacks, the Chanters against Djinns) 7 558 steps (−4.8 %), −5.62 % Ir (−0.8 % per step);
  re-baselined. `BattleState` stays 6 200 bytes. Replays re-recorded with new agent seeds: a676,
  a1386, a1473, a471, a673 (a Drake's redirect in four of them).
- **Tests:** `tests/creature_defences.rs` (the chance and the max rule, Hypnotize and Dispel, the
  Genie's usefulness; the dragon's 20 % over 300 casts; a creature's cast sent away untested with
  BAT-QUIRK-9; the Sultan's shield against the spell's values, Q-0010 with two Sultans, shots,
  fire-immune strikers, the hero's bonus; the halves in both profiles, the largest of two, the
  stage against a shielded target, retaliations); the ability's compile checks and pack syntax.
- **Review** (independent, no defect in the logic): tests made stronger (the stage shown by a
  shielded target, the largest of two penalties, retaliations, Q-0010 with two Sultans, Dispel
  and the Genie's usefulness, the resolved spell handles), the innate trait's doc.

### T7c2b: the Phoenix's rebirth (2026-10-09)

- **Decisions** (the user, BAT-Q17, BAT-Q18): increment 13 = T7c2b and T7c3, closing T7; the CI
  fix as shared #41. A sacrificed Phoenix is never reborn (BAT-QUIRK-17, exploit-fix for now; a
  later variant may rebirth it right after the sacrifice). Agent decisions: one ability `rebirth`
  with a divisor, using the creature's casts per battle; only a stack that died in the action just
  checked qualifies (no corpse marked yet), which also excludes a multi-cast creature's earlier
  death (data only); the draws on the battle's generator; an active stack its own hero's spell
  killed keeps the turn it is in, having acted (the original's dispatcher re-picks only a dead
  one, as the engine does); event `Reborn`.
- **Code reading** (`installation.md` §7.21, "The Phoenix's rebirth, read for T7c2b"): 0x469440
  (both sides' records; dead, casts left, not a clone; casts lowered first; start div 5 plus
  `rand() mod 5 = 0` per leftover; the acted bit; 0x5A7560 with last argument 0, permanent),
  0x445A60 (the strike action, read: shot or melee, then the check), the dispatcher's order after
  it (acted bit, the end test 0x4762F0, good morale 0x464920 clearing the acted bit), the cast
  routine's damage cases, the area and mass handlers, Land Mine and Fire Wall; not Sacrifice,
  whose victim keeps its casts (BAT-QUIRK-17); 0x4687C0 overwrites the hex records. Map: 0x445A60
  read, 0x5A4BC0, 0x5A67C0.
- **Content:** ability `rebirth` (divisor); pack format v29, ruleset hash v38; the gap-fill's
  `exe-phoenix-rebirth` (Phoenix, divisor 5).
- **Battle:** `raise::rebirths` before the end check of melee, shot, cast, Fire Wall and mine
  (BAT-MAG-059; `StackTraits::REBIRTH`, `raise::revive` shared with the spells' raise); good
  morale lifts the acted mark (BAT-MOR-003). State hash unchanged (v38).
- **Tests:** `tests/rebirth.rs` (a fifth, acted, before the end check, one cast; the draws over
  200 seeds; the retaliation and the good-morale turn; a spell, a shot, a mine; clones,
  sacrificed and earlier deaths; the effects it died with; the active stack its hero's spell
  killed); the property test counts the rebirth's cast and creatures.

### T7c3: Fear and T7's loose ends (2026-10-09)

- **Decisions** (the user, BAT-Q17, BAT-Q18): Fear as the code reads, the frightening stacks by
  real side (BAT-QUIRK-18, the rule). Agent decisions: ability `fear` (percent); the largest
  chance among the living frighteners, one roll; immune: no morale (war machines included) and
  `fear` itself; event `Frightened`; two corrections found in the reading, as the original: the
  battle's first activation without Tactics has no bad-morale or Fear check (BAT-MOR-004 had it),
  and good morale after an action that left the stack blind, petrified or paralysed gives no turn
  now but lets it act later in the round once that ends (the T7b4 review's finding).
- **Code reading** (`installation.md` §7.21, "Fear, read for T7c3"): 0x464D40 (no morale flag
  `0x20000` or creature 132 spared; the other side's creature-132 counts by real side;
  `rand() mod 10 = 0`; the acted bit), the pick 0x465080 (bad morale, then Fear, outside the
  tactics and wait phases, only with argument 1), its callers (the dispatcher and the tactics end
  with 1, the battle start 0x462A20 with 0). Map: 0x462A20, 0x464B40, 0x464D40.
- **Content:** ability `fear` (percent); pack format v29 (with `rebirth`); the gap-fill's
  `exe-fear` (Azure Dragon, 10).
- **Battle:** `morale::frightened` at the pick after bad morale and at the good-morale turn
  (`turn::morale_turn`, shared by the commanded and berserk turns), `turn::activate_first`
  (BAT-MAG-060, BAT-MOR-003, BAT-MOR-004; `StackTraits::FEAR`). The stale "creature casters'
  re-casts" item: the creature's cast goes through the same effect routine; tested (a re-cast
  keeps the first value and the longer duration, BAT-MAG-009).
- **Tests:** `tests/fear.rs` (the first activation, immunity, round 2's freeze; the largest
  chance and the wait phase; the hypnotized Azure; the good-morale turn of a commanded and of a
  berserk stack; the first pick after Tactics); `tests/morale.rs` (the first activation; a stack
  blinded by its own action passes its morale turn and acts once struck); the Wraith's and the
  Enchanter's no-activation tests now go through the morale turn.
- **Measurements** (both tasks): on the old workload the code takes +0.66 % Ir with 7 568 steps
  instead of 7 558 (the first activation's and the morale turn's corrections change the battles;
  `raise::rebirths` 0.65 M and `morale::frightened` 0.74 M Ir of it); the new workload (the
  synthetic Brutes reborn at a fifth, Corwin's Drakes frightening at 10 %) 7 607 steps, 569.4 M
  Ir (+0.19 % per step against master's), re-baselined; `BattleState` stays 6 200 bytes; release
  `h3 bench` about 116 k steps/s. Replays re-recorded with new agent seeds a1798 (Brea wins),
  a1788, a116, a762, a959.
- **Review** (independent): a reborn stack kept the effects it died with until the command's end
  (fixed: removed before the raise, tested); a multi-cast creature's later rebirth onto taken
  hexes (fixed: only this action's dead); the first pick's checks after an action (no reachable
  case: war machines have speed 0); tests added for the berserk morale turn, Tactics, the active
  stack; bad morale's `WAITED` gate against the original's wait phase (pre-existing, inbox).

### Increment 14 start and T8a: available spells and masteries in the Heroes context (2026-10-10)

- **Start:** master green with original data (the sandbox's full verify; CI's run of the #42
  merge green, #41's fix holding). The user confirmed increment 13's own calls (BAT-Q19 (4)).
- **The T8 cut** (`installation.md` §7.22; BAT-Q19): a reading of the 224 callers of
  `hero_wears_artifact` (0x4D91F0) placed every magic artifact; the split T8a to T8e (epic file).
  Found on the way: Eagle Eye is rolled inside the battle at each enemy hero cast (0x59FF83), the
  Orb of Vulnerability returns "fully affected" before every resistance step, the opening spells
  are cast at the side's first stack activation, the mana artifacts have no battle effect (moved
  to E08–E10).
- **Decisions** (the user, BAT-Q19): the artifacts' spells exist outside battles (the adventure
  screen's book), so the hero's available spells are derived once in the Heroes context, not in
  the battle; the same for a spell's mastery; increment 14 = T8a alone; the seam change in one
  `cross-lane` pull request. Agent decisions: `masteries` covers every spell above none, beside
  the unchanged `spell_book` (Sacrifice's test under BAT-QUIRK-5 reads Resurrection's mastery);
  the book in spell order; `spells` may appear on any effect list (only artifacts have them in
  the packs), `scroll` on artifacts only; equipping Titan's Thunder adds a Spell Book in the
  original (0x4E2A64), an equipping rule for E08–E10.
- **Code reading:** 0x4D95D0 (the available spells from the learned ones, a worn scroll's spell
  and the sets of the artifacts the loader 0x44CD50 marks from 0x63B998), 0x4D9350 (the sets;
  0x4D9532 clears Titan's Lightning Bolt from all but Titan's Thunder's), 0x4E5080 and 0x4E5100
  (the mastery), 0x4E2A00. Map: 0x44CD50, 0x4D91F0, 0x4D9350, 0x4D9532, 0x4D95D0, 0x4E2A00,
  0x63B98C, 0x63B998; 0x4E5080 and 0x4E5100 re-read.
- **Content:** pack format v30: hero effects `spells` (a school, a level or a spell),
  `spell_mastery` and `scroll`, spell attribute `granted_by_name_only` (from the flag 0x2000);
  the gap-fill's `exe-available-spells` for the Spell Scroll, the Tomes, Spellbinder's Hat, the
  Sea Captain's Hat, Armageddon's Blade (still pending for T8b) and Titan's Thunder; ruleset hash
  v39. The synthetic pack has a codex, a wand and a scroll.
- **Heroes context:** `ArtifactInstance` (type and a scroll's spell) in `HeroDetails`;
  `available_spells` and `spell_masteries` (HER-SPL-002, HER-SPL-003, HER-ART-003);
  `derive_battle_hero` uses them (HER-BAT-011).
- **Seam and battle:** `BattleMagic.masteries` replaces `schools` (ADR-0013 point 2, ADR-0014
  points 2 and 10 amended); the battle keeps them as two bit planes (`Masteries`) and reads a
  spell's mastery there (BAT-MAG-002), the quirk's test Resurrection's; state hash v39,
  observation encoding v12 (the book's features carry 1 + the mastery), C ABI v16, replay format
  v20 (`masteries` per spell key in a hero entry).
- **Tests:** h3-heroes (the scroll, tome, crown and wand of the test pack; validation of
  instances), h3-content (the grants, the hash, the compile errors), h3-pack (the syntax; the
  harvested Tomes, Hat, Titan's Thunder, Blade, Sea Captain's Hat and scrolls give the original's
  spells and masteries; random legal heroes with scrolls and learned spells).
- **Measurements:** behaviour unchanged (every golden replay's command log is the same, only the
  formats and hashes moved; re-recorded with the same seeds); `random-rollouts-full` 568.3 M Ir
  (−0.19 %, the same 7 607 steps), re-baselined; `BattleState` 6 264 bytes (+64); release `h3
  bench` about 112 k steps/s.
- **Review** (independent): no bug; fixed: `spells` on a skill was ignored while `spell_mastery`
  counted (both read every effect source now), `scroll` off artifacts and grants naming a
  creature ability rejected at compile, the scenario writer keeps every mastery, stale docs, a
  test for the crown alone; BAT-Q19's record reworded to the `masteries` field. Left for T9: the
  observation shows masteries of book spells only.
- **A property gap from T7b3** (found by the coverage run's random cases, reproduced on master):
  `every_step_preserves_the_invariants` took a slot a Pit Lord's Demons take over (a dead created
  stack's, `RoseFromCorpse`) for a stack whose casts changed; such a slot starts afresh like one
  a created stack takes (BAT-MAG-052). The property now says so; the case is in the proptest
  regression file. The engine was right.

### Increment 15: T8b, the casting limits and the heroes' spell defences (2026-10-10)

- **Start:** CI green on `master` (the T8a merge); the local baseline verify with the original
  data showed `check-exe-map` red on `master`: the adventure lane's survey (#33, docs only, so no
  CI ran) cites four H3API addresses without map entries (0x4B18C0, 0x4E4070, 0x4E4990,
  0x4E4D90); inferred entries cover them now. The user (BAT-Q20): increment 14's own calls
  confirmed; T8b in two parts, T8b1 the casting limits, T8b2 the stack defences, in one battle
  pull request labelled `cross-lane` (the `BattleEffect` additions of ADR-0014 point 10).
- **Code reading** (`installation.md` §7.23): 0x41F890 (the Orb of Inhibition on either hero) and
  all its callers (the hero's cast in 0x59FE30, the creature cast and raise test 0x4476C0, the
  empty-hex test, the Enchanters' 0x447FE0 keeping the side's count, the AI's 0x4396E0); the
  hero's spell book button 0x59E900 (new map entry: no hero, cast this round, garrison, the Orb,
  then the book; Cursed Ground's level 1 and Recanter's Cloak's level 2 after the choice);
  0x5A8090's level test (the Cloak: 0 for every spell above level 2, abilities and positive
  spells too, before every other test), its Sphere (Dispel, Dispel Helpful Spells) and Blade
  (Armageddon) cases; 0x44A4D0: Power of the Dragon Father first (levels 1–4, no polarity test,
  Dispel excepted because its case returns earlier), the per-spell switch with the Pendants
  (table 0x44A92C), the Orb of Vulnerability on the casting side's or the stack's hero
  (0x44A746: 1, skipping the mind, fire and per-creature immunities and every resistance; the
  Unicorn aura still applies in 0x5A8090), then the Badge of Courage with the mind spells.
  Every artifact test is the controlling hero's (0x442690). The Archangel's raise tests
  Resurrection through 0x5A8090 (the Cloak stops it), the Pit Lord's search 0x5A3E40 calls no
  effect test (it does not).
- **Finding outside T8b** (QUEUE inbox, for the checkpoint): the same switch holds absolute
  cases the packs do not model: Bless and Curse spare the undead and creatures with maximum
  damage 0, Precision needs a shooter, Mirth and Sorrow spare creatures without morale,
  Fortune, Misfortune and Slayer creatures with maximum damage 0.
- **Content:** pack format v31: hero effects `no_casting`, `spell_level_limit` (T8b1),
  `spell_immunity`, `spell_level_immunity`, `mind_immunity`, `negate_immunities` (T8b2), ability
  `spell_immunity` with `absolute` (the Troglodytes' Blind and Stone Gaze, the Gargoyles'
  Poison); ruleset hash v40. The harvester writes a creature's whole ability list as a `sod`
  override when an ability names a `sod` spell: Titan's Lightning Bolt joins the Earth and Magma
  Elementals' immunities and the Air and Storm Elementals' double damage. The gap-fill gives the
  Orb of Inhibition, Recanter's Cloak, the eight Pendants, the Sphere, the Orb of Vulnerability,
  Badge of Courage (with its morale), Armageddon's Blade's immunity and Power of the Dragon
  Father's level immunity (source `exe-casting-limits`, `exe-spell-defences`); the pending list
  keeps Eagle Eye's three artifacts, the Angelic Alliance and the Armor of the Damned (T8c, T8d).
  The synthetic pack has an artifact for each new kind.
- **Heroes context:** HER-BAT-015 (one `NoCasting`, the smallest `SpellLevelLimit`), HER-BAT-016
  (`SpellImmunity` per spell, the largest `SpellLevelImmunity`, `MindImmunity`,
  `NegateImmunities`), each artifact type once.
- **Seam and battle:** six `BattleEffect` variants. `HeroMagic` keeps the limits, the defences
  and the negation (hashed; state hash v40); `BattleState.limits` (`MagicLimits`: no casting, the
  level limit, whether any hero guards) is derived at the start for the hot paths, not hashed.
  BAT-MAG-061 in `check_spell` (`CastingForbidden`), `casters::ready` (creature casts and
  raises) and the periodic cast; BAT-MAG-062 in `check_spell` (`SpellLevelBarred`),
  `affects_stack` and the raise tests. T8b2: `affects` takes the heroes' `Guard` (the controlling
  hero's defences, the negation of the casting side's or that hero), in the original's order;
  `affects_stack`, `share`, `affect_chance` and `resists` take the casting side (the striker's
  controller for the strike's spells, the obstacle's side for Fire Wall and mines); the strike's
  dispel of positive spells asks the Sphere.
- **Tests:** `tests/limits.rs` (both heroes' and creatures' casts, raises, the random caster's
  draw, the Enchanters' count, the strike's magic; the level limit for heroes, creatures,
  abilities, the Archangel and the Pit Lord), `tests/hero_defences.rs` (a hypnotized stack's
  controller, every Dispel against the Sphere, the Dragon Father's levels and Dispel, the Badge,
  the negation from either hero and not for the enemy's own-side spells, absolute immunities, the
  aura), h3-heroes, h3-content, h3-pack (syntax; `tests/harvested_defences.rs` on the harvested
  packs: the elementals against Titan's Lightning Bolt, the Orb against Troglodytes and Air
  Elementals, the artifacts' effects).
- **A property gap from T7c3** (the coverage run's random cases, persisted): good morale for a
  stack its own action's retaliation blinded may end the round, and the new round clears its
  good-morale mark; the property now allows it. The engine was right.
- **Review** (independent): no bug; done: BAT-MAG-063 says that the strike's untested spells
  (Bind, Acid Breath) ask no hero, as the original; the harvested-pack test checks every Pendant,
  the Orb of Inhibition, Recanter's Cloak, the Dragon Father and the Storm Elementals' double
  damage; the Dragon Father test covers a level-0 ability (Poison); a wrong `covers` tag. Left:
  the harvester's `sod` test looks at top-level string parameters only (enough for today's data).
- **The other sources** (the user, after the PR: the wiki on the Orb of Inhibition; BAT-Q21):
  the reading was compared with the 15 artifacts' wiki pages, the Faerie Dragon's and VCMI 1.7.5.
  Most of T8b agrees; five points conflict or rest on the reading alone (the Faerie Dragon under
  the Orb of Inhibition, the Unicorn aura and the absolute immunities under the Orb of
  Vulnerability, the Cloak against creatures, the Dragon Father against level-0 abilities). The
  user: the engine follows the wiki and VCMI where both disagree with the reading (ability
  `ignores_magic_block` on the Faerie Dragon; the negation lifts the aura), the reading on the
  rest, until an experiment batch (2e) settles all five; T8b3 compares the 0x44A4D0 finding with
  them and fixes it next. The process rule ("consensus before trust") went to `shared/` (#47).
- **Measurements:** behaviour unchanged (every golden replay's command log is the same; the
  hashes moved, re-recorded); `random-rollouts-full` 576.7 M Ir (+1.49 %, the same 7 607 steps;
  the new tests in `check_spell` and `affects`, after a first +3.9 % that `MagicLimits` brought
  down), re-baselined; `BattleState` 6 320 bytes (+56); release `h3 bench` about 120 k steps/s.

### Increment 16: T8b3, the per-spell target conditions; batch 2e; the coverage flake (2026-10-10)

- **Start:** #46 (T8b) and #47 merged; CI green on `master` (8fa46be, after the server lane's
  #44); the local full verify with the original data green on 039e50f. The user's cut (BAT-Q22):
  T8b3 (compare with the wiki and VCMI, then fix), experiment batch 2e for BAT-Q21's five points
  and T8b3's, and fixed tests for the coverage lines only the property tests reached.
- **Code reading** (re-read, `installation.md` §7.23): 0x44A4D0's switch (0x44A598, index table
  0x44A92C) reads the creature type record (116 bytes: flags +0x10, maximum damage +0x60), before
  the Orb of Vulnerability's test 0x44A746: Bless 0 for undead and maximum damage 0, Curse the
  same before the Pendant of Holiness, Precision 0 without the shooter flag, Mirth and Sorrow 0
  without morale (`0x20000`), Fortune, Misfortune and Slayer 0 for maximum damage 0. Maximum
  damage 0: the First Aid Tent and the Ammo Cart (the Ballista has 2–3). Also read: the Mighty
  Gorgons' death stare (0x440BC0, case 103) rolls with only the living-flag test, no effect test,
  as the engine (E03); BAT-Q21's list of "level-0 abilities" the Dragon Father stops named it
  loosely (Poison is the one that passes the effect test; the Thunderbirds' bolt tests Lightning
  Bolt, level 2).
- **The other sources** (BAT-Q22): VCMI 1.7.5 `config/spells/timed.json` target conditions; the
  wiki's Bless, Curse, Precision, Mirth, Sorrow, Fortune, Misfortune, Slayer and `War Machine`
  pages and the `Spell Immunity Table` template, which rendered this time (it had not for
  BAT-Q21). Agreed: the undead under Bless and Curse (with the Orb too), Precision's shooters, the
  Tent and the Ammo Cart under Bless and Curse, Mirth's and Sorrow's creatures without the Orb.
  Conflicts: the Ballista under Bless and Curse (VCMI: immune; the wiki with the code), golems and
  elementals under the Orb for Mirth and Sorrow (VCMI: cast, no effect; the wiki's table with the
  code for golems), Fortune, Misfortune and Slayer at the Tent and the Ammo Cart (VCMI: cast, no
  effect; the wiki ambiguous). **The user:** the code on all three until batch 2e. Also found: the
  rendered table keeps the Troglodytes' Blind under the Orb (BAT-Q21 point 3: the wiki with the
  code).
- **Content:** pack format v32: spell key `needs_damage` (`TargetConditions.needs_damage`, the
  creature type's maximum damage above 0); `admits` takes the creature's maximum damage; ruleset
  hash v41. The gap-fill gives Bless and Curse `excludes = ["undead"]` and `needs_damage`,
  Precision `requires = ["shooter"]`, Mirth and Sorrow `excludes = ["no_morale"]`, Fortune,
  Misfortune and Slayer `needs_damage` (sources `exe-spell-conditions`, `vcmi-spell-conditions`,
  `wiki-spell-immunities`); the synthetic Boon and Bane mirror Bless and Curse. The conditions
  are absolute already (`magic::affects`), so the Orb lifts none, and every caster, the strike's
  spells and the drawn spells follow them. BAT-MAG-067.
- **Tests:** h3-content (`admits` by damage), h3-pack (the syntax; `tests/harvested_spell_conditions.rs`
  on the harvested SoD packs: every spell's targets among Pikemen, Archers, Skeletons, Stone
  Golems, Air Elementals, Giants and the three war machines, and under the Orb).
- **Batch 2e** (`experiments/specs/E04b-batch-2e.toml`, staged): OI (Faerie Dragons and Ogre Magi
  under the Orb of Inhibition), OV (Blind at Troglodytes, Mirth and Sorrow at golems and
  elementals, Poison at Gargoyles under the Orb of Vulnerability), UA (the Unicorn aura under it:
  four Expert Slows), RC (Recanter's Cloak against Stone Gaze and the Archangel), DF (Power of the
  Dragon Father against Poison and the Thunderbirds' bolt, the Death Stare as a control), WM
  (the six spells at the war machines). The compiler accepted it as written.
- **The coverage flake** (QUEUE inbox): `cargo llvm-cov -p h3-battle -p h3-heroes` skipping every
  property test found twelve regions only the random cases reached (the obstacle hashes and
  `retain`, `castable_spells` off-turn, the drawn spell's and the luck modifiers' hash, the
  petrified half in Q-0005's stages, the defender's two-hex attack positions, a breath from the
  tail, `screen_centre`'s even rows and two-hex arms, a thunderbolt by chance); each has a fixed
  test now (a test creature `squall`, 50 %). The battle lane's gated crates are clean; h3-kernel
  (shared) and h3-map (adventure) have property tests too and were not checked.
- **Review** (independent, read-only): no bug; every target test goes through `affects`. Done:
  the hash test's `needs_damage` variant, a Chain Lightning case that fails without the tail
  offset (checked by removing it), the luck stat's encoding tested directly, the Orb test's full
  target lists. Left: the harvester has no unit test for timed spells (`needs_damage` is checked
  by the harvest against the original data only).
- **Measurements:** golden replays re-recorded (the ruleset hash; `full-s3`'s commands unchanged,
  its last checkpoint and final hash moved: an Expert Boon skips the synthetic Aid Tent and Supply
  Wagon); `bench-check` within its baseline (no re-baseline); state hash and its layout unchanged.

### Increment 17: T8c, the opening spells, and T8d, Eagle Eye in battle (2026-10-10)

- **Start:** #52 and #53 (increment 16) merged, CI green on `master` (c1288a9; the adventure
  lane's #50 after it); the sandbox's full verify with the original data green on f5b396c. Batch
  2e not played yet (its template on `master` blank). The user's cut (BAT-Q23): T8c and T8d, one
  `cross-lane` pull request; T8e next.
- **Code reading** (`installation.md` §7.24): the setup sets a flag per side (0x463B05); the
  activation 0x465330 does nothing more in the tactics phase, else, while the acting side's
  (the controller's) flag is set, casts Angelic Alliance's Prayer (power 10) and Armor of the
  Damned's Slow, Curse, Weakness, Misfortune (power 50) at mastery 3, each if 0x5A40D0 finds a
  hex the target test accepts, and clears the flag. 0x5A40D0's only callers are these five: the
  address map had named it the creature casters' target check (renamed). The cast routine's
  caster kind 2 takes the side's hero but skips the Orb test, Eagle Eye, the cost, the cast mark
  (0x59FEE4) and the duration bonus (0x5A0098); the mass handler tests and rolls each stack and
  applies the spell with the hero (its specialty). Eagle Eye: 0x59FF52 (kind 0 only) draws
  `random(1, 100) ≤ trunc(p × 100)` for the opposite hero if it has the skill, has not learned
  the spell and its level is at most the skill's + 1, into a unique set per side (0x5A9450);
  p (0x4E4420) is the table value grown for a specialist, plus 0.05/0.10/0.15 for the three
  artifacts, 0 without the skill, at most 1. After the battle (0x469FE0) only the winner's hero
  learns, with a worn Spell Book, spells up to Wisdom + 2.
- **The other sources** (BAT-Q23): the wiki agrees on every opening point (cast "once an allied
  creature is able to take its turn", later after a bad-morale turn; it works in an anti-magic
  garrison; the specialists improve it) and on Eagle Eye's chances, levels, artifacts, specialty,
  the win and Wisdom (its SoD text is the `swh` template's second argument; the summariser
  first reported HotA's). VCMI 1.7.5 casts both sides' opening spells at the end of the tactics
  phase, before round 1, and rolls Eagle Eye after the battle, once per recorded hero cast (the
  same odds), without a book or Wisdom test. **The user:** the code and the wiki on the timing
  until an experiment; the seam as proposed; the learning after the battle left open (BAT-Q24).
- **Seam** (`cross-lane`): `BattleEffect::OpeningSpell { spell, mastery, power }`,
  `BattleMagic.eagle_eye: Option<EagleEye { chance: Rate, spells }>`, `HeroOutcome.learned`
  replacing `spells_cast` (nothing read it); ADR-0014 points 2 and 10 amended.
- **Content:** pack format v33: hero effects `opening_spell` (36) and `eagle_eye` (37); ruleset
  hash v42. The harvester reads Eagle Eye's value table (0x63EA28) with `max_level` the skill's
  level + 1; the gap-fill gives Angelic Alliance and Armor of the Damned their opening spells and
  the three Eagle Eye artifacts `eagle_eye` with `needs_skill` (the last E04b `pending` entries).
  `h3-heroes`: HER-BAT-017 (in the order of the artifact types via `grown_effects`), HER-SPL-004
  (a `RateSum`, so Q-0005's single-precision value; the combat spells not learned up to the
  largest `max_level`). The synthetic pack has `keen_eye`, `dawn_mantle` and `hawk_feather`.
- **Battle:** `magic::opening` at the start of `turn::activation` (after the checks that pass a
  stack over; never in the tactics phase, which has its own activations): a free mass cast of the
  controller's hero per opening spell that passes the target test (hypnotized stacks counted,
  then passed over by the mass cast), an `OpeningSpell` event; `magic::eagle_eye` in the hero
  branch of `apply_cast`, before the cast's draws; the percent `trunc(rate × 100)` at the setup
  (a Basic Eagle Eye specialist at level 2: 43 % with Q-0005). The setup refuses an opening
  spell with a target at its mastery, damage or another non-timed effect, an ability, a mastery
  above 3, power 0 or more than eight, and an Eagle Eye above 1 or listing an ability. State hash
  v41 (`HeroMagic`: the opening spells, their due flag, Eagle Eye's percent, spells and picks; the
  cast spells gone); replay format v21. BAT-MAG-068, BAT-MAG-069; BAT-END-003 reports the picks.
- **Review** (independent, read-only): no bug. Done: the version bumps (ruleset hash v42, replay
  format v21), abilities refused as opening spells, tests for a resisted opening Slow, a
  hypnotized stack's activation, an Enchanter's casts against Eagle Eye, a damaging opening
  spell. Left: the scenarios' heroes have no opening spells or Eagle Eye yet (T9's workload).
- **Measurements:** golden replays re-recorded (state hash v41, ruleset hash v42, replay format
  v21; their commands unchanged); `bench-check` within its baseline (no re-baseline).

### Increment 18: T8e, the data quirks through gated overrides (2026-10-10 to 2026-10-11)

- **Start:** #59 and #60 (increment 17) merged, CI green on `master` (e8e480c); the sandbox's
  full verify with the original data green on 39b52da. Batch 2e not played yet. The user's cut
  (BAT-Q25): T8e alone, the ADR-0007 amendment as a draft first; S-05 classified fix-optional
  (BAT-QUIRK-19). T9 proposed next, in two parts (observation completeness, measurements).
- **ADR:** the draft (battle `adr-drafts/`) accepted as written on 2026-10-11 and moved into
  ADR-0007 by #64 (`shared/`), with one line in ADR-0006 and one sentence in
  `04-rules-and-sources.md` §4: data quirks get `quirk.content.<name>` flags in the registry;
  the pack keeps the original's value; an override with `fixes = "<flag>"` carries the fix and
  applies only where the profile turns the flag off; rule code never reads a `content` flag.
- **Compiler** (`h3-content`): the profile is resolved before the packs; `applies` admits an
  ungated override always and a gated one when its registered `quirk.content` flag is off
  (`CompileError::NotADataQuirk` otherwise). A creature patch goes onto a copy, so a gated
  override's target and references are resolved in every profile, and it is kept (then validated
  with its entity) only where it applies; the new
  `HeroOverride` (`HeroPatch.specialty`) works the same way, after the pack's heroes (its closure
  includes the pack itself). Registry: `quirk.content.roland_crusader_specialty` (17, Q-0006) and
  `quirk.content.elemental_levels` (18, BAT-QUIRK-19), with constants for tests only. The ruleset
  hash's encoding is unchanged (v42): the compiled data and the quirk bits were hashed already.
- **Packs** (format v34, `h3-pack`): `fixes` on `[[creature_override]]`, `[[hero_override]]`
  with `target`, `fixes`, `specialty`. The harvester's gap-fill `data_quirks.base_level` (the
  three elementals: the override gives each its base creature's level from the upgrade pairs,
  refusing a creature that is no upgrade or already has it) and `data_quirks.specialty_creature`
  (Roland: the Swordsman, refusing the record's own creature; every entry must meet a creature
  specialist); both flags in both harvested profiles; the generated packs are compiled in both
  profiles. `ab` gains three gated creature overrides and Roland's gated hero override.
- **Tests:** the compiler's gate in both profiles, every check in `faithful` too, the ungated
  hero override; the pack syntax; the harvester's output and its four refusals; with the
  harvested packs, `the_data_quirks_are_fixed_in_strict_fixed` (levels 4/3/5 against 3/5/4,
  Roland's specialty the Crusader against the Swordsman and his battle effects equal to Sorsha's,
  with and without SoD) and `the_profiles_differ_exactly_by_the_quirk_flags` (19 flags), so E04b
  exit criterion 3's data part holds. Catalog: BAT-MAG-004 and BAT-MAG-020 name BAT-QUIRK-19,
  HER-BAT-008 the fixed Q-0006.
- **Review** (independent, read-only): no bug. Done: the comments and `packs/README.md` say what
  runs in every profile (a gated override's target and references; its values are validated
  with the entity where it applies, and the harvester compiles both profiles); each harvester
  specialty fix must meet exactly one specialist (Lord Haart's name is shared); tests for a
  discarded patch's stats and abilities and for the load order of gated and ungated overrides.
  Left: profile errors now come before content errors (nothing relied on the old order).
- **Measurements:** the golden replays and `bench-check` unchanged (the synthetic packs have no
  data quirks, their ruleset hashes stay); the coverage gate green after `cargo llvm-cov clean`
  (stale profiles had reported doc-comment lines).

### Batch 2e folded in (2026-10-11, increment 18)

- **Played** by the user in SoD with plain HD (`experiments/results/E04b-batch-2e.md`, committed
  from the user's folder). The game cannot be saved during a battle, so the encounters that
  planned in-battle saves (OV's 12, UA's 4 casts, RC's 18 attacks) were replayed from the start
  or cut short (RC: 8 attacks); a trap in `docs/experiments/README.md` now.
- **BAT-Q22** (WM, OV): all three points as the code reads: Bless and Curse at the Ballista
  (shown after the cast), Mirth and Sorrow refused at golems and elementals under the Orb of
  Vulnerability, Fortune, Misfortune and Slayer refused at the Tent and the Ammo Cart. OV's Sorrow
  line lists three answers for four targets (Stone Golems, Air Elementals refused; Giants
  allowed; the Gnolls' missing), which changes nothing.
- **BAT-Q21:** point 1 against the engine: the Orb of Inhibition stops the Faerie Dragons (no
  spell cursor in rounds 1 and 2), as the code reads, against the wiki and VCMI. The gap-fill
  drops the Faerie Dragon's `ignores_magic_block` (the ability stays in the format, no harvested
  creature has it). Point 2 as the engine: under the Orb of Vulnerability Expert Slow slowed all
  three stacks beside the Unicorns in four casts (7 % with the aura's 20 % kept), against the
  reading of 0x5A8090. Points 3–5 as the engine (the code): Troglodytes refuse Blind and
  Gargoyles were never poisoned (12 attacks) under the Orb of Vulnerability; the Cloak bars the
  Archangel's resurrection and, weakly, the Stone Gaze (0 of 8); the Dragon Father, weakly, the
  bolt (0 of 3) and Poison (0 of 4), the Death Stare killing as the control.
- **Docs:** observation sources for BAT-MAG-061, 062, 064, 066, 067; `installation.md` §7.23;
  the answered BAT-Q21 and BAT-Q22; `packs/README.md` (`ignores_magic_block`). Candidates for a
  later batch: the weak points with long rounds instead of reloads, besides increment 17's.

### T9a: the observation's completeness and the spectator viewer (2026-10-11, increment 19)

- **The user (BAT-Q26):** increment 19 = T9a with SRV-Q4's spectator viewer, added without
  breaking `h3-protocol`; per-hex planes for the battlefield; a hero's hidden magic private to its
  side (the agent had recommended the battle-wide limits public); one `cross-lane` pull request.
- **The audit** (every field of `BattleState`, its hero records and side tables, against the
  structured `Observation` and the flat encoding v12): missing from both were the wait phase,
  the clone links, the summons' bound creature, the temporary raises and the corpses' order,
  Bind's binders, Poison's tenths with the start hit points, the hexes a removal cleared (so
  `removable_at` could not be computed), the heroes' cost change and the whole hidden magic
  (resistance, duration bonus, limits, defences, negation, opening spells, Eagle Eye); missing
  only from the flat encoding were the battlefield (natural obstacles, anchors, removable ones,
  every spell obstacle), the terrain, the last side to act, the hero's spell bonuses, the
  masteries of spells outside the book (BAT-QUIRK-5), the start count, the turn-started flag,
  the raised and native traits, the materialised effect values beyond the attack (protections,
  Slayer, Fire Shield, Frenzy, Anti-Magic, Magic Mirror, the statuses: more than the eight
  effect slots can carry) and each effect's mastery. Left out on purpose: the RNG, the
  outcome's bookkeeping (a vanished clone's creatures, a paid surrender's price), the summon
  templates (what a cast would create; its result is public) and the battle-wide `limits`
  (derived from the heroes, private by BAT-Q26).
- **`h3_battle`:** `Observation` gains `spectator`, `wait_phase`, `clone_links`, `summoned`,
  `temporary`, `corpses`, `binders`, `poison`, `start_hp`, `obstacles_cleared` (with
  `Observation::removable_at`, BAT-MAG-034) and `cost_change`; a spell obstacle's `revealed`
  there tells whether the other side sees it now (natives included). `OwnHeroView` gains the
  hidden magic; `Defences` and `Opening` are public. `Viewer { Side, Spectator }`, `observe_as`
  and `project_as`; `observe` and `project` delegate (`h3-protocol` unchanged). A spectator's
  observation is the attacker's seat without either private part, the drawn spell, the cast
  candidates or the obstacles only one side sees; it is told the public events.
- **Encoding v13** (`h3_sim::observation`, 8308 floats, was 2792): 20 globals; 13 public hero
  features each; the viewer's private part once (64 values: the old ten, the spell bonuses, the
  hidden magic, four spell-bonus and eight opening slots; then book, mastery, immunity and Eagle
  Eye vectors of 128); 14 battlefield planes of 255 (one value per hex index, as the action
  encoding's hex targets: blocked, anchor, removable index + 1; per side and kind the rounds
  left, −1 for the whole battle, and the value; the viewer's obstacles the opponent sees); 98
  features per stack (47 values, 27 effect values, eight effect triples with the mastery). A
  side has at most one spell obstacle on a hex (`obstacles::free` and the random draw skip what
  the side sees), so the per-side planes are lossless; Bind's binders take two features of 21
  bits each (exact in `f32`). C ABI v17 (ADR-0011 also records v16, which T8a had left out).
- **Tests:** `tests/observe.rs` (per-stack bookkeeping, the hidden magic to its side only, the
  spectator), observation asserts in the clone, summon and cost tests; the hygiene properties:
  the hidden magic varied in the twin, a spectator unchanged by the RNG, either private part,
  the drawn spell and the obstacles only one side sees, and equal to a side's view stripped of
  its private part; `removable_at` against the state's in every generated game; `h3-sim`'s
  encoding tests per block.

### T9b: the casting workload and the measurements (2026-10-11, increment 20)

- **The user (BAT-Q27):** increment 20 = T9b, then batch 2f staged (increment 17's opening-spell
  candidates, T8b's leftovers, BAT-Q24's learning); batch 2e's weak results decisive unless the
  usual sources agree against them. None do (BAT-Q21's comparison): the engine stays as it is.
- **Workload** (`packs/synthetic/scenarios/full.toml`): the heroes gain the hero-side magic of T8
  that no rollout exercised. Brea: an opening Hasten at Expert (power 3), a Pendant against
  Spark, Basic Eagle Eye (40 %, ten of Corwin's spells up to level 2) and six spells at
  Advanced (Spark, Cinders, Haze, which turns mass, Charm, Scorch, Snare). Corwin: an opening
  Drag and Bane at Expert (power 2), 10 % resistance, a Pendant against Cinders and Advanced
  Eagle Eye (50 %, thirteen of Brea's spells up to level 3). Neither Eagle Eye lists the other
  side's hidden spell (Mire, Snare): its draw comes before the obstacles' own and shows in the
  learner's private part, so the server's hygiene tests would see it once the wire carries the
  hidden magic (the review). Left out on purpose, as each would
  silence much of the workload (unit and property tests cover them): the Orb of Inhibition (no
  casts at all), Recanter's Cloak (no spell above level 2), the Dragon Father (Corwin's own
  spells up to level 4 at his stacks), the Badge of Courage (Brea's Charm and Haze at Corwin's
  stacks), the Orb of Vulnerability (it lifts every resistance in the battle). **Resistance on
  Corwin, not Brea:** on Brea's stacks it draws on Corwin's opening spells, a public roll before
  the first cast, and `h3-protocol`'s hygiene tests (`hidden_cast`) compare the other viewer's
  frames up to that cast across two seeds (a standing trap now, with Eagle Eye's lists).
- **Balance:** Corwin wins almost every random battle: 8 attacker victories in agent seeds 1 to
  2 399 (the old scenario: none in seeds 1 to 199, its `full-s3-a1798` one of the rare ones);
  the workload's armies, not a rule.
- **Tests:** `h3-pack`'s `scenario_heroes` and `h3-sim`'s hero encoding test (each side's
  resistance, opening spells, Eagle Eye's spells by index and Pendant in its own private part)
  follow the scenario; the hidden-magic layout test starts from no hidden magic.
- **Replays:** the five `full-s3-a*` replays get new agent seeds (fixtures README): 1367 (an
  attacker victory), 955 (a redirect), 1766 (a sacrifice, a created stack vanishing, the Fiends'
  raise), 303 (created stacks, Blink, mines) and 2378 (twenty-three mines or pits, the Fiends'
  raise); every full replay has the three opening spells and Eagle Eye's picks. `duel-s1` and `skirmish-s7`
  unchanged. Replay format, state hash, encodings and C ABI unchanged (no engine change).
- **Measurements** (cloud sandbox, 2 vCPU, release, one core; `h3 bench --observe 1` and the
  criterion benches `observe` and `observe + encode` are new):

  | Measure | T9b |
  |---|---|
  | `h3 bench --games 2000` (random legal play) | ≈ 104 k steps/s (three runs 99.7–108.7 k; the old workload on the same binary 100–112 k) |
  | the same with the observation encoded before each step (`--observe 1`) | ≈ 69 k steps/s (69.6 k, 68.9 k): ≈ 4.8 µs per step for the encoding in the loop, more than in isolation |
  | criterion `legal_commands` / `step` / `BattleEnv` step via mask | 3.16 µs / 5.31 µs / 2.40 µs |
  | criterion `observe` / `observe + encode` (8 308 floats) | 0.97 µs / 3.47 µs |
  | `BattleState` size / clone; `BattleEnv` size / fork | 6 464 bytes / ≈ 0.67 µs (≈ 0.10 µs per KB); 6 496 bytes / ≈ 2.25 µs |
  | `state_hash` | ≈ 4.27 µs |
  | `bench-check` `random-rollouts-full` | 571.60 M Ir for 7 388 steps (77.4 k Ir/step; was 576.70 M for 7 607, 75.8 k): +2.1 % per step from the workload alone; re-baselined |

  Exit criterion 5 met, with a thin margin in this sandbox (one run of three below 100 k). **For E05:** the full flat observation costs about a third of a step
  (the planes and stacks are mostly rewritten zeros); the batch API may encode less often or
  only what changed. `state_hash` (4.3 µs at 6.3 KB) suits checkpoints, not per-node hashing in
  a search.
