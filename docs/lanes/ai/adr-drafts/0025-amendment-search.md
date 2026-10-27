# ADR-0025 amendment (draft): a search agent's worlds

- Status: Accepted by the user on 2026-10-26 (E06 T8a; the user chose a determinized search from
  the start the same day); goes into ADR-0025 through a `shared/` pull request.
- Amends: ADR-0025 point 12 (information: the state, its hash and forks are trusted tooling,
  never an agent's input) with a point 12a.
- Detail: `docs/epics/E06-design.md` §14.

## Context

E06's T8 (decision 2) searches over forks of the battle. A fork of the true state would show
the searcher the other side's hero's private part, its hidden spell obstacles, a random caster's
drawn spell and the battle's stream, which predicts every roll: what no human player sees
(ADR-0025 point 12, `docs/06-security.md`). The user chose a determinized search from the start,
so that the searcher is a legitimate agent on the ladder and later in any seat.

## Decision

12a. **A search agent's worlds.** A search agent reads its observation and *worlds*: copies of
     the battle in which everything its side may not know is replaced by a guess drawn from the
     world's seed, the rest as it is.
     1. One trusted function makes a world (`h3_batch_determinize`; in the engine
        `BattleState::determinized`, beside `observe` and its list of what a side may not
        know): the other side's hero's private part from a guess consistent with its public
        part, its hidden spell obstacles and a drawn spell placed or drawn again by their rules,
        the battle's stream re-seeded.
     2. A world depends only on what the viewer may know and its seed: two states that differ
        only in the rest give the same world (a test of the function). A change of `observe`'s
        list changes the function in the same change.
     3. A world's forks (`h3_env_fork`) and steps are the agent's to use; the true state's
        fork and hash stay trusted tooling, never an agent's input.
     4. The guess is the AI lane's (the encounter sampler's hero draws, `h3-sim`); a better
        guess changes no other lane.

## Consequences

- A search agent plays under the same information rule as a network; it may take the ladder's
  seats and, later, a server's.
- The battle lane keeps the list of what is hidden and the function that removes it in one
  place; the AI lane the guess.
- A guess unlike the opponent's real hero makes the search weaker, never unfair.

## As built (T8b, 2026-10-27)

- Built as decided: `BattleState::determinized` and `reseed` (`h3-battle` `world.rs`),
  `h3_batch_determinize` and `h3_env_reseed` (C ABI v34); point 2's test is a unit test and a
  property over arbitrary games, with the viewer's observation of a world equal to its
  observation of the battle.
- **One gap against point 2** (registered, AI-STUB-1; for the user's review at T8b's
  checkpoint): a world keeps the number and value of the other side's hidden obstacles (only
  their hexes are drawn again), which follow the caster's private mastery, and the last drawn
  spells of random casters not active (read only by the computer's cast choice).
