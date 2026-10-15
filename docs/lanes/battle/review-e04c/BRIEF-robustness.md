# E04c independent review (task T10b): the robustness reviewer

The repository `/home/claude/homm3` is a headless, deterministic rules engine for Heroes of Might
and Magic III (Rust). Epic E04c added the special battlefields and sieges: `BattleSetup.battlefield`
and `BattleSetup.siege` (walls with hit points, a gate, a moat, the Tower's hidden mines, turrets
off the field at hexes 254, 255 and 251, the attacker's Catapult and its wall-shot command, the
Cyclops' wall shot, Earthquake, the computer's aim for the automatic turrets and Ballista, the
defender's retreat and no surrender). Rules `BAT-SIE-001..029`, `BAT-TER-002..006`,
`BAT-WAR-001..008`. The epic closes after this review. You did not write it. Your job: **find
inputs that make a battle at a town or on a special battlefield panic, overflow, loop without a
bound, corrupt its state or reject its own legal commands**, and leave bounded regression tests
behind. Other reviewers read the rules against their sources; do not do that.

The reference is E04b's closure review (`docs/epics/E04b-log.md`, grep "Round 1, list A" and
"Properties" in the entry "T11b"), which extended `extreme_heroes_and_counts_play_without_failures`
and `extreme_casting_heroes_play_without_failures` (`crates/h3-battle/src/tests/properties.rs`)
and `crates/h3-pack-battle/tests/harvested_heroes.rs`.

## Read first (in slices)

- `AGENTS.md` ("Non-negotiables"), `docs/06-security.md` ("Denial of service and resource
  limits"), `docs/05-testing.md` ("Invariants every context must provide property tests for").
- `docs/lanes/battle/HANDOFF.md`, "Standing traps" (stack ids and turrets, test traps, the
  background jobs, the disk).
- In `crates/h3-battle/src/tests/properties.rs`: `arb_town`, `situate`, `Place` (line ~208), the
  siege parts of `arb_game`, `every_step_preserves_the_invariants` (its siege assertions around
  lines 1395 and 1492), `walls_and_the_moat_agree_with_movement`,
  `the_walls_property_s_games_reach_the_gate_the_moat_and_the_breaches`,
  `extreme_heroes_and_counts_play_without_failures` (its town half, line ~2462) and
  `extreme_casting_heroes_play_without_failures` (line ~2787). Find out what they really exercise
  of E04c: which fort levels, factions, moats, wall hit points, turret counts, Ballistics levels
  and Catapult controls; do the extreme heroes' setups put huge stacks in the moat, shoot walls
  with the Cyclops, cast Earthquake and Teleport; do the games reach a destroyed keep and towers;
  are the special battlefields drawn with extreme heroes?

## Then look for failures

1. **By reading** the siege code (`crates/h3-battle/src/siege.rs`, `aim.rs`, `war.rs`,
   `terrain.rs`, the siege parts of `obstacles.rs`, `movement.rs`, `combat.rs`, `damage.rs`,
   `magic.rs` (Earthquake, Teleport), `turn.rs`, `flee.rs`, `state.rs`, `created.rs`;
   `crates/h3-content-battle/src/siege.rs`, `battlefield.rs`; `crates/h3-heroes/src/derive.rs`
   for Ballistics): arithmetic that can overflow with extreme but valid input (overflow checks are
   on in every profile, so an overflow is a panic): the turrets' damage from the keep's count and
   the hero's attack, the moat's damage against counts up to `u32::MAX`, wall hit points and the
   Catapult's damage, the computer's worth (`aim.rs`: Fight Value × count, the turns until a stack
   can attack), the shot's wall line (`LineAxis`: start and end hexes at the field's edges, the
   turrets' hexes 254/255/251); indexing and `unwrap`/`expect` on reachable paths (a wall target id
   from a command, a turret's part, a faction's record, a hex of the moat's rows); loops whose
   bound is not obvious (the automatic aim's search over stacks and paths, the Catapult's target
   search, Earthquake's draw when few targets stand, the search for a free hex at a wall or in the
   moat); fixed-capacity storage and what happens when it is full (wall parts, turrets, the
   obstacles list with the mines). Which bounds does the setup's soundness check put on
   `BattleSetup.siege` (fort level, faction handle, the moat, hit points, a town without a keep)
   and on siege data from a pack (`h3-content-battle`'s compile checks: the moat's rows and
   damage, the mines' count, the wall hexes)? Can a valid pack or a valid `BattleSetup` reach a
   panic? What does a setup with a town's turrets and no stacks for the defender do? A defender
   with only turrets, an attacker with only a Catapult?
2. **By running.** Write throwaway stress tests in your worktree and run many random battles at
   towns (every faction's siege record, every fort level, with and without the moat's second row
   under both game versions, with and without the Tower's mines) and on every special battlefield,
   to their end or to a step bound, asserting: no panic, every command the engine lists as legal
   (including the wall shots: the Catapult's targets, the Cyclops', and casts: `cast_targets` ×
   `second_targets` with Earthquake and Teleport) is accepted, the state's invariants hold, the
   battle ends or the bound is reported with the state that does not end.
   - the synthetic ruleset of the tests (`crates/h3-battle/src/tests/support.rs`: its siege
     records `t:castle`, `t:inferno`, …) with extreme heroes on both sides (largest attack,
     Ballistics at every level, Artillery, every Catapult control) and huge stacks;
   - the harvested packs (`packs/base`, `ab`, `sod`): sieges of every faction, both quirk profiles
     (`faithful`, `strict-fixed`), armies with Cyclopes, flyers, teleporters, two-hex stacks, war
     machines and casters of Earthquake and Teleport;
   - the sim and the C ABI: `h3-sim`'s action decoding and `legal_mask` for the wall-shot actions
     and out-of-range ones (`crates/h3-sim/src/action.rs`), the random agent over the `siege`
     scenario (`h3 bench`'s workload `random-rollouts-siege`: read `benches/battle/README.md` or
     `benches/README.md` for how it is run); `h3-ffi`'s functions with wrong sizes and indices
     for the new observation fields (read `crates/h3-ffi/src/lib.rs` and its tests first).
   Use release builds for volume. Record what you ran: configurations, numbers of battles, seeds.

## Working rules

- Work only in your own worktree, never in `/home/claude/homm3` itself (other reviewers read it):
  `git -C /home/claude/homm3 worktree add /home/claude/wt-robust -b review/robustness HEAD`,
  then always `export CARGO_TARGET_DIR=/home/claude/wt-robust/target CARGO_INCREMENTAL=0` (the
  pinned toolchain is set up; `packs/` is in the worktree;
  `ln -s /home/claude/homm3/original-data /home/claude/wt-robust/original-data` if a test needs
  the original files, which stay read-only and are never committed).
- The machine has two cores and 7 GB of memory: read first, then build; one cargo job at a time;
  never more than `-j 2`. Keep an eye on the disk (`df -h /home/claude`): stay below 8 GB for your
  target directory.
- A build or a long run outlasts a tool call: start it with `nohup … > log 2>&1 &`, poll the log,
  kill by pid (never `pkill -f`). The VM can restart (background jobs die, files stay): check
  `uptime` and `ps` before waiting on a job.
- **Do not fix engine bugs.** Report each with a minimal reproduction (setup, seed, the commands)
  and a failing test saved as a patch file in the scratch folder (not committed).
- **Keep** what is worth keeping, committed on your local branch `review/robustness` (no push):
  one bounded test for extreme sieges in the style of the existing ones (or an extension of
  `extreme_heroes_and_counts_play_without_failures`' town half), and a bounded harvested-pack
  siege test if none exists; fixed seeds or a modest number of proptest cases; `// covers: <rule
  ids>` markers; no `HashMap`/`HashSet`; `cargo fmt`; clippy-clean (`cargo clippy -p <crate>
  --all-targets -- -D warnings`). A line only a proptest reaches makes the coverage gate flaky
  (HANDOFF): tests must not add engine code, so this only matters if you touch test support code.
- No sub-agents, no WebFetch or WebSearch. Keep your context small: read a function, note it,
  move on; never print a long test log, grep it.

## Your report

Write `<SCRATCH>/review/robustness.md`:

```
# Review robustness
## What the existing tests exercise of E04c (and what they do not)
## What I ran (configuration, battles, seeds, result)
## Findings
### F-robust-<n>: <the claim in one line>
- kind: panic | overflow | unbounded loop | state corruption | legal command rejected | validation gap
- severity: high (reachable in normal play) | medium (extreme but valid input) | low (only an invalid pack or setup that validation should refuse)
- reproduction: setup, seed, commands; the failing test's patch file
- where: file:line; why it fails
- fix sketch
## Tests kept (file, test names, commit ids on review/robustness; run time)
## Not exercised
```

Final message to the coordinator: at most 15 lines (what ran, findings by severity in one line
each, the commits kept, what is left).

`<SCRATCH>` is `/tmp/claude-0/-home-claude/373aaa37-9bc2-57bf-9416-b7e65b53287d/scratchpad`.
