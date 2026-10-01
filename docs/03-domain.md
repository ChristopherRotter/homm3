# 03 — Domain model: bounded contexts and language

This is the first cut. Contexts are refined when their epics start. Splitting or merging is allowed but must be recorded (ADR) because it changes crate boundaries.

## Context map

| Context | Responsibility | Depends on |
|---|---|---|
| **Kernel** (shared kernel) | IDs, numerics, PRNG, hashing, errors, modifier primitives (ADR-0012) | — |
| **Content** | Static definitions: creature types, spells, artifacts, skills, hero classes/heroes, town types/buildings, terrain, object types; content packs; `Ruleset` compilation | Kernel |
| **Contracts** | Seam types exchanged between contexts (`BattleSetup`, `BattleOutcome`, …) | Kernel, Content |
| **Battle** | Tactical combat: hex field, stacks, turn order (initiative), movement, attacks, retaliation, morale/luck, damage, obstacles, war machines, battle spells, surrender/retreat, outcome | Kernel, Content, Contracts |
| **Adventure** | Map, tiles/terrain, objects and their behaviors, movement and pathing, fog of war/exploration, resources on map, visiting, guards/monsters, events | Kernel, Content, Contracts |
| **Town** | Buildings and construction, weekly growth, recruitment, town-specific mechanics (mage guild, market, etc.) | Kernel, Content, Contracts |
| **Heroes & Army** | Hero progression (levels, primary/secondary skills, specialties), artifacts and equipment, spell book, army composition/garrison | Kernel, Content, Contracts |
| **Magic** | Spell definitions' effects as used by battle and adventure (may live inside Battle/Adventure initially) | Kernel, Content |
| **Game** (application) | Players, teams, turn/calendar (day/week/month), victory/defeat conditions, diplomacy/alliances, scenario setup, command routing, visibility/projection | all domain contexts |

Outside the domain: **Formats/Import** (anti-corruption layer for original formats and archives), **Sim/AI**, **FFI**, **CLI**, **Server**, **Clients**.

## Seams (the important one: adventure ↔ battle)

Adventure and Battle never import each other.

1. Adventure/Game detect an engagement and build a `BattleSetup`: armies, heroes' *derived battle profile* (attack/defense/power/knowledge, relevant skills, artifacts' effects, spell book and mana), terrain/battlefield type, obstacles, siege info, rules-relevant flags. All derived values are computed **before** the seam, so Battle needs no knowledge of hero management.
2. Battle runs standalone from a `BattleSetup` (this is also what lets the battle AI train in isolation).
3. Battle returns a `BattleOutcome`: winner, casualties, survivors, experience inputs, artifacts/spell-scroll capture, etc. Game applies it to adventure/hero state.

Other seams (town ↔ heroes, heroes ↔ magic, content ↔ all) are introduced the same way: a small contract type, owned by `contracts`, no reach-through.

## Ubiquitous language (glossary — extend as contexts grow)

- **Ruleset** — immutable compiled content + parameters + enabled rule modules for a game.
- **Content pack** — a set of definitions (`base`, `ab`, `sod`, later others) with a manifest, namespaced IDs and load order.
- **Profile** — a named set of quirk toggles/parameters (e.g., `faithful`, `strict-fixed`).
- **Quirk** — a known deviation/bug of the original, classified in the quirks ledger.
- **Stack** — a number of identical creatures acting as one unit (in an army or in battle).
- **Army** — up to N stacks (N is a ruleset parameter, 7 in the original) belonging to a hero or garrison.
- **Hero**, **Town**, **Dwelling**, **Garrison**, **Mine**, **Resource**, **Artifact**, **Spell**, **Skill (primary/secondary)**.
- **Adventure map**, **Tile**, **Object**, **Fog of war**.
- **Day / Week / Month** — the calendar (weekly growth, month/week events).
- **Battlefield**, **Hex**, **Initiative/Speed**, **Retaliation**, **Morale**, **Luck**, **Obstacle**.
- **War machine** — a machine a side brings into battle besides its army (Ballista, First Aid Tent, Ammo Cart; later siege machines): a stack with flag `war_machine` at a fixed hex, controlled automatically without the hero's skill, never retaliating, unaffected by morale; a side with only war machines left has lost.
- **Round** — every living stack acts once (normal phase, fastest first; then the **wait phase**, slowest first). **Active stack** — the stack whose turn it is. **Wait** — postpone the turn to the wait phase (once per round). **Defend** — end the turn with a defense bonus until the stack's next turn. **Attack** — one swing of a stack (its melee attack, a second attack, a retaliation or a shot); it strikes the target and the secondary targets its pattern hits (breath, multi-headed, area shots). **Strike** — the damage application of an attack on one stack. **Formation** — starting rows of an army by stack count.
- **Ability** — a typed behaviour primitive of a creature type (flying, hate, jousting, …), referenced from content (ADR-0012). **Modifier** — a numeric change to a stat of a bearer (stack, side) with a source and an expiry; effective stats are materialised from base stats and modifiers (ADR-0012).
- **Side** (`BattleSide`) — attacker or defender in a battle; the battle context's actor identity.
- Sim-layer terms (not domain language): **episode**, **step**, **truncation** (step limit), **action index**, **observation encoding**, **reward**.
- **Command** — a player's intent, validated and applied. **Event** — a fact emitted by applying a command. **Observation** — a player-specific projection of state. **Actor** — the trusted identity issuing a command.
- **Rule** — a specified piece of game logic with an ID in the rule catalog.

Use these words in code, docs, tests and commit messages. If the original game's term and a technical term differ, the original game's term wins inside domain code.

## Open domain design questions (deferred)

- ~~Shape of the bonus/modifier system in the kernel~~: decided in ADR-0012 (E03): typed **abilities** in content for conditional behaviour, kernel **modifiers** (bearer, stat, op, amount, source, expiry) with materialised effective stats.
- Whether Town and Heroes stay separate crates or start inside Adventure.
- How Magic is split between Battle and Adventure.
- Representation of map objects (closed enum vs. data-driven behavior primitives) — decided in the adventure epic with the ADR.
