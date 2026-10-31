# Documentation index

Working project name: **h3** (provisional; see OPEN-QUESTIONS Q2). Last structural update: 2026-10-24 (housekeeping: short start-of-session files, archives next to the files they were moved from, per-lane traps; `08-work-process.md`, "Keeping the start-of-session reading small"). Code layout and quick start: [../README.md](../README.md).

## How to read these docs

| Doc | Purpose |
|---|---|
| [01-vision.md](01-vision.md) | Goals, non-goals, success criteria, decisions already taken |
| [02-architecture.md](02-architecture.md) | Principles, layers, crates, state/command/event model, performance, extension model |
| [03-domain.md](03-domain.md) | Bounded contexts, seams, ubiquitous language |
| [04-rules-and-sources.md](04-rules-and-sources.md) | Source of truth, provenance, rule catalog, quirks and profiles |
| [05-testing.md](05-testing.md) | Test strategy, gates, and the mechanisms that keep agents from skipping tests |
| [06-security.md](06-security.md) | Threat model, trust boundaries, information hiding, anti-exploit design |
| [07-simulation-and-ai.md](07-simulation-and-ai.md) | Simulation environment, baselines, RL strategy, training automation |
| [08-work-process.md](08-work-process.md) | How agents work in increments, handoff protocol, when to ask |
| [BACKLOG.md](BACKLOG.md) | Coarse-grained epics, ordered; only the next ones are refined |
| [HANDOFF.md](HANDOFF.md) | The lane index, the traps that cross lanes and where the other traps live (size budget, `08-work-process.md`) |
| [lanes/](lanes/) | One folder per work lane (battle, adventure, town, ai, server, client, map-client): `HANDOFF.md` (state, next step, the last three sessions), `TRAPS.md` (standing traps by area), `QUEUE.md` (open tasks and inbox) with `QUEUE-ARCHIVE.md`, `QUESTIONS-ANSWERED.md`, ADR drafts; adventure also `OVERVIEW.md` (what is built), client `ASSET-NOTES.md`; ownership in `../lanes.toml` |
| [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md) | Decisions waiting for the user (answered ones: the lane's `QUESTIONS-ANSWERED.md`, `adr/answered-questions.md`) |
| [epics/](epics/) | Refined epics (only those about to be worked on), their design notes (`<epic>-design.md`, pruned as tasks implement them and deleted when the epic closes) and logs (`<epic>-log.md`, the full record of done tasks) and archives (`<epic>-archive.md`, closed increments and milestones) |
| [adr/](adr/) | Architecture Decision Records (binding) |
| [pull_request_template.md](pull_request_template.md) | Definition-of-Done checklist for changes |
| [rules/](rules/) | Rule catalog (machine-checked), quirks ledger, [source surveys](rules/surveys/) per epic |
| [formats/](formats/) | Observed layouts of the original installation's files (catalogue), the map format we write |
| [exe/](exe/) | The address map of `Heroes3.exe` for code reading: our readings, record layouts, H3API's names (guesses); the tool `scripts/exe.py` |
| [experiments/](experiments/) | Staged battles in the original game: flow, maps, results |

## Planning philosophy

Decide upfront only what is expensive to change later: determinism, state/command model, context boundaries, security model, the compiled-ruleset approach, licensing/IP stance, and the provenance/testing machinery. Everything else is deferred and refined just in time. The first increment is a **walking skeleton** (a minimal battle end to end, including command validation, observation, determinism replay, benchmark) so those upfront decisions get tested early.

## Status of upfront decisions

| Topic | Status | Where |
|---|---|---|
| Rust core, Cargo workspace, no unsafe in logic | Accepted | ADR-0002 |
| Determinism & numeric model | Accepted | ADR-0003 |
| Server-authoritative state/command/event model | Accepted | ADR-0004 |
| Bounded contexts & enforced boundaries | Accepted | ADR-0005 |
| Config tiers 1–3, compiled ruleset | Accepted | ADR-0006 |
| Provenance, quirks ledger, profiles | Accepted | ADR-0007 |
| Security model | Accepted | ADR-0008 |
| License & reference-material policy (GPL-2.0-or-later) | Accepted | ADR-0009 |
| Testing gates & rule traceability | Accepted | ADR-0010 |
| Portability & C ABI | Accepted | ADR-0011 |
| Modifiers & abilities | Accepted | ADR-0012 |
| Heroes in battle (entities, `BattleHero` seam) | Accepted | ADR-0013 |
