# Draft: ADR-0017 amended for E09 increment 4 (the Tavern)

- Status: **Draft**, the hire's half written by E09 T33a (2026-11-16), the information's half
  (the Thieves' Guild's view, the Den of Thieves, the views and events, the env) by T33b
  (2026-11-16), **accepted** at its checkpoint (`TWN-Q18`, decisions 114 to 122); then moved into
  `docs/adr/0017-game-orchestration.md` by the increment's `shared/` task with batch TWN-4's
  verdicts.
- Decisions: the epic's 99, 103 to 107 (`TWN-Q16`), 108, 109 to 112 (`TWN-Q17`), 113 (T33a's
  agent's calls: among them **the taverns' stream**, a new named kernel stream, confirmed at
  `TWN-Q18`: decision 115), 114 to 122 (`TWN-Q18`: T-16 and T-17 `fix-optional`, TD-22 and TD-23
  the reading, the views, the env, the cut).
- Working detail: survey `docs/rules/surveys/E09-tavern.md` §2 to §6, §12, §13; design note
  `docs/epics/E09-design.md` Part F §59 to §80.

## Point 1, after E09 increment 3's sentence "**The setup** gains **whether the map is RoE's** …"

**E09's increment 4** (the Tavern; E09 decisions 99, 103 to 113) adds two commands and one
decision kind (point 6). **`TownCommand::Hire { town, slot }`** (ADR-0028 point 6; TWN-TAV-003)
hires the player's tavern hero of that slot in an own town with a standing Tavern and no
visitor, for the part's price, while the player has fewer heroes on the map than the part's
`most_heroes` (8; a garrison hero is not counted, TWN-ROW-002): the hero stands on the town's
entrance as its visitor. **`Manage::Dismiss { hero }`** (GAM-MAN-003; E09 decision 112): the
player's hero on the map leaves the game (point 11), refused for a town's visitor and for the
player's last hero on the map while it owns no town (the original's `hero_screen_refresh`
0x4E1A50, survey `E09-tavern.md` §12); a garrison hero is not on the map and is not dismissed
(the town screen offers no dismissal); the loss condition's "lose this hero" waits for E10's
scenarios (`TWN-STUB-16`). Both are their player's, in its turn, with no decision open
(GAM-DEC-001), each refusal typed and changing nothing.
**The state.** **The pool** (TWN-SET-005 amended) gains a status **offered** (a tavern hero's
type: taken by no colour, out of every draw; the original's 0x40) and, per hero type, **its
hero's id once it has a record** (E09 decision 109). A type's **record** (its hero id and its
details in the store) is made the first time a draw offers the type; a placed or a generated
hero's type has its record from the build. From then on the type keeps its id and its details
whatever happens to it: a hero that leaves the game and is drawn again, by any player, comes
back with its level, experience, skills, spells and artifacts (the original's 156 hero records,
survey §4). The store of heroes grows by one record at each type's first draw (the next id, so in
the order of the draws) and never shrinks. A new record's details are GAM-HER-001's defaults under
the map's **map-wide entry** of its type (SoD's customised hero types; `GameSetup` gains them for
every type, the import reads them, `cross-lane`): the default experience's draw, or a custom
experience's levels at load (GAM-SET-002), then its level seed (GAM-HER-002), each from the
stream of the draw that made it (point 4), and its spell points full. Beside each recorded type
off the map the pool keeps its **away record**, what the adventure keeps of a hero on the map
and hands back when it leaves (ADR-0016 point 3): its points left and the day's maximum, its
formation and tactics flag and, while it is a tavern hero, its army; its battle bonuses and timed
marks are dropped at the leave (E08 decision 307). **Each player** gains **its faction** (the
chosen faction: TWN-SET-001's draw kept; a Random choice no start reads is drawn at the first
fill instead, from the taverns' stream, so the setup stream does not move; none without a choice)
and **its two tavern heroes** (the survey's offers: slot 0 and slot 1, each a hero id or empty,
and per slot the **returning mark**, the original's bit 17 of +0x105: the hero came back by a
retreat or a surrender since its owner's turn last ended). The pool, the away records, the
factions and the slots are one block shared by forks and copied on a write (a fill, a hire, a
leave, a turn's end that clears a mark).
**The draw** (TWN-TAV-001; `tavern_hero_draw` 0x4BB5E0): per hero class of the core, in class
order, its weight the part's tavern weight of the player's faction (ADR-0028 point 5) when the
class has a free type the player's colour may take (the masks), else 0; the other slot's class 0
while another class has a free type; for slot 0 (own faction only) every class not of the
player's faction 0 while one of its faction's weighs; a class by one draw over the total weight,
then one of its free takeable types in type order, uniform. No free takeable type: the slot stays
empty. A player without a faction weighs every class 1 and has no own faction (the agent's call:
only setups without towns lack one). The SoD edition's Conflux rule is not modelled (dead in this
build, E11's lead).
**Increment 4's setup** gains, per hero type, the map's customised entry (above); nothing else:
the masks, the allowed heroes and the prisoners are increment 1's.

## Point 2, after E09 increment 3's sentence "… the hire dialog's view is its owner's, as on the map."

**E09's increment 4** (the Tavern; TWN-TAV-002 to -009): **a player's two tavern heroes are its
own** (as its treasury): its observation gives, by slot, each tavern hero's view (its id and type,
its details, its army by slot, its formation and tactics flag: what the original's tavern dialog
and its view-only hero screen show; no points, no returning mark), whether or not it owns a
Tavern (the agent's call: they tell nothing of another player), and `TavernOffered` tells it a
slot's new view (or its emptying) after every change: a week's release and fill (in player, then
slot order, after the towns' week), a hire's replacement, a hero's return; the game's start's fill
is the first observation's. **A hire** is told as `HeroHired` (the hero, its type, its owner, its
tile; its slot to the owner alone) to the owner and to every player that sees the tile (ADR-0016
point 6), after the price's `ResourcePaid` and before the replacement's `TavernOffered`; the
owner's view of the hero moves from the slot to its heroes, its points by `MovementChanged`. **The
Thieves' Guild** (TWN-TAV-007 to -009; ADR-0028 point 8) is a **query**, not a part of the
observation (the original's window is a snapshot when it opens; an observation's view would need
every command to tell every viewer's changed ranks): `h3_game::thieves_guild(state, rules,
viewer)` answers while the viewer's turn runs and it owns a town with a standing Tavern, or its
open decision is a tavern decision, at its level (its towns with a Tavern, counted to 5; 0 at a
map Tavern without one: two rows and no part); otherwise none. A **Den of Thieves' visit** tells
the visitor's owner the guild at everything (`ThievesGuildSeen`, the view in the event). **The
guild's view:** the players in the game in id order; the rows its level shows (2 rows at 0 or 1, 4
at 2, 6 at 3, 8 at 4, all 9 from 5: towns, heroes, gold, wood and ore, the four rare resources,
obelisks visited, artifacts, army strength, gold income), each as its places, from the highest
value, equal values sharing a place (in player order within it), never a value; per player from
level 1 its best hero's type, from 2 that hero's four primary skills (each 0 to 99), from 4 its
best creature (survey `E09-tavern.md` §6, §13; TD-23 the reading, decision 118: the best hero
by army power, the towns' shown armies in the army row and the best creature). The artifacts row
ranks the backpacks alone under `TWN-QUIRK-13` (T-17, decision 117: `fix-optional`, on in
`faithful`; off, the worn and the backpack's artifacts, VCMI's). The player's tactic (from 3) is the
computer's play and not shown (`TWN-STUB-18`); the obelisks row reads 0 for every player until
increment 6 places the obelisks (`TWN-STUB-17`). A defeated player's column is E10's (every
player is in the game until then). The guild shows nothing a click opens: another player's best
hero and creature open no screen (`thieves_guild_proc` 0x5C9930).

## Point 3, the day's start, after E09 increment 3's week text ("… the original's every hero record, `new_day` 0x4C80CF, 0x4C8D2A).")

**From E09 increment 4** (TWN-TAV-002; GAM-CAL-002 amended) a new week's start, after the towns'
week (the week's kind, the neutral garrisons, the Portals) and before the adventure's week step,
gives **the taverns' week** (the original's `new_day_week_events` 0x4C8780: after the towns'
growth, before the dwellings): every player's two slots, in player order, are **released**
(each type free again, the slot empty) but a slot whose returning mark is set (TD-18, E09
decision 103); then each player in player order **fills** its empty slots, slot 0 then slot 1
(`tavern_new_heroes` 0x4C8450): each the draw of point 1, the type offered, its record made if
it has none, then the type's **full starting army** (GAM-SET-001's draw, replacing whatever army
its away record held; a rolled war machine gives its artifact where a position takes it, as
GAM-HER-001), its backpack's artifacts equipped from the last where a free position takes each
(`hero_equip_artifact` 0x4E2A00) and its spell points full. A player defeated (E10) is not
filled. **The game's start** fills every player the same way (nothing to release) after the
build's last draw, before day 1's start-of-day step (`new_game_heroes`' taverns, 0x4C00C0, after
the main towns' heroes).
**At its owner's turn start** each tavern hero gets its day's maximum and points (TWN-TAV-005;
ADV-MOV-002's derivation over its away record's army, as a hero on the map's), after the
player's heroes on the map and before the garrison heroes (the original's `next_player_turn`);
no spell point is regained off the map (TD-20, E09 decision 104). **At its owner's turn end** its
two slots' returning marks are cleared (`next_player_turn` 0x4C705D, after the victory tests),
before the next group's turn starts.

## Point 4, after E09 increment 3's draws ("… The towns' stream is server-only (point 5) and hashed with the state.")

**E09's increment 4** draws from **the taverns' stream** (`StreamId::TAVERNS`, a new named kernel
stream beside `TOWNS`, a `shared/` change; E09 decisions 113 and 115: no existing draw moves,
so the replays' outcomes keep but for the new state) at the game's start's fill and at
every week's fill, per filled slot in this order: the class, the type, a new record's draws (its
experience or its levels at load, then its level seed), then its army (the first count, then each
later slot's roll and, when given, its count; the original's `draw_starting_army` 0x4C9730 order).
A Random faction choice that no start read is drawn there at the player's first fill, before its
slot 0. **A hire's replacement** (TWN-TAV-004) draws from **the hirer's stream** after the hire
is checked (a refused hire draws nothing): the class, the type, a new record's draws; its army is
one creature of the type's first starting stack, without a draw. **A hero's return** after a
retreat or a surrender (TWN-TAV-006) draws its slot, one draw of two (the original's
`random_between(0, 1)`, then the other slot when that one holds a returning hero), from **the
stream of the player whose hero returns**: its own command (the flight, the surrender) ends the
battle. A loss, a draw, a capture's leave and a dismissal draw nothing. The map Tavern's hire
draws as the town's.

## Point 6, pending decisions, after E09 increment 3's "**The town's two rows open no decision** …"

**E09 increment 4** adds **the tavern decision** (GAM-DEC-010; E09 decisions 106, 111): a visit
of the adventure map's Tavern (class 95, ADR-0016 point 4) opens it for the visitor's owner
(`map_tavern_visit` 0x5D7E90); its answers are `Hire { slot }` and `Close`. A hire is checked as
the town's (the slot's tavern hero, `most_heroes`, the price), without a visitor test, and is
refused while a hero stands on the Tavern's visited tile (VCMI's; the original never reaches it,
since a hero on the tile hides the Tavern from a step, ADR-0016 point 3); a refused answer is
typed and leaves the decision open. A hire places the hero on the Tavern's visited tile, draws
the slot's replacement and closes the decision (the original's dialog returns after one hire,
`map_tavern_hire` 0x4D7890); no guild spells. Its view (`DecisionView::Tavern`) is the visitor
and the Tavern by type and anchor, told whole by `TavernOpened` to the visitor's owner; the offers
it hires from are that owner's tavern heroes (point 2), so no view of it waits on a later event.
The town's tavern opens no decision (the town command, point 1), as the town's two rows.

## Point 10, two new sentences

A visit's outcome may name **a Den of Thieves** (ADR-0016 point 4): the game composes the
visitor's owner's Thieves' Guild at everything after the visit's other effects and tells it
(point 2); nothing is paid or changed.

A hire's application (the town's or the map Tavern's) is infallible once checked: the price
paid, the hero placed by the adventure with its away record, its type taken by the player's
colour (the colour's bit added to the type's mask, `hero_place` 0x4D7900), the slot's returning
mark cleared, the replacement drawn into the slot. A town's hire would teach the town's guild
spells (`town_heroes_learn_spells` 0x5BE030); the guild is increment 5's (`TWN-STUB-15`).

## Point 11, after E09 increment 3's sentence ("… an empty garrison hero yields the town without a battle (T-5, `TWN-QUIRK-11`).")

**From E09 increment 4** a hero that leaves the map **leaves the game** (TWN-TAV-006; GAM-BAT-003
amended; the original's `hero_leave_game` 0x4D9EC0): after a battle (lost, retreated,
surrendered, a draw, a winner with created stacks only), a garrison hero without a stack at its
town's capture (T-5) and a dismissal. The adventure hands back its away record (ADR-0016 point
3); its type goes free and its details stay in the store with its level, skills, spells and the
artifacts it keeps (a beaten hero those the winner found no room for, GAM-BAT-004; a retreated or
surrendered hero all of them, the battle's capture returning at once; a dismissed hero all of
them, E09 decision 105). **After a retreat or a surrender** (the battle's result for its side)
its hero goes at once into its owner's tavern: a slot drawn (point 4), the other when that slot
holds a returning hero; that slot's tavern hero, if any, goes free; the type offered, the
returning mark set. A surrendered hero keeps its army (its side's survivors), a retreated hero
keeps one creature of its type's first starting stack; both keep their points left and the day's
maximum and their spell points as they were. **Every other leave** frees the type: the spell
points full (HER-MAN-001's maximum) and the points left and the day's maximum the land maximum,
which the original computes after clearing the army (`hero_max_land_movement` 0x4E4990 from the
empty slots' start, speed 20: the table's most) under `TWN-QUIRK-12` (T-16, decision 116:
`fix-optional`, on in `faithful`; off, the land maximum of the one creature it keeps), the
type free for any player's later draw, which gives it a new army. A new record's points are the
land maximum of the army its first draw gives it (the original sets every record's at the new
game with its full starting army; the agent's call). `ADV-STUB-9`'s retreat facts stay E10's whole
(only a computer's hero reads them; their Tavern fact is `has_tavern` over the owner's towns from
increment 4 on).

## Point 8, after E09 increment 3's entries ("… so the rows' stack moves are a client's.")

**E09's increment 4** appends (`cross-lane`, the AI lane's paths, E09 decision 7) **the town's
hire**: two entries, `HIRE + s` for the tavern hero of slot s, with the town by its entrance in
the tile head (one action per `Hire` the legal commands list: a standing Tavern, no visitor, the
heroes on the map below `most_heroes`, the price in the treasury; encoding v12). **The tavern
decision** takes the answers by role (encoding v13, or with v12 when they land together): its
`Hire { slot }` of slot 0 and 1 the first and second answers, its `Close` the closing role, each
legal as the legal commands list it. **The dismissal has no entry** (a management command, as the
army commands: E08 decision 99), nor has the Thieves' Guild (a query, no command): the AI lane may
read the query for its agents.

## Point 9, after E09's rows

**E09 increment 4** bumps the game state hash (the pool's statuses, ids and away records, the
players' factions and tavern heroes, the taverns' stream), the adventure ruleset part and state
hash (the kind `Tavern`, ADR-0016), the map import (class 95, the map-wide entries in the setup)
and the adventure replay format (`town hire`, `answer tavern`, `manage dismiss`), each
`cross-lane`; the adventure ruleset part and the map import again for the Den of Thieves (class
97, with the Tavern's task); the protocol (the server lane's, `cross-lane`: `TownCommand::Hire`,
`Manage::Dismiss`, the tavern decision and its answers, the observation's tavern heroes,
`TavernOffered`, `HeroHired`, `TavernOpened`, `ThievesGuildSeen`, the guild's request and answer)
and the env's action encoding (point 8: v12, v13; the AI lane's, `cross-lane`). Shared:
`StreamId::TAVERNS` (decision 115) and the quirk flags of `TWN-QUIRK-12` and `-13`. The numbers are the next at merge, in order with E08's increment 13.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 4's `shared/` task; decisions 99, 103 to 114 and `TWN-Q18`'s):
  points 1 to 4, 6, 8 to 11 (the hire, the dismissal, the pool's records and tavern heroes, the
  taverns' week and stream, the tavern decision, a hero that leaves the game; the tavern heroes'
  view and events, the Thieves' Guild's query and a Den's view, the env's hire and answers).
