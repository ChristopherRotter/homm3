# E09 log: done tasks in full

The full record of E09's done tasks (what was built, agent decisions marked "change freely",
measurements). **Not start-of-session reading:** the epic file keeps one line per done task; read
the matching section here when a task builds on an earlier one or a decision is questioned.

## Done

### T0 — E09 refined, the town lane opened (2026-10-31)

One session, the user present; branch `shared/town-lane` (`cross-lane`). Starting point: `master`
at 11f4a137 (E08 T84a merged), the adventure lane next at E08 T84b. The user's answers (TWN-Q1,
two rounds of `AskUserQuestion`) are the epic's decisions 1–6; the agent's calls are decision 7.

- `lanes.toml`: the lane `town` (`TWN`) with `crates/h3-towns/`, the town modules of the adventure
  lane's crates (`h3-adventure` and `h3-game` `src/town*`, `h3-map` `src/import/town*` and
  `src/writer/town*`), the town catalog and quirks ledger, E09's epic, surveys and experiments
  (moved from the adventure lane), its folder; its queue common; `verify_with = ["adventure"]`.
- `xtask`: `Lane::verify_with` (validated: another lane), `Config::verified_lanes`;
  `verify::LaneScope::lanes`, the scope's packages and replays the union of the covered lanes',
  the benchmark gate's workloads by `bench::Selection::Lanes`; tests for each. *Agent's call
  (change freely):* the scoped verify of a lane owning sub-paths of another lane's crates runs that
  lane's crates, rather than `lane_packages` counting any owned sub-path (which would have widened
  the adventure and AI scopes to `h3-sim` and `h3-cli`).
- Docs: `AGENTS.md` (the lane, its reading list), `docs/08-work-process.md` ("Lanes": ownership,
  ids, versions), the global HANDOFF's table, `docs/README.md`, `docs/03-domain.md` (the Town
  crate decided), BACKLOG (E09's row, the ordering rationale), the registries' TWN blocks,
  `scripts/sandbox-setup.sh`'s comment; the town folder (HANDOFF, QUEUE with T1a–T2 and an inbox
  of the earlier notes for E09, TRAPS, QUESTIONS-ANSWERED); the adventure lane's inbox note.
