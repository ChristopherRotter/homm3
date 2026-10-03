# E04a T1 source survey: the 144 artifacts

Status: research notes for E04a T1 (2026-10-03). Target ruleset: SoD (GOG Complete 4.0); RoE/AB membership noted.

## Sources and conventions

- **original-data**: `H3bitmap/artraits.txt` (SoD copy), `H3ab_bmp/ArTraits.txt` (AB-era copy), `H3bitmap/ArtSlots.txt`, `H3ab_bmp/Artslots.txt` from `original-data/extracted/`. Parsed with Python `csv` (TAB delimiter, `"` quote char, cp1252). Description texts were read to check numbers but are not reproduced.
- **vcmi**: VCMI 1.7.5 (a local clone, tag 1.7.5, commit 6901839). Cited as `path Lx–y`, URL `https://github.com/vcmi/vcmi/blob/1.7.5/<path>#L<x>-L<y>`. Example: `config/artifacts.json L41–53` = https://github.com/vcmi/vcmi/blob/1.7.5/config/artifacts.json#L41-L53. The full table cites `config/artifacts.json` by line range only.
- **wiki**: https://heroes.thelazy.net, retrieved 2026-10-03, with WebFetch. `curl` is blocked by the proxy, so pages came through WebFetch's summarizer. Where it mattered I fetched the raw wikitext (`?action=raw`) and asked for verbatim text. Facts marked *(raw)* come from wikitext. Pages used:
  - List of artifacts: https://heroes.thelazy.net/index.php/List_of_artifacts. Its rows live in `Template:Artifact table - Weapon / Shield / Torso / Helmet / Feet / Cape / Necklace / Ring / Miscellaneous`, all *(raw)*, e.g. https://heroes.thelazy.net/index.php?title=Template:Artifact_table_-_Weapon&action=raw.
  - Artifact: https://heroes.thelazy.net/index.php/Artifact.
  - Combination artifact: https://heroes.thelazy.net/index.php/Combination_artifact.
  - Single artifact pages: `https://heroes.thelazy.net/index.php/<Title>`, for Angelic_Alliance, Armor_of_the_Damned, Power_of_the_Dragon_Father, Cloak_of_the_Undead_King, Elixir_of_Life, Bow_of_the_Sharpshooter, Armageddon%27s_Blade_(artifact), Shackles_of_War, Spirit_of_Oppression, Hourglass_of_the_Evil_Hour, Badge_of_Courage, Bow_of_Elven_Cherrywood, Still_Eye_of_the_Dragon, Quiet_Eye_of_the_Dragon, Vial_of_Dragon_Blood, Golden_Bow, Ring_of_Vitality, Necklace_of_Swiftness, Amulet_of_the_Undertaker, Statesman%27s_Medal, Orb_of_Vulnerability, Orb_of_Inhibition, Recanter%27s_Cloak, Pendant_of_Negativity, Spell_Book, Grail.
  - Rule pages: Regeneration, Archery, Morale, Necromancy, Dragon, War_Machine, Combat (Retreat and Surrender sections).
- **Hierarchy**: original data > observation > reverse engineering > VCMI > wiki. HotA behaviour is ignored except for "HotA changed this" notes.

## 1. Layout of `artraits.txt` / `ArtSlots.txt`

**`ArtSlots.txt`** (original-data): one slot name per line. SoD has 19 lines: Head, Shoulders, Neck, Right Hand, Left Hand, Torso, Right Ring, Left Ring, Feet, Misc 1–4, War Machine 1–4, Spell Book, Misc 5. The AB copy has 18 lines, the same list without Misc 5. The line order is the slot id 0–18. It matches VCMI `ArtifactPosition` (`lib/constants/EntityIdentifiers.h L755–761`: HEAD=0 … MACH1=13 … MACH4=16, SPELLBOOK=17, MISC5=18, BACKPACK_START=19) and the per-hero slot loop in the h3m reader (`lib/mapping/MapFormatH3M.cpp L1646–1647`, 19 slots in SoD maps and 18 in RoE/AB maps per `lib/mapping/MapFeaturesH3M.cpp L61, L107`). This matters for the T2 map writer.

**`artraits.txt`** (SoD copy, original-data):

| Row / column | Content |
|---|---|
| record 0 | super-header: col 2 = `Hero Slots`, all other cells empty |
| record 1 | column names (below) |
| records 2–145 | 144 artifacts. **Row order = artifact id = VCMI `index`** (all 144 checked: `config/artifacts.json` indices 0–143 match the names in row order) |
| records 146–360 | 215 tab-only padding rows. Both copies have exactly 361 CSV records |
| col 0 `Name` | display name. Id 7 is spelled `Centaurs Axe` here, while its own description header is `{Centaur's Axe}` |
| col 1 `Cost` | gold value (integer). **Blank for ids 129–143** (combinations and unused). VCMI reads it as 0 and takes `value` from JSON (`CArtHandler.cpp L82`, `config/artifacts.json` e.g. L1942) |
| cols 2–20 | 19 slot flags, `x` = allowed. Order: Spell Book, War Machine 4, War Machine 3, War Machine 2, War Machine 1, **Misc 5**, Misc 4, Misc 3, Misc 2, Misc 1, Feet, Left Ring, Right Ring, Torso, Left Hand, Right Hand, Neck, Shoulders, Head. This is ArtSlots ids 17…0 reversed, with Misc 5 placed among the misc columns. VCMI's `ART_POS_LIST` follows this column order (`lib/entities/artifact/CArtHandler.cpp L29–48`). Misc artifacts flag all 5 misc columns and rings flag both ring columns |
| col 21 `Class (S,T,N,J,R)` | S = special, T = treasure, N = minor, J = major, R = relic (VCMI mapping `CArtHandler.cpp L67–68`) |
| col 22 `Description` | quoted multi-line text starting `{Name}`. Physical lines: SoD 651, AB 619 |

The cells are inconsistent: a "no" is a single space `' '` in the old columns but an empty string `''` in the SoD-added Misc 5 column and on rows 129–143. A parser must treat `''` and `' '` alike. VCMI's parser only tests `== "x"` (`CArtHandler.cpp L84–90`).

**AB copy vs SoD copy** (Python diff by name and by row):

| Aspect | AB `H3ab_bmp/ArTraits.txt` | SoD `H3bitmap/artraits.txt` |
|---|---|---|
| size | 26 851 B | 29 374 B |
| columns | 22 (no Misc 5) | 23 (Misc 5 inserted between War Machine 1 and Misc 4) |
| artifact rows | 129 (ids 0–128) | 144 (ids 0–143) |
| class counts | S 7, T 37, N 21, J 39, R 25 | S 7, T 37, N 21, J 39, R 40 |
| ids 0–128 | — | names, costs, classes and descriptions **identical**. The only change: the 46 misc-slot artifacts (incl. Spell Scroll) also get `x` in Misc 5 |
| added rows | — | 129–140: the 12 combination artifacts. 141–143: Diplomat's Suit, Mired in Neutrality, Ironfist of the Ogre (unused; description is a placeholder). All 15 have a blank cost and class R |
| `ArtSlots.txt` | 18 lines | 19 lines (Misc 5 appended) |

## 2. Membership by expansion

| Set | Ids | Count | Sources |
|---|---|---|---|
| RoE | 0–126 | 127 | VCMI `lib/mapping/MapFeaturesH3M.cpp L55` (`artifactsCount = 127`, 18 slots L61); wiki list templates mark all of 7–126 `roe` *(raw)* |
| AB | 127 Vial of Dragon Blood, 128 Armageddon's Blade | +2 = 129 | VCMI `MapFeaturesH3M.cpp L88` ("+ Armaggedon Blade and Vial of Dragon Blood"); wiki marks both `ab` *(raw)*; the AB copy of artraits has exactly 129 rows |
| SoD | 129–140 combinations; 141–143 unused | +15 = 144 | VCMI `MapFeaturesH3M.cpp L102` ("+ Combined artifacts + 3 unfinished artifacts"); wiki marks the 12 combinations `sod` *(raw)*; Artifact page: "Traces of 3 unrealized ones are in the game files" |
| SoD slot change | Misc 5 | 4 → 5 misc slots | wiki Artifact: "In The Restoration of Erathia and Armageddon's Blade, there are just 4 misc. slots"; `MapFeaturesH3M.cpp L107` (19 slots); AB `ArtSlots.txt` has 18 lines |

Ids 0–6 (Spell Book, Spell Scroll, Grail, war machines) are not in the wiki list templates. They are RoE by the VCMI count and the AB copy. **HotA changed this**: HotA adds artifacts (e.g. Pendant of Downfall, Ring of Suppression) and makes some unused traces real (an "Ironfist of the Ogre" combination). None of that applies here.

## 3. Full artifact table

Legend: **slot** comes from the original table (`Misc×5` = any misc slot, 4 in RoE/AB; `Ring` = either ring). For combinations the slot is the one the combination occupies plus the slots its parts lock (derived from the component slots; the wiki `blocked` fields agree where fetched: Elixir of Life, Bow of the Sharpshooter, Power of the Dragon Father). **cls**: tre/min/maj/rel/spe from col 21. **cost**: col 1; for combinations `— (VCMI value)`. The wiki costs and classes agree with the original table for all 134 rows it lists (scripted check). **exp**: section 2. **VCMI effect**: `A/D/P/K±n` = PRIMARY_SKILL attack/defence/spellpower/knowledge; `[add]` ADDITIVE_VALUE, `[≤]` INDEPENDENT_MIN (cap: "at most"), `[≥]` INDEPENDENT_MAX ("at least"), `[%base]` PERCENT_TO_BASE; `{...}` limiter/propagator; `instance:` = `instanceBonuses` (these stack per copy); `ai` = addInfo. Bonuses without a propagator sit on the hero node and reach the hero's own stacks through the bonus tree. `BATTLE_WIDE` reaches every unit and both heroes on the battlefield (`lib/battle/BattleInfo.cpp L62–69` attaches both armies to the battle node). **kinds/scope**: section 4.

| id | name (artraits.txt) | slot(s) | cls | cost | exp | VCMI key · lines | VCMI effect (compressed) | kinds (own; +parts) | scope |
|---:|---|---|---|---:|---|---|---|---|---|
| 0 | Spell Book | Book | spe | 0 | RoE | `spellBook` L2–6 | — | spell.book | E04b |
| 1 | Spell Scroll | Misc×5 | spe | 2500 | RoE | `spellScroll` L7–11 | — | spell.grant | E04b |
| 2 | The Grail | — (backpack) | spe | 0 | RoE | `grail` L12–16 | — | grail | adventure |
| 3 | Catapult | WM4 | spe | 1000 | RoE | `catapult` L17–22 | warMachine=catapult | war_machine.catapult | E04c |
| 4 | Ballista | WM1 | spe | 2500 | RoE | `ballista` L23–28 | warMachine=ballista | war_machine | E04a |
| 5 | Ammo Cart | WM2 | spe | 1000 | RoE | `ammoCart` L29–34 | warMachine=ammoCart | war_machine | E04a |
| 6 | First Aid Tent | WM3 | spe | 750 | RoE | `firstAidTent` L35–40 | warMachine=firstAidTent | war_machine | E04a |
| 7 | Centaurs Axe | RHand | tre | 2000 | RoE | `centaurAxe` L41–53 | A+2 | stat.attack | E04a |
| 8 | Blackshard of the Dead Knight | RHand | min | 3000 | RoE | `blackshardOfTheDeadKnight` L54–66 | A+3 | stat.attack | E04a |
| 9 | Greater Gnoll's Flail | RHand | min | 4000 | RoE | `greaterGnollsFlail` L67–79 | A+4 | stat.attack | E04a |
| 10 | Ogre's Club of Havoc | RHand | maj | 5000 | RoE | `ogresClubOfHavoc` L80–92 | A+5 | stat.attack | E04a |
| 11 | Sword of Hellfire | RHand | maj | 6000 | RoE | `swordOfHellfire` L93–105 | A+6 | stat.attack | E04a |
| 12 | Titan's Gladius | RHand | rel | 10000 | RoE | `titansGladius` L106–124 | A+12; D-3 | stat.attack, stat.defense | E04a |
| 13 | Shield of the Dwarven Lords | LHand | tre | 2000 | RoE | `shieldOfTheDwarvenLords` L125–137 | D+2 | stat.defense | E04a |
| 14 | Shield of the Yawning Dead | LHand | min | 3000 | RoE | `shieldOfTheYawningDead` L138–150 | D+3 | stat.defense | E04a |
| 15 | Buckler of the Gnoll King | LHand | min | 4000 | RoE | `bucklerOfTheGnollKing` L151–163 | D+4 | stat.defense | E04a |
| 16 | Targ of the Rampaging Ogre | LHand | maj | 5000 | RoE | `targOfTheRampagingOgre` L164–176 | D+5 | stat.defense | E04a |
| 17 | Shield of the Damned | LHand | maj | 6000 | RoE | `shieldOfTheDamned` L177–189 | D+6 | stat.defense | E04a |
| 18 | Sentinel's Shield | LHand | rel | 10000 | RoE | `sentinelsShield` L190–208 | D+12; A-3 | stat.defense, stat.attack | E04a |
| 19 | Helm of the Alabaster Unicorn | Head | tre | 1000 | RoE | `helmOfTheAlabasterUnicorn` L209–221 | K+1 | stat.knowledge | E04b |
| 20 | Skull Helmet | Head | tre | 3000 | RoE | `skullHelmet` L222–234 | K+2 | stat.knowledge | E04b |
| 21 | Helm of Chaos | Head | min | 4000 | RoE | `helmOfChaos` L235–247 | K+3 | stat.knowledge | E04b |
| 22 | Crown of the Supreme Magi | Head | min | 5000 | RoE | `crownOfTheSupremeMagi` L248–260 | K+4 | stat.knowledge | E04b |
| 23 | Hellstorm Helmet | Head | maj | 6000 | RoE | `hellstormHelmet` L261–273 | K+5 | stat.knowledge | E04b |
| 24 | Thunder Helmet | Head | rel | 10000 | RoE | `thunderHelmet` L274–292 | K+10; P-2 | stat.knowledge, stat.power | E04b |
| 25 | Breastplate of Petrified Wood | Torso | tre | 1000 | RoE | `breastplateOfPetrifiedWood` L293–305 | P+1 | stat.power | E04b |
| 26 | Rib Cage | Torso | min | 3000 | RoE | `ribCage` L306–318 | P+2 | stat.power | E04b |
| 27 | Scales of the Greater Basilisk | Torso | min | 4000 | RoE | `scalesOfTheGreaterBasilisk` L319–331 | P+3 | stat.power | E04b |
| 28 | Tunic of the Cyclops King | Torso | maj | 5000 | RoE | `tunicOfTheCyclopsKing` L332–344 | P+4 | stat.power | E04b |
| 29 | Breastplate of Brimstone | Torso | maj | 6000 | RoE | `breastplateOfBrimstone` L345–357 | P+5 | stat.power | E04b |
| 30 | Titan's Cuirass | Torso | rel | 10000 | RoE | `titansCuirass` L358–376 | P+10; K-2 | stat.power, stat.knowledge | E04b |
| 31 | Armor of Wonder | Torso | min | 4000 | RoE | `armorOfWonder` L377–407 | A+1; D+1; P+1; K+1 | stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 32 | Sandals of the Saint | Feet | rel | 8000 | RoE | `sandalsOfTheSaint` L408–438 | A+2; D+2; P+2; K+2 | stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 33 | Celestial Necklace of Bliss | Neck | rel | 12000 | RoE | `celestialNecklaceOfBliss` L439–469 | A+3; D+3; P+3; K+3 | stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 34 | Lion's Shield of Courage | LHand | rel | 16000 | RoE | `lionsShieldOfCourage` L470–500 | A+4; D+4; P+4; K+4 | stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 35 | Sword of Judgement | RHand | rel | 20000 | RoE | `swordOfJudgement` L501–531 | A+5; D+5; P+5; K+5 | stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 36 | Helm of Heavenly Enlightenment | Head | rel | 24000 | RoE | `helmOfHeavenlyEnlightenment` L532–562 | A+6; D+6; P+6; K+6 | stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 37 | Quiet Eye of the Dragon | Ring | tre | 2000 | RoE | `quietEyeOfTheDragon` L563–581 | instance: A+1, D+1 | stat.attack, stat.defense | E04a |
| 38 | Red Dragon Flame Tongue | RHand | min | 4000 | RoE | `redDragonFlameTongue` L582–600 | A+2; D+2 | stat.attack, stat.defense | E04a |
| 39 | Dragon Scale Shield | LHand | maj | 6000 | RoE | `dragonScaleShield` L601–619 | A+3; D+3 | stat.attack, stat.defense | E04a |
| 40 | Dragon Scale Armor | Torso | rel | 8000 | RoE | `dragonScaleArmor` L620–638 | A+4; D+4 | stat.attack, stat.defense | E04a |
| 41 | Dragonbone Greaves | Feet | tre | 2000 | RoE | `dragonboneGreaves` L639–657 | P+1; K+1 | stat.power, stat.knowledge | E04b |
| 42 | Dragon Wing Tabard | Shoulders | min | 4000 | RoE | `dragonWingTabard` L658–676 | P+2; K+2 | stat.power, stat.knowledge | E04b |
| 43 | Necklace of Dragonteeth | Neck | maj | 6000 | RoE | `necklaceOfDragonteeth` L677–695 | P+3; K+3 | stat.power, stat.knowledge | E04b |
| 44 | Crown of Dragontooth | Head | rel | 8000 | RoE | `crownOfDragontooth` L696–714 | P+4; K+4 | stat.power, stat.knowledge | E04b |
| 45 | Still Eye of the Dragon | Ring | tre | 2000 | RoE | `stillEyeOfTheDragon` L715–731 | MORALE=1; LUCK=1 | morale, luck | E04a |
| 46 | Clover of Fortune | Misc×5 | tre | 1000 | RoE | `cloverOfFortune` L732–743 | LUCK=1 | luck | E04a |
| 47 | Cards of Prophecy | Misc×5 | tre | 1000 | RoE | `cardsOfProphecy` L744–755 | LUCK=1 | luck | E04a |
| 48 | Ladybird of Luck | Misc×5 | tre | 1000 | RoE | `ladybirdOfLuck` L756–767 | LUCK=1 | luck | E04a |
| 49 | Badge of Courage | Misc×5 | tre | 1000 | RoE | `badgeOfCourage` L768–784 | MORALE=1; MIND_IMMUNITY=0 | morale, spell.immunity_mind | E04a/E04b |
| 50 | Crest of Valor | Misc×5 | tre | 1000 | RoE | `crestOfValor` L785–796 | MORALE=1 | morale | E04a |
| 51 | Glyph of Gallantry | Misc×5 | tre | 1000 | RoE | `glyphOfGallantry` L797–808 | MORALE=1 | morale | E04a |
| 52 | Speculum | Misc×5 | tre | 1000 | RoE | `speculum` L809–820 | SIGHT_RADIUS=1 | vision | adventure |
| 53 | Spyglass | Misc×5 | tre | 1000 | RoE | `spyglass` L821–832 | SIGHT_RADIUS=1 | vision | adventure |
| 54 | Amulet of the Undertaker | Neck | tre | 2000 | RoE | `amuletOfTheUndertaker` L833–844 | UNDEAD_RAISE_PERCENTAGE=5[add] | outcome.necromancy_percent | E04a |
| 55 | Vampire's Cowl | Shoulders | min | 4000 | RoE | `vampiresCowl` L845–856 | UNDEAD_RAISE_PERCENTAGE=10[add] | outcome.necromancy_percent | E04a |
| 56 | Dead Man's Boots | Feet | maj | 6000 | RoE | `deadMansBoots` L857–868 | UNDEAD_RAISE_PERCENTAGE=15[add] | outcome.necromancy_percent | E04a |
| 57 | Garniture of Interference | Neck | maj | 2000 | RoE | `garnitureOfInterference` L869–880 | MAGIC_RESISTANCE=5 | spell.magic_resistance | E04b |
| 58 | Surcoat of Counterpoise | Shoulders | maj | 4000 | RoE | `surcoatOfCounterpoise` L881–892 | MAGIC_RESISTANCE=10 | spell.magic_resistance | E04b |
| 59 | Boots of Polarity | Feet | rel | 6000 | RoE | `bootsOfPolarity` L893–904 | MAGIC_RESISTANCE=15 | spell.magic_resistance | E04b |
| 60 | Bow of Elven Cherrywood | Misc×5 | tre | 2000 | RoE | `bowOfElvenCherrywood` L905–926 | PERCENTAGE_DAMAGE_BOOST(damageTypeRanged)=5[add] {needs archery} | ranged.archery_bonus | E04a |
| 61 | Bowstring of the Unicorn's Mane | Misc×5 | min | 4000 | RoE | `bowstringOfTheUnicornsMane` L927–948 | PERCENTAGE_DAMAGE_BOOST(damageTypeRanged)=10[add] {needs archery} | ranged.archery_bonus | E04a |
| 62 | Angel Feather Arrows | Misc×5 | maj | 6000 | RoE | `angelFeatherArrows` L949–970 | PERCENTAGE_DAMAGE_BOOST(damageTypeRanged)=15[add] {needs archery} | ranged.archery_bonus | E04a |
| 63 | Bird of Perception | Misc×5 | tre | 1000 | RoE | `birdOfPerception` L971–982 | LEARN_BATTLE_SPELL_CHANCE=5[add] | spell.learn_after_battle | E04b |
| 64 | Stoic Watchman | Misc×5 | tre | 2000 | RoE | `stoicWatchman` L983–994 | LEARN_BATTLE_SPELL_CHANCE=10[add] | spell.learn_after_battle | E04b |
| 65 | Emblem of Cognizance | Misc×5 | min | 3000 | RoE | `emblemOfCognizance` L995–1006 | LEARN_BATTLE_SPELL_CHANCE=15[add] | spell.learn_after_battle | E04b |
| 66 | Statesman's Medal | Neck | maj | 5000 | RoE | `statesmansMedal` L1007–1018 | SURRENDER_DISCOUNT=10 | outcome.surrender_discount | E04a |
| 67 | Diplomat's Ring | Ring | maj | 5000 | RoE | `diplomatsRing` L1019–1030 | SURRENDER_DISCOUNT=10 | outcome.surrender_discount | E04a |
| 68 | Ambassador's Sash | Shoulders | maj | 5000 | RoE | `ambassadorsSash` L1031–1042 | SURRENDER_DISCOUNT=10 | outcome.surrender_discount | E04a |
| 69 | Ring of the Wayfarer | Ring | maj | 5000 | RoE | `ringOfTheWayfarer` L1043–1054 | STACKS_SPEED=1 | unit.speed | E04a |
| 70 | Equestrian's Gloves | Ring | min | 3000 | RoE | `equestriansGloves` L1055–1067 | MOVEMENT(heroMovementLand)=300[add] | move.land | adventure |
| 71 | Necklace of Ocean Guidance | Neck | maj | 10000 | RoE | `necklaceOfOceanGuidance` L1068–1081 | MOVEMENT(heroMovementSea)=1000[add]; waterOnly | move.sea | adventure |
| 72 | Angel Wings | Shoulders | rel | 20000 | RoE | `angelWings` L1082–1093 | FLYING_MOVEMENT=0[≤] | move.fly | adventure |
| 73 | Charm of Mana | Misc×5 | tre | 500 | RoE | `charmOfMana` L1094–1105 | MANA_REGENERATION=1 | mana.regen | E04b |
| 74 | Talisman of Mana | Misc×5 | tre | 1000 | RoE | `talismanOfMana` L1106–1117 | MANA_REGENERATION=2 | mana.regen | E04b |
| 75 | Mystic Orb of Mana | Misc×5 | tre | 1500 | RoE | `mysticOrbOfMana` L1118–1129 | MANA_REGENERATION=3 | mana.regen | E04b |
| 76 | Collar of Conjuring | Neck | tre | 500 | RoE | `collarOfConjuring` L1130–1141 | SPELL_DURATION=1 | spell.duration | E04b |
| 77 | Ring of Conjuring | Ring | tre | 1000 | RoE | `ringOfConjuring` L1142–1153 | SPELL_DURATION=2 | spell.duration | E04b |
| 78 | Cape of Conjuring | Shoulders | tre | 1500 | RoE | `capeOfConjuring` L1154–1165 | SPELL_DURATION=3 | spell.duration | E04b |
| 79 | Orb of the Firmament | Misc×5 | maj | 6000 | RoE | `orbOfTheFirmament` L1166–1178 | SPELL_DAMAGE(air)=50 | spell.damage_school | E04b |
| 80 | Orb of Silt | Misc×5 | maj | 6000 | RoE | `orbOfSilt` L1179–1191 | SPELL_DAMAGE(earth)=50 | spell.damage_school | E04b |
| 81 | Orb of Tempestuous Fire | Misc×5 | maj | 6000 | RoE | `orbOfTempestuousFire` L1192–1204 | SPELL_DAMAGE(fire)=50 | spell.damage_school | E04b |
| 82 | Orb of Driving Rain | Misc×5 | maj | 6000 | RoE | `orbOfDrivingRain` L1205–1217 | SPELL_DAMAGE(water)=50 | spell.damage_school | E04b |
| 83 | Recanter's Cloak | Shoulders | maj | 8000 | RoE | `recantersCloak` L1218–1230 | BLOCK_MAGIC_ABOVE=2[≤] {BATTLE_WIDE} | spell.block_level_battle | E04b |
| 84 | Spirit of Oppression | Misc×5 | tre | 2000 | RoE | `spiritOfOppression` L1231–1243 | MORALE=0[≤] {BATTLE_WIDE} | morale.cap_battle | E04a |
| 85 | Hourglass of the Evil Hour | Misc×5 | tre | 2000 | RoE | `hourglassOfTheEvilHour` L1244–1256 | LUCK=0[≤] {BATTLE_WIDE} | luck.cap_battle | E04a |
| 86 | Tome of Fire Magic | Misc×5 | rel | 20000 | RoE | `tomeOfFireMagic` L1257–1269 | SPELLS_OF_SCHOOL(fire)=0 | spell.grant | E04b |
| 87 | Tome of Air Magic | Misc×5 | rel | 20000 | RoE | `tomeOfAirMagic` L1270–1282 | SPELLS_OF_SCHOOL(air)=0 | spell.grant | E04b |
| 88 | Tome of Water Magic | Misc×5 | rel | 20000 | RoE | `tomeOfWaterMagic` L1283–1295 | SPELLS_OF_SCHOOL(water)=0 | spell.grant | E04b |
| 89 | Tome of Earth Magic | Misc×5 | rel | 20000 | RoE | `tomeOfEarthMagic` L1296–1308 | SPELLS_OF_SCHOOL(earth)=0 | spell.grant | E04b |
| 90 | Boots of Levitation | Feet | rel | 10000 | RoE | `bootsOfLevitation` L1309–1321 | WATER_WALKING=0[≤]; waterOnly | move.water_walk | adventure |
| 91 | Golden Bow | Misc×5 | maj | 8000 | RoE | `goldenBow` L1322–1340 | NO_DISTANCE_PENALTY=0[add] {SHOOTER_ONLY}; NO_WALL_PENALTY=0[add] {SHOOTER_ONLY} | ranged.no_range_penalty, ranged.no_wall_penalty | E04a/E04c |
| 92 | Sphere of Permanence | Misc×5 | maj | 7500 | RoE | `sphereOfPermanence` L1341–1357 | SPELL_IMMUNITY(dispel) ai=1; SPELL_IMMUNITY(dispelHelpful) ai=1 | spell.immunity_named | E04b |
| 93 | Orb of Vulnerability | Misc×5 | rel | 25000 | RoE | `orbOfVulnerability` L1358–1391 | NEGATE_ALL_NATURAL_IMMUNITIES(immunityBattleWide)=0 {BATTLE_WIDE}; NEGATE_ALL_NATURAL_IMMUNITIES(immunityEnemyHero)=0; MAGIC_RESISTANCE=0[≤] {BATTLE_WIDE}; SPELL_RESISTANCE_AURA=0[≤] {BATTLE_WIDE} | spell.negate_resistance_battle | E04b |
| 94 | Ring of Vitality | Ring | tre | 5000 | RoE | `ringOfVitality` L1392–1403 | STACK_HEALTH=1[add] | unit.hp_flat | E04a |
| 95 | Ring of Life | Ring | min | 5000 | RoE | `ringOfLife` L1404–1415 | STACK_HEALTH=1[add] | unit.hp_flat | E04a |
| 96 | Vial of Lifeblood | Misc×5 | maj | 10000 | RoE | `vialOfLifeblood` L1416–1427 | STACK_HEALTH=2[add] | unit.hp_flat | E04a |
| 97 | Necklace of Swiftness | Neck | tre | 5000 | RoE | `necklaceOfSwiftness` L1428–1439 | STACKS_SPEED=1 | unit.speed | E04a |
| 98 | Boots of Speed | Feet | min | 6000 | RoE | `bootsOfSpeed` L1440–1452 | MOVEMENT(heroMovementLand)=600[add] | move.land | adventure |
| 99 | Cape of Velocity | Shoulders | maj | 10000 | RoE | `capeOfVelocity` L1453–1464 | STACKS_SPEED=2 | unit.speed | E04a |
| 100 | Pendant of Dispassion | Neck | tre | 1000 | RoE | `pendantOfDispassion` L1465–1477 | SPELL_IMMUNITY(berserk)=0 | spell.immunity_named | E04b |
| 101 | Pendant of Second Sight | Neck | maj | 5000 | RoE | `pendantOfSecondSight` L1478–1490 | SPELL_IMMUNITY(blind)=0 | spell.immunity_named | E04b |
| 102 | Pendant of Holiness | Neck | tre | 1000 | RoE | `pendantOfHoliness` L1491–1503 | SPELL_IMMUNITY(curse)=0 | spell.immunity_named | E04b |
| 103 | Pendant of Life | Neck | tre | 2500 | RoE | `pendantOfLife` L1504–1516 | SPELL_IMMUNITY(deathRipple)=0 | spell.immunity_named | E04b |
| 104 | Pendant of Death | Neck | tre | 2500 | RoE | `pendantOfDeath` L1517–1530 | SPELL_IMMUNITY(destroyUndead)=0 {IS_UNDEAD} | spell.immunity_named | E04b |
| 105 | Pendant of Free Will | Neck | tre | 1000 | RoE | `pendantOfFreeWill` L1531–1543 | SPELL_IMMUNITY(hypnotize)=0 | spell.immunity_named | E04b |
| 106 | Pendant of Negativity | Neck | maj | 5000 | RoE | `pendantOfNegativity` L1544–1562 | SPELL_IMMUNITY(lightningBolt)=0; SPELL_IMMUNITY(chainLightning)=0 | spell.immunity_named | E04b |
| 107 | Pendant of Total Recall | Neck | tre | 3000 | RoE | `pendantOfTotalRecall` L1563–1575 | SPELL_IMMUNITY(forgetfulness)=0 | spell.immunity_named | E04b |
| 108 | Pendant of Courage | Neck | maj | 7000 | RoE | `pendantOfCourage` L1576–1592 | MORALE=3; LUCK=3 | morale, luck | E04a |
| 109 | Everflowing Crystal Cloak | Shoulders | maj | 5000 | RoE | `everflowingCrystalCloak` L1593–1606 | instance: GENERATE_RESOURCE(crystal)=1 | resource.daily | adventure |
| 110 | Ring of Infinite Gems | Ring | maj | 5000 | RoE | `ringOfInfiniteGems` L1607–1620 | instance: GENERATE_RESOURCE(gems)=1 | resource.daily | adventure |
| 111 | Everpouring Vial of Mercury | Misc×5 | maj | 5000 | RoE | `everpouringVialOfMercury` L1621–1634 | instance: GENERATE_RESOURCE(mercury)=1 | resource.daily | adventure |
| 112 | Inexhaustible Cart of Ore | Misc×5 | min | 5000 | RoE | `inexhaustibleCartOfOre` L1635–1648 | instance: GENERATE_RESOURCE(ore)=1 | resource.daily | adventure |
| 113 | Eversmoking Ring of Sulfur | Ring | maj | 5000 | RoE | `eversmokingRingOfSulfur` L1649–1662 | instance: GENERATE_RESOURCE(sulfur)=1 | resource.daily | adventure |
| 114 | Inexhaustible Cart of Lumber | Misc×5 | min | 5000 | RoE | `inexhaustibleCartOfLumber` L1663–1676 | instance: GENERATE_RESOURCE(wood)=1 | resource.daily | adventure |
| 115 | Endless Sack of Gold | Misc×5 | rel | 10000 | RoE | `endlessSackOfGold` L1677–1690 | instance: GENERATE_RESOURCE(gold)=1000 | resource.daily | adventure |
| 116 | Endless Bag of Gold | Misc×5 | maj | 7500 | RoE | `endlessBagOfGold` L1691–1704 | instance: GENERATE_RESOURCE(gold)=750 | resource.daily | adventure |
| 117 | Endless Purse of Gold | Misc×5 | maj | 5000 | RoE | `endlessPurseOfGold` L1705–1718 | instance: GENERATE_RESOURCE(gold)=500 | resource.daily | adventure |
| 118 | Legs of Legion | Misc×5 | tre | 5000 | RoE | `legsOfLegion` L1719–1731 | instance: CREATURE_GROWTH(lvl2)=5 {TOWN_AND_VISITOR} | growth.flat | adventure |
| 119 | Loins of Legion | Misc×5 | min | 5000 | RoE | `loinsOfLegion` L1732–1744 | instance: CREATURE_GROWTH(lvl3)=4 {TOWN_AND_VISITOR} | growth.flat | adventure |
| 120 | Torso of Legion | Misc×5 | min | 5000 | RoE | `torsoOfLegion` L1745–1757 | instance: CREATURE_GROWTH(lvl4)=3 {TOWN_AND_VISITOR} | growth.flat | adventure |
| 121 | Arms of Legion | Misc×5 | maj | 5000 | RoE | `armsOfLegion` L1758–1770 | instance: CREATURE_GROWTH(lvl5)=2 {TOWN_AND_VISITOR} | growth.flat | adventure |
| 122 | Head of Legion | Misc×5 | maj | 5000 | RoE | `headOfLegion` L1771–1783 | instance: CREATURE_GROWTH(lvl6)=1 {TOWN_AND_VISITOR} | growth.flat | adventure |
| 123 | Sea Captain's Hat | Head | rel | 15000 | RoE | `seaCaptainsHat` L1784–1814 | WHIRLPOOL_PROTECTION=0; MOVEMENT(heroMovementSea)=500[add]; SPELL(summonBoat)=3[≥]; SPELL(scuttleBoat)=3[≥]; waterOnly | move.sea, adv.spell, adv.whirlpool_protection | adventure |
| 124 | Spellbinder's Hat | Head | rel | 30000 | RoE | `spellbindersHat` L1815–1827 | SPELLS_OF_LEVEL(spellLevel5)=3 | spell.grant | E04b |
| 125 | Shackles of War | Misc×5 | maj | 5000 | RoE | `shacklesOfWar` L1828–1838 | BATTLE_NO_FLEEING {BATTLE_WIDE} | battle.no_flee | E04a |
| 126 | Orb of Inhibition | Misc×5 | rel | 20000 | RoE | `orbOfInhibition` L1839–1849 | BLOCK_ALL_MAGIC {BATTLE_WIDE} | spell.block_all_battle | E04b |
| 127 | Vial of Dragon Blood | Misc×5 | rel | 20000 | AB | `vialOfDragonBlood` L1850–1871 | A+5 {DRAGON_NATURE}; D+5 {DRAGON_NATURE}; class=SPECIAL | unit.attack_defense_filtered | E04a |
| 128 | Armageddon's Blade | RHand | rel | 50000 | AB | `armageddonsBlade` L1872–1915 | SPELL(armageddon)=3[≥]; SPELL_IMMUNITY(armageddon)=0; A+3; D+3; P+3; K+6; class=SPECIAL | stat.attack, stat.defense, stat.power, stat.knowledge, spell.grant, spell.immunity_named | E04a/E04b |
| 129 | Angelic Alliance | RHand + locks Torso, Feet, Neck, LHand, Head | rel | — (84000) | SoD | `angelicAlliance` L1916–1943 | NONEVIL_ALIGNMENT_MIX=0; OPENING_BATTLE_SPELL(prayer)=10 (+ all part bonuses) | morale.alignment_mix, spell.cast_at_start; +parts: stat.attack, stat.defense, stat.power, stat.knowledge | E04a/E04b |
| 130 | Cloak of the Undead King | Shoulders + locks Neck, Feet | rel | — (12000) | SoD | `cloakOfTheUndeadKing` L1944–1977 | IMPROVED_NECROMANCY(skeleton) ai=0; IMPROVED_NECROMANCY(walkingDead) ai=1; IMPROVED_NECROMANCY(wight) ai=2; IMPROVED_NECROMANCY(lich) ai=3 (+ all part bonuses) | outcome.necromancy_creature; +parts: outcome.necromancy_percent | E04a |
| 131 | Elixir of Life | Misc×5 + locks Ring×2 | rel | — (20000) | SoD | `elixirOfLife` L1978–2013 | STACK_HEALTH=25[%base] {needs LIVING}; HP_REGENERATION=50 {needs LIVING} (+ all part bonuses) | unit.hp_percent, unit.regeneration; +parts: unit.hp_flat | E04a |
| 132 | Armor of the Damned | Torso + locks RHand, LHand, Head | rel | — (12000) | SoD | `armorOfTheDamned` L2014–2052 | OPENING_BATTLE_SPELL(slow)=50; OPENING_BATTLE_SPELL(curse)=50; OPENING_BATTLE_SPELL(weakness)=50; OPENING_BATTLE_SPELL(misfortune)=50 (+ all part bonuses) | spell.cast_at_start; +parts: stat.attack, stat.defense, stat.knowledge, stat.power | E04a/E04b |
| 133 | Statue of Legion | Misc×5 + locks Misc×4 | rel | — (25000) | SoD | `statueOfLegion` L2053–2073 | CREATURE_GROWTH_PERCENT=50 {PLAYER} (+ all part bonuses) | growth.percent; +parts: growth.flat | adventure |
| 134 | Power of the Dragon Father | Torso + locks Ring×2, RHand, LHand, Feet, Shoulders, Neck, Head | rel | — (42000) | SoD | `powerOfTheDragonFather` L2074–2122 | LEVEL_SPELL_IMMUNITY=4[≥]; A+6; D+6; P+6; K+6 (+ all part bonuses) | stat.attack, stat.defense, stat.power, stat.knowledge, spell.immunity_level; +parts: morale, luck | E04a/E04b |
| 135 | Titan's Thunder | RHand + locks LHand, Head, Torso | rel | — (40000) | SoD | `titansThunder` L2123–2143 | SPELL(titanBolt)=3 (+ all part bonuses) | spell.grant, spell.book; +parts: stat.attack, stat.defense, stat.knowledge, stat.power | E04a/E04b |
| 136 | Admiral's Hat | Head + locks Neck | rel | — (25000) | SoD | `admiralsHat` L2144–2162 | FREE_SHIP_BOARDING=0; waterOnly (+ all part bonuses) | move.boat_transfer; +parts: move.sea, adv.spell, adv.whirlpool_protection | adventure |
| 137 | Bow of the Sharpshooter | Misc×5 + locks Misc×2 | rel | — (12000) | SoD | `bowOfTheSharpshooter` L2163–2194 | NO_DISTANCE_PENALTY=0[add] {SHOOTER_ONLY}; NO_WALL_PENALTY=0[add] {SHOOTER_ONLY}; FREE_SHOOTING=0[add] {SHOOTER_ONLY} (+ all part bonuses) | ranged.shoot_adjacent, ranged.no_range_penalty, ranged.no_wall_penalty; +parts: ranged.archery_bonus | E04a/E04c |
| 138 | Wizard's Well | Misc×5 + locks Misc×2 | rel | — (3000) | SoD | `wizardsWell` L2195–2213 | MANA_PERCENTAGE_REGENERATION=100 (+ all part bonuses) | mana.regen_full; +parts: mana.regen | E04b |
| 139 | Ring of the Magi | Ring + locks Neck, Shoulders | rel | — (3000) | SoD | `ringOfTheMagi` L2214–2232 | SPELL_DURATION=50 (+ all part bonuses) | spell.duration | E04b |
| 140 | Cornucopia | Misc×5 + locks Shoulders, Ring×2 | rel | — (20000) | SoD | `cornucopia` L2233–2271 | instance: GENERATE_RESOURCE(crystal)=4, GENERATE_RESOURCE(gems)=4, GENERATE_RESOURCE(mercury)=4, GENERATE_RESOURCE(sulfur)=4 (+ all part bonuses) | resource.daily | adventure |
| 141 | Diplomat's Suit | Shoulders | rel | — | SoD | `unusedArtifact1` L2273–2278 | type=[CREATURE] → hero slots cleared (CArtHandler.cpp L402–403, L431–439); class=SPECIAL | none (unused) | none |
| 142 | Mired in Neutrality | Torso | rel | — | SoD | `unusedArtifact2` L2279–2284 | type=[CREATURE] → hero slots cleared (CArtHandler.cpp L402–403, L431–439); class=SPECIAL | none (unused) | none |
| 143 | Ironfist of the Ogre | RHand | rel | — | SoD | `unusedArtifact3` L2285–2290 | type=[CREATURE] → hero slots cleared (CArtHandler.cpp L402–403, L431–439); class=SPECIAL | none (unused) | none |

## 4. Effect-kind vocabulary

These are proposed closed-enum effect primitives for the content packs (ADR-0013 open point 2). Each kind belongs to the epic that must model it. "n" counts artifacts that have the kind as an **own** effect. A combination artifact also delivers its parts' effects, listed as "via parts". Selectors the engine needs: own army (default), own living stacks, own dragons, own shooters, own undead (Pendant of Death limiter), battle-wide (both sides), all enemy stacks (Armor of the Damned).

| scope | kind | meaning (engine semantics) | VCMI encoding | n | artifact ids (own effect) |
|---|---|---|---|---:|---|
| E04a | `stat.attack` | hero Attack ± n (added to every own stack's attack); negatives allowed (12, 18). Hero total clamped ≥ 0 | PRIMARY_SKILL attack | 19 | 7, 8, 9, 10, 11, 12, 18, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 128, 134 · via parts: 129, 132, 135 |
| E04a | `stat.defense` | hero Defense ± n; negatives allowed (12, 18). Clamped ≥ 0 | PRIMARY_SKILL defence | 19 | 12, 13, 14, 15, 16, 17, 18, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 128, 134 · via parts: 129, 132, 135 |
| E04a | `morale` | +n morale to own army (cap ±3) | MORALE | 5 | 45, 49, 50, 51, 108 · via parts: 134 |
| E04a | `luck` | +n luck to own army (cap ±3) | LUCK | 5 | 45, 46, 47, 48, 108 · via parts: 134 |
| E04a | `morale.cap_battle` | **both sides**: positive morale removed (morale ≤ 0); negative morale stays | MORALE=0 [≤] BATTLE_WIDE | 1 | 84 |
| E04a | `luck.cap_battle` | **both sides**: luck ≤ 0 | LUCK=0 [≤] BATTLE_WIDE | 1 | 85 |
| E04a | `morale.alignment_mix` | changes the alignment-mix morale rule for the wearer's army (good + neutral towns count as one) | NONEVIL_ALIGNMENT_MIX | 1 | 129 |
| E04a | `unit.speed` | +n speed to all own stacks | STACKS_SPEED | 3 | 69, 97, 99 |
| E04a | `unit.hp_flat` | +n hit points to all own stacks | STACK_HEALTH [add] | 3 | 94, 95, 96 · via parts: 131 |
| E04a | `unit.hp_percent` | +25 % of *base* HP (rounded down), living stacks only | STACK_HEALTH [%base] {LIVING} | 1 | 131 |
| E04a | `unit.regeneration` | top creature regains up to 50 HP at the start of its turn, living only | HP_REGENERATION=50 {LIVING} | 1 | 131 |
| E04a | `unit.attack_defense_filtered` | +5 A/+5 D to own stacks matching a creature selector (dragons) | PRIMARY_SKILL {DRAGON_NATURE} | 1 | 127 |
| E04a | `ranged.archery_bonus` | +n % ranged damage, added to the Archery %; **no effect without the Archery skill** | PERCENTAGE_DAMAGE_BOOST(ranged) [add] {needs Archery} | 3 | 60, 61, 62 · via parts: 137 |
| E04a | `ranged.no_range_penalty` | own shooters ignore the range (distance) penalty | NO_DISTANCE_PENALTY {SHOOTER_ONLY} | 2 | 91, 137 |
| E04a | `ranged.shoot_adjacent` | own shooters may shoot while adjacent to an enemy | FREE_SHOOTING {SHOOTER_ONLY} | 1 | 137 |
| E04a | `war_machine` | places a war-machine stack for the hero (WM1 Ballista, WM2 Ammo Cart, WM3 First Aid Tent) | `warMachine` | 3 | 4, 5, 6 |
| E04a | `battle.no_flee` | **both sides**: no retreat, no surrender (hero-vs-hero only) | BATTLE_NO_FLEEING BATTLE_WIDE | 1 | 125 |
| E04a | `outcome.surrender_discount` | −n % surrender cost (outcome input; the orchestrator computes the cost) | SURRENDER_DISCOUNT | 3 | 66, 67, 68 |
| E04a | `outcome.necromancy_percent` | +n % Necromancy (additive; needs the skill; total ≤ 100 %) | UNDEAD_RAISE_PERCENTAGE [add] | 3 | 54, 55, 56 · via parts: 130 |
| E04a | `outcome.necromancy_creature` | raised creature by Necromancy level; works without the skill | IMPROVED_NECROMANCY | 1 | 130 |
| E04b | `stat.power` | hero Spell Power ± n (clamped ≥ 1) | PRIMARY_SKILL spellpower | 19 | 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 41, 42, 43, 44, 128, 134 · via parts: 129, 132, 135 |
| E04b | `stat.knowledge` | hero Knowledge ± n (clamped ≥ 1) | PRIMARY_SKILL knowledge | 19 | 19, 20, 21, 22, 23, 24, 30, 31, 32, 33, 34, 35, 36, 41, 42, 43, 44, 128, 134 · via parts: 129, 132, 135 |
| E04b | `spell.book` | spell book (slot 17); Titan's Thunder adds one | id 0 / ArtifactUtils::checkSpellbookIsNeeded | 2 | 0, 135 |
| E04b | `spell.grant` | adds spells to the book (scroll, tomes, level-5 hat, Armageddon, Titan's Bolt) | SPELL, SPELLS_OF_SCHOOL, SPELLS_OF_LEVEL | 8 | 1, 86, 87, 88, 89, 124, 128, 135 |
| E04b | `spell.cast_at_start` | casts expert spells at battle start: own side (AA Prayer, 10 rounds) or **enemy** (AotD: Slow, Curse, Weakness, Misfortune; 50 rounds) | OPENING_BATTLE_SPELL | 2 | 129, 132 |
| E04b | `spell.duration` | +n rounds to the hero's spells | SPELL_DURATION | 4 | 76, 77, 78, 139 |
| E04b | `spell.damage_school` | +50 % damage for one school | SPELL_DAMAGE(school) | 4 | 79, 80, 81, 82 |
| E04b | `spell.immunity_named` | own stacks immune to the named spell(s) | SPELL_IMMUNITY | 10 | 92, 100, 101, 102, 103, 104, 105, 106, 107, 128 |
| E04b | `spell.immunity_level` | own stacks immune to spells of level 1–4, **friendly ones included** | LEVEL_SPELL_IMMUNITY=4 | 1 | 134 |
| E04b | `spell.immunity_mind` | own stacks immune to mind spells (SoD bug; HotA fixed it) | MIND_IMMUNITY | 1 | 49 |
| E04b | `spell.magic_resistance` | +n % magic resistance | MAGIC_RESISTANCE | 3 | 57, 58, 59 |
| E04b | `spell.block_level_battle` | **both sides**: no spells of level 3+ (also blocks passive casts in VCMI) | BLOCK_MAGIC_ABOVE=2 BATTLE_WIDE | 1 | 83 |
| E04b | `spell.block_all_battle` | **both heroes**: no hero spellcasting | BLOCK_ALL_MAGIC BATTLE_WIDE | 1 | 126 |
| E04b | `spell.negate_resistance_battle` | **all units on the field**: natural magic resistance and immunities removed (asymmetric targeting rules) | NEGATE_ALL_NATURAL_IMMUNITIES, MAGIC_RESISTANCE≤0, SPELL_RESISTANCE_AURA≤0 | 1 | 93 |
| E04b | `spell.learn_after_battle` | +n % Eagle Eye chance (post-battle) | LEARN_BATTLE_SPELL_CHANCE | 3 | 63, 64, 65 |
| E04b | `mana.regen` | +n spell points per day | MANA_REGENERATION | 3 | 73, 74, 75 · via parts: 138 |
| E04b | `mana.regen_full` | full spell points every day | MANA_PERCENTAGE_REGENERATION=100 | 1 | 138 |
| E04c | `war_machine.catapult` | Catapult (WM4), sieges only | `warMachine` | 1 | 3 |
| E04c | `ranged.no_wall_penalty` | own shooters ignore the wall/obstacle penalty (sieges) | NO_WALL_PENALTY {SHOOTER_ONLY} | 2 | 91, 137 |
| adventure | `move.land` | +n land movement points | MOVEMENT(land) | 2 | 70, 98 |
| adventure | `move.sea` | +n sea movement points | MOVEMENT(sea) | 2 | 71, 123 · via parts: 136 |
| adventure | `move.fly` | permanent Fly | FLYING_MOVEMENT | 1 | 72 |
| adventure | `move.water_walk` | permanent Water Walk | WATER_WALKING | 1 | 90 |
| adventure | `move.boat_transfer` | no boarding/landing penalty; movement converted land↔sea | FREE_SHIP_BOARDING | 1 | 136 |
| adventure | `vision` | +1 scouting radius | SIGHT_RADIUS | 2 | 52, 53 |
| adventure | `resource.daily` | +n resource per day (stacks per copy) | GENERATE_RESOURCE (instance) | 10 | 109, 110, 111, 112, 113, 114, 115, 116, 117, 140 |
| adventure | `growth.flat` | +n weekly growth of one creature level in the town the hero is in | CREATURE_GROWTH {TOWN_AND_VISITOR} | 5 | 118, 119, 120, 121, 122 · via parts: 133 |
| adventure | `growth.percent` | +50 % growth in all the player's towns | CREATURE_GROWTH_PERCENT {PLAYER} | 1 | 133 |
| adventure | `adv.whirlpool_protection` | no troop loss in whirlpools | WHIRLPOOL_PROTECTION | 1 | 123 · via parts: 136 |
| adventure | `adv.spell` | expert Summon Boat / Scuttle Boat (adventure spells) | SPELL [≥] | 1 | 123 · via parts: 136 |
| adventure | `grail` | build the Grail structure (backpack only) | id 2 | 1 | 2 |
| none | `none` | unused traces (not equippable in VCMI) | — | 3 | 141, 142, 143 |

**Counts per scope** (an artifact with kinds in several scopes is counted in each; the last column assigns it to the earliest epic only, and sums to 144):

| scope | artifacts with ≥ 1 own effect | incl. effects inherited from parts | by first epic that must model it |
|---|---:|---:|---:|
| E04a | 59 | 61 | 61 |
| E04b | 65 | 65 | 53 |
| E04c | 3 | 3 | 1 |
| adventure | 26 | 26 | 26 |
| none | 3 | 3 | 3 |

- **E04a first (61)**: 4–18, 31–40, 45–51, 54–56, 60–62, 66–69, 84, 85, 91, 94–97, 99, 108, 125, 127–132, 134, 135, 137. Several of these also carry E04b parts and need them later: the all-four-stat artifacts 31–36, 128, 129, 132, 134, 135, Badge of Courage's mind immunity, and the opening spells of 129/132. 91 and 137 also carry the E04c wall-penalty part.
- **E04b first (53)**: 0, 1, 19–30, 41–44, 57–59, 63–65, 73–83, 86–89, 92, 93, 100–107, 124, 126, 138, 139.
- **E04c first (1)**: 3 (Catapult). **Adventure (26)**: 2, 52, 53, 70–72, 90, 98, 109–123, 133, 136, 140. **None (3)**: 141–143.

### Stacking and combination semantics (engine-relevant)

- **Identical artifacts do not stack**, except Quiet Eye of the Dragon (37) and the resource producers. Sources: wiki Artifact: "Two similar artifacts will not stack their effect except for Quiet Eye of the Dragon and all the resource-producing artifacts"; wiki Quiet Eye *(raw)*: "Equipping two Quiet Eyes of the Dragon will grant +2 Attack and +2 Defense". VCMI does the same: `bonuses` sit on the shared artifact type node and count once, while `instanceBonuses` count per copy (`docs/modders/Entities_Format/Artifact_Format.md L55–68`). Only 37 and 109–117 and 140 use `instanceBonuses`. The engine needs a per-kind "stacks per copy" flag. Duplicates are possible for rings (2 slots) and misc (5 slots), for example two Rings of Vitality = +1 HP, two Still Eyes = +1/+1, two Clovers = +1 luck.
- Different artifacts of the same kind add up, e.g. Necklace of Swiftness + Ring of the Wayfarer + Cape of Velocity = +4 speed, or three different luck charms = +3. Morale and luck are capped at ±3 (wiki Morale: "can vary from -3 to +3").
- **Combination = its own bonuses + all part bonuses.** Sources: wiki Combination artifact: "Original artifacts still give their regular bonuses and the bonuses given by combining are added to those"; VCMI `CCombinedArtifactInstance::addPart` → `attachToSource(part)` (`lib/entities/artifact/CArtifactInstance.cpp L40–51`).
- Hero primary-skill totals are clamped to Attack ≥ 0, Defense ≥ 0, Power ≥ 1, Knowledge ≥ 1 (VCMI `config/gameConfig.json L522` `minimalPrimarySkills [0,0,1,1]`, `lib/bonuses/BonusCache.cpp L99–102`). This matters for the −3/−2 relics. Not separately wiki-checked.
- Archery artifacts add to the Archery percentage and do nothing without the skill. Sources: wiki Archery: "additive with the skill … Bowstring … and Advanced Archery … 35%" and "If a hero does not have the archery skill, these artifacts have no effect"; wiki Bow of Elven Cherrywood *(raw)*; VCMI limiter `HAS_ANOTHER_BONUS_LIMITER` on `secondarySkill.archery` (`config/artifacts.json L905–926`).
- Necromancy artifacts add their percentage to the skill, need the skill, and the total is capped at 100 %. Sources: wiki Necromancy, wiki Amulet of the Undertaker *(raw)*: "If the equipped hero does not have the Necromancy secondary skill, the Amulet of the Undertaker has no effect"; VCMI `lib/mapObjects/CGHeroInstance.cpp L980–987` (needs an `IMPROVED_NECROMANCY` source, i.e. the skill or the Cloak; clamp 0–100).
- Speed artifacts do not raise the hero's land movement allowance, which comes from the slowest creature (wiki Necklace of Swiftness note). That is an adventure concern only.

## 5. Combination artifacts (SoD, ids 129–140)

| id | name | parts (ids) | parts give | the combination adds (VCMI, matches wiki unless noted) | wiki total |
|---:|---|---|---|---|---|
| 129 | Angelic Alliance | 31–36 | +21 A/D/P/K | alignment-mix morale rule (**D1**); expert Prayer, 10 rounds, own army, at battle start | +21 all; Expert Prayer 10 rds; no penalty for Castle/Rampart/Tower/Fortress/Stronghold (SoD) |
| 130 | Cloak of the Undead King | 54, 55, 56 | +30 % Necromancy (needs skill) | works without the skill (as 30 % skeletons); Basic → Walking Dead, Advanced → Wights, Expert → Liches (`IMPROVED_NECROMANCY` with required level in addInfo); upgraded form at 2/3 rate if no slot (`CGHeroInstance.cpp L1036–1050`). **D8** | +30 % Necromancy, creature by level |
| 131 | Elixir of Life | 94, 95, 96 | +4 HP all stacks | +25 % *base* HP (rounded down) and regeneration ≤ 50 HP per turn, living only | same (wiki Elixir *(raw)*, Regeneration) |
| 132 | Armor of the Damned | 8, 14, 20, 26 | +3 A, +3 D, +2 P, +2 K | expert Slow, Curse, Weakness, Misfortune on **all enemy** stacks, 50 rounds (**D2, D3**) | same |
| 133 | Statue of Legion | 118–122 | +5/+4/+3/+2/+1 growth L2–6 in visited town | +50 % growth in all the player's towns | same; needs all 5 misc slots (SoD only) |
| 134 | Power of the Dragon Father | 37–45 | +10 A/D/P/K, +1 morale, +1 luck | +6 A/D/P/K; own stacks immune to level 1–4 spells **including friendly ones** (VCMI `lib/spells/TargetCondition.cpp L229–237` does not look at the caster's side) | +16 all, +1/+1, immunity L1–4 |
| 135 | Titan's Thunder | 12, 18, 24, 30 | +9 A, +9 D, +8 P, +8 K | expert Titan's Lightning Bolt (`SPELL titanBolt=3`); assembling it in the right hand gives a permanent spell book (`ArtifactUtils.cpp L167–179`, `server/CGameHandler.cpp L3041–3042`) | same; 600 damage, no mana (original text) |
| 136 | Admiral's Hat | 71, 123 | +1500 sea movement, boats, whirlpool | no boarding/landing penalty | same |
| 137 | Bow of the Sharpshooter | 60, 61, 62 | +30 % Archery (needs skill) | no range penalty, no wall penalty, shoot while adjacent (own shooters) | same (wiki *(raw)*) |
| 138 | Wizard's Well | 73, 74, 75 | +6 mana/day | full mana each day | same |
| 139 | Ring of the Magi | 76, 77, 78 | +6 spell duration | +50 → +56 total | same |
| 140 | Cornucopia | 109, 110, 111, 113 | +1 each of crystal, gems, mercury, sulfur | +4 each → +5 total each | same |

Unused (141–143): Diplomat's Suit (Shoulders), Mired in Neutrality (Torso), Ironfist of the Ogre (Right Hand). They have class R, a blank cost and a placeholder description in the original table, and no components anywhere. VCMI keeps them as `unusedArtifact1–3` because maps may reference them, with `type: [CREATURE]`, which clears their hero slots, and class SPECIAL (`config/artifacts.json L2272–2290`). The engine should keep the ids reserved and non-equippable. **HotA changed this**: HotA's Ironfist of the Ogre and Diplomat's Cloak are new combinations.

**Assembly and disassembly** (recorded only; commands are out of scope):
- SoD, from the wiki Combination artifact page: "Combining is performed by equipping all the required artifacts, and right-clicking one of them. This will also dismantle the combination artifact back to its original pieces." The combination "reserves the same artifact slots as its original artifacts". In the backpack it takes one slot (wiki Artifact).
- VCMI: assembly validated in `server/CGameHandler.cpp L3017–3048` with `ArtifactUtils::assemblyPossibilities` (`lib/entities/artifact/ArtifactUtils.cpp L205–232`, default `onlyEquiped=false`, so parts in the backpack also count; see **D6**). The combination goes to the part's slot and the other part slots are marked `locked` (`lib/entities/artifact/CArtifactSet.cpp L132–175`). Disassembly is refused for fused artifacts (none in core) and when the backpack cannot take the parts (`CGameHandler.cpp L3050–3063`).
- Combinations never appear randomly: VCMI `legalArtifact` returns false for combined artifacts (`CArtHandler.cpp L452–457`).
- After a won battle, VCMI moves the loser's worn (not locked) and backpack artifacts to the winner, so a combination moves whole. Parts that land on equipment prompt assembly (`server/battles/BattleResultProcessor.cpp L522–560`, `askAssemble`).

## 6. Special artifacts

| id | artifact | slot | facts |
|---:|---|---|---|
| 0 | Spell Book | 17 (Spell Book) | Bought in a Mage Guild for 500 gold, or given by Titan's Thunder; "cannot be traded to another hero, nor can it be lost after a defeat" (wiki Spell Book). VCMI: unmovable slot (`ArtifactUtils.cpp L80–89`), never moved to the winner; heroes whose type has `haveSpellBook` start with it (`CGHeroInstance.cpp L400–410`). E04b |
| 1 | Spell Scroll | any misc | Cost 2500 in the table. One artifact id; the spell is per instance (VCMI: a `SPELL` bonus on the instance, `CArtifactInstance.cpp L87–94`). Wiki Artifact: scrolls "have no class", AI value 500/2000/3000/4000/5000 by spell level. E04b |
| 2 | The Grail | none (backpack only; no slot flag) | Lost for good when its carrier retreats, surrenders or is defeated (wiki Grail). VCMI never moves it to the winner (`BattleResultProcessor.cpp L556`) and erases it from a removed hero (`lib/gameState/GameStatePackVisitor.cpp L402–405`). An outcome input; otherwise adventure |
| 3 | Catapult | 16 (WM4) | Every hero has one (VCMI `CGHeroInstance.cpp L413–417`; wiki War Machine). Cannot be moved (`CGameHandler.cpp L2681–2682`). Deployed only for the attacker of a town with walls (`BattleInfo.cpp L318–319`). Not lost when destroyed (`BattleResultProcessor.cpp L80–81`). E04c |
| 4 | Ballista | 13 (WM1) | Cost 2500 |
| 5 | Ammo Cart | 14 (WM2) | Cost 1000 |
| 6 | First Aid Tent | 15 (WM3) | Cost 750 |

**How a hero brings war machines.** A war machine *is* the artifact in its slot (WM1 Ballista, WM2 Ammo Cart, WM3 First Aid Tent, WM4 Catapult). The original table allows each machine only in its own column, so a hero has at most one of each. Wiki War Machine: "Occupies the War Machine A/B/C/D slot"; bought from a town's Blacksmith or a War Machine Factory; "War Machines (except Catapults) can be traded between heroes, but cannot be dismissed or placed in the hero's backpack"; "They do not participate in combat when battling creature banks"; Catapults only in sieges.

VCMI battle setup (`lib/battle/BattleInfo.cpp L296–331`): for each side with a hero, every filled WM slot creates a war-machine stack of count 1 via the artifact's `warMachine` creature. Positions come from `config/gameConfig.json L630–631`: attacker WM1..WM4 at hexes 52, 18, 154, 120; defender at 66, 32, 168, 134. Exceptions: no Catapult unless sieging walls, and no Ballista for a defender in a town (E04c).

After the battle, a destroyed Ballista, Cart or Tent is removed from the hero; the Catapult never is (`BattleResultProcessor.cpp L72–88`). The winner never receives war machines or the spell book: transfer covers only `commonWornSlots`, i.e. Head…Misc5 (`ArtifactUtils.cpp L91–111`), plus the backpack. Big artifacts cannot go to the backpack (`CArtifact.cpp L163–166, L183–188`; `CGameHandler.cpp L2679–2680`).

VCMI quirk: the Ammo Cart's effect also applies in creature-bank battles where the cart is not on the field (`lib/CStack.cpp L308–326`). For E04a, `BattleSetup.war_machines` is derived from the hero's WM1–WM3 slots (WM4 only in E04c sieges). The outcome reports which machines died.

**Lock slots.** A combination locks the slots of its other parts (VCMI `ArtSlotInfo.locked`, `CArtifactSet::lockSlot` `CArtifactSet.cpp L278–288`). Locked slots cannot be filled, moved, or taken by the winner (`ArtifactUtils::isArtRemovable` L160–165, `CGameHandler.cpp L2676–2677`). `ArtifactID::ART_SELECTION = 144` and `ART_LOCK = 145` (`lib/constants/EntityIdentifiers.h L809–810`) are UI icon frames, not artifacts. The engine should model a lock as slot state, not as an artifact id.

**Backpack.** Wiki Artifact: holds up to 64 artifacts; items gained from defeated heroes or banks with no space "will just disappear". VCMI: unlimited by default (`config/gameConfig.json L518` `backpackSize: -1`). See **D7**.

## 7. Negative and side-wide effects (the engine must touch the other side)

| id | artifact | who is affected | effect | scope | sources |
|---:|---|---|---|---|---|
| 84 | Spirit of Oppression | **both sides**, every unit | positive morale removed (≤ 0) | E04a | original text ("both you and your opponent"); wiki Morale ("negates all positive Morale"); VCMI `MORALE=0 [≤] BATTLE_WIDE` L1231–1243 |
| 85 | Hourglass of the Evil Hour | **both sides** | positive luck removed (≤ 0) | E04a | original text; wiki *(raw)*; VCMI L1244–1256 |
| 125 | Shackles of War | **both heroes**, hero-vs-hero only | no retreat, no surrender | E04a | original text ("neither you nor your opponent"); wiki ("hero-to-hero battles only"); VCMI `BATTLE_NO_FLEEING BATTLE_WIDE` L1828–1838 + `lib/battle/CBattleInfoEssentials.cpp L310–312` (needs a hero on the other side) |
| 132 | Armor of the Damned | **all enemy stacks** | expert Slow, Curse, Weakness, Misfortune for 50 rounds (speed, damage, attack, luck down) | E04b (but changes E04a stats) | original text; wiki; VCMI L2014–2052, cast in `server/battles/BattleFlowProcessor.cpp L198–229` |
| 83 | Recanter's Cloak | **both heroes** (VCMI: all casts incl. passive) | no spells of level ≥ 3 | E04b | original text ("neither you nor your opponent"); VCMI L1218–1230, `lib/spells/BattleSpellMechanics.cpp L216–222` |
| 126 | Orb of Inhibition | **both heroes** | no hero spellcasting | E04b | wiki ("Prevents either hero from casting any spells during combat"); VCMI L1839–1849, `lib/battle/CBattleInfoCallback.cpp L148–149` (HERO mode only) |
| 93 | Orb of Vulnerability | **all units on the field** | natural magic resistance and immunities removed; wiki: the owner may target both sides, the opponent only the owner's troops | E04b | original text; wiki; VCMI L1358–1391 (two subtypes: battle-wide and enemy-hero) |
| 12, 18, 24, 30 | Titan relics | own hero (self-negative) | −3 D / −3 A / −2 P / −2 K | E04a/E04b | original text; VCMI |
| 134 | Power of the Dragon Father | own stacks (self-negative) | friendly level 1–4 spells blocked too | E04b | wiki; VCMI `TargetCondition.cpp L229–237` |

Not side-wide, though they might look it:
- Pendant of Negativity (106) only makes the **own** units immune to Lightning Bolt and Chain Lightning. Original text: "renders your units immune"; wiki: not to Titan's Lightning Bolt.
- Sphere of Permanence (92) protects the own stacks from Dispel.
- Armageddon's Blade (128) makes the own troops immune to Armageddon. Wiki: Orb of Vulnerability does not remove that immunity.

No SoD artifact lowers the enemy's morale or luck directly. **HotA changed this**: HotA adds such artifacts (Pendant of Downfall, Ring of Suppression, Hideous Mask, Runes of Imminency, Demon's Horseshoe, Shaman's Puppet, Seal of Sunset, Charm of Eclipse, Plate of Dying Light).

Consequence for E04a exit criterion 4 ("a side's hero never changes the other side's stacks except through modelled events"): items 84, 85 and 125 are E04a effects whose bearer is **both sides**. They need a battle-wide modifier source (ADR-0012 pool with a "battle" bearer, or an explicit "affects both sides" flag) and an event or observation entry so the opponent can see why its morale or luck is capped.

## 8. Disputes

| # | What | Source A | Source B | Impact | Suggested SoD experiment |
|---|---|---|---|---|---|
| **D1** | Which creatures Angelic Alliance (129) lets mix without a morale penalty | original text (artraits.txt id 129): Rampart, Tower, Fortress, Stronghold, Castle. Wiki Angelic Alliance *(raw)*, SoD branch of `{{swh}}`: same five, and "Conflux creatures, or neutral creatures" in the army prevent the bonus | VCMI: every faction whose alignment is not EVIL counts as one (`lib/mapObjects/army/CArmedInstance.cpp L86–97`). Conflux and the neutral faction are "neutral" (`config/factions/conflux.json L6`, `neutral.json L7`), so Castle + Conflux gives +1 in VCMI and 0 in SoD | E04a morale (+1 vs 0 or a penalty) | Hero with AA, army Castle + Conflux (and a variant Castle + a neutral such as Peasants): read the army morale tooltip, then one combat with morale-relevant creatures. **HotA changed this** (Conflux and HotA towns allowed) |
| **D2** | When the opening spells of Armor of the Damned (and Angelic Alliance) take effect | VCMI: all `OPENING_BATTLE_SPELL`s are cast once after tactics, attacker first (`server/battles/BattleFlowProcessor.cpp L198–229`, called L241) | wiki Armor of the Damned: only "once an allied creature is able to take its turn; if a creature misses a turn due to low morale … does not cast its spells until the next one is able to do so". The AA page only says "at the start of combat" | Faster enemy stacks may act un-slowed before the wearer's first stack; E04b, but it changes E04a turn order and stats | AotD hero with slow troops (e.g. Zombies) vs a hero with faster troops: is the first enemy stack already Slowed when it acts? Repeat with AA and Prayer |
| **D3** | Does Orb of Inhibition (126) cancel artifact opening spells? | VCMI: `BLOCK_ALL_MAGIC` is checked only for hero casting (`CBattleInfoCallback.cpp L148`), so AA and AotD casts go through; Recanter's Cloak and Cursed Ground do block them (`BattleSpellMechanics.cpp L216–222`) | wiki AA and AotD: "can be mitigated by … Cursed Ground, or anti-magic artifacts. However, it works normally in Anti-Magic Garrisons" (does not say which artifacts); Orb page: "Prevents either hero from casting" | E04b | AotD hero vs a hero wearing Orb of Inhibition; also vs Recanter's Cloak (Misfortune and Prayer are level ≥ 3) |
| **D4** | Class of Vial of Dragon Blood (127) and Armageddon's Blade (128) | original data: `R` (relic) in both copies; wiki: Relic | VCMI JSON overrides both to `SPECIAL` (`config/artifacts.json L1850–1871, L1872–1915`), which keeps them out of random generation (`CArtHandler.cpp L452–466`) | Adventure only (random relic pool, RMG). Not E04 | Map editor or random map: can a random relic become one of them? Low priority |
| **D5** | Armageddon's Blade power/knowledge | wiki `Template:Artifact table - Weapon` *(raw)*: +6 Power, +3 Knowledge | original text (+3 Power, +6 Knowledge), VCMI (`P+3, K+6`, L1872–1915) and the wiki article "Armageddon's Blade (artifact)" (+3 P, +6 K) | E04b (power), small | None needed: original data decides (P+3, K+6). Optional: equip and read the hero screen |
| **D6** | Combination assembly with parts in the backpack | wiki: all parts must be **equipped**, then right-click | VCMI: `assemblyPossibilities(..., onlyEquiped=false)` also accepts backpack parts (`ArtifactUtils.cpp L205–232`, `GameStatePackVisitor.cpp L984–1029`); a VCMI extension | None in battle; matters for later equip commands | Not needed now (the SoD UI clearly requires equipping) |
| **D7** | Backpack capacity | wiki Artifact: 64; excess loot from defeated heroes disappears | VCMI: unlimited (`config/gameConfig.json L518`) | Outcome application (artifact capture), adventure | Later epic: a defeated hero with many artifacts vs a winner with a full backpack |
| **D8** | Creature raised by Cloak of the Undead King at Basic Necromancy | original text (id 130): "Zombies" | VCMI: Walking Dead (`config/artifacts.json L1944–1977`), with the upgrade (Zombies) at 2/3 rate only when no slot is free; wiki: "Walking Dead (100%) or Zombies (66.6%)" | E04a outcome input (Necromancy) | Hero with Cloak + Basic Necromancy and free slots wins vs living creatures; record the raised type |

### Open checks (no source conflict, but unknown for SoD and VCMI has an opinion)

1. **War machines and hero stat artifacts.** In VCMI, war-machine stacks attach to the hero's army node (`lib/CStack.cpp L73–79`), so `STACK_HEALTH` (94–96, 131) and `STACKS_SPEED` (69, 97, 99) from artifacts also raise a Ballista's or Tent's HP and speed. The Elixir's 25 % and regeneration are excluded by the LIVING limiter. Wiki War Machine only says machines are "immobile and unaffected by Morale" and that the Ballista is affected by Luck. E04a impact: turn order and Ballista HP. Experiment: hero with Cape of Velocity + Ring of Vitality and a Ballista; read the Ballista's speed and HP in battle.
2. **Golden Bow / Bow of the Sharpshooter on the Ballista.** VCMI's `SHOOTER_ONLY` limiter (`lib/bonuses/Limiters.cpp L29`) matches the Ballista (it has `SHOOTER`, `config/creatures/special.json`), so the Golden Bow removes its range penalty. The wiki is silent. Experiment together with Q23 (damage at distance > 10 hexes with and without the Golden Bow).
3. **Spirit of Oppression with a Mirth spell.** Only positive morale is capped (VCMI and wiki Morale). Its interaction with morale *spells* is E04b.
4. **Armageddon's Blade's casting is E04b**; its +3 A / +3 D are E04a.

## 9. What E04a needs from this (summary for ADR-0013 and T10)

- E04a effect kinds (20): `stat.attack`, `stat.defense`, `morale`, `luck`, `morale.cap_battle`, `luck.cap_battle`, `morale.alignment_mix`, `unit.speed`, `unit.hp_flat`, `unit.hp_percent`, `unit.regeneration`, `unit.attack_defense_filtered`, `ranged.archery_bonus`, `ranged.no_range_penalty`, `ranged.shoot_adjacent`, `war_machine`, `battle.no_flee`, and the outcome inputs `outcome.surrender_discount`, `outcome.necromancy_percent`, `outcome.necromancy_creature`.
- Flags each kind needs: stacks-per-copy (only Quiet Eye in E04a), bearer (own side / both sides / enemy side), creature selector (all, living, dragons, shooters), a dependency on a secondary skill (Archery, Necromancy).
- Combination artifacts in E04a are needed for their inherited stats (129, 132, 134, 135) plus Elixir of Life (131), Bow of the Sharpshooter (137) and the Cloak (130, outcome input).
- Values to verify by observation: D1 and D8, plus open checks 1 and 2. D2 and D3 feed E04b.
