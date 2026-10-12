# Trial: Ghidra pseudocode, through REA, as a backend for exe readings (2026-10-12)

A time-boxed trial with a report only, not an adoption. Battle lane, branch
`battle/exe-decompile-trial`. GOG Complete 4.0 `Heroes3.exe` only (SHA-256 `aac39e8c…`, as in
`docs/exe/map.toml`). Nothing from the executable, Ghidra or REA is committed. The raw outputs,
the Ghidra projects and the trial's scripts are in the gitignored `original-data/ghidra/` (the
scripts and notes also on the user's machine, `original-data/ghidra/trial/`). Quotes below are
single expressions. The user decided during the session that the addresses this report cites get
map entries (7 added to `docs/exe/map.toml`, the probe's marked provisional) so that
`check-exe-map` passes. The brief had kept `docs/exe/` unchanged.

**Result: drop**, for now, both REA and Ghidra headless as the `decompile` backend. Criteria 1, 2
and 4 of the decision rule fail. Criterion 3 holds only in a weak sense. Ghidra's pseudocode is
about 2.5 times smaller than `exe.py fn`, not 3 times. Out of the box it silently loses every
x87 value that reaches the runtime's `_ftol`, which this executable uses for each float-to-integer
truncation. It also drops a store into a reused argument slot (Curse's `max(1, …)`), and it hides
`this` when a callee's convention is unknown. REA adds an evidence envelope and generic limits
around the same Ghidra text, but no warning about any of these, and it costs a full import and
analysis (about 200 s) per CLI call.

## Versions and setup

| Item | Version, time, problem |
|---|---|
| REA | `rea-agents@6.4.0` (npm `latest`, published 2026-10-11; repo HEAD `fd95880`), run as `npx -y rea-agents@6.4.0`. First `npx` 14 s, later calls about 9 s before any work. `rea doctor --provider ghidra` passed |
| Ghidra | 12.1.4 PUBLIC 20260921, the only line REA accepts (12.1.x). SHA-256 `ddac49f9…`, as in the release notes |
| JDK | OpenJDK 21.0.12.1 (the image's Ubuntu package, with `javac`). No install needed |
| Host | The cloud sandbox: 2 CPUs, 7 GB, Node 22.22.0, Python 3.13 |
| Direct headless import | Import and default auto-analysis 188 s (the decompiler's switch analysis 63 s). Decompiling the 11 functions afterwards: `cast_spell` 2.8 s, every other one at most 0.11 s. Reopening the saved project, seeding and decompiling: 10–20 s |
| REA per call | Each `rea decompile` call imports the executable into a fresh temporary project and runs the full auto-analysis again: 183–212 s over eight calls (197 s for the first, with nothing else running). Unattended: yes (no prompts, exit 0) |

Problems, in order:

1. **Ghidra could not be fetched.** The sandbox proxy refuses GitHub release downloads (cloning
   public repositories works; release assets do not). Per the brief there was no attempt around
   it. The user downloaded the zip into `original-data/ghidra/`.
2. **The zip is 570 MB, over the 400 MB staging limit.** A trimmed copy was made on the user's
   machine with `zip --out -d`: Extensions, docs, docker, `Ghidra/Debug` and the 37 non-x86
   processors were removed, leaving 305 MB. `zip` could not replace its temporary file in the
   connected folder (no delete permission). It left an empty `ghidra_12.1.4_trimmed.zip` and
   `trim.log` there, which the user can delete. The output was renamed to
   `ghidra_12.1.4_x86only.zip`. REA's doctor and Ghidra both accepted the trimmed tree.
3. **REA reviewed before the first run.**
   - `install.sh` resolves the latest release through api.github.com unless `--version` is given,
     installs globally (on Linux into `~/.local`), and then **starts `rea setup` on its own when a
     terminal is present**. Not used.
   - `package.json`: no install script in REA itself. `prepare` only installs husky from a source
     checkout. There are 37 runtime dependencies (among them `playwright-core`,
     `@lydell/node-pty`, `isomorphic-git`) and an optional `frida`, whose own install script
     downloads a binary.
   - The README says targets are analysed locally. A search of `src/` found no telemetry or
     update calls, except in `rea update`.
   - Ghidra sessions use an ephemeral project (`-readOnly -deleteProject`), private
     home/cache/config/temp directories and a current-user socket. REA never opens user projects
     and never installs Ghidra or Java.
   - Nothing else surprising. Files written outside the working directory: the npm cache
     (`~/.npm`) and temporary directories.

Decision rule 4 (setup reproducible from `scripts/sandbox-setup.sh` without hand-staged steps)
**fails today**: Ghidra needs a user download plus the trim. It could be met by the user keeping
the trimmed zip in `original-data/ghidra/` and `sandbox-setup.sh` unpacking it, just as it copies
the staged exe.

## Name seeding

A generator (Python, about 150 lines) builds the seed files from `map.toml` (228 functions),
`h3api.toml` (265 more names; conventions, argument counts and return types for 325) and
`structs.toml`. A Ghidra post-script (Java, about 120 lines) applies them after the analysis:

- 14 structures: 122 of their 157 fields typed (lists, vectors, tuples and references skipped,
  as were fields inside an array already typed);
- 493 function names, with 0 failures;
- a class namespace per `this` type, so that `this` gets the struct type.

It runs in 10 s on the saved project. Three variants were decompiled: unseeded (identical to
REA's output, checked text-equal for 4 functions), names (all seeds), and names plus (names and 6
hand seeds: `_ftol`'s x87 input, `random_between` as fastcall, and thiscall on four battle-stack
functions).

- **Readability.** A large gain wherever `this` is typed. `this->spell_duration[0x29]` (Bless),
  `creature_info.flags` and `blind_factor` replace raw offsets, and callees carry the map's names.
  The output grows by 0–15 %.
- **The seed that matters most** is the runtime's `_ftol` (0x6179D4) with its real input, ST0 as
  a `float10`. Without it, Ghidra prints `__ftol()` with no argument, and the whole x87
  expression feeding it disappears from the output. With it, the formulas come back in order,
  with their single-precision stores.
- **The map records arguments for only 33 of its 228 functions**, so conventions come mostly from
  H3API guesses. Where a callee's convention is unknown, its `this` (ECX) disappears from the
  call. In 0x443B90 that reads as the target's attack against the striker's defense, the roles
  swapped.
- **Seeds can mislead:**
  - H3API guesses become indistinguishable from read names. 0x617E5C is shown as
    `H3Random__Rand` with no guess marker, where `exe.py` marks it `?`. That guess is right: the
    trial read it as the C library's `rand` and mapped it. The thiscall guess
    `H3CreatureBank::UpgradeStack` types creature ids as `H3CreatureBank *`. A follow-up would
    prefix guesses (`h3api_…`) and never take a class or `this` type from a guess.
  - Our own layout is wrong in one place. In `structs.toml`, `battle_manager.local_human`
    (+0x54A4) is `i32[2]`, but the code reads bytes per side (in 0x46A4A0, and in more than 15
    byte reads that `exe.py grep "0x54a4\]"` lists), with the sides' players as integers at
    +0x54A8/+0x54AC. The seeded output therefore names +0x54A8 `local_human[1]`. It should be
    `u8[2]`.
- **REA's seeds** (`REA_GHIDRA_SEED_FILE`) take only an address, a kind and a name, applied before
  the auto-analysis: no types, conventions, structures or `_ftol` input. See the REA section.

## The test set

`asm` is the `exe.py fn` output in characters, header included (what an agent reads). For
`cast_spell` it is `--all`, because `fn` refuses without a slice (4 976 characters of site list;
`switch` 2 900). `C` is Ghidra's pseudocode in characters: unseeded (= REA's text) / names /
names plus. The ratio is `asm / C` for names plus. A tokenx estimate gives lower ratios (median
about 1.9), because the listing's tokens are cheaper than its characters suggest.

| Address | Name | asm | C (unseeded / names / plus) | Ratio | Correctness | Float |
|---|---|---:|---|---:|---|---|
| 0x46A4A0 | `battle_is_hidden` | 1 855 | 736 / 514 / 514 | 3.6 | agrees 3/3; the names output misleads at +0x54A8 (our struct error) | – |
| 0x50B230 | `random_between` | 841 | 241 / 252 / 263 | 3.2 | agrees 2/2; extra: `b == a` and `b < a` return early | – |
| 0x468860 | `stack_stats_dialog` | 3 525 | 1 360 / 1 409 / 1 409 | 2.5 | agrees 3/3 | – |
| 0x4E4D90 | `hero_max_movement` (`CalcMaxMovement`, a 0x12-byte wrapper) | 740 | 122 / 179 / 179 | 4.1 | agrees 1/1 | – |
| 0x4E4990 | `hero_max_land_movement` (the real medium case) | 11 393 | 5 002 / 5 620 / 6 138 | 1.9 | unseeded and names: 4/5, the speed table and Logistics lost behind `__ftol()` (**silent**); plus: 5/5 and extras | the lost part is float |
| 0x443B90 | `damage_stage2_factor` | 5 998 | 2 035 / 2 073 / 1 995 | 3.0 | unseeded and names: attack and defense roles appear swapped (**silent**); plus: 7/7 | doubles visible; one width misrendered |
| 0x4E5760 | `hero_spell_damage_bonus` | 5 961 | 2 111 / 2 427 / 3 212 | 1.9 | unseeded and names: 1/5, every float step lost and the specialty bonus looks discarded (**silent**); plus: 5/5 | plus: order and f32 stores right; constant widths hidden |
| 0x443160 | `strike_base_damage` (struct, thiscall) | 4 708 | 2 092 / 2 102 / 2 234 | 2.1 | 3/4: Curse reads `max(param_1, min − v)` in every variant (**silent**) | – |
| 0x59FE30 | `cast_spell` (large, switch) | 121 797 (`--all`) | 52 787 / 57 158 / 57 404 | 2.1 | switch: 37 value groups identical to `exe.py switch`; 5 site claims agree; the rolls' `trunc(p × 100)` inputs lost (missing); parameters typed `battle_stack *` | rolls via `_ftol` |

**Median ratio: 2.5** (names plus; 2.6 unseeded, 2.5 names). Decision rule 1 (at least 3) fails.
Two probe functions measured on the side: 0x469880 4 543 → 2 068 (2.2) and 0x43F2C0
17 328 → 7 331 (2.4).

## Discrepancies

Every one was checked against the listing.

| # | Function | What the pseudocode shows | Class | Silent? |
|---|---|---|---|---|
| 1 | 0x4E4990 | The land value as `__ftol()` with no input: the speed table 0x698A98, the Logistics rate 0x63EA68, the specialist growth and `(L + 1) × m` are all gone, and the slowest-speed loop looks unused | decompiler error (x87 ST0 input of a runtime helper not modelled) | yes, contradicts (unseeded, names; fixed by the `_ftol` seed) |
| 2 | 0x4E5760 | `f32(value)`, × 1.5, × (1 + Sorcery) gone; returns `crt_ftol()`; the specialty call's result looks discarded | decompiler error (same) | yes, contradicts (unseeded, names; fixed by the seed) |
| 3 | 0x443160 | Curse: `max(param_1, min − v)`. The code stores 1 into the argument's slot (0x443274) and takes the larger by pointer; Ghidra drops the store | decompiler error (merged variables: a reused argument slot) | yes, contradicts, in every variant |
| 4 | 0x443B90 | `stack_attack_against(target, shot)`, `stack_defense_against(striker, 1)`: the ECX `this` of each call is hidden, so the roles read swapped | decompiler error (unknown callee convention) | yes, contradicts (unseeded, names; fixed by thiscall seeds) |
| 5 | 0x443B90 | `blind_factor * (float)local_c`, where the code does `fld dword; fmul qword` (an extended product, no f32 rounding of the factor) | decompiler error (wrong width) | yes; misleading, the listing catches it |
| 6 | 0x59FE30 | Eagle Eye's and the resistance roll's `trunc(p × 100)` appear as `crt_ftol()` after the call that computes `p`; without the fastcall seed `random_between()` shows no bounds | missing information | no contradiction; the gap is visible |
| 7 | 0x469880 | The `dx = 0` angle as `(double)(0x40568000 << 0x20)`, an integer conversion of the bit pattern, instead of ±90.0 | decompiler error (type punning rendered as a conversion) | misleading; the same group results by luck |
| 8 | 0x46A4A0 | With names: +0x54A8 read as `local_human[1]` | seed data error (`structs.toml`) | misleading |
| 9 | 0x50B230, 0x4E4990, 0x443160, 0x4E5760 | Correct details the readings leave out (see below) | extra findings | – |

**Silent errors** (contradicting a verified reading with no warning in the output):

- the unseeded output, which is what REA returns: **5** (discrepancies 1 to 5); in the non-float
  functions: **2** (1 and 3);
- with all seeds: 2 (3 and 5), of which 1 in a non-float function (3).

Ghidra's only warnings were generic ones (an overlap of global names in 0x4E4990, and two blocks
removed as unreachable in `cast_spell`, which are inlined string-length clamps; harmless).

Decision rule 2 (zero silent errors in the non-float functions) **fails** in every variant.

**Extra findings** (correct per the listing, not in the readings):

- `random_between` returns `a` early when `b == a` or `b < a`.
- `hero_max_land_movement`:
  - a Logistics specialist's rate grows by `× (1 + 0.05 × level)` (f32);
  - land bonuses: artifacts 98 and 70, or a combination containing them, and hero flag
    +0x105 bit 1;
  - sea bonuses: a Navigation specialist, artifacts 123 and 71, a per-object count;
  - a computer player's hero gets **+75** points when the game's +0x1F6D8 is above 2, and **+125**
    when its player record's +0x34 is also 2. The field meanings are H3API guesses.

  These are leads for the adventure lane (E08), unconfirmed.
- `hero_spell_damage_bonus`: the specialty bonus is converted to f32 and added to `v` before the
  final truncation. The reading says "added as an integer", which gives the same result for any
  realistic value.
- `strike_base_damage`:
  - above 10 creatures it rolls 10 times and scales the sum by `n / 10`;
  - a nonzero argument returns the average `(min + max) × n / 2`;
  - the Ballista's damage is scaled by its hero's +0x476, plus 1, capped.

## Float fidelity (0x443B90, 0x4E5760)

| Aspect | Unseeded (= REA) | With the `_ftol` seed |
|---|---|---|
| Operation order | 0x443B90 right; 0x4E5760 lost entirely | right in both |
| f32 versus f64 | 0x443B90's double locals right, one misrendered `(float)` cast (5); 0x4E5760 lost | stores to f32 locals and arguments visible; constants print without width (`0.05` is the f32 `0.0500000007…`, which only the listing shows) |
| Truncation | every `_ftol` loses its input | `crt_ftol(expr)` at the right places |
| Rounding and precision control | invisible: the x87's extended intermediates and the 53-bit control word (ADR-0003, Q23, the E04b x87 decision) never appear | the same |

What to trust the pseudocode for: the control flow, which factors apply and when, the order of
the multiplications, and (with the seed) where values are stored as f32 and where truncated. What
needs the listing: every constant's width, any mixed f32/f64 product (5), anything feeding a
runtime helper by the x87 stack, and the intermediate precision.

Decision rule 3 holds only in the weak sense: every float error found is visible in the listing,
but none is flagged, so the cross-check has to be complete to catch them.

## REA versus Ghidra headless

- **What REA adds:**
  - an evidence record per result (target digest, Ghidra version, analysis profile, operation,
    `confidence = observed`) and a list of limits;
  - snapshots that should replay an exact repeated query without starting Ghidra. Here the
    snapshot was saved with 0 entries, and three identical `decompile` calls on it took 202,
    212 and 195 s, so there was no reuse for this PE;
  - call, xref, data-type and p-code operations, and seeds;
  - a pinned, scriptable CLI and an MCP server.
- **What it costs:**
  - Node 22+ and 37 runtime dependencies;
  - about 9 s of startup per call, and **a full import and analysis per CLI call** (183–206 s,
    against 10 s for a seeded direct run on a saved project). Only the MCP server keeps a session
    across calls, and the brief allowed the CLI only;
  - the JSON doubles the pseudocode (`raw_result` and `normalized_result`) and adds about 3 KB:
    736 characters of C became 4 420 bytes, 2 111 became 7 264. The default TOON format is the
    same size (7 202 bytes against 7 200 for 0x469880).
- **Honesty of limits:** the six limits are the same boilerplate on every result (ephemeral
  project, address format, "Pseudocode is Ghidra decompiler output, not original source"). None
  names the losses above, and the confidence stays `observed`. REA would flag terminal calls if
  there were any.
- **Seeds:** names only, applied before the analysis. No types or conventions, so neither the
  `_ftol` input nor `this` types can be seeded through it, and its read-only Windows mode cannot
  annotate at all. A seeded REA run of 0x443160 (`REA_GHIDRA_SEED_FILE` with the 493 names)
  applied every seed and recorded the seed file's SHA-256 and the outcome counts in its limits,
  which is good provenance. Its C equals the direct names-only output: 2 102 characters, with the
  same Curse error.
- **View:** REA adds nothing to the pseudocode's correctness or size. Its value is provenance and
  orchestration, not reading quality. For this project, `exe.py` already carries the provenance
  (the exe's SHA-256 is checked against the map), and a direct headless script on a saved,
  seeded project is 20 times faster per query.

## Client probe: the shooting animation group by angle

*Provisional, unconfirmed by experiment.* GOG Complete 4.0. A disclosure first: the probe
function was **located through the listing** (`exe.py callees` and `fn`) while Ghidra was still
missing, so the reading was not "pseudocode first". Ghidra's pseudocode of 0x469880 was read
afterwards and checked claim by claim against the listing: it gives the whole answer, with
discrepancy 7.

**Caller path** (a human player's shot; not `ai_*`, the tutorial, version or network branches):

`strike_action` 0x445A60 → `shot_action` 0x440160 (0x445BB8) → `stack_shot` 0x43F900 (0x440259,
0x440275, 0x4402D7) → the shot animation 0x43F2C0 (0x43F9DC; skipped while `battle_is_hidden`) →
0x469880 (0x43F409). The other caller of 0x469880, 0x465AD0 from `shot_action` 0x4401C3, was not
read.

**The choice (0x469880):**

- Arguments: the creature in ECX, the shooter's screen x in EDX, then its y, its facing and the
  target point, the projectile's DEF, and out-pointers for the start point, the group and the
  projectile frame.
- **Origin.** The start point is the shooter's position plus the creature's "straight" missile
  offset from the 84-byte records at 0x67FF24, which hold three int16 pairs (up, straight, down),
  then f32 missile-frame angles from +0xC. The x offset is subtracted when the facing is 0.
- **Angle.** With `dx`, `dy` from that start point to the target (screen y grows downwards):
  - `dx ≠ 0`: `angle = atan(−dy / |dx|) × 57.2957795` (f64), stored as f32;
  - `dx = 0`: `angle = +90` if `dy > 0`, else `−90`.
- **Group:**
  - `angle > 25.0` → group **14** (shoot up);
  - else `angle > −25.0` → group **15** (straight);
  - else group **16** (down).

  The thresholds are f64 constants and the comparisons strict.
- **Then:** the start point is recomputed with the chosen group's offset. The projectile frame is
  the number of leading midpoints `(a[i−1] + a[i]) / 2` of the record's angle list that are at
  least the angle. The list length is the projectile DEF's first group's frame count.
- **Possible quirk:** for a target exactly above or below the start point (`dx = 0`) the sign is
  inverted compared with every other angle. A target straight below gets +90, so the up group and
  the steepest upward frame. It is rare, since the start point carries the creature's offset.
  A candidate for the client lane's ledger once confirmed.
- Not read: the 0x43F2C0 side of the target point (the ±22 adjustment for two-hex targets and the
  height term) and the other caller.

## Recommendation: drop

Do not adopt REA or Ghidra headless as the `decompile` backend now. Pseudocode stays out of the
reading policy: at most an agent's private lead, always confirmed against the listing.

What would have to change before trying again:

1. `map.toml` records conventions (`conv`, `this`) for the functions it names.
2. Seeds keep H3API guesses marked.
3. A seed library covers the runtime's x87 helpers (`_ftol`; `fabs`/`atan` at 0x617F52/0x617DC4).

Even then the size gain stays near 2.5, and the argument-slot error (3) is not seedable.

**The follow-up, if the user still wants one** (no `shared/` code PR for "drop"):

- A `shared/` doc change to `docs/exe/README.md`, idea 5: the trial's outcome and the three
  conditions above.
- A common-path fix in `structs.toml`: `battle_manager` +0x54A4 is `u8[2]`, and the sides' player
  indices are `i32[2]` at +0x54A8 (any lane, with a re-read of its other users).
- The client lane: the probe answer as a provisional lead (now the map's `shot_angle_group`,
  0x469880; for its QUEUE inbox, by the client lane).
- Adventure: the computer players' movement bonus as a lead for E08.

If the user decides to adopt "Ghidra headless only" anyway, the `shared/` PR would contain:

- `scripts/exe.py decompile ADDR`: a saved project under `original-data/ghidra/`, seeded from the
  map, H3API with guess prefixes, the structs and the x87 helper seeds, printing C with a header
  that names its rank;
- `sandbox-setup.sh`: unpack `original-data/ghidra/ghidra_*_x86only.zip` when staged, run one
  import (about 3 min), in the background like the verify;
- a policy line in `docs/04-rules-and-sources.md` §2: pseudocode is a lead, never a reading; a
  claim from it needs the listing at every float operation, every runtime helper call and every
  call whose callee convention is not in the map.
