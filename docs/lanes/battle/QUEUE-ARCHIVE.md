# Battle lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/battle/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-11-07 (the adventure lane, E08 T102, `shared/e08-t102`, `cross-lane`; for your
  information, nothing to do): **your harvester and packs** gained increment 11's shared part: the
  gap-fill's Sea Captain's Hat carries the new kind `whirlpool_protection` beside its spells and
  sea points, with three sources (`exe-whirlpool-protection`, `vcmi-whirlpool-protection`,
  `wiki-whirlpool-protection`); `base` re-harvested: only the Hat's line. `synthetic-ext`'s
  `wayfarer_sash` carries the kind (`h3-pack-battle`'s every-kind check). Pack format v51, core
  ruleset hash v52: your ten replays' headers moved only (synthetic faithful
  `0x9c7c97831143d1f1`); your HANDOFF's shared versions updated. The experiments README's batch
  E11 row is **evaluated** (E08 T101). No battle rule reads the kind.

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
  *Done 2026-11-02:* refined into E26 (BAT-Q63; `docs/epics/E26-fair-system-actor.md`): the port
  on its side's world, the queue's E26 tasks.

- 2026-10-30 (the client lane, E17 M5 T5b2; **a difference from the original, for you to
  look at**): the turn order after a created stack's round. `seq-summon.toml` (hot seat, seed 1):
  Orrin summons two Fire Elementals (speed 6) at the Swordsmen's (speed 5) turn, the Swordsmen
  defend, the elementals wait, the Archers defend, the elementals defend at their waited turn;
  round 2 then starts with the elementals in the original (the harness's memory: the active
  stack stays attacker record 1 at 1 200 ms) but with the Swordsmen in the engine
  (`RoundStarted { round: 2 }`, `TurnStarted { stack: 0 }`). The client's `summon-turn`
  sequence stops before round 2; the commands that showed it are in its comment's history (the
  E17 log, "M5 T5b2"). **Settled 2026-11-02 (113th session):** no difference of the engine: the scenario loads `base` alone, where the elementals are neutral (speed 6, the Swordsmen native on grass at 6, the tie to the lower slot); with `ab` they are Conflux natives (7) and the engine's round 2 starts with them, as the original's (SoD). A note in the client lane's inbox.

- 2026-10-29 (the client lane, E17 M5 T5a, `cross-lane` with the user's consent; **done in
  your crate, for you to review**): a Clone or Summon Elemental that finds no place now reports
  it: `EventKind::NotCreated { side, creature, count, original }` (public; BAT-MAG-037's "Not
  modelled" sentence replaced), from `created::clone`'s `None` place and after `created::summon`'s
  columns; no state changes. The original shows a message box for a clone (0x5A6E22, GENRLTXT
  189); its summon writes its line and calls 0x47A100 with hex −1, which reads past the hex
  records (CLT-Q33, `docs/epics/E17-design.md` §7.25). Wire v36 (`h3-protocol`, the server lane's
  inbox). Tests: `tests/created.rs`'s two no-place cases. **Settled 2026-11-02 (113th session):** reviewed: nothing to change.

- 2026-10-29 (the client lane, E17 M5 T4c2b, `cross-lane` with the user's consent; **done in
  your crate, for you to review**): a placement in the tactics phase is the original's walk
  (action 2 through the move handler 0x445950; `docs/epics/E17-design.md` §7.24). `h3_battle`
  `tactics::place` now walks a walker's placement position by position along
  `movement::placement_path` (the pathfinder with the tactics query, 0x5239D0, 0x4B34DE) for a
  recording sink, so viewers animate it; and for every sink where a step may open a town's closed
  gate (a defender's stack), which opens it at the step (0x442186) and closes it behind the stack
  (`siege::close_gate`), as a walk does. Found by `walls_and_the_moat_agree_with_movement`; tests
  in `tests/tactics.rs` and `tests/sieges.rs`; BAT-TAC-002 and BAT-MOV-003 amended. The
  pathfinder's "not modelled" note dropped the placement. Nothing to act on unless you disagree. **Settled 2026-11-02 (113th session):** reviewed: nothing to change.

- 2026-10-29 (the adventure lane, E08 T69; **observations for BAT-TER-002 and BAT-GEO-004**,
  both `implemented`): batch E8b read the original's battles at sea (`experiments/results/E08-batch-8b.md`,
  `-run2.md`; survey `E08-sea.md` §15). One hero in a boat (E8-11, E8-13, E8-14): water, no special
  field, the ship flag, one obstacle 55 (three hexes: 98, 114, 115; 79, 96, 97; 90, 106, 107), as
  `deck`; both in boats (E8-12): exactly the 32 hexes of `boats` (0x63D368). Add them as
  observations when you next touch the catalog; nothing to change. **Settled 2026-11-02 (113th session):** both sources added.

- 2026-10-28 (the client lane, E17 M5 T4a; **for you to judge: `h3_formats::text` and quotes**):
  the executable's text-table readers make each run of quotes in a field one, after cutting a
  quoted field's closing quote (`text_table_records` 0x5BBBA0 at 0x5BBD29 to 0x5BBD6A; the
  columns' reader 0x5BBE70 has the same code; `docs/epics/E17-design.md` §7.19).
  `h3_formats::text::TextTable::parse` keeps them as the file has them (its test
  `quoted_fields_span_lines_and_keep_inner_quotes`), so GENRLTXT's record 33 reads
  `""I will accept…""` where the game shows `"I will accept…"`. The client's catalogue makes the
  runs one where it reads a field (`h3_view::text::pack`), so nothing of yours has to change for
  it; a harvested name or text with doubled quotes would differ from the game's. Your call
  whether the parser should follow the executable. **Settled 2026-11-02 (113th session):** `TextTable::parse` follows the executable: a run of quotes in a quoted field reads as one (`installation.md` §4); no harvested text had one. Notes in the client and map-client lanes' inboxes.

- 2026-10-27 (the AI lane, E06 T9d; **for you, a comment only**, when you next touch the file):
  `crates/h3-battle/src/world.rs`'s module documentation cites `docs/epics/E06-design.md` §14,
  deleted at E06's closure: its text is kept in `docs/epics/E06-log.md`, "The design note at
  closure", §14. Also for your information: E06 T9b's review found three gaps of a world against
  ADR-0025 point 12a.2 (the other hero's morale and luck modifiers kept, the magic limits
  recomputed with the guess, a commanded war machine automatic without a guess), now AI-STUB-1's
  (E25, `docs/stubs.toml`); nothing to do before E25. **Settled 2026-11-02 (113th session):** the citation fixed.

- 2026-10-27 (the adventure lane, E08 T60a, `adventure/e08-t60a`, `cross-lane` with the user's
  consent; for your information, nothing to do): **`HeroDetails` gained `bonuses`**
  (`h3_heroes::BattleBonuses`: morale and luck until the end of the hero's next battle, and
  once-marks; HER-ADV-008). Every `HeroDetails` literal of `h3-pack-battle`'s tests (`src/tests.rs`,
  eleven `tests/harvested_*.rs`) got `bonuses: Default::default()`; a new literal needs it too.
  `derive_battle_hero` adds the bonuses to the own side's `Morale` and `Luck` effects, so a battle
  from the map may now carry morale or luck a hero's skills and artifacts do not explain; the
  seam and your rules are unchanged, and every bonus is 0 until increment 7's objects (T62). **Settled 2026-11-02 (113th session):** informational.

- 2026-10-27 (the adventure lane, E08 T59, `shared/e08-t59`, `cross-lane`; for your information,
  nothing to do): **your harvester and packs** gained increment 7's shared part: the gap-fill's
  `[heroes.params]` `mana_ceiling = 999` (source `exe-mana-ceiling`) written into `[hero_params]`
  (pack format v48, read by `h3-heroes` from E08 T60), and both profiles name
  `quirk.adventure.roe_witch_hut_mask` and `quirk.adventure.full_backpack_refuses` (registry
  positions 37 and 38, `ADV-QUIRK-21` and `-22` in `docs/rules/quirks-adventure.md`): your next
  flag is position 39. `base` re-harvested: three lines. Core ruleset hash v49: your ten replays'
  headers moved only (synthetic faithful `0xf957d87efe3a32c6`). **Settled 2026-11-02 (113th session):** informational.

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
  rule reads the kinds or the ability. **Settled 2026-11-02 (113th session):** informational.

- 2026-10-29 (the adventure lane, E08 T70, `shared/e08-t70`, `cross-lane`; for your information,
  nothing to do): **your harvester and packs** gained increment 8's shared part: the gap-fill's
  `[[heroes.skill_effects]]` for Navigation (`navigation` 1, 2, 3), `sea_movement_points` for the
  Necklace of Ocean Guidance (1 000) and the Sea Captain's Hat (500, beside its two spells),
  `free_boarding` for the Admiral's Hat, the three out of the `pending` list, four sources
  (`exe-sea-movement`, `exe-free-boarding`, `vcmi-sea-movement`, `wiki-sea-movement`); pack format
  v49, read by `h3-heroes` from E08 T71. `base` and `sod` re-harvested: only those lines;
  `synthetic-ext`'s `wayfarer_sash` carries the three kinds (`h3-pack-battle`'s every-kind check).
  Core ruleset hash v50: your ten replays' headers moved only (synthetic faithful
  `0x382b666869803877`). No battle rule reads the kinds. **Settled 2026-11-02 (113th session):** informational.

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
  catches most. A trap for your `TRAPS.md` ("Battle code"). Known gap: AI-STUB-1. **Settled 2026-11-02 (113th session):** the trap is in `TRAPS.md` ("Battle code").

- 2026-10-24 (the E11 refinement, `shared/e11-refinement`, Q58): **E11 is yours to lead**
  (`docs/epics/E11-historical-rules.md`): the historical rules of RoE and AB from the user's own
  original media. **The order (the user, 2026-10-24):** E23 closed first (`battle/e23-closure`),
  then one session on this inbox's leftovers, and only then E11's T1 (sourcing and catalogue, with
  the user, when the media are at hand). The sketch's "AB and SoD completeness" is folded; E03's
  note that AB/SoD abilities "stay E11" is history (built in E04b). **Settled 2026-11-02 (113th session):** the order stands in the queue's "Next".

- 2026-10-23 (the AI lane, E06 T6a; the preview part done 2026-10-24, BAT-DMG-011): **the
  gated batch benchmark with `System` seats** (ADR-0027 point 11, E23 exit criterion 4),
  `benches/ai/batch-system-neutral-v1.toml` (the neutral kinds, the system actor defending; 6.81
  G instructions at `POLICY_VERSION` 19, about 532 000 a caller step), is lane-only: a change of
  `h3_battle_ai` moves it, and the AI lane re-baselines it. **Settled 2026-11-02 (113th session):** the trap is in `TRAPS.md` ("Packs, replays, benchmarks").

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
  set, and attacks best on the neutral set (value +0.285, Greedy's +0.223). **Settled 2026-11-02 (113th session):** the trap is in `TRAPS.md` ("Packs, replays, benchmarks").

- 2026-10-20 (the adventure lane, E08 T24b1, `adventure/e08-t24b1`; hygiene, with your next change to the file): `crates/h3-harvest/data/gapfill.toml`'s source `exe-necromancy` has `url = "docs/epics/E08-design.md"`; that note was rewritten for E08 increment 4 and no longer holds the Necromancy section: `docs/epics/E08-log.md` (T19a2b) does. Increment 4 will also want gap-filled hero data (the experience table, the forced skills, the classes' offer intervals: `E08-design.md` §1, §10); the cut (T24b2) asks you. **Settled 2026-11-02 (113th session):** the source's url already reads `docs/exe/map.toml`.

- 2026-10-24 (the adventure lane, E08 T49a, `adventure/e08-t49a`, `cross-lane` with the user's
  consent; for your information, done in your paths): **`HeroDetails.worn` is by position**
  (`Vec<Option<ArtifactInstance>>`, one entry per position of `h3_heroes::positions`: SoD's 19,
  AB's 18; ADR-0013 point 1). `h3_heroes::wearing(ruleset, list)` places a list at the first
  positions that take its artifacts (one that finds none is left out), `first_position` gives one;
  `validate` refuses a wrong length or an artifact on another slot's position. Your
  `h3-pack-battle` tests (`src/tests.rs`, eleven `tests/harvested_*.rs`) build details with
  `wearing`; the random details place each draw at its first position (the same draws, the same
  kept artifacts); `hero_details_fit_the_harvested_slots` states the Alliance's locked head and
  AB's four misc positions by position. `war_machines` lists them in position order. Nothing else
  of yours changed; `h3-experiment` builds maps through the writer, untouched.

- 2026-10-23 (the adventure lane, E08 T45, `shared/e08-t45`, `cross-lane` with the user's consent;
  for your information, nothing to do): **pack format v47, core ruleset hash v48**: two hero-effect
  kinds (`meeting_spells`, `gives_spell_book`; ADR-0013 points 3 and 14) that only `h3-heroes`
  reads (from E08 T49), nothing the battle sees. In your paths: the gap-fill gives Scholar's levels
  (2, 3, 4) and Titan's Thunder's `gives_spell_book` (out of the pending list), with six sources;
  `packs/base/core.toml` and `packs/sod/core.toml` re-harvested (only those lines);
  `synthetic-ext`'s `wayfarer_sash` uses both kinds (your every-kind check); the ten battle
  replays' headers (`ruleset_hash` only: synthetic faithful `0x3156c56f2211ba02`). Your HANDOFF's
  shared versions say so.

- 2026-10-22 (the adventure lane, E08 T41's shared part, `shared/e08-t41`, `cross-lane` with the
  user's consent; for your information, nothing to do): **flag 30 retired**
  (`quirk.adventure.flying_inland_price`, `ADV-QUIRK-16` withdrawn at ADV-Q22: the original's
  flying price is the rule in every profile). Its registry position stays taken (`RETIRED`; your
  next flag is still 33); in your paths both harvested profiles (`h3-harvest`) and
  `packs/base/core.toml` lose the line. `base:faithful`'s core ruleset hash moved
  (`0xde583eb06ef4dd73`); strict-fixed's, the synthetic packs' and the battle replays' did not.

- 2026-10-22 (the adventure lane, E08 T39, `adventure/e08-t39`, `cross-lane` with the user's
  consent, E08 decision 86; for information, nothing to do): Q-0005's ledger entry
  (`docs/rules/quirks-ledger.md`) names HER-ADV-003, a Logistics specialist's day's land points in
  single precision (`h3_heroes::land_points`, the same `grown` arithmetic as Learning's;
  observed by batch E5's E5-2), in its rules, regression tests and decisions.

- 2026-10-22 (the adventure lane, E08 T36b, `adventure/e08-t36b`, `cross-lane` with the user's
  consent, E08 decision 81; for information): `h3-experiment`'s `[map]` takes `banned_skills`
  (skill names; the writer's `Map::banned_skills`, SoD's header bits); batch E5e uses it to force
  a level-up's two offers. Nothing else of the crate changed.

- 2026-10-22 (the adventure lane, E08 T36a, `adventure/e08-t36a`, `cross-lane` with the user's
  consent; for information): `h3-experiment`'s `[[map.object]]` takes the trees (classes 135,
  137, 155) and the Border Gates (212) since the writer's `PLAIN_OBJECTS` holds them (no body);
  batches E5a and E5b use them. Nothing else of the crate changed.

- 2026-10-22 (the adventure lane, E08 T35b, `shared/e08-t35b`, `cross-lane` with the user's consent,
  E08 decision 79; for your information, nothing to do): **T35 is built** (the note below).
  `h3_content::HeroEffect` gained `pathfinding`, `land_movement`, `land_movement_points`, `flying`,
  `water_walking` and `scouting_radius` (codes 45 to 50; no `BattleEffect`: `derive_battle_hero`
  ignores them, no match needed a new arm); the registry flags 30 to 32
  (`quirk.adventure.flying_inland_price`, `drop_ignores_artifacts`, `radius_waits_for_a_step`;
  on in `base:faithful`, off in strict-fixed; your next flag is 33); **pack format v46, core
  ruleset hash v47**. In your paths: the gap-fill's three skill entries, five artifact entries and
  eight sources (`exe-movement`, `exe-step-cost`, `exe-scouting`, `vcmi-movement`, four wiki
  pages), the six artifacts out of the `pending` list, both harvested profiles with the three
  flags, `exe-necromancy`'s url now `docs/exe/map.toml`; `packs/base/core.toml` re-harvested (only
  those lines), `packs/README.md`'s effect table; your ten golden replays' headers (`ruleset_hash`
  only: commands and state hashes unchanged). Q-0005's rules list gains HER-ADV-003 with E08 T39,
  when the rule exists.

- 2026-10-22 (the adventure lane, E08 T34b, `adventure/e08-t34b`; for your information, nothing
  to do now): **increment 5's proposed cut touches your paths** once the user accepts it
  (ADV-Q21; `docs/epics/E08-design.md` §4, §11, §12). T35, a `shared/` and `cross-lane` pull
  request: `h3-content` gains six hero-effect kinds (codes 45 to 50, adventure-only: the battle's
  translation skips them) and three quirk flags (the next free registry positions, 30 to 32 unless
  you take one first), pack format v46 and core ruleset hash v47 (or the next free numbers), the
  harvester's gap-fill gives Pathfinding, Logistics, Scouting and six artifacts their effects
  (the packs regenerated, `packs/README.md`), its source `exe-necromancy` repointed to `E08-log.md`,
  Q-0005's ledger entry names HER-ADV-003, and your golden replays' headers move with the faithful
  hash. T36: batch E5's specs in `h3-experiment` (a ground under a hero, a border gate, unexplored
  strips), `cross-lane`.

- 2026-10-21 (the adventure lane, E08 T32a, `adventure/e08-t32a`, `cross-lane` with the user's consent, E08 decision 67; for your information, nothing to do): `h3-experiment`'s `[map]` gained `level_limit` (the scenario's) and `[[map.object]]` (a visitable object without a body, the Cartographers so far: `h3_map::writer::Plain`, `PLAIN_OBJECTS`; the tiles it covers stay free of heroes, monsters, piles and towns); `Tables` gained `plains`; `docs/experiments/README.md` documents both and lists batch E4.

- 2026-10-20 (the adventure lane, E08 T27, `adventure/e08-t27`, `cross-lane` with the user's consent, E08 decision 62; for your information, nothing to do): `h3_heroes::HeroDetails` gained `offered` (`OfferLevels`, the levels of the last Wisdom and school offers) and the details' validation (so `derive_battle_hero`) now refuses an experience above 1 810 034 206 and a level above the experience's level (HER-EXP-002: level 74 is the highest). Your `h3-pack-battle` tests were changed with it: every literal gained `offered: Default::default()`, a hero above level 1 carries `h3_heroes::experience_for(params, level)`, and the random heroes of `harvested_heroes.rs` and `harvested_sieges.rs` take levels 1 to 74 (was 255). A new test hero above level 1 needs its experience too.

- 2026-10-20 (the adventure lane, E08 T26b, `shared/e08-t26b`, `cross-lane` with the user's consent, E08 decision 61; for your information, nothing to do): **T26 is built** (T24b2's note of the same day). `packs/{base,ab,synthetic}/core.toml` gained the hero data from the harvester's gap-fill: `[hero_params]`'s `experience`, `max_level`, `wisdom` and `schools` (`[heroes.params]`; `HeroPacks.skills` carries the skills' keys to `write_hero_params`), every `[[hero_class]]`'s `offer_intervals` (`[heroes.offer_intervals]`), `ab`'s `[[hero_class_override]]` for the Elementalist under `quirk.content.elementalist_offer_intervals` (`[[data_quirks.class_intervals]]`, ADV-QUIRK-10); pack format v44, core ruleset hash v46; two flags at registry positions 27 (`quirk.adventure.level_up_from_seed`) and 28, both in the harvested profiles (so BAT-QUIRK-25's `quirk.battle.original_ai_slips` takes the next free position, 29) (the strict profile's comment gained the missing BAT-QUIRK-24). `HeroClassDef` has a new required field (`offer_intervals`) and `HeroParamsDef` four: your test packs in `h3-battle`'s `tests/support.rs` and `h3-content-battle`'s `tests/spells.rs` gained the field. Your ten replays keep their commands and state hashes; only `ruleset_hash` in their headers moved. Still to come: **T32a** (round 4) gives `h3-experiment`'s spec the map's level limit and, if the writer refuses it, two grounds on one tile (batch E4, survey `E08-progression.md` §7).

- 2026-10-20 (the adventure lane, E08 T22b, `adventure/e08-t22b`; for BAT-SET-001, nothing to do now): **the formation tables read in the code.** `battle_war_machine_stacks` (0x463600, from 0x463652) places each army in slot order on `battle_army_hexes` (0x63D0A8: the attacker 1, 35, 69, 86, 103, 137, 171, the defender 15, 49, 83, 100, 117, 151, 185) at the index `loose_formation_slots` (0x63D118) or `tight_formation_slots` (0x63D1DC) gives for the number of stacks: both match BAT-SET-001's pack tables row for row, so its tight table no longer rests on VCMI and the wiki alone (an `original-data` source when you next touch it). The tight table applies only when the side's hero has the formation bit (+0x48 bit 0) **and** its player is human (+0x54A4): a computer player's hero always fights loose (a quirk candidate for E23 or E10; the battle reads `BattleSetup.formations` and `human` as the game sets them). With its argument set the function takes another table (a creature bank's, not read). The adventure's side: `ADV-QUIRK-8` (the map's tight flag reaches no hero in the original; the engine keeps it), survey `E08-battles.md` §6.4.

- 2026-10-19 (the adventure lane, E08 T22a1, `adventure/e08-t22a1`, `cross-lane` with the user's consent; for your information, nothing to do): `h3-experiment` gained batch E3's shape: `[map] difficulty` (the template's line), `[map.underground]` with its patches (`rock` too), an encounter's `level`, a monster's `disposition` and `flees` (the defaults keep the old savage monster that never flees, so the committed specs build the same maps), a pile row's `level`, `guards` and `question`; `h3_map::writer` the `Underground` level and `Pile.guards`. The spec format in `docs/experiments/README.md` (yours) says so.

- **From the AI lane (2026-10-19, E05 T7, `cross-lane` with the user's consent; for your information):** `h3_battle::preview` (BAT-DMG-011, AI-Q6) reads an attack's or shot's first strike before it is made, through `combat::strike_preview` on the real strike's stages (`strike_inputs`) without the rolls. A change to the strike's stages flows into it; `tests/preview.rs` checks that a real strike lands within its preview. It shows no retaliation, second strike, secondary target or reflection (the user): keep it so unless AI-Q7 says otherwise.

