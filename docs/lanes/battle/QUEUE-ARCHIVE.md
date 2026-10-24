# Battle lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/battle/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

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

