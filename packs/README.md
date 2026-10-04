# Content packs

Human-editable content packs (ADR-0006), parsed by `h3-pack` and compiled into a `Ruleset` by
`h3-content`. Keys are namespaced `pack:name`.

| Pack | Purpose |
|---|---|
| `synthetic/` | Invented creatures with neutral names for tests, benchmarks, replays and the walking skeleton. **Not** original game data (docs/05-testing.md "Test data policy"). |
| `synthetic-ext/` | Invented expansion of `synthetic`: exercises dependencies and overrides the way `ab`/`sod` do. |
| `base/` | **Harvested**: The Restoration of Erathia, latest patch (Q15): resources, 10 terrains, 8 factions with native terrains, 78 obstacle types, 121 creatures (118 plus the war machines Ballista, First Aid Tent and Ammo Cart, E03 T8) with their upgrades, battle parameters, profiles; since E04a T3 also 4 spell schools, 80 spells, 28 secondary skills, 16 hero classes and 128 heroes, and (T3b) 14 artifact slots (four misc positions) and 127 artifacts. |
| `ab/` | **Harvested**: Armageddon's Blade (depends on `base`): Conflux, 23 creatures, elementals moved to Conflux and given their upgrades; the 2 Conflux hero classes and 28 heroes (Conflux, Sir Mullich, the campaign heroes); Vial of Dragon Blood and Armageddon's Blade. |
| `sod/` | **Harvested**: The Shadow of Death (depends on `ab`): SoD-only changes as overrides; Titan's Lightning Bolt; the fifth misc position and the 12 combination artifacts. |

The harvested packs `base` (RoE), `ab` and `sod` are generated from the user's installation by
`crates/h3-harvest` and committed (E02 T4; OPEN-QUESTIONS Q13: gameplay data and names only, no
flavor text). Do not edit them by hand: change the harvester or its gap-fill file and re-run
`cargo run -p h3-harvest -- --install original-data/install --out packs`;
`cargo xtask verify --with-original-data` checks that they match the installation. Original files stay in the gitignored `original-data/`; the installation itself is
never written to.

## Enabling packs

Which packs are enabled is configuration (a scenario's `packs = [...]`, the CLI). The compiler
(ADR-0006, E02 amendment):

- rejects an enabled pack whose `depends` are not all enabled (`MissingDependency`), unknown or
  duplicate pack ids, and dependency cycles;
- loads packs in dependency order: dependencies first, ties broken by pack id. The order in which
  packs are listed does not matter;
- gives each pack a **dependency closure** (itself plus everything it depends on, transitively).
  A pack may only refer to, or override, content of its closure.

For the harvested packs: `sod` depends on `ab`, `ab` on `base`. Enabling `sod` without `ab` is an
error.

## Format v2 (E02)

Strict TOML: unknown fields and numbers that do not fit their type are errors. Every definition is
validated again by the compiler.

```toml
[pack]
id = "synthetic"                 # [a-z0-9_-]+, the namespace of the pack's keys
version = "0.1.0"
depends = []                     # pack ids

[[resource]]                     # resource kinds are data
id = "coin"
name = "Coin"

[[terrain]]                      # terrain types (E03; base: harvested from TERRNAME.txt)
id = "meadow"
name = "Meadow"

[[faction]]
id = "vale"
name = "Vale"
native_terrain = "meadow"        # optional: +1 attack/defense/speed there (BAT-TER-001)

[[obstacle]]                     # obstacle types (E03, BAT-GEO-004)
id = "crag"
terrains = ["meadow"]            # at least one
width = 3                        # usual obstacles: free hexes to the right of the anchor
height = 2                       # usual obstacles: rows needed above the anchor
blocked = [[0, 0], [1, 0], [1, -1]]  # [dx, dy] from the anchor (bottom-left hex, dy <= 0)
# absolute = true                # fixed position: `blocked` lists [x, y] hexes of the field

[[creature]]
id = "brute"
name = "Brute"                   # display names (default "")
plural = "Brutes"
faction = "vale"                 # optional; omitted = neutral
level = 2                        # >= 1 (default 1)
attack = 7                       # required battle stats
defense = 4
damage = [2, 5]                  # min >= 1 (war machines: or [0, 0]), max >= min
hit_points = 18                  # >= 1
speed = 5                        # >= 1 (war machines: >= 0)
shots = 0                        # default 0
casts = 0                        # spell casts per battle, default 0
cost = { coin = 120, shard = 1 } # resource -> amount, default {}
growth = 7                       # weekly growth, default 0
horde_growth = 3                 # default 0
fight_value = 220                # default 0
ai_value = 250                   # default 0
adventure_count = [10, 25]       # wandering-stack size range, high >= low, default [0, 0]
flags = ["alive", "two_hex"]     # closed set, see below; default []

[[creature_override]]            # change a creature of a pack in the dependency closure
target = "synthetic:wisp"        # always a full key
faction = "marsh"                # any creature field except `id`; only listed fields change
level = 2                        # `faction = ""` makes the creature neutral
```

**References.** A reference without `:` (`faction = "vale"`, `cost = { coin = 1 }`) means a key
of the pack itself; `pack:name` (quoted as a TOML key: `{ "synthetic:coin" = 1 }`) refers to
another pack, which must be in the dependency closure. Cost entries are stored in key order; a
resource may appear once.

**Overrides** are applied in load order after the target was defined; list fields (`cost`,
`flags`) are replaced as a whole. The patched creature is validated like a definition.

**Creature flags** (`h3_content::CreatureFlag`, mirroring the executable's flag bits,
`docs/formats/installation.md` §5): `two_hex`, `flying`, `shooter`, `breath_attack`, `alive`,
`catapult`, `war_machine`, `king_1`, `king_2`, `king_3`, `mind_immune`, `no_melee_penalty`,
`fire_immune`, `strikes_twice`, `no_enemy_retaliation`, `no_morale`, `undead`,
`multi_headed`, `area_attack`, `dragon`. Read by battle rules so far: `two_hex`, `flying`,
`shooter`, `breath_attack`, `alive`, `no_melee_penalty`, `fire_immune`, `strikes_twice`,
`no_enemy_retaliation`, `no_morale`, `undead` (E03 T3–T6) and `war_machine` (E03 T8: placed at
a fixed hex, acts automatically, never retaliates, no morale, not counted for alignment or the
end of the battle; not allowed in army slots, BAT-WAR-001); `multi_headed` and `area_attack` are
required by the matching abilities. The others are data only until their rules arrive.

**Abilities** (E03, ADR-0012; `h3_content::Ability`): behaviour beyond the flag bits, with
parameters. `abilities = [ ... ]` on a creature (or an override, replacing the whole list), each
entry an inline table with `kind` and that kind's parameters; unknown kinds or parameters are
errors:

| `kind` | Parameters | Meaning (rule ids in `docs/rules/catalog/battle.toml`) |
|---|---|---|
| `hate` | `creature`, `percent` | +`percent` % base damage against that creature (one entry per target; BAT-DMG-004) |
| `jousting` | `percent_per_hex` | +% base damage per hex travelled before a melee attack (BAT-DMG-005) |
| `charge_immunity` | — | attackers get no jousting bonus against it (BAT-DMG-005) |
| `defense_reduction` | `percent` (1–100) | the target's defense counts `100 − percent` % (rounded down; BAT-DMG-006) |
| `death_blow` | `chance_percent` | chance of +100 % base damage per attack (BAT-DMG-007) |
| `life_drain` | `percent` (1–1000) | heals/resurrects itself by that share of the damage dealt to living targets (BAT-DMG-008) |
| `regeneration` | — | top creature regains its lost hit points at the start of its turn (BAT-TRN-005) |
| `death_stare` | `chance_percent` | per creature, kills one living enemy creature after attacking (BAT-DMG-009) |
| `extra_retaliations` | `count` | retaliates `1 + count` times per round (BAT-RET-003) |
| `unlimited_retaliations` | — | retaliates against every attack (BAT-RET-003) |
| `strike_and_return` | — | returns to its starting hex after a melee attack (BAT-ATK-004) |
| `attacks_all_adjacent` | — | melee hits every adjacent enemy (requires flag `multi_headed`; BAT-ATK-003) |
| `three_headed_attack` | — | melee hits the target and the hexes beside it (requires `multi_headed`; BAT-ATK-003) |
| `teleports` | — | moves like a flyer (path irrelevant; BAT-MOV-002) |
| `morale_aura`, `luck_aura` | `amount`, `affects` (`allies`/`enemies`) | battle-wide morale/luck change; equal auras do not stack (BAT-MOR-001, BAT-LCK-001) |
| `minimum_morale`, `minimum_luck` | `value` | lower bound of its morale/luck (BAT-MOR-001, BAT-LCK-001) |
| `no_distance_penalty` | — | its shots have no distance penalty (BAT-RNG-002) |
| `area_shot` | `effect` (`fireball`/`death_cloud`) | its shots also hit the stacks around the target (requires flag `area_attack`; BAT-RNG-004) |
| `first_aid` | `min_hp`, `max_hp` (1 ≤ min ≤ max) | war machine only (requires flag `war_machine`; First Aid Tent): on its turn heals the top creature of a random hurt allied stack by a uniform `min_hp..=max_hp` (BAT-WAR-003) |
| `ammo_supply` | — | war machine only (requires flag `war_machine`; Ammo Cart): while it lives its side's shots use no ammunition; it never acts (BAT-WAR-004) |

`creature` references follow the reference rules above (own pack, or `pack:name` in the
dependency closure) and may name creatures defined later in the same pack. At most one entry of
each kind, except `hate` with different targets; `extra_retaliations`/`unlimited_retaliations` and
the two attack patterns exclude each other. A ruleset has at most 8 distinct aura values (kind,
amount, `affects`; `MAX_AURA_VALUES`), which sizes the battle's modifier pool. The rule ids say
which rule reads each kind.

**Upgrades** (pack format v3, E04a T3): `upgrades = ["halberdier"]` on a creature (or an
override, replacing the list) names the creature types it upgrades into: at most 3, not itself,
none twice; references follow the reference rules and may name later creatures of the pack.
Hero effects that select "a creature" apply to it and its direct upgrades.

## Hero content (format v3, E04a T3)

Spell schools, spells, secondary skills, hero classes and hero types (ADR-0013). References follow
the reference rules above; every kind may refer to content of its dependency closure.

```toml
[[spell_school]]
id = "storm"
name = "Storm"

[[spell]]                          # a data row; what a spell does is battle magic (E04b)
id = "spark"
name = "Spark"
category = "combat"                # adventure | combat | ability (a creature ability)
level = 1                          # 1..=5; abilities 0..=5
schools = ["storm"]                # at least one; abilities none
cost = [6, 5, 5, 5]                # mana by mastery: none, basic, advanced, expert
power = 10                         # effect per point of spell power (default 0)
effect = [10, 10, 20, 30]          # base effect by mastery
polarity = "negative"              # positive | neutral | negative

[[secondary_skill]]
id = "marksmanship"
name = "Marksmanship"
basic = [{ kind = "ranged_damage", bp = 1000 }]   # hero effects per level (default [])
advanced = [{ kind = "ranged_damage", bp = 2500 }]
expert = [{ kind = "ranged_damage", bp = 5000 }]

[[hero_class]]
id = "warden"
name = "Warden"
faction = "vale"
primary = [2, 2, 1, 1]             # starting attack, defense, power, knowledge
primary_gain = [[40, 40, 10, 10], [30, 30, 20, 20]]  # level-up chances (%): levels 2-9, 10+; rows sum to 100
skill_weights = { marksmanship = 6, command = 8 }    # level-up weights; unlisted skills weigh 0

[[hero]]
id = "ansel"
name = "Ansel"
class = "warden"
skills = [{ skill = "command", level = 1 }]          # starting skills, levels 1..=3, at most 8
spell_book = false                 # default false; starting `spells = [...]` need it
army = [{ creature = "brute", count = [4, 7] }]       # at most army_slots stacks; 1 <= low <= high
war_machines = ["bolt_thrower"]    # creatures with flag war_machine (not in `army`)
specialty = [{ kind = "skill_specialty", skill = "marksmanship", bp_per_level = 500 }]
```

**Hero effects** (`h3_content::HeroEffect`, ADR-0013 §3): what a skill level, a specialty or an
artifact gives. Percentages are **basis points** (`bp`, 1/100 %). The translation to the
battle (`h3-heroes`, E04a T4) and the rules that read them come with E04a T4–T10. At most 16 per
list; one entry per kind, except `morale`/`luck` for different sides and `creature_stat` for
different creatures or statistics.

| `kind` | Parameters | Meaning |
|---|---|---|
| `morale`, `luck` | `amount` (≠ 0), `affects` (`own` default, `enemy`, `both`) | battle-long morale/luck change of the affected stacks (Leadership, Luck) |
| `ranged_damage`, `melee_damage` | `bp` (≥ 1) | base-damage bonus of the hero's shots (Ballista included) / melee attacks (Archery, Offense) |
| `damage_reduction` | `bp` (1..=10000) | less physical damage to the hero's stacks (Armorer) |
| `surrender_discount` | `bp` (1..=10000) | cheaper surrender (Diplomacy) |
| `tactics` | `level` (1..=3) | the hero's Tactics level |
| `artillery` | `shots` (≥ 1), `double_damage_chance_bp` (≤ 10000) | the hero commands its Ballista |
| `first_aid` | `heal_bp` (≥ 1) | the hero commands its First Aid Tent; its largest heal grows by `heal_bp` of the tent's own `first_aid` maximum |
| `creature_stat` | `creatures` (omitted: all; `{ creature = "x" }`: x and its upgrades; `{ flag = "dragon" }`), `stat` (`attack`, `defense`, `speed`, `hit_points`, `min_damage`, `max_damage`), `amount` (≠ 0) | fixed change of a creature statistic |
| `skill_specialty` | `skill`, `bp_per_level` (≥ 1) | specialty: the skill's scalable values grow by `bp_per_level` of themselves per hero level |
| `creature_specialty` | `creature`, `bp_per_step` (≥ 1), `speed` (default 0) | specialty: attack and defense of the creature and its upgrades grow by `bp_per_step` of the creature's value (rounded up) per full step of (creature level) hero levels; speed + `speed` |
| `spell_specialty` | `spell` | specialty: stronger casts of the spell (formula per spell: E04b) |
| `primary_skills` | `attack`, `defense`, `power`, `knowledge` (each default 0, not all 0) | changes the hero's primary skills (artifacts); the only kind that counts once **per equipped copy** |
| `morale_at_most`, `luck_at_most` | `value`, `affects` | caps the morale/luck of the affected stacks for the battle (Spirit of Oppression, Hourglass of the Evil Hour: `value = 0`, `affects = "both"`) |
| `alignment_group` | `factions` (2..=16, distinct) | creatures of these factions count as one alignment for the army-alignment morale of the hero's side (BAT-MOR-002; Angelic Alliance) |
| `creature_stat_percent` | `creatures`, `stat`, `bp` (≥ 1) | change of a creature statistic by a share of the creature's own value (Elixir of Life: +25 % hit points) |
| `regeneration` | `creatures`, `hit_points` (≥ 1) | the top creature of each selected stack regains up to `hit_points` at the start of its turn (Elixir of Life) |
| `no_range_penalty`, `shoot_when_adjacent` | `creatures` | the selected creatures shoot without the range penalty / also while an enemy is adjacent (Golden Bow, Bow of the Sharpshooter) |
| `no_retreat` | — | neither side may retreat or surrender (Shackles of War) |

The three `*_specialty` kinds are allowed in a hero's `specialty` only.

**Artifacts** (format v3, E04a T3b; `h3_content::ArtifactType`, ADR-0013 §3). A slot is a *kind*
of equipment position with a count; an artifact fits any position of its slot.

```toml
[[artifact_slot]]
id = "charm"
name = "Charm"
count = 2                          # positions of this kind on a hero (1..=8, default 1; all slots together at most 32)
backpack = false                   # optional (default true): its artifacts never go to the backpack (since E04a T4a; HER-ART-002)

[[artifact_slot_override]]         # a later pack changes a slot of its closure (name and/or count)
target = "synthetic:charm"
count = 3

[[artifact]]
id = "sight_lens"
name = "Sight Lens"
class = "treasure"                 # treasure | minor | major | relic | special
cost = { coin = 2000 }             # value by resource (default: none)
slot = "charm"                     # omitted: carried in the backpack only
needs_skill = "marksmanship"       # optional: the effects apply only while the hero has the skill
war_machine = "bolt_thrower"       # optional: the war machine (flag war_machine) it places in battle
effects = [{ kind = "ranged_damage", bp = 500 }]   # hero effects, no *_specialty kinds
parts = ["a", "b"]                 # a combination's parts (2..=16, distinct)
```

The compiler checks after all packs: effect lists as above; a war-machine artifact has a slot
and is neither a combination nor a part; a combination's parts have slots, are not combinations
themselves and belong to no other combination, its own slot is one of its parts' slots, and the
slots have enough positions for all parts. A combination **locks** one position of every part's
slot except one of its own (`ArtifactType::locks`, derived: Angelic Alliance in the right hand
locks the torso, feet, neck, left hand and head). It gives its own effects, and its parts give
theirs (the derivation, E04a T10). Identical artifacts count once, except `primary_skills`
(the original sums its artifact table per copy: two Quiet Eyes of the Dragon give +2/+2).

**What the harvested hero content leaves out** (the harvester's report lists it; sources and
layouts: `docs/formats/installation.md` §6–§7, `crates/h3-harvest/data/gapfill.toml`):

- Skills without effects yet: the magic skills, Sorcery, Resistance, Intelligence, Mysticism,
  Wisdom, Eagle Eye (E04b), Ballistics (E04c), Necromancy and Learning (outcome application,
  E08–E10), and the adventure skills. Their `[[secondary_skill]]` entries exist without levels.
- Specialties not modelled: 13 resource and 2 creature-upgrade specialties (adventure), Adrienne
  (none). Spell specialties carry only the spell; their formulas come with E04b.
- Spells: town chances, AI values and the target/flag attributes (E04b, E09); hero classes:
  aggression and tavern weights (E09); hero availability (campaign heroes, Lord Haart the Knight
  retired by the expansions; E08–E10).
- Checked by experiment batch 1a (SoD, `experiments/results/E04a-batch-1a.md`): Artillery gives
  one Ballista shot at Basic and two at Expert (R6, R7; dispute D1 goes to VCMI and the wiki, not
  the skill text); the creature specialties follow the executable's specialty table: Catherine
  and Sorsha raise Swordsmen and Crusaders, Roland only Crusaders (B4–B6, dispute H-1; the wiki's
  "Catherine only Crusaders" does not hold in SoD). Xeron's +1 speed (H-2) is VCMI and the wiki,
  not observed yet.

**What the harvested artifacts leave out** (the harvester's report lists it; facts and sources:
`docs/formats/installation.md` §6, §7.3, `crates/h3-harvest/data/gapfill.toml` `[artifacts]`,
research notes and disputes in `docs/rules/surveys/E04-artifacts.md`):

- The primary skills come from the executable's table; war machines, combination parts and every
  other effect from the gap-fill file. Effects no epic models yet are **pending** (the pack marks
  such entries `# pending: <epic>`): spells, spell immunity and resistance, mana, spell duration
  and damage, Eagle Eye (E04b; 43 artifacts, among them the opening spells of Angelic Alliance and
  Armor of the Damned and Titan's Thunder's spell book), the Catapult and the siege obstacle
  penalty (E04c), Necromancy (the three Necromancy artifacts and the Cloak of the Undead King;
  outcome application, E08–E10, together with the Necromancy skill) and the adventure effects
  (movement, vision, resources, growth, the Grail; E08–E10).
- Not harvested: the descriptions, the three unused SoD rows (Diplomat's Suit, Mired in Neutrality,
  Ironfist of the Ogre), the combinations' costs (blank in the original table; VCMI's values are
  its own).
- Angelic Alliance's `alignment_group` names the five towns of the SoD text and the wiki (dispute
  D1: VCMI lets every non-evil faction mix, Conflux included; an experiment for batch 1b).
  Speed and hit-point artifacts use the selector "all creatures", so they also reach war machines
  as in VCMI (open check 1); the Golden Bow's "shooters" include the Ballista (open check 2).
  Both are experiment candidates; the rules come with E04a T10.

**Battle parameters** (`[battle]`, Tier 2; exactly one enabled pack provides them): field
size, army slots, damage rolls, attack/defense rates and caps, Defend bonus, the formation rows
`loose_formation` and `tight_formation` (`*_formation[n - 1]` = starting rows of an army with `n`
stacks in slot order, distinct rows of the field; the tight table since E03 T9, BAT-SET-001),
and since E03 `ranged_full_damage_distance`, `ranged_penalty_percent`, `melee_penalty_percent`
(BAT-RNG-002/003) and the morale and luck roll tables `morale_dice`, `good_morale_chances`,
`bad_morale_chances`, `luck_dice`, `good_luck_chances` (chances out of the die's sides per level,
bad-morale chances below the die's sides so that no stack freezes for certain,
at most 8 levels; the table lengths bound morale and luck, BAT-MOR-001..004, BAT-LCK-001/002),
and `war_machine_positions` (E03 T8: three head hexes `[x, y]` of the attacker's war machines by
kind — shooting machine, ammo supply, first aid — mirrored for the defender, BAT-WAR-001). A war
machine's kind follows from its data: flag `shooter`, ability `ammo_supply` or ability
`first_aid`. Since E04a T4a the compiler also checks that the positions leave room for two-hex
machines: each head hex has a hex behind it on the field, and no two footprints of either side
(the defender's mirrored) share a hex.

**Hero parameters** (`[hero_params]`, Tier 2, E04a T4a; at most one enabled pack provides them,
a ruleset without them cannot validate hero details): `primary_min` (four values, attack,
defense, power, knowledge: lower totals count as these), `primary_max` (the highest primary
skill, base and effective; every minimum at most this) and `backpack` (the most artifacts in a
hero's backpack). Rules HER-DET-001 and HER-ART-002; the base pack's values come from the
gap-fill file (`[heroes.params]`: 0/0/1/1, 99, 64).

**Profiles** (`[[profile]]` with `quirks = { "quirk.<context>.<name>" = true | false }`; unlisted
flags are off): the registered flags are in `h3_content::REGISTRY` and the quirks ledger
(`docs/rules/quirks-ledger.md`); the packs' `faithful` profiles turn Q-0003 and Q-0005 on,
`strict-fixed` off. Profile-level Tier-2 overrides are not implemented yet (backlog).

The compiled ruleset's canonical hash (encoding v11, `RULESET_HASH_VERSION`) covers packs,
profile, quirks, resources, terrains, factions, obstacles, all creature fields including abilities
and upgrades, the battle parameters and the hero content with the artifacts and the hero
parameters; replays record it.

## Scenarios

A scenario (`synthetic/scenarios/*.toml`) is a battle setup for tests, the CLI and benchmarks:

```toml
[scenario]
id = "skirmish"
packs = ["synthetic"]            # enabled packs (any order)
profile = "synthetic:faithful"
obstacles = [58, 75]             # optional: blocked hexes, row-major (BAT-GEO-003)
terrain = "synthetic:meadow"     # optional: the battlefield's terrain (BAT-TER-001)
random_obstacles = true          # optional: generate obstacles for the terrain (BAT-GEO-004;
                                 # the tools use stream OBSTACLES of the battle seed); not
                                 # together with `obstacles`
attacker_war_machines = ["synthetic:bolt_thrower", "synthetic:aid_tent"]  # optional, per side:
defender_war_machines = ["synthetic:supply_wagon"]  # war machines by key, one per kind (BAT-WAR-001)
tight_formation = ["defender"]   # optional: sides in tight formation, the others loose (BAT-SET-001)

[scenario.attacker_hero]         # optional, per side (`defender_hero`): the battle hero leading
hero_type = "synthetic:brea"     # the side, given directly (BAT-HER-001, Q31); a side without
attack = 2                       # one is not led by a hero (quirk Q-0003). Every field but
defense = 2                      # hero_type defaults to 0 (effects: none); the rest of the battle
power = 1                        # hero (war-machine control, other effects, spell book, …) joins
knowledge = 1                    # the format with the rules that read it (E04a T7–T10, E04b)
tactics = 2
mana = 10
max_mana = 10
ranged_bp = 1000                 # damage bonuses in basis points (BAT-HER-004): the side's shots,
melee_bp = 1000                  # its melee strikes, and the reduction of strikes against it
reduction_bp = 500               # (at most 10 000)
effects = [{ kind = "morale", amount = 1 }, { kind = "luck_at_most", value = 0, affects = "enemy" }]
                                 # battle effects as in "Hero effects": so far `morale`, `luck`,
                                 # `morale_at_most`, `luck_at_most` (BAT-HER-003)

[[attacker]]                     # one entry per stack
slot = 0
creature = "synthetic:spearman"  # full key
count = 30

[[defender]]
slot = 0
creature = "synthetic:brute"
count = 8
```

The battle context validates the setup when the battle starts (slots, counts, obstacle hexes,
war machines, heroes).
