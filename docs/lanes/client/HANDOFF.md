# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

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

*Dictated by the user in the asset-notes session (E17a, 2026-10-12) and recorded as given; the refinement judges them. The research items at the end are the agent's reading of the notes.*

- **Archives and names:** asset names mostly follow patterns, but not reliably. Duplicates and dead (unused) entries exist; no corrupted entries are known. Which archive wins when a name is in both a SoD and an AB archive is not known: the exe should tell.
- **Formats:** fan projects solved palettes, PCX and DEF. MMArchive seems to work flawlessly (whether it is open source is unknown); `github.com/neunato/homm3-unpacker` worked mostly well for the user. MMArchive also handles the sound effects; the music (MP3) is easy. No community tool is known to handle the video archives well: needs investigation, low priority.
- **Fonts:** the exe tells which font is used where. The user's past font troubles (their own C# MonoGame engine) came from that engine's limits and from not knowing the format.
- **Animation and draw order:** the animation groups and timing come from VCMI and the exe, cross-validated. Draw order is not straightforward; the exe tells. In the first iteration, animation speed, frame rate and timings match the original exactly.
- **Resolution:** the native layout and resolution first; plan ahead for HD.
- **Known asset issues:** no conclusive list; the HD patch's notes or content may tell more. Most common: misplaced content (textures or rendered fonts one pixel off), less often clipping. Some textures have minor bugs (miscoloured pixels) that need an ad-hoc fix mechanism applied when loading. Unexplained: some extracted textures render with a few pixels slightly off-colour against the game; unknown whether the game applies a filter or the user's tooling was wrong. Shadows, above all those of adventure map objects, always looked off in the user's engine: they converted the keyed colours (purple hues) to alpha transparency, which may not be the right approach; needs investigation.
- **Wishes:** an agent that observes or drives the original game (a custom MCP or similar), feasibility unknown. Screenshots of a comprehensive, representative set of situations in the original, diffed exactly against our output, so that changes such as offset fixes are deliberate; feasibility to be evaluated. Loading as fast as the original: no long content rebuild, work with the native files as much as possible. Content configured per expansion (RoE, AB, SoD, …), aligned with the gameplay rules' configuration.

**Research items for the refinement:**
1. Archive precedence (SoD against AB) and name lookup, from the exe.
2. The off-colour pixels. Hypothesis: the original draws in 16-bit colour, so palette colours are quantised when blitted. Check in the exe; it decides what an exact screenshot diff compares.
3. The licences of MMArchive and homm3-unpacker before reading their code (licence questions stop the line).
4. The video archives' containers and codecs (low priority).
5. Capturing the original: computer use on the user's machine, or a hook in the game; the reference screenshot set; exact diff or tolerance.
6. Per-expansion content: how it maps to the engine's ruleset profiles.
7. The HD patch: its notes and content as a source for asset fixes, its resolution handling as a reference for HD.
8. Shadows and the other keyed colours: what the game does with each reserved palette index when it draws (hypothesis: it darkens the pixel already on screen by a fixed operation rather than blending with an alpha), and what happens where shadows overlap. Read in the exe; tied to item 2.

## Standing traps

(None yet; the global HANDOFF's traps apply.)

## Recent sessions

(None yet.)
