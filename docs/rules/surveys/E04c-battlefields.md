# E04c T1 source survey: the battlefields (what the battlefield seam decides)

Date: 2026-10-13. Scope: just in time (the user, 2026-10-13, BAT-Q37): the special battlefields
of increment 1 in full, sieges as an outline for a later increment (§6). Per-spell and per-creature
numbers are in the earlier surveys and the catalog; this file collects what a battlefield changes.

Sources: `code` = the reading of `Heroes3.exe` in `docs/formats/installation.md` §7.26 (with §7.16,
§7.21 to §7.24 for magic); `original-data` = the game's own texts (`ARRAYTXT.TXT`, `GENRLTXT.TXT`,
`randtvrn.txt` in `H3bitmap.lod`) and the executable's obstacle tables; `vcmi` = VCMI 1.7.5
(`config/battlefields.json`, `config/obstacles.json`, `lib/gameState/CGameState.cpp`
`battleGetBattlefieldType`; paths relative to https://github.com/vcmi/vcmi/blob/1.7.5/); `wiki` =
https://heroes.thelazy.net/index.php/Terrain and /Combat, fetched 2026-10-13 through a summarising
tool (short quotes only).

## 1. The battlefields of the original

The battle manager keeps three facts besides the terrain (`code` §7.26): a **special field**
(none, the shore 0, or 1–9), **a battle on a ship**, and **an anti-magic garrison**. Which picture
the battle shows follows from them (`code` 0x4646D0). Together they give twelve special
battlefields, the twelve VCMI defines beside its terrain variants:

| Battlefield (VCMI key) | Arises (`code` §7.26) | Terrain | What it changes in battle | Expansion |
|---|---|---|---|---|
| shore (`sand_shore`) | a tile next to water, above ground, no special ground | sand | its own obstacles | RoE |
| Magic Plains (`magic_plains`) | object 46 (SoD: 230) | the tile's | hero spells at Expert in every school; creature casts, strike spells and opening spells at Expert | RoE |
| Cursed Ground (`cursed_ground`) | object 21 (SoD: 223) | the tile's | morale 0, luck 0, no native bonus; no creature casts; spells above level 1 neither cast nor take effect; its own obstacles | RoE |
| Holy Ground (`holy_ground`) | object 227 | the tile's | morale +1 for Castle, Rampart, Tower creatures, −1 for Inferno, Necropolis, Dungeon | SoD |
| Evil Fog (`evil_fog`) | object 224 | the tile's | the reverse of Holy Ground | SoD |
| Clover Field (`clover_field`) | object 222 | the tile's | luck +2 for Stronghold, Fortress, Conflux creatures | SoD |
| Lucid Pools (`lucid_pools`) | object 228 | the tile's | hero spells of Water Magic at Expert | SoD |
| Fiery Fields (`fiery_fields`) | object 226 | the tile's | hero spells of Fire Magic at Expert | SoD |
| Rocklands (`rocklands`) | object 231 | the tile's | hero spells of Earth Magic at Expert | SoD |
| Magic Clouds (`magic_clouds`) | object 229 | the tile's | hero spells of Air Magic at Expert | SoD |
| a ship's deck (`ship`) | a hero in a boat, a Shipwreck or a Derelict Ship | water | only the water obstacle (type 55) | RoE |
| two boats (`ship_to_ship`) | both heroes in boats | water | 32 fixed blocked hexes, no other obstacles | RoE |

The special fields 1–9 also have their own obstacles (§3). A special field and a ship are
exclusive in practice (special grounds lie on land). The **anti-magic garrison** is no field:
it bans every cast, the hero's and the creatures', as the Orb of Inhibition does, the Faerie
Dragon's key path excepted (`code` §7.23); it can fall on a shore tile.

Every field's effect reads the stack's **town** (its creature's faction), not an alignment:
creatures without a town (the neutral creatures) are never affected by Holy Ground, Evil Fog or
Clover Field (`code` byte tables 0x43E118, 0x43E130, 0x43DFF4).

## 2. The effects compared

| Point | `code` | `original-data` | `vcmi` | `wiki` | Engine (T2) |
|---|---|---|---|---|---|
| Cursed Ground, morale | 0 for every stack, negative included | "No morale effects on cursed ground" | at most 0 (`MORALE` 0, `INDEPENDENT_MIN`): negative stays | "Negates all positive morale effects" | **D-1** |
| Cursed Ground, luck | 0 (no bad luck in SoD) | "No luck effects on cursed ground" | `NO_LUCK` | positive luck negated | 0 |
| Cursed Ground, native bonus | none | — | not modelled | "Disables all native terrain bonuses" | none |
| Cursed Ground, spells | hero: level 1 only; creatures: none; effects above level 1: none | GENRLTXT: "You cannot cast spells over level one here" | `BLOCK_MAGIC_ABOVE` 1 (every caster) | hero spells above level 1 blocked, creatures cannot cast | as `code` |
| Magic Plains | hero: every school at Expert; other casts at Expert | randtvrn: "even the common man can cast spells like the greatest wizard" | `MAGIC_SCHOOL_SKILL` any 3 (heroes only) | "all spells … at expert level", creature spellcasting included | as `code` (**D-3**) |
| Fiery Fields, Rocklands, Magic Clouds, Lucid Pools | the hero's spells of the school at Expert; other casts not | — | `MAGIC_SCHOOL_SKILL` of the school | "Spells cast by creatures are not affected" | as `code` |
| Holy Ground, Evil Fog | ±1 by town 0–2 / 3–5 | "Creature of good town alignment on Holy Ground +1" and three more | ±1 by alignment `good`/`evil` | ±1 good/evil | as `code` |
| Clover Field | +2 for towns 6–8; none without a town | "Creature of neutral town alignment on Clover Field +2" | +2 for alignment `neutral`, **neutral creatures included** | contradicts itself ("Neutral creatures get +2 luck"; "Neutral creatures are not affected") | as `code` (**D-2**) |
| Underground battles | always subterranean (S-22) | — | the terrain's battlefields of the layer | "the only combat terrain for the underground layer" (fixed in HotA) | the adventure lane's derivation |
| Gold mine, crystal cavern, abandoned mine | subterranean | — | — | subterranean for those three | the adventure lane's derivation |
| Coast | a tile next to water fights on sand, the shore field | — | coastal tile → `sand_shore` | "Sand is a combat terrain for all tiles next to water" | the adventure lane's derivation |
| Boats | both heroes in boats: 32 blocked hexes | — | `ship_to_ship`: the same 32 `impassableHexes` | "Boat" picture | as `code` |
| Deck | a hero in a boat or a Shipwreck or Derelict Ship: water | — | `ship` with obstacle 55 | "Boat (Deck)": monsters or locations on a boat, or a hero in a boat against one on land | as `code` |

## 3. Obstacles

- The executable's two tables (91 usual types at 0x63C7C8, 34 absolute obstacles at 0x63BEC0)
  equal VCMI's `obstacles.json` record by record: terrains, special fields, size, blocked hexes
  (`code` §7.26, compared 2026-10-13). So harvesting them from the executable changes no type the
  packs have.
- The packs (gap-fill from VCMI, E03) hold the 78 types with a terrain; **47 are missing**: the
  shore's 5 usual and 6 absolute, the water type 55 (the deck), and the SoD fields' 35 (Holy
  Ground 5, Evil Fog 5, Clover Field 4 + 1, Lucid Pools 4 + 1, Fiery Fields 5 + 1, Rocklands
  4 + 1, Magic Clouds 3 + 1). Cursed Ground and Magic Plains have only types that already carry a
  terrain (rough and dirt; grass and swamp), so their sets are in the packs without the field.
- **Choice** (`code`): on a special field (shore included) the generator takes only the field's
  types, absolute and usual; otherwise the terrain's (the deck: water). The two boats place no
  random obstacle, only their 32 hexes, as absolute obstacles are placed (a blocked bit, no
  object).
- The engine's generator (BAT-GEO-004) is VCMI's port and keeps its placement; only the type set
  changes with the battlefield.

## 4. What the battle setup must receive, and who derives it

The battle needs: the terrain (as today), the battlefield (none or one of the twelve), whether the
battle is at an anti-magic garrison, and the obstacles drawn for that battlefield. The derivation
from the adventure map is the adventure lane's (`h3-game` builds `BattleSetup`); the original's
order (`code` §7.26): siege; a hero in a boat or a ship's bank → water and deck (two boats when both
heroes sail); underground → subterranean; a gold, crystal or abandoned mine → subterranean; a tile
next to water → sand and the shore; else the tile's terrain; then a special ground on the tile
replaces the field (the shore's too), and an anti-magic garrison sets its flag. The client lane's
pictures follow the same facts (`E17-design.md` §5; CLT-Q4 is the dirt, grass and snow variant).

## 5. Disputes and quirks

- **D-1 Cursed Ground and negative morale.** `code` and the game's texts: morale 0 for everyone,
  so no stack freezes from bad morale there. `vcmi` and `wiki`: only positive morale is negated.
  Real gameplay impact (an undead-mixed army never freezes on Cursed Ground). Recommendation: the
  code (the rank of a reading, `docs/04-rules-and-sources.md` §2, backed by the game's own text),
  with a confirmation row in the next experiment batch (a reading against VCMI and the wiki).
  **The user (BAT-Q38): the code**, but the intent may have been only non-positive morale (the
  wiki's and VCMI's reading), which would make the lost negative morale a bug: candidate S-28.
- **D-2 Clover Field and neutral creatures.** `code` and the texts: only towns 6–8; `vcmi`: neutral
  creatures too. Recommendation: the code, a confirmation row. **The user (BAT-Q38): the code, the
  distinction noted:** the original's "neutral" is the towns' alignment (Stronghold, Fortress,
  Conflux), VCMI's the creatures' (`alignment` `neutral` also takes in the creatures without a
  town, the Peasant to the Azure Dragon), so those get +2 in VCMI and nothing in the original.
- **D-3 Magic Plains and creature casts.** `code` and `wiki`: Expert for creature, strike and
  opening spells too; `vcmi`: heroes only. Recommendation: the code (the reading, the wiki agree).
- **D-4 The SoD objects 223 (Cursed Ground) and 230 (Magic Plains).** 0x4FCE20 knows only 21 and
  46. The map loader is not read; if it does not turn 223 and 230 into 21 and 46, a battle on the
  SoD versions would have no field. For the adventure lane's derivation; a reading of the loader
  or an experiment settles it.
- **S-22** (underground battles always subterranean, `E04-quirks.md`, classified `preserve`):
  confirmed by the reading (0x464451). The adventure lane's derivation implements it.
- **S-28 Cursed Ground's morale 0 for everyone** (quirk candidate, the user at BAT-Q38; to be
  investigated). The code and the text give every stack morale 0, negative morale included; the
  field's intent may have been "no positive morale" (the wiki and VCMI), which would make the lost
  negative morale a bug, `fix-optional` (a flag keeping negative morale, off in `faithful`). Next:
  T3's row for D-1 shows what the game does; whether it is a bug, and gets a flag (Q30), is the
  user's call then. Until then the engine follows the code (BAT-TER-003). Nothing else in §1
  looks like a bug; the fields' texts describe what the code does.

## 6. Sieges (outline for a later increment)

Known so far (`E04-battle.md` §7 "Siege inputs", the battle queue's inbox, `code` §7.15 to §7.24):
the fort level (walls, keep, moat, towers), the moat by faction (damage 0x469E50, its hexes as
stops 0x4B3290, the Tower's land mines by Land Mine's mastery, 0x466290 at 0x4663BE), the walls'
hexes (table 0x63BD00, the blocked bit 0x2), the town's terrain (§7.26), the arrow towers (S-06,
S-07), Earthquake (S-08), the Catapult and Ballistics, Teleport's siege limits, the defender's
town armies in the spell cost (0x4E5240), the summon's siege gate (0x467130), no retreat or
surrender for the besieged, +500 experience for the capture. Each is surveyed when its increment
is cut.

## 7. Experiment candidates

For the next batch the user plays (SoD; the `.h3m` writer needs the special ground objects first):
D-1 and S-28 (an army with negative morale on Cursed Ground: does it freeze?), D-2 (neutral creatures on
Clover Field: luck +2?), D-4 (a battle on object 223 and on 230), and one control per field family
(a Holy Ground morale value, an elemental field's Expert spell).
