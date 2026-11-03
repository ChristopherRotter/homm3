# ADR-0017 amendment (draft): towns in the game: the setup, the pool, the build, the day, sieges

- Status: Draft (E09 T2a, 2026-11-02), with the draft ADR-0028 beside it; the user's decisions 2,
  6, 8 to 12, 14 and 16 (TWN-Q1, TWN-Q2) and 17 to 20 (TWN-Q3). Accepted at T2b's checkpoint
  (`TWN-Q4`), it goes into ADR-0017 through the `shared/` pull request that creates the crates;
  the code is the town lane's `src/town*` of `h3-game`, the state struct and hash `cross-lane`.
- Amends: point 1 (the state, the setup, the expansions), point 3 (the day's start), point 4 (the
  setup stream), point 9 (the versions), point 10 (outcomes), point 11 (battles).
- Working detail: the survey `docs/rules/surveys/E09-towns.md` §2, §4, §5, §8, §9, §11; the
  design note `docs/epics/E09-design.md` (T2b).

## Context

ADR-0017 keeps "towns later" in `GameState`, leaves "the pool and the tavern" to E09, has the
Plague change nothing before towns, and lets "a rule the packs cannot express yet (the moat,
E09)" gain data or a configuration input. E09's increment 1 brings the store of towns
(ADR-0028), the players' faction choice and starting-hero pick (decisions 6 and 9), the pool and
its draws for the main town's and the random hero (decision 8, T-2 to T-4 by decision 12), the
towns' income with the silo (decision 10), capture and sieges (decision 16).

## Decision (amendment)

1. **Point 1, the state:** `GameState` holds the **store of towns** (`h3-towns`' `Towns`,
   ADR-0028 point 4) beside the adventure's state, and the **hero pool** (decision 18) beside
   the store of heroes: per hero type of the core ruleset whether it is free, taken (by a
   colour, or as a prisoner) or out of the game (the map does not allow it), and the colours that
   may take it (SoD's disposed heroes' masks); one block shared by forks and copied on a write.
   **The pool's life:** built when the game is built from the setup's allowed heroes and masks;
   every placed hero's and prisoner's type taken, then each player's **pick reserved**, before
   any draw (T-2 and T-3 fixed: the original marks neither in time, decision 12); a drawn or
   generated hero's type taken; a hero that leaves the map in play goes back free (the
   original's 0x4D9EC0; nothing draws from the pool in play before increment 3's Tavern, which
   also takes `ADV-STUB-9`'s retreat facts).
2. **Point 1, the setup:** `GameSetup` gains the towns' setups (`h3-towns`', joined to the
   adventure's town objects by `TownId`) and, per player, its **allowed factions**, its **faction
   choice** (a faction or Random), its **starting-hero pick** (a hero type or none), its **main
   town** with whether it generates a hero, and its **random-hero flag**; the game also gains
   the **allowed heroes** and the **colour masks** (ADR-0015's amendment). The game validates
   (one check per invariant, ADR-0015 point 4): a choice is a faction of the player's allowed
   set that has a town record in the ruleset; a pick is in **the original's offer** (decision 9:
   for a player whose choice is a faction, not Random, and whose main town generates a hero or
   who has the random-hero flag, the first 16 in type order of the free heroes of the faction's
   two classes it may take), and no two players pick one hero. A random hero's owner is a player
   of the game.
3. **Point 1, the expansions:** the moat and Conflux are data of the packs (ADR-0028 point 7);
   nothing in the game reads a version, so "the moat, E09" is closed. The factions a Random
   choice or a neutral random town takes are the ruleset's with a town record, a player's within
   its allowed set (decision 14: with AB or SoD enabled, an RoE-format map's player whose eight
   bits are all allowed may have Conflux; one restricted to fewer stays restricted).
4. **Point 4, the setup stream's new placeholders,** drawn when the game is built **before every
   existing one** (a drawn hero's army and experience read its type; a random dwelling of E08's
   increment 10 reads its town's faction), in this order:
   1. each player's Random choice, in colour order: uniform over its allowed factions;
   2. in object order, each random town's faction (survey §2.3, TD-1 by the reading: an alignment
      to a colour that plays gives that colour's faction, else its owner's, else uniform over the
      ruleset's factions) and each random hero's type (its owner's pick for the owner's first
      random hero, else the draw below);
   3. each main town's hero, per player in colour order when its main town generates one: a
      human player's pick, else the draw; it stands on the town's entrance as its visitor;
   4. the towns in `TownId` order: a standard set's level-2 dwelling (the part's chance, 30 in 100,
      survey §5.3, TD-3 by the reading), a neutral town's garrison without one of the map's (four
      rolls by the part's odds, TD-4, TD-5 by the reading), a garrison's random stacks by the
      town's faction;
   then the existing placeholders in their order (the starting armies, the drawn heroes' with
   them, the default experiences, the objects' contents, the level seeds last).
   **The draw** (`hero_draw_of_faction`, finding 10): uniform over the free types of the
   faction's two classes the player may take, else over every free type it may take, else a
   typed failure of the build (T-4, decision 12: no shipped map reaches it); no tavern weights.
5. **Point 3, the day's start:** from day 2 each town pays its owner, with E08's income
   (GAM-RES-004's path), the day's income `h3-towns` derives (the gold by its highest hall and the
   Grail's, the Resource Silo's resources; decision 10); a neutral town pays nobody. The Plague
   and the week's creature stay increment 3's.
6. **Point 9, the versions:** the town lane's rows (ADR-0028 point 9). Increment 1 bumps the game
   state hash (the store of towns, the pool), the adventure state hash and ruleset part (the town
   type and kind), the map import and the adventure replay format (the town part's hash in its
   header), each `cross-lane`; the env's encodings are the AI lane's.
7. **Point 10, outcomes:** increment 1 adds **no command and no decision kind**: entering, capture
   and the siege are steps of `MoveHero` (ADR-0016's amendment). The step's outcome names the
   town entered, captured, or the siege to start (the town and its defender); the game applies a
   **capture** in both contexts at once and infallibly: the adventure's (the owner, the garrison
   emptied, the reveal by the town's radius, which `h3-towns` gives) and the store's (the
   **Capitol's limit**: a captured town with a Capitol whose new owner holds another becomes a
   City Hall, survey §8.4; the start keeps every Capitol the map gives).
8. **Point 11, sieges:** the game builds a siege's `BattleSetup` as any battle's, with
   `siege: Some(..)` from `h3-towns`' derivation (ADR-0023's amendment) and the defending hero's
   town bonuses (ADR-0023 point 3) from the same derivation. The defender is the visitor with the
   garrison merged into its army first (ADR-0016's amendment, TD-7), else the garrison as
   `Defender::Garrison`, played by the town's owner (who answers outside its turn, as a mine's
   owner) or the neutral seat. An attacker's win captures the town (point 10) with the attacker
   outside; the survivors stay in the armies that fought; a beaten hero leaves the map and goes
   back to the pool. A siege of an ally's visitor is refused by the step (a typed refusal, a stub
   until E10, decision 16).
9. **Point 2, viewers:** the observation adds to each town a viewer sees the public facts and the
   garrison by the viewer's level (ADR-0016's amendment, point 6), which the game derives: the
   viewer's towns with a Tavern from the store and the owners, the owner's team, a hero within
   reach with expert Visions or a Rogue in its army.

## Consequences

- `GameState::new` grows the pool and the town draws; their order is part of the game state hash
  and the replays (each adventure golden replay re-recorded with the bump).
- Batch TWN-1 (T3) confirms TD-3, TD-5, TD-7 and a siege's aftermath (decision 15): confirmed
  (T3c, survey §15); its building 23 involves no draw and leaves step 4 as it is (decision 25).
- E10's scenario setup keeps the rest of the setup (teams, conditions, the computer's choices);
  the faction choice and the pick are its seam with E09 (decision 6).

## Alternatives considered

- The original's unreserved pick and late marks (T-2, T-3): one hero could stand twice on the map
  (decision 12: fixed, not a flag).
- The draws after the existing placeholders: the drawn heroes' armies and experiences would need
  a second walk.
- Every starting hero drawn until E10; the pick for every colour (TWN-Q2, decision 9).
