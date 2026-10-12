# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`. Increment 1 (T1a–T10c, decisions 1–12) closed on 2026-10-12. **Increment 2** refined with the user the same day (T10c, ADV-Q9; decisions 13–18 in the epic): object types, the treasury, pickups, the Cartographer as the first pending decision, a few visitable objects. The user takes one or two tasks per round, the cut confirmed at each round's checkpoint. T11a, T11b (ADV-Q10 answered, the ADRs amended) and T12 (in two parts) are done; T12 waits for its checkpoint. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T13 — Treasury and pickups

- **Scope:** treasuries in `GameState` with starting resources (decision 16), the two visit modes in `MoveHero` (entering ends the move; a neighbour's visit), resource piles, random resources and campfires (gain, removal, events with their viewers), the treasury in the owner's observation only; the play properties: ownership isolation over treasuries (and each player's stream if a command draws, decision 17), conservation (a treasury changes only by a visit's rule), observation hygiene. T12b left `AdventureState::remove_object` (drop its `expect(dead_code)`) and the live tiles for the visits; `MoveHero`'s `Visit` error becomes the visit.
- **Depends on:** T12. **Status:** waiting (T12's checkpoint).

### E08 T14 — The Cartographer: the first pending decision

- **Scope:** the Cartographer's offer as a pending decision of its visitor's owner, the answer command (accept: the cost from the treasury, the level revealed; decline), legality while it is open (its answers only, no `EndTurn`), atomicity and legality consistency with decisions; the env's action encoding v2 (a decision head), the replay format if the new command needs it, golden replays re-recorded.
- **Depends on:** T13. **Status:** waiting (T13).

### E08 T15 — Visitable objects

- **Scope:** the Redwood Observatory and the Pillar of Fire (a reveal around the object), Keymaster's tents (a key per player and colour) with border guards (removed for everyone when a key holder accepts; the yes/no is a second offer if ADV-Q10's D-7 says so) and border gates (passable for such a player), the per-player object state in the observation.
- **Depends on:** T13 (visits); T14 too if the guard's question is an offer (D-7). **Status:** waiting (T13).

### E08 T16 — Closure of increment 2

- **Scope:** the duel and XL maps regenerated with objects, golden replays and the three benchmarks re-baselined (`02-architecture.md` §8); batch 2: the increment's disputes and D-3 (sand onto a cobblestone road with 200 points left, the survey's example; the writer lays east–west roads only and interior terrain pictures at edges, north–south roads when a batch needs them), with the spec stamp (decision 18: a short hash of the spec in the map's description and the template; `cross-lane` in `h3-experiment`); the exit criteria audited; scoring and the checkpoint with the user.
- **Depends on:** T14, T15. **Status:** waiting.

## Inbox

- 2026-10-12 (T12b): `ObjectView` has no object index (it would count unseen objects); T14's offer and its events name the object by its visitable tile or anchor, not by index, in what a player sees.
- 2026-10-12 (T12b, not measured): `remove_object` scans every object for each tile it frees (other objects covering it); a per-tile cover count in the static map if pickups show up in a profile.
- 2026-10-12 (T12a): the guards record of a pile counts even when empty (`Unsupported`); read the loader (0x4A4BF4's flag) with increment 3's guarded piles.

- 2026-10-12 (T11a): not read, each for the increment that needs it: the computer player's own handling of the Cartographer and observatories (the human path only), the guarded pile's path (0x4A4780, inc. 3), how the loader fits a pile amount above 0x7FFFF (T12 bounds it), the game flag +0x1F69D's 50/50 000 start (probably the tutorial), the computer players' daily resource bonus by difficulty (the wiki's Difficulty level; VCMI has a weekly one; inc. 4), the flying and boat test before a visit from a neighbour (0x4E5CE0, inc. 3), the wiki's "tournament rules" start on impossible (5/2/2 500, probably HD).
- 2026-10-12 (T11a): the original lets a hero visit the object it stands on again (the space bar); a "visit here" command would let a player re-ask a declined Cartographer without stepping off: with an increment that needs it.
- 2026-10-12 (T11a, T12b): `docs/06-security.md`'s viewer knowledge list (a shared path) gains increment 2's rows (survey §8) with T13's observation: typed objects shown whole once a covered tile is seen (T12b), contents never, treasuries the owner's only.

- 2026-10-12 (T10c): `docs/BACKLOG.md`'s E08 row still says "increment 1": update it (in progress, increment 2 refined) with the lane's next `shared/` pull request (T11b's accepted ADRs).

- 2026-10-11 (T9b, measured): the reachable-tiles query is 96 % of a random step (≈ 263 k Ir per query on the duel, 342 k on 144 × 144; of a query `Reach::search` 56 % on its own, the step price 28 %, the heap 7 %; zeroing map-sized arrays 5 % of a step): the first lever on the adventure's speed (`benches/adventure/` gates it), for an increment that needs speed or a performance task.
- 2026-10-11 (T9b, measured): an agent reading both mask heads before a step doubles the duel's step (70.9 → 141 µs; ×1.7 on 144 × 144): `legal_mask`, `tile_mask` and the step's decode each run the query. Keep the current state's queries in the env, with E05's batch API (or E13's macro-actions), which owns the mask API.
- 2026-10-11 (T9b, measured): `observe` rebuilds one `TileView` per tile: +15 µs per duel step (+21 %), +99 µs on 144 × 144 (+54 %). Cached static planes with E13's flat encoding. (Closed by the same measurement: the reveal's per-row writes, `fog::reveal` 0.8 % of a step.)
- 2026-10-11 (T9b): the battle workload's `collect = "*random_rollout*"` also names the adventure's `random_rollout`, harmless (the battle's run never calls it): the battle lane may qualify it (`*h3_sim::rollout::random_rollout*`) as hygiene.
- 2026-10-11 (T9a): a player's hero slots are its heroes by id, the first eight; they shift when a hero of a lower id joins or leaves (recruiting, defeat: increments 3 and later). Stable slots (a slot table in the game, as the original's hero list) when an agent needs them (E13), with an encoding version bump.
- 2026-10-11 (T9a, the user): the env's observation is `h3_game`'s view; a flat, versioned encoding comes with E13's macro-actions (or E05's batch API). Episodes end only by the day limit and carry no reward until victory and defeat exist (E10).
- 2026-10-11 (T8b2b): the hygiene test compares two games while the other players' commands have the same outcome in both (about four steps in five over 200 cases); it varies the streams only through the seed (no command draws yet, see the T8b2a note below).

- 2026-10-11 (T8b2a): moves are not listed one by one (the user): `h3_game::legal_moves` gives a hero's `Reach`. T9's env builds its mask from it (a bit per tile and hero; ADR-0017 point 8) and the path only for the action taken.
- 2026-10-11 (T8b2a, the user): clients compute their own paths and `apply` validates them. For the client and server lanes: the pathfinder on a player's observation instead of the state (moves go to explored tiles only, where the observation shows all the query reads: terrain, roads, objects, visible heroes, the own hero's army and points), or a server query for the reach of the player's own hero.
- 2026-10-11 (T8b2a): the play properties (`crates/h3-map/src/tests/play.rs`) take about 6 s for 3 × 48 cases in a debug build. `Plan` and `lay_out` are test code; the generated maps are written once by a test and committed (T9a, T9b).

- 2026-10-11 (T8b1): the fringe is the eight tiles around an explored one (ADR-0016's "touching"); how the original draws its half-dark border (which hidden tiles get an edge sprite) was not read: a code reading or a look in SoD with a single explored diagonal, with T10's batch.
- 2026-10-11 (T8b1): of another player's hero the observation shows tile, owner and identity (ADV-FOG-006, the user): survey what the original's right-click view shows (army, approximate counts) with the battle seam (increment 3). The spectator viewer for the adventure when the server lane hosts adventure games.
- 2026-10-11 (T8a): the Scouting specialty multiplies the radius by 1 + 0.05 × level (0x4E4070, truncated), not "+1 every six levels" (survey §2): compare with the wiki and VCMI with the adventure profile (increment 3).
- 2026-10-11 (T8a): the original's reveal sets the bits of every player of the revealer's team (allies share the map; VCMI too): with teams (E10). The new day reveals again around everything a player owns (0x4C7C50): with the first rule that hides tiles (Cover of Darkness, towns).
- 2026-10-11 (T7b): the step's price leaves out Pathfinding, Nomads, flying, water walking, Favorable Winds and Cursed Ground (0x4B1740 has them): increment 3 with the adventure profile, Cursed Ground with objects.
- 2026-10-10 (T6b review): building a game emits no events (the first `TurnStarted` goes to a `NoopSink`): a replay needs none (T9b: commands and hashes); a client's event stream (server lane) needs a start event or reads the start from `observe`. `GameState` keeps no hero types yet: the game's store of heroes (hero details) comes with `h3-heroes` in increment 3.
- 2026-10-10 (T6b): the original plays every human player before every computer player each day (0x4C6FE0; VCMI `TurnOrderProcessor` too). The engine has no human/computer distinction; decide with the server lane's seats or the turn-group work whether a schedule should order players so (GAM-TRN-001's note).
- 2026-10-10 (T6b): weeks of seven days and months of four (wiki Time; VCMI parameters) come with the first rule that reads them (week events, increment 4); GAM-CAL-001 counts days only.
- 2026-10-10 (T6b): the starting army's other callers (the tavern, 0x4C86A0 and 0x4D9EC0) give one creature of the first stack; for the towns epic. The war machine of a starting army (artifact 4 or 6; rolled at 88 % with its slot, ADV-Q6 answered by batch 1b) for increment 3 with the hero details.
- 2026-10-10 (T6b): `docs/formats/installation.md` §7.1 says the nine war machines are the heroes' *third* starting creature; the hero table has them second (battle-owned doc; tell the battle lane or fix it with the formats lane).

- 2026-10-10 (T5a/T6a): for E09 (towns): a hero placed on a town's blocked tile is the town's visiting hero (VCMI `CGameState::placeHeroesInTowns` moves it to its own town's entrance, an "H3M format quirk"; 3 shipped maps), and a hero generated at a main town (129 maps); both are `Unsupported` in the import until towns exist.
- 2026-10-10 (T5a/T6a): not checked yet, each for the increment that needs it: a hero type placed twice on one map, the most heroes a player may have on the map (8 in the original; with recruiting), a war machine in a hero's custom army (battle content; with the battle seam, increment 3).
- 2026-10-10 (T5a): `h3_formats::h3m`'s comment on `mask_from_text` says `Objects.txt` masks are "top left first"; they are anchor first (VCMI `readTxt`; `docs/formats/h3m.md` corrected). The writer's code is right; fix the comment when the writer moves into `h3-map` (battle-owned until then).

- 2026-10-10 (T4): check `packs/base/adventure.toml`'s land table against the installation's `movement.txt` under `--with-original-data` (an `examples/original_data.rs` in `h3-pack-adventure` needs `h3-formats` as a dev-dependency to read the LOD: a shared `arch.toml` change), or harvest the table when adventure data is harvested.
- 2026-10-10 (T3b): ~~fixtures in `fixtures/maps/` from the user's editor maps, so CI reads real editor output~~ dropped by the user (2026-10-11: too little gain for the time); reconsider only if a reader or import bug shows up that real editor output in CI would have caught. The shipped maps stay the reader's test, locally (`verify --with-original-data`).

- 2026-10-10 (T2): the game context's rule ids use the existing prefix `GAM-` (`check-rules`, catalog file `docs/rules/catalog/game.toml`), not `GAME-` as the T1b survey proposed.
- 2026-10-10 (T2): already allowed by `arch.toml`, so T5–T9 need no shared change: `h3-sim` → `h3-adventure`, `h3-game` (dev: `h3-map`, `h3-pack-adventure`); `h3-cli` → the four crates; `h3-map` dev → `h3-pack`, `h3-pack-adventure`. Golden replays go to `fixtures/replays/adventure/` with the test `crates/h3-cli/tests/adventure_replays.rs` (both adventure-owned): `cargo xtask determinism` runs it once it exists, and `verify --scope adventure` runs the determinism step once the folder has files (T9).
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator). ~~Keep it in the formats adapter so both can extend it~~: E08 decision 2 moves it into `h3-map` after E04b closes, beside the reader (2026-10-08).
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
