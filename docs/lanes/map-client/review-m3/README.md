# E24 M3's review (T7): the reviewers' brief and findings

The user (2026-10-30, at T7's cut): M3's review runs beside T7a's recordings, in up to three
concurrent reviewer sub-agents, each given a fraction small enough to stay under about half a
context. They leave their findings here, on the branch `map-client/e24-m3-t7-review`; the next
session consolidates them and fixes (T7b). Nothing here is merged as is.

## The fractions

| File | Area | Code (paths under `clients/`) | Design (`docs/epics/E24-design.md`) |
|---|---|---|---|
| `F1-panels.md` | the panels as drawn and over time | `h3-map-view/src/{panels,minimap,lists,status,record,layout}.rs`; `tables.rs`'s `[map.records]`, `[map.minimap]`, `[map.lists]`, `[map.status]` parts; `h3-ui/src/paint.rs`; the binary's `paint.rs`, `texts.rs`; `timeline.rs`'s `Minimap`, `Draw` and `spend` … `frame_drawn` | §4.1, §4.4, §4.5, §4.7, §4.8, §4.9 |
| `F2-redraw.md` | the frame's redraw | `h3-map-view/src/{compose,damage,draw}.rs`; `scene.rs`'s M3 parts (`TileIndex`, `ViewPlace`, `scene_in`); `h3-render/src/screen.rs`'s `Widening`, `cut`, `copy_rect`, `widen_rect`; `h3-sdl/src/lib.rs`'s `present_part`, `present_read_part`; the binary's `render.rs` and `window.rs`'s present path | §4.2, §4.3, §4.6 |
| `F3-hover.md` | the mouse-over: hints and cursor | `h3-map-view/src/hover.rs`, `hover/contents.rs`; `live.rs`'s `[hover]` part (visits, contents, costs); `timeline.rs`'s `Cursor` and `look_again` … `selection`; `h3-ui/src/cursor.rs`'s M3 part | §4.10, §4.11, §4.12 |
| `F4-input.md` | the clicks, keys, the walk's input, the sound without the focus | `timeline.rs`'s `Action` … `Unplayable`, `run` … `choose_town`, `edge` … `spend`; `h3-map-view/src/keys.rs`, `h3-ui/src/keys.rs`; `h3-ui/src/music.rs`'s focus; `h3-sdl/src/lib.rs`'s `Audio::hold`; the binary's `window.rs` (events, focus) and `audio.rs` (focus) | §4.13, §4.14, §4.15 |
| `F5-harness.md` | the harness's M3 additions | `capture/h3map.py`, `capture/h3mem.py` (M3's diff), `map/masks.toml`, `map/sequences.toml`'s keys; `h3-map-view/src/situation.rs`'s M3 parts; the binary's `sequences.rs` | the runbook `clients/capture/README.md`'s map sections |
| `F6-rest.md` | what F1 to F4 left (their "Not covered") | the binary's `window.rs` from line 720, `gate.rs`; `h3-ui/src/ui.rs`'s `Dialog::draws`; `h3-ui/src/paint.rs`'s `reach`; `h3-ui/src/cursor.rs`'s `Cursor::rect` against the cursor's drawer; `tables.rs`'s parts F1 left; `hover.rs`'s cases beyond the design's readings, spot-checked against the executable (the 42-class table 0x63AA58 among them) | §4.2 (the cursor's rectangle), §4.10 |

## Rules for a reviewer

- **The snapshot:** this worktree, `/home/claude/review`, is `master` at e456dd6 (T6b3 merged).
  Change no code here; write only your own findings file in this folder. No git commands: the
  runner commits and pushes.
- **What M3 is:** `docs/epics/E24-map-client.md`, the M3 section (lines 94–212) and "M3's review
  list" (from line 260): the items there are known, provisional choices queued for the user;
  report one only to argue it is wrong. M3's changes in a file: `git diff e8a7990a -- <path>`
  (e8a7990a is `master` just before M3 T1a), read in slices.
- **No builds, no tests, no cargo, no Wine:** the machine has two CPUs and the runner's captures
  of the original run beside you. Read and grep only. To check a code reading you doubt,
  `nice -n 19 python3 scripts/exe.py fn <addr> --around <addr> 40` or `lookup <name>` (at most
  about ten calls; `docs/exe/README.md`).
- **Reading discipline:** never read a design section, log or test file whole when a slice does;
  open tests only to check what one claims to cover.
- **What to look for,** most severe first: a bug (a panic, overflow, wrong rectangle, off by one,
  state left wrong); a divergence from the original as the design note's reading gives it; a
  crafted input file (map, TOML, live part, timeline) that crashes or hangs the viewer; a test
  that does not test what it says, or a branch no test reaches; a comment or doc that says
  something the code does not; a simplification worth having; a frame cost worth cutting. No
  style nits, nothing about naming unless it misleads.
- **Write as you go:** add each finding to your file once traced, so an interrupted run loses
  nothing. Stop reading at about half your context: write what you have and list in "Not
  covered" what is left.
- **Final reply** to the runner: at most 120 words: the counts by severity, your file's path,
  what was not covered.

## A finding's form

```
## F<n>-<k> <short title>

- **Severity:** bug | divergence | robustness | test gap | docs drift | simplification | performance
- **Verdict:** CONFIRMED (traced through the code) | PLAUSIBLE (not fully traced)
- **Where:** `path:line`
- **What:** one or two sentences.
- **Failure scenario:** concrete inputs or state → the wrong output or crash.
- **Fix:** one or two lines; whether it changes a drawn frame (then the captures it reaches are
  recorded again).
```

A file starts with `# F<n> — <area>: findings`, then `Covered:` and `Not covered:` lines, then the
findings, most severe first.

## Summary for T7b (the runner, 2026-10-30)

42 findings in six files, each reviewer fresh, run beside T7a's recordings (reported tokens: F1
243k, F2 227k, F3 242k, F4 215k, F5 210k, F6 202k):

| File | Bug | Divergence | Robustness | Test gap | Docs drift | Simplification | Performance |
|---|---|---|---|---|---|---|---|
| F1 panels | | | 2 | | 1 | 1 | 1 |
| F2 redraw | 1 | | 4 | 1 | 1 | | 1 |
| F3 hover | | 1 | 2 | | 3 | | |
| F4 input | 3 | 2 | 3 | | | 1 | |
| F5 harness | | | 7 | | 2 | | |
| F6 rest | | | 2 | 2 | 1 | | |

- **The bugs first:** F2-1 (a change of the buttons alone never presented), F4-1 (Ctrl during a
  held hotkey: the button stuck, its command lost), F4-2 (the focus lost during a minimap drag:
  the view jumps to the corner), F4-3 (a press taken from the queue after a key's step: a stale
  pointer); then the divergences F3-2, F4-4, F4-5. A fix that changes a drawn frame needs its
  captures again (the runbook, "Which captures a change needs").
- **Already done by T7a** (`map-client/e24-m3-t7a`): F5-9 (`h3map.py`'s mask comment, rewritten
  with the hint's mask gone).
- **Not covered by any reviewer:** `tables.rs` outside F1's parts; the hint cases beyond the
  design's readings but the four F6 spot-checked; most of `h3mem.py`'s offsets re-read against the
  executable (`node_cost`'s underground index among them); `h3adv.py`; the unit tests beyond
  spot checks.
