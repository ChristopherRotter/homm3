# E04a T1 source survey: hero classes, heroes and specialties

Date: 2026-10-03. Target: RoE + AB + SoD (GOG Complete 4.0). Research notes for E04a T3 (content), T4 (`h3-heroes`), T5 and T10; the facts become rules with catalog ids there.

Sources: `original-data` = the copies in `original-data/` (text tables and `Heroes3.exe`; offsets and layouts in [`docs/formats/installation.md`](../../formats/installation.md) §6–§7); `vcmi` = VCMI 1.7.5 (commit `6901839`), URLs `https://github.com/vcmi/vcmi/blob/1.7.5/<path>`; `wiki` = https://heroes.thelazy.net, retrieved 2026-10-03 (raw wikitext; in `{{swh|A|B}}`, A is HotA and B is SoD).

## 1. Tables

| Table | Rows | Contents | AB vs SoD copy |
|---|---|---|---|
| `HCTRAITS.TXT` | 2 header rows + 18 classes (then blank rows) | 51 columns: name, aggression (AI), 4 initial primary skills, primary-skill gain probabilities for levels 2–9 (4) and 10+ (4), 28 secondary-skill probabilities, 9 tavern selection weights (one per town) | identical |
| `HOTRAITS.TXT` | 2 header rows + 156 heroes | 10 columns: name, then 3 × (low, high, creature name "reference only") for the starting army | identical |
| `HeroSpec.txt` | 2 header rows + 156 | 3 columns: specialty short name, long name, right-click text (numbers only in the text) | identical |
| `Heroes3.exe` hero table | 156 records | sex, class, two starting secondary skills, spell book flag, starting spell, the three starting creature types, portraits, expansion flags (§2) | — |
| `Heroes3.exe` specialty table | 156 records | specialty kind and parameters (§3) | — |

Row order is the hero id in all of them and equals VCMI's `index` (`config/heroes/*.json`, all 156 checked by name).

## 2. Membership by expansion (original data)

The executable's hero record has four flag bytes at +0x38 (inference from the values; the meaning of byte 1 in particular):

| Flags (bytes 0–2) | Ids | Count | Reading |
|---|---|---:|---|
| 1, 1, 0 | 0–127 except 4 | 127 | RoE heroes, still available after the expansion |
| 1, 0, 0 | 4 (Lord Haart, Knight) | 1 | RoE only: in AB/SoD he is replaced by the campaign hero 150 (Lord Haart, Death Knight) |
| 0, 1, 0 | 128–144 | 17 | AB: the 16 Conflux heroes and Sir Mullich (144) |
| 0, 1, 1 | 145–155 | 11 | campaign heroes (Adrienne, Catherine, Dracon, Gelu, Kilgor, Lord Haart (DK), Mutare, Roland, Mutare Drake, Boragus, Xeron) |

So `base` (RoE) has heroes 0–127; the expansions add 128–155 and retire hero 4 from the hero pool. The Conflux heroes are AB; whether each campaign hero (144–155) came with AB or SoD is not in the data, so T3 sources it per hero. The 18 classes: 8 town pairs (RoE) + Planeswalker and Elementalist (AB, Conflux). 93 of the 156 heroes start with a spell book.

## 3. Specialty kinds (original data: `Heroes3.exe` specialty table)

Record: i32 kind, i32 id, i32 attack, i32 defense, i32 damage, i32 upgrade 1, i32 upgrade 2, then three text pointers (zero in the file, filled from `HeroSpec.txt`). VCMI's `config/heroes/*.json` agrees on the kind of every row (counts checked by script: 35 `secondary` plus Sylvia and Voy as explicit bonuses, 49 + 6 `creature`); the fixed-stat parameters were compared row by row.

| Kind | Count | Parameters | Scope | Formula (sources) |
|---:|---:|---|---|---|
| 0 secondary skill | 37 | skill id | E04a for Archery (1), Offense (2), Armorer (3), First Aid (3); E04b for Sorcery (5), Resistance (1), Intelligence (3), Mysticism (4); outcome for Necromancy (2), Eagle Eye (7); adventure for the rest | the skill's value × (1 + 0.05 × hero level). `original-data`: each skill getter in the exe multiplies its table value by `1 + 0.05 × level` when the hero's specialty is kind 0 with that skill (e.g. First Aid at VA 0x4E4920; constant 0.05 at 0x63EAE4). `wiki` Hero specialty: "A * (1 + (0.05 * B))". `vcmi` `specialtySecondarySkillGrowth = 5` (`config/gameConfig.json` L539) |
| 1 creature (scaling) | 55 | creature id (49 creature heroes, 6 Ballista heroes) | E04a | attack and defense of that creature **and its upgrade** + 5 % of the creature's base value per *n* hero levels (n = creature level; 5 for the Ballista), rounded up; +1 speed (not for the Ballista). `wiki` Hero specialty (SoD branch: "{{swh\|20%\|5%}} every n levels"; Ballista "{{swh\|30%\|5%}} for every 5 levels (rounded up)"); `vcmi` `CHeroHandler::createCreatureSpecialty` (`lib/entities/hero/CHeroHandler.cpp` L142–181: `val = 5 × (level / creatureLevel)` as percent of the creature's own value, `BonusList.cpp` L103–107 rounds up) |
| 2 resource | 13 | resource id | adventure | — |
| 3 spell | 34 | spell id | E04b | SoD: effect × (1 + 0.03 × level / target creature level) (`wiki` Hero specialty, SoD branch) |
| 4 fixed creature stats | 11 | creature id, attack, defense, damage | E04a | flat bonus to the creature and its upgrade: Pasis/Monere psychic elementals +3/+3; Thunar/Erdamon earth elementals +2/+1/+5 damage; Ignissa/Fiur fire +1/+2/+2; Lacus/Kalt water +2 attack; Kilgor Behemoths +5/+5/+10; Lord Haart (DK) Black Knights +5/+5/+10; Xeron Devils +4/+2 |
| 5 speed | 1 | +2 | E04a | Sir Mullich: +2 speed to all creatures |
| 6 creature upgrade | 2 | base, upgrade 1, upgrade 2 | adventure | Dracon (Enchanters), Gelu (Sharpshooters): upgrade on the map; no battle effect in the table |
| 7 dragons | 2 | attack, defense | E04a | Mutare and Mutare Drake: dragons +5/+5 (selector: the exe's dragon flag) |
| −1 none | 1 | — | — | Adrienne (Fire Magic, a starting skill rather than a bonus) |

Battle-relevant in E04a: 37 skill rows of which 9 are battle skills, 55 + 11 + 1 + 2 creature-stat rows. The skill getters and the specialty kinds give the derivation everything it needs; no formula has to come from text.

## 4. Precision

The specialty terms are not whole percents: Orrin at level 7 with Expert Archery gets 50 % × 0.35 = 17.5 %; a Basic Armorer specialist gains 0.25 % per level. Every product of a skill table value (5, 10, 15, 20, 25, 30, 40, 50, 60, 100 %) with `1 + 0.05 × level` is a whole number of **basis points** (1/100 %). The original computes these in single precision (`fmul` on the table floats), which belongs to Q23.

## 5. Primary skills

- A hero's attack is added to the attack of each of its creatures, defense likewise (`wiki` Attack, Defense; `vcmi` hero bonuses propagate to the army). The battle needs only the effective totals.
- `wiki` Primary skill: "The upper limit is 99. At 129 the skill value is reset to 0." Progression is out of scope; the derivation caps at the ruleset's limit and the overflow becomes a quirk entry (S-08 in [E04-quirks.md](E04-quirks.md)).
- Minimums: attack and defense ≥ 0, power and knowledge ≥ 1 (`vcmi` `config/gameConfig.json` L522 `minimalPrimarySkills`), needed because artifacts can be negative (Titan's Gladius −3 defense).
- Artifact primary-skill bonuses are a static table in the exe (installation.md §7.3); VCMI agrees on every artifact.

## 6. Disputes and open points

| # | What | Sources | Impact | Experiment (SoD) |
|---|---|---|---|---|
| H-1 | Catherine and Roland (Swordsmen specialty) | `wiki` Swordsman and Crusader: both "bugged to only affect Crusaders" (fixed in HotA). `original-data`: the specialty table names creature 6 (Swordsman) for Catherine like Sorsha, but 7 (Crusader) for Roland. `vcmi`: both Swordsman (with upgrade) | low (campaign heroes) | Catherine and Roland each with Swordsmen; compare attack in the stack window with Sorsha's at equal level |
| H-2 | Xeron's +1 speed | `wiki` and `vcmi`: +1 speed. `original-data`: the fixed-stat record has no speed field (it may be hard-coded) | low | Xeron with Devils: read speed |
| H-3 | Creature specialty rounding | `wiki` "rounded up" (Ballista text) and `vcmi` round up the percentage of the base value; the original computes in float | low | folded into the Q23 batch (read attack/defense of a level-1 specialty stack at a few hero levels) |
