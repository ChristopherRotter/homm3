# HANDOFF

Living state. The last agent updates this at the end of every session; the next agent reads it first. Newest entry on top; keep old entries short (or prune) once superseded.

---

## 2026-09-30 — E00 foundations (planning session with the user)

**State of the repo:** documentation only. No code, no Cargo workspace, no CI yet. `cargo xtask verify` does **not** exist yet (created in E01 task T1).

**What was done:**
- Defined vision, architecture principles, bounded contexts, source-of-truth and provenance approach, quirks/profiles, testing gates, security model, simulation/AI strategy, work process.
- Recorded ADR-0001 … ADR-0011.
- Wrote the backlog (coarse) and refined E01 (walking skeleton).

**What is next:** Start E01 (`docs/epics/E01-walking-skeleton.md`), task T1 (workspace scaffold + `xtask verify` + CI skeleton).

**Waiting on the user (see OPEN-QUESTIONS):**
- Q1 license choice and VCMI policy — interim: do **not** copy/translate VCMI code. No `LICENSE` file has been added yet; add one once decided (E01 T1 may proceed without it, but note it).
- Q2 project/crate naming (working prefix `h3-`).
- Q8 which game version is the "original" baseline; Q9 whether base content tables may be committed.
- Others are lower priority and do not block E01.

**Traps / notes for the next agent:**
- Do not assume original data file names or layouts from memory; catalogue them from the files the user supplies (E02).
- Keep the skeleton *minimal*: its purpose is to exercise the architecture, not to be a nice battle system. Resist adding spells, heroes, or abilities.
- All tests in CI must run on synthetic content packs (neutral invented names).
- Determinism/numeric policy in ADR-0003 is strict: integers by default, floats only via `kernel::num` (no platform transcendentals, no fast-math), no `HashMap`/`HashSet` in logic crates, every sort has a unique tie-breaker.
- The Rust toolchain/versions are not pinned yet; pin in `rust-toolchain.toml` during T1 and record the MSRV policy in ADR-0002 if it needs refining.
