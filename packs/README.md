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

**Battle parameters** (`[battle]`, Tier 2) and **profiles** (`[[profile]]` with `quirks`) are
unchanged from E01; exactly one enabled pack provides `[battle]`. Profile-level Tier-2 overrides
are not implemented yet (backlog).

The compiled ruleset's canonical hash (encoding v2, `RULESET_HASH_VERSION`) covers packs, profile,
quirks, resources, factions, all creature fields and the battle parameters; replays record it.
