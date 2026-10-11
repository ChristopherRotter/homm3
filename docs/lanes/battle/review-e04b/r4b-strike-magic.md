# Review r4b-strike-magic: BAT-MAG-040 to BAT-MAG-047 (creature magic after a melee strike), BAT-ATK-004 as E04b changed it, quirk Q-0005 (its Poison and Age part)

Method: every rule printed with `rules.py`; `installation.md` §7.20 read; the implementing code read
in full (`after_strike.rs`, `binds.rs`, `vitality.rs`, `effects.rs`, `combat.rs`, `turn.rs`, the parts
of `magic.rs`, `damage.rs`, `raise.rs`, `created.rs`, `obstacles.rs`, `command.rs`, `control.rs`,
`movement.rs` they call), the packs' ability and spell rows and the gap-fill entries
(`crates/h3-harvest/data/gapfill.toml` L1125–L1265). The executable was re-read with
`scripts/exe.py` (addresses per rule below; nothing of §7.20's reading turned out wrong). The pack's
`polarity` and `category = "ability"` were compared with the executable's spell table (0x685450,
type at +0 and flag 0x8 at +0xC) for all 81 spells: no mismatch. `SPTRAITS.TXT` and `CRTRAITS.TXT`
were extracted to the scratchpad and read for the levels and effect values. VCMI 1.7.5 was read for
every rule (none of the eight entries cites it). No batch touches these abilities except batch 2e
(RC, DF, OV: no Stone Gaze, bolt or Poison ever landed, which is what those rows test).
No robustness or security problem found in these rules' code (the bit shifts by stack id are bounded by
`STACK_CAPACITY <= 64`, the damage products fit `u64`, the events are public as in the original).
No cargo run, no file of the repository changed.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-040 | every clause as stated; the summary names no caster of Disease | exe 0x441CB0 (whole), 0x441610 (whole), 0x440500 (whole, with its index table: 22/23 Bind, 24/25 Blind 20, 59 Disease 20, 66/67/141 Curse 25, 69 Age 20, 76/77/106/107 Stone Gaze 20, 81 Paralyze 20, 109 Poison 30, 135 Acid Breath), 0x5A8640, 0x5A8090 (the path of a strike's spell and its tail 0x5A8592), 0x468990 (the pending loop 0x4690B9–0x469130, the deaths 0x4692A4–0x4692ED), 0x59FE30 (0x59FE30–0x5A0354, 0x5A107F), 0x5A3950, 0x59E060, callers of 0x441610, 0x440500, 0x440BC0, 0x5A8640 and every write of +0xEC; spell table 0x685450; SPTRAITS (Blind 50/50/25/0, Curse 0/0/1/1, Weakness 3/3/6/6). VCMI `server/battles/BattleActionProcessor.cpp` L1242–L1297, `config/creatures/*.json` | `tests/strike.rs` L112–L308: chance, shots, killed target, immunities, Anti-Magic, resistance, hypnotized striker, Magic Mirror, the flies | F-5, F-7 |
| BAT-MAG-041 | as stated; "nothing if 0" missing in code and text | exe 0x440BC0 case 93 (0x441185–0x44135B), 0x5A78E0 (head), 0x5A7BB0, 0x44B4B0, 0x44A4D0 (0x44A83A for creatures 113, 125). VCMI `config/creatures/stronghold.json` L266–L277, `config/spells/ability.json` L294–L318 | `strike.rs` L329–L363 (value, creature factors, level immunity, chance) | F-5, F-8 |
| BAT-MAG-042 | as stated; last sentence stale | exe 0x5A81CA–0x5A8231 (the test: Sphere, then any active positive spell 10–80), 0x5A177E–0x5A17D0 (ends 10–69 of positive type), 0x4413C4–0x44148C (flies: no draw, Weakness at mastery 2 unless +0x24C). VCMI `ability.json` L319–L347 | `strike.rs` L275–L308 | ok (F-7 for the stale sentence, F-5 for the uncited comparison) |
| BAT-MAG-043 | as stated ("strikes" includes shots in code and executable) | exe 0x441DED (Stone Gaze read after the strike), 0x441E91–0x441EAD, 0x444090 (ends 62, 70, 74 at every call), 0x444510 (zeroes the duration at once), 0x443F40 (0x44401A: half, melee and ranged), 0x443B90 (0x443CD5–0x443D82: Blind's Advanced effect / 100 as f64, the smaller with Blind's f32), 0x465222–0x465246, 0x44A6F7. VCMI `ability.json` L2–L54, L169–L221; `BattleActionProcessor.cpp` L295–L334; `lib/battle/BattleInfo.cpp` L742–L745; `lib/battle/CUnitState.cpp` L117–L139, L486–L490 | `strike.rs` L408–L457; `limits.rs` L263–L276, L318–L350 | ok for text↔code; F-5, F-6 |
| BAT-MAG-044 | as stated, except "a raise counts its full hit points" for the rebirth | exe 0x444F1E–0x44502C (Poison, Age at a cast), 0x4471F1–0x4472A9 (round start), 0x444740–0x444797 (Age's end), 0x446509–0x446563 (Cure), 0x43D89D, 0x44A713–0x44A734, every use of +0x4A4, 0x444120 (ends every spell before the dead bit), 0x469440 (rebirth), 0x5A7560. VCMI `ability.json` L55–L97, L222–L258; `server/battles/BattleFlowProcessor.cpp` L935–L947 | `strike.rs` L507–L605; `vitality.rs` L139–L176 | F-2, F-5, F-6, F-8 |
| BAT-MAG-045 | as stated | exe 0x440681–0x4406FF, 0x43EE10 (whole), its five callers (0x4420A3 after the path is found, 0x4441C7, 0x4B49E1, 0x4B4EB1, 0x5A1B88 with 0x5A1B91), 0x444565–0x444605, 0x4B34F2–0x4B3515, 0x523A12, 0x523AB2, the fixed 255 (switch at 0x44490D: 47, 59, 72), every use of +0x2B8. VCMI `ability.json` L98–L126; `BattleFlowProcessor.cpp` L905–L933; `lib/spells/effects/Timed.cpp` L235 | `strike.rs` L635–L864 | ok (F-5, F-6 for the expiry, F-7 for the ledger) |
| BAT-MAG-046 | as stated for a living, not defending stack | exe 0x4407EA–0x440817, 0x5A1E70–0x5A1E8D, Defend 0x47974A–0x4798D7, turn start 0x446E5D–0x446E8D, 0x442590, 0x444120 (0x444228–0x444244) with 0x444510's head, 0x5A7560. VCMI `ability.json` L382–L418; `config/creatures/neutral.json` L339–L344 | `strike.rs` L882–L910 | F-1, F-4 |
| BAT-MAG-047 | as stated | exe 0x44149F–0x441586 (target alive, 20 %, one roll of +0xD0..+0xD4 × count, halved toward 0, only above 0, no test, then the pending casts again). VCMI `neutral.json` L333–L338; `BattleActionProcessor.cpp` L1444–L1467 | `strike.rs` L912–L954 | ok for text↔code; F-5 (VCMI: 30 %) |
| BAT-ATK-004 | as stated; the return is not a flight | exe 0x478900–0x478A35 (the return 0x4789F8), 0x445D10, 0x445950, 0x4B49C0, 0x441F70, 0x46A570, 0x46A520. VCMI `BattleActionProcessor.cpp` L337–L351: checked, it says what the entry says (returns if moved, alive, not NOT_ACTIVE, without BIND_EFFECT) | `strike.rs` L781–L808 | F-3 (F-7 for the ledger) |
| Q-0005 (Poison, Age) | both variants in `vitality::hit_points` | second look: the constants' widths at 0x444F24 (f32 0.1), 0x444F3E (f64 0.5), 0x444F81 (f32 0.5), 0x444F92 (f32 0.95); the same code at 0x447205, 0x444753, 0x446526. Confirmed as the ledger describes (9 hit points poisoned once and aged: 4; exact 5). Class fix-optional fits | the function's test pins both variants; no test pins the flag's use in a battle | F-6 |

## Findings

### F-r4b-strike-magic-1: a stack that dies loses Acid Breath's (and Disrupting Ray's) defense loss; in the original it keeps it through its death and a raise
- rules: BAT-MAG-046 (and BAT-MAG-009, BAT-MAG-010, BAT-MAG-016 for Disrupting Ray: other slices)
- kind: behaviour-bug
- severity: medium (a stack must die and be raised; with Disrupting Ray and Resurrection that is ordinary late-game play)
- confidence: high for both readings (code and executable); an experiment would make it certain (Disrupting Ray on a stack, kill it, Resurrection, right-click its defense)
- evidence: `effects::remove_dead` (`crates/h3-battle/src/effects.rs` L927–L945) keeps only the entries of living stacks, permanent ones (`rounds: None`) included, and recomputes; it runs after every command (`command.rs` L478), before a rebirth (`raise.rs` L325) and before a slot is reused (`created.rs` L409). BAT-MAG-046 says "for the rest of the battle ... which no dispel ends". The original: the death routine 0x444120 calls the end routine 0x444510 for every spell whose duration is above 0 (0x444228–0x444244); 0x444510 returns at once for spell 47 (its first test after the duration), so Disrupting Ray is not ended and nothing is given back (`installation.md` §7.9 says so itself: "Disrupting Ray is never ended"); Acid Breath (0x5A1E70) only lowers the defense field +0xCC and stores no duration or value, so there is nothing to end; the raise 0x5A7560 sets the count, the lost hit points and the temporary counter, removes the corpse and clears the dead bit; neither it nor its callee 0x43EA70 writes +0xCC (the executable's only writes of a stack's +0xCC are the setup 0x43D81E, the end routine, the apply routine, the turn start, Defend, Disrupting Ray and Acid Breath). Scenario: Rust Dragons strike 10 serfs (defense 10) once: 7. The serfs die. Resurrection (or an Archangel, or a rebirth) brings them back. Original: defense 7. Engine: 10.
- fix or test sketch (`needs-run`; the kill as `tests/raise.rs` L29–L35 does it, the raise as `tests/strike.rs` L551):
  ```rust
  let mut s = duel(&r, "corroder", "serf", 1);
  hit(&mut s, &r, 1, (7, 5));
  assert_eq!(s.at(StackId(1)).defense_now(), 7);
  // The serfs die; the end of the command drops a dead stack's effects.
  let total = s.at(StackId(1)).total_hp();
  crate::damage::apply_damage(s.at_mut(StackId(1)), total);
  crate::effects::damaged(&mut s, &r, StackId(1), &mut Vec::<Event>::new());
  crate::effects::remove_dead(&mut s, &r);
  crate::raise::mark_corpses(&mut s);
  crate::raise::raise(&mut s, &r, (StackId(1), 20), false, &mut Vec::<Event>::new());
  assert_eq!(s.at(StackId(1)).defense_now(), 7);              // the engine: 10
  ```
  Fix: `remove_dead` keeps permanent entries (what the end routine refuses to end), as `dispel_ends` already does.

### F-r4b-strike-magic-2: a Phoenix that dies aged is reborn with about half its creatures
- rules: BAT-MAG-044 ("a stack that dies loses it ... so a raise counts its full hit points per creature"), BAT-MAG-059
- kind: behaviour-bug
- severity: medium (Ghost Dragons against Phoenixes; Age lasts 3 rounds)
- confidence: high from the code; `needs-run`
- evidence: `raise::rebirths` reads `health = stack.max_hp` at `crates/h3-battle/src/raise.rs` L313, then calls `effects::remove_dead` at L325 (which ends Age and restores `max_hp`), then `revive(.., creatures * health)` at L326. With Age active at the death `health` is the halved value, so the hit points given raise half the creatures at the restored value. The original: the rebirth check 0x469440 needs the dead bit, so the death routine has run and Age's end (0x444740) has recomputed +0xC0; it then raises `+0xC0 × creatures` (0x46959E–0x4695AD). Scenario (the synthetic pack): 10 phoenixes of 10 hit points, aged (5), die. Original: 10 div 5 = 2 creatures. Engine: 2 × 5 = 10 hit points at 10 per creature = 1 creature. Poison alone does not differ (its tenths stay after the death on both sides).
- fix or test sketch (`needs-run`, after `a_reborn_phoenix_comes_back_without_the_effects_it_died_with`, `tests/rebirth.rs` L216):
  ```rust
  let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "phoenix", 10)], 1);
  let id = spell(&r, "wither");
  let cast = effects::TimedCast { id, spell: r.spell(id).unwrap(), mastery: 0, rounds: 3,
                                  own: crate::magic::SpellBonus::default() };
  effects::apply(&mut s, &r, cast, StackId(1), &mut Vec::<Event>::new());
  assert_eq!(s.stacks()[1].max_hp, 5);
  kill(&mut s, &r, 1);
  let mut events = Vec::new();
  rebirths(&mut s, &r, &mut events);
  assert_eq!(reborn(&events), vec![(1, 2)]);                  // the engine: (1, 1)
  ```
  Fix: read `max_hp` after `remove_dead`.

### F-r4b-strike-magic-3: the Harpies' return does not enter the hexes it lands on, so a Fire Wall on their start hex does not burn them
- rules: BAT-ATK-004, BAT-MAG-028
- kind: behaviour-bug (the rule text has the same gap: "the return is a move event and costs nothing")
- severity: medium (rare: the Harpies must start their turn standing in a Fire Wall)
- confidence: high for the executable's path; medium that nothing else intervenes; `needs-run` for the engine's side
- evidence: `combat::melee` sets `state.at_mut(attacker).hex = start` and emits `Moved` (`crates/h3-battle/src/combat.rs` L755–L769) without `obstacles::travel`; after an attack with a move `command.rs` L423–L425 calls nothing more. The original: the return at 0x4789F8 calls the move handler (0x445D10 → 0x445950), which marks the hexes the stack now stands on as the move's start (0x46A520, called at 0x4459B2) and, for a flyer, runs the flight 0x4B49C0; the flight ends with the hex trigger (0x4B4A1E → 0x441F70 → 0x46A570: Fire Wall 0x59FCD0, Land Mine, moat) for the hexes landed on. BAT-MAG-028 itself says a flyer is burnt "only on the hexes it lands on". Scenario: a Fire Wall covers (6, 5); Harpies that stand on it attack a stack from (8, 5) and return. Original: burnt on landing. Engine: not burnt.
- fix or test sketch (`needs-run`, with `burning` and `burns` of `tests/obstacles.rs` L46–L285):
  ```rust
  let mut s = burning(&r, &[("harpy", 20, 6, 5)], &[("fixed", 20, 9, 5)]);
  let attack = Command::Attack { target: StackId(1), from: at(&r, 8, 5) };
  let events = act(&mut s, &r, attack).unwrap();
  assert_eq!(s.stacks()[0].hex, at(&r, 6, 5));
  assert_eq!(burns(&events).len(), 1);                        // the engine: 0
  ```
  Fix: return through `obstacles::travel` (it also releases the stacks a returning binder binds, as the original's flight does at 0x4B49E1; no harvested creature has both abilities).

### F-r4b-strike-magic-4: the original keeps the Defend bonus inside the defense field, so Acid Breath tests and lowers defense + bonus; the engine tests and lowers the defense without it and recomputes the bonus
- rules: BAT-MAG-046; the same mechanism reaches every defense change on a defending stack (BAT-MAG-009, BAT-MAG-011, BAT-TRN-004: other slices)
- kind: behaviour-bug
- severity: medium (one defense point; for Acid Breath a defending target is needed; for a hero's Stone Skin, Prayer or Disrupting Ray on a defending stack it is normal play)
- confidence: high for the executable; high for the engine's code; an experiment would show it (defend with a stack of defense 10, cast Stone Skin +6 on it, right-click: 18 by the code, the engine has 19)
- evidence: the Defend action computes `max(1, defense × 20 / 100)` from +0xCC once, stores it at +0x4DC and adds it to +0xCC (0x47974A–0x4798D7); the stack's turn start subtracts +0x4DC again while the defending bit is set (0x446E5D–0x446E8D); the damage reads +0xCC as it is (0x442590). Acid Breath's test reads +0xCC > 0 (0x4407F8) and its handler lowers +0xCC by 3 to at least 0 (0x5A1E70–0x5A1E8D). The engine: `effective_defense` (`crates/h3-battle/src/damage.rs` L295–L302) computes the bonus from the current `defense_now()` at every strike; `spell_after_strike` tests `stack.defense_now() <= 0` without the bonus (`after_strike.rs` L139–L144); `effects::deltas` floors the defense without it (`effects.rs` L495–L499). Scenarios: (a) defense 12, defending (+2 = 14), struck by Rust Dragons: original 11 until its next turn, then 9; engine 9 + 1 = 10 until then, then 9. (b) defense 0, defending (+1): original: the breath lands (1 → 0) and the next turn start leaves −1; engine: no breath, 1 while defending, then 0. (c) defense 10, defending (+2), Stone Skin +6: original 18, engine 16 + 3 = 19.
- fix or test sketch (`needs-run`):
  ```rust
  let mut s = duel(&r, "corroder", "peasant", 1);             // defense 1
  activate(&mut s, 0); hit(&mut s, &r, 1, (7, 5));            // 1 -> 0
  activate(&mut s, 1); act(&mut s, &r, Command::Defend).unwrap();
  activate(&mut s, 0);
  let events = hit(&mut s, &r, 1, (7, 5));
  assert_eq!(strike_spells(&r, &events).len(), 1);            // the original casts it; the engine: 0
  ```
  Fix: store the bonus at Defend (a stack field or a pseudo-effect that ends at the next activation) and let the defense changes see it.

### F-r4b-strike-magic-5: all eight rules rest on the code reading alone; VCMI contradicts it on ten points with gameplay effect and no open question records them
- rules: BAT-MAG-040 to BAT-MAG-047
- kind: source
- severity: medium (if the other sources are right, chances and effects are wrong in normal play; otherwise docs only)
- confidence: high that the comparison is missing and that VCMI says what is listed; my re-read confirms the executable on every point, so each is a conflict of sources, not a reading error
- evidence: each entry has one source, `original-data`, the reading of §7.20; `docs/experiments/README.md` L113–L114 says of T7a1 "the wiki was not consulted". `docs/04-rules-and-sources.md` §2, "Consensus before trust" (2026-10-10, two days after T7a), asks for the comparison with the wiki and VCMI, their citation, and for a contradiction an entry in `OPEN-QUESTIONS.md` with an experiment and the user's decision ("the reading's side is never implemented alone"). The battle lane has no open question; the points are only experiment candidates, none staged (README L113–L126). VCMI 1.7.5 against the executable:

  | point | executable (re-read) | VCMI 1.7.5 |
  |---|---|---|
  | Curse of Black and Dread Knights | 25 % (0x440711) | 20 % (`config/creatures/necropolis.json` L376–L381, L411–L416) |
  | Curse of Mummies | 25 % (the same case, creature 141) | 50 % (`neutral.json` L591–L596) |
  | the Rust Dragon's extra damage | 20 % (0x4414BC) | 30 % (`neutral.json` L333–L338); the amount agrees (25 per dragon) |
  | the Unicorns' Blind | mastery 0: the struck stack later retaliates at 50 % (0x4690F8–0x469102) | level 3: no retaliation (`rampart.json` L313–L319, L350–L356; `config/spells/timed.json` L1438–L1450) |
  | the Dragon Flies' Weakness | mastery 2: −6 (0x44147C–0x441487) | level 0: −3 (`fortress.json` L165–L170) |
  | the Thunderbirds' bolt | tested and dealt as Lightning Bolt, spell 17 (0x4411D4, 0x4411F5): Earth and Magma Elementals immune, the dragons' level immunity, Anti-Magic, Protection from Air | its own spell 77 of level 0 without a school (`stronghold.json` L266–L277, `ability.json` L294–L318): none of these |
  | Stone Gaze and magic resistance | type −1 in the spell table (0x687980), so 0x5A8640's draw with resistance and aura | `indifferent`, so no resistance roll (`ability.json` L46–L48; `lib/spells/BattleSpellMechanics.cpp` L455–L462) |
  | Poison | a tenth at the cast and at each round start, floor at half, kept after it ends until a Cure | −10 % per own turn to −30 %, given back when it ends (`ability.json` L55–L97; `BattleFlowProcessor.cpp` L935–L947) |
  | Age's rounding | trunc(H / 2 + 0.95): 25 → 13 | −50 %, by integer percent (25 → 12, not run) |
  | Bind's end | at once when the last binder moves (also to another adjacent hex) or dies; 255 rounds | at the bound stack's turn start if no binder is adjacent; permanent (`BattleFlowProcessor.cpp` L905–L933) |

  A smaller one: VCMI rolls magic resistance for Dispel Helpful Spells (a negative, magical spell), the executable does not. VCMI agrees, and could be cited, on: every other chance (Unicorns, Medusas, Basilisks, Scorpicores, Zombies, Ghost Dragons, Thunderbirds 20 %, Wyvern Monarchs 30 %, the flies, Dendroids and the Rust Dragon's defense loss 100 %); 10 damage per Thunderbird; 3 rounds for a creature's cast (`lib/battle/CUnitState.cpp` L444–L450); the cast after every melee strike, retaliations included, on a living target, immunity before the chance, no Magic Mirror (`BattleActionProcessor.cpp` L1242–L1297, `BattleSpellMechanics.cpp` L520); Medusas only in melee; the Troglodytes' immunity; a petrified stack takes half and retaliates in full once the strike has ended its petrification, a paralysed one at 25 % (the bonuses end at the damage, `BattleInfo.cpp` L742–L745, and the retaliation is tested after the strike, `BattleActionProcessor.cpp` L323–L331); no second strike for an attacker that lost its turns (L307); Acid Breath −3, permanent, cumulative, ignoring immunity; Dispel Helpful Spells ending positive effects only and ignoring immunity.
- fix or test sketch: add the VCMI (and wiki) sources to the eight entries; one `OPEN-QUESTIONS.md` entry for the ten points with an experiment batch (the candidates are already listed in `docs/experiments/README.md`), and the user's decision for each until then.

### F-r4b-strike-magic-6: test gaps
- rules: BAT-MAG-044 (Q-0005), BAT-MAG-043, BAT-MAG-045
- kind: test-gap
- severity: low
- confidence: high
- evidence: (a) Q-0005's two variants are pinned only on the function (`vitality.rs` L153–L175); `vitality::materialize` reads the flag at L88 and no test runs Poison or Age in a battle with `t:single` and a value where the variants differ (every such test uses 10 hit points), so a wrong or constant flag there passes. (b) A shot at a petrified stack is halved too (`combat.rs` L288 sets `petrified` for every strike; the original's 0x44401A is outside the melee/ranged branch); only melee is tested (`strike.rs` L410–L427). (c) Bind's end after its rounds (the stack free, its binders forgotten) is not tested (`strike.rs` L662–L664 counts one round).
- fix or test sketch: (a) a 9-hit-point living creature in the synthetic pack, poisoned once and aged by two strikes, `max_hp` 5 with the default profile and 4 with `t:single`.

### F-r4b-strike-magic-7: rule texts and the ledger
- rules: BAT-MAG-040, BAT-MAG-041, BAT-MAG-042, BAT-MAG-043, BAT-MAG-045, BAT-ATK-004
- kind: rule-text
- severity: low
- confidence: high
- evidence: (a) No rule names the Zombies' Disease (20 %, living creatures only, 3 rounds; `packs/base/pack.toml` L1444, the spell at L4041–L4053): BAT-MAG-040 lists Blind, Curse, Dispel Helpful Spells and Weakness, BAT-MAG-043 to 046 the others, and "Disease" appears in the catalog only in Cure's list and in BAT-MAG-064; the epic says "Disease on BAT-MAG-009", whose summary does not mention it. (b) BAT-MAG-042 ends with "The Sphere of Permanence's protection joins with the artifacts (E04b T8)": it is implemented (`after_strike.rs` L147–L154, BAT-MAG-063). (c) BAT-MAG-043 says "halves the damage of strikes": the code and the executable halve shots too; the text should say so. (d) BAT-MAG-041 does not say what a bolt of 0 damage does (F-8). (e) Two bugs of the original that the engine deliberately does not reproduce have no entry in `docs/rules/quirks-ledger.md`, unlike BAT-QUIRK-12 or BAT-QUIRK-16: the bound Harpy that flies back and stays bound for 255 rounds (survey S-14, BAT-Q9; BAT-ATK-004 has `quirks = []`) and the binder listed on a stack its strike killed (BAT-Q8). `docs/04` §4 asks for an entry for every known difference.

### F-r4b-strike-magic-8: two event differences
- rules: BAT-MAG-041, BAT-MAG-044
- kind: behaviour-bug
- severity: low (events only)
- confidence: high from the code
- evidence: (a) A bolt whose damage comes to 0 does nothing in the original (0x4411FD–0x441202 leaves when 0x5A78E0's result is not above 0: no damage call, no report); `after_strike::thunderbolt` (`after_strike.rs` L219–L251) emits `StrikeSpell` and `SpellDamage` with damage 0 and runs `effects::damaged`. Reachable: one Thunderbird against Diamond Golems (10 div 20 = 0, 0x44B570). (b) `effects::count_down` compares the hit points after Poison's tenth with `max_hp` taken before the stack was recomputed (`effects.rs` L874–L878); when Age ends at the same round start the value before is the aged one, the new one is higher and no `Poisoned` event is emitted although Poison took a tenth. The original ends Age first (the countdown loop) and compares with the recomputed value (0x44720B, 0x4472A2), so it reports.
- fix or test sketch: (a) return before the events when the damage is 0, as `extra_damage` does; (b) recompute before taking `before`.

## Wiki checks wanted

1. Black Knight, Dread Knight, Mummy: the chance of Curse (the executable: 25 % for all three; VCMI 20 %, 20 %, 50 %).
2. Rust Dragon: the chance of the extra acid damage (executable 20 %, VCMI 30 %) and whether the defense loss survives the target's death and resurrection.
3. Unicorn: the level of its Blind (does a stack blinded by Unicorns retaliate at half when attacked, or not at all?).
4. Dragon Fly: the level of its Weakness (−3 or −6).
5. Thunderbird: is the bolt treated as Lightning Bolt (Earth and Magma Elementals, Green, Red, Gold and Azure Dragons immune; Protection from Air; Anti-Magic)?
6. Stone Gaze: may Dwarves or a hero's Resistance resist it; does a petrified stack retaliate against the attack that frees it; is a shot at it halved?
7. Paralyze: the retaliation at 25 %.
8. Poison: a tenth per round or per turn, the floor (50 %?), whether the hit points stay lowered after it ends, what Cure and Dispel do.
9. Age: the rounding of the halved hit points (25 → 13 or 12).
10. Bind: what frees the stack (the binder's move to another adjacent hex, its death, Teleport, Dispel).
11. Defend: is the bonus fixed when the stack defends (a Stone Skin cast afterwards does not raise it)?
12. Harpy: is it burnt when it returns onto a Fire Wall?
13. BAT-ATK-004's two wiki quotations (not checkable here).

## Not reviewed, left open

- 0x44A4D0's resistance arithmetic and the hero's resistance getter (BAT-MAG-008), 0x44B4B0 beyond the cases named, the Orb of Vulnerability, the Pendants and Power of the Dragon Father on a strike's spell (BAT-MAG-063 to 066): read only as far as the strike's test passes through them.
- Cursed Ground (0x5A80D1: no spell above level 1) and Magic Plains (0x59FEBC: mastery 3 for a creature's cast) for these abilities: E04c.
- The Defend bonus of a war machine and the turn start's other work (0x446E30 beyond its Defend part); the computer player's paths (0x41E…–0x43C…); the message routine 0x5A8950.
- No test was run; four sketches are marked `needs-run`.
- F-4's wider reach (Stone Skin, Prayer, Disrupting Ray, Disease on a defending stack) and F-1's Disrupting Ray belong to other slices; I traced the code for both but wrote no test for them.
