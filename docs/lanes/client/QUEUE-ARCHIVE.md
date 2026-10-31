# Client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-10-31 (E17 M6 T1 done; the entry verbatim):

### E17 M6 T1 — the answered slips (done 2026-10-31)

- **Scope:** the user's answers of 2026-10-31 (the lane's `QUESTIONS-ANSWERED.md`), each fixed in
  the client's profile with the original's kept for the comparisons only (`h3_render::Bugs`, as
  the cursor's x): CLT-Q26 (the book's last hovered item forgotten at each opening,
  `book::Memory`); CLT-Q27 (1) (a Faerie Dragon's aim given up posts the mouse, as the book's
  does) and (2) (the aim's start keeps the cursor's answer: a click before a move casts there);
  CLT-Q28 (1) (the box's text centred on the box) and (2) (the box framed in the acting side's
  player's colours), with the user's directive (the epic's "Decisions"): every text the client
  draws where the original sets it off its centre is centred (the readings of design §7.17 to
  §7.24 and §9 checked for such places); CLT-Q33 (no log line when nothing was summoned);
  CLT-Q35 (the result window says "fled" or "surrendered" only for the side that did).
- **Acceptance:** each fix tested in the client's profile and the original's kept for the
  comparisons (`drawn` all the same); `clients/scripts/check.sh` all gates; the design and M6's
  review list name the fixes.
- **Depends on:** nothing. **Status:** ready.

- 2026-10-31 (E17 M5 T8c, M5's closure at the user's checkpoint; verbatim): the queue's M5
  header, T8's entry, the inbox entries M6's tasks took (T2 the stack's info, T6 the rest) and
  the informational notices, moved out unchanged.

### E17 M5 — the queue's header at M5's end

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23,
the adventure map client being the map-client lane's E24). M5 (live play) refined at M4's
closure (T3c, 2026-10-27, with the user): hot seat in-process first, then a game on a server;
its tasks below, in order (T1 and T2 done 2026-10-27, each cut in two by the user; T2b whole,
the user; T3 cut in two, T3a done 2026-10-27, T3b 2026-10-28; T4 cut in three, T4a done
2026-10-28, T4b cut in two, T4b1 done 2026-10-28, T4b2 and T4c1 done 2026-10-28, T4c2a and
T4c2b 2026-10-29: T4 done; T5 cut in two, T5a done 2026-10-29, T5b cut in two, T5b1 and T5b2
done 2026-10-29: T5 done; T6 cut in two by the user, T6a done 2026-10-30, T6b cut in two by
the user, T6b1 and T6b2 done 2026-10-30; T7 kept whole by the user, done 2026-10-30). The original files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M5 T8 — M5's closure (done 2026-10-31, T8c)

- **Scope:** a fresh reviewer, the battle recipes' starting bonus set to Gold (M4's review item
  15), every capture afresh, the review list, the next milestone refined with the user (the
  ladders, the pause, the server browser; HD; graphics options).
- **The user's cut (2026-10-30):** concurrent sub-agents, two capturing (one per harness
  instance: the runbook's "Two instances") and one reviewing or fixing, each given a fraction
  of about half a context; the runner keeps only their reports. **T8a done** (log, "M5 T8a"):
  the two instances, the bonus Gold, reviewers R1 (network, accounts), R3 (the play core), R4
  (the spell book) and their fixes; all 91 stills and sequences 0 to 27 as recorded.
- **T8b cut in two by the user (2026-10-30, the agent's proposal):** **T8b1** the captures
  (sequences 28 to 64), R7 and R2 (in two: R2a the decoder, R2b the clip) with their fixers;
  **T8b2** R4b, R5a to R5c, R6, the leftovers, then `drawn` and `check.sh`. **T8b1 done** (log,
  "M5 T8b1": sequences 28 to 64 as recorded, R7, R2a and R2b fixed; review items 73 to 77).
- **T8b2 done** (2026-10-31, log "M5 T8b2"; the user's fractions: three concurrent sub-agents,
  the runner the leftovers): reviewers R4b, R5a to R5c, R6 and their fixers; the leftovers (an
  entered stack carries the playhead, the name lookup's wait, `RecordError::Line` in numbers, the
  sequences tool naming a bad clip, `bink-reference.py` on a refused packet, R7-8 the Gold bonus
  read in memory); `drawn` and `check.sh`; review items 78 to 86, CLT-Q35.
- **T8c (the user's checkpoint):** the review list (items 1 to 86), CLT-Q35, the next
  milestone refined; M5 done.
- **Acceptance:** the review list accepted; every capture as recorded or its difference listed;
  the next milestone's tasks in this queue; M5 done.
- **Depends on:** T1 to T7 (done 2026-10-30). **Status:** T8a, T8b1 and T8b2 done; T8c ready (the user).

### Inbox entries taken by M6 (2026-10-31)

- 2026-10-29 (the user, at E17 M5 T5b1's start; for a free slot or M5's closure): **the
  Demons' dissolve, re-investigated.** The slip of T5a (the saved picture is the back buffer)
  is the rock clipped out; the Imps' corpse gone before the fade-out's end is not part of it in
  the user's view. Find where the corpse leaves the original's picture and model it if it is not
  the slip (design §7.25, "To re-investigate"; `seq-demons.toml`'s 8 dissolve frames).

- 2026-10-28 (the adventure lane, E08 T67; CLT-Q5's answer, decision 158): **boats in
  `h3_map::writer`.** An empty boat is a `Plain` object of class 8 (`PLAIN_OBJECTS`, the
  `Objects.txt` templates `AVXboat0` to `AVXboat2`, the type as subclass): as a hero's template
  it stands one column west of its position, on water, blocked and visited there. A map cannot
  put a hero in a boat (the original makes no boat for a hero placed on water, survey
  `docs/rules/surveys/E08-sea.md` §2, §5; batch E8's E8-5 tests it): a situation with a hero in a
  boat places the hero on the coast beside an empty boat and boards it in the game (a click on
  the boat), then the deck (a hero on land attacks it from the shore) or the two boats (both
  boarded, side by side). Batch E8b's E8-11 and E8-12 (`experiments/specs/E08-batch-8b.toml`) are
  the two setups. Also new: Flotsam, the Sea Chest, the Survivor and holes as `Plain` objects, an
  Ocean Bottle or sign with its text (`RewardBody::Sign`), the spec's `[[map.bottle]]` and an
  opponent monster's own `at` (any terrain).

- 2026-10-28 (E17 M5 T4b2b, CLT-Q30, the user: listed now, exact later): **`h3-bink` to decode
  as the game's `BINKW32.DLL` (Bink 0.5b).** Its inverse DCT is x87 single-precision floating
  point (`ExpandPlane`'s 8 × 8 transform: √2, 1.848, 2.613 and 1.082 as floats, float
  quantizers), FFmpeg's integer, so five clips differ by one level on a few pixels (`rtstart`,
  `rtloop`, `Surrender`, `defendloop`; `Defendall` up to 388 a frame; design §7.21). The task:
  the DLL's transform and quantization in integer arithmetic (no floats in the client: exact
  binary32 emulation or a proof that a fixed-point form rounds alike on every input the
  bundles allow), checked against the DLL's own planes (`binkprobe frames <vid> <clip> 7`:
  YUY2) on the ten clips and the synthetic samples; FFmpeg's check (`bink-check.sh`) then
  becomes the DLL's. Then the captures' listed DCT pixels (`result-retreat` 2 700 and 3 000,
  `defended`) go.

- 2026-10-28 (the map client lane, E24 M3 T2a; additive, nothing of yours drawn changed): **the
  dialog items' drawers in the common crate.** `h3_ui::paint::draw` draws each `h3_ui::ui::Draw`
  (picture, button, text, label, darken) as `h3_view::compose` does (ported from it, the same
  pixels) from any batch implementing `h3_ui::paint::Assets` (`sprite`, `picture`, `palette`,
  `font`), and `h3_ui::paint::names` gives the assets a draw needs; the map viewer draws its
  adventure dialog with it. M3's exit wants `h3-ui` the only copy of what both viewers share:
  your compositor's `Draw::Dialog` arms and `batch.rs`'s names switched to it (your `Batch`
  implementing `Assets`) remove your copy. Also additive: `Dialog::draws` gives a button the
  player's colours (`Shown::players`) and a DEF (`Shown::defs`) by its id, which no battle
  dialog sets (`h3drive.py drawn` all the same).

- 2026-10-27 (E17 M5 T2a, the click read; for M5 T4 or later): **the stack's and the turret's
  info by the mouse** are not built: a left press on a stack to look at (kind 5) opens its stats
  dialog (`stack_stats_dialog` 0x468860, modal), on a turret (22) its info (0x47A380); the right
  button pressed over the battlefield shows a popup while held (0x4769C0, not read), over a
  control its help (0x472A50). `mouse::click` gives no command there (design §9.11).

- 2026-10-24 (the map-client lane, E24 M1 T6b's reviewer, R2-15; for you to take or leave):
  **`h3-view`'s `tests/purity.rs` has three holes** its copy in `h3-map-view` had: `sources` skips
  every folder named `bin` at any depth (only `src/bin/` is the adapter), a denied lint relaxed
  through its group (`#[allow(clippy::restriction)]`) or by an attribute over several lines is not
  seen, and the required header is matched as a substring (a commented-out `// #![no_std]`
  passes). `clients/crates/h3-map-view/tests/purity.rs` (E24 M1 T6b) has the fixes:
  `walk` from `src` leaving out `src/bin` only, `attributes` joining an attribute's lines, the
  `GROUPS` list, the header as a line of its own.

- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R4-2: `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s
  boxes, the log's text: digests recorded again from `master`'s viewer first, the runbook's
  rule). R4-9: a stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an
  artifact over almost anything; the counts stay exact). (R2-7, R2-8 with R1-3 and R3-1 went into
  M5 T1, T6 and T5 at M4 T3c; the entry's full text in `QUEUE-ARCHIVE.md`.)

### Informational notices (moved 2026-10-31)

- 2026-10-30 (the adventure lane, E08 T79a; for information, nothing to do): the harness's
  reads and steps for batch E9a (`clients/capture/README.md`, "Mines, Lighthouses and Shipyards").
  Behaviour that changed for every recipe: `h3lod.identify` also knows `MINEEVNT.TXT` (a mine's
  flag message, `MINEEVNT:<n>`; it was named GENRLTXT:4, its picture's caption) and lets a
  record's title braces be missing; a walk records the cursor before its first click (`cursor`);
  a battle's record keeps each side's owner and controllers (`sides`); the boat dialog (vtable
  0x643908) is a screen a step's `actions` play, cancelled when left open. New: `mine:x,y`,
  `tile:x,y`, `screens`, `opening:sides`, `opening:stacks:<side>`, `opening:slots:<side>`, `stops`,
  `battle = "auto again"`, `row:<column>#2`; `h3mem.mines`, `tile_word`, `battle_sides`,
  `auto_combat_on`. The harness check passed.

- 2026-10-30 (the adventure lane, E08 T79b; for information, nothing to do): the harness's
  reads for batch E9b (`clients/capture/README.md`, "The day's start"). Behaviour that changed for
  every recipe: `object:x,y` and `word:x,y` of an object a hero stands on read it through the
  hero's record (+0xC, +0x14; they read the hero's class 34 and index before), `fill` and
  `compare` key a row label repeated in a table `<label>#<n>` (its second row stayed empty before),
  `message_record` names a message whose text is one of the spec's own `the map's text "..."`.
  New: `income[:<colour>]`, `gardens[:<colour>]`, `growth:x,y`, `gain[:<colour>]`, the weekly
  objects' `object:` texts (Windmill, Water Wheel, Mystical Garden), an end-turn step's `record`
  and `note` (the new turn's messages: `end_turn(game, mem, events)`, `dismiss(..., mem, events)`,
  both optional), a recipe's `gold = true`; `h3mem.player`'s `income` and `gardens`,
  `h3mem.monster`'s `sixteenths` and `word`. The harness check passed.

- 2026-10-30 (the map client lane, E24 M3 T6a2b, the user's consent; for your information,
  nothing to do): `h3-view`'s test `the_committed_catalogue_parses` lists SEERHUT among the
  tables the catalogue's keys read: the map hint's quest texts (`map.quest.<type>`, a form per
  SeerHut.txt column) are keys, which every client reads. The catalogue also has 33 more `map.`
  keys and 13 optional families only the map viewer lists (`clients/text/keys.toml`).

- 2026-10-29 (the adventure lane, E08 T68b; for information, nothing to do): the harness's
  reads and steps for batch E8b (`clients/capture/README.md`, "Boats and the sea", since T68b).
  Behaviour that changed for every recipe: `h3lod.identify` also knows `RANDSIGN.TXT` (a sign's
  or Ocean Bottle's default text, `RANDSIGN:<n>`) and lets a record's doubled quote match one or
  two (ADVEVENT 126 was not identified); a message one of the map's message records holds reads
  `the map's text "..."` before any table is tried (GENRLTXT 16, `%s the %s`, took it);
  `object:x,y` reads `none (class 3: the coast)` for a coastal tile without an object; `select`'s
  `controls` count only tiles that hold an object. New: `object:x,y` for the Sea Chest, the Survivor and the Ocean
  Bottle (an artifact chest and a Survivor hold an artifact for `require` and `select`), `select`
  by `is`, `count` and `then`, `h3mem.open_tiles(…, water=True)` (a hero in a boat approaches over
  water), `h3mem.messages`, table rows filled step by step (`row:<first column>`), a grid step's
  `note` at every tile, and the reads `battle:tile`, `boat_record:x,y`, `treasury_changed`,
  `text:<literal>`. The harness check passed.

- 2026-10-28 (the adventure lane, E08 T68a; for information, nothing to do): the harness's
  reads and steps for batch E8a (`clients/capture/README.md`, "Boats and the sea"). Behaviour that
  changed for every recipe: `h3mem.path` (the `marks`, `mark` and `path_tiles` reads) is empty
  while the map shows no path (adventure manager +0x4C, as `path_arrows`): a click that found no
  path left the last path's arrows, which it read before. New: the battle's ship flag in
  `battlefield`, `h3mem.obstacle_hexes`; `h3adv`'s `boat`, `boat[<hero>]`, `boat:x,y`,
  `boats:used`, `drawn`, `opening:field`, `battlefield:ship`, `blocked`, a Flotsam's `object:x,y`,
  and the steps `key`, `again_if`, `only_if = "aboard"` and `dump`. The harness check passed.

- 2026-10-28 (the map client lane, E24 M3 T1b; **your window's keys changed**; the user: every
  key through the table, no cut): **the battle window's keys go through `h3_ui::keys`** (#548,
  `cross-lane`, `map-client/e24-m3-t1b`, named in the map client lane's HANDOFF until it merges). The new `h3_view::keys` holds the battle's commands with their names and the
  original's keys by **context** (`Context::of`: your M5 T4b1's result window, else a message box,
  else the book, else an aim, else the battle): the window's own (`h3_ui::keys::Window`, the map's: F2 music, F3 the creatures'
  sounds, F5 pause, F6 a frame on, F7 restart, F8 vsync, F9 the cursor, F11 fullscreen, Esc
  quit; P, →, Home, M, E, V and H gone), `log-up`/`log-down` (↑ ↓), `dragon-spell` (F),
  the panel's seven buttons (`BUTTONS`: S, R, O, A, C, W, D and Space), the book's
  (`book::Key`: Esc and Enter close, A, C, ← → ↑ ↓) and the box's (`message::Key`: Enter OK,
  Esc Cancel), `aim-cancel` (Esc, the right button), `result-ok` (Enter and Escape, 0x4715C3;
  merged with #547: `result::window`'s OK and the stills' `one_button` lost their `keys`). The user's `keys.toml` (`--keys`, else the
  working directory) has a `[battle]` table (the map's `[map]`). API changes: `Book::key` and
  `book::Open::key` take a `book::Key`; `message::Open::key` a `message::Key` and the key's name
  (its release still by name); `h3_ui::ui::Button` lost `keys` and `Dialog::button_for_key`
  (the layout's panel `keys` gone: the table's defaults are their only copy); the stills' `key:`
  actions go through `Keys::original()` by context; `h3_sdl::Key::ALL` gained the missing
  `Return`. Behaviour changed: the book takes every key while open (the panel's letters and F
  reached the battle under it); unattended, Esc quits whatever is open and the right button no
  longer gives an aim up; a chord with Ctrl held gives nothing unless bound. `viewer_key` and
  `book_key` are gone. The 2026-10-26 entry below (custom key bindings) is done by it; your
  M5 T3b's F is the table's `dragon-spell`. A branch of yours merges `master` in after it merges.

- 2026-10-28 (the map client lane, E24 M3 T0; **decided by the user for both lanes**; the
  proposal of 2026-10-27 as it stood: `docs/lanes/map-client/QUEUE-ARCHIVE.md`): **the common crate
  `h3-ui`** (`clients/crates/h3-ui`, common to both client lanes as the foundation is: a
  `shared/` pull request for `lanes.toml` first) takes `h3-view`'s `clock`, `cursor` with
  `batch::Sprite`, `ui` (with draw items of its own instead of `scene::Draw`'s) and `text`, and
  `h3-map-view`'s `keys` (generic over a viewer's commands) and `music`; `layout` and `mouse`
  stay in `h3-view` (battle-only). The map client lane moves it as **E24 M3 T1a** in a
  `cross-lane` pull request (the user's consent): `h3-view`'s imports change, nothing it draws
  or plays. Then **E24 M3 T1b** takes `h3-view window`'s keys through the table (the 2026-10-26
  entry below), **both windows' own keys the map's** (the user: F2 music, F3 effects, F5 pause,
  F6 the next frame, F7 restart, F8 vsync, F9 the cursor's kind, F11 fullscreen, Esc quit; P,
  →, Home, M, E, F, V and H gone), so M5 T3b's F key is free. Starting M5 T3b before T1a and
  T1b merge means merging `master` in after them (the imports); the map client lane tells you
  here when each merges.

- 2026-10-26 (the map client lane, E24 M2 T1b; **for you, at M3 T0**): **custom key bindings**
  (the user, 2026-10-26: keys hard-coded so far are to be phased into bindings the player can
  change). The map client's window (E24 M2 T4) takes every key through a table of actions with
  the original's keys as defaults and a user file overriding them; at E24 M3 T0 that table moves
  into the common crate both lanes share (with `clock`, `cursor`, `ui`, …), and `h3-view
  window`'s keys (now matched in its event loop, `window.rs`) go through it too. Nothing to do
  before M3 T0's proposal; `h3-sdl`'s `Key` grows additively meanwhile (a common path).

- 2026-10-30 (E17 M5 T7 done, kept whole by the user; the entry verbatim):

### E17 M5 T7 — accounts

- **Scope:** the account's key from the login and the password (argon2id with the server lane's
  parameters, the seed as Ed25519's secret key; the test vector in `NOTES.md`, "The live
  client"), derived once per device and cached (never the password); the login signed only for
  the address dialed, a registration's display name; seats by account (joined without a token,
  re-taken after a drop); `Rekey` after a password change; the goodbyes `Rekeyed`,
  `CommandLimit`, `Abandoned`. Its dependencies (argon2, Ed25519): asked of the user at the task.
- **Acceptance:** the test vector reproduced; a battle played logged in on a local `h3-server`,
  the seat re-taken after a drop; the Definition of Done.
- **Depends on:** T6b2 (done 2026-10-30); the user's choice of the dependencies. **Status:**
  ready (T6a's `remote` keeps the welcome's challenge out until the login needs it).

- 2026-10-30 (E17 M5 T6b2 done: T6 done; the entry verbatim):

### E17 M5 T6b2 — the record that opens mid-battle

- **Scope:** a record that opens mid-battle (R2-8: the snapshot's dead stacks as corpses, a raise
  of one drawn, the box by the timeline's living set; M4 T3a's R1-3: the moat's strip by the
  fixed moat rows, 0x63BCE8 and 0x63BCF4, not the record's moat) for a join mid-battle and a
  re-join after a server restart (the numbering afresh: a new timeline); the window re-dialing
  until a restarted server answers. The detail: `QUEUE-ARCHIVE.md`, 2026-10-23 (T9b). Where
  T6b1 stops (design §11.2): `online::Online::take` refuses a fetched record that does not extend
  the one taken; `Ground::load` keeps the record's first snapshot as the opening.
- **Acceptance:** a join mid-battle; a server restart and a re-join; the mid-battle record drawn
  against the original where it has a reference; the Definition of Done.
- **Depends on:** T6b1 (done 2026-10-30); the original data staged. **Status:** ready.

- 2026-10-29 (E17 M5 T5b2 done: T5 done; the entry verbatim):

### E17 M5 T5 — what a played battle shows

Cut in two by the user (2026-10-29): T5a the dissolves and the created stacks (with the engine's
event for a stack without a place, `cross-lane`; done 2026-10-29), T5b the panel by each turn's
controller, the sounds' waits and the palette effects, cut in two again by the user (2026-10-29,
at its start): T5b1 the palette effects (done 2026-10-29), T5b2 the panel and the sounds' waits.
The detail: `QUEUE-ARCHIVE.md`, 2026-10-22 (T7c-3).

#### E17 M5 T5b2 — the panel by the turn's controller, the sounds' waits (done 2026-10-29, kept whole by the user)

- **Scope:** the panel's side by each turn's controller (R3-1: Hypnotize, a created stack); the
  battle waiting for the heal's sound and the other effects' (the Pit Lord's `Resurect.wav`,
  0x5A754E, among them). Recorded against the original.
- **Acceptance:** the new sequences as recorded, or each difference listed; the Definition of
  Done.
- **Depends on:** T5a (done). **Status:** ready.

- 2026-10-29 (E17 M5 T5b1 done; the entry verbatim):

#### E17 M5 T5b1 — the palette effects (done 2026-10-29)

- **Scope:** `draw_battle_stack`'s palette effects (0x43E280 to 0x43E4C7; the user,
  2026-10-29): a clone recoloured (0x522B50 with 0.67, 1, 2, 2 on palette entries 10 to 255: RGB
  to HSV in single precision, the hue set, saturation and brightness moved toward 1, back to
  RGB; `clone`'s frames differ by it alone, design §7.25), Bloodlust's glow (the same routine)
  and Stone Gaze's (0x522A10), Stone Gaze's petrified look (0x522D50). Recorded against the
  original.
- **Acceptance:** the new sequences as recorded, or each difference listed; `clone`'s colours
  exact; the Definition of Done.
- **Depends on:** T5a (done). **Status:** done (`client/e17-m5-t5b1`).

- 2026-10-28 (E17 M5 T4b1: `master` merged in after #545, the imports followed; the entry verbatim):

- 2026-10-28 (the map client lane, E24 M3 T1a; **your imports changed**): **`h3-ui` built**
  (`clients/crates/h3-ui`, common to both client lanes since #541's `lanes.toml`; the move in a
  `cross-lane` pull request of the map client lane's, named in its HANDOFF until it merges).
  `h3-view`'s `clock`, `cursor`, `ui` and `text` are `h3_ui::clock`, `cursor`, `ui` and `text`
  (`h3_view::{clock, cursor, ui, text}` gone; the binary's own `text` adapter stays);
  `batch::Sprite` is `h3_ui::sprite::Sprite`; the scene's five dialog variants (`Picture`,
  `Button`, `Text`, `Label`, `Darken`) are one, `Draw::Dialog(h3_ui::ui::Draw)`, with
  `PlayerColours` in `h3_ui::ui` (`Dialog::draws` gives `ui::Draw`s; `scene::panel` and
  `Book::draws` and the binary's message boxes wrap them; `compose` and `batch::names` match
  them inside `Draw::Dialog`; your M5 T4a's `size` and `player` merged into them, `message` on
  `h3_ui::{ui, text}`);
  `tests/purity.rs`'s allowlist for `h3-view` gains `h3-ui` (`serde_json` is now the binary's
  only, left in the list). Nothing drawn or played changed (`h3drive.py drawn` and `h3map.py
  drawn` all the same). `h3-ui`'s own `tests/purity.rs` has R2-15's fixes (the 2026-10-24 entry
  stands for `h3-view`'s). A branch of yours from before it (M5 T3b) merges `master` in after it
  merges: the conflicts are these paths. T1b (the battle window's keys through `h3_ui::keys`)
  follows.

- 2026-10-27 (M4 T3c: taken by R2-7 into M5 T1, R2-8 with M4 T3a's R1-3 into T6, R3-1 into T5; R4-2 and R4-9 stay in the inbox; the entry verbatim):

- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R2-7: a timeline that steps forward (`Moment::at`, `Heroes::posed` and `Timeline::at`
  rescan the battle for every frame: 2.3 ms a frame over 82 seconds, `play`; for live play's long
  games). R2-8: a record that opens mid-battle (the snapshot's dead stacks as corpses, a raise of
  one drawn; the box by the timeline's living set, not the first snapshot's `alive`; with it
  M4 T3a's R1-3: the moat's strip over a stack by the fixed moat rows, 0x63BCE8 and 0x63BCF4,
  not the record's moat, which leaves out the bridges while the opening's gate is open). R4-2:
  `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s boxes, the log's
  text: digests recorded again from `master`'s viewer first, the runbook's rule). R4-9: a
  stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an artifact over almost
  anything; the counts stay exact). R3-1 goes with the created stacks below.

- 2026-10-27 (M4 T3c: taken by M5 T3; the entry verbatim):

- 2026-10-21 (T7b-1): **a hero's aim.** Between the book's place and the target's click the
  cursor is the spell's (`crspell.DEF`, animated: not read; the viewer leaves it as it was) and the
  status line says "Cast %s on %s" (GENRLTXT 28, built *provisional*, not forced; 27 elsewhere);
  `enter_player_cast` 0x59EC50's loop is not read (which targets it offers, the cursor's frames):
  with the mouse's model when a recording aims a hero's area spell at a hex (T7c-1's are the
  Faerie Dragon's clicks), and for live play. And `cast`'s 71
  pixels at 0: the original redraws the battlefield over a rock's top (obstacle 22) for the one
  frame of the click (0x479038's path, not read).

- 2026-10-27 (M4 T3c: taken by M5 T4 (the volumes to `NOTES.md`, "Options and layouts"; `remote_turn` to T6); the entry verbatim):

- 2026-10-21 (T8): the battle's result window (`battle_result_dialog` 0x4702D0, not read beyond
  its tune: the heroes, the texts, the casualties, OK) and the whole screen darkened under it
  (0x44E5F0 from 0x47782C): `end`'s last frame differs by them (its counts). The volume settings
  (0x5996C0) and a track resuming where it stopped (0x59AA82) with the client's options and a
  second battle (later); the network's `remote_turn` in the end's choice with live play.

- 2026-10-27 (M4 T3c: taken by M5 T5; the entry verbatim):

- 2026-10-22 (T7c-3): **created stacks are not drawn.** `StackCreated` (Summon Elemental, Clone)
  and a Pit Lord's `RoseFromCorpse` add a stack the record's first snapshot lacks, and the scene
  draws only the snapshot's stacks (a wire `Stack` each): the viewer needs the new stack's
  numbers (from the ruleset, or a later snapshot in the record), its record index and the
  dissolve that shows it (`create_battle_stack`, 75 ms steps, the blend not read; T9d built a
  dissolve's time for Sacrifice's victim, `anim::GONE`, `DISSOLVE_STEP`, and the frame check of
  `sacrifice` lists its blend: `screen_dissolve` 0x602DC0, read for both); with them the
  message when no hex is free, a clone's `Vanished`, Teleport's dissolve (moved at once now) and
  Sacrifice's victim leaving no corpse; and the panel's side by each turn's
  controller (T9b, R3-1: `scene::controller` takes the opening's, so Hypnotize and a created stack
  are not followed). The battle waits for the heal's sound (T3b) and the other
  effects' (T7b-2b, T7c-3). NOTES, "The battle screen".

- 2026-10-27 (M4 T3c: taken by M5 T4; the entry verbatim):

- 2026-10-20 (T6a): the tactics phase's panel (0x46C050, not read) and Auto Combat switched on
  (its button selected, the panel dimmed while stacks play themselves) wait for a record with a
  tactics phase and for live play. Design §9.7.

- 2026-10-27 (M4 T3c: taken by M4, done 2026-10-27; the entry verbatim):

- Sieges (the battle lane, 2026-10-14 to 16): the turrets as stacks (drawn as the town's
  shooters since T1b), the Catapult, the gate opening before the step. NOTES, "Sieges"; M4 T2b
  to T2c-2 took the shots, the gate, the moat, the mines, Earthquake and the mouse (design
  §10.2).

- 2026-10-27 (M4 T3c: taken by M5 T6 and T7; the ladders, the pause and the server browser after M5; the entry verbatim):

- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".

- 2026-10-26 (the adventure lane, E08 T57; for information, nothing to do): the harness's
  reads and steps for batch E7 (`clients/capture/README.md`, "Rewards on the map"). Behaviour that
  changed for every recipe: `before:<observation>` is now read right before the step (any
  observation, tiles too; it was the step's snapshot), a message box whose OK waits for one of two
  pictures is answered by `choice` (it stopped the batch before), and each message closed with its
  OK keeps its pictures (`message_pictures`). New: `h3mem.object_at`, `universities`,
  `dialog_pictures`, `open_tiles`, `map_kind`, a hero's spell points, bonuses and status flags, a
  stack's army slot; `h3adv`'s `each`/`select` grids, list records and `+` fields, `once`,
  `days`, `near` and `[[require]]`. The harness check gained Win's chest and Warrior's Tomb.

- 2026-10-26 (T2c-2): **the status line at a human turn with the mouse parked** is kept in hot
  seat and emptied with one human side, built as found (*provisional*; design §10.2, "Recorded
  (M4 T2c-2)"). The dispatch is read (an item writes its hint only on entering, `hovered_item`
  0x6AAC68); open: the item under the park point and what makes it entered again (`aim`'s line
  emptied at 8 175, `hit`'s kept). A capture reading 0x6AAC68 and that item's id per frame
  would settle it (T3's captures or live play). Done at M4 T3b-1 (2026-10-27,
  design §10.2 "Recorded afresh").

- 2026-10-23 (the map-client lane, E24 T0; for later, nothing to do now): **what the map client
  will need of `h3-view`.** E24 M1 (stills of the map view) builds only on the common foundation;
  M2's window will want `clock` and `sound`, M3's panels `ui`, `text` and `layout`, in a common
  crate. When M2 or M3 is refined, the map-client lane proposes the move here first (a new common
  crate is a `lanes.toml` change, shared); your M4 waits for none of it.

- 2026-10-23 (the adventure lane, E08 T47a; for information, nothing to do): the harness's
  clicks and reads for batch E6 (`clients/capture/README.md`, "Screens and their items"): the
  screen showing is the topmost dialog's kind by its vtable (`h3mem.dialog`), and a click finds
  its item by id among the dialog's items in memory, which now carry their boxes, states and texts
  (`h3mem.dialog_items`; `dialog_text` shares `item_text`); the hero screen, exchange, army,
  creature and split dialogs' items by name (`h3adv.SCREEN_ITEMS`); a step's `screen = "hero"`
  and `actions`; a battle's tactics phase ended at once in `fight` (its message's OK, then Start).
  The harness check gained Kit and Full (Blue's eight heroes): passed under Wine.

- 2026-10-21 to 22 (the adventure lane, E08 T32b1 to T37b; for information, nothing to do): the
  harness's reads and steps for batches E4 and E5 (moved to NOTES, "The adventure lane's
  harness", at T9c; the harness is common since the split, 2026-10-23).

- 2026-10-20 (the adventure lane, E08 T22a2b): its three recipes, its auto combat's change, for
  information (NOTES, "The adventure lane's harness"; the winner of −1 fixed in E17 M3 T6c-2).

