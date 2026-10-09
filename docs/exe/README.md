# Heroes3.exe address map

Code reading of the user's `Heroes3.exe` settles rules that sources and experiments leave open
(OPEN-QUESTIONS Q29, Q57; `docs/04-rules-and-sources.md` §2). This folder is its shared memory:
what each address we met is, so the next agent looks it up instead of reading the same code again,
plus a tool that shows one function at a time with those names filled in.

Before the map (until 2026-10-08) every session disassembled the whole `.text` into one flat file
and grepped it. That listing had no function bounds, names or callers. What earlier sessions had
learned lay in `docs/formats/installation.md` §7 as prose by task: one function was described in up
to ten subsections, and one address was misnamed for a while (0x469E50, the moat, called a
Quicksand test). The prose stays; the map indexes it.

## Files

| Path | What | Owner (`lanes.toml`) |
|---|---|---|
| `docs/exe/map.toml` | Our readings: functions, notable sites inside them, tables, globals. One line per entry, sorted by address | common: any lane, in the pull request of its reading |
| `docs/exe/structs.toml` | Record layouts we read (the battle stack, the battle manager, hex records, objects, the static tables' records) | common |
| `docs/exe/h3api.toml` | H3API's names and layouts ported onto this build. **Generated, guesses**; never edited by hand | shared |
| `scripts/exe.py` | The reading tool (below) | shared |
| `scripts/exe_h3api.py` | The H3API port (`exe.py port-h3api`) | shared |
| `xtask/src/exemap.rs` | `cargo xtask check-exe-map`, a verify gate | shared |

## What may be committed

Addresses, our names, record layouts and findings in our own words and numbers. Never the
executable, a listing, decompiled code or anything `scripts/exe.py` prints (Q29; ADR-0009,
amendment 3). The tool reads the gitignored copy in `original-data/` and writes only there
(`listing` → `original-data/exe/`). H3API's names and field comments are committed in
`h3api.toml` under its MIT notice.

## Setup

1. The executable: `original-data/install/Heroes3.exe` (a cloud session stages it from the user's
   repo folder, `docs/08-work-process.md`, "Cloud sandbox"; the tool refuses any other build than
   SHA-256 `aac39e8c…`, `installation.md` §1).
2. `pip install capstone pefile` (in a sandbox add `--break-system-packages`;
   `scripts/sandbox-setup.sh` does it). Python 3.11 or later.

Each command disassembles `.text` afresh (about 3 seconds) and needs nothing else.

## Reading code

```sh
scripts/exe.py lookup 0x5A8090          # what the map, H3API and the code know about an address
scripts/exe.py lookup resistance        # entries, H3API names and struct fields matching a text
scripts/exe.py fn spell_effect_factor   # one function, annotated (a name or any address inside it)
scripts/exe.py fn 0x5A02EA --around 20  # 20 instructions each side of a site (any long function)
scripts/exe.py fn cast_spell --from 0x5A0336 --to 0x5A0360   # a range inside a function
scripts/exe.py switch cast_spell        # a switch's cases: value -> handler, grouped by handler
scripts/exe.py callers 0x4448F0         # call sites, grouped by calling function
scripts/exe.py callees 0x59FE30         # the calls a function makes
scripts/exe.py refs 0x63B850            # instructions that use a table or global (its whole size)
scripts/exe.py refs battle_manager_pointer --group   # one line per referring function
scripts/exe.py grep '\+ 0x4e0\]'        # a field offset, a constant, a mnemonic (regex, lower case)
scripts/exe.py grep 'call 0x50b230' --fn berserk_turn
scripts/exe.py data 0x63B850 --count 18 # table values (--type u8 i8 u16 i16 u32 i32 f32 f64 ptr str)
scripts/exe.py listing                  # everything, to original-data/exe/heroes3.asm
```

The annotations: a call or reference shows its target's name (ours plain, `(inferred)`/`(guess)`
appended when not read, `h3api:Name?` for an H3API guess, `sub_XXXXXX` for an unnamed function,
`name+0x12` inside one), the import's DLL name, a string's text, and for an x87/SSE memory
operand **the constant's value and width** (`= 0.05 (f64)`; the trap of `installation.md` §7.10).
Switch tables are decoded as `dd`/`db` rows with their case labels, so the sweep stays in step
after them; `switch` resolves them (value, index, case, handler). Branches inside the same
function are not annotated.

**Keep the listing small.** A listing costs about 34 bytes per instruction, and a few functions
are huge (`cast_spell` 3 500 instructions, about 115 KB; some unnamed ones 180 KB), so `fn`
prints a function of more than 600 instructions only sliced (`--around`, `--from`/`--to`) or
with `--all`; without a slice it lists the map's sites inside the function instead, as a table
of contents. `refs` and `grep` stop at `--limit` (200). Start from `lookup`, `switch`,
`callees` and `callers`, then read the part that decides. The map files are for querying, not
reading: `map.toml` is about 70 KB and `h3api.toml` about 190 KB; `lookup <text>` searches both
and the struct fields without the executable.

Function bounds are inferred: a function starts at a call target, at a code address stored as a
value (vtables, callbacks) or reached by a tail jump when it is 16-aligned after padding, or at an
entry of the map; it ends at the next start. A start missed by these rules makes the function above
it look longer; a map entry fixes that.

## Recording a reading

In the same commit as the finding (the finding itself goes into `installation.md` §7 or the lane's
equivalent, as before):

1. Every address the docs or code cite needs a map entry that covers it: a function covers its
   whole size, a table or global its size, a site only itself. `scripts/exe.py cited` lists the
   uncovered ones with their functions; `cargo xtask check-exe-map` fails on them (in verify and
   CI). Add a `site` entry when the instruction itself means something (a roll, a case handler);
   a call site that only shows "called from here" is covered by its function.
2. Name an address only after reading it, else mark it `inferred` and say what is known
   (`installation.md` once called the moat's damage a Quicksand test). An `inferred` name is
   read far more often than its tag: when H3API names the address, prefer a neutral name or
   H3API's meaning over a name taken from the one site read inside it (0x468860 was
   `faerie_spell_reader` until a reading showed the stack's stats dialog).
3. Run `scripts/exe.py check --fix-sizes` (fills or corrects function sizes; the CI check has no
   executable and relies on them) and `scripts/exe.py check`, which exits 1 while anything is
   wrong. `cargo xtask verify --with-original-data` runs the same check when the executable is
   staged.
4. A reading that confirms an H3API name: write our entry and add `h3api = "…"`; one that
   contradicts it: say so in the summary (the word H3API must appear). `check-exe-map` fails an
   entry at an address H3API names that does neither. Never edit `h3api.toml`.
5. Record layouts go into `structs.toml`; a field inside an array field (one spell's duration) may
   have its own line.
6. Say which path the reading followed. The executable also holds the computer player's own
   logic (`ai_*` entries; 0x43C800 writes a cast that Magic Mirror never redirects), branches on
   the game version (`[game]+0x1F698`, H3API's `mapKind`), on the tutorial, on a hidden or quick
   battle (0x46A4A0) and on network play. A rule is what the human player's action path does;
   a reading names the caller path it took and the branches it did not follow.

**Entry format** (`map.toml`, one line, keys in this order):
`{ va = 0x5A8090, kind = "function", name = "spell_effect_factor", size = 0x5A4, confidence = "read", summary = "…", args = "…", returns = "…", h3api = "…", refs = ["installation.md §7.8"] }`.
`kind`: function, site, table, global. `name`: snake_case, unique, our words. `size`: bytes
(functions and tables need it; a global defaults to 4). `args`/`returns`/`h3api` optional; `refs`
point to the doc sections (and rule ids) that hold the detail. **Structs** (`structs.toml`):
`[[struct]]` with `name`, `size`, `summary`, optional `refs`, `h3api`, then
`fields = [ { offset, name, type, summary, confidence, refs? }, … ]`, sorted by offset.

## Confidence and reliability

- `read`: an agent read the code and the docs describe what it does.
- `inferred`: the role follows from callers, data or behaviour; the code was not read (or not in
  the part that matters).
- `guess`: a name from elsewhere, not checked (all of `h3api.toml`).

A rule may rest on a reading (`original-data`, §2 of `04-rules-and-sources.md`) only where the
reading is `read` and complete for that rule. What made readings fail before: reading one path of
several (§7.8's first reading, refuted by batch 2a and re-read), a float constant of the wrong
width, an unread callee assumed. So: read every case and caller that decides the rule
(`callers`, `switch`), let the tool show constant widths, and write down what was not read (as
§7 does: "not read"). A reading of behaviour is an agent's interpretation, not data: it ranks
with an experiment, and an experiment that contradicts it stands while the reading goes back
for a re-read (user, 2026-10-09, Q57).

Experiments stay in the loop for the readings most likely to be wrong, even when a reading
"settles" the rule: queue a confirmation experiment (the lane's `docs/experiments/`) when a
reading contradicts VCMI or the wiki, when the result rests on a float boundary or truncation
(the Q-0005 class), or when it depends on values the executable loads at run time from the
`.TXT` tables. A quirk flag confirmed by a reading alone needs a second look first: a fresh
agent's re-read of the deciding paths, or VCMI's agreement (`04-rules-and-sources.md`, "Confirm
before flagging").

## The H3API port

[H3API](https://github.com/RoseKavalier/H3API) (MIT, © 2021 RoseKavalier) is a C++ header
library made by reverse engineering the 3.2 build of the executable: about 340 wrapped functions
with calling conventions and arguments, about 45 global pointers and tables, and about 100 record
layouts. The user's GOG Complete build (SoD 4.0) is a close relative: its record layouts are the
same (H3API's `H3CombatCreature` is the 0x548-byte stack record, `H3Spell` the 0x88-byte spell
record), its code moved by a constant amount per region (+0x2E0 around 0x44xxxx, −0x310 around
0x5Axxxx), and the lower `.data` stayed in place while the managers' pointers moved by −0x50.

`scripts/exe.py port-h3api <clone>` (clone with `git clone --depth 1
https://github.com/RoseKavalier/H3API.git`) matches without the 3.2 binary:

- a function where the shifted address is a function start here, the shift is shared by at least
  three of its six nearest H3API neighbours (or taken from the nearest such anchor), and the
  return pops what H3API's convention and argument count say; the `evidence` column says which
  held (`weak: own shift` and `ret … not …` are the doubtful ones);
- a global pointer where the callers of its class's methods load `this` from it (H3CombatManager
  0x6993D0 from 144 calls, H3Main 0x6994E8), else by the shift of the nearest such pointer when the
  code uses the address and it holds a static table;
- a table H3API takes from an instruction's operand by reading that operand here.

Result on 2026-10-09 (H3API commit `92255ab`, 2024-11-11): 325 of 336 functions, 44 data items,
102 structs. Where the port and our map name the same address (53 cases: the cast routine
0x59FE30, the target test 0x5A39C0, applying a spell 0x4448F0, the usefulness test 0x447A80,
Magic Mirror's chance 0x4487F0, the hero's skill getters, …) the names agree in meaning and the
entries carry `h3api = "…"` (51 read, 2 inferred); `check-exe-map` keeps every such overlap
tagged or disputed. Three `inferred` names seeded from one site each had disagreed with the port
until the review of 2026-10-09 read them (0x468860 is the stack's stats dialog, as H3API said).
One function had moved between the builds: H3API's `Rand(low, high)` (0x50C7C0 in 3.2) is
0x50B230 here, which the port could not find (the map has it). So treat a guess as a pointer to
where to read, never as a source for a rule.

## Status and next steps

Seeded 2026-10-09 (`shared/exe-map`): 253 entries (160 functions, 47 sites, 39 tables, 7
globals; 22 of them inferred) cover all 314 addresses cited in the docs and code; structs for the
battle stack (70 fields), battle manager, hex record, battle object and the static tables'
records. About 7 700 functions are detected, 160 named by us, about 280 more by H3API only. The
review of 2026-10-09 (same branch) checked the function bounds against the executable (of
7 741 functions, 32 end without a return, all after a call that never returns), added `switch`,
the slicing of `fn`, `refs --group`/`--limit`, the H3API-overlap rule of the gate, the exit
codes and the `verify --with-original-data` run of `check`.

Next (any lane may pick these up; the tool's own improvements are shared changes):

1. Battle lane: new readings record their addresses here (the gate enforces it). Optionally,
   when a §7 subsection is rewritten anyway, point to map names instead of repeating a function's
   description.
2. Adventure lane: H3API names much of the adventure map (`H3AdventureManager`, `H3Hero`,
   `H3Town`, objects); start a reading with `lookup <topic>` and `fn` on those guesses.
3. The port: functions that moved between builds (as `Rand`) are missed or weakly matched (38
   `weak` matches); a byte-pattern match would need the 3.2 executable, which the user's
   installation does not have. H3API's `PtrAt`/`IntAt` data and its constants are not ported.
4. The tool: name struct fields in `fn` when the base register's type is known (e.g. `this` of a
   battle-stack method); a `--depth` for `callees` to see a call tree; a cache of the sweep keyed
   by the executable's SHA-256 and the map's modification time (each command now takes about
   3 s, which only matters for scripted use).
5. Later, if reading stays the main source: Ghidra headless (Apache-2.0) as a `decompile`
   backend seeded with the map's names. Pseudo-C is several times denser than a listing for the
   same function; nothing of it would be committed, as now. The x87 width traps stay with the
   listing.
