# Draft: ADR-0017 amended for E09 increment 5 (the mage guild)

- Status: **Draft**, written by E09 T43 (2026-11-20), **accepted** at its checkpoint (`TWN-Q33`,
  decisions 178 to 181); then moved into `docs/adr/0017-game-orchestration.md` by the increment's `shared/`
  task with batch TWN-5's verdicts.
- Decisions: the epic's 162, 163, 165 to 171 (`TWN-Q31`: TD-10 and TD-25 the original, T-18
  `fix-optional`, TD-27 the original's triggers with an opening command), 172 to 176 (`TWN-Q32`:
  T43 whole, the weights in the town records, the Aurora now, `Open` listed where it changes
  something, `BuySpellBook`), 177 (T43's agent's calls), 178 to 181 (`TWN-Q33`: the opening
  teaches both heroes whenever a guild stands, **the towns' stream** for the draw over the agent's
  new stream, the views and events, the cut).
- Working detail: survey `docs/rules/surveys/E09-guild.md` §3 to §6, §11 to §15; design note
  `docs/epics/E09-design.md` Part G §82 to §100.

## Point 1, after E09's increment 4 paragraph ("**E09's increment 4** (the Tavern; …")

**E09's increment 5** (the mage guild; E09 decisions 162, 167 to 177) adds two town commands
(ADR-0028 point 6). **`TownCommand::Open { town }`** (TWN-GLD-006; TD-27, E09 decision 170)
stands for the original's town screen opened from the town list and its guild's window looked at:
in an own town, in its turn, with no decision open, it gives the Mana Vortex (TWN-GLD-009) to the
town's garrison hero, then to its visitor, then teaches the visitor, then the garrison hero
(TWN-GLD-004); refused only for an unknown town or another's (`UnknownTown`, `NotOwner`), changing
nothing; listed by `legal_commands` only where it would change something (E09 decision 175).
**`TownCommand::BuySpellBook { town }`** (TWN-GLD-007; E09 decision 176) buys the core's spell
book for the town's visitor, else its garrison hero, in an own town with a standing Mage Guild, for
the part's price (500 gold), while that hero wears none; then the town's heroes learn. Both are
their player's, in its turn, with no decision open (GAM-DEC-001), each refusal typed and changing
nothing.
**The state.** Each town's record (ADR-0028 point 2) gains its guild's twenty slots, drawn when the
game is built (TWN-GLD-002, the towns' stream, point 4), its Mana Vortex's week flag and, static,
its Aurora set; the store of towns shares and hashes them as the rest (the town state hash v4).
**The setup** gives each town its map's **required** and **banned** spells (`TownSetup`, the
import: TWN-SET-012; an RoE map's required set empty, its banned set read), each the ruleset's
(`GameSetupError`'s spell refusal, after the scenario's bans); the scenario's banned spells
(GAM-SET-003) are read by the draw and, with `TWN-QUIRK-14`'s flag off, by the Aurora set, both at
the build: the game keeps no ban after it.
**The build** draws every town's guild from the towns' stream after the towns' records exist (each
faction resolved, its buildings set; the original's `town_setup` 0x5C08B0 after the buildings), in
`TownId` order; then the main towns' heroes learn (TD-10, E09 decision 167: the visitor, then the
garrison hero; the original's `main_town_heroes` 0x4CA1A2), after every draw of the build.

## Point 2, after E09's increment 4 paragraph ("**E09's increment 4** (the Tavern; TWN-TAV-002 to -009) …")

**E09's increment 5** (TWN-VEW-004 amended): **a town's guild is its owner's** (the original
shows it in the owner's town screen only; `E09-guild.md` §6): the owner's town view gains, per
level 1 to 5, the spells the guild shows (TWN-GLD-003: none above the guild level; the drawn
spells but never the unshown slots), whether a building of role `aurora` teaches every spell there
(derivable, given for the client's scroll), and the Mana Vortex's week flag (the owner knows
whether it took the Vortex this week). No other viewer sees any of it, not even Visions' top level.
A build that changes what the guild shows (a level, the Library), a capture and the Vortex's
cleared flag are told as the town view's change (`TownChanged`, TWN-VEW-001's events), after the
command's own events; every week's set flag in the week's views. **Learning** is told as
`HeroChanged { change: LearnSpell(spell) }` per spell, in the order learned, to the hero's owner
(as a meeting's, E08 T51b); **the Vortex** as `HeroChanged { change: SetSpellPoints(points) }` to
the hero's owner, then the town's `TownChanged`; the **night's refill** as the day's regain is
(HER-MAN-002: no event, the points in the owner's observation). The book's purchase is told as
`ResourcePaid`, then `HeroChanged { change: AddArtifact(book) }`, then the learning's.

## Point 3, after E09's increment 4 paragraph on the taverns' week

**From E09 increment 5** (TWN-GLD-008, -009; GAM-CAL-002 amended) **every week's first day** sets
every town's Mana Vortex flag with the towns' week (the original's `new_day_week_events`
0x4C8A09), and **every new day**, after every hero's daily regain (HER-MAN-002), raises each town's
visitor's, then garrison hero's spell points to their maximum (HER-MAN-001), never lowering them,
for each town in `TownId` order with a building of role `guild` ever built (the original's `new_day`
0x4C82E4: no spell book, no owner test; `E09-guild.md` §14). Day 1 has no new day, so no refill.
**No turn start teaches or gives the Vortex** (TD-27, E09 decision 170): a hero that gains Wisdom
or a spell book while it stands in a town learns at the next trigger (point 10).

## Point 4, after E09's increment 4 paragraph on the taverns' stream

**E09's increment 5** draws **the guilds from the towns' stream** (`StreamId::TOWNS`; E09
decision 179, the user's over a new named stream: no `shared/` change, but every later draw of the
towns' stream moves, so the replays that play a week re-record their outcomes): when the game is
built, before any other draw of that stream, every town in `TownId` order, its levels 1 to 5, each
slot not given to a required spell and with a total weight above 0 one draw over the total (the
original's `random_between(1, total)` and its running sums). Nothing in play draws a guild. The
original's one generator interleaves these draws with the rest of its setup's; the engine's order
is its own (E08 decision 17).

## Point 8, after E09's increment 4 paragraph

**E09's increment 5** appends (`cross-lane`, the AI lane's paths, E09 decision 7) **the town's
opening** (one entry, the town by its entrance in the tile head; one action per `Open` the legal
commands list, so only where it would change something: encoding the next at merge) and **the
spell book** (one entry, the same head; listed where the town's guild stands, its hero lacks a book
and the treasury holds the price; the next version, or the same when both land together). The env
reads the guild through the owner's town view; no learning is an action.

## Point 9, after E09's increment 4 sentence

**E09's increment 5** bumps the game state hash (the store's v4 folded in), the map import (the
towns' required and banned spells into `TownSetup`) and the adventure replay format (`town open`,
`town book`), each `cross-lane`; the replays that play a week re-record their outcomes (the guild's
draw moves the towns' stream, decision 179); the protocol (the server lane's, `cross-lane`: the two
commands and the new `TownCommandError`s, the owner's guild, Aurora mark and week flag in the town
view) and the env's action encoding (point 8, the AI lane's, `cross-lane`). The adventure state
hash and ruleset part do not move (`AdventureOutcome`'s town entered is not state, ADR-0016 point
4). Shared: the quirk flag of `TWN-QUIRK-14` (no version moves: the profiles' ruleset hashes do);
no new stream (decision 179). The numbers are the next at merge, in merge order with E08's
increment 14 (its game state hash v35, replay format v21 and encoding v14 planned) and the client
lane's replay format v20 (E27 M4 T3a).

## Point 10, after E09's increment 4 paragraph on a hire's application

**E09's increment 5: the town's heroes learn** (TWN-GLD-005; the original's nine callers of
`town_heroes_learn_spells` 0x5BE030, `E09-guild.md` §12; TD-27 the original's) at each of these, as
the town's part returns it (ADR-0028 point 2: the visitor first, then the garrison hero, each its
`LearnSpell`s), in one infallible application with the command:
- **a visit's end**: a step that enters a town of the team, a `Revisit` there and an undefended
  town's capture (the outcome's town entered, ADR-0016 point 4), after the capture's own effects:
  first the opening's Vortex (the garrison hero, then the visitor; the original's town screen
  opened by the visit, `town_screen_init` 0x5C63C0), then the learning;
- **a siege's end** (whoever won; not a battle against a visitor beside a garrison hero, which is
  no siege): after the battle's outcome and a capture, the town's visitor and garrison hero as they
  then stand (E09 decision 171; the attacker stays outside, ADR-0016 point 4), no Vortex;
- **a build**, the command's and a town event's (TWN-BLD-005, TWN-EVT-003): after `TownBuilt` and
  its views, the learning, then the Vortex (the garrison hero, then the visitor; the original's
  `town_build_effects` 0x5BEDE0 learns before the building's visit);
- **a town's hire** (TWN-TAV-003; lifts `TWN-STUB-15`): after the placement and its events, the
  learning (the hired visitor, then the garrison hero), then the Vortex to the hired hero
  (`town_tavern_click` 0x5D846C); the map Tavern's hire teaches nothing;
- **a move between the town's bars** (TWN-ROW-002 to -004): after the move, while a visitor stands,
  the learning (the original's bars rebuilt, `town_screen_bars` 0x5C7007);
- **the opening** (`Open`) and **the book's purchase** (`BuySpellBook`), point 1;
- **the start** (TD-10; lifts `TWN-STUB-6`), point 1.
Town Portal's and Castle Gate's arrivals (increment 7) and the computer's re-visits and book
purchases (E10, E13) are not built here. **The Vortex** (TWN-GLD-009) is the town's part's check
per hero in the order named, its flag cleared at the first that takes it. **The book's purchase**:
the price paid, the book given (`HeroChange::AddArtifact` of the core's spell book, worn at its
position), then the learning; no refusal after the checks.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 5's `shared/` task): points 1, 2, 3, 4, 8, 9, 10 amended for E09
  increment 5, the mage guild: `TownCommand::Open` and `BuySpellBook`, the guild in the store and
  its draw at the build from the towns' stream, the setup's town spells, the owner's guild view,
  the night's refill and the Vortex's week, the env's two entries, the versions, the learning's
  triggers.
