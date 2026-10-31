# Battle lane — QUEUE

Pre-refined tasks of the battle lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

E23, the original's battle AI, is **closed** (2026-10-24; `docs/epics/E23-original-battle-ai.md`, its log's "T7 — the closure"; the cuts of its tasks: OPEN-QUESTIONS BAT-Q52 to BAT-Q61 and the epic file). E04b closed on 2026-10-12, E04c on 2026-10-16 (their epic files and logs; their cuts: OPEN-QUESTIONS BAT-Q10 to BAT-Q27, BAT-Q37 to BAT-Q50).

**Next (the user, 2026-10-24):** the inbox session's leftovers are done (#458, #464; CLT-Q4/Q6's seam and carry, the 112th session); next **E11, the historical rules of RoE and AB** (refined 2026-10-24, Q58; `docs/epics/E11-historical-rules.md`): T1, the sourcing and catalogue with the user's own original media.

## Inbox

Consolidated on 2026-10-19 (the user): the entries from E01 to E04c's closure are merged below by kind, done and duplicate parts dropped; the origin stays in brackets (an epic and task, a review report such as r3 or F-r2b-2). History: git. Items for other lanes went into their inboxes; the First Aid Tent became BAT-Q51. The addresses are readings of the executable, the reviews' not read a second time. Where the readings stand: `installation.md` §7.8–§7.33; the reviews' reports stay on `battle/e04b-t11b-review-reports` and `battle/e04c-t10b-review-reports`; their experiment candidates are in `docs/experiments/README.md` ("From the battle queue's inbox", "From the E04b review", "From E04c").

- 2026-10-30 (the client lane, E17 M5 T5b2; **a difference from the original, for you to
  look at**): the turn order after a created stack's round. `seq-summon.toml` (hot seat, seed 1):
  Orrin summons two Fire Elementals (speed 6) at the Swordsmen's (speed 5) turn, the Swordsmen
  defend, the elementals wait, the Archers defend, the elementals defend at their waited turn;
  round 2 then starts with the elementals in the original (the harness's memory: the active
  stack stays attacker record 1 at 1 200 ms) but with the Swordsmen in the engine
  (`RoundStarted { round: 2 }`, `TurnStarted { stack: 0 }`). The client's `summon-turn`
  sequence stops before round 2; the commands that showed it are in its comment's history (the
  E17 log, "M5 T5b2").

- 2026-10-29 (the client lane, E17 M5 T5a, `cross-lane` with the user's consent; **done in
  your crate, for you to review**): a Clone or Summon Elemental that finds no place now reports
  it: `EventKind::NotCreated { side, creature, count, original }` (public; BAT-MAG-037's "Not
  modelled" sentence replaced), from `created::clone`'s `None` place and after `created::summon`'s
  columns; no state changes. The original shows a message box for a clone (0x5A6E22, GENRLTXT
  189); its summon writes its line and calls 0x47A100 with hex −1, which reads past the hex
  records (CLT-Q33, `docs/epics/E17-design.md` §7.25). Wire v36 (`h3-protocol`, the server lane's
  inbox). Tests: `tests/created.rs`'s two no-place cases.

- 2026-10-29 (the client lane, E17 M5 T4c2b, `cross-lane` with the user's consent; **done in
  your crate, for you to review**): a placement in the tactics phase is the original's walk
  (action 2 through the move handler 0x445950; `docs/epics/E17-design.md` §7.24). `h3_battle`
  `tactics::place` now walks a walker's placement position by position along
  `movement::placement_path` (the pathfinder with the tactics query, 0x5239D0, 0x4B34DE) for a
  recording sink, so viewers animate it; and for every sink where a step may open a town's closed
  gate (a defender's stack), which opens it at the step (0x442186) and closes it behind the stack
  (`siege::close_gate`), as a walk does. Found by `walls_and_the_moat_agree_with_movement`; tests
  in `tests/tactics.rs` and `tests/sieges.rs`; BAT-TAC-002 and BAT-MOV-003 amended. The
  pathfinder's "not modelled" note dropped the placement. Nothing to act on unless you disagree.

- 2026-10-29 (the adventure lane, E08 T69; **observations for BAT-TER-002 and BAT-GEO-004**,
  both `implemented`): batch E8b read the original's battles at sea (`experiments/results/E08-batch-8b.md`,
  `-run2.md`; survey `E08-sea.md` §15). One hero in a boat (E8-11, E8-13, E8-14): water, no special
  field, the ship flag, one obstacle 55 (three hexes: 98, 114, 115; 79, 96, 97; 90, 106, 107), as
  `deck`; both in boats (E8-12): exactly the 32 hexes of `boats` (0x63D368). Add them as
  observations when you next touch the catalog; nothing to change.

- 2026-10-28 (the client lane, E17 M5 T4a; **for you to judge: `h3_formats::text` and quotes**):
  the executable's text-table readers make each run of quotes in a field one, after cutting a
  quoted field's closing quote (`text_table_records` 0x5BBBA0 at 0x5BBD29 to 0x5BBD6A; the
  columns' reader 0x5BBE70 has the same code; `docs/epics/E17-design.md` §7.19).
  `h3_formats::text::TextTable::parse` keeps them as the file has them (its test
  `quoted_fields_span_lines_and_keep_inner_quotes`), so GENRLTXT's record 33 reads
  `""I will accept…""` where the game shows `"I will accept…"`. The client's catalogue makes the
  runs one where it reads a field (`h3_view::text::pack`), so nothing of yours has to change for
  it; a harvested name or text with doubled quotes would differ from the game's. Your call
  whether the parser should follow the executable.

- 2026-10-27 (the AI lane, E25's refinement, AI-Q10; **for you, later: a fair system actor**):
  the user decided that an actor reading hidden facts must not train agents, and that a **fair
  variant** of the system actor is phased in, for you to design and build at a later time
  (ADR-0027 point 4's announcement; an amendment of that point, your draft). It decides only on
  what its side may know; once built it is the default everywhere (the env, training, the
  server, the clients), with today's faithful, all-seeing computer kept as a game's option.
  Today's stays as it is: E25's target, playing in no agent's training. What it reads that a
  player may not (the AI lane's reading, not exhaustive): the enemy hero's spell book,
  masteries, spell bonuses and cost changes (`aim.rs`: `best_area_spell` for the area threat,
  BAT-AI-007; `spell_protection`, BAT-AI-016; `hero_value` in the odds, BAT-AI-008), the enemy
  hero's damage skills (`protection`'s Armorer) and its stacks' magic resistance
  (`magic::effect_factor`) in the estimates, the enemy side's hidden pits in its reach maps
  (`computer::reach_map`, BAT-AI-007). The ways offered at the refinement: the port unchanged on
  a world built from its side's observation, the hidden part guessed by a prior (E06's
  determinized worlds, `BattleState::determinized`, ADR-0025 point 12a) or assumed absent; or its
  reads rewritten. The AI lane would train against it and may imitate it (E25 decision 3).

- 2026-10-27 (the AI lane, E06 T9d; **for you, a comment only**, when you next touch the file):
  `crates/h3-battle/src/world.rs`'s module documentation cites `docs/epics/E06-design.md` §14,
  deleted at E06's closure: its text is kept in `docs/epics/E06-log.md`, "The design note at
  closure", §14. Also for your information: E06 T9b's review found three gaps of a world against
  ADR-0025 point 12a.2 (the other hero's morale and luck modifiers kept, the magic limits
  recomputed with the guess, a commanded war machine automatic without a guess), now AI-STUB-1's
  (E25, `docs/stubs.toml`); nothing to do before E25.

- 2026-10-27 (the adventure lane, E08 T60a, `adventure/e08-t60a`, `cross-lane` with the user's
  consent; for your information, nothing to do): **`HeroDetails` gained `bonuses`**
  (`h3_heroes::BattleBonuses`: morale and luck until the end of the hero's next battle, and
  once-marks; HER-ADV-008). Every `HeroDetails` literal of `h3-pack-battle`'s tests (`src/tests.rs`,
  eleven `tests/harvested_*.rs`) got `bonuses: Default::default()`; a new literal needs it too.
  `derive_battle_hero` adds the bonuses to the own side's `Morale` and `Luck` effects, so a battle
  from the map may now carry morale or luck a hero's skills and artifacts do not explain; the
  seam and your rules are unchanged, and every bonus is 0 until increment 7's objects (T62).

- 2026-10-27 (the adventure lane, E08 T59, `shared/e08-t59`, `cross-lane`; for your information,
  nothing to do): **your harvester and packs** gained increment 7's shared part: the gap-fill's
  `[heroes.params]` `mana_ceiling = 999` (source `exe-mana-ceiling`) written into `[hero_params]`
  (pack format v48, read by `h3-heroes` from E08 T60), and both profiles name
  `quirk.adventure.roe_witch_hut_mask` and `quirk.adventure.full_backpack_refuses` (registry
  positions 37 and 38, `ADV-QUIRK-21` and `-22` in `docs/rules/quirks-adventure.md`): your next
  flag is position 39. `base` re-harvested: three lines. Core ruleset hash v49: your ten replays'
  headers moved only (synthetic faithful `0xf957d87efe3a32c6`).

- 2026-10-30 (the adventure lane, E08 T81b, `shared/e08-t81b`, `cross-lane`; for your information,
  nothing to do): **your harvester and packs** gained increment 9's shared part: the gap-fill's
  `[[heroes.skill_effects]]` for Estates (`daily_resource` gold 125, 250, 500) and Mysticism
  (`mana_regain` 2, 3, 4), `[heroes.resource_specialty]` (specialty kind 2: 1 a day, gold 350),
  `[heroes.params]`' `base_mana_regain` and `regain_specialist_bonus` (1, 1), the income and mana
  artifacts' effects (the Cornucopia's 4 of four, the Wizard's Well's `full_mana`), the
  `[[creature_abilities_outside_battle]]` table (the Crystal Dragon's `weekly_resource`, written as
  the core file's `abilities_outside_battle`: `split.rs` keeps `abilities` the battle file's), four
  sources (`exe-daily-income`, `exe-mana-regain`, `vcmi-income`, `wiki-income`); the `pending`
  list keeps the Grail and the Legion. Pack format v50; `base`, `ab` and `sod` re-harvested: only
  those lines. `synthetic-ext` carries the four kinds (`wayfarer_sash`) and the ability (the
  Lurker); `h3-pack-battle`'s every-kind check counts six generic arguments. Core ruleset hash
  v51: your ten replays' headers moved only (synthetic faithful `0xb0bd4ac2601bb638`). No battle
  rule reads the kinds or the ability.

- 2026-10-29 (the adventure lane, E08 T70, `shared/e08-t70`, `cross-lane`; for your information,
  nothing to do): **your harvester and packs** gained increment 8's shared part: the gap-fill's
  `[[heroes.skill_effects]]` for Navigation (`navigation` 1, 2, 3), `sea_movement_points` for the
  Necklace of Ocean Guidance (1 000) and the Sea Captain's Hat (500, beside its two spells),
  `free_boarding` for the Admiral's Hat, the three out of the `pending` list, four sources
  (`exe-sea-movement`, `exe-free-boarding`, `vcmi-sea-movement`, `wiki-sea-movement`); pack format
  v49, read by `h3-heroes` from E08 T71. `base` and `sod` re-harvested: only those lines;
  `synthetic-ext`'s `wayfarer_sash` carries the three kinds (`h3-pack-battle`'s every-kind check).
  Core ruleset hash v50: your ten replays' headers moved only (synthetic faithful
  `0x382b666869803877`). No battle rule reads the kinds.

- 2026-10-27 (the AI lane, E06 T6f and T8b, `claude/jolly-volta-qejdfa`, `cross-lane` with the
  user's consent; **for you**): two additions to `h3-battle` the AI lane made and reads.
  **`h3_battle::turn_order(&Observation) -> TurnOrder`** (`turn.rs`): the rest of the round's
  turns and the next round's, sharing `next_active`'s key (`OrderKey`), so a change of the order
  rules (BAT-TRN-001..003, BAT-SIE-011/016) changes both at once; the AI lane's observation v22
  encodes it per stack, and a battle client could show it as a queue. **A search agent's worlds,
  `BattleState::determinized` and `reseed`** (`world.rs`, ADR-0025 point 12a): a copy with
  everything a side may not know (`observe_as`'s list) replaced; `state::world_templates`,
  `obstacles::place_unseen_again` and `Created::set_templates` serve it, and `templates` takes
  the armies' creatures. **A change of what a side may not know** (a new hidden field, a new
  hidden obstacle kind) **changes `world.rs` in the same change** (the ADR's point 2), as it
  changes `observe_as`; the property `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`
  catches most. A trap for your `TRAPS.md` ("Battle code"). Known gap: AI-STUB-1.

- 2026-10-24 (the E11 refinement, `shared/e11-refinement`, Q58): **E11 is yours to lead**
  (`docs/epics/E11-historical-rules.md`): the historical rules of RoE and AB from the user's own
  original media. **The order (the user, 2026-10-24):** E23 closed first (`battle/e23-closure`),
  then one session on this inbox's leftovers, and only then E11's T1 (sourcing and catalogue, with
  the user, when the media are at hand). The sketch's "AB and SoD completeness" is folded; E03's
  note that AB/SoD abilities "stay E11" is history (built in E04b).

- 2026-10-23 (the AI lane, E06 T6a; the preview part done 2026-10-24, BAT-DMG-011): **the
  gated batch benchmark with `System` seats** (ADR-0027 point 11, E23 exit criterion 4),
  `benches/ai/batch-system-neutral-v1.toml` (the neutral kinds, the system actor defending; 6.81
  G instructions at `POLICY_VERSION` 19, about 532 000 a caller step), is lane-only: a change of
  `h3_battle_ai` moves it, and the AI lane re-baselines it.

- 2026-10-23 (the AI lane, E06 T7c, `claude/bold-curie-a42rc4`; **for you, with each
  `h3_battle_ai::POLICY_VERSION` bump**): the system actor is now a bot of the AI lane's tables,
  `python/records/bots-report-v1.json` (it against Random, Aggressor, Greedy on the report set)
  and the new `bots-neutral-v1.json` (every bot attacking it on the neutral set v1); both name
  the policy version (`engine.system_policy`), and the AI lane's Python test fails when it is
  stale. A pull request that changes the computer's choices regenerates both (from the root, in
  release about 85 s and 15 s; `python/records/README.md`):
  `cargo run --release -p h3-cli -- ai eval --set report-v1 --json python/records/bots-report-v1.json`
  and the same with `--set neutral-v1 --json python/records/bots-neutral-v1.json`. Today's
  numbers (version 18): the system actor beats Greedy 0.653 and Aggressor 0.681 on the report
  set, and attacks best on the neutral set (value +0.285, Greedy's +0.223).

- 2026-10-22 (E23 T4c's blind check; a rule to confirm): the original's end routine 0x444510 returns early only for Disrupting Ray (47), so a dispel or Cure that reaches Acid Breath (80) may end it; BAT-MAG-010 keeps every permanent effect. Read the dispels' callers or play a batch row (Cure on a stack under Acid Breath) and align BAT-MAG-010 or record why not.

- 2026-10-20 (the adventure lane, E08 T24b1, `adventure/e08-t24b1`; hygiene, with your next change to the file): `crates/h3-harvest/data/gapfill.toml`'s source `exe-necromancy` has `url = "docs/epics/E08-design.md"`; that note was rewritten for E08 increment 4 and no longer holds the Necromancy section: `docs/epics/E08-log.md` (T19a2b) does. Increment 4 will also want gap-filled hero data (the experience table, the forced skills, the classes' offer intervals: `E08-design.md` §1, §10); the cut (T24b2) asks you.

- **For E23 (the original's battle AI), to read with it:** the original's pathfinder 0x4B3400 is read (§7.40, §7.41) and built (`pathfinder.rs`, E23 T1c): the walks and the berserk turn follow it. `aim.rs`'s threat records (0x422B20, the turrets' and the Ballista's turns) still search on their own (flyers by hex distance, no marks, no first-in-first-out): move them onto `pathfinder::Search`'s marks (BAT-Q53) with the computer's walker (T3). T1b's readings at risk from T1c: S-11's three outcomes, a berserk flyer ending on a hex it passed over, the war machine aimed at by its front, a walk beyond the speed into its side's seen pit (`E23-log.md`). The automatic Catapult's aim with Ballistics (the computer's and hidden battles': the gate first, §7.29) is not modelled: the agent aims. The moat avoidance's further "automatic" cases (0x474BF0: a hidden battle, an options flag, a remote turn; BAT-SIE-022). BAT-Q33 (who commands a side) decides with it.

- **The controller against the owner** (the engine mostly reads the owner; a hypnotized stack's morale and luck are the controller's since batch 2g HM): a hypnotized shooter's bows, which the original looks for on the controller's hero for the walls' halving (0x46751A to 0x46753A, BAT-SIE-008) and the distance's (0x467600, BAT-RNG-002), where the engine sets traits from the owner's hero at the start (`hero::trait_effects`; F-r2b-2); Hypnotize ending on the active stack mid-turn keeps the acting side until the next activation (0x4653AE), the engine hands the turn to the owner at once (r3); the battle's end with only hypnotized stacks left (unread; the engine counts them for their owner).

- **War machines:** the original sets up none with the battle's bank flag (+0x53C5; 0x463758 to 0x46375D): the setup has no creature bank yet (a side without a hero brings none since 2026-10-24, BAT-WAR-001); the Ammo Cart's test is "the shooter's side's hero wears artifact 5" (0x43F9E1) without a test that the cart stands (BAT-WAR-004; r3); the original's Defend gives a war machine no bonus, nor any stack in the tactics phase (0x479722; does the engine?); the Genie's and the Enchanters' usefulness tests count a Ballista without shots as a shooter only under Q-0004, the original always (0x4428F0).

- **Turns** (batch 2g aligned the wait phase, the freeze's tie-break and the wait phase's turn start, and recorded the good morale before a re-entry, BAT-MOR-003): a `rebirth` creature a mine or Fire Wall kills on its way walks on and attacks after its rebirth (the original probably too, 0x478900; r5b); an active Phoenix killed by its own hero's cast and reborn stays the active stack (unread; r1); an active stack hit by a redirected Blind in its own turn finishes it (r2b); every routine that turns a stack (0x446720: the strike, the melee attack's struck stack, a walk's steps, the creature's cast, the flight, the teleporting move, the raise; §7.25) may measure at its turned front hex: the breath's second hex and the attack patterns are leads, the placement and the berserk shooter's distance (#150's call 2) unread.

- **The surrender's price:** the halving before the discount (0x47506B; one Pixie at 10 %: 10, not 11) is not observed; the temporary creatures are left out since batch 2g SP (BAT-FLE-002).

- **Numbers and draws:** BAT-MAG-058 takes the larger of a cast Fire Shield and the creature's own, the original the cast one first (no difference with the base values; r2b); the physical damage's stage 1 runs its hero step also at a rate of 0, rounding through `f32` (0x44371C; a difference only above 2^24; r2a); the original rolls also at a chance of 0 or 100 and walks an area's stacks from the last to the first (only the random stream differs; r1); the obstacle generator is not draw-exact (BAT-GEO-004 is VCMI's: the original draws one `rand()` per tried type and anchor, 91 usual types, 34 absolute, 151 anchors, through the C runtime's generator; its Stronghold siege branch 0x466147 is not modelled; does it keep an obstacle off another's free anchor? ADV-QUIRK-5's "layouts match the original's" needs both).

- **Other readings:** the corpse limit of 14 per hex (not modelled; it also bounds the Pit Lord's search); the stack bit `0x4000000` set after a creature's cast; a `rebirth` creature with several casts (data only); BAT-QUIRK-17's later variant, a sacrificed Phoenix reborn right after the sacrifice (BAT-Q18); the creature flag bit `0x800` (Arch Mage, Beholder, Evil Eye; `h3-harvest`); the Faerie Dragon's key path under the Orb refuses a Chain Lightning at the caster's side's stacks, which the engine offers (BAT-Q32 (1)); the knights' Curse stays at 25 % and mastery 0 until a statistics batch (BAT-Q28 row 13); Q43 (the sprite offset). From E04c's security review: whether the turrets' counts (BAT-SIE-016) show the town's building count where the original does not (its tower window is unread), and how long one command runs with many rounds of turret turns inside it (no bound measured).

- **Code left over:** BAT-STUB-1's per-stack mark at the next observation-encoding bump, with BAT-Q35's answer (`cross-lane` with the AI lane). From E04b's review fixes: a bound on the `SpellDamage` rates in `hero::check` (the fix saturates instead; r1 F-5); a `fixed` duration of up to 65 535 rounds on a turn-stopping change still compiles (r2b F-7); `h3_battle::stack_spell` answers without a viewer and `BattleState::spell_obstacles()` returns the hidden ones too (none leaks today). Doubts of the fold-in's read, none a defect: at a round start the end check comes before the obstacles' countdown, so `ObstacleRemoved` can follow `BattleEnded`, a draw there untested (A6); without BAT-QUIRK-5 a created stack is still a Sacrifice target (B7); a walker that strikes and returns is put back without entering anything when its start is out of reach, its path computed once across rebirths (B9); `magic::opening` asks for an active effect where a hero's Expert Dispel does not; a stack that resists while a chain looks for its next stack is passed over without `SpellResisted`.

- **Docs:** code-reading sources that word older readings: BAT-MAG-006, 022, 032, 035, 048, 049, 057, 066, BAT-MOR-004; `packs/README.md`'s "Profiles" names only Q-0003 and Q-0005; the battle tests' profile `t:faithful` has no quirks (a misleading name); `installation.md` §7.23 lists Armageddon's Blade's test before the polarity tests, the code has them first (0x5A82F8).

- **Performance:** `random-rollouts-full` was re-baselined in E04b T11b at +3.3 % Ir per step without a per-function profile (candidates: a `may_affect` more per spell without a target in `castable_spells`, the automatic Ballista's reach, dead stacks' kept effects), and stands at +1.34 % on `master` (2026-10-19). Its Ir swung by 7 to 12 % with the inlining of `Field::neighbor` three times in E04a: a precomputed neighbour table in `Field` if it swings again.

- **Tooling, later:** `scripts/exe.py switch` misses a `dec` (and `inc`) before the range check, so it labels the dispatcher's cases (0x47902F) one lower than the action (E23 T1a1; `Code._switch_base` handles `lea`, `add`, `sub` only: a `shared/` fix); a `Trace` sink for rule-level explanations (02-architecture §10); profile-level Tier-2 parameters (ADR-0014; data quirks are done, ADR-0007); `cargo-fuzz` targets for `h3-formats`; the `.h3m` writer's tile pictures beyond dirt, grass and water (from a game map first); a batch for what batch 2g left (BAT-Q28 rows 1, 3, 5, 6, 11, 12 and 13's statistics, BAT-Q32's key path, the survey's S-26, S-27).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
