# ADR-0016 amendment (draft): increment 14, the trades and services

- Status: Draft (E08 T127b, 2026-11-18), accepted at T127b's checkpoint (ADV-Q58, E08 decisions
  370 to 373); applied to
  ADR-0016 by increment 14's `shared/` task (T131) with batch E14's outcome (T130), then deleted.
- Amends: ADR-0016 point 1 (storage: increment 14's objects), point 4 (increment 14's kinds),
  point 6 (information: the markets, the Altar, the Hill Fort); the Consequences, the
  Alternatives and the revision history.
- Detail: increment 14's design note `docs/epics/E08-design.md` §1 to §12; survey
  `docs/rules/surveys/E08-trades.md`; E08 decisions 361 to 369 and the checkpoint's.

## Point 1 (after "Increment 13's objects")

   - **Increment 14's objects** (E08 decision 361): **no new block.**
     - **What the build drew** for a Black Market is the static map's: its seven artifacts in the
       draw's order (three treasures, three minors, one major; an entry the pool could not fill
       left out), beside every other object's drawn contents.
     - **Its sold entries** are a map in increment 7's visits block, by object index a bit per
       entry, absent for a market nothing was bought at; a purchase copies the block once.
     - **Nothing else:** the Trading Posts, the Freelancer's Guild, the Altar of Sacrifice, the
       Hill Fort and the War Machine Factory keep no state (no visit limit, no owner, no week);
       the Altar's and the Hill Fort's visits set increment 12's kind marks.

## Point 4 (after "Increment 13's kinds" and batch E13's paragraph)

   - **Increment 14's kinds** (the import maps classes 2, 7, 35, 99, 106, 213 and 221 at subclass
     0, 221 to 99's type with its class kept for its picture; another subclass is refused, no body
     is read). **Every one is entered** (the original's class lists, E08 decision 317's rule: the
     hero stands on the entrance, the move ends there, `Revisit` visits it again), none from
     above; visited every time, by every player (the original's human-only branches are the
     computer player's behaviour, E10's):
     - **The market** (`market`; ADV-MKT-001): the screens it shows (resources for resources,
       resources to a player, resources for an artifact, creatures for resources), the count of
       Marketplaces it trades as (5 for every map object) and a Black Market's stock classes. The
       base pack's Trading Post (99, 221; trade and transfer) and Black Market (7; purchase), the
       ab pack's Freelancer's Guild (213; sale).
     - **The rates** (ADV-MKT-002): the adventure part's market parameters (two value tables by
       resource, three factor tables by the count 1 to 10) and one computation in `h3-adventure`
       that the town's Marketplace reuses with its own count (E08 decision 361): each step as the
       original stores it, binary32 products and quotients rounded half up at the x87's 53 bits,
       computed exactly in integers (`h3_kernel::arith::X87`, as quirk Q-0005's rates); a rate is
       a lot (so many given for one, or one for so many). D-71 the original's (E08 decision 365).
     - **The Black Market's draw** (ADV-MKT-006), when the game is built, from the setup stream in
       object order with increment 7's rewards and increment 13's banks: one artifact per stock
       class in order from the one pool of the random artifacts, chests, finds and banks (the bans
       and the map's artifacts marked). Never again (no restock).
     - **The Altar of Sacrifice** (`altar`; ADV-ALT-001 to -003): the experience by artifact class
       (1 000, 1 500, 3 000, 6 000, a special one 0), the artifacts never sacrificed (the Spell
       Book, the Catapult), the most per sacrifice (23), a creature's experience (5 × ⌊AI value /
       40⌋) and the factions whose heroes sacrifice artifacts or creatures (by the hero's class's
       faction: Castle, Rampart, Tower artifacts; Inferno, Necropolis, Dungeon creatures;
       Stronghold, Fortress, Conflux both). Each item's experience is the hero's gain
       (HER-EXP-001) truncated per artifact and per stack (D-73 the original's); the special
       artifacts go for 0 beside others (D-72 the original's, A-47 accepted: E08 decision 367);
       a sacrifice of no experience is refused; the army keeps one creature. Its visit sets the
       team's kind mark.
     - **The Hill Fort** (`hill_fort`; ADV-HFT-001, -002): a stack of a creature with an upgrade
       in the enabled packs (the four elementals none under the base pack alone, every map alike:
       E08 decision 378) upgrades for the cost difference times the count, the gold alone scaled by the creature's
       level (free at 1, 25, 50 and 75 % at 2 to 4, whole from 5) and rounded down once per stack
       (D-74 the original's). Its visit sets the team's kind mark.
     - **The War Machine Factory** (`war_machine_factory`; ADV-WMF-001): the war machines it sells
       (the Ballista, the First Aid Tent, the Ammo Cart, never the Catapult), each at its
       creature's cost, one of each the hero lacks, bought as its artifact into its position; the
       visit asks the hire question, and its yes opens increment 10's hire dialog with the factory
       as its source.
     - **A Black Market's purchase** is refused before paying when the hero cannot receive the
       artifact (no position takes it and the backpack is full: A-46 fixed, E08 decision 366).
     Batch E14 (E08 T129, T130; survey `E08-trades.md` §21) confirmed …*(the verdicts, at T131)*.

## Point 6 (after "From increment 13 every viewer that sees a creature bank")

   From increment 14 every viewer that sees a **market**, an **Altar of Sacrifice**, a **Hill
   Fort** or a **War Machine Factory** sees its type, the same to all (no owner); the market's rates
   are the ruleset's, public. **A Black Market's stock** is server-only but in its visitor's open
   decision, whose view shows the seven with the sold ones empty to the owner alone; a viewer keeps
   nothing of a market it visited (the original shows the stock only in the window). The Altar's
   and the Hill Fort's kind marks are increment 12's (told to the team that gains one). A trade's,
   a sacrifice's and an upgrade's changes are the game's events to the owner; a transfer shows
   its recipient its gain alone. `docs/06-security.md` lists it (E08 T131).

## Consequences (append)

- Increment 14 (E08 T127b): the seven classes are typed by five kinds of two packs, all entered;
  the market's rates live in the adventure part and `h3-adventure` and serve the town's
  Marketplace (E09 increment 5), so the town lane reads them through `h3-game` and `h3-towns`
  needs no rate; a Black Market's stock is the static map's with its sold entries in the visits
  block (adventure state hash's next version); the Hill Fort's elementals follow the enabled packs,
  not the map's format (batch E14, E08 decision 378); 1 058 objects on 148 shipped maps stop being inert blockers.

## Alternatives considered (append)

- **Increment 14's rates in the core ruleset** (`h3-content`, a `shared/` change and ADR-0015
  point 3 amended), so `h3-towns` could read them: not chosen, as the trade and the treasury are
  `h3-game`'s for both lanes (increment 10's hire dialog is the precedent). **Finite rate tables
  compiled into the part:** not chosen; they grow with the content and need the same computation
  to check them. **A Black Market's last seen stock kept by its visitor** (a knowledge as a
  visited University's skills): not chosen, a snapshot that grows stale as others buy; the
  original shows nothing. **An RoE map's Hill Forts marked by the import** (the original's RoE
  elementals refused on an RoE-format map): not chosen (batch E14's E14-14, E08 decision 378,
  replacing T127b's mark): the original's game version is the scenario dialog's, from the players'
  editions, so the Complete edition plays an RoE-format map at SoD's and upgrades them; the packs
  decide (E08 decision 104), the original's version 0 (an RoE campaign) the recorded deviation.

## Revision history (append)

- 2026-11-?? (E08 T131's `shared/` part; drafted in E08 T127b, survey `E08-trades.md`, the
  checkpoints ADV-Q56 to ADV-Q59): point 1's increment 14 objects (no new block: the Black
  Market's stock drawn, its sold entries in the visits block), point 4's increment 14 kinds (the
  market with its rates, the Black Market's draw, the Altar, the Hill Fort, the factory; all
  entered; D-71, D-73, D-74 and D-72 the original's, A-46 fixed, A-47 accepted), point 6's
  information (the stock in the visitor's decision only). Batch E14's outcome (T130, survey §21):
  every point as drafted but the Hill Fort's RoE mark, withdrawn (E08 decision 378).
