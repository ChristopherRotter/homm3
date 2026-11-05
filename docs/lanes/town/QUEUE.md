# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. Round 8
(the user, 2026-11-04): T8a cut in two and the order changed: T8a1, T7b2 (cut in two: T7b2a, T7b2b),
T8a2, then T8b (rounds 2 to 6, T7a, T7b1, T8a1 and T7b2a are done, `E09-log.md`); each later round's entries are written when it comes
(the epic's list), each task's cut confirmed at its start. Every entry's
acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T7b2b — the main towns' heroes** (`cross-lane`)

- **Scope:** design §14 step 3 (T7b2 cut in two at the store, the user, 2026-11-04; T7b2a built
  steps 2's towns and 4 and the store, `E09-log.md`): each main town's hero when it generates one
  (TWN-SET-009: a human player's pick, else the draw `HeroPool::draw` with the player's resolved
  faction, colour order), on the town's entrance as its visitor, its hero id after the setup's
  heroes in colour order, its army and experience the type's placeholders (GAM-SET-001,
  GAM-HER-001, drawn in hero id order with the existing ones); its draws between step 2's
  (`town::draw_towns`) and step 4's (`town::build_towns`); the unused picks freed after it (today
  at the end of `town::draw_heroes`); `town::main_town_heroes` and `TWN-STUB-3` lifted. No version
  moves (those maps are refused until it); a replay of a shipped map with a main town's hero if
  the census finds a small one.
- **Acceptance:** as T7a's; the determinism replays green; the census's 125 maps refused for a
  main town's hero build or are refused by a typed check.
- **Depends on:** T7b2a (its pull request merged). **Status:** next; its cut confirmed at its
  start.

**E09 T8a2 — the step onto a town** (`cross-lane`)

- **Scope:** design §9 and §11 (TWN-VIS-001, -002, TWN-CAP-001, -002): the step's three cases by
  the owner's team, the garrison and the visitor (entered; defended: visited from the tile before,
  its siege T8b's; undefended: entered and captured), the visitor on the entrance, the reach and the
  legal moves (a defended entrance as a monster's tile; measured against `master`, skill
  `homm3-bench-regression`), `AdventureState::capture_town` and the game's composition
  (`h3_towns::reveal`, the Capitol's limit, `TownCaptured`, `TownChanged`); `TWN-STUB-4` lifted.
- **Also:** `TWN-STUB-5` (narrowed by T7b2a): a town that reveals every tile (a Tower's Grail)
  reveals only its own level at the start (`h3_game`'s `town::start_radius`, the radius 255); the
  capture's reveal needs the adventure to take a whole-map reveal, the start's then too.
- **Depends on:** T7b2b (the main towns' heroes). **Status:** waiting; its cut confirmed at its
  start.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-05 (the adventure lane, E08 T95b, `adventure/e08-t95b`; **for a town's recruitment,
  E09 decision 4**): the hire dialog is built to be reused (increment 10's design note §13,
  "E09's reuse"). `h3_adventure::HireDialog { target: HireTarget, source: HireSource }`: extend
  `HireSource` with a town's dwelling and `HireTarget` with the garrison; the map's purchase rule
  stays one (`AdventureState::hire_stock`, `hire_room`, `check_hire` → `HirePurchase`, `hire`;
  ADV-HIR-002), and so does the game's decision (`h3-game`'s `state/hire.rs`: `Decision::Hire`,
  `Answer::Hire(HireAnswer::{Buy, Close})`, the legal set of each creature's most, `hire_range`,
  `DecisionView::Hire`, `GameState::hire_dialog`; GAM-DEC-008). The env's four purchases (action
  encoding v7) take the source's order, so a town's dwelling of up to four creatures needs no
  new entry. Game state hash v25 is T95b's.
- 2026-11-04 (the adventure lane, E08 T94b, `adventure/e08-t94b`; **for the towns' external
  dwellings' growth**; its versions half done by E09 T8a1, v21): A dwelling's counts and guards as
  they stand:
  `AdventureState::dwelling_counts` and `dwelling_guards` (by object index; none of a block while
  every dwelling is as its week began); the week's refill of every dwelling is the first step of
  `AdventureState::start_week`, before the weekly objects and the camps, reading no week's or
  month's kind (ADV-DWL-003; your towns' growth is separate, survey `E08-dwellings.md` §6.1).
- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **for the towns' external
  dwellings' growth**, E09 survey §8.3; adventure design note §11): the static map lists the
  dwellings in object order (`StaticMap::dwellings`, `dwelling_index`; a random dwelling not tied
  to a town is resolved into its drawn faction's and level's dwelling when the game is built, so
  it is listed as a placed one), their owners are `AdventureState::owner` (ownable since T93a, the
  setup's owner at the start) and their creatures the kind's (`ObjectKind::Dwelling::creatures`,
  each creature's faction the core's). Their counts and guards come with T94b (adventure state hash
  v20). A random dwelling tied to a town still waits for your random towns (ADV-STUB-13); its draw
  comes after theirs in the placement walk (ADR-0017 point 4).
- 2026-11-04 (the adventure lane, E08 T93b, `adventure/e08-t93b`; **for T7a, the import**):
  `h3-map`'s `import/codes.rs` has `FACTIONS`, the original's faction codes 0 to 8 to the core
  factions' keys (`base:castle` … `ab:conflux`), for the towns' import to reuse (E08 design §4,
  §11). A random dwelling of class 216 or 217 whose town identifier names a town of the map stays
  an untyped blocker (ADV-STUB-13, epic E09: it takes the town's drawn faction once random towns
  are resolved); one whose identifier no town has is refused (`ImportError::DwellingTown`). Both
  bump map import (T93b v16, E09 T7a1 v17).

- 2026-11-04 (the client lane, E17 M6 T3c3, `client/e17-m6-t3c3`; **when you change
  `h3-towns`' or `h3-pack-towns`' public API**): the battle client's binary now compiles the packs'
  town part (`h3_pack_towns::compile`) and reads `TownRuleset::town`, `TownRecord::building` and
  `buildings`, `Building::key` and `slot` and `BuildingId::from_index` (a siege's buildings and a
  Grail building by key in the stats dialog, the original's map by slot; design E17 §12.11). CI's
  clients job watches both crates (#704): a change there runs `clients/scripts/check.sh`, locally
  `scripts/verify-once.sh clients --base origin/master`; a dependency change updates
  `clients/Cargo.lock`.
- 2026-11-03 (the battle lane, CLT-Q40; its siege half done by T6b, `QUEUE-ARCHIVE.md`): the
  owner's Grail building that adds a hero's morale or luck (the Colossus, the Spirit Guardian)
  names itself as `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when
  the game gives it.
- 2026-11-03 (the adventure lane, E08 T87c, ADV-Q41; **for your increment 3's recruitment, nothing
  to build now**): E08's increment 10 is dwellings and hiring (decision 231): the map's creature
  dwellings (17, 20), the random dwellings with a faction set and the Refugee Camp, their weekly
  creatures and **the hire dialog**, built in the adventure lane first (E09 decision 4); the random
  dwellings tied to a random town wait for your random towns. Its survey (T88a,
  `docs/rules/surveys/E08-dwellings.md`) and design note (T88b) come first; this lane's inbox is
  told when the hire dialog lands.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 3 (weights: only the tavern's draw reads them, survey `E09-towns.md` §4.4), increment
  4 (chances).
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 3.
- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 3, with E10.
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 4.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 4.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 4.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 4.
