# ADR-0016 amendment (draft): the town as a map object

- Status: Draft (E09 T2a, 2026-11-02), with the draft ADR-0028 beside it; the user's decisions 2,
  11, 13 and 16 (TWN-Q1, TWN-Q2) and 17 (TWN-Q3). Accepted at T2b's checkpoint (`TWN-Q4`), it
  goes into ADR-0016 through the `shared/` pull request that creates the crates; the code is the
  town lane's `src/town*` of `h3-adventure`, the state structs and hashes `cross-lane`.
- Amends: point 1 (the storage), point 3 (heroes on the map), point 4 (the kinds), point 5 (fog
  of war), point 6 (information hiding).
- Working detail: the survey `docs/rules/surveys/E09-towns.md` §7 to §9, §12; the design note
  `docs/epics/E09-design.md` (T2b).

## Context

The adventure keeps a town as an untyped blocker (its footprint), so a hero on the gate leaves
upward (the town queue's T14a note) and nothing visits it. Decision 2 keeps the town as a map
object in the adventure, as a mine and its guards: its tiles and entrance, its owner, its
garrison, its visiting hero; the town's own record (faction, buildings) is `h3-towns`' (ADR-0028
point 2), and `h3-game` composes the two.

## Decision (amendment)

1. **Point 1, the storage** (E09's increment 1): a town object's static part holds its template
   masks and its `TownId` (ADR-0028 point 3); a random town is a town of its drawn faction from
   the build on (its picture is the client's, from the faction the game shows). Towns are
   **ownable objects**: their owners in the owners' block by ownable index with the mines', the
   owners at load the setup's. A town's **garrison** is its army on objects (the table increment
   3 made, a mine's guards' since increment 9): the setup's (its random stacks resolved when the
   game is built, ADR-0017's amendment) until a rule writes it. No other town state is the
   adventure's.
2. **Point 3, heroes on the map:** a town's **visiting hero** is the hero standing on its
   entrance (the original's `hero_show`, survey §7.3), never a stored field: an entering hero,
   the main town's hero, a hero moved into a town at the import. A garrison hero (the town
   screen's swap) comes with increment 3. A hero on a town's entrance hides the town as on an
   owned object's: a step onto the tile meets it (its own team) or attacks the town (another
   team, below).
3. **Point 4, the kinds: `Town`** (classes 98, and 77 resolved; the type's visit properties:
   **entered, not visitable from above**, so no flyer comes down onto the gate from the three
   tiles above and a hero leaves it as off every such object). The step onto a town's entrance
   (survey §7.1, decision 16), its outcome composed by the game (ADR-0017's amendment):
   - a town of the visitor's **own or an allied** team (allies with E10's teams: until then a
     team is one player) is **entered**: the visit ends the walk on the entrance and changes
     nothing (no service in increment 1); **never captured**;
   - a town of **another team or none** that is **defended** (a visiting hero, or a creature in
     its garrison) is **visited from the tile before** (the step paid, the hero stays, as a
     monster) and the visit is a **siege**: against the visiting hero with the garrison merged
     into its army first (TD-7 by the reading: each stack's AI value, survey §9.2; what stays
     in the garrison does not fight and is lost with the town), else against the garrison as
     an army without a hero (E08's `Defender::Garrison`), the defending side the town's owner's
     (the neutral seat for none). A siege of a town whose visitor is an ally's hero is a
     **typed refusal**, a stub until E10 (decision 16);
   - an **undefended** town of another team or none is **entered and captured**.
   - **Capture** (a town is never flagged; ADV-OWN-001's flagging is the other ownable kinds'):
     the owner changes, the garrison is emptied, the reveal of point 5; `h3-towns`' part of it
     (the Capitol's limit) is the game's composition. **After a siege won the attacker stays on
     the tile it attacked from** (finding 19) and enters by a later step, which finds the town
     its own; a defender's win or a battle with no winner changes no owner, the survivors staying
     in the armies that fought (the visitor's, or the garrison as a mine's guards).
4. **Point 5, fog of war:** a change of a town's owner reveals to the new owner the tiles within
   the town's **reveal radius** of its entrance (`h3-towns`' derivation: 5; a Tower with its
   Lookout Tower 20; a Tower with the Grail every tile of every level; survey §8.3), its team's
   from E10; the start of the game reveals the same around every town owned at load, with the
   other owned objects (E08 T80's `reveal_owned`).
5. **Point 6, information hiding** (decision 11; the original's levels, survey §12): a viewer
   that has explored a tile of a town sees the town as an owned object (its type, tiles,
   `TownId`, owner) and, through the game, its **public facts**: its faction, whether it has a
   Capitol and a fort and the fort's level, its name, and whether it is defended where the step
   tells it (the cursor). Its **garrison** another team's viewer sees by its **level**, the
   number of its own towns with a Tavern: none at 0, the stacks' creatures at 1, with their size
   words at 2 or more. The counts, the hall, the income, the silo's resources and the garrison
   hero's mark are the **owner's team's**, or a viewer's whose hero within reach has expert Visions
   or a Rogue (the creature) in its army (survey §12.3; Visions when it is placed, E08's "Not
   placed yet"). The visiting hero is
   seen as any hero. Every other building and every derived fact (the Escape Tunnel, ADR-0023
   point 2) stays the owner's. A capture is seen where its tiles are; its reveal is told to the
   new owner alone. `docs/06-security.md` lists it (T2b's design note gives the words).

## Consequences

- The adventure state hash and ruleset part (the town type and its kind) and the map import bump
  (the adventure lane's, `cross-lane`); the replays with a town re-recorded.
- The adventure never reads a town's buildings: the game passes it the reveal radius and adds
  the public facts to the observation.

## Alternatives considered

- A stored visiting hero: a second record of what the hero's tile already says.
- Capture as a flag (ADV-OWN-001's step): a town's capture empties the garrison, reveals by the
  town's radius and leaves the attacker outside after a siege, none of which a flag does.
- The ally's defence built now; no teams until E10 (TWN-Q2, decision 16).
