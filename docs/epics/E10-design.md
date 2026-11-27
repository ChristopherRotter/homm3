# E10 design note: increment 1 (the teams and the game's end)

Written by T1b (2026-11-27; E10 decisions 4, 5, 7 and the checkpoint's 9 to 19). The code cites it
as "increment 1's design note §M". Inputs: the survey `docs/rules/surveys/E10-teams.md` ("survey
§N" below), ADR-0016, ADR-0017, ADR-0028 (point 6, the town commands). Drafts:
`docs/lanes/adventure/adr-drafts/` `0016-e10-increment-1.md` (points 1, 4, 5, 6),
`0017-e10-increment-1.md` (points 1, 2, 3, 6, 8, 9) and `0028-e10-increment-1.md` (point 6: an
ally's town, `cross-lane`, decision 17). ADR-0015 needs none (no crate, no dependency).

**The checkpoint's answers** (ADV-Q65, the user at T1b's start; decisions 9 to 18): T1b whole (9);
the build teams first, then the end, then the hand-over and the hand-overs, the batch and the audit
(10); the check at the original's moments (11, D-80); the warnings' boxes as the original's, A-55
and A-56 accepted (12); no check at the first turn's start, A-57 accepted (13); the original's end
once no human is left, the result naming the team left when the players left are one team (14,
D-84); a removed player's colour handed to the computer player, as the original's network drop,
and increment 1 builds the engine's half (15, D-83; D-81 moot); the allies' gifts give-only into
empty places (16, D-82); an ally's town built by this lane, `cross-lane` (17); batch GAM-1 before
the build's last round (18). After the note: decision 19 (§20).

## Part A — the teams

### 1. The setup and the import (GAM-TEM-001; map import v30)

- `PlayerSetup` gains `team: u8`. `h3_map::import` sets it from `Teams::of_player[colour]` when
  the map has teams (`RawMap::teams`), else the player's index (every colour its own team, E10
  decision 2). The census (survey §1): every map's codes are dense below its `count` over the
  playable colours, so a team is an index `0..teams` either way.
- `GameState::new` checks the teams (*agent's call*): every team below the player count and every
  team below the highest one used held by a player, else `GameSetupError::Team` (a setup the
  increment cannot represent, as the other checks). No team changes during play: no diplomacy
  (decision 2; survey §6, no code writes the bytes).
- The writer and `h3-experiment` keep their `teams` field (E08 T89a); the import's version moves
  because `GameSetup` now carries what it dropped (map import v30).

### 2. The state: the team in the game and the adventure (`allied` by state)

- `h3-game`'s `PlayerState` holds the team (copied from the setup, hashed with the player).
- `h3-adventure` holds the teams by player in its **static part** (ADR-0016 point 1: never
  changed, shared by forks, hashed once). `h3_adventure::allied(a, b)` (`state/economy.rs`, "the
  player alone until E10") becomes `AdventureState::allied(&self, a, b)`: same team, a player with
  itself included (`players_same_team` 0x5296D0). Every one of the 22 sites (survey §7) passes the
  state; the game's sites (`garrisons.rs`, `town/view.rs`, `computer/start.rs`) call it through
  the adventure state they hold. The adventure's own "same player" tests that the original makes
  team tests move too: the meeting (`state.rs`' `Meets::Hero`'s owner test, `jumps.rs`' exit
  holding an own hero; §5) and the teleporters' exits (`teleport_exit_draw` 0x4CDB80, survey §6).
- The 72 places that name E10 as the place teams or the end arrive (survey §7; `git grep -n E10
  crates`) are swept by the task that builds their rule: each comment rewritten or its code
  changed (T3a the teams', T4a the defeat's); none left naming E10 for increment 1's facts.
- Measured at T3a's first build (`homm3-bench-regression`): `allied` sits in the step and the
  garrisons' passage (the reach); a table read per call instead of an equality.

### 3. The fog per team (GAM-TEM-002, amends ADV-FOG-003; adventure state hash v31)

Every reveal goes to the player and every player of its team, the Cover of Darkness hides from
every team but the owner's (survey §6, "The fog"). With fixed teams the members' explored layers
are always equal, so the adventure keeps **one explored layer per team** (`explored: Vec<BitGrid>`
by team, not by player; *agent's call*): a reveal writes it once, a viewer reads its team's. Every
reveal ADR-0016 point 5 lists (a step's, a placement's, a flag's, a town's, a jump's, the door's,
the Cartographer's, `strict-fixed`'s new-day reveal, the failed casts' tiles) is its team's; the
fringe is the team's. A defeated player's layer is its team's and stays.

### 4. What allies share on the map (the rules that name the team keep their rules)

The rules that already say "the team" (ADV-VIS-006, ADV-BON-009, ADV-OWN-001, ADV-DWL-004,
ADV-GAR-003, -004, ADV-SHY-003, ADV-MIN-002, ADV-BNK-003, ADV-TEL-003; survey §12) lose their
"the player alone until E10" and are re-tested with allies on batch E10's map (E08 T89a's
`teams`): an ally's mine, Lighthouse, dwelling, Shipyard or garrison is not flagged; a mine's
Leave Guards is the owner's alone; an ally's dwelling opens the hire dialog **read-only**, `Close`
alone (batch E10's E10-6; ADR-0017 point 6 already says it, built now: `dwellings.rs`' "no
read-only flag yet"); an ally's Shipyard offers its boat; a garrison of the team is exchanged and
passed short of the destination without a visit (`step_garrison_pass` 0x48154B); a visit's marks
(the tiles' visited bits, the kinds, the banks', the obelisks') are the team's; a monolith's exit
holding a hero of the team is no candidate. **Not shared** (survey §6): treasuries, the border
gates' keys, the turn, a town's recruitment and buildings.

### 5. The meeting of an ally's hero (GAM-TEM-003, amends GAM-MET-001, GAM-DEC-003; decision 16)

- A step toward a hero of the team meets it (`visit_hero` 0x4A2800); an enemy's is a battle as
  before (GAM-BAT-004), in a town the town's visit (TWN-VIS-002). The adventure's outcome names the
  `Meeting` as for an own hero; the Scholar's teaching runs as between one's own (HER-SPL-007: both
  heroes learn, `meeting_scholar` 0x4A2940).
- The game opens the meeting as the **visitor's owner's** decision (GAM-DEC-001; `MeetingOpened`
  to it alone). While it is open, the management commands act on the visitor's own hero as today;
  onto the ally's hero only **gifts**: a stack moved into an **empty slot** of the ally's army
  (`swap_army_move` 0x5B0DA0: no merge, no swap; the visitor's last stack stays, ADV-ARM-002), an
  artifact moved into an **empty place** or the backpack (`swap_worn_click` 0x5AF590,
  `swap_backpack_click` 0x5AF990). Nothing is taken from the ally's hero and none of its own
  settings changes (formation, flag, assembly, a split on its army): each such command is a typed
  refusal (`GameCommandError::AllyHero`, *agent's call* for the name), changing nothing.
- The ally is told its hero's changes as its own (`StackChanged`, `HeroChanged` to the owner), not
  the meeting. The system actor's default answer stays `Close`.

### 6. The views for the team (GAM-TEM-004, amends ADV-FOG-006, ADV-GAR-006, the town's view)

A viewer sees a hero of its team at the popup's top level (`hero_popup` 0x416590, 0x4166BB: level
2, the exact army, as Visions 2), a town of its team at level 3 (`town_popup_level` 0x416E63) and
a garrison of its team at level 3 (`garrison_popup` 0x416F80); a mine of its team names its
resource (`mine_hint_text` 0x40D670). The owner's own facts stay the owner's (a hero's details,
a town's buildings outside a visit, the treasury). Each `PlayerView` (public) gains its `team`.

### 7. An ally's town (the town lane's code, `cross-lane`; decision 17, `TWN-STUB-21`)

The town modules (`src/town*` of `h3-adventure`, `h3-game`, `h3-map`; `h3-towns`) are the town
lane's (`lanes.toml`); the user consented to this lane building them in `cross-lane` pull
requests (decision 17), the town lane's inbox told at each.

- **Entering:** a step onto a gate of a town of the team is no siege and enters it (TWN-VIS-001
  reads `allied` already); its guild teaches the visitor its spells as an own town's (0x5BE030;
  `guild.rs`' "an ally's visitor of the town still takes what the owner's opening gives"), and its
  buildings are told to the visitor's player at that visit (ADR-0016 point 6, built for allies now).
- **The trades** (`TWN-STUB-21` lifted): the window's hero at an ally's town is the visitor, its
  player the actor: the Marketplace's window, the Artifact Merchants and the Freelancer's Guild at
  the **current player's** count (TD-29, batch TWN-6's TWN-6-2: an ally's town counts the visitor's
  player's Marketplaces, `TWN-QUIRK-17`'s at least 1), the University, the Skeleton Transformer and
  the Blacksmith (E09 T59) as at an own town. **Building and recruiting stay the owner's** (TD-11:
  the build dialog's owner test 0x5D5AE2; the hire sells nothing to an ally's visitor): typed
  refusals as today (`NotOwner`).
- **The siege of an ally's visitor** (ADR-0016 point 4: "a typed refusal, a stub until E10"):
  another team's step onto the gate of a town whose visitor is the owner's ally. Its original rule
  (is the garrison merged into the ally's army, TD-7, and whose side is it) is **read at T3c's
  start** (`town_visit` 0x4AAFD0 with `town_defended` 0x5BDFE0; survey `E09-towns.md` §7.2, §9.2)
  and checked by batch GAM-1's row 8 (§19).
- **Town Portal** lists the towns of the hero's team (survey §6): written into `ADV-STUB-20`'s
  reason by T3a for E09's increment 7, which builds the spell.

## Part B — the game's end

### 8. The state of a player and of the game (game state hash v41)

- `PlayerState` gains `defeated: Option<Defeat>` (`Defeat { day: u32, why: DefeatReason }`;
  `DefeatReason::{NoHeroNoTown, DaysOut}`) and `days_without_town: Option<u8>` (the original's
  +0x3D: `None` for its −1, else 0 to 7). The `human` flag it already holds can now change (§14).
- `GameState` gains `result: Option<GameResult>` (`GameResult { winner: Option<u8>, why:
  EndReason }`, the winner a team; `EndReason::{Conquest, NoHumanLeft}`; increment 2 adds the
  conditions' reasons). *Agent's call* for every name.
- In the observation: each `PlayerView` its `team`, `defeated` and `human` (public: the kingdom
  panel's flags, survey §6); its `days_without_town` its owner's only; `GameObservation.result`
  public.

### 9. The check and its moments (GAM-END-006; decisions 11, 13)

`GameState::check_end` (*agent's call* for the name) does survey §2's eliminations in colour order,
then §12's verdict. It is idempotent and runs (survey §4.1):

1. **after every command and system command** that the game accepted, except `EndTurn` (a
   player's or the system's: its own two points stand for it): the original's 37 calls follow everything that can change a hero's or
   a town's count or an owner, and a check that finds nothing changes nothing;
2. **at a turn's end**, after `TurnEnded` and before that player's count-down (`next_player_turn`
   0x4C7019, 0x4C7133);
3. **at a human player's turn start** (`player_turn_start` 0x4CC831; `PlayerState::human`), after
   the turn start's steps; a computer's turn start runs none; **the game's first turn runs none**
   (A-57 accepted, `ADV-QUIRK-61`: `play_run` 0x4EFC58; building the state checks nothing).

A days' defeat therefore lands at the first check after the seventh turn's end: the next player's
turn end, a human's turn start, or (the engine's residual, §21) the next player's first command,
where the original waits for its first checked action. The same seven turns (D-80).

### 10. Elimination and the seven days (GAM-END-001, -002)

At each check, for every player not defeated, by its heroes (on the map, a town's visitor or
garrison hero; a prisoner is nobody's) and its towns (survey §2's table):

| Heroes | Towns | Days | The check |
|---|---|---|---|
| any | ≥ 1 | any | the days cleared (`None`) |
| 0 | 0 | any | **defeated**, `NoHeroNoTown` (§11) |
| ≥ 1 | 0 | `None` | the days set to 7: `LastTownLost` (§15) |
| ≥ 1 | 0 | 0 | **defeated**, `DaysOut` (§11) |
| ≥ 1 | 0 | 1 to 7 | nothing |

**The count-down** at each turn's end of the outgoing player (point 2 of §9, after its check):
its days lowered by one when above 0 (every player's turn, a computer's too). **At its turn start**
a player with days tells `DaysLeft { player, days }` to itself (ArrayTxt 128, at 1 ArrayTxt 129;
the box is a human's, the event every player's). So: a start without a town sets 7 at the first
End Turn and shows 6 to 1 at the next turn starts; a town lost in an enemy's turn sets 7 at that
capture's check and shows 7 to 1; seven own turns either way (survey §2's timeline; `game-week`).

### 11. A defeated player (GAM-END-003; survey §3, `player_defeated` 0x4F11A0)

In the original's order, at the check: its `defeated` set (the day, the reason); its
**Shipyards, mines and Lighthouses, dwellings and garrisons neutral** (each told as
`ObjectFreed { object_type, anchor }` to the players that see it and to the old owner, *agent's
call*; a mine's guards and a garrison's army kept; ADV-OWN-004, the dwellings' town counts
following); its **heroes into the pool from the last of its list** by E09's leave
(`town::leave::leave` with `Leave::Gone`: off the map, a town's slot cleared, a boat freed
(`BoatRemoved`), one creature of the first starting stack, spell points full, artifacts kept;
`HeroRemoved` to the players that saw it); its **two tavern offers freed** (a retreated or
surrendered hero still in them goes to the pool; the original's step 1). Not touched: its
treasury, its marks, its artifacts on the map. Then:

- **Its turn**, when it is the one acting (its last hero lost in its own battle): ended at once,
  `TurnEnded` told, the group's turn going on with the others (ADR-0017 point 3, "or left the game";
  *agent's call*, batch GAM-1's row 7 checks it).
- **It leaves every loop over players** (the original's readers of the defeated bytes, game
  +0x1F636): the turn order (`next_player_turn` 0x4C70BB), the day's income (`player_daily_income`
  0x4B8B03; GAM-RES-003 to -005), the week's tavern heroes (`new_day_week_events` 0x4C8A85,
  `tavern_new_heroes`), the Thieves' Guild's rows (`thieves_guild_fill` 0x5DDDB9), the market's
  recipients (`market_window_run` 0x5EA315; ADV-MKT-004's "while nobody leaves it");
  increment 2's conditions' tests. The computer's (`ai_*`) are E31's.
- Its commands: it never acts again (`NotYourTurn`); a system command naming it is refused
  (`GameCommandError::Defeated`).

### 12. The verdict (GAM-END-004, -005; decision 14)

After the eliminations, with `left` the players not defeated (survey §4.2, the viewing human's
team read for every human at once):

1. **One team left** and a human among `left`: the game is over, `Conquest`, that team the winner
   (GENRLTXT 660 for its humans; a defeated ally is of the winning team, the clients decide what
   it shows);
2. **No human among `left`**: over, `NoHumanLeft`, the winner the one team left if `left` is one
   team (computers only: the human's screen is the original's "lost" either way), else none;
3. otherwise the game goes on.

A game **built with no human player** never ends by rule 2 (the original cannot start one): it
plays to one team (*agent's call*, for E31's computer games and an env that marks no seat).
Increment 2's special conditions come before rule 1 (the original's order) and its loss is then
overwritten by a conquest in the same check (survey §4.2).

### 13. After the end (GAM-END-007; survey §4.3)

The check that ends the game sets `result`, ends the acting players' turns (no `TurnEnded`) and
**drops every open decision** (a battle's aftermath's level-up, the capture's town: the original
shows no town screen after a capture's check; *agent's call*), then tells `GameOver { result }` to
every player. The command that ended it keeps its events up to the check. From then **every
command and system command is refused** (`GameCommandError::GameOver`, nothing changed),
`legal_commands` is empty and no day or turn advances.

### 14. The hand-over (GAM-END-008; decision 15)

`SystemCommand::HandOver { player }` (the host's alone, ADR-0017 point 2; SRV-Q29's "removing a
player"): the player's `human` cleared, `PlayerHandedOver { player }` told to every player
(GENRLTXT 470's chat line, survey §5), then the check (so the last human handed over ends the game,
§12 rule 2). Its turn, its decisions and its pieces stay as they are: the host plays the colour
from then on with its system commands (point 2's `EndTurn`, `Answer`, `Battle`) and, with E31,
the original's computer player; the battles read the cleared flag (BAT-MOR-004's human side).
Refused, changing nothing: a defeated player (`Defeated`), one already a computer
(`GameCommandError::NotHuman`, *agent's call*). No player command removes a player (the original
has no resignation; the server maps a resign or a dropped seat to it).

## Part C — events, views, the wire, the env

### 15. Events and projections (decision 12: A-55, A-56 accepted, `ADV-QUIRK-59`, `-60`)

| Event | Told to | The clients' box (the original's) |
|---|---|---|
| `LastTownLost { player }` | the player | GENRLTXT 7 only when the player is the one acting (A-55) |
| `DaysLeft { player, days }` | the player, at its turn start | ArrayTxt 128 (`days`), 129 at 1 |
| `PlayerDefeated { player, why }` | every player | 96 to itself when acting, else 6 (`NoHeroNoTown`); 8 to itself when acting, else 9 (`DaysOut`): in practice 9, its own name to itself (A-56) |
| `ObjectFreed { object_type, anchor }` | the players that see it, the old owner | none |
| `PlayerHandedOver { player }` | every player | GENRLTXT 470 |
| `GameOver { result }` | every player | GENRLTXT 660 to a winning human; the end screens (E27 M5) |

The engine tells the facts whenever they happen; A-55 and A-56 are the clients' rule (the shell's
and the map client's), accepted as the original's boxes: `ADV-QUIRK-59` (A-55) and `-60` (A-56)
record them with the box each client shows. A days' reset is no event (the town's capture tells
it; the owner's observation shows the days).

### 16. The wire, the env, the server, the clients

- **Wire** (`h3-protocol`, the server lane's; the global trap "engine types on the wire",
  `cross-lane` at each task): the new events, `SystemCommand::HandOver`, the errors `AllyHero`,
  `Defeated`, `GameOver`, `NotHuman`, the observation's fields; protocol v86 at the first.
- **The env** (`h3-sim`, the AI lane's, `cross-lane`): `AdventureEnv`'s step reports the end
  (`result`) and each player's defeat; a defeated or ended seat is never `to_move`; the action
  encoding stays v21 (the hand-over is the host's, no player entry). The reward is E13's.
- **The server** (`h3-server`, its lane): a hosted whole game ends at `GameOver` ("never ends
  before E10" lifted there), a seat that leaves is handed over (when is the server's call), its
  turns ended by the host's clock until E31.
- **The clients:** the end screens (E27 M5: the result, each colour's defeat, why and the day),
  `MCL-STUB-4` (the map client's seven days, E24) lifted by §15's events.

### 17. Replays, maps, properties, benchmarks

- **Replays** (the adventure replay format v28): the word `hand-over <player>`, the game's end
  recorded and checked at the replay's end; golden replays of the three shipped
  maps that start Red without a town (Adventures of Jared Haret, Knee Deep in the Dead, Pirates!:
  defeated after seven own turns), an allied map's meeting and shared fog, a conquest and a
  no-human end; the existing ones rehashed (`homm3-rerecord-replays`).
- **Maps** (`h3-experiment`'s writer): an allied hot seat with the six rows' layouts (§19).
- **Properties:** no command is accepted after `GameOver`; a defeated player is in no loop and
  owns nothing; the team's members see the same tiles; the check is idempotent (a second call
  changes nothing); the random play's oracles (`play.rs`) know the new events.
- **Benchmarks:** the random-play workloads at their baselines (the `allied` read, §2; the check
  after every command, §9), a game played to its end baselined.

### 18. Versions, rule ids, quirks, stubs

- **Versions:** map import v30 (§1), adventure state hash v31 (§2, §3), game state hash v41 (§8),
  adventure replay format v28 (§17); protocol v86 (the server lane's). The adventure ruleset part,
  the core ruleset hash and the quirk flags stay (no data, no flag).
- **Rule ids** (survey §12): `GAM-TEM-001` to `-004`, `GAM-END-001` to `-008`; amended: ADV-FOG-003,
  -006, ADV-GAR-006, GAM-MET-001, GAM-DEC-003, ADV-OWN-004 (built), the town lane's trades' rules
  (TWN-STUB-21's) and the siege's (TWN-VIS-002) by T3c.
- **Quirks:** `ADV-QUIRK-59` (A-55), `-60` (A-56), `-61` (A-57), accepted, no flag; written by the
  task that builds their events (T4a).
- **Stubs:** `TWN-STUB-21` and `MCL-STUB-4` lifted; `AI-STUB-18` (the computer's ally values) stays
  E31's with the teams readable; `ADV-STUB-20` gains the team list (§7). No new stub.

## Part D — batch GAM-1 and the cut

### 19. Batch GAM-1 (survey §10, amended; decision 18)

Hot seat, Easy, on maps written by `h3-experiment` (`teams` since E08 T89a), two runs each, on the
client lane's capture harness; run **before the build's last round** (decision 18), so the
build's rounds 3 and 4 start from the readings and round 6 folds its outcome in. Rows GAM-1-1 to
-6 as survey §10, and two added:

| Row | What | Predicted | Settles |
|---|---|---|---|
| GAM-1-7 | Red and Blue humans on two teams, Tan computer: Red, without a town, loses its last hero attacking Tan's in its own turn | GENRLTXT 96, then the turn passes to Blue at once (or End Turn alone is offered) | §11's turn |
| GAM-1-8 | Red and Blue allied humans, Tan computer: Blue's hero visits Red's town (with a garrison); Tan's hero steps onto the gate | as T3c's reading (§7): the defending army and side | §7's siege |

Rows 1 to 3 and 5 to 7 need colours started without a town or a hero: T5a checks the writer's
fields and asks the battle lane's consent for any it adds. A verdict against a reading moves its rule in round 6 (T6), before the audit.

### 20. Shared and `cross-lane` work; the building tasks; the exit criteria

**Decision 19** (the user after the note): the cut, the drafts and the agent's calls as written
(*filled in at the acceptance*).

**Shared:** T2 (`shared/`, `cross-lane`): the three drafts applied as accepted (ADR-0016 points 1,
4 to 6; ADR-0017 points 1 to 3, 6, 8, 9; ADR-0028 point 6); `docs/03-domain.md`'s terms (team, an
ally, defeated, the seven days, the game's result, a hand-over); `docs/06-security.md`'s lines (the
hand-over the host's alone, every command refused after the end, a player's days its own); the
audit's `shared/` task (T7) amends them as built with batch GAM-1's outcome. **`cross-lane`
elsewhere:** the town modules (T3a's `allied` sites in `town.rs`, `town/view.rs`; T3c whole;
decision 17), `h3-protocol` at each wire task, `h3-sim` (T6), `h3-experiment` (T5a, if a field is
needed; its consent asked there). **The inboxes:** town (decision 17, T3c's scope), server (§16,
the hand-over, protocol v86), AI (the terminal, the `human` marks and §12's rule 2, E31 plays a
handed-over colour), client (E27 M5's facts, A-55 and A-56's boxes), map client (`MCL-STUB-4`'s
events).

**The building tasks**, in rounds of one or two (each task's cut confirmed at its start):

| Round | Tasks | Content |
|---|---|---|
| 2 | **T2** | `shared/` (`cross-lane`): the drafts applied, the glossary, `docs/06-security.md` |
| 3 | **T3** | T3a: the teams in the setup and the state, `allied` by state at every site, the fog per team, §4's rules re-tested with allies, the E10 marks of the teams (§1 to §4; map import v30, adventure state hash v31), measured at once. T3b: the allies' meeting and the views (§5, §6). T3c (`cross-lane`): an ally's town (§7), the siege's reading first |
| 4 | **T4** | T4a: the defeat (§8 to §11: the check's moments, elimination, the seven days, a defeated player's steps and the loops; game state hash v41; ADV-QUIRK-59 to -61). T4b: the verdict and after the end (§12, §13), the events and the wire (§15, §16; protocol v86) |
| 5 | **T5** | Batch GAM-1: T5a its maps (§19), T5b the run on the harness, scored (needs the PC) |
| 6 | **T6** | The hand-over (§14), the env's terminal (`cross-lane`), the replay format v28, the maps, replays, properties and benchmarks (§17); batch GAM-1's outcome folded in |
| 7 | **T7** | The audit against the exit criteria; the ADR amendments as built (`shared/`); increment 1 closed; increment 2's survey queued |

*Agent's call:* the teams first (decision 10), the hot-path change measured in the first task; the
town part last in its round (another lane's files; its reading first); the end's state and check
before its events and wire, so protocol v86 moves once; the hand-over with the env and the replays,
so the replay format moves once.

**Increment 1's exit criteria:**
1. The 160 shipped maps import with their teams (51 with, 109 each colour its own); `allied` reads
   them at every site; no team changes during play (GAM-TEM-001).
2. Every reveal and hide is the team's; the members of a team see the same tiles (GAM-TEM-002).
3. An ally's hero is met with give-only gifts; an ally's ownables are not flagged, a garrison of
   the team passed, an ally's dwelling read-only; the views at the team's level; an ally's town
   entered, taught, traded at and never besieged, built and recruited by its owner alone
   (GAM-TEM-003, -004, the town lane's rules; §4 to §7).
4. A player with no hero and no town is defeated at the check; one with heroes and no town after
   seven of its own turns, a town back clearing the days; the three shipped maps without a starting
   town replayed to their seventh turn (GAM-END-001, -002).
5. A defeated player's ownables are neutral with their guards and armies, its heroes in the pool
   from the last with one creature and their artifacts, its tavern offers free; it is in no turn
   and no loop over players (GAM-END-003).
6. The game ends by conquest when one team with a human is left, and when no human is left, the
   winner the team left if one; checked at the original's moments, none at the first turn's start
   (GAM-END-004 to -006).
7. After the end every command is refused, changing nothing, and no decision is open (GAM-END-007).
8. The hand-over is the host's alone and makes the colour a computer for the battles and the
   verdict (GAM-END-008).
9. A viewer sees and is told what §8 and §15 say; the wire, the env's terminal and the replays as
   §16, §17.
10. Every new rule has its id, source and tests; the gates pass (coverage, determinism with the new
    and rehashed replays, the benchmarks at their baselines); batch GAM-1 run and scored (T5), its
    rows' rules `verified`.

### 21. Risks and open points

- **Elimination touches every loop over players** (the epic's risk): §11's list is the original's
  readers of the defeated bytes; a loop the engine has and the original's list lacks (the env's
  seats, the server's) is found by the property "a defeated player is in no loop" and the sweep of
  the E10 marks.
- **The check after every command** (§9) can defeat a player out of days at the next player's
  first command, where the original waits for a checked action: the same turn and the same seven
  days; the events' order differs only then. The audit looks at it on the replays.
- **The team in the hot path** (§2): a regression above the gate is settled at T3a.
- **The town part is another lane's code** (decision 17): T3c's pull request may meet the town
  lane's own work in the same files; it merges `master` first and tells the town lane's inbox at its
  start.
- **No human left** reads `PlayerSetup::human`: an env or a server that marks its seats wrongly
  ends a game early (or, with no human at the build, never by that rule); the AI and server lanes'
  inboxes say so.
- **Not read:** the network's turn and messages (0x691209), the cheats' win and loss, the
  computer's reads of the teams (E31), the scenario's conditions (increment 2).
