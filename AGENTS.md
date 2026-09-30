# AGENTS.md — read this first

This repository is a **headless, deterministic, high-performance rules engine for Heroes of Might and Magic III** (RoE base game plus the official AB and SoD extensions), written in Rust. Its main purpose is to serve as a fast simulation backend for reinforcement-learning self-play; playable clients come later and live outside the core.

You are one of many agents working on this over time. No single session finishes the project. **Your job is to move the repo forward one coherent increment and leave it in a state the next agent can pick up.**

## Start-of-session checklist

1. Read `docs/HANDOFF.md` (current state, what to do next, traps).
2. Read `docs/OPEN-QUESTIONS.md` (anything blocking or recently answered).
3. Read the epic file you are working on (`docs/epics/`) and the relevant parts of `docs/BACKLOG.md`.
4. Skim `docs/02-architecture.md` and the ADRs in `docs/adr/` that touch your task. ADRs are binding; do not silently deviate.
5. Run `cargo xtask verify` (once it exists — see the backlog; until then run the commands listed in HANDOFF) to confirm the repo is green **before** you change anything.

## Non-negotiables

- **Game logic is pure.** Logic crates have no I/O, no clock, no threads, no globals, no ad-hoc floating point (floats only via the audited `kernel::num` surface, ADR-0003), no `HashMap`/`HashSet`, no sorts without a unique tie-breaker, no graphics/audio/i18n/asset concerns. Enforced by `cargo xtask check-arch` (ADR-0003, ADR-0005).
- **Deterministic.** Same ruleset + initial state + seed + command sequence ⇒ bit-identical state and events on every platform (ADR-0003).
- **Server-authoritative and validated.** Every command is validated against state and the *trusted* actor identity. Never trust payload-supplied ownership, prices or amounts. A rejected command must leave state untouched (ADR-0004, ADR-0008, `docs/06-security.md`).
- **Domain-driven.** Code follows the bounded contexts in `docs/03-domain.md`. Cross-context interaction goes only through the contract types. Use the ubiquitous language from the glossary.
- **Nothing gameplay-relevant is hard-coded** when it can be data (tiers 1–3 in ADR-0006). But **architecture and performance win** over configurability when they conflict; record such a call in an ADR.
- **Every game rule has a rule ID, a source, and a test.** Rule IDs live in `docs/rules/catalog/`. Sources are labelled (wiki pages with URL, original data, observation, VCMI behavior…). See `docs/04-rules-and-sources.md`.
- **Known original bugs** are handled per the quirks ledger and profiles (`docs/rules/quirks-ledger.md`). Exploits and state-corrupting bugs are always fixed.
- **100% coverage** on core crates, with an explicit, reviewed exclusion list. Plus property tests, determinism replays and the other gates in `docs/05-testing.md`.
- **License hygiene.** The project is **GPL-2.0-or-later** (ADR-0009). VCMI (GPL-2+) may be read and its code ported or adapted, with an origin comment and a `kind = "vcmi"` rule source; ported code must still obey our purity/determinism/validation rules. Dependencies must be GPLv2-compatible (`deny.toml`). Do not commit original game files, assets or flavor text (harvested gameplay data and names in `packs/` are the exception, Q13).
- **The original game installation is read-only.** The user's installation (on the user's machine at `D:\Heroes3\HoMM 3 Complete`, connected to sessions as an additional folder) must never be modified: do not edit, create, rename, move or delete anything in it, do not run tools that write into it, and never request delete permission for it. Copy what you need into the repo's `original-data/` directory, which is gitignored; never commit original files (ADR-0009; `check-arch` fails if anything under `original-data/` is tracked). The *harvested* packs under `packs/` (gameplay data and names, no flavor text) are committed by design (Q13).
- **No scope creep into clients, assets, graphics or sound** inside the core.

## When to stop and ask

Stop, write the question in `docs/OPEN-QUESTIONS.md` (context, options, your recommendation), and ask the user interactively if they are present, when you hit any of:

- a change to an ADR-level decision, a seam/contract type, or a public crate boundary,
- license, legal or IP doubts,
- conflicting sources for a rule with real gameplay impact,
- an ambiguous quirk classification (is this bug harmless-and-accepted, or must it be fixed?),
- a dependency that is heavy, not GPLv2-compatible (see `deny.toml`), or unmaintained,
- anything irreversible or that would invalidate a lot of prior work,
- being unsure whether something is "big". When in serious doubt, ask.

If the user is not present, do the safe preparatory work, document the decision needed, and stop at that point rather than guessing.

## Working rules

- Small, coherent commits; keep the main line green. Partial delivery is fine; a red or undocumented repo is not.
- Incomplete features must be visible: use the stub registry convention in `docs/08-work-process.md` (no silent `todo!()`, no untracked `#[ignore]`).
- Prefer the smallest design that satisfies the current epic. Do not over-specify the future; do not paint the project into a corner either (see the guardrails in ADR-0006).
- Keep documentation current in the same change as the code (architecture, rules catalog, quirks ledger, ADRs).

## Interruptions and resuming

The user may stop a session at any moment (for example by shutting down the computer) and resume later, in the same task or in a new one. Work so that an unplanned stop costs at most the step in progress:

- **The user's repo folder is the only durable copy.** A cloud sandbox is temporary: it can be reclaimed while idle, and anything that exists only there (uncommitted edits, unsynced commits, installed tools) is lost with it.
- **Sync after every commit.** If you work in a cloud clone, bring each commit into the user's repo right away (git bundle → fast-forward the user's branch). Never let more than the current step exist only in the sandbox.
- **Keep `docs/HANDOFF.md` resumable.** At every task boundary (and before anything long-running) add or refresh a short "in progress" note in the newest entry: what is done, what is next, what is half-done and where.
- **When the user says they are about to stop, checkpoint first:** finish or cleanly back out of the current step, commit, update HANDOFF, sync, and confirm before they shut down.
- **Git on the user's machine:** get delete permission for the repo folder before running git there (git must be able to remove its own lock files), and never leave a `.git/*.lock` behind. Do not start a git operation on the user's machine right before an announced stop.
- **When resuming:** read HANDOFF, then compare the user's repo (branch, last commit, working tree) with any sandbox copy. **The user's repo wins**: the user may have amended or added commits in the meantime, so rebuild your working copy from it before continuing. Re-create sandbox tooling as HANDOFF describes.

## End-of-session checklist

1. `cargo xtask verify` is green (or the failure is documented in HANDOFF with cause and next step).
2. Update `docs/HANDOFF.md`: what you did, what is next, decisions taken, traps, anything half-done.
3. Update the epic file / `docs/BACKLOG.md` status; add newly discovered work as coarse backlog items.
4. New questions → `docs/OPEN-QUESTIONS.md`; new decisions → ADR.
5. Commit.

## Map of the docs

`docs/README.md` indexes everything. Key files: vision (`01`), architecture (`02`), domain (`03`), rules & sources (`04`), testing (`05`), security (`06`), simulation & AI (`07`), work process (`08`), plus `BACKLOG.md`, `HANDOFF.md`, `OPEN-QUESTIONS.md`, `epics/`, `adr/`, `rules/`.
