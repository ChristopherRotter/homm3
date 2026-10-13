# Client lane — HANDOFF

The client lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E17 M0 done, M1 under way (2026-10-12).** The workspace `clients/` (`h3-assets`, `h3-render`, `h3-view`; pure `no_std` libraries, adapters in `h3-view`'s binary), its gates `clients/scripts/check.sh` (the full local check; the CI job `clients.yml` runs the same); ADR-0020 accepted (point 7: loading at checkpoints). M1 T1–T6 done: the decoders (`h3_assets`; `h3_assets::build` makes synthetic assets), the real-asset gate `cargo run --release -p h3-view -- gate` (from `clients/`; the inventory in `clients/out/gate/`), the code readings (design §3, §4), the compositor `h3-render` and `h3-view sprite`, the asset tables `clients/assets/` (`h3-view harvest`, the loader `h3_view::tables`, design §5). Records: `docs/epics/E17-log.md`; findings: `docs/epics/E17-design.md`. **Next: M1 T7** (a battle still), then T8.
- **Pull requests waiting (2026-10-13):** `shared/obstacle-type` (#147, `cross-lane`: the natural obstacles' types from the generator to the wire, CLT-Q3, the user's choice before T7) and this lane's `client/e17-obstacle-types-and-requirements` (docs). T7 starts once the first is merged.
- **The user's requirements (2026-10-13)**, for the refinements (epic, "Decisions"): every text through a string catalogue, language packs swappable; graphics options (shadows and the like the original's way or a modern one), `faithful` the default and the reference.
- The lane owns `clients/` (its own Cargo workspace, `clients/README.md`), `docs/epics/E17*` and this folder. Branches `client/<task>`. Ids: questions `CLT-Q<n>`, quirks (asset fixes) `CLT-QUIRK-<n>`, stubs `CLT-STUB-<n>`. ADR drafts go to `adr-drafts/` and get a number when the user accepts them.
- **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- The lane needs the user's computer online: stage the original files from `original-data/install/` (since 2026-10-12 also the sound, video and `MP3/` copies). The gate needs `Data/H3sprite.lod` and `Data/H3ab_spr.lod` and reads `h3maped.exe` when present, which `scripts/sandbox-setup.sh` does not copy: stage them into the clone's `original-data/install/`.
- **Open:** the engine's two-hex position becomes the rear hex (the user, 2026-10-13; the battle lane's QUEUE inbox): until then T7 converts head to rear in one place. CLT-Q1 (the walk's path, needed by M2); CLT-Q2 (the table-only LOD reader in `h3-formats`, for the battle lane; until it lands the client reads whole archives).

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

**Research items** (the agent's reading of the notes; all nine with their state in the epic file's table; 1, 2, 6 and 8 done): 3. the licences of MMArchive and homm3-unpacker before reading their code (licence questions stop the line); 4. the video archives (low priority); 5. capturing the original (M3); 7. the HD patch, its notes as a source of fixes, its resolution handling for HD; 9. the adventure map's object draw order (the user, 2026-10-12: surprisingly complicated, neither the user nor VCMI got it exactly right; a code reading with reference frames).

## Standing traps

- **CI may be out** (billing, 2026-10-12; `docs/05-testing.md` "When CI cannot run"): run `clients/scripts/check.sh` and record its result in the pull request. It needs `cargo-deny`, `cargo-llvm-cov` and the LLVM tools (in a sandbox, `LLVM_COV=/usr/lib/llvm-20/bin/llvm-cov LLVM_PROFDATA=/usr/lib/llvm-20/bin/llvm-profdata`, as `scripts/sandbox-verify.sh` sets them).
- The coverage gate reads cargo-llvm-cov's summary; generic code can show false region misses there (the engine reads the merged view, `xtask/src/coverage.rs`): port that before blaming the tests.
- A new dependency of a pure crate goes into `h3-view/tests/purity.rs`'s allowlist deliberately; a window or audio crate belongs in an adapter crate, not in `h3-view`'s pure library.
- `winit`, `cpal` and `hound` are Apache-2.0-only, which `deny.toml` rejects (ADR-0009 point 3); so are, through them, Bevy, eframe and `rodio`. Not before the M2 evaluation and the user's licence decision.
- `ci.yml` ignores `clients/**`; the client's job `clients.yml` watches `clients/` and the engine crates the client builds, which `check.sh` keeps complete ("ci paths").
- `h3-protocol` depends on `h3-battle`, `h3-content`, `h3-contracts` and `h3-kernel`: a path dependency on it builds those too, and their changes can break the client.
- `check-exe-map` scans only `docs/` and `crates/`: an address the client's code uses needs its map entry and a citation in the design notes too.
- Never run the original game from the installation: it writes saves, settings and DDrawCompat's log into its folder. Reference captures use a full copy under `original-data/` (M3).

## Recent sessions

- **2026-10-13, CLT-Q3 (the user present).** #138, #139, #144 found merged. T7's still needs each obstacle's type, which the engine dropped at generation: the user chose to carry it to the wire first (`shared/obstacle-type`, `cross-lane`: the seam's `Obstacle.kind` by key, state hash v42, replay format 23, protocol v22); T7 waits for its merge. The user's two new requirements (string catalogue, graphics options) recorded in the epic.
- **2026-10-13, E17 M1 T5 + T4 (the user present).** #135, #136 found merged. T5 (design §4): a stack's canvas at (x − 196, y − 267), mirrored at x − 254; a two-hex stack anchored at its rear hex (the engine's tail); outline, count box, obstacle layers, draw order; VCMI differs in one obstacle's y and the order inside a row. T4: the tables (toml 1.1 without std, review list), every pack key covered but water and rock.
- **2026-10-12, E17 M1 T1 + T6 (the user present).** T1 (design §3): the base archives first; palettes truncated to the 16-bit screen; shadows as shifts with masks, compounding; runs ≥ 8 skipped. T6: `h3-render` follows it; `Frame::runs`; `h3-view sprite`.
