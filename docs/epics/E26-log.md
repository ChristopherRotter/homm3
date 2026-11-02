# E26 — log

The record of E26's tasks: what each built and decided, beyond the one line in the epic file.

## T1 — the design (2026-11-02, increment 1)

- The user's decisions at the refinement (BAT-Q63): the port unchanged on its side's world, not
  its reads rewritten; nothing assumed of the other hero's private part now, a sampled prior
  updated by what the side has seen later; the epic's shared files on its branch with the
  `cross-lane` label. Recorded in the epic's decisions, ADR-0027 point 4's amendment,
  `BACKLOG.md`, `lanes.toml` (`docs/epics/E26*` is the battle lane's).
- Found while refining: `BattleState::determinized` (E06 T8) already is the world the fair actor
  needs, with its property `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`. For
  the side to act the drawn spell is never redrawn: the active stack is the side's own.

## T2 — `choose_fair` (2026-11-02, increment 1)

- **Built:** `h3_battle_ai::choose_fair` (`lib.rs`, BAT-AI-025): the world from
  `determinized(ruleset, side, None, rng.clone().next_u64())`, `unchecked` on it, `checked` on
  the battle, Defend for a world the battle refuses (`map_or`: no branch of its own). Never
  inlined, as `choose`. No caller changed; `POLICY_VERSION` stays 19 (`choose`'s choices are
  unchanged).
- **Tests** (`h3-battle-ai`'s `src/tests/fair.rs`): twins whose other hero differs only in its
  book, masteries and damage skills (Archery, Offense and Armorer as `DamageBonuses`), the other
  side defending, compared while the side's observations agree (they part when the skills change
  an outcome): 70 decisions over 24 battles, the all-seeing command differed in 12, the fair one
  in none, and the stream is left the same. With no other hero the fair command and stream are
  `choose`'s (150 decisions); with one, the engine accepts the fair command.
- **Moved to T3:** the other side's hidden pit (the test ruleset has no hidden obstacle; the
  harvested packs' Quicksand and Land Mine).
