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
3. Battle returns a `BattleOutcome`: result (a victory of one side, or a draw when both armies are wiped out at once), casualties, survivors, experience inputs, artifacts/spell-scroll capture, etc. Game applies it to adventure/hero state.

Other seams (town ↔ heroes, heroes ↔ magic, content ↔ all) are introduced the same way: a small contract type, owned by `contracts`, no reach-through.

## Ubiquitous language (glossary — extend as contexts grow)

- **Ruleset** — immutable compiled content + parameters + enabled rule modules for a game.
- **Content pack** — a set of definitions (`base`, `ab`, `sod`, later others) with a manifest, namespaced IDs and load order.
- **Profile** — a named set of quirk toggles/parameters (e.g., `faithful`, `strict-fixed`).
- **Quirk** — a known deviation/bug of the original, classified in the quirks ledger.
- **Stack** — a number of identical creatures acting as one unit (in an army or in battle).
- **Army** — up to N stacks (N is a ruleset parameter, 7 in the original) belonging to a hero or garrison.
- **Hero**, **Town**, **Dwelling**, **Garrison**, **Mine**, **Resource**, **Artifact**, **Spell**, **Skill (primary/secondary)**.
- **Hero** — the person across contexts; never a type name (ADR-0013). **Hero type** (`HeroType`, content) — the definition: class, specialty, starting skills and army. **Hero details** (`HeroDetails`, Heroes context) — a hero's lasting state: level, experience, primary and secondary skills, artifacts, spell book, mana; not the army. **Battle hero** (`BattleHero`, contract) — what a battle needs, all derived from the details before the seam. The hero on the adventure map is an Adventure type (E08).
- **Specialty** — a hero type's built-in bonus (skill, creature, resource, spell, fixed creature stats, speed, creature upgrade, dragons). **Effect** — a typed primitive of an artifact, a skill level or a specialty (`HeroEffect` in content); the translation turns effects into resolved **battle effects** (`BattleEffect`).
- **Artifact type** (`ArtifactType`, content) — the definition of an artifact: class, value, slot, effects, a war machine it places, the parts of a **combination artifact**. **Artifact slot** (`ArtifactSlot`) — a kind of equipment position with a count (two rings, four or five misc positions); an artifact fits any position of its slot, an artifact without a slot is carried in the backpack only. A combination occupies its own slot and **locks** one position of each other part's slot. Which artifacts a hero wears belongs to the hero details: **worn** artifacts (by slot kind, positions interchangeable) and the **backpack** (up to 64 artifacts; never war machines or the spell book), HER-ART-001/002. **Hero parameters** (`HeroParams`, Tier 2) — the limits of hero details (primary skills, backpack).
- **Tactics phase** — the placement phase before round 1 for the side with the higher Tactics level. **Retreat** — a side with a hero leaves the battle and loses its army. **Surrender** — a side with a hero buys its way out of a battle against another hero, keeping its army.
- **Adventure map**, **Tile**, **Object**, **Fog of war**.
- **Day / Week / Month** — the calendar (weekly growth, month/week events).
- **Battlefield**, **Hex**, **Initiative/Speed**, **Retaliation**, **Morale**, **Luck**, **Obstacle**.
- **War machine** — a machine a side brings into battle besides its army (Ballista, First Aid Tent, Ammo Cart; later siege machines): a stack with flag `war_machine` at a fixed hex, controlled automatically without the hero's skill, never retaliating, unaffected by morale; a side with only war machines left has lost.
- **Round** — every living stack acts once (normal phase, fastest first; then the **wait phase**, slowest first). **Active stack** — the stack whose turn it is. **Wait** — postpone the turn to the wait phase (once per round). **Defend** — end the turn with a defense bonus until the stack's next turn. **Attack** — one swing of a stack (its melee attack, a second attack, a retaliation or a shot); it strikes the target and the secondary targets its pattern hits (breath, multi-headed, area shots). **Strike** — the damage application of an attack on one stack. **Formation** — how an army is arranged at the start of a battle: **loose** (spread from top to bottom) or **tight** (clustered around the middle), a per-army choice; the ruleset gives the starting rows by stack count for each.
- **Ability** — a typed behaviour primitive of a creature type (flying, hate, jousting, …), referenced from content (ADR-0012). **Modifier** — a numeric change to a stat of a bearer (stack, side) with a source and an expiry; effective stats are materialised from base stats and modifiers (ADR-0012).
- **Spell school** — air, earth, fire or water; a spell belongs to one or more. **Mastery** — the level (none, Basic, Advanced, Expert) at which a hero casts a spell: the best of its schools' masteries (the magic school skills), raised by an artifact (Armageddon's Blade), derived once per spell in the Heroes context (E04b T8a). **Learned spells** — what a hero keeps whatever it wears; its **available spells** add the spells of its worn artifacts and **scrolls** (an **artifact instance** holding a spell); both the adventure map and the battle use the one derivation in the Heroes context. **Spell book** — what lets a hero cast its available spells (the Spell Book artifact). **Battle magic** (`SpellMagic`, content) — what a spell does in battle: the **target** its caster names (none, a stack, a hex), the stacks a **cast** reaches (an **area** around a hex, a **chain** from a stack, or every stack meeting its **target conditions**) and its **spell effects** (closed kinds: damage, timed, … ; ADR-0014). A **mass version** — from a mastery (usually Expert) the spell takes no target and reaches every stack of its side. **Timed effect** — what a timed spell leaves on a stack for a number of rounds (or the rest of the battle): an entry of the battle's **effect list** (one per stack and spell) with the changes made at the cast, given back when it **ends**; **counter** spells end each other's effects (Bless and Curse).
- **Side** (`BattleSide`) — attacker or defender in a battle; the battle context's actor identity. A stack's side is its owner; its **controller** is the side that commands its turns and whose hero's bonuses it fights with, the other side while it is **hypnotized** (BAT-MAG-026). A **berserk** stack takes automatic turns against the nearest stack and is every stack's **enemy** (BAT-MAG-025); otherwise stacks of different controllers are enemies.
- Sim-layer terms (not domain language): **episode**, **step**, **truncation** (step limit), **action index**, **observation encoding**, **reward**.
- **Command** — a player's intent, validated and applied. **Event** — a fact emitted by applying a command. **Observation** — a player-specific projection of state. **Actor** — the trusted identity issuing a command.
- **Rule** — a specified piece of game logic with an ID in the rule catalog.

Use these words in code, docs, tests and commit messages. If the original game's term and a technical term differ, the original game's term wins inside domain code.

## Open domain design questions (deferred)

- ~~Shape of the bonus/modifier system in the kernel~~: decided in ADR-0012 (E03): typed **abilities** in content for conditional behaviour, kernel **modifiers** (bearer, stat, op, amount, source, expiry) with materialised effective stats.
- Whether Town and Heroes stay separate crates or start inside Adventure.
- ~~Whether the Heroes context gets its own crate~~: direction decided in E04 refinement (OPEN-QUESTIONS Q27): a domain crate `h3-heroes` holds only the hero's lasting details (`HeroDetails`, provisional) and the pure translation to the battle's own `BattleHero` seam type; the hero on the map is an Adventure type (E08); no type is named plain `Hero`. Formalised by ADR-0013 (accepted 2026-10-03); the crate exists since E04a T4a. Town stays open.
- How Magic is split between Battle and Adventure.
- Representation of map objects (closed enum vs. data-driven behavior primitives) — decided in the adventure epic with the ADR.
