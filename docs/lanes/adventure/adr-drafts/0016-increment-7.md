# ADR-0016 amendment (draft): rewards on the map: the objects, their state, events and the move

- Status: Proposed (E08 T55b, 2026-10-26), from T55a's survey and the answers at ADV-Q28 (the
  epic's decisions 129–136). **Accepted by the user** (ADV-Q29, 2026-10-26; decisions 138–141).
  The round-3 `shared/` task (T59) applies it to ADR-0016, with batch E7's outcome for A-23
  written in (decision 131).
- Amends: ADR-0016 points 1, 3, 4 and 6.
- Detail: `docs/epics/E08-design.md` part B (§7–§13); facts: survey
  `docs/rules/surveys/E08-rewards.md` §1–§8, §9, §13.

## Context

Increment 7 types 31 object classes that are blockers today (survey §10): treasure chests,
placed and random artifacts, Spell Scrolls, the monsters' treasure artifacts, twenty objects that
teach or hold a find, Pandora's Box, events and the Pyramid. The original bounds their state in
ways the engine does not copy (decision 133, the user: "We want to allow maps even when every
single tile is filled with a box, or every single tile with an object of some class"): 32
once-per-hero marks per class (A-17), 400 reward records (A-19). Events are hidden objects that
stop a hero, and the original's pathfinder lets a human's path find them (A-23, §8.5).

## Decision

1. *(amended, storage)* Increment 7's state, bounded only by the map's objects (decision 133):
   - **Per object** (persistent tables in one dimension by object index, as point 1 says): an
     object **emptied** (a Corpse, Wagon, Lean To or Warrior's Tomb searched, a Pyramid looted)
     beside the removal; the **army left** by a battle (a box's, an event's, a guarded artifact's
     or scroll's guards, besides a monster's) in a table of leaves whose copy on a write is
     bounded, not the whole map's (today's one ordered map behind an `Arc` copies every entry:
     16.5 M instructions with 41 472 entries, measured).
   - **Per hero, by hero id:** the **once-per-hero marks** (which objects of the once-per-hero
     classes the hero has visited), a bit set by object index kept by `HeroId` beside the heroes
     on the map, not in them, so a hero that leaves the map keeps its marks (the original's hero
     record) and a fork does not copy one set per hero.
   - **Per player:** its **knowledge of objects** (the original's team marks: which objects a hero
     of the player has visited), a bit set by object index per player.

   These three live in one block behind one `Arc`, none until the first visit that writes one, as
   the flags of point 1: a fork costs one reference count more, a write copies the block's handles
   and one table and leaf of the set it changes (T55b's measurement: 1 to 3 thousand instructions
   with eight heroes and players, on the XL map's 41 472 objects; `E08-design.md` §9). The reward
   records are static (point 1's static map), one per box or event, never shared (A-19 not
   reproduced).
3. *(amended, heroes)* A visit's changes to the hero come back as hero changes (ADR-0013 point 2
   and its point 15); its morale and luck bonuses are the details'. The once-per-hero marks are
   the adventure's (point 1).
4. *(amended, objects)* **Increment 7's kinds** (closed variants with the type's parameters in
   `adventure.toml` and the instance's from the map; the import maps classes 4, 5, 6, 22, 23, 26,
   32, 39, 41, 47, 51, 61, 63, 65 to 69, 81, 88 to 90, 93, 100 to 102, 104, 105, 107, 108 and 113):
   - **Pickups that go to the hero:** a **treasure chest** (gold drawn, or a treasure artifact;
     gold asks gold or experience), a **placed or random artifact** (a random one drawn at the
     build by its classes), a **Spell Scroll** (its spell from the map). A guarded artifact
     always asks (D-29), a guarded scroll only with a text; a win picks it up. A **monster's
     treasure artifact** goes to the hero after a win or a join (the import takes it: the 8
     refused maps move on).
   - **Teachers:** the once-per-hero gifts (Marletto Tower, the Garden of Revelation, the
     Mercenary Camp, the Star Axis, the Arena, the Library of Enlightenment, the Schools of Magic
     and War, the Learning Stone, the Tree of Knowledge), the **Shrines**, the **Witch Hut**, the
     **Scholar** and the **University**.
   - **Finds** emptied by a visit: the Corpse, the Wagon, the Lean To and the Warrior's Tomb.
   - **Pandora's Box** (a reward record, a question, guards announced) and the **Pyramid** (a
     drawn spell, a question at every visit, a monster battle with a second creature, a looted
     Pyramid's luck).
   - **Events:** a reward record with its colours, computer flag and removal flag. An event is no
     visitable object: nothing is drawn or observed of it (point 6); a hero that **enters its
     tile by a step** fires it when the hero's owner's colour is in its colours (a player that is
     not human, ADR-0017 point 1's flag, only with the computer flag), and **the move ends
     there**: the rest of the path is dropped, the move's outcome, never a rejection. Its guards
     fight at once (no question); without the removal flag it fires again at every later entry,
     never for a hero that starts its move on it. **The reachable-tiles query** (A-23, decision
     131; the outcome of batch E7's row E7-23 written in by T59): if the row confirms the
     original's stop, an event's tile is an entrance for every colour (a path's end, never passed)
     under the `faithful` profile, and the query never reads events (blind) under `strict-fixed`;
     if it does not, the batch's behaviour is the rule for both.
   - **Visits.** A question is a pending decision of the visitor's owner (ADR-0017 point 6). A
     refusal (a full backpack, no spell book, too little Wisdom, a known spell or skill, eight
     skills, a price the owner cannot pay, a mark already set, an emptied find) is the visit's
     outcome, never a rejected step, and changes nothing but the visitor's owner's knowledge,
     which the original records first. A full backpack refuses an artifact or a scroll, turns a
     chest's artifact into its gold choice, a Corpse's into gold and a Wagon's into its resources,
     leaves a Tomb's, and loses a box's, an event's or a monster's (survey §8.2). After a won
     guards' battle the visit's rest (the pickup, the rewards, the spell) comes after the battle's
     levels (the original's order).
   - **Random contents of increment 7 are drawn when the game is built**, from the setup stream
     in object order, the random artifacts first: the random artifacts and a chest's, a Corpse's,
     a Wagon's and a Tomb's artifact from one pool of the ruleset's artifacts that are placed at
     random (never one banned, placed or drawn before, until the pool is exhausted: D-32), a
     chest's gold, the Tree's price, a Shrine's spell (no spell of its level repeated until each
     is out: D-34), a Witch Hut's skill, a Scholar's kind and reward, a University's four skills,
     the finds' contents, the Pyramid's spell. The scenario's bans (skills, spells, artifacts)
     are static inputs. Nothing a visit does draws.
   - **No original bound** (decision 133): a mark per object (A-17), a record per box or event
     (A-19); the import refuses a box's or an event's creature list longer than an army (A-24).
     An RoE map's Witch Hut without a mask takes the fixed 14 skills under `faithful` and every
     skill but Leadership and Necromancy under `strict-fixed` (A-21, a quirk with a flag).
6. *(amended, information)* Chests, scrolls and boxes look alike; a placed artifact's picture names
   it; every content and every draw stays hidden until a visit. A visit tells its visitor's owner
   what the original's dialog shows and records the owner's **knowledge** of the object (a
   Shrine's spell, a hut's skill, a Tree's price, a University's skills, a find emptied, a Pyramid
   looted), which that player's observation shows from then on. A hero's once-per-hero marks and
   bonuses are its owner's. An **event is never observed**, not even by its visitor's owner after
   it fired; its firing is told to the visitor's owner alone, its removal to no one. Removals of
   picked-up objects are seen where the tiles are, as a pile's. Under A-23's `faithful` stop the
   legal moves reveal where events are, as the original's path does; `docs/06-security.md` lists
   that leak beside the fringe's.

## Consequences

- The adventure state grows by one block (the state hash and its version); the army left changes
  its storage; `fork-xl-144` moves by a reference count at most once a reward was visited.
- Every caller of the reachable-tiles query (`legal_moves`, the env's masks, the properties'
  plans) meets events' tiles as entrances under `faithful`, if batch E7 confirms A-23.
- The monster battle gains a second creature (the Pyramid's), which `split` lays out by the
  original's order table (ADV-MON-012).
- The import maps 31 more classes and reads the scenario's spell and artifact bans.

## Alternatives considered

- **The original's bounds** (32 marks per class, 400 records) or a smaller bound of the engine's:
  not chosen (decision 133).
- **The marks on each hero on the map:** a fork copies a set per hero (453 instructions with eight
  heroes, measured, against 19 for one block) and a hero that leaves the map loses them.
- **Events as visitable objects** (seen, entered as any): the original never draws them.
- **A-23 as the original's everywhere, or blind everywhere** (decision 131: a flag, if E7-23
  confirms the stop).
- **VCMI's side of the disputes** (decision 129: the code's side, confirmed or reverted by
  batch E7).
