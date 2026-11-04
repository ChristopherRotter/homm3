# E08 design note

Increment 10's note: dwellings and hiring (E08 decision 231). Increment 9's note (mines, the week
and owned objects) is in `E08-log.md` ("Increment 9's design note at its closure", moved by T87c);
the code's comments cite these notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 10's
amendments, drafted in the lane's `adr-drafts/` (`0016-increment-10.md` by T88b1,
`0017-increment-10.md` by T88b2; applied to ADR-0016 and -0017 by T92, `shared/e08-t92`, with batch
E10's outcome, and deleted); written from T88a's survey and the answers at ADV-Q41 and
ADV-Q42 (the epic's decisions 231, 232, 235–238, 240). The task that builds a section deletes it;
the file goes when the epic closes.
Facts and sources: survey `docs/rules/surveys/E08-dwellings.md` (§ numbers are the survey's
unless they name this note); the proposed rule ids are the survey's §14.

Written in two parts by side (decision 239): **parts A and B** by T88b1, the content and the map
(`h3-adventure`, `h3-pack-adventure`, `h3-map`); **part C** by T88b2, the game, the env, batch E10
and the cut (§12 on). ADR-0015 needs no amendment: its point 4's import still draws nothing, a
random dwelling stays a placeholder the build resolves, as a random monster. *Agent's call (change
freely)* marks a choice the user may still change; the checkpoint (ADV-Q43, T88b2) accepted the
note and both drafts as written, the cut as §17 proposes (decisions 241–243).

**D-51 is the original's rule, not a quirk** (decision 238, revising 234): a dwelling's guards are
its own creatures, renewed every week whoever owns it (§6.1), never troops its owner left (an own
dwelling offers no Leave Guards, §4.1), so they fight unowned, played by the neutral seat. No flag;
an owned mine's guards stay their owner's side (ADV-MIN-002: its Leave Guards). Batch E10 has no
row E10-2.

## Part A — the content (the adventure part, its packs)

### 1. The kinds (`h3-adventure`'s `ObjectKind`; ADV-DWL-001, -005, ADV-RFG-001)

Three kinds, each **ownable but the camp**, **entered**, not visitable from above (17, 20, 78 and
216–218 are on neither of the original's lists, `object_classes_not_entered` 0x63AA58 and
`object_classes_from_above` 0x63AB00):

- **`dwelling`** (classes 17 and 20): `creatures`, one to four distinct core creatures in the
  original's record order (the dialog's and the guards' slot order: the Conflux's Air, Fire, Earth,
  Water; the Factory's Stone, Iron, Gold, Diamond; §2.1, §11 "Not disputes"), no war machine; and
  `footprint`, the kind's own template, `blocked` and `visitable` cells as (columns left, rows up)
  offsets from the anchor, at least one visitable (each of the 82 has one: its entrance), in the
  8 × 6 block of `TileMask`; *agent's call (T93a, correcting "disjoint"):* the masks the import
  makes from the template, so the entrance, impassable and visitable in every original
  dwelling, is in both. A placed dwelling keeps the map's template; the
  footprint is what a random dwelling becomes (§5). Its name is the type's (`CrGen1.txt`,
  `CrGen4.txt` by subclass).
- **`random_dwelling`** (216 to 218): no parameters; what it may become is the map's (the setup's
  holds, §5) and the part's table (§2).
- **`refugee_camp`** (78): `levels`, inclusive (1 to 7): its creature is drawn among the random
  monsters of those levels (ADV-MON-003's set, the part's `random_monsters`: no war machine,
  `not_random_monsters` left out, the creatures of the ruleset's packs, so RoE rules draw among
  RoE's), its count that creature's growth. Owned by nobody, never flagged.

Hash tags follow the part's last (the building task's); ruleset part v16.

### 2. The parameters, the table and the derivations

- **`[params.dwellings]`**, required when the part has a dwelling type (as `[params.months]` needs
  `[params.monsters]`; it needs `[params.monsters]` too, for the guards' size word, §10):
  `guard_level` (5: a creature of this level or more guards), `guard_factor` (3: its guards are
  this times its growth), `free_level` (1: a creature of this level or less joins its owner's
  visitor free). The original's numbers (§2.2, §4.4); *agent's call:* data, not constants
  (ADR-0006).
- **`[[random_dwelling]]`**, one per faction: `faction` (a core faction) and `dwellings`, exactly
  seven dwelling types, the one a random dwelling of level n becomes. The compile checks each is a
  dwelling with a creature of that faction and level that is no creature's upgrade (the original's
  `town_creatures[faction × 14 + level]`, the unupgraded one). *Agent's call:* the table, not the
  original's derivation (the last class-17 subclass listing the creature, the Golem Factory for a
  Stone Golem, §3.2 steps 3–4): the same 63 outcomes (T93a: not 62), as data a pack can read, and a test over the
  base and ab packs checks the table against that derivation (the duplicates 7, 13, 16, 47 and 51
  never drawn; VCMI's `bannedForRandomDwelling` agrees, §3.2).
- **Derived at compile**, kept in the compiled kind: a dwelling's **full guards**, for each creature
  of `guard_level` or more a stack of `guard_factor` × growth, one stack each in creature order,
  slots 0 up (the original's `army_add_stack` into an empty army; a Dragon Cliffs 3 Green Dragons,
  the Factory 9 Gold and 6 Diamond Golems, the Conflux 12 Earth Elementals, every level-1 to -4
  dwelling none); a stack above the stack cap is a compile error. Its **shift**: the run of
  bottom-row cells from the anchor leftwards that are neither blocked nor visitable, stopping at
  the first that is, within the template's width (0x502CD0 to 0x502D44, read for this note; of
  the 82 only the Angel Portal's is 1, its anchor cell free and its entrance one left; no run
  reaches the width, so the footprint needs no width). Its **full counts** are each creature's growth, read from the core
  where needed, not stored.
- **Costs:** a creature's cost is the core's (`CreatureInfo::cost`, gold and at most one other
  resource in the packs; the original's dialog handles just that, §5.1). The purchase's bound takes
  every resource of the cost (§8); a test keeps the packs at one other resource at most, so the two
  agree.

### 3. The packs

- **`base`** (`adventure.toml`): the RoE objects: the dwellings of the eight RoE factions, the RoE
  elementals' and the Unicorn's duplicates, the Elemental Conflux and the Golem Factory (class 20,
  base creatures), the Refugee Camp, the random dwelling, `[params.dwellings]` and the eight
  factions' tables. **`ab`**: the objects AB added: Conflux's dwellings, the four Altars (69–72,
  base creatures in an AB object), the special dwellings (62–67), the neutral ones (73–79), the
  Altar of Thought (60), Conflux's table. *Agent's call:* a type in the pack that introduced its
  object (an AB map's Altar under RoE rules is `MissingContent`, as any AB object); the building
  task lists which subclass is whose (`dwelling_creatures` 0x63D570 and the creatures' packs) and
  checks it on the RoE-format maps (T93b's census: all 49, 709 dwellings, 695 of class 17 and 14 of
  class 20, import under `base` alone). `sod` adds none.
- **The harvest's check** (with the original data, `h3-map`'s tests): each dwelling type's
  footprint equals its class and subclass's last template in `Objects.txt` converted as the import
  converts a map's template; each name equals `CrGen1.txt`'s or `CrGen4.txt`'s row; the creatures
  equal the reading's tables (§2.2), the Conflux's and the Factory's in their order. The types are
  hand-written source with their sources (`# original subclass N`), as the mines'.
- **The synthetic packs** gain a small dwelling, a two-creature one and a camp for the crates' own
  tests.

## Part B — the map (`h3-map`, `h3-adventure`)

### 4. The import (`h3-map`; map import v16)

- **Classes:** 17 by subclass (80 keys), 20 by subclass (2), 78 `base:refugee_camp`; 216, 217 and
  218 **not tied to a town** `base:random_dwelling` with the holds of §5 (217's level its subclass,
  218's faction its subclass, normalised to a faction set and a level range). The faction codes 0–8
  become keys through a factions table in `codes.rs` (the town lane's import reuses it, its
  inbox told). 216–218 take the scenery `Ownable` (GAM-BAT-011: the original's class is 17 or 20
  from the load on).
- **Owners:** a dwelling's and a random dwelling's owner as `ObjectSetup::owner`, refused for a
  colour nobody plays (`ImportError::ObjectOwner`) and on water (`Unsupported::OwnedOnWater`), as
  a mine's.
- **Tied to a town** (an identifier that names a town of the map): a blocker as today, **ADV-STUB-13**
  (registered by the building task) until E09's random towns resolve it (decision 231; 1 397 on the
  shipped maps, all tied to random towns; one tied to a fixed town appears on none).
- **Refused** (input bounds, as A-31; typed errors the building task names): classes 18 and 19
  (A-36: the original stops with an error box); a 17 subclass past 79 or a 20 one past 1, a 217
  subclass past 6, a 218 subclass past 8 (`ImportError::Subclass`, A-36); a random dwelling with an
  empty faction set (A-34), an identifier no town of the map has (A-35), a level byte past 6 or a
  lowest above the highest (A-36). None on the shipped maps (§1). Each gets a ledger entry as
  `ADV-QUIRK-33` (`exploit-fix`, no flag), numbered from 36.

### 5. The setup and the build (`h3-adventure`)

- **`Holds::RandomDwelling { factions, levels }`**: distinct core factions, each with a
  `[[random_dwelling]]` table, and an inclusive level range within 1 to 7; else
  `SetupError::Holds` (`check_holds`). Only for the random dwelling kind. Built by T93b with the
  import (decision 250).
- **A hero on a random dwelling's visitable tile** is refused (`SetupError`, named by the building
  task): any outcome blocks one of its two bottom cells, so the setup's validity would depend on
  the draw (`AdventureState::new` promises it does not). *Agent's call*; the census checks the
  shipped maps (none expected).
- **The resolution, when the game is built** (`StaticMap::new` with the setup stream, before the
  static map's tiles, lists and owners are made: in the placement walk, before every other draw
  of the static map and after E09's placeholders, ADR-0017 point 4): for each random
  dwelling in object order, a faction uniform among its set, a level uniform in its range, the
  table's dwelling type. The object takes that type (as a random resource takes its drawn pile
  type, ADV-PIK-002), the type's footprint as its masks and the anchor moved right by the type's
  shift (the original's anchor, which the map client's captures compare; the moved anchor may lie
  one column past the right edge, where only its free cell is: the static map ignores a cell off
  the map as it does past the top and left). Its owner is the setup's. From then on it is a
  dwelling like a placed one; nothing remembers it was random.
- **Lists:** the static map lists the dwellings (both kinds, in object order: the **dwelling
  index**) and the camps (the **camp index**) beside its ownable and weekly lists; the dwellings
  join the ownable list.
- **As built (T94a, decision 251; *agent's calls*, change freely):** the refusal and the name
  check do not depend on the draw. A hero (`SetupError::RandomDwellingTile`, checked before its
  tile) or a boat (`SetupError::Boat`) on a random dwelling's entrance or on any tile an outcome of
  its set and range covers from its moved anchor is refused (`dwellings::refused_tiles`); the
  name check (`SharedAnchor`) takes every outcome's type and moved anchor, not the placeholder's,
  so it runs after the holds' check. The resolved object holds nothing and is drawn on no further
  tile, as a placed dwelling; its owner and scenery stay the setup's. `StaticMap::dwellings`,
  `dwelling_index`, `camps`, `camp_index`; the dwellings were ownable since T93a.

### 6. The state (`h3-adventure`; adventure state hash v20)

- **Owners:** the dwellings' in the owners' block (ownable index), the setup's at the start, with
  the start's reveal around every owned one (ADV-OWN-002, radius 3: the original's
  `setup_map_objects` calls `set_dwelling_owner` for each).
- **A dwelling's week** (*agent's call*): one block by dwelling index, **none while every dwelling
  is as its week began** (each creature's count its growth, its guards the kind's full guards):
  when the game is built and after every week's step, which writes none (the refill of every
  dwelling at once, §9). The first visit that changes a dwelling makes the block (every entry
  full, then the change), a fork sharing it, a write copying it (99 dwellings on the largest map).
  An entry: the counts by the kind's creature order (u32: growth is a u16, a purchase lowers
  them) and the guards as they stand (the full guards, a battle's survivors by slot, or none once
  beaten). Equality and the hash read each dwelling's counts and guards as they stand, so no block
  and a block of full entries are one state (ADR-0016 point 2). Not the armies on objects: the
  week then resets one field, not every dwelling's entry of a shared table.
- **A camp's week:** one block by camp index, each its creature and count, drawn when the game is
  built and at every week's step (§9), as the weekly objects' block; a purchase lowers the count.
  None for a map without camps.

### 7. The visits (ADV-DWL-004, ADV-OWN-001 amended, ADV-HIR-001, ADV-RFG-001)

The step onto a dwelling's entrance (the kind is entered; the visit ends the walk there):

1. **Its owner's team** (the visitor's own; an ally's from E10's teams, until then `allied` is
   the player alone): no guards, no flag; on to the hire question.
2. **Another team's or neutral, its guards standing:** the guards' question, an offer
   (`OfferKind`, a new variant: *agent's call*) that carries what the original's question shows,
   the first occupied slot's creature and the size word of the guards' **total** count (GENRLTXT
   422; §4.2). Declined: nothing. Accepted: the guards' battle as they stand, unsplit, unowned,
   without a hero (part C §12: `Defender::Guards` even for one stack, unlike a neutral mine's single
   stack, which `monster_battle` splits). Lost, retreated, surrendered or a draw: the survivors by
   slot are the dwelling's guards until the week's step, nothing else changes. Won: the guards are
   gone this week; after the winner's levels the rest of the visit (GAM-DEC-007, `give_won`'s
   outcome) flags the dwelling, then asks the hire question.
3. **Unguarded** (or after that win): **flagged** for the visitor's owner (increment 9's `flag`:
   `ObjectFlagged`, radius 3 for the new owner); then the hire question. The flag needs no answer.
4. **The hire question** (ADVEVENT 35, or 36 for four creatures; a yes/no offer naming the
   dwelling's creatures). Declined: the visit ends, the flag stays. Accepted, on the visitor's
   **own** dwelling: each creature of `free_level` or less with a count **joins free**, all of it,
   where ADV-ARM-007's room is (a stack of the type within the stack's most, else the first free
   slot), its count set to 0; with no room none of it joins and the count is kept (D-52 as the
   original, decision 235: no merge of the hero's stacks); a count of 0 is told as "none here".
   Each told to the owner alone (part C's events). Then, if the dwelling has a creature above
   `free_level`, **the hire dialog** opens (the outcome's new field, part C §13); else the visit
   ends. On an **ally's** dwelling (E10's teams): no free creatures, the dialog read-only (part C
   §13; the original disables every button, §4.4).

**The Refugee Camp's visit** (`visit_refugee_camp`, §7), any player's hero: a count of 0 is told
as deserted (ADVEVENT 44), nothing else; else the hire question naming its creature (ADVEVENT 112;
the creature is hidden until then, §10); accepted: the hire dialog for its creature and count. No
flag, no guards, no free creatures, whatever the creature's level.

Every seat gets the same questions (as increment 9's Leave Guards: the original asks a human; a
computer player's hiring is E10's). A hero standing on an owned dwelling's entrance hides it, as
any owned object (ADR-0016 point 3); a hero does not revisit the dwelling it stands on.

### 8. The purchase's map side (ADV-HIR-002 in part)

The hire dialog is the game's pending decision (part C §13: its command, the treasury, when it
closes). The map's part, `AdventureState`'s purchase for the dialog's hero and source (a dwelling
or a camp at the hero's tile): the creature is one of the source's, the amount from 1 to its count,
and the army has room (ADV-ARM-007's, as above; none: refused, as the original's GENRLTXT 426 with
nothing bought); then the count goes down by the amount and the creatures join. Its price, the
creature's cost times the amount by resource, is the map's answer to the game, which checks the
treasury before and pays after (ADR-0004: nothing from the payload). A refusal leaves the state
untouched (typed errors). The dialog lists the source's creatures, as the original's lists the
record's four (no class-17 dwelling of level 1 opens it; class 20 has none of level 1). The most a
player may buy, `min(count, each resource ÷ its cost)` (§5.2), is part C's legal set.

### 9. The week's step (ADV-DWL-003, ADV-RFG-001; GAM-CAL-002 amended)

In `AdventureState::start_week`, before the weekly objects' contents: **every dwelling refilled**
(owned or not: the week's block set to none, §6), reading no week's or month's kind (the original's
refill reads none; neither the week's creature, the month's doubling nor a Plague reaches an
external dwelling, §6.1). Then **the camps' draws with the weekly objects' in one pass in object
order** (the original's map scan holds case 78 beside them; *agent's call:* one pass, so the
day stream's order follows the object order), then the monsters' growth as before. When the game
is built the camps draw with the weekly objects' first week (`first_week`), in object order. The
counts never accumulate; unbought creatures are lost.

A defeated player's dwellings become neutral, their guards kept (ADV-OWN-004, with E10's defeat,
§8); "flag all creature dwellings" (5 shipped maps) is E10's scenario.

### 10. Information (ADV-OWN-003 amended)

- **A dwelling** shows its type (so its name and creatures; a random one's drawn type and its
  footprint at the moved anchor, as the original draws it) and its owner, to every viewer that
  sees it; **never its counts or guards** (the hint and the popup show neither, §9:
  `ObjectView::guards` stays none for it).
- **A Refugee Camp** shows its type only, never its creature or count (no case of its own in the
  hint or the popup); no knowledge is kept after a visit (VCMI shows a visited camp's creature: not
  followed).
- **To the visitor's owner alone:** the guards' question's creature and size word, the hire
  question's creatures (a camp's creature), the free creatures joined or refused, and part C's
  dialog (counts and costs) and purchases.
- **To the viewers of the tiles:** the flag (`ObjectFlagged`, as increment 9's) and the guards'
  battle as any battle (ADR-0016 point 6); a battle's change of a dwelling's guards needs no event
  (nothing shows them). The week's refill and the camps' draws tell no one.
- `docs/06-security.md` gains the lines with the build (the shared pull request).

### 11. Ids, quirks and seams (parts A and B)

- **Rule ids** (written by the building tasks, the survey's §14): ADV-DWL-001 to -005, ADV-HIR-001,
  ADV-HIR-002 (with part C), ADV-RFG-001; ADV-OWN-001, -002, -003, ADV-OBJ-002, ADV-MON-003,
  GAM-CAL-002 amended. Part C adds GAM-DEC-008 and the guards' battle (GAM-BAT).
- **Quirks:** A-34, A-35 and A-36 recorded as refused (§4, from `ADV-QUIRK-36`). D-51 and D-52 are
  the original's rules, no entry (decisions 238, 235).
- **Stub:** ADV-STUB-13, a random dwelling tied to a town is a blocker until E09's random towns.
- **The town lane's seams** (its inbox told when built): the towns' external dwellings' growth
  (E09 survey §8.3) reads an owned dwelling's owner and its creatures' faction from the adventure
  (the owners and the kind; under RoE rules the elementals count for no faction, §2.2); a random
  dwelling tied to a town takes the town's drawn faction (E09's random towns); the factions table
  of §4; E09's recruitment reuses the hire dialog (part C).
- **Shared needs:** none for parts A and B (the core's creatures hold the faction, level, cost and
  growth used here).
- **Versions** (parts A and B): map import v16, adventure ruleset part v16, adventure state hash
  v20. E09's T7a and T8a bump the same three; the second to merge bumps again (the town lane's
  note of 2026-11-02; part C §15 orders them).

## Part C — the game, the env, batch E10 and the cut

Written by T88b2 (decision 239, whole: decision 240): §12 the guards' battle, §13 the game (the
hire dialog as a pending decision, GAM-DEC-008, its answer and events), §14 the env, replays,
fixtures, properties and benchmarks, §15 versions, §16 batch E10, §17 shared and `cross-lane`
work, the building tasks and the exit criteria, §18 risks. ADR-0017's amendment (points 1, 3, 4,
6, 8, 10, 11; drafted here, applied by T92).

### 12. The guards' battle (GAM-BAT-001 amended; ADV-DWL-004)

- **The question** (§7 step 2): a new offer kind, *agent's call* `OfferKind::Guarded { creature,
  count_word }` (no price): the first occupied slot's creature and `MonsterParams::count_word` of
  the guards' total, as GENRLTXT 422 shows them (VCMI's word reads the first stack: a text
  difference). Not increment 9's bare `Fight`: the mine's question names neither. Declined:
  nothing, the hero stays on the entrance.
- **The battle:** `BattleStart` with `Defender::Guards { army }` (the guards as they stand,
  unsplit; ADV-PIK-004's kind), so the side is **unowned** and the neutral seat plays it whoever
  owns the dwelling (decision 238; `Defender::Garrison` stays the owned mine's). Fought at the
  entrance (the visited tile): its terrain, coast and special ground as any battle's (ADR-0022),
  no type terrain (the original: the tile's battlefield, unlike the three subterranean mines);
  the obstacles from the tile under `ADV-QUIRK-5`; the setup's facts as any (GAM-BAT-010 to -013).
- **Its end** (`AdventureState::end_unowned_battle`, the object at the tile a dwelling): the
  defender's survivors by slot become the dwelling's guards in its week entry (§6), none after a
  draw; no event (nothing shows them, §10). Lost or retreated (no surrender to an unowned side):
  the hero leaves the map (GAM-BAT-003), the owner unchanged. Won: the guards none; the visit's
  rest (GAM-DEC-007: `AdventureOutcome::reward_after`, given by `give_won` after the winner's
  levels) flags the dwelling (§7 step 3) and asks the hire question. The experience and the rest
  of GAM-BAT-002 as any battle's.

### 13. The game (`h3-game`; GAM-DEC-008; game state hash v23)

- **Two offers** (`Answer::Offer`, the system actor's `Decline`): the guards' question (§12) and
  **the hire question**, *agent's call* `OfferKind::Hire { creatures }` (no price): the dwelling's
  creatures in its order, a camp's one creature (hidden until then, §10). Accepted:
  `AdventureState::accept_offer` gives the free creatures (§7 step 4: each `CreaturesJoined`, a new
  `JoinRefused { hero, creature, count }` without room, a new `NoneToRecruit { object_type,
  anchor, creature }` for a count of 0; each to the owner alone) and returns the dialog in the
  outcome's new field **`hire: Option<HireDialog>`** (the hero, the source by object index, type
  and anchor, `read_only`), or none (no creature above `free_level`). A camp's count of 0 makes no
  offer: `ObjectEmpty` (ADVEVENT 44's "deserted"), to the owner.
- **The hire dialog** (`Decision::Hire { owner, dialog }`, GAM-DEC-008; the original's
  `H3RecruitMgr`): its owner's alone; holds no one else; blocks its owner's moves and `EndTurn`
  like an offer; opened with `HireOpened` to the owner. **The answer** `Answer::Hire(HireAnswer)`:
  - **`Buy { creature, amount }`**, in this order of refusal, each typed and leaving the state
    untouched: a read-only dialog; a creature not the source's (by type: a source's creatures are
    distinct); an amount of 0 or above the count; no room in the hero's army (ADV-ARM-007: a stack
    of the type that takes the amount within the stack's most, else a free slot; GENRLTXT 426);
    the treasury short of cost × amount for any resource. Then the game pays the price
    (`ResourcePaid` per resource, the core's order) and the adventure's infallible purchase lowers
    the count and adds the creatures (`CreaturesJoined`, to the owner). A dialog of **one**
    creature closes after its purchase (`DecisionClosed`), as the original's; one of several
    stays open.
  - **`Close`** (the system actor's default, the original's Close): `DecisionClosed`. *Agent's
    call:* in the kind's own answer, as the University's `Leave`; `Answer::Close` stays the
    exchange dialogs'.
- **The legal set** (`legal_commands`): for each creature in the source's order whose **most** is
  at least 1, `Buy` of the most, then `Close`; read-only: `Close` alone. The most is
  `min(count, each resource of the cost: treasury ÷ cost, the room's bound)` (the room's: the
  stack's most if a slot is free, else the most less the smallest count among the hero's stacks
  of the creature, else 0). The query **`hire_range(state, rules, player, creature)`** gives
  `1..=most`, as `split_range` gives a split's counts; every amount in it is accepted. *Agent's
  call:* the room in the bound, so every listed answer applies; the original's slider leaves it
  out and refuses at Recruit, the same rule.
- **The view** (`DecisionView::Hire(HireView)`, to its owner alone, ADR-0016 point 6): the hero,
  the source by type and anchor, `read_only`, and for each creature its count and its cost by
  resource (the treasury is the owner's own: a client draws the slider). No most in the view.
- **An ally's dialog** (E10's teams): opened `read_only` after the hire question, nothing free,
  only `Close` (the original disables every button, §4.4). *Agent's call:* the decision, not an
  event, so a client draws one dialog. Unreachable until E10 (`allied` is the player alone), so
  not built now: the building task leaves the branch out (no stub: no player meets it).
- **The state:** the open decision in `GameState`'s queue (the dialog's counts are the
  adventure's, read at each view and purchase; the decision holds the names only). Game state
  hash v23 (the kind and its hash). **E09's reuse** (its decision 4, increment 3): a town's
  recruitment opens the same decision with a town dwelling as the source and the hero's or the
  garrison's army as the target: `HireDialog` names its source and target as enums E09 extends;
  the purchase rule (§8) and the answer stay one.

### 14. The env, replays, fixtures, properties, benchmarks

- **The env** (`h3-sim`, the AI lane's; `cross-lane` in the game's task with the user's consent,
  as decisions 138 and 203): action encoding **v7** appends four entries, `18 + c`, `c < 4`: buy
  the most of the open dialog's creature `c` (the source's order, the original's four), masked
  by `legal_commands`; `Close` on entry 10 (close or leave); the two questions on 9 and 10.
  *Agent's call:* the most only, as the original's Max then Recruit; an amount head is the AI
  lane's. The observation the env returns is `GameObservation` (decision 187): no flat encoding.
- **Replays:** adventure replay format v10 (`Answer::Hire`; E09's T5a2 took v9 first); every
  golden replay rehashed (the hashes' versions; no recorded map holds a dwelling, a random
  dwelling or a camp). New, on
  **`e08-dwellings-36.h3m`** (`crates/h3-map/tests/dwellings_map.rs`, `h3_map::writer`): two
  players; a Guardhouse (level 1), a Barracks, a Dragon Cliffs (guarded), the Golem Factory and
  the Elemental Conflux (class 20), neutral and owned at load by each; random dwellings of 216,
  217 and 218 (the Angel Portal's move among their outcomes, one drawn by the seed search); two
  Refugee Camps; a hero with seven stacks (D-52), gold without crystal. Random play over 22 days
  (three weeks), scripted ones: a lost then a won guards' battle and the flag; the free creatures
  joined and refused; the dialog's most, a purchase of less, refusals for room and the treasury,
  a one-creature dialog closing; a camp visited twice in a week and after it; a random dwelling
  visited at its drawn entrance.
- **Properties** (`h3-map`'s `play.rs`, `view.rs`): the plans place the new classes
  (`DWELLINGS`), the end of turn over weeks; the oracles learn the two questions, the free
  creatures and the dialog; checks: §6's invariants after every command; every listed `Buy`
  accepted, an amount past the range or another creature rejected with the state unchanged; the
  treasury and the army change by the price and the amount; after a week's step every dwelling
  full and every camp's creature of its set and levels with its growth; a resolved random dwelling
  is a dwelling of its holds with its type's footprint at the moved anchor; no viewer sees a
  dwelling's counts or guards or a camp's creature but the visitor's owner through its question,
  dialog and events.
- **Benchmarks** (`benches/adventure/`, on **`e08-dwellings-144.h3m`**: the shipped maps' most
  dwellings (99) and camps, random dwellings, a monster on every hundredth free tile):
  `day-start-dwellings-xl-144` (the `EndTurn` that starts day 8: the refill and the camps' draws),
  `fork-dwellings-xl-144` (forks after a visit's write: one block copy), `random-play-dwellings-36`
  (hiring in random play). The existing workloads measured against `master` right after the
  visits' first build (`homm3-bench-regression`): none plays a dwelling, so a change is the
  dispatch's and is fixed, not re-baselined, unless its reason is written.
- **`--with-original-data`** (`crates/h3-map/examples/original_data.rs`): every shipped map's
  static layer with the new classes: the 2 061 dwellings typed (709 under `base` alone on the 49
  RoE-format maps, T93b's census), the owners at load, the 206 random dwellings resolved for a fixed seed (each a
  dwelling of its set and range, its entrance on the map), the 1 397 tied to towns as blockers
  (ADV-STUB-13), the 321 camps; the harvest's check (§3).

### 15. Versions, rule ids, quirks

| Version | Change | Task |
|---|---|---|
| adventure ruleset part v16 | the three kinds, `[params.dwellings]`, `[[random_dwelling]]` (§1, §2) | T93a |
| map import v16 | classes 17, 20, 78, 216–218, the owners, the refusals (§4) | T93b |
| adventure state hash v20 | the owners, a dwelling's week, the camps, the resolved random dwellings (§5, §6) | T94 |
| game state hash v23 | the hire dialog (§13) | T95b |
| adventure replay format v10 (E09's T5a2 took v9) | `Answer::Hire` | T95b |
| env action encoding v7 (the AI lane's) | the four hire entries | T95b (`cross-lane`) |

E09 bumps the same rows (`E09-design.md` §19: replay format v9 by T5a, map import v16 by T7a,
game state hash v23 by T7b, part v16 and state hash v20 by T8a): **the second to merge bumps
again**, each change once (the table's numbers are this increment's if it merges first).

Rule ids (the survey's §14): ADV-DWL-001 to -005, ADV-HIR-001, -002, ADV-RFG-001, GAM-DEC-008 new;
amended: ADV-OWN-001, -002, -003, ADV-OBJ-002, ADV-MON-003, GAM-CAL-002, GAM-BAT-001 (§12).
ADV-OWN-004 (a defeat's) stays E10's. Quirks: `ADV-QUIRK-36` A-34, `-37` A-35, `-38` A-36 (refused
at the import, `exploit-fix`, no flag, as `ADV-QUIRK-33`). Stub: ADV-STUB-13 (a random dwelling
tied to a town, E09).

### 16. Batch E10

Survey §13's nine rows in two runs (decision 237), on the harness (`clients/capture/`, common) in
SoD on Easy, hot seat, from maps `h3-experiment` writes: **E10a** the visits (E10-1, E10-3 to
E10-7, days 1 and 2), **E10b** the week (E10-8 to E10-10, to day 8). E10-2 dropped (decision 238).
Two predictions added to the survey's rows: **E10-4** records whether the Barracks' dialog is
closed after its purchase (§5.3: a single creature's closes; §13 follows it); **E10-1** records the
question's size word against the guards' total and the first stack (§12).

- **The writer and the spec's fields:** built by T89a (`E08-log.md`, T89a; decisions 244, 245) —
  the fixtures of §14 reuse the writer. **The specs** (T89b): `E08-batch-10a.toml`, `-10b.toml`
  and their templates, with teams in the hot-seat map (E10-6; if the original's hot seat refuses
  teams, E10-6 is scored from the reading and says so) and a town for each player in E10b (past
  day 7: the lane's `TRAPS.md`).
- **The harness** (T90; common, CLT-Q18: built with the harness check, the client and map-client
  lanes' inboxes told): the dwelling records (game +0x4E39C: creatures, counts, guards; the owner
  +0x57 is read), a camp's tile words (+0x22, +0x0), the hire dialog (the most +0xAC, the amount
  +0xB8, the items Max 532, Recruit 30722, Close 30721; whether it is open), the battle's
  defending side's owner (E9-2's read), the hint and popup texts (E10-10). The recipes, the runs.
- **The scoring** (T91): survey §16 (new), each row against its prediction; a row against the
  reading opens an `ADV-Q` before the shared task; the rows each new rule cites (`verified` when
  built).

### 17. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** the `shared/` task (T92): ADR-0016's and -0017's drafts applied with
batch E10's outcome; `docs/03-domain.md`'s terms (dwelling, random dwelling, Refugee Camp, a
dwelling's guards, free creatures, hire question, hire dialog); `docs/06-security.md`'s lines
(§10, §13). **No shared code:** the core's creatures hold the faction, level, cost and growth; the
part's kinds and packs are the lane's. `cross-lane`: batch E10's spec (`h3-experiment`, T89); the
harness (common, T90); the env's encoding (`h3-sim`, T95b). The inboxes: the town lane's (the
factions table, T93b; the dwellings' owners and kind for its growth, T94; the dialog to reuse,
T95b), the AI lane's (v7, T95b), the map-client lane's (the views' drawn type and moved anchor,
T94).

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut
confirmed at its start). **Batch E10 runs before the build and before the shared task** (decision
237), so the ADRs are applied once with the rows' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T89** | Batch E10's maps: T89a the writer's classes, the spec's fields (`cross-lane`); T89b the two specs and templates (§16; decision 244) |
| 3 | **T90** + **T91** | T90: the harness's reads and clicks, the recipes, the runs E10a and E10b. T91: scored (survey §16) |
| 4 | **T92** | `shared/`: the two ADRs with T91's outcome, the glossary, `docs/06-security.md` |
| 5 | **T93** | T93a: the kinds, the parameters, the table, the derivations, the packs, the harvest's check (§1–§3; part v16). T93b: the import, its refusals and quirk entries, ADV-STUB-13 (§4; map import v16) |
| 6 | **T94** | A hero on a random dwelling's entrance refused (the holds' type and check came with T93b, decision 250), the resolution, the lists, the owners and the start's reveal, the week's and the camps' blocks, the week's step, the views (§5, §6, §9, §10; adventure state hash v20) |
| 7 | **T95** | T95a: the visits, the guards' question and battle, the flag, the free creatures, the camp's visit (§7, §12). T95b: the purchase, the hire dialog and its legal set, view and events, the env (§8, §13, §14's env; game state hash v23, replay format v10, `cross-lane` v7) |
| 8 | **T96** + **T97** | T96: the dwellings maps, their replays and properties, the benchmarks, `--with-original-data` (§14). T97: the audit against the exit criteria; increment 10 closed; increment 11 refined |

**Increment 10's exit criteria:**
1. Dwellings (17, 20), random dwellings not tied to a town (216–218) and Refugee Camps import typed
   with their owners; A-34 to A-36 are refused and recorded; those tied to a town are blockers
   (ADV-DWL-001, -005, ADV-OBJ-002, ADV-OWN-002).
2. A random dwelling resolves when the game is built into its drawn faction's and level's dwelling
   with that type's footprint at the original's moved anchor; a hero on its visitable tile is
   refused (ADV-DWL-005).
3. Every dwelling holds its creatures' growth and its guards (the factor times the growth of each
   creature of the guard level or more) from the build and each week, owned or not, no week's or
   month's kind; each camp a creature of its levels with its growth (ADV-DWL-002, -003,
   ADV-RFG-001, GAM-CAL-002).
4. Another team's visit to a guarded dwelling asks, then fights the guards unsplit and unowned at
   its entrance, their survivors kept until the week; won or unguarded it is flagged (radius 3);
   one's own team's skips both; no Leave Guards (ADV-DWL-004, ADV-OWN-001, GAM-BAT-001).
5. The hire question on one's own dwelling joins the free creatures (all of each or none, D-52)
   and opens the hire dialog; a purchase is bounded by the count, the treasury and the room, paid
   and counted down; the dialog stays open until closed, a one-creature dialog closing after its
   purchase; a camp's for any visitor (ADV-HIR-001, -002, GAM-DEC-008, ADV-RFG-001).
6. A viewer sees a dwelling's type and owner, never its counts or guards nor a camp's creature;
   the questions, the joins, the dialog and the purchases are its visitor's owner's alone
   (ADV-OWN-003).
7. Every new rule has its id, source and tests; `ADV-QUIRK-36` to `-38` recorded; the gates pass
   (coverage, determinism with the new and rehashed replays, the benchmarks at their baselines and
   the new ones baselined); the env plays the hire dialog.
8. Batch E10 run and scored (T91); the consensus rows `verified`.

### 18. Risks and open points

- **Two lanes in the same crates:** E09's T5a, T7a, T7b and T8a bump the versions of §15 and
  touch `h3-game`'s composition and the static map; each merge in order, the second rebasing and
  bumping again (the town lane's `TRAPS.md`); T95b's `HireDialog` keeps E09's source and target
  open (§13).
- **The resolution changes the static map's draws:** the random dwellings draw first in the
  placement walk (ADR-0017 point 4), so on a map with one every later setup draw moves;
  no recorded game has one. A moved anchor past the right edge is the static map's first cell off
  the map: tests at the edge (§5).
- **The dialog's legal set by query:** the env buys the most only (v7); an agent saving gold
  cannot buy less until the AI lane adds an amount head.
- **Teams:** every team test is `allied`, the player alone until E10; an ally's read-only dialog
  (§13) and a defeated player's dwellings (§9) come with E10.
- **Not read** (survey §4.4, §7): `ai_dwelling_visit` 0x527B00 and `ai_refugee_camp_hire` 0x527E10
  (a computer player's hiring, E10); the dialog's slider and its Portal of Summoning use (E09's
  town building); `dwelling_offer_hire`'s second free-creatures loop (no dwelling reaches it).
- **Not in increment 10** (decision 231): garrisons, the teleporters, creature banks, quests,
  towns' dwellings and recruitment (E09; it reuses §8 and §13), the random dwellings tied to a
  town, a computer player's hiring and a defeat (E10).
