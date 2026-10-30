# ADR-0016 amendment draft: E08 increment 9, mines, the week and owned objects

- Status: Proposed (E08 T77b, 2026-10-30); accepted by the user at ADV-Q38 (E08 decisions
  201–204) with decision 201's two changes written in, applied by T81 with batch E9's outcome
  (folded in by E08 T80: D-47, D-49 and A-32 confirmed, A-33 withdrawn; survey §19)
- Amends: ADR-0016 points 1, 3, 4, 5 and 6, and its consequences
- Sources: survey `docs/rules/surveys/E08-economy.md` (T77a1 §0 to §6, T77a2 §7 to §18); the
  answers at ADV-Q36 and ADV-Q37 (the epic's decisions 192, 193, 196–199); the working detail in
  `docs/epics/E08-design.md` parts B and C.

Applied by the increment's `shared/` task with batch E9's outcome (design §18): a dispute batch E9
reverts (decision 196) is rewritten there before the ADR is amended. *Agent's call* marks a choice
the user accepted at the checkpoint (ADV-Q38, E08 decision 204) and may still change.

## Point 1 (three kinds of storage): a fifth bullet after increment 8's boats

> - **Increment 9's owned objects, the week and the month's monsters** (E08 decision 192):
>   - **Owners**: every ownable object (the mines, the abandoned mines, the Lighthouses, the
>     Shipyards; the dwellings come with increment 10) has an owner or none, held in one block by
>     ownable index (the static map lists the ownable objects in object order), shared by forks
>     and copied at a flag. The owners at load come from the setup.
>   - **A mine's guards** are the armies on objects (the table increment 3 made for the armies
>     battles leave): what a Leave Guards dialog leaves there and what a battle leaves of them;
>     an abandoned mine's Troglodytes drawn when the game is built are what the build drew for it
>     (as increment 7's draws), the table overriding them once written.
>   - **The weekly objects' contents** (the Windmill's resource and amount, the Water Wheel's
>     gold, the Mystical Garden's gift and whether it is full): one block by weekly index (the
>     static map's list of them), written at a visit (emptied) and at every new week (all of
>     them).
>   - **A monster's fraction** (the original's sixteenths, ADV-MON-013): a persistent table by
>     object index beside the armies on objects, none for a fraction of 0; the count as it
>     stands stays the armies' table's or the static map's.
>   - **Objects added in play** (the month's monsters, ADV-MON-014): records appended after the
>     static map's objects, so an added object's index follows theirs and every table by object
>     index serves it unchanged; the layer of objects that left the map (increment 2) becomes the
>     layer of the objects' **live presence**: the objects that left and the objects added, with
>     the tiles each blocks, is visited from or guards. A fork costs a reference count; a month's
>     spawn writes one block and the tiles' leaves it touches.
>   - **A Shipyard's boat tile** is computed once when the game is built (ADV-SHY-001) and kept
>     with the static map: the original never moves it.

*Agent's call:* the owners as one block, not a per-tile layer: an XL map has at most a few
hundred ownable objects (§1's census: 26 mines of one kind at most on a shipped map), so a flag
copying the block costs less than a grid's leaf, and the views read every owner. The month's
monsters as added objects with indices after the static ones, not a table of their own as the
boats: a monster is the same kind wherever it came from (its visit, answer, guard zone, battle,
growth and view), and the boats' own table exists because a boat moves.

## Point 3 (heroes on the map): appended after increment 8's sentence

> From increment 9 an owned mine's visit by its owner opens the **Leave Guards** dialog: the
> army dialog of increment 6 between the hero's army and the mine's guards (merge, swap, split;
> the hero's last stack stays), whose group is kept as the mine's guards when it closes, never
> lost. Every player's hero gets it (the original shows it to a human only; a computer player's
> handling is E10's). A hero standing on an owned object's entrance hides it: a step onto that
> tile meets or fights the hero, the object is not visited and its owner does not change. A
> hero does not revisit the object it stands on (the original has no revisit command).

## Point 4 (map objects): a sub-list after increment 8's kinds

> - **Increment 9's kinds** (the import maps classes 42, 53, 55, 87, 109, 112 and 220; 220 as
>   53's subclass 7, as the original's `object_class_values`):
>   - **Ownable kinds** are entered (the visit ends the walk on the entrance). **Flagging**
>     (ADV-OWN-001): a visit flags a neutral object or another team's for its visitor's owner,
>     never an own or allied one (allies come with E10's teams: until then a team is one
>     player); nothing is paid. The owners at load are the map's; the import refuses an owner
>     colour no player of the scenario plays (an invalid map, ADV-OWN-002), and an ownable
>     object whose entrance lies on water (`Unsupported`: how a hero reaches it was not read;
>     none of the shipped maps has one). A defeated player's objects become neutral, a mine
>     keeping its guards (ADV-OWN-004), with E10's defeat.
>   - **Mines** (`mine`: its resource and daily amount, the type's battle terrain if any;
>     ADV-MIN-001, -002): an own mine's visit opens the Leave Guards dialog (point 3), an ally's
>     does nothing; another's with guards asks first (a guarded pile's question, ADR-0017 point
>     6), then fights them: a **neutral mine's single stack** as a wandering monster (split by the
>     army ratio table, its draws by the profile, `ADV-QUIRK-6`; D-47 as batch E9 decides), any
>     other guards as an army without a hero, their owner's or none; a win clears the guards and
>     flags the mine; a loss, a retreat or a surrender leaves the guards' survivors on the mine
>     and its owner unchanged. A battle on a crystal,
>     gold or abandoned mine's entrance is fought on its type's battle terrain (the subterranean,
>     GAM-BAT-001 amended), whatever lies under it. The guards' slot is the slot the stack holds
>     (A-29 fixed: the original reads slot 0, which only E10's defeat can leave empty).
>   - **The abandoned mine** (`abandoned_mine`; ADV-MIN-003): its resource drawn when the game is
>     built among the map's set (equal chances; the original draws at the map's load), its
>     guards drawn then too: the kind's creature (Troglodytes), 100 to 200 inclusive (D-48 as
>     read); its daily amount its drawn resource's mine's. The import refuses an empty set or a
>     bit past the seven resources (A-31: an input bound, no flag).
>   - **Lighthouses** (`lighthouse`, the points it adds; ADV-OWN-005): flagged by a visit; each
>     Lighthouse a player owns adds the kind's points (500) to the day's sea points of that
>     player's heroes (ADR-0013 point 13.6), read where the day's sea maximum is computed.
>   - **Shipyards** (`shipyard`: the boat's price and type, the offsets; ADV-SHY-001 to -003):
>     the boat tile is the first of the kind's twelve offsets around the entrance, on its level,
>     that is water no object blocks and no object is visited from, when the game is built; none
>     found, the Shipyard never builds. Its visit flags it, then makes the boat offer for the
>     price (ADR-0017 point 6) unless the tile is missing or holds a boat or a hero (the visit
>     tells its owner why); an accepted offer pays and puts an empty boat of the kind's type on
>     the tile, owned by the builder (ADV-SEA-001's record). An own or allied Shipyard also
>     builds without a visit (`BuildBoat`, ADR-0017 point 1), under the same refusals.
>   - **The weekly objects** (`windmill`, `water_wheel`, `garden`; ADV-WEK-001 to -003): visited
>     from the neighbouring tile, owned by nobody, never asking. **Their contents are the
>     object's, one a week:** the week's first visit, by any player's hero, takes them and empties
>     the object for every player; every later visit that week, by any player, finds it empty and
>     is told so (the user at ADV-Q38, E08 decision 201: never once per player). A visit also marks
>     the visitor's team (the Garden's mark the visitor's owner alone), a mark no week clears and
>     that never gates the gain: the original keeps it for its hint only (ADV-VIS-006). The
>     contents when the game is built and at every new week, visited or not: the Windmill one of
>     its resources (mercury to gems) and an amount of 3 to 6, both drawn; the Water Wheel its
>     first week's gold (500) at the build, its later gold (1 000) every week; the Garden full
>     with one of its two gifts (5 gems or 500 gold), drawn.
>   - **The week's step** (ADR-0017 point 3): the weekly objects' contents as above, then **every
>     wandering monster grows** unless it does not grow (ADV-MON-013): its count kept in the
>     original's sixteenths (the count × 16 plus the fraction), a tenth of that value added,
>     truncated, the result capped at 4 000 × 16 (A-32: a larger stack from the map becomes 4 000
>     at its first week; no flag; batch E9's E9-10); a battle's survivors keep the fraction (D-49,
>     confirmed by batch E9's E9-9).
>   - **The month's step** (ADR-0017 point 3), in a month of a creature (ADV-MON-014): every land
>     tile of both levels in level, row and column order that no object blocks or is visited
>     from, with no event and no hero, gets a monster with one chance in 200: the
>     month's creature, a count of twice a draw from the creature's adventure-map range, an
>     aggression drawn 1 to 10, growing, fleeing as any, without treasure (D-50 as read). It is
>     an object added in play (point 1) and guards as any monster from then on.

*Agent's call:* the weekly contents drawn in object order (the static map's order), not the
original's tile walk: no draw is the original's anyway (ADR-0003's streams), and object order is
the setup's. The monster's fraction kept apart from its count: a battle writes the count as it
writes it today, the week reads both.

## Point 5 (fog of war): appended after increment 5's sentence

> From increment 9 a **flag** reveals the tiles within 3 of the object's entrance to its new
> owner (its team from E10; ADV-FOG-002's disk), and the start of the game reveals the same around
> every object owned at load, as the original does (batch E9's E9-4 and E9-6).

*Batch E9 (E08 T80):* A-33 withdrawn. The reading had the original skip an owned Lighthouse at
load; its game start reveals around every owned object (`reveal_owned` from
`adventure_manager_start`, survey §19), as VCMI. One rule for every owned object, no quirk.

## Point 6 (information hiding): appended after increment 8's sentences

> From increment 9 every viewer sees an **owned object's owner** where it sees the object. A
> mine's daily production, and so an abandoned mine's drawn resource, is its owner's team's
> knowledge, never another's; a mine's guards are seen as a guarded pile's (their creatures and
> words), their counts by the owner's team. A Shipyard's boat tile is static and public with it.
> A weekly object's **visited mark** is its visitor's team's (the Garden's its visitor's owner's),
> kept through the weeks (the original's hint reads "visited" from it, never "emptied"); its
> contents are never shown, before a visit or after. A month's monster is seen as any monster where its
> tile is; a monster's fraction is never shown (its word follows its count). A flag, a built
> boat and an added monster are seen where their tiles are; a mine's income, a weekly object's
> gain and the guards' exchange are told to their owner alone.

## Consequences: appended

> - Increment 9: the adventure state gains the owners, the weekly contents, the monsters'
>   fractions and the objects added in play (state hash); the part gains six kinds and the
>   week's, the month's and the Shipyard's parameters (part version); the import types seven
>   classes and refuses three inputs (map import version). A day's start writes every growing
>   monster and every weekly object once a week: a fork after it shares nothing of those tables
>   until the next fork copies them again (the `day-start-xl-144` workload measures it).

## Revision history line

> - 2026-10-?? (E08 T?? `shared/`; the user at ADV-Q36 to ADV-Q38, E08 decisions 192–204): points 1, 3, 4, 5 and 6
>   amended for E08 increment 9, mines, the week and owned objects: the owners, a mine's guards,
>   the weekly contents, the monsters' fractions and the objects added in play; Leave Guards; the
>   kinds mine, abandoned mine, Lighthouse, Shipyard, Windmill, Water Wheel and Garden with
>   flagging, the boat tile and the week's and month's steps; the flag's and the start's reveal;
>   owners public, a mine's production its owner's. Drafted in E08 T77b (survey
>   `E08-economy.md`); batch E9's outcome folded in (E08 T80).
