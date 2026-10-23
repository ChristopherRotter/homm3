# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client (opened 2026-10-23 when the user split the client lane at
E17 M3's closure, CLT-Q18; `docs/BACKLOG.md`). The lane's state, ownership and the asset notes:
`HANDOFF.md`.

### E24 T0 — E24 refined with the user

- **Scope:** the epic file `docs/epics/E24-map-client.md` (goal, the user's decisions, milestones;
  a design notes file when there is detail) and its first milestone's tasks in this queue, each
  about half a context. The agent's starting proposal, for the user to judge: a first milestone of
  **stills of an adventure map**, an `.h3m` (a shipped map, or a situation `h3_map::writer`
  writes) drawn from its own data with the original's assets: the terrain with its rivers and
  roads, the objects and their shadows in the original's draw order (research item 9, a code
  reading of the executable's map drawing confirmed by reference frames), heroes, the fog of war,
  the adventure screen's frame and panels; each compared exactly with the original's frames,
  which the harness captures at the same view (`h3adv.py` loads the map and moves the view; a
  capture set of map situations as E17 M1 to M3 built for the battle). Later milestones,
  refined when they come: the map animated (the terrain's palette cycles, the objects'
  animations, a hero's walk), the panels and the hero and town screens, the engine's game shown
  (a record of whole games: the server lane's protocol has no adventure messages yet, E10/E16),
  hot seat (the inbox). The refinement names what this lane needs of the client lane's
  `h3-view` (a common crate, `HANDOFF.md`) and of other lanes (their inboxes), and the open
  questions it raises (`MCL-Q<n>`).
- **Acceptance:** the epic file and the first milestone's tasks, agreed by the user; the
  BACKLOG's E24 row says "Refined" (a `shared/` pull request, or with the user's consent the
  same pull request, `cross-lane`, as the AI lane's opening did).
- **Depends on:** nothing. **Status:** ready (with the user).

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-10-23 (the client lane, E17 M3 T9c, at the split): moved here from the client lane.
  **Research item 9** (the user, 2026-10-12, at E17 M0): the adventure map's object draw order is
  surprisingly complicated; the user's own engine and VCMI (at the time) did not get it exactly
  right: a code reading of the executable's map drawing (the order of objects, their shadows,
  heroes and overlapping tiles) when the adventure map viewer is refined, with reference frames
  from the original to confirm. E17 M1 T5 read only the battle's (`docs/epics/E17-design.md` §4).
- 2026-10-23 (the client lane, E17 M3 T9c, at the split): moved here from the client lane's
  inbox, the adventure lane's entry of 2026-10-21 (E08 T30, decision 65, ADR-0017 point 6 amended
  in #353; for the hot-seat client, when it plays whole games): **a level-up holds the game.**
  While a hero's level-up with two offers is open, the engine accepts no command but its owner's
  answer (`h3_game::GameState::waiting_for` names the owner; `level_up(owner)`, or the owner's
  `GameObservation.decision` as `DecisionView::LevelUp`, gives the hero, level, primary skill and
  the two offers; `Answer::First`/`Second` answer it). A battle's defender that won answers it
  outside its turn. The user's requirement for hot seat: when the player who must act changes,
  show a handover screen that hides everything (map, hero list, the dialog) until that player
  confirms it is in control, then the level-up dialog alone, and hand back to the player whose
  turn runs the same way, so no one sees another player's view.
- 2026-10-23 (the client lane, at the split): the user's graphics options (2026-10-13, at E17
  M1: per effect the original's way or a modern one, shadows first; `faithful` the default and
  the only setting the reference frames check) bind this lane too: the client lane's
  `NOTES.md`, "Options and layouts". An HD layout needs the view's origin and viewport in the
  layout format.
