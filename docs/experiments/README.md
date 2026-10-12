# Experiments in the original game

Staged battles that the user plays in the original game to settle disputed rules and to confirm
quirks before they are implemented (OPEN-QUESTIONS Q28, Q30; E04a T2). The engine writes the maps;
agents never write into the installation. Results hold for the user's version only: **SoD,
Complete 4.0**; a rule settled here is `observation`-sourced with that version.

## Flow

1. An agent writes a **spec** (`experiments/specs/<batch>.toml`, committed) and compiles it:

   ```
   cargo run -p h3-experiment -- experiments/specs/<batch>.toml --install original-data/install \
       --map experiments/out/<batch>.h3m --template experiments/results/<batch>.md
   ```

   The map goes into the gitignored `experiments/out/`; the **results template** is committed.
   `--list heroes|creatures|skills|artifacts|spells` prints the names a spec may use (heroes with
   class and specialty).
2. The user copies the map into the game's `Maps` folder, plays it as the template says and fills
   in the template in the repo.
3. The agent evaluates the filled template against the spec's predictions and records the result
   in the rule catalog (the rule's source becomes `observation`), the quirks ledger (a confirmed
   quirk, Q30) or OPEN-QUESTIONS (if the result is unclear).

## Spec format (`h3-experiment`, `crates/h3-experiment/src/lib.rs`)

- `[map]`: `name`, `description`, `size` (36, 72, 108 or 144), `terrain` (any land terrain; the
  battlefield of every encounter on it), `instructions` (the template's numbered introduction).
  `[[map.patch]]` (E08 T10a, the adventure map's experiments): `terrain`, `from = [column, row]`,
  `to` (inclusive), a rectangle of another terrain; a later patch covers an earlier one. The
  writer gives every tile an interior picture of its terrain (the ranges the game's own maps use,
  `docs/formats/h3m.md`), so the game draws a hard edge between two terrains.
  `[[map.road]]` (E08 T10a): `kind` (dirt, gravel, cobblestone), `row`, `from` and `to` (the
  west and east end's columns): a straight road of at least two tiles with the pictures the
  game's own maps give such a road.
- `[[encounter]]`: `id` (also the hero's name in the game), `title`, `questions`, `procedure`
  (steps), `record` (`field = "…"` for a value, `table = [columns]` + `rows`, or `labels` for rows whose
  first column is filled in), `predictions`
  (for the evaluating agent; never shown in the template).
- `[encounter.hero]`: `type` (name, or id if the name occurs twice), `specialty` (checked against
  the executable so that no hero with an interfering specialty slips in), `owner` (red/blue),
  `at = [column, row]` (the hero's tile instead of the encounter's place in the grid; the
  opponent stands one column east unless it has its own `at`), `defaults = true` (E08 T10a: army,
  experience, primary and secondary skills, artifacts and spells all the hero type's own, as the
  map editor places a hero; the spec then lists none of them, and `primary` is required only
  without it),
  `experience`, `primary`, `skills` (`[name, level]`), `army` (`[creature, count]`), `artifacts`
  (slot name → artifact; slots are checked against `ARTRAITS.TXT`), `backpack` (artifact names,
  carried but not worn; E04b batch 2f), `spells` (`SPTRAITS.TXT`
  names, ids below 70; needs a worn `spell book`, E04b batch 2a). Skills, artifacts and spells
  replace the hero type's defaults, so heroes have no spell book unless one is listed. A hero
  starts with full mana: 10 × knowledge (no Intelligence), so knowledge sets the mana.
- `[encounter.opponent]`: `monster` + `count` (a savage wandering monster that never flees) or
  `[encounter.opponent.hero]` (a Blue hero). `[[encounter.also]]` (E08 T10a): more heroes of the
  encounter, each with its own `at`, named as their hero type unless `name` is given (scouts that
  explore a route, inspections of many heroes). Encounters are laid out in a grid; a hero type
  may appear once per map, a player may have eight heroes, two objects never share a tile.

Running a spec in our engine (to print predictions automatically) is not built (BACKLOG; Q37:
revisit with the next batches, E04b); the predictions are computed by hand or with a script and
written into the spec.

## Status

| Batch | Purpose | State |
|---|---|---|
| smoke (`E04a-smoke.h3m`) | prove that the game accepts our files | **works** (user, 2026-10-03) |
| 1a (`E04a-batch-1a`) | Q23 damage rounding (three encounters that separate exact, single, double and x87-style arithmetic), Q21 death stare, the hero's attack in the Ballista, Basic Artillery and First Aid control, Ballista ammunition (S-01), Elixir of Life regeneration (S-04), the Swordsman specialties (S-03) | **evaluated** (2026-10-03): `docs/epics/E04a-log.md`, "The design note at closure"; the First Aid Tent with the skill is left for 1b |
| 1b (`E04a-batch-1b`) | tactics: zone, war machines, placement rules, the disabled Tactics formation (OQ-1–OQ-3); surrender price, discounts and consent (D-1, D-2, D-5); Shackles of War against a monster (D-6); experience for war machines and with Learning, captured war machines (D-3, OQ-6, OQ-5); Necromancy rounding (D-7); the commanded First Aid Tent (T7b leftovers); war machines with creature artifacts and the Golden Bow, the Ballista's range penalty (artifacts open 1–2); Angelic Alliance mixing (artifacts D1); Xeron's speed (H-2) | **evaluated** (2026-10-04): `docs/epics/E04a-log.md`, "The design note at closure"; decisions in OPEN-QUESTIONS Q32; not staged: D8 (an E08–E10 input) |
| 2a (`E04b-batch-2a`) | Chain Lightning by screen or hex distance and its tie order (Q44, X-6), aiming Fireball and Implosion and the aimed stack's resistance roll (BAT-MAG-003/007/008, X-5), a spell specialist's rounding with Sorcery against creature levels, Golems and elementals and the Gold Dragon's immunity (X-1, X-2), the Orb with Sorcery (X-1), costs with Mages, Pegasi and Familiars and casting in the Ballista's turn (X-3, X-4), the tactics hand-over (Q40) | **evaluated** (2026-10-06): `docs/rules/surveys/E04b-magic.md` §13, `docs/epics/E04b-log.md` (T2e); Q40 answered; the costs (R6) open for T7 |
| 2b (`E04b-batch-2b`) | the x87 precision of the damage stages, 53 or 64 bits (Q50), Counterstrike's extra round (Q-0008), Fire Shield's reflection base (Q-0010), Shield with Basic Armorer in one product and Armorer alone (BAT-DMG-002, Q-0005), a counter pair: Bloodlust and Weakness add up, Haste and Slow cancel (Q47), an Armorer specialist at level 10 (Q45 (b)) | **evaluated** (2026-10-07): every encounter as the engine's faithful prediction (53 bits; Q-0008, Q-0010, Q47, Q45 (b) observed); `docs/epics/E04b-log.md` (batch 2b folded in) |
| 2c (`E04b-batch-2c`) | the anchor hexes of natural obstacles (Q55: Remove Obstacle, Fire Wall and a flyer on the free hex beside an obstacle, three battlefields), Fire Wall burning on entering, Wait and Defend, the defender's Force Field and Fire Wall shapes (BAT-MAG-027/028), a walker's path among equally short ones past a Fire Wall (BAT-MOV-003), Berserk between two equally near stacks (BAT-MAG-025), Hypnotize against a wounded stack, Expert and Basic Cure on a hypnotized stack (BAT-MAG-026, Q-0012) | **evaluated** (2026-10-07): every encounter as the engine's faithful prediction (H1's round 3 after the user's correction, BAT-Q2); A2 confirms Q55's anchors; `docs/epics/E04b-log.md` (batch 2c folded in) |
| 2d (`E04b-batch-2d`) | where a clone appears, its turn, shots, rounds and death by any damage, and the surrender price with a clone (CL1; BAT-MAG-038, BAT-FLE-002); a clone on its own crowded two-hex original (CL2, BAT-QUIRK-7); Resurrection's and Sacrifice's targets (RA1, BAT-QUIRK-5, S-15) and corpses (RA4); Basic Resurrection's temporary creatures at the battle's end, with a Ballista too (RA2, RA3; BAT-QUIRK-6, S-20); Necromancy and experience from created stacks, the elementals' native terrain (NE; BAT-Q7, BAT-TER-001); Summon Elemental's place, a dead elemental's slot, the limit of 20 stacks, a side left with only elementals against an elimination, a retreat and a surrender (SU; BAT-MAG-037/039, BAT-Q7) | **evaluated** (2026-10-08): as the code reading everywhere but the elementals' native terrain (natives on grass, as the engine; the reading of 0x43D730 refuted); the engine's surrender price refuted (created stacks left out; fixed); BAT-QUIRK-7 observed (fix-optional), BAT-Q7 answered; `docs/epics/E04b-log.md` (batch 2d folded in) |
| 2e (`E04b-batch-2e`) | the code reading of the casting limits and the spell defences where the wiki or VCMI disagree (BAT-Q21, BAT-Q22): the Faerie Dragons and Ogre Magi under the Orb of Inhibition (OI), Blind at Troglodytes, Mirth and Sorrow at golems and elementals and Poison at Gargoyles under the Orb of Vulnerability (OV), the Unicorn aura under it (UA, mass Slow four times), Recanter's Cloak against Stone Gaze and the Archangel (RC), Power of the Dragon Father against Poison and the Thunderbirds' bolt with the Death Stare as a control (DF), Bless, Curse, Precision, Fortune, Misfortune and Slayer at the war machines (WM; BAT-MAG-061, 062, 064, 066, 067) | **evaluated** (2026-10-11): the code reading everywhere but the Unicorn aura, which the Orb lifts as the wiki and VCMI say (12 of 12 slowed); the Faerie Dragons are stopped by the Orb of Inhibition (the engine followed the wiki and VCMI: fixed); RC's Stone Gaze (8 attacks) and DF's bolt and Poison (3 and 4) are weak, as no save in battle allowed the planned tries, but decisive with UA's aura (the user, BAT-Q27); `docs/epics/E04b-log.md` (batch 2e folded in) |
| 2f (`E04b-batch-2f`) | increment 17's opening-spell candidates and T8b's leftovers (BAT-Q27): when the defender's Armor of the Damned casts with its first stack blinded (AD; the code and the wiki against VCMI, BAT-Q23) and under the Orb of Inhibition (AO); BAT-Q24's learning after a won battle: a level-1 and a level-3 spell without Wisdom (EL), a spell book in the backpack (EB), Basic Wisdom as the control (EW); Power of the Dragon Father against its hero's Bless (DF), the Pendant of Second Sight against the Unicorns' Blind (PS), a hypnotized stack and its owner's Pendant of Life against Death Ripple (HP; BAT-MAG-063, 064, 068, 069) | **evaluated** (2026-10-11): every point as the engine and the code: the opening spells at the first stack that takes its turn and under the Orb; Eagle Eye's learning needs a worn book and Wisdom + 2 (BAT-Q24: the code, against VCMI; the message names the unlearned picks too); the Dragon Father, the Pendant (0 blinds in 41 attacks) and the controller's Pendant as predicted; `docs/epics/E04b-log.md` (batch 2f folded in) |
| E08 1a (`E08-batch-1a`) | the adventure map's first batch (E08 T10a): when a day's movement points are set, two hot-seat battles that take a slow stack out of an army in the other player's turn, on day 1 and day 2, and a dismissed slow stack as their control (ADV-Q4, ADV-MOV-002); which tile prices a step off a road, on grass onto swamp and on swamp onto grass (ADV-Q3's D-2, ADV-MOV-003); the land table as a control (ADV-MOV-001). Paths are counted by their marks: scouts explore each walker's row beyond the farthest predicted step. Both specs were reviewed by a second agent before staging | **evaluated** (2026-10-12): as the code reading everywhere: points set at the owner's turn start on day 1 and day 2 (2000, not 1560; ADV-Q4 answered), off a road the tile left prices the step both ways (9 and 14 steps), the land table (15). The map played was a stale build without Q4c (the review's control); the user's statement stands in for it (no change of the day's points when a stack is dismissed or gained). `docs/epics/E08-log.md`, T10c |
| E08 1b (`E08-batch-1b`) | the starting army of sixteen heroes placed with everything their hero type's own, over up to eight loads: the independence of the second and third slots' rolls and the war machine's chance (ADV-Q6, GAM-SET-001) | **evaluated** (2026-10-12): one load sufficed: a hero with its third stack but not its second (VCMI's single roll refuted) and a war-machine hero without its war machine (the wiki's 100 % refuted), as the code reading; ADV-Q6 answered; `docs/epics/E08-log.md`, T10c |

**How the user plays** (2026-10-03): the Complete edition's `Heroes3.exe` started through a HotA
installation's HD Mod launcher (HD Mod 5.6 R9). The HD Mod's always-on changes are crash fixes; its
gameplay fixes come only with the optional HD+ setting (Clone, Quick Combat, neutrals with the
Angelic Alliance, a hero without an army, AI turns, merchants;
https://www.sites.google.com/site/heroes3hd/eng/original-game-bug-fixing). Batch 1a was played with
HD+ on (no fix touches it); from batch 1b on the user plays with plain HD. Observation sources name
the setup. **The game cannot be saved during a battle** (batch 2e): an encounter that needs many
tries replays the battle from a save made before it, so plan few, long rounds instead of reloads.
A stack either moves or Defends in a turn, never both (batch 2f): a procedure that walks a stack
up says so and lets it end its turn there.

Not staged: Q22 (jousting: path length or distance). Showing it needs a controlled detour and,
because Champion damage is random, many repetitions; reading the game's code is the cheaper last
resort (Q29) when T-tasks need it. Q26 and First Aid with the skill are settled by the code
reading in `docs/formats/installation.md` §7.5.

**Candidates for the next batch** (E04a T12c review, 2026-10-05): rules implemented after VCMI or
by generalisation and not observed in SoD. (1) Spirit of Oppression against a creature's minimum
morale (Minotaurs; VCMI lets the cap win, BAT-HER-003; the original's text only says it negates
all morale bonuses). (2) Luck and the Artillery double damage on one shot: added or multiplied
(VCMI adds, BAT-WAR-006). (3) The surrender price with an Ammo Cart and a First Aid Tent (only the
Ballista was observed, BAT-FLE-002). (4) The Bow of the Sharpshooter: may a shooter next to an
enemy still attack in melee (BAT-HER-006 lets it only shoot). (5) The order in which the tactics
phase hands over the stacks and what "next creature" does (OPEN-QUESTIONS Q40, with the
experiment; the user: the order is a regular round's, but "next creature" acts more like Wait).
Batch 2a staged (5) with the magic candidates (answered: Q40); (1)–(4) wait for a later batch,
as do Q-0005's single-precision resistance (a one-point difference in a chance), Q43 (the sprite
offset in Chain Lightning's jumps), a skill specialist's growth at a high level (Q45) and the
spell costs with Mages and a dead Pegasus (R6, T7); from T4a, a Magic Mirror sending a spell to the
caster's own war machine (VCMI never does) and Sorrow surviving a Basic Anti-Magic. Batch 2c staged
Q55 with T5a's and T4c's candidates (the user, Q56); the others above still wait. From T5c1
(Quicksand, read in the code, `docs/formats/installation.md` §7.16): the pits by mastery (4, 4, 6,
8) and none on a corpse's hex; the natives' sight fixed at the battle's start (BAT-QUIRK-1: a side
whose natives died still sees Quicksand cast later); a two-hex walker stopped by its tail; whether
the interface offers an attack from a seen pit and what happens (BAT-QUIRK-2); Fire Wall's cursor
refused on the other side's hidden pit (Q-0002); a hypnotized walker stopped by its owner's pit
without revealing it. From T5c2 (Land Mine, `installation.md` §7.17): the spell book refusing Land
Mine against an army whose only natives are immune to it (Green Dragons on grass: the code
refuses, the wiki says it is cast); a stack teleported onto an unseen enemy mine that then
defends (the mine goes off); a walker that survives a mine walking on and still attacking; a
hypnotized stack crossing its owner's mine unharmed and the caster's mine harmed; a fire-immune
stack crossing a mine that stays. From T5b (Remove Obstacle, `installation.md` §7.15): Remove
Obstacle at Basic refused on a Fire Wall and at Advanced on a Force Field; a usual obstacle's anchor
hex that a second usual obstacle covers (Remove Obstacle there takes the covering one, after which
the hex leads to no obstacle: 0x466B30 clears it); the original's random obstacles placed on
another obstacle's free anchor hex (the generator's anchor test was not read). Batch 2d staged T6a's and T6b's
candidates (the user, T6 checkpoint 2026-10-08: BAT-QUIRK-7, BAT-Q7 and the clone's shots before
they are decided) but two: a clone killed by 0 damage (the original kills a clone whatever the
damage, VCMI 1.7.5 not by 0; no hero spell deals 0 in SoD) and the game setting that keeps the
elementals' native terrain (0x43D730, not read; batch 2d showed them native in the user's setup). New from batch 2d: a reused record's losses (SU a's casualties list 2 Air Elementals of at least 4 killed) and so the Necromancy from elementals killed before their record was reused. From T7a1 (the strike's spells,
`installation.md` §7.20; the wiki was not consulted): Black Knights cursing with 25 %; a Dragon
Fly's Weakness at Advanced (−6 attack) and not renewed on a weakened stack; a Thunderbird's bolt
refused by Green Dragons (Lightning Bolt's level) and halved by Protection from Air; a retaliating
Unicorn's Blind stopping a Wolf Rider's second strike; a hypnotized Unicorn blinding its owner's
stack. From T7a2 (the statuses): a petrified stack struck for half that then retaliates (the
strike ends the petrification first); Poison's lost hit points staying after it ends and coming
back with a Cure; 9-hit-point creatures poisoned once and aged keeping 4 (Q-0005; exact 5);
Age leaving a wounded top creature at 1 hit point. From T7a3 (Bind, Acid Breath): a bound stack
freed by its Dendroid's walk or death and by Teleport on either; still bound while a second
Dendroid binds it;
Dendroids binding a Ballista (no test); Dispel ending Bind, Cure not; a raised stack that the
same Dendroid killed and strikes again (BAT-Q8); Acid Breath stopping at defense 0 and not
undone by Dispel; the Rust Dragon's extra damage (half a roll of 50 per dragon: 25 each). Batch 2e staged BAT-Q21's and BAT-Q22's points (T8b, T8b3); T8b's other candidates wait: a Unicorn's Blind against a Pendant of Second Sight, a hypnotized stack protected by its controller's Pendant, a hero's own Bless at its Dragon Father stacks.

**From the battle queue's inbox** (moved there at E04b T11a, the closure's triage; none staged
yet). From T4b: a blind stack acting in the same round after a shot ends its blindness; one
forgetful archer dealing full damage. From T7b2a: a Faerie Dragon's Chain Lightning or Fireball
aimed at an own stack (the cursor allows it, a hero's Chain Lightning does not); its Magic Arrow
sent by an enemy Magic Mirror to an own Black Dragon (BAT-QUIRK-9's faithful variant); a Frost
Ring that must be centred on a stack it then spares. From T7b2b: a Genie that only ever casts
Slayer while a dead king-1 enemy lies on the field (BAT-QUIRK-10). From T6c and T7b3: an
Archangel resurrecting a wounded Archangel stack of its own side (the cursor allows another, not
itself); Demons raised from Demons into the same slot, and their place in the turn order on a
speed tie; a Pit Lord's raise with a hypnotized Pit Lord. From T7b4: Enchanters whose side's
first stack is dead (BAT-QUIRK-13: no spell on their own side any more) or a shooter (never
Bloodlust); the round of the first cast in the first battle after launching the game against a
later one (BAT-QUIRK-14); a hypnotized Enchanter casting for its controller. From T7c1: a Wraith
that waits (no drain at its turn in the wait phase) and one with good morale (two drains in a
round; VCMI drains once per round); a hero's Familiars channelling above its maximum mana;
Familiars of the caster's side hypnotized by the enemy (they channel to the enemy hero). From
T7c2a: a hero's Magic Arrow at Faerie Dragons sent back about one cast in five; a hypnotized
Faerie Dragon (Hypnotize takes it, as the code reads); Efreet Sultans reflecting 20 % of a
strike's first stage, raised by their hero's Sorcery; Psychic Elementals against Skeletons
(half) and against Pikemen (full); Magic Elementals against Black Dragons (half) and against Red
Dragons (full). From T7c2b and T7c3: a Phoenix killed by a retaliation getting its good-morale
turn; Phoenixes killed by their own hero's Armageddon; a stack frozen by Fear on its good-morale
turn; Fear at the battle's first activation (never, without Tactics); a hypnotized Azure Dragon
frightening its controller's stacks (BAT-QUIRK-18). From T8c and T8d (left out of batch 2f): a
first stack frozen by bad morale delaying the opening spells; Eagle Eye's float truncation (a
Basic Eagle Eye specialist at level 2 has 43 %, not 44 %: statistical, many casts; only if a
cheap setup exists). From batch 2f's design: the engine never offers a hero's negative spell at
its own hypnotized stack (the spell's side is tested by the stack's owner), while the defences go
by its controller (BAT-MAG-063); the original's cursor there is not read. From E04b T11a: a stack
whose blindness ends in the wait phase is not checked for bad morale (BAT-MOR-004, the code).

**From the E04b review** (T11b, 2026-10-12; none staged: "batch 2g" in the battle queue's inbox).
The review's code readings that stand against VCMI or the wiki, or alone (OPEN-QUESTIONS BAT-Q28, BAT-Q32 and BAT-Q33; BAT-Q29 to BAT-Q31 are answered and the engine follows the readings there, so their rows are confirmations only; the quirk candidates S-23 to S-27 of `docs/rules/surveys/E04b-magic.md` §16), grouped by
set-up ("row" is a row of BAT-Q28). *One Faerie Dragon session* (dragons led by a hero; the F key
casts the drawn spell, the cursor is the control; it joins T7b2a's candidates above): F at a stack
under Expert Magic Mirror, ten casts: none is sent away (the reading, VCMI, the wiki and the engine since BAT-Q29's answer); with Fireball or Frost Ring drawn, F and a click on an empty hex
between two stacks: cast (the reading) or refused, and the two corners, F after the side's hero has
cast this round and F for dragons without a hero (the reading: it reads a hero that is not there,
so a battle that may crash; BAT-Q32); one dragon's Fireball under the Orb of Tempestuous Fire: 120
(the reading) or 80 (the engine and VCMI), its Magic Arrow 70 either way (BAT-Q28 row 2); batch
2e's OI set-up again with F pressed on the dragons' turn (the batch tried only the cursor; BAT-Q32,
BAT-Q21 point 1). *One row each:* Chain Lightning's tie: an enemy stack A as the first target, one
stack of each side of one creature type (Q43's offset cancels) two hexes from A in A's row: the
attacker's stack is struck second (the reading) or the defender's (row 1); defense 12, Defend,
Expert Stone Skin, right-click: 20 (the reading) or 21, and a defending stack of defense 0 struck
by Rust Dragons: Acid Breath lands (the reading) or not (row 3); an Archangel with 5 hit points
left attacks 20 Efreet Sultans and dies of the reflection, then a second stack attacks the Sultans
in that round: not struck back (the reading) or struck back (row 4, S-23); Cavaliers with their
head hex on a seen Quicksand pit ordered three hexes along the row: one hex (the reading) or three
(row 5); a stack acts in round 1 and is killed, is raised in round 2 on another stack's turn: no
turn in round 2 (the reading) or one, with a control killed before it acted (row 6); the surrender
dialog with 5 of 10 Swordsmen (750) and after a Basic Resurrection to 8: still 750 (the reading) or
1 200 (row 7); stacks of speed 3 and 4 wait, and in the slower one's turn of the wait phase its
hero summons Air Elementals (speed 7): the stack of speed 4 acts next (the reading) or the
elementals, and is Wait offered to them (the reading: no); with it, a stack frozen by bad morale
followed by a speed tie across the sides: the frozen stack's side goes first (the reading) or the
other (rows 8, 9); Wolf Riders die, a Swordsman dies later on their rear hex, Resurrection aimed at
the Wolf Riders' front hex: the Swordsman comes back (the reading) or the Wolf Riders (row 10,
S-24); Basic Dispel's cursor on an own stack whose only effect is Acid Breath: refused (the
reading) or offered (row 12); Frenzy on a stack that defends twice and then attacks: still frenzied
(VCMI) or not (the reading; BAT-MAG-019; only if the wiki is silent); Berserk on a stack whose own
side has a hidden pit on its short way: does it go round (the reading and the engine; BAT-Q31, answered: kept as the original). The earlier
candidate "whether the interface offers an attack from a seen pit" (BAT-QUIRK-2) is answered by the
review's reading: it offers none, and the engine accepts none since BAT-Q30's answer; one look at the cursor confirms it. Two rows on battles
that may crash or hang, only for their ledger lines: a Fire Wall removed at once by Expert Dispel,
a Force Field cast onto its hexes, then the round in which the Fire Wall would have ended (S-26);
elementals summoned in the wait phase and blinded before their turn (S-27). *Statistics rows:*
neutral stacks at morale −3 over a few hundred activations: 18.75 % freeze (the reading) or 25 %
(BAT-Q33, S-25); the strike's spells where VCMI contradicts the reading (BAT-Q28 row 13; most are
candidates "From T7a1" to "From T7a3" above): by counting, the Black and Dread Knights' and the
Mummies' Curse (25 %; VCMI 20, 20 and 50 %) and the Rust Dragon's extra damage (20 %; VCMI 30 %);
by one observation each, new here: a stack blinded by Unicorns retaliating at half, Stone Gaze
resisted by Dwarves, Age on 25 hit points (13, the reading, or 12), Poison counted per round or per
own turn; Dwarves beside the first target of a four-target Chain Lightning, the other stacks far:
struck in about 99 % of the casts (a roll at every jump) or 80 % (VCMI: one roll per cast;
BAT-MAG-006). *The Orb of Vulnerability rows* (BAT-MAG-066; BAT-Q21 asked for experiments on this
Orb): batch 2e's UA again with a control, the same battle without the Orb to show that the set-up
produces resists, and at least 30 rolls beside the Unicorns with it (0.8^30 = 0.1 % under the
reading; row 11); the defender's hero wears the Orb and the attacker's Chain Lightning jumps near
the attacker's own Black Dragons: struck (the reading) or not (row 11); A wears the Orb, B the
Pendant of Negativity: A's Lightning Bolt cursor at B's stack (no target by the reading and the
wiki, a hit in VCMI), and Slow from the Orb's owner at stacks under Power of the Dragon Father
(BAT-MAG-063, 064).

**Wiki checks the review asks for** (no wiki access in the review; each with its rule or question;
"row" is a row of BAT-Q28). Chain Lightning: equal distances beyond "randomly chosen", and are own
stacks preferred (row 1). Sorcery and its specialists: a worked number that tells 58 from 57
(BAT-MAG-004, Q-0005). Unicorn: whose Unicorns protect a hypnotized stack (BAT-MAG-008). Spell
book, Combat: can a mass or battle-wide spell that would affect nobody be cast (BAT-MAG-003).
Faerie Dragon: do the hero's Sorcery, Orbs or specialty raise its area spells and chain (row 2);
can its area spells be aimed at an empty hex, and
the wording on the Orb of Inhibition, the anti-magic garrison and Cursed Ground (BAT-Q32). Magic Mirror (checked 2026-10-12: the page says "Spells cast by creatures (unlike the ones cast by human-controlled heroes) will never be reflected", BAT-Q29): can the reflected spell go to the caster's war machines (BAT-MAG-022; the page's parenthesis fits the reading that the computer player's spells are never reflected). Morale,
Luck and their four spells: does a base beyond ±3 absorb the spells (BAT-MAG-011); do computer or
neutral armies freeze less often (BAT-Q33). Dispel, Hypnotize: does Expert Dispel end Hypnotize, on
the caster's own stack too (BAT-MAG-016, 026). Disrupting Ray, Rust Dragon, Resurrection: does the
defense loss survive death and a raise (BAT-MAG-010, 046). Defend, Stone Skin, Prayer: is the +20 %
fixed when the stack defends (row 3). Slayer: the list at no skill (BAT-MAG-014). Behemoth, Ancient
Behemoth: the rounding of the defense reduction (defense 10: 5 or 6; BAT-DMG-006, Q-0005). Frenzy:
how long it lasts, and does a retaliation end it (BAT-MAG-019). Fire Shield: a retaliation against
an attacker the shield killed, and the HotA changelog (row 4, S-23). Anti-Magic: which active
spells the cast removes, and are the own side's spells blocked too (BAT-MAG-021). Adela (the Bless
specialty): the levels divided first (VCMI multiplies first; BAT-MAG-020). Hypnotize: who may
attack a hypnotized stack, does it block the hypnotizer's shooters, the First Aid Tent and the Ammo
Cart for it (BAT-MAG-026). Berserk: can a player attack or shoot his own berserk stack
(BAT-MAG-025). Quicksand: is a two-hex creature stopped a second time, and can a creature in a pit
attack by stepping one hex (row 5). Quicksand, Land Mine: the natives' sight for a side that
"started the battle with" natives, or only while they live (BAT-QUIRK-1, whose strict-fixed variant
follows neither that wording nor VCMI, which counts dead natives too). Harpy, Fire Wall: is a Harpy
burnt when it returns onto a Fire Wall hex (BAT-ATK-004). Remove Obstacle, Dispel, Fire Wall, Force
Field: a crash, or a later obstacle vanishing, after a timed obstacle was removed early (S-26).
Resurrection, Archangel: does a stack raised in a later round act if it had acted in the round it
died, and the HotA changelog (row 6). Sacrifice with Efreet, Efreet Sultans or a Phoenix: may a
fire-immune creature be raised (BAT-MAG-036, BAT-QUIRK-5). Surrender: is the price taken from
creatures raised by Basic Resurrection (row 7). Clone: do expired clones count towards a side's 20
stacks (BAT-MAG-037). The strike's spells (BAT-MAG-040 to 047, row 13): Black Knight, Dread Knight,
Mummy: the chance of Curse; Rust Dragon: the chance of the extra damage; Unicorn: the level of its
Blind (a retaliation at half, or none); Dragon Fly: the level of its Weakness (−3 or −6);
Thunderbird: is the bolt treated as Lightning Bolt (immunities, Protection from Air, Anti-Magic);
Stone Gaze: can Dwarves or a hero's Resistance resist it, the retaliation against the attack that
frees the stack, is a shot halved; Paralyze: the retaliation at 25 %; Poison: per round or per
turn, the floor, do the hit points stay lowered, Cure and Dispel; Age: the rounding (25 to 13 or
12); Bind: what frees the stack. BAT-ATK-004's two wiki quotations (the reviewer could not check
them). Ogre Mage: the duration of its Bloodlust (6; VCMI 3; BAT-MAG-048). Master Genie: the
duration (6; VCMI 5), Frenzy in the list, Prayer's and Slayer's conditions (BAT-MAG-050).
Enchanter: the eight spells and the round of the first cast (BAT-MAG-053). Azure Dragon: who is
immune to Fear (the Gargoyles decide), its chance, a stack that waited (BAT-MAG-060). Phoenix: the
rebirth's numbers, once per battle, does a failed rebirth use it up (BAT-MAG-059). Psychic and
Magic Elemental: the half damage (BAT-DMG-010 has no source beside the reading). Combat: the
wait-phase order of a stack that did not wait, and can it wait; does a frozen stack count as its
side's chance to act (rows 8, 9). Familiar, Mage, Pegasus, Wraith: can mana go above the maximum
through the channel (BAT-MAG-055). Orb of Vulnerability: the sentence on artifact immunities and on
the Unicorns' aura, and does either cite a test (row 11; BAT-MAG-063, 064, 066). Recanter's Cloak,
Power of the Dragon Father: statements on creature abilities (BAT-MAG-062, 064). Eagle Eye (SoD):
does any page give the odd values 43 % and 64 % (BAT-MAG-069, Q-0005). Badge of Courage: "immunity
to mind spells" for SoD (BAT-MAG-065).

The smoke map is generated by `cargo run -p h3-formats --example smoke_map --
original-data/extracted/H3bitmap/Objects.txt experiments/out/E04a-smoke.h3m`.
