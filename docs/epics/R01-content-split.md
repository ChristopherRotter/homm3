# R01 — The content split

Status: **in progress** (2026-10-12, the battle lane after E04b's closure; the user handed the
choice of the next increment to the agent, who took R01 as BACKLOG and the last checkpoint
recommended). A `shared/` + `cross-lane` refactor with the battle lane paused (ADR-0015 point 3;
E08 decisions 2 and 3). Decisions marked *provisional* wait for the user's checkpoint on the pull
request.

## Goal

Battle and adventure depend only on what is theirs: `h3-content` keeps the core every context
reads, `h3-content-battle` holds the battle's ruleset part, `h3-pack` splits to match, each pack
folder gets one file per context, and the `.h3m` writer moves from `h3-formats` into `h3-map`.
Behaviour does not change: every golden replay keeps its state and event hashes (only headers
change).

## Decisions (provisional unless the ADRs already say so)

1. **Two pull requests.** R01a moves the writer (`shared/r01a-h3m-writer`, small, independent);
   R01b splits the content (`shared/r01b-content-split`). Both `cross-lane`; R01a merges first.
2. **The cut.** ADR-0015 point 3's core list read as follows; everything else in today's
   `h3-content` is the battle's.

   | `h3-content` (core) | `h3-content-battle` (the battle part) |
   |---|---|
   | keys, manifests, the dependency closure, profiles, the whole quirk registry | — |
   | resources, factions, terrains | obstacle types (`ObstacleType`) |
   | creature identity and its non-battle data (`CreatureInfo`, which gains `speed` and `flags`), upgrades | per creature: attack, defense, damage, hit points, shots, casts, abilities (`CreatureType` keeps copies of `speed` and `flags` for the hot path) |
   | spell schools; the spell row: key, name, category, level, schools, cost, power, effect, `granted_by_name_only` (a grant's property, read by `SpellGrant`) | per spell: polarity, `SpellMagic`, counters, dispels, spares, `tests_as`, `summons` |
   | the hero content whole: classes, hero types with specialties, secondary skills with their effects, artifact slots, artifacts with their effects, hero parameters, `HeroEffect` | `BattleParams` (field, morale, luck, war machine positions, the formations' rows, tactics zones, surrender, casts per round, side stack limit) and the battle's bounds (`MAX_FIELD_HEXES`, `MAX_SIDE_STACKS`, …) |
   | `Formation` (an army's formation is set on the adventure map; `h3-contracts` uses it), `MAX_ARMY_SLOTS` | the checks that read both: abilities against flags, strike spells, raised creatures, obstacles against the field, auras, hero armies against the battle's army slots |

   Hero effects stay core: they are the hero context's vocabulary, which `h3-heroes` translates to
   both seams (ADR-0013 point 2; E08 decision 5), and they select creatures by flags. The table's
   "battle effects" are the spells' timed effects (`spell.rs`). `h3-heroes` then reads the core
   only.
3. **The battle part's shape** mirrors `AdventureRuleset` (ADR-0015 point 2): compiled on a core
   from one `BattlePack` per core pack (no manifest of its own; the folder pairs it), immutable,
   `Arc`-shared, with `core_hash()` and its own `content_hash()` (encoding v1, the core's hash
   not part of it). *Provisional:* `BattleRuleset` also keeps the `Arc<Ruleset>` it was compiled
   on and dereferences to it, so battle code holds one handle; its rows shadow the core's
   accessors of the same name (`creature`, `spell`, `spells`) and carry copies of the core fields
   the battle reads, so call sites stay as they are. Its errors are `BattleCompileError`.
4. **Core accessors stay for the other lanes:** the core gains `creature(id)` returning its row
   (`CreatureInfo`), so the adventure's `core.creature(id).speed` reads on unchanged;
   `creature_info` stays as an alias for now.
5. **Pack files:** `pack.toml` becomes `core.toml` (with the `[pack]` manifest) and `battle.toml`
   (no manifest, like `adventure.toml`); pack format v35. The harvester writes both;
   `--with-original-data` compares both. A creature or spell override splits by field the same
   way; `fixes` gates each half.
6. **Crates and owners:** `h3-content` and `h3-pack` (core) become shared paths (E08 decision 2's
   table); `h3-content-battle` and the new `h3-pack-battle` (battle text, battle scenarios, the
   harvested-pack tests) are the battle lane's. `arch.toml`: `h3-content-battle` is a shared-layer
   logic crate on `h3-kernel` and `h3-content`; `h3-battle`, `h3-sim`, `h3-ffi`, `h3-protocol`,
   `h3-server` and `h3-cli` gain the edge, `h3-adventure`, `h3-game`, `h3-map` and `h3-heroes`
   do not.
7. **Versions:** core ruleset hash v43 (the battle data leaves it), battle part hash v1, pack
   format v35, battle replay format v22 (the header records the core's and the part's hashes).
   State hash, encodings and the C ABI unchanged unless a task finds otherwise. The adventure's
   golden replays record the core's hash: their headers are re-recorded (`cross-lane`).

## Tasks

- **R01a — the writer.** `h3_formats::h3m` (and its example) into `h3_map::write`; its own error
  type; `h3-experiment` depends on `h3-map`; `h3-map`'s round-trip tests use it directly; docs
  (`docs/formats/h3m.md`, `docs/experiments/README.md`, the HANDOFF traps that name it).
- **R01b1 — `h3-content-battle`.** The new crate per decisions 2–4; `h3-content` and
  `h3-content-battle` build and pass their tests at 100 % coverage; tests split with the code.
- **R01b2 — packs.** `h3-pack` (core) and `h3-pack-battle`; the five packs split into `core.toml`
  and `battle.toml`; the harvester; `--with-original-data` re-harvests identically.
- **R01b3 — the dependents.** `h3-battle`, `h3-heroes`, `h3-sim`, `h3-ffi`, `h3-cli`,
  `h3-protocol`, `h3-server` and the adventure crates' tests build and pass; replay headers; golden
  replays re-recorded with unchanged state and event hashes; benchmarks within the gate.
- **R01b4 — closure.** Full verify with the original data; ADR-0006 and ADR-0015 amendments, the
  docs that name moved items (`docs/02` §3, `docs/03`, `packs/README.md`, the HANDOFFs, the global
  traps), BACKLOG; the pull request.

## Other lanes

- **Adventure** (T13 on `adventure/e08-t13`): its crates read core items only; R01b touches its
  tests' pack builders (no battle parameters or battle stats needed any more), `h3-pack-adventure`'s
  pack names and the adventure replays' headers. Whichever merges second rebases.
- **Server** (#130 open): `h3-protocol` and `h3-server` name `BattleRuleset` and
  `h3_content_battle::ObstacleKind`; nothing else.
