# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E17 M0–M2 done; M3 T1a–T7a-2b, T7b-1, T7b-2a, T7b-2b-1, T8 done; H1, H2 done (2026-10-19: the adventure lane's harness); next: M3 T7b-2b-2, T7b-2b-3, T7c** (`QUEUE.md`; T9 closes M3). Screens from the executable's dialog items (`h3_view::ui`; design §9.7). Workspace `clients/` (what is built: `clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§9; records in `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture only what it lists or the task changes, all at a milestone's closure or after a harness change (runbook, "Which captures a change needs"). Detail for later tasks: `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs.
- The lane owns `clients/`, `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (the autonomy contract below), a fresh reviewer agent first.
- Stage first (the user's computer online) from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the four end tunes; for captures `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`; `cp -rn` the staged `original-data/.` into the clone before `capture/setup.sh`. Frames stay in their sandbox (`kept/` keeps numbers).
- **Open:** CLT-Q4 (the background variant); CLT-Q5 (boats in `h3-map`'s writer); CLT-Q6 (player colours, a hero's sex).

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

- **CI may be out** (billing, since 2026-10-12): `clients/scripts/check.sh` stands in, its result in the pull request (needs `cargo-deny`, `cargo-llvm-cov`, the LLVM tools `scripts/llvm-tools.sh` picks).
- Coverage reads cargo-llvm-cov's summary: generic code can show false region misses (take a concrete type, T4b-1; the engine reads the merged view).
- A new dependency of a pure crate goes into `h3-view/tests/purity.rs`'s allowlist deliberately; a window or audio crate belongs in an adapter crate, not in `h3-view`'s pure library.
- Sound in a sandbox: `SDL_AUDIO_DRIVER=disk` (the mix in `./sdlaudio.raw`, S16 stereo, 44.1 kHz).
- `winit`, `cpal`, `hound` (and Bevy, eframe, `rodio`) are Apache-2.0-only (`deny.toml`, ADR-0009): the window is SDL 3 (`h3-sdl`; building it: CMake, a C compiler, `clients.yml`'s apt list). Windows exes: target `x86_64-pc-windows-gnu`, `mingw-w64`; the user's `clients/out/win/`.
- `clients/Cargo.lock` is a common path: the verify's `clients-lock` gate (#202) fails when an engine crate's dependencies change it; fix: `cargo update --workspace --manifest-path clients/Cargo.toml`.
- `ci.yml` ignores `clients/**`; `clients.yml` watches `clients/` and the engine crates the client builds (`h3-protocol` builds `h3-battle`, `h3-content`, `h3-contracts`, `h3-kernel`: they can break it), which `check.sh` keeps complete ("ci paths").
- Never run the original game from the installation: it writes saves, settings and DDrawCompat's log into its folder. Captures run the copy under `original-data/wine/` (the runbook's "Traps"); a Wine prefix sits in a directory the shell's user owns (`/root/...` in a sandbox). Captures never enter git (ADR-0009).
- The original's memory holds the frame drawn, the screen the one shown (design §7.1, §7.9): never render memory's poses as the screen. More in the runbook (`Combat Speed`, acting as a turn starts, both heroes named, steps ≤ 100 ms, `hot_seat`, the adventure harness's settings).
- No float arithmetic or platform transcendental in the client's drawing and timing, the binary included (`clippy.toml`): f32 behaviour in fixed point (`anim::f32_round`), tangents by series; SDL's floats only at the adapter's edge (coordinates, `audio::gain`).
- The engine rolls its own damage: the frame check lists a box (` n…`, T5b-1) or the log's item
  (` log`, T6b) whose number or text is not the original's.
- Timelines' records: army, then war machines (design §7.15).
- The harness's `actions` are the viewer's mouse (`pointer::inputs`, design §9.10): a new action
  kind needs both; `hex_point` and `PARK` mirror `h3drive.py`'s; a `key:` right after a `cast:`
  acts at the cast's end in both (T7b-1).
- The original's rolls: forced (`roll:<name>`, `fail`; runbook); the engine's by a sequence's
  `seed` (`resist`: 3). The frame check takes the heroes from memory but cast frames.
- `turn:` naming the active stack passes at once: after its hero's cast a stack acts by `key:`
  only (pick speeds so turns alternate, T7b-2a). A spell's handler: `Player::cast` by
  `SpellLook::original`, its effects at the handler's point (the box's tint follows them).

## Recent sessions

- **2026-10-21, E17 M3 T7b-2b-1 (the harness forces the original's rolls).** Split with the user
  (T7b-2b-2 morale, luck, fear; T7b-2b-3 abilities); nine roll calls redirected in `xdd.dll`.
- **2026-10-21, E17 M3 T7b-2a (tint, Haste/Slow walks, Dispel, Cure, Ray; #363, merged).**
  Split with the user; `haste`, `dispel` recorded, frames exact.
- **2026-10-21, E17 M3 T7b-1 (a hero's cast on one stack; #361, merged).** `cast`, `resist`,
  `arrow` recorded, frames exact.
