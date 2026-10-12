# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E17 refined (2026-10-12).** Decisions: the epic file `docs/epics/E17-clients.md` ("Decisions", user) and the ADR draft `adr-drafts/client.md` (Proposed; accepted at M0's checkpoint). Milestones M0 (the workspace) and M1 (assets and a battle still, headless) are in `QUEUE.md`; M2 (the replay viewer, the window/audio evaluation) and M3 (fidelity) are sketched in the epic file. **Next: M0 T1 or M1 T1** (both ready; M1 T1 is a code reading, independent of M0).
- The lane owns `clients/` (its own Cargo workspace, `clients/README.md`), `docs/epics/E17*` and this folder. Branches `client/<task>`. Ids: questions `CLT-Q<n>`, quirks (asset fixes) `CLT-QUIRK-<n>`, stubs `CLT-STUB-<n>`. ADR drafts go to `adr-drafts/` and get a number when the user accepts them.
- **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- The lane needs the user's computer online: the original files are staged from `original-data/install/`, which since 2026-10-12 also holds `Data/{Heroes3,H3ab_ahd}.snd`, `Data/{VIDEO,H3ab_ahd}.vid` and `MP3/` (57 tracks), copied from the installation and verified identical (`docs/formats/installation.md` §2, battle-owned, still lists only the earlier copies).
- **Open:** CLT-Q1 (the walk's path, needed by M2).

## Autonomy contract (agreed by the user, 2026-10-12)

- **Agent decides, marked provisional:** the inner structure of the client crates, names, test layout, CLI flags, caching, debug output; asset fixes whose source is a code reading or a reference frame (each an entry with its id). Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency (`png`, an MP3 decoder), the asset tables' and layouts' formats, a visual deviation from the original without a reference. List them in the epic's review list and go on.
- **Stop the line:** a change to an engine crate or another lane's paths (CLT-Q1 is one), shared paths (CI, `lanes.toml`: a `shared/` PR), heavy dependencies (the window/audio library, a GPU stack, a UI framework, a video codec), licence questions (reading MMArchive's, homm3-unpacker's or DDrawCompat's code included), committing anything taken from the original beyond identifiers and numbers (ADR-0009), anything that would write into the installation. Write the question (`CLT-Q<n>`), stop that task and take the next independent one.

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

- `winit`, `cpal` and `hound` are Apache-2.0-only, which `deny.toml` rejects (ADR-0009 point 3); so are, through them, Bevy, eframe and `rodio`. Not before the M2 evaluation and the user's licence decision.
- `ci.yml` ignores `clients/**` (a push that changes only `clients/` starts no engine run): the client has its own job (M0 T2).
- `h3-protocol` depends on `h3-battle`, `h3-content`, `h3-contracts` and `h3-kernel`: a path dependency on it builds those too, and their changes can break the client.
- Never run the original game from the installation: it writes saves, settings and DDrawCompat's log into its folder. Reference captures use a full copy under `original-data/` (M3).

## Recent sessions

- **2026-10-12, E17a + E17 refinement (the user).** The asset notes dictated and recorded (above). Missing original files copied into `original-data/install/`. Refinement: Rust confirmed; a software compositor; the window/audio library chosen at M2 by an evaluation, GPLv3 for the client allowed if the winner needs it (the user decides then); asset tables per expansion, committed, no executable at run time; reference captures from a copy of the game through computer use; first milestone M0 + M1. ADR draft, epic file, queue, CLT-Q1 written.
