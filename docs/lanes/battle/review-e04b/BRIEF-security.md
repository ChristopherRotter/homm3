# E04b independent review (task T11b): the security reviewer

The repository `/home/claude/homm3` is a headless, deterministic rules engine for Heroes of Might
and Magic III (Rust), meant to run server-side with untrusted clients. Epic E04b added battle magic
(hero spells, timed effects, obstacles with hidden information, created stacks, creature magic,
casting limits; rules `BAT-MAG-001..069`). The epic closes after this review. You did not write
it. Your job: **check what E04b added against `docs/06-security.md`**: commands that carry intent
only and are validated against the trusted actor, hidden information that never reaches the other
side, and bounded work. Other reviewers read the rules against their sources and stress the engine
with extreme input; do not do that.

## Read first (in slices)

- `docs/06-security.md` (trust boundaries, information hiding, denial of service; skip the hosted
  games part), `docs/05-testing.md` ("Invariants …"), `AGENTS.md` ("Non-negotiables").
- `docs/adr/0014-battle-magic.md` (the decisions on hidden information and the action shape;
  points 7 and 8), and the lane's traps in `docs/lanes/battle/HANDOFF.md` ("Standing traps": the
  bullets on hidden obstacles, the observation and `h3-protocol`'s hygiene tests).
- What is private in E04b is decided in answered questions: grep `docs/OPEN-QUESTIONS.md` for
  `BAT-Q11`, `BAT-Q12`, `BAT-Q26` and `Q55` and read only those entries (the file is 100 KB).

## What to check

1. **Commands.** `crates/h3-battle/src/command.rs`, `legal.rs`, the validation in `magic.rs` and
   `casters.rs`, the errors in `error.rs`. For every command E04b added or changed (the hero's
   cast, the creature's cast, their targets and second targets): it carries intent only (no cost,
   damage, mastery, owner or result from the payload); it is checked against the trusted actor
   (whose turn, whose hero, whose stack; a hypnotized or berserk stack; the tactics phase); every
   reference is checked (stack ids, hexes in range, spell ids, second targets); validation is
   complete before any change **and before any random draw** (a rejected command must leave the
   state untouched, its RNG included); and an error never tells the issuer a hidden fact.
2. **Hidden information.** List what is hidden from whom (a hero's spell book and its contents,
   mana, the hidden magic of T9a, a Faerie Dragon's drawn spell, Quicksand and Land Mines and who
   sees them, Eagle Eye, the opening spells). For each, follow every way out of the state:
   - `observe_as` and the spectator view (`crates/h3-battle/src/observe.rs`), the flat encoding
     (`crates/h3-sim/src/observation.rs`);
   - every event kind E04b added, its visibility and its projection (`event.rs`): does a public
     event carry or imply a private fact (a count, an id, a position, a draw)?
   - the errors of a rejected command, and **the legal commands themselves**: do `legal_commands`,
     `cast_targets`, `second_targets` or the sim's `legal_mask` differ with the other side's hidden
     objects (a hex not offered, a move cut short, a cast refused)? What does the mover learn, and
     is that what the original shows too (see quirks `BAT-QUIRK-1`, `2` and `4` in
     `docs/rules/quirks-ledger.md`)?
   - side channels: the order and number of random draws (can a viewer tell a hidden draw happened,
     or its result, from public rolls?), state hashes, replays.
   - `h3-protocol`'s mapping of the battle's E04b types (`crates/h3-protocol/src/map/`, the server
     lane's code: note problems, it is not ours to change): does the wire use the projections?
3. **The hygiene properties** in `crates/h3-battle/src/tests/properties.rs`
   (`observations_do_not_depend_on_hidden_state`, `hidden_obstacles_do_not_show`) and
   `tests/observe.rs`: what do they vary and compare, and which hidden state of E04b do they not
   vary?
4. **Bounded work and storage.** For E04b's paths: work per command (target enumeration, chain
   targets, automatic turns, periodic casts) and fixed-capacity storage (effects per stack, created
   stacks, spell obstacles, binds), and what happens at the bound. Can a player make a battle that
   never ends or a command that costs unbounded work using legal commands only?
5. **The C ABI** (`crates/h3-ffi/src`): the `unsafe` code E04b touched (the factorized action,
   observation buffers): sizes, indices and null pointers checked before use.

## Rules for working

- **Do not change any file in the repository.** Do not commit. Your only output is your report.
- **Do not run `cargo` or any build** (two cores, the build directory is in use). If a claim needs
  a test run, write the test as a sketch into the finding and mark it `needs-run`.
- No WebFetch or WebSearch, no sub-agents. Read big files in slices; `grep -n`, then read ranges.
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

`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.
