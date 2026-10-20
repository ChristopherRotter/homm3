# E06 — Log

The full record of E06's done tasks (`docs/epics/E06-battle-ai.md` keeps one line each). Read an
entry only where a task builds on it.

## T1 — Design and ADR draft (2026-10-20)

- **The seam:** offered T1 narrowed to the run's lifecycle (config, record, resume) with the gate,
  the ladder and the league designed at their own increments; the user chose T1 as queued, the
  whole pipeline's design in one note.
- **Written:** `E06-design.md` (§1 the run and its config, §2 the schedule in samples, §3 the
  resumable state, §4 the records, §5 the held-out sets, §6 the gate, §7 the ladder, §8 the
  league, §9 the costs, §10 the increments) and the ADR draft, nine decisions.
- **The user's review** (2026-10-20): every point as proposed, the ADR accepted as **ADR-0026**
  (#306, `shared/`). The points reviewed:
  1. TOML configs in, JSON records out; committed records in `python/records/`, configs in
     `python/configs/`.
  2. On resume only the budget, threads, device and checkpoint cadence may change; anything else
     is a new run (`--init-from`).
  3. The learning rate's horizon fixed at the start; an extended budget runs at the floor.
  4. Checkpoints every 5 minutes of wall time (they change no training); every training cadence
     in samples.
  5. The report set v1: 500 per (kind, side), stratified; 250 per kind for intermediate
     ladders. A separate gate set.
  6. The SPRT: elo0 0, elo1 20, α = β = 0.05, at most 4 000 pairs, no promotion at the cap, no
     second condition against the bots.
  7. The targets judged on the latest promoted network, point estimates against the targets.
  8. Ratings only in `h3 ai eval`, outside the promotion.
  9. The league: self-play 0.5, PFSP "hard" (power 2), 16 members, 4 live per rollout, a
     snapshot every 1 M samples.
  10. T2 in three pull requests (T2a to T2c); the observations in place moved to T5.
- **The cut:** increment 1 continues with T2a, then T2b and T2c (design §10); T1 and T2a go to
  `master` in one pull request (the user).
- **Verify:** the sandbox's full verify on engine code `d95c95dd` (no original data) green; the
  scoped verify of the docs change with no failing step. CI's runs on `master` never started
  (minutes).
