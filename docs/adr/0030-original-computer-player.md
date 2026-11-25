# ADR-0030: The original's computer player: a policy module of the game

- Status: Accepted (the user, 2026-11-25, AI-Q32, at E31 increment 2's start; drafted at E31 T1b,
  the checkpoint AI-Q31: point 1 a module for now, points 5, 7 and 10 as proposed)
- Date: 2026-11-24
- Deciders: the user; drafted by the ai lane's agent (E31 T1b)

<!-- Working detail: docs/epics/E31-design.md; the reading: docs/rules/surveys/E31-computer-player.md. -->

## Context

E31 ports the adventure turns of the original's computer player (`Heroes3.exe`, SoD) as a policy
outside the game state, as ADR-0027 did for battle. The reading is complete: 255 functions in three
ranges, each in a phase (survey §4.3), the draws (§4.4), 1 097 x87 instructions (§4.5), what
persists (§4.6), every `ai_*` call sorted into a choice or a rule (§3.6, a ninth rule in §5.3),
the hidden reads and the engine's gaps (§5). The refinement fixed reading first, the computer's rules built by their lanes,
faithful then fair, the memory decided after the reading, port now and stub the gaps (AI-Q22).
The policy reads the whole game (`h3-game`, application layer); `arch.toml` lets a crate depend
only on strictly lower layers, so a crate beside `h3-battle-ai` could not depend on `h3-game`.

## Decision

1. **A module of `h3-game` for now** (the user, AI-Q31): `h3_game::computer` in
   `crates/h3-game/src/computer/`, owned by the ai lane by path (`lanes.toml`), under `h3-game`'s
   purity scan and coverage gate. It uses only `h3-game`'s public surface and the crates below it,
   so that its **later refactor into a crate of its own** (a `policy` layer above application was
   the draft's proposal) moves files and changes no choice. The path is registered with
   `docs/rules/catalog/computer.toml` and `docs/rules/quirks-ai.md` (this ADR's `shared/` pull
   request, before increment 2's code); the ai lane's scoped verify runs the adventure lane's
   crates too.
2. **The function**: `h3_game::computer::choose(&GameState, &GameRuleset, PlayerId, &mut Memory,
   &mut Rng) -> Option<GameCommand>`: the next command of a computer player when the game asks
   something of it on the map (its turn, an open decision in any player's turn), `None` otherwise.
   One call, one command; the host applies it and calls again; a turn ends with `EndTurn`. Its
   battle side is played by the host's battle seat with `h3_battle_ai::choose` (faithful) or
   `choose_fair` (fair), as its player's `GameCommand::Battle` (the module cannot call
   `h3-battle-ai`: one layer). Hosts: the env's computer seat (`h3-sim`), the server's computer
   players and the shell's, each through its lane's inbox.
3. **Always legal**: every command is one `h3_game::apply` accepts for that player (ADR-0004).
   Where the original's step is illegal under the engine's rules, or its rule is missing (a gap),
   the step is dropped (the hero is done for the turn; an answer is `default_answer`'s) and the
   gap is an `AI-STUB-n` naming its epic. Property tests on random and shipped maps, every
   difficulty: no command refused, no fallback where the port is complete, a turn ends within a
   bound of commands, the same command for the same state, memory and stream.
4. **It reads the whole state and writes nothing.** What the original writes and undoes inside a
   value (the hire tried in `ai_town_hero_prospects`, a skill level set and reset, the walk's boat
   marks in the map's tiles, survey §3.6) the port computes on its own values; the pathfinder
   takes the marks as an overlay of the query (the adventure lane's API, its inbox).
5. **The memory is the policy's, outside the state** (decision 5): `h3_game::computer::Memory`, one per
   computer player: carried between its turns only each hero's destination and its points beyond
   (+0x35, +0x41); within a turn the phase, the heroes done, the must-go, the records and weights,
   the danger map and the threats, rebuilt at its turn start (§4.6). A plain value with a canonical
   encoding and a hash; the host keeps it beside the journal and saves it with a saved game; a
   replay rebuilds it by running the policy at the journal's computer commands, checked command by
   command. **No field moves into the state**: no engine rule reads one today; a rule that needs
   the computer's numbers (a default answer, a quick battle) computes them from the state it sees
   (point 6). A record starts empty at the player's first turn start in a game (the original's
   survive from an earlier game, §4.6: a quirk candidate, not reproduced).
6. **One implementation of the computer's numbers.** The policy's values live in
   `h3_game::computer`; a rule of `h3-game` that reads one (the level-up's default answer,
   ADV-STUB-8) calls it there. A rule of a lower crate (the quick battle, §3.6 row 1; the
   defender's split, row 8) gets the value moved to a public `computer` module of that crate (owned
   by the ai lane by path, as ADR-0027 point 6's `aim.rs`), and both call it.
7. **Battles.** A battle the game fights is played on the computer's side by the host's seat
   (point 2). The
   original fights no battle without a human side (`ai_quick_battle`, §2.2): until the battle and
   adventure lanes decide that rule (decision 2), every battle is fought, the neutral side by the
   system actor's seat; the policy does not depend on which.
8. **Its draws come from the caller's stream** (ADR-0017 point 4, ADR-0027 point 5): the
   destination's noise, 76 to 100 for each candidate scored (§1.9), in the original's order;
   not draw-exact with its `rand()`. The turn start's reseed through the Grail target (§4.4)
   waits for its experiment.
9. **The original's arithmetic**: every value through `h3_kernel::arith::X87` with the original's
   widths, order and truncations (`crt_ftol` toward zero, the f32 stores of f64 results). `X87`
   is nonnegative below 2^64; the computer's values are signed: `h3_kernel::arith::SignedX87`
   (sign and magnitude, `_ftol`'s truncation and its callers' low 32 bits; audited against IEEE
   doubles, ADR-0003's revision of 2026-11-25), joined with this ADR.
10. **The original's slips behind one fix-optional flag**: where a value reads a wrong record,
    artifact, argument or test, or computes a slip (a 32-bit wrap, a wrong constant) and changes
    only the computer's choices (§5's `slip` rows), AI-QUIRK-1 `quirk.computer.original_ai_slips`
    keeps it in `faithful` and values as the rule in `strict-fixed`; each is confirmed by a blind
    re-read before it is built (Q30) and its rule tests both. A candidate that changes the game's
    state is its own entry in its lane's ledger; orders, ties and draws are ported as read.
11. **The difficulty and `human`**: it reads `GameSetup`'s difficulty (all five) and the players'
    `human` flags where the original does (the danger map, the allies' shares), and plays a
    player as the computer whatever its `human` says.
12. **Rule ids `CPU-<area>-nnn`** in `docs/rules/catalog/computer.toml`, one per routine of the
    reading (the frame, the planning, the walk, the values by family, the simulated combat, the
    towns, the hires, the answers), each with its code reading's source (`original-data`, the survey's §) and its trace.
13. **Cost and version**: a gated benchmark counts instructions per computer turn from increment
    2 (`benches/ai/`); increment 5's checkpoint sets the budget (until then 50 ms per turn on the
    sandbox). An optimisation never changes a choice. `h3_game::computer::POLICY_VERSION` is bumped with
    every change of a choice and joins the lane's engine identity.
14. **The fair variant** (increment 6): this port, unchanged, on its player's world: a copy of the
    game with everything the player may not know replaced (the projection, `docs/06-security.md`;
    built in `h3-game` by its lanes, as `BattleState::determinized` for battle), its first version
    with plain guesses; §5's hidden reads are its checklist. Property: the command is a function of
    the player's projection. `choose` stays the faithful original.

## Consequences

- `h3-game` holds a second lane's module, as it holds the town lane's: its build, coverage and
  `check-lane` cover both; the hosts (`h3-sim`, `h3-server`, the shell's adapter) call it with
  their battle seat beside it.
- The memory is not hashed state: a replay or a saved game carries it or rebuilds it (point 5); a
  host that loses it changes only the computer's destinations of one turn.
- The nine rule rows (§3.6, §5.3) are the lanes' (their inboxes, E31 T1b); the policy plans as a
  human until each lane decides (a path on explored tiles only, ADV-MOV-008), and a decided rule changes the policy's plan only where the rule is the original's.
- The slips' flag changes the faithful ruleset hashes when it is registered (increment 2 or
  later), `cross-lane` for the replays' headers, as BAT-QUIRK-25's did.

## Alternatives considered

- **A crate `h3-game-ai` in a new `policy` layer** (the draft's proposal): the clean boundary, but
  a layer change before any code; the user chose the module now and the crate later (AI-Q31).
- **An adapter crate** (`h3-sim`'s layer): adapters are not logic crates (no purity scan, no
  coverage gate).
- **The policy on domain crates only** (no `h3-game`): its answers and commands are `h3-game`'s
  types, and it must see the whole game.
- **The memory in the game state**: hashed and saved for free, but every computer's private plan
  in every player's state and its encodings; the original keeps none of it in a save but the
  destination.
- **The quick battle decided here**: it is a rule of the battle's outcome, the lanes' (decision 2).
- **A flag per slip, or the battle's flag reused**: as BAT-Q55 decided for battle, one flag; the
  computer player's own, so each AI's original can be chosen alone.
