# ADR-0017 amendment draft: E08 increment 9, mines, the week and owned objects (the game side)

- Status: Proposed (E08 T77b, 2026-10-30); accepted by the user at ADV-Q38 (E08 decisions
  201–204) with decision 201's two changes written in, applied by T81 with batch E9's outcome
- Amends: ADR-0017 points 1, 3, 4, 6, 8, 10 and 11, and its consequences
- Sources: survey `docs/rules/surveys/E08-economy.md` §2 to §5 (T77a1), §7 to §13 (T77a2); the
  answers at ADV-Q36 and ADV-Q37 (the epic's decisions 192, 193, 196–199); the working detail in
  `docs/epics/E08-design.md` part C.

Applied by the increment's `shared/` task with batch E9's outcome (design §18). *Agent's call*
marks a choice the user accepted at the checkpoint (ADV-Q38, E08 decision 204) and may still
change.

## Point 1 (one entry point): appended after increment 8's sentence

> **Increment 9** adds one command, **`BuildBoat`** (an adventure command naming a Shipyard by
> its type and anchor, as the observation names it): a boat built from an own or allied Shipyard
> without a visit (ADV-SHY-003; the original's click on the Shipyard, VCMI's and the wiki's). No
> hero is named; the player's turn must be running with no open decision; the Shipyard must lie
> on a tile the player has explored, its boat tile must exist and be free, and the treasury must
> pay the kind's price, which the game checks before the adventure applies. Flagging, a mine's
> visit, a Lighthouse's and the weekly objects' are steps of `MoveHero`; the day's start is
> `EndTurn`'s (the last group's), no command of its own. The setup gains the ownable objects'
> **owners** (ADR-0016 point 1, from the map) and the map's **timed events** (the game's: for
> each its players, whether it applies to humans and to computers, its first day, its interval,
> its seven resources and its message by index); the import refuses an owner colour nobody
> plays.

## Point 3 (turn groups): appended after increment 5's sentence

> From increment 9 the **day's start** has the original's order (`new_day` 0x4C7FE0; GAM-CAL-002),
> on every day advance, so from day 2 (the game's start runs none of it: day 1 gives no income,
> grows and resets nothing): the calendar; on the first day of a week (days 8, 15, …: weeks of
> seven days) **the adventure's week step** (ADR-0016 point 4: the weekly objects, then the
> monsters' growth); on the first day of a month (days 29, 57, …: months of four weeks) **the
> month's kind** drawn, then **the adventure's month step** (a creature month's monsters); then
> **the day's income** of every player (GAM-RES-003 to -005; every player not defeated, E10's
> defeat) added to its treasury; then **every hero's spell points** (HER-MAN-002: every hero of
> the store); then the adventure's start-of-day step (increment 5's reveal under `strict-fixed`).
> Then the first group's turn starts. At **each player's turn start**, after the contexts'
> start-of-turn steps, the map's **timed events** fire for that player (GAM-CAL-004): in the
> map's order, each whose players include it, which applies to its kind (human or computer, the
> setup's flag) and whose day it is (its first day, the file's day + 1, or every interval after
> it); its resources are added to the treasury, a loss floored at 0. Day 1's fire when the game
> is built, with the first group's turn.
>
> The **month's kind** is game state: normal (the first month's), a creature's month (one of the
> parameters' twelve creatures) or the Plague; drawn at each new month as the parameters' odds
> (5, 4 and 1 in 10). The Plague changes nothing before the towns (E09). **The week's kind** (a
> week of a creature) and its towns' effects come with E09 (decision 197).

*Agent's call:* the week and the month are derived from the day (no state of their own); the
original's three words are one number here. The month's and the week's names (the original
draws one for its message) are not drawn: a name is the client's, and no draw of ours is the
original's.

## Point 4 (streams): appended after increment 7's sentence

> Increment 9's **setup placeholders**, drawn from the setup stream when the state is built, in
> one walk after increment 7's and 8's in object order: an abandoned mine's resource then its
> guards' count, a Windmill's resource then amount, a Garden's gift. **The day's start draws from
> the game's stream** (the day advance's): at a new week the Windmills' resource and amount and
> the Gardens' gift, in object order; at a new month its kind, a creature month's creature, then
> the spawn's walk, each candidate tile one draw and each monster placed its count and its
> aggression right after its tile's draw. E09's week of a creature draws from a stream of its own
> (decision 197), so adding it shifts none of these.

*Agent's call:* the first week's contents from the setup stream with the other placeholders, the
later weeks' from the game's: a game's build draws nothing from the game's stream today.

## Point 6 (pending decisions): appended

> Increment 9's decisions reuse increment 2's to 6's kinds: **a guarded mine's question** is a
> guarded pile's (`Fight`, accepted: the battle; declined: nothing); **the Leave Guards dialog** is
> the army dialog (GAM-DEC-004) over the mine's guards as its group, which its owner's `Close`
> keeps on the mine (a join's group is lost at its close, as before); **the Shipyard's boat
> offer** at a visit is an offer with the kind's price, made only when the treasury can pay
> (else refused, as every price), accepted: the price paid and the boat built. Nothing at the
> day's start asks.

## Point 8 (`AdventureEnv`): appended

> Increment 9 adds `BuildBoat` to the action encoding (one entry per Shipyard the viewer may use,
> by its anchor), masked by its legality; the observation the env encodes gains the owners, the
> weekly marks and the month's kind, which no flat encoding carries yet (E08 decision 187). The
> encoding's change is the AI lane's (point 8's ownership), made `cross-lane` in the game's task
> with the user's consent, as increment 8's observation was (E08 decision 172).

## Point 10 (gains and costs by outcome): appended

> Increment 9's outcomes: a weekly object's gain (as a pickup's resources); a flag (the object
> and its new owner); a boat offer accepted or `BuildBoat` (the boat and the price, which the game
> pays: for `BuildBoat` checked before the adventure applies, so a rejection leaves everything
> untouched). **The day's income** is no outcome but the game's own step: the game asks the
> adventure for each player's production (its mines' daily amounts; on a week's first day, for
> each creature type with a **`weekly_resource` ability** in one of the player's heroes' armies,
> its amount once, however many: the Crystal Dragons' 3 crystal) and `h3-heroes` for each of its
> heroes' income on the map (ADR-0013 point 16), adds
> them and pays them, saturating (GAM-RES-001), one `ResourceGained` per resource and player in
> player order. **A timed event's** resources are added the same way, its losses taken down to 0
> at most (`ResourceLost`, the amount actually taken).

## Point 11 (battles through the game): appended

> **Increment 9's battles** (GAM-BAT-001 amended): a mine's guards defend on the mine's entrance.
> A neutral mine's single stack is a wandering monster's defender (split as a monster's,
> `ADV-QUIRK-6`'s draws; D-47 as batch E9 decides); any other guards defend as an army without a
> hero: a neutral mine's played by the neutral seat, an owned mine's by its **owner**, who answers
> outside its turn as a hero battle's defender (the original names the owner the defending side's;
> whether it hands that player the control was not read: batch E9's E9-2 records it). The battle
> terrain is the mine type's when it has one (the subterranean of the crystal, gold and abandoned
> mines), else the tile's. The guards' survivors stay on the mine; a win flags it for the
> attacker.

*Agent's call:* the owner plays its mine's guards (a side with an owner is its owner's, as a
hero's), not the neutral seat; E9-2 checks it in hot seat.

## Consequences: appended

> - Increment 9: the core creature gains **abilities** outside battle (the user at ADV-Q38, E08
>   decision 201: the Crystal Dragons' crystal is an innate creature ability), a closed core kind
>   beside its growth and flags, whose first is `weekly_resource { resource, amount }`; the battle
>   part's abilities (ADR-0012) stay the battle's, and ADR-0015 point 2's core row gains "a
>   creature's abilities outside battle" (the shared task).
> - Increment 9: one command (`BuildBoat`), so the adventure replay format moves; the game state
>   hash gains the month's kind and the timed events; the env's action encoding changes (the AI
>   lane's, `cross-lane`). The day's start becomes `EndTurn`'s costliest step on a new week and a
>   new month (the scan over the weekly objects and monsters; the spawn's walk over every tile).

## Revision history line

> - 2026-10-?? (E08 T?? `shared/`; the user at ADV-Q36 to ADV-Q38, E08 decisions 192–204): points 1, 3, 4, 6, 8, 10 and
>   11 amended for E08 increment 9, mines, the week and owned objects: `BuildBoat`; the owners and
>   the timed events in the setup; the day's start in the original's order from day 2 (the week's
>   and the month's steps, the income, the spell points) and the timed events at each player's
>   turn start; the month's kind; the streams of the setup's and the day's draws; Leave Guards as
>   a kept army dialog, the boat offer; the env's `BuildBoat`; a mine's guards in battle, an owned
>   mine's played by its owner. Drafted in E08 T77b (survey `E08-economy.md`), batch E9's outcome
>   folded in.
