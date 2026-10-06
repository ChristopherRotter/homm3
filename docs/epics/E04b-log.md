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
