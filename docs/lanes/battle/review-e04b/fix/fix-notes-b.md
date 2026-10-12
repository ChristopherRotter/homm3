# Fix notes of worker `b` (E04b T11b, list B)

Worktree `/home/claude/wt-b`, branch `battle/e04b-t11b-b` (pushed after every commit).
Status while working: see the end of this file ("Progress").

## B1. An aged Phoenix is reborn whole — done, commit `abc6c04`

- Test first: `tests/rebirth.rs::an_aged_phoenix_comes_back_whole` failed on the unchanged code
  with `(count, max_hp, top_hp) = (1, 10, 10)` instead of `(2, 10, 10)`: the reason of the
  findings.
- Change: `raise::rebirths` reads `max_hp` after `effects::remove_dead`.
- Golden replay: `full-s3-a955` changes (see "Golden replays").

## B2. A dead stack keeps its permanent effects — done, commit `956b5db`

- Tests first (all four failed on the unchanged code for the findings' reason: the dead stack
  had lost its entry, the raised or reborn stack had its full defense):
  `tests/raise.rs::a_raised_stack_keeps_the_permanent_effects_it_died_with` (Expert Disrupting
  Ray and Haste, death, Resurrection: the ray's entry and −5 stay, Haste is gone; the entry is in
  the observation while the stack is dead),
  `tests/strike.rs::acid_breath_stays_on_a_stack_that_dies_and_is_raised`,
  `tests/rebirth.rs::a_reborn_phoenix_keeps_the_permanent_effects_it_died_with`,
  `tests/created.rs::a_reused_record_starts_without_the_dead_stacks_permanent_effects` (this one
  also fails when `created::add` does not clear the record: checked by taking the call out).
- Change: `effects::remove_dead` keeps the entries with `rounds == None` and recomputes only
  the stacks that lost an entry; new `effects::forget(state, id)` removes every entry of one
  stack; `created::add` calls it for the record it reuses and no longer calls `remove_dead`
  (in play no stack is dead with timed effects when a stack is created: every creation is the
  first thing its command does; the call only mattered to unit tests that kill by hand).
- Callers of `remove_dead` checked: `command::apply` (end of every command: unchanged call),
  `raise::rebirths` (unchanged call; the reborn stack is recomputed from what it kept by
  `revive`), `created::add` (replaced as above).
- Places that assumed "a dead stack has no effects":
  - `tests/properties.rs`, `every_step_preserves_the_invariants`: the invariant is now "an entry
    is on a living stack or has no duration"; I also widened "each stack's values are those of
    its entries" from living stacks to all stacks (it holds: 3 runs × 96 games before the commit).
  - The observation (`observe_as`: `effects: state.effects().to_vec()`), `h3-sim`'s flat
    encoding (per stack: number of effects and the first entries, dead stacks included) and
    `h3-protocol`'s mapping (`stack(..)` lists the effects of every stack) take the list as it
    is: a dead stack's permanent entry now shows there, and its `Stack.spells.defense` stays
    lowered. No field, encoding or version changed.
  - `magic::admits`, `accepts`, `skips_hypnotized`, `casters` (candidates), `after_strike`,
    `combat::specialist_terms`, `magic::reflection` read the list only for living stacks (or
    for the struck stack within its command, as before).
  - `effects::count_down` already skips entries of dead stacks; `round_retaliations` walks every
    entry as before (a permanent Counterstrike exists in no pack).
  - `raise::eligible` leaves Anti-Magic out for a dead stack ("a dead one's effects ended at
    its death"): unchanged, the comment now says "timed effects". A pack with a *permanent*
    Anti-Magic would keep it on the dead stack and this test would not see it; no pack has one.
  - `binds::materialize` binds only a living stack: a pack with a permanent Bind would bind the
    stack again when it is raised (the harvested Bind has 255 rounds and ends at the death).
- Comments updated: `effects::remove_dead`, `raise::raise`, `raise::revive`, `raise::rebirths`,
  `raise::eligible`, `binds::died`, `created::add`.
- Golden replays: six change (see "Golden replays").

## Golden replays (not regenerated)

`cargo test -p h3-cli` fails in `golden_replays_reproduce_their_hashes`. Checked per fix by
recording the eight scenarios with the same seeds into the scratchpad
(`<SCRATCH>/b-logs/rec.sh`, `cmp.py`; nothing written into the repository):

| after | replay | what changed | why |
|---|---|---|---|
| B1 | `full-s3-a955` | command 133 (`A heal 1`) is rejected ("the target is not a hurt allied stack"); a new recording has 472 commands instead of 458, same winner | at command 127 the attacker's Brutes (stack 1, rebirth) die of a retaliation whose strike spells include the synthetic Age (3 rounds); they are now reborn whole (3 creatures at 18 of 18 hit points), so the First Aid Tent has no patient in them |
| B2 | `duel-s1` | final hash only (371 commands and all 14 checkpoints agree) | the dead Spearmen keep the permanent entry of the Brutes' strike spell (synthetic Acid Breath, spell handle 45, defense −3) in the hashed effect list |
| B2 | `skirmish-s7`, `full-s3`, `full-s3-a1766`, `full-s3-a2378`, `full-s3-a303` | the same commands (399, 519, 282, 535, 542) and winners; the state hashes differ from the checkpoints after 225, 225, 125, 375, 175 commands on | the same: from the first death of a stack that carries a permanent entry |
| B2 | `full-s3-a1367` | nothing | |

The benchmark baselines (`benches/battle/`) follow the replays' workloads: not run, not touched.

## Progress

- B1, B2 done. B3 next.
