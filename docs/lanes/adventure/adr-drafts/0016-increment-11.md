# ADR-0016 amendment (draft): increment 11, the teleporters (the map side)

- Status: **Accepted** at T98b2's checkpoint (ADV-Q46, 2026-11-06; E08 decision 272; the revisit
  with decision 273), to be applied by T102. Drafted by E08 T98b1 (2026-11-06), from the survey `docs/rules/surveys/E08-teleporters.md`
  and the user's answers at ADV-Q44 and ADV-Q45 (E08 decisions 260, 264–269); for the user's
  acceptance at T98b2's checkpoint before anything is built (decision 261), then into ADR-0016
  through the round's `shared/` pull request with batch E11's outcome.
- Amends: ADR-0016 points 1, 3, 4, 5 and 6 (each text below goes in after the point's increment 10
  part); its consequences, alternatives and revision history.
- Detail: `docs/epics/E08-design.md` parts A and B (§1 to §11). The game's side (the battle and the
  meeting at an exit, the jump held as a rest, the visitor's stream, `Revisit`, the env) is
  ADR-0017's draft `0017-increment-11.md` (T98b2, design part C), which also added `Revisit`'s
  lines to point 4 here (its last bullet).
- ADR-0015 needs no amendment: the import still draws nothing, and the channels are the static
  map's, built with the game.

## Context

Increment 11 types the teleporters, inert blockers until now: the Subterranean Gates (103: 602 on
98 maps, the only link between the levels on 98 of the 104 two-level maps), the Monoliths (43, the
one-way entrances: 193; 44, their exits: 157; 45, the two-way: 495; each by subclass, its channel)
and the Whirlpools (111: 224, reached by boat); together on 146 of the 160 shipped maps. The
original builds a list of exits per channel and pairs the gates at a new game. A step onto an
entrance ends the walk, and its visit moves the hero to an exit (drawn for a monolith or a
Whirlpool, the pair for a gate), fighting an enemy hero who stands there first and, at a gate,
meeting one of the team instead; a Whirlpool first halves the hero's weakest stack. The arrival
keeps the points, fights no guard and visits nothing. No human path goes through a teleporter. The
user chose the original in every dispute (D-53 to D-58) and fixed A-37, a Whirlpool landing on an
enemy hero, by the battle (E08 decisions 264, 265).

## Decision

**Point 1** (storage), a part **Increment 11's channels** (E08 decision 260):

- **The channels are static**, built with the game into the static map from its typed objects,
  without a draw: for each one-way channel its exits' visitable tiles, for each two-way channel its
  members', one list of every Whirlpool's tiles with each tile's Whirlpool, and the gates' pairs;
  each list in the original's walk order (level, row, column: the tile index). They are shared by
  forks behind the static map's `Arc`, read by a jump's visit alone and never by the reach, and
  follow from the objects the static map's hash covers. No new per-game block: a jump writes the
  hero's tile, a Whirlpool its army, a reveal its owner's explored tiles.
- **The surfaces** are checked when the game is built: a monolith's or a gate's visitable tile on
  water, or a Whirlpool's on land, is refused (A-42: a jump between surfaces would put a hero in a
  boat on land or a walker on water; none on the shipped maps).

**Point 3** (heroes on the map), after increment 10's text: **a jump moves a hero without a step**
(point 4): its tile becomes the exit, its points and the day's maximum are kept, and a hero in a
boat jumps with its boat (an aboard boat's record keeps the tile it was boarded on). A Whirlpool's
loss lowers or empties one stack of the army and never empties the army.

**Point 4** (kinds), after the Visits bullet's last sentence: increment 11's jump is the first
visit that draws. A part **Increment 11's kinds** (the import maps classes 43, 44 and 45 by
subclass, 103 and 111; every one entered, none ownable, only the Whirlpool visitable from above):

- **Monoliths** (`monolith`: a channel number and a way; ADV-TEL-001, -004): an `entrance` (43)
  jumps to one of its channel's `exit` monoliths (44), a `two_way` monolith (45) to another of its
  channel's; an `exit` leads nowhere, told to its visitor's owner. A one-way channel and a two-way
  channel of one number are distinct.
- **The Subterranean Gate** (`subterranean_gate`; ADV-TEL-002, -005): paired when the game is
  built, in the walk's order each unpaired gate with the unpaired later gate of another level at
  the least truncated distance (an integer square root of the squared column and row distances),
  the first on a tie (D-56); one without a pair leads nowhere. Its jump draws nothing; a hero of
  the mover's team on the pair is met and the mover stays, an enemy hero fought.
- **The Whirlpool** (`whirlpool`: a divisor, the original's 2; ADV-TEL-006): first the loss, unless
  the hero is protected (`h3-heroes`' `whirlpool_protection`, from an artifact effect kind: the Sea
  Captain's Hat's, the Admiral's Hat's through its part): the occupied slot with the least creature
  fight value × count (a 64-bit product without the original's bound, A-38), the first on a tie,
  keeps its count divided by the divisor, rounded down, a lone stack at least one (D-54, D-55); every
  hero alike (A-40 not reproduced). Then the jump over the other Whirlpools' tiles (D-57), a hero
  in a boat with its boat.
- **The exit** (ADV-TEL-003): a monolith's or a Whirlpool's candidates are its channel's tiles, in
  walk order, on which a hero of another team stands, or no hero and no empty boat stands and which
  the visit does not exclude (a two-way monolith its own tile, a Whirlpool its own six). None: no
  jump (the hero stays; a Whirlpool's loss is taken); one: no draw; else one uniform draw from the
  visitor's stream (the Visits bullet; ADR-0017 point 4). No command names an exit (D-53).
- **The battle at an exit:** an enemy hero on the exit (a monolith's, a gate's, or a Whirlpool's
  drawn tile: A-37 fixed so, E08 decision 265) is fought first, as a visit's battle with another
  player's hero, the mover attacking from the entrance; the tiles within 1 of the exit are revealed
  to the mover's owner before it (point 5). The jump to that exit is the visit's rest after a won
  battle (ADR-0017's draft, points 6 and 11); a lost battle ends it.
- **The arrival** (ADV-TEL-007): no guard test and no visit at the exit, the hero's scouting radius
  revealed (point 5); the move ends. The step onto the entrance is a path's last: no path goes
  through a teleporter (D-58), and the reach meets an entrance as any entered object's tile, a
  destination, never its exits.
- **The import** refuses a monolith subclass past 7 (A-39: the original writes past its eight
  lists): an input bound, no flag.
- **The revisit** (ADV-TEL-008; E08 decision 268, the original's space key; added by T98b2): a
  hero standing on the visitable tile of an entered object whose kind has a built visit may visit
  it again (ADR-0017's `Revisit`): the kind's own visit with what it opens, as the step onto the
  tile gives it, but without a step (no points spent or required, the points and the day's maximum
  kept, no step's reveal or event) and without the guard's test (the original tests a guard only at
  a step's end; batch E11's E11-19 confirmed it). Every entered kind alike: a two-way monolith or a
  gate leads back and a Whirlpool takes its loss again, a one-way exit tells "leads nowhere" again,
  a weekly object tells it was visited; a neighbour-visited object, a boat or a tile without an
  object has no revisit.

**Point 5** (fog), after increment 10's text: from increment 11 a **jump** reveals the tiles within
the hero's scouting radius of the exit to its owner, as a step's reveal; an exit a hero occupies
first reveals the tiles within 1 of it to the mover's owner (the original's, before the battle or
the meeting).

**Point 6** (information), after increment 10's text: from increment 11 every viewer that sees a
**teleporter** sees its type (a monolith's channel: the picture's colour), never a channel's exits
or a gate's pair. A **jump** is seen as a step's two ends by the players that see the hero on
either (a new event with the step's fields and projection): a viewer that sees one end only is told
that one, so no viewer learns a hidden exit (A-41 not reproduced). A jump's reveals are told by the
explored tiles alone. A Whirlpool's loss and "leads nowhere" are told to the visitor's owner alone;
a battle at an exit is seen as any battle, a meeting at a gate as any meeting; a revisit is told
only by what its visit tells (no step).
`docs/06-security.md` lists it (the round's `shared/` task).

**Consequences**, a line: increment 11 makes the five classes typed objects (adventure ruleset part
v18, map import v20; the adventure state hash unmoved), so on 146 of the 160 shipped maps the reach
gains the entrances as destinations; the jump's draw gives `AdventureState::apply` the visitor's
stream; the Whirlpool's protection is a core artifact effect kind (pack format v51, core ruleset
hash v52).

**Alternatives considered**, a line: increment 11 (ADV-Q45): VCMI's paths through two-way
monoliths and gates with the exit named by the command (D-53, D-58), its weakest stack by AI value
and the last slot on a tie (D-54), its loss of `max(1, ⌊n / 2⌋)` (D-55), its squared distance for
the gates (D-56) and its Whirlpool draw in two steps (D-57); A-37's tile not a candidate, or VCMI's
stay without a battle; A-38's lone stack past the bound kept as a `fix-optional` flag; the
protecting artifacts as the Whirlpool kind's parameter; an exit on the other surface no candidate
instead of A-42's refusal.

**Revision history**, a line: points 1, 3, 4, 5 and 6 amended for E08 increment 11, the
teleporters: the static channels and the gates' pairs, the surfaces' check; a jump moves a hero
without a step; the kinds monolith, Subterranean Gate and Whirlpool with the drawn exit, the battle
at an exit, the arrival and the Whirlpool's loss, the import's bound, the revisit of an entered
object; a jump's reveals; a teleporter's type public, a jump seen as a step's two ends. Drafted in
E08 T98b1 (survey `E08-teleporters.md`; the revisit by T98b2); batch E11's outcome folded in.
