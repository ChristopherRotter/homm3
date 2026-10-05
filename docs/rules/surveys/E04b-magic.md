# E04b T1 source survey: battle magic (what ADR-0014 decides)

Date: 2026-10-05. Scope: just in time (OPEN-QUESTIONS Q41): the mechanics ADR-0014 must decide, with labelled sources. **Per-spell numbers are not collected here**; each task surveys its own spells. Earlier surveys already cover Sorcery, Resistance, Eagle Eye and Intelligence (`E04-skills.md` §1, §2.10, §2.11), the outline of spell effect classes and siege inputs (`E04-battle.md` §7), the spell specialty formula (`E04-heroes.md` §3) and the quirks S-09 to S-21 (`E04-quirks.md`).

Sources: `original-data` = the user's SoD installation (`docs/formats/installation.md` §7.6: the spell table's flag bits, decoded in this task); `vcmi` = VCMI 1.7.5 (paths relative to https://github.com/vcmi/vcmi/blob/1.7.5/); `wiki` = https://heroes.thelazy.net/index.php/<Page>, fetched 2026-10-05 through a summarising tool (short quotes only, so a rule that rests on a wiki fact re-checks the page); `manual` = the RoE manual pages on the wiki.

## 1. The spell table

- 81 rows: 10 adventure (0–9), 60 combat (10–69; Earthquake 14 goes to E04c), 11 creature abilities (70–80). The AB and SoD copies of `SPTRAITS.TXT` differ only in row 57 (AB "Fear (disabled)", SoD Titan's Lightning Bolt); RoE's table is not in the installation (Q15) (`original-data`).
- The executable's flag word gives per spell: combat/adventure, ability, timed, single target, mass at Expert, hex/area target, obstacle target, damage, mind spell, and AI categories (`installation.md` §7.6). These are the spell flags the BACKLOG asked for; pack format v4 can harvest them instead of gap-filling targeting and the mind flag.
- Creature flags already harvested that magic reads: `mind_immune`, `fire_immune`, `undead`, `alive`, `war_machine`, `dragon` (`packs/README.md`).

## 2. Casting by a hero

- **Spell book** required (`manual` p. 55: "Before they can cast spells, heroes must possess a spell book"; `vcmi` `lib/battle/CBattleInfoCallback.cpp` L117–L159: no book, `BLOCK_ALL_MAGIC`, cast limit, tactics phase).
- **Once per round, on an own creature's turn:** `wiki` Combat: "A hero may cast a spell only right after activation of one of their creatures, just once per round." `vcmi`: a per-side counter reset in `BattleInfo::nextRound` (`lib/battle/BattleInfo.cpp` L651–L670); the action is accepted while one of the side's units is active (`server/battles/BattleActionProcessor.cpp` L1705–L1748); the creature still acts afterwards. **Not in the tactics phase** (`vcmi` L135). Open: may a hero cast during the turn of a war machine it commands (Ballista with Artillery, First Aid Tent)? (experiment candidate X-3).
- **Mastery** for a spell: the best of its schools' levels, raised by "any school" and per-spell bonuses (Tomes, terrains in E04c), at most Expert (`vcmi` `lib/mapObjects/CGHeroInstance.cpp` L792–L813). It selects the cost, the effect value, the area and the mass version.
- **Cost:** the row's cost at that mastery, minus the largest reduction among the caster's living stacks (Mage, Arch Mage: 2), plus the largest increase among the enemy's (Pegasus, Silver Pegasus: 2), at least 0 (`vcmi` `CBattleInfoCallback.cpp` L2111–L2136). **Mana channelling:** a living Familiar on the other side gives that side's hero 20 % of the cost (`vcmi` `lib/spells/BattleSpellMechanics.cpp` L371–L382). Wraiths drain the enemy hero's mana each turn (`vcmi` `MANA_DRAIN` 2). Creature casts cost no mana.
- **Mana after the battle:** spent mana stays spent; gains are capped (`E04-battle.md` §7). The outcome needs each hero's mana left.
- **Effect value:** `power × power factor + effect[mastery]` (`vcmi` `lib/spells/CSpellHandler.cpp` L421–L424), then per target and caster modifiers (§4). **Duration** of timed effects: spell power plus duration bonuses (Ring of Conjuring and others; `vcmi` `CGHeroInstance.cpp` L855–L862; `wiki` Primary skill: "1 point of Power skill equals 1 round of duration").
- **Eagle Eye** needs the distinct spells the enemy hero cast (`vcmi` `usedSpellsHistory`; `E04-skills.md` §2.11, dispute D6).
- **Magic blocked or limited** by artifacts (Orb of Inhibition, Recanter's Cloak) and battlefields (Cursed Ground; E04c): a per-battle spell-level limit and a "no magic" flag (`E04-battle.md` §7).

## 3. Effect kinds and targeting

| Kind (`vcmi` effect class) | Spells | Targeting |
|---|---|---|
| damage | Magic Arrow, Ice Bolt, Lightning Bolt, Titan's Lightning Bolt, Implosion; Chain Lightning; Frost Ring, Fireball, Inferno, Meteor Shower; Death Ripple, Destroy Undead, Armageddon; Thunderbolt, Death Cloud, Acid Breath (abilities) | one stack; a chain from one stack; a hex with a ring or radius; every stack with a condition |
| timed (buff/debuff) | the 30 or so spells with flag `0x4`, several abilities | one stack; every own or enemy stack at Expert (flag `0x40`); Berserk an area by mastery |
| dispel | Dispel, Cure (with heal), Anti-Magic (with immunity), Dispel Helpful (ability) | one stack; mass at Expert |
| heal / raise | Cure, Resurrection, Animate Dead, Sacrifice | one stack, dead stacks for raising; Sacrifice takes **two** targets (the dead stack and the victim) |
| summon | the four Elementals; Pit Lords' demons (ability) | none (placed by rule) / a dead stack |
| clone | Clone | one own stack |
| teleport | Teleport | **two** targets: a stack and a destination hex (`vcmi` `lib/spells/effects/Teleport.cpp` L53–L60) |
| obstacle | Quicksand, Land Mine (random hexes), Force Field, Fire Wall (a hex) | none or a hex |
| remove obstacle | Remove Obstacle (also part of Dispel in VCMI) | an obstacle |

- **Area shapes** (`vcmi` `config/spells/offensive.json`): Frost Ring a ring of radius 1 without its centre; Fireball and Meteor Shower radius 1; Inferno radius 2; Berserk radius 0/0/1/2 by mastery; Death Ripple, Destroy Undead, Armageddon every stack that meets their condition. Mass spells at Expert affect only the side the spell is good or bad for ("smart" targets).
- **Counters:** casting one spell of a pair removes the other: Bless/Curse, Bloodlust/Weakness, Haste/Slow, Mirth/Sorrow, Fortune/Misfortune, Berserk/Hypnotize (`vcmi` `config/spells/timed.json`).
- **Durations end on rounds or events** (`vcmi` `timed.json`, `ability.json`): rounds for most; until the stack is attacked (Blind, Stone Gaze, Paralyze), until its own attack (Berserk), until it attacks (Frenzy), permanent (Disrupting Ray, Acid Breath; Bind until the binding ends, see S-14). VCMI counts every stack's rounds down at the start of a new round (`BattleInfo.cpp` L651–L670). Re-casting a spell replaces its effect, except Disrupting Ray and Acid Breath, which stack (`vcmi` `lib/spells/effects/Timed.cpp` L202–L205). The original keeps one duration per spell and stack with 8-bit counters (quirk S-17: effects "expire after 255 rounds").
- **Encoding consequence:** a cast is spell + first target (stack, hex, obstacle or none) + an optional second target (Teleport, Sacrifice). Creature casts (Faerie Dragon, Archangel, Pit Lord, Ogre Mage, the AB elementals) replace the creature's action; hero casts do not.

## 4. Resolution against a stack

Order in `vcmi` (`BattleSpellMechanics.cpp` L339–L541, `CSpellHandler.cpp` L378–L419, `CGHeroInstance.cpp` L815–L853):

1. **Valid target** (legality, before the cast): conditions per spell (undead, living, shooter, mind spells and `mind_immune`); level immunity (Green, Red, Azure Dragons ≤ 3; Gold Dragon ≤ 4; Black Dragon and Magic Elemental all); school immunity (fire: Fire and Energy Elementals, Efreet, Firebird, Phoenix); per-spell immunity (elementals, Troglodytes against Blind and Stone Gaze). VCMI checks "absolute" immunities first; the Orb of Vulnerability (or a receptive target) skips the "normal" ones (`vcmi` `lib/spells/TargetCondition.cpp` L306–L330, L450–L485). An area spell skips immune stacks.
2. **Magic resistance** roll per affected stack, for negative magical spells: Dwarves 20 %, Battle Dwarves 40 %, Crystal Dragon 20 %, the Resistance skill, artifacts; VCMI sums them (cap 100) and combines the Unicorns' adjacent aura multiplicatively (`lib/CStack.cpp` L92–L108). `wiki` Resistance: it "works against all spellcasters including neutral creatures". Open (no conflict): add or multiply in SoD (`E04-skills.md` §2.10).
3. **Magic Mirror:** a negative hero spell aimed at one stack is redirected with the mirror's chance to a random stack of the caster's side, which may then be immune or resist (`vcmi` L514–L541; `wiki` Magic Mirror: 20/30/40 %, "redirected to a random enemy troop"; area spells and creature spells are never reflected).
4. **The effect.** Damage per target: protection by school (one school only), the Golems' reduction (50/75/85/95 %), double damage to elementals of the opposed element, then the caster's bonuses (Sorcery, spell specialty, Orbs) with truncation after each factor (`vcmi`). Rounding in SoD unconfirmed (Q-0005 precedent; experiment X-1).
5. Events: resisted, reflected and affected stacks are reported (`vcmi` `BattleSpellCast`).

## 5. Stacks created in battle

| Source | Rule | Sources |
|---|---|---|
| Summon Elemental | power × 2/3/4 creatures (Basic/Advanced/Expert); on the caster's side; one elemental type per battle (HotA removed it); gone after the battle | `wiki` Summon Water Elemental; `vcmi` `lib/spells/effects/Summon.cpp` L41–L200 |
| Clone | creature level ≤ 5/6/7; one clone per stack at a time; lasts power rounds; removed by any damage or when the original dies | `wiki` Clone; `vcmi` `Clone.cpp` L29–L115; the HD+ setting changes Clone (the user plays plain HD) |
| Resurrection | temporary at Basic, permanent at Advanced and Expert; never above the stack's starting count | `wiki` Resurrection; `vcmi` `other.json` |
| Animate Dead, Sacrifice | permanent; undead only / living only | `vcmi` `other.json` |
| Pit Lord demons | from a dead allied stack, by its hit points | `vcmi` `config/creatures/inferno.json` |
| Phoenix rebirth | not a spell, but a raised stack | `vcmi` `REBIRTH` (not modelled yet) |

- `wiki` Summon Water Elemental: "If you killed all enemies but lost all your army besides the summoned elementals, both sides lose the battle." (T6, with S-20.)
- A per-side limit on stacks in the original is not documented (wiki searches found none); VCMI has none beyond a free hex. Our `STACK_CAPACITY` (2 × (16 + 3)) needs room for summons and clones.

## 6. Hidden battlefield objects

- Quicksand and Land Mine are hidden from the enemy; the caster's side sees them; a triggered Quicksand pit is revealed to all; a side with a stack fighting on its native terrain sees them, and its natives cross mines safely (`wiki` Quicksand, Land Mine; `vcmi` `lib/battle/CObstacleInstance.cpp` L152–L162, `nativeVisible`). Visibility therefore changes during a battle (natives die, pits trigger).
- `wiki` Land Mine: it cannot be cast while the enemy has native-terrain stacks that are not immune; the caster's troops cross mines safely; flying creatures ignore Quicksand; Magic Elementals are still caught by it. A mine lasts until touched; Quicksand lasts the whole battle and is removed only by Expert Remove Obstacle or Expert Dispel.
- Force Field and Fire Wall are visible (`E04-battle.md` §7).

## 7. Creature magic (for T7; `vcmi` `config/creatures/*.json`)

- **After an attack** (spell rows 70–80 and hero spells): Stone Gaze (Medusas, Basilisks), Paralyze (Scorpicore), Poison (Wyvern Monarch), Disease (Zombie), Age (Ghost Dragon), Curse (Black and Dread Knights, Mummy), Blind (Unicorns), Weakness and Dispel Helpful (Dragon Flies; Serpent Fly only Dispel), Bind (Dendroids), Thunderbolt (Thunderbird), Acid Breath (Rust Dragon). Death Cloud and Death Stare are already E03 abilities.
- **Casters:** Faerie Dragon (damage spells at random weights), Archangel (Resurrection), Pit Lord (demons), Ogre Mage (Bloodlust), Storm/Ice/Energy/Magma Elementals (protections), Master Genie (a random beneficial spell), Enchanter (casts a mass spell on its own; VCMI keeps a per-side cooldown, `BattleInfo.cpp` L656); `casts` per battle is in the packs.
- **Defences and auras:** resistances and immunities (§4), Magic Mirror 20 % (Faerie Dragon), the Unicorn aura, Golem reductions; **mana:** Mage, Pegasus, Familiar, Wraith (§2). Not magic but missing in our `Ability` set: Phoenix rebirth, Azure Dragon fear.

## 8. Quirks by task

T2: S-18 (Magic Arrow 0 damage to golems). T3: S-16, S-17, S-19. T4: S-09, S-10, S-11, S-13. T6: S-15, S-20 (exploit fix), S-21. T7: S-12, S-14 (exploit fix). No new quirk found; the summon "both sides lose" rule (§5) is a rule, not a quirk.

## 9. Disputes and open points

| # | Question | Sources | Needed by |
|---|---|---|---|
| M-1 | Which spells affect war machines | the executable's bit `0x1000` sits on Implosion, Death Ripple, raising, mind spells, Teleport, Clone and the abilities; VCMI also excludes war machines from Bless, Curse, Bloodlust, Haste, Slow, Counterstrike and others | T2 (Implosion), T3 |
| M-2 | Magic resistance: add or multiply sources | VCMI adds; the wiki is silent (`E04-skills.md` §2.10) | T2d |
| M-3 | When durations count down | VCMI: all stacks at the start of a round; SoD unobserved | T3 |
| M-4 | Hero cast during a commanded war machine's turn | wiki ("one of their creatures") vs VCMI (any own unit) | T2b |
| M-5 | Per-side stack limit | not documented | T6 |
| M-6 | Spell damage rounding | VCMI truncates after each factor; Q-0005 used single precision for physical damage | T2b |

## 10. Experiment candidates for batch 2a (end of increment 1)

X-1 rounding of spell damage with Sorcery, an Orb and a specialist on a known stack, and Magic Arrow on Gold Golems (S-18); X-2 immunities and double damage (Ice Bolt on Fire Elementals, a level 4 spell on a Gold Dragon); X-3 a hero cast during a commanded Ballista's turn (M-4); X-4 spell costs with Mages, Pegasi and a Familiar; X-5 Implosion on a Ballista (M-1); X-6 Chain Lightning's order and falloff (T2d). Plus the E04a leftovers in `docs/experiments/README.md` (Q40 among them). The experiment spec must gain spells and mana (the `.h3m` writer already writes a hero's spells).
