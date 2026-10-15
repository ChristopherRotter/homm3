# E04c independent review (task T10b): the security reviewer

The repository `/home/claude/homm3` is a headless, deterministic rules engine for Heroes of Might
and Magic III (Rust), meant to run server-side with untrusted clients. Epic E04c added the special
battlefields and sieges: `BattleSetup.battlefield` and `BattleSetup.siege` (walls with hit points,
a gate that opens and closes, a moat, the Tower's hidden land mines, turrets off the field, the
attacker's Catapult with its wall-shot command, the Cyclops' wall shot, Earthquake, the computer's
aim for the automatic turrets and Ballista, the defender's retreat through the Escape Tunnel and no
surrender). Rules `BAT-SIE-001..029`, `BAT-TER-002..006`, `BAT-WAR-001..008`. The epic closes after
this review. You did not write it. Your job: **check what E04c added against `docs/06-security.md`**:
commands that carry intent only and are validated against the trusted actor, hidden information
that never reaches the other side, and bounded work. Other reviewers read the rules against their
sources and stress the engine with extreme input; do not do that.

## Read first (in slices)

- `docs/06-security.md` (trust boundaries, information hiding, denial of service; skip the hosted
  games part), `docs/05-testing.md` ("Invariants …"), `AGENTS.md` ("Non-negotiables").
- `docs/adr/0023-*.md` (the siege seam) and `docs/adr/0014-battle-magic.md` points 7 and 8 (hidden
  information and the action shape), and the lane's traps in `docs/lanes/battle/HANDOFF.md`
  ("Standing traps": the bullets on stack ids and turrets, hidden obstacles, the observation and
  `h3-protocol`'s hygiene tests).
- What is hidden in a siege: the Tower's mines (BAT-SIE-007: `python3 <SCRATCH>/review/rules.py
  BAT-SIE-007 BAT-SIE-010 BAT-SIE-011 BAT-SIE-013 BAT-SIE-023` prints the entries), the computer's
  desperate mode (BAT-SIE-023), a hero's Ballistics.

## What to check

1. **Commands.** `crates/h3-battle/src/command.rs`, `legal.rs`, the validation in `siege.rs` and
   `war.rs`, the errors in `error.rs`. For every command E04c added or changed (the Catapult's wall
   shot and its target, the Cyclops' wall shot, the turn of a stack in the moat, a walk or flight
   across the walls and the gate, Teleport across them, a cast at a wall hex): it carries intent
   only (no damage, hit points, owner or result from the payload); it is checked against the
   trusted actor (whose turn, whose Catapult, which side may shoot walls, the tactics phase);
   every reference is checked (wall target ids, hexes in range); validation is complete before
   any change **and before any random draw** (a rejected command must leave the state untouched,
   its RNG included); and an error never tells the issuer a hidden fact (a mine's hex).
2. **Hidden information.** The Tower's mines (who sees them, who steps on them), and anything else
   you find E04c keeps private. Follow every way out of the state:
   - `observe_as` and the spectator view (`crates/h3-battle/src/observe.rs`), the flat encoding
     (`crates/h3-sim/src/observation.rs`: the siege fields E04c added);
   - every event kind E04c added, its visibility and its projection (`event.rs`: wall damage, the
     gate, the moat, the mines, the turrets' shots);
   - the errors of a rejected command, and **the legal commands themselves**: do `legal_commands`,
     the path offered to a walker or the sim's `legal_mask` differ with a hidden mine? What does
     the mover learn, and is that what the original shows too (the mines are the Quicksand/Land
     Mine mechanism: quirks `BAT-QUIRK-1` and `4` in `docs/rules/quirks-ledger.md`)?
   - the computer's aim (`aim.rs`: the automatic turrets and Ballista read only what their side
     sees? the HANDOFF says `obstacles::public_pits`);
   - side channels: the order and number of random draws (the turrets' damage rolls, the Catapult's
     hit roll, Earthquake's draw), state hashes, replays;
   - `h3-protocol`'s mapping of the battle's E04c types (`crates/h3-protocol/src/map/`, the server
     lane's code: note problems, it is not ours to change): does the wire use the projections?
3. **The hygiene properties** in `crates/h3-battle/src/tests/properties.rs`
   (`observations_do_not_depend_on_hidden_state`, `hidden_obstacles_do_not_show`, the walls
   property) and `tests/observe.rs`: what do they vary and compare at a town, and which hidden
   state of E04c do they not vary?
4. **Bounded work and storage.** For E04c's paths: work per command (the shot's wall line, the
   Catapult's target search, the automatic aim's search over stacks and paths, Earthquake) and
   fixed-capacity storage (wall parts, turrets' records), and what happens at the bound. Can a
   player make a siege that never ends (a defender behind walls that nobody can reach, the keep
   shooting for ever) or a command that costs unbounded work using legal commands only? What ends
   such a battle (BAT-END-001, the round limit)?
5. **The C ABI** (`crates/h3-ffi/src`): the `unsafe` code E04c touched (the wall-shot action, the
   observation buffer's new siege fields): sizes, indices and null pointers checked before use.

## Rules for working

- **Do not change any file in the repository.** Do not commit. Your only output is your report.
- **Do not run `cargo` or any build** (two cores, the build directory is in use). If a claim needs
  a test run, write the test as a sketch into the finding and mark it `needs-run`.
- No WebFetch or WebSearch, no sub-agents. Read big files in slices; `grep -n`, then read ranges.
  Keep your context small: read a function, note the finding, move on.
- A component with no finding is a normal result; say so. Do not report style. Do not report
  anything you have not traced in the code. The coordinator verifies every finding before acting
  on it, so exact evidence matters more than the number of findings.

## Your report

Write `<SCRATCH>/review/security.md`:

```
# Review security
## What is hidden from whom (table: fact | hidden from | where it lives in the state | ways out checked)
## Commands (table: command | intent only | actor check | references | validated before change and draw | errors)
## Findings
### F-sec-<n>: <the claim in one line>
- kind: validation | leak | side channel | unbounded work | abi | test-gap
- severity: high (a client can cheat or read hidden state in normal play) | medium | low
- confidence: high | medium | low, and what would raise it
- evidence: file:line references; a concrete scenario (who does what, what they learn or change)
- fix or test sketch
## Gaps in the hygiene properties
## Not reviewed, left open
```

Final message to the coordinator: at most 12 lines (what you checked, findings by severity in one
line each, what is left).

`<SCRATCH>` is `/tmp/claude-0/-home-claude/373aaa37-9bc2-57bf-9416-b7e65b53287d/scratchpad`.
