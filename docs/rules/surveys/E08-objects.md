# E08 objects survey (E08 T11a): object types and visits, pickups, the Cartographer, observatories, Keymaster's tents, the treasury

Date: 2026-10-12. Scope: E08 increment 2 (decisions 13–18; T12 object types in the import, T13
treasury and pickups, T14 the Cartographer, T15 visitable objects, T16 batch 2), plus what later
increments need from the same code, marked *(inc. 3)* or *(inc. 4)*. Input, not rules
(`README.md`): a fact becomes binding when a task writes it into the catalog with its sources and
tests.

**Sources** (retrieved 2026-10-12):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited here has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the
  human player's path; the computer player's own handling of these objects was not read.
- `vcmi`: VCMI 1.7.5 (commit `6901839a`), cited as `file: function` or the config file;
  links `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `h3api`: H3API (commit `92255ab`, MIT) for names and layouts (`H3GlobalObjectSettings`,
  `H3MapItem`'s access bits, `H3Main::keymasterVisited`): guesses, each checked in the code below.
- `maps`: a census of the 164 maps shipped with the user's installation, read with `h3-map`
  (counts in §1).
- `wiki`: read 2026-10-12 through the summarising fetch tool (the first attempts waited for the
  user's approval and timed out; the user approved later the same day):
  [Resources](https://heroes.thelazy.net/index.php/Resources),
  [Campfire](https://heroes.thelazy.net/index.php/Campfire),
  [Cartographer](https://heroes.thelazy.net/index.php/Cartographer),
  [Redwood Observatory](https://heroes.thelazy.net/index.php/Redwood_Observatory),
  [Pillar of Fire](https://heroes.thelazy.net/index.php/Pillar_of_Fire),
  [Keymaster's Tent](https://heroes.thelazy.net/index.php/Keymaster%27s_Tent),
  [Border Guard](https://heroes.thelazy.net/index.php/Border_Guard),
  [Border Gate](https://heroes.thelazy.net/index.php/Border_Gate),
  [Difficulty level](https://heroes.thelazy.net/index.php/Difficulty_level). Short quotes only;
  re-check a fact against the page before a rule cites it with `high` confidence. The wiki has no
  page on random resources (a search found none) and says nothing about visit modes.

**Second reading:** a fresh agent re-read the deciding code without this survey (2026-10-12)
and confirmed §3.1's ranges, §3.3's setup and visit, §3.1's visit and 0x4E3600's wrap, §2.1's
two class lists and the move's use of the first, and §7.2's tables (`docs/exe/README.md`,
"Confirm before flagging").

Where the reading and VCMI agree, the fact has its consensus (`docs/04-rules-and-sources.md`,
"Consensus before trust"); where they disagree, §11 lists the dispute for the user.

## 1. The objects in the map file

| Class | Subclass | Object | Body (`h3m.md`) | Shipped maps: objects (maps) | Visit mode (§2.1) |
|---|---|---|---|---|---|
| 79 | 0–6 resource | resource pile | message (with guards), u32 amount (0 random), 4 unused | 9 156 (149) | from a neighbour |
| 76 | 0 | random resource | as class 79 | 18 922 (154) | from a neighbour |
| 12 | 0 | campfire | none | 1 572 (142) | from a neighbour |
| 13 | 0 water, 1 land, 2 subterranean | Cartographer | none | 58, 30, 18 (66) | entered, not from above |
| 58 | 0 | Redwood Observatory | none | 288 (95) | entered, not from above |
| 60 | 0 | Pillar of Fire | none | 71 (38) | entered, not from above |
| 10 | 0–7 colour | Keymaster's tent | none | 283 (89) | entered, not from above |
| 9 | 0–7 colour | border guard | none | 219 (74) | from a neighbour |
| 212 | 0–7 colour | border gate (AB, SoD) | none | 73 (33) | passable with the key, else from a neighbour |

Resource codes: 0 wood, 1 mercury, 2 ore, 3 sulfur, 4 crystal, 5 gems, 6 gold (the file's,
the executable's and VCMI's order). Colours: 0 light blue, 1 green, 2 red, 3 dark blue,
4 brown, 5 purple, 6 white, 7 black (VCMI `config/objects/moddables.json`; the tents' colours
are not the players' colours).

**Instance data of the resource bodies** (census): 27 893 piles and random resources have a
random amount (0), 185 a fixed one. A fixed gold amount counts in hundreds: VCMI's reader
multiplies it by 100 (`lib/mapping/MapFormatH3M.cpp: readResource`, `amountMultiplier`), and the
original's visit does the same (§3.1). Shipped fixed amounts of the other resources run from 1 to 30,
gold's from 10 to 1 000 (1 000 to 100 000 gold). 44 bodies carry a message, 39 of them with guards; a guarded pile
starts a battle before the pickup *(inc. 3)*. The original keeps a pile's amount in the low 19
bits of the tile's word (the setup tests `& 0x7FFFF`, 0x4C1AE4) and flags guards in bit 31
(0x4A4BF4); how the loader fits a larger file amount was not read, so T12 bounds the amount at
0x7FFFF (524 287) or rejects it.

**What stays a `Blocker`:** every class not in the table, as in increment 1 (ADV-OBJ-001):
its blocked and visitable tiles block. The classes above become typed; a guarded pile cannot be
typed before battles exist (§11, ADV-Q10).

**The templates' groups** (census): resource and random-resource templates have group 5, every
other object here group 0. VCMI reads groups 2–5 (creature, hero, artifact, resource) as
"visitable from above" (`lib/mapObjects/ObjectTemplate.cpp: isOnVisitableFromTopList`); the
original keys the same property by class (§2.3), so the import needs no group.

## 2. Visits

### 2.1 Entered or visited from a neighbouring tile

The original holds a settings record per object class (232 classes of 0x10 bytes at 0x691698,
pointer 0x660428; H3API's `H3GlobalObjectSettings`), filled at start-up from four class lists
in the executable (0x41B500): "cannot be entered" (0x63AA58), "visitable from above" (0x63AB00),
"removable" (0x63A9D0) and a value per class (0x63A6E4), plus the decoration flag.

- **Cannot be entered** (42 classes; ours: 9 border guard, 12 campfire, 76 random resource, 79
  resource): the move's step toward such an object's visitable tile (`hero_move_step`, the
  default case of the switch at 0x480C12, 0x480F27) does not enter it. The hero stays on its tile,
  pays the step's price as if it had stepped (0x480240: the points drop by the step's price and to
  0 below the cheapest next step, ADV-MOV-006's rule), and the visit runs. VCMI marks the same
  classes `blockVisit` (`CGResource::initObj`, `CGBorderGuard::initObj`, the campfire's
  `blockedVisitable`) and charges the step's price before such a visit
  (`server/CGameHandler.cpp: moveHero`, `tmh.movePoints` before `blockingVisit`; without D-3's
  drop to 0, which the engine already follows, ADV-Q3). Not read: a hero wearing Angel Wings
  (artifact 72) or with hero field +0x112 set takes a further test first (0x4E5CE0, H3API's
  `CanFlyOnTile`) *(inc. 3: flying, boats)*.
- **Entered** (every other class; ours: 10, 13, 58, 60): the hero steps onto the visitable tile;
  the visit runs there and the move ends.
- **The border gate** (212) is entered only by a player who holds its colour's key (the switch's
  case 212, 0x480EE2): without the key it is visited from the neighbouring tile like the classes
  above.

### 2.2 A move ends at a visitable tile

A step onto a visitable tile visits its object (`hero_move_step` keeps the step's object for the
visit, the default case of the switch at 0x481544), and the visit ends the move. Only two
classes let a step that is not the move's last skip the visit (0x48160A): garrisons (33,
*later*) and, for a key holder, the border gate (212). The pathfinder's pass-through test
(0x56AB40, called by `pathfinder_run`; read in its cases 9 and 212 and its default) lets a key
holder's path cross a gate. VCMI: a visit ends a path except at teleporters, garrisons and border
gates (`lib/pathfinder/PathfindingRules.cpp: MovementAfterDestinationRule`, `DESTINATION_VISIT`):
the same for our classes. A path may still end on a visitable tile; it never continues past one.

### 2.3 Not from above

An entered object whose class is not "visitable from above" cannot be entered from the three
tiles above its visitable tile: `pathfinder_expand` skips a step down, down-right or down-left
(directions 3–5 of `step_directions`) onto such a tile (0x4B26B1). A step up, up-left or
up-right *off* such a tile is blocked for a walking hero and open to a flying one (0x4B2682 sets
the "walking blocked" flag that 0x4B29AC tests) *(inc. 3: flying)*. The "visitable from above"
list holds 9, 12, 76, 79 and 212 but not 10, 13, 58 or 60. VCMI keeps a direction mask per
template (`visitableFrom`, "all but top" unless the class is on its list, which has the same
members for our classes) and checks it both ways, entering and leaving
(`lib/mapping/CMap.cpp: canMoveBetween`); for a walking hero the two agree. This extends
ADV-MOV-007 (proposed ADV-OBJ-004).

### 2.4 Removal

A picked-up pile, a campfire and a passed border guard leave the map (0x4AABB0, called by each
visit; not read inside); VCMI `removeObject`. Their blocked tiles become free for everyone; the
removable list (0x63A9D0) holds 9, 12, 76, 79 and 212 (the gate is never removed by a visit).

## 3. Pickups

### 3.1 Resource piles (class 79)

- **Visit** (0x4A4BE0, from the visit dispatch 0x4A84F0's case 79): a guarded pile (bit 31 of
  the tile's word) goes to 0x4A4780 (the guards' question and battle, not read; *inc. 3*).
  Otherwise the owner gains the amount of the subclass's resource, gold × 100 (0x4A4C25), through
  0x4E3600 (§7.1); the pile is removed. The random draw at 0x4A4D18 picks the pickup sound only.
  VCMI `CGResource::onHeroVisit` → `collectRes`: the same.
- **A random amount (0) is drawn at setup**, not at the visit: `new_game_heroes` (the new
  game's setup, 0x4BFE70) calls 0x4C0CC0, which walks every tile (level, row, column) and, for a
  pile whose amount is 0 (0x4C1AE4), draws `random_between(5, 10)` for wood, ore and gold and
  `random_between(3, 6)` for mercury, sulfur, crystal and gems (0x4C1AD1). Gold becomes 500 to
  1 000 in steps of 100 at the visit.
- **VCMI disagrees on the ranges:** `config/objects/moddables.json`, `resource.amounts`:
  wood and ore 6–10, the four rare resources 3–5, gold 5–10 × 100, drawn at setup
  (`ResourceInstanceConstructor::randomizeObject` from `CGResource::initObj`).
- **The wiki agrees with the reading** (Resources: "5-10 units of Wood", the same for ore,
  "3-6 units" of mercury, sulfur and crystal, "500-1000 units of gold" per pile; gems' amount not
  stated). So the reading has its consensus and VCMI is the outlier. Dispute **D-5** (§11) stays
  for the user (a reading that contradicts VCMI); its batch-2 experiment becomes an optional
  confirmation.

### 3.2 Random resources (class 76)

At setup, before the amounts (`new_game_heroes` calls 0x4C9DD0 at 0x4BFFF7, then 0x4C0CC0 at
0x4BFFFE): 0x4C9DD0 walks the tiles in the same order and turns each random resource into a pile
(class 79) of `random_between(0, 6)` (0x4C9EFB): the seven resources with equal chances, gold
included. Its amount is then the file's (gold × 100 at the visit) or, if 0, drawn as in §3.1.
VCMI `CGResource::pickRandomObject`: `nextInt(WOOD, 6)`, the file's amount times the new type's
multiplier, then the random amount as for a pile: the same. After setup a random resource is a
pile like any other, and its picture shows its resource.

### 3.3 Campfires (class 12)

- **Setup** (0x4C0CC0, case 12 at 0x4C0EE5): the amount `random_between(4, 6)` first, then the
  resource `random_between(0, 5)` (wood to gems, no gold), kept in the tile's word.
- **Visit** (0x4A1120): the owner gains 100 × amount gold and the amount of the drawn resource
  (two calls of 0x4E3600: gold first, 0x4A1185), and the campfire is removed.
- VCMI `config/objects/rewardablePickable.json`, `campfire`: the same six resources and pairs
  (4 and 400, 5 and 500, 6 and 600), drawn at setup (`Rewardable::Info::configureRewards`, the
  map dice); its dice give the pairs 33 %, 33 % and 34 % where the original's draw is uniform, a
  difference no experiment can see and no rule needs.
- The wiki (Campfire, Resources): "between 400–600" gold and "4–6 other resources", and the
  campfire "disappears after the first visit"; it does not say whether the two amounts are
  linked or which resources are possible (Resources lists all six non-gold ones). Agreed.

### 3.4 When the draws happen (decision 17)

All of increment 2's random parts are drawn **at setup**: random resources' types, random
amounts, campfires. The original draws them from its one generator in tile order (level, row,
column; where these two walks fall among the setup's other draws, the starting armies' included,
was not read beyond the two calls); the engine draws them from the setup stream in a canonical order of its
own (object index). No visit of increment 2 draws (the sound's draw is presentation), so no
command of increment 2 advances a player's stream, and the ownership-isolation property does not
need to watch the streams yet (decision 17's condition does not arise).

## 4. The Cartographer (class 13)

Read in the visit dispatch's case 13 (0x4A8A14):

- **Once per player and kind:** a player who already bought a kind (water, land or subterranean)
  gets a message and nothing more; the flags are three bytes of player bits in the game (+0x4E372
  by subclass), so every Cartographer of a kind counts as one. VCMI: `visitMode: playerGlobal`
  (`config/objects/cartographer.json`): the same.
- **The price:** with less than 1 000 gold the visitor gets a message and nothing more (player
  record +0xB4 against 0x3E8). Otherwise a yes/no question (a message box of mode 2): yes pays
  1 000 gold (subtracted directly, 0x4A8B60), reveals (below) and sets the flag; no changes
  nothing, and the player may come back and be asked again. VCMI: `canRefuse`, a limiter of
  1 000 gold and `gold: -1000`: the same.
- **The reveal** (0x4CA240): every tile on **both levels** whose terrain is in the kind's mask
  becomes explored for the visitor and every player of its team. The masks are set at a new game
  (0x4CE457, game +0x4E36C): water `0x100` (water), land `0xBF` (dirt, sand, grass, snow,
  swamp, rough, lava) and subterranean `0x40` (subterranean terrain). So the land Cartographer
  also reveals the dirt and lava of the underground, and neither reveals rock. VCMI reveals by
  terrain layers (`lib/rewardable/Interface.cpp: grantRewardBeforeLevelup`, `revealTiles` with
  `surface 1, water -1, rock -1` for land and `subterra 1, surface -1, …` for subterranean, over
  both levels): for the ten base terrains the same three sets. The wiki (Cartographer) agrees by
  terrain, not by level: the land ("tower") one "reveals all tiles, except water and
  subterranean", the water one "all water tiles", the subterranean ("stalagmite") one "all
  subterranean tiles (not to be mixed with the whole underground layer)". It does not mention
  rock, which the reading and VCMI leave hidden. Price: 1 000 gold in SoD (HotA raised it to
  10 000).
- **Entered, not from above** (§2). The water Cartographer stands in the water: a hero reaches it
  only by boat *(water and boats: not in increment 2)*, so its kind is typed but unreachable.
- **The decision:** the yes/no question is a pending decision of the visiting hero's owner
  (decision 15; §10).

## 5. The Redwood Observatory (class 58) and the Pillar of Fire (class 60)

Read in the visit dispatch's cases 58 and 60 (0x4A9992, 0x4A9A12; the two cases are the same but
for their message): every visit (no flag, no cost) calls `reveal_disk` with radius 20 (0x14,
0x4A9AC6) around the visited tile (the visit's position argument, the tile the hero entered) on
its level, for the visitor and its team: `dx² + dy² ≤ 420` (ADV-FOG-002's disk). VCMI
`config/objects/observatory.json`: radius 20 around the hero's sight centre, every terrain,
`visitMode: unlimited`; `getTilesInRange` with `DIST_2D` is the same disk, and the hero stands on
the visited tile. The wiki (Redwood Observatory, Pillar of Fire): "a 20 tile radius around the
observatory", working on either level. Agreed. Entered, not from above. (The wiki adds that
tiles under a Cover of Darkness revealed this way are hidden again the next day: towns, E09.)

## 6. Keymaster's tents (10), border guards (9) and border gates (212)

The keys are eight bytes of player bits in the game, one per colour (+0x4E364; H3API's
`keymasterVisited`).

- **Keymaster's tent** (0x4A8843): a player without the colour's key gets it (its bit is set)
  and a message; with it, a message. No cost, no draw; entered, not from above. VCMI
  `CGKeymasterTent::onHeroVisit` (`visitedObjectsGlobal` by colour): the same.
- **Border guard** (0x4A8734): visited from a neighbour. Without the key: a message, nothing
  else. With the key: a human player is asked a yes/no question (mode 2); yes, or a computer
  player without a question, **removes the guard from the map for everyone** (0x4AABB0); no
  leaves it. VCMI `CGBorderGuard::onHeroVisit` / `blockingDialogAnswered`: a question to every
  player, `removeObject` on yes: the same. The epic's "removed for a player with the key" means
  removed by such a player, for all: other players may then pass. The question is a second
  pending decision (§10).
- **Border gate** (0x4A8DD0, 0x480EE2, 0x56ABB1): never removed. With the key the player's
  heroes enter and cross it like a free tile (§2.2); without it they visit it from the
  neighbouring tile and get a message. VCMI `CGBorderGate::passableFor` (the key) and
  `onHeroVisit` (a message without it): the same. AB and SoD only (class 212; RoE has no gates).
- The wiki agrees on each point: the tent lets the player's heroes pass guards and gates of its
  colour (Keymaster's Tent; eight colours); a guard "cannot be passed until one of any player's
  heroes has visited" the tent, asks a key holder whether to pass, and then "disappears from the
  map": "any hero of any player can pass after that" (Border Guard); a gate "cannot be entered
  or crossed unless one of the player's heroes" has the key and does not disappear, from
  Armageddon's Blade on (Border Gate).

## 7. The treasury and the starting resources

### 7.1 The treasury

- **Seven resources** per player, 32-bit signed, in a player record (game +0x20AD0 + 0x168 ×
  colour; resources at +0x9C, gold +0xB4). VCMI `PlayerState::resources`, `config/resources.json`
  (same order).
- **A gain** goes through 0x4E3600 (the hero's owner): the amount is added with 32-bit wrap,
  and a negative result is set to 0; there is no upper bound. A gain that passes 2³¹ − 1 wraps
  negative and drops the resource to 0: candidate **A-5** (§9). VCMI clamps every change to
  0 … 10⁹ (`GameStatePackVisitor::visitSetResources`, `PLAYER_RESOURCES_CAP`).
- **A cost** is checked before it is paid (the Cartographer: gold ≥ 1 000, then subtracted).
  VCMI's server checks too ("the server is responsible to check if player can afford deal").
- What other players see: nothing of another player's treasury in increment 2 (the Thieves'
  Guild ranks players by wealth: towns, E09).

### 7.2 Starting resources by difficulty (decision 16)

Read in `new_game_heroes` (0x4C01EE): for each playing colour, a human player (record +0xE2)
copies the row of the chosen difficulty (game +0x1F6D8, 0 easy … 4 impossible) from 0x678170, a
computer player from 0x6781FC (and draws its AI type, `random_between(0, 2)`, an AI matter):

| Difficulty | Human: wood, ore / rare / gold | Computer: wood, ore / rare / gold |
|---|---|---|
| 0 easy | 30 / 15 / 30 000 | 5 / 2 / 5 000 |
| 1 normal | 20 / 10 / 20 000 | 10 / 4 / 7 500 |
| 2 hard | 15 / 7 / 15 000 | 15 / 7 / 10 000 |
| 3 expert | 10 / 4 / 10 000 | 15 / 7 / 10 000 |
| 4 impossible | 0 / 0 / 0 | 15 / 7 / 10 000 |

"Rare" is each of mercury, sulfur, crystal and gems. A game flag (game +0x1F69D, read by the
end-turn button and the battle start; probably the tutorial, not read) gives a human 50 of each
and 50 000 gold instead (0x6779C8). VCMI `config/difficulty.json` has the same ten rows
(`human`, `ai`; pawn … king) and applies them by `PlayerState::human`
(`CGameState::initDifficulty`): the same. The difficulty is the one chosen for the game, not the
map's (the header's difficulty byte is the map's rating). Decision 16 stands: the reading gives
no reason to overrule it (every player gets the human row; the computer rows are recorded for
a later human/computer distinction). The wiki (Difficulty level) has the same ten rows. It also
lists a daily bonus for computer players from their weekly income (expert +25 % gold, +39 % wood
and ore, +14 % rare; impossible +50 %, +53 %, +28 %; easy −25 % gold) and a "tournament rules"
start on impossible (5 wood and ore, 2 rare, 2 500 gold); neither was read in the code, the
bonus belongs to the week and to a human/computer distinction *(inc. 4)*, and VCMI has a weekly
bonus too (`RESOURCES_WEEKLY_BONUSES_AI`).

## 8. What each viewer sees and learns (for `docs/06-security.md`)

| Fact | The visiting player | Other players |
|---|---|---|
| A pile's resource (its picture), a random resource's drawn resource, a tent's or guard's colour | public on explored tiles (and the fringe under `ADV-QUIRK-1`) | the same |
| A pile's or campfire's amount, a campfire's resource | learned at the pickup (the gain) | hidden (VCMI's hover shows the name only) |
| A pickup | its treasury's change | the object's removal where they see a tile of it |
| A Cartographer's or tent's flag | its own (VCMI's hover: visited or not, for the viewer) | hidden |
| A reveal (Cartographer, observatory) | its new explored tiles | hidden |
| A border guard's removal, a hero on a gate | — | public where they see the tiles |
| The treasury | its own | hidden |

An object is seen as a whole once any tile it draws on is explored (VCMI
`CGameState::isVisibleFor`: "visible when at least one tile is visible"); the original draws
tile by tile (`ADV-QUIRK-1`). T11b decides what the observation shows (inbox T8b1).

## 9. Quirk candidates: for the user to classify

| Id | Title | Impact | Suggested class | Confidence it exists in SoD | Confirmation |
|---|---|---|---|---|---|
| A-5 | A resource gain past 2³¹ − 1 wraps and drops the resource to 0 (0x4E3600: add, then 0 if negative) | state-corrupting, unreachable in increment 2 (the largest single gain of a shipped map is 100 000 gold) | always fixed: saturate at 2³¹ − 1 (`docs/04-rules-and-sources.md` §4: state-corrupting bugs are fixed; no flag) | read once; VCMI caps at 10⁹ instead | a second look when a rule can reach it (weekly income, increment 4) |

Not quirks, recorded as rules: the land Cartographer's reveal of underground dirt and lava (the
reading and VCMI agree, §4); a declined Cartographer asks again on the next visit (§4).

## 10. Pending decisions (input for T11b)

Increment 2 has two questions a player answers before the game goes on, both yes or no:

| Offer | When | Yes | No |
|---|---|---|---|
| The Cartographer (decision 15) | a visit by a player without the kind and with ≥ 1 000 gold | pay 1 000 gold, reveal the kind, set the flag | nothing (asked again on the next visit) |
| The border guard | a visit by a player with the colour's key | remove the guard for everyone | nothing |

The original asks only human players; its computer players buy nothing here (the computer's
handling of the Cartographer was not read) and always pass a guard. The engine has no
human/computer distinction, so every player is asked (VCMI asks every player too). A decline
leaves the hero where the visit put it; asking again needs a new visit (stepping off and on, or
a "visit the object I stand on" command, which the original has as the space bar; not in
increment 2).

## 11. Disputes and their confirmations

| Id | Question | Sources | Needed by | Settle by |
|---|---|---|---|---|
| D-5 | A random pile's amount: wood and ore 5–10 or 6–10; mercury, sulfur, crystal, gems 3–6 or 3–5 | the reading and the wiki (5–10, 3–6; gold 5–10 × 100 everywhere) against VCMI (6–10, 3–5) | T13 (data) | the user (ADV-Q10; recommended: the reading, which has the wiki's consensus); optional confirmation in batch 2: a map with 30 random wood piles and 30 random mercury piles in a row, picked up one by one with the amounts noted; if the reading is right, at least one wood pile of 5 shows with 99.6 % and one mercury pile of 6 with 99.98 % (either refutes VCMI), and VCMI's ranges would show neither |
| D-6 | Guarded piles (39 bodies in the shipped maps): typed or not in increment 2 | the reading: a guarded pile asks and fights before the pickup (0x4A4780, not read) | T12 | the user (ADV-Q10): a typed `Unsupported` import error until battles exist (recommended), or an inert `Blocker` |
| D-7 | The border guard's question: a pending decision or no question | the reading: humans are asked, computers pass; VCMI asks everyone; the wiki shows the key holder's question | T11b, T15 | the user (ADV-Q10): a second pending decision of the same shape as the Cartographer's (recommended) |

No other point needs a confirmation experiment: the reading and VCMI agree on everything else
(visit modes, directions, campfires, random resources, the Cartographer, observatories, keys,
gates, starting resources), and the wiki agrees wherever it speaks (it is silent on visit modes,
directions and random resources).

## 12. Proposed rule ids (increment 2)

Catalogs: `docs/rules/catalog/adventure.toml` (ADV) and `game.toml` (GAM). Ids are proposals;
the implementing task writes the entries.

| Id | Rule | Task | Open |
|---|---|---|---|
| ADV-OBJ-002 | Object types: the import's classes and subclasses, kinds and parameters (§1); every other class a `Blocker` | T12 | D-6 |
| ADV-OBJ-003 | Visit modes: entered (the move ends there) or from a neighbouring tile (the step's price paid, the hero stays); a path passes no visitable tile (§2.1, §2.2) | T13 | — |
| ADV-OBJ-004 | Not from above: entered objects of the listed classes are not entered from, nor left toward, the three tiles above (§2.3) | T13 | — |
| ADV-OBJ-005 | A removed object frees its tiles (§2.4) | T13 | — |
| ADV-PIK-001 | Resource pile: the gain (gold × 100), removal; a random amount drawn at setup (§3.1) | T12, T13 | D-5 |
| ADV-PIK-002 | Random resource: its resource drawn at setup, seven equal chances (§3.2) | T12 | — |
| ADV-PIK-003 | Campfire: amount 4–6 and one of six resources at setup; 100 × amount gold and the amount of the resource; removal (§3.3) | T12, T13 | — |
| ADV-VIS-001 | Cartographer: once per player and kind, 1 000 gold, the yes/no offer, the reveal by terrain mask on both levels (§4) | T14 | — |
| ADV-VIS-002 | Redwood Observatory and Pillar of Fire: radius 20 around the visited tile, every visit (§5) | T15 | — |
| ADV-VIS-003 | Keymaster's tent: the colour's key for the player (§6) | T15 | — |
| ADV-VIS-004 | Border guard: with the key and a yes, removed for everyone (§6) | T15 | D-7 |
| ADV-VIS-005 | Border gate: passable with the key, else visited from a neighbour (§6) | T15 | — |
| GAM-RES-001 | The treasury: seven resources per player, never negative, gains saturate (§7.1) | T13 | A-5 |
| GAM-RES-002 | Starting resources by the game's difficulty: the human row for every player (§7.2) | T13 | — |
| GAM-DEC-001 | A pending decision: its owner's only legal commands are its answers until answered (§10; decision 15) | T14 | D-7 |
