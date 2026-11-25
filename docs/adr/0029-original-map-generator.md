# ADR-0029: The original's map generator in `h3-rmg` (E14)

- Status: Accepted (user, 2026-11-24, RMG-Q2: the draft confirmed at the checkpoint after E14
  increment 1, its seam the experiment writer; numbered and moved here by E14 T2a, 2026-11-24)
- Date: 2026-11-24
- Deciders: the user (RMG-Q2) + the rmg lane's session (E14 T1c3)
- Amends with it: ADR-0003 point 7 (the original's generator beside `Rng`; its revision history)
- Working detail: the survey `docs/rules/surveys/E14-generator.md`, the oracle
  `docs/lanes/rmg/ORACLE.md`, the ledger `docs/rules/quirks-rmg.md`

<!-- Longer than the usual 80 lines: one record for the generator's crate, seam, stream, floats,
     data, profiles, versions and test seam, which E14's increments 2 to 7 all build on. -->

## Context

E14 ports the random map generator of `Heroes3.exe` (SoD's code, GOG Complete) so that the same
template table, choices and seed give the same `.h3m`, byte for byte (decision 1). Increment 1
read it (survey `docs/rules/surveys/E14-generator.md`: 255 functions, 15 steps, two painters
outside the range, 136 rules in §3.2), built the oracle and the phase oracle (`docs/lanes/rmg/
ORACLE.md`: 21 cases, each step's cells and the draw log as digests) and classified 23 quirks
(`docs/rules/quirks-rmg.md`, 18 flags). Decision 2 left the seam to `h3-map` open; ADR-0003's PRNG,
float and profile rules were written for the game, not for a port of the original's own stream.

## Decision

1. **Crates.** `h3-rmg`: a new context, *map generation*, in the domain layer, a logic crate
   (`no_std`, `check-arch`'s purity, the coverage gate); it depends on `h3-kernel` and the core
   `h3-content` only (creatures, heroes, spells, artifacts, factions, terrains by handle; the
   active profile's flags). It holds the settings and their checks, the rmg part and its compiler,
   the 15 steps, both painters as the generator calls them, the teams' draw and `GeneratedMap`.
   `h3-pack-rmg` (adapter) parses `packs/*/rmg.toml` (as `h3-pack-towns`). `h3-map` gains
   `src/generated*` (the rmg lane's paths, as the town lane's `import/town*`) and the edge
   `h3-map → h3-rmg` at increment 7. No other crate depends on `h3-rmg` but adapters.
2. **The seam** (RMG-Q2: the experiment writer, its gaps closed). One entry,
   `h3_rmg::generate(core, rmg part, settings, seed, trace)` → `Result<GeneratedMap,
   GenerateError>`: pure, never panics; the seed a `u32` (the original's `time()` in seconds).
   The settings hold every choice resolved: the original draws its Random choices from the game's
   earlier `rand` state, which no port reproduces, so `h3_rmg::resolve_random` applies the
   original's formulas (RMG-SET-001 to -003) to a stream the caller seeds. `GeneratedMap` is what
   `rmg_write_map` writes, in its order: the header and the description, the players and the drawn
   teams, the tiles as given, the template table as given (the two unused entries first), the
   objects in the writer's order with their bodies and the generator's serials. **The file goes
   through `h3_map::writer`:** `h3_map::generated::to_writer` turns it into the writer's `Map` and
   `Extras`, and `write_with` gives the uncompressed `.h3m` byte for byte (the gzip stream is not
   compared: the game's zlib at level 6 against miniz) once survey §3.3.3's seven gaps are closed
   (one object list in the caller's order, identifiers as given, the template table as given, a
   Seer's Hut's reward, the banned artifacts as a whole set, the AB layout and RoE's towns and
   `Extras`, the main town's type byte and a main town of another colour). **The game reads those
   bytes** as any map file (`h3_map::read`, `import`), with the untrusted input's checks: no
   generated-map path into `GameSetup`. The gaps are closed by the writer's owners from their
   inboxes before increment 7 (1 to 6 the adventure lane's, 7 the town lane's `writer/town.rs`);
   one still open then is the rmg lane's, `cross-lane` with the user's consent.
3. **The stream.** The C runtime's `rand` (state × 214 013 + 2 531 011, bits 16 to 30; `srand`
   sets the state) seeded once with the seed, from `srand` through the writer's teams' draws:
   every draw in the original's order with its modulus and signedness. It lives in
   `h3_kernel::rng::CrtRand` (a shared change at increment 2), not in `h3-rmg`: it is the third use
   after `h3_adventure::split::TileRand` and `h3_heroes::level_up::OriginalDraws` (their lanes may
   move to it), tested once (KER-RNG-004: the code reading's vectors; the phase oracle's logs check
   the whole stream from increment 2's draws before step 1 on, point 8). It is not a named stream of
   the kernel's `Rng` (ADR-0003 point 7): a caller that derives a map's seed from a game's seed
   draws the `u32` from its own stream; inside the generator nothing else draws.
4. **Floats.** The original's x87 code runs at the thread's 53-bit precision control (0x27F: the
   game never sets it, `docs/formats/installation.md`; the phase oracle reads it once at increment
   3), so each operation is IEEE binary64 and every conversion truncates (`crt_ftol`).
   `trunc(sqrt(n))` of an integer `n` (steps 1, 2, 5 to 8, 10; `vector_length_int`) is the integer
   square root, equal for every `n < 2^52` (the domain's `n` is at most 82 944), tested over the
   domain: no float. Step 1's and step 2's layout products `trunc(f64(r) × T[k] + f64(o))` and
   step 2's bound tests use `kernel::num::Real`, the tables (`rmg_layout_cos` 0x682500,
   `rmg_layout_sin` 0x682600, four decimals) as `Real(k) / Real(10 000)`, the nearest double; a
   test with the original data checks their bits against the executable's. Those rules say
   `numeric = "float"`; no transcendental, no float `sqrt`.
5. **Pack data.** `packs/ab/rmg.toml` holds AB's three tables as the game reads them: `rmg.txt`'s
   templates, zones and connections in row order with the ids as read, `objects.txt`'s template
   lines in file order, `rand_trn.txt`'s rows. `packs/sod/rmg.toml` holds SoD's additions (three
   templates; `objects.txt`'s new and changed lines) and the generator's constants read from SoD's
   executable (the object generators, limits, reward budgets, guard tables, densities; each with
   its address, `kind = "reverse-engineering"`), each added by the increment that ports its step.
   `h3-harvest` writes both (`cross-lane`; Q13's numbers and names, the template lines' picture
   names among them: RMG-Q2). The harvest keeps the values
   as read; a data quirk's fix is a gated override applied by the rmg part's compiler (8XM8's ids,
   RMG-QUIRK-22: ADR-0007's amendment E04b T8e, its scope extended to the part that alone reads
   them). The rmg part (`RmgRuleset`) compiles from the enabled packs' `rmg.toml` in load order,
   immutable, with its own hash; it is not in `GameRuleset` (a game reads the map, not the
   generator's data). A ruleset without `sod` has no generator (decision 3; AB's code is E11's).
   The algorithm stays code (Tier 3): the steps, the painters' picture sets, the layout tables,
   the cell layout, each with its address as source.
6. **Profiles and flags.** The ledger's 18 flags join the closed registry (`h3_content::REGISTRY`:
   17 `quirk.rmg.<name>`, one `quirk.content.rmg_8xm8_zone_ids`), each with the increment that
   ports its step; the game's profiles set them, `faithful` on (the oracle's maps), `strict-fixed`
   off: the ledger's "fixed" is `strict-fixed`, and there is no generator profile apart from the
   game's (a generated map and the game it starts share one ruleset). The `exploit-fix` entries
   (RMG-QUIRK-3 to -6) are typed `GenerateError`s in every profile. The oracle model of RMG-QUIRK-1
   and -2 (`faithful`: the bytes read uninitialised as a fresh start's first map holds them under
   the oracle) is measured by the phase hooks when step 2 is ported and kept in `h3-rmg` as
   observed constants, not pack data; its form is increment 3's checkpoint; `strict-fixed` zeroes
   them.
7. **Versions and provenance.** `RMG_GENERATOR_VERSION` (`h3-rmg`, the rmg lane's) changes whenever
   the same rmg part, profile, settings and seed can give another map; `RMG_RULESET_HASH_VERSION`
   versions the part's hash; `rmg.toml`'s format is in `packs/README.md`. A generated map's
   provenance is (generator version, part hash, profile, settings, seed): the CLI prints it, a
   game kept on a generated map records it beside the map's content hash (ADR-0015's naming is
   unchanged). The map's format version (RoE 0x0E, AB 0x15, SoD 0x1C) is a setting (RMG-SET-004).
8. **The comparison seam.** Every draw carries its original call site (the call's address, a
   `u32`), and `generate` reports each draw and each step's end to its `Trace` (generic, a no-op in
   play). Tests turn the port's cells after step k into the phase oracle's dump format 1 (`xdd.c`
   section 10) and its draws into the log's lines: CI compares their digests with
   `clients/capture/rmg/phases.txt` and the file's with `kept.txt` (digests only, ADR-0009); a
   local run beside the oracle compares the whole logs and dumps and names the first divergence.

## Consequences

- Increment 2's `shared/` pull request (T2a): the two crates (`Cargo.toml`, `arch.toml`,
  `lanes.toml`; `check-arch` rejects the edge `h3-game → h3-rmg`), `CrtRand`, the context in
  `docs/03-domain.md`, the catalog context `RMG` (`docs/04-rules-and-sources.md` §3), this ADR
  numbered; the edge `h3-map → h3-rmg` with increment 7's shared change. Each flag adds a line to
  the packs' profiles (`core.toml`, a shared file) in its increment's shared change.
- `faithful`'s byte-for-byte claim holds for a fresh start's first map under the oracle's
  environment; Windows' memory and a second map in one game stay out of reach (RMG-QUIRK-1).
- The writer's seven gaps are increment 7's prerequisite, in the adventure and town lanes'
  inboxes with this order; a generated game parses the bytes it wrote (one map, cheap).
- A game on a generated map plays E12's objects (Seer's Huts, Obelisks, Sanctuaries) as blockers
  until E12; Dragons' neutral main town is refused until the town lane answers (RMG-QUIRK-23).
- The generator is benchmarked when it is whole (increment 7), the baseline for E30.

## Alternatives considered

- A writer of the reader's raw model (`RawMap` to bytes, round-trip tested over the shipped maps),
  the generator converting into `RawMap` and the game importing it without bytes: the session's
  recommendation; the user kept the experiment writer (RMG-Q2), whose typed objects the other
  lanes' experiment maps already use.
- `h3-rmg` writing the `.h3m` itself (a port of `rmg_write_map`): a second writer and a file format
  in a logic crate, against decision 2.
- `rand` local to `h3-rmg` (the epic's first agent decision): a third copy of the same generator.
- Integers everywhere: step 1's tables are not k / 10 000 in binary; `Real` reproduces binary64
  without a proof per site.
- `objects.txt` read from the installation at run time: E02's amendment (no run-time import, Q13);
  a generator that needs the installation could not serve the server or training.
- A generator profile apart from the game's: a third axis in every setup with no consumer yet
  (E30 may add one).

## Revision history

- 2026-11-24: Drafted and confirmed (E14 T1c3, RMG-Q2); numbered by E14 T2a (point 3's test vectors
  the code reading's, the phase oracle's logs not kept; the `h3-map` edge moved to increment 7 as
  point 1 says; the edge `h3-game → h3-rmg` forbidden in `arch.toml`).
