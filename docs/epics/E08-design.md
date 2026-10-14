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
- **T19a2** (the battle's outcome, the first writer; decision 29): `h3_heroes::HeroChange` (increment 3:
  `GainExperience(u32)`, `SetSpellPoints(u16)`, `LearnSpell(SpellId)`, `AddArtifact(ArtifactTypeId)`
  placed by the capture rule, `RemoveArtifact(slot or backpack index)`), each checked by
  `h3_heroes::check_change` (the caller checks first, so the game's `apply_change` is infallible);
  the Necromancy share and the Learning factor (their pack values are harvested since T18b).
- **T20a** (before T20b's monster answer, the first reader): the game passes its store
  (`&[Arc<HeroDetails>]` by `HeroId`) to `AdventureState::apply` and the queries (`reach`); the
  adventure reads the visiting hero's details only through `h3-heroes`' derivations: in increment
  3 `attack_defense` (the effective Attack and Defense, as the original's hero record
  +0x476/+0x477 holds them, clamped 0–99 — read whether +0x476 includes the artifacts) and
  `join_bonus` (0–3 from the `join_bonus` kind: Diplomacy's 1, 2, 3 into the packs then, so no
  code reads the skill by name, ADR-0016 point 4); the game's difficulty as a static input
  (`AdventureSetup.difficulty`), read only by ADV-MON-007's Easy rule (D-10). Increment 4 adds
  movement and scouting.
- Increment 3's adventure visits change no hero (a joining group changes the army only); the
  battle's outcome does (§5).

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
  Favorable Winds, has none); D-4 settled by the battle lane's batch 3a (E04c T3): 223 and 230
  fight on Cursed Ground and Magic Plains. The import keeps grounds as untyped objects with no
  blocked tiles today (`h3-map`'s writer places them since E04c T3).

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
- **T19a2**: the heroes' part of the outcome as `HeroChange`s (§2), each into the store after
  the adventure's part (`ADV-STUB-3` at `GameState::end_battle`): experience to the winner's hero
  only: killed creatures' hit points (no war machines) + 500 for a defeated hero (not after a
  retreat or surrender), × (1 + Learning), rounded down; Necromancy per killed stack, rounded
  down (`E04-battle.md` §4.5; skeletons by the skill, the Cloak in increment 4);
  `HeroOutcome::learned` (Eagle Eye) into the spells; `HeroOutcome::mana`; a destroyed war
  machine's artifact removed (the catapult stays); the loser's artifacts (worn and backpack, not
  war machines nor the spell book) to the winner, each into its first free fitting slot, else
  the backpack (VCMI's placement, decision 22). One infallible function per context, in a fixed
  order (adventure, heroes, treasuries).
- **T20a**: a `Ground` object on the tile replaces the field (the ground kinds); **T20b**: the
  unowned side (`Actor::System`, the neutral seat; decision 29), the monster's split and its
  survivors (ADV-MON-009, -010, A-6), a beaten monster's treasure.
- Not in increment 3: CLT-Q4's background variant (trees, mountains and dead vegetation within 3
  tiles) needs those decorations typed (E12's long tail); viewers keep the third picture.

## 6. Hero against hero (GAM-BAT-004, -005; T19a1, T19a2)

Built by T19a1: the visit from the neighbouring tile at the step's price, the battle at once, both
players own it; the visit concerns the hero whatever object lies under it (only leaving the
visitor's tile across its top is refused); a surrender's price from the loser's owner to the
winner's (D-13). Only the attacker's stream moves. Left for T19a2: the captured artifacts, +500.

## 7. Guarded piles (ADV-PIK-004; T21)

- The import: an empty guards record is a plain pile (3 shipped); guards up to seven stacks with
  an optional text (the text itself is presentation: a flag); D-6's `Unsupported` lifted.
- The visit (from the neighbour, as a pile): a question → `Fight` offer; no question → the battle
  at once; guards unsplit; a win picks the pile up, a loss or retreat leaves the guards' survivors
  in `army_left`.

## 8. The env and replays (T19b)

- Action encoding v3 (`h3-sim/src/adventure/action.rs`): `Action::{Adventure(adventure::Action),
  Battle(h3_sim::action::Action)}`, a phase per state; the masks of the other phase are empty.
- `to_move()` yields the acting players without a battle and, for each open battle, its side to
  act's owner; the neutral seat (`Seat::Neutral`, a battle action issued as `Actor::System`) comes
  with T20b (decision 29). `random_rollout` plays every seat at random; the original's battle AI
  replaces the neutral seat's stand-in when the battle lane has ported it (decision 27), and until
  then results against monsters are provisional. Until v3 the env offers no move onto another
  player's hero (`ADV-STUB-4`, T19a1).
- Replay format v3: each command with its actor, battle commands (v2 cannot read them back,
  `ADV-STUB-4`); golden replays with hero battles (T19b) and monsters (T20b). The property plans
  play battles since T19a1 (the driver acts as the owner of the side to act; they end mostly by a
  retreat or a surrender): T19b makes them play more to a victory.
- Benchmarks: the three workloads re-baselined with battles; `fork-xl-144` with a battle open.

## 9. Versions

Game state hash v6 (heroes' details since T18b, battles since T19a1; offer kinds later); adventure
state hash v8 (the formation since T18b, the coast since T19a1; monsters' static data and army
left, the difficulty later); adventure ruleset part v6 (the battle terrains since T19a1; monster
and ground kinds, T20a); map import v5 (hero customisations since T18b, the coast since T19a1;
monsters, guarded piles later); action encoding v3; adventure replay format v3. Each bumps once
in the increment, in the task that first changes it.

## 10. Shared and cross-lane work

- Done: T18a (#185: the ADR amendments, `arch.toml`, the kinds `necromancy`, `experience_gain`,
  `max_mana`, `join_bonus`, pack format v37); T18b (the harvester reads Necromancy's, Learning's
  and Intelligence's tables; `h3_heroes::max_mana`); T19a (`shared/e08-t19a`: `ADV-QUIRK-5`'s
  flag, `h3-map`'s battle part for its tests, battles in `06-security.md`; T19a1 `cross-lane`:
  the flag on in `base:faithful`, the harvester's profile).
- `cross-lane` (battle-owned), still to come: Diplomacy's `join_bonus` values (T20a; no table:
  the skill's level), the necromancy artifacts' parts (increment 4), `h3-heroes`' additions
  (`attack_defense`, `join_bonus` in T20a; the Necromancy share, the Learning factor, the capture's
  placement, `HeroChange` and its check in T19a2); `h3-experiment`'s spec for batch E3 (monsters,
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
| 2 | T19a1 + T19a2 | split at the checkpoint (decision 29): T19a1 done (2026-10-14): battles through the game, hero against hero, the armies' and the map's outcome (§5, §6); T19a2 the heroes' outcome, `HeroChange` (§2, §5) |
| 3 | T19b (+ T20a?) | the env, replays, properties and benchmarks with battles (§8); the cut at the round-2 checkpoint |
| 4 | T20a + T20b | monsters on the map, guarded tiles, grounds, the adventure's reads of the store, `attack_defense`, `join_bonus`, the difficulty (§2, §3); the monster's answer, offers, joining, the split, its battle, the unowned side (§4) |
| 5 | T21 + T22a | guarded piles and the maps with monsters, replays, benchmarks, the audit (§7); batch E3's spec and run (cross-lane, the client lane's recipes) |
| 6 | T22b | batch E3 scored, increment 3 closed, increment 4 refined |
