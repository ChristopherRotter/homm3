# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E17 M0, M1 done; M2 T1, T3, T4a, T6, T4b-1 done 2026-10-13; next: M2 T4b-2** (fidgets, turns, draw layers, a full battle frame by frame; `QUEUE.md`) or **T2** (the window/audio evaluation: ends at the user's choice). Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020. Built: decoders, the real-asset gate, code readings (design §3–§5, §7), compositor, asset tables, `h3-view render`/`sequences`/`situation`, `h3_view::anim` (§7.8, §7.9), the capture harness (runbook `clients/capture/README.md`, design §6: the original under Wine on a virtual clock, its memory read; the still set, the recorded sequences, the frame check `h3drive.py frames`, the adventure map's batches `h3adv.py batch`). Records: `docs/epics/E17-log.md`.
- **Pull request (2026-10-13):** T4b-1, `client/e17-m2-t4b1-frame-check`.
- **The user's requirements (2026-10-13)**, epic "Decisions": a string catalogue, language packs; graphics options (`faithful` the default and the reference); automatic, exhaustive reference diffs.
- The lane owns `clients/` (its own Cargo workspace, `clients/README.md`), `docs/epics/E17*` and this folder. Branches `client/<task>`. Ids: questions `CLT-Q<n>`, quirks (asset fixes) `CLT-QUIRK-<n>`, stubs `CLT-STUB-<n>`. ADR drafts go to `adr-drafts/` and get a number when the user accepts them (ADR-0021 is the server's: the next free number).
- **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- The lane needs the user's computer online: stage the original files from `original-data/install/` into the clone's `original-data/install/` (`scripts/sandbox-setup.sh` does not copy them all). The gate and `render` need the four LODs (one per staging call; the gate also `h3maped.exe`), `h3-view situation` `H3bitmap.lod`; the captures also `Heroes3.exe`, the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid`.
- **Open:** CLT-Q2 (the table-only LOD reader in `h3-formats`, for the battle lane; until it lands the client reads whole archives); CLT-Q4 (how a record carries the background variant; the scene takes an empty surrounding's).

## Autonomy contract (agreed by the user, 2026-10-12)

- **Agent decides, marked provisional:** the inner structure of the client crates, names, test layout, CLI flags, caching, debug output; asset fixes whose source is a code reading or a reference frame (each an entry with its id). Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency (`png`, an MP3 decoder), the asset tables' and layouts' formats, a visual deviation from the original without a reference. List them in the epic's review list and go on.
- **Stop the line:** a change to an engine crate or another lane's paths (CLT-Q1 was one), shared paths (CI, `lanes.toml`: a `shared/` PR), heavy dependencies (the window/audio library, a GPU stack, a UI framework, a video codec), licence questions (reading MMArchive's, homm3-unpacker's or DDrawCompat's code included), committing anything taken from the original beyond identifiers and numbers (ADR-0009), anything that would write into the installation. Write the question (`CLT-Q<n>`), stop that task and take the next independent one.

## Asset notes

*Dictated by the user in the asset-notes session (E17a, 2026-10-12) and recorded as given; the refinement judges them. The research items at the end are the agent's reading of the notes.*

- **Archives and names:** asset names mostly follow patterns, but not reliably. Duplicates and dead (unused) entries exist; no corrupted entries are known. Which archive wins when a name is in both a SoD and an AB archive is not known: the exe should tell.
- **Formats:** fan projects solved palettes, PCX and DEF. MMArchive seems to work flawlessly (whether it is open source is unknown); `github.com/neunato/homm3-unpacker` worked mostly well for the user. MMArchive also handles the sound effects; the music (MP3) is easy. No community tool is known to handle the video archives well: needs investigation, low priority.
- **Fonts:** the exe tells which font is used where. The user's past font troubles (their own C# MonoGame engine) came from that engine's limits and from not knowing the format.
- **Animation and draw order:** the animation groups and timing come from VCMI and the exe, cross-validated. Draw order is not straightforward; the exe tells. In the first iteration, animation speed, frame rate and timings match the original exactly.
- **Resolution:** the native layout and resolution first; plan ahead for HD.
- **Known asset issues:** no conclusive list; the HD patch's notes or content may tell more. Most common: misplaced content (textures or rendered fonts one pixel off), less often clipping. Some textures have minor bugs (miscoloured pixels) that need an ad-hoc fix mechanism applied when loading. Unexplained: some extracted textures render with a few pixels slightly off-colour against the game; unknown whether the game applies a filter or the user's tooling was wrong. Shadows, above all those of adventure map objects, always looked off in the user's engine: they converted the keyed colours (purple hues) to alpha transparency, which may not be the right approach; needs investigation.
- **Wishes:** an agent that observes or drives the original game (a custom MCP or similar), feasibility unknown. Screenshots of a comprehensive, representative set of situations in the original, diffed exactly against our output, so that changes such as offset fixes are deliberate; feasibility to be evaluated. Loading as fast as the original: no long content rebuild, work with the native files as much as possible. Content configured per expansion (RoE, AB, SoD, …), aligned with the gameplay rules' configuration.

**Research items** (the agent's reading of the notes; all nine with their state in the epic file's table; 1, 2, 6 and 8 done): 3. the licences of MMArchive and homm3-unpacker before reading their code (licence questions stop the line); 4. the video archives (low priority); 5. capturing the original (M3); 7. the HD patch, its notes as a source of fixes, its resolution handling for HD; 9. the adventure map's object draw order (the user, 2026-10-12: surprisingly complicated, neither the user nor VCMI got it exactly right; a code reading with reference frames).

## Standing traps

- **CI may be out** (billing, since 2026-10-12; `docs/05-testing.md` "When CI cannot run"): run `clients/scripts/check.sh` and record its result in the pull request. It needs `cargo-deny`, `cargo-llvm-cov` and the LLVM tools (in a sandbox, `LLVM_COV=/usr/lib/llvm-20/bin/llvm-cov LLVM_PROFDATA=/usr/lib/llvm-20/bin/llvm-profdata`, as `scripts/sandbox-verify.sh` sets them).
- The coverage gate reads cargo-llvm-cov's summary; generic code can show false region misses there (the engine reads the merged view, `xtask/src/coverage.rs`): port that before blaming the tests.
- A new dependency of a pure crate goes into `h3-view/tests/purity.rs`'s allowlist deliberately; a window or audio crate belongs in an adapter crate, not in `h3-view`'s pure library.
- `winit`, `cpal` and `hound` are Apache-2.0-only, which `deny.toml` rejects (ADR-0009 point 3); so are, through them, Bevy, eframe and `rodio`. Not before the M2 evaluation and the user's licence decision.
- `ci.yml` ignores `clients/**`; the client's job `clients.yml` watches `clients/` and the engine crates the client builds, which `check.sh` keeps complete ("ci paths").
- `h3-protocol` depends on `h3-battle`, `h3-content`, `h3-contracts` and `h3-kernel`: a path dependency on it builds those too, and their changes can break the client.
- Never run the original game from the installation: it writes saves, settings and DDrawCompat's log into its folder. Captures run the copy under `original-data/wine/` (`clients/capture/README.md`, its "Traps": the game's own cursor, the panel by turn, XWD colour maps); a Wine prefix must sit in a directory the shell's user owns (`/root/...` in a sandbox). Captures never enter git (ADR-0009).
- Reading the original's state (`capture/h3mem.py`): memory holds the frame drawn, the screen the one shown, and lags it by a step after an animation (design §7.1, §7.9; runbook "Traps"): never render memory's poses as the screen. `Combat Speed` (the prefix's registry, 0): reset it after a run at another.
- No float arithmetic or platform transcendental in the client, the binary included (`clippy.toml`): f32 behaviour in fixed point (`anim::f32_round`), tangents by series.
- The adventure map's harness (`h3adv.py`): `Show Route` on (`setup.sh`), heroes selected from the hero list only, real time (no heavy build beside a batch): the runbook's "Traps".

## Recent sessions

- **2026-10-13, E17 M2 T4b-1 (the user present).** T4b split; the user chose the frame check: every distinct battlefield of the three recorded sequences against the viewer's frame (`h3drive.py frames`). Walk and shot exact, melee 68 of 75 (the walk-by listed, design §7.9). Built the projectile, idle steps and outlines; corrected the shooter's strike loop and the flight's f32 points (read).
- **2026-10-13, E17 M2 T6 (the user present).** The inbox's high-priority request: `h3adv.py` plays experiment batches on the adventure map; E08 batch 2 as the user's results.
- **2026-10-13, CLT-Q1 and M2 T4a (the user present).** CLT-Q1 built cross-lane (#169): a walk's `Moved` hex by hex, wire v29. T4a: `h3_view::anim`; `h3-view sequences` gives all three recorded timelines exactly.
