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
