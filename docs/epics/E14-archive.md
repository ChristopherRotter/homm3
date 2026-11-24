# E14 — archive: closed increment 1

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
