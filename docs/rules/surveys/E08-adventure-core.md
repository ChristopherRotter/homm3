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
  `0x4C7FE0` when the day's last player has played, and then sets maximum (+0x49) and current
  (+0x4D) points to `0x4E4D90`'s value for the new current player's heroes on the map, its two
  tavern heroes and the heroes in its towns. The new day sets no movement itself, nor do the
  functions it calls or theirs. `0x4F4C00` does the same for the network game's hand-over (a
  branch the reading policy leaves aside); a placed hero gets its points at setup
  (`0x486590`). Not read: the computer player's own turn loop beyond the call into `0x4C6FE0`.
  The wiki and VCMI disagree, so this is a conflict of sources: the user chose the reading until
  a confirmation experiment settles it (ADV-Q4, ADV-MOV-002). Settled by batch 1a (E08 T10c): a
  hero that lost its slow stack in the other player's turn has its fast army's points at its
  owner's turn start, on day 1 and day 2.
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
  ability, *(inc. 3)*. **Read in E08 T7b** (0x44C590, called by 0x4B18C0; confirmed by an
  independent second reading): the original walks the army's seven slots in order and takes
  each creature's faction's native terrain (0x643698); a neutral creature reads the entry before
  the table, -1, which is also the "none yet" marker. So neutral stacks *before* the first
  faction stack are skipped, the first faction stack sets the terrain, and any later stack that
  differs, a neutral one included, makes it none: candidate **A-4**, against the wiki's "all the
  creatures" and VCMI 1.7.5 (any neutral stack cancels it). Confirmed and classified at once
  (the user, 2026-10-11): `ADV-QUIRK-4`, fix-optional. The four elementals count as neutral
  there when a game flag is clear (RoE content); the packs already make them neutral in `base`
  and Conflux in `ab`.
- **Diagonal steps** cost the straight cost × √2, rounded down. The wiki's table lists 141, 176,
  212, 247 for terrain and 106, 91, 70 for roads. VCMI computes `static_cast<int>(cost * M_SQRT2)`
  and gets the same numbers. In integers the rule is exactly `isqrt(2 · cost²)`, so no float is
  needed (ADR-0003).
- **Read in E08 T7a (CR-1).** The step's price comes from `0x4B1740`, called by the step wrapper
  `0x4B18C0` (H3API `GetMovementCost`) with the source tile, the direction (odd = diagonal,
  `0x678150`), the points left, the Pathfinding level, the destination's road byte, the fly and
  water-walk levels, the army's native terrain and whether it holds Nomads. Only the *source*
  tile's terrain prices a step; the destination counts only through its road byte: the road cost
  (`0x63E510` rows 10–12 via `0x63E5E0`: 75, 65, 50, flat over Pathfinding) when both tiles have
  a road; else 100 when the source terrain is the army's native terrain and the tile is not
  Cursed Ground (object 21); else the terrain's cost by Pathfinding level (`0x63E510`, four per
  terrain: the packs' costs without Pathfinding). **D-2 settled for VCMI's side:** stepping off a
  road costs the terrain under the road; the wiki's "terrain tile" for a road-to-terrain step
  disagrees (ADV-Q3; the user chose the reading, 2026-10-11). Also there, for later increments: Nomads make sand count as dirt,
  Favorable Winds (object 225) take a third off on water, water walking and flying replace the
  cost (`0x6778AC`: 140, 140, 120, 100 by level).
- **The diagonal factor** is the f32 1.41421354 (`0x63E5F0`), applied on the x87 and truncated
  (`crt_ftol`). For every straight cost below 5741 that equals `isqrt(2 · cost²)` exactly
  (checked for 1 to 100 000: the first cost where they part is 5741), so the integer rule holds
  for all of the original's costs.

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
- **Read in E08 T7a (CR-1).** *The exception* (`0x4B1740` from 0x4B181A; confirmed by an
  independent second reading): for a diagonal the points left are compared with T, the source
  tile's plain terrain cost by Pathfinding, without the road cost and without the native 100.
  Points below T: the step costs trunc(straight × √2). T ≤ points < T × √2: it
  costs its *straight* cost (roads and native terrain included). Points ≥ T × √2: trunc(straight
  × √2). *The bound, exactly* (re-read in T7b): T and the points go to f32 exactly, T × 1.41421354
  is formed at the x87's 53 bits (exact for these sizes, never an integer) and compared with the
  points, so the straight cost applies for T ≤ points ≤ trunc(T × √2) = `isqrt(2·T²)`: with 141
  points on grass the diagonal costs 100. The clean check (both quirks off) keeps that bound and
  that charge and only replaces T by the step's own straight cost; VCMI charges every remaining
  point instead and stops one point lower (points < trunc(straight × √2)). So A-2 and A-3 exist as the wiki describes them (`ADV-QUIRK-2`, `ADV-QUIRK-3`, class
  fix-optional as suggested, pending the user's checkpoint). The exception does not charge the
  remaining points; the rule below then usually zeroes them. *After each step* (the hero's move
  `0x4805E0`, 0x481182): the points left are set to 0 when they are fewer than the cheapest
  straight step from the new tile (`0x4B1880`: its road's cost if it has a road, else 100, the
  terrain's cost on Cursed Ground). No affordable step is lost by it in increment 1 (every step
  from a tile costs at least that much), but the points left differ from the wiki's and from
  VCMI's (VCMI zeroes below the step's own straight cost, which in the example above forbids the
  road step the original allows): **D-3 is a conflict of sources** (reading against VCMI, the
  wiki silent; ADV-Q3: the user chose the reading, 2026-10-11). *A step the hero cannot pay* stops its move and sets its points to 0
  (0x480864): a path longer than the day's points wastes what is left at its end; MoveHero takes
  only paths payable today, so this needs no rule in increment 1.
- **The original's pathfinder** (read 2026-10-11, after T7b; the user asked whether a query can
  miss the best path). `0x4194A0` (H3API `MovementCalculationsMouse`) starts `0x56B440` with the
  hero's current points (+0x4D). The search keeps **one node per tile** (H3Pathfinder +0x24,
  30-byte nodes), pops the cheapest node of its open list and expands it (`0x4B2300`) in the
  eight directions, pricing each step with `0x4B1740` at the points left on the node (start points
  minus the node's cost +0x18, 0x56BB62); `0x4B1A70` replaces a neighbour's node only when the new
  one is strictly better (turn, then cost). The D-3 drop to 0 is not applied there (only in the
  move), which changes no reachable tile in increment 1. So the original, like VCMI and our
  `Reach`, is not exact when a step's price depends on the points left: with the diagonal
  exception, reaching a tile with more points can make the next diagonal dearer (257 points on
  swamp pay 247 for a diagonal, 245 pay 175), and the one-node search keeps the more. The path it
  shows can then leave fewer points than a path the player clicks step by step (example and
  measurements in `E08-log.md`, T7b follow-up).

### 1.4 Where a hero may step

- One of the eight neighbouring tiles on the same level, inside the map; never rock; water
  only by boat (not in increment 1); never a blocked tile of an object. Visiting: ADR draft
  "adventure state", point 4.
- **Only explored tiles** (ADV-MOV-008, E08 T8a). VCMI's pathfinder treats an unexplored tile
  as blocked (`lib/pathfinder/PathfinderUtil.h: evaluateAccessibility`). The user confirmed it
  in SoD (2026-10-11, A-1 question 4): no path onto a dark tile, even an empty one. *Read in
  T8a:* the pathfinder's expansion (0x4B2300, site 0x4B25D4) tests a neighbour's vision word
  (0x4F79B0) against the current player's bit (0x69CCC4): an unexplored neighbour is
  inaccessible for a human player (0x4BADA0, the player record's +0xE2), unless the search's mode
  argument is 3 (the mouse's search 0x4194A0 passes 0; mode 3 not read). For a computer player it
  is accessible from an explored tile (one step into the shroud), or anywhere while the player
  record's +0x3E is above 0 (not read): the computer player's own path, not a rule (the engine
  has no computer players). The fringe's tiles are unexplored, so they are not entered either.
  Then the legal moves never depend on hidden tiles, and the reachable-tiles query and the action
  mask leak nothing.
- Tiles are revealed after each step, so a long move can open new ground. A path is still
  validated against the tiles explored when the command is given (the ADR draft: validated step
  by step, rejected whole).

## 2. Fog of war

- **Radius:** 5 without Scouting; 6, 7 and 8 with Basic, Advanced and Expert Scouting
  (`original-data`: `skill_scouting`, i32 5, 6, 7, 8; VCMI `baseScoutingRange` 5; the wiki's
  SoD values +1, +2, +3; HotA uses +1, +3, +5). Artifacts add +1 each (Speculum, Spyglass), and
  the Scouting specialty adds +1 every six levels: *(inc. 3)*.
- **Radius, read in E08 T8a** (CR-3, 0x4E4070): `skill_scouting` at the hero's Scouting level
  (+0xCC); with Scouting and a Scouting specialty the radius times 1 + 0.05 × the hero's level
  (f32, truncated); +1 for each of the artifacts 52 (Speculum) and 53 (Spyglass), worn or in a
  worn combination. The specialty is a percentage, not the "+1 every six levels" above: checked
  against the wiki and VCMI when the adventure profile comes (increment 3).
- **Shape (ADV-FOG-002):** VCMI reveals a tile when `round(√(dx² + dy²)) ≤ r` on the same level
  (`lib/callback/CGameInfoCallback.cpp: getTilesInRange`, `DIST_2D`). In integers that is
  `dx² + dy² ≤ r² + r` (r = 5: up to 30). *Read in E08 T8a (F-1 settled):* the original's
  reveal (0x49D160) tests `sqrt(dx² + dy²) ≤ r + 0.5` in f64 (0x617B94 is the C library's
  `sqrt`), on the map and the level: the same integer test, since dx² + dy² is an integer and
  (r + ½)² = r² + r + ¼. It sets the bit of every player of the revealer's team (the team bytes at
  game +0x1F879) in each tile's vision word (one bit per player, array 0x6989F8): allies share
  the map *(E10)*.
- **When (ADV-FOG-003):** at the start of the game, around every object a player owns (vcmi
  `CGameState::initFogOfWar`), and after each step of a move around the hero's new tile (vcmi
  `CGameHandler::moveHero`). *Read in E08 T8a:* the move's step (0x4805E0) reveals around the
  step's destination before the hero's position changes (0x48106B); a placed hero's setup
  (0x486590) reveals around it (0x486E25); the new day (0x4C7FE0) calls 0x4C7C50, which reveals
  around every owned hero (and further owned objects, not read): nothing new while no rule hides a
  tile again. Increment 1 models no owned objects, so a game opens with only its heroes'
  surroundings explored. Towns and mines add theirs once they are owned (increment 2 or later): a
  recorded simplification, not a rule.
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
increment 3. Settled by batch 1b (E08 T10c): a hero with its third stack but not its second
(VCMI refuted) and a war-machine hero without its war machine (the wiki refuted), as the reading
has them.

## 4. Quirk candidates: for the user to classify

Classes as in `docs/04-rules-and-sources.md` §4. A candidate becomes an `ADV-QUIRK-<n>` entry
in `docs/rules/quirks-adventure.md` once it is confirmed for SoD. Until then the engine does the
clean behaviour.

| Id | Title | Impact | Suggested class | Confidence it exists in SoD | Confirmation |
|---|---|---|---|---|---|
| A-1 | The fog's fringe shows hidden tiles' terrain and the object sprites drawn into them | information (what a player sees) | fix-optional (faithful shows, strict-fixed shows terrain, roads and rivers only; E08 decision 7) | **confirmed** (the user in SoD, 2026-10-11: terrain, objects and creatures show on the half-dark border; no path onto a dark tile; `ADV-QUIRK-1`) | done; the flag `quirk.adventure.fringe_shows_objects` (E08 T8b1) |
| A-2 | The diagonal exception ignores roads in its check but not in the cost | gameplay-altering (small): an extra cheap road step at the end of a day | fix-optional (the user, 2026-10-11) | **confirmed** (E08 T7a: CR-1 and a second reading; `ADV-QUIRK-2`) | done |
| A-3 | The diagonal exception "forgets" that a native army has no terrain penalty: it compares with the penalised straight cost | gameplay-altering (small): native swamp and snow armies lose the last diagonal | fix-optional (the user, 2026-10-11) | **confirmed** (E08 T7a: CR-1 and a second reading; `ADV-QUIRK-3`) | done |
| A-4 | Neutral stacks before the first faction stack keep the army's native terrain | gameplay-altering (small): slot order decides the native bonus | fix-optional (the user, 2026-10-11) | **confirmed** (E08 T7b: 0x44C590 and a second reading; `ADV-QUIRK-4`) | done |

**Checking A-1 (for the user).** In a game (SoD), move a hero next to the edge of the shroud
near an object you have not seen yet, such as a mine, a creature or a tree. Look at the
half-dark border tiles. Answer three questions:
1. Can you see terrain there?
2. Can you see parts of an object whose tile is still dark?
3. Does a creature standing one tile beyond the edge show at all?
4. Can you order a hero onto a dark tile (does the cursor offer a path)?

A screenshot is not needed; the four answers are enough. Q30's rule then makes the candidate
`ADV-QUIRK-1` with the class you choose.

**The user's answers (SoD, 2026-10-11):** (1) terrain is visible; (2) objects are visible;
(3) creatures are visible; (4) no path is offered onto a dark tile, even an empty one. The
candidate is confirmed as `ADV-QUIRK-1` with the suggested class (Q30's rule: the user may
object at the checkpoint); (4) is ADV-MOV-008, also read in the code (§1.4). Not asked: a hero
standing on a fringe tile. The user's choice for E08 T8b1 (2026-10-11): it shows there as
creatures do, under the flag (ADV-FOG-005).

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
| D-1 | Movement points: at the start of the day for all players, or at the start of each player's turn | wiki and VCMI say start of day; the original's call sites (read in T6b) say start of the owner's turn | T6/T7 (which hook) | read in T6b (§1.1); the user chose the reading; confirmed by batch 1a (ADV-Q4, E08 T10c) |
| D-2 | Stepping off a road: the destination's terrain or the terrain under the road | wiki vs VCMI; read in T7a: the terrain under the road (VCMI's side) | T7 | read (§1.2); the user chose the reading (ADV-Q3, 2026-10-11); confirmed by batch 1a (E08 T10c) |
| D-3 | Does a step that leaves fewer points than its straight cost take all the points | VCMI only; read in T7a: the points drop to 0 below the cheapest next step instead | T7 | read (§1.3); a conflict with VCMI: the user chose the reading (ADV-Q3, 2026-10-11) |
| D-4 | Points for speeds 0–2 | wiki vs VCMI; no army creature is that slow | T4 (data only) | settled in T4: `movement.txt` has the wiki's row (§1.1) |
| F-1 | Shape of the revealed area | VCMI; read in E08 T8a: the same disk | T8 | settled: read (§2), VCMI agrees (ADV-Q5) |

Code readings (Q57: preferred to experiments when a reading can settle the point). Each is
queued in the task that needs it, and each quirk then needs a second look:
- **CR-1** the step cost: H3API `H3Hero::GetMovementCost` (0x4B18C0); D-2, D-3, A-2, A-3.
- **CR-2** the day's points: H3API `H3Hero::CalcMaxMovement` (0x4E4D90) and `MaxLandMovement`
  (0x4E4990) with their callers; the speed table, which speed counts *(inc. 3)*, D-1, D-4.
- **CR-3** the reveal: H3API `H3Hero::GetScoutingRadius` (0x4E4070) and its callers; F-1. Read
  in E08 T8a (§2, §1.4).

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
| ADV-MOV-008 | A path uses explored tiles only | T8a | — |
| ADV-FOG-001 | The scouting radius of a hero (5; Scouting by level, data) | T4, T8 | — |
| ADV-FOG-002 | The shape of a revealed area | T8 | F-1 |
| ADV-FOG-003 | Tiles are revealed at the start of a game and after each step of a move | T8 | — |
| ADV-FOG-004 | Explored tiles stay explored and show their current content | T8 | — |
| ADV-FOG-005 | What a player sees on the fringe | T8b1 | — |
| ADV-FOG-006 | What a player sees of the heroes (another player's: tile, owner, identity) | T8b1 | — |
| GAME-TRN-001 | Players act one at a time in colour order | T6 | — |
| GAME-TRN-002 | The day advances after the last player's turn; start-of-day steps run once for everyone | T6 | D-1 |
| GAME-CAL-001 | The calendar: days, weeks of 7 days, months of 4 weeks (parameters) | T6 | — |
