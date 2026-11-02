# E26 — The fair system actor

Status: **in progress**, refined 2026-11-02 (BAT-Q63); increment 1 open · Lane: **battle**
(the callers' switch to it: the AI, adventure, server and client lanes, by their inboxes) ·
Follows E23 (the original's battle AI, ADR-0027) and E25's refinement (AI-Q10)

## Goal

A system actor that decides only on what its side may know. The original's computer (E23,
`h3_battle_ai::choose`) reads the whole state, the other side's hero's spell book, skills and
hidden pits among it (ADR-0027 point 4 lists what was found). The user decided at E25's
refinement (AI-Q10) that such an actor trains no agent and that a fair variant follows. Once
built it is the default everywhere (the env, training, the server, the clients), with today's
all-seeing computer kept as a game's option. The AI lane may train against it and imitate it
(E25 decisions 2 and 3).

## Decisions at the refinement (the user, 2026-11-02, BAT-Q63)

1. **The port unchanged on its side's world** (ADR-0027 point 4 amended). The fair actor runs
   the original's computer (`h3_battle_ai::unchecked`) on `BattleState::determinized` for its side, the
   search agent's world (E06 T8, ADR-0025 point 12a): a copy of the battle with everything the
   side may not know (`observe_as`'s list) replaced. Its choice is validated on the battle.
   Fair by construction: the world reads nothing hidden (the property
   `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`), and a change of what a
   side may not know changes the actor's world and the search agent's together. Rejected: its
   reads rewritten one by one (many touch points, a fair branch in shared functions, and only
   as fair as the list of reads).
2. **Increment 1 assumes nothing** of the other side's hero's private part: a hero of its type
   with the public part the battle shows (`guess = None`: its primary skills, mana, whether it
   carries a book, the war machines it commands; no spell, no secondary skill, no spell bonus).
   Its hidden pits are placed again by their rule, an enemy random caster's drawn spell drawn
   again (the world's own rules).
3. **Later, a sampled prior and memory** (increment 2): the guess drawn per decision from a
   prior over the hero's private part, updated by what the side has seen of it as it comes in
   (the spells it cast). Refined after increment 1's measurement.
4. **One branch, the `cross-lane` label** for the epic's shared files (this ADR amendment,
   `BACKLOG.md`, `lanes.toml`), not a separate `shared/` pull request (the user, 2026-11-02).

**Agent decisions** (change freely):

- `h3_battle_ai::choose_fair` beside `choose`, which stays the all-seeing original. Additive:
  no caller changes in the battle lane's pull requests; each lane switches its own.
- The world's seed is drawn from a copy of the caller's stream, so the policy's own draws keep
  the original's order. Where the side may know everything (no other hero, no hidden pit, no
  drawn spell of the other side) the world is the battle and the fair choice is `choose`'s.
- One `POLICY_VERSION` for both: a change of the port's choices or of the world (`world.rs`,
  `observe_as`'s list) bumps it.
- A world the battle refuses (`SetupError`; a bare hero never is) gives Defend.

## Non-goals

- A new policy: the fair actor is the port, as read, on another state.
- Switching the callers (`h3-sim`'s `Policy::System`, the adventure's neutral seat, the server's
  unowned sides, the clients' seats) or a game option's type: their lanes', by the inbox notes
  of T5.
- The prior's data and memory before increment 1 is measured.

## Increment 1: the fair actor on a world that assumes nothing

| Task | Scope | Status |
|---|---|---|
| T1 | The design: ADR-0027 point 4 amended, this file, `BACKLOG.md`, `lanes.toml`, BAT-Q63, the queue's tasks | done 2026-11-02 |
| T2 | `choose_fair` (BAT-AI-025) and its unit tests: one per leak ADR-0027 point 4 lists | ready |
| T3 | The properties: reads nothing hidden, no fallback, the original where nothing is hidden, determinism (test ruleset and harvested packs) | ready after T2 |
| T4 | `h3 ai --sight fair`: its cost per decision against `choose` and its strength against the all-seeing one, recorded | ready after T2 |
| T5 | The hand-over and the checkpoint: the other lanes' inbox notes, docs, TRAPS, the closure | after T3, T4 |

The tasks' scope and acceptance: the battle lane's `QUEUE.md`; their record: `E26-log.md`.

## Increment 2 (sketch, refined after T4's measurement): the guess

- A public record of what a side saw of the other side's hero: the spells it cast (a state
  field, its observation and the AI lane's encoding, the protocol: `cross-lane`, a state hash
  bump). It also closes `world.rs`'s first known gap (a guessed book may lack a spell cast).
- A prior over the hero's private part (its book by its class and level, its skills, its
  bonuses), drawn per decision from the policy's stream, conditioned on the record; its data
  in the packs.
- AI-STUB-1's gaps: the number of the hidden obstacles from the guessed mastery, the drawn
  spells of the stacks not active.
- Measured against increment 1's fair actor and the all-seeing one.

## Exit criteria

1. The fair actor reads nothing its side may not know: a property over random battles, both
   sides, sieges and special battlefields, with twins that differ only there.
2. The engine accepts every fair choice (ADR-0027 point 3): no fallback fires.
3. Its cost per decision is measured and inside ADR-0027 point 11's budget, its strength
   against the all-seeing one recorded.
4. The callers' lanes have it in their inboxes; the docs and the ADR say what it is.
5. Increment 2's exit criteria come with its refinement.

## Risks

- **Cost:** a copy of the battle, a hero check and the templates per call, two calls per
  activation (the hero's cast, the stack's action). T4 measures it; a cheaper world (no
  templates when no spell creates a stack) only if it is over the budget.
- **A world's legal commands may differ from the battle's** where legality reads something
  hidden; the validation on the battle then gives Defend. T3's property finds such a case; a
  stub records it.
- **Weaker play** where the original's estimates rely on the other hero's magic (its area
  threat, protections, the odds): measured in T4, the reason for increment 2.
