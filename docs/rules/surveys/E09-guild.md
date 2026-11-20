# E09 mage guild survey (E09 T42): the guild's spells, learning them, the spell points

Date: 2026-11-19. Scope: E09 increment 5 (the mage guild; the epic's decisions 162 and 163),
input for T43's ADR amendment drafts, the design note's Part G and the cut. **Cut in two** by
the user at T42's start (decision 165): **T42a** (this part, §0 to §9) what the guild holds: the
spells' town chances, the draw, the levels and the Library, the map's required and banned
spells, a level built later, the Conflux's Aurora Borealis, the guild's window, the towns'
census; **T42b** (§10 to §19) what a hero gets from it: learning at each of
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
- **T42b** (retrieved 2026-11-20): the code reading again. **Read whole:**
  `town_heroes_learn_spells` 0x5BE030 (a third time), `town_screen_open` 0x5BE210,
  `hero_primary_clamped` 0x5BDE40. **Read in part:** each of the nine callers at its call;
  `town_screen_bars` 0x5C6E10 (its tail), `ai_town_visit` 0x5253D0 (to 0x525545 and its end),
  `ai_hero_turn` 0x5261F0 (its start), `new_day` 0x4C7FE0 (0x4C8237 to 0x4C8434),
  `town_visit_buildings` 0x5BD8E0 (its first branch), `hero_reset` 0x4D8720 (0x4D889F to
  0x4D890A), the ends of `town_screen_init` 0x5C63C0, `town_castle_gate` 0x5D8480 and
  `town_tavern_click` 0x5D82B0; `GENRLTXT.TXT`. VCMI 1.7.5: `server/CGameHandler.cpp`
  (`giveSpells`, `changeSecSkill`, `heroVisitCastle`, `visitCastleObjects`, the build's end,
  `buyArtifact`), `lib/mapObjects/CGTownInstance.cpp` (`onHeroVisit`),
  `lib/mapObjects/CGHeroInstance.cpp` (`getManaNewTurn`, `maxSpellLevel`),
  `server/processors/NewTurnProcessor.cpp`, `server/processors/HeroPoolProcessor.cpp`,
  `config/factions/dungeon.json`, `config/gameConfig.json`, `lib/rewardable/Reward.cpp`,
  `Limiter.cpp`, `Configuration.h`. The wiki's "Mana Vortex" and "Mage Guild" through search
  results (the pages are not reachable from the session). RoE's manual (the Town Screen, the
  Magic System, the Dungeon's buildings).

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

**T42b** (what a hero gets, §10 to §19):

9. **Who learns** (§11): the town's visitor, then its garrison hero (or the one hero a caller
   names), each wearing a spell book in a town with any Mage Guild: every spell the guild shows
   at levels 1 to Wisdom + 2 (none 2, Expert 5; HER-SPL-005's bound), or §5's Aurora set. A known
   spell stays; no owner test (an allied visitor learns), no map-wide bans.
10. **When** (§12): at the end of every visit of an own or allied town, a capture and a siege
    battle at it; at every build (a town event's too); at a tavern hire; at the start for a main
    town's hero (TD-10); after the guild's window; whenever the town screen's hero bars are built
    with a visitor there, so at **every opening of the owner's town screen**, visit or not, and at
    a move between the garrison and the entrance; at Town Portal's and Castle Gate's arrivals; at
    the computer's visits. **Not** at the new day or a human's turn start, nor when a hero in a
    town gains Wisdom or a book (it learns at the next trigger); a placed hero on an entrance learns
    nothing at the start.
11. **The spell book** (§13): the human is asked at the guild (500 gold, §6); the computer buys
    one without asking at each visit of a town with a guild while its owner has 500 gold; else a
    hero has one by its type, the map's artifacts, Titan's Thunder or a reward, and learns at the
    next trigger.
12. **The spell points** (§14): at every new day, after each hero's daily regain, each town with a
    Mage Guild raises its visitor's and garrison hero's points to their maximum (HER-MAN-001),
    never lowering them; **no spell book needed**; a visit alone restores nothing.
13. **The Mana Vortex** (§15; the Dungeon's building 21, slot 18): **once a week a town** (its
    flag set at the start and every week), a hero below twice its maximum gets twice it, no book
    needed. Given at every opening of the town screen and at a build with the visit (the garrison
    hero first, then the visitor), a human's tavern hire, Castle Gate's arrival and the computer's
    visit; not at Town Portal's arrival, not at a turn's start.
14. **The census** (§10): 561 heroes on an entrance at the start (538 main towns' heroes, 23
    placed), none in a garrison; in a town with a guild 33 main towns' heroes and 8 placed ones
    with a book; Wisdom's bound bites for none; one Vortex at the start, in a neutral town.
15. **VCMI 1.7.5** (§16) learns the same set, buys the book alike, refills the points and gives the
    Vortex alike, but by other triggers: every turn start visits each town for its garrison hero
    and visitor (learning, the Vortex), a skill changed in town teaches at once, a visit teaches
    the visitor only, a town event's build only a guild's, the Library or the Aurora (a player's
    build both, the garrison hero first); nothing at the start.
16. **For the checkpoint** (§17): TD-10, TD-25, TD-26 with T-18, and **TD-27** (when the heroes learn
    and get the Vortex: the original's screen opening against VCMI's turn start); batch TWN-5's
    five rows (§18).

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

## 10. The heroes' census (T42b; the same throwaway example over the built games, not committed)

Over the 160 shipped maps as `h3-map`'s `original_data` builds them (seed 0: the main towns'
heroes are drawn by the engine's stream, not the original's, so their types vary by seed; the
T7b2b census counted the same 33 below):

- **249 towns start with a guild** (T42a), 142 of them owned; **74** hold a hero on the entrance
  at the start; **none a garrison hero** (the original writes none at a new game, `E09-towns.md`
  §4.6).
- **561 heroes stand on a town's entrance at the start:** 538 main towns' heroes (all drawn: the
  import picks none) and 23 placed heroes moved onto an entrance (TWN-SET-010).
- **In a town with a guild:** 60 main towns' heroes, **33 with a spell book** (they learn the
  guild's spells at the start in the original, TD-10; 27 without), and 14 placed heroes, **8 with
  a spell book** on 5 maps (`Dead and Buried` 2, `Hoard` and its Allied version 2 each, `The
  Gauntlet`, `Wings of War`), which learn nothing at the start (§12) and so at their first
  trigger.
- **Wisdom's bound bites for none of the 41** with a book: no such guild stands above its hero's
  Wisdom + 2.
- **One town starts with the Mana Vortex** (a neutral Dungeon on `Caught in the Middle`), no hero
  in it; the others come from builds and town events (not counted).

## 11. Learning (`town_heroes_learn_spells` 0x5BE030, read whole again)

1. **The heroes:** with argument 0 the town's visitor (+0x10), then its garrison hero (+0xC), each
   that exists (both −1: nothing); with a hero, that hero alone (whether it stands there is not
   tested: the one such caller passes the visitor, §12).
2. **Each hero learns only** while it wears a spell book (`hero_wears_artifact` 0x4D91F0 with 0:
   the book's position) in a town whose every-built set (+0x158) holds building 0 (Mage Guild 1;
   a higher level supersedes it, so the set holds 0 whenever any guild stands).
3. **A Conflux with the Aurora Borealis:** §5's branch (every spell of a level up to Wisdom + 2 the
   town does not ban, but 57).
4. **Otherwise** level after level from 1 while the level is at most Wisdom + 2 and at most the
   guild level + 1: the level's first `count` slots (+0x44 + 0x18 × (level − 1); `count` the
   town's +0xBC + level − 1, §3.3: the level's shown spells, 0 for a level not built, so the
   level past the guild read at 0x5BE195 adds nothing), each through `hero_learn_spell` 0x4D9330
   (the learned and the available bits set; a known spell unchanged).
5. **Wisdom** is the hero's skill level byte +0xD0 (0 none, 1 to 3): levels 1 and 2 without it,
   3 at Basic, 4 at Advanced, 5 at Expert, as the Shrines, the Scholar, Eagle Eye and the boxes
   (HER-SPL-005's `learn_level`; the engine's `can_learn_spell` with the known spells counting,
   HER-CHG-004, gives the same set spell by spell).
6. **Not tested:** the owner (the town's or the hero's: an allied visitor learns), the map-wide
   bans (the guild's spells were drawn without them, §3.1; the Aurora's T-18), the class. Nothing
   is told but by the screens.

## 12. When a town's heroes learn (the nine callers)

| Caller (call) | When | Heroes |
|---|---|---|
| `town_visit` 0x4AAFD0 (0x4AB3C3) | the end of every visit of an own or allied town (after the human's town screen or the computer's visit), of an undefended town captured, and after a siege's battle at the town whoever won (not after the battle against a visitor beside a garrison hero, which returns first); **before** the visitor's level-up (`hero_level_up` 0x4AB3CB) and the scenario's tests | 0 |
| `town_screen_bars` 0x5C6E10 (0x5C7007; named here) | the town screen's two hero bars built: at every opening of the screen (`town_screen_load` 0x5C6D01), after a garrison click, the visitor put into the garrison (`town_hero_in_click` 0x5D5150), the garrison hero sent out (`town_hero_out_click` 0x5D5220) | 0, only while a visitor stands there and Mage Guild 1 stands |
| `town_screen_proc` 0x5D3240 (0x5D3589) | after the guild's window (§6), so a book just bought teaches at once | 0 |
| `town_build_effects` 0x5BEDE0 (0x5BF034) | after every building built (the hall's, the computer's, a town event's, a campaign's), once the guild's counts are set again (§4); before the building's visit (§15) | 0 |
| `town_tavern_hire` 0x5C12E0 (0x5C138E) | a tavern hire (the human's and the computer's), the hero placed as the visitor | 0 |
| `main_town_heroes` 0x4CA040 (0x4CA1A2) | the start: a main town's hero placed on its entrance (TD-10) | 0 (a hero already on the entrance learns in the new one's place; the engine refuses that start, `TWN-QUIRK-6`) |
| `hero_relocate_spell` 0x41D360 (0x41D867) | Town Portal's arrival (`adventure_spell`'s case 9), the hero on the town's entrance; no building visit (§15) | 0 |
| `town_castle_gate` 0x5D8480 (0x5D862E) | Castle Gate's arrival, before the screen is rebuilt | 0 |
| `ai_town_visit` 0x5253D0 (0x52553E) | the computer's visit: inside `town_visit`'s (which then teaches with 0 too), and at the start of every turn of a computer hero standing on a town's entrance (`ai_hero_turn` 0x526262, by `town_at_entrance`), after its spell book is bought (§13) | the hero alone |

- **The town screen opens without a visit too:** `town_screen_open` 0x5BE210 (read whole: the
  town manager's town, the manager run by 0x4B0C70, whose init is `town_screen_init` 0x5C63C0,
  then the visitor selected for the shown player) is called by `town_visit` and by the adventure
  screen (its town button 0x409EA3, 0x409EC6, and two clicks in `hero_move_loop` 0x4081F3,
  0x4082F0). So the owner's every opening of a town with a visitor teaches both heroes, and its
  building visits run (§15).
- **Nothing else teaches:** not the new day, not a turn's start of a human (the computer's heroes
  on an entrance re-visit, above), not a level-up, a secondary skill or a spell book a hero gets
  while standing in a town. Such a hero learns at the next of the triggers above; the visit's
  level-up comes after its learning, so a Wisdom level it brings counts at the next visit,
  opening or build. A placed hero moved onto an entrance at the start learns nothing
  (`placed_heroes_setup` 0x4CAE10, §10).

## 13. The spell book

- **The human** buys one at the guild's window (§6): the visitor, else the garrison hero, asked
  (GENRLTXT 215) for 500 gold of the current player, refused below it (GENRLTXT 214, the window not
  opened); then the town's heroes learn (§12).
- **The computer** (`ai_town_visit` 0x5254C2 to 0x525531) buys one without asking at every visit
  (§12's two): its hero's owner (+0x22) with 500 gold or more (the player record +0xB4), Mage
  Guild 1 standing, the hero without a worn book: the book given (`hero_give_artifact` 0x4E3070)
  and 500 gold taken; then the hero learns. Whether a computer buys is the computer player's
  choice (E10, E13); the purchase itself is the human's rule.
- **Other sources:** the hero type's flag (the hero table +0x1C; `hero_reset` 0x4D88DA puts the
  book at its position and learns the type's spell +0x20), a placed or customised hero's
  artifacts (`placed_hero_setup` 0x4D8B30), Titan's Thunder (HER-ART-007), any artifact given
  (`hero_give_artifact`'s callers: the rewards and the other objects; a battle's spoils never
  hold it). None
  teaches the guild's spells by itself: a hero standing in a town that gets one learns at the
  next trigger (§12). The book never moves nor is captured (HER-ART rules).

## 14. The spell points restored (`new_day` 0x4C7FE0, 0x4C82E4 to 0x4C8434)

- **After every hero record's daily regain** (0x4C8237 to 0x4C82DE, HER-MAN-002), each town in
  order whose every-built set holds building 0 raises its visitor's, then its garrison hero's
  spell points (+0x18) to the hero's maximum: trunc(intelligence × f32(10 × Knowledge clamped to
  1 to 99)) (`hero_intelligence` 0x4E48B0; HER-MAN-001's). Points above it stay (never lowered).
- **Not tested:** a spell book, the owner (an allied visitor is refilled), the guild's level.
- **Only at the new day:** a visit, a build or a hire restores nothing (the manuals: "after staying
  for one day", "when he or she starts a new day in a town with a mage guild"); day 1 has no new
  day (GAM-CAL-002).

## 15. The Mana Vortex (`town_visit_buildings` 0x5BD8E0, its first branch 0x5BD8EB to 0x5BD996)

- **The test and the effect:** a Dungeon (faction 5) whose week flag (+0x33, `structs.toml`
  `week_flag`) is set, with building 21 standing (+0x150; the pack's `mana_vortex`, the map's
  and the pack's slot 18, requiring Mage Guild 1), and a hero whose spell points are below twice
  its maximum (2 × trunc(intelligence × f32(10 × Knowledge clamped to 1 to 99)),
  `hero_primary_clamped` 0x5BDE40): the points set to twice the maximum and the flag cleared; a
  message (GENRLTXT 580, an OK box) when the town's owner is a human here. No spell book needed;
  a hero at or above twice its maximum leaves the flag set for the next.
- **The week:** the flag is set to 1 by `town_init` 0x5BDF40 and for every town at each week's
  first day (`new_day_week_events` 0x4C8A09): **once a week a town**, not a hero.
- **When** (`town_visit_buildings`' callers, its heroes in order):

  | Caller (calls) | When | Heroes |
  |---|---|---|
  | `town_screen_init` 0x5C63C0 (0x5C65ED, 0x5C661A) | every opening of the town screen (§12: a visit's or the owner's at will) | the garrison hero, then the visitor |
  | `town_build_effects` 0x5BEDE0 (0x5BF1B1, 0x5BF1CC, 0x5BF1F6) | a build with the visit (the hall's and the computer's buys, the town events; a campaign's buildings without) | the garrison hero, then the visitor |
  | `town_tavern_click` 0x5D82B0 (0x5D846C) | after a human's hire | the hired hero (the visitor) |
  | `town_castle_gate` 0x5D8480 (0x5D8686, 0x5D86B3) | Castle Gate's arrival | the visitor |
  | `ai_town_visit` 0x5253D0 (0x525977, 0x52598B, 0x5259B1) | the computer's visit (§12) | the garrison hero, then the visitor |

  Not Town Portal's arrival (only the learning, §12), not a visit but through the screen or the
  computer's, not a turn's start. So with both heroes in a town the garrison hero takes it, and
  a garrison hero alone gets it when its owner opens the town screen.
- The rest of `town_visit_buildings` (from 0x5BD99A: the Castle's Stables, the Tower's Wall of
  Knowledge and the other buildings a hero visits once) is increment 6's.

## 16. VCMI 1.7.5, the wiki and the manuals compared (T42b)

- **VCMI's learning** (`CGameHandler::giveSpells`): a spell book, then each guild level up to
  the lesser of `mageGuildLevel` and `maxSpellLevel` (the bonus `MAX_LEARNABLE_SPELL_LEVEL`, base
  2 in `gameConfig.json`, Wisdom adding), the level's shown spells (`spellsAtLevel`): **the same
  set.** Its callers differ: a visit (`heroVisitCastle` from `CGTownInstance::onHeroVisit`: an own
  or allied town, an undefended one captured) teaches **the visitor only**; a player's build
  teaches both through its building visit (`buildStructure`'s end: the garrison hero first), a
  town event's build (`force`, `NewTurnProcessor`) only for a guild level, a Library or the
  Aurora and with no building visit; a bought book teaches its hero; a secondary
  skill changed while in a town teaches **at once** (`changeSecSkill`); a hire visits the town
  (`HeroPoolProcessor`, `objectVisited`); and **every player's turn start** makes each of its
  towns' garrison hero, then visitor, visit the town (`NewTurnProcessor::onPlayerTurnStarted`):
  learning and the buildings. The start teaches nothing (TD-10).
- **VCMI's book** (`buyArtifact`): Mage Guild 1, 500 gold of the hero's owner, none yet: the same
  rule (its client asks). Its computer player's purchases are its AI's.
- **VCMI's spell points** (`CGHeroInstance::getManaNewTurn`): a hero whose visited town (visitor or
  garrison) has Mage Guild 1 gets the greater of its points and its limit, else its regain to the
  limit: the same, no book tested.
- **VCMI's Mana Vortex** (`dungeon.json`'s `special2`, a rewardable building): visited once, reset
  every 7 days with its visitors, a limiter of mana below 200 % of the limit, the mana set to
  200 % of it (`Reward::calculateManaPoints`, `Limiter`): **the same effect and week.** Its
  triggers are the town's visits above: the turn start and a build with the garrison hero first,
  "consistent with original H3" by its comment. VCMI's bug 735 (an observation by a user) claims
  the visitor first at a build or the week's first opening; the reading, VCMI's code and the wiki
  give the garrison hero.
- **The wiki:** "Mage Guild": the spells taught, the points refilled when a hero stays overnight,
  the book for 500. "Mana Vortex" (read through search results; the page itself is not reachable
  from the session): twice the maximum, one hero a week, with two heroes in town only the
  garrison hero, so the bonus goes to it once the player opens the town menu; the AI never builds
  it.
- **The manuals** (RoE's): a visiting hero learns any new spells its Wisdom allows; its points are
  replenished after a day in a town with a guild; some heroes come with a book, others buy one at
  the guild; the Vortex doubles the spell points of visiting heroes.

## 17. Disputes, quirk candidates and open points (T42b)

- **TD-10 — a main town's hero learns at the start** (increment 1's, `E09-towns.md`'s table;
  `TWN-STUB-6`). The reading again: `main_town_heroes` 0x4CA1A2 teaches the visitor and the
  garrison hero. VCMI: nothing at the start. The wiki and the manuals: silent. 33 of 538 main
  towns' heroes (§10). Recommended: the original. Row TWN-5-4.
- **TD-27 — when a town's heroes learn and get the Mana Vortex.** The reading: at the triggers of
  §12 and §15, among them every opening of the owner's town screen (learning with a visitor, the
  Vortex to the garrison hero first); never at a human's turn start; a computer's hero standing
  on an entrance re-visits at each of its turns. VCMI: at every turn start (each town's garrison
  hero, then visitor), at once on a skill changed in town, besides its visits, builds, a bought
  book and a hire. The wiki: "once the player opens the town menu". Impact: a garrison
  hero's Vortex and a hero's new Wisdom wait for its owner's action in the original (free, at any
  time of its turn) and come by themselves at the turn start in VCMI; the engine has no town
  screen, so the original's needs a command of its own (T43's design). Recommended: the
  original's triggers, the screen's opening as a free command of the owner's. Row TWN-5-5.
- **Agent's calls** (change freely): the computer's purchase without asking, and its heroes'
  re-visit at each turn, are the computer player's (E10, E13), the purchase's rule the human's; the
  visit's level-up after its learning as read; the learning after a siege teaches whoever then
  stands in the town (the attacker stands outside until it enters).
- **Quirk candidates:** none (next `T-19`). Castle Gate's and the tavern click's building visit of a
  visitor of −1 (a null hero) cannot happen: the arriving or hired hero is the visitor.
- **Open for T43:** which engine commands teach (the visit, `Revisit`, the build, the hire, the
  garrison moves, the book's purchase, the start) and how TD-27's opening is modelled; the
  town's refill in the new day after HER-MAN-002 (GAM-CAL-002's order); the Vortex's week flag
  in the town state (a hash) and its message as an event; the book's purchase as a town command
  (the price, 500, in the part), who may buy (the reading: the visitor, else the garrison hero,
  the current player paying; an allied visitor too, as TD-11's question) and what the owner's
  view tells (`docs/06-security.md`); TD-25, TD-26, T-18 and the T43 points of §8.

## 18. Batch TWN-5's rows (§9's three with T42b's; run on the capture harness)

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| TWN-5-1 | TD-25 (§9 a) | an AB map, a town requiring and banning one level-1 spell, a level-1 guild built | the guild's five spells | the spell absent |
| TWN-5-2 | TD-26, T-18, §5 (§9 b) | a SoD map banning spell A map-wide; a Conflux banning spell B with the Grail building and a level-1 guild; a hero with a spell book and Expert Wisdom visits | the spells learned, by level | A learned, B not, every level to 5 |
| TWN-5-3 | §3.3, §4 (§9 c) | a Tower with a level-5 guild; the Library built in the game with a visitor | each level's count before and after; the visitor's spells | one more a level at once; the spare's spell learned by the visitor |
| TWN-5-4 | TD-10, §12 | an SoD map: red's main town a Tower with Mage Guild 2 built that generates a hero; red picks a hero with a spell book and no Wisdom | the hero's learned spells on day 1 before any action | the guild's level-1 and level-2 spells learned beside its own |
| TWN-5-5 | TD-27, §15 | red's Dungeon with Mage Guild 1 and the Mana Vortex, a garrison hero with a spell book; on a week's first day: the turn's start, then the town screen opened from the town list, then a second hero's visit; the next week the visitor and the garrison hero both there, the screen opened | the garrison hero's and the visitor's spell points at each step | unchanged at the turn's start; the garrison hero's doubled at the opening; the visitor's not that week; the next week the garrison hero's again, the visitor's not |

**Not rows:** the learning's set and Wisdom's bound (§11: VCMI agrees, the engine's HER-CHG-004
already observed by E08's batches), the book's purchase (read whole, VCMI agrees), the night's
refill (VCMI agrees), the Vortex's effect and week (VCMI and the wiki agree). **What the harness
may lack** (`docs/experiments/README.md`; checked at the batch's cut): the town screen opened
from the town list in a recipe; reads of a hero's spell points (+0x18) and learned spells
(+0x3EA) at a turn's start.

## 19. Not read (T42b)

The rest of `town_visit_buildings` (increment 6), `mage_guild_dialog_create`, the executive's run
of a manager (0x4B0C70), `hero_move_loop`'s two openings of the screen, Town Portal's and Castle
Gate's other steps (increment 7), the computer's planning around its visits (E10, E13).

## 20. The checkpoint (T42b; `TWN-Q31`, the user, 2026-11-20)

Asked interactively (`AskUserQuestion`, one round of four, a recommendation each; the Aurora's
question asked again in other words after the user found its first wording ambiguous); the
epic's decisions 167 to 170, each as recommended, and the agent's calls of 171:

- **TD-10** (decision 167): the original. A main town's hero with a spell book learns its town's
  guild spells at the start (the visitor and the garrison hero, `main_town_heroes`' call);
  `TWN-STUB-6` lifts when it is built; row TWN-5-4 observes it.
- **TD-25** (decision 168): the original. A spell the town both requires and bans is never in its
  guild (the ban wins, as the map-wide bans do); row TWN-5-1.
- **TD-26 with T-18** (decision 169): the original's set with **T-18 `fix-optional`**. The Aurora
  never teaches a spell the town bans (nor 57); a spell the map bans is taught in `faithful` and
  not in `strict-fixed`. The flag waits for its confirmation (`docs/04-rules-and-sources.md`,
  "Confirm before flagging": row TWN-5-2, or a second look at 0x5BE10C to 0x5BE172); over the
  original without a flag, both bans respected without a flag, and VCMI's set.
- **TD-27** (decision 170): the original's triggers (§12, §15) **with a free command of the
  owner's that stands for opening the town screen** (the garrison hero, then the visitor visit the
  town's buildings; with a visitor both learn), its shape T43's; over VCMI's visits at every turn
  start and over the original's triggers without the opening.
- **The agent's calls** (decision 171, change freely): §17's (the computer's purchase and its
  heroes' re-visits the computer player's, E10 and E13; the visit's level-up after its learning;
  a siege's learning by whoever then stands there); batch TWN-5's five rows (§18) for T43's cut,
  run as TWN-1 to TWN-4 were.
