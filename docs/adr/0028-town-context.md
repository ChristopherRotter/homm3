# ADR-0028: The town context: `h3-towns`, the store of towns, its ruleset part and seams

- Status: Accepted (user, 2026-11-02, TWN-Q4: the draft accepted as written; moved here by E09
  T4a, 2026-11-03, with batch TWN-1's outcome, survey §15). Its choices were made by the user:
  the crate and the split of a town at E09's refinement (the epic's decisions 2 and 3, TWN-Q1),
  the store, the pool, the part and the moat's data at the drafts (decisions 17 to 20, TWN-Q3).
- Date: 2026-11-02
- Deciders: user; town lane session (E09 T2a)
- Amends with it: ADR-0015, ADR-0016, ADR-0017, ADR-0023 (E09 increment 1's amendments, each in
  its revision history or status line); ADR-0016 and ADR-0017 again with increment 2's (E09 T15,
  2026-11-07; `TWN-Q10`, decision 48); ADR-0013, ADR-0016, ADR-0017 and ADR-0023 with increment
  3's (E09 T24, 2026-11-12; `TWN-Q14`, decision 83); ADR-0013, ADR-0016 and ADR-0017 with
  increment 4's (E09 T35a, 2026-11-16; `TWN-Q18`, `TWN-Q19`, decisions 121, 126, 127); ADR-0013,
  ADR-0016 and ADR-0017 with increment 5's (E09 T45, 2026-11-20; `TWN-Q31` to `TWN-Q33`,
  decisions 162 to 181); ADR-0016 and ADR-0017 with increment 6's (E09 T55a, 2026-11-25;
  `TWN-Q35` to `TWN-Q37`, decisions 197 to 217).
- Working detail: the survey `docs/rules/surveys/E09-towns.md` (§0's findings, §15 batch TWN-1),
  the design note `docs/epics/E09-design.md` (T2b); increment 2's the survey
  `docs/rules/surveys/E09-construction.md` (§13 batch TWN-2) and the design note's Part D (T13).

<!-- Decisions only; the readings are the survey's, the build's detail the design note's. -->

## Context

E09 builds towns on E08's adventure map. `docs/03-domain.md` gives the town context buildings,
construction, growth, recruitment and the town's services; decision 2 keeps the town as a map
object (its tiles and entrance, owner, garrison, visiting hero) in the adventure, as a mine and
its guards, and has `h3-game` compose the visit, recruitment, the town's part of the day and the
siege's facts. ADR-0005 forbids edges between domain crates; ADR-0015 point 5 makes identities
that cross contexts contract types; ADR-0017's store of heroes is the shape of per-entity state
that forks share. Increment 1 reads from a town its faction, its buildings at the start and the
forbidden ones, its hall (the day's gold), its fort (the siege, the views), its Resource Silo, its
Tavern (another player's view) and the Capitol's limit; every later increment adds state to the
same record (the day's construction, the available creatures, the guild's spells).

## Decision

1. **Two new crates.**

   | Crate | Layer | Logic | Holds |
   |---|---|---|---|
   | `h3-towns` | domain | yes | the town context: every town's own record, its rules and derivations, its ruleset part and that part's compiler |
   | `h3-pack-towns` | adapter | no | `packs/*/towns.toml` → `h3-towns`' definitions |

   `h3-towns` is a logic crate (`no_std`, `check-arch` purity, the 100 % coverage gate). It
   depends on the kernel, the core content, `h3-contracts` and the hero model `h3-heroes` (the
   shared layer, for the services that change a hero from increment 5); no domain crate depends
   on it, `h3-game` does. Both crates and the pack files `packs/*/towns.toml` are the town
   lane's (`lanes.toml`).
2. **A town is split between two contexts, each fact held once** (decision 2):
   - **`h3-towns`** holds the town's own record: its **faction**, the buildings **built** and
     **forbidden**, its map **events** (kept from the import for the increments that apply them;
     increment 1 applies none), the map's name if it gives one (an unnamed town's name is the
     client's: the original's lists are text); later increments add the day's construction, the
     available creatures, the guild's spells. Its rules are pure functions of a town, the part
     and plain inputs: the **hall level** and the **day's income** (the gold by the highest hall,
     the Grail's, the silo's resources), the **fort level**, the **siege's facts** (point 7), the
     **reveal radius** of a capture, whether it has a **Tavern** (another player's view level),
     the **Capitol's limit** (given the owner's other towns).
     **Increment 2** (construction with the town events; decisions 37, 41–47) adds to the town's
     record **the day's mark** (whether a building was bought in it today, the original's +0x2,
     decision 43) and to its rules, each a pure function of a town, the part and plain inputs the
     game gives: **buildable now** (not ever built, every requirement in the ever-built set, not
     forbidden, not of role `grail`, a hall-4 building only while the player holds no Capitol,
     the day's mark clear; the hall's offer: a horde whose level's upgraded dwelling is ever built
     is offered as its upgraded horde), **the build** (the building, what it supersedes and what
     then builds automatically into the ever-built set; the mark set or not), **an event's
     buildings** (the event's set less what is ever built, what another of the event's buildings
     supersedes and a `shipyard` without water; in the part's order) and **the mark's life**
     (cleared at every day's start; at a capture kept, but cleared when the old owner is not human
     and the new one is). The record's map events are applied from increment 2 (resources and
     buildings; their creatures with increment 3's recruits, decision 37). The adventure holds
     the town's **boat spot** (ADR-0016 point 1); "whether the town has water" is the game's
     input to `h3-towns`, never a second copy. Every derivation of increment 1 (hall, income,
     fort, siege, reveal, Tavern, Capitol) reads the record as a build leaves it, so a build
     changes them at once.
     **Increment 3** (growth; E09 decisions 65, 69 to 78) adds to the town's record **the available
     creatures**, a count by the faction's **index** (0 to 6 the levels' creatures, 7 to 13 their
     upgrades; the original's i16 counts +0x16), each at most 32 767 (T-10 fixed: a count saturates
     where the original's wraps), and **the Portal of Summoning's** creature and count (+0x3C,
     +0x40; none until one is drawn). Its rules, each a pure function of a town, the part, the core
     and plain inputs the game gives:
     - **A dwelling's growth** at an index (TWN-GRO-001; survey §1): none without the level's
       dwelling ever built, none at a base index whose upgraded dwelling is ever built; else the
       creature's growth (the core's), plus the castle's (a standing fort level 3 adds the growth
       again, level 2 half of it, truncated); for an owned town plus the owner's share of the two
       (the Statue of Legion's half, truncated) and the Legion's amount at the index's level (the
       inputs); plus the first ever-built horde of the index (its creature's horde growth, the
       core's); plus the owner's dwellings of the index's creature (an input); then, with a building
       of role `grail` ever built, plus half of the sum, truncated.
     - **The week's growth** (TWN-GRO-003; §2): each index whose growth is above 0 gains it, plus
       the week's bonus when its creature is one of the week's, halved (truncated) in a town without
       an owner, saturating.
     - **The month** (TWN-GRO-004; §3): in a creature's month each index whose growth is above 0 and
       whose creature is the month's doubles its count (the exact creature, T-13 `fix-optional`:
       with its flag off, both indexes of the level holding it), not in a month whose first week is
       a `week_of` week; in the Plague each such index loses its growth (at least 0), then halves
       (T-12 `fix-optional`: with its flag off, a neutral town loses the half growth it got).
     - **A dwelling's first creatures** (TWN-GRO-005; §6): a dwelling built sets its index's count
       to its creature's growth; an upgraded dwelling built moves the base index's count to its own.
       The setup's, a command's and an event's builds alike.
     - **A town event's creatures** (TWN-EVT-004 amended; §6): each level's count to the upgraded
       dwelling's index when it is ever built, else to the dwelling's, else none; saturating.
     - **A neutral town's garrison week** (TWN-GRO-006; §4, TD-15 by the reading): with draws from
       the stream the game passes (the towns'), whether its garrison grows (the part's odds by a
       standing fort), the level (three draws, widening with the weeks), the creature (the level's,
       its upgrade when the garrison holds the upgrade), its room (a stack of it, an empty slot,
       else the stack of least AI value times count below the new one's, replaced; else nothing),
       the town's two counts of the level lowered by it, then a 5-in-100 upgrade of every stack of a
       base creature. It reads and returns the garrison as `h3_contracts::ArmyStack`s; the adventure
       still holds the army.
     - **The Portal of Summoning** (TWN-GRO-007; §5, TD-16 by the reading, T-11 by decision 71):
       given the owner's dwellings' creatures in object order, one dwelling drawn, then one of its
       creatures; the count set to that creature's growth (not added; no castle, Grail, week, month
       or Plague); no dwelling: none. The same draw when the Portal is built, when its owner flags a
       dwelling while it has none and at a capture that finds it empty (decision 82).
     Every derivation reads the record as the last write left it, so a build, an event and a week
     change the counts at once. Batch TWN-3 confirmed the readings (E09 T23, survey `E09-growth.md`
     §17: TWN-3-1 to -5; T-12 and T-13 stand `fix-optional`).
     **Increment 3's recruitment** (TWN-REC-001, -002; survey §5, §8) adds three rules over the same
     record: **the source** of a standing building (ever built, no built building superseding it) of
     role `dwelling` (its index), `horde` (its horde's index) or `portal` (the Portal), else a typed
     refusal, as is a base index whose upgraded dwelling is ever built (its count moved, its growth
     none); **the stock** of a source: an index below 7 its creature with the index's count, an
     upgraded index its upgrade then its base creature, **both with the index's one count** (the
     original's second creature), the Portal its creature and count (none while nothing is
     summoned); **the purchase**: a creature of the stock, an amount from 1 to the count, typed
     refusals otherwise, and its write, the source's count lowered by the amount. The army the
     creatures join, its room and the price are not the town's (point 6). A town holds **no garrison
     hero**: the hero in the garrison and its army are the adventure's (ADR-0016 point 3; batch
     TWN-3's TWN-3-6: a purchase joins the garrison hero's army).
     **Increment 5** (the mage guild; E09 decisions 162, 167 to 176) adds to the town's record
     **its guild's slots**: per spell level 1 to 5 the part's slots of the level plus one, the
     **spare** (6, 5, 4, 3, 2: the original's `guild_spell_slots` 0x6782A4 plus the slot the
     Tower's Library shows), each a spell of the core or empty, **drawn once when the game is
     built** for every town, whatever it has built (the original's `town_setup_spells` 0x5BE600;
     survey `E09-guild.md` §3), and never drawn again (a capture, a build, a week keep them); **the
     Mana Vortex's week flag** (the original's +0x33: set when the game is built and at every
     week's first day, cleared when a hero takes the Vortex; every town holds it, only a standing
     building of role `mana_vortex` reads it); and, static, **the town's Aurora set** (the spells a
     building of role `aurora` teaches there, fixed at the build: every hero spell the town's map
     record does not ban, less those granted by name only, less the scenario's bans when
     `TWN-QUIRK-14`'s flag is off). Its rules, each a pure function of a town, the part, the core
     and plain inputs the game gives:
     - **The draw** (TWN-GLD-002; survey §3): with draws from the stream the game passes (the
       towns', ADR-0017 point 4; E09 decision 179) and the town's **required** and **banned**
       spells and the scenario's bans (the setup's, TWN-SET-012, GAM-SET-003), each level's slots
       in order: the first required spell of the level in spell order not excluded, else one drawn
       by the faction's guild weights (point 5) among the level's spells not excluded, else the
       slot empty; **excluded** a spell the town bans, the scenario bans, one granted by name only
       (Titan's Lightning Bolt, the original's flag 0x2000) and every one taken. A required spell
       the town bans is never taken (TD-25, E09 decision 168).
     - **What the guild shows** (TWN-GLD-003; §3.3, §4): the **guild level**, the highest level of
       a standing building of role `guild` (0 none; the faction's highest is its record's
       buildings); each level up to it shows its first slots, the part's count plus one with a
       building of role `library` ever built, fewer where the draw left the end of the level
       empty; a level above it shows none. A level or the Library built later shows at once.
     - **What a hero learns there** (TWN-GLD-004; §5, §11): with a building of role `guild` ever
       built and a worn spell book (the details' test, `h3-heroes`), in a town with a building of
       role `aurora` ever built every spell of the Aurora set whose level is at most the hero's
       learnable level (HER-SPL-005), the guild level notwithstanding; else every spell the guild
       shows at a level up to the learnable level; each the hero does not know, in level then slot
       order (the Aurora's in spell order), returned as `h3_heroes::HeroChange::LearnSpell`s (point
       6). No owner test, no class.
     - **The Mana Vortex** (TWN-GLD-009; §15): with a standing building of role `mana_vortex` and
       the week flag set, a hero whose spell points are below the part's Vortex factor (2) times its
       maximum (HER-MAN-001) gets that many (`HeroChange::SetSpellPoints`) and the flag is cleared;
       a hero at or above it leaves the flag set. No spell book needed.
     - **The night's refill's town** (TWN-GLD-008; §14): whether a building of role `guild` was
       ever built (the game raises the town's heroes' points).
     - **The spell book's check** (TWN-GLD-007; §6, §13): a building of role `guild` standing, else
       a typed refusal; the price the part's (point 5).
     Batch TWN-5 (E09 T44; survey `E09-guild.md` §21) read every rule so: a spell both required and
     banned in no slot, the map's ban and Titan's Lightning Bolt in none, no spell twice in a level
     (TWN-5-1, TD-25); the Aurora teaching 68 spells at Mage Guild 1, the map's ban among them
     (TWN-5-2, TD-26; T-18 confirmed); a Tower's guild showing 5, 4, 3, 2, 1, with the Library 6,
     5, 4, 3, 2 (TWN-5-3); the Vortex once a week a town, the garrison hero first (TWN-5-5).
     **Increment 6** (the town's trades; E09 decisions 197, 201 to 213) adds **nothing to the
     town's record** (design note `E09-design.md` Part H §101, call 14): the Artifact Merchants'
     stock and the artifact pool's marks are the adventure's (ADR-0016 point 1; E09 decision 212),
     the docked ship (building 20) is the boat spot's occupancy, derived where a client asks (T-27,
     decision 207); the town state hash stays v4. Its rules, each a pure function of `h3-towns`
     over the store, the part, the core and plain inputs the game gives:
     - **The count** (TWN-MKT-001; D-70, TD-29; survey `E09-trades.md` §2.1):
       `h3_towns::marketplaces(store, part, player)`, the number of the **trader's** towns (the
       window's owner's) with a building of role `marketplace` ever built (a capture moves one to
       its new owner's count at once), whoever owns the town traded at, the town itself needing
       none; the game clamps it to the market parameters' rows, at most 10 and **at least 1**
       (decision 203: a trader without one trades as with one; T-19 fixed, `TWN-QUIRK-17`). D-70's
       rows are E08's `MarketParams` (wood or ore for 37, 62 and 112 gold at 2, 4 and 8; decision
       201). Batch TWN-6 confirmed both (E09 T54, survey `E09-trades.md` §14): the rows at 2, 4 and
       8 Marketplaces (TWN-6-1; VCMI's 38, 63 and 113 refuted) and the trader's count at an ally's
       town, there 0, every price 0 and the artifact given free (TWN-6-2: T-19).
     - **What a town's window offers** (TWN-MKT-002; §2.2): **trades and the transfer** whenever it
       opens; **the artifact purchase and sale** with a hero where a building of role
       `artifact_merchants` stands; **the creature sale** with a hero where one of role
       `freelancers` stands (a town holds at most one of the two). **It opens** where a building of
       role `marketplace` stands (with or without a hero, §2.1), or one of role
       `artifact_merchants` or `freelancers` stands and the window has a hero (point 6); else
       refused, `NoMarket` (none of the three stands) or `NoVisitor` (no hero the rule names: the
       original's GENRLTXT 274). Confirmed by batch TWN-6's TWN-6-5 (TD-28): with a garrison hero
       and no visitor the merchants, the Freelancer's Guild, the Blacksmith and the Ballista Yard
       each answer GENRLTXT 274 and open nothing, the Marketplace's window opens.
     - **The Blacksmith's machine** (TWN-SMT-001; §4.3, §4.6): the named building of the town's
       faction's record, standing, with a `war_machine` (`NoWarMachine` else; `UnknownBuilding`,
       `NotStanding` as the recruitment's); the machine that artifact, its price the machine's
       creature's cost in the core, resource by resource (T-25, decision 207, `TWN-QUIRK-22`; 2 500,
       1 000 and 750 gold with the shipped tables).
     - **The boat, the University and the Transformer** (TWN-SHY-001, TWN-UNI-001, TWN-TRF-001;
       §5.3, §6.3, §7.5): a standing building of role `shipyard` (`NotStanding`), `university`
       (`NoUniversity`) or `transformer` (`NoTransformer`); the part's boat price and type, its four
       schools and their price (point 5); a creature's target by the part's table,
       `transformer_dragons_into` for the listed dragons and hydras, else `transformer_into`. The
       spot and its state are the adventure's (one fixed tile, TD-31, confirmed by batch TWN-6's
       TWN-6-6), the scenario's banned skills the game's (point 6).
   - **The adventure** holds the map object of kind `Town` (ADR-0016 points 1, 3 and 4, E09's amendment): its static
     part (footprint, entrance, the `TownId`), its **owner**, its **garrison** (the armies on
     objects, as a mine's guards) and its **visiting hero** (the hero on its entrance).
   - **`h3-game`** composes them (ADR-0017 points 1, 3, 4, 10 and 11, E09's amendment): the build (a random town's faction, the
     starting heroes and the pool), the visit and the capture, the siege's setup, the day's
     income, the views.
3. **`TownId`** (`h3-contracts`, ADR-0015 point 5): a town's index among the map's towns (classes
   77 and 98) in object order, dense from 0, fixed when the game is built (no rule creates or
   removes a town). The adventure's town object holds it, `h3-towns`' records are by it, and the
   commands, events and observations name a town by it.
4. **The store of towns** (decision 17): `GameState` holds `h3-towns`' `Towns` beside the
   adventure's state: every town's record by `TownId`, each shared between forks and copied on
   its first write, the list shared too (a fork costs one reference count; a write copies the
   list of references and the one town), as the store of heroes (ADR-0017 point 1). Nothing sees
   the sharing (ADR-0016 point 2's rule): `h3-towns` has its own canonical state hash with a
   version of the town lane's, which the game state hash folds in, in `TownId` order. A map
   without towns has an empty store.
5. **The town ruleset part** (decision 19): `TownRuleset`, compiled on the core as the adventure
   and battle parts (ADR-0015 point 2, its R01 amendment point 2): its own canonical hash and
   version, the core's recorded beside it; core entities (factions, creatures, resources) by
   handle, by key in its source. Per faction a **town record**: its buildings (identities, the
   hall and fort level each gives, which it supersedes), the silo's resources, the moat and the
   siege's natural obstacles (point 7); the part's parameters (the gold by hall level and the
   Grail's, the neutral garrison's odds by level, the standard set's level-2 dwelling chance, the
   hall a second Capitol becomes). Only what an increment reads; increment 2 adds requirements
   and costs. One file per pack folder, `towns.toml` (base: the eight factions; AB: Conflux;
   SoD: its overrides), overrides as ADR-0006's. `GameRuleset` bundles the part; a replay's
   header and the server's manifest record its hash. Content is harvested where the original
   has it as data; a constant of the executable is tier-2 data whose source is the reading
   (decision 10).
   **Increment 2** gives each building of a town record its **direct requirements** (buildings
   of the same record: the executable's lists, survey `E09-construction.md` §6) and its **cost**
   (resources of the core, `Building.txt`'s rows, §5); the compiler closes the requirements (the
   original's `building_requirements` 0x697798) and refuses a requirement outside the record, a
   cycle, a building requiring itself and a resource the core lacks. A new role **`shipyard`**
   names the building that needs water (decision 46). The content is harvested (decision 23's
   route: `h3-harvest`'s gap-fill, `cross-lane`): the costs from `Building.txt`, the requirements
   from the executable's lists; TD-9 the executable's (decision 41), as batch TWN-2 observed it
   (survey §13; no gap-fill override).
   **Increment 3** gives buildings three roles: **`horde`** (a level and whether upgraded, as
   `dwelling`: its growth is the core's `horde_growth` of that index's creature, the figures the
   executable's `town_hordes` 0x6887A0 copies), **`portal`** (the Portal of Summoning) and
   **`week_of`** (a level: while the building stands in any town, every week is the week of that
   level's two creatures of its faction, the Deity of Fire's week of the Imp). Each town record
   gains **`roe`** (whether its creatures may be the week's on an RoE map; false for the AB pack's
   Conflux). The parameters gain the week's creature chance (1 in 4) and bonus (5) and a neutral
   town's garrison week (the odds 40 and 80 in 101 by a standing fort, the level weights 2, 3, 4, 5,
   4, 3, the weeks' cap 8, the upgrade's 5 in 100). The creatures' growths stay the core's. The
   content: `horde` harvested from the executable's horde table, `portal` and `week_of` the
   harvest's gap-fill roles (the Dungeon's and the Inferno's specials), the parameters tier-2 data
   of the reading.
   **Increment 4** (the Tavern; E09 decisions 99, 110) gives each town record its **tavern
   weights**: per hero class of the core (by key) the weight of that class in a tavern's draw for a
   player of the record's faction (`HCTRAITS.TXT`'s tavern column of the faction: 5 or 6 a class in
   this build's data, each column summing to 100; survey `E09-tavern.md` §2.2). A class the record
   does not list weighs 0; the compiler refuses a class the core lacks, a class listed twice and a
   record whose weights sum to 0. An edition without Conflux has no Conflux record, so RoE's missing
   column needs no override (the inbox's E04a T3a note). The parameters gain the **hire's price**
   (resources of the core: 2 500 gold, `hero_price` 0x67814C, the same for every hire and every
   edition; tier-2 data of the reading). The role `tavern` (increment 1) names the building that
   opens a town's tavern; `most_heroes` (increment 3) bounds the hire. The content: the weights
   harvested from both LODs' `HCTRAITS.TXT` (the base pack's eight records, the AB pack's Conflux;
   `h3-harvest`, `cross-lane`), the price tier-2 data. Batch TWN-4 (E09 T34; survey `E09-tavern.md`
   §14) read the weights through the draws' ranges: 12 for a faction's two classes in slot 0, 100
   less the avoided class's 5 or 6 in slot 1 and at a replacement, the Conflux classes weighing in
   every such draw (TWN-4-1, -2, -8), and the price 2 500 gold at every hire (TWN-4-2).
   **Increment 5** (the mage guild; E09 decisions 162, 173) gives each town record its **guild
   weights**: per spell of the core (by key) the weight of that spell in the record's guild draw
   (`SPTRAITS.TXT`'s town column of the faction, the original's spell record +0x44 + 4 × faction;
   each level's weights summing to 100 at every level the faction's guild reaches, 0 at the others:
   survey `E09-guild.md` §2.2). A spell the record does not list weighs 0; the compiler refuses a
   spell the core lacks or holds as a creature ability, and a spell listed twice. An edition
   without Conflux has no Conflux record (RoE's missing column needs no override; RoE's own values
   are E11's). Four new **roles**: **`guild`** (a level, 1 to 5: the Mage Guilds, read standing for
   the guild level and ever built for a guild's presence), **`library`** (at most one building of a
   record: the Tower's Library, building 22, one more slot shown at every level; read ever built),
   **`aurora`** (at most one: the Conflux's Aurora Borealis, building 26, every spell of the town's
   Aurora set taught; read ever built) and **`mana_vortex`** (at most one: the Dungeon's Mana
   Vortex, building 21; read standing). The parameters gain **the guild's slots** by level (5, 4,
   3, 2, 1, `guild_spell_slots` 0x6782A4; the spare is the rule's one more), **the spell book's
   price** (resources of the core: 500 gold, `mage_guild_open` 0x5CE560; the same for every
   purchase and edition) and **the Vortex's factor** (2, `town_visit_buildings` 0x5BD8E0), each
   tier-2 data of the reading. The spell book itself is the core's (the first artifact with the
   `spell_book` effect, as HER-ART-007's). The content: the weights harvested from both LODs'
   `SPTRAITS.TXT` (the base pack's eight records, the AB pack's Conflux; the two copies' columns
   are equal; `h3-harvest`, `cross-lane`), the roles the harvest's gap-fill (the Mage Guilds by
   their keys' levels, the three specials by faction), the parameters tier-2 data. Batch TWN-5
   (E09 T44) saw the slots through the shown counts (TWN-5-3) and the Vortex's factor as 40 for a
   maximum of 20 (TWN-5-5); a Castle's level-5 slots stayed empty, its weights 0 there.
   **Increment 6** (the town's trades; TWN-TRD-001; E09 decisions 197, 213) gives the town records'
   buildings **six fields** (as `tavern`, `guild` and `library`): **`marketplace`** (a flag:
   building 14 of every faction, the count's building and the window's entry),
   **`artifact_merchants`** (a flag: the Tower's, the Dungeon's and the Conflux's 17),
   **`freelancers`** (a flag: the Stronghold's 21), **`war_machine`** (an artifact of the core with
   a `war_machine` creature: each faction's Blacksmith 16 by `blacksmith_war_machines` 0x6AA9F8,
   Castle, Dungeon and Conflux `ballista`, Tower, Inferno and Stronghold `ammo_cart`, Rampart,
   Necropolis and Fortress `first_aid_tent`; the Stronghold's Ballista Yard 22 `ballista`),
   **`university`** (a flag: the Conflux's 21) and **`transformer`** (a flag: the Necropolis's 22).
   The compiler refuses a `war_machine` artifact the core lacks or that names no creature, and each
   flag twice in a record (`marketplace` too: one count a town). The parameters (`[town_params]`)
   gain **`merchants_stock`** (the classes in entry order: treasure three times, minor three times,
   major; `new_game_heroes` 0x4C0484, survey `E09-trades.md` §3.1, confirmed by batch TWN-6's
   TWN-6-8 with D-32, the map's artifacts left out while the pool lasts; at most the adventure's
   `MAX_STOCK`, 8, none `special`), **`boat_price`** (1 000 gold, 10 wood) and **`boat_type`** (1,
   every town's; code, §5.3), **`university_skills`** (Fire, Air, Water and Earth Magic by key;
   code, §6.3) and **`university_price`** (2 000 gold, §6.4), and **the Transformer's table**
   (`skeleton_transformer_targets` 0x6412FC, §7.5): `transformer_into` (`skeleton`),
   `transformer_dragons_into` (`bone_dragon`) and the creatures that become it, each pack naming
   its own as the Altar's factions (`[[transformer_dragon]]`: the base pack's Green, Gold, Red and
   Black Dragons, the Hydra and the Chaos Hydra; the AB pack's Azure, Crystal, Faerie and Rust
   Dragons). The most Marketplaces counted is E08's `MarketParams::counts` (10). Each is tier-2
   data of the reading, harvested or the harvest's gap-fill (`h3-harvest`, `cross-lane`). **Not
   data:** the screens a town offers (point 2's rule), the visitor-only rule (`TWN-QUIRK-15`'s
   flag), the sale's refusals (the core's slots). **The core** (`shared/`, E09 T55b; HER-ART-008,
   design note Part H §109): a combination artifact whose pack names no cost costs the
   resource-wise sum of its parts' costs (the original's `artifact_table_load` 0x44D031; SoD's
   twelve sums, from the Angelic Alliance's 84 000 down to 3 000, survey §3.4; confirmed by batch
   TWN-6's TWN-6-4: the Ring of the Magi's 3 000 sold for 600 gold at one Marketplace), so the
   merchants, the Black Markets and the sale read one cost; the core ruleset hash moves.
6. **Seams.** `h3-towns` and `h3-adventure` never import each other; `h3-game` reads both. The
   only new contract type of increment 1 is `TownId`. The siege's facts cross to the battle as
   ADR-0023's `Siege` (point 7). From increment 5 a town's service reads a hero's details by
   reference and returns `h3_heroes::HeroChange`s, which the game applies (ADR-0017 point 10), as
   a visit of the adventure does; recruitment returns the creatures for the game to place in an
   army the adventure holds.
   **Increment 2** adds the **town command**, `h3_towns::TownCommand` (decision 45; increment 2's
   one variant `Build { town: TownId, building: BuildingId }`), which the game wraps as
   `GameCommand::Town` (ADR-0017 point 1) and whose later variants are the town's own actions
   (increment 3's recruitment, increment 5's services). `h3-towns` validates the town's part
   (buildable now) and names the price (the part's cost), each refusal typed
   (`TownCommandError`); the game checks the turn, the owner and the treasury, pays and applies
   the build infallibly (ADR-0017 point 10). No new contract type: the command and its errors are
   `h3-towns`' types, which `h3-game` re-exports. The town events cross the same way: `h3-towns`
   says which buildings an event builds, the game pays its resources and writes the store.
   **Increment 3's growth** crosses as plain inputs and the store's writes: the game gives each rule
   whether the town is owned, the owner's share (`h3-heroes`' town growth of the owner's heroes,
   ADR-0013 point 16.4), the Legion's amounts (the garrison hero's and the visitor's), the owner's
   dwellings' creatures (the adventure's owners and kinds), the week's kind and the stream; it
   writes the counts and the Portal into the store and the returned garrison into the adventure
   (ADR-0016 point 1) in one infallible application. No new contract type.
   **Increment 3's recruitment** adds the town command's second variant, **`TownCommand::Recruit {
   town, building }`** (the original's click paths name a building: a dwelling, a horde, the
   Portal): `h3-towns` gives the source or its typed refusal; the game checks the turn, the owner
   and that no decision is open, and opens E08's **hire dialog** (ADR-0017 point 6) with the town's
   source and the garrison as its target, its creatures and counts read from the store at each view.
   **A purchase crosses three contexts**, as E08's purchase rule (ADV-HIR-002) parts it: `h3-towns`
   checks the creature and the amount against the stock, the adventure the room in the target army
   (the garrison hero's army when one stands, else the town's garrison; ADR-0016 point 3) and the
   game the treasury against the core's price; then the game pays, writes the lowered count into the
   store and calls the adventure's join, in one infallible application. The town's refusals reach
   the player as the hire dialog's (`HireError`'s offer and amount), one vocabulary for both
   sources. No new contract type: the adventure's `HireSource` and `HireTarget` name the town by
   `TownId` and the index as a plain number.
   **Increment 4's hire** (TWN-TAV-003; E09 decision 111) adds the town command's fourth variant,
   **`TownCommand::Hire { town, slot }`** (the town screen's tavern: `town_tavern_click` 0x5D82B0,
   `town_tavern_hire` 0x5C12E0): `h3-towns` checks the town's part (a standing building of role
   `tavern`, `has_tavern`) and names the price (the part's), each refusal typed
   (`TownCommandError`); the adventure checks the entrance (no visitor) and places the hired hero
   there; the game checks the turn, the owner, that no decision is open, the slot's offer, the
   player's heroes on the map against `most_heroes` and the treasury, then pays, places, marks the
   pool and draws the slot's replacement, in one infallible application (ADR-0017 point 10; batch
   TWN-4's TWN-4-6: the button disabled with a visitor and at 8 heroes on the map, a garrison hero
   not counted). The adventure map's Tavern (class 95) hires through the game's tavern decision
   (ADR-0017 point 6), not a town command: `h3-towns` takes no part in it but the part's price. No
   new contract type: the slot is a plain number (0 or 1).
   **Increment 5** (the mage guild; E09 decisions 170, 175, 176) adds the town command's fifth and
   sixth variants. **`TownCommand::Open { town }`** (TWN-GLD-006; TD-27's opening, E09 decision
   170: the original's town screen opened from the town list, `town_screen_open` 0x5BE210, and its
   guild's window, both free and at will): `h3-towns` names nothing but the town's part (the
   Vortex's and the learning's predicates of point 2); the game checks the turn, the owner and
   that no decision is open, then gives the Vortex (the garrison hero, then the visitor) and
   teaches (the visitor, then the garrison hero), in one infallible application; nothing else, so
   an opening that changes nothing is legal and changes nothing. **`TownCommand::BuySpellBook {
   town }`** (TWN-GLD-007; the guild's offer, `mage_guild_open` 0x5CE560): `h3-towns` checks the
   town's part (a standing guild) and names the price (the part's), each refusal typed
   (`TownCommandError`); the game checks the turn, the owner, that no decision is open, the hero
   (the visitor, else the garrison hero; none a refusal) and that it wears no spell book, then the
   treasury, then pays, gives the book and teaches the town's heroes, in one infallible application
   (ADR-0017 point 10). Learning everywhere crosses as the heroes' changes (point 2's set, read
   from the details by reference, applied by the game as `HeroChange::LearnSpell`, as a visit of
   the adventure), the Vortex as one `HeroChange::SetSpellPoints` and the store's write of the
   cleared flag. No new contract type. Batch TWN-5's TWN-5-5 (E09 T44) saw the opening give the
   Vortex at will, the garrison hero first, and a turn's start never.
   **Increment 6** (the town's trades; E09 decisions 207, 209 to 213) adds five variants, each
   `h3-towns` checking the town's part (point 2) and the game the turn, the owner and that no
   decision is open, each refusal typed (`TownCommandError`) and changing nothing; every one is the
   town owner's (an ally's town waits for E10's teams, `TWN-STUB-21`). **`TownCommand::Market {
   town }`** (TWN-MKT-002; decision 209, as `Recruit` opens the hire dialog) names no building: the
   game names **the window's hero** (the visitor; with `TWN-QUIRK-15`'s flag off, `strict-fixed`,
   T-28, decision 202, the garrison hero when the town has no visitor; never a field; T-28
   confirmed by batch TWN-6's TWN-6-5) and opens **E08's market decision** with **the town as its
   place** (ADR-0017 point 6); the adventure's checks (`markets.rs`, `cross-lane`) take the window
   as the game resolves it, the count read at each view and answer (no build can happen while the
   window is open). **`TownCommand::University { town, hero }`** (TWN-UNI-001; decision 210): the
   game checks the student, the town's visitor or its garrison hero and the actor's
   (`NotTownHero`; decision 207), leaves out the scenario's banned skills with `TWN-QUIRK-16`'s
   flag off (`strict-fixed`, T-23, confirmed by batch TWN-6's TWN-6-7: the original offers and
   teaches a banned school) and opens **E08's University decision** with the town as its place.
   **`TownCommand::BuyWarMachine { town, building }`** (TWN-SMT-001; decision 211, the original's
   own Buy-or-Cancel dialog): the game checks the window's hero (none `NoVisitor`), that it wears
   no such machine (`HasWarMachine`; the backpack never holds one, HER-ART-002), its room
   (defensive: `h3_heroes::check_change` with `AddArtifact`), then the treasury; then pays and
   gives the machine. **`TownCommand::BuildBoat { town }`** (TWN-SHY-001; not E08's `BuildBoat`,
   which names a map Shipyard): the adventure gives the town's spot and refuses it while an empty
   boat or a hero stands there (`BoatSpotTaken`; T-21 fixed by ADV-SHY-003's rule, decision 207,
   `TWN-QUIRK-19`; T-21 confirmed by batch TWN-6's TWN-6-6: the original launches a second boat
   onto the first); the game checks the treasury, then launches the boat and pays.
   **`TownCommand::Transform { town, army, slot }`** (TWN-TRF-001; decision 207): the army an
   `h3_towns::TownArmy`, `Visitor` (the visitor's) or `Garrison` (the upper bar's,
   `HireTarget::Garrison`'s meaning; no hero needed); the game checks the army (`NoVisitor`,
   `UnknownSlot`, `EmptySlot`) and that the stack's creature is not its own target
   (`NothingToTransform`: the Skeleton), then sets the slot's creature to the target, its count
   kept, free (T-26 fixed by construction, `TWN-QUIRK-23`). Each applies infallibly once checked
   (ADR-0017 point 10). Part H's new refusals: `NoMarket`, `NoWarMachine`, `HasWarMachine`,
   `BoatSpotTaken`, `NoUniversity`, `NotTownHero`, `NoTransformer`, `NothingToTransform`
   (`NoVisitor`, `UnknownBuilding`, `NotStanding`, `UnknownSlot` and `EmptySlot` reused; the
   treasury's refusal the game's, as every price's). No new contract type: the commands and their
   errors are `h3-towns`' types, which `h3-game` re-exports; the two places are E08's visits
   (`MarketVisit`, `UniversityVisit`, the adventure lane's) naming a town by `TownId`, and the new
   answer `MarketAnswer::SellArtifact` is E08's decision's (ADR-0017 point 6).
7. **The siege's facts and the editions by the packs** (decisions 14 and 20): `h3-towns` derives
   `Siege` (ADR-0023 point 2, E09's amendment) from the town and its faction's record; no game version exists.
   The faction's record holds the moat's kind (absent, harmless, damaging) and whether a siege
   there draws natural obstacles: the base pack gives them as RoE and AB play them (harmless; the
   Stronghold none, with natural obstacles), the SoD pack overrides them as SoD plays them
   (damaging; the Stronghold's moat, no natural obstacles); a Tower's kind decides its land
   mines' damage, as ADR-0023 point 2 has it.
   Conflux is the AB pack's faction, so the factions a random town or a Random choice may take
   are the ruleset's factions with a town record (eight with the base pack alone, nine with AB),
   within the map's allowed set for a player (ADR-0017 point 1).
8. **The hero pool is the game's** (decision 18; ADR-0017 point 1): `h3-towns` holds no hero, no
   pool and no tavern heroes. A town's **Tavern** is a building of role `tavern` whose hire the game
   composes (point 6) from the pool, the players' two tavern heroes each (the survey's offers), the
   store of heroes and the adventure's placement; the draw's weights are the part's (point 5).
   **The Thieves' Guild** (increment 4's information; TWN-TAV-007 to -009; E09 decisions 6, 99) is a
   projection the game composes (ADR-0017 point 2), as a town's view: its level is the number of the
   viewer's towns where `has_tavern` holds (a standing building of role `tavern`, the Castle's
   Brotherhood of the Sword among them; `player_tavern_count` 0x4CCE30), counted to 5; `h3-towns`
   gives that predicate and each town's garrison, nothing else. The rows' values and the per-player
   parts read what the game holds (the treasuries, the day's gold, the store of heroes) and the
   adventure's owners, heroes and armies; no town state moves. Batch TWN-4's TWN-4-7 (E09 T34) saw
   the level so: 2, 4, 6, 8 and 9 rows at one to five towns with a Tavern, a Castle with its Tavern
   and the Brotherhood counted once.
9. **Versions owned by the town lane:** `h3-towns`' state hash and the town part's hash (v1 when
   the crate is created). Its changes bump the adventure lane's versions it touches (the game
   state hash, the import, the adventure replay format) in `cross-lane` pull requests.
   **Increment 2** bumps the town ruleset part (v4: the requirements, the costs, the role
   `shipyard`) and the town state hash (v2: the day's mark); the adventure lane's versions it
   touches, each `cross-lane`: the adventure ruleset part (the kind `Town`'s boat-spot offsets),
   the game state hash (it folds the store's hash in), the adventure replay format (the town
   command); the env's action encoding is the AI lane's (ADR-0017 point 8).
   **Increment 3's growth** bumps the town state hash (v3: the counts, the Portal) and the town part
   (v5: the roles, `roe`, the parameters); `cross-lane`: the game state hash (the week's kind, the
   towns' stream, the store), the protocol (the week's kind, the owner's town view), the adventure
   replays re-recorded; `shared/`: the kernel's stream id, `h3-content`'s two hero effects and two
   quirk flags (pack format v52, core ruleset hash v53; E09 T24). **Increment 3's recruitment and
   rows** move no version of the town's own (the counts are growth's v3; no new role or parameter);
   `cross-lane`: the game state hash (the hire dialog's town sources and target), the adventure
   state hash (a town's garrison hero, ADR-0016 point 3), the adventure replay format (`town
   recruit`, the hero moves), the env's action encoding (ADR-0017 point 8) and the protocol.
   **Increment 4** bumps the town ruleset part (v7: the tavern weights, the hire's price); the town
   state hash does not move (nothing of the tavern is the town's). The adventure lane's versions it
   moves (`cross-lane`): the game state hash (the pool's statuses and records, the players' factions
   and tavern heroes), the adventure ruleset part (the kinds `Tavern` and `DenOfThieves`; the
   adventure state hash does not move, the kinds hold no state), the map import (classes 95 and 97,
   the map-wide hero entries in the setup), the adventure replay format (the hire, the tavern
   decision's answers, the dismissal); the protocol (the server lane's: the hire, the dismissal, the
   tavern decision, the tavern heroes' view and its events, a hired hero's appearance, the Thieves'
   Guild's request and a Den's view) and the env's action encoding (the AI lane's: the town's hire,
   then the tavern decision's answers); each `cross-lane`. The Den of Thieves' kind moves the
   adventure ruleset part and the map import with the Tavern's, in one task. `shared/` (E09 T35b):
   the kernel's `StreamId::TAVERNS` and the quirk flags of `TWN-QUIRK-12` and `-13` (no version
   moves: the profiles' ruleset hashes do).
   **Increment 5** bumps the town ruleset part (v9: the guild weights, the roles `guild`,
   `library`, `aurora` and `mana_vortex`, the slots, the book's price, the Vortex's factor) and the
   town state hash (v4: the guild's slots, the Vortex's week flag, the Aurora set). The adventure
   lane's versions it moves (`cross-lane`): the map import (the towns' required and banned spells
   into the setup), the game state hash (it folds the store's in), the adventure replay format
   (`town open`, `town book`; the growth replays re-recorded: the draw moves the towns' stream,
   decision 179); the protocol (the server lane's: the two commands and their refusals, the owner's
   guild and week flag in the town view) and the env's action encoding (the AI lane's: the
   opening, then the book), each `cross-lane`; `h3-harvest`'s spell columns (the battle lane's,
   `cross-lane`). `shared/` (E09 T45): the quirk flag of `TWN-QUIRK-14` (no version moves: the
   profiles' ruleset hashes do); no new stream (decision 179). The adventure state hash and part do
   not move (the outcome's town entered, ADR-0016 point 4, is not hashed).
   **Increment 6** bumps the town ruleset part (v10: the six building fields, the parameters; T56);
   the town state hash (v4), the map import and the adventure ruleset part do not move (nothing new
   in a town's record, the map or the adventure's tables). The adventure lane's versions it moves
   (`cross-lane`): the adventure state hash (the pool's marks, the merchants' stock; T57), the game
   state hash (the adventure's folded in, T57; the decisions' places, T58a and T59b) and the
   adventure replay format (`town market`, `town machine`, `town boat`, `town university`, `town
   transform`, `answer market sell-artifact`; every golden adventure replay re-recorded at T57);
   the protocol (the server lane's: the places, `SellArtifact`, the five commands and their
   refusals) and the env's action encoding (the AI lane's: 42 entries, ADR-0017 point 8), each
   `cross-lane`; `h3-harvest`'s six fields and parameters (the battle lane's, `cross-lane`).
   `shared/` (E09 T55b): the quirk flags of `TWN-QUIRK-15` and `-16`, confirmed by batch TWN-6's
   TWN-6-5 and TWN-6-7 (no version moves: the profiles' ruleset hashes do), and the combinations'
   costs (the core ruleset hash); no new stream (decision 214). Each number is the next at merge,
   in merge order with the adventure lane's open work (E08 T142b, #1162): the second to merge
   rebases and bumps again.

## Consequences

- Two `shared/` pull requests after acceptance (E09 T4, cut in two at its start, the user): T4a
  this ADR and the four amendments into `docs/adr/`, `docs/03`'s terms; T4b the two crate
  skeletons, `arch.toml`'s edges (`h3-game` → `h3-towns`; `h3-towns` → kernel, content,
  contracts, heroes; `h3-pack-towns` → `h3-towns`, `h3-content`, `h3-pack`, as
  `h3-pack-adventure`), `lanes.toml` (the adapter and `packs/*/towns.toml` to the town lane),
  `TownId`, the versions rows; `docs/02` §3 follows.
- `cross-lane` after it: `GameState` and its hash, `GameRuleset`, the adventure replay's header
  (the part's hash), each adventure golden replay re-recorded (skill `homm3-rerecord-replays`);
  the server's manifest (the server lane's inbox) and the env (the AI lane's) as their notes.
- A fork costs one more reference count (the XL fork benchmark, `bench-check`).
- A town is never half in one context: a capture writes the owner in the adventure and the
  Capitol's limit in the store in one infallible application (ADR-0017 point 11's rule).
- Increment 2: one `shared/` pull request after acceptance and batch TWN-2 (E09 T15, design
  §35's round 3): its amendments into this ADR, ADR-0016 and ADR-0017, `docs/03-domain.md`'s
  terms (the day's build, the hall's offer, a town event), `docs/06-security.md`'s town section.
  `cross-lane` after it: the harvest (`h3-harvest`, the battle lane's), the kind `Town`'s offsets
  and the town's boat spot (the adventure lane's static map), `GameCommand` and the replay
  format, the turn start's and the day's calls, the env (the AI lane's); each adventure golden
  replay re-recorded (skill `homm3-rerecord-replays`). A town's day's mark makes a fork's first
  build in a town copy that town (the store's rule); the day's start writes only the towns built
  in the day before.
- Increment 3: each week writes every town with a dwelling (a fork's first week copies most towns);
  the day's start computes each town's growth twice at a month's first day (the week and the month),
  measured against `day-start-builds-36`.
- Increment 3's recruitment reads the counts from the store at each view and purchase (no copy in
  the dialog); a purchase writes one town and one army.
- Increment 4: two `shared/` pull requests after batch TWN-4 (E09 T35, cut in two at its start,
  decision 125): T35a this ADR's and the three amendments' text (ADR-0013, ADR-0016, ADR-0017),
  `docs/03-domain.md`'s terms, `docs/06-security.md`'s section; T35b the taverns' stream and the two
  quirk flags. `cross-lane` after them: the harvest of `HCTRAITS.TXT`'s tavern columns
  (`h3-harvest`, the battle lane's), the game's pool and slots (the game state hash), the
  adventure's kinds and placement, the import, the replay format, the protocol and the env; the
  golden replays re-recorded. The town part costs nothing a day: a fill reads its weights, a hire
  its price and `has_tavern`.
- Increment 5: one `shared/` pull request after batch TWN-5 (E09 T45, kept whole): this ADR's and
  the three amendments' text (ADR-0013, ADR-0016, ADR-0017), `docs/03-domain.md`'s terms,
  `docs/06-security.md`'s guild lines and the quirk flag of `TWN-QUIRK-14` (the profiles' ruleset
  hashes, the golden replays' headers). `cross-lane` after it: the harvest of `SPTRAITS.TXT`'s
  town columns (`h3-harvest`, the battle lane's), the import's town spell sets, the game state
  hash, the replays (the guild's draw moves the towns' stream: the week replays' outcomes move),
  the outcome's town entered, the replay format, the protocol and the env. Each town holds twenty
  slots, a flag and a static set; the night's refill reads one predicate per town a day, and
  nothing in play draws a guild.
- Increment 6: two `shared/` pull requests after batch TWN-6 (E09 T55a and T55b): T55a this ADR's
  and the two amendments' text (ADR-0016, ADR-0017), `docs/03-domain.md`'s terms,
  `docs/06-security.md`'s section; T55b the quirk flags of `TWN-QUIRK-15` and `-16` and the
  combinations' costs in the core (the core ruleset hash). `cross-lane` after them: the six
  building fields and the parameters (`h3-harvest`, the battle lane's), the stock and the pool's
  marks in the adventure state (its hash; every golden adventure replay re-recorded at T57), the
  towns' stream's draws from day 29, the decisions' places, the replay format, the protocol and
  the env's 42 entries. A town's record gains nothing; the stock is redrawn once a month.

## Alternatives considered

- The town records inside the adventure state, `h3-towns` a library of rules over types passed
  in (TWN-Q3): the adventure lane's structs would change with every town feature.
- The same store with an id local to `h3-game` (TWN-Q3): a mapping in every projection and seam,
  as ADR-0015 found for local ids.
- The pool in `h3-towns` (TWN-Q3): the random hero is drawn with no town involved, and the store
  of heroes is the game's.
- The town data as a section of `adventure.toml`, or in the core content (TWN-Q3): the adventure
  lane's adapter, or a `shared/` pull request, for every town content change.
- The moat in the battle part's siege record, or an edition derived from the packs' names
  (TWN-Q3): each change `cross-lane`; a rule keyed by names, against ADR-0006.
- A module inside `h3-adventure` split out later; `h3-towns` owning the town's armies (TWN-Q1,
  decision 2).
- Increment 2 (`TWN-Q9`): a game command `GameCommand::Build { town, building }`, the rules as
  `h3-towns` functions, each later town action a variant of its own (the game's command type
  grows with every town service, and the town context's validation has no command of its own to
  name); a pending decision opened by entering an own town, its answers building (the original
  builds from the town list without a visit); the Shipyard refused until increment 4, or
  buildable without water until then (the Castle's Lighthouse needs the Shipyard; a Shipyard
  without water is a deviation the start already removes in the original).
- Increment 2: the requirements stored closed in the pack (the closure written by the harvester:
  two sources of one fact; the executable's lists are direct); the day's mark as the day of the
  town's last build (no write at the day's start: two states that play alike would hash apart;
  the original's flag is kept).
- Increment 4 (`TWN-Q17`): the tavern weights in the core's hero classes, a weight per faction (a
  `shared/` change, and an override for RoE's missing Conflux column); a pending decision for the
  town's hire too (the town screen's tavern needs no answer, as the town's rows).
- Increment 5 (`TWN-Q32`, `TWN-Q33`): the guild weights in each spell record of the battle lane's
  pack (nine values by faction key; decision 173); the guilds from a new named stream (no existing
  draw moving; the user chose the towns' stream, decision 179) or the setup stream; the opening
  listed always (as `Revisit`) or implicit in every town command (decision 175); the spell book as
  a decision at the opening (the original's question) or bought when affordable (the computer's
  way; decision 176); the Aurora Borealis left to increment 7 (decision 174).
- Increment 6 (`TWN-Q37`): the month's draw of the merchants' stock over a named stream of its own
  (the user chose the towns' stream, decision 214); the env without the artifact sale's 22 entries
  (decision 216 kept the 42).

## Revision history

- 2026-11-02: Drafted (E09 T2a); accepted as written at T2b's checkpoint (the user, TWN-Q4,
  decision 21).
- 2026-11-03 (E09 T4a `shared/`): moved here as ADR-0028 (the next free number) with the four
  amendments; batch TWN-1 (T3c, survey §15) confirmed TD-3, TD-5, TD-7 and the siege's
  aftermath, so no point changed; its unpredicted building 23 changes no point (decision 25:
  re-read in T5b). The Consequences' `shared/` work split in two (T4a, T4b).
- 2026-11-07 (E09 T15 `shared/`; the user at `TWN-Q7` to `TWN-Q10`, decisions 37, 41–48): points
  2, 5, 6 and 9 amended for increment 2, construction with the town events (with ADR-0016 points
  1, 4, 6 and ADR-0017 points 1 to 4 and 6 to 10): the day's mark and the rules buildable now,
  the build, an event's buildings and the mark's life; the part's requirements, costs and role
  `shipyard`; the town command `TownCommand::Build`; the versions. Drafted in E09 T13 (design
  note `E09-design.md` Part D, §23 to §36); batch TWN-2 (T14c, survey `E09-construction.md` §13)
  confirmed decisions 41 (TD-9, the executable's list) and 42 (TD-11, the owner builds), so no
  point changed.
- 2026-11-12 (E09 T24 `shared/`; `TWN-Q12` to `TWN-Q14`, decisions 65 to 83): points 2, 5, 6
  and 9 amended for increment 3, growth, recruitment and the army rows (with ADR-0013 point 16.4,
  ADR-0016 points 1, 3, 4, 6, ADR-0017 points 1 to 4, 6, 8 to 11 and ADR-0023 points 1 and 3):
  the available creatures and the Portal, the growth rules, the recruitment's source, stock and
  purchase; the roles `horde`, `portal`, `week_of`, `roe` and the week's parameters; the inputs
  and `TownCommand::Recruit`; the versions; the Consequences. Drafted in E09 T22a and T22b
  (design note `E09-design.md` Part E); batch TWN-3 (T23, survey `E09-growth.md` §17) confirmed
  every row, so no point changed.
- 2026-11-16 (E09 T35a `shared/`; `TWN-Q15` to `TWN-Q19`, decisions 99 to 127): points 5, 6, 8 and 9
  amended for increment 4, the Tavern (with ADR-0013 points 14.5 and 16.5, ADR-0016 points 3, 4, 6
  and ADR-0017 points 1 to 4, 6, 8 to 11): the tavern weights and the hire's price;
  `TownCommand::Hire`; the pool's tavern heroes the game's and the Thieves' Guild's level by the
  taverns; the versions (the adventure state hash unmoved, as the design note's §77); the header,
  the Consequences and Alternatives. Drafted in E09 T33a and T33b (design note `E09-design.md` Part
  F); batch TWN-4 (T34, survey `E09-tavern.md` §14) confirmed every row, so no point of this ADR
  changed; the two points the reading missed were decided at T35's start (decisions 126, 127;
  ADR-0017 point 3).
- 2026-11-20 (E09 T45 `shared/`; E09 decisions 162 to 181; `TWN-Q31` to `TWN-Q33`): points 2, 5, 6
  and 9 amended for increment 5, the mage guild (with ADR-0013 points 15.5 and 16.6, ADR-0016
  point 4, ADR-0017 points 1 to 4 and 8 to 10): the guild's slots, the Vortex's flag and the
  Aurora set; the guild weights, the four roles, the slots, the book's price and the Vortex's
  factor; `Open` and `BuySpellBook`; the versions; the Consequences and Alternatives. Drafted in
  E09 T43 (design note `E09-design.md` Part G §82 to §100), accepted at `TWN-Q33`; batch TWN-5's
  outcome (T44, survey `E09-guild.md` §21: every row as read, T-18 confirmed) folded in.
- 2026-11-25 (E09 T55a `shared/`; E09 decisions 197 to 217; `TWN-Q35` to `TWN-Q37`): points 2, 5,
  6 and 9 amended for increment 6, the town's trades (with ADR-0016 points 1, 4 and 6, ADR-0017
  points 1, 3, 4, 6 and 8 to 10): nothing new in a town's record; the count, the window's offer,
  the machine, the boat, the University and the Transformer's target; the six building fields,
  the parameters and the core's combination costs; `Market`, `University`, `BuyWarMachine`,
  `BuildBoat` and `Transform`; the versions; the Consequences and Alternatives. Drafted in E09 T53
  (design note `E09-design.md` Part H §101 to §120), accepted at `TWN-Q37`; batch TWN-6's outcome
  (T54, survey `E09-trades.md` §14: every row as read, T-19, T-20, T-21, T-23 and T-28 confirmed)
  folded in.
