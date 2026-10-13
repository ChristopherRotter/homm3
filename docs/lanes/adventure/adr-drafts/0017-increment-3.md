# ADR-0017 amendment draft: heroes' details, battles and their seeds, the unowned side

- Status: Accepted by the user at ADV-Q13 (2026-10-13; drafted in E08 T17b, revised after the
  review), to be applied by T18a; amends ADR-0017 (Accepted 2026-10-10, amended 2026-10-12)
- Deciders: user + adventure lane (E08 increment 3: decisions 19–27; ADV-Q11 to ADV-Q13)
- Input: survey `docs/rules/surveys/E08-battles.md`; working detail `docs/epics/E08-design.md`
  (§1–§9). On acceptance a `shared/` pull request writes the points below into ADR-0017 and
  adds the revision line at the end.

<!-- Decisions only. Each block replaces or extends the named point of ADR-0017; text not named
     here stays as it is. The hero model (`h3-heroes` beneath the contexts) is the ADR-0013
     amendment draft beside this one; ADR-0013 point 5 already has the orchestrator apply an
     outcome. Revised after ADV-Q13 (the user's changes to the first draft). -->

## Context

Increment 3 (decisions 19–24) brings the heroes' details into the game, wandering monsters,
guarded piles and battles started on the map, played through `h3-game` with the battle context
as it is (`BattleSetup`, `BattleState`, `BattleOutcome`) and the hero crate's translations
(ADR-0013 point 2). Point 4 left open where a battle's seed and obstacles come from; point 6
named battles as pending decisions without saying who plays a side that no player owns
(wandering monsters, guards). The survey found the original seeds a battle's obstacles from its
tile and that VCMI does the same (survey §4.2).

## Decision

**Point 1, extended (the store of heroes).** `GameState` holds every hero's details
(`h3_heroes::HeroDetails`) by `HeroId`, each shared between forks and copied on its first write.
The game builds them when the state is built: from the setup's customisations or the hero type's
and class's defaults (a default experience is a setup placeholder, drawn from the setup stream),
spell points full, validated by `h3-heroes`; a setup the increment cannot represent fails the
import (`Unsupported`: an experience of level 2 or more, decision 22). A hero that leaves the map
keeps its details (the pool and the tavern are E09's). The contexts **read a hero's details by
reference**: the game passes its store to the adventure's commands and queries (and later the
towns'), which read through `h3-heroes`' derivations (the hero model, ADR-0013 point 2 amended);
what a command changes in a hero comes back as typed **hero changes** in its outcome (point 10),
which the game applies through `h3-heroes`. No adventure profile seam (E08 decision 5 withdrawn,
decision 25). `GameRuleset` bundles the battle part (ADR-0015 point 2) beside the core and the
adventure part.

**Point 2, amended (the unowned side).** "`Actor::System` for the server's trusted session layer
only" becomes: `Actor::System` for the trusted layer that hosts the game only (the server's
session layer, the env), never for a player. A battle side without a player (wandering monsters,
an object's guards) is played by `Actor::System`, which gains battle commands for an unowned side
only; no player can command it, and the system actor commands no owned side. Its policy stays
outside the state, as an agent's choices (point 4): the server's or the env's bot. The faithful
policy is **the original's battle AI**, which the battle lane ports (decision 27; its inbox); it
may read the whole battle state, as the original's does, since it runs in the trusted host; until
then the env plays the side at random. The system actor arrives with increment 3 for this; its
other commands stay the server lane's (M5), which also picks its default for a battle whose player
is gone.

**Point 4, extended (a battle's randomness).** A battle's **obstacles** are drawn by the battle
lane's generator (`h3_battle::generate_setup_obstacles`, until the battle lane ports the
original's: CLT-Q4's note, the adventure queue's inbox) from a kernel `Rng` seeded by the profile
(`ADV-QUIRK-5`, fix-optional, decision 26): in `faithful` with the original's tile seed `110 291 x
+ 167 801 y + 81 569` (32-bit; the defender's tile: the monster's, the guarded object's or the
defending hero's; the level not included), so a tile's obstacles are predictable, as in the
original; in `strict-fixed` with a second draw from the attacking player's stream. The battle's
**own seed** (damage, luck, morale and every later draw) is drawn from the attacking player's
stream when the battle starts and lives in the battle's state, server-only as every stream (point
5). A monster's **split** into stacks (ADR-0016 point 4) takes its two draws from the tile in
`faithful` (`ADV-QUIRK-6`, its own flag) and from the attacking player's stream in `strict-fixed`.
Setup placeholders of increment 3 (the setup stream): a random monster's creature and count, every
monster's aggression, a hero's default experience.

**Point 6, extended (battles and chained decisions).** Increment 3's offers: **join for free**,
**join for gold**, **pursue a fleeing monster** (decision 22; survey §3.2), **fight an object's
guards** (decision 19; survey §5.1), each yes or no. Answering an offer may open the next decision
of the same visit in the same command (a declined join goes on to the flee offer or a battle; an
accepted pursuit or fight opens the battle): a player still has at most one open decision. A
**battle** is a pending decision owned by the players of its sides: the visitor's, and against
another player's hero the defender's too, who plays outside its turn. While a battle is open its
owners' only legal commands are `GameCommand::Battle(h3_battle::Command)` by the owner of the side
to act (the system actor for an unowned side); neither owner can end its turn; it blocks no one
else. The command that ends the battle applies its outcome (point 11) and closes the decision.

**Point 8, extended (the env with battles).** Action encoding v3 has two phases: while the
player to move is in a battle its action is the battle lane's encoding (`h3_sim::action`, entry
and heads) and every adventure entry is masked; otherwise v2's entries and offers. `to_move`
yields seats: an acting player without a battle, and for a battle the owner of its side to act
or the **neutral seat** for an unowned side, which
the caller plays with a policy of its choice (`random_rollout` plays it at random). The
observation (`h3_game`'s view) adds the open battle as its owner's side sees it
(`h3_battle::observe_as`). The adventure replay format v3 records each command with its actor
(a player or the system).

**Point 9, extended (versions).** Increment 3 bumps the game state hash to v5 (the store of
heroes, battles in progress, the new offer kinds), the adventure state hash to v7, the adventure
ruleset part to v6, the map import to v4, the action encoding to v3 and the adventure replay
format to v3 (`E08-design.md` §9).

**Point 10, extended (outcomes with heroes, armies and battles).** A context's outcome may also
name hero changes (`h3_heroes::HeroChange`, checked by the context against the details it read),
a hero's army change (a group joining) and a battle to start (the visitor, the defender: a monster,
an object's guards or another player's hero, the tile). A join for gold whose price the payer
cannot pay is not opened and counts as declined, so the visit's chain goes on, without the refusal
event this point gives other offers (the original asks only when the player can pay and goes on
as after a no, survey §3.2).

**Point 11, new: battles through the game** (ADR-0013 point 5). The game builds the
`BattleSetup`: the armies from the adventure (a monster's stacks by its split), each hero's
`BattleHero` and war machines from `h3-heroes`, the battlefield from the tile in the original's
order (E04c), the obstacles (point 4), the formations, the funds (the owner's surrender
resource). It starts the battle with the seed of point 4, keeps it in the decision (shared
between forks, copied on the first write), routes its commands, and maps its events to the
owners of the sides they are visible to (an unowned side's to no player). When it ends the game
applies the outcome at once and infallibly, in every context (the heroes' part as hero changes
through `h3-heroes`): casualties (a monster's or guards'
survivors stay with the object), experience as a number for the winner's hero (no levels until
increment 4), Necromancy, Eagle Eye's spells, spell points, destroyed war machines' artifacts,
the loser's artifacts to the winner (the first fitting slot, else the backpack: decision 22), a
surrender's gold from the loser's owner to the winner's (D-13), a hero that lost, retreated or
surrendered leaves the map, a beaten monster leaves it and its treasure's resources go to the
winner's owner, a won guarded pile is picked up; a monster that survives keeps every stack's
survivors (A-6 fixed, decision 23).

## Consequences

- `arch.toml` (shared): `h3-game` gains `h3-heroes`, `h3-battle` and `h3-content-battle`,
  `h3-adventure` gains `h3-heroes` (in the shared layer, the ADR-0013 amendment draft); no new
  contract type.
- Battles against monsters played at random by the env are easier than the original's: training
  results against monsters are provisional until the original's battle AI is ported.
- Conservation becomes: an army changes only by a battle's outcome or a joining group; a treasury
  only by a visit's gain (a won battle's treasure or pile included), an accepted offer's price
  or a surrender. Legality consistency, atomicity,
  ownership isolation (a battle's draws from its attacker's stream only), observation hygiene (a
  battle's private side, ADR-0013 point 9) and clone equivalence are property-tested with battles
  open (the increment's exit criteria).
- The simultaneous-play inventory (point 7) gains a monster two players can fight and a battle
  writing two players' heroes, armies and treasuries.
- Battles never reach the wire through `h3-game` in E08: the server keeps hosting battles on
  their own (ADR-0018) until it hosts whole games.

## Alternatives considered

- Obstacles and the split from the tile in every profile: exploitable, if mildly (a defender
  picks its tile, an attacker knows the battlefield or tunes its army before it commits; the
  user, ADV-Q13); from a stream in every profile: unlike the original's fixed layouts and splits.
- An adventure profile seam (E08 decision 5, this draft's first version): every read a copied
  field, every write an outcome variant and code in `h3-game` (the user, ADV-Q13).
- The battle's own seed from the tile as well: every roll predictable before the battle, against
  decision 9.
- The unowned side played inside the engine: a bot in the core (E05 and E06 own bots); the
  attacking player commanding both sides: it would throw the monsters' side.
- A dedicated `Actor::Neutral`: a second trusted actor for one job; the system actor already is
  the trusted, non-player one.
- One flat action space with the battle's entries appended: couples two encodings and their
  versions; the phase keeps the battle lane's encoding as it is.

## Revision history line (for ADR-0017)

- 2026-10-1x: points 1, 4, 6 and 8–10 extended, point 2 amended and point 11 added for increment
  3: the store of heroes read by reference and changed through hero changes, the system actor for
  an unowned battle side (the original's battle AI its faithful policy), a battle's obstacles and
  a monster's split by the profile (from the tile in `faithful`, ADV-QUIRK-5 and -6) and its seed
  from the attacker's stream, chained offers and battles as pending decisions, the env's battle
  phase, battles through the game (E08 T17b, the user at ADV-Q12 and ADV-Q13; survey
  `E08-battles.md`).
