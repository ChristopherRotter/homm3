# ADR-0001: Record architecture decisions

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

The project is built incrementally by many agent sessions with no shared memory beyond the repo. Upfront decisions must be few, explicit and findable; later decisions must be recorded as they are made.

## Decision

Use lightweight ADRs in `docs/adr/` (template `0000-template.md`). Architecture-level decisions, seam/contract changes and significant dependency choices require an ADR. Accepted ADRs are binding; changing one means writing a superseding ADR. `AGENTS.md` makes reading relevant ADRs part of the session start.

## Consequences

Agents can see why things are the way they are and cannot silently deviate. The cost is a few minutes of writing per decision.

## Alternatives considered

Decisions scattered in commit messages or chat (lost between sessions); a single growing design doc (hard to supersede individual decisions).
