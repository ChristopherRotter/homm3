# E08 design note: increment 3

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendment drafts for increment 3 and the ADR-0013 one (E08 T17b;
`docs/lanes/adventure/adr-drafts/`; accepted by the user at ADV-Q13 with its changes). The task
that builds a section deletes it; the file goes when the epic closes.
Facts and sources: survey `docs/rules/surveys/E08-battles.md` (§ numbers below are the survey's
unless they name this note). Increment 2's note went with T15b; its versions are in `E08-log.md`.

## 1. The store of heroes (GAM-HER-001; T18b)

- `GameState.heroes: Vec<Arc<HeroDetails>>` by `HeroId` (a fork costs a reference count per hero;
  `Arc::make_mut` copies one hero on a write; T14b measured the heroes' clone at 15 % of
  `fork-xl-144` with an allocation per army: measure again). Validated with `h3_heroes::validate`
  when the state is built; a `GameSetupError::Hero(index, DetailsError)`.
- `GameHeroSetup` gains `details: HeroDetailsSetup`: each field `Option` (the map's customisation)
  or the default. Defaults (§6.2): primary skills from the class (`HeroClass::primary`), secondary
  skills, spell book and spell from the type, level 1, **experience a setup placeholder**: 40 +
  a draw of 0–50 from the setup stream (D-12; the original's `rand() mod 51` bias is not
  reproduced), drawn in hero order after the starting armies; spell points full (the maximum
  from `h3-heroes`, Knowledge × 10 with Intelligence: new content, §10), worn artifacts and the
  backpack from the map (validated: slots, backpack size), the starting army's war machine as its
  artifact (GAM-SET-001, already drawn). A custom primary skill is copied without the worn
  artifacts' bonuses and clamped (§6.3).
- The import (`h3-map`, `MAP_IMPORT_VERSION` 4) reads the body's customisations (the body wins
  over the map-wide `custom_heroes` entry; `ADV-STUB-1` closed): experience, primary and secondary
  skills, spells, artifacts (worn by slot, the backpack), formation; name, portrait, biography,
  sex and patrol are presentation or computer-player data and stay ignored. `Unsupported`: an
  experience of 1 000 or more, the body's or else the map-wide entry's (decision 22: 42 placed
  heroes; the 15 map-wide entries with a gameplay field all have one, survey §6.1), which needs
  increment 4's levels.
- The formation: kept with the army, `HeroOnMap.formation` (`HeroDetails` holds no army,
  ADR-0013 point 1); the map's flag goes to `BattleSetup.formations` (one shipped hero is tight;
  VCMI resets every hero to loose, the original was not read: E3-9 settles it; until then the
  map's).
- Observation: the owner sees its heroes' details (experience, primary and secondary skills,
  artifacts, spells, spell points); other players see identity, owner and class (§9).

## 2. Reading and changing a hero's details (the hero model, decision 25; T18a, T18b)

- `h3-heroes` in the `shared` layer (`arch.toml`, T18a; the ADR-0013 amendment draft):
  `h3-adventure` and `h3-game` depend on it, `h3-battle` never. No adventure profile (decision 5
  withdrawn).
- Reads: the game passes its store (`&HeroStore`, the heroes' `Arc<HeroDetails>` by `HeroId`) to
  `AdventureState::apply` and the queries (`reach`; the observation for the owner's heroes). The
  adventure reads the visiting hero's details only through `h3-heroes`' derivations: in increment
  3 `attack_defense` (the effective Attack and Defense, as the original's hero record
  +0x476/+0x477 holds them, clamped 0–99 — the building task reads whether +0x476 includes the
  artifacts) and `join_bonus` (0–3, from a new hero-effect kind, VCMI's
  `WANDERING_CREATURES_JOIN_BONUS`: Diplomacy 1, 2, 3 by level, so no code reads the skill by name,
  ADR-0016 point 4); increment 4 adds movement and scouting.
- Writes: `h3_heroes::HeroChange` (increment 3: `GainExperience(u32)`, `SetSpellPoints(u16)`,
  `LearnSpell(SpellId)`, `AddArtifact(ArtifactTypeId)` placed by the capture rule,
  `RemoveArtifact(slot or backpack index)`), each checked by `h3_heroes::check_change` (the caller
  checks first, so the game's `apply_change` is infallible). Increment 3's adventure visits change
  no hero (a joining group changes the army only); the battle's outcome does (§5).
- The game's difficulty reaches the adventure as a static input (`AdventureSetup.difficulty`),
  read only by ADV-MON-007's Easy rule (D-10: every player on Easy).

## 3. Monsters on the map (ADV-MON-001 to -006; T20a)

- Ruleset part v6: kind `Monster { levels: Option<(u8, u8)> }` for the nine classes
  (`adventure.toml`, one type per class: `monster` without levels, `random_monster` 1–7,
  `random_monster_1` … `_7`); instance data from the map validated at setup: the creature (class
  54's subclass mapped by the import; none for a random monster), count (0 = drawn; at most 4 095, the import's bound), disposition (compliant, friendly,
  aggressive, hostile, savage), never flees, does not grow (kept, unused until increment 5),
  treasure resources. The creature's AI value, cost, upgrades and adventure-map range are core
  content already (`CreatureInfo::ai_value`, `cost`, `upgrades`, `adventure_count`).
- Setup draws (setup stream, object order; after the starting armies, the heroes' experience and
  increment 2's contents): a random monster's creature
  (uniform over the allowed ids of its levels: AB and SoD without 132–137, RoE 0–117; levels as
  the executable's under `BAT-QUIRK-19`, §1.4), then the count from `adventure_count` when 0
  (§1.5), then the aggression by disposition (−4, 1–7, 1–10, 4–10, 10; §1.3). All three into the
  static map.
- Per-object state: `army_left` (none until a battle the monster survives writes its count).
- Guarded tiles (ADV-MON-005): the guard of a tile is the first monster in the 3 × 3 scan
  (columns from the left, each from the top; the row above left out where the scanned tile's
  object cannot be entered from above; same water status), recomputed lazily from the static map
  and the occupancy layer (no per-tile flags: a removal frees the tiles). ADV-MON-006: a step onto
  a guarded tile ends the move and visits the guard; `Reach` does not continue from a guarded tile
  but the start (the pathfinder's rule, §2.2).
- Observation (ADV-FOG-006 extended): an `ObjectView` of a monster shows its creature and the
  count's word (0x44ADE0: 1–4 few … 1 000+ legion); never the count, aggression, flags or treasure.
- Ground kinds: `Ground { battlefield: ContentKey }` for classes 21 (Cursed Ground), 46 (Magic
  Plains) and SoD's 222–224 and 226–231 (the battlefield keys of `E04c-battlefields.md` §1; 225,
  Favorable Winds, has none); D-4 (223 and 230 not in 0x4FCE20's list) as a reading of the loader
  settles before T20a builds them (or a batch-E3 row).

## 4. The monster's answer and the offers (ADV-MON-007, -008; T20b)

- Inputs (§3.1): power = ⌊army value × √((1 + 0.05 D)(1 + 0.05 A))⌋ (the army value in 32-bit
  integers, the factor in double precision with its square root stored as a float: `kernel::num`
  operations, ADR-0003); strength = AI value × count; r = power / strength in single precision;
  St by the thresholds with D-8 (r > 0.5 for −1); Sy with D-9 (the own type counts; the related
  type by faction and level, so `BAT-QUIRK-19` breaks the elementals' pairs in `faithful`); L
  `h3_heroes::join_bonus`, +1 (at most 3) on Easy (D-10); M the aggression; never flees.
- The decision (§3.2) as a function returning `MonsterAnswer::{Fight, OfferJoin,
  OfferJoinForGold(price), OfferPursue}` (the monster leaves only after a declined pursuit or by
  joining); the chain continues in `AdventureState::decline_offer` (new, infallible):
  a declined join goes to step 4 (fight at M = C or when it never flees, else the pursue offer).
- Offer kinds (adventure `OfferKind`): `Join` (no price), `JoinForGold { price }` (the creature's
  gold cost × count; not opened when the treasury cannot pay: the game calls `decline_offer`, no
  `OfferRefused` event), `Pursue` (accept: the battle; decline: the monster leaves, no treasure),
  `Fight` (a guarded pile's question). `Answer::{Accept, Decline}` and the env's two answer
  entries serve all of them.
- Joining: the group becomes one stack (merged with a stack of its creature, else the first free
  slot of seven); with neither the join offer is skipped as declined (**`ADV-STUB-2`** at the skip:
  the original's army dialog, 0x5D11D0, not read; a feature to-do for increment 4's army
  management). Then the treasure's resources to the owner (gains), the monster leaves the map.
- Events: the offers as increment 2's (`OfferOpened`, `OfferAnswered`) with the new kinds; a
  monster's flight and joining as removals (`ObjectRemoved`) seen where the tiles are.

## 5. Battles through the game (GAM-BAT-001 to -003; T19a)

- `Decision::Battle(Arc<OpenBattle>)`: the `BattleState`, the sides' owners `[Option<PlayerId>; 2]`
  (attacker the visitor's), the heroes `[Option<HeroId>; 2]`, the origin (`Hero(HeroId)`,
  `Monster(anchor)`, `Guards(anchor)`), the tile. Hashed with the battle's own hash.
- `GameCommand::Battle(h3_battle::Command)`: accepted from the owner of `side_to_act()` (in its
  turn or not), or from `Actor::System` for an unowned side; `GameCommandError::{NotYourSide,
  Battle(CommandError)}`. `legal_commands` maps `h3_battle::legal_commands` for the side to act's
  owner; the other owner gets none while it waits.
- The setup (GAM-BAT-001): attacker = the visitor's army (`h3_contracts::Army` from the hero's
  stacks), defender = the other hero's army, the monster's split (ADV-MON-009) or the guards as
  given; `heroes` from `h3_heroes::derive_battle_hero`; `war_machines` from `h3_heroes::war_machines`;
  `terrain` the defender's tile's; `battlefield` in the original's order (E04c §4): underground →
  subterranean; a tile next to water → sand and the shore; a `Ground` object on the tile replaces
  the field (sieges, boats and mines do not occur in increment 3); `obstacles` from
  `generate_setup_obstacles` with `Rng::from_seed(s)`: under `ADV-QUIRK-5` (`faithful`)
  s = `u64::from(110_291 x + 167_801 y + 81_569)` (wrapping u32 arithmetic, the defender's tile,
  the level not included), else (`strict-fixed`) the attacker's stream's `next_u64`; `funds` the
  owners' gold; formations from the heroes on the map. The battle's seed: `next_u64` of the
  attacker's stream (drawn first, so both profiles draw it alike). The split's two draws
  (ADV-MON-009): the tile's C-runtime generator under `ADV-QUIRK-6`, else two
  `range_inclusive(1, 100)` from the attacker's stream.
- Events: `GameEvent::Battle { event, seen_by: Viewers }` (a side's event to its owner, a public
  one to both owners); the start and the end as game events (`BattleStarted`, `BattleEnded` with
  the result) seen by the owners; other players see the removals afterwards.
- Observation: `GameObservation.battle: Option<h3_battle::Observation>` for its owner's side.
  The neutral seat's policy runs in the trusted host and may read the whole `BattleState` (the
  original's battle AI does; the battle lane ports it, decision 27); the env's stand-in plays at
  random.
- Applying the outcome (GAM-BAT-002, -003; §8, `E04-battle.md` §4): survivors to the armies (a
  damaged top creature forgotten); experience to the winner's hero only: killed creatures' hit
  points (no war machines) + 500 for a defeated hero (not after a retreat or surrender), ×
  (1 + Learning), rounded down; Necromancy per killed stack, rounded down (`E04-battle.md` §4.5;
  skeletons by the skill, the Cloak in increment 4); `HeroOutcome::learned` (Eagle Eye) into the
  spells; `HeroOutcome::mana`; a destroyed war machine's artifact removed (the catapult stays);
  the loser's artifacts (worn and backpack, not war machines nor the spell book) to the winner,
  each into its first free fitting slot, else the backpack (VCMI's placement, decision 22);
  a hero that lost, retreated or surrendered leaves the map, a draw both; a beaten monster's
  treasure (resources) to the winner's owner and the monster off the map (ADV-MON-010); a monster
  that survives keeps every stack's survivors, an upgraded stack's as its own creature (A-6 fixed,
  decision 23). The heroes' part as `HeroChange`s through `h3-heroes` (§2).
  One infallible function per context, called in a fixed order (adventure, heroes, treasuries).
- Not in increment 3: CLT-Q4's background variant (trees, mountains and dead vegetation within 3
  tiles) needs those decorations typed (E12's long tail); viewers keep the third picture.

## 6. Hero against hero (GAM-BAT-004, -005; T19a)

- The visit (ADR-0016 point 3 amended): a step toward another player's hero visits it from the
  neighbouring tile at the step's price (as ADV-OBJ-003) and starts the battle at once; both
  players own it. A surrender's price (`BattleOutcome::surrender_cost`) leaves the loser's owner's
  gold and enters the winner's owner's (D-13; GAM-RES-001's saturation).
- Ownership isolation: only the attacker's stream moves (the seed); the defender's commands draw
  from the battle's stream, which belongs to the battle.

## 7. Guarded piles (ADV-PIK-004; T21)

- The import: an empty guards record is a plain pile (3 shipped); guards up to seven stacks with
  an optional text (the text itself is presentation: a flag); D-6's `Unsupported` lifted.
- The visit (from the neighbour, as a pile): a question → `Fight` offer; no question → the battle
  at once; guards unsplit; a win picks the pile up, a loss or retreat leaves the guards' survivors
  in `army_left`.

## 8. The env and replays (T19b)

- Action encoding v3 (`h3-sim/src/adventure/action.rs`): `Action::{Adventure(adventure::Action),
  Battle(h3_sim::action::Action)}`, a phase per state; the masks of the other phase are empty.
- `to_move()` yields seats (`Seat::{Player(PlayerId), Neutral}`): the acting players without a
  battle and, for each open battle, its side to act's owner or the neutral seat; `step` for the
  neutral seat takes a battle action and issues it as `Actor::System`. `random_rollout` plays
  every seat at random; the original's battle AI replaces the neutral seat's stand-in when the
  battle lane has ported it (decision 27), and until then results against monsters are
  provisional.
- Replay format v3: each command with its actor; golden replays with hero battles (T19b) and
  monsters (T20b); the property plans (`crates/h3-map/src/tests/play.rs`) play battles to the end.
- Benchmarks: the three workloads re-baselined with battles; `fork-xl-144` with a battle open.

## 9. Versions

Game state hash v5 (heroes' details, battles, offer kinds); adventure state hash v7 (the
formation, monsters' static data and army left, the difficulty); adventure ruleset part v6 (monster and
ground kinds); map import v4 (monsters, guarded piles, hero customisations); action encoding v3;
adventure replay format v3. Each bumps once in the task that first changes it.

## 10. Shared and cross-lane work

- `shared/` (T18a): the accepted ADR amendments applied (ADR-0013, -0015, -0016, -0017 and the
  glossary); `arch.toml` (`h3-heroes` in the `shared` layer; `h3-adventure` → `h3-heroes`;
  `h3-game` → `h3-heroes`, `h3-battle`, `h3-content-battle`); `h3_content` hero-effect kinds for Necromancy (raise share by
  level), Learning (experience share), Intelligence (spell points share) and Diplomacy's join
  bonus (§2).
- `cross-lane` (battle-owned): the effects' values in the packs (the harvester's gap-fill; the
  skills' levels, Isra's and Vidomina's specialties, the necromancy artifacts' parts for increment
  4), `h3-heroes`' additions (`attack_defense`, `join_bonus`, the maximum spell points, the Necromancy
  share, the Learning factor, the capture's placement, `HeroChange` and its check); `h3-experiment`'s spec for batch E3
  (monsters, custom heroes, the difficulty); the client lane's recipes (its inbox, 2026-10-13).
  **The handover of `h3-heroes`**: as soon as the battle lane is done with it (the user,
  ADV-Q13; the battle lane's inbox asks it to say when), a `shared/` change of `lanes.toml`; the
  additions above are `cross-lane` until then.
- The battle lane (its inbox): the original's battle AI, the faithful policy of the neutral seat
  and of a computer player's battles (decision 27).

## 11. The building tasks (confirmed at ADV-Q13)

| Round | Tasks | Content |
|---|---|---|
| 1 | T18a + T18b | the shared groundwork with the ADR amendments (§10); the store of heroes, defaults, reads and changes, the import of hero customisations (§1, §2) |
| 2 | T19a + T19b | battles through the game, hero against hero, the unowned side (§5, §6); the env, replays and properties with battles (§8) |
| 3 | T20a + T20b | monsters on the map, guarded tiles, grounds (§3); the monster's answer, offers, joining, the split, its battle (§4) |
| 4 | T21 + T22a | guarded piles and the maps with monsters, replays, benchmarks, the audit (§7); batch E3's spec and run (cross-lane, the client lane's recipes) |
| 5 | T22b | batch E3 scored, increment 3 closed, increment 4 refined |
