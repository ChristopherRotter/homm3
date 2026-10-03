# E04a T1 source survey: the 28 secondary skills

Version of the target: RoE + AB + SoD, GOG Complete 4.0. Survey date 2026-10-03.

## 0. Sources and conventions

| Label | What | Notes |
|---|---|---|
| `original-data` | `H3bitmap/SSTRAITS.TXT` (SoD), `H3ab_bmp/SSTraits.txt` (AB), `SkillLev.txt`, `HCTRAITS.TXT` | **The SoD and AB SSTRAITS files are byte-identical** (md5 `fe9e398f324e303429a3268abc172603`), as are both `SkillLev.txt` files. So the skill texts did not change between AB and SoD. No RoE copy is available, so RoE differences cannot be read from data. The skill numbers are only in the description texts. `SkillLev.txt` holds `Basic/Advanced/Expert` and `Bas/Adv/Exp`. |
| `vcmi` | VCMI 1.7.5 (`git describe` = `1.7.5`, commit `6901839`), a local clone | URLs: `https://github.com/vcmi/vcmi/blob/1.7.5/<path>#L<n>` |
| `wiki` | https://heroes.thelazy.net, retrieved 2026-10-03 | Fetched through a summarising fetch tool. Short quotes were checked against the raw wikitext (`?action=raw`) where noted. In `{{swh|A|B}}`, **A is the HotA value and B is the SoD value** (Template:Swh, https://heroes.thelazy.net/index.php/Template:Swh). `{{gt|..}}` only colours text green. |

- **Index order**: rows 0–27 of SSTRAITS = the `"index"` values in VCMI `config/skills.json` (lines 3, 30, 60, …, 843).
- **Internal names in `HCTRAITS.TXT`** (header row, `original-data`) differ from the display names. Armorer = "Defense", Artillery = "Battlefield Ballistics", Ballistics = "Siege Ballistics", Tactics = "Battle Tactics", Scholar = "Magic Scholar", Resistance = "Magic Resistance", Fire Magic = "School of Fire Magic" (the other schools likewise). Use these names when matching other tables.
- **Some VCMI values are offsets on base bonuses.** `config/gameConfig.json` gives every hero base values that the skill adds to (L805-L848, https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L805):
  - spell damage 100 (L809)
  - max learnable spell level 2 (L816)
  - mana regeneration 1 per day (L831)
  - experience gain 100 % (L837)
  - mana per knowledge 1000 % (L843)

  So VCMI's Mysticism value 1/2/3 means a total of 2/3/4, and its Wisdom value 1/2/3 means spell levels 3/4/5.
- **Data quirk**: the Basic Estates text says "With Eestates" in both files (typo).

## 1. Summary table

Scope tags: `E04a` = battle effect without magic, `E04b` = battle magic, `E04c` = siege, `outcome` = input to what happens after the battle, `adventure` = map or town only.

| # | Skill (VCMI id) | Scope | B / A / E (SSTRAITS) | VCMI bonus type: B / A / E (`skills.json` lines) | Wiki SoD B / A / E | Agree? |
|---|---|---|---|---|---|---|
| 0 | Pathfinding (`pathfinding`) | adventure | rough-terrain penalty −25 / −50 / −75 % | `ROUGH_TERRAIN_DISCOUNT` 25/50/75 (L2-L27) | −25/−50/−75 MP per tile, minimum 100 (HotA: minimum 95/90/85) | yes |
| 1 | Archery (`archery`) | **E04a** | ranged damage +10 / +25 / +50 % | `PERCENTAGE_DAMAGE_BOOST` `damageTypeRanged` 10/25/50 (L29-L58) | 10/25/50 % | yes |
| 2 | Logistics (`logistics`) | adventure | land MP +10 / 20 / 30 % | `MOVEMENT` `heroMovementLand` `PERCENT_TO_BASE` 10/20/30 (L59-L88) | 10/20/30 (HotA 5/10/20) | yes |
| 3 | Scouting (`scouting`) | adventure | sight +1 / 2 / 3 | `SIGHT_RADIUS` 1/2/3 (L89-L117) | 1/2/3 (HotA 1/3/5) | yes |
| 4 | Diplomacy (`diplomacy`) | **outcome** (surrender) + adventure | negotiate with weaker monsters; surrender cost −20 / 40 / 60 % | `WANDERING_CREATURES_JOIN_BONUS` 1/2/3, `SURRENDER_DISCOUNT` 20/40/60 (L118-L154) | −20/40/60 % surrender | yes on the %; **cost base disputed (D4)** |
| 5 | Navigation (`navigation`) | adventure | sea MP +50 / 100 / 150 % | `MOVEMENT` `heroMovementSea` 50/100/150 (L155-L185) | 50/100/150 % | yes |
| 6 | Leadership (`leadership`) | **E04a** | morale +1 / 2 / 3 | `MORALE` 1/2/3 (L186-L212) | +1/+2/+3 | yes |
| 7 | Wisdom (`wisdom`) | adventure (unlocks spell learning; E04b only via Eagle Eye) | learn level 3 / 4 / 5 spells | `MAX_LEARNABLE_SPELL_LEVEL` +1/2/3 over base 2 (L213-L240) | 3rd/4th/5th | yes |
| 8 | Mysticism (`mysticism`) | adventure | 2 / 3 / 4 SP per day | `MANA_REGENERATION` +1/2/3 over base 1 (L241-L269) | 2/3/4 (HotA: % of max SP) | yes |
| 9 | Luck (`luck`) | **E04a** | luck +1 / 2 / 3 | `LUCK` 1/2/3 (L270-L296) | +1/+2/+3 | yes |
| 10 | Ballistics (`ballistics`) | **E04c** | controls the Catapult; aimed shots; Adv: two shots; Exp: two shots at maximum damage | `CATAPULT_EXTRA_SHOTS` 1/2/3 (used as the `catapultShot` spell level) + `MANUAL_CONTROL` catapult 100 (L297-L330) | see §2.9 | yes |
| 11 | Eagle Eye (`eagleEye`) | **outcome** (E04b) | 40 / 50 / 60 % chance to learn spells of level ≤ 2 / 3 / 4 | `LEARN_BATTLE_SPELL_CHANCE` 40/50/60, `LEARN_BATTLE_SPELL_LEVEL_LIMIT` 2/3/4 (L331-L366) | same (HotA 30/40/50 %, levels ≤ 3/4/5) | yes; Wisdom check disputed (D6) |
| 12 | Necromancy (`necromancy`) | **outcome** | 10 / 20 / 30 % of the killed raised as Skeletons | `UNDEAD_RAISE_PERCENTAGE` 10/20/30, `IMPROVED_NECROMANCY` skeleton 1/2/3 (L367-L403) | 10/20/30 (HotA 5/10/15) | yes on the %; **algorithm disputed (D5)** |
| 13 | Estates (`estates`) | adventure | 125 / 250 / 500 gold per day | `GENERATE_RESOURCE` gold 125/250/500 (L404-L433) | 125/250/500 (HotA 250/500/1000) | yes |
| 14–17 | Fire / Air / Water / Earth Magic (`fireMagic` …) | **E04b** | reduced cost; Adv "increased", Exp "maximum" effectiveness | `MAGIC_SCHOOL_SKILL` `spellSchool.x` 1/2/3 (L434-L549) | text only; numbers are in the spell tables | yes |
| 18 | Scholar (`scholar`) | adventure | teach spells of level ≤ 2 / 3 / 4 | `LEARN_MEETING_SPELL_LIMIT` 2/3/4 (L550-L576) | same | yes |
| 19 | Tactics (`tactics`) | **E04a** | placement within 3 / 5 / 7 hex rows | `BEFORE_BATTLE_REPOSITION` 2/4/6 + `BEFORE_BATTLE_REPOSITION_BLOCK` 2/4/6 (L577-L610) | 3/5/7; negates 1/2/3 levels of the enemy's Tactics | yes |
| 20 | Artillery (`artillery`) | **E04a** (arrow-tower control: E04c) | controls the Ballista; "two shots" at **every** level; double damage 50 / 75 / 100 % | `BONUS_DAMAGE_CHANCE` ballista 50/75/100, `HERO_GRANTS_ATTACKS` ballista **0/1/1**, `MANUAL_CONTROL` ballista + arrowTower 100, `BONUS_DAMAGE_PERCENTAGE` ballista 100 (L611-L664) | Basic: 50 % double, **one shot**; Adv: two shots, 75 %; Exp: two shots, 100 % | **dispute D1 (shots at Basic)** |
| 21 | Learning (`learning`) | **outcome** | experience +5 / 10 / 15 % | `HERO_EXPERIENCE_GAIN_PERCENT` `PERCENT_TO_BASE` 5/10/15 over base 100 (L665-L693) | 5/10/15 (HotA 20/40/60 plus extra levels) | yes |
| 22 | Offense (`offence`) | **E04a** | melee damage +10 / 20 / 30 % | `PERCENTAGE_DAMAGE_BOOST` `damageTypeMelee` 10/20/30 (L694-L723) | 10/20/30 | yes |
| 23 | Armorer (`armorer`) | **E04a** | all damage to the hero's troops −5 / 10 / 15 % | `GENERAL_DAMAGE_REDUCTION` `damageTypeAll` 5/10/15 (L724-L753) | 5/10/15 | yes on the %; **rounding quirk (D3)** |
| 24 | Intelligence (`intelligence`) | adventure (max SP; battle reads current SP, E04b) | max SP +25 / 50 / 100 % | `MANA_PER_KNOWLEDGE_PERCENTAGE` `PERCENT_TO_BASE` 25/50/100 over base 1000 (L754-L782) | 25/50/100 (HotA 20/35/50) | yes |
| 25 | Sorcery (`sorcery`) | **E04b** | spell damage +5 / 10 / 15 % | `SPELL_DAMAGE` `spellSchool.any` 5/10/15 over base 100 (L783-L812) | 5/10/15 (HotA 10/20/30) | yes |
| 26 | Resistance (`resistance`) | **E04b** | troops' magic resistance 5 / 10 / 20 % | `MAGIC_RESISTANCE` 5/10/20 (L813-L841) | 5/10/20 | yes |
| 27 | First Aid (`firstAid`) | **E04a** | controls the tent; heals 50 / 75 / 100 HP of the first unit of the selected stack | `SPECIFIC_SPELL_POWER` `spell.firstAid` 50/75/100 + `MANUAL_CONTROL` firstAidTent 100 (L842-L877) | **random 1–50 / 1–75 / 1–100** | **dispute D2** |

All VCMI rows: `config/skills.json`, https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L1. Wiki rows: the page with the skill's name, e.g. https://heroes.thelazy.net/index.php/Pathfinding. Luck is at https://heroes.thelazy.net/index.php/Luck_(secondary_skill).

**Specialties**: per-level boosts exist for Archery, Offense, Armorer, Necromancy, Eagle Eye, Learning, Logistics, Navigation, Mysticism, Estates, Intelligence, Sorcery, Resistance and First Aid. The Ballista and Catapult specialties are creature specialties. Details: see the specialties survey. VCMI's `"specialty": ["main"]` arrays in `skills.json` mark which skills have a generic one. The wiki formula is "5 % bonus per level to the skill", which enters the damage formula as I3 / R3 (§3).

## 2. Battle-relevant skills: mechanics

### 2.1 Archery (#1, E04a)

- **original-data**: SSTRAITS row 1: "increases the damage done by range attacking creatures by 10% / 25% / 50%".
- **vcmi**:
  - `skills.json` L29-L58: `PERCENTAGE_DAMAGE_BOOST`, subtype `damageTypeRanged`, `BASE_NUMBER` 10/25/50.
  - Applied in `lib/battle/DamageCalculator.cpp` L233-L245 (https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/DamageCalculator.cpp#L233) only when `info.shooting`, as `val/100`. It is added to the attack-factor sum (L566-L570).
- **wiki** (Archery, https://heroes.thelazy.net/index.php/Archery):
  - "increases the base damage of ranged attacks of both ranged creatures and war machines (the ballista and cannon[HotA])".
  - "Bug: The archery skill reduces damage done by arrow towers (fixed in HotA)".
  - The artifacts (+5/+10/+15 %, and +30 % for Bow of the Sharpshooter) are "additive with the skill" and have no effect without the skill. VCMI models the same: the artifact bonus has a `HAS_ANOTHER_BONUS_LIMITER` on the Archery skill (`config/artifacts.json` L905-L922).
- **Damage formula**: I2 term (§3), **additive** with the attack bonus, luck and the others.
- **Applies to**: every shot of the hero's creatures (including area shots), and the **Ballista**.
  - wiki (War Machine, https://heroes.thelazy.net/index.php/War_Machine): "Both the Artillery and Archery secondary skills increase the damage of the Ballista."
  - vcmi: war machines attach to the hero's army node and so inherit all hero bonuses (`lib/CStack.cpp` L69-L79, https://github.com/vcmi/vcmi/blob/1.7.5/lib/CStack.cpp#L69).
- **Arrow towers** (E04c): the wiki's bug makes Archery *reduce* tower damage. VCMI has no such bug (D7).

### 2.2 Offense (#22, E04a)

- **original-data**: "increases all hand-to-hand damage inflicted by the hero's troops by 10% / 20% / 30%".
- **vcmi**: `skills.json` L694-L723, `PERCENTAGE_DAMAGE_BOOST` `damageTypeMelee` 10/20/30. `DamageCalculator.cpp` L242-L244 applies it whenever `!info.shooting`, which includes retaliations and a shooter forced into melee.
- **wiki** (Offense, https://heroes.thelazy.net/index.php/Offense): "increases the base damage of hand-to-hand attacks of creatures".
- **wiki** (Damage): "Because creatures cannot perform ranged and melee attacks at the same time, Archery and Offense modifiers cannot affect damage simultaneously". This implies that shooters striking in melee get Offense, which agrees with VCMI.
- **Special cases**: no source lists an exception. The Ballista never melees, so Offense is irrelevant to it.

### 2.3 Armorer (#23, E04a)

- **original-data**: "reduces all damage inflicted against the hero's troops by 5% / 10% / 15%".
- **vcmi**:
  - `skills.json` L724-L753: `GENERAL_DAMAGE_REDUCTION` `damageTypeAll` 5/10/15.
  - `DamageCalculator.cpp` L339-L345: it reads the **defender's** bonuses that are not of `SPELL_EFFECT` source. It is one multiplicative defence factor `(1 − r)` (L572-L576), separate from Shield (L347-L360) and petrification (L443-L450).
  - The Armorer specialty adds to the same bonus, so skill and specialty sum inside one factor.
- **wiki** (Armorer, https://heroes.thelazy.net/index.php/Armorer): "Armorer only reduces physical damage dealt to the hero's troops in melee or ranged combat; Armorer does not reduce spell damage".
- **wiki** (Damage, https://heroes.thelazy.net/index.php/Damage, section on Armorer's side effects):
  - "First, heroes with Armorer take extra damage from arrow towers (fixed in HotA)".
  - "Second, damage is reduced by 1 if creatures from a hero with Armorer take an amount of damage that is exactly an integer value."
  - Example: 100 Peasants attack Peasants led by a hero with Basic Armorer, ADD 0: "damage is not 100 × 1 × (1 − 0.05) = 95, but 94". 99 Peasants: "94.05 … rounded off in the usual way, that is, to 94".
  - Worked example: 2002 → 2001.
- **Damage formula**: the wiki factor is `(1 − R2 − R3)`: skill and specialty are added, then this factor multiplies the others.
- **Against the Ballista's shots**: yes. The Ballista's shot is a normal ranged strike, and Armorer reduces "melee or ranged combat" damage (wiki). VCMI applies the defender's factor regardless of the attacker.
- **Protecting the hero's own war machines**: VCMI yes (inheritance, `CStack.cpp` L73-L78). The wiki is silent, so this is unverified (low impact).
- **Is the −1 quirk floating-point noise?** Checked by computation: no. Both float64 and float32 give 100 × (1 − 0.05) = 95.0 exactly. It is therefore systematic, not an artefact of floating point. Two hypotheses fit the wiki's two data points:
  - (A) Wiki: after the final floor, subtract 1 if the exact product is an integer.
  - (B) Armorer reduces by `floor(x·r) + 1` at its own step. This is the same pattern VCMI uses for the Behemoth, `floor(p·D) + 1`, see Q19.

  The two give the same result whenever Armorer is the last factor applied. They differ only when range or melee penalties, Shield and so on come after it. See D3.

### 2.4 Leadership (#6) and Luck (#9), E04a

- **original-data**: "increases your hero's troops' morale by 1/2/3" and "increases your hero's luck by 1/2/3". `HCTRAITS.TXT`: the Leadership probability is 0 for Death Knight and Necromancer. The wiki also says "Leadership is completely unavailable for Necropolis heroes".
- **vcmi**:
  - `MORALE` and `LUCK` `BASE_NUMBER` 1/2/3 on the hero, inherited by all its units.
  - The sum is clamped to ±(length of the chance vector) = ±3 (`lib/BasicTypes.cpp` L81-L127, https://github.com/vcmi/vcmi/blob/1.7.5/lib/BasicTypes.cpp#L81).
  - Units that are `NON_LIVING`, `MECHANICAL`, `UNDEAD`, `SIEGE_WEAPON` or `NO_MORALE` ignore morale (L72-L76). Nothing in that code excludes war machines from luck.
  - Chances (`config/gameConfig.json` L589-L599): good morale 1/2/3 of 24, bad morale 2/4/6 of 24, good luck 1/2/3 of 24, bad luck disabled (`[]`).
  - A lucky strike adds +1.0 to the attack-factor sum (`DamageCalculator.cpp` L247-L252).
- **wiki**:
  - Luck (secondary skill), https://heroes.thelazy.net/index.php/Luck_(secondary_skill): "Maximum positive luck is +3".
  - Luck, https://heroes.thelazy.net/index.php/Luck: chances 1/24, 1/12, 1/8. A lucky strike deals "double total base damage" (SoD); negative luck only exists in HotA. The Luck (secondary skill) page also says bad-luck attacks are "present in the original game's code, but do not normally work".
  - War Machine: "Ballistas and Cannons are affected by Luck", and "War Machines are immobile and unaffected by Morale".
- **Damage formula**: luck is I4 = +1.00 in the bonus sum in SoD. HotA instead multiplies the final damage by 2 (L1).
- **Already in the engine**: E03 rules for morale and luck, Q17 (no luck without a hero) and Q18 (luck only on the primary target). T5 only adds the hero's +1/+2/+3 as side modifiers.
- **VCMI deviation not to port**: VCMI biases its luck, morale and ability rolls (`abilityBias` 25, `moraleBias` 8, `luckBias` 8; `config/gameConfig.json` L585-L602; `lib/callback/GameRandomizer.cpp` L28-L44). Successive rolls are therefore not independent. The H3 sources describe plain independent chances. This also affects Artillery's double-damage roll (`rollCombatAbility`).

### 2.5 Tactics (#19, E04a)

- **original-data**: "rearrange your troops just before combat, within three / five / seven hex rows of the commanding hero".
- **wiki** (Tactics, https://heroes.thelazy.net/index.php/Tactics):
  - "within 3 / 5 / 7 rows … It negates one / two / three levels of an enemy hero's Tactics skill."
  - Same level: "Cancelling each other out, neither hero may arrange their army". Advanced vs Basic: 2 − 1 = 1, so the Advanced hero acts as Basic. Expert vs Basic: acts as Advanced. Expert vs Advanced: acts as Basic. The weaker hero gets nothing.
- **vcmi**:
  - Rule: `lib/battle/BattleInfo.cpp` L397-L437 (https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L397).
    - `diff = own BEFORE_BATTLE_REPOSITION (2/4/6) − enemy BEFORE_BATTLE_REPOSITION_BLOCK (2/4/6)`.
    - The side with `diff > 0` gets `tacticDistance = 1 + diff`, so Basic 3, Advanced 5, Expert 7, Expert vs Basic 5, Expert vs Advanced 3.
    - Both sides positive "will not happen" in H3; VCMI would favour the attacker.
    - The hero's own reposition value only counts if `tacticFormationEnabled` (a hero-window toggle, default true). The block value always counts.
  - Allowed hexes: `lib/battle/CBattleInfoCallback.cpp` L1666-L1679 (https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L1666). The attacker may place on columns `1..=dist`; the defender on `BFIELD_WIDTH − dist − 1 ..= BFIELD_WIDTH − 2`. Column 0 and column 16 are the off-field edge columns of VCMI's 17-wide grid, so these are the first 3/5/7 columns of the 15 playable ones.
  - Creature-bank layouts set `tacticsAllowed: false` (`config/gameConfig.json` L670-L681).
- **All three sources agree** (the wiki's formula and VCMI's are equivalent). The open design point (who moves and with which commands) is ADR-0013 question 7.

### 2.6 Artillery (#20, E04a)

| Aspect | original-data (SSTRAITS) | wiki (Artillery, https://heroes.thelazy.net/index.php/Artillery; War Machine) | vcmi |
|---|---|---|---|
| Control | "gives control of the ballista to the hero" | "gives the hero control of the Ballista, Arrow Towers, and the Cannon[HotA]". Trivia: "The ability to control the Arrow Towers was added in Armageddon's Blade." | `MANUAL_CONTROL` ballista and arrowTower, value 100 (`skills.json` L626-L637). The roll at activation always succeeds (`server/battles/BattleFlowProcessor.cpp` L463, https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleFlowProcessor.cpp#L463). |
| Shots per turn | "allowing two shots" at **Basic, Advanced and Expert** | Basic: no mention of two shots. Advanced and Expert: "The Ballista shoots twice". | `HERO_GRANTS_ATTACKS` **0 / 1 / 1** (L646-L662), added to the ranged attack count (`server/battles/BattleActionProcessor.cpp` L441-L456, https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L441) |
| Double damage | 50 % / 75 % / "inflict double damage" (always) | "50% / 75% / 100% chance to inflict 200% base Damage" | `BONUS_DAMAGE_CHANCE` 50/75/100, rolled for **each strike** (`BattleActionProcessor.cpp` L1076-L1081). If it succeeds, `BONUS_DAMAGE_PERCENTAGE` 100 adds +1.0 to the attack-factor sum (`DamageCalculator.cpp` L261-L269). |
| Ballista base damage | n/a | "(2–3)×(hero's attack + 1)" in SoD; HotA uses "+5". Raw text: `{{swh|(2–3)*(hero's attack + 5)|(2–3)*(hero's attack + 1)}}`. | min and max × (hero attack + 1) (`DamageCalculator.cpp` L64-L75, https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/DamageCalculator.cpp#L64). **Only** attack from `HERO_BASE_SKILL` and `ARTIFACT` sources counts. The Ballista's own attack in the attack/defence difference is its creature attack plus all inherited hero attack. |
| Ammunition | n/a | "Contrary to what its in-game statistics indicate, a Ballista has unlimited shots (fixed in HotA)" | 24 shots, unlimited only with an Ammo Cart (`lib/battle/CUnitState.cpp` L92-L95; `lib/CStack.cpp` L308-L326). Side finding: D8. |

- **Mechanics to model**:
  - The hero picks the Ballista's target.
  - Number of shots: 1 + 0/1/1 (D1).
  - Each strike rolls the double-damage chance on its own, plus luck. Both are additive I terms: lucky plus double = base × (1 + I1 + I2 + 1 + 1 …).
  - Archery applies (I2).
  - Without Artillery the E03 automatic behaviour stays (BAT-WAR-002).
- **Other levels**: Ballista specialists and the chance to start with a Ballista: see specialties. Arrow-tower control: E04c.

### 2.7 First Aid (#27, E04a)

| Aspect | original-data | wiki (First Aid, https://heroes.thelazy.net/index.php/First_Aid; War Machine) | vcmi |
|---|---|---|---|
| Control | "gives control of the first aid tent" | control of the tent | `MANUAL_CONTROL` firstAidTent 100 (`skills.json` L854-L859). Automatic otherwise: a random allied stack that can be healed (`BattleFlowProcessor.cpp` L705-L738). |
| Heal | "healing 50 / 75 / 100 points of damage to the first unit of the selected stack" | SoD part of `swh`: "healing **1-50** / **1-75** / **1-100** points of damage to the first unit of the selected stack" (HotA: 40–50 / 60–75 / 80–100 plus a health bonus) | Fixed 50 / 75 / 100. The heal action casts `firstAid` at level 0 (`BattleActionProcessor.cpp` L579). The effect value = count × `SPECIFIC_SPELL_POWER` inherited from the hero (`lib/battle/CUnitState.cpp` L452-L455). Without the skill it falls back to the spell's level-0 power of **10** (`lib/spells/ISpellMechanics.cpp` L320-L323; `config/spells/vcmiAbility.json` L52-L104). |
| Resurrect | not mentioned ("damage to the first unit") | War Machine: "cannot resurrect killed creatures". The tent heals the "top creature of an allied stack". | `healLevel: "heal"`, so living units only (`lib/spells/effects/Heal.cpp` L67-L69); no war machines (`vcmiAbility.json` target condition) |
| Without the skill | n/a | 1–25 random (SoD) | 10 fixed. Already decided as Q26: follow the wiki. |

### 2.8 Necromancy (#12, outcome, outline)

- **original-data**: "10% / 20% / 30% of the creatures killed in combat … brought back … as Skeletons". `HCTRAITS`: probability 10 for Death Knight and Necromancer, 0 for all other classes.
- **wiki** (Necromancy, https://heroes.thelazy.net/index.php/Necromancy):
  - Three limits: "the total number of raised Skeletons can never exceed the number of creatures slain", "the total health of raised Skeletons can never exceed the total health of killed creatures", and the skill used "can never be more than 100%".
  - "These rules in SoD are applied to every killed stack individually (in HotA … by the overall number of killed creatures), and always rounded towards 1."
  - Example: "50% necromancy … 100 peasants … only 8 skeletons".
  - "All killed enemy creatures can be raised … This includes sacrificed creatures, cloned creatures, summoned elementals and even war machines."
  - If there is no free slot but there are Skeleton Warriors: raised "as Skeleton Warriors, but only 66.6% compared to Skeletons".
  - Necromancy Amplifier: +10 % in SoD. The Cloak of the Undead King changes the raised type.
  - Bug: AI heroes always raise exactly 1 (fixed in HotA).
- **vcmi**:
  - Code: `lib/mapObjects/CGHeroInstance.cpp` L978-L1072 (https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapObjects/CGHeroInstance.cpp#L978).
  - Percentage = clamp(`UNDEAD_RAISE_PERCENTAGE`, 0, 100).
  - Per **creature type**: `min(HP_killed / HP_raised, 1) × count × pct / 100`. This is ×2/3 if raised as the upgrade because no slot is free. The total is summed in double, truncated once, minimum 1.
  - Applied for the winner only (`server/battles/BattleResultProcessor.cpp` L507-L512).
  - Casualties exclude summoned stacks (VCMI clones are summoned) and turrets (L671-L687).
- **Disputes**: per stack vs aggregate rounding, and the summoned/cloned exclusion (D5). Seam input needed: the enemy's killed creatures **per stack**, with type and count, including summons, clones and war machines.

### 2.9 Ballistics (#10, E04c, outline)

- **original-data**: Basic: catapult control, "aimed shots with increased damage". Advanced: "two aimed shots with increased damage". Expert: "two aimed shots at maximum damage".
- **wiki** (Ballistics, https://heroes.thelazy.net/index.php/Ballistics), SoD values (HotA raises the Expert chances to 100 % and Drawbridge Basic/Advanced to 60 %):

| | None | Basic | Adv | Expert |
|---|---|---|---|---|
| Shots | 1 | 1 | 2 | 2 |
| Keep | 5 % | 7 % | 7 % | 10 % |
| Tower | 10 % | 15 % | 15 % | 20 % |
| Drawbridge | 25 % | 30 % | 30 % | 40 % |
| Intended wall | 50 % | 60 % | 60 % | 75 % |
| 0 / 1 / 2 damage | 10 / 60 / 30 % | 0 / 50 / 50 % | 0 / 50 / 50 % | 0 / 0 / 100 % |

- **vcmi**: identical numbers in `config/spells/vcmiAbility.json` L107-L180 (`catapultShot` levels none/basic/advanced/expert, `targetsToAttack` 1/1/2/2). The skill's `CATAPULT_EXTRA_SHOTS` 1/2/3 becomes the cast level (`BattleActionProcessor.cpp` L482-L483).

### 2.10 Resistance (#26, E04b, outline)

- All three sources: 5 / 10 / 20 %.
- **wiki**: negated by the Orb of Vulnerability. Resistance artifacts work "with or without the resistance skill".
- **vcmi**: all `MAGIC_RESISTANCE` bonuses (skill, creature, artifact) are **summed** and capped at 100 (`lib/BasicTypes.cpp` L41-L46). The Unicorn aura is then combined multiplicatively (`lib/CStack.cpp` L92-L107).
- **Open**: the wiki does not say whether skill and creature resistance add or multiply (Magic resistance and Dwarf pages checked). Open for E04b, no dispute yet.

### 2.11 Eagle Eye (#11, outcome/E04b, outline)

- **original-data**: 40 % for spell levels 1–2, 50 % up to level 3, 60 % up to level 4, for spells "used in combat".
- **wiki** (Eagle Eye, https://heroes.thelazy.net/index.php/Eagle_Eye): "Spells learned by Eagle Eye are learned after the fight is won [SoD]. For a spell to be learned, the hero must have the appropriate level of Wisdom". Artifacts add +5/+10/+15 %.
- **vcmi**: `server/battles/BattleResultProcessor.cpp` L466-L484 (https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L466). The winning hero, if it has units left, gets one roll per distinct spell the **enemy side** used: `level ≤ limit`, not already in the book, `nextInt(99) < chance`. There is **no Wisdom check and no spell-book check** (D6).
- **Seam input**: the list of spells the enemy cast (E04b).

### 2.12 Diplomacy (#4): surrender (outcome)

- **original-data and wiki**: the surrender cost is reduced by 20 / 40 / 60 %.
- **wiki** (Surrender, https://heroes.thelazy.net/index.php/Surrender):
  - "The amount of gold required to surrender is equal to **half** the recruitment cost of the remaining creatures in the hero's army, with any resources other than gold being excluded".
  - Surrender is hero vs hero only. It is impossible when defending a siege or with the Shackles of War. In SoD the enemy may refuse; in HotA it cannot.
- **vcmi** (`lib/battle/CBattleInfoCallback.cpp` L2347-L2369, https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L2347; `lib/battle/Unit.cpp` L238-L245):
  - Formula: Σ **full** gold recruit cost × count, over living units that come from army slots (no war machines, no summons), then `× (100 − discount) / 100` truncated.
  - Discount: Diplomacy 20/40/60, plus Statesman's Medal, Diplomat's Ring and Ambassador's Sash at +10 each, all added (`config/artifacts.json` L1007-L1037).
- **Disputes**: full vs half cost (D4). The epic leaves the cost to the orchestrator, but the seam must expose the surviving army and the discount.
- **Joining monsters**: adventure only (VCMI `WANDERING_CREATURES_JOIN_BONUS` 1/2/3).

### 2.13 Learning (#21, outcome)

- All three sources: +5 / 10 / 15 % experience. The wiki's HotA values (20/40/60 % plus extra levels) are inside `swh` arg 1.
- **vcmi**: `exp × HERO_EXPERIENCE_GAIN_PERCENT / 100`, truncated (`lib/mapObjects/CGHeroInstance.cpp` L782-L785). The base 100 comes from `gameConfig.json` L837-L841 and Learning adds 5/10/15 % of the base. The raw experience = Σ max HP × killed for enemy casualties (`BattleResultProcessor.cpp` L212-L224). This is outside this survey's scope; it belongs to the experience survey.

## 3. Damage formula: how Archery, Offense and Armorer enter

**wiki** (Damage, https://heroes.thelazy.net/index.php/Damage), SoD:

```
DMGf = DMGb × (1+I1+I2+I3+I4+I5) × (1−R1) × (1−R2−R3) × (1−R4) × (1−R5) × (1−R6) × (1−R7) × (1−R8)
```

| Term | Meaning (wiki) | VCMI function (`DamageCalculator.cpp`) |
|---|---|---|
| I1 | 0.05 × (A − D) if A > D, cap +300 % (Q11) | `getAttackSkillFactor` L210-L224, cap 4.0 (`gameConfig.json` L606-L612). Not followed (Q11). |
| I2 | Archery 0.10/0.25/0.50 **or** Offense 0.10/0.20/0.30 | `getAttackOffenseArcheryFactor` L233-L245 (specialty included in the same bonus) |
| I3 | specialty: 0.05 × I2 × hero level (also Adela's Bless) | inside I2's bonus value in VCMI |
| I4 | lucky strike +1.00 (SoD) | `getAttackLuckFactor` L247-L252 |
| I5 | death blow, **Ballista double +1.00**, hate +0.50, jousting, elementals | `getAttackDeathBlowFactor`, `getAttackDoubleDamageFactor` L261-L269, hate, jousting … |
| R1 | 0.025 × (D − A), cap 0.70 | `getDefenseSkillFactor` L322-L337 |
| R2 + R3 | **Armorer 0.05/0.10/0.15 + specialty 0.05 × R2 × level, summed into one factor** | `getDefenseArmorerFactor` L339-L345 (one bonus type, so summed) |
| R4 | Shield / Air Shield | `getDefenseMagicShieldFactor` L347-L360 |
| R5 | range penalty or melee penalty for shooters, 0.50 | `getDefenseRangePenaltiesFactor` L362-L392 |
| R6 | wall penalty, 0.50 (siege) | `getDefenseObstacleFactor` L394-L406 |
| R7 | blind / paralysis retaliation, Forgetfulness | `getDefenseBlindParalysisFactor`, `getDefenseForgetfulnessFactor` (separate factors in VCMI) |
| R8 | petrified, Psychic/Magic Elemental cases | `getDefensePetrificationFactor`, `getDefenseMagicFactor`, `getDefenseMindFactor` (separate factors in VCMI) |

**VCMI structure** (`DamageCalculator.cpp` L556-L593, https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/DamageCalculator.cpp#L556):

```
attackFactorTotal  = 1.0 + Σ attackFactors          // L563-570, all additive
defenseFactorTotal = Π (1 - min(1.0, factor))       // L572-576, all multiplicative
dmin/dmax = max(1, floor(base × attackFactorTotal × defenseFactorTotal))   // L578-583, double precision
```

The base damage is the stack's rolled damage before these factors. Bless and Curse change it first (L79-L131). For the Ballista it is multiplied by (hero attack + 1) before that (L64-L75).

**Consequences for the engine** (exact integers, one final floor, BAT-DMG-002):

1. Archery and Offense are **additive** percentages inside the bonus sum, together with I1, luck, the Ballista double, hate and jousting. They never apply together.
2. Armorer is **one multiplicative** reduction factor, `(100 − armorer% − armorer_specialty%)`. It is not combined with Shield, the penalties or R1.
3. **Rounding**: the wiki floors the final damage ("rounded off in the usual way … to 94"). VCMI floors a double product. In exact rational arithmetic the order of the factors does not matter.
4. **Precision of the specialty terms** I3 and R3: e.g. Orrin at level 7 with Expert Archery has I3 = 0.05 × 7 × 0.50 = 17.5 %. These terms need units of 1/20 % (or a rational), not whole percents. This is a heads-up for the specialties and pack format.
5. **Armorer quirk** (D3): with Armorer active the original deals one less than the exact floor when the exact value is an integer (wiki). VCMI and our engine both give 95 in the Peasants case. This is a candidate quirk entry, and it belongs with Q23.
6. **The defender's Armorer applies to every physical strike** against the hero's units, including Ballista shots and retaliations. It does not apply to spells. Arrow towers (E04c) are affected inversely by a bug (D7).

## 4. Applicability matrix (question 5)

| Effect | Creature melee | Creature shot | Ballista shot | Arrow tower shot (E04c) | Retaliation | Sources |
|---|---|---|---|---|---|---|
| Offense (attacker's hero) | yes | no | no (never melees) | no | yes | wiki Offense/Damage; vcmi `DamageCalculator.cpp` L233-L245 |
| Offense for a shooter in melee | yes (with the 50 % melee penalty) | – | – | – | yes | wiki Damage (implied); vcmi |
| Archery (attacker's hero) | no | yes | **yes** | wiki: bug, *reduces* tower damage; vcmi: normal bonus if the towers' army has the skill | no | wiki Archery, War Machine; vcmi `CStack.cpp` L69-L79 |
| Armorer (defender's hero) | yes | yes | **yes** | wiki: bug, *increases* damage; vcmi: normal reduction | yes | wiki Armorer/Damage; vcmi L339-L345 |
| Armorer protecting war machines | vcmi yes (inheritance); wiki silent | | | | | low impact, unverified |
| Artillery double damage | – | – | yes, per strike | not in VCMI (only the Ballista subtype) | – | vcmi `BattleActionProcessor.cpp` L1076-L1081 |
| Luck | yes | yes | yes ("Ballistas … are affected by Luck") | ? (E04c) | yes | wiki War Machine; E03 rules |
| Morale | yes (living, non-undead) | yes | no | no | – | wiki War Machine; vcmi `BasicTypes.cpp` L72-L76 |

## 5. RoE / AB / SoD differences

- The SSTRAITS texts are identical in AB and SoD (byte-identical files), so no skill-text change from AB to SoD.
- The wiki's Artillery trivia says arrow-tower control was added in AB. So under RoE rules Artillery gives Ballista control only.
- No source names any other RoE-era skill difference. The RoE text table is not available locally (Q15: historical balance at the latest patches).
- Everything the wiki marks HotA (Scouting 1/3/5, Logistics 5/10/20, Estates 250/500/1000, Intelligence 20/35/50, Sorcery 10/20/30, Necromancy 5/10/15, Learning 20/40/60, the Eagle Eye rework, the First Aid rework, the Ballistics Expert hit chances, Ballista +5 and unlimited-shots fix, Mysticism %, luck as an outer multiplier, Resistance replaced by Interference) is **out of scope**: HotA changed it.

## 6. Disputes

| # | What | Source A | Source B | Gameplay impact | Suggested experiment (SoD) |
|---|---|---|---|---|---|
| **D1** | Artillery: Ballista shots at Basic | original-data: "allowing two shots" at every level | wiki: Basic has no "shoots twice", Advanced and Expert do. vcmi: `HERO_GRANTS_ATTACKS` 0/1/1 (`skills.json` L646-L662). | Basic Artillery Ballista damage per turn ×2 vs ×1 | Hero with Basic Artillery and a Ballista vs a large neutral stack; count Ballista shots in the combat log over 2–3 turns (1 vs 2 per turn). Control: Advanced → 2. |
| **D2** | First Aid heal with the skill: fixed or random | original-data: "healing 50 / 75 / 100 points"; vcmi: fixed 50/75/100 | wiki (SoD branch of `swh`): random **1–50 / 1–75 / 1–100** | Expected heal halves; changes RL value of the tent | Basic First Aid hero; heal a top creature missing ≥ 60 HP (e.g. a wounded Angel) about 10 times; varying amounts → wiki, always 50 → VCMI or text. Pairs with Q26 (tent without the skill, already following the wiki). |
| **D3** | Armorer rounding: one less when the exact value is an integer | wiki (Damage): 100 Peasants vs Basic Armorer, ADD 0 → **94**; 99 Peasants → 94; worked example 2002 → 2001 | vcmi: `floor(100 × 0.95)` = **95** in double; our exact-integer BAT-DMG-002 also gives 95 | −1 damage in exact cases (frequent with small fixed-damage stacks and round numbers); also decides hypothesis A or B (§2.3); relates to Q23 | Both heroes with attack 0 and defense 0. 100 Peasants (attacker) strike Peasants of a hero with Basic Armorer: 94 vs 95. Controls: 99 Peasants → 94 for all; 20 Peasants → 18 (wiki) vs 19. To separate A from B, add a later factor, e.g. a shooter at long range; with random damage, log many shots and compare the predicted value sets. |
| **D4** | Surrender cost base | wiki (Surrender): **half** the gold recruit cost of the remaining creatures | vcmi: **full** gold recruit cost × count (`Unit.cpp` L238-L245, `CBattleInfoCallback.cpp` L2357-L2367) | 2× cost; affects retreat/surrender decisions of the RL agent and the outcome | Surrender with a known army, e.g. 10 Pikemen at 60 gold: 300 (wiki) vs 600 (VCMI); with Basic Diplomacy 240 vs 480. Also check rounding with an odd total. |
| **D5** | Necromancy raising algorithm | wiki: per **killed stack**, "always rounded towards 1"; includes sacrificed, cloned and summoned creatures and war machines | vcmi: aggregated per creature **type**, summed in double, truncated once, minimum 1; summoned stacks (incl. clones) and turrets excluded (`BattleResultProcessor.cpp` L671-L687; `CGHeroInstance.cpp` L1057-L1071) | Number of Skeletons raised (outcome) | Basic Necromancy vs two separate stacks of 5 Peasants (both killed). Per stack: 5 × 1/6 × 10 % → each rounds "towards 1" → 2 Skeletons. VCMI: 0.17 total → 1. Second test: kill an enemy's summoned elemental and check whether it adds Skeletons. |
| **D6** | Eagle Eye prerequisites | wiki: needs the appropriate level of **Wisdom**; learned after the fight is won | vcmi: checks only the level limit and "not already in the book"; no Wisdom or spell-book check (`BattleResultProcessor.cpp` L469-L483) | Outcome only (E04b input) | Hero with Expert Eagle Eye and no Wisdom wins against a hero who cast a level-3 spell; repeat about 10 times (60 % chance each). Never learned → Wisdom required. |
| **D7** | Arrow towers vs Archery and Armorer (E04c) | wiki: Archery *reduces* and Armorer (and Air Shield) *increases* arrow-tower damage, "as if the sign … would be plus instead of minus" (bug, fixed in HotA) | vcmi: normal Archery bonus and Armorer reduction | Siege damage | Defer to E04c; quirk-survey candidate (`fix-optional`?). |
| **D8** (side finding, war machines) | Ballista ammunition | wiki (War Machine): "Contrary to what its in-game statistics indicate, a Ballista has unlimited shots" | vcmi and our E03 BAT-WAR-002: 24 shots, unlimited only with an Ammo Cart | Long battles with Artillery | Let a Ballista without an Ammo Cart fire more than 24 shots (about 13 turns with Advanced Artillery against a high-HP target). |
| (decided) | I1 cap | wiki +300 % | vcmi 4.0 (+400 %) | – | Already Q11: wiki followed. |
| (decided) | Tent without First Aid | wiki 1–25 random | vcmi fixed 10 | – | Already Q26: wiki followed, experiment planned. |

**Not disputes, but open or VCMI-only points to keep in mind:**
- VCMI's biased RNG for luck, morale and abilities (`abilityBias`); do not port.
- Whether the hero's attack counts both in the Ballista multiplier and in I1. VCMI does both; the wiki only states the multiplier. Experiment: compare Ballista damage against targets with the same base damage but different defense.
- How Resistance combines with creature resistance (E04b).
- VCMI lets the attacker win a double-Tactics tie, which cannot happen in H3.
