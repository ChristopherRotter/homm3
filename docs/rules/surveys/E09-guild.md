# E09 mage guild survey (E09 T42): the guild's spells, learning them, the spell points

Date: 2026-11-19. Scope: E09 increment 5 (the mage guild; the epic's decisions 162 and 163),
input for T43's ADR amendment drafts, the design note's Part G and the cut. **Cut in two** by
the user at T42's start (decision 165): **T42a** (this part, §0 to §9) what the guild holds: the
spells' town chances, the draw, the levels and the Library, the map's required and banned
spells, a level built later, the Conflux's Aurora Borealis, the guild's window, the towns'
census; **T42b** (to come, from §10) what a hero gets from it: learning at each of
`town_heroes_learn_spells`' nine callers with the spell book and Wisdom, the spell book, the
spell points restored, the Dungeon's Mana Vortex, the heroes' census, batch TWN-5's rows, then
the checkpoint `TWN-Q31` over both parts. Input, not rules (`README.md`): a fact becomes
binding when a task writes it into the catalog with its sources and tests.

Terms: a **spell** is one of the 70 hero spells (0 to 69, `SPTRAITS.TXT`'s rows in order; 70 to
80 are the creature abilities, never in a guild); its **level** is 1 to 5. A town's **guild
level** is its highest Mage Guild built (buildings 0 to 4, levels 1 to 5; 0 none). A level's
**slots** are the spells it can show: 5, 4, 3, 2, 1 for levels 1 to 5 (`guild_spell_slots`
0x6782A4), one more with the Tower's Library (building 22). The **spare** is the one more slot
the draw fills at every level of every town, Library or not. A **weight** is a spell's town
chance for the town's faction (`SPTRAITS.TXT`'s column of the faction). The map's town sets are
the **required** spells (the map's "must appear", AB and SoD) and the **banned** spells (the
town's "may not appear", every version); the scenario's own **map-wide bans** (SoD's header)
are another set.

**Sources** (retrieved 2026-11-19):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml`. The reading follows the
  human player's path (the new game's setup, the town screen, a build) and names the computer's
  where it differs. **Read whole:** `town_setup_spells` 0x5BE600 (again, at its details),
  `town_heroes_learn_spells` 0x5BE030 (again, its Conflux branch), `spell_table_load` 0x59E090,
  `mage_guild_open` 0x5CE560, `mage_guild_dialog_fill` 0x5D6EF0. **Read in part:**
  `spell_row_load` 0x59E150 (its numeric columns), `load_town_body` 0x5019F0 (its two spell
  sets, 0x501D52 to 0x501F8D), `town_build_effects` 0x5BEDE0 (its guild part, again),
  `new_game_bans` 0x4C2450 (its spell bans, 0x4C27AD to 0x4C28A1), `town_screen_proc` 0x5D3240
  (the guild's click, 0x5D3572 to 0x5D3595). **Data:** `SPTRAITS.TXT` from `H3bitmap.lod` and
  `H3ab_bmp.lod` (`extracted.tar`), the guild window's DEFs in `H3sprite.lod` (`SpellScr.def`,
  `TPMageS.def`), `GENRLTXT.TXT`, `BldgSpec.txt`.
- VCMI 1.7.5 (GPL-2+), read from the tag's sources: `lib/gameState/CGameState.cpp`
  (`initTowns`, the spells), `lib/mapObjects/CGTownInstance.cpp` (`spellsAtLevel`,
  `mageGuildLevel`), `server/CGameHandler.cpp` (`giveSpells`, the build's spells),
  `lib/callback/CGameInfoCallback.cpp` (`getAllowedSpells`), `config/factions/conflux.json`.
- The Heroes 3 wiki, "Mage Guild" (heroes.thelazy.net/index.php/Mage_Guild): its prose and its
  table of each spell's chance to appear in each town.
- The manuals (the user's `HoMM 3 Complete`): RoE's (the town's Mage Guild, the Magic System,
  the Tower's Library); AB's and SoD's say nothing more on the guild.

## 0. Findings in short (T42a)

1. **The weights are `SPTRAITS.TXT`'s nine town columns** (§2): `spell_row_load` 0x59E150 reads
   fields 16 to 24 into the spell record's +0x44 to +0x64, one i32 a faction, Castle to Conflux.
   Each faction's weights of a level sum to 100 at every level it can build; Castle's level 5 and
   Stronghold's and Fortress's levels 4 and 5 are all 0 (the guilds stop at 4 and 3). Titan's
   Lightning Bolt (57) is 0 everywhere and banned in every game by its flag. The AB and SoD copies
   have the same columns; RoE's table is not in the installation (E11). Not harvested yet: 630
   numbers.
2. **The draw happens once, at the new game, for all five levels** (§3, `town_setup_spells`):
   built or not, every town fills its six, five, four, three and two slots (the spare included);
   a slot takes **the first required spell of its level** (in spell order) not excluded, else a
   draw weighted by the faction's column among the level's spells not yet taken; nothing left
   gives an empty slot. **Excluded:** the town's banned spells, the map-wide bans and Titan's
   Lightning Bolt. A required spell is taken whatever its weight (a Castle may show Death
   Ripple); a banned one never, even when required. Nothing redraws them later: a capture, a
   build or a week keeps them.
3. **What the guild shows** (§3.3, §4, §6): per built level its slots (the Library's one more in
   a Tower), fewer only where the draw left empty slots at the end. A level built later, the
   Library built later, a town event's build: the counts are recomputed at once and the town's
   heroes learn (T42b). The faction's highest level is its building list's (Castle 4, Stronghold
   and Fortress 3, the others 5).
4. **The chances check out against the wiki** (§3.5): the exact chance of each spell to be among
   a level's shown slots, computed from the weights as draws without replacement, gives the
   wiki's table to the percent (Bless 54, 32, 31/39 in Castle, Rampart, Tower without and with
   the Library; Magic Arrow 89, 88/94; Death Ripple 95 in Necropolis). The wiki says its values
   are not in the game files; they follow from `SPTRAITS.TXT`'s.
5. **The Aurora Borealis** (§5, the Conflux's Grail building, 26): a hero who learns in a
   Conflux with it learns every spell of a level up to Wisdom's + 2, the guild's level
   notwithstanding (a Mage Guild must have been built), but the town's banned spells and 57.
   **The map-wide bans are not tested there** (T-18): a spell the scenario bans is learned. The
   guild window shows the "all spells" scroll in each shown slot.
6. **The guild's window** (§6): opened from the town screen; a visitor (else the garrison hero)
   without a spell book is first offered one for 500 gold (GENRLTXT 215; with less gold GENRLTXT
   214 and **the window does not open**; refused, it does not open either). After the window the
   town's heroes learn. (Read for T42b, which owns the spell book.)
7. **The census** (§1): no shipped map asks the draw anything hard. 32 towns on 10 maps have
   required spells (never more than a level shows, never above the faction's top, none in a
   Tower); one requires a spell it also bans (`Faeries`, TD-25). 396 towns on 65 maps ban spells;
   only the 14 random towns of `Heroes of Might not Magic` (and its Allied version) leave a level
   short: 12 ban 69 spells and keep nothing. 249 towns start with a guild (34 at level 5), five
   neutral Towers with their Library; no fixed Conflux exists on the shipped maps, and five
   random towns on AB or SoD maps start with the Grail building (an Aurora when they become
   Conflux; `The Great Race` also bans 7 spells map-wide).
8. **VCMI 1.7.5** (§7) draws the same in law (one weighted order of all a town's spells, cut by
   level; the same slots and Library) but adds every required spell, banned or not (TD-25), and
   its Aurora teaches the map's allowed spells, ignoring the town's bans (TD-26: the original
   does the opposite).

## 1. The census (the 160 maps of the installation; a throwaway example over `h3_map::read`, not committed)

The tutorial (`.tut`) included: 160 maps (RoE 49, AB 55, SoD 56), 1 468 towns (592 fixed, 876
random; 889 owned at load), as `E09-towns.md` §1.1. **No fixed Conflux** exists; the fixed towns
by faction: Castle 119, Dungeon 89, Rampart 83, Inferno 80, Tower 69, Stronghold 62, Fortress
58, Necropolis 32.

- **Map-wide bans** (SoD's header): 26 maps, 210 bits, none past 69 (as `h3m.md`, "Import");
  `Heroes of Might not Magic` and its Allied version ban 64 each, `The Great Race` 7, three
  others 5. By spell: Water Walk on 19 maps, Dimension Door 17, Fly 15, Summon Boat 14, Scuttle
  Boat 14, Town Portal 6, Teleport 6, Resurrection 4.
- **Required spells** (AB and SoD; an RoE map has no field): 32 towns on 10 maps (as
  `E09-towns.md` §1.3), 34 bits: 30 towns with one, 2 with two; by level 1: 31 (Summon Boat on
  29, Stone Skin on 2), 2: 2 (Scuttle Boat), 5: 1 (Dimension Door). 10 fixed (6 Castles of `Back
  For Revenge` and its Allied version, Rampart towns of `Faeries` and `Valleys of War`, a Dungeon
  of `War of the Mighty` and its Allied version), 22 random. **No town requires more spells of a
  level than it shows, none a level above its faction's top, no Tower any.** None requires 57 or
  a bit past 69.
- **Required and banned at once:** one town, `Faeries`' Rampart owned by colour 7 (Dimension
  Door, level 5; no guild at the start; TD-25). Required and banned map-wide: none.
- **Banned spells** (the town's): 396 towns on 65 maps (RoE 21, AB 25, SoD 19), 1 779 bits, 112
  fixed; per town 1: 65, 2: 196, 3: 42, 4: 68, 5: 7, 6: 2, 7: 1, 12: 1, 15: 2, 69: 12. By spell:
  Dimension Door 290, Fly 281, Summon Boat 142, Scuttle Boat 88, Teleport 80, Water Walk 63, Town
  Portal 55. None bans 57 or a bit past 69.
- **Short pools:** with the town's and the map's bans and 57 out, no fixed town has fewer spells
  of positive weight (or required) at a level of its faction than the level shows (a Tower
  counted with its Library), so no fixed guild shows an empty slot. Of the random towns, the 14
  of `Heroes of Might not Magic` and its Allied version would (for every faction): 12 ban every
  spell but 57 (every level empty, whatever is built), the neutral one of each (index 372) bans
  15 and, with the map's 64, keeps Death Ripple, Armageddon, Animate Dead and Air Elemental.
- **Guilds standing at the start** (custom sets only: the standard set builds none,
  `E09-towns.md` §5.3): 249 of the 574 custom sets on 67 maps (RoE 44, AB 99, SoD 106); by top
  level 1: 122, 2: 60, 3: 28, 4: 5, 5: 34; 142 owned, 107 neutral; 134 random, 115 fixed. Every
  such set holds every level below its top; none above its faction's top. 59 of them ban spells,
  43 stand on maps with map-wide bans.
- **The Library** (Tower, the map's bit 18) built at the start: 5 neutral Towers, each with a
  level-5 guild (`A Viking We Shall Go` and its Allied version, `Back For Revenge` and its
  Allied version, `The Mandate of Heaven`).
- **The Grail building** (bit 17) built at the start: 3 fixed (two Towers of `A Viking We Shall
  Go`, a Castle of `For Sale`) and 7 random towns; of those, the 5 on AB or SoD maps can become a
  Conflux (`Island King` and its Allied version, neutral, level-5 guild; `Race for the Town`,
  `The Great Race`, `Unexpected Inheritance`, no guild); the two of `Good to Go` stand on an RoE
  map, where a random town is never a Conflux (`load_town_body` 0x5020C5).
- **Forbidden guild levels** (custom sets): 10 towns (from level 1 in 3, from 2 in 2, from 3 in
  5); none of them has required spells.

## 2. The spells' town chances (`SPTRAITS.TXT`; `spell_table_load` 0x59E090, `spell_row_load` 0x59E150)

### 2.1 The load

At start-up (`game_init` 0x4ED81F) `spell_table_load` reads `sptraits.txt` as a text table
(fewer than 92 rows refuses it) and hands each spell row to `spell_row_load` in order: rows 5 to
14 the 10 adventure spells, rows 18 to 77 the 60 combat spells, rows 81 to 91 the 11 creature
abilities (ids 0 to 80). `spell_row_load` fills the spell record (`spell_table` 0x685450, 0x88
bytes): name +0x10, abbreviation +0x14, level +0x18, the school bits +0x1C (Earth 8, Water 4, Fire
2, Air 1: a field neither empty nor a space), the four costs +0x20, the power +0x30, the four
effects +0x34, **the nine town chances +0x44 to +0x64** (fields 16 to 24, `atoi`, one i32 a
faction in the town order Castle to Conflux), the four AI values +0x68, then the four texts.
Loaded once, for every game: no edition reloads it.

### 2.2 The values

Every faction's weights of a level sum to **100** at each level its guild can reach; the levels
it cannot reach are all 0: Castle's level 5, Stronghold's and Fortress's levels 4 and 5. Of the
15 spells of levels 1 to 4 and the 10 of level 5 (Titan's Lightning Bolt counted), a faction
gives a positive weight to 13 to 15 a level (level 5: 6 to 8). The 0s at reachable levels shut a
spell out of a faction's draw (it can come only as required): e.g. Death Ripple (24) and Animate
Dead (39) only in Necropolis, Armageddon (26) only in Inferno, Necropolis and Dungeon, Bless (41)
not in Inferno or Necropolis, Ice Bolt (16) not in Inferno, Disguise (4) not in Fortress, the
level-5 elementals and Magic Mirror by faction. **Titan's Lightning Bolt** (57, level 5) is 0 in
every column and banned in every game by its record's flag 0x2000 (`new_game_bans` 0x4C286C).

The AB copy (`H3ab_bmp.lod`) differs from SoD's (`H3bitmap.lod`) only in row 57 ("Fear
(disabled)", level 4; `installation.md` §7.6), whose nine chances are 0 as well: the guild's
draw is the same with either copy.

### 2.3 RoE and the harvest

RoE's own `SPTRAITS.TXT` is not in the installation (Q15): it has no Conflux column (the inbox's
E04a T3a note) and its values may differ (E11). The packs hold none of the 630 numbers today
(`h3-harvest`'s heroes module: "SPTRAITS: town chances … not harvested"). Two homes fit the
packs as they are (T43's call): **each town record** (`[town.guild]`, a weight per spell key, as
E09 T36 put the hero classes' tavern weights into `[town.tavern]`: the town lane's files, the AB
pack's Conflux record bringing its own column) or **each spell record** (the battle lane's pack,
nine values by faction key). The slot counts (5 to 1) and the spare are the executable's
(`guild_spell_slots`), as parameters of the town part.

## 3. The draw (`town_setup_spells` 0x5BE600, read whole)

Called by `town_setup` 0x5C08B0 at the new game (`setup_towns` 0x4CAA70, after the map's read:
`new_game_bans` set the map-wide bans first), with the town's map record, after the buildings.

### 3.1 What is excluded

A 70-bit set starts with, for each spell 0 to 69, the town's banned bit (the town's +0xD4, the
map record's +0x70 copied by `town_setup`) or the game's ban byte (game +0x4A + spell:
the scenario's map-wide bans and the flag 0x2000, `new_game_bans`). Every spell taken joins it.

### 3.2 Slot by slot

For each level L = 1 to 5 and each of its `guild_spell_slots[L] + 1` slots (6, 5, 4, 3, 2), in
the town's slots +0x44 + 0x18 (L − 1) on:

1. Scan the spells 0 to 69 in order; for each not excluded **of level L** add its weight (the
   record's +0x44 + 4 × the town's faction) to a total, and **if the map record's required set
   (+0x7C) holds it, take it at once** (the scan stops; its weight does not matter).
2. Else, with a total of 0, the slot stays empty (−1) without a draw.
3. Else `random_between(1, total)` (the game's one generator, in the setup's order of the towns
   and their other draws) and the first spell, in order, whose running weights reach the roll;
   a weight of 0 is never chosen.

The slot's spell joins the excluded set. So the required spells of a level fill its first slots
in spell order (the spare too, when there are more than the level shows: no shipped map has
that); a banned spell never appears, required or not; the draw is without replacement, slot by
slot. An empty slot is only ever followed by empty slots of its level (a later slot can find
neither a required spell nor a weight).

### 3.3 The guild's level and counts

After the five levels: the guild level +0x14 is the highest Mage Guild in the standing set
(+0x150, buildings 4 down to 0; 0 none) and, for each level 1 to the guild level, its count
+0xBC + (L − 1) is `guild_spell_slots[L]`, **plus one for a Tower (faction 2) whose every-built
set (+0x158) holds building 22** (the Library), then less the empty slots at the count's end.
The counts of the levels above the guild level are 0. These counts are what the guild shows and
teaches (`town_heroes_learn_spells`, the window).

### 3.4 The map's sets (`load_town_body` 0x5019F0, 0x501D52 to 0x501F8D)

After the buildings: on a map of version above RoE (game +0x1F86C not 0xE) 9 bytes, bit `i` of
byte `i / 8` the spell `i` for 0 to 69, into the record's **required** set +0x7C; on an RoE map
nothing is read and the set stays empty. Then 9 bytes in every version, the same way, into the
**banned** set +0x70; bits 70 and 71 are dropped. A short read fails the map. `h3_map`'s reader
already holds both (`TownBody::required_spells`, `banned_spells`); the import does not carry them
yet.

### 3.5 The chances against the wiki

Taking each level's shown slots as draws without replacement by the weights, the exact chance of
a spell to show (a throwaway script over `SPTRAITS.TXT`) gives the wiki's table to the percent on
every row compared: Bless 54, 32, 31 and 39 (Tower without and with the Library), 0, 0, 19, 20,
31, 31; Magic Arrow 89, 89, 88 and 94, then 88 or 89; Death Ripple 95 and Animate Dead 44 in
Necropolis, 0 elsewhere. The wiki calls its table its own estimates, the values "not in the game
files"; they are consequences of the file's columns. With the original's draw and VCMI's (§7)
giving the same law, an experiment on the chances is not needed.

## 4. The guild built later (`town_build_effects` 0x5BEDE0, its guild part)

Every build of the game's paths (the hall's offer `town_buy` 0x5BF3C0, the town events
`town_events_apply` 0x5BFEB0, the computer's `ai_town_visit` 0x5253D0, the campaigns) goes
through `town_build_effects`: after `town_build` and the every-built set recomputed,

- **a Mage Guild 0 to 4:** the guild level becomes that level (`building + 1`) and every count of
  levels 1 to it is recomputed as in §3.3 (the Library's one more by the every-built set);
- **the Tower's Library (22):** the counts recomputed the same way (so the spare of each built
  level shows at once);
- then, after **every** build, `town_heroes_learn_spells` with 0 (the visitor, then the garrison
  hero; T42b).

A town event builds without a requirement test (`E09-construction.md` §10), so it may raise the
guild by several levels at once (each bit in id order): the counts follow. The spells
themselves are never drawn again: a guild level built on day 100 shows what the new game drew.
The faction's highest level is its building list's (`faction_buildings` 0x6976F0; the packs'
`mage_guild_level_*` records: Castle to level 4, Stronghold and Fortress to 3, the others to 5);
a level the faction lacks or the map forbids is never built.

## 5. The Aurora Borealis (the Conflux's Grail building, 26)

`town_heroes_learn_spells`' first branch (read whole at T7b2b; again here): a hero with a spell
book, in a town whose every-built set holds a Mage Guild (building 0), learns, when the town is a
**Conflux (faction 8) whose every-built set holds building 26**, every spell 0 to 69 **not in
the town's banned set** (+0xD4) whose level is at most Wisdom's level + 2 (hero +0xD0), but 57;
the guild's slots and level are not looked at. Otherwise the guild's own branch (T42b).

- **The map-wide bans (game +0x4A) are not tested** (0x5BE10C to 0x5BE172): a spell the scenario
  bans is learned there (quirk candidate T-18). Reachable on a shipped map: `The Great Race`
  bans 7 spells map-wide and has a random neutral town with the Grail building at the start
  (§1), a Conflux when it resolves to one.
- **The guild's level does not bound it:** a Conflux with the Grail and a level-1 guild teaches a
  level-5 spell to an Expert in Wisdom. VCMI says so in a comment; the game's own description of
  the building (`BldgSpec.txt`) says it puts every spell into the guild.
- The town's slots and counts stay as drawn: the window shows the "all spells" scroll in each of
  them (§6), and what another reader of the slots sees (the computer's valuation) is the drawn
  spells.
- Building 26 stands only with the Grail (increment 7's `town_grail_build` 0x5D30D0), a custom
  set at the start or a town event (`town_events_apply` makes no Grail test); the Grail
  building's gold and growth are built already (increments 1 and 3), so its spells are the only
  part of it missing.

## 6. The guild's window (`mage_guild_open` 0x5CE560, `mage_guild_dialog_fill` 0x5D6EF0)

From the town screen's click on the Mage Guild (`town_screen_proc` 0x5D357F); the owner's
screen (`E09-towns.md` §7.2).

1. **The spell book offer** (read whole; for T42b): the hero is the town's visitor, else its
   garrison hero; with one that carries no spell book (`hero_has_artifact` 0x4D91B0 with 0, the
   backpack included) and a player whose both human flags are set (`player_is_human_both`): in a
   campaign, hero 45 (Yog) on one scenario is refused (GENRLTXT 737) and nothing opens; with less
   than 500 gold (player +0xB4) GENRLTXT 214 and **the window does not open**; else GENRLTXT 215
   asks, and on no the window does not open, on yes the hero gets the spell book
   (`hero_give_artifact` 0x4E3070) and pays 500 gold. A hero with a book, or no hero, goes
   straight to the window.
2. **The window** (`mage_guild_dialog_create` 0x5CC980, not read: twenty scroll items, 6 + 5 + 4
   + 3 + 2, and the guild's picture by faction): per level 1 to 5 and slot 0 to 5,
   `mage_guild_dialog_fill` shows **the slot's spell** (`SpellScr.def`'s frame of the spell) for
   a slot below the level's count; in a Conflux with building 26 **frame 70 instead**, the "all
   spells" scroll; another state (the empty slot; its picture not looked at) for a slot below the
   level's slots (the Library's one more in a Tower) but at or above the count, so a level not
   built yet shows its slots empty; nothing past the slots, and nothing at a level the town's
   buildable set lacks (`town_building_allowed` 0x5C12A0: a level of the faction the map did not
   forbid).
3. Back in the town screen: `town_heroes_learn_spells` with 0 (0x5D3589), then the screen redrawn.

So a player sees a town's guild in its own town screen only; nothing else shows the spells (the
quick view shows none, `E09-towns.md` §12.3).

## 7. VCMI 1.7.5, the wiki and the manuals compared

- **VCMI's draw** (`CGameState::initTowns`): the required spells first, each appended to its
  level's list in the map's order, then every other spell of the town's allowed set (the map's
  spells less the town's banned, less those of weight 0 for the faction, the special ones, the
  creature abilities and the map-wide bans) in one weighted order without replacement, appended
  to its level. A level shows the first `spellsAtLevel` of its list: `6 − level`, one more with a
  building of the Library's kind, 0 above the guild level (`CGTownInstance::spellsAtLevel`).
  **Same in law as the original** (a level's subsequence of one weighted order of all spells is
  distributed as a weighted order of that level's spells alone), different in the draws it
  makes. **One difference:** VCMI adds a required spell even when the town or the map bans it
  (TD-25; the original excludes it). VCMI's slots past what a level can show are its whole list,
  the original's the spare only: no shipped map tells them apart.
- **VCMI's Aurora Borealis** (`giveSpells`, `conflux.json`'s `auroraBorealis`): with a Mage Guild
  built, every spell of the map's allowed set (`getAllowedSpells`: the map-wide bans out) of a
  level the hero can learn, "even if only level 1 mages guild built" (as the original). It does
  not read the town's banned set; the original reads only that (TD-26).
- **VCMI's build** (`CGameHandler`, the build's end): a Mage Guild, a Library or the Aurora built
  gives the visitor and the garrison hero their spells; the original teaches after every build
  (the same where it matters: nothing else changes what a hero can learn there).
- **The wiki** ("Mage Guild"): the slots by level, the highest levels by town (Castle 4,
  Stronghold and Fortress 3), the town's probabilities choosing a random set for each level, the
  Library's one more spell a level, the Aurora teaching every spell not banned (it does not say
  by whom), the map maker able to ban almost every spell so that slots stay empty, the spell book
  for 500 gold, the spell points refilled overnight in town (T42b); its table of chances matches
  the reading exactly (§3.5).
- **The manuals** (RoE's): a random selection of spells as each level is built; the Library makes
  one extra spell available per level; clicking the guild shows its spells and, for a hero
  without a spell book, sells one.

## 8. Disputes, quirk candidates and open points (T42a)

- **TD-25 — a spell both required and banned by the town.** The reading: excluded, never in the
  guild (§3.2; the map-wide bans the same). VCMI: added first, as required. The wiki is silent.
  One shipped town (`Faeries`, Dimension Door in a Rampart owned by colour 7, guild not built at
  the start). Recommended: the original (the ban wins). Experiment: a town requiring and banning
  a level-1 spell with a level-1 guild built; is it shown?
- **TD-26 — the Aurora Borealis' spells.** The reading: every spell not banned by the town, the
  map-wide bans not tested (T-18). VCMI: the map's allowed spells, the town's bans not tested.
  The wiki: every spell "not banned", by whom unsaid. Reachable on one shipped map (`The Great
  Race`, if its random town becomes a Conflux). Recommended: the original's set for the town's
  bans, the map's bans by T-18's classification. Experiment: a SoD map banning a spell
  map-wide and the town banning another, a Conflux with the Grail and a guild, a hero with a book
  and Expert Wisdom; which does it learn?
- **T-18 (quirk candidate) — the Aurora teaches a spell the scenario bans.** The only one of the
  original's paths read so far that ignores the map-wide bans (the guild's draw reads them, §3.1,
  and so does the random Shrines' draw, `E08-rewards.md` §6.5). Candidate class: fix-optional (a
  map's intent broken, no state corrupted); the user's at the checkpoint, with TD-26.
- **Open for T42b:** every caller of `town_heroes_learn_spells` (a hero's learning, Wisdom's
  bound, the order of the visitor and the garrison hero); the guild branch's loop reads the count
  one level past the guild level (0, harmless; 0x5BE195); the spell book bought elsewhere; the
  spell points; the Mana Vortex.
- **Open for T43:** where the weights live (§2.3); the draw's stream (the original's one
  generator interleaves the towns' setup draws; the engine draws from streams, as the taverns'
  `StreamId::TAVERNS`); the spare drawn for every town (the original's, and what makes a
  Library built later show at once) or only for a faction with a Library; whether the Aurora
  belongs to increment 5 (its spells are a branch of the learning, its building stands at the
  start or by events already) or 7 (the Grail found); the guild in the owner's town view and its
  telling (`docs/06-security.md`).

## 9. For T42b and batch TWN-5 (from T42a's readings)

Rows for T42b to fold into batch TWN-5 (the readings at risk: two contradict VCMI, one is the
Aurora's run-time branch):

| Row | Setup | Observe | Settles |
|---|---|---|---|
| a | an AB map, a town requiring and banning one level-1 spell, a level-1 guild built | the guild's five spells | TD-25 |
| b | a SoD map banning spell A map-wide; a Conflux banning spell B, with the Grail building and a level-1 guild; a hero with a spell book and Expert Wisdom visits | the spells learned (A and B), the levels learned (to 5) | TD-26, T-18, §5's level bound |
| c | a Tower with a level-5 guild; the Library built in the game | each level's count before and after, the spare's spell learned at once by the visitor | §3.3, §4 |

Not needed: the chances (§3.5), the faction's highest levels (the building lists).
