# Content packs

Human-editable content packs (ADR-0006), parsed by `h3-pack` and compiled into a `Ruleset` by
`h3-content`. Keys are namespaced `pack:name`.

| Pack | Purpose |
|---|---|
| `synthetic/` | Invented creatures with neutral names for tests, benchmarks, replays and the walking skeleton. **Not** original game data (docs/05-testing.md "Test data policy"). |
| `synthetic-ext/` | Invented expansion of `synthetic`: exercises dependencies and overrides the way `ab`/`sod` do. |
| `base/` | **Harvested**: The Restoration of Erathia, latest patch (Q15): resources, 8 factions, 118 creatures, battle parameters, profiles. |
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

[[faction]]
id = "vale"
name = "Vale"

[[creature]]
id = "brute"
name = "Brute"                   # display names (default "")
plural = "Brutes"
faction = "vale"                 # optional; omitted = neutral
level = 2                        # >= 1 (default 1)
attack = 7                       # required battle stats
defense = 4
damage = [2, 5]                  # min >= 1, max >= min
hit_points = 18                  # >= 1
speed = 5                        # >= 1
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
`multi_headed`, `area_attack`, `dragon`. Flags are data only until the rules that read them
arrive (E03).

**Abilities** (E03, ADR-0012; `h3_content::Ability`): behaviour beyond the flag bits, with
parameters. `abilities = [ ... ]` on a creature (or an override, replacing the whole list), each
entry an inline table with `kind` and that kind's parameters; unknown kinds or parameters are
errors:

| `kind` | Parameters | Meaning (rule ids in `docs/rules/catalog/battle.toml`) |
|---|---|---|
| `hate` | `creature`, `percent` | +`percent` % base damage against that creature (one entry per target) |
| `jousting` | `percent_per_hex` | +% base damage per hex travelled before a melee attack |
| `charge_immunity` | — | attackers get no jousting bonus against it |
| `defense_reduction` | `percent` (1–100) | the target's defense counts `100 − percent` % (rounded down) |
| `death_blow` | `chance_percent` | chance of +100 % base damage per attack |
| `life_drain` | `percent` | heals/resurrects itself by that share of the damage dealt to living targets |
| `regeneration` | — | top creature regains its lost hit points at the start of its turn |
| `death_stare` | `chance_percent` | per creature, kills one living enemy creature after attacking |
| `extra_retaliations` | `count` | retaliates `1 + count` times per round |
| `unlimited_retaliations` | — | retaliates against every attack |
| `strike_and_return` | — | returns to its starting hex after a melee attack |
| `attacks_all_adjacent` | — | melee hits every adjacent enemy (requires flag `multi_headed`) |
| `three_headed_attack` | — | melee hits the target and the hexes beside it (requires `multi_headed`) |
| `teleports` | — | moves like a flyer (path irrelevant) |
| `morale_aura`, `luck_aura` | `amount`, `affects` (`allies`/`enemies`) | battle-wide morale/luck change; equal auras do not stack |
| `minimum_morale`, `minimum_luck` | `value` | lower bound of its morale/luck |
| `no_distance_penalty` | — | its shots have no distance penalty |
| `area_shot` | `effect` (`fireball`/`death_cloud`) | its shots also hit the stacks around the target (requires flag `area_attack`) |

`creature` references follow the reference rules above (own pack, or `pack:name` in the
dependency closure) and may name creatures defined later in the same pack. At most one entry of
each kind, except `hate` with different targets; `extra_retaliations`/`unlimited_retaliations` and
the two attack patterns exclude each other. Abilities are data only until the rule that reads
them arrives (E03 tasks, see the epic file).

**Battle parameters** (`[battle]`, Tier 2) and **profiles** (`[[profile]]` with `quirks`) are
unchanged from E01; exactly one enabled pack provides `[battle]`. Profile-level Tier-2 overrides
are not implemented yet (backlog).

The compiled ruleset's canonical hash (encoding v3, `RULESET_HASH_VERSION`) covers packs, profile,
quirks, resources, factions, all creature fields including abilities, and the battle parameters;
replays record it.

## Scenarios

A scenario (`synthetic/scenarios/*.toml`) is a battle setup for tests, the CLI and benchmarks:

```toml
[scenario]
id = "skirmish"
packs = ["synthetic"]            # enabled packs (any order)
profile = "synthetic:faithful"
obstacles = [58, 75]             # optional: blocked hexes, row-major (BAT-GEO-003)

[[attacker]]                     # one entry per stack
slot = 0
creature = "synthetic:spearman"  # full key
count = 30

[[defender]]
slot = 0
creature = "synthetic:brute"
count = 8
```

The battle context validates the setup when the battle starts (slots, counts, obstacle hexes).
