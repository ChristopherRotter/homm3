# Adventure lane — HANDOFF

The adventure lane's living state (`docs/08-work-process.md`, "Lanes"). Read after the global `docs/HANDOFF.md`. Rewrite **Current state** each session, keep **Standing traps** to adventure traps that still apply, add one short entry to **Recent sessions** and keep at most three (`cargo xtask check-docs`: 10 000 bytes, three entries). Tasks waiting: `QUEUE.md`.

## Current state

- **Not started.** The lane owns (`lanes.toml`) the new `h3-adventure` crate, later the town, the ADV rules (`docs/rules/catalog/adventure.toml`, `town.toml`), its own quirks ledger (`docs/rules/quirks-adventure.md`), its benchmark workload (`benches/adventure/`) and the epic files E07–E10 and E12–E14. Creating the crate touches `Cargo.toml` and `arch.toml`: the crate skeleton is its own `shared/` pull request, merged first.
- **Next:** the refinement session with the user (brief below); it fills `QUEUE.md` with the first tasks. Branches: `adventure/<task>`.
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>`, stubs `ADV-STUB-<n>`. ADR drafts go to `docs/lanes/adventure/adr-drafts/` and get a number when the user accepts them.
- **Working agreement:** increments of a couple of tasks with a user checkpoint, as in the battle lane; the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md).

## Brief for the refinement session (E07/E08)

Adventure moved ahead of E05/E06 in the backlog (Phase 0). Prepare each point with options and a recommendation; the user decides.

1. **Map reader or synthetic maps first.** E07 reads `.h3m` maps; a writer exists (`h3_formats::h3m`, for experiment maps; `docs/formats/h3m.md`), and E08 needs maps to run on. Options: (a) the reader first, so E08 runs on real maps from the start; (b) synthetic maps built in code or in a small TOML format first, the reader later; (c) both thin. The formats crates belong to battle until a formats/tools lane opens (`lanes.toml`), so a reader in `h3-formats` means opening that lane or a cross-lane pull request: part of the decision.
2. **The first adventure-map slice:** the smallest end-to-end loop (state, commands with validation, events, observation, a determinism replay, a benchmark workload), as E01 was for the battle. A candidate: tiles and terrain, one hero moving with movement points and pathing, the day advance, picking up resources, fog of war; no towns and no battles yet. The map-object representation (closed enum or data-driven behaviour primitives; `docs/03-domain.md`, "Open domain design questions") needs an ADR draft.
3. **The adventure ↔ battle seam:** `BattleSetup` and `BattleOutcome` exist (`h3-contracts`, `docs/03-domain.md`, "Seams"), and the battle runs standalone from them. What adventure builds before the seam (the hero's battle profile through `h3_heroes::derive_battle_hero`, armies, terrain, obstacles) and applies after it (batch 1b's findings on experience, Necromancy and captured war machines: `QUEUE.md`, "Inbox"). Seam changes are `shared/` pull requests and stop the line in both lanes.
4. **`h3-heroes` ownership:** battle owns it now; it moves to adventure with a handover checkpoint when hero progression starts (E08–E10: experience, levels, skill offers, the hero commands). Decide when, and what the battle lane finishes in it first (Spell Scroll instances, E04b T8).
5. **Questions for the user:** the rules of the first slices (RoE objects only, or the expansions' too); how adventure content enters the packs while `h3-content` and `h3-pack` stay with battle; the adventure benchmark workload; how much of the town (E09) the first slices need.

## Standing traps

(None yet; the global HANDOFF's traps apply.)

## Recent sessions

(None yet.)
