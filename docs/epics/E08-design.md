# E08 design note: increment 5

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 5's amendments
of ADR-0013 (point 13), ADR-0016 (points 3–5) and ADR-0017 (points 1, 3, 9), drafted in T34b from
the answers at ADV-Q20 (the epic's decision 75) and at T34b (decision 77: D-26 revised), accepted
at ADV-Q21 with the cut (decision 78; ADR-0016's with A-13's caveat restated) and applied in T35a
(#377). The task that builds a section
deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-hero-movement.md` (§ numbers are the survey's unless they name this note).

Increment 4's note was moved by T34b into `E08-log.md` ("Increment 4's design note at its
closure"): every section was built (T25 to T33); the code's citations of it were repointed (T35b,
T39, T40).

Written whole (decision 76): **part A**, the hero side (`h3-heroes` and the core content); **part
B**, the map, the game, the batch and the cut.

## Part A — the hero side (`h3-heroes`, the core content)

**Built** (the log): the core data in T35b (`shared/e08-t35b`: the six hero-effect kinds, the
gap-fill's effects, the flags 30 to 32; flag 30 to be retired in T41's shared part, `ADV-QUIRK-16`
withdrawn at ADV-Q22, decision 85), the derivations in T39 (`h3_heroes`' `movement.rs`:
`land_points`, `movement_speed`, `movement` with its `Movement` of the Pathfinding, flying and
water-walking levels, `scouting_radius`; HER-ADV-003 to -007 in `heroes.toml`; Q-0005's entry
names HER-ADV-003), and the adventure part in T40 (ruleset part v10: `[params]`' `fly_cost` and
`water_walk_cost`, the `ab` pack's `[[terrain_walker]]` for Nomads, both Cursed Grounds'
`no_native`; the log). Part B reads them: since T40 `day_points`, `Pricing`'s Pathfinding and the
reveals' radius (§5, §6); flying and water walking since T41 (§6, §7).

## Part B — the map, the game, the batch and the cut

Rules of `h3-adventure` and `h3-game` around part A's: when a hero's properties are read, how
its steps are priced and where it may pass and stop. ADR-0016 and ADR-0017 hold the
decisions.

### 5. The day's points and the radius; the build and the day (ADV-MOV-001, -002, ADV-FOG-001, -003)

**Built by T40** (the log): `day_points` from the store, `AdventureState::start_game` after the
levels at load, `start_turn` with the store, the radius per move, `ADV-QUIRK-18` and the strict
`AdventureState::start_day`, called in `advance_day` and for day 1. A step over a tile a flight
passes reveals as any step (T41's passage).

### 6, 7. The step's price; the passage, the reach and `MoveHero` (ADV-MOV-005 to -007, -009, -010)

**Built by T40 and T41** (the log): `Pricing` with the hero's Pathfinding, a terrain walker and
Cursed Ground (T40), the fly and water-walk prices, the diagonal's check and the drop under
`ADV-QUIRK-17` (T41, `Pricing::lifted`); `AdventureState::passage` (this note's table, now in its
doc comment and ADR-0016 point 4; `ADV-QUIRK-15`), `MoveHero`'s passage with `NoStop` and a
flight's guard and visit at its end, the reach's pass nodes (`Reach::search_layered`). The
exhaustive search's equality without the diagonal exception was measured (the log), not added as
a test, as for the walker in increment 1.

### 8. The env, replays, fixtures, properties and benchmarks

**Built by T42** (the log): the plans' kits, flyers and water walkers beside water and Nomads, a
move's end where its hero may stop, `the_plans_fly_and_walk_on_water`; the duel's and XL map's
kits; the env's test of a flyer's and a water walker's masks; the replays re-recorded (six
searched, one added); the reach workload's walkers and lifted heroes apart (`reach-xl-144`,
`reach-lifted-xl-144`); the Tutorial's hero's day's points checked under `--with-original-data`.

### 9. Rule ids and quirks (the map side)

**Built** (T40, T41): ADV-MOV-001 to -007 and ADV-FOG-001, -003 amended, ADV-MOV-009 and -010 new
(`verified` by batch E5); the ledger holds `ADV-QUIRK-15`, `-17` and `-18` (`-16` withdrawn, flag
30 retired in T41's shared part).

### 10. Versions

Increment 4's last: core ruleset hash v46, pack format v45 (the battle lane's E23 T3-0), game
state hash v11, adventure state hash v10, adventure ruleset part v9, map import v8, action
encoding v4, adventure replay format v5. Increment 5: the pack format v46 and the core ruleset hash
v47 (T35: the six kinds, the three flags), the adventure ruleset part v10 (the fly and water-walk
prices, the terrain walker; Cursed Ground's `no_native` in the same bump if E5-6 is scored first,
which the cut does; bumped by T40). The game and adventure state hashes, the map import, the action encoding and
the replay format do not change (no new state; a flyer's destinations are tiles of the same
mask): T41 and T42 bumped nothing. Each bumps once, in the task that first changes it.

### 11. Batch E5

The plan is the survey's §9: rows E5-1 to E5-10, E5-8 dropped (A-14 withdrawn), E5-10's
prediction corrected by T34b (§4.2: nothing new after the battle nor at day 2; the radius 6 after
the first hero's step). *Agent's call (change freely):* **E5-11**, an Angel Wings hero facing a
border gate whose key it lacks, clicks a free tile beyond it; the reading predicts a path over the
gate (an entrance a flyer passes, §3.1 point 7), which no other source states (§13).

- **Spec and scorer: built** (T36a, T36b; decisions 80, 81; the log): batches E5a to E5e
  (`experiments/specs/E08-batch-5*.toml`, their templates in `experiments/results/`), E5-2 on two
  maps (a hero type once per map; the level-6 heroes with eight skills, since the evaluation at
  load takes Logistics whenever offered), E5-3 and E5-10 on a map banning all but Logistics and
  Scouting (the forced offers), the gains in the day from the backpack instead of a pile;
  `crates/h3-map/examples/e4_score.rs` scores E5-2's points (D-22) and E5-3's and E5-10's dialogs.
  Since T39 the scorer calls `h3_heroes::land_points` under `faithful` and `strict-fixed` (the
  same §1.3 vectors; its reports on batch E5 unchanged).
- **The harness's reads** (`clients/capture/`, client-owned; built by this lane, `cross-lane`, as
  T32b1): a tile's explored bit for a player (`explored:x,y`, the vision word against the player's
  bit; E5-9, E5-10, E5-11), a hero's points at a turn start without a walk (check whether a read
  step's `points` gives it), the next turn's start after `end_turn` (exists), the path's mark
  (exists); for E5-3b and E5-10b an artifact moved from the backpack to a slot in the hero screen
  (or the rows are skipped, as the survey's plan allows).
- **The run: done** (T37b, decision 83; the log): the recipes
  `clients/capture/batches/E08-batch-5*.toml`, every batch on the harness (decision 50) at its
  first run, E5c and E5d twice; the results and the scorer's reports in `experiments/results/`.
- **The scoring: done** (T38, decisions 84, 85; survey §13): every row the reading's but E5-4,
  whose inland road steps cost 50; the re-read found the tile bit the flying price tests means "no
  object blocks the tile", not the coast (§6 point 4). The same misreading had made A-14 and
  `ADV-QUIRK-14` (A-11, the computer's terrain shares): ADV-Q22 (decision 85) withdraws
  `ADV-QUIRK-16` and `-14`; ADV-MAP-002's count corrected by T38b (three golden replays' hashes moved).

### 12. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T35's `shared/` pull request (the three accepted drafts applied;
`h3-content`'s kinds and flags, `h3-pack`'s syntax; part A §4) with the gap-fill and the packs
(battle-owned) and both lanes' replays' headers as `cross-lane` in it, as T26b; batch E5's spec
(`h3-experiment`, `docs/experiments/`; T36) and the harness's reads and recipes (`clients/capture/`;
T37), `cross-lane`; the client lane's inbox if `clients/` needs the new kinds mapped. Everything
else is the lane's own (`h3-heroes`, `h3-adventure`, `h3-game`, `h3-map`, `h3-pack-adventure`, the
adventure packs, the env's and the CLI's adventure sub-paths, fixtures, replays, benchmarks).

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint). *Confirmed at ADV-Q21 (decision 78):* batch E5 runs **before** the build (rounds 1 and 2), so
that its answers (D-23's Cursed Ground above all, and D-22, D-24) shape the rules once instead of
reopening them, and nothing in the batch needs the engine's increment 5 (it tests the original;
the scorer needs only increment 4's draws):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T35** + **T36** | T35: the shared groundwork (`shared/`, `cross-lane`): ADR-0013, -0016 and -0017 amended from the accepted drafts; the six kinds, the three flags, their syntax; the packs' effects through the gap-fill; pack format v46, core ruleset hash v47; both lanes' replay headers; `clients/` checked (part A §1, §4). T36: batch E5's specs, results templates and scorer (§11; `cross-lane`) |
| 2 | **T37** + **T38** | T37: the harness's reads, the recipes and the run of batch E5 (`cross-lane`). T38: batch E5 scored, D-23 decided by E5-6; the survey's §8 and the rules' sources updated; an ADV-Q if a row contradicts the reading. *Added by decision 85:* **T38b** (done), ADV-MAP-002's count corrected (every tile no object blocks but rock, water included), `ADV-QUIRK-14` withdrawn, before round 3 |
| 3 | **T39** + **T40** | T39 (done alone, decision 86): `h3-heroes`' derivations (part A; HER-ADV-003 to -007, §1.3's vectors; Q-0005's line). T40 (done whole, decision 87): the day's points, the radius and the walking price (§5, §6 points 1, 2, 7): ADV-MOV-001 to -004 amended, the build's order, `start_turn` and the reveals with the store, `ADV-QUIRK-18` and the strict start-of-day reveal, Nomads, Cursed Ground (confirmed); adventure ruleset part v10 |
| 4 | **T41** (done whole, decision 88) | Flying and water walking (§6 points 3 to 6, §7): the price (the original's, decision 85; flag 30 retired and ADR-0016 point 4's words corrected in its shared part), the drop (`ADV-QUIRK-17`), the passage, the reach's two kinds of node, `MoveHero`'s passage, end, visits and guards; ADV-MOV-006, -007 amended, -009, -010; `ADV-QUIRK-15` and `-17`. The largest task: cut at its start if needed (the price and the drop; the reach and `MoveHero`) |
| 5 | **T42** (done whole, decision 89) + **T43** | T42: the env, the plans and properties, the fixtures and replays, the benchmarks, `--with-original-data` (§8). T43: the audit against the exit criteria, increment 5 closed, increment 6 refined |

*Not chosen* at ADV-Q21: the batch at the end as in increment 4 (T35, T39 to T42 first, then
the batch and its scoring: the order of increments 3 and 4, at the risk of reopening a built rule
and building Cursed Ground late).

**Increment 5's exit criteria**:
1. A hero's day's points follow its Logistics (a specialist's growth by Q-0005), its creature
   specialty's speed and its worn Boots of Speed and Equestrian's Gloves (HER-ADV-003, -004,
   ADV-MOV-001), set at the build after the levels at load and at each turn start of its owner
   (ADV-MOV-002).
2. A step's price follows the hero's Pathfinding, Nomads in its army, flying and water walking
   (the original's flying price, decision 85), and Cursed Ground (E5-6); the drop to 0
   follows `ADV-QUIRK-17`.
3. A flying or water-walking hero passes as §7's table, never over an unexplored tile
   (`ADV-QUIRK-15`), and ends a move only where it may stop; the reach reports those tiles with
   paths `MoveHero` accepts; the env, the replays and the plans play flyers and water walkers.
4. A hero's radius follows Scouting and its worn Speculum and Spyglass (HER-ADV-006); a grown
   radius shows at the hero's next step, in `strict-fixed` also at the next day's start
   (`ADV-QUIRK-18`).
5. Every new rule has its id, source and tests; `ADV-QUIRK-15`, `-17` and `-18` are in the ledger (`-16` withdrawn, flag 30 retired) and
   Q-0005's entry names HER-ADV-003; the gates pass (coverage, determinism with the re-recorded
   replays, the benchmarks with the walker's search at its baseline and a flyer's baselined).
6. Batch E5 run and scored; D-22 to D-26, A-13's original path and E5-11 `verified` or reopened.

### 13. Risks and open points

- **The passage table rests on the code reading alone** where the wiki and VCMI are silent: a
  flyer passing over a border gate without its key, over another player's hero, and a step up
  off an object not visited from above landing only in flight. Fixed tests pin the reading
  (`crates/h3-adventure/src/state/tests/flight.rs`, T41);
  E5-11 checks the gate in SoD; the wiki's Border Gate and Border Guard pages are worth a look in
  T41. E5-11 confirmed the gate (a flyer passes over its entrance without the key, no message).
  A-14's gap (0x4E5CE0) was the tile bit misread as the coast (T38): the test passes on every free
  land tile, so a flyer's start node is a walker's, as built.
- **A-13's cost** (the user at ADV-Q20 and ADV-Q21: "probably fix, but gameplay impacts have to
  be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like
  allowing to route to the closest unexplored, but free hex; TBD later anyway"): a flight over a
  wide obstacle range stops at its unexplored edge. Increment 5 builds the fix; the compromise is
  designed later (the queue's inbox), with what a flight's end beyond hidden tiles may tell the
  player (`docs/06-security.md`). Keep the reach's passage rules in one place, so that the
  compromise changes the unexplored tile's row of §7's table and nothing else.
- **The reach's speed:** the second layer only for flyers and water walkers; the walker's hot
  loop must not gain a branch (`bench-check`'s +2 %). T41: `reach` +0.89 %, `play` +0.83 % with
  the walker's own unlifted price (the log); a flyer's and a water walker's reach baselined by
  T42 (`reach-lifted-xl-144`).
- **Not in increment 5** (survey §6): the sea's points (Navigation, its specialists, the
  Necklace of Ocean Guidance, the Sea Captain's Hat, the Admiral's Hat, Lighthouses, Favorable
  Winds), Stables, the spells Fly, Water Walk and Dimension Door with their levels 0 to 2 (the
  day's end of a spell, `new_day` 0x4C80CF), the computer players' bonus (E10), the cheat flag,
  the new-day reveal after a Cover of Darkness (E09).
