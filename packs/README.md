# Content packs

Human-editable content packs (ADR-0006), parsed by `h3-pack` and compiled into a `Ruleset` by
`h3-content`. Keys are namespaced `pack:name`.

| Pack | Purpose |
|---|---|
| `synthetic/` | Invented creatures with neutral names for tests, benchmarks, replays and the walking skeleton. **Not** original game data (docs/05-testing.md "Test data policy"). |
| `synthetic-ext/` | Invented expansion of `synthetic`: exercises dependencies and overrides the way `ab`/`sod` do. |
| `base/` | **Harvested**: The Restoration of Erathia, latest patch (Q15): resources, 10 terrains, 8 factions with native terrains, 78 obstacle types, 121 creatures (118 plus the war machines Ballista, First Aid Tent and Ammo Cart, E03 T8), battle parameters, profiles. |
| `ab/` | **Harvested**: Armageddon's Blade (depends on `base`): Conflux, 23 creatures, elementals moved to Conflux. |
| `sod/` | **Harvested**: The Shadow of Death (depends on `ab`): SoD-only changes as overrides. |

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
`first_aid`.
**Profiles** (`[[profile]]` with `quirks = { "quirk.<context>.<name>" = true | false }`; unlisted
flags are off): the registered flags are in `h3_content::REGISTRY` and the quirks ledger
(`docs/rules/quirks-ledger.md`); the packs' `faithful` profiles turn Q-0003 on, `strict-fixed`
off. Profile-level Tier-2 overrides are not implemented yet (backlog).

The compiled ruleset's canonical hash (encoding v8, `RULESET_HASH_VERSION`) covers packs, profile,
quirks, resources, terrains, factions, obstacles, all creature fields including abilities, and the battle parameters;
replays record it.

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
led_by_hero = ["attacker"]       # optional: sides led by a hero (Q17, quirk Q-0003)
attacker_war_machines = ["synthetic:bolt_thrower", "synthetic:aid_tent"]  # optional, per side:
defender_war_machines = ["synthetic:supply_wagon"]  # war machines by key, one per kind (BAT-WAR-001)
tight_formation = ["defender"]   # optional: sides in tight formation, the others loose (BAT-SET-001)

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
war machines).
