# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024). M3 (fidelity) below, refined at M2's closure (T5c) and agreed by the user there, each task about half a context; the exit: the reference set passes, or each difference is a listed fix or a classified quirk. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M3 T2 — Every battlefield

- **Scope:** the special battlefields from the battle's own facts: the observation's
  `battlefield` (protocol v28, the packs' keys `base:shore`, `base:magic_plains`,
  `base:cursed_ground`, `base:deck`, `base:boats`, `sod:holy_ground` … `sod:magic_clouds`) picks
  the picture of 0x4646D0; the asset tables' special battlefields keyed by the packs' keys (now
  VCMI's names under `base`); water and rock battles render (the decks and boats); design §5's
  stale sentence on the 47 obstacle types no pack defines. The capture set gains each field the
  harness can set up; where `h3-map`'s writer lacks the object, a question to the adventure lane
  (`CLT-Q<n>`). The background variants 0 and 1 stay with CLT-Q4.
- **Acceptance:** every special field renders; those captured match outside the masks.
- **Depends on:** nothing. **Status:** ready.

### E17 M3 T3 — Heroes and war machines on the battlefield

- **Scope:** the heroes (design §4.5: their battle DEFs by class and sex, the flags; their idle
  groups and their sounds, not read), the war machines (the Ballista of the smoke capture, the
  First Aid Tent, the Ammo Cart: places, DEFs, the tent's and the Ballista's actions), each
  capture mask removed as it is drawn. The Catapult, walls and towers come with sieges (later).
- **Acceptance:** the smoke battle and a hero battle of the capture set match outside the
  remaining masks.
- **Depends on:** nothing. **Status:** ready.

### E17 M3 T4 — Fonts and the string catalogue

- **Scope:** which font draws what in the battle (the executable), a decoder of the original's
  bitmap fonts (`.FNT`) in `h3-assets` (the gate decodes every one), text drawn by `h3-render` as
  the executable does; the string catalogue and the language packs (the user's decision of
  2026-10-13 and the agent's notes in the inbox below): its format (JSON, TOML or Fluent, chosen
  with the placeholders and plurals the battle's texts need), the original's text read at run
  time from the user's `.TXT` entries through committed keys; an ADR-0020 amendment drafted in
  `adr-drafts/` for the user's checkpoint; the licence question on committing translations of
  the original's text filed as `CLT-Q<n>` (stop the line).
- **Acceptance:** the gate decodes every font; a text drawn by the viewer matches the original's
  pixel for pixel.
- **Depends on:** nothing. **Status:** ready.

### E17 M3 T5 — The count boxes

- **Scope:** each stack's count box (design §4.3: its place, `cranim.txt`'s offset, the colours
  by the stack's effects, the digits in the battle's font), drawn when the original draws it
  (not during a stack's own animation: to read); their masks removed from the capture set; a stale box the
  original's partial redraw leaves under a walker (design §7.11) is a redraw artifact, not
  copied: T1c lists it (`melee` at 3 966 ms: 231 pixels, the frame check's other count) until
  the boxes are drawn; then classified (the box's earlier place) or masked.
- **Acceptance:** the capture set matches with the count boxes drawn.
- **Depends on:** T4. **Status:** ready.

### E17 M3 T6 — The battle interface

- **Scope:** the panel below the battlefield (its picture, buttons and their states), the log's
  lines from the string catalogue as the engine's events produce them, the heroes' portraits if
  the panel shows them, the cursor rules of the executable (which cursor over which hex; T5a's
  `CRDEFLT.DEF` frame 0 is provisional), the mouse over a stack in the window (group 1 and the
  outline 0x70, read and recorded in T1b-2: the inbox); its masks removed. May split when refined.
- **Acceptance:** a frame with the panel matches the original's outside the listed differences.
- **Depends on:** T4. **Status:** ready.

### E17 M3 T7a — Code reading: the magic animations

- **Scope:** the user (T5c): the plethora of spell animations. How the executable draws a spell
  or a magic animation (H3API's PlayMagicAnimation and its table: each animation's DEF, where it
  is drawn: over a stack, a hex, the field, from caster to target; its timing and its place in
  `strike_animation`'s L = max(the magic animation's frames, A + R − 1), §7.4; the spells'
  sounds by the spell table's names); the hero's and a creature caster's cast animation; morale
  and luck through the same table. Recorded sequences of a few spells of each kind in the
  original (the harness casts from a hero's spell book: to build).
- **Acceptance:** design §7 gains the reading with every address in the map; each kind has at
  least one recorded sequence.
- **Depends on:** nothing (T1a's battle in progress). **Status:** ready.

### E17 M3 T7b — Spells on stacks, morale and luck

- **Scope:** built from T7a: the spells cast on stacks (blessings, curses, damage on one stack,
  `SpellCast`, `StackCast`, `EffectApplied`, `SpellResisted`), Haste and Slow's walk time (from T1b
  at its re-cut: the harness casts from T7a's spell book), good and bad morale (`GoodMorale`,
  `BadMorale`: the user at T5b, the Marksmen's second shot showed nothing) and luck (the strike's
  `lucky`), their sounds, a magic animation lengthening a strike's loop.
- **Acceptance:** their recorded sequences as recorded and their frames match.
- **Depends on:** T7a. **Status:** ready.

### E17 M3 T7c — Area spells, rays and spell obstacles

- **Scope:** built from T7a: the area spells (Fireball, Inferno, Meteor Shower, Frost Ring,
  Armageddon, Death Ripple and the like), the lightning spells' rays (`draw_ray` read and built at
  T1b-1, design §7.12; not yet its branches, randomness and colours 300, 301, 303, 305, nor
  Chain Lightning's call), the spell obstacles in the
  scene (fire walls, force fields, quicksand, land mines: `ObstaclePlaced`, `ObstacleRevealed`,
  `ObstacleTriggered`; `scene::battle` leaves them out so far), Teleport, created and raised
  stacks (`StackCreated`, `SpellRaised`, `Reborn`: drawn once the snapshot lacks them), their
  sounds. May split when refined.
- **Acceptance:** as T7b.
- **Depends on:** T7a. **Status:** ready.

### E17 M3 T8 — The sound's remainder

- **Scope:** the battle's end (`battle_end` 0x477470 calls `play_music` twice: the victory and
  defeat tunes, with the results), the start sound's volume (its loader's default, not read);
  the volume settings (0x5996C0) and a track resuming where it last stopped (0x59AA82) wait for
  the client's options and a second battle (later).
- **Acceptance:** the end's tunes play at the times the executable plays them.
- **Depends on:** nothing. **Status:** ready.

### E17 M3 T9 — M3's closure

- **Scope:** M3's review list with the user, a fresh reviewer agent over M3, the next milestone
  refined (sieges, live play, HD or the adventure map: the user's choice), M3 marked done.
- **Acceptance:** M3's exit holds; the review list answered or carried on.
- **Depends on:** T1b-1, T1b-2, T2–T8. **Status:** waiting.

## Inbox

- 2026-10-15 (E17 M3 T1b-2): **for T6, the mouse over a stack** (design §7.13; read,
  `battle_mouse_stack` 0x477C20, and recorded: the sequence `hover`): the stack the mouse enters
  starts group 1 at frame 0 at once (unless it shows it: then it stands), the idle step plays it
  as a fidget (stalls, the fidget clock drawn again at its end), and the stack takes the mouse's
  outline (entry 0x70, the cyan ramp) until the mouse leaves it. The window (`h3-view window`)
  has a mouse but no such state yet; with the cursor rules, the hover's timeline (`hover.txt`)
  checks it. Also there: where a two-hex striker stands by the cursor's slot inside the target's
  hex (`cursor_attack_from_hex` 0x474A00, not read further; the harness's offsets in §7.13).
- 2026-10-15 (E17 M3 T1b-1): **for T2:** a fresh `h3-view harvest` writes 35 obstacles more
  into `base.toml` than the committed table holds (`obstacle_61` on, "not in a pack yet", and
  absolute ones), the packs' numbering having grown; T2 reconciles them with the special
  battlefields.
- 2026-10-14 (the battle lane, E04c T7, `cross-lane`; for your information): a town's turrets are battle stacks now (BAT-SIE-016): with a Citadel the defender gets the keep, with a Castle also two towers, each a stack of the Arrow Tower (`base:arrow_tower`, 149) at the original's off-field positions 254 (keep), 255 (upper) and 251 (lower tower), so a `StackView`'s hex can be one of those (no field hex); they shoot first every round (`Struck` with the turret as attacker). The creature has no picture in the executable's creature table, so `h3-view`'s `tests/tables.rs` lists it among `GAPS`; the original draws turrets from the town's records (0x63CF88, `installation.md` §7.27). No wire change; observation encoding v18 (a turret flag per stack), C ABI v22.
- 2026-10-14 (the battle lane, E04c T6, `cross-lane`; for your information): the base pack gained the Catapult creature (145, `base:catapult`), so `clients/assets/base.toml` gained its entry as `h3-view harvest` writes it (`smcata.def`, missile `SMCatx.def`, the harvested times and offsets) for `every_pack_key_has_an_entry`; the battle sets the Catapult up in a siege for an attacker with a hero (BAT-SIE-009), and the protocol (v31) has `ShootWall`, `WallShot`, `WallQuaked`, `WallDestroyed` and `Siege.parts`.
- 2026-10-13 (the adventure lane, E08 T17b; its `cross-lane` pull request, the user at ADV-Q12): **what increment 3's batch E3 needs from the harness** (`docs/rules/surveys/E08-battles.md` §13; the spec and its recipes come with E08 T22a, after four building rounds, so not before the lane's next milestones): a visit that asks Yes/No (a monster's offer to join, to join for gold, to pursue; a guarded pile's question), a battle started from the map (by stepping next to a monster, by visiting an enemy hero in hot seat) with its stacks' counts read at the start and the end, a surrender and both players' gold before and after (E3-8), the underground (a monster on level 1, E3-4), memory reads of a monster's tile word (count and aggression, E3-3, optional) and of the 156 heroes' experience after a new game (E3-7), a hero's level, primary and secondary skills at the start (E3-6), the formation in a battle (E3-9). The game's difficulty at the start (Easy and Normal, E3-1, E3-2).
- 2026-10-13 (E17 M2 T6, done the same day: the harness plays the adventure's batches, `capture/h3adv.py batch`, design §6.3): **what it does not do yet**, each built when a batch needs it: the hero list's scrolling (more than five heroes per player), the underground (the path's arrows of level 1, the minimap's level), a visit that asks Yes/No, a battle started from the map (the adventure lane's increment 3: monsters and battles; a battle's recipe steps, quick combat or the battle's own clicks), a second day. (The battle lane's cross-check of its E04a batch 1a is done there: #179, merged 2026-10-13, replays R1 with its own `experiments/agent/crosscheck_r1.py`.) A new batch needs its recipe (`capture/batches/`): the domain lanes ask here with the spec.
- 2026-10-13 (E17 M2 T1): **the background variants** 0 and 1 need trees, mountains or dead vegetation around the battle (design §5: `h3-map`'s writer has no decorative objects yet: an adventure-lane change); CLT-Q4: how a record carries the variant. (The rest of the capture set's growth went into M3 T1b, T2, T3, T5 and T6 at T5c.)

- 2026-10-13 (the user, at E17 M1): **text through a string catalogue.** Every string the client shows comes from a key → string catalogue (nested JSON or similar), easy to translate, language packs swappable; not the original's bare text files. The agent's notes for the refinement (M3, with the fonts and the battle interface, where the first text appears): the original's text cannot be committed (ADR-0009), so the original's language is read at the checkpoint from the user's own `.TXT` entries through a committed catalogue of keys (key → archive entry, row, column), and a localized installation is a language pack of its own; the client's own strings (lobby, accounts, ladders: no original text) are committed in any language; a translation of the original's text is likely a derivative work (a licence question for the user before any is committed); the catalogue's format (flat or nested JSON, TOML, Fluent for plurals and cases) and how placeholders and plurals work; fonts: the original's are 8-bit bitmap fonts in a code page, so a language with other letters needs fonts of its own (or a modern renderer as a graphics option). An ADR-0020 amendment at that refinement.
- 2026-10-13 (the user, at E17 M1): **graphics options.** Eventually the user chooses per effect whether it is drawn the original's dated but familiar way or a modern one (shadows first). The agent's notes: `faithful` stays the default and the only setting the reference frames check (ADR-0020 point 9); an option is a field of a settings value the compositor takes, never a fork of the scene model, so the draw list keeps naming what is drawn and only `h3-render` decides how; a modern shadow may want more than the 16-bit screen (alpha over a 24-bit frame), which ADR-0020 point 3 has to allow; independent of the asset fixes' `faithful`/`fixed` profile (point 6). An ADR-0020 amendment when the first option is built.

- 2026-10-13 (E17 M1 T8, the reviewer): an HD layout needs the battlefield's origin and viewport in the layout format (the heroes, war machines and the water and rock battles went into M3 T2 and T3 at T5c).
- 2026-10-12 (the user, at E17 M0): the adventure map's object draw order is surprisingly complicated; the user's own engine and VCMI (at the time) did not get it exactly right. Research item 9 (HANDOFF): a code reading of the executable's map drawing (the order of objects, their shadows, heroes and overlapping tiles) when the adventure map viewer is refined, with reference frames from the original to confirm.
- 2026-10-12 (E17 M0 checkpoint, the user): **CLT-Q2**, a table-only LOD reader in `h3-formats` for the battle lane (`docs/OPEN-QUESTIONS.md`). When it lands, the client's archive set switches from whole reads to the directories at start and a batch's entries by offset; `h3-view gate` then measures that.
- 2026-10-09 (server lane, SRV-Q3, the user): the server's first client is its own headless test client (M1); the client lane's viewer comes next. Proposal for the client refinement: the viewer reads `h3-protocol` messages live over the WebSocket or from a *record* file (one viewer's message log, JSON lines) with the same code, so the record format is the viewer's file format and one code path plays a live game and a recorded one. Open for the client ADR: whether the client workspace depends on `crates/h3-protocol` by path. Context: `docs/epics/E16-server.md`, `docs/epics/E16-design.md`.
- 2026-10-11 (server lane, E16 M2 T5e; cross-lane, the user): two things a client of the server must do. (1) **The content check** (`docs/epics/E16-design.md` §3): if the client has the packs and the profile that `Joined`'s manifest (or the lobby's `Offer`) names, it compiles them and compares the ruleset hash; equal means its own previews (damage ranges, reachable hexes) are safe, different or absent means the server's queries only and a visible note. The server never learns the result. (2) **A server restart**: a restored game's seats re-join with their tokens and get a fresh snapshot; the viewer's message numbers and its record start afresh (a `FetchRecord` then begins at the re-join), so a client drops what it numbered before; the whole game is in the replay once it is over (`FetchReplay`, or the public archive: `ListReplays`, `FetchArchived`). Context: `crates/h3-protocol/src/wire.rs`, `docs/06-security.md` ("Hosted games").
- 2026-10-12 (server lane, E16 M3 T5; cross-lane, the user): what a client of a server with accounts must do (wire 13; `docs/epics/E16-design.md` §10, ADR-0019 point 1). (1) **The key**: the login name (3–20 of `a–z 0–9 _ -`, lowercased by the client) and the password derive the account's Ed25519 key: `seed = argon2id(password, salt = "h3-account-v1:" + login, 64 MiB, 3 passes, 1 lane, 32 bytes)`, the seed as Ed25519's secret key; the test vector (login `alice`, password `correct horse battery staple`): seed `a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69`, public key `b17dbb2796b53c1c636e9a5981d6424ac5ee096b89b47223567831f435a8af14`. Derive once per device and cache the key, never the password. (2) **The login**: sign `Login` (`h3_protocol::account::login_message`) only if the `Welcome`'s `id` is the address the client dialed (a hostile server could relay the login otherwise); a registration names a display name. After a password change, `Rekey` with both keys at the next login on each server the client knows. (3) **Seats by account**: a seat taken logged in belongs to the account; its `Joined` carries no token, and the client re-takes it after a drop or a restart by logging in and joining without a token; `GameInfo::seated` names the seats' accounts by display name. (4) **New goodbyes**: `Rekeyed` (the account re-keyed elsewhere: log in again), `CommandLimit` (the game is void), `Abandoned` (a waiting game its players left). The server's test client (`crates/h3-server/src/client.rs`) shows the derivation, the login and the re-key (it signs the welcome's id without that check: a test tool).
- 2026-10-12 (server lane, E16 M4 T7; cross-lane, the user): what a client of a server with ladders must do (wire 20; `docs/epics/E16-design.md` §11, ADR-0019 points 4–9). (1) **Queueing**: `ListLadders` → `Ladders` (each ladder's id, name, season, waiting count and its bots' display names); logged in, `Queue { seq, ladder, bots }` (`bots: false`: people only, however long the wait; left out: a bot after the ladder's wait) → `Queued`, or `Refused` (`NotLoggedIn`, `NoSuchLadder`, `AlreadyQueued`: one queue at a time, `InRankedGame`: its unfinished match on that ladder, `Draining`); `Dequeue` → `Dequeued`. (2) **The accept step**: `MatchFound { ladder, accept_ms, bot }` (show the countdown, and say when the opponent is a bot); `AcceptMatch` (nothing answers until both accepted) → `MatchReady { ladder, game, seat }`, then `Join` that game as that seat, no token; `DeclineMatch` → `Dequeued` (out of the queue); `MatchCancelled` (the other declined, expired or left: still queued, the wait kept); `Unqueued { Expired | Draining | Full }`. (3) **Reconnecting by account**: a ranked game's seats are the accounts'; after a drop or a restart, log in and `Join` without a token, within the ladder's grace per absence and total per game (5 and 10 minutes by default); `GameInfo::ladder` marks a match's game in the lobby, `Seated::bot` a bot's seat. Answer the server's WebSocket pings (any library does): a connection silent for 30 s is closed (`IdleTimeout`). (4) **The end**: `GameEnded` may carry `Forfeit { side }` (a player away too long), `NoShow {}` (both away: void, unrated) or `Unverified {}` (void, unrated); a rated game's seats then get `Rating { n, ladder, before, after }` in whole points. (5) **The pause by agreement**: `ProposePause { seq, ms }` → `PauseProposed { n, seat, ms }` to every viewer; the other seat `AnswerPause { seq, accept }` → `PauseAnswered`; a proposal lapses at the next command; agreed, the game pauses (`Goodbye { Paused }`) and the players re-join by account before its due time; rejections `NotRanked`, `AlreadyAsked`, `NoPausesLeft`, `BadPauseLength`, `NoProposal`. (6) **Leaderboards**: `Leaderboard { seq, ladder, season, page }` → `LeaderboardPage` (pages of 50, `LeaderboardRow::bot`), logged in `Standing { seq, ladder }` → `OwnStanding`. The server's tests (`crates/h3-server/src/tests/ladders.rs`) show each flow over loopback.
- 2026-10-13 (server lane, E16 M6 T6; cross-lane, the user's M6 cut, SRV-Q23): what a client needs to find servers and know it reached the right one, a server browser's inputs (wire 27; `docs/epics/E16-design.md` §12, ADR-0021; mDNS later, with the browser). Keys and signatures are lowercase hex; the checks are `h3-protocol`'s, signatures Ed25519 checked strictly (`verify_strict`, no weak key). (1) **The welcome**: `Hello { protocol, client, name, nonce, address }` with a fresh 32-byte nonce and the address dialed; the `Welcome`'s `signature` by its `key` over `account::welcome_message(id, nonce, challenge)` must verify and its `id` must be the address dialed (the login's rule too). **Pin** the key per address on first contact (a bookmark is an address and a key) and warn loudly when it changes, as SSH does; one server may answer at several addresses with one key. (2) **The card**: `Describe { seq }` → `Described { seq, card }` (after the hello, no login): `card::read(&card, now, verify)` checks the bounds, the signature by the card's own key, the expiry and an issue at most 10 minutes ahead; the card's `key` must be the welcome's. It names the server, its addresses, protocol and engine versions, rulesets (packs, profile, ruleset hash: group servers by the content the client has, the content check of 2026-10-11), ladders, anonymous play, an operator contact, optional counts, and its peers. A web page may carry a card too (`h3-server card`). (3) **The browser's list**: `ListPeers { seq, from }` → `Peers { seq, from, total, entries }`, pages of 50 `RelayedCard { card, via, list }`. Read each card as in (2). Without `list`, check the chain: `card::chain` over the relaying server's card, the cards `via` names in that order (found by key in the same answer) and the card itself; drop a card that fails. With `list`, `Lists { seq }` → `ListsAre { seq, lists }` gives the signed lists (at most 16): `list::read` each, then `list::names(list, card)`. (4) **Trust is the player's**: show each server with its chain (who vouches) or its list, and the card's age (`issued`); merge what several known servers relay, one entry per key, keeping the shortest chain; let the player choose which list keys to trust; contact a server only when the player picks it, so browsing reveals the player's address to no unknown server. A client with an HTTP client may also fetch list files itself (`SignedList` as JSON; ADR-0021 point 5). (5) **Versions**: before 1.0 a server greets only its own protocol version (`Goodbye { Protocol }` otherwise), and a card says its server's version, so the browser can mark a server it cannot join. The server's test client (`crates/h3-server/src/client.rs`: `hello`, `describe`, `lists`, `relayed`) does each check; `crates/h3-server/src/tests/peers.rs` and `tests/lists.rs` show them over loopback.
