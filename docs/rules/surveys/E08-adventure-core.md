# E08 adventure-core survey (E08 T1b): movement, fog of war, the day, simultaneous turns

Date: 2026-10-09. Scope: E08 increment 1 (T4 content, T6 the day, T7 movement, T8 fog), plus
what increment 3 needs from the same rules, marked *(inc. 3)*. Input, not rules (`README.md`):
a fact becomes binding when a task writes it into the catalog with its sources and tests.

**Sources** (retrieved 2026-10-09):
- `original-data`: tables of the user's `Heroes3.exe` already in the address map
  (`docs/exe/map.toml`): `skill_scouting` (0x63E9D8) and `faction_native_terrain` (0x643698).
- `vcmi`: VCMI 1.7.5 (commit `6901839a`), cited as `file: function`; links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: [Movement](https://heroes.thelazy.net/index.php/Movement) and
  [Scouting](https://heroes.thelazy.net/index.php/Scouting), read as raw wikitext through the
  summarising fetch tool. Quotes came back in pieces; re-check every wiki fact against the page
  before a rule cites it with `high` confidence.
- `community`: the HD mod's
  [simultaneous turns page](https://www.sites.google.com/site/heroes3hd/eng/simultaneous-turns-st),
  the [h3hota.com rules](https://h3hota.com/en/rules), and the wiki's
  [Credits](https://heroes.thelazy.net/index.php/Credits) (baratorch: "HD Mod Compatibility &
  Simultaneous Turns" for HotA).

No code reading was done for this survey. Section 6 lists the readings that would settle each
dispute; the function names come from H3API (`docs/exe/h3api.toml`). They are guesses that show
where to read, not facts.

## 1. Movement

### 1.1 Movement points of a day

| Slowest speed in the army | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11+ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| wiki | 1300 | 1360 | 1430 | 1500 | 1560 | 1630 | 1700 | 1760 | 1830 | 1900 | 1960 | 2000 |
| vcmi (`config/gameConfig.json`, `movementPointsLand`) | 1500 | 1500 | 1500 | 1500 | 1560 | 1630 | 1700 | 1760 | 1830 | 1900 | 1960 | 2000 |

- The two tables differ only below speed 3. No creature that can stand in an army has speed
  below 3 in our packs (only the war machines have speed 0), so the difference never shows in
  RoE, AB or SoD content. The table is data (`adventure.toml`). **Settled in E08 T4 (D-4):**
  the original reads the table from `movement.txt` (H3bitmap.lod; H3ab_bmp.lod holds the same
  file): column 2, speeds 0–20, the wiki's row with 2000 from speed 11. `0x4D7240` fills
  `0x698A98` from it; `0x4E4990` (land) takes the lowest speed of the seven slots, starting from
  20, so faster armies get 2000; the creature of a creature specialty and its upgrade count one
  faster there *(inc. 3)*; Logistics then multiplies the points by 1 + its rate *(inc. 3)*.
  At sea the points come from the same file's column 4 by Navigation level (1500, 2250, 3000,
  3750; `0x698AEC`).
- **Which speed counts:** the slowest creature in the army (wiki, vcmi
  `CGHeroInstance::getLowestCreatureSpeed`). The wiki says artifacts that raise creature speed
  (Necklace of Swiftness) do not count but creature specialties do *(inc. 3)*. VCMI reads each
  stack's speed with its bonuses and marks the point `TODO`.
- **When:** the wiki says "Base movement is calculated only at the beginning of each day". VCMI
  computes it once at the change of day for every player's heroes, from the army they have then
  (`server/processors/NewTurnProcessor.cpp: updateHeroesMovementPoints`). That is the known
  trick: give a hero fast creatures before the day ends, take the slow ones back after it
  starts. Whether the original computes it at the start of the day or at the start of each
  player's turn shows only when another player's actions change a hero's army in between:
  dispute **D-1**. **Read in E08 T6b (CR-2's callers):** the original sets them at the start
  of the owner's *turn*. The End Turn handler `0x409A70` (after its confirmation box) calls the
  next-player step `0x4C6FE0`; that steps the current player `0x69CCA8` on, calls the new day
  `0x4C7FE0` when the day's last player has played, and then sets current (+0x49) and maximum
  (+0x4D) points to `0x4E4D90`'s value for the new current player's heroes on the map, its two
  tavern heroes and the heroes in its towns. The new day sets no movement itself, nor do the
  functions it calls or theirs. `0x4F4C00` does the same for the network game's hand-over (a
  branch the reading policy leaves aside); a placed hero gets its points at setup
  (`0x486590`). Not read: the computer player's own turn loop beyond the call into `0x4C6FE0`.
  The wiki and VCMI disagree, so this is a conflict of sources: the user chose the reading until
  a confirmation experiment settles it (ADV-Q4, ADV-MOV-002).
- **No carry-over:** unused points "cannot be stored or preserved over to next day" (wiki). VCMI
  sets every hero to its limit at the change of day.
- On water a hero always has 1500 (wiki, vcmi `movementPointsSea`); water is not in
  increment 1. Logistics, Boots of Speed, Stables and the other bonuses are *(inc. 3)*: the
  adventure profile.

### 1.2 The cost of a step

| Terrain | Dirt | Sand | Grass | Snow | Swamp | Rough | Subterranean | Lava | Water | Rock |
|---|---|---|---|---|---|---|---|---|---|---|
| Straight cost | 100 | 150 | 100 | 150 | 175 | 125 | 100 | 100 | 100 (boat) | impassable |

| Road | Dirt | Gravel | Cobblestone |
|---|---|---|---|
| Straight cost | 75 | 65 | 50 |

- Wiki and VCMI (`config/terrains.json`, `config/roads.json`) agree on both tables. VCMI's
  terrain indices are the `.h3m` terrain codes 0–9 in that order. VCMI ignores rivers when it
  prices a step; the wiki says nothing about them.
- **Which tile prices the step.** The wiki's list: "Terrain ⇄ Terrain move — starting tile;
  Road ⇄ Terrain move — terrain tile; Road ⇄ Road move — starting tile". VCMI
  (`lib/pathfinder/CPathfinder.cpp: getTileMovementCost`) uses the starting tile's road if both
  tiles have a road, otherwise the starting tile's terrain. They differ when a hero steps off a
  road. The wiki uses the destination's terrain, VCMI the terrain under the road. Example: from
  a road on grass onto swamp, 175 by the wiki, 100 by VCMI. Dispute **D-2**.
- **Native terrain.** If every creature in the army is native to the terrain, the penalty is
  removed and the step costs 100 (wiki "all the creatures in hero's army are native"; vcmi
  `CGHeroInstance::getNativeTerrain`: one native terrain shared by all stacks, otherwise none).
  Neutral creatures are native nowhere (vcmi `config/factions/neutral.json`), so one neutral
  stack removes the bonus. A faction's native terrain is original data (`faction_native_terrain`)
  and already in the packs. Nomads remove the sand penalty for the whole army; that is a creature
  ability, *(inc. 3)*.
- **Diagonal steps** cost the straight cost × √2, rounded down. The wiki's table lists 141, 176,
  212, 247 for terrain and 106, 91, 70 for roads. VCMI computes `static_cast<int>(cost * M_SQRT2)`
  and gets the same numbers. In integers the rule is exactly `isqrt(2 · cost²)`, so no float is
  needed (ADR-0003).

### 1.3 Too few points: the last step and the diagonal exception

- A hero cannot move when it has "no movement points or too few of them to move to the next
  square" (wiki). VCMI agrees: a path step it cannot pay for moves to the next day.
- **The diagonal exception** (wiki, "one notable, complex and buggy exception"): a hero with
  fewer points than a diagonal step costs, but at least what a straight step costs, may make the
  diagonal step as its last move, and it uses all of the hero's remaining points. VCMI implements
  a clean version (`CPathfinderHelper::getMovementCost`): it compares with the straight cost of
  the same step, roads and native terrain included. The wiki describes two bugs in the
  original's check: quirk candidates **A-2** and **A-3**.
- **VCMI's last-step rule** (`getMovementCost`, `checkLast`): when the points left after a step
  are fewer than that step's straight cost, the step takes all remaining points. The wiki has no
  such rule. It matters when a cheaper step could follow, for example sand onto a road with 200
  points: 50 points are left, enough for a cobblestone step by the wiki, none by VCMI. Dispute
  **D-3**.

### 1.4 Where a hero may step

- One of the eight neighbouring tiles on the same level, inside the map; never rock; water
  only by boat (not in increment 1); never a blocked tile of an object. Visiting: ADR draft
  "adventure state", point 4.
- **Only explored tiles.** VCMI's pathfinder treats an unexplored tile as blocked
  (`lib/pathfinder/PathfinderUtil.h: evaluateAccessibility`). The original appears to offer no
  path into the shroud; the user's look at a map (A-1, question 4) confirms it. Then the legal
  moves never depend on hidden tiles, and the reachable-tiles query and the action mask leak
  nothing.
- Tiles are revealed after each step, so a long move can open new ground. A path is still
  validated against the tiles explored when the command is given (the ADR draft: validated step
  by step, rejected whole).

## 2. Fog of war

- **Radius:** 5 without Scouting; 6, 7 and 8 with Basic, Advanced and Expert Scouting
  (`original-data`: `skill_scouting`, i32 5, 6, 7, 8; VCMI `baseScoutingRange` 5; the wiki's
  SoD values +1, +2, +3; HotA uses +1, +3, +5). Artifacts add +1 each (Speculum, Spyglass), and
  the Scouting specialty adds +1 every six levels: *(inc. 3)*.
- **Shape:** VCMI reveals a tile when `round(√(dx² + dy²)) ≤ r` on the same level
  (`lib/callback/CGameInfoCallback.cpp: getTilesInRange`, `DIST_2D`). In integers that is
  `dx² + dy² ≤ r² + r` (r = 5: up to 30). The original's shape is not read: dispute **F-1**.
- **When:** at the start of the game, around every object a player owns (vcmi
  `CGameState::initFogOfWar`), and after each step of a move around the hero's new tile (vcmi
  `CGameHandler::moveHero`). Increment 1 models no owned objects, so a game opens with only its
  heroes' surroundings explored. Towns and mines add theirs once they are owned (increment 2 or
  later): a recorded simplification, not a rule.
- **Explored stays explored, and shows live content.** VCMI keeps one explored bit per tile and
  team and shows the current content of every explored tile. Tiles are hidden again only by
  darkness effects such as the Necropolis' Cover of Darkness (vcmi `CGameHandler`, new day:
  `darknessSelector`). This matches the ADR draft's model (point 5). Allies share the map in
  VCMI (`TeamState::fogOfWarMap`) *(E10, teams)*.
- **The fringe:** candidate **A-1**, section 4.

## 3. The day

- **Order:** players act one at a time in colour order, red first (the order of the players in
  the `.h3m` file; vcmi `TurnOrderProcessor` keeps the awaiting players in colour order, but
  `mustActBefore` lets human players act before computer players, a VCMI choice).
- **Day advance:** after the last player's turn, once for everyone (vcmi
  `TurnOrderProcessor::doStartNewDay` → `NewTurnProcessor::onNewTurn`). The day counter
  advances and movement points are reset (1.1). Later increments add mana, income, and on a new
  week growth and the week's creature. A week has 7 days and a month 4 weeks (vcmi
  `GENERAL_DAYS_PER_WEEK`, `GENERAL_WEEKS_PER_MONTH`, as parameters).
- **Hooks (ADR-0017 point 3):** D-1 moved the movement points to the start of each player's
  turn (E08 T6b, the reading above), so in increment 1 the start of a turn has one rule
  (ADV-MOV-002) and the start of a day none.
- **Humans first (read in E08 T6b):** `0x4C6FE0` walks the colours once for the players whose
  human flag (player +0xE2) is set and once for the others, and calls the new day after the
  second walk: the original, like VCMI, plays every human before every computer each day. The
  engine has no human/computer distinction (an inbox item for the turn-group work).

### 3.1 The starting army (GAM-SET-001; read in E08 T6b)

A hero the map gives no stacks gets its type's starting army. `0x4C9730` (called with its
one-creature flag clear by `0x486590`, which fills a hero from the map's record, when the hero
has no creatures afterwards) clears the seven slots, then:

1. slot 0: the hero table's first type (`0x679DD0` + 0x24) and a count drawn between the first
   bounds (+0x44, +0x48, filled from `HOTRAITS.TXT`);
2. a roll of 1–100; at most 88 gives the second type (+0x28): the Ballista (146) and the First
   Aid Tent (147) become the artifacts 4 and 6 and take no slot, any other creature takes slot 1
   with a count from +0x4C, +0x50;
3. a *new* roll of 1–100; at most 25 gives the third type (+0x2C) in the next free slot (1 when
   the second gave no stack) with a count from +0x54, +0x58.

With the flag set (callers `0x4C86A0`, `0x4D9EC0`, not read) only one creature of the first
stack comes. A campaign case swaps hero 45's record for hero 96's. The nine heroes with a war
machine have it in the second of their three slots (the table; `installation.md` §7.1 says the
third, a slip). Sources: the wiki's hero template has 88 % and 25 % for SoD and 100 % for the
war machine of those nine heroes; VCMI (`CGHeroInstance::initArmy`, `startingStackChances
[100, 88, 25]`) uses *one* roll for every slot, so its third stack never comes without the
second, and gives the war machine at 88 %. Conflicts of sources (ADV-Q6): the rolls'
independence (reading against VCMI; the wiki is silent) and the war machine's chance (reading
and VCMI against the wiki and the user's understanding). The engine draws the stacks as the
reading has them (the user, 2026-10-10); war machines are not on the map hero before
increment 3.

## 4. Quirk candidates: for the user to classify

Classes as in `docs/04-rules-and-sources.md` §4. A candidate becomes an `ADV-QUIRK-<n>` entry
in `docs/rules/quirks-adventure.md` once it is confirmed for SoD. Until then the engine does the
clean behaviour.

| Id | Title | Impact | Suggested class | Confidence it exists in SoD | Confirmation |
|---|---|---|---|---|---|
| A-1 | The fog's fringe shows hidden tiles' terrain and the object sprites drawn into them | information (what a player sees) | fix-optional (faithful shows, strict-fixed shows terrain, roads and rivers only; E08 decision 7) | medium (the user's earlier observation; VCMI's pathfinder note: "Objects are usually visible on FoW border anyway") | the user looks at a map (below) |
| A-2 | The diagonal exception ignores roads in its check but not in the cost | gameplay-altering (small): an extra cheap road step at the end of a day | fix-optional | medium (wiki Movement) | code reading CR-1 and a second look |
| A-3 | The diagonal exception "forgets" that a native army has no terrain penalty: it compares with the penalised straight cost | gameplay-altering (small): native swamp and snow armies lose the last diagonal | fix-optional | medium (wiki Movement) | code reading CR-1 and a second look |

**Checking A-1 (for the user).** In a game (SoD), move a hero next to the edge of the shroud
near an object you have not seen yet, such as a mine, a creature or a tree. Look at the
half-dark border tiles. Answer three questions:
1. Can you see terrain there?
2. Can you see parts of an object whose tile is still dark?
3. Does a creature standing one tile beyond the edge show at all?
4. Can you order a hero onto a dark tile (does the cursor offer a path)?

A screenshot is not needed; the four answers are enough. Q30's rule then makes the candidate
`ADV-QUIRK-1` with the class you choose.

## 5. Simultaneous turns (captured for a later decision, E08 decision 8)

| | HD mod (baratorch; HotA's online play uses it, wiki Credits) | VCMI 1.7.5 |
|---|---|---|
| Who | two players in a multiplayer game | any players allowed by the options; not two on one connection |
| Until | a day set by the host ("till what day (including)"; ladder templates use e.g. 115, h3hota.com) | at least `requiredTurns` days, at most `optionalTurns` days |
| Ends early | "if both players interacted with the same objects" (checked after both turns), or "if the players start interacting with each other" | at the start of each day, for every pair "in contact": a tile reachable this turn by a hero of one player (guards ignored) or an owned object's entrance is also reachable by the other (`TurnOrderProcessor::playersInContact`); allies may be exempt (`ignoreAlliedContacts`) |
| On a conflict | the blue player's day is replayed: "the last blue player's turn is cancelled and it goes back to the last red player's turn"; an attack is fought as in sequential play | no conflict can happen: contact is predicted conservatively at the start of the day, and nothing is rolled back |

For the commutation target (ADR draft "game orchestration", point 7): HD detects conflicts
after the fact and rolls back a whole day. That fits a client with a full copy of the game, not
our server-authoritative model with per-command validation. VCMI's conservative prediction at
the start of the day needs no rollback and is deterministic. With our per-player streams it
comes close to the commutation target, but shared state (the tavern pool, global counters)
still has to be partitioned. The agent's recommendation when the increment comes: VCMI's
contact rule, with HD's day limits as parameters.

## 6. Disputes and the readings that settle them

| Id | Question | Sources | Needed by | Settle by |
|---|---|---|---|---|
| D-1 | Movement points: at the start of the day for all players, or at the start of each player's turn | wiki and VCMI say start of day; the original's call sites (read in T6b) say start of the owner's turn | T6/T7 (which hook) | read in T6b (§1.1); the user chose the reading until an experiment confirms it (ADV-Q4) |
| D-2 | Stepping off a road: the destination's terrain or the terrain under the road | wiki vs VCMI | T7 | CR-1 |
| D-3 | Does a step that leaves fewer points than its straight cost take all the points | VCMI only | T7 | CR-1 |
| D-4 | Points for speeds 0–2 | wiki vs VCMI; no army creature is that slow | T4 (data only) | settled in T4: `movement.txt` has the wiki's row (§1.1) |
| F-1 | Shape of the revealed area | VCMI only | T8 | CR-3 |

Code readings (Q57: preferred to experiments when a reading can settle the point). Each is
queued in the task that needs it, and each quirk then needs a second look:
- **CR-1** the step cost: H3API `H3Hero::GetMovementCost` (0x4B18C0); D-2, D-3, A-2, A-3.
- **CR-2** the day's points: H3API `H3Hero::CalcMaxMovement` (0x4E4D90) and `MaxLandMovement`
  (0x4E4990) with their callers; the speed table, which speed counts *(inc. 3)*, D-1, D-4.
- **CR-3** the reveal: H3API `H3Hero::GetScoutingRadius` (0x4E4070) and its callers; F-1.

## 7. Proposed rule ids (increment 1)

Catalogs: `docs/rules/catalog/adventure.toml` (ADV) and `game.toml` (GAME; to adventure in T2).
Ids are proposals; the implementing task writes the entries.

| Id | Rule | Task | Open |
|---|---|---|---|
| ADV-MOV-001 | A hero's movement points for a day come from the slowest creature's speed (table as data) | T4, T7 | D-4 |
| ADV-MOV-002 | Points are set at the start of each day; unused points are lost | T6, T7 | D-1 |
| ADV-MOV-003 | The straight cost of a step: terrain and road costs (data), and which tile prices the step | T4, T7 | D-2 |
| ADV-MOV-004 | An army native to the terrain steps without its penalty | T7 | — |
| ADV-MOV-005 | A diagonal step costs the straight cost × √2, rounded down | T7 | — |
| ADV-MOV-006 | A step needs its full cost; the diagonal exception | T7 | D-3, A-2, A-3 |
| ADV-MOV-007 | Steps go to one of eight neighbours on the same level, never onto rock, water (no boat) or a blocked tile | T7 | — |
| ADV-MOV-008 | A path uses explored tiles only | T7 | — |
| ADV-FOG-001 | The scouting radius of a hero (5; Scouting by level, data) | T4, T8 | — |
| ADV-FOG-002 | The shape of a revealed area | T8 | F-1 |
| ADV-FOG-003 | Tiles are revealed at the start of a game and after each step of a move | T8 | — |
| ADV-FOG-004 | Explored tiles stay explored and show their current content | T8 | — |
| ADV-FOG-005 | What a player sees on the fringe | T8 | A-1 |
| GAME-TRN-001 | Players act one at a time in colour order | T6 | — |
| GAME-TRN-002 | The day advances after the last player's turn; start-of-day steps run once for everyone | T6 | D-1 |
| GAME-CAL-001 | The calendar: days, weeks of 7 days, months of 4 weeks (parameters) | T6 | — |
