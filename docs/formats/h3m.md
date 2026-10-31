# Map files (`.h3m`)

Read by `h3-map` (E08 T3, adventure lane) and, for staged experiments, written by `h3_map::writer`
(E04a T2, OPEN-QUESTIONS Q28; in `h3-formats` until the content split, R01a). Sources: the field order
and sizes of VCMI 1.7.5's map reader (`lib/mapping/MapFormatH3M.cpp`, `MapReaderH3M.cpp`,
`MapFeaturesH3M.cpp`, `lib/mapObjects/ObjectTemplate.cpp`;
https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapping/MapFormatH3M.cpp), checked against the maps
that ship with the user's installation and maps the user made in the editor. Everything is
little-endian; a string is a `u32` length plus Windows-1252 bytes; a bool is a byte (non-zero =
true); a position is x, y, level (one byte each); a map file is one gzip member (RFC 1952) of the
bytes below.

## Reading: the three versions (E08 T3)

| Part | RoE `0x0E` | AB `0x15` | SoD `0x1C` |
|---|---|---|---|
| version, any players, size, underground | u32, u8, u32, u8 | same | same |
| name, description, difficulty | string, string, u8 | same | same |
| level limit | — | u8 | u8 |
| 8 players | below | below | below |
| victory, loss | below | same | same |
| teams | u8 count; if > 0, a team byte per colour | same | same |
| allowed heroes (bit = allowed) | 16 bytes | 20 bytes | 20 bytes |
| campaign placeholders | — | u32 count, a hero byte each | same |
| disposed heroes | — | — | u8 count; hero, portrait, name, players mask |
| map options | 31 bytes (zero) | same | same |
| banned artifacts (bit = banned) | — | 17 bytes | 18 bytes |
| banned spells, skills (bit = banned) | — | — | 9, 4 bytes |
| rumors | u32 count; name, text | same | same |
| customised hero types | — | — | 156 × (bool; if set, the body below) |
| terrain | 7 bytes per tile, level by level, row by row | same | same |
| object templates | u32 count; template below | same | same |
| objects | u32 count; position, u32 template index, 5 unused bytes, the body of the template's class | same | same |
| timed events | u32 count; timed event below | same | same |
| padding | 124 zero bytes in every shipped map; the reader accepts zeros only | same | same |

**Player.** Human, computer (bools). Nobody plays the colour: 6 (RoE), 12 (AB) or 13 (SoD) unused
bytes follow. Otherwise: AI tactic (i8); SoD only, the town-choice byte; allowed towns (u8 in RoE,
u16 from AB on); random town (bool); main town (bool; if set, from AB on a generate-hero bool and a
town-type byte, then its position; RoE always generates the hero); random hero (bool); main hero
type (`0xFF` none; else portrait byte and name); from AB on an unknown byte and the hero list
(u32 count; hero type byte and name each).

**Victory** (u8; `0xFF` standard): allows standard victory, applies to AI (bools), then by code:
0 artifact (u8 in RoE, u16 from AB), 1 creature (same width) and an i32 count, 2 resource u8 and
an i32 amount, 3 town position, hall level, castle level, 4 grail town, 5 hero, 6 town, 7 monster
(positions), 8 dwellings, 9 mines (nothing), 10 artifact u8 and town position. Codes 11 and 12
are HotA's and rejected. **Loss** (u8; `0xFF` standard): 0 town, 1 hero (positions), 2 days (u16).

**Customised hero type** (SoD; also the core of the hero object's body, T3b): experience (bool +
u32), secondary skills (bool + u32 count + skill and level bytes), artifacts (bool; one code per
slot, 18 before SoD, 19 in SoD, "none" `0xFF` or `0xFFFF`; u16 backpack count and codes), biography
(bool + string), sex (byte, `0xFF` default), spells (bool + 9 bytes), primary skills (bool + 4
bytes).

**Tile:** terrain (0 dirt … 9 rock), picture, river (0 none … 4 lava), river picture, road (0 none
… 3 cobblestone), road picture, flags. The reader keeps every code raw; the import maps them.
Flag `0x40` marks the coast: in the game's own maps (the installation's, E04c T3: 164 then, four of them the user's own) every
land tile with water among its eight neighbours has it and no other tile; the game takes the coast
from it (a map without it has no shore battlefield, E04c batch 3b). VCMI 1.7.5 ignores the bit and
looks at the same eight neighbours (`CMap::isCoastalTile`).

**Template:** name (string), passability and visitability (6 bytes each, bit set = passable or
visitable; bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the
bottom right corner, the object's position: VCMI `ObjectTemplate::readMap`; corrected in E08 T5a,
the notes had it mirrored, which towns showed: their gate sits on their bottom row),
two terrain masks (u16 each; our writer reads the second as the editor's menu, VCMI as the allowed
terrains), class and subclass (u32 each), group and priority (bytes), 16 unused bytes.

**Shared parts of the bodies.** An *id* (artifact or creature code) is one byte in RoE and two
from AB on; "none" is all bits set. An *army* is 7 × (creature id, u16 count). A *message* is a bool;
if set, a string, a has-guards bool with an army, and 4 unused bytes. *Rewards* (events, Pandora's
boxes): u32 experience, i32 mana, i8 morale, i8 luck, 7 × i32 resources, 4 primary-skill bytes,
then u8-counted lists of (skill, level) bytes, artifact ids, spell bytes and (creature id, u16)
stacks, 8 unused bytes. A *timed event*: name and message strings, 7 × i32 resources, a players
byte, SoD's human bool, a computer bool, u16 first day, u16 interval, 16 unused bytes.

**Object bodies by class** (VCMI 1.7.5's `readObject`; HotA's branches left out):

| Class | Body |
|---|---|
| 26 event | message, rewards, players byte, computer and remove-after-visit bools, 4 unused |
| 34 hero, 62 prison, 70 random hero | from AB a u32 identifier; owner, hero type; name (bool + string); experience (SoD: bool + u32; before: u32, 0 = default); portrait (bool + byte); skills (bool + u32 count + pairs); army (bool + army); formation; artifacts (as the customised hero); patrol byte; from AB biography (bool + string) and sex; AB one spell byte, SoD spells (bool + 9 bytes) and primary skills (bool + 4 bytes); 16 unused |
| 54 monster, 71–75 and 162–164 random monsters | from AB a u32 identifier; u16 count (0 random); disposition; treasure (bool; string, 7 × i32, artifact id); never-flees and no-growth bools; 2 unused |
| 59 bottle, 91 sign | string, 4 unused |
| 83 seer hut | RoE: an artifact id (none = no quest); from AB a quest (below); with a quest a reward (code 0–10: nothing, u32 experience, u32 mana, i8 morale, i8 luck, resource byte + u32, primary skill + byte, skill + level, artifact id, spell byte, creature id + u16), without one 1 unused byte; then 2 unused |
| 215 quest guard | a quest: mission code 0 none (nothing follows), 1 level u32, 2 primary skills 4 bytes, 3 hero identifier u32, 4 monster identifier u32, 5 u8-counted artifact ids, 6 u8-counted stacks, 7 resources 7 × u32, 8 hero type, 9 player; then i32 deadline and three strings (first visit, next visit, completed) |
| 113 witch hut | from AB the possible skills (4 bytes) |
| 81 scholar | i8 bonus kind, bonus byte, 6 unused |
| 33, 219 garrison | u32 owner, army, from AB a removable bool, 8 unused |
| 5 artifact, 65–69 random artifacts | message |
| 93 spell scroll | message, u32 spell |
| 76, 79 resource | message, u32 amount, 4 unused (the experiment writer's class-79 piles: no message, E08 T16b) |
| 77, 98 town | from AB a u32 identifier; owner; name (bool + string); garrison (bool + army); formation; buildings (bool: built and forbidden, 6 bytes each; else a has-fort bool); from AB the required spells (9 bytes); the banned spells (9 bytes, every version); u32-counted town events (timed event, 6 building bytes, 7 × u16 creatures, 4 unused); SoD's alignment byte; 3 unused |
| 53, 220 mine | subclass below 7: u32 owner; else (abandoned) 4 bytes of possible resources |
| 17–20 dwellings, 42 lighthouse, 87 shipyard | u32 owner |
| 216–218 random dwellings | u32 owner; 216 and 217 a u32 town identifier and, when 0, the faction set; 216 and 218 the lowest and highest level bytes |
| 88–90 shrines | u32 spell |
| 6 Pandora's box | message, rewards |
| 36 grail (subclass below 1000) | i32 radius |
| 214 hero placeholder | owner, hero type; for type `0xFF` a power rank byte |
| every other class | nothing |

**Input bounds** (`h3_map::Limits`; above one the file is rejected before anything is
allocated, and every count must also fit the bytes left). Peaks in the 160 maps shipped with the
game (49 RoE, 55 AB, 56 SoD, `Tutorial.tut` included; E08 T21b, 2026-10-19, the same peaks as the
164 of 2026-10-10, four of which were the user's own; `cargo xtask verify --with-original-data`,
which runs `crates/h3-map/examples/original_data.rs`;
every one reads to its padding, and every body kind but the hero placeholder occurs):

| Limit | Default | Shipped peak |
|---|---|---|
| file (compressed) | 4 MiB | 123 343 bytes |
| map (inflated) | 16 MiB | 525 124 bytes |
| map side | 144 (two levels) | 144 |
| string | 65 536 bytes | 1 314 |
| heroes per player | 64 | 6 |
| campaign placeholders | 156 | 0 |
| rumors | 256 | 19 |
| secondary skills per hero | 28 | 8 |
| backpack | 256 | 2 |
| object templates | 4 096 | 841 |
| objects | 65 536 | 7 009 |
| timed events (map or town) | 1 024 | 100 (a town: 37) |

Versions the reader rejects: Heroes Chronicles (`0x1D`), HotA (`0x20`), WoG (`0x33`) and every
other code.

## Import (E08 T5a, T6a, T12a, T13a, T18b, T19a, T20a2a, T21a, T29, T49a, T49b, T54, T61a, T61b, T72, T83a; `MAP_IMPORT_VERSION` 15)

`h3_map::import` turns a raw map and the game's rules into `h3_game::GameSetup`, which the game
then validates (ADR-0015 point 4); `import_map` is its static layer. The import maps codes to
content and draws nothing; it rejects what the current increment cannot represent with a typed
`Unsupported` error and checks no game invariant. A replay names its map by content hash, path and
`MAP_IMPORT_VERSION` (ADV-Q2): any change to what an import yields bumps it.

- **Players:** the colours whose human or computer flag is set, in colour order; a player's
  `PlayerId` is its index there; since v11 (E08 T54, GAM-BAT-010, ADV-Q27) a colour a human may
  play counts as human (`PlayerSetup.human`), a computer-only one does not. Unsupported: a playing
  colour whose main town generates its hero (RoE always does; towns are E09).
- **The scenario's level limit and banned skills** (v8, E08 T29, GAM-SET-002): the header's level
  limit (AB and SoD; 0 none, always 0 in RoE) and SoD's banned skills, each set bit the skill of
  its code (`SSTRAITS.TXT` order; a set bit past the 28 codes is an unknown code; none in the
  shipped maps).
- **The scenario's banned spells and artifacts** (v13, E08 T61b, GAM-SET-003): SoD's banned
  spells, each set bit the spell of its code (none past the 70 hero spells in the shipped maps),
  and AB's and SoD's banned artifacts, each set bit among the edition's artifacts (127, 129, 144)
  that the codes name (141) the artifact of its code; the bits past them ban nothing: an AB set's
  7 padding bits and the original's three unused records 141 to 143, which the editor bans by
  default on a SoD map (123 such bits on 41 shipped maps). The game passes them, with the banned
  skills, to the adventure's build, whose draws give none of them (design §9).
- **Difficulty** (v3): `h3_game::DEFAULT_DIFFICULTY`, normal. The game's difficulty is chosen when a
  game starts; the header's difficulty byte is the map's rating and is not read into the setup
  (survey `E08-objects.md` §7.2).
- **Static map:** the side and levels from the terrain block; terrain codes 0–9 → `base:dirt` …
  `base:rock`, roads 1–3 → `base:dirt_road`, `base:gravel_road`, `base:cobblestone_road`, rivers
  1–4 → `base:clear_river` … `base:lava_river` (0 is none); since v5 (E08 T19a) the coast flag
  `0x40` as the tile's `coast`, which a battle on the map reads (GAM-BAT-001). Every object but the heroes keeps its
  template's masks: blocked where the passability bit is clear, visitable where the visitability
  bit is set (ADV-OBJ-001). An anchor may lie past the right or bottom edge: the shipped maps
  anchor objects up to 5 columns right and 2 rows below it; tiles off the map are ignored.
- **Object types** (v2, E08 T12a, ADV-OBJ-002; `crates/h3-map/src/import/codes.rs`): class 79
  by subclass 0–6 → `base:pile_wood`, `_mercury`, `_ore`, `_sulfur`, `_crystal`, `_gems`,
  `_gold`; 76 → `base:random_resource`; 12 → `base:campfire`; 13 by subclass 0–2 →
  `base:cartographer_water`, `_land`, `_subterranean`; 58 → `base:redwood_observatory`; 60 →
  `base:pillar_of_fire`; 10, 9 and 212 by subclass (the key colour: 0 light blue, 1 green, 2 red,
  3 dark blue, 4 brown, 5 purple, 6 white, 7 black) → `base:keymaster_<colour>`,
  `base:border_guard_<colour>`, `ab:border_gate_<colour>`. A subclass outside these is an error
  (`Subclass`); classes 12, 58, 60 and 76 ignore it. A resource body's amount (76, 79) is the
  pile's fixed amount (0 is none: drawn when the game is built; gold counts in hundreds, the type
  multiplies), which the setup bounds at 0x7FFFF; its message's text is presentation. Every other
  class is a blocker. Since v12 (E08 T61a, increment 7; increment 7's design note §7 in `docs/epics/E08-log.md`
  until T61a, then `E08-log.md`) 31 more classes, each to one type: 101 → `base:treasure_chest`,
  5 → `base:artifact` (its subclass the artifact), 65–69 → `base:random_artifact`,
  `_treasure_artifact`, `_minor_artifact`, `_major_artifact`, `_relic_artifact`, 93 →
  `base:spell_scroll`, 4 `base:arena`, 23 `base:marletto_tower`, 32 `base:garden_of_revelation`,
  41 `base:library_of_enlightenment`, 47 `base:school_of_magic`, 51 `base:mercenary_camp`, 61
  `base:star_axis`, 100 `base:learning_stone`, 102 `base:tree_of_knowledge`, 107
  `base:school_of_war`, 88–90 → `base:shrine_of_magic_incantation`, `_gesture`, `_thought`, 113
  `base:witch_hut`, 81 `base:scholar`, 104 `base:university`, 22 `base:corpse`, 39
  `base:lean_to`, 105 `base:wagon`, 108 `base:warriors_tomb`, 6 `base:pandoras_box`, 26
  `base:event`, 63 `base:pyramid`; every subclass of these but 5's is ignored. Since v11 (E08 T54, GAM-BAT-011) every object keeps its scenery by class
  (`ObjectSetup.scenery`, `codes::scenery`): trees 114, 115, 135, 137, 155, 159, 160, mountains
  123, 133, 134, 136, 157, 158, dead vegetation 119, ownable 17, 20, 33, 42, 53, 77, 87, 98.
- **Guarded piles** (v7, E08 T21a, ADV-PIK-004): a pile's or random resource's guards record gives
  its guards (`ObjectSetup.guards`): the stacks by slot (creature by original id, the count) and
  whether the message has a text (the visit asks first; the text itself is not kept). A record
  without stacks is a plain pile, as the original's stack count reads it (0x4A47E0). Unsupported:
  a random stack among the guards (the 14 codes below "none", as a hero's); `import_map` keeps
  that pile unguarded. The setup checks the stacks as a hero's army, no war machine among them.
  Since v12 (E08 T61a) an artifact's (placed or random), a scroll's, a Pandora's Box's and an
  event's message record give guards the same way: an artifact's always ask (D-29), a scroll's
  only with a text (as a pile's), a box's and an event's never (announced).
- **What increment 7's objects hold** (v12, E08 T61a; `ObjectSetup.holds`, `h3_adventure::Holds`,
  checked against the type's kind when the game is built): a placed artifact (5) its subclass's
  artifact; a scroll (93) the Spell Scroll (artifact code 1) with the spell of its u32's low byte;
  a Shrine (88–90) its spell, 255 none (drawn when the game is built); a Witch Hut (113) its mask's
  skills, each set bit the skill of its code (a bit past the 28 codes an unknown code), an empty
  mask every skill (the original's new game, 0x4C207D), none on an RoE map, which stores no mask
  (its type's skills by the profile, A-21); a Scholar (81) its gift, kind −1 none (drawn), 0 the
  primary skill of its code (0–3, else an unknown code), 1 the skill, 2 the spell, another kind an
  unknown code; Pandora's Box (6) and an event (26) a reward record (`RewardRecord`): the
  experience's u32 as it is (2³¹ and above gives none), the primary skills' bytes as signed, the
  mana, morale and luck, the skills (code, level), artifacts and spells by code, the resources that
  are not 0 (negative a loss), the creatures by slot in the list's order; an event also its colours,
  computer flag and removal flag. Unsupported: a record's creature list longer than an army's
  slots (A-24, always fixed) or with a random code; `import_map` keeps the record without its
  creatures. The other classes hold nothing: their contents are drawn when the game is built
  (T61b).
- **Monsters** (v6, E08 T20a2a, ADV-MON-001): class 54 → `base:monster` with the creature its
  subclass names (by original id, as a hero's stacks; 122, 124, 126 and 128 are unknown codes);
  71 → `base:random_monster`, 72–75 → `base:random_monster_1` to `_4`, 162–164 →
  `base:random_monster_5` to `_7`, their creature drawn when the game is built (ADV-MON-003).
  The body's count (0 drawn, ADV-MON-004; the setup bounds it at 4 095, the original's 12 bits),
  disposition code (0 compliant to 4 savage, a row of the monsters' aggression, ADV-MON-002),
  never flees and does not grow; the treasure's resources above 0 in resource order (its text is
  presentation; no shipped map has one below 0). Until v12 a treasure with an artifact was
  unsupported (E08 decisions 22 and 48); since v12 (E08 T61a) its artifact is kept
  (`MonsterSetup.artifact`; the setup takes one a hero may hold, without a spell).
- **Special grounds** (v6, E08 T20a2a, ADV-GRD-001): 21 → `base:cursed_ground`, 46 →
  `base:magic_plains`, 222–224 and 226–231 → `sod:clover_field`, `sod:cursed_ground`,
  `sod:evil_fog`, `sod:fiery_fields`, `sod:holy_ground`, `sod:lucid_pools`, `sod:magic_clouds`,
  `sod:magic_plains`, `sod:rocklands`; 225 (Favorable Winds) stayed a blocker without tiles
  until v14. A ground lies on its picture's width × height tiles to the left of and above its anchor (its
  templates block and are visited nowhere): the size of the picture the template names, from
  `codes::GROUND_PICTURES` (the first two bytes of each picture's `.msk` in `H3sprite.lod` and
  `H3ab_spr.lod`, read 2026-10-15; the original places every object on its `.msk`'s rectangle,
  0x514610, 0x505B20; VCMI `ObjectTemplate::readMsk` the same), else 1 × 1 (the original's
  `default.msk`).
- **The sea** (v14, E08 T72, increment 8's design note §4 in `docs/epics/E08-log.md`): a boat (class 8) is no object
  but one of the map's boats (`MapSetup::boats`, ADV-SEA-001): its template's one visitable tile
  (else `BoatTile`) and its subclass 0–2 as its type (else `Subclass`), no body (the original's
  loader reads none); the setup refuses a boat off water, on a tile an object blocks or is
  visited from, or on another boat's (`SetupError::Boat`). 29 → `base:flotsam`, 82 →
  `base:sea_chest`, 86 → `base:shipwreck_survivor`, 59 → `base:ocean_bottle` (its message read and
  dropped: the text is the client's), 124 → `base:hole`, 225 → `sod:favorable_winds`; a hole and
  the winds lie on their pictures' tiles as a ground (`GROUND_PICTURES` gains the eight holes',
  every one 1 × 1, and the winds' eight, 6 × 4, 8 × 6, 6 × 6, 8 × 3, 3 × 6, 3 × 3, 5 × 2 and
  2 × 5 for `AVXfw0` to `AVXfw7`, read by E08 T67). Unsupported: a hero object on water
  (`HeroOnWater`, ADV-SEA-007, D-44). The generated maps' blockers (`h3-map`'s `tests/play.rs`)
  moved from class 124 to 118 (a crater), which stays a blocker.
- **Increment 9's objects** (v15, E08 T83a, increment 9's design note §4; ADV-OBJ-002): 53 by
  subclass, 0 to 6 the mines by resource (`base:sawmill`, `alchemists_lab`, `ore_pit`,
  `sulfur_dune`, `crystal_cavern`, `gem_pond`, `gold_mine`), 7 `base:abandoned_mine` (a subclass
  past 7 is `Subclass`), 220 (SoD) `base:abandoned_mine` whatever its subclass (the original's
  53.7), 42 `base:lighthouse`, 87 `base:shipyard`, 112 `base:windmill`, 109 `base:water_wheel`,
  55 `base:mystical_garden`; 220 is ownable scenery as 53 (GAM-BAT-011). A mine's, Lighthouse's
  and Shipyard's owner byte (`Body::Mine`, `Body::Owned`): `0xFF` none, else the player of that
  colour, a colour nobody plays (or past the eight) refused (`ObjectOwner`, ADV-OWN-002); an
  abandoned mine's set (`Body::AbandonedMine`): the first byte's bits 0 to 6 as the resources it
  may draw (`Holds::Resources`, in the core's order; the other three bytes dropped, as the
  original's loader), an empty set or bit 7 refused (`MineResources`, A-31, ADV-MIN-003).
  Unsupported: an ownable object whose entrance is on water (`OwnedOnWater`). **The map's timed
  events** (`RawMap::events`) become `GameSetup::events` in the map's order (GAM-CAL-004): the
  players of their colours that play (a colour nobody plays left out), the human and computer
  flags as read, the file's first day + 1, the interval, the resources without zeros, the index
  for the message; none refused (more than a `u16` names is `Events`). The census's static layer
  is validated in a game of the map's players (an owner is one).
- **Placed heroes** (class 34): owner colour → player (a colour nobody plays is an error), hero
  type by original id → its key (the packs' `# original id`), the tile its template marks
  visitable (the editor's one column left of the position), the army's stacks by slot (creature by
  original id, the count: its 16 bits whole, where the original loads 32 768 or more as no
  creature, ADV-QUIRK-19; above the ruleset's most creatures in a stack the setup is invalid,
  ADV-ARM-007, E08 T50) or, when the map gives no stack, the hero type's starting army as a
  placeholder that the game draws when it is built (E08 T6b; VCMI `CGHeroInstance::initHero`).
  Unsupported: a random hero (class 70), a campaign placeholder (214), a random stack (the 14 codes
  below the format's "none"), a hero standing on a town's blocked tile (VCMI moves it to its own
  town's entrance, an "H3M format quirk", `CGameState::placeHeroesInTowns`; towns are E09).
  A hero on the visitable tile of an object visited from a neighbouring tile (a pile, a campfire,
  a border guard) imports but fails validation (ADV-MAP-001, E08 T14a); no shipped map has one.
- **A placed hero's details** (v4, E08 T18b, GAM-HER-001): the formation (0 loose, 1 tight; the
  original never applies it, every hero starts loose: kept here, `ADV-QUIRK-8`) and
  `h3_game::HeroDetailsSetup`, each field the body's, else its hero type's map-wide entry's
  (SoD's customised hero types), else none (the game's default): experience (before SoD 0 is
  the default; SoD's flag makes 0 an experience; since v8 passed as it is, the game gives its
  levels at load, GAM-SET-002), primary skills (clamped to the hero parameters' bounds, 0–99 with
  Spell Power and Knowledge at least 1, as the original's carried-over hero's setup 0x486590;
  the placed hero's, `placed_hero_setup` 0x4D8B30, copies them as they are, and no shipped map has
  one above 9: survey `E08-progression.md` §3.1, §6.1),
  secondary skills (skill code by `SSTRAITS.TXT` order, the level as stored, 1–3), artifacts (since
  v9, E08 T49a, each worn entry at its position, ADR-0013 point 1: the map's entries 0–12 (head
  to the fourth misc) are the content's 0–12, its 13–17 (the four war machines, the spell book)
  the content's 14–18 and SoD's 18 (the fifth misc) the content's 13, each the entry's slot and
  rank among that slot's positions (`codes::WORN`), equipped in the map's order only where the
  position takes it (`h3_heroes::equip`: empty, of the artifact's slot, the locks fitting), else
  left out, as the original's loader (`placed_hero_setup` 0x4D8D63, `hero_equip_artifact`
  0x4E2A00 refusing what `hero_can_place_artifact` 0x4E2550 refuses, the fifth misc before SoD)
  and VCMI's (`loadArtifactToSlot`): the Tutorial's second hero's Shackles of War, stored on the
  left hand, is dropped; with RoE's rules, which have four misc positions, so is an artifact on
  the fifth; since v10 (E08 T49b, HER-ART-007) an equipped Titan's Thunder adds a Spell Book on
  the empty book position, as the original's equip (0x4E2A64), so the map's own book entry,
  later in its order, finds the position taken (no shipped map's hero wears or carries the
  Thunder); the backpack in order as before; codes by the packs' `# original id`, 141–143
  unused), spells (SoD's 9-byte set, spell `i` at bit `i mod 8` of byte
  `i / 8`; AB's one byte: `0xFF` none, `0xFE` the type's own, else that spell beside the type's).
  The validation (`h3_heroes::validate`) checks the rest, so a war machine in the backpack or a
  scroll without its spell fails it (no shipped map that imports has one). Until v8 an experience
  of 1 000 or more was unsupported (E08 decision 22). Ignored: name, portrait, patrol (a computer
  player's), biography; since v11 (E08 T54, GAM-BAT-012) the sex is kept (0 male, 1 female,
  any other the type's; the body's over the entry's).

The maps of the user's installation (`crates/h3-map/examples/original_data.rs`): until
2026-10-18 the folder held 164, four of them the user's own (`experiment`, `Untitled` and edited
copies of `All for One` and `Terrible Rumor`), and the runs up to T21a's below counted them; the
160 shipped are counted since T21b's run (the last of the paragraph). 2026-10-12, import v2: every static layer imports and validates; 9 import and
validate whole (`Adventures of Jared Haret`, `Faeries`, `Knee Deep in the Dead`, `Pandora's Box `,
`Race for Ardintinny`, `Tutorial.tut`, `Unholy Quest` and the user's `Untitled` and
`experiment`); 129 are unsupported for a hero generated at a main town, 18 for a random hero, 5 for
a guarded resource pile (`Myth and Legend` among them, whole in v1), 3 for a hero standing in a town
(the first reason found counts: players, the static layer, then the heroes). E08 T14a's run (import v3,
the same outcomes): no tile of a static layer is the visitable tile of two or more objects (0 of
101 246 visitable tiles, heroes left out; which object such a tile visits: ADV-OBJ-003). E08
T18b's run (import v4): 5 import and validate whole (`Pandora's Box ` with 27 customised heroes,
`Race for Ardintinny`, `Tutorial.tut`, `Untitled`, `experiment`); `Adventures of Jared Haret`,
`Faeries`, `Knee Deep in the Dead` and `Unholy Quest` are unsupported for a hero with an
experience of 1 000 or more (4); the other outcomes as before. E08 T20a2a's run (import v6): 4
import and validate whole (`Race for Ardintinny`, `Tutorial.tut`, `Untitled`, `experiment`); 8
are unsupported for a monster with an artifact (`Pandora's Box ` among them, whole before), 4 for
a guarded pile, 129 for a hero at a main town, 15 for a random hero, 3 for a hero's experience, 1
for a hero in a town. The census: 14 021 monsters, 357 with a treasure (55 with an artifact, none
with a resource below 0), no disposition beyond savage; 1 887 special grounds, every picture in
the table; 10 939 tiles under two or more grounds, 142 of them (in 9 maps) under grounds of
different battlefields, where the last in map order counts (ADV-GRD-001); 5 maps stack two grounds
of one type on one anchor, which the setup allows for grounds only. E08 T21a's run (import v7): the
same 4 import and validate whole; the 4 maps unsupported for a guarded pile before now count for
a random hero (19), the other outcomes as before. 39 piles have a guards record: 3 without a
stack (plain), 36 guarded (30 ask first, 9 with two stacks), none with a random stack. E08
T21b's run (import v7, the 160 shipped maps): 2 import and validate whole (`Race for Ardintinny`,
`Tutorial.tut`); 128 are unsupported for a hero generated at a main town, 18 for a random hero, 8
for a monster with an artifact, 3 for a hero's experience, 1 for a hero in a town. The census:
13 863 monsters, 355 with a treasure (53 with an artifact, none with a resource below 0), none
beyond savage; 1 861 special grounds, every picture in the table, 10 771 tiles under two or more
(the 142 tiles under grounds of different battlefields are T20a2a's count over the 164); 7 typed
objects of one type on one anchor in 5 maps, all grounds (`Back For Revenge`, `Carpe Diem` and
their allied versions, `Deluge`); no tile visited from two objects (0 of 100 311 visitable tiles);
the 39 guards records as T21a's. E08 T29's run (import v8): 5 import and validate whole
(`Adventures of Jared Haret`, `Faeries`, `Knee Deep in the Dead`, `Race for Ardintinny`,
`Tutorial.tut`; survey `E08-progression.md` §6.5), their 9 heroes levelled at load with 41 levels;
128 are unsupported for a hero generated at a main town, 18 for a random hero, 8 for a monster with
an artifact, 1 for a hero in a town. No map has a level limit; 19 ban skills, none past the 28
codes. E08 T49a's run (import v9): the same 5 import and validate whole with the same levels
at load; the Tutorial's second hero leaves its Shackles of War out (stored on the left hand,
which the first version of v9 refused to validate); the other outcomes as T29's. E08 T61a
(import v12, the 31 classes and the monsters' artifacts): its run was not made (the user's computer
was not reachable that session). E08 T61b's run (import v12, then v13 with the bans; the 160
shipped maps): every static layer imports and validates; 8 import and validate whole (`Adventures
of Jared Haret`, `Faeries`, `Knee Deep in the Dead`, `Myth and Legend`, `Pandora's Box `, `Race
for Ardintinny`, `Tutorial.tut`, `Unholy Quest`): of the 8 maps refused for a monster's artifact
before, 3 import whole, 3 stop at a random hero and 2 at a hero in a town; 128 are unsupported for
a hero generated at a main town, 21 for a random hero, 3 for a hero in a town. The monsters' and
grounds' census as T21b's. The guarded objects by kind (T61a's guards): 36 piles (30 ask first, 9
with two stacks, the 39 guards records as before), 33 artifacts (all ask, 18 with two or more
stacks), 3 scrolls (all ask: each has a text), 12 boxes and 87 events (none asks). 26 maps ban
spells (210 bits), 111 ban artifacts (1 061 bits); every map that imports whole validates with its
bans. E08 T72's run (import v14, the 160 shipped maps): every static layer imports and validates,
its boats included; the same 8 import and validate whole, the other outcomes as T61b's; no hero
on water. The sea's census: 118 boats on 41 maps (types 0, 1, 2: 20, 51, 47); 2 019 Flotsam,
826 Sea Chests, 362 Shipwreck Survivors, 29 Ocean Bottles; 732 holes and 154 Favorable Winds,
every winds picture in the table; the special grounds' census as T21b's (1 861, every picture in
the table); 9 typed objects of one type on one anchor in the same 5 maps (grounds, holes and
winds, which the setup allows). E08 T83a's run (import v15, the 160 shipped maps): every static
layer imports and validates in a game of its map's players; the same 8 import and validate
whole, the other outcomes as T61b's; no owner nobody plays, no empty or out-of-range abandoned
mine set, no ownable object entered from water. Increment 9's census: 5 915 mines (60 owned at
load), 158 abandoned mines (7 with one resource, 151 with two to six, none with all seven), 83
Lighthouses (none owned), 331 Shipyards (1 owned), 780 Windmills, 430 Water Wheels, 286 Mystical
Gardens; 666 timed events, 61 repeating, none for no player that plays. E08 T83b1's run (the
static layer built as a game builds it, adventure state hash v18): 61 objects owned at the start,
every abandoned mine drawn within its set and 100 to 200, every Shipyard with a boat tile but
Hoard's at (62, 38, 0) in both Hoard maps (survey `E08-economy.md` §1), every weekly object's
first week its kind's, every owner checked.

## The subset we write (E04a T2)

### Order of a SoD map (version `0x1C`), as we write it

| Part | Bytes | Our value |
|---|---|---|
| version | u32 | `0x1C` |
| any players, size, underground | u8, u32, u8 | 1, 36/72/108/144, 0 or (E08 T22a1) 1 |
| name, description | string, string | |
| difficulty, level limit | u8, u8 | 1 (normal), 0 or (E08 T32a) the map's limit |
| 8 players | see below | |
| victory, loss condition | u8, u8 | `0xFF`, `0xFF` (standard) |
| teams | u8 | 0 |
| allowed heroes | 20 bytes, bit = allowed | the editor's default: all but hero 4 and the campaign heroes 145–155 |
| campaign placeholders | u32 | 0 |
| disposed heroes | u8 | 0 |
| map options | 31 bytes | zero |
| banned artifacts, spells, skills | 18, 9, 4 bytes, bit = banned | the editor's default (artifacts 127–143), none, none |
| rumors | u32 | 0 |
| customised hero types | 156 × bool | all 0 |
| terrain | 7 bytes per tile, level by level, row by row | type, picture, river ×2, road ×2, flags |
| templates | u32 count + records | name, 6 bytes passable, 6 bytes visitable, u16 terrains, u16 editor terrains, u32 class, u32 subclass, u8 group, u8 priority, 16 zero bytes |
| objects | u32 count + records | x, y, z (u8 each), u32 template index, 5 zero bytes, body; the surface's objects, then the underground's |
| timed events | u32 | 0 |

**Player** (playable): human, computer (bools), AI tactic (i8), town choice offered (u8), allowed
towns (u16 bit mask), random town (bool), has main town (bool; 0 here), random hero (bool), main
hero type (u8, `0xFF` none; if set: portrait u8 `0xFF` and a name string), one unknown byte (0), and
the list of the player's heroes (u32 count, then hero type u8 + name string). A colour nobody plays
is 15 bytes (two zero bools and 13 bytes the game skips; the editor writes an empty player there).

**Template masks:** `Objects.txt` gives 48 characters per mask, the anchor first: character
`8r + c` is the tile `c` columns left of and `r` rows above the bottom right corner (VCMI
`ObjectTemplate::readTxt`); in the file, bit `j` of byte `i` is character `47 − (8i + j)`. Hero templates are not in `Objects.txt`:
the editor writes `ahNN_e.def` (NN = class), class 34, subclass = class, group 3, terrains `0xFF`,
masks blocked/visitable at one tile, which sits one column *left* of the object's position — so a
hero standing on (x, y) is written at (x + 1, y).

**Hero body** (class 34): u32 identifier, owner u8, hero type u8, then optional parts each behind a
bool: name (string), experience (u32), portrait (u8), secondary skills (u32 count, then skill u8 +
level u8), army (7 × (u16 creature, u16 count), `0xFFFF` empty); formation u8; artifacts (19 × u16
by slot, `0xFFFF` empty, then u16 backpack count and u16 ids); patrol radius u8 (`0xFF` none);
biography (bool + string); sex i8 (`-1` default); spells (bool + 9-byte mask); primary skills (bool
+ 4 × u8); 16 zero bytes.

**Monster body** (classes 54, 71, 162–164): u32 identifier, u16 count (0 = random), disposition u8
(0 compliant … 4 savage), message and treasure (bool; 0 here, since E08 T56 a treasure: its text,
seven i32 resources, the artifact u16 or `0xFFFF`), never flees (bool), does not grow (bool), 2
zero bytes.

**Terrain pictures** of interior tiles (the eight neighbours on the level have the tile's terrain),
counted over 164 maps (E08 T10a; four of them the user's own): dirt 21–44, sand 0–23, grass, snow, swamp, rough,
subterranean and lava 49–72, water 21–32, rock 0–7; in each range the first eight pictures are
about ten times as common as the others. The other pictures are the edges between terrains. The
experiment writer (`h3_map::writer`) gives every tile one of the first eight interior pictures of
its terrain, edges included, which the game draws as a hard edge.

**Road pictures** of a straight east–west road (counted over the same maps, E08 T10a): 15 at
the west end, 12 and 13 between, 15 at the east end with the road's mirror flag (bit 4 of the
tile's flags, `0x10`); a north–south road uses 14 at its ends and 10 and 11 between. The
experiment writer lays east–west roads only.

**What the experiment writer writes** (`h3_map::writer`): a base terrain with
rectangles of other terrains (water among them, its land neighbours with the coast flag `0x40`,
E04c T3), east–west roads, players without a main town (Red and, with heroes or a town of
its own, Blue), heroes (their own fields or their hero type's), wandering monsters (class 54),
since E08 T16b resource piles (class 79 by resource, the template from `Objects.txt`, the body
without a message: amount 0 is drawn when the game starts; since E08 T22a1 a guarded pile's
message: its text, the question asked before the battle, empty when the guards fight at once,
the guards' flag set and their army) and, since E04c T3, the special
grounds (classes 21, 46, 222–231, no body), written before the other objects, and since E04c T5b
towns of a fixed faction (class 98, the template from `Objects.txt`: its entrance two columns west
of its position; no name, no events, no required or banned spells, the owner's alignment). Since
E08 T22a1 an underground level may follow: its own base terrain and rectangles (no roads), its
objects after the surface's (z = 1), a player's heroes on it in its hero list after the
surface's. Since E08 T32a the scenario's level limit and the Cartographers (class 13, no body,
the template from `Objects.txt`); since E08 T36a the trees (classes 135, 137, 155) and the
Border Gates (212, the key's colour as subclass), no body either; since E08 T36b SoD's banned
skills (the header's four bytes, bit `code` for the skill of that code). `h3-map`'s read-back test reads them back and imports them.

**Increment 7's objects and the RoE format** (E08 T56, batch E7's maps; `writer::RewardObject`,
`Monster::treasure`, `write_in`): the objects that hand a visiting hero something, with the bodies
the reader reads: an artifact (5, the artifact as subclass) or a random artifact (65–69): its
message record; a Spell Scroll (93): the record and the spell as a u32; a Shrine (88–90): the spell
as a u32, `0xFF` drawn at the new game; a Witch Hut (113): its four bytes of skills (the editor's
`0x0FFFEFBF` when none is given); Pandora's Box (6): the record and a reward record; an event (26):
the record, a reward record, the players' bits, the computer's flag, the removal flag and 4 zero
bytes. The **message record** is a bool, then the text, the guards' bool and seven slots, 4 zero
bytes; the **reward record** experience u32, mana i32, morale and luck i8, seven i32 resources,
four primary skill bytes, then u8-counted lists of (skill, level), artifacts, spells (u8) and
(creature, u16 count), 8 zero bytes (as "Reading: the three versions" above). A wandering monster
may carry a treasure (above). Increment 7's objects without a body are plain objects: the Corpse
(22), the Pyramid (63), the Learning Stone (100), the Treasure Chest (101), the University (104),
the Wagon (105), the Warrior's Tomb (108). **An RoE map** (`writer::Format::RoE`, version `0x0E`):
no level limit; an unplayable colour is 6 bytes after its two bools; a played one has no town
choice, one byte of allowed towns (eight factions) and no unknown byte or hero list; 16 bytes of
allowed heroes (all 128), no placeholders, disposed heroes, banned artifacts, spells or skills or
customised hero types; a hero's body has no identifier, always an experience u32 (0 for the hero
type's own), 18 artifact slots, nothing after its patrol but the 16 zero bytes; a monster's no
identifier; artifact and creature codes are one byte (`0xFF` none); a Witch Hut has no body. The
writer refuses in RoE what it cannot hold: a level limit, banned skills, a town, a hero type,
artifact or creature past RoE's (128, 127, 118), a hero's spells, primary skills or fifth misc
artifact, a Witch Hut's skills. `h3-map`'s `tests/rewards.rs` reads both back and imports them.

**Increment 8's objects** (E08 T67, batch E8's maps; survey `docs/rules/surveys/E08-sea.md` §5,
CLT-Q5): plain objects without a body: an empty boat (8, the boat's type 0 to 2 as subclass,
`AVXboat0` to `AVXboat2`), Flotsam (29), the Sea Chest (82), the Shipwreck Survivor (86) and holes
(124); an Ocean Bottle (59) or a sign (91) with its text (`writer::RewardBody::Sign`: the string
and 4 zero bytes). Favorable Winds (225) is a special ground (no body). A hero or a monster may
stand on a water tile (the writer checks no terrain under an object); a hero placed on water stands
in no boat (§2). `h3-map`'s `tests/sea.rs` reads them back and imports the map.

**The map client's additions** (E24 M1 T3a, `writer::write_with` and its `Extras`; the map client
lane's additive change, the user's consent of 2026-10-23; a map without them is written byte for
byte as before, a test pins one map of every kind by its CRC-32): **tiles as given**, the seven
bytes as stated (terrain and picture, river and picture, road and picture, the flags with their
mirror bits; the coast bit is not computed for them, but a given tile's terrain counts for its
neighbours'), and **objects of any template** but the four whose bodies name a hero (34, 62, 70,
214), written after the map's own, their position up to 7 columns past the right edge and 5 rows
past the bottom one (the game's SoD maps place decorations up to 5 and 2 past), an owner for the
classes whose body names one. Each gets the body the editor gives an object left as placed: nothing
optional, no message, guards or quest, a random count or amount, neutral; where the editor
chooses, the most common body in the 56 SoD maps: a monster aggressive (3 580 of 4 978), an event
for every player, not the computer, removed after its visit (59 of 103), a scholar's bonus random
(271 of 290), a garrison's troops removable (71 of 82), a shrine's spell random (`0xFF`, 403 of
413), a witch hut's skills all but Leadership and Necromancy (213 of 259), an abandoned mine's
resources all but wood (45 of 54), a town the standard buildings with a fort (284 of 564); a
random dwelling every faction and level, a scroll spell 0 and the grail radius 0 (no default
seen).

**Town buildings** (a town's custom `built` and `forbidden` bits; VCMI 1.7.5's
`config/gameConfig.json`, `mapFormat.restorationOfErathia`): 0 Town Hall, 1 City Hall, 2 Capitol,
3 Fort, 4 Citadel, 5 Castle, 6 Tavern, 7 Blacksmith, 8 Marketplace, 9 Resource Silo, 10 the Tower's
and the Dungeon's Artifact Merchants, 11–15 Mage Guild 1–5, 16 Shipyard, 17 Grail, 18–21 the
faction's special buildings (Castle: 18 Lighthouse, 19 Brotherhood of the Sword, 20 Stables;
Stronghold: 18 Escape Tunnel, …; `h3-experiment`'s table), 22, 25, 28, 31, 34 the dwellings of
levels 1–5 with their upgrades after each and a horde slot after that (24, 27, 30, 33, 36), 37 and
38 level 6, 39 and 40 level 7. The Village Hall has no bit: every town has it. `h3-map`'s read-back test reads each
kind back and imports a random pile as its typed pile without an amount.
