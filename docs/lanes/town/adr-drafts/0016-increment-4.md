# Draft: ADR-0016 amended for E09 increment 4 (the Tavern)

- Status: **Draft**, the hire's half written by E09 T33a (2026-11-16); T33b adds the
  information's half (the Den of Thieves' kind, the views) and asks the checkpoint (`TWN-Q18`);
  then moved into `docs/adr/0016-adventure-state.md` by the increment's `shared/` task. Every
  change here is in the adventure lane's paths: `cross-lane`, merged in order with its pull
  requests.
- Decisions: the epic's 99, 105, 106, 109, 111, 112, 113.
- Working detail: survey `docs/rules/surveys/E09-tavern.md` §3, §4, §12; design note
  `docs/epics/E09-design.md` Part F §64, §66, §67, §69.

## Point 3, after E09 increment 3's garrison hero ("… the counts being the town's (ADR-0028 point 6).")

**E09's increment 4** (TWN-TAV-003, -006; E09 decisions 109, 111, 112): **a hero placed in play**.
The adventure **places** a hero the game hires (ADR-0017 point 1): onto a town's entrance, where
it becomes the town's visitor (the original's `hero_place` 0x4D7900, `hero_show`), or onto the
adventure map's Tavern's visited tile (below), refused, typed, when a hero already stands there;
the hero joins its owner's heroes on the map with the **away record** the game gives (its army,
its points left and the day's maximum, its formation and tactics flag; no battle bonuses, no timed
marks). No reveal: neither hire reveals around the hero (the reading: `town_tavern_hire`
0x5C12E0 and `map_tavern_hire` 0x4D7890 read whole, no reveal call; the town's own radius and the
visitor's walk explored the tiles), and nothing is visited. **A hero that leaves the map** (a
battle's, a capture's empty garrison hero's, a dismissal's) **hands back its away record** to the
game (its army, points, formation and tactics flag as they were; its battle bonuses and timed
marks dropped, E08 decision 307), which keeps or clears its parts by the leave's kind (ADR-0017
point 11). A dismissal removes a hero on the map as a lost hero's removal does: its boat freed
(ADV-SEA-006), the object under it restored (a town's visitor gone), told as `HeroRemoved` to its
owner and to the players that see its tile. The adventure checks the dismissal's map half (the
hero not on a town's entrance); the game the rest (its player, its last hero, ADR-0017 point 1).
The setup's stand rule (ADV-MAP-001's `StandError::Visited`) is unchanged: only a hire places a
hero on a tile its object is visited from.

## Point 4, a new kind after E09's kind `Town`

- **E09's kind `Tavern`** (increment 4; class 95; E09 decision 106; survey `E09-tavern.md` §3):
  **visited from a neighbour** (its entrance blocked, VCMI's `blockVisit`), not visitable from
  above as the template says, no owner, no mark, no draw. A step toward its entrance from a
  neighbouring tile visits it (the move ends, the hero stays on the tile before) and the visit's
  outcome (ADR-0017 point 10) opens **the game's tavern decision** for the visitor's owner
  (ADR-0017 point 6): every hero visits, the original's computer's own paths being E13's. A hero
  standing on the entrance (a hero hired there) hides the Tavern: a step onto that tile meets or
  fights the hero, as on an owned object's entrance (point 3), and a `Revisit` of a hero standing
  on it visits nothing (the original's case 95 skips the visit when the visitor stands on the
  visited tile, 0x4AA183). The import maps class 95 to it (the 93 Taverns on 47 shipped maps,
  survey §1); the writer writes it.

## Point 6, a new sentence

**From E09 increment 4** a hired hero's appearance is told as a hero's coming into sight: to its
owner, and to every player that sees its tile (the original shows it on the map; T33b's views
give the events and their fields). Its away record and its tavern slot are its owner's (T33b).

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 4's `shared/` task): points 3, 4, 6 (a hero placed by a hire and its
  away record, the dismissal's removal, the kind `Tavern`; T33b's Den of Thieves and views).
