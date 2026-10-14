# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E17 M0, M1, M2 done (M2 closed 2026-10-14, T5c); next: M3 T1a** (`QUEUE.md`: T1a–T9, refined with the user). Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020, ADR-0024. Built: decoders (images, sounds), the real-asset gate, compositor, asset tables, `h3-view render`/`sequences`/`situation`/`play`/`window`, `h3_view::anim`, the platform adapter `h3-sdl`, the capture harness (`clients/capture/README.md`); readings and evaluations in design §3–§8. Records: `docs/epics/E17-log.md`.
- **Pull requests (2026-10-14):** T5c (`client/e17-m2-t5c-closure`) open; #202–#204 merged.
- **The user's requirements (2026-10-13)**, epic "Decisions": a string catalogue, language packs; graphics options (`faithful` the default and the reference); automatic, exhaustive reference diffs.
- The lane owns `clients/`, `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- The lane needs the user's computer online (it may sleep: stage all a task needs first) for the original files, from `original-data/install/` into the clone's (`sandbox-setup.sh` copies only some): the four LODs (one per staging call) for the gate (also `h3maped.exe`, `Heroes3.snd`, `H3ab_ahd.snd`), `render`, `play` and `window` (its sound: the two `.snd`, `MP3/COMBAT01`–`04`); the captures also `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`.
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

- **CI may be out** (billing, since 2026-10-12; `docs/05-testing.md`): run `clients/scripts/check.sh`, its result in the pull request. It needs `cargo-deny`, `cargo-llvm-cov` and the LLVM tools (a sandbox: `LLVM_COV`, `LLVM_PROFDATA` as `scripts/sandbox-verify.sh` sets them).
- The coverage gate reads cargo-llvm-cov's summary; generic code can show false region misses there (the engine reads the merged view, `xtask/src/coverage.rs`): port that before blaming the tests.
- A new dependency of a pure crate goes into `h3-view/tests/purity.rs`'s allowlist deliberately; a window or audio crate belongs in an adapter crate, not in `h3-view`'s pure library.
- Sound in a sandbox: `SDL_AUDIO_DRIVER=disk` writes the mix to `./sdlaudio.raw` (S16 stereo 44.1 kHz, real time); else no device, no sound.
- `winit`, `cpal`, `hound` (and Bevy, eframe, `rodio`) are Apache-2.0-only, which `deny.toml` rejects (ADR-0009); the window is SDL 3 (`h3-sdl`). Building `h3-view` builds SDL from source: CMake, a C compiler, the X11 headers (`clients.yml`'s list; a fresh sandbox lacks Xcursor's). Windows exes: `rustup target add x86_64-pc-windows-gnu`, `mingw-w64`, `--target x86_64-pc-windows-gnu`; put into the user's `clients/out/win/` (`device_commit_files`) they find the checkout from their place.
- `clients/Cargo.lock` is a common path: the verify's `clients-lock` gate (#202) fails when an engine crate's dependencies change it; fix: `cargo update --workspace --manifest-path clients/Cargo.toml`.
- `ci.yml` ignores `clients/**`; the client's job `clients.yml` watches `clients/` and the engine crates the client builds, which `check.sh` keeps complete ("ci paths").
- `h3-protocol` builds `h3-battle`, `h3-content`, `h3-contracts`, `h3-kernel`: their changes can break the client.
- Never run the original game from the installation: it writes saves, settings and DDrawCompat's log into its folder. Captures run the copy under `original-data/wine/` (`clients/capture/README.md`, its "Traps": the game's own cursor, the panel by turn, XWD colour maps); a Wine prefix must sit in a directory the shell's user owns (`/root/...` in a sandbox). Captures never enter git (ADR-0009).
- Reading the original's state (`capture/h3mem.py`): memory holds the frame drawn, the screen the one shown, and lags it by a step after an animation (design §7.1, §7.9; runbook "Traps"): never render memory's poses as the screen. `Combat Speed` (the prefix's registry, 0): reset it after a run at another.
- No float arithmetic or platform transcendental in the client's drawing and timing, the binary included (`clippy.toml`): f32 behaviour in fixed point (`anim::f32_round`), tangents by series; SDL's floats only at the adapter's edge (coordinates, `audio::gain`).
- The adventure map's harness (`h3adv.py`): `Show Route` on (`setup.sh`), heroes selected from the hero list only, real time (no heavy build beside a batch): the runbook's "Traps".

## Recent sessions

- **2026-10-14, E17 M2 T5c (the user present).** M2 closed: a fresh reviewer's 12 findings checked (10 fixed; design §7.3–§8.2), the review list accepted, M3 refined (rays, spell animations: the user). The user heard T5b's sound clean, in sync, balanced.
- **2026-10-14, E17 M2 T5b (the user present).** The decoder (1 232 entries equal SDL's), the battle's sounds read, cues, the tables' `sounds`, the window's audio; shared #202 (`clients-lock`). Design §8.2.
- **2026-10-14, E17 M2 T5a (the user present).** T5 split; `h3-sdl`, `h3-view window`: frames read back exact under Xvfb; a Windows build for the user (`clients/out/win/`). Design §8.1.
