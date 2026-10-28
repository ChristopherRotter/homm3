# ADR-0017 amendment draft: E08 increment 8, boats and the sea (the game side)

- Status: Proposed (E08 T66b, 2026-10-28; for the user at ADV-Q33)
- Amends: ADR-0017 points 1, 8, 10 and 11, and its consequences
- Sources: survey `docs/rules/surveys/E08-sea.md` §7, §9, §11 (T66a2); the answers at ADV-Q32 (the
  epic's decisions 162–168); the working detail in `docs/epics/E08-design.md` part C.

Applied by the increment's `shared/` task with batch E8's outcome (design §18). *Agent's call*
marks a choice the user may change at the checkpoint.

## Point 1 (one entry point): appended after increment 7's setup

> **Increment 8** adds no command and no decision: boarding, sailing and landing are steps of
> `MoveHero`'s path, and the sea's pickups give at once. The setup gains the map's **boats**
> (ADR-0016 point 1), which the import fills; a hero placed on water is refused (`Unsupported`,
> D-44). The turn start's points follow the hero's surface (ADR-0016 point 3).

## Point 8 (`AdventureEnv`): appended

> Increment 8 adds no entry to the action encoding: a boarding or a landing is a destination
> among the reachable tiles, so the env's masks offer them as any other. The observation the env
> encodes gains the boats on the viewer's explored tiles (tile, type, facing) and each seen hero's
> boat; the encoding's change is the AI lane's (point 8's ownership), made `cross-lane` in the
> game's task with the user's consent, as increment 7's typed answers were (E08 decision 138).

*Agent's call:* `cross-lane` in the task that builds the game's observation (design §15),
recommended at ADV-Q33 so that the env plays on at sea in the same pull request; the other way is
the AI lane's inbox after the build, the env encoding no boat until then.

## Point 10 (gains and costs by outcome): appended

> Increment 8's outcomes reuse increment 7's: a sea pickup's resources and artifact (with the full
> backpack's turn or loss), a removal; a boarding or a landing reports the hero's boat and its
> points after the conversion (to its owner); a bottle's message (to its visitor's owner).

## Point 11 (battles through the game): appended

> **Increment 8's battles** (GAM-BAT-001 amended): before the underground, a battle in which
> **either** hero is in a boat is fought on water's battle terrain with the ship's **deck**; when
> **both** are, on the **two boats** (their fixed blocked hexes, the battle part's); a special
> ground on the battle's tile then replaces the battlefield as on land (no shipped map has one on
> water). Nothing else in a battle reads a boat (the original's three tests, survey §7.1). A
> monster or an event's guards on water fight a boat's hero on the deck; a landing's guards fight
> on the shore, the hero out of its boat; a flyer that attacks a monster on the water from the
> shore (D-45's path, ADR-0016 point 4) fights on the water tile's own terrain, no boat in the
> battle (the reading; VCMI's deck by the tile). A hero in a boat that loses, retreats or
> surrenders leaves the map **with its boat** (GAM-BAT-003 amended, ADV-SEA-006); a winner in a
> boat stays in it.

## Consequences: appended

> - Increment 8: no command, decision or answer is new, so the adventure replay format keeps its
>   version; the game state hash follows the adventure's. The battle's setup reads the heroes'
>   boats (no seam change: `BattleSetup` already names its terrain and battlefield). The env's
>   observation encoding changes (the AI lane's, `cross-lane`).

## Revision history line

> - 2026-10-?? (E08 T?? `shared/`; the user at ADV-Q32 and ADV-Q33): points 1, 8, 10 and 11
>   amended for E08 increment 8, boats and the sea: no new command or decision, the boats in the
>   setup and a hero on water refused, the env's observation with the boats, the battle at sea (the
>   deck by either hero's boat, the two boats by both) and a boat leaving with its beaten hero.
>   Drafted in E08 T66b (survey `E08-sea.md`), batch E8's outcome folded in.
