# E08 design note: increment 3

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 3's
amendments of ADR-0013, -0016 and -0017 (drafted in E08 T17b, accepted by the user at ADV-Q13
with its changes, applied by T18a, #185). The task that builds a section deletes it; the file goes
when the epic closes.
Facts and sources: survey `docs/rules/surveys/E08-battles.md` (§ numbers below are the survey's
unless they name this note). Increment 2's note went with T15b; its versions are in `E08-log.md`.

## 1. The store of heroes (GAM-HER-001)

Built by T18b (2026-10-14): `h3_game::GameState`'s store, `HeroDetailsSetup`, the import of the
customisations (`docs/formats/h3m.md`, "Import"), the formation on `HeroOnMap`, HER-MAN-001; the
formation into `BattleSetup.formations` since T19a1.

## 2. Reading and changing a hero's details (the hero model, decision 25)

`h3-heroes` is in the `shared` layer since T18a. Moved at the round-1 checkpoint (decision 28)
to the tasks that first use them:
- **T19a2a** built (2026-10-14; decision 32): `h3_heroes::HeroChange` (`GainExperience`,
  `SetSpellPoints`, `AddArtifact` placed by HER-CHG-002, `RemoveArtifact(ArtifactPlace)`),
  `check_change` and `apply_change` (HER-CHG-001; the game applies only checked changes),
  `gained_experience` (HER-EXP-001), `next_capture` (HER-ART-004), `lost_war_machine`
  (HER-WAR-002). **T19a2b**: `LearnSpell(SpellId)`, the Necromancy share, the raised creature.
- **T20a1** built (2026-10-14; decision 35): `h3_heroes::attack_defense` (HER-ADV-001: the
  battle hero's effective Attack and Defense, HER-BAT-001, clamped 0–99; read: the original's
  hero record +0x476 to +0x479 holds the worn artifacts' bonuses, `hero_equip_artifact` adds them
  and `hero_remove_artifact` subtracts them) and `join_bonus` (HER-ADV-002: 0–3 from the
  `join_bonus` kind, Diplomacy's 1, 2, 3 in the packs, so no code reads the skill by name,
  ADR-0016 point 4).
- **T20a2a** built (2026-10-15; decision 36): the game's difficulty as a static input
  (`AdventureSetup.difficulty`, `AdventureState::difficulty`, in the state hash's bump to v9), read
  only by ADV-MON-007's Easy rule (D-10, T20b).
- **T20a2b** built (2026-10-15): the game passes its store (`&[Arc<HeroDetails>]` by `HeroId`)
  to `AdventureState::apply` and `reach`; the adventure reads the visiting hero's details only
  through `h3-heroes`' derivations (in increment 3 the two above; T20b's answer is the first
  reader, through the guard's visit). Increment 4 adds movement and scouting.
- Increment 3's adventure visits change no hero (a joining group changes the army only); the
  battle's outcome does (§5).

## 3. Monsters on the map (ADV-MON-001 to -006; T20a)

Built by T20a2a (2026-10-15; decision 36; `E08-log.md`): the monster and ground kinds with the
monsters' parameters and `not_random_monsters` (part v7), the import of monsters and grounds
(`docs/formats/h3m.md`, v6), the setup's checks, the draws (ADV-MON-002 to -004), the static map's
monsters and drawn tiles, `army_left` (written from T20b), what a viewer sees (ADV-FOG-006), a
ground's battlefield (ADV-GRD-001; the topmost, the last in map order); adventure state hash v9.
T20a2b built guarded tiles (2026-10-15): `guard_of` (ADV-MON-005, the original's scan), a step
onto a guarded tile the path's last and `Reach` ending there (ADV-MON-006); the guard's visit is
`ADV-STUB-5` until T20b.

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

Built by T20b1 (2026-10-15; decision 37; `E08-log.md`), as above with these details: the answer
(`answer.rs`, `Temper`) computed in `kernel::arith::X87` (the factor's terms and product in double
precision, the root and the army's value stored as floats, the ratio divided at 53 bits and stored
as a float); the related type by the content's upgrades (the first upgrade, else the creature whose
first upgrade it is with its faction and level); the price resource a parameter of the monsters
(`[params.monsters] price_resource`, part v8); the offers opened, chained and declined by the game
(`GameState::settle_outcome`; game state hash v7); `AdventureEventKind::CreaturesJoined` to the
owner. A step that enters a guarded tile whose own object makes an offer defers the guard's visit
(`AdventureOutcome::guard_after`, kept on the decision): the original asks the object's question
first, and a player has at most one open decision; the game calls `AdventureState::visit_guard`
after the answer, or at once when the offer does not open. A fight and an accepted pursuit do
nothing until T20b2 (`ADV-STUB-6`).

Built by T20b2 (2026-10-15; decision 37): a fight (and an accepted pursuit) returns a
`BattleStart` whose defender is `Defender::Monster { creature, count }`, unsplit, at the
monster's tile; the game splits it (`h3_adventure::split`, ADV-MON-009) with the draws its
profile gives (`TileRand`, the C runtime's `rand` seeded by the tile, under `ADV-QUIRK-6`, flag 24;
else the attacker's stream after the battle's seed, before the obstacles'), the defending side
without a hero and without an owner; `Actor::System` plays it (`GameCommand::Battle` to the first
battle whose side to act is unowned, `GameState::unowned_battle`; any other command
`NotForSystem`); at the end `AdventureState::end_monster_battle` (ADV-MON-010) keeps the
survivors as the monster's count (A-6 fixed, `ADV-QUIRK-7`) or removes it, with the treasure
after a victory. The env's neutral seat plays the unowned side since T20b3a (`Seat::Neutral`,
`ADV-STUB-7` closed); replay format v4 (`system battle …`) came with T20b2, since the writer must
name the new actor.

## 5. Battles through the game (GAM-BAT-001 to -003; T19a1, T19a2)

Built by T19a1 (2026-10-14; decision 29): `Decision::Battle(Arc<OpenBattle>)` (state, owners,
heroes, tile; game state hash v6), `GameCommand::Battle` from the owner of the side to act
(`NotYourSide`, `NoBattle`, `BattleOpen`, `Battle(CommandError)`), `legal_commands` mapped,
`GameEvent::{BattleStarted, Battle, BattleEnded}` seen by the owners, `GameObservation.battle`;
the setup from the visit (`h3_adventure::BattleStart`: both sides' armies and formations, the
tile, `AdventureState::battle_ground`: the part's `BattleTerrains`, underground then the coast,
whose flag the static map keeps since import v5), the battle heroes with their maximum spell
points, the war machines, the funds, the seeds (`ADV-QUIRK-5`, flag 23); the outcome's armies,
the heroes leaving the map (`HeroRemoved`) and a surrender's gold (GAM-BAT-005). Left:
- **T19a2a** built (2026-10-14): after the adventure's part, `GameState::heroes_after_battle`:
  each side's spell points and destroyed war machines' artifacts, the staying winner's
  experience (the loser's army and raised records, +500 after a victory over a hero, Learning)
  and the capture, each change told to the hero's owner (`GameEvent::HeroChanged`). The reading
  (2026-10-14): `battle_end` 0x477470 gives a winner with a hero the experience (0x46A350:
  −500 again after a retreat or surrender, × the Learning getter 0x4E4840 on f32 points), the
  raised creatures, Eagle Eye, then the capture 0x46A070 (nothing after a retreat or surrender;
  worn by position skipping the book, the Grail and the war machines, then the backpack from
  its end, each through `hero_give_artifact` 0x4E3070: equip, else the backpack's end, at most
  64; it stops at the first the winner cannot take). Not built: a captured part completing a
  combination (SoD asks a human, builds at once for the computer), Titan's Thunder's spell book
  (0x4E2A64), the Grail's exclusion (the packs do not mark it).
- **T19a2b** built (2026-10-14, decision 33): `h3_heroes::necromancy` (HER-NEC-001, -002),
  `raised_count` and `AdventureState::raise` (GAM-BAT-006, `CreaturesRaised` to the owner),
  `learn_level` and `learns_spell` with `HeroChange::LearnSpell` (HER-SPL-005, -006, GAM-BAT-007),
  after the experience and before the capture; the data in `shared/e08-t19a2b`. The plan, as
  built: Necromancy per killed stack, rounded down (`E04-battle.md` §4.5),
  the raised creature by the `raises` kind (decision 30). D-9 read in the code (2026-10-14):
  Necromancy (0x4775E1) counts every killed record, summoned stacks, clones and war machines too
  (E04c batch 2d NE:7): per record `min(killed, trunc(f32(min(hp, raised_hp) × killed) × rate /
  f32(raised_hp)))`, at least 1 in all if anything was killed, the winner only, the rate at
  most 1.0; the power (0x4E3CD0): the skill's share × (1 + 0.05 × level) for the specialist,
  +0.05/0.10/0.15 for artifacts 54–56 only with the skill, +0.10/0.20 by town buildings, +0.30
  for the Cloak (130) only without it; the creature (0x4E3C60): Skeleton, with the Cloak by the
  Necromancy level (0 Skeleton, 1 Walking Dead, 2 Wight, 3 Lich); a full army (0x469F20): the
  upgrade at ⌈2c/3⌉ ((2c + 2) / 3) if it has a free slot, the elementals 112–115 excepted, else
  nothing. Eagle Eye: `HeroOutcome::learned` into the winner's spells, with a worn spell book,
  of a level its Wisdom allows (BAT-Q24: Wisdom + 2), a `shared/` Wisdom kind first. Checked again
  at the build: the count's x87 arithmetic per record (0x477643: fild, fstp f32, fmul the share,
  fdiv f32(raised hp), ftol; `X87::over`), the Cloak's 30 % without the skill a rule (decision 33).
  Not built: the Necropolis buildings' +10/+20 % (E09), the elementals' exception to the upgrade.
- **T20a2a** built (2026-10-15): a ground on the tile replaces the field (ADV-GRD-001); **T20b2**
  built (2026-10-15): the unowned side (`Actor::System`; decision 29), the monster's split and its
  survivors (ADV-MON-009, -010, A-6), a beaten monster's treasure (§4).
- Not in increment 3: CLT-Q4's background variant (trees, mountains and dead vegetation within 3
  tiles) needs those decorations typed (E12's long tail); viewers keep the third picture.

## 6. Hero against hero (GAM-BAT-004, -005; T19a1, T19a2)

Built by T19a1: the visit from the neighbouring tile at the step's price, the battle at once, both
players own it; the visit concerns the hero whatever object lies under it (only leaving the
visitor's tile across its top is refused); a surrender's price from the loser's owner to the
winner's (D-13). Only the attacker's stream moves. T19a2a added the captured artifacts and the +500 (§5).

## 7. Guarded piles (ADV-PIK-004; T21a)

- The import: an empty guards record is a plain pile (3 shipped); guards up to seven stacks with
  an optional text (the text itself is presentation: a flag); D-6's `Unsupported` lifted.
- The visit (from the neighbour, as a pile): a question → `Fight` offer; no question → the battle
  at once; guards unsplit; a win picks the pile up, a loss or retreat leaves the guards' survivors
  in `army_left`.

Built by T21a (2026-10-15; decision 41; `E08-log.md`): `Guards` (`ObjectSetup.guards`,
`MapObject.guards`; checked as a hero's army, no war machine, piles only), the import (map import
v7; a random stack among the guards unsupported, none shipped), `OfferKind::Fight`,
`Defender::Guards`, `AdventureState::{guards_left, end_unowned_battle}`: the battle at the pile's
tile (the code reading: the visited tile's position, 0x4A4886); a draw leaves the pile without
guards, so the next visit picks it up; the plans' guarded piles (`play.rs`'s `GUARDED`); adventure
state hash v10, game state hash v8.

## 8. The env and replays (T19b1, T19b2)

Built by T19b1 (2026-10-14, decision 34): action encoding v3 (`Action::{Adventure(MapAction),
Battle(h3_sim::action::Action)}`, `Phase` by whether the player is in a battle, each phase's masks
empty in the other; `ADV-STUB-4` closed: another player's hero's tile is a legal destination);
`to_move` with each open battle's owner of the side to act; the env's battle masks; the random
agent plays a battle with the battle's own agent (`h3_sim::rollout::choose`: it never retreats or
surrenders); replay format v3 (`Actor` per command, `<player> battle <command>` in the file); the
golden replays re-recorded, two with hero battles; the game's battle sink passes `RECORDS`; the
fork workload with a battle open (the heroes seek each other: random play seldom fights). Built
by T19b2 (decision 35): the property plans play battles to a victory (the driver's runs of
fighting steps, its hero casts; captures, Necromancy and Eagle Eye's learning happen in them).
Built by T20b3a (decision 38): the neutral seat (`Seat::Neutral`, its battle actions issued as
`Actor::System`; decision 29), `to_move` yielding seats (`ADV-STUB-7` closed), its masks, steps and
observation; `random_rollout` and the recorder play it at random until the battle lane has
ported the original's battle AI (decision 27). Built by T20b3b1 (decisions 39, 40): the property
plans with monsters and grounds, the driver playing the system actor, the checks of joins,
flights, treasures and monster battles; `MonsterChanged`; a monster shows on the fringe in either
profile. Left:
Built by T20b3b2 (decision 41, after T21a): the fixture maps with the plans' monsters, guarded
piles and grounds (beside each hero a monster and a pile; none within two tiles of a hero), nine
golden replays (four with their options, five searched: monsters joining, fleeing, pursued,
beaten and winning, piles asking, fought at once, beaten and winning, hero battles in both
profiles), the env's and the CLI's tests on the changed duel, the three benchmarks re-baselined.

## 9. Versions

Shared (T19a2b): core ruleset hash v44, pack format v40. Game state hash v8 (heroes' details since T18b, battles since T19a1; the monsters' offer kinds since T20b1; a guarded pile's offer since T21a); adventure
state hash v10 (the formation since T18b, the coast since T19a1; the monsters, drawn tiles, armies
left and the difficulty since T20a2a; the guards since T21a); adventure ruleset part v8 (the battle terrains since
T19a1; the monster and ground kinds and the monsters' parameters since T20a2a; the price resource
since T20b1); map import v7 (hero customisations since T18b, the coast since T19a1, monsters and
grounds since T20a2a, the piles' guards since T21a); action encoding v3 (since T19b1); adventure replay
format v4 (actors since T19b1, the system actor since T20b2). Each bumps once in the increment,
in the task that first changes it.

## 10. Shared and cross-lane work

- Done: T18a (#185: the ADR amendments, `arch.toml`, the kinds `necromancy`, `experience_gain`,
  `max_mana`, `join_bonus`, pack format v37); T18b (the harvester reads Necromancy's, Learning's
  and Intelligence's tables; `h3_heroes::max_mana`); T19a (`shared/e08-t19a`: `ADV-QUIRK-5`'s
  flag, `h3-map`'s battle part for its tests, battles in `06-security.md`; T19a1 `cross-lane`:
  the flag on in `base:faithful`, the harvester's profile); `shared/e08-t19a2` (#194: `raises`);
  `shared/e08-t19a2b` (`cross-lane`, decision 33: `learn_level`, `unless_skill`, `X87::over`,
  the harvested Wisdom, `raises` and Necromancy artifacts, the replays, the heroes' changes in
  `06-security.md`); `shared/e08-t19b2` (`h3-battle` as `h3-map`'s dev-dependency); T20a1
  (`cross-lane`: `attack_defense`, `join_bonus`, Diplomacy's values in the gap-fill and the packs).
- `cross-lane` (battle-owned), still to come: `h3-heroes`' additions of later increments (built:
  `HeroChange`, its check, the Learning factor and the capture in T19a2a, Necromancy and the
  learning in T19a2b, the adventure's reads in T20a1); `h3-experiment`'s spec for batch E3 (monsters,
  custom heroes, the difficulty); the client lane's recipes (its inbox, 2026-10-13).
  **The handover of `h3-heroes`**: as soon as the battle lane is done with it (the user,
  ADV-Q13; the battle lane's inbox asks it to say when), a `shared/` change of `lanes.toml`; the
  additions above are `cross-lane` until then.
- The battle lane (its inbox): the original's battle AI, the faithful policy of the neutral seat
  and of a computer player's battles (decision 27).

## 11. The building tasks (confirmed at ADV-Q13)

| Round | Tasks | Content |
|---|---|---|
| 1 | T18a + T18b | done (2026-10-14): the shared groundwork with the ADR amendments (§10); the store of heroes, its defaults, the import of hero customisations (§1), trimmed at the checkpoint (decision 28) |
| 2 | T19a1 + T19a2 | split at the checkpoint (decision 29): T19a1 done (2026-10-14): battles through the game, hero against hero, the armies' and the map's outcome (§5, §6); T19a2 split again (decision 32): T19a2a done (2026-10-14), the heroes' outcome without pack changes; T19a2b done (2026-10-14, decision 33), Necromancy and Eagle Eye (§2, §5); round 2 closed |
| 3 | T19b1 + T19b2 | split at the checkpoint (decision 34): T19b1 done (2026-10-14), the env, replays and benchmarks with battles, the versions (§8); T19b2 done (2026-10-14, decision 35), the property plans to a victory; round 3 closed |
| 4 | T20a + T20b | T20a split (decision 35): T20a1 done (2026-10-14), the heroes' groundwork (`attack_defense`, `join_bonus`, Diplomacy's values; with T19b2), T20a2 split again (decision 36): T20a2a done (2026-10-15), monsters and grounds as data with every bump; T20a2b done (2026-10-15), guarded tiles and the store passed through; T20b split in three (decision 37): T20b1 done (2026-10-15), the answer and its offers; T20b2 done (2026-10-15), the monster's battle; T20b3 split (decision 38): T20b3a done (2026-10-15), the neutral seat; T20b3b split (decision 39): T20b3b1 done (2026-10-15), the plans with monsters; T20b3b2 done (2026-10-15, after T21a, decision 41), the maps, replays and benchmarks with monsters and guarded piles; round 4 closed. Monsters on the map, guarded tiles, grounds, the adventure's reads of the store, `attack_defense`, `join_bonus`, the difficulty (§2, §3); the monster's answer, offers, joining, the split, its battle, the unowned side (§4) |
| 5 | T21 + T22a | T21 split (decision 41): T21a done (2026-10-15), guarded piles (§7), before T20b3b2 so the maps, replays and benchmarks change once; T21b done (2026-10-19, decision 42), the audit; T22a split (decisions 43, 44): T22a1 done (the tools), T22a2a done (2026-10-20, the monster rows' batches E3a and E3b with their recipes), T22a2b next (the hero rows), then T22a3 the run |
| 6 | T22b | batch E3 scored, increment 3 closed, increment 4 refined |
