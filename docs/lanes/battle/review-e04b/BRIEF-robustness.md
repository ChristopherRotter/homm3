# E04b independent review (task T11b): the robustness reviewer

The repository `/home/claude/homm3` is a headless, deterministic rules engine for Heroes of Might
and Magic III (Rust). Epic E04b added battle magic (hero spells, timed effects, obstacles with
hidden information, created stacks, creature magic, casting limits; rules `BAT-MAG-001..069`). The
epic closes after this review. You did not write it. Your job: **find inputs that make the battle
with magic panic, overflow, loop without a bound, corrupt its state or reject its own legal
commands**, and leave bounded regression tests behind. Other reviewers read the rules against their
sources; do not do that.

The reference is E04a's closure review (`docs/epics/E04a-log.md`, the entry "T12c", its
"Robustness" part): it added `extreme_heroes_and_counts_play_without_failures`
(`crates/h3-battle/src/tests/properties.rs`) and `crates/h3-pack/tests/harvested_heroes.rs`, and
ran 20 000 extreme and 3 000 harvested battles before committing bounded versions.

## Read first (in slices)

- `AGENTS.md` ("Non-negotiables"), `docs/06-security.md` ("Denial of service and resource
  limits"), `docs/05-testing.md` ("Invariants every context must provide property tests for").
- `docs/lanes/battle/HANDOFF.md`, "Standing traps" (test traps, coverage traps, background jobs).
- The existing tests named above and `every_step_preserves_the_invariants`, `arb_command`,
  `arb_extreme_hero`, `random_legal_play_finishes_battles` in `properties.rs`. Find out what they
  really exercise of E04b: do the extreme heroes cast, with which spells, masteries, power and mana?
  Do the armies hold casting creatures, raisers, Enchanters, Phoenixes, creatures that create
  stacks? Do the harvested battles cast the real spells? What does the property over steps offer as
  casts (the HANDOFF says `legal_commands` lists no casts)?

## Then look for failures

1. **By reading** the magic code (`crates/h3-battle/src/`: `magic.rs`, `effects.rs`, `casters.rs`,
   `created.rs`, `raise.rs`, `obstacles.rs`, `after_strike.rs`, `mana.rs`, `control.rs`,
   `binds.rs`, `hero.rs`, `turn.rs`; `crates/h3-heroes/src/spells.rs`, `derive.rs`): arithmetic
   that can overflow with extreme but valid input (overflow checks are on in every profile, so an
   overflow is a panic): spell value = power × factor + effect, damage against counts up to
   `u32::MAX`, hit-point pools, resurrection and clone and summon counts, mana, durations,
   percentages; indexing and `unwrap`/`expect` on reachable paths; loops whose bound is not
   obvious (chain targets, automatic actions of berserk or hypnotized stacks, periodic casters,
   the search for a free hex or record); fixed-capacity storage and what happens when it is full
   (effects per stack, created stacks' records and slots, spell obstacles, binds, raised records).
   Which bounds does the setup's soundness check put on a hero's values (power, knowledge, mana,
   spell lists, school levels, effects) and on spell data from a pack (`h3-content`'s compile
   checks for spells and abilities)? Can a valid pack or a valid `BattleSetup` reach a panic?
2. **By running.** Write throwaway stress tests in your worktree and run many random battles to
   their end or to a step bound, asserting: no panic, every command the engine lists as legal
   (including casts: `cast_targets` × `second_targets`, creature casts) is accepted, the state's
   invariants hold, the battle ends or the bound is reported with the state that does not end.
   - the synthetic ruleset of the tests with extreme casting heroes on both sides (largest power,
     knowledge and mana, every spell at every mastery, extreme hero effects) and huge stacks;
   - the harvested packs (`packs/base`, `ab`, `sod`): heroes that know every combat spell, armies
     drawn from every creature (casters, Faerie Dragons, Master Genies, Enchanters, Pit Lords,
     Archangels, Phoenixes, Wraiths, Familiars, elementals, Golems, dragons, war machines), both
     quirk profiles (`faithful`, `strict-fixed`);
   - the sim and the C ABI: `h3-sim`'s action decoding and `legal_mask` for every action index and
     out-of-range ones, the random agent over the `full` scenario with casting; `h3-ffi`'s functions
     with wrong sizes and indices (read `crates/h3-ffi/src` and its tests first).
   Use release builds for volume. Record what you ran: configurations, numbers of battles, seeds.

## Working rules

- Work only in your own worktree, never in `/home/claude/homm3` itself (other reviewers read it,
  and its build directory belongs to the verify):
  `git -C /home/claude/homm3 worktree add /home/claude/wt-robust -b review/robustness HEAD`,
  then always `export CARGO_TARGET_DIR=/home/claude/wt-robust/target` (the pinned toolchain is set
  up; `packs/` is in the worktree; `ln -s /home/claude/homm3/original-data original-data` if a
  test needs the original files, which stay read-only and are never committed).
- **Do not start a build before the baseline verify has finished**: wait until
  `tail -n 3 /home/claude/homm3/target/sandbox/verify.log` shows `EXIT=`. The machine has two
  cores and 7 GB of memory: read first, then build; one cargo job at a time; never more than
  `-j 2`. Keep an eye on the disk (`df -h /home/claude`): stay below 8 GB for your target
  directory.
- A build or a long run outlasts a tool call: start it with `nohup … > log 2>&1 &`, poll the log,
  kill by pid (never `pkill -f`). The VM can restart (background jobs die, files stay): check
  `uptime` and `ps` before waiting on a job.
- **Do not fix engine bugs.** Report each with a minimal reproduction (setup, seed, the commands)
  and a failing test saved as a patch file in the scratch folder (not committed).
- **Keep** what is worth keeping, committed on your local branch `review/robustness` (no push):
  one bounded test for extreme casting heroes in the style of the existing ones, and the bounded
  harvested-pack test if the existing one does not cast; fixed seeds or a modest number of proptest
  cases; `// covers: <rule ids>` markers; no `HashMap`/`HashSet`; `cargo fmt`; clippy-clean
  (`cargo clippy -p <crate> --all-targets -- -D warnings`). A line only a proptest reaches makes
  the coverage gate flaky (HANDOFF): tests must not add engine code, so this only matters if you
  touch test support code.
- No sub-agents, no WebFetch or WebSearch.

## Your report

Write `<SCRATCH>/review/robustness.md`:

```
# Review robustness
## What the existing tests exercise of E04b (and what they do not)
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

`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.
