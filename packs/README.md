# Content packs

Human-editable content packs (ADR-0006). Keys are namespaced `pack:name`. Since pack format v35
(R01, the content split) a pack folder holds one file per context:

| File | Holds | Parsed by, compiled by |
|---|---|---|
| `core.toml` | the `[pack]` manifest; resources, factions, terrains, creatures (identity and non-battle data: name, faction, level, speed, cost, growth, values, flags, upgrades), creature overrides' core halves, spell schools, spells (name, category, level, schools, cost, power, effect, `granted_by_name_only`), secondary skills, hero classes, heroes, hero overrides, artifact slots and their overrides, artifacts, `[hero_params]`, profiles | `h3-pack` into a `ContentPack`; `h3-content` into the core `Ruleset` |
| `battle.toml` | no manifest (it belongs to its folder's `core.toml`): `[battle]`, obstacles, each creature's battle data (`[[creature]]` with its `id`: attack, defense, damage, hit points, shots, casts, abilities), creature overrides' battle halves (`target`, `fixes` and those fields), each spell's battle magic (`[[spell]]` with its `id`: polarity and every targeting and effect field) | `h3-pack-battle` into a `BattlePack`; `h3-content-battle` into the `BattleRuleset` on the core |
| `adventure.toml` | the adventure map's data (owned by the adventure lane; format: `crates/h3-pack-adventure/src/lib.rs`) | `h3-pack-adventure` |

The TOML keys are the same in either file; the sections below show each table with all its
fields, whichever file a field lives in. Every creature and every spell of a pack has a row in
both files (an adventure spell's battle row holds its polarity); an override goes to the file(s)
whose fields it changes, an override without changes to `core.toml`, and `fixes` gates each half.
A pack folder without `battle.toml` (or `adventure.toml`) reads as an empty one.

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
`cargo run -p h3-harvest -- --install original-data/install --out packs` (it writes `core.toml`
and `battle.toml`); `cargo xtask verify --with-original-data` checks that both match the
installation. Original files stay in the gitignored `original-data/`; the installation itself is
never written to. A pack folder's `adventure.toml` is the adventure context's file (ADR-0006, E08
amendment): hand-written source owned by the adventure lane, which the harvester never writes and
`--with-original-data` does not check; its format is in `crates/h3-pack-adventure/src/lib.rs`.

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
blocked = [[0, 0], [1, 0], [1, -1]]  # [dx, dy] from the anchor (bottom-left hex, dy <= 0); a
                                 # usual obstacle blocks at most 8 hexes (the battle's
                                 # removable piece, BAT-GEO-003)
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
speed = 5                        # >= 1 (war machines: 0)
shots = 0                        # default 0
casts = 0                        # spell casts per battle, default 0 (`spell_caster`, `random_spell_caster`; BAT-MAG-048, BAT-MAG-049)
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

**Data quirks** (format v34, E04b T8e; ADR-0007, amendment E04b T8e): an override with
`fixes = "quirk.content.<name>"` fixes a quirk in the data. The entity keeps the original's
value; the override's target and references are resolved in every profile, and it is applied
(then validated with its entity) only in profiles that turn the flag off (`strict-fixed`; the
harvester compiles its packs in both profiles). The flag must be a registered `quirk.content` flag
(`CompileError::NotADataQuirk`). `[[hero_override]]` changes a hero type of the closure, the
pack's own included: `target` (a full key), optional `fixes`, and `specialty` (replaces the whole
list). The harvested `ab` pack fixes Roland's specialty (Q-0006) and the Ice, Magma and Energy
Elementals' levels (BAT-QUIRK-19) so:

```toml
[[creature_override]]
target = "ab:ice_elemental"
fixes = "quirk.content.elemental_levels"
level = 3

[[hero_override]]
target = "ab:roland"
fixes = "quirk.content.roland_crusader_specialty"
specialty = [{ kind = "creature_specialty", creature = "base:swordsman", bp_per_step = 500, speed = 1 }]
```

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
| `first_aid` | `min_hp`, `max_hp` (1 ≤ min ≤ max) | war machine only (requires flag `war_machine`; First Aid Tent): on its turn heals the top creature of a random hurt stack of its own side (also one the other side controls, BAT-MAG-026) by a uniform `min_hp..=max_hp` (BAT-WAR-003) |
| `ammo_supply` | — | war machine only (requires flag `war_machine`; Ammo Cart): while it lives its side's shots use no ammunition; it never acts (BAT-WAR-004) |
| `spell_level_immunity` | `max_level` (1..=5) | spells of this level and below cannot affect it (dragons, Magic Elemental; BAT-MAG-005) |
| `school_immunity` | `school` | the school's spells cannot affect it (one entry per school; fire immunity; BAT-MAG-005) |
| `spell_immunity` | `spell`, `absolute` (default false; since v31) | the spell cannot affect it (one entry per spell; elementals; BAT-MAG-005); an `absolute` one also not when the immunities are negated (Troglodytes against Blind and Stone Gaze, Gargoyles against Poison; BAT-MAG-066) |
| `spell_vulnerability` | `spell`, `percent` (1..=1000) | +`percent` % damage from the spell (one entry per spell; elementals; BAT-MAG-004) |
| `spell_damage_reduction` | `percent` (1..=100) | damage from spells × (100 − `percent`) / 100, rounded down (Golems; BAT-MAG-004) |
| `magic_resistance` | `percent` (1..=100) | the creature lets `100 − percent` % of hostile spells through (Dwarves, Crystal Dragon; BAT-MAG-008) |
| `magic_resistance_aura` | `percent` (1..=100) | the chance of adjacent stacks of the side that controls it (BAT-MAG-026) to be affected by a hostile spell × (100 − `percent`) %, the strongest aura once (Unicorns; BAT-MAG-008) |
| `spell_after_strike` | `spell`, `chance_percent` (1..=100), `mastery` (0..=3), `rounds` (≥ 1), `living`, `unless_active`, `defended` (v21) (default false) | since v19: after each melee strike casts `spell` (timed or dispel effects only) on the struck stack: the chance drawn (none at 100), the stack alive, of a living creature if `living`, not under the spell if `unless_active`, with a defense above 0 if `defended`, then the spell's test without the side test (none for a spell that `ignores_immunity`; a `bind` spell only once per striker); at `mastery`, lasting `rounds`, without a hero (one entry per spell; Unicorns, Black Knights, the flies, BAT-MAG-040; Dendroids, Rust Dragons, BAT-MAG-045, BAT-MAG-046) |
| `spell_damage_after_strike` | `spell`, `chance_percent` (1..=100), `per_creature` (≥ 1) | since v19: after each melee strike deals `per_creature` × its creatures damage as `spell` (a damage spell) to the struck stack, after the chance and the spell's test (the Thunderbird; BAT-MAG-041) |
| `damage_after_strike` | `chance_percent` (1..=100), `percent` (1..=100) | since v21: after each melee strike, on a living struck stack and after the chance, deals `percent` % of one roll of its damage range × its creatures (truncated; nothing if 0), without a test (the Rust Dragon; BAT-MAG-047) |
| `spell_caster` | `spell`, `mastery` (0..=3), `power` (≥ 1) | since v22: its stack may cast `spell` as its action, its creature's `casts` times a battle, at another living stack the spell may be aimed at (a creature's cast: `mastery`, value `power` × power factor + effect, timed effects lasting `power` rounds, no hero bonuses or cost, no resistance roll for the aimed stack); the spell has damage, timed, dispel or heal effects only, no health limit, and takes a target stack at `mastery` in every profile (Ogre Mage, the AB elementals; BAT-MAG-048) |
| `random_spell_caster` | `mastery` (0..=3), `power_per_creature` (≥ 1) | since v23: at each activation of its stack while it has casts left it draws one of its creature's `drawn_spell` entries by weight and may cast it as its action, its creature's `casts` times a battle, at a living stack other than itself that the spell can affect by its polarity (either side for a neutral spell; a hex spell at a hex of that stack, never an empty hex): a creature's cast at `mastery` with power `power_per_creature` × its creatures; the draw is private to its controller (the Faerie Dragon; BAT-MAG-049). Excludes `spell_caster`; needs `drawn_spell` entries |
| `drawn_spell` | `spell`, `weight` (≥ 1) | since v23: one spell a random caster (since v24 a helpful caster: then a stack spell; since v26 a periodic caster: then a spell without a target or damage) may draw, with its weight among the creature's entries (one entry per spell); the spell has damage, timed, dispel or heal effects only, no health limit or obstacle, and takes a target stack or hex at the caster's mastery in every profile (BAT-MAG-049) |
| `helpful_spell_caster` | `mastery` (0..=3), `power` (≥ 1) | since v24: its stack may cast as its action, its creature's `casts` times a battle, at another stack its side owns for which one of its creature's `drawn_spell` entries is useful (the spell's `useful`); the spell is drawn at the cast among the useful ones by weight: a creature's cast at `mastery` with `power` (the Master Genie; BAT-MAG-050). Excludes `spell_caster` and `random_spell_caster`; needs `drawn_spell` entries |
| `stack_raiser` | `spell` (a spell with a raise effect), `hit_points_per_creature` (≥ 1) | since v25: its stack may raise as its action, its creature's `casts` times a battle, another stack of its side, living or dead, for good by `hit_points_per_creature` × its creatures hit points, under `spell`'s test (its conditions, level and immunities; the hit points must reach one creature's health): the Archangel with Resurrection and 100 (BAT-MAG-051). Excludes the other caster kinds |
| `corpse_raiser` | `creature`, `hit_points_per_creature` (≥ 1) | since v25: its stack may raise as its action, its creature's `casts` times a battle, a stack of `creature` from the corpse of a stack of its side whose creature is `alive`: `min(start, min(hit_points_per_creature × its creatures, start × health) div the creature's health)` creatures with the corpse's start count and health, which take that stack's army slot: the Pit Lord's Demons with 50 (BAT-MAG-052). Excludes the other caster kinds |
| `periodic_spell_caster` | `rounds` (≥ 1), `mastery` (0..=3), `power` (≥ 1) | since v26: no action; at its stack's activation, when its controller's side has cast no such spell for `rounds` round starts (every battle starts ready), casts on its own one of its creature's `drawn_spell` entries, drawn by weight among those useful (the spell's `useful`) on a stack of the side it aims at (its own, the enemy's for a negative spell; with `quirk.battle.periodic_first_stack` on only that side's first stack), at `mastery` with `power` and no target: the Enchanter, every three rounds at Expert with power 3 (BAT-MAG-053). Excludes the other caster kinds; needs `drawn_spell` entries, each taking no target at `mastery` and doing no damage (so its cast ends no battle) |
| `spell_cost` | `amount` (−128..=127, not 0), `affects` (`allies` or `enemies`) | since v27: while its creature is in a side's army, alive or not (a stack created in the battle never counts), the hero of the side `affects` names pays `amount` more mana for each spell (at least 1; a spell costing 0 stays 0): a hero's cost changes by the largest increase and the largest reduction among such abilities, once each (the Pegasi +2 for the enemy hero, the Mages −2 for their own; BAT-MAG-054) |
| `mana_channel` | `percent` (1..=100) | since v27: after a hero's cast against its controller's side, while its stack lives, the hero of that side gains `percent` % of the cast's cost, rounded down, once per cast (the largest percent among that side's stacks; no cap at the maximum): the Familiars' 20 % (BAT-MAG-055) |
| `mana_drain` | `amount` (≥ 1) | since v27: at each activation of its stack in a round's normal phase (not once the waited stacks act), the hero of the side opposite its controller loses `amount` mana, at least 0: the Wraith's 2 (BAT-MAG-056) |
| `innate_spell` | `spell` (a spell with a timed `fire_shield` or `magic_mirror` change), `value` (1..=100) | since v28: its stack counts as having the spell's effect at `value` whenever the spell is not active on it with a larger value (the spell itself is not on it: Dispel cannot end it, and Hypnotize's refusal of a mirrored stack does not see it): the Efreet Sultan's 20 % Fire Shield (BAT-MAG-058), the Faerie Dragon's 20 % Magic Mirror (BAT-MAG-057). Several for different spells; the rules read only these two changes |
| `damage_penalty` | `creature`, `percent` (1..=100) | since v28: its strikes deal `percent` % less damage to `creature` (the original's damage stage 2, beside the defense factor; the largest penalty that applies): the Magic Elemental's half against Magic Elementals and Black Dragons (BAT-DMG-010). Several for different creatures |
| `damage_penalty_by_flag` | `flag` (a creature flag), `percent` (1..=100) | since v28: as `damage_penalty`, against creatures with flag `flag`: the Psychic Elemental's half against mind-immune creatures (BAT-DMG-010). Several for different flags |
| `rebirth` | `divisor` (≥ 1) | since v29: after the action that killed its stack (a strike, a cast, a mine's or Fire Wall's damage), if the stack has a cast left (its creature's `casts`) and is not a clone (nor a sacrificed one, BAT-QUIRK-17), one cast is used and the stack comes back for good on its hexes, without the effects it died with, with its count at the battle's start div `divisor` creatures, plus one for each of the count mod `divisor` others with a 1-in-`divisor` chance; it counts as having acted this round, and no creature comes back if none is drawn (the cast is used either way): the Phoenix's fifth (BAT-MAG-059) |
| `fear` | `percent` (1..=99: never certain, so that the frightened side gets to act) | since v29: while its stack lives, a stack of the other side (taken from the frightened stack's controller; the frightening stacks count by their real side, BAT-QUIRK-18) that is picked to act outside the round's wait phase, after the bad-morale check and not at the battle's first activation without a tactics phase, is frozen with `percent` % chance (one roll, the largest chance among the side's living stacks with `fear`): it loses its turn and counts as having acted. Creatures without morale and those with `fear` are never frozen: the Azure Dragon's 10 % (BAT-MAG-060) |
| `ignores_magic_block` | — | since v31: the creature's casts ignore a hero's `no_casting` (the Orb of Inhibition): the Faerie Dragon, whose F key casts under the Orb in the original, as the wiki says (BAT-MAG-061; the user, BAT-Q32 (2), 2026-10-13; batch 2e's cursor row saw the Orb stop the cursor) |

`creature` references follow the reference rules above (own pack, or `pack:name` in the
dependency closure) and may name creatures defined later in the same pack. At most one entry of
each kind, except `hate`, the three immunities, `spell_vulnerability` and `spell_after_strike` with different targets
(`school` and `spell` references follow the same rules); `extra_retaliations`/`unlimited_retaliations` and
the two attack patterns exclude each other. A ruleset has at most 8 distinct aura values (kind,
amount, `affects`; `MAX_AURA_VALUES`), which sizes the battle's modifier pool. The rule ids say
which rule reads each kind.

**Upgrades** (pack format v3, E04a T3): `upgrades = ["halberdier"]` on a creature (or an
override, replacing the list) names the creature types it upgrades into: at most 3, not itself,
none twice; references follow the reference rules and may name later creatures of the pack.
Hero effects that select "a creature" apply to it and its direct upgrades.

## Hero content (format v3, E04a T3; battle magic format v4, E04b T2a; v5 T3a; v6 T3b1; v7 T3b1b; v8 T3b2; v9 T3c; v10 T4a; v11 T4b; v12 T4c; v13 T5a; v14 T5c1; v15 T5c2; v16 T5b; v17 T6a; v18 T6b; v19 T7a1; v20 T7a2; v21 T7a3; v22 T7b1; v23 T7b2a; v24 T7b2b; v25 T7b3; v26 T7b4; v27 T7c1; v28 T7c2a; v29 T7c2b; v30 T8a; v31 T8b)

Spell schools, spells, secondary skills, hero classes and hero types (ADR-0013). References follow
the reference rules above; every kind may refer to content of its dependency closure.

```toml
[[spell_school]]
id = "storm"
name = "Storm"
protection_rank = 1                # v6: where a spell of several schools meets this school's
                                   # protection under quirk Q-0007 (lower first; default 0)

[[spell]]                          # a data row, and since format v4 its battle magic
id = "spark"
name = "Spark"
category = "combat"                # adventure | combat | ability (a creature ability)
level = 1                          # 1..=5; abilities 0..=5
schools = ["storm"]                # at least one; abilities none
cost = [6, 5, 5, 5]                # mana by mastery: none, basic, advanced, expert
power = 10                         # effect per point of spell power (default 0)
effect = [10, 10, 20, 30]          # base effect by mastery
polarity = "negative"              # positive | neutral | negative
target = "hex"                     # v4: none (default) | stack | hex
area = { inner = 0, radius = [1, 1, 1, 2] }  # v4, target hex: hexes at distance inner..=radius[mastery]
side = "any"                       # v4: whose stacks it reaches: any (default) | enemy | own
excludes = ["war_machine"]         # v4: creature flags a reached stack may not have (`requires`: must have)
effects = [{ kind = "damage" }]    # v4: what it does to each stack reached (default none: not castable)

[[spell]]                          # a timed spell (pack format v5, E04b T3a)
id = "hasten"
category = "combat"
level = 1
schools = ["storm"]
cost = [6, 5, 5, 5]
effect = [3, 3, 5, 5]              # v: the change by mastery
polarity = "positive"
target = "stack"
side = "own"
mass = "expert"                    # v5: from this mastery no target, every stack it may reach
                                   # (v11: `fixed_mass = "expert"`: the mass version's later
                                   # mastery without quirk Q-0011, as Forgetfulness's)
mind = false                       # v5: a mind spell (creatures with flag mind_immune are immune)
excludes = ["war_machine"]
duration = "rounds"                # v5: rounds (default: power + duration bonuses) | permanent
                                   # (not with a blind, petrify or paralyze change: a stack must
                                   # get its turns back);
                                   # v8: own_turn (1 turn on the active stack, else 2, counted at
                                   # the stack's own turn start: Frenzy); v12: fixed, with
                                   # `fixed_rounds = 255` (whatever the power: Berserk)
cumulative = false                 # v5: a cast on an affected stack applies its changes again
ends = "never"                     # v11: never (default) | damaged (any damage the stack takes
                                   # ends the effect: Blind); v12: own_attack (its melee attack or
                                   # shot ends it, not a retaliation: Berserk)
health_limit = false               # v12: a stack's full hit points must not exceed the cast's
                                   # value (Hypnotize)
refuses = []                       # v12: timed changes whose active effects keep a stack from
                                   # being a target (Hypnotize: ["magic_mirror"])
counters = ["slow"]                # v5: spells whose effects a cast ends on each stack it affects
effects = [{ kind = "timed", change = "speed" }]
specialist = { kind = "creature_level", values = [3, 3, 2, 2, 1, 1, 0] }
                                   # v9: what a hero specialised in the spell adds to its value
                                   # by a rule of the spell's own: by the stack's creature level
                                   # 1 to 7, `{ kind = "fixed", amount = 2 }` or `{ kind =
                                   # "total", value = 3 }` (the value becomes 3); without it the
                                   # hero's spell_specialty rate scales the value (BAT-MAG-020)

[[spell]]                          # a dispel spell (pack format v7, E04b T3b1b)
id = "purge"
category = "combat"
level = 1
schools = ["storm"]
cost = [5, 4, 4, 4]
effect = [0, 0, 0, 0]
polarity = "neutral"
target = "stack"
side = "own"
any_side = "advanced"              # v7: from this mastery it reaches either side
needs_effect = true                # v7: a stack it is aimed at needs an active timed effect
ignores_immunity = true            # v7: no immunity, magic resistance or aura applies; v10: nor
                                   # Anti-Magic, and a Magic Mirror never sends it away; v21:
                                   # cast after a strike, no test at all (Bind, Acid Breath)
mass = "expert"
spares = ["poison"]                # v7: its dispel effect ends every effect but these (or
                                   # `dispels = [...]`: only these; not both)
clears_obstacles = "expert"        # v13: from this mastery a cast also removes every spell
                                   # obstacle (Expert Dispel; needs a dispel effect)
# dispels_positive = true          # v19: the dispel effect ends only positive spells' effects,
                                   # and a strike casts it only on a stack with one (Dispel
                                   # Helpful Spells, a creature ability; BAT-MAG-042)
# restores_health = true           # v20: (with a heal effect) a cast first gives back what
                                   # Poison took (Cure; BAT-MAG-044)
# useful = "wounded"               # v24: when a helpful caster finds it useful on a stack
                                   # (BAT-MAG-050): always (default), enemy_melee,
                                   # enemy_shooter, enemy_spell_book, wounded, melee, shooting,
                                   # not_acted, enemy_king
effects = [{ kind = "dispel" }]

[[spell]]                          # an obstacle spell (pack format v13, E04b T5a)
id = "scorch"
category = "combat"
level = 2
schools = ["storm"]
cost = [8, 6, 6, 6]
power = 10
effect = [10, 10, 20, 50]          # a fire obstacle keeps power × power + effect as its value
polarity = "neutral"
target = "hex"                     # an obstacle spell has target hex, no area
obstacle = { steps = ["top_right", "top_left"], hexes = [2, 2, 3, 3], mirrored = true, separate = true, rounds = 2 }
                                   # v13: the hexes after the target hex, each the previous one's
                                   # neighbour (right, bottom_right, bottom_left, left, top_left,
                                   # top_right) as the attacker casts it; `hexes`: how many by
                                   # mastery (1..=steps + 1); `mirrored`: the defender's hero casts
                                   # them mirrored left to right (default false); `separate`: one
                                   # obstacle per hex (default false: one over all); `rounds` it
                                   # lasts, the cast's included (v14: default 0, the whole battle)
effects = [{ kind = "obstacle", obstacle = "fire" }]   # v13: wall | fire; v14: pit; v15: mine

[[spell]]                          # an obstacle spell of a random shape (pack format v14, E04b
id = "mire"                        # T5c1): no target; each hex a random free one
category = "combat"
level = 2
schools = ["stone"]
cost = [8, 6, 6, 6]
effect = [0, 0, 0, 0]
polarity = "neutral"
obstacle = { hexes = [4, 4, 6, 8], random = true, separate = true }
                                   # v14: `random` (default false): no steps, not mirrored, hexes
                                   # 1..=8 by mastery; the spell's target is none
effects = [{ kind = "obstacle", obstacle = "pit" }]
                                   # v15: `mine` (Land Mine, BAT-MAG-033): hidden as a pit, with
                                   # the cast's value (`power` x 10 + `effect`) as its damage to a
                                   # stack of the other side that enters it unseen; not cast while
                                   # the other side's natives would see it

[[spell]]                          # a teleport (pack format v13): an own stack to a hex
id = "blink"
category = "combat"
level = 3
schools = ["storm"]
cost = [15, 12, 6, 3]
effect = [0, 0, 0, 0]
polarity = "positive"
target = "stack"
side = "own"
excludes = ["war_machine"]
effects = [{ kind = "teleport" }]  # v13: the cast's second target is the hex the stack goes to

[[spell]]                          # a remove effect (pack format v16, E04b T5b): the obstacle on
id = "uproot"                      # the target hex goes (BAT-MAG-034)
category = "combat"
level = 2
schools = ["stone"]
cost = [7, 5, 5, 5]
effect = [0, 0, 0, 0]
polarity = "neutral"
target = "hex"
removes = { fire = "advanced", wall = "expert", pit = "expert", mine = "expert" }
                                   # v16: a removable natural obstacle at every mastery; a spell
                                   # obstacle of a kind from the mastery named (default: never)
effects = [{ kind = "remove_obstacle" }]

[[spell]]                          # a raise effect (pack format v17, E04b T6a): a Resurrection
id = "revive"                      # (BAT-MAG-035)
category = "combat"
level = 4
schools = ["stone"]
cost = [20, 16, 16, 16]
power = 50
effect = [40, 40, 80, 160]
polarity = "positive"
target = "stack"                   # an own stack, living or dead
side = "own"
requires = ["alive"]
excludes = ["war_machine"]
temporary_below = "advanced"       # v17: below this mastery the creatures raised stay for the
                                   # battle only (default: always for good)
effects = [{ kind = "raise" }]

[[spell]]                          # a sacrifice effect (v17): a Sacrifice (BAT-MAG-036)
id = "offer"
category = "combat"
level = 5
schools = ["storm"]
cost = [25, 20, 20, 20]
power = 1
effect = [3, 3, 6, 10]
polarity = "positive"
target = "stack"                   # the stack it raises; the cast's second target is the victim
side = "own"
requires = ["alive"]
excludes = ["undead", "war_machine"]
tests_as = "revive"                # v17: a raise spell; under quirk BAT-QUIRK-5 the target is
                                   # tested as that spell's (required with a sacrifice effect)
effects = [{ kind = "sacrifice" }]

[[spell]]                          # a clone effect (v18, E04b T6b): a Clone (BAT-MAG-038)
id = "copy"
category = "combat"
level = 4
schools = ["storm"]
cost = [24, 20, 20, 20]
power = 0
effect = [1, 1, 2, 3]              # the highest creature level it copies, by mastery
polarity = "positive"
target = "stack"                   # an own living stack
side = "own"
excludes = ["war_machine"]
effects = [{ kind = "clone" }]

[[spell]]                          # a summon effect (v18): a Summon Elemental (BAT-MAG-039)
id = "conjure"
category = "combat"
level = 5
schools = ["stone"]
cost = [25, 20, 20, 20]
power = 0
effect = [2, 2, 3, 4]              # creatures per point of the caster's power, by mastery
polarity = "neutral"
summons = "shade"                  # v18: the creature (required with a summon effect)
effects = [{ kind = "summon" }]
# granted_by_name_only = true      # v30: only a `spells` effect naming it gives it, never one by
                                   # school or level (Titan's Lightning Bolt, the spell flag 0x2000)

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

**Battle magic** (pack format v4, E04b T2a; `h3_content::SpellMagic`, ADR-0014 point 1): what
the caster names (`target`), the stacks a cast reaches and what it does to each. Without `target`
or with `target = "none"` a spell reaches every stack that meets its conditions (Armageddon).
`area` (target `hex` only) and `chain = { targets = [4, 4, 5, 5], factor_bp = 5000 }` (target
`stack` only: stacks struck in all by mastery, 1..=16; each further stack takes `factor_bp` of
the previous one's value) are exclusive; `requires`/`excludes` are creature flags (not both for one
flag). Effect kinds (at most four, distinct): `damage` (the value `power × power factor +
effect[mastery]` as damage; BAT-MAG rules, E04b T2b/T2d); `timed` (v5, E04b T3a: while the effect
lasts, `change` alters the stack by `v = effect[mastery]`: `speed` + v (never on war machines),
`speed_percent` × v % (rounded down, at least 1), `attack` + v, `attack_loss` − v (at most down to
0), `defense` + v, `defense_loss` − v (at most down to 0, never given back), `melee_attack` /
`ranged_attack` + v on melee / ranged attacks, `maximum_damage` (each creature deals its maximum +
v), `minimum_damage` (its minimum − v, at least 1), `morale`, `morale_loss`, `luck`, `luck_loss`
± v; BAT-MAG-009 to 011; since v6, E04b T3b1: `melee_protection` / `ranged_protection` (melee
strikes / shots taken deal v %, BAT-MAG-012), `school_protection` (spells of this spell's schools
deal v %, BAT-MAG-013), `slayer` (+ v attack against king creatures by mastery, BAT-MAG-014),
`retaliations` (+ v per round, BAT-MAG-015); since v8, E04b T3b2: `fire_shield` (melee strikes
taken reflect v % of their first stage onto the striker, BAT-MAG-018), `frenzy` (attack + defense
× v %, defense 0 when struck, BAT-MAG-019); since v10, E04b T4a: `spell_immunity` (no spell below
level v reaches the stack unless it is a creature ability or ignores immunity; the first cast ends
its negative effects below level v, BAT-MAG-021), `magic_mirror` (a stack spell aimed at it by
the other side's hero goes with a chance of v % to a random living stack of the caster's side; a
creature's spell is never sent away,
BAT-MAG-022); since v11, E04b T4b: `forgetfulness` (a shooter counts half its creatures, at
least one, on every strike; from the first cast's Advanced mastery it cannot shoot,
BAT-MAG-023), `blind` (the stack loses its turns; struck in melee while blind it retaliates
with v % of its damage, not at all at 0, BAT-MAG-024); since v12, E04b T4c: `berserk` (its
turns are automatic, attacking the nearest stack; next to any stack it counts as an enemy, but
its own side may not attack it, BAT-MAG-025), `hypnotize` (the other side controls it and may
attack it, its own side may not, BAT-MAG-026); since v20, E04b T7a2:
`petrify` (the stack loses its turns, takes half the damage of strikes and does not retaliate),
`paralyze` (it loses its turns; struck in melee it retaliates with v %, the smaller with Blind's;
both BAT-MAG-043), `poison` (its hit points per creature fall by a tenth of their start value at
the cast and at each round start while it lasts, to half, and stay down after it ends) and `age`
(halved while it lasts; both BAT-MAG-044); since v21, E04b T7a3: `bind` (the stack cannot move,
it attacks only next to it; it ends when its last binder, a striker whose `spell_after_strike`
cast it, moves, dies or is teleported, or when the stack is teleported, BAT-MAG-045)); since v13, E04b T5a: `obstacle` (with `obstacle = "wall"` or `"fire"` and the
spell's `obstacle` shape: places obstacles on the target hex's shape for its rounds; a wall blocks
its hexes, BAT-MAG-027; fire burns a stack that enters a hex or acts on one with the value as the
spell's damage, BAT-MAG-028; since v14, E04b T5c1, `"pit"`: hidden from the other side until revealed,
it stops a walker that enters it, BAT-MAG-031/032) and `teleport` (moves the target stack to the cast's second target,
a free hex, BAT-MAG-029); since v16, E04b T5b: `remove_obstacle` (target `hex`: takes away the
removable natural obstacle there, or the spell obstacles there of the kinds in `removes` from the
mastery named, as the caster sees them; BAT-MAG-034); since v17, E04b T6a: `raise` (target `stack`: an
own stack, living or dead, with fewer creatures than at the start, gains the value in hit points up to
that count; below `temporary_below` for the battle only; BAT-MAG-035) and `sacrifice` (target `stack`,
second target a victim of the side: kills it and raises the target by `(its creature's hit points + the
value) × its creatures`; `tests_as` names the raise spell whose test the target takes under quirk
BAT-QUIRK-5; BAT-MAG-036), each the spell's only effect, without area, chain or mass version; since
v18, E04b T6b: `clone` (target `stack`: an own stack of a creature level up to effect[mastery], no
clone and without one, is copied for the caster's power + its duration bonus rounds; BAT-MAG-038) and
`summon` (no target: effect[mastery] × the caster's power creatures of `summons` join the caster's
side; one summoned creature per side and battle; BAT-MAG-039), each the spell's only effect, a ruleset
having at most four summon spells; both create a stack only while the side has fewer than
`side_stack_limit` stacks (BAT-MAG-037); since
v7, E04b T3b1b: `dispel` (ends the stack's
timed effects: those of the spells in `dispels`, or without that list every one but those in
`spares`; never a permanent one; BAT-MAG-016) and `heal` (heals the top creature by the value,
raising none; BAT-MAG-017), applied after damage and before timed changes. `mass` needs target
`stack`, `fixed_mass` a `mass` and a later mastery; `duration`, `ends`, `cumulative`,
`health_limit` and `refuses` need a timed effect (an area timed spell, Berserk, has target
`hex`), `counters` too (and may not name the
spell itself); `dispels`, `spares` and `dispels_positive` need a dispel effect, `restores_health` a heal effect; `any_side` (a mastery) needs a `side`;
`needs_effect` needs target `stack`. Later tasks add kinds. Adventure spells have no battle magic; a spell without effects cannot be cast
yet. `side` limits whose stacks a stack spell may be *aimed* at (BAT-MAG-003; a chain's jumps
and an area reach either side): the harvester writes `enemy` for every single-target spell that
harms (the original's target test), and the gap-fill file for Chain Lightning, which is neutral
but which the original's targeting code refuses on the caster's own stacks (E04b T2e). At most four spell schools and
128 spells per ruleset (`MAX_SPELL_SCHOOLS`, `MAX_SPELLS`: the battle keeps one mastery per school
and spell books as bitsets).

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
| `artillery` | `shots` (≥ 1), `double_damage_chance_bp` (≤ 10000) | the hero commands its Ballista: `shots` per command at one target, each with the chance of +100 % damage (BAT-WAR-006); several such effects: the largest of each value (HER-BAT-005) |
| `first_aid` | `heal_bp` (≥ 1) | the hero commands its First Aid Tent; its largest heal grows by `heal_bp` of the tent's own `first_aid` maximum (BAT-WAR-007); several such effects add up, a skill's grown by its skill specialty (HER-BAT-005) |
| `creature_stat` | `creatures` (omitted: all; `{ creature = "x" }`: x and its upgrades; `{ flag = "dragon" }`), `stat` (`attack`, `defense`, `speed`, `hit_points`, `min_damage`, `max_damage`), `amount` (≠ 0) | fixed change of a creature statistic |
| `skill_specialty` | `skill`, `bp_per_level` (≥ 1) | specialty: the skill's scalable values grow by `bp_per_level` of themselves per hero level |
| `creature_specialty` | `creature`, `bp_per_step` (≥ 1), `speed` (default 0) | specialty: attack and defense of the creature and its upgrades grow by `bp_per_step` of the creature's value (rounded up) per full step of (creature level) hero levels; speed + `speed` |
| `spell_specialty` | `spell`, `bp_per_level` (≥ 1) | specialty: the cast's damage to a stack grows by `bp_per_level` per (hero level div the stack's creature level), rounded up (SoD 300: 3 %; HER-BAT-012, BAT-MAG-004) |
| `primary_skills` | `attack`, `defense`, `power`, `knowledge` (each default 0, not all 0) | changes the hero's primary skills (artifacts); the only kind that counts once **per equipped copy** |
| `morale_at_most`, `luck_at_most` | `value`, `affects` | caps the morale/luck of the affected stacks for the battle (Spirit of Oppression, Hourglass of the Evil Hour: `value = 0`, `affects = "both"`) |
| `alignment_group` | `factions` (2..=16, distinct) | creatures of these factions count as one alignment for the army-alignment morale of the hero's side (BAT-MOR-002; Angelic Alliance) |
| `creature_stat_percent` | `creatures`, `stat`, `bp` (≥ 1) | change of a creature statistic by a share of the creature's own value (Elixir of Life: +25 % hit points) |
| `regeneration` | `creatures`, `hit_points` (≥ 1) | the top creature of each selected stack regains up to `hit_points` at the start of its turn (Elixir of Life) |
| `no_range_penalty`, `shoot_when_adjacent` | `creatures` | the selected creatures shoot without the range penalty / also while an enemy is adjacent (Golden Bow, Bow of the Sharpshooter) |
| `no_retreat` | — | neither side may retreat or surrender (Shackles of War) |
| `school_mastery` | `school`, `level` (1..=3) | the hero casts the school's spells at this mastery; the highest counts (the magic school skills; HER-BAT-011) |
| `spell_damage` | `bp` (≥ 1), `school` or `spell` (neither: every spell) | more damage from the hero's spells of that scope; sums per scope (Sorcery, the Orbs, Ciele; HER-BAT-012) |
| `spell_book` | — | the hero can cast its available spells (the Spell Book; HER-SPL-002, HER-BAT-011) |
| `magic_resistance` | `bp` (1..=10000) | the hero's stacks resist hostile spells: subtracted from their creatures' share; several sources add (Resistance, three artifacts; HER-BAT-013, BAT-MAG-008) |
| `spell_duration` | `rounds` (≥ 1) | the hero's timed spells last longer; artifact types add once each, a combination's parts too (the Conjuring artifacts, Ring of the Magi; HER-BAT-014, BAT-MAG-010) |
| `spells` | exactly one of `school`, `level` (1..=5), `spell` | since v30: while worn, the hero has every spell of the school or of the level (never a creature ability nor a spell with `granted_by_name_only`) or the named spell (the Tomes, Spellbinder's Hat, Titan's Thunder, Armageddon's Blade, the Sea Captain's Hat; HER-SPL-002) |
| `spell_mastery` | `spell`, `level` (1..=3) | since v30: the hero casts the spell at least at this mastery (Armageddon's Blade: Armageddon at Expert; HER-SPL-003) |
| `scroll` | — | since v30: each copy of the artifact holds a spell of its own, which the hero has while the copy is worn (the Spell Scroll; HER-ART-003) |
| `no_casting` | — | since v31: neither side casts in the hero's battles: no hero's spell, no creature's cast or raise, no periodic cast; the strike's creature magic stays (the Orb of Inhibition; HER-BAT-015, BAT-MAG-061) |
| `spell_level_limit` | `level` (1..=4) | since v31: in the hero's battles no spell above `level` is cast by a hero or affects a stack, whoever casts it; the smallest counts (Recanter's Cloak: 2; HER-BAT-015, BAT-MAG-062) |
| `spell_immunity` | `spell` | since v31: the stacks the hero controls are never affected by the spell, whoever casts it (the Pendants, the Sphere of Permanence, Armageddon's Blade; HER-BAT-016, BAT-MAG-063) |
| `spell_level_immunity` | `level` (1..=5) | since v31: the stacks the hero controls are never affected by a spell up to `level`, Dispel excepted; the largest counts (Power of the Dragon Father: 4; BAT-MAG-064) |
| `mind_immunity` | — | since v31: the stacks the hero controls are immune to mind spells unless negated (Badge of Courage; BAT-MAG-065) |
| `negate_immunities` | — | since v31: the side's spells, and spells at the stacks the hero controls, ignore the creatures' immunities and all magic resistance (the Orb of Vulnerability; BAT-MAG-066) |
| `opening_spell` | `spell`, `mastery` (0..=3), `power` (≥ 1) | since v33: the side casts the spell for free at its first stack activation outside the tactics phase, at the mastery, its power the effects' duration, if it affects some stack; in the order of the artifact types (Angelic Alliance: Expert Prayer, 10; Armor of the Damned: Expert Slow, Curse, Weakness, Misfortune, 50; HER-BAT-017, BAT-MAG-068) |
| `eagle_eye` | `bp` (1..=10 000), `max_level` (0..=5, default 0) | since v33: Eagle Eye's chance to pick a spell at each enemy hero cast grows by `bp` (a specialty grows the skill's), spells up to the largest `max_level` may be picked (the skill: 40/50/60 %, levels 2/3/4; the Bird of Perception, the Stoic Watchman, the Emblem of Cognizance: +5/+10/+15 % with the skill; HER-SPL-004, BAT-MAG-069) |

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
theirs (the derivation, HER-BAT-001). Identical artifacts count once, except `primary_skills`
(the original sums its artifact table per copy: two Quiet Eyes of the Dragon give +2/+2).

**What the harvested hero content leaves out** (the harvester's report lists it; sources and
layouts: `docs/formats/installation.md` §6–§7, `crates/h3-harvest/data/gapfill.toml`):

- Skills without effects yet: Intelligence, Mysticism, Wisdom, Ballistics (E04c), Necromancy
  and Learning (outcome application, E08–E10), and the adventure skills. Their
  `[[secondary_skill]]` entries exist without levels. The magic school skills give
  `school_mastery` (gap-fill), Sorcery `spell_damage`, Resistance `magic_resistance` and Eagle Eye
  `eagle_eye` with the level limit the skill's level + 1 (the executable's value tables, E04b T2a,
  T2d and T8d).
- Specialties not modelled: 13 resource and 2 creature-upgrade specialties (adventure), Adrienne
  (none). Spell specialties are `spell_specialty` with the SoD growth of 3 % per level (gap-fill
  `[heroes.spell_specialty]`), Ciele's and Luna's a fixed `spell_damage` (+50 % Magic Arrow,
  +100 % Fire Wall).
- Spells: battle magic only for the 13 damage spells (E04b T2a): `target` and the `damage`
  effect from the executable's flag word (installation.md §7.6), areas, Chain Lightning's chain
  and side and the target conditions from the gap-fill file (`[[spells.targeting]]`); the other kinds come
  with their tasks (E04b T3–T7). Town chances and AI values (E09); hero classes:
  aggression and tavern weights (E09); hero availability (campaign heroes, Lord Haart the Knight
  retired by the expansions; E08–E10).
- Checked by experiment batch 1a (SoD, `experiments/results/E04a-batch-1a.md`): Artillery gives
  one Ballista shot at Basic and two at Expert (R6, R7; dispute D1 goes to VCMI and the wiki, not
  the skill text); the creature specialties follow the executable's specialty table: Catherine
  and Sorsha raise Swordsmen and Crusaders, Roland only Crusaders (B4–B6, dispute H-1; the wiki's
  "Catherine only Crusaders" does not hold in SoD). Xeron's +1 speed (H-2, VCMI and the wiki)
  holds in SoD (batch 1b B7).

**What the harvested artifacts leave out** (the harvester's report lists it; facts and sources:
`docs/formats/installation.md` §6, §7.3, `crates/h3-harvest/data/gapfill.toml` `[artifacts]`,
research notes and disputes in `docs/rules/surveys/E04-artifacts.md`):

- The primary skills come from the executable's table; war machines, combination parts and every
  other effect from the gap-fill file. Effects no epic models yet are **pending** (the pack marks
  such entries `# pending: <epic>`): none for E04b since T8d (the opening spells of Angelic
  Alliance and Armor of the Damned and Eagle Eye's three artifacts were its last; the mana
  artifacts and the spell book that equipping Titan's Thunder adds went to E08–E10), the Catapult
  and the siege obstacle penalty (E04c), Necromancy (the three Necromancy artifacts and the Cloak of the Undead King;
  outcome application, E08–E10, together with the Necromancy skill) and the adventure effects
  (movement, vision, resources, growth, the Grail; E08–E10).
- Not harvested: the descriptions, the three unused SoD rows (Diplomat's Suit, Mired in Neutrality,
  Ironfist of the Ogre), the combinations' costs (blank in the original table; VCMI's values are
  its own).
- Angelic Alliance's `alignment_group` names the five towns of the SoD text and the wiki; batch
  1b R1 confirmed it in SoD (Conflux and neutral creatures stay separate; dispute D1: VCMI lets
  every non-evil faction mix; BAT-HER-007).
  Speed and hit-point artifacts use the selector "all creatures"; the battle gives war machines
  the hit points but not the speed (batch 1b R8, BAT-HER-005). The Golden Bow's "shooters"
  include the Ballista (open check 2, batch 1b R8; BAT-HER-006).

**Battle parameters** (`[battle]`, Tier 2; exactly one enabled pack provides them): field
size, army slots, damage rolls, attack/defense rates and caps, Defend bonus, the formation rows
`loose_formation` and `tight_formation` (`*_formation[n - 1]` = starting rows of an army with `n`
stacks in slot order, distinct rows of the field; the tight table since E03 T9, BAT-SET-001),
and since E03 `ranged_full_damage_distance`, `ranged_penalty_percent`, `melee_penalty_percent`
(BAT-RNG-002/003) and the morale and luck roll tables `morale_dice`, `good_morale_chances`,
`bad_morale_chances`, `luck_dice`, `good_luck_chances` (chances out of the die's sides per level,
bad-morale chances below the die's sides so that no stack freezes for certain,
at most 8 levels; the table lengths bound morale and luck, BAT-MOR-001..004, BAT-LCK-001/002),
and `war_machine_positions` (E03 T8: three front hexes `[x, y]` of the attacker's war machines by
kind — shooting machine, ammo supply, first aid — mirrored for the defender, BAT-WAR-001). A war
machine's kind follows from its data: flag `shooter`, ability `ammo_supply` or ability
`first_aid`. Since E04a T4a the compiler also checks that the positions leave room for two-hex
machines: each front hex has a hex behind it on the field (a two-hex machine's position), and no
two footprints of either side (the defender's mirrored) share a hex. Since E04a T8 `tactics_zone_columns` (one width per skill
level: the tactics zone of a side whose Tactics level exceeds the enemy's by `d` is
`tactics_zone_columns[d - 1]` columns wide, each between 1 and the standable columns; `[3, 5, 7]`
in `base`, BAT-TAC-001; ruleset hash v12). Since E04a T9b the optional `surrender = { resource,
cost_percent }` (the price of a surrender is `cost_percent` of the summed cost in `resource` of
the side's living stacks; `{ resource = "gold", cost_percent = 50 }` in `base`; without it no
side can surrender; BAT-FLE-002, ruleset hash v13). Since E04b T2b `hero_casts_per_round` (how
many spells a hero may cast per round, 1 in `base`; BAT-MAG-001, ruleset hash v15). Since E04b
T6b `side_stack_limit` (a side creates stacks, Clone and Summon Elemental, only while it has fewer
stacks, dead ones included; 20 in `base`, at most 21; BAT-MAG-037, ruleset hash v27).

**Hero parameters** (`[hero_params]`, Tier 2, E04a T4a; at most one enabled pack provides them,
a ruleset without them cannot validate hero details): `primary_min` (four values, attack,
defense, power, knowledge: lower totals count as these), `primary_max` (the highest primary
skill, base and effective; every minimum at most this) and `backpack` (the most artifacts in a
hero's backpack). Rules HER-DET-001 and HER-ART-002; the base pack's values come from the
gap-fill file (`[heroes.params]`: 0/0/1/1, 99, 64).

**Profiles** (`[[profile]]` with `quirks = { "quirk.<context>.<name>" = true | false }`; unlisted
flags are off): the registered flags are in `h3_content::REGISTRY` and the quirks ledger
(`docs/rules/quirks-ledger.md`); the packs' `faithful` profiles turn Q-0003 and Q-0005 on,
`strict-fixed` off (since E04b T8e also the data quirks' `quirk.content` flags, which only gated
overrides read). A flag the profile names must be registered (`CompileError::UnknownQuirk`). A
retired flag is not: its position in the registry stays reserved, its name is gone, and a profile
that still names it, on or off, does not compile; the pack format's version does not change when
a flag comes or goes. Retired: `quirk.battle.attack_from_seen_pit` (BAT-QUIRK-2) and
`quirk.battle.creature_mirror_untested` (BAT-QUIRK-9), both on 2026-10-12;
`quirk.battle.natives_sight_at_start` (BAT-QUIRK-1) on 2026-10-13.
Profile-level Tier-2 parameters are not implemented yet (backlog).

The compiled ruleset's canonical hash (encoding v15, `RULESET_HASH_VERSION`) covers packs,
profile, quirks, resources, terrains, factions, obstacles, all creature fields including abilities
and upgrades, the battle parameters and the hero content (spells with their battle magic) with
the artifacts and the hero parameters; replays record it.

## Scenarios

A scenario (`synthetic/scenarios/*.toml`, parsed by `h3-pack-battle`) is a battle setup for tests,
the CLI and benchmarks:

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
attacker_funds = 2000            # optional, per side (`defender_funds`): what the side can pay
                                 # for a surrender (BAT-FLE-002; default 0)

[scenario.attacker_hero]         # optional, per side (`defender_hero`): the battle hero leading
hero_type = "synthetic:brea"     # the side, given directly (BAT-HER-001, Q31); a side without
attack = 2                       # one is not led by a hero (quirk Q-0003). Every field but
defense = 2                      # hero_type defaults to 0 (effects: none, no spell book); the
power = 1                        # Catapult's control joins with sieges (E04c)
knowledge = 1
tactics = 2
mana = 10
max_mana = 10
ranged_bp = 1000                 # damage bonuses in basis points (BAT-HER-004): the side's shots,
melee_bp = 1000                  # its melee strikes, and the reduction of strikes against it
reduction_bp = 500               # (at most 10 000); optional `ranged_single`, `melee_single`,
                                 # `reduction_single`: the original's single-precision value of
                                 # the rate as an f32 bit pattern, which a derived hero carries
                                 # under quirk Q-0005 (OPEN-QUESTIONS Q45, E04b T3c); the tent's
                                 # and the `spell_damage` and `magic_resistance` effects' as
                                 # `single`
surrender_discount_bp = 4000     # optional: the surrender discount (BAT-FLE-002, at most 10 000)
spell_book = ["synthetic:spark"] # optional: the spell book by spell key (BAT-MAG-001; E04b T2b;
                                 # a derived hero's in spell order, HER-SPL-002)
ballista = { shots = 1, double_damage_chance_bp = 5000 }   # optional: the side commands its
                                 # Ballista (Artillery, BAT-WAR-006)
tent = { heal_bonus_bp = 10000 } # optional: the side commands its First Aid Tent (First Aid,
                                 # BAT-WAR-007; the bonus of its largest heal)
masteries = { "synthetic:spark" = 2 }  # optional: the mastery per spell by spell key, 1..=3 (none
                                 # for a spell not listed; BAT-MAG-002); replaces `schools` since
                                 # replay format v20 (E04b T8a, the masteries are derived per
                                 # spell, HER-SPL-003)
effects = [{ kind = "morale", amount = 1 }, { kind = "luck_at_most", value = 0, affects = "enemy" }]
                                 # battle effects as in "Hero effects": so far `morale`, `luck`,
                                 # `morale_at_most`, `luck_at_most` (BAT-HER-003), `no_retreat`
                                 # (BAT-FLE-001), `stack_stat` (BAT-HER-005: `creatures` as in
                                 # "Hero effects" (omitted: all), `stat`, `op` = `add` |
                                 # `percent_bp` (rounded up) | `percent_bp_down`, `amount`),
                                 # `regeneration`, `no_range_penalty`, `shoot_when_adjacent`
                                 # (BAT-HER-006), `alignment_group` (BAT-HER-007; at most 16
                                 # factions), `spell_damage` (`bp`, `school` or `spell` by key),
                                 # `spell_specialty` (`spell`, `bp_per_level`, `level`: the
                                 # hero's level; BAT-MAG-004), `magic_resistance` (`bp`;
                                 # BAT-MAG-008) and `spell_duration` (`rounds`; BAT-MAG-010),
                                 # parameters as in "Hero effects"

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
