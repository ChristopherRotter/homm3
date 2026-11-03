# E08 design note

Increment 10's note: dwellings and hiring (E08 decision 231). Increment 9's note (mines, the week
and owned objects) is in `E08-log.md` ("Increment 9's design note at its closure", moved by T87c);
the code's comments cite these notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 10's
amendments, drafted in the lane's `adr-drafts/` (`0016-increment-10.md` by T88b1; ADR-0017's by
T88b2); written from T88a's survey and the answers at ADV-Q41 and ADV-Q42 (the epic's decisions
231, 232, 235–238). The task that builds a section deletes it; the file goes when the epic closes.
Facts and sources: survey `docs/rules/surveys/E08-dwellings.md` (§ numbers are the survey's
unless they name this note); the proposed rule ids are the survey's §14.

Written in two parts by side (decision 239): **parts A and B** by T88b1, the content and the map
(`h3-adventure`, `h3-pack-adventure`, `h3-map`); **part C** by T88b2, the game, the env, batch E10
and the cut (§12 on). ADR-0015 needs no amendment: its point 4's import still draws nothing, a
random dwelling stays a placeholder the build resolves, as a random monster. *Agent's call (change
freely)* marks a choice the user may still change at the checkpoint (ADV-Q43, T88b2).

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
  offsets from the anchor, disjoint, at least one visitable (each of the 82 has one: its
  entrance), in the 8 × 6 block of `TileMask`. A placed dwelling keeps the map's template; the
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
  Stone Golem, §3.2 steps 3–4): the same 62 outcomes, as data a pack can read, and a test over the
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
  checks it on the 44 RoE-format maps (695 dwellings import under `base` alone). `sod` adds none.
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
  `SetupError::Holds` (`check_holds`). Only for the random dwelling kind.
- **A hero on a random dwelling's visitable tile** is refused (`SetupError`, named by the building
  task): any outcome blocks one of its two bottom cells, so the setup's validity would depend on
  the draw (`AdventureState::new` promises it does not). *Agent's call*; the census checks the
  shipped maps (none expected).
- **The resolution, when the game is built** (`StaticMap::new` with the setup stream, before the
  static map's tiles, lists and owners are made; the draw's place among the build's draws is
  ADR-0017 point 4's amendment, T88b2): for each random dwelling in object order, a faction
  uniform among its set, a level uniform in its range, the table's dwelling type. The object takes
  that type (as a random resource takes its drawn pile type, ADV-PIK-002), the type's footprint
  as its masks and the anchor moved right by the type's shift (the original's anchor, which the
  map client's captures compare; the moved anchor may lie one column past the right edge, where
  only its free cell is: the static map ignores a cell off the map as it does past the top and
  left). Its owner is the setup's. From then on it is a dwelling like a placed one; nothing
  remembers it was random.
- **Lists:** the static map lists the dwellings (both kinds, in object order: the **dwelling
  index**) and the camps (the **camp index**) beside its ownable and weekly lists; the dwellings
  join the ownable list.

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

Written by T88b2 (decision 239): §12 the guards' battles, §13 the game (the hire dialog as a pending
decision, GAM-DEC-008, its commands and events), §14 the env, replays, fixtures, properties and
benchmarks, §15 versions, §16 batch E10, §17 shared and `cross-lane` work, the building tasks and
the exit criteria, §18 risks.
