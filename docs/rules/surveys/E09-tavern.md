# E09 Tavern survey (E09 T32): the Tavern's hire, the heroes that come back, the Thieves' Guild

Date: 2026-11-15. Scope: E09 increment 4 (the Tavern; the epic's decisions 99 and 100), input for
T33's ADR amendment drafts, the design note's Part F and the cut. Kept whole (the user at T31's
start, 2026-11-15; the agent had proposed T32a, the hire, then T32b, the information). Input, not
rules (`README.md`): a fact becomes binding when a task writes it into the catalog with its
sources and tests. A hero **type** is one of the 156 (0 to 155, `HOTRAITS.TXT`'s order, the
packs' `[[hero]]` ids); a **class** one of the 18 (`HCTRAITS.TXT`'s order, 0 Knight to 17
Elementalist; a faction's two are 2f and 2f + 1). A player's two **offers** are its tavern
slots, the player record's +0x28 (slot 0) and +0x2C (slot 1). The pool's states are the type
table game +0x4DF18: −1 free, a colour 0 to 7 owned, 0x40 reserved (out of every draw), 0x41 a
prisoner's (survey `E09-towns.md` §4.2).

**Sources** (retrieved 2026-11-15):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml` or `structs.toml`. The
  reading follows the human player's path (the town screen, the adventure map, the week) and names
  the computer's where it differs. **Read whole:** `tavern_hero_draw` 0x4BB5E0, `tavern_new_heroes`
  0x4C8450, `tavern_slot_replace` 0x4C86A0, `hero_leave_game` 0x4D9EC0, `map_tavern_hire`
  0x4D7890, `town_tavern_hire` 0x5C12E0, `map_tavern_visit` 0x5D7E90, `tavern_dialog_hint`
  0x5D7920, `draw_starting_army` 0x4C9730 (its first part, again), `player_obelisks_visited`
  0x4BABA0. **Read in part:** `tavern_dialog` 0x5D7EC0 (its set-up), `town_tavern_click` 0x5D82B0,
  `new_day_week_events` 0x4C8780 (the taverns), `new_game_heroes` 0x4BFE70 (the taverns),
  `next_player_turn` 0x4C6FE0 (its first part), `visit_object` 0x4A84F0 (cases 95 and 97),
  `thieves_guild_fill` 0x5DDA10 (what decides the rows), `thieves_guild_row_value` 0x5DEE70 (its
  switch), `new_game_bans` 0x4C2450 (the disposed heroes, the customised types' reader
  `map_custom_heroes_read` 0x4C2CE0), `placed_heroes_setup` 0x4CAE10 (again), `hero_place` 0x4D7900
  (again, its flag), the readers of the banned skills (game +0x4E658). **Data:** `hero_price`
  0x67814C (2 500), `edition` 0x67F554 (3), `HCTRAITS.TXT` from `H3bitmap.lod` and `H3ab_bmp.lod`
  (the same file: nine tavern columns, Castle to Conflux).
- VCMI 1.7.5 (GPL-2+), read from the tag's sources: `server/processors/HeroPoolProcessor.cpp`,
  `lib/gameState/TavernHeroesPool.cpp`, `lib/gameState/GameStatePackVisitor.cpp` (a hero removed),
  `lib/gameState/CGameState.cpp` (`obtainPlayersStats`), `lib/callback/CGameInfoCallback.cpp`
  (`getThievesGuildInfo`), `lib/entities/hero/CHeroClassHandler.cpp` (the weights),
  `config/heroClasses.json`.
- The Heroes 3 wiki, "Tavern" (heroes.thelazy.net/index.php/Tavern); its other pages found by
  search said nothing on the Thieves' Guild's levels, the Den of Thieves or the returning heroes.
- The manuals (the user's `HoMM 3 Complete`): RoE's (the town's tavern, "Retreat", "Surrender",
  "Loss of Troops", "Dismiss Hero").

## 0. Findings in short

1. **Two offers a player, the same in all its taverns** (§2): per player (not per town) two
   slots; **slot 0 is of the player's faction** (its chosen faction, game +0x1F6B0: the faction it
   started as, not the town's), slot 1 of any; each a class drawn by `HCTRAITS.TXT`'s tavern
   column of the player's faction (weights 5 or 6 in this build's data, each column summing to
   100), the other offer's class avoided while another is free, then a type of the class
   uniformly. The wiki agrees on slot 0 ("the left-most hero ... the faction the player started
   as"); VCMI draws the same way.
2. **The week** (§2.3): at the week's start every slot is released (its type free again), except
   an offer that came back by a retreat or a surrender since its owner's turn last ended (bit 17
   of the hero's +0x105, cleared at the end of its owner's turn); then each player not defeated,
   colour by colour, fills its empty slots (slot 0, then 1). The game's start does the same
   before day 1. A new week's offer has a **full starting army** (`draw_starting_army`), its spell
   points full and its backpack artifacts equipped where a slot takes them.
3. **The hire** (§3): 2 500 gold (`hero_price`, every edition, every difficulty); refused with
   less gold, with 8 heroes on the map (the garrison heroes not counted: the player's list), and
   in a town while a hero visits it (the dialog's Recruit button; the town needs a Tavern, or
   Castle's Brotherhood, to open). The hired hero stands on the **town's entrance** as its visitor
   and learns the town's guild spells; **its slot is refilled at once**, any faction, the other
   offer's class avoided, the new offer with **one creature** of its first stack (the wiki: "only
   one non-upgraded first level creature"). The adventure map's Tavern hires onto its visited tile
   (refused while the visitor stands there), learns no spells and refills the same way.
4. **The heroes that come back** (§4, `hero_leave_game`): a hero that leaves the game (lost,
   retreated, surrendered, dismissed, its player defeated) is freed. **A retreated or surrendered
   hero goes at once into one of its owner's two slots** (a random one, not one already holding
   such a hero if the other does not), the offer it replaces freed; **a surrendered hero keeps its
   army**, every other gets one creature of its first stack; a retreated or surrendered hero keeps
   its spell and movement points as they were, the others are refilled. A lost hero keeps its
   level, skills, spells and the artifacts the winner had no room for (GAM-BAT-004); a retreated
   one all its artifacts (the manual: "keeps all carried artifacts"); a dismissed one all its
   artifacts as far as read. A freed hero may be drawn later by any player.
5. **The map's hero settings** (§5): SoD's customised hero types are applied to every type's
   record before the first draw, so a customised type is offered as the map sets it
   (`Marshland Menace`: 15 Fortress types at 2 000 experience); the disposed heroes' colour masks
   limit who is offered them; **a banned secondary skill does not touch a hired hero** (only the
   level-up offers, Universities, Witch Huts and Scholars read the bans): 110 allowed types on the
   19 maps that ban skills start with one.
6. **The SoD rule on Conflux heroes is dead in this build** (§2.2): `tavern_hero_draw` keeps the
   Conflux classes out of a non-Conflux player's offers on an SoD map only when the edition number
   is 2 (likely the SoD retail edition; not checked); this build's is 3, never written, so Conflux heroes can be any
   player's slot-1 offer, as in VCMI. A historical edition's rule (E11).
7. **The Thieves' Guild** (§6): by the viewer's taverns (owned towns with a Tavern, or Castle's
   Brotherhood: `player_tavern_count`): rows towns and heroes at 0 or 1, gold and wood and ore from
   2, the rare resources and obelisks from 3, artifacts and army strength from 4, income from 5;
   per player its best hero from 1 and three further parts from 2, 3 and 4. **The Den of
   Thieves** shows everything (99), on every visit, after its message. VCMI 1.7.5 has the same
   thresholds.
8. **Disputes** (§8): TD-18 the week's kept returning offer (VCMI clears both), TD-19 the slot a
   returning hero takes, TD-20 the offers' daily points, TD-21 the hero limits (VCMI's two caps).
   **Quirk candidates:** T-14 a dismissed hero's artifacts back in the pool (if confirmed, any
   player may hire them); T-15 the replacement's one creature taken from a hero's starting army
   that has no first stack of its own (not reachable with the shipped heroes). Batch TWN-4's
   rows: §9.

## 1. The census (the 159 `.h3m` maps of the installation; a throwaway example over `h3-map`'s reader, not committed)

The tutorial (`.tut`) is left out: its towns are E09-towns.md §1.1's 1 468 less these 1 465.

- **Versions:** RoE 48, AB 55, SoD 56. **Allowed heroes** (the header's bits): RoE 106 to 128
  (the original adds the 16 Conflux types outside a campaign, E09-towns.md §4.2), AB 61 to 144,
  SoD 69 to 155. **The pool is never short at the start:** the fewest free types (allowed, less
  the placed heroes and prisoners) a player are 61 for 2 players (`Twins`), 69 for 6
  (`Heroes of Might not Magic`).
- **Disposed heroes** (SoD): 832 on 9 maps, each with a colour mask (none 0xFF).
- **Customised hero types** (SoD's map-wide entries): flagged on 9 maps (832 entries: those
  maps flag most types), but a field is set on only 17: **experience on 15**, all in
  `Marshland Menace` (2 000 each: the Beastmasters 112 to 119 and the Witches 121 to 127; its
  144 allowed types, 5 players, 14 towns, 6 with a Tavern at the start, no map Tavern), and the
  sex of Elleshar (26) in `Last Chance` and its Allies version. No customised skills, artifacts,
  spells, primary skills or biography.
- **Banned skills:** 19 SoD maps; Navigation on 18 of them (Eagle Eye on 4, Scouting on 1, and
  13 skills on `Heroes of Might not Magic` and its Allies version). **The allowed types not
  placed on those maps whose starting skills include a banned one: 110** (1 to 19 a map; 19 on
  each `Heroes of Might not Magic`).
- **Towns:** 1 465 (875 random); 887 owned at the start, 578 of them with a Tavern standing (the
  546 standard sets all have one, E09-towns.md §5.3; 32 of the 341 custom ones); 578 neutral, 52
  of them with a Tavern (custom sets). **The 742 players by the taverns they own at the start:**
  none 228, one 462, two 43, three 6, four 3 (the Thieves' Guild's levels at the start, §6).
- **Map objects:** the Tavern (class 95) 93 on 47 maps, the Den of Thieves (97) 227 on 107 maps
  (E08 T97c's census: both import blockers today).
- **Conflux heroes:** allowed on every AB map and on 43 of the 56 SoD maps (each with a player
  that may play another faction), none on 13 SoD maps; the SoD rule would bite on those 43 in the
  SoD edition, never in this build (§2.2).

## 2. The offers (`tavern_hero_draw` 0x4BB5E0, `tavern_new_heroes` 0x4C8450, read whole)

### 2.1 Where they are

Each player record holds two hero types (+0x28, +0x2C; −1 empty): the player's offers in every
tavern it opens, its towns' and the map's (the code has no other store; the manual says only
that two heroes are in each tavern and change each week, and the rumour is the one "identical in
all of your town taverns"; the wiki's RoE note that the map's taverns once had other heroes is an
older patch's). An offered type is reserved (0x40): no other draw gives it, no other player is offered
it. The tavern dialog shows the two offers (portrait, name, level, class, its artifacts' count),
the first present one selected (`tavern_dialog` 0x5D7EC0).

### 2.2 The draw (`tavern_hero_draw`; arguments the colour, a class to avoid, own faction only, a forced class)

1. The faction is the colour's chosen faction (`chosen_factions`, game +0x1F6B0: the faction the
   player plays, a Random choice drawn at the start), never a town's.
2. Per class its weight, `HCTRAITS.TXT`'s tavern column of that faction (the class record +0x34 +
   the faction), and its count: the types of the class that are free and takeable by the colour
   (the mask +0x4DFB4, the disposed heroes'). No free type at all: no offer (−1).
3. A class without a free type weighs 0. On an SoD game (`map_kind` +0x1F698 at least 2) **and
   the edition 2**, a non-Conflux faction's classes 16 and 17 weigh 0 while other types are free,
   unless forced: **never in this build** (`edition` 0x67F554 is 3 and has no writer; the same
   number makes a neutral random town's faction set nine on AB and SoD maps, E09-towns.md §2.3).
4. The class to avoid weighs 0 while other types are free.
5. Own faction only: when the classes of the faction (the class record +0, the faction) weigh
   anything, every other class weighs 0.
6. A forced class is taken when it weighs (the tavern passes none).
7. Else a class by `random_between(1, total weight)` over the classes in order, then the n-th
   free and takeable type of the class in type order, `n = random_between(1, its count)`: two draws
   of the game's one generator.

**The weights** (both LODs' `HCTRAITS.TXT`, the same nine columns): each column sums to 100; a
class weighs 6 or 5 in every faction's column (Castle's: its own and Rampart's and Tower's classes
and Stronghold's and Fortress's four 6, Inferno's, Necropolis', Dungeon's and Conflux's 5). So the
draw is nearly uniform over the classes with a free type, and slot 0's own-faction restriction is
what shapes the offers. RoE's file lacks the Conflux column (E04a T3a's note; the user's RoE media,
E11).

### 2.3 Filling the slots (`tavern_new_heroes`), the week and the start

- For slot 0, then 1, only an empty one: `tavern_hero_draw` with the player, **the other slot's
  class to avoid** (none when it is empty), **own faction only for slot 0**, nothing forced; the
  type reserved; the hero's backpack artifacts equipped where a slot takes them (from the last;
  `hero_equip_artifact` 0x4E2A00) and taken from the backpack; its spell points its Knowledge (1
  to 99) × 10 × its Intelligence factor; `draw_starting_army` with the one-creature flag clear:
  the type's **full starting army** (it clears the seven slots first: whatever army the hero had
  is replaced). The tutorial's weeks 1 and 2 give fixed classes (not followed: no tutorial).
- **The week** (`new_day_week_events`, after the towns' growth and the week's count, before the
  dwellings): every player's slots, in colour order, released unless the slot's hero has bit 17 of
  +0x105 (its type free again, the slot −1); then each player not defeated, colour 0 to 7, fills
  its slots. So the week's draws see every released type, its own old offers and other players'
  among them.
- **The start** (`new_game_heroes` 0x4C00C0, after the main towns' heroes): the same release and
  fill, before day 1.
- **Bit 17** (the returning offer, §4) is set by `hero_leave_game` and cleared for the current
  player's two offers when its turn ends (`next_player_turn`'s first part, 0x4C705D, after the
  victory tests), and by a hire (`hero_place`'s flag clears bits 17 and 18). So the week keeps an
  offer that came back after its owner's turn ended that day (an attack in a later player's turn),
  and releases one that came back in its owner's own turn.
- **The offers' points:** at each of its owner's turn starts its two offers get their current and
  maximum movement points (`next_player_turn`, its map entry's reading); spell points only at a
  draw (above) or a leave (§4). *Amended by batch TWN-4 (§14):* the new day also gives every hero
  record its day's spell points, and clears bit 17 of every record after the week's release.

## 3. The hire (`town_tavern_click` 0x5D82B0, `map_tavern_visit` 0x5D7E90, the tavern dialog, `town_tavern_hire` 0x5C12E0, `map_tavern_hire` 0x4D7890)

- **The checks** (the dialog's Recruit button, 0x5D81EE; its hints `tavern_dialog_hint` give the
  refusal texts): disabled with the player's gold below 2 500, no offer selected, **8 heroes on the
  map** (the player's list count +0x1: a garrison hero is out of it, TWN-ROW-002; the 8 is
  hard-coded in the dialog, as in the town screen's move out, TWN-ROW-003's `most_heroes`), or, in
  a town's tavern, **a visitor in the town** (+0x10; the manual: "You cannot hire a hero if you
  currently have another hero visiting the town"). A computer player's dialog hides the button
  (its hires are its own code, not read: E13). The town's tavern opens from the town screen's Tavern
  building (5, or Castle's Brotherhood 22 over it).
- **A town's hire** (`town_tavern_hire`): the slot found, 2 500 gold taken, `hero_place` at the
  town's position (+5 to +7: its entrance, E09-towns.md §2.4) with the flag that clears bits 17
  and 18: the hero joins its owner's list and becomes the town's visitor (`hero_show`); **the
  town's guild spells learned** (`town_heroes_learn_spells` 0x5BE030 with 0: a hero with a spell
  book, the visitor first; E09-towns.md §4.5); then the slot refilled (`tavern_slot_replace`).
- **The replacement** (`tavern_slot_replace` 0x4C86A0): `tavern_hero_draw` with the player, the
  other slot's class to avoid, **any faction**, nothing forced; the type reserved; spell points
  full; `draw_starting_army` with the one-creature flag set: **one creature of the type's first
  starting stack** (the hero table's first; a base level-1 creature for every shipped hero). No
  backpack equipping. None when no type is free.
- **The adventure map's Tavern** (class 95, `visit_object`'s case 95; a human only): skipped when
  the visitor stands on the visited tile itself; else the dialog (0x6AAA48 set: no visitor test),
  then `map_tavern_hire`: the price, `hero_place` at the visited tile (the Tavern's), the slot
  refilled; no guild spells. VCMI places at the Tavern's visitable tile too and refuses while it is
  occupied.
- **The price** is one global for every hire (the computer's too) and every edition.

## 4. A hero that leaves the game (`hero_leave_game` 0x4D9EC0, read whole)

Every caller passes (1, 0): a battle's lost or fled heroes (`adventure_battle` 0x4AE6D5, 0x4AE779,
0x4AE806; `battle_aftermath` 0x4AC962), a garrison hero without a stack at its town's capture
(`town_visit` 0x4AB148, `TWN-QUIRK-11`), the hero screen's Dismiss (0x4E19C2), a defeated player's
heroes (`player_defeated` 0x4F13C6, which then releases its two offers), the network.

1. Off the map: a garrison hero leaves its town (+0xC −1), the hero its tile, its boat freed
   (E08 T66a1); the object under it restored (a town's visitor −1).
2. **Its army's seven slots cleared, unless the battle was surrendered.**
3. Out of its owner's list; **its type free** (−1).
4. **After a retreat or a surrender:** into its owner's tavern at once: slot `random_between(0,
   1)`, the other one when that slot's hero has bit 17 (a returning offer already); the offer in
   that slot, when reserved, freed; the type reserved; bit 17 set. So a player has at most two
   returning offers, and a third replaces one of them.
5. Owner −1 (E08 T62b).
6. **Without bit 17** (a lost, dismissed or defeated player's hero): spell points refilled
   (Knowledge × 10 × the Intelligence factor, the Intelligence specialist's growth by level) and
   movement points the land maximum. A retreated or surrendered hero keeps both as they were.
7. **Unless surrendered:** `draw_starting_army` with the one-creature flag: one creature of its
   first starting stack (a retreated hero: the manual's "loses all troops", one creature left; a
   lost or dismissed hero: the same, until a draw gives it a new army).

**What it keeps:** its record, so its level, experience, skills, spells and artifacts. A lost
hero's artifacts are the winner's until one finds no room (GAM-BAT-004: the rest stay with the
loser), so it comes back with those; a retreated or surrendered hero keeps all of them (the
battle's capture returns at once after a retreat or a surrender: `battle_capture_artifacts`
0x46A070); a dismissed hero: nothing on its path strips them (the dialog's own handler before its
0x81 result not read; T-14). A freed type is drawn later by any player (the manual's "may end up
in the service of one of your enemies"), and that draw gives it a new army (a full one at a week's
fill, one creature as a replacement).

## 5. The map's hero settings

- **Customised types** (SoD): `map_custom_heroes_read` 0x4C2CE0 reads each flagged type's body into
  its setup record (game +0xA4 + 0x334 × type, a placed hero's layout); a placed hero's body
  overrides its type's record (`load_hero_body`); `placed_heroes_setup` applies **every** type's
  record to its hero (`placed_hero_setup` 0x4D8B30), placed or not, before the taverns' first fill.
  So an offer is the map's customised hero: `Marshland Menace`'s 15 types come at 2 000 experience
  (level 3; its levels at load as GAM-SET-002 gives them). The disposed heroes' names and portraits
  go into the same records (0x4C2AC8).
- **Colour masks** (SoD's disposed heroes, `hero_pool_init`): a type is drawn only for a colour in
  its mask (§2.2 step 2); `hero_place` adds the hiring colour's bit.
- **Banned skills** (game +0x4E658): read by the level-up offer (`hero_secondary_offer`
  0x4DAD32), Universities, Witch Huts and Scholars (`university_setup`, `setup_map_objects`,
  `load_scholar_body`) and the save, never by the taverns, the hire or the heroes' setup: **a
  hired type keeps a banned starting skill** (19 maps, 110 types: §1). VCMI checks a ban only when a
  skill is learned (`CGHeroInstance::canLearnSkill`), not at a hero's setup or a hire.
- **Not in the pool:** a type the map does not allow (0x40 from the start), a prisoner's (0x41),
  the placed heroes' (their owners). A campaign's carried heroes and their class redraw
  (`hero_setup_redraw_by_class` 0x486110) are out of scope.

## 6. The Thieves' Guild and the Den of Thieves (`thieves_guild_fill` 0x5DDA10, `thieves_guild_row_value` 0x5DEE70)

- **Opened from** the tavern dialog's button (its handler 0x5D7BFF passes −1: the viewer's
  tavern count, `player_tavern_count` 0x4CCE30: its towns with a Tavern or Castle's Brotherhood;
  the map's Taverns count nothing) and from **the Den of Thieves** (class 97, `visit_object`'s case
  97: its message, ADVEVENT's, then the window with 99), on every visit, no mark.
- **The rows** (the count, or 99): 2 rows with none or one tavern, 4 with two, 6 with three, 8
  with four, 9 with five or more (the Den's). In order: 0 towns, 1 heroes (the list's count), 2
  gold, 3 wood and ore, 4 mercury, sulfur, crystal and gems, 5 obelisks visited
  (`player_obelisks_visited` 0x4BABA0), 6 artifacts worn and in the backpack, 7 army strength
  (`hero_army_power` and the towns' garrisons), 8 gold income. Each ranks the players not
  defeated, with their flags.
- **Per player** (not defeated), by the count: from 1 its **best hero** (the greatest
  `hero_army_power`), its portrait; from 2, 3 and 4 a further part each (read by T33b, §13: the
  best hero's primary skills, the player's tactic, its best creature).
- **VCMI 1.7.5** (`obtainPlayersStats`, `getThievesGuildInfo`): towns and heroes from 0, the best
  hero from 1, gold, wood and ore and the best hero's details from 2, the rare resources,
  obelisks and the personality (the computer's AI tactic; none for a human) from 3, artifacts,
  army strength and the best creature (by AI value) from 4, income from 5; a Den of Thieves 20
  (everything); the count its THIEVES_GUILD_ACCESS bonuses (a tavern each). **The same
  thresholds** as the reading; VCMI's best hero is its own `findBestHero` (compared by T33b, §13:
  TD-23).
- The view is the viewer's alone (a projection, decision 6): what it shows of other players is the
  information.

## 7. VCMI 1.7.5, the wiki and the manuals compared

| Point | Reading | VCMI 1.7.5 | Wiki / manual |
|---|---|---|---|
| Offers per player, all taverns | yes | `TavernHeroesPool` per player | manual: two in each tavern |
| Slot 0 native | the chosen faction's classes when they weigh | `pickClassFor(isNative)`: the player's `castle` | wiki: "the faction the player started as" |
| Weights | HCTRAITS' columns | HCTRAITS (legacy data) | — |
| Other offer's class | avoided while others are free | avoided (any class in the tavern) | — |
| SoD Conflux rule | edition 2 only (dead here) | none | — |
| The week | released but a kept returning offer | both slots cleared (`onNewWeek`) | manual: they "change each week" (TD-18) |
| Full army at the week | yes | `FULL_ARMY` | wiki: "regular army" |
| Hired: replacement | at once, any faction, one creature | at once, `SINGLE_UNIT` | wiki: at once, one level-1 creature |
| Price | 2 500 | `HERO_GOLD_COST` 2 500 | wiki 2 500 |
| Limit | 8 on the map | 8 on the map and a total cap (settings) | — (TD-21) |
| Town's visitor | refuses | refuses | manual: refuses |
| Town hire: spells | guild spells learned | `objectVisited(town)` | — |
| Retreat | owner's slot at once, one creature, points kept | `onHeroEscaped`: the same, a role-based slot | manual: in your taverns, artifacts kept |
| Surrender | army kept | army kept | manual: troops and artifacts kept |
| Lost, dismissed | freed, one creature, points refilled | freed (`RemoveObject`), army as it was, the Grail removed | manual: may come back, maybe for an enemy |
| Offers' points | movement at the owner's turn | movement and mana each day (`onNewDay`) | — (TD-20) |
| Thieves' Guild | §6 | the same thresholds | — |
| Den of Thieves | everything | everything | — |

VCMI's slot for a returning hero (`selectSlotForRole`): an empty slot first, else the slot of the
lower role (a full-army offer before a single-unit one before a retreated before a surrendered),
else one of the two (TD-19). Its RNG is a stream per player (`getRandomGenerator(player)`).

## 8. Disputes, quirk candidates and open points

**Disputes** (two sources or more against one; the checkpoint decides):
- **TD-18 — the week keeps a returning offer.** The reading: a retreated or surrendered hero that
  came back after its owner's turn ended stays offered through the week's release. VCMI clears
  both slots every week; the manual says only that the heroes change each week. Impact: a
  defender that flees or surrenders on the week's last day keeps its offer (and its army, if
  surrendered) past the week. Recommended: the reading (it is the original's code path; batch row
  TWN-4-3).
- **TD-19 — the slot a returning hero takes.** The reading: a random slot, the other when it
  holds a returning offer; VCMI: by roles (it spares a returning offer and a surrendered one before
  a retreated one). Impact: which offer a flight removes. Recommended: the reading.
- **TD-20 — the offers' daily points.** The reading: movement at the owner's turn start, spell
  points only at a draw or a leave; VCMI: movement and the day's spell point regain for every
  offer each day. Impact: a returning offer's spell points when hired days later. Recommended: the
  reading. (Its spell-point half refuted by the code read after batch TWN-4, §14: every record
  regains each day, as VCMI's; decision 127.)
- **TD-21 — the hero limits.** The reading: 8 on the map, garrison heroes not counted, a constant;
  VCMI: the same 8 on the map and a total cap of its settings. The town part's `most_heroes` (8,
  TWN-ROW-003) already holds the reading. Recommended: the reading, one bound.

**Quirk candidates** (`T-<n>`; the ledger after the checkpoint):
- **T-14 — a dismissed hero's artifacts back in the pool.** Nothing on `hero_leave_game`'s path or
  after the hero screen's dismiss strips them; if the dialog does not either, a dismissed hero is
  offered later (to any player) with its artifacts, a transfer between players through the pool.
  VCMI keeps them (all but the Grail). Batch row TWN-4-5 first; then harmless-and-accepted or
  fixed (the checkpoint).
- **T-15 — the replacement's one creature for a hero whose first starting stack is a war
  machine.** `draw_starting_army`'s one-creature path takes the hero table's first creature; no
  shipped hero's starting army holds a war machine (the packs' `[[hero]]` armies: creatures only;
  the Ballista and the First Aid Tent come as artifacts), so it is not reachable on the shipped data. Noted for the packs'
  validation (a hero's first stack must be a creature).

**Not quirks:** `tavern_hero_draw` with colour −1 (no tavern caller); a class weight of 0 for every
class while types are free (`random_between(1, 0)`: not reachable, every weight is 5 or 6); a
hired returning offer's bit 17 (cleared by the hire).

**Open points:** the computer's hires (its own functions that read the price; E13), whether it uses the map's
Taverns (the wiki: it does not); the tavern dialog's dismiss-handler question (T-14); the rumours
(text, no rule; decision 99); the network's paths. The three per-player Thieves' Guild parts: read
by T33b (§13, with TD-23 and T-17).

## 9. Batch TWN-4's rows (for T33's cut; run by T34 on the capture harness, scored in §14)

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| TWN-4-1 | §2 | a Castle player with a Tavern, four weeks, nothing hired | the two offers each week (types, classes, armies) | slot 0 a Castle class each week; the two classes differ; full starting armies |
| TWN-4-2 | §3 | the same, one hire from each slot | the replacement's type and army; the hired hero's tile and spells | at once, any faction, the other's class avoided, one creature; on the entrance, the guild's spells learned |
| TWN-4-3 | §4, TD-18 | red's hero attacked on day 7 in blue's turn (after red's), flees; another on day 6 in red's turn, flees | red's offers on day 8 | the day-7 fugitive still offered, the day-6 one released |
| TWN-4-4 | §4 | a hero surrenders, another retreats, a third loses | the offers, their armies, spell and movement points; the lost one's later draw | the surrendered with its army, the fled with one creature, both with their points; the lost one free |
| TWN-4-5 | T-14 | a hero with artifacts dismissed, the type then drawn (a map with few free types) | its artifacts when offered again | the reading: kept |
| TWN-4-6 | §3 | 8 heroes on the map and a garrison hero; a visitor in the town | the Recruit button | disabled at 8 on the map whatever the garrison; disabled with the visitor |
| TWN-4-7 | §6, §13, TD-23, T-17 | a player with 1 to 5 taverns; a Den of Thieves; a hero with worn artifacts only beside one with a backpack; a player whose most experienced hero is not its strongest; a town garrison stronger than its owner's heroes | the guild's rows' places and per-player parts | §6's thresholds; the Den everything; the artifacts row by the backpacks alone; the best hero by army power; the army row and the best creature with the town's garrison |
| TWN-4-8 | §2.2 | a Castle player on an SoD map with Conflux heroes allowed, many weeks | slot 1's classes | a Conflux class now and then (the edition 3) |

**Not rows:** the weights (data), the price and the 8 (constants read), the bans (no reader on
the hire's path), the customised types (applied as a placed hero's, E08's rows), the map Tavern
(the same hire; class 95 is an import blocker until the drafts place it). **What the harness
lacks** (`docs/experiments/README.md`): reads of the player record's two offers (+0x28, +0x2C)
and a hero's +0x105; a retreat and a surrender driven in a recipe (the battle lane's harness has
both); the Thieves' Guild window's capture.

## 10. Not read

The tutorial's fixed classes (`tavern_new_heroes`' branch on +0x1F69D), a campaign's carried
heroes (`hero_setup_redraw_by_class` 0x486110), the computer's hire logic (the other readers of
`hero_price`), the network (0x4821B7), the rumours' draw (0x4C8D48, E24's), the
tavern dialog's dismiss handler, the Thieves' Guild window's hints (`thieves_guild_hint` 0x5C9710:
the names over the portraits; presentation). The window's per-player parts: read by T33b (§13).

## 11. The checkpoint (T32; `TWN-Q16`, the user, 2026-11-15)

Asked interactively (`AskUserQuestion`, one round of four, a recommendation each); the epic's
decisions 103 to 107, each as recommended:

- **TD-18** (decision 103): the reading. The week releases every offer but a retreated or
  surrendered hero that came back after its owner's turn ended (the mark cleared at its owner's
  turn end and by a hire); row TWN-4-3 observes it.
- **TD-19 and TD-20** (decision 104): the reading for both. A returning hero takes a random one of
  its owner's two slots, the other when that one holds a returning offer, the offer it replaces
  freed; the offers' movement points are set at their owner's turn start, their spell points only
  at a draw or a leave.
- **T-14** (decision 105): the original's, not a quirk. A dismissed hero keeps its artifacts in
  the pool (VCMI agrees but drops the Grail, which the reading keeps); row TWN-4-5 confirms the
  dialog adds nothing.
- **The map's Tavern** (decision 106): in increment 4. Class 95 hires as a town's tavern onto its
  visited tile (refused while the visitor stands there), no guild spells; its object kind is the
  adventure lane's (`cross-lane`, placed by the drafts).
- **The agent's calls** (decision 107, change freely): TD-21 as read (one bound, the town part's
  `most_heroes`, on the map only); T-15 noted for the packs' validation (a hero's first starting
  stack is a creature); the SoD edition's Conflux rule (§2.2) to E11 as a lead (the battle lane's
  inbox); batch TWN-4's rows (§9) for T33's cut, run as TWN-1 to TWN-3 were.


## 12. The dismissal (read by T33a, 2026-11-16; decision 112)

The hero screen's button 129 dismisses (its result 0x81: `hero_leave_game` 0x4D9EC0 with (1, 0),
0x4E19C2; a lost hero's path, §4, without bit 17). `hero_screen_refresh` 0x4E1A50 enables it
(0x4E1C2B) unless:

- the caller passed its flag (`hero_screen_no_dismiss` 0x698A90 from `hero_screen_show`'s edx):
  the town screen (the visitor's and the garrison hero's portraits, `town_garrison_click`
  0x5D4EBF, 0x5D4F13) and the tavern dialog (an offer's screen, 0x5D7D7D) pass 1, so a garrison
  hero, off the player's list, is never dismissed;
- the hero stands on a town's entrance, its visitor (0x4E1A6B to 0x4E1A7A: the tile's kept class
  98 with its entrance bit);
- `hero_mission_critical` 0x5F2A40 holds: the loss condition "lose this hero" names its type, or
  a campaign's scenario keeps it (out of scope);
- the screen is view-only (`hero_screen_view_only` 0x6AA9D8);
- the player owns no town (player +0x3E) and has one hero on the map (+0x1).

The dialog asks first (GENRLTXT 23; presentation). Nothing on this path strips the hero's
artifacts (T-14, decision 105). VCMI 1.7.5 (`CHeroWindow::update`) blocks the button for a
player's one hero without a town, a mission-critical hero and a garrison held in an exchange
window; it does not test a town's visitor (a dispute only of the button: TD-22 noted, recommended
the reading). The other callers of `hero_screen_show` (the adventure map's hero list and its
selected hero, the exchange) not read for their flag.

## 13. The Thieves' Guild's rows and parts (read by T33b, 2026-11-16)

`thieves_guild_fill` 0x5DDA10 read from its rows' loop to its end, `thieves_guild_row_value`
0x5DEE70 read whole, `thieves_guild_rank_sort` 0x5DF160 and the window's handler
`thieves_guild_proc` 0x5C9930 at its clicks.

- **The players:** those still in the game (game +0x1F636 by colour clear), in colour order, a
  column each; a player out of the game values −1 in every row, ranks last and is not drawn.
- **A row's places:** every player's value sorted from the highest (`thieves_guild_rank_sort`: an
  exchange sort, so the order within equal values is not the colour order); equal values share a
  place, their flags in one column (0x5DDE9F to 0x5DDEDF). Only the places show, never a value.
- **The values** (beyond §6): towns, the towns whose owner is the player; heroes, its heroes on
  the map (player +0x1: a garrison hero is off the list, `E09-log.md` T22b); the resources, its
  treasury; obelisks, §6's; **artifacts**, over its heroes on the map the backpack's count
  (`hero_backpack_count` 0x4D90C0 with 1: +0x3D4) plus `hero_worn_count` 0x4D9070 with 1, which
  counts **nothing** with that argument (its flag's test skips every position, 0x4D9090; with 0,
  the tavern dialog's, it counts the worn artifacts but the spell book and the four war machines):
  the row ranks the backpacks alone (**T-17**); **army**, over its heroes on the map
  `hero_army_power` 0x427650 (the army's AI value times the Attack and Defense factor, truncated)
  plus, for each of its towns that `town_defended` 0x5BDFE0 holds (a visitor, or a creature in
  the shown army), the value of the army `town_garrison_army` 0x5C1460 shows (the garrison hero's,
  else the town's) without a factor (`army_power` 0x427690 with no hero: the 32-bit sum);
  **income**, `player_gold_income` 0x4C7930 with 1 (the day's gold: the gold mines, the towns and
  their garrison heroes, the gold artifacts, each listed hero's own; the computer's difficulty
  factor and the handicap, neither modelled here).
- **The parts** (each from its count; a Den's 99 shows all four): **1** the player's best hero:
  over its heroes on the map in its list's order the greatest `hero_army_power`, a strictly greater
  one replacing (the first of equals; a power of 0 is never chosen; none without heroes), shown by
  its portrait (its type); **2** that hero's four primary skills (+0x476 to +0x479), each shown
  clamped to 0–99; **3** the player's tactic (player +0x34, `ai_type`: a computer's drawn at the
  new game, 3 for a human) as a line of text: the computer's play (E13), no rule; **4** its best
  creature: over the armies its towns show (`town_garrison_army`, in its town list's order), then
  its heroes' on the map, the creature with the greatest AI value (creature record +0x40), a
  strictly greater one replacing; none without a creature; shown by its portrait.
- **The clicks** (`thieves_guild_proc`): a best hero's portrait opens its hero screen and a best
  creature's its creature dialog **only in the viewer's own column**; another player's opens
  nothing (its hint names it, `thieves_guild_hint` 0x5C9710).
- **VCMI 1.7.5** (`lib/gameState/GameStatistics.cpp`, `CGameState::obtainPlayersStats`): the best
  hero the one with the most experience (`findBestHero`, the first of equals); artifacts the worn
  and the backpack's (`getNumberOfArts`); army the heroes' strengths without the garrison heroes
  (`getArmyStrength`'s default) and no town's own garrison; the best creature over the heroes'
  armies only; income the owned objects' daily gold; the places as the reading (`getRank`). The
  manuals name the report only ("compares your performance against those of your opponents").
- **TD-23 — the best hero, the army row and the best creature.** The reading: by army power, the
  towns' shown armies in both; VCMI: by experience, the heroes' armies only (no garrison hero in
  the army row). Impact: which hero's portrait and skills, and which ranks, a viewer sees
  (information only). Recommended: the reading (batch row TWN-4-7).
- **T-17 — the artifacts row ranks the backpacks alone** (quirk candidate): `hero_worn_count`'s
  argument 1 turns its count off, so a hero's worn artifacts never count (VCMI counts them).
  Information only, no state touched. Recommended: the original's, no flag (`preserve`), row
  TWN-4-7 confirming.
- **Decided at `TWN-Q18`** (the user, 2026-11-16; the epic's decisions 117, 118): TD-23 the
  reading; T-17 a `fix-optional` flag (`TWN-QUIRK-13`: the backpacks alone in `faithful`, the
  worn and the backpack's off), not the agent's `preserve`.

## 14. Batch TWN-4 scored (T34, 2026-11-16)

Two runs of each part (the capture harness under Wine, the Complete 4.0 executable in SoD on Easy,
hot seat, Red and Blue human; `experiments/results/E09-batch-4a.md`, `-4b.md`, `-4c.md`, the main
records, and their `-run2.md`). The predictions are the specs' (`experiments/specs/E09-batch-4a.toml`
to `-4c.toml`). The harness (E09 T34; `clients/capture/README.md`, "Taverns and the Thieves' Guild")
reads the offers (player +0x28, +0x2C), every hero record by name on the map or off it, the pool
(game +0x4DF18), the tavern dialog and the Thieves' Guild window from memory, opens a town from the
town list, and xdd.dll logs the taverns' draws **with their ranges** (`tavern_class` 0x4BB7AA,
`tavern_type` 0x4BB7CF, `leave_slot` 0x4DA1C6): each class draw's range is the total of the weights
it drew over. Row TWN-4-5's "map with few free types" is the harness's narrowed pool (every free
type marked 0x40 but the lost hero's: the map format's allowed set is not in the writer). **Every
row as the reading, in every read of both runs; no reading refuted. Two points the reading had not
listed, read in the code after the runs: the new day clears the returning mark of every hero record
and gives every record its day's spell points (below).**

- **TWN-4-1 (§2.2, §2.3).** Eight weeks a run (the start and seven weeks by the written date), the
  four offers read on Red's turn: **slot 0 of the player's faction every time** (Red's 16 a Castle
  class, Knight or Cleric; Blue's 16 a Stronghold one, Barbarian or Battle Mage), **its class draw
  over 1-12 every time** (the faction's two classes, 6 each: own faction only); **slot 1 another
  class than slot 0's every time** (any faction: a Castle class in Red's slot 1 at run 2's M2 W2),
  its range **1-94 every time** (100 less the avoided class's 6). The type draws' ranges the class's
  free types (Red's Knights 1-3: five placed, Lord Haart not allowed; the Clerics 1-7). Every offer
  a **full starting army** (the first stack in its range; a second in 27 and 32 of the 32 offers of
  the two runs, 88 in 100 expected; a third in 5 and 1: Drakon's three stacks of Gnolls, Cuthbert's
  Griffins), its spell points Knowledge x 10 x the Intelligence factor (Andra 25), no returning
  mark; types came again (Tyris, Sir Mullich and Gird three times in run 1). **Confirmed** (the
  wiki's slot 0, VCMI's draw).
- **TWN-4-8 (§2.2, finding 6).** **All 32 slot-1 class draws of the two runs over 1-94**: the two
  Conflux classes weigh 5 for both players (the edition 2's rule would give 84). Conflux heroes
  offered in slot 1: Ciele (run 1, the start), Kalt and Inteus (run 2), and as replacements (TWN-4-2:
  Inteus, Kalt, Brissa). **The reading confirmed**: the SoD retail's rule is dead in this build.
- **TWN-4-2 (§3).** Three hires a run (R1 slot 0, R2 slot 1, R3 slot 0), each 2 500 gold: the hired
  hero **on the town's entrance** as its visitor (7, 10; 15, 10; 23, 10), the army it was offered
  with; **a hero with a spell book learned the town's guild spells** (run 1's Clavius, run 2's
  Xarfax with its own Fireball besides; the debug run's Fafner), a Knight none. **The slot refilled
  at once**: one draw pair, the class range **95 with the other slot's class weighing 5** (a Death
  Knight, a Demoniac, a Heretic, an Elementalist), **94 with it weighing 6** (a Wizard, a Witch),
  any faction (a Demoniac, a Witch, a Death Knight, Conflux classes), each new offer **one creature**
  of its first starting stack (1 Imp, 1 Gremlin, 1 Pikeman, 1 Gnoll, 1 Skeleton, 1 Pixie) with its
  spell points full. **Confirmed.**
- **TWN-4-6 (§3, TD-21).** Both runs alike: R1 with a visitor **disabled** (state 0xE; the status
  line TvrnInfo row 2, "a Hero in this town"); the visitor into the garrison (Red 5 on the map, the
  garrison hero off the list); **enabled**; after the hire, the hired hero the visitor: **disabled**
  (row 2); R3 at **7 on the map with a garrison hero** (8 heroes): **enabled**, a hire there to 8;
  R4 at **8 on the map**: **disabled** (TvrnInfo row 1, "%d Heroes"). **Confirmed**: the garrison
  hero counts in neither case (VCMI's cap of 16 in all is not the original's; TD-21 as decided).
- **TWN-4-3 (§4, TD-18).** Both runs alike (the same draws: the game's generator is reseeded the
  same way at each start and day). Day 6, Red's turn: TWN-4-3's retreat put it in slot 1
  (`leave_slot` 1), 1 Pikeman, its points 1 530 of 1 630 and spell points 3 as they were, marked;
  **on Blue's day 6 both marks clear** (Red's turn end); on Red's day 7 its points 1 560 of 1 560 (the
  turn start's, TD-20). Day 7, Blue's turn: TWN-4-3f's retreat into slot 1 (`leave_slot` 1, neither
  marked) freed TWN-4-3 (pool free), 1 Pikeman, spell points 4, marked. **Day 8 (week 2): TWN-4-3f
  still in slot 1** (its one Pikeman); slot 0's TWN-4-4 (day 6) **released and drawn anew**: the
  week's one Red draw pair (`tavern_class 3 of 1-6`, the Cleric avoided; `tavern_type 7 of 1-8`)
  picked Tyris again in both runs, now with a full new army (14 Pikemen, 7 Archers, 2 Griffins) and
  10 spell points in place of its kept 4 Swordsmen, 3 Monks and 2: a release, then a draw. **The
  reading confirmed**; VCMI's release of both refuted (TD-18 as decided).
- **TWN-4-4 (§4).** TWN-4-4's surrender (1 200 gold) into **the other slot** (`leave_slot` 1 drew
  TWN-4-3's marked slot: slot 0 taken), **its army kept** (4 Swordsmen, 3 Monks), points 1 530 and
  spell points 2 as they were, marked. TWN-4-4b's loss to 30 Black Dragons: owner none, **its type
  free**, no slot (Red's offers unchanged), **1 Centaur** (its first starting stack, not its 2
  Pikemen), **points 2 000 of 2 000** (the land maximum over the cleared army: the refill comes
  before the one creature, T-16's reading) and **spell points 10** (refilled from 1). TWN-4-5's
  replacements (below) drew it later with its record. **Confirmed.**
- **TWN-4-5 (§4, §12, T-14).** Dismissed (GENRLTXT 23, yes): owner none, its type free, **the Badge
  of Courage worn and the Clover of Fortune in the backpack kept**, 1 Gremlin, points 2 000 and
  spell points 10 refilled. With the pool narrowed to TWN-4-5's and TWN-4-4b's types, the first
  hire's replacement **drew TWN-4-5** (`tavern_class 11 of 1-12`: the Ranger's 6 and the
  Alchemist's 6, the other slot a Cleric) with its two artifacts as they were (no equipping at a
  replacement: the Clover still in the backpack) and 1 Gremlin; the second hire drew TWN-4-4b
  (`1-6`, the Ranger alone), 1 Centaur; the hired TWN-4-5 took its artifacts onto the map. **The
  reading confirmed: T-14 holds** (a dismissed hero's artifacts go back into the pool with it).
- **TWN-4-7 (§6, §13, TD-23, T-17).** Both runs alike, every read as the spec predicted: 2, 4, 6,
  8, 9 rows at one to five taverns (the Castle's Brotherhood with the Tavern counted once, N5) and
  9 at the Den (its message ADVEVENT 142); the parts from 1 (the best hero), 2 (its skills), 3 (the
  tactic: both human, GENRLTXT 500) and 4 (the best creature). **The best hero Red's TWN-4-7s**
  (the strongest army) and its skills **5 5 1 1**, not the most experienced TWN-4-7e's; **artifacts
  1st Blue** (one in a backpack against Red's three worn: the backpacks alone, T-17); **army 1st
  Red** and **Red's best creature the Archangel** (R1's garrison counted). Equal values share a
  place (towns at one tavern, obelisks). **The reading confirmed; VCMI's best hero, artifacts, army
  and best creature refuted** (TD-23, T-17 as decided).

**Not predicted, read in the code after the runs** (`new_day` 0x4C7FE0, read at these sites):

- **The returning mark lasts until the next new day at most.** After the week's and the month's
  events (0x4C8017, 0x4C8031) every hero record (156: 0x4C80CF to 0x4C80FD) has bits 0 and 17 of
  +0x105 cleared (the address map's E08 T34b reading): TWN-4-3f's mark was gone on Red's day 8 (its
  prediction said "until Red's turn ends"). §2.3 listed only the owner's turn end and a hire. The
  week's release (before the clear) keeps a marked offer, so TD-18 stands; a difference shows only
  for a second leave into the same tavern on a later day before its owner's turn ends (the slot
  `leave_slot` avoids).
- **Every hero record regains its day's spell points** (0x4C8237: the 156 records, on the map or
  not): 1 plus Mysticism and the mana artifacts (`hero_mana_regained` 0x4E3F40), up to Knowledge x
  10 x the Intelligence factor, the Wizard's Well's full: TWN-4-3f's kept 4 were 5 on day 8. §2.3's
  "spell points only at a draw or a leave" is wrong for the days between: a tavern offer (a
  returning one with its kept points, a week's below its maximum) and a hero out of the game gain
  them every day.

Both for T35's checkpoint (design note §63, §67, §68; the drafts `adr-drafts/0017-increment-4.md`):
recommended as the original (faithful), the new day's clear of the mark after the week and the
day's spell points over every hero record. **Decided at T35's start** (`TWN-Q19`, the
epic's decisions 126 and 127): both as the original, in ADR-0017 point 3 and ADR-0013 point 16.5
(E09 T35a).
