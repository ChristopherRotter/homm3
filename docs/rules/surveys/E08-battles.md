# E08 battles survey (E08 T17a): wandering monsters, guarded piles, heroes' details, battles on the map

Date: 2026-10-13. Scope: E08 increment 3 (decisions 19–21): the heroes' details in the game,
wandering monsters and guarded piles, battles started on the map and played through `h3-game`,
applying their outcomes; sized for the user's two calls at the checkpoint (monsters joining and
fleeing; battles between two players' heroes) and as input for T17b's ADR drafts. Later
increments' parts are marked *(inc. 4)* or *(inc. 5)*. Input, not rules (`README.md`).

**Sources** (retrieved 2026-10-13):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path: a visit of a monster (`visit_object`'s class 54, 0x4A97B2) and a step onto a
  guarded tile (0x480090 → 0x481900 → 0x4A79C0), both into the same decision (0x4A7740). Not
  followed: the computer player's own answers (0x527F60, 0x52BD10), network play (0x691209),
  campaigns.
- `vcmi`: VCMI 1.7.5 (commit `6901839`): `lib/mapObjects/CGCreature.cpp`,
  `lib/mapObjects/CGResource.cpp`, `lib/mapObjects/CGHeroInstance.cpp`,
  `lib/mapping/CMap.cpp` (`guardingCreaturePosition`), `lib/mapping/MapFormatH3M.cpp`
  (`readHero`), `lib/callback/GameRandomizer.cpp`, `lib/battle/BattleInfo.cpp`,
  `server/battles/`, `config/gameConfig.json`, `config/skills.json`; links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read 2026-10-13 through the summarising fetch tool, which would not quote more than
  125 characters at a time:
  [Wandering creature](https://heroes.thelazy.net/index.php/Wandering_creature),
  [Diplomacy](https://heroes.thelazy.net/index.php/Diplomacy). Re-check a fact against the page
  before a rule cites it with `high` confidence.
- `maps`: a census of the 164 maps shipped with the user's installation, read with `h3-map`
  (a throwaway example, not committed).
- Earlier surveys this one builds on: `E04-battle.md` §2–§4 (retreat, surrender, the outcome:
  experience, artifacts, war machines, Necromancy; batch 1b's answers in `E04a-log.md`),
  `E04c-battlefields.md` §4 (the battlefield from the tile), `E08-objects.md` (visits, pickups).

**Second reading:** a fresh agent re-read the deciding code without this survey (2026-10-13)
and confirmed §3.1 and §3.2 (0x4A7740, 0x4A75C0: the inputs, the thresholds and their
boundaries, the Easy rule, the four outcomes), §4.1 and §4.3 (0x4AC580, 0x44ADC0: the seed's
coefficients, the split, the upgraded stack, the count written back) and §7.3 (the surrender's
transfer); the constant 0.333 is the double 0.333, not a third (`docs/exe/README.md`, "Confirm
before flagging").

Where the reading agrees with VCMI or the wiki the fact has its consensus
(`docs/04-rules-and-sources.md`, "Consensus before trust"); §13 lists the disputes.

## 0. Findings in short

- The monster's answer is the formula VCMI ported and the wiki describes (§3): a power ratio
  step, sympathy and Diplomacy against the monster's aggression decide fight, join free, join for
  gold, or an offer to let it flee. Three differences: on **Easy** a player's Diplomacy counts one
  level higher (the reading and the wiki; VCMI lacks it, D-10); VCMI 1.7.5 **does not count the
  monster's own type** for sympathy (the reading and the wiki do, D-9); the ratio's boundaries
  (D-8).
- **The split into stacks is not random** (§4.1): the original seeds the C runtime's generator
  from the monster's tile, so the number of stacks and the upgraded stack are a function of the
  tile and the two armies. VCMI reproduces the first draw exactly and approximates the second
  (41 of 41 472 tiles of a 144 × 144 two-level map differ, D-11).
- **The battle's obstacles are seeded from the tile too** (§4.2, `battle_side_setup`; VCMI does
  the same). The engine draws them from a kernel stream today: where a battle's seed comes from
  is T17b's call.
- After a battle the monster survives, **its upgraded stack's survivors vanish** (§4.3; the wiki
  calls it a bug, VCMI fixes it): quirk candidate A-6.
- **Guarded piles** (§5): an empty guards record is a plain pile (3 shipped piles become
  importable); a guarded pile asks with its message, or fights at once without one; its guards
  fight as the map gives them, unsplit.
- **Custom experience** (§6.3): 42 of the 116 placed heroes have one, every one of at least
  1 000 (level 2 or more). Without levels (increment 4), increment 3 must refuse them or import
  them at level 1; how the original derives the level and its gains at load was not read.
- **Hero against hero** (§7): an enemy hero's visit is a battle at once; both players play it.
  A surrender's gold goes to the enemy player in the original (VCMI only takes it, D-13).
- A census (§1.1): 14 021 monsters in 162 maps (1 570 fixed, 12 451 random by level; 13 239 random
  counts), 36 guarded piles, 116 placed heroes.

## 1. Monsters in the map file

### 1.1 Classes and the census

Class 54 is a monster of a fixed creature (its subclass is the creature id); classes 71 (any
level), 72–75 (levels 1–4) and 162–164 (levels 5–7) are random monsters (`maps`: the body is the
same for all nine, `h3_map::model::MonsterBody`). The census of the 164 shipped maps:

| What | Count |
|---|---|
| maps with monsters | 162 of 164 |
| class 54 / 71 / 72 / 73 / 74 / 75 / 162 / 163 / 164 | 1 570 / 88 / 2 215 / 3 419 / 3 262 / 1 994 / 979 / 369 / 125 |
| count random (0) / fixed; the largest fixed count | 13 239 / 782; 3 000 (`Pandora's Box`) |
| disposition compliant / friendly / aggressive / hostile / savage | 136 / 38 / 11 618 / 1 023 / 1 206 |
| never flees / does not grow | 1 149 / 506 |
| a message (treasure record) / with resources / with an artifact | 357 / 97 / 55 |

Of the 9 maps that import whole today (`docs/formats/h3m.md`, "Import"), 7 hold monsters, as
inert blockers: `Adventures of Jared Haret` (24), `Faeries` (35), `Knee Deep in the Dead` (26),
`Pandora's Box ` (232), `Race for Ardintinny` (93), `Unholy Quest` (52), `experiment` (25).

### 1.2 What the loader keeps (0x5013B0)

`original-data`: the monster's body becomes the word of its visitable tile (`map_item` +0x0): the
count in bits 0–11, the aggression in bits 12–16 (signed), "never flees" bit 17, "does not grow"
bit 18, the message's index in bits 19–26 and "has a message" bit 31. The message record (text,
seven resources, an artifact) lives in a list of the game. Two consequences:

- **A count above 4 095 loses its high bits** (the loader keeps `count & 0xFFF`; the visit and the
  battle read 12 bits). No shipped map has one (largest 3 000). The wiki gives the editor's limit as
  4 000 in RoE and 9 999 in AB, so an AB or SoD map can hold a count the SoD game cuts. VCMI keeps
  the count whole. Suggested: an input bound of the import (refuse counts above 4 095), not a quirk
  flag (§10).
- **At most 256 monster messages** are told apart (8 bits); the loader stops storing indices at
  4 000 messages. No shipped map comes close (357 messages in all).

### 1.3 Aggression by disposition

`original-data` (0x5013B0's switch, at load): compliant −4, friendly `random_between(1, 7)`,
aggressive `random_between(1, 10)`, hostile `random_between(4, 10)`, savage 10. `vcmi`
`CGCreature::initObj`: the same values, rolled when the game starts. `wiki`: the same five
values ("Default level is 'Aggressive'"; the random map generator's monsters are 4–10). Consensus.

### 1.4 Random monsters' creature (0x4C92C0)

`original-data`: at a new game (`resolve_random_objects`, before the counts) a random monster
becomes class 54 with a creature drawn uniformly (`random_between(0, n − 1)` over the allowed ids)
among creature ids 0–144 whose level lies in the class's range (71: 0–6, 72: 0, …, 164: 6;
zero-based); AB and SoD exclude 132–137 (the four dragons of the expansion, the Enchanter, the
Sharpshooter), RoE allows only 0–117; ids 122, 124, 126 and 128 have level −1 and never come.
`vcmi` `GameRandomizer::rollCreature`: the same set (`excludeFromRandomization` on 132–137).
Consensus. The levels are the executable's, so under `BAT-QUIRK-19` (`faithful`) a random level-4
monster may be an Ice Elemental, a level-3 one a Magma Elemental, a level-5 one an Energy
Elemental.

### 1.5 Random counts

`original-data` (`setup_map_objects`, case 54 at 0x4C1923): a count of 0 becomes
`random_between(low, high)` from the creature's adventure-map range (the creature record's +0x6C
and +0x70, `CRTRAITS.TXT`'s "AdvMap Low/High"). `vcmi` `initObj`: the same (`advMapAmountMin/Max`).
`wiki`: random "between 1–99, depending on the creature type". Consensus. The packs do not hold
the two values yet: a harvest of two `CRTRAITS.TXT` columns (battle-owned harvester, `cross-lane`).

### 1.6 When the draws happen (decision 17)

All at setup in the original: the aggression when the map is loaded (in file order), the random
creatures, then the counts, at a new game (tile order). The engine draws them from the setup
stream when the game is built (ADR-0017 point 4), as increment 2's pickups.

## 2. Guarded tiles and when a battle starts

### 2.1 Which tiles a monster guards (0x41A460)

`original-data`: for a tile (x, y, level) the game scans the 3 × 3 block around it, clipped to the
map, column by column from the left and in each column from the top; the first tile whose top
object (`map_item` +0x1E) is a monster (54) on its visitable tile (+0xD bit 0x10), and whose water
status (terrain 8) equals the scanned tile's, is the guard. When the scanned tile holds an object
that cannot be entered from above (`object_settings` +0x1 clear), the row above is left out. A
monster guards its own eight neighbours on its level, land monsters land tiles and water
monsters water tiles.

`vcmi` `CMap::guardingCreaturePosition`: the same order (`dx` outer, `dy` inner), the same water
test, `checkForVisitableDir` for the direction; VCMI also returns a monster on the tile itself
first. `wiki`: "The priority is top-to-down, left-to-right"; a hero "starts the fight by moving
onto a square next to the creature". Consensus.

The guard flags are kept per tile (the vision word's bit 8, `tile_vision_word`) and recomputed for
the whole map at a new game and whenever an object is removed (0x4CA410, called by
`remove_map_object`).

### 2.2 The step onto a guarded tile (0x480090, 0x481900)

`original-data`: after each step of a move (`hero_move_step`, also `visit_from_neighbour`) the
game tests the new tile's guard flag; on a guarded tile the first guard (§2.1) gets the same
visit as a click on it (0x4A79C0 → 0x4A7740, §3) and the move ends. A hero at sea is attacked at
once; a flying hero (Angel Wings, artifact 72, or a fly spell, hero +0x112) only at its move's
destination *(inc. 4)*. The pathfinder does not go on from a guarded tile other than the start
(`pathfinder_run`, 0x56BC05; read in part), so a path ends on the first guarded tile; a hero
standing on a guarded tile may leave it.

`vcmi`: the pathfinder stops on a guarded tile except the start (`CGPathNode`, `CPathfinder`);
the server starts the fight after the step. `wiki`: "A monolith or similar object does not
trigger combat" *(teleporters, inc. 4 or later)*. Consensus.

### 2.3 Clicking the monster

Class 54 is visited from the neighbouring tile and may be visited from above (the two class
lists, `E08-objects.md` §2.1): the hero pays the step's price and stays (`visit_from_neighbour`,
ADV-OBJ-003), then the same decision. A hero that walks up to a monster is attacked on the guarded
tile it steps on first, possibly by another monster that guards it.

## 3. The monster's answer: fight, join, pay or flee (0x4A7740)

### 3.1 Inputs

- **The hero's power** (0x427650): the army's value (0x44AC80: the sum over its seven slots of the
  creature's AI value, record +0x40, times the count, in 32-bit integers) times the factor
  √((1 + 0.05 · Defense) · (1 + 0.05 · Attack)) (0x4E5400: Attack and Defense, hero +0x476/+0x477,
  clamped to 0–99; computed in double precision, the square root stored as a float), truncated.
  `vcmi` `getValueForDiplomacy` ("H3 formula"), `wiki` ("H = √((1 + 0.05A)(1 + 0.05D))"): the
  same.
- **The monster's strength**: its creature's AI value times its count.
- **The ratio** r = power / strength, both converted to single precision and divided there
  (0x4A77D4 to 0x4A7803). VCMI divides in double precision.
- **The strength step St**: r ≥ 7 → 11; r ≥ 1 → ⌊2r − 2⌋ (0x4A7840, truncation of a
  non-negative value); r > 0.5 → −1; r > 0.333 → −2; else −3 (the constants 7.0, 1.0, 0.5 and 0.333
  are doubles). VCMI and the wiki take r ≥ 0.5 and r ≥ 0.333 for −1 and −2 (D-8: only an exact
  0.5 differs; a single-precision r never equals the double 0.333).
- **Sympathy Sy** (0x4A75C0): the monster's related type is its upgrade, else its downgrade, found
  through the town creature table by the creature's faction and level (0x47B1A0, 0x47B220); a
  neutral creature has none, and in a RoE game neither do the four elementals. The hero's
  creatures of the monster's own type or the related one: none → 0, some → 1, more than half of
  the hero's creatures (by count, `2 · similar > total`) → 2. Because the related type is found by
  level, `BAT-QUIRK-19`'s levels break the downgrade of the Ice, Magma and Energy Elementals in
  `faithful`, which the `wiki` describes ("Water/Ice, Fire/Energy, and Earth/Magma elementals count
  as different types"). `vcmi` `takenAction` counts only the monster's upgrades and downgrades,
  **not its own type** (D-9); the `wiki` counts the same type.
- **Diplomacy L**: the hero's Diplomacy level 0–3 (+0xCD). **On Easy** (the game's difficulty 0,
  `game` +0x1F6D8) a human player's L becomes min(L + 1, 3) (0x4A787E). `wiki` (Diplomacy): "When
  playing at easy difficulty, heroes receive a diplomacy bonus even when they have not learned the
  skill"; "No Diplomacy acts as basic Diplomacy, basic Diplomacy acts as advanced Diplomacy", and
  advanced and expert both act as expert. `vcmi`: Diplomacy gives 1, 2, 3
  (`WANDERING_CREATURES_JOIN_BONUS`), no difficulty rule (D-10).
- **The monster's aggression M** (§1.3) and its "never flees" bit.

### 3.2 The decision

With C = St + Sy + L:

1. M > C: **fight** (§4).
2. Else, M ≤ Sy + L + 1: **offer to join for free**. Yes: the monster joins (§3.3). No: go to 4.
3. Else, M ≤ Sy + 2L + 1: **offer to join for gold**, the creature's gold cost (record +0x38) times
   the count, made only when the player can pay (0x4A72A5; otherwise as a no). Yes: the price is
   paid, the monster joins. No: go to 4.
4. M = C, or the monster never flees: **fight**; after a declined offer at M = C the monster says
   it is insulted first. Else: **the monster offers to flee** ("pursue them?"): yes fights; no lets
   it go: the monster leaves the map without its treasure (0x4A6DF0).

So a compliant monster (M = −4) always offers to join for free (C ≥ −3), and a savage one
(M = 10) never offers to join (Sy + 2L + 1 ≤ 9). Every offer is a yes or no question to the
visiting player.

`vcmi` `takenAction`, `joinDecision`, `fleeDecision`: the same thresholds with VCMI's own L and
Sy (§3.1), the same order and outcomes; it offers the gold price even when the player cannot pay
and treats a yes then as a no ("You don't have enough gold"), which ends the same way. `wiki`:
the same two checks, fight below C, the join threshold the same; its negotiation and flee
thresholds came through the fetch tool as "Sy + D + 2" with the wiki's D = L + 1, which differs
from Sy + 2L + 1 for L ≠ 2; the tool would not quote the line, so it is unconfirmed (the reading
and VCMI agree on Sy + 2L + 1: consensus). "If C = M, they fight": agrees.

### 3.3 Joining and its treasure

`original-data` (0x4A7000, 0x4A7250): the whole group joins the hero's army as one stack
(0x44ACE0); when the army has no room the human player gets the army dialog (0x5D11D0, not read);
the monster's treasure is given (0x4A6B30: its message, its artifact to the hero, its resources
to the owner) and the monster leaves the map. `vcmi`: the same (`joiningPercentage` 100 by
default, `giveReward`, `tryJoiningArmy`).

### 3.4 Messages

A monster with a message shows it first at every visit (bit 31); the texts of the offers come
from the game's text tables. Presentation only.

## 4. The battle against a monster

### 4.1 The split into stacks (0x4AC580)

`original-data`, before the battle:

1. **The seed**: s = 248 071 x + 276 589 y + 310 111 z + 155 303 (32-bit, the monster's visitable
   tile), given to the C runtime's `srand` (0x50C5F0); `rand` is the Microsoft one
   (state ← state · 214 013 + 2 531 011, value = bits 16–30), and `random_between(1, 100)` is
   `1 + rand() mod 100`.
2. **The number of stacks** n: the hero's army value (no Attack or Defense factor) over the
   monster's strength, in double precision, against the table 0x6776E0: below 0.5 → 7, below 0.67
   → 6, below 1 → 5, below 1.5 → 4, below 2 → 3, else 2.
3. **First draw**: d₁ = `random_between(1, 100)`; d₁ ≤ 20 → n − 1; d₁ ≥ 80 and n < 7 → n + 1. Then
   n is clamped to [1, count].
4. **The stacks**: count div n each; the first count mod n stacks get one more.
5. **The upgraded stack**: when the creature is a town's unupgraded creature (0x47B120), n > 1
   and the monster is alone (no second creature type), a second draw d₂ = `random_between(1, 100)`
   ≤ 50 turns stack ⌊n / 2⌋ into the upgrade, same count.

The draws are a function of the tile and the two armies: the same visit always splits the same
way. `vcmi` (`getDefaultNumberOfStacks`, `fight`): the same table (with 0.67 as a float), and
its `hashByPosition` is exactly the first `rand()` after `srand(s)` (its constants are s's
coefficients times 214 013 modulo 2³², plus the step); `containsUpgradedStack` replaces the second
draw by a floating-point formula that agrees with it on 41 431 of the 41 472 tiles of a
144 × 144 two-level map and differs at values right on the boundary (D-11; the first such tile on
a 36 × 36 two-level map is (12, 27, underground), d₂ = 50). `wiki`: the split table by
"relative strength" with ±1 ("There is some variance, but it is not random"); the upgraded stack
"always appears in the middle of the army", "chosen randomly at game start, not by the map maker".
Consensus on the mechanism.

The function also takes up to two more creature types with their own number of stacks (for its
three other callers, not read *(inc. 4 or later)*); then the upgraded stack is skipped and the
stacks are reordered by a table.

### 4.2 The battle (0x4AD470)

The hero attacks; the monster's stacks defend without a hero, at the monster's tile (its
terrain and battlefield, `E04c-battlefields.md` §4; VCMI starts every battle at the defender's
tile). The function passes s on as the battle's stored seed (a battle not against a wandering
monster passes −1 and gets `random_between(1, 1000)`). `battle_side_setup` then seeds the
generator again from the tile, `srand(110 291 x + 167 801 y + 81 569)` (the level does not enter,
0x463A08), stores the battle seed (0x66D840), and sets up the battlefield and its obstacles
(0x4643B0). `vcmi` `BattleInfo::setupBattle` draws the obstacles from a `RandGen` seeded with the
same expression; the engine's `h3_battle::generate_setup_obstacles` draws them from a kernel `Rng`
(BAT-GEO-004). Not read: whether the battle's later draws (damage, luck, morale) continue from
these seeds.

### 4.3 After the battle (0x4A6C60)

`original-data`: the monster's count becomes the survivors **of its own creature type** (0x44ADC0
counts one type), so the survivors of an upgraded stack vanish (A-6, §10). Then:

- the hero's side won: the treasure (0x4A6B30), the "defeat a monster" victory test (0x5F2390,
  inferred), and the monster, now at 0, leaves the map;
- both sides died (a draw): the monster leaves the map without its treasure;
- the hero lost or retreated: the monster stays with its survivors; the hero leaves the map (§7.4).

`vcmi` `battleFinished`: the same for a win and a draw; after a loss it turns upgraded survivors
back into the base creature and keeps them. `wiki`: "A bug removes troops equal to the upgraded
stack's size from the wandering army when a hero is defeated or retreats"; the monster again has
an upgraded stack at the next battle. The reading and the wiki agree.

## 5. Guarded piles and other guarded objects

### 5.1 Guarded piles (0x4A4780)

`original-data`: a pile's tile word has bit 31 when its body has a message record (`E08-objects.md`
§3.1); the record holds a text, a "has guards" flag and an army (0x4C bytes; the army at +0x14).
`visit_guarded_resource`:

- no guards (flag clear, or no stack): **the plain pickup**, with the message shown;
- guards and a text: the text as a yes or no question; no: nothing happens, the hero stays;
- guards and no text: the battle at once;
- the battle: the hero attacks the guards **as the map gives them** (no split, no upgraded stack;
  they defend without a hero), stored seed `random_between(1, 1000)`; a win gives the pickup
  (gold times 100) and removes the pile; a loss or retreat leaves the pile with the guards'
  survivors (the army is passed by address and updated by the battle, inferred).

`vcmi` `CGResource::onHeroVisit`: guards and a message → the question, guards without one → the
battle at once, no guards → the pickup; the pickup after a win. Consensus.

`maps`: 36 guarded piles (29 of class 79 and 7 random resources, class 76; 27 with one stack, 9 with
two), 3 piles with an empty guards record (by the reading, plain piles: today's import refuses
them as guarded, `Unsupported`), and 5 random resources with a message and no guards record (plain
today).

### 5.2 Other guarded objects

Artifacts (class 5: 14), random artifacts (66–69: 19), spell scrolls (93: 3), Pandora's boxes and
events carry guards too *(inc. 4: artifacts and rewards)*; they stay blockers.

## 6. The heroes' details

### 6.1 Two layers in the map file

SoD maps customise hero types for every appearance (`RawMap::custom_heroes`) and each placed hero
in its body (`HeroBody`); the body wins. `maps`:

| What | Count |
|---|---|
| map-wide hero type entries / with a gameplay field | 832 / 15 (all with experience ≥ 1 000) |
| placed heroes (class 34) | 116 |
| custom experience / of at least 1 000 | 42 / 42 |
| custom secondary skills / primary skills / spells / artifacts | 21 / 6 / 7 / 27 (2 with a war machine's slot) |
| custom army / with a war machine as a creature | 58 / 0 |
| tight formation / a patrol radius | 1 / 15 |

Of the 9 maps that import whole, 7 have placed heroes with custom details (`Pandora's Box ` 27).

### 6.2 Defaults

When a field is not customised (`vcmi` `CGHeroInstance::initHero`, `initArmy`, `initExp`; the
original's hero and class tables, `docs/formats/installation.md` §7.1):

- **Experience**: random. `original-data` (`new_game_heroes`, 0x4BFE94): `random_between(0, 50)`
  + 40, so 40–90, for all 156 hero records at a new game; `vcmi`: 40–89 (D-12). Level 1 either
  way.
- **Primary skills**: the class's initial values (`vcmi`); the original's class table was not read
  for this.
- **Secondary skills, the spell book and the spell**: the hero type's (hero table +0xC, +0x1C,
  +0x20; `vcmi` the same).
- **Army**: the starting army (GAM-SET-001), its war machine rolled at 88 % (batch 1b) and given as
  its artifact (`vcmi` `initArmy`: the war machine's artifact slot).
- **The catapult**: every hero gets one (`vcmi`: "everyone has a catapult"); the original was not
  read; it matters in sieges only (E04c).
- **Spell points and movement**: full (`vcmi`: `manaLimit`, `movementPointsLimit`); movement as
  ADV-MOV-001.
- **The level-up seed** (hero +0x8F): `random_between(1, 255)` per hero at a new game; the
  original's level-up offers come from it *(inc. 4)*.

### 6.3 A custom experience

`original-data` (`map_hero_setup`, 0x486624): a custom experience copies the experience **and the
level** (+0x55) from the map's hero; a custom set of primary skills is copied without the worn
artifacts' bonuses and clamped (0–99; Spell Power and Knowledge at least 1). Not read: where the
level is computed and whether the hero gains the levels' primary and secondary skills at load.
`vcmi`: `levelUpAutomatically` at the start, with random primary and secondary gains; HotA's map
format adds an option that prevents those gains. Every custom experience in the shipped maps is
at least 1 000, so increment 3 (experience as a number, no levels) must either refuse such heroes
(`Unsupported`, until increment 4) or import them at level 1 with their experience (wrong primary
and secondary skills): a call for the user (§12), with experiment E3-6 (§13) for the original's
behaviour.

### 6.4 Formation, patrol, presentation

- **Formation**: the map's flag (loose or tight) is read (`vcmi` `readHero`) but `initHero` sets
  every hero to loose afterwards (VCMI resets it); the original's handling was not read. One
  shipped hero is tight. The battle takes it as `BattleSetup.formations` (BAT-SET-001).
- **Patrol radius**: a computer player's hero stays within it (`vcmi` `patrol`); the engine has no
  human or computer player (decision 16): ignored.
- Name, portrait, biography and sex: presentation.

## 7. Battles between two players' heroes

### 7.1 Starting one (0x4A2800)

`original-data`: a hero is visited from the neighbouring tile (class 34). A hero of the same team
(`game` +0x1F879) is met (the exchange screen); an enemy hero in a town is the town's visit
*(a siege, E09)*; any other enemy hero is **attacked at once, without a question**: the visitor
attacks at the defender's tile, both heroes and armies in the battle, stored seed
`random_between(1, 1000)`. `vcmi` `CGHeroInstance::onHeroVisit`: the same three cases. `wiki`:
enemy heroes do not guard tiles as monsters do.

### 7.2 Who plays it

Both players play their side (the original's hot seat and network play); the defender acts
outside its turn. This is ADR-0017 point 6's battle "owned by both players". The original's
retreat and surrender need a hero on the side (`E04-battle.md` §2–§3).

### 7.3 Results

`E04-battle.md` §2–§4 with batch 1b's answers (`E04a-log.md`):

- **Victory**: the loser's hero leaves the map; its artifacts go to the winner's hero (not its war
  machines, OQ-5; not the spell book; the backpack too); +500 experience for a defeated hero.
- **Retreat**: the hero leaves the map with its artifacts; the winner gets experience for the
  killed creatures only.
- **Surrender**: the hero leaves the map with its artifacts and its surviving army; the price
  (BAT-FLE-002) is taken from its owner's gold and **added to the other side's owner's gold**
  (`original-data` `action_dispatcher`, 0x479637 to 0x47966F). `vcmi` `doSurrenderAction` only takes
  it (D-13).
- **Draw**: both heroes leave the map.

### 7.4 A hero that leaves the map

`original-data` (0x4AD470 after the battle; the winner at `battle_manager` +0x13D48, inferred):
when the defending side won, the attacking hero is taken off the map (0x4ACBB0, inferred) and back
into the heroes' pool (0x4D9EC0 with argument 1, inferred: one creature of its first starting
stack, as the tavern's heroes); the player's defeat is tested next (0x5F2E10, inferred). The pool,
the tavern and re-hiring are E09's; defeat and victory are E10's. For increment 3 a hero that
loses, retreats or surrenders leaves the map, and nothing brings it back yet.

### 7.5 Retreat and surrender without towns

The reading found no condition on towns and the game's texts have none (`GENRLTXT.TXT`: the
retreat's question and the Shackles of War's two refusals); the retreat button's own conditions
were not read. Increment 3 has no towns: retreat and surrender stay as BAT-FLE-001/002 allow.

## 8. Applying an outcome

The orchestrator's part (ADR-0013 point 5), from `E04-battle.md` §4 and batch 1b:

- **Casualties**: each army stack's count becomes its survivors (`BattleOutcome::survivors`); a
  damaged top creature is forgotten; a destroyed war machine removes its artifact (the catapult
  stays); a monster or pile keeps its survivors (§4.3, §5.1).
- **Experience** (a number in increment 3): the killed creatures' hit points (no war machines),
  +500 for a defeated hero (not after a retreat or surrender), times 1 + Learning, rounded down
  (R5: ⌊550 × 1.15⌋ = 632); the winner's hero only; nothing in a draw. `hero_gain_experience`
  (0x4E33B0) was not read: the levels it grants and a scenario's level limit come with
  increment 4 and E10.
- **Necromancy**: per killed stack, rounded down (D-7, batch 1b), from `BattleOutcome::stacks`; the
  raised creature by the skill's level *(the Cloak of the Undead King: artifacts, inc. 4)*.
- **Eagle Eye**: `HeroOutcome::learned` (BAT-Q24: whether the hero learns them is the
  orchestrator's).
- **Spell points**: `HeroOutcome::mana`.
- **The loser's surviving war machines** are not captured (OQ-5).

`BattleOutcome` already carries what these need.

## 9. What each viewer sees and learns (for `docs/06-security.md`)

| Fact | The visiting or fighting player | Other players |
|---|---|---|
| A monster's creature type | public on explored tiles (and the fringe under `ADV-QUIRK-1`) | the same |
| A monster's size as a word: few (1–4), several (5–9), pack (10–19), lots (20–49), horde (50–99), throng (100–249), swarm (250–499), zounds (500–999), legion (1 000 or more) | public (0x44ADE0; VCMI `getQuantityID` the same) | the same |
| Its exact count, aggression, "never flees", "does not grow", treasure | hidden; an offer tells the visitor the decision's outcome, a battle shows the exact stacks | hidden |
| The split and the battle's course | the battle's players | hidden |
| A monster joining, fleeing or beaten | its army, treasury and the monster's removal | the removal where they see it; the word of a monster that survived |
| An enemy hero (VCMI's basic view) | identity, class, and its army's types with the words above; in a battle against it, its primary skills, luck, morale and spell points | the same basic view on explored tiles |
| A surrender's gold | both treasuries' changes to their owners | hidden |

The original's right-click views were not read beyond the words (0x44ADE0); the Visions spell
shows more *(spells on the map, inc. 4 or later)*.

## 10. Quirk candidates: for the user to classify

| Id | Title | Impact | Suggested class | Confidence it exists in SoD | Confirmation |
|---|---|---|---|---|---|
| A-6 | After a battle a monster survives, its upgraded stack's survivors vanish (0x44ADC0 counts only the monster's own type) | gameplay-altering (small): a retreat or a lost battle shrinks the monster by those survivors | fix-optional (on in `faithful`, off in `strict-fixed`: the survivors stay, as base creatures as VCMI does) | high: the reading and the wiki | experiment E3-5 (optional) |

Not a quirk flag: a monster count above 4 095 in a map (§1.2), suggested as an input bound of the
import (refused, a typed error), since no shipped map has one.

## 11. Pending decisions (input for T17b)

Increment 3 adds questions to the visiting player, each a yes or no, and battles:

| Offer | When | Yes | No |
|---|---|---|---|
| Join for free | §3.2 step 2 | the group joins (the army's room: §12), its treasure | step 4: a battle, or the flee offer |
| Join for gold | §3.2 step 3, the player can pay | pay, the group joins, its treasure | step 4 |
| Flee | §3.2 step 4 | the battle | the monster leaves, no treasure |
| A guarded pile's message | §5.1, guards and a text | the battle | nothing |

A battle is owned by its players (one against a monster or guards; both against another
player's hero) and blocks only them. Chained: a declined offer may lead to the next question or a
battle in the same command's answer. The original asks only human players; every player is asked
in the engine (as increment 2's offers).

## 12. Sizing the user's calls at the checkpoint

- **Monsters joining and fleeing** (decision 19's open part): three more offer kinds of
  increment 2's shape (§11), the decision's inputs (§3.1: power in single precision, sympathy by
  faction and level, Diplomacy, the game's difficulty), the army's room when a group joins (the
  original's army dialog was not read: the engine needs a rule, for example refusing the offer
  when no slot fits, or letting the player merge first), and gold paid through the treasury.
  Without them every monster fights (the original's step 1 alone), which changes the game for the
  138 compliant and friendly monsters of the shipped maps most.
- **Battles between two players' heroes** (decision 19's open part): a battle owned by two players
  (ADR-0017 point 6), the defender acting outside its turn, the artifacts captured by the winner
  (the slot rules of increment 4, or the backpack only), +500 experience, the surrender's gold
  moving between treasuries (D-13), both heroes in `BattleSetup.heroes`. Without them an enemy
  hero stays an obstacle (today's state).
- **Custom experience** (§6.3): refuse heroes whose experience exceeds level 1's range, or import
  level 1 with the experience as a number.
- **Where a battle's seed and obstacles come from** (ADR-0017 point 4, T17b): the original's tile
  (public data: predictable obstacles, and perhaps rolls, in a game where seeds are server-only,
  decision 9) or a draw from the attacking player's stream (the obstacles then differ from the
  original's at the same tile). The split (§4.1) is a different case: a documented, deterministic
  rule of the tile and the armies that no player's stream needs.
- **Treasures with an artifact** (55 monsters): giving an artifact needs increment 4's slot rules;
  until then such a monster could be typed `Unsupported` (or its artifact go to the backpack).

## 13. Disputes and their confirmations

Planned for an agent-run batch on the client lane's harness (decision 21; `docs/experiments/README.md`).
Its adventure part plays batches since E17 M2 T6 (batch 2 reproduced) but not yet what these need:
Yes/No dialogs (the offers), battles started from the map, the underground (E3-4), and memory reads
of a monster's tile word (E3-3) and of the heroes' experience (E3-7); the user plays what it
cannot. The spec shape needs monsters (creature, count, disposition, never flees),
heroes with custom primary and secondary skills and armies, and the game's difficulty at the start
(`h3-experiment`, battle-owned: `cross-lane`).

| Id | Question | Sources | Needed by | Settle by |
|---|---|---|---|---|
| D-8 | The strength step at r = 0.5 exactly: −2 (reading) or −1 | the reading against VCMI and the wiki | the decision | the user; recommended: the reading (impact: an exact ratio only). E3-3 optional, with a memory read of the monster's aggression |
| D-9 | Does the monster's own type count for sympathy? | the reading and the wiki (yes) against VCMI 1.7.5 (only upgrades and downgrades) | the decision | the user; recommended: the reading. E3-2: a savage group of 10 Pikemen (AI value 80) against a hero (Attack and Defense 0, no Diplomacy) with 56 Pikemen (r = 5.6, St = 9) on Normal: the reading offers to flee (C = 11), VCMI fights (C = 9); a control hero with 39 Halberdiers (AI value 115, r ≈ 5.61): both offer to flee |
| D-10 | Easy shifts Diplomacy up one level (at most expert) | the reading and the wiki against VCMI (no rule) | the decision | the user; recommended: the reading, for every player on Easy (as decision 16's starting resources). E3-1, read with E3-2: a savage group of 10 Pikemen against 52 Pikemen (r = 5.2, St = 8, Sy = 2, no Diplomacy): on Easy the flee offer (C = 11), on Normal a battle at once (C = 10); VCMI fights on both |
| D-11 | The upgraded stack's draw: the second `rand()` after the tile's seed, or VCMI's approximation | the reading against VCMI (differ on 41 of 41 472 tiles) | the split | the user; recommended: the reading (VCMI approximates the same draw). E3-4: Pikemen at (12, 27, underground) on a 36 × 36 two-level map, n > 1: the reading gives an upgraded stack (d₂ = 50), VCMI none; the number of stacks checks the first draw too (d₁ = 44: no change to n) |
| D-12 | The default experience: 40–90 or 40–89 | the reading against VCMI; the wiki not read | the setup | the user; recommended: the reading. E3-7: memory reads of the 156 heroes' experience after a new game (a 90 shows with 95 % if the reading is right) |
| D-13 | A surrender's gold: to the enemy player or nowhere | the reading against VCMI; the wiki not read | hero against hero | the user, if hero battles come in; recommended: the reading. E3-8: a hot-seat battle, one side surrenders, the other's gold before and after |

More planned checks without a dispute: **E3-5** (A-6): retreat from a monster with an upgraded
stack, visit again and read the stacks' counts; **E3-6** (§6.3): a hero with a custom experience
of 5 000: its level, primary and secondary skills at the start; **E3-9** (§6.4): a hero with the
map's tight formation in the battle of E3-8.

## 14. Proposed rule ids (increment 3)

Catalogs: `docs/rules/catalog/adventure.toml` (ADV), `game.toml` (GAM). Ids are proposals; the
implementing task writes the entries.

| Id | Rule | Open |
|---|---|---|
| ADV-MON-001 | Monsters in the import: classes 54, 71–75, 162–164; the body's count, disposition, never flees, does not grow, treasure (§1.1, §1.2) | the count bound |
| ADV-MON-002 | Aggression by disposition, drawn at setup (§1.3) | — |
| ADV-MON-003 | Random monsters' creature by level, drawn at setup (§1.4) | — |
| ADV-MON-004 | Random counts from the creature's adventure-map range, drawn at setup (§1.5) | the harvest |
| ADV-MON-005 | Guarded tiles: the 3 × 3 scan, its order, water, not from above (§2.1) | — |
| ADV-MON-006 | A step onto a guarded tile ends the move and starts the guard's visit; a path does not go on from a guarded tile but the start (§2.2) | — |
| ADV-MON-007 | The monster's answer: power, sympathy, Diplomacy and Easy, the four outcomes (§3) | D-8, D-9, D-10; joining and fleeing (user) |
| ADV-MON-008 | Joining (free or for gold) and fleeing: the army, the treasure, the removal (§3.2, §3.3) | user |
| ADV-MON-009 | The split: the tile's seed, the stack count, the upgraded stack (§4.1) | D-11 |
| ADV-MON-010 | After a monster's battle: survivors of its own type, treasure, removal (§4.3) | A-6 |
| ADV-PIK-004 | Guarded piles: no guards is a plain pile; the question, the battle against the map's guards, the pickup after a win (§5.1) | — |
| GAM-HER-001 | Hero details at setup: the map's two layers and the defaults (§6) | D-12, custom experience (user) |
| GAM-BAT-001 | A battle on the map: who attacks, the tile, the battlefield, the obstacles and the seed, a pending decision owned by its players (§4.2, §7.1, §7.2) | the seed (T17b) |
| GAM-BAT-002 | Applying an outcome: casualties, experience as a number, Necromancy, war machines, spell points (§8) | — |
| GAM-BAT-003 | A hero that loses, retreats or surrenders leaves the map (§7.3, §7.4) | — |
| GAM-BAT-004 | Hero against hero: the visit, both players, captured artifacts, +500 (§7) | user |
| GAM-BAT-005 | A surrender's gold goes to the other side's owner (§7.3) | D-13 |

## 15. The inbox's increment-3 items

- *A war machine in a hero's custom army* (T5a/T6a): none in the shipped maps (§6.1).
- *The starting army's war machine* (T6b): its artifact on the hero (§6.2).
- *What another player's hero shows* (T8b1): §9.
- *The guarded pile's path, the empty guards record* (T11a, T12a): §5.1.
- *Applying an outcome* (E04a T9): §8.
- *The battlefield from the tile* (E04c T2b): `E04c-battlefields.md` §4; the obstacles' seed §4.2.
- *A player without towns retreating or surrendering* (E04a T9): §7.5.
