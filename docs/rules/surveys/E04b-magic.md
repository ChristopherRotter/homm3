# E04b T1 source survey: battle magic (what ADR-0014 decides)

Date: 2026-10-05. Scope: just in time (OPEN-QUESTIONS Q41): the mechanics ADR-0014 must decide, with labelled sources. **Per-spell numbers are not collected here**; each task surveys its own spells. Earlier surveys already cover Sorcery, Resistance, Eagle Eye and Intelligence (`E04-skills.md` §1, §2.10, §2.11), the outline of spell effect classes and siege inputs (`E04-battle.md` §7), the spell specialty formula (`E04-heroes.md` §3) and the quirks S-09 to S-21 (`E04-quirks.md`).

Sources: `original-data` = the user's SoD installation (`docs/formats/installation.md` §7.6: the spell table's flag bits, decoded in this task); `vcmi` = VCMI 1.7.5 (paths relative to https://github.com/vcmi/vcmi/blob/1.7.5/); `wiki` = https://heroes.thelazy.net/index.php/<Page>, fetched 2026-10-05 through a summarising tool (short quotes only, so a rule that rests on a wiki fact re-checks the page); `manual` = the RoE manual pages on the wiki.

## 1. The spell table

- 81 rows: 10 adventure (0–9), 60 combat (10–69; Earthquake 14 goes to E04c), 11 creature abilities (70–80). The AB and SoD copies of `SPTRAITS.TXT` differ only in row 57 (AB "Fear (disabled)", SoD Titan's Lightning Bolt); RoE's table is not in the installation (Q15) (`original-data`).
- The executable's flag word gives per spell: combat/adventure, ability, timed, single target, mass at Expert, hex/area target, obstacle target, damage, mind spell, and AI categories (`installation.md` §7.6). These are the spell flags the BACKLOG asked for; pack format v4 can harvest them instead of gap-filling targeting and the mind flag.
- Creature flags already harvested that magic reads: `mind_immune`, `fire_immune`, `undead`, `alive`, `war_machine`, `dragon` (`packs/README.md`).

## 2. Casting by a hero

- **Spell book** required (`manual` p. 55: "Before they can cast spells, heroes must possess a spell book"; `vcmi` `lib/battle/CBattleInfoCallback.cpp` L117–L159: no book, `BLOCK_ALL_MAGIC`, cast limit, tactics phase).
- **At most once per round, on an own creature's turn, and never required** (the side may always act without casting; user, 2026-10-05): `wiki` Combat: "A hero may cast a spell only right after activation of one of their creatures, just once per round." `vcmi`: a per-side counter reset in `BattleInfo::nextRound` (`lib/battle/BattleInfo.cpp` L651–L670); the action is accepted while one of the side's units is active (`server/battles/BattleActionProcessor.cpp` L1705–L1748); the creature still acts afterwards. **Not in the tactics phase** (`vcmi` L135). Open: may a hero cast during the turn of a war machine it commands (Ballista with Artillery, First Aid Tent)? (experiment candidate X-3).
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
- **Counters:** casting one spell of a pair replaces the other's effect with its own, so the later spell wins (Bless, then Curse leaves the stack cursed, not neutral; user, 2026-10-05): Bless/Curse, Bloodlust/Weakness, Haste/Slow, Mirth/Sorrow, Fortune/Misfortune, Berserk/Hypnotize (`vcmi` `config/spells/timed.json`). **Superseded in T3a** (§14, Q47): the executable cancels only Bless/Curse and Haste/Slow (Berserk/Hypnotize: T4).
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
- **Stack limit** (`original-data`, code reading for Q42, `installation.md` §7.7): the original keeps 21 stack records per side; Clone and Summon Elemental are refused while the caster's side has 20 or more stacks (with a message), Summon Elemental also when the side already summoned another elemental type. Raising reuses the dead stack's record. VCMI has no limit beyond a free hex; the wiki pages read do not mention one.

## 6. Hidden battlefield objects

- Quicksand and Land Mine are hidden from the enemy; the caster's side sees them; a triggered Quicksand pit is revealed to all; a side with a stack fighting on its native terrain sees them, and its natives cross mines safely (`wiki` Quicksand, Land Mine; `vcmi` `lib/battle/CObstacleInstance.cpp` L152–L162, `nativeVisible`). Visibility therefore changes during a battle (natives die, pits trigger). *(E04b T5c1: the original fixes the natives' sight at the battle's start and gives it to an obstacle when it is placed, `installation.md` §7.16; VCMI's living natives are the fix of BAT-QUIRK-1.)*
- `wiki` Land Mine: it cannot be cast while the enemy has native-terrain stacks that are not immune; the caster's troops cross mines safely; flying creatures ignore Quicksand; Magic Elementals are still caught by it. A mine lasts until touched; Quicksand lasts the whole battle and is removed only by Expert Remove Obstacle or Expert Dispel.
- Force Field and Fire Wall are visible (`E04-battle.md` §7).

## 7. Creature magic (for T7; `vcmi` `config/creatures/*.json`)

- **After an attack** (spell rows 70–80 and hero spells): Stone Gaze (Medusas, Basilisks), Paralyze (Scorpicore), Poison (Wyvern Monarch), Disease (Zombie), Age (Ghost Dragon), Curse (Black and Dread Knights, Mummy), Blind (Unicorns), Weakness and Dispel Helpful (Dragon Flies; Serpent Fly only Dispel), Bind (Dendroids), Thunderbolt (Thunderbird), Acid Breath (Rust Dragon). Death Cloud and Death Stare are already E03 abilities.
- **Casters:** Faerie Dragon (damage spells at random weights), Archangel (Resurrection), Pit Lord (demons), Ogre Mage (Bloodlust), Storm/Ice/Energy/Magma Elementals (protections), Master Genie (a random beneficial spell), Enchanter (casts a mass spell on its own; VCMI keeps a per-side cooldown, `BattleInfo.cpp` L656); `casts` per battle is in the packs.
- **Defences and auras:** resistances and immunities (§4), Magic Mirror 20 % (Faerie Dragon), the Unicorn aura, Golem reductions; **mana:** Mage, Pegasus, Familiar, Wraith (§2). Not magic but missing in our `Ability` set: Phoenix rebirth, Azure Dragon fear.
  *(T7c2a: the Faerie Dragon's mirror and the Efreet Sultan's 20 % Fire Shield as `innate_spell`,
  the Psychic and Magic Elementals' half damage as `damage_penalty`, BAT-MAG-057, 058, BAT-DMG-010;
  the Phoenix's rebirth is T7c2b.)* *(T7c2b, T7c3: the Phoenix's rebirth as `rebirth`, BAT-MAG-059;
  the Azure Dragon's Fear as `fear`, BAT-MAG-060.)*

## 8. Quirks by task

T2: S-18 (Magic Arrow 0 damage to golems). T3: S-16, S-17, S-19. T4: S-09, S-10, S-11, S-13. T6: S-15, S-20 (exploit fix), S-21. T7: S-12, S-14 (exploit fix). No new quirk found; the summon "both sides lose" rule (§5) is a rule, not a quirk.

## 9. Disputes and open points

| # | Question | Sources | Needed by |
|---|---|---|---|
| M-1 | Which spells affect war machines (**settled** by code reading, §12: flag `0x1000`) | the executable's bit `0x1000` sits on Implosion, Death Ripple, Bloodlust, Haste, Slow, Counterstrike, raising, the control spells, Teleport, Clone and the abilities; VCMI also excludes war machines from Bless, Curse, Mirth, Sorrow and Frenzy | T2 (Implosion), T3 |
| M-2 | Magic resistance: add or multiply sources (**settled**, §12: hero and creature add, the aura multiplies) | VCMI adds; the wiki is silent (`E04-skills.md` §2.10) | T2d |
| M-3 | When durations count down (**settled** by code reading, §14: every stack at the start of a round) | VCMI: all stacks at the start of a round; SoD unobserved | T3 |
| M-4 | Hero cast during a commanded war machine's turn | wiki ("one of their creatures") vs VCMI (any own unit) | T2b |
| M-5 | Per-side stack limit | settled by code reading (§5; `installation.md` §7.7, §7.19): 21 records, Clone and summons refused at 20 stacks in use, dead ones included; a new created stack takes a dead created stack's record (E04b T6b, BAT-MAG-037) | T6 |
| M-6 | Spell damage rounding (**settled**, §12) | VCMI truncates after each factor; Q-0005 used single precision for physical damage | T2b |

## 10. Experiment candidates for batch 2a (end of increment 1)

X-1 rounding of spell damage with Sorcery, an Orb and a specialist on a known stack, and Magic Arrow on Gold Golems (S-18); X-2 immunities and double damage (Ice Bolt on Fire Elementals, a level 4 spell on a Gold Dragon); X-3 a hero cast during a commanded Ballista's turn (M-4); X-4 spell costs with Mages, Pegasi and a Familiar; X-5 Implosion on a Ballista (M-1); X-6 Chain Lightning's order and falloff (T2d). Plus the E04a leftovers in `docs/experiments/README.md` (Q40 among them). The experiment spec must gain spells and mana (the `.h3m` writer already writes a hero's spells).

## 11. T2a: the damage spells' targeting and the hero's magic (E04b T2a, 2026-10-05)

- **Targeting** (pack format v4): the executable's flag word gives the 13 damage spells' target
  (`0x10` one stack, `0x80` a hex, neither: every stack) and the damage kind (`0x200`). Areas,
  Chain Lightning and the conditions come from `vcmi` `config/spells/offensive.json` L94–L360:
  Frost Ring the hexes at distance 1, Fireball and Meteor Shower distance 0–1, Inferno 0–2; Chain
  Lightning 4, 4, 5, 5 stacks with half the previous damage each; Implosion and Death Ripple spare
  war machines (`SIEGE_WEAPON`), Death Ripple spares undead, Destroy Undead needs undead. The
  game's own spell descriptions (`SPTRAITS.TXT`, `original-data`) agree: "adjacent to the target
  location, without inflicting damage on the target", "the target and any adjacent units",
  "Strikes four units" (five at Advanced and Expert), "half damage", "all non-undead units". The
  flag `0x1000` sits on Implosion and Death Ripple among the damage spells, as VCMI's war-machine
  exclusion (dispute M-1 stays open for the timed spells; X-5 checks Implosion on a Ballista).
- **Spell specialists** (`wiki` Hero specialty, SoD argument of `{{swh}}`, the template shows
  `{{{2}}}` without HotA): `C * (1 + 0.03 * L / n)`, `L` the hero level, `n` the target's
  creature level; HotA uses 10 % and 5 % for Meteor Shower, Chain Lightning, Resurrection and
  Animate Dead. Fixed bonuses: Ciele +50 % Magic Arrow (`wiki` Ciele, no version difference),
  Luna +100 % Fire Wall (HotA 25 %). `vcmi` agrees (`SPECIAL_SPELL_LEV` 3 × level;
  `SPECIFIC_SPELL_DAMAGE` 50 and 100) and divides the level term by the creature level in
  integers before the factor (`CGHeroInstance.cpp` L833–L834); SoD's rounding is X-1.
- **Sorcery** 5/10/15 % from the executable (`installation.md` §7.4), a Sorcery specialist grows
  it by 5 % per level; **the Orbs** +50 % for their school (`wiki` Orb of Tempestuous Fire: "+50%
  damage to fire spells"; `vcmi` `SPELL_DAMAGE` per school, the best of a spell's schools counts).
- **Magic school skills** set the school's mastery to their level (`wiki` Air Magic; `vcmi`
  `MAGIC_SCHOOL_SKILL` 1/2/3, `config/skills.json` L434–L550).

**T2b (2026-10-05).** Single-target damage spells reach only enemies: `vcmi`
`targetModifier.smart` on Magic Arrow, Ice Bolt, Lightning Bolt, Implosion, Chain Lightning and
Titan's Lightning Bolt (`lib/spells/effects/UnitEffect.cpp` L288–L293: a smart spell reaches the
side its polarity disfavours), not on the area and every-stack spells. The executable gives Chain
Lightning neutral polarity (it can strike own stacks), so its harvested side is any; T2d settles
its first target with X-6. M-4 is implemented as VCMI (a hero may cast while a war machine it
commands awaits a command; X-3 checks); M-6 as VCMI's order with exact basis points (X-1 checks).

## 12. T2d: code reading of the damage spells, immunities and resistance (2026-10-06)

The executable answered most of T2d's open points (`original-data`, `docs/formats/installation.md`
§7.8; Q29, insight only). Where it differs from VCMI, the original wins:

- **Damage order** (settles M-6 and X-1's rounding): the caster's bonuses come *before* the
  target's reductions: value → × 1.5 for an Orb of one of the spell's schools → × (1 + Sorcery),
  kept fractional → truncate → + the specialty bonus (`ceil(3 % × (hero level div creature
  level) × value)`, Ciele `value div 2`, Luna `value`) → the creature's double damage or Golem
  reduction (integer) → protection spells (T3). VCMI applies the reductions first and truncates
  after every factor; BAT-MAG-004 is corrected accordingly.
- **Immunities and resistance** (settles M-1 and M-2): spell flag `0x1000` spares war machines;
  the per-creature lists match VCMI except that Titan's Lightning Bolt joins the Earth/Magma
  Elementals' immunity and the Air/Storm Elementals' double damage; the hero's resistance
  (skill, Resistance specialty, three artifacts) is *subtracted* from the creature's effect
  factor (VCMI adds percentages: the same), the Unicorn aura multiplies by 0.8 once; one roll
  `1..100` per stack; single-precision values shift 5 % and 10 % hero resistance by one point
  (quirk Q-0005).
- **Chain Lightning** (most of X-6): the first target may be any stack (neutral polarity); if it
  resists, the spell ends; each jump halves the value before the caster's bonuses (integer), so
  each target gets its own specialty and reduction; the next target is the nearest stack not yet
  hit by screen distance (hex centres 44 × 42 pixels, a two-hex stack's centre shifted half a hex
  towards its second hex; plus a sprite offset not modelled, OPEN-QUESTIONS Q43), ties to the
  defender's stacks first, then by stack record; a candidate that fails its resistance roll is
  skipped for that jump only. The wiki's random tie-break (`wiki` Chain Lightning: "If two or
  more targets are at the equal distance from previous target, then the stack is randomly
  chosen") is not in the code. Batch 2a checks the geometry (user, 2026-10-06).
- **User decisions** (2026-10-06): the specialty effect carries the per-level rate and the hero
  level (contract change); Chain Lightning by screen geometry without the sprite offset, checked
  in batch 2a; magic resistance in single precision under Q-0005.
- Settled by batch 2a and the re-read in T2e (§13): an area spell may target a hex where it
  affects no stack. S-18 (Magic Arrow 0 damage to Golems) needs a value below
  20 (Diamond) or 7 (Gold) before the reduction, which a hero's cast cannot produce (power ≥ 1 gives
  at least 20); it remains for creature casters (T7).

## 13. Batch 2a results and the targeting code (E04b T2e, 2026-10-06)

`experiments/results/E04b-batch-2a.md` (SoD 4.0 Complete, plain HD) against the spec's
predictions (`experiments/specs/E04b-batch-2a.toml`):

- **Confirmed:** Chain Lightning by screen distance and the code's tie order (R1: A, C, B, then the
  caster's Pikemen; R2: A, R, L; every count as predicted; Q44, X-6); the damage order and
  rounding (R4: 119, 115, 17, 230; R5: 173, 25; X-1, X-2); immunities (Gold Dragon, Efreeti,
  Implosion on a Ballista; X-2, X-5); a hero casts in its commanded Ballista's turn (R6; X-3,
  M-4); Familiars return 20 % of the enemy hero's cost, rounded down (R6: 1 of 8; for T7); speed
  ties by army slot (R7e).
- **Refuted, fixed in T2e:** Chain Lightning cannot be aimed at the caster's own stacks (R1); an
  area spell may be aimed at an empty hex or an immune stack's hex, which is spared (R3b, R3c);
  the stack on an area spell's centre does not stop the spell by resisting (R3: the Gnolls beside
  the Battle Dwarves were struck by all six Fireballs, the Dwarves resisted three, no log line);
  the tactics hand-over (R7b, R7c; OPEN-QUESTIONS Q40).
- **Why the first code reading was wrong:** the aimed stack's roll at 0x5A02EA applies only to a
  spell that takes a target stack (0x59E060), never to an area spell; and the targeting code
  (0x59F700, 0x5A39C0) had not been read. Both are in `installation.md` §7.8 now, with the flag
  `0x20` (Forgetfulness: mass from Advanced, quirk S-10) settled in §7.6.
- **Open for T7** (X-4, M-?): costs. Red's Magic Arrow cost 5 in rounds 1 and 2 (Mages and the
  enemy's Pegasus alive: VCMI 5 − 2 + 2) and 5 again in round 3 after the Pegasus died (VCMI: 3);
  Blue's Ice Bolt cost 8. Either the Pegasus' surcharge outlives it (fixed at the start of the
  battle?) or the Mages' discount does not apply; T7 reads the cost code before implementing it.
  *(Settled in T7c1: the cost reads the armies, so the dead Pegasus still counts, BAT-MAG-054,
  BAT-QUIRK-15; the Familiars' 1 of 8 is BAT-MAG-055.)*
- **Noted for later:** the combat log names the caster's extra damage and the Golems' reduction
  (R5, the user): `SpellDamage` now carries the value, the caster's damage and the damage dealt.
  The Sorcery specialist's growth in single precision generalises to every skill specialty
  (OPEN-QUESTIONS Q45).

## 14. T3a: the timed value spells (E04b T3a, 2026-10-06)

The executable's code (`docs/formats/installation.md` §7.9) decides the rules; VCMI 1.7.5
(`config/spells/timed.json`, `lib/battle/DamageCalculator.cpp`) agrees on the values and
differs where noted. Values by mastery (none, Basic, Advanced, Expert) from `SPTRAITS.TXT`.

| Spell | Effect while active | Values |
|---|---|---|
| Haste | speed + v (not war machines) | 3, 3, 5, 5 |
| Slow | speed × v %, rounded down, at least 1 | 75, 75, 50, 50 |
| Bloodlust / Precision | attack + v on melee / ranged attacks | 3, 3, 6, 6 |
| Weakness | attack − v, at most down to 0 (the amount taken is given back) | 3, 3, 6, 6 |
| Stone Skin | defense + v | 3, 3, 6, 6 |
| Disrupting Ray | defense − v at every cast, at most down to 0, for the rest of the battle | 3, 3, 4, 5 |
| Prayer | attack, defense + v; speed + v except war machines | 2, 2, 4, 4 |
| Bless / Curse | base damage per creature: max + v / max(1, min − v) | 0, 0, 1, 1 |
| Mirth / Sorrow | morale ± v | 1, 1, 2, 2 |
| Fortune / Misfortune | luck ± v | 1, 1, 2, 2 |

- **Duration:** the caster's power + 1 (Collar), 2 (Ring), 3 (Cape of Conjuring), 50 (Ring of
  the Magi, whose parts count as well). VCMI: power plus its `SPELL_DURATION` bonuses (same
  values).
- **Mass versions** at Expert (flag `0x40`): every stack of the caster's side for a positive
  spell, of the enemy for a negative one; immune stacks are passed over; every stack draws
  for resistance (VCMI: "smart" mass targets, the same). Disrupting Ray has no mass version.
- **Counters (refutes the 2026-10-05 reading of VCMI):** only Bless/Curse and Haste/Slow end
  each other. Bloodlust and Weakness, Mirth and Sorrow, Fortune and Misfortune can be active at
  once and add (Mirth +2 with Sorrow −2: morale +0). VCMI counters all five pairs. The user chose
  the executable (Q47); batch 2b checks one pair.
- **Re-casting an active spell** keeps the longer duration and the first cast's value (VCMI:
  replaces). For a hero's casts both give the same; it matters when creatures cast (T7). ADR-0014
  point 5 amended (Q47).
- **Rounds fall** for every stack at the start of a round (M-3, as VCMI); an effect cast in round
  n with duration d is active in rounds n to n + d − 1. A stack that dies loses its effects.
- **Disrupting Ray never ends** (the end routine returns at once for it; VCMI: permanent).
  Quirk S-17 therefore concerns Berserk and Bind only (T4, T7).
- **Order of the stat changes:** Weakness, Stone Skin, Prayer, Haste and Disrupting Ray change
  the stack's values when cast and change them back when they end, so the result can depend on
  the order (a defense pushed to 0 by Disrupting Ray goes below 0 when Stone Skin ends; the
  original's damage formula then counts the negative defense). Morale and luck: the base value is
  clamped to ±3 before the spells, the minimums and caps apply after them, the rolls clamp again
  (a base luck of −1 with Fortune +2 gives +1).
- **Not in T3a:** the hero specialties in these spells (T3c); Magic Mirror on the negative ones
  (T4); creature casters (T7).

## 15. T3b1 preparation: what the code reading must settle (2026-10-06, a session without the original files)

T3b1 (Q48): Shield, Air Shield, the four Protections, Slayer, Counterstrike, Dispel, Cure. The
session had no executable, so the rules wait for a session that can read it; this section is the
reading list. Values by mastery from the packs (`SPTRAITS.TXT`); flags from §7.6's values;
VCMI 1.7.5 from `config/spells/timed.json`, `other.json`, `lib/battle/DamageCalculator.cpp`,
`lib/spells/CSpellHandler.cpp` (`adjustRawDamage`), `lib/spells/effects/Dispel.cpp`.

| Spell (id) | Values | Flags (§7.6) | VCMI 1.7.5 |
|---|---|---|---|
| Shield (27) | 85, 85, 70, 70 (melee damage × v %) | timed, mass at Expert, protective | melee reduction 15/30 %, one factor among the defense factors |
| Air Shield (28) | 75, 75, 50, 50 (ranged damage × v %) | timed, mass at Expert, protective | ranged reduction 25/50 % |
| Protection from Air, Fire, Water, Earth (30–33) | 70, 70, 50, 50 (spell damage × v %) | timed, mass at Expert, protective | 30/50 % for the school; the first school of the spell that has one (air, fire, water, earth), integer `× (100 − r) / 100` |
| Slayer (55) | 8 attack | timed, one target (no mass) | `SLAYER` = mastery (none 0, so none affects no creature); attack + v against a creature with king level ≤ it |
| Counterstrike (58) | 1, 1, 2, 2 retaliations | timed, mass at Expert, war machines not affected | `ADDITIONAL_RETALIATION`; war machines excluded too |
| Dispel (35) | — | not timed, mass at Expert, neutral | none/Basic own stacks, Advanced any stack, Expert every stack and also the spells' battlefield obstacles; removes every spell effect except persistent spells (Disrupting Ray, Clone, Acid Breath); ignores immunity |
| Cure (37) | power × 5 + 10, 10, 20, 30 | not timed, mass at Expert, beneficial | heals the top creature (no raising) and removes negative spell effects; castable only if one of the two applies |

**Already known from earlier readings:** the Protections (0x5A7BB0, §7.8): the first of the
spell's schools in the order earth, air, fire, water for which the stack has its protection
multiplies by the stored `f32` factor and truncates. That confirms quirk S-19 for SoD (Magic
Arrow takes the first protection, not the strongest; suggested class fix-optional, flagged with
T3b1). Durations, the re-cast rule, the round-start count-down and death follow T3a (§14).
Counterstrike spares war machines (bit `0x1000`); the others affect them.

**To read in the executable** (addresses where known):

1. 0x4448F0 (apply): what Shield, Air Shield, the Protections, Slayer and Counterstrike store
   (`v` as an integer, or `v / 100` as `f32` like Slow) and whether one changes a record field.
2. 0x444510 (end): what Counterstrike gives back, if anything.
3. The physical damage routine (callers of 0x442410 and 0x442590): where Shield's and Air
   Shield's factors enter, their order with Armorer, the shooting penalties and the obstacle
   penalty, and their rounding (single-precision as Q-0005 or a truncation of their own); whether
   Air Shield covers the Ballista's shots.
4. 0x5A7BB0: the stored factor's value, and nothing for a spell without a school.
5. 0x442410 (attack): Slayer's king levels by mastery (is none like Basic, or nothing as in
   VCMI?), melee and ranged alike.
6. Retaliations: where the per-round count is reset and how Counterstrike adds to it, with
   unlimited retaliation (Royal Griffins) and the Griffins' two.
7. Dispel (a case of the switch at 0x5A2838): which effects it ends (each through 0x444510, so
   Disrupting Ray's defense stays?), the side by mastery, Expert on every stack; whether it ends
   battlefield spells (Fire Wall, Force Field, Quicksand, Land Mine; T5); resistance and immunity
   on an enemy stack.
8. Cure: which effects it ends (polarity −1, or a list; creature-ability effects for T7), the
   heal (top creature, capped, no raising), and whether it is legal on a healthy stack without
   negative effects.

**Content to add with T3b1** (pack rows written by hand from these flags and the gap-fill,
confirmed with `verify --with-original-data` on the user's machine, Q48): timed changes for the
melee and ranged damage factors, a school protection, Slayer's king levels and extra
retaliations; effect kinds Dispel and Cure (heal); Dispel's side by mastery.

**Experiment candidates (batch 2b, if the code leaves them open):** Shield with Armorer on a
fixed hit (rounding); Counterstrike on Griffins; Cure on a healthy stack.

**Results of the reading** (E04b T3b1, 2026-10-06; `installation.md` §7.10):

1. Shield, Air Shield and the Protections store `v / 100` as `f32`; Slayer stores the mastery;
   Counterstrike stores `v` and adds it to the retaliations left at once.
2. Ending gives nothing back; Counterstrike's extra retaliations stay until the next round start.
3. Physical damage has **three truncated stages** (integer bonuses; defense and penalties;
   Shield or Air Shield, petrification and Armorer), not one rounding as BAT-DMG-002 had it; the
   attack and defense rates are doubles. Air Shield takes every shot, Shield every melee strike.
   The user chose to rebuild BAT-DMG-002 in these stages now (Q50).
4. Confirmed: the first protected school in the order earth, air, fire, water; `f32(0.7)` makes
   Basic protection one lower on round values (S-19 stands).
5. Slayer: king 1 at any mastery (VCMI's "none affects nothing" is wrong), king 2 from Advanced,
   king 3 at Expert; +8 as a constant; melee and shots.
6. Retaliations: 1, Griffins 2, Royal Griffins unlimited, + Counterstrike while active, reset at
   the round start **before** durations fall, so Counterstrike's extra lasts one round longer.
7. Dispel: everything but Poison (Disrupting Ray stays); below Advanced own stacks only, below
   Expert only a stack with a spell; Expert every stack it can affect without a resistance roll,
   and the spell obstacles; Sphere of Permanence protects; never mirrored. Settled in T3b1b
   (2026-10-07): Dispel's own test returns 1 before the immunity, resistance and aura tests, so
   no stack is immune to it and none resists it (single target or mass), unlike the first
   reading's "then the usual immunity tests".
8. Cure: top creature only, no raising; ends 14 negative spells (not Bind); castable on any own
   stack it can affect, healthy or not (VCMI's condition is not in the original).

**Experiment candidates for batch 2b:** the x87 precision (4 Angels attacking with one defense
point against them: 195 at 53 bits, 194 at 64; Q50); Counterstrike's extra round; Shield with
Basic Armorer on a round value.

## 16. The review's quirk candidates (E04b T11b, 2026-10-12)

The independent review of E04b re-read the executable and met five original oddities that no
survey lists. By "Confirm before flagging" (`docs/04-rules-and-sources.md` §4) an oddity becomes a
quirk with a flag only after a second, independent confirmation (an experiment, original data, a
fresh agent's re-read of the deciding paths, or VCMI's agreement); until then the engine keeps the
clean behaviour and the candidate waits here with what would confirm it. An `exploit-fix` candidate
is never implemented: a confirmation only adds its ledger line. The ids continue `E04-quirks.md`
(S-01 to S-22). The experiments are in `docs/experiments/README.md` ("From the E04b review"); in
brackets, the review's finding (report, number).

### S-23 — The struck stack retaliates at a striker that the Fire Shield's reflection killed
- Original (code reading, one reviewer; r2b F-2): before the retaliation the melee routine 0x441CB0
  tests the struck stack and the strike (0x441DD4–0x441E01), but not whether the attacker still has
  creatures, as it does before the second strike (0x441EAF). An attacker that died of the
  reflection is struck back: the blow kills nothing and the struck stack has spent a retaliation.
- Against it: VCMI 1.7.5 strikes back only at a living attacker
  (`server/battles/BattleActionProcessor.cpp` L326).
- For it (the wiki checks, 2026-10-13): Horn of the Abyss fixed it, so the original has it: "Fixed
  a bug: when attacking stack died from Fire Shield, retaliation strike into nowhere still
  happened" (the HotA changelog, 1.7.1,
  https://heroes.thelazy.net/index.php/Horn_of_the_Abyss_(Changelog)); the Fire Shield page is
  silent.
- Would confirm it: one Archangel with 5 hit points left attacks 20 Efreet Sultans and dies of the
  reflection; a second stack attacks the Sultans in the same round and is struck back (clean) or
  not (the reading). Or a second agent's re-read of 0x441DC5–0x441E5F.
- Suggested **fix-optional** (the reviewer's: a wasted retaliation); the class is the user's
  (OPEN-QUESTIONS BAT-Q28, row 4).
- The engine today (clean): no retaliation at a dead stack; the struck stack keeps its retaliation
  (BAT-RET-001, BAT-MAG-018).

### S-24 — A hero's raise aimed at a two-hex corpse raises the top corpse on that corpse's rear hex
- Original (code reading, one reviewer; r4a F-5): the handler of Resurrection and Animate Dead
  (0x5A1907) does not raise the dead stack found on the aimed hex: it takes that stack's position
  hex, for a two-hex stack its rear hex, searches again from there (0x5A1917–0x5A1928) and raises
  what this second search returns. So when a later corpse B lies on the rear hex of a two-hex
  corpse A, a raise aimed at either hex of A brings back B, and A cannot be raised while B lies
  there. Sacrifice (0x5A1A49) and the Archangel (0x4485BE) raise the stack they found.
- For it (the wiki checks, 2026-10-13): Horn of the Abyss fixed it, so the original has it: "A bug
  has been fixed for the situations where the hind hex of a two-hex creature's corpse would be
  covered by a one-hex creature's corpse; in this event, it would be impossible to target the big
  creature with the Resurrection and Raise Dead spells, even when pointing at its front hex" (the
  HotA changelog, 1.5.4, https://heroes.thelazy.net/index.php/Horn_of_the_Abyss_(Changelog)).
  VCMI was not compared.
- Would confirm it: Wolf Riders die; a Swordsman dies later on their rear hex; Resurrection aimed
  at the Wolf Riders' front hex: who comes back? Or a second agent's re-read of the handler and
  its two searches (0x5A3CC0, 0x5A3F50).
- Suggested **fix-optional** (unintended and mild; whether the community knows it was not
  searched) (BAT-Q28, row 10).
- The engine today (clean): the corpse aimed at is raised, through any of its hexes on which no
  corpse lies above it (BAT-MAG-035).

### S-25 — A side no human commands escapes one bad-morale freeze in four
- Original (code reading; two reviewers read the same bytes; r5b F-3): after its roll the
  bad-morale check 0x464B40 gives a side whose byte +0x54A4 is 0 a second roll, `random(1, 4)`,
  and a 1 lets the stack off (0x464BDB–0x464BF8). The battle's setup (0x4639F0) fills the byte from
  the side's player and leaves 0 for a side without one. Unsure: what the byte means ("a human
  commands the side" rests on its setter 0x4CE940 and on H3API's names). Computer players' and
  neutral stacks would freeze at 6.25, 12.5 and 18.75 %, a human's at 8.3, 16.7 and 25 %.
- Other sources: VCMI has no such rule (`server/battles/BattleFlowProcessor.cpp` L378–398), nor
  has the wiki as the catalog quotes it (a wiki check is wanted).
- Would confirm it: a few hundred activations of neutral stacks at morale −3 (25 % or 18.75 %).
- Suggested class: none of the three fits well. The second roll looks deliberate, so a confirmed
  S-25 is a rule of the original, for which the battle must know who commands a side (a seam
  question, BAT-Q33); as a quirk it would be **preserve**.
- The engine today: one roll for every side (BAT-MOR-004), the original's rule for a human's side.

### S-26 — An obstacle removed early is removed again when its rounds run out
- Original (code reading, one reviewer, not observed; the control and battlefield report's "left
  open"): a removed battlefield object stays in the list with its rounds, and the round start's
  countdown (0x476162–0x47618F) does not ask whether an object is already removed (Expert Dispel's
  loop does, 0x5A170F). A Fire Wall or Force Field that Remove Obstacle or Dispel took away early
  is therefore removed a second time when its rounds run out: whatever was placed on its hexes
  since loses its hex marks and index (a later obstacle vanishes), and the removal calls through
  the object's graphic, which the first removal set to 0 (0x466C39: a crash is possible).
- Would confirm it: the wiki (Remove Obstacle, Dispel, Fire Wall, Force Field: a crash, or a later
  obstacle vanishing), or one row on a battle that may crash: a Fire Wall removed at once by
  Expert Dispel, a Force Field cast onto its hexes, then the round in which the Fire Wall would
  have ended.
- Suggested **exploit-fix** (a crash or a corrupted battlefield).
- The engine today: a removed obstacle leaves the state at once; nothing is removed twice
  (BAT-MAG-030, BAT-MAG-034).

### S-27 — A stack created in the wait phase and blinded before its turn stops the battle
- Original (code reading, one reviewer: "seems"; the mana and defences report's "left open"): the
  original runs no turn start (0x446E30) in the wait phase, so the "turn started" byte +0x4F0 is
  never set for a stack that is first picked there. A stack created in the wait phase (Summon
  Elemental, Clone) and blinded before its turn then seems to make the pick 0x465080 loop for
  ever.
- Would confirm it: a second agent's re-read of the pick's loop for a blind stack in the wait
  phase, or one row on a battle that may hang: both sides have a stack that waited; in the wait
  phase one hero summons elementals, then the other hero casts Blind on them before their turn.
- Suggested **exploit-fix** (a soft-lock).
- The engine today: does not have it; a blind stack's turn starts and passes in every phase
  (BAT-TRN-001, BAT-MAG-024).
