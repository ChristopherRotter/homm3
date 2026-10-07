# Client lane — HANDOFF

The client lane's living state (`docs/08-work-process.md`, "Lanes"). Read after the global `docs/HANDOFF.md`. Rewrite **Current state** each session, keep **Standing traps** to client traps that still apply, add one short entry to **Recent sessions** and keep at most three (`cargo xtask check-docs`: 10 000 bytes, three entries). Tasks waiting: `QUEUE.md`.

## Current state

- **Not started.** The lane owns `clients/`: a separate Cargo workspace with its own lockfile, excluded from the engine workspace (`exclude = ["clients"]` in the root `Cargo.toml`), with its own CI job, added by the pull request that creates the workspace (`.github/` is shared). Layout and asset policy: `clients/README.md`.
- **Next:** the asset-notes session (the user dictates what they know about the assets and their traps, recorded below), then the client refinement (`QUEUE.md`). Branches: `client/<task>`.
- **Timing:** implementation waits until the battle actions are mostly stable (E04b). Open option, the user decides: the asset and render-to-PNG layer could start earlier, since it does not depend on the battle.
- The lane needs the user's computer online: the sprite archives are read from the user's installation.
- **Checkpoints** per milestone, under an autonomy contract agreed in the refinement (the three tiers as in the server lane's draft); a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `CLT-Q<n>`, quirks `CLT-QUIRK-<n>`, stubs `CLT-STUB-<n>`. ADR drafts go to `docs/lanes/client/adr-drafts/` and get a number when the user accepts them.

## Agreed (user, 2026-10-06/07)

- Rust by default, unless the client refinement finds good arguments against it; the client ADR chooses the renderer.
- The walking skeleton uses the real assets from the start, no placeholder graphics: a battle replay viewer first, later a full game record/replay.
- Assets are read from the user's installation at run time and never reach git or CI (ADR-0009). Asset decoding (`clients/crates/h3-assets`: DEF, PCX, palettes, fonts, sound) lives in the client workspace and uses `crates/h3-formats` for LOD (a battle-owned crate until a formats/tools lane opens); the engine core stays free of graphics.
- Decoders are tested on small synthetic sprites, plus a local real-asset gate.
- A headless render-to-PNG mode lets agents check visuals themselves; screenshots stay local and gitignored.

## Asset notes

*Placeholder for the user's asset knowledge and traps, dictated in the asset-notes session. First input to the client refinement.*

## Standing traps

(None yet; the global HANDOFF's traps apply.)

## Recent sessions

(None yet.)
