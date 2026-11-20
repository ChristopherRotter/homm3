# Draft: ADR-0028 amended for E09 increment 5 (the mage guild)

- Status: **Draft**, written by E09 T43 (2026-11-20), for the user's acceptance at T43's
  checkpoint (`TWN-Q33`) before anything is built; then moved into
  `docs/adr/0028-town-context.md` by the increment's `shared/` task with batch TWN-5's verdicts
  (as increments 2's T15, 3's T24 and 4's T35a).
- Decisions: the epic's 162 (content), 163 (the round), 165 to 171 (the survey's cut and its
  checkpoint `TWN-Q31`: TD-10 and TD-25 the original, T-18 `fix-optional`, TD-27 the original's
  triggers with an opening command), 172 (T43 whole), 173 to 176 (`TWN-Q32`: the weights in the
  town records, the Aurora in this increment, `TownCommand::Open` listed where it changes
  something, `TownCommand::BuySpellBook`), 177 (T43's agent's calls).
- Working detail: the survey `docs/rules/surveys/E09-guild.md` (§2 to §6 what the guild holds,
  §11 to §15 what a hero gets), the design note `docs/epics/E09-design.md` Part G (§82 to §100).
  Each block below says where it goes in the ADR.

## Point 2, after increment 3's recruitment paragraph ("… a purchase joins the garrison hero's army).")

**Increment 5** (the mage guild; E09 decisions 162, 167 to 176) adds to the town's record **its
guild's slots**: per spell level 1 to 5 the part's slots of the level plus one, the **spare**
(6, 5, 4, 3, 2: the original's `guild_spell_slots` 0x6782A4 plus the slot the Tower's Library
shows), each a spell of the core or empty, **drawn once when the game is built** for every town,
whatever it has built (the original's `town_setup_spells` 0x5BE600; survey `E09-guild.md` §3), and
never drawn again (a capture, a build, a week keep them); **the Mana Vortex's week flag** (the
original's +0x33: set when the game is built and at every week's first day, cleared when a hero
takes the Vortex; every town holds it, only a standing building of role `mana_vortex` reads it);
and, static, **the town's Aurora set** (the spells a building of role `aurora` teaches there,
fixed at the build: every hero spell the town's map record does not ban, less those granted by name
only, less the scenario's bans when `TWN-QUIRK-14`'s flag is off). Its rules, each a pure function
of a town, the part, the core and plain inputs the game gives:
- **The draw** (TWN-GLD-002; survey §3): with draws from the stream the game passes (the guilds',
  ADR-0017 point 4) and the town's **required** and **banned** spells and the scenario's bans (the
  setup's, TWN-SET-012, GAM-SET-003), each level's slots in order: the first required spell of the
  level in spell order not excluded, else one drawn by the faction's guild weights (point 5) among
  the level's spells not excluded, else the slot empty; **excluded** a spell the town bans, the
  scenario bans, one granted by name only (Titan's Lightning Bolt, the original's flag 0x2000) and
  every one taken. A required spell the town bans is never taken (TD-25, E09 decision 168).
- **What the guild shows** (TWN-GLD-003; §3.3, §4): the **guild level**, the highest level of a
  standing building of role `guild` (0 none; the faction's highest is its record's buildings);
  each level up to it shows its first slots, the part's count plus one with a building of role
  `library` ever built, fewer where the draw left the end of the level empty; a level above it
  shows none. A level or the Library built later shows at once.
- **What a hero learns there** (TWN-GLD-004; §5, §11): with a building of role `guild` ever built
  and a worn spell book (the details' test, `h3-heroes`), in a town with a building of role
  `aurora` ever built every spell of the Aurora set whose level is at most the hero's learnable
  level (HER-SPL-005), the guild level notwithstanding; else every spell the guild shows at a level
  up to the learnable level; each the hero does not know, in level then slot order (the Aurora's in
  spell order), returned as `h3_heroes::HeroChange::LearnSpell`s (point 6). No owner test, no class.
- **The Mana Vortex** (TWN-GLD-009; §15): with a standing building of role `mana_vortex` and the
  week flag set, a hero whose spell points are below the part's Vortex factor (2) times its maximum
  (HER-MAN-001) gets that many (`HeroChange::SetSpellPoints`) and the flag is cleared; a hero at or
  above it leaves the flag set. No spell book needed.
- **The night's refill's town** (TWN-GLD-008; §14): whether a building of role `guild` was ever
  built (the game raises the town's heroes' points).
- **The spell book's check** (TWN-GLD-007; §6, §13): a building of role `guild` standing, else a
  typed refusal; the price the part's (point 5).

## Point 5, after increment 4's paragraph

**Increment 5** (the mage guild; E09 decisions 162, 173) gives each town record its **guild
weights**: per spell of the core (by key) the weight of that spell in the record's guild draw
(`SPTRAITS.TXT`'s town column of the faction, the original's spell record +0x44 + 4 × faction; each
level's weights summing to 100 at every level the faction's guild reaches, 0 at the others: survey
`E09-guild.md` §2.2). A spell the record does not list weighs 0; the compiler refuses a spell the
core lacks or holds as a creature ability, and a spell listed twice. An edition without Conflux has
no Conflux record (RoE's missing column needs no override; RoE's own values are E11's). Four new
**roles**: **`guild`** (a level, 1 to 5: the Mage Guilds, read standing for the guild level and
ever built for a guild's presence), **`library`** (at most one building of a record: the Tower's
Library, building 22, one more slot shown at every level; read ever built), **`aurora`** (at most
one: the Conflux's Aurora Borealis, building 26, every spell of the town's Aurora set taught; read
ever built) and **`mana_vortex`** (at most one: the Dungeon's Mana Vortex, building 21; read
standing). The parameters gain **the guild's slots** by level (5, 4, 3, 2, 1, `guild_spell_slots`
0x6782A4; the spare is the rule's one more), **the spell book's price** (resources of the core:
500 gold, `mage_guild_open` 0x5CE560; the same for every purchase and edition) and **the Vortex's
factor** (2, `town_visit_buildings` 0x5BD8E0), each tier-2 data of the reading. The spell book
itself is the core's (the first artifact with the `spell_book` effect, as HER-ART-007's). The
content: the weights harvested from both LODs' `SPTRAITS.TXT` (the base pack's eight records, the
AB pack's Conflux; the two copies' columns are equal; `h3-harvest`, `cross-lane`), the roles the
harvest's gap-fill (the Mage Guilds by their keys' levels, the three specials by faction), the
parameters tier-2 data.

## Point 6, after increment 4's hire

**Increment 5** (the mage guild; E09 decisions 170, 175, 176) adds the town command's fifth and
sixth variants. **`TownCommand::Open { town }`** (TWN-GLD-006; TD-27's opening, E09 decision 170:
the original's town screen opened from the town list, `town_screen_open` 0x5BE210, and its guild's
window, both free and at will): `h3-towns` names nothing but the town's part (the Vortex's and the
learning's predicates of point 2); the game checks the turn, the owner and that no decision is open,
then gives the Vortex (the garrison hero, then the visitor) and teaches (the visitor, then the
garrison hero), in one infallible application; nothing else, so an opening that changes nothing is
legal and changes nothing. **`TownCommand::BuySpellBook { town }`** (TWN-GLD-007; the guild's
offer, `mage_guild_open` 0x5CE560): `h3-towns` checks the town's part (a standing guild) and names
the price (the part's), each refusal typed (`TownCommandError`); the game checks the turn, the
owner, that no decision is open, the hero (the visitor, else the garrison hero; none a refusal) and
that it wears no spell book, then the treasury, then pays, gives the book and teaches the town's
heroes, in one infallible application (ADR-0017 point 10). Learning everywhere crosses as the
heroes' changes (point 2's set, read from the details by reference, applied by the game as
`HeroChange::LearnSpell`, as a visit of the adventure), the Vortex as one
`HeroChange::SetSpellPoints` and the store's write of the cleared flag. No new contract type.

## Point 9, after increment 4's sentence

**Increment 5** bumps the town ruleset part (v9: the guild weights, the roles `guild`, `library`,
`aurora` and `mana_vortex`, the slots, the book's price, the Vortex's factor) and the town state
hash (v4: the guild's slots, the Vortex's week flag, the Aurora set). The adventure lane's versions
it moves (`cross-lane`): the map import (the towns' required and banned spells into the setup), the
game state hash (it folds the store's in; the guilds' stream), the adventure replay format (`town
open`, `town book`); the protocol (the server lane's: the two commands and their refusals, the
owner's guild and week flag in the town view) and the env's action encoding (the AI lane's: the
opening, then the book), each `cross-lane`; `h3-harvest`'s spell columns (the battle lane's,
`cross-lane`). `shared/`: the kernel's `StreamId::GUILDS` and the quirk flag of `TWN-QUIRK-14` (no
version moves: the profiles' ruleset hashes do). The adventure state hash and part do not move
(the outcome's town entered, ADR-0016 point 4, is not hashed).

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 5's `shared/` task; decisions 162 to 177 and T43's checkpoint):
  points 2, 5, 6, 9 (the guild's slots, the Vortex's flag and the Aurora set; the guild weights,
  the four roles, the slots, the book's price and the Vortex's factor; `Open` and `BuySpellBook`;
  the versions).
