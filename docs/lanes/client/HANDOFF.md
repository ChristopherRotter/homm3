# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E17 M0–M2 done; M3 T1a (2026-10-14) and T1c (2026-10-15) done; in progress: M3 T1b-1, shots and rays** (2026-10-15, branch `client/e17-m3-t1b1-shots`; T1b re-cut into T1b-1 and T1b-2 with the user; `QUEUE.md`; T2–T8 also ready). Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ the amendment draft in `adr-drafts/`), ADR-0024. Built: decoders, the real-asset gate, compositor, asset tables, `h3-view` (`render`, `sequences`, `situation`, `play`, `window`), `h3_view::anim`, `h3-sdl`, the capture harness (`clients/capture/README.md`); design §3–§8; records in `E17-log.md`.
- **Pull requests (2026-10-15):** none open (T1c #216 and the shared #219 merged).
- **The user's requirements (2026-10-13)**, epic "Decisions": a string catalogue, language packs; graphics options (`faithful` the default and the reference); automatic, exhaustive reference diffs.
- The lane owns `clients/`, `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- The lane needs the user's computer online (it may sleep: stage all first) for the original files, from `original-data/install/` into the clone's: the four LODs (one per staging call), `h3maped.exe`, the two `.snd` (sound: also `MP3/COMBAT01`–`04`); the captures also `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`. Captured frames live only in their sandbox: a new one captures again (runbook).
- **Open:** CLT-Q2 (a table-only LOD reader in `h3-formats`; until then whole archives); CLT-Q4 (how a record carries the background variant).

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

- **CI may be out** (billing, since 2026-10-12): `clients/scripts/check.sh` stands in, its result in the pull request (needs `cargo-deny`, `cargo-llvm-cov`, the LLVM tools: in a sandbox `LLVM_COV`, `LLVM_PROFDATA`).
- The coverage gate reads cargo-llvm-cov's summary; generic code can show false region misses there (the engine reads the merged view, `xtask/src/coverage.rs`): port that before blaming the tests.
- A new dependency of a pure crate goes into `h3-view/tests/purity.rs`'s allowlist deliberately; a window or audio crate belongs in an adapter crate, not in `h3-view`'s pure library.
- Sound in a sandbox: `SDL_AUDIO_DRIVER=disk` (the mix in `./sdlaudio.raw`, S16 stereo 44.1 kHz, real time).
- `winit`, `cpal`, `hound` (and Bevy, eframe, `rodio`) are Apache-2.0-only, which `deny.toml` rejects (ADR-0009); the window is SDL 3 (`h3-sdl`). Building `h3-view` builds SDL: CMake, a C compiler, `clients.yml`'s apt list. Windows exes: `rustup target add x86_64-pc-windows-gnu`, `mingw-w64`, `--target x86_64-pc-windows-gnu`; put into the user's `clients/out/win/` (`device_commit_files`) they find the checkout from their place.
- `clients/Cargo.lock` is a common path: the verify's `clients-lock` gate (#202) fails when an engine crate's dependencies change it; fix: `cargo update --workspace --manifest-path clients/Cargo.toml`.
- `ci.yml` ignores `clients/**`; the client's job `clients.yml` watches `clients/` and the engine crates the client builds, which `check.sh` keeps complete ("ci paths").
- `h3-protocol` builds `h3-battle`, `h3-content`, `h3-contracts`, `h3-kernel`: their changes can break the client.
- Never run the original game from the installation: it writes saves, settings and DDrawCompat's log into its folder. Captures run the copy under `original-data/wine/` (`clients/capture/README.md`, its "Traps": the game's own cursor, the panel by turn, XWD colour maps); a Wine prefix must sit in a directory the shell's user owns (`/root/...` in a sandbox). Captures never enter git (ADR-0009).
- Reading the original's state (`capture/h3mem.py`): memory holds the frame drawn, the screen the one shown, and lags it by a step after an animation (design §7.1, §7.9; runbook "Traps"): never render memory's poses as the screen. `Combat Speed` (the prefix's registry, 0): reset it after a run at another.
- No float arithmetic or platform transcendental in the client's drawing and timing, the binary included (`clippy.toml`): f32 behaviour in fixed point (`anim::f32_round`), tangents by series; SDL's floats only at the adapter's edge (coordinates, `audio::gain`).
- A battle in progress: act as the stack's turn starts (`turn:`, runbook): a click before is lost, one after lets fidgets in.
- The adventure map's harness (`h3adv.py`): `Show Route` on (`setup.sh`), heroes selected from the hero list only, real time (no heavy build beside a batch): the runbook's "Traps".

## Recent sessions

- **2026-10-15, E17 M3 T1c (the user present).** T1a's frames were lost with its sandbox: the six sequences captured again (counts as recorded). `h3cap.redraws`: ours with 1–4 more shadows, or an earlier moment's with ≥ 1; frame files `<ms> <differing> <artifacts>`; `behind` all artifacts, `melee` 3 966's stale count box listed for T5. The user asked for a leaner verify: `shared/verify-once`.
- **2026-10-14, E17 M3 T1a (the user present).** A battle in progress in hot seat; `climb`, `behind` recorded (timelines as recorded, frames 43/43, 164/194); the walk-by read: dirty rectangles, not copied (the user), T1c classifies.
- **2026-10-14, E17 M2 T5c (the user present).** M2 closed: the reviewer's findings (10 fixed), the review list accepted, M3 refined.
