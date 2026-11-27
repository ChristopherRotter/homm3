# E14 — archive: closed increments 1 and 2

Moved out of `E14-original-map-generator.md` on 2026-11-24 at increment 1's checkpoint (RMG-Q2,
T1c3), verbatim, so that the epic file holds what is open and the decisions that apply to the
whole epic (decisions 1 to 8 stay there). Each task's full record is in `E14-log.md` (grep its
id).

## Increment 1 — the reading's map, the oracle, the survey (refined)

| Task | What | Out |
|---|---|---|
| T1a1 | The generator's frame: `random_map_generate`, 0x54BF00, 0x54C090, the constructor and its settings, `GenerateMap`'s phases in order, `WriteMap`'s outline; H3API's `H3RMG` records ported into `structs.toml` and checked against the code | `map.toml` entries; the survey's §1 (phases, records, files read) |
| T1a2 | Every function of the closure inside 0x530EE0–0x54D0F0 named at least `inferred` and placed in a phase; the 250 outside classified (mapped already, table and object helpers, I/O); every `rand` draw site and every float site listed per phase; any clock or iteration limit found (a wall-clock limit would break the oracle) | `map.toml`; the survey's §2 (per phase: reads, draws, floats, size) |
| T1b1 | The oracle (T1b cut by the user, 2026-11-23): the original generating under Wine with `time()` fixed (the capture harness, `clients/capture/`; a fake clock), the choices given (no Random); twice the same seed gives the same bytes, another seed other bytes; whether the memory-dependent draws vary; which archive's `rmg.txt` the game reads; `h3_map`'s reader reads each map; the corpus's spec (sizes, levels, players, teams, water, monsters, seeds) | `docs/lanes/rmg/ORACLE.md`; `clients/capture/h3rmg.py`; the corpus and its hashes (`clients/capture/rmg/`) |
| T1b2 | The phase oracle: every draw of a generation logged with its site and value (`xdd.dll`'s logged sites over survey §2.6's), and the generator's cells dumped after each of the 15 steps (an oracle for increments 3 to 5 before the file is complete); in the same sandbox as T1b1 (its staged set) | ORACLE.md; the harness's logs and dumps |
| T1c1 | The painters outside the range (T1c cut by the user into three, 2026-11-24): the terrain painter (steps 3, 6, 11) and the road and river painter (14, 15) read, their draws in order, checked against the phase oracle; read in thirds, two by worker sub-agents | the survey's §2.7; `map.toml` |
| T1c2 | The survey's §3 (the template format and both tables' differences, each phase's rules, the objects placed against the engine's and the writer's coverage); the quirk ledger `docs/rules/quirks-rmg.md`: every candidate of §1, §2 and ORACLE.md classified with the user, the memory-dependent ones among them | survey §3, the ledger |
| T1c3 | The ADR draft (the crate, the seam to `h3-map`, the `rand`, the floats under `kernel::num`, the pack data, the profiles, the generator's version); the cut of increments 2 to 6 with exit criteria; the checkpoint RMG-Q2 | ADR draft, this file |

**Exit criteria:** (1) every function of the generator's range in the map, at least `inferred`,
with its phase; (2) every draw and float site listed; (3) the oracle generates a map for a given
seed and choices, reproducibly, and the reader reads it; (4) the survey and the ADR draft
written; (5) increments 2 to 6 cut with exit criteria and confirmed at the checkpoint.

### Tasks done (increment 1)

- **T1a1** (2026-11-23): the frame read (`random_map_generate`, the parameters and the run's
  codes, the constructors and the seed, the files read, `rmg_generate_map`'s template draw,
  positions and 15 steps, `rmg_write_map`'s outline with the teams drawn at write time); 46 map
  entries, 12 records in `structs.toml`; the survey's §1.
- **T1a2** (2026-11-23; cut by the user into T1a2a to T1a2e, five worker sub-agents in one
  session): every function of the range placed (254 of its 255 function starts with an entry,
  237 read, the last a false start with a site entry); the steps named from their reading; the
  outside callees classified per part; every draw site entered and ordered (survey §2.6), the
  floats (x87, every conversion truncating) and the limits (no clock but the seed, one retry
  loop in step 10, two memory-dependent draws); the survey's §2.
- **T1b1** (2026-11-23; T1b cut into T1b1 and T1b2 by the user): the oracle (`xdd.dll`'s fixed
  date `H3TIME` for `time()`, `clients/capture/h3rmg.py`, a corpus of 21 cases); every map the
  same from fresh starts, in a second generation in one game and after another history (no
  memory-dependent variation under Wine); SoD's `rmg.txt` is the one read (21 of 21 templates
  predicted, AB's 13); the plain writer as read; `h3_map` reads all 21, 20 import and validate
  (one refused by TWN-SET-001, for T1c). `docs/lanes/rmg/ORACLE.md`.
- **T1b2** (2026-11-24): the phase oracle (`xdd.c` section 10, `H3RMG`: every draw of the
  generation logged with its site, the cells dumped after each step, the hooks on their own
  stack; `h3rmg.py --phases`, `phases`; `rmg/phases.txt`); 21 of 21 reproducible, no map changed
  by the hooks; a third of the stream is the terrain and road painters' draws outside the range
  (not in survey §2.6); 0x532A5B draws in step 2 in 18 of 21; `l1-4p-b`'s second map in one game
  differs by it, moved by the environment's size; cell bits the reset leaves to memory. ORACLE.md,
  "The phase oracle".
- **T1c1** (2026-11-24; T1c cut by the user into T1c1 to T1c3, T1c1 read in thirds: the runner
  one, two worker sub-agents one each): the painters outside the range read (survey §2.7): the
  terrain painter (the frame, the ten picture sets, adding cells to two row-major point sets, the
  pinched cells, the last pass's transition rule over the whole map) and the road and river
  painter (the stroke, the shape by the neighbours, the redraws); 148 map entries (139 new); their draws'
  rows for §2.6 (§2.7.5); checked against the phase oracle (`s1-2p`, `s2-2p`, `m1-4p`): every
  terrain, road and river cell's picture as the rules give. Nothing memory-dependent reaches a
  result (the cell words' fill is never read before a load); nothing ordered by address.
- **T1c2** (2026-11-24; no cut, the user: three worker sub-agents for survey §3's parts, the
  ledger with the user): survey §3: the template format and both tables (AB's a byte prefix of
  SoD's, three templates added), 136 rules by phase with provisional ids, the objects placed
  against `h3_map`'s reader and writer (a census of the corpus, seven writer gaps to the adventure
  and town lanes); the ledger `docs/rules/quirks-rmg.md`, RMG-QUIRK-1 to -23 (the memory-dependent
  draws modelled on the oracle, two flags; the faults fixed; 14 map-content flags and one
  cosmetic flag; 8XM8's ids a data quirk with a flag, the neutral main town as the original); TWN-SET-001's refusal explained.
- **T1c3** (2026-11-24): the ADR draft (`docs/lanes/rmg/adr-drafts/h3-rmg-generator.md`: the
  crates, the seam through `h3_map::writer` with its seven gaps closed, `CrtRand` in the kernel,
  the floats as binary64 and integer square roots, the pack data, the 18 flags in the game's
  profiles, the versions, the trace); increments 2 to 7 cut in the run's order with exit criteria,
  increment 2's tasks refined; the checkpoint RMG-Q2 (the user: the writer's gaps closed, the
  picture names harvested, the game's profiles, six increments). Increment 1 closed.

## Increment 2 — templates, the stream, the start (closed 2026-11-25)

Moved out of `E14-original-map-generator.md` on 2026-11-25 at increment 2's checkpoint (decision
10), verbatim: its row of the increments' table (which stays there, marked closed), the exit
criteria every increment shares (they stay there too), its task table and its tasks done.

| Inc | Scope | Rules (survey §3.2) | Ledger | Not read, read first | Its own exit |
|---|---|---|---|---|---|
| 2 | **Templates, the stream, the start:** the crates, `CrtRand`, the three tables harvested, the rmg part, the settings, the template's choice, the positions | SET-001 to -009, TPL-001 to -008 (17) | -6; -11 (the tables' half); -22 | none | the 21 cases' kept and drawn templates (`kept.txt`'s names) and the draws before step 1 equal; the harvested tables equal the text tables (with the original data) |

**Every increment's exit criteria**, beside the Definition of Done: (a) its rules in the catalog,
implemented, each with a test; (b) on the 21 corpus cases in `faithful`, the port's draws and its
cells after each of its steps equal the phase oracle's (`clients/capture/rmg/phases.txt`'s
digests in CI, the whole logs and dumps beside the oracle; a divergence named by step and draw);
(c) its ledger entries confirmed by (b) or re-read, each flag's `strict-fixed` behaviour tested,
its `exploit-fix` cases typed errors; (d) a property test: arbitrary settings, seeds and the packs'
tables run its steps without a panic; (e) its "not read" items (survey §3.2.15) read first, into
the survey; (f) the checkpoint with the user.

**Increment 2's tasks** (refined with the cut):

| Task | What | Out |
|---|---|---|
| T2a | `shared/`: `h3-rmg` and `h3-pack-rmg` created (`Cargo.toml`, `arch.toml`, `lanes.toml`), `h3_kernel::rng::CrtRand` with test vectors, the context in `docs/03-domain.md`, the catalog context `RMG`, the ADR numbered | the crates, the kernel's generator, the ADR in `docs/adr/` |
| T2b | `cross-lane` (`h3-harvest`): `rmg.txt`, `objects.txt` and `rand_trn.txt` into `packs/ab/rmg.toml` and `packs/sod/rmg.toml`; `h3-pack-rmg`'s parser; the rmg part's compiler and hash; 8XM8's gated override and its flag; `packs/README.md` | the packs' `rmg.toml`, the rmg part |
| T2c | The settings and their checks (RMG-SET-001 to -009, RMG-QUIRK-6), `resolve_random`, the templates kept and drawn and the positions (RMG-TPL-001 to -008, RMG-QUIRK-11's flag), `Trace`; checked on the 21 cases | `h3-rmg`'s start, the increment's exit |

### Tasks done (increment 2)

- **T2a** (2026-11-24, `shared/e14-t2a`, `cross-lane`): `h3-rmg` and `h3-pack-rmg` created empty
  (`h3-game → h3-rmg` forbidden), `h3_kernel::rng::CrtRand` (KER-RNG-004, the code reading's
  vectors), the context *map generation*, the catalog context `RMG`, ADR-0029 numbered.
- **T2b** (2026-11-25, `shared/e14-t2b`, `cross-lane`; no cut, a worker for the re-hash): the rmg
  part (`h3_rmg::RmgRuleset`, `RMG_RULESET_HASH_VERSION` 1), `h3-pack-rmg`'s parser and
  `packs/synthetic/rmg.toml`, the three tables harvested into `packs/ab/rmg.toml` and
  `packs/sod/rmg.toml` (`h3-harvest`'s `rmg`), `quirk.content.rmg_8xm8_zone_ids` (registry 51, the
  profiles) with 8XM8's six gated overrides (RMG-QUIRK-22), `packs/README.md`'s format, the
  `original_data` check (the harvested tables equal the text tables).
- **T2c** (2026-11-25, `shared/e14-t2c`, `cross-lane`; whole, decision 9): the settings and
  `resolve_random`, the templates kept and drawn, the positions, `generate`'s frame (`Trace`,
  `GenerateError`, `RMG_GENERATOR_VERSION` 1; the steps RMG-STUB-1), the executable's constants
  harvested (`[generator]`, `RMG_RULESET_HASH_VERSION` 2), `quirk.rmg.roe_monster_town_byte` (52) and
  `quirk.rmg.unowned_start_seventh` (53, RMG-QUIRK-24), the registry at 128 flags
  (`RULESET_HASH_VERSION` 54), the catalog's first 17 rules; the 21 cases' templates and draws
  before step 1 the original's.
- **The checkpoint** (2026-11-25, decision 10): the exit criteria (a) to (e) held (the record:
  `E14-log.md`, "Increment 2's checkpoint"); RMG-QUIRK-11 and -24 confirmed by two readings, not
  yet by the oracle; increment 3 cut into T3a to T3e. Increment 2 closed.

## Increment 3 — zones and terrain (closed 2026-11-27)

Moved out of `E14-original-map-generator.md` on 2026-11-27 at increment 3's checkpoint (decision
11), verbatim: its row of the increments' table (which stays there, marked closed), its task table
and its tasks done.

| Inc | Scope | Rules (survey §3.2) | Ledger | Not read, read first | Its own exit |
|---|---|---|---|---|---|
| 3 | **Zones and terrain:** steps 1 to 3, the terrain painter (§2.7.1 to §2.7.3) | ZON-001 to -014, TER-001 to -003, TER-005 to -018 (31) | -1 (the measurement, the model); -3 (its steps' cases); -12; -14; -21 (the painter's) | ZON-009's triangulation, ZON-012's clip corners, ZON-013's fill tail, ZON-014's water test; whether a divide by zero reaches code 1 | first the harness records each step's draw count and digest and RMG-QUIRK-1's bytes (`phases.txt` again); the control word read; the cells after steps 1 to 3 (the terrain layer) equal |

**Increment 3's tasks** (refined at increment 2's checkpoint, decision 10; in this order, each
about one session, the oracle's set staged from the PC for each):

| Task | What | Out |
|---|---|---|
| T3a (done) | The phase oracle again (`clients/capture/`, Wine): per step its draw count and its log's digest in `phases.txt` (a port of steps 1 to k compares its draws without the later ones); RMG-QUIRK-1's bytes, the temporary zone's and each water zone's unwritten fields (+0x40 to +0x49, +0x84, +0x94, +0xC to +0x18, its zone generator's +8) as `rmg_zone_generator_init` 0x5329E0 finds them, per case, and what moves them (T1c2's `m2-4p`: repeated corpus runs, the environment's size); the x87 control word at the generator's start (ADR-0029 point 4); corpus cases drawing 2SM2i(2) with three players and 2SM0k (their seeds found by the port's draw) for RMG-QUIRK-24 | `phases.txt`'s new format (ORACLE.md), the measurement (ORACLE.md, the ledger's -1), the corpus with its new cases (`kept.txt`, the T2c corpus test) |
| T3b (done) | Exit (e), the items not read, into survey §2.2 and §3.2.4: the triangulation's insertion and finish (`triangulation_insert` 0x5FD790, `triangulation_finish` 0x5FDB40: whether Delaunay, what +0x1C and +0x20 hold), the fill's tail 0x53D640, the water zones' distance test and `rmg_zone_graph_distances`' order, the corner cases of `rmg_zone_borders_draw` and `rmg_segment_clip`, whether a division by zero reaches code 1 (RMG-QUIRK-3) | the survey, ZON-009 and -012 to -014 rewritten, `map.toml` |
| T3c (done) | Step 1 (RMG-ZON-001 to -008): the zone generators, the layout tables (`rmg_layout_cos`, `_sin`, the terrain towns), the generator's cell record (the dump's 12 words), the comparison seam (ADR-0029 point 8: the port's cells in dump format 1 and its draws as the log's lines, their digests against `phases.txt` per step in CI); RMG-QUIRK-12, -3's step 1 case (an empty candidate list) a typed error; the increment's four flags registered in one re-hash (`quirk.rmg.zone_memory_model`, `terrain_towns_count`, `partition_units`, `cosmetic`; decision 10, `cross-lane`) | the cells after step 1 and the draws to it equal on the corpus |
| T3d (done) | Step 2 (RMG-ZON-009 to -014): the triangulation, the borders, the fill, the water zones and their connections; RMG-QUIRK-1's oracle model from T3a's measurement (its form decided at the increment's checkpoint, ADR-0029 point 6), -14, -3's step 2 cases; T3b's corner cases (the ledger's open table: the edge walk's chords checked against the oracle's step 2 cells) | the cells after step 2 and the draws to it equal |
| T3e (done) | Step 3 and the terrain painter (RMG-TER-001 to -003, -005 to -018; survey §2.7.1 to §2.7.3): the boxes and centroids, the islands' coasts, the painter's sets, pinches and transitions; RMG-QUIRK-21's painters' half | the cells after step 3 and the draws to it equal; the increment's exit (a) to (e) |

### Tasks done (increment 3)

- T3a (2026-11-25): the phase oracle's memory record (the x87 control word 0x027F, step 2's zone
  bytes at `rmg_zone_generator_init`), `phases.txt`'s per-phase draws and digests, `zones.txt`,
  the oracle's game environment fixed (`H3_FIXED_ENV`), the corpus at 24 (RMG-QUIRK-24's three
  cases, confirmed), what moves RMG-QUIRK-1's bytes (one agent); log `## T3a`.
- T3b (2026-11-26): step 2's items not read, read (code reading only): the triangulation is
  Lischinski's incremental Delaunay one in integers, +0x1C/+0x20 the left triangle's circumcentre;
  the borders' walk and edge walk, the clip (a side's line), the fill (4-connected), the water
  connections and their distance test; code 1 is only the file's open failure (RMG-QUIRK-3 corrected);
  five corner cases into the ledger's open table; log `## T3b`.
- T3c (2026-11-26, whole at the user's choice): step 1 ported (RMG-ZON-001 to -008, `layout.rs`),
  the generator's map (`Cells`, RMG-SET-010) and the comparison seam (dump format 1, the draws'
  digests against `phases.txt` per phase in CI); the oracle's zone generators after each step
  (`layout.txt`, the user's addition): 388 of 388 the port's on the corpus; the layout tables
  harvested (`RMG_RULESET_HASH_VERSION` 3); the four flags at 56 to 59 (one agent; `faithful`'s
  core hash re-pinned, `cross-lane`); RMG-QUIRK-12 confirmed, -3's step 1 cases typed; log
  `## T3c`.
- T3d (2026-11-26, whole at the user's choice): step 2 ported (RMG-ZON-009 to -014,
  `triangulation.rs`, `partition.rs`): the step's draws and cells equal the oracle's on 24 of 24
  with `zones.txt`'s bytes (22 with the provisional model's own), the water zones `water.txt`'s
  (the oracle's new record, 44 in 4 cases); RMG-QUIRK-1's provisional form, -14 confirmed, -3's and
  -4's step 2 cases typed; T3b's five corner cases counted on the corpus (the ledger's open
  table); log `## T3d`.
- T3e (2026-11-26, whole at the user's choice; the harvest `cross-lane`): step 3 and the game's
  terrain painter ported (RMG-TER-001 to -003, -005 to -018, `painter.rs`, `terrain.rs`), the
  picture sets harvested (`RMG_RULESET_HASH_VERSION` 4), `rmg_island_fill` read: the step's draws
  and cells equal the oracle's on 24 of 24 at the first run; RMG-QUIRK-21's painter's half on its
  flag, -3's and -4's step 3 cases typed; increment 3's exit (a) to (e) met; log `## T3e`.
