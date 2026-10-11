# Review r6-limits-opening: BAT-MAG-061..069, HER-SPL-004, HER-BAT-015..017, Q-0006, BAT-QUIRK-19, exit criterion 3

Method: every rule read against its implementing functions (followed to the callees), against
`installation.md` §7.23 and §7.24, and against my own re-read of the executable. VCMI 1.7.5
(`git describe`: 1.7.5) opened for every "VCMI" source of the slice. Batches 2e and 2f read
whole. No cargo run, no file changed. Test sketches are marked `needs-run`.

Executable re-read (addresses, all through `scripts/exe.py`): 0x5A8090 whole; 0x44A4D0 whole with
both switches (0x44A598, 0x44A7E7); 0x41F890 whole and the gates of its callers 0x4476C0
(0x4476FE, 0x4477A4), 0x447FE0 (0x447FFB), 0x59FE30 (0x59FEEF); 0x59E900 whole; 0x59FE30 from its
start to 0x5A0350 (caster kinds, 0x59FF52 Eagle Eye, cost, 0x5A0098 duration, 0x5A02CB/0x5A02EA)
and the timed case 0x5A107F–0x5A10EC; 0x5A66D0 whole; 0x5A8640 whole; 0x440500 (first cases, its
switch, every tested spell); the three test sites of 0x440BC0; 0x465330 up to the creature cases;
0x465080 whole; 0x5A40D0 whole; 0x5A39C0 (stack branch); 0x5A3950; 0x5A3CC0 whole; 0x4E4420 whole
with the table 0x63EA28; 0x4D91F0 whole; 0x469FE0 whole; 0x442690; 0x43EA70 whole; 0x441FA0 (start
and end); 0x4448F0 (0x444D9C–0x444EA9); 0x463B05; every push or compare of the constant 93 in the
image. Tables: creature table 0x6703B8 records 30, 112–115, 123, 125, 127, 129; specialty table
0x678420 records 146, 152; `SPTRAITS.TXT` levels (read from `H3bitmap.lod` into my scratchpad).

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-061 | every clause as stated (`magic.rs:618`, `casters.rs:305`, `casters.rs:382`) | exe 0x41F890 (both heroes, artifact 126), its callers' gates, 0x59E900 (already-cast before the Orb); VCMI `CBattleInfoCallback.cpp` L115–158, `artifacts.json` orbOfInhibition; 2e OI row as cited | `limits.rs` (5 tests), `harvested_defences.rs` | ok (F-8: two stale comments) |
| BAT-MAG-062 | as stated (`magic.rs:655`, `:764`, `raise.rs:126`) | exe 0x5A80F6–0x5A8135 (level > 2, either hero, artifact 83), 0x59EB5A (book: > 2), 0x5A3CC0 (Archangel through 0x5A8090 with spell 38), 0x440500 call sites; VCMI `BattleSpellMechanics.cpp` L217–222; 2e RC row (8 attacks, 17 %) | `limits.rs` (boundary: level 2 passes, 3 barred) | ok |
| BAT-MAG-063 | as stated (`magic.rs:710`, `after_strike.rs:149`) | exe 0x5A8144, 0x5A81CF (Sphere, controlling hero), 0x5A83D1 (Blade), 0x44A598 cases (Pendants 100–107, each spell); VCMI `artifacts.json`, `TargetCondition.cpp`; 2f PS, HP rows as cited | `hero_defences.rs` | F-3 (the VCMI source's "agrees") |
| BAT-MAG-064 | as stated (`magic.rs:720`) | exe 0x44A50F–0x44A52C (level ≤ 4 signed, level 0 included, no polarity test), Dispel and 78 return in 0x5A8090 before it; `SPTRAITS.TXT` levels; VCMI `TargetCondition.cpp` L122–153, L217–234; 2e DF, 2f DF rows | `hero_defences.rs` (boundary `<=` pinned with level 1; level 0 Poison) | F-3, F-4 |
| BAT-MAG-065 | as stated (`magic.rs:724`) | exe 0x44A77E–0x44A7A5 (after the Orb test; artifact 49 on the controlling hero); VCMI `artifacts.json` badgeOfCourage | `hero_defences.rs` | ok |
| BAT-MAG-066 | as stated; the aura lifted by decision against the reading | exe 0x44A746–0x44A775, 0x44A8AE, 0x44A7B7–0x44A80D, 0x5A8592–0x5A85CF, 0x43EA70, 0x5A66D0, 0x44B4B0 (no artifact test: golems, elementals stay); VCMI `TargetCondition.cpp` L306–330, L453–464, `artifacts.json` L1358; 2e OV, UA rows | `hero_defences.rs`, `mines.rs`, `harvested_defences.rs`, `harvested_spell_conditions.rs` | F-2, F-3, F-7 (and F-1 beside it) |
| BAT-MAG-067 | as stated (pack conditions, `magic.rs:733`) | exe cases 0x44A663, 0x44A644, 0x44A62C, 0x44A6E0, 0x44A66B (creature record +0x10, +0x60), all before 0x44A746; VCMI `config/spells/timed.json`; 2e WM, OV rows | `harvested_spell_conditions.rs` reproduces WM and OV | ok |
| BAT-MAG-068 | as stated (`magic.rs:931`, `turn.rs:265`, `hero.rs:379`) | exe 0x465330 (tactics exit, flag, 129 then 132, spells 48; 54, 42, 45, 52; mastery 3, power 10 / 50), 0x465080 (activation after 0x465222 and 0x465283), 0x5A40D0, 0x5A39C0, 0x59FE30 kind 2 (no Orb test, no Eagle Eye, no cost, no cast mark, no duration bonus), 0x5A66D0, 0x463B05; VCMI `BattleFlowProcessor.cpp` L198–249; 2f AD, AO rows | `opening.rs` (9 tests) | ok; F-6 (two untested clauses) |
| BAT-MAG-069 | as stated (`magic.rs:444`, `:991`, `:2222`) | exe 0x59FF52–0x59FFC4 (kind 0 only, opposite hero, skill > 0, learned +0x3EA, level ≤ skill + 1, draw ≤ trunc(p × 100)), 0x4E4420, 0x469FE0; VCMI `BattleResultProcessor.cpp` L466–484, `skills.json`; 2f EL, EB, EW rows | `eagle.rs` | ok; F-5 |
| HER-SPL-004 | as stated (`derive.rs:153`) | exe 0x4E4420 (f32 table, specialty type 0 subtype 11, f32(level) × 0.05f + 1, artifacts 63, 64, 65 in this order, each sum stored f32, cap 1) | `tests/derive.rs:1267` | ok; F-5 |
| HER-BAT-015 | as stated (`derive.rs:287`) | exe 0x4D91F0 (19 positions, then the combination holding it), 0x41F890, 0x59E900, 0x5A80F6 | `tests/derive.rs:341`, `harvested_defences.rs` | ok |
| HER-BAT-016 | as stated (`derive.rs:306`) | exe 0x5A8090 and 0x44A4D0 artifact ids 92, 128, 134, 100–107, 49, 93 | `tests/derive.rs:390`, `harvested_defences.rs` | ok |
| HER-BAT-017 | as stated (`derive.rs:184`) | exe 0x4653DA–0x4654DA; VCMI `artifacts.json` | `tests/derive.rs:1267`, `h3-pack/src/lib.rs:4491` | ok |
| Q-0006 | both variants in data (`packs/ab/pack.toml:785`), class fits | exe specialty table: hero 152 kind 1 creature 7, hero 146 creature 6; batch 1a B4–B6 rows (Roland 12/13/5 and 15/14/7) | `the_data_quirks_are_fixed_in_strict_fixed` asserts both variants; `harvested_hero_effects_reproduce_the_experiments` | ok |
| BAT-QUIRK-19 | both variants in data (`packs/ab/pack.toml:535–547`), class fits | exe creature table, 0-based levels: Water 2, Earth 4, Fire 3; Ice 3, Magma 2, Energy 4, Storm 1 (as the ledger says) | `the_data_quirks_are_fixed_in_strict_fixed` (with and without SoD) | ok |
| exit criterion 3 | registry 23 flags = 19 in `quirks-ledger.md` + 4 in `quirks-adventure.md`, both directions; every test a ledger entry names exists; profiles hold only quirk flags; no profile-name logic; S-14 (`combat.rs:750–760`, no flag) and S-20 (`state.rs:1203–1212`, no flag) fixed in every profile with tests | — | `the_profiles_differ_exactly_by_the_quirk_flags` | ok; F-9 (wording) |

## Findings

### F-r6-limits-opening-1: The Unicorn aura reaches a stack by its real side in the original, by its controller in the engine (outside my slice: BAT-MAG-008, BAT-MAG-026)
- rules: BAT-MAG-008, BAT-MAG-026 (found while re-reading the aura for BAT-MAG-066)
- kind: behaviour-bug
- severity: medium (a hypnotized stack beside Unicorns)
- confidence: high on the reading (two symmetric compares, the only code that fills the lists);
  medium that play shows it (no observation). An experiment would raise it (below).
- evidence: 0x43EA70 is the only function that inserts into the aura lists +0x514/+0x524 (its four
  calls of the insert 0x423130; the other users at 0x43EC50, 0x442010, 0x44413C, 0x44460A and
  0x444DA5 only remove through 0x43EA00 or erase, 0x437A00 copies a record). In both halves it takes
  the Unicorn's side through the hypnosis flip (`+0x288`: 0x43EAEE and 0x43EB9C) and compares it
  with the other stack's `+0xF4` unflipped (0x43EB0D, 0x43EBC1): `controller(Unicorn) ==
  real side(protected stack)`. 0x5A8090 reads the protected stack's list +0x524 (0x5A8599).
  `installation.md` §7.14 (line 784) and BAT-MAG-026 say "of the same controller". The engine:
  `magic.rs:1932–1955` compares `n.controller() == stack.controller()`.
  Scenario: A's Pikemen stand beside A's Unicorns; B hypnotizes them; A's hero then casts Fireball
  (neutral) on them, or A's Black Knights strike them (Curse). Original: the Pikemen's list still
  holds A's Unicorns (rebuilt at the hypnosis, 0x444EA9, with the same result), so × 0.8: 20 %
  resist. Engine: no aura (the Unicorn's controller A ≠ the Pikemen's controller B). The reverse:
  B's Unicorns beside the hypnotized Pikemen give no aura in the original and one in the engine.
  A hypnotized Unicorn is the same in both (it protects the stacks owned by its controller's side).
- fix or test sketch: in `aura` compare `n.controller() == stack.side`; correct BAT-MAG-026's and
  BAT-MAG-008's "same controller" and §7.14. Test (`needs-run`, in `tests/spells.rs`): arena with
  the defender's `fixed` (id 1) beside its `unicorn` (id 2); `affect_chance(.., bolt, StackId(1))`
  is 80; give id 1 `hypnotize`; it must stay 80 (today 100). Experiment: the scenario above with
  20 Fireball rolls.

### F-r6-limits-opening-2: The aura under the Orb of Vulnerability: my re-read confirms the reading the engine departs from; the observation behind the departure is 7 % and has no control
- rules: BAT-MAG-066 (BAT-Q21 point 2, BAT-Q27)
- kind: question
- severity: medium (Unicorns against an Orb of Vulnerability)
- confidence: medium that the engine differs from the original (about even); a re-test settles it.
- evidence: 0x5A8592–0x5A85CF: for a non-positive spell at a stack with a non-empty list +0x524 the
  result of 0x44A4D0 is multiplied by 0.8f with no artifact test; 0x44A4D0 returns 1 under the Orb
  (0x44A746); the mass handler 0x5A66D0 (the path of Expert Slow: 0x5A10C2) rolls `random(1, 100)`
  against trunc of that, 80. The image tests artifact 93 in two places only, both in 0x44A4D0
  (every other push, load or compare of the constant 93 is a screen coordinate or picture index,
  a map object type, an obstacle value or text parsing).
  0x43EA70 fills the list without an artifact test, at the walk's end (0x4423BE), so UA's three
  walked stacks had it. So the code keeps the aura on every path I found. Against it: batch 2e UA,
  12 of 12 rolls slowed, which has a 0.8^12 = 6.9 % chance under the reading, with no control run
  (the same battle without the Orb, to show that the set-up produces resists at all). The
  catalog labels this observation `confidence = "high"` for "OV, UA" together. The user decided
  (BAT-Q27) that the weak result stands with the wiki and VCMI; I do not dispute the decision,
  only its footing: the wiki and VCMI may share one origin, and the reading has now been done
  twice independently.
- fix or test sketch: nothing in code. Split the observation's confidence (OV high, UA low) and
  queue a decisive re-test: UA with a control battle without the Orb, and at least 30 rolls
  beside the Unicorns (0.8^30 = 0.1 %).

### F-r6-limits-opening-3: VCMI lets the Orb of Vulnerability lift the Pendants', Armageddon's Blade's and the Dragon Father's immunities; the catalog says VCMI agrees
- rules: BAT-MAG-063, BAT-MAG-064, BAT-MAG-066 (BAT-Q21, "agreed by all")
- kind: source
- severity: low
- confidence: high (read in VCMI 1.7.5)
- evidence: `lib/spells/TargetCondition.cpp`: `AbsoluteSpellCondition` (L157–173) and
  `AbsoluteLevelCondition` (L122–153) count only bonuses whose `addInfo` is 1;
  `TargetCondition::isReceptive` (L453–464) returns true once a negation item is receptive
  (`ImmunityNegationCondition`, L306–330), before the normal conditions (`NormalSpellCondition`,
  `NormalLevelCondition`). In `config/artifacts.json` only the Sphere of Permanence carries
  `"addInfo": 1` (L1347, L1352); the Pendants (e.g. L1544), Armageddon's Blade (L1872) and Power of
  the Dragon Father (L2079–2083) are plain bonuses, exactly like the Troglodytes' Blind, which the
  catalog does list as a VCMI disagreement. Scenario: A wears the Orb, B the Pendant of
  Negativity, A casts Lightning Bolt at B's stack. Code reading (0x44A5F6 before 0x44A746) and the
  engine (`magic.rs:710`): no target. VCMI: it hits. BAT-MAG-063's VCMI source says "(agrees)" at
  confidence high, BAT-MAG-064's and BAT-MAG-066's name other differences but not this one, and
  BAT-Q21 lists "the artifacts' immunities ... it keeps" under "agreed by all". The clause "no
  negation lifts it" therefore rests on the reading (confirmed by my re-read) and the wiki's
  sentence as quoted in the catalog; no batch observed it.
- fix or test sketch: correct the three source notes and BAT-Q21's list; add one cursor check to
  the next batch (the scenario above, and Slow from an Orb's owner at stacks under the Dragon
  Father).

### F-r6-limits-opening-4: The Thunderbirds' bolt is no level-0 ability: batch 2e DF's evidence for the Dragon Father against level 0 is 4 Poison attacks (24 %), not "12 %"
- rules: BAT-MAG-064 (BAT-Q21 point 5)
- kind: source
- severity: low
- confidence: high
- evidence: the strike's bolt is tested and dealt as Lightning Bolt, spell 17, level 2 (the case
  at 0x441185: 20 %, then 0x4411D4 passes 17 to the strike test and 0x4411F5 to the damage
  modifiers, 10 per creature; spell 77 does not appear in it), and the pack agrees
  (`packs/base/pack.toml:2197`, `spell = "base:lightning_bolt"`). BAT-Q21 point 5 names
  "Thunderbolt" among the level-0 abilities, and BAT-MAG-064's 2e note weighs "3 Thunderbird
  attacks ... and 4 Wyvern Monarch attacks" together as "12 % to see neither without the
  immunity". For the disputed point (does a level immunity cover level 0, which VCMI denies) only
  Poison counts: 0 of 4 at 30 % is 0.7^4 = 24 %. The clause itself is sound by my re-read: 0x44A518
  compares the level signed with 4 and jumps only when greater, `SPTRAITS.TXT` gives Poison level
  0, and Poison reaches 0x44A4D0 through 0x5A8640.
- fix or test sketch: reword the note (the bolt shows the Dragon Father against a strike's level-2
  spell; Poison alone the level-0 point, 24 %).

### F-r6-limits-opening-5: Quirk Q-0005 changes Eagle Eye's chance, but neither the two rules' `quirks` nor the ledger entry say so, and no test pins the original's sums
- rules: BAT-MAG-069, HER-SPL-004, quirk Q-0005
- kind: rule-text (with a test-gap)
- severity: low
- confidence: high
- evidence: both summaries name Q-0005, both have `quirks = []` (`battle.toml:3439`,
  `heroes.toml:152`); the ledger's Q-0005 (`quirks-ledger.md:78–88`) lists neither rule under
  "rules affected" nor an Eagle Eye test. The flag does change play: computed from 0x4E4420's
  arithmetic (f32 table and constants, each sum stored as f32, `trunc(p × 100)`), the original
  gives Advanced Eagle Eye with the Emblem of Cognizance 64 % (0.5 + 0.15f is an exact tie,
  rounded to even), Basic with Stoic Watchman and Emblem 64 %, Basic with the Bird of Perception
  45 % (a conversion of 45 % to f32 alone would give 44 %), a Basic specialist 43 % at level 2 and
  51 % at level 6, an Advanced specialist 64 % at level 6. The engine's `RateSum` follows the same
  steps, so I expect these values; the tests check only 44 % → 43 % with a hand-made rate
  (`eagle.rs:141`) and a derivation whose f32 sum equals the basis points' own value
  (`tests/derive.rs:1293`).
- fix or test sketch: add "Q-0005" to both rules and Eagle Eye to the ledger entry. Test
  (`needs-run`, `h3-pack`, harvested packs): derive Orrin with Advanced Eagle Eye and
  `base:emblem_of_cognizance`, start a battle, `observe(..).own_hero.eagle_chance` is 64 in
  `base:faithful` and 65 in `base:strict-fixed`; Basic with `base:bird_of_perception` 45 in both;
  `base:serena` (a specialist) at level 2 with Basic: 43 and 44.

### F-r6-limits-opening-6: Two clauses of the opening spells' timing have no test: bad morale or Fear passing the first stack over, and the cast coming before the Enchanters' own
- rules: BAT-MAG-068
- kind: test-gap
- severity: low
- confidence: high
- evidence: `opening.rs:226` covers only a stack that loses its turns (`NO_TURNS`); in
  `turn.rs:233–241` bad morale and Fear are separate branches, so moving `activation` before them
  would pass every test, while the original activates only after 0x465283. The order inside
  `turn.rs:265–269` (opening, drain, periodic) has a gameplay effect no test sees: with Armor of
  the Damned and Enchanters on one side, Slow and Weakness are already on the enemy's stacks when
  the Enchanters' table is counted (0x4653BC before 0x465503), so they draw another spell.
- fix or test sketch (`needs-run`): give the defender's hero of `opening.rs` `Morale { amount: -3,
  affects: Own }`, loop seeds until `BadMorale { stack: StackId(2) }` appears after the attacker's
  Defend, and assert `BadMorale < TurnStarted(3) < OpeningSpell`. Second test: a `chanter` army with
  an `opening("sloth", 50)` hero; assert the `OpeningSpell` event precedes `PeriodicCast` and the
  periodic spell is never `sloth`.

### F-r6-limits-opening-7: No test asks the strike's (or a creature's) casting side for the negation
- rules: BAT-MAG-066 (with BAT-MAG-040)
- kind: test-gap
- severity: low
- confidence: high
- evidence: `after_strike.rs:146` takes the striker's controller as the casting side, as the
  original does (0x44054E–0x44057B pass the flipped side, 0x5A80A1 takes that side's hero). Every
  test with `NegateImmunities` is a hero's cast or a mine (`hero_defences.rs`, `mines.rs:321`);
  batch 2e's strike case (Poison at Gargoyles) is an absolute condition and shows nothing here.
- fix or test sketch (`needs-run`, `hero_defences.rs`): attacker's `blinder` (Blind at 100 %)
  strikes the defender's `statue` (mind-immune): no `StrikeSpell` without effects, `[blind]` when
  the attacker's hero has `NegateImmunities`, and again when only the defender's has it.

### F-r6-limits-opening-8: Two doc comments still say the Faerie Dragon ignores the Orb of Inhibition
- rules: BAT-MAG-061
- kind: rule-text
- severity: low
- confidence: high
- evidence: `crates/h3-content/src/ability.rs:370–372` ("the Faerie Dragon, after the wiki and
  VCMI against the code reading until an experiment settles it") and
  `crates/h3-battle/src/casters.rs:381` name the Faerie Dragon as the creature with
  `ignores_magic_block`. Batch 2e settled it the other way; no harvested creature has the ability
  (`harvested_defences.rs:189–195`), as the rule says.
- fix or test sketch: reword both (an ability no harvested creature has; kept for packs).

### F-r6-limits-opening-9: Exit criterion 3's wording and `docs/04` §4 do not match four confirmed quirks without a flag
- rules: exit criterion 3; BAT-QUIRK-10, 11, 15, 18 (not my slice's quirks)
- kind: rule-text
- severity: low
- confidence: high
- evidence: the criterion says "confirmed quirks are in the ledger with flags", `docs/04` §4 says
  `preserve` is "controlled by a quirk flag" and `strict-fixed` has "all quirks fixed". The ledger
  holds four entries of class "preserve, no flag: the rule in every profile" (each a user
  decision: BAT-Q12, Q13, Q15, Q18), so `strict-fixed` keeps them. The part I was asked to check
  holds: the two profiles differ exactly by the 23 registered flags.
- fix or test sketch: one sentence in `docs/04` §4 (a quirk the user keeps in every profile has no
  flag) or in the criterion.

## Wiki checks wanted

1. Orb of Vulnerability: the exact sentence on artifact immunities (Pendants, Power of the Dragon
   Father, Armageddon's Blade) and on the Unicorns' aura; whether either cites a test (F-2, F-3).
2. Unicorn and Hypnotize: anything on whose Unicorns protect a hypnotized stack (F-1).
3. Recanter's Cloak and Power of the Dragon Father: any statement on creature abilities (Stone
   Gaze, Paralyze, Age; Poison) beyond what the catalog quotes.
4. Eagle Eye (SoD text): whether any page gives the odd values (43 %, 64 %) of F-5.
5. Badge of Courage: "immunity to mind spells" for SoD (BAT-MAG-065 has no observation).

## Not reviewed, left open

- The wiki quotes in the catalog (no access); I took them as written.
- The application of `HeroOutcome.learned` after the battle (adventure lane; the battle QUEUE's
  inbox holds it). I did re-read 0x469FE0: a worn Spell Book and level ≤ Wisdom + 2, as BAT-Q24.
- The ledger entries other than Q-0006 and BAT-QUIRK-19: only that their flags and test names
  exist. Q-0001 and Q-0002 name no test (placeholders from before E04b).
- The computer player's paths (0x4396E0, the 0x42xxxx callers of 0x44A4D0), the E04c parts of the
  same functions (Cursed Ground, the anti-magic garrison, the arrow tower), the byte +0xEA that
  0x5A829B tests (H3API guesses a losses marker), and the inside of 0x4448F0.
- Two notes without effect on a rule: §7.23 lists Armageddon's Blade's test before the polarity
  tests (the code has the polarity tests first, 0x5A82F8, then the switch); §7.23 reads the
  spell-book test of 0x41F890 as tied to Cursed Ground (it applies to every hero cast).
