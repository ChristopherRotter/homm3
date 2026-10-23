# ADR-0027: The original's battle AI: a policy crate beside the battle

- Status: Accepted (E23 T1b, BAT-Q55; point 9 the user's own: one flag for all the slips); point 14
  added 2026-10-23 (the user, BAT-Q61 and BAT-Q56; E23 T6)
- Date: 2026-10-20
- Deciders: the user; drafted by the battle lane's agent (E23 T1b)

## Context

E23 ports the original's computer as the policy of a battle side (the epic's goal; ADR-0017 point
4, ADR-0025 point 5). Its reading is complete for a side without a hero (`installation.md` §7.34 to
§7.46; the survey `docs/rules/surveys/E23-battle-ai.md`); the hero's cast choice and retreat test
follow (T1a6). The refinement decided a crate of its own and that `h3-battle` keeps the
computer's numbers its rules use (BAT-Q52). Layers forbid a domain crate to depend on another.

## Decision

1. **The crate `h3-battle-ai`**: a logic crate (pure, `no_std`, the coverage gate) in the
   **application** layer, depending on `h3-kernel`, `h3-content`, `h3-content-battle`,
   `h3-contracts` and `h3-battle`. The battle lane owns it. Its registration (the workspace,
   `arch.toml`, `lanes.toml`) is a `shared/` pull request before T2.
2. **The function**: `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) ->
   Command` for the side to act. One call gives one command. The hero's cast and the stack's
   action of an activation are two calls. Once the hero has cast this round, the state says so and
   the hero's part is skipped, so the draws come in the original's order.
3. **Always legal, always the same**: it validates its choice (`h3_battle::validate`). If the
   original's choice is illegal under the engine's rules, it falls back to Defend and records the
   difference as a stub. A property test asserts that no fallback fires and that the command is the
   same for the same state and stream: random battles, both sides, every difficulty, special
   battlefields and sieges (exit criterion 1).
4. **It reads the whole state, never an observation, and writes nothing** (ADR-0017 point 4: the
   trusted host). It predicts with the original's estimates, never by applying commands to a copy.
5. **Its draws come from the caller's stream**, never the battle's (ADR-0003 streams). They keep
   the original's order and distributions (`random(a, b)` as `range_inclusive`), but are not
   draw-exact with its `rand()` (a non-goal).
6. **One implementation of the computer's numbers.** `h3-battle` exposes a public module
   `computer` with the numbers its rules already use. That is `aim.rs`: the battle context, worth,
   kill value, the damage estimate, the shot value, the turns, the threat records. It also exposes
   the pathfinder's `Search` (0x4B3400). The rules and the policy call the same functions.
   Anything else the policy needs from private state is added there by the battle lane. Everything
   else lives in `h3-battle-ai`: the turn, the shooter's and the walker's choices, the walk, the
   predictions, the spell context and the evaluators. All its arithmetic goes through
   `h3_kernel::arith::X87` (ADR-0003), as `aim.rs` does.
7. **A move carries only a destination.** The engine walks the original's path (BAT-MOV-003, E23
   T1c). The original's walk ends on its own path to the target, which is the search tree's path,
   so the destination alone gives the original's steps.
8. **The difficulty and `human`**: the policy reads `BattleSetup.difficulty`. It plays its side as
   the computer's whatever `human` says (AI-Q3), and reads the other side's `human` where the
   original reads the acting side's player (the predictions' retaliations on Easy).
9. **The original's slips behind one fix-optional flag** (the user, BAT-Q55): where its estimate
   reads a wrong record, argument, spell, side, mastery or test compared with the rule it
   estimates (survey §4), BAT-QUIRK-25 `quirk.battle.original_ai_slips` keeps the slip in
   `faithful` and estimates as the rule in `strict-fixed`; a later slip joins the same flag. Each
   slip's rule names both variants and its tests cover both. Not slips, ported as read in every
   profile: its orders and ties (the library sort of the order of action above 16 stacks, §7.37;
   the later record, the first of equal hexes) and its draws. The flag is registered (`h3-content`,
   `shared/`) right before the battle pull request that builds the first slip.
10. **Rule ids `BAT-AI-nnn`** in `docs/rules/catalog/battle.toml`, one per routine of the reading:
    the dispatch, the shooter, the hex scores, the walker's choice, the walk, the fallbacks, each
    cast kind, the context, the evaluators by family, and later the hero's cast and the retreat
    test. Each has the source `code` (its §7.x) and its batch rows (survey §3) once played. The
    automatic machines keep BAT-WAR-008 and BAT-SIE-020 to 023.
11. **The cost budget is measured, then set.** From T2 on, a gated benchmark (`benches/ai/`, with
    the AI lane: a batch with `System` neutral seats) counts instructions per decision. T2's
    checkpoint sets the budget from that count; until then the ceiling is 1 ms per decision on the
    sandbox. An optimisation never changes a choice (for example, caching the threat records'
    searches within one decision).
12. **A behaviour version**: `h3_battle_ai::POLICY_VERSION`, bumped with every change of a choice.
    The AI lane's engine identity includes it, so a training record names its opponent. Replays
    store commands and do not change.
13. **The data it needs**: the creatures' Fight and AI values are in the packs. The spells' AI
    values (`SPTRAITS.TXT`, four per mastery: the hero's value in the odds, §7.39) join the battle
    pack before T3, because the neutral side reads them for the enemy hero.

14. **The adventure's facts** (amendment, E23 T6; the user, BAT-Q61 option a, BAT-Q56 option b):
    the retreat test (0x41E570) and the odds' hero value (0x420A80) read facts of the adventure
    that no battle rule needs. The setup carries them derived, as ADR-0023 carries a town's:
    `BattleSetup.retreat: [RetreatFacts; 2]` (the hero's experience, its artifacts' worth by the
    computer's count, a Tavern town to be hired again from, whether the scenario wants the hero
    defeated; private to the side) and `BattleSetup.attacker_ground` (the special battlefield of
    the attacking hero's own tile; public). The battle keeps them as hashed state; no rule reads
    them, so they stay out of the observation (ADR-0014 point 8 covers what rules read). The
    defaults are a hero of whom nothing is known: it never retreats (no Tavern), and the attacker
    stands on an ordinary ground. The game orchestrator derives them (the adventure lane);
    scenarios and replays name them. The viewers' setup fields (CLT-Q4, CLT-Q6) come when a
    full game's viewer needs them.

## Consequences

- Callers: `h3-sim`'s `Policy::System`, and the adventure's neutral seat (`AdventureEnv`, the
  server's unowned sides at M5). Both lanes get the function in their inboxes once the neutral side
  is complete (T4).
- `aim.rs` and `pathfinder.rs` turn public through `computer`: their signatures become a surface
  that the battle lane changes together with `h3-battle-ai`.
- A change to a rule the computer estimates moves the shared numbers once. A choice drifts only if
  the policy's own code changes.
- New pack data (the spells' AI values) bumps the pack format and the core ruleset hash; the new
  quirk flag changes the faithful ruleset hashes (the golden and the adventure lane's faithful
  replays' headers, `cross-lane`).
- In `strict-fixed` the `System` seat is not the original's computer but its corrected estimate:
  a training run that wants the original's opponent uses `faithful`.

## Alternatives considered

- **A module of `h3-battle`**: rejected by the user at the refinement (BAT-Q52).
- **A domain-layer crate**: domain crates never depend on each other (ADR-0005).
- **The policy on observations**: the original reads everything. An observation-only policy would
  not be the original's.
- **Predicting by applying commands to a forked state**: costlier, and not what the original does.
  Its estimates are not the rules' outcomes, slips included (point 9).
- **The slips ported as read without a flag** (the draft's first proposal), **a flag per slip**, or
  **fixed in every profile**: the user chose one flag for all (BAT-Q55).
- **The battle's stream for the policy's draws**: it would couple a seat's choices to the battle's
  rolls. ADR-0017 point 4 keeps agents' choices off the game's streams.
