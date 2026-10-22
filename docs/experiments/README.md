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
   **The stamp** (E08 decision 18, from batch 2): 8 hex digits of the CRC-32 of the spec's text,
   Windows line endings made Unix ones (a Windows and a Linux checkout agree). The map's
   description starts with `Stamp <hex>.` (the scenario list shows it) and the template's top
   says `**Stamp `<hex>`.**`; the user compares them before playing. A mismatch means the map
   is not from the template's build (batch 1a's stale map, T10c): compile again and copy the map
   in the same step. An agent checks a map in hand by reading its description back (`h3-map`) or
   by the stamp of the committed spec (`h3_experiment::stamp`). The stamp covers the spec's text
   only, not the compiler or the installation's tables: after a compiler change, compile every
   staged batch again before the user plays it.
2. The user copies the map into the game's `Maps` folder, plays it as the template says and fills
   in the template in the repo.
3. The agent evaluates the filled template against the spec's predictions and records the result
   in the rule catalog (the rule's source becomes `observation`), the quirks ledger (a confirmed
   quirk, Q30) or OPEN-QUESTIONS (if the result is unclear).

**Agents running a batch themselves** (the user, 2026-10-13): the client lane's capture harness
(`clients/capture/README.md`, E17 M2 T1/T3) is a good basis for agents to play these experiments
without the user. It runs the original game unattended under Wine in the cloud sandbox (a copy in
`original-data/wine/`, never the installation), steers its menus by fixed coordinates and
recognises screens by probe pixels (`h3drive.py`), runs it on a virtual clock (`xdd.dll`) and
reads its state from memory (`h3mem.py`: the battle's stacks; the adventure map's heroes,
players and paths; the address map in `docs/exe/`). Before a batch relies on it: plan it with the
batch (recipes for the batch's own clicks and memory reads for what the template asks),
cross-check the first run against a batch the user already played (the battle's E04a batch 1a:
its R1 matched the user's log, E04c T3), and record the run's setup in the source (the same Complete 4.0 `Heroes3.exe`,
but under Wine and without the HD mod the user plays with). The staging it needs (the
executable, its DLLs, the LODs and sound archives) is listed in the runbook; the user's computer
must be online for it.

**On the adventure map it is built** (E17 M2 T6, 2026-10-13; `clients/capture/README.md`, "An
experiment batch"): a recipe in `clients/capture/batches/<batch>.toml` (the spec's procedure as
steps: a hero, a tile clicked twice, the template's fields each step fills, turns ended) and

```
python3 capture/h3adv.py batch capture/batches/<batch>.toml   # from clients/
```

compile the spec, check the stamp, play the map in hot seat and write the results template filled
in (`clients/out/batches/<batch>/<batch>.md`, a note at its top on how it was played), which an
agent evaluates like a filled one from the user. E08 batch 2 cross-checked: D3a–D3d as the user's
results, every field; D5's piles within their ranges (`docs/epics/E17-design.md` §6.3). Since
E17 H1 and H2 (2026-10-19) it also answers Yes/No questions, plays battles started from the map
(by auto combat, a retreat or a surrender), chooses the difficulty, walks on the underground,
scrolls the hero list, plays a second day and reads heroes' experience, levels and skills and a
tile's monster (the runbook, "An experiment batch"); what a batch still needs the client lane
builds when the batch asks (its queue's inbox).

**The battle's openings** (E04c T3): `experiments/agent/opening.py` (below, "One encounter's
map"). **Trap: the starting bonus.** The harness begins a single scenario with its default, a
random starting bonus, which a third of the time gives the hero an artifact (seen: Necklace of
Swiftness, Clover of Fortune, Cape of Conjuring, and a +1 morale): read the hero's worn artifacts
(hero +0x12D) and play a battle again when one appears (`opening.py --allow`). The first battle
batches run so: E04c 3a and 3b.

**A siege played by an agent** (E04c T5e): `experiments/agent/siege.py <out> <maps…>` plays each
map's encounter in hot seat (Red and Blue human, so the agent commands both armies, a town's
garrison too) from a plan in the script (round, stack, action: move, attack from a side, shoot,
wait, defend; every other turn defends), stepping the clock 10 ms at a time and reading the
stacks, the gate, the walls and the mines after each step (`timeline.jsonl`, `steps.jsonl`, one
summary line per step). Rounds are counted by the stacks' turn flags (+0x4F0, cleared at a round's
start); the keep and the Catapult act on their own. Batch 3c ran so. `siege.py`'s "walls" are the
parts' picture state (+0x13FA8); their hit points are +0x13F60.

**The Catapult and Earthquake played by an agent** (E04c T6d): `experiments/agent/catapult.py`
builds on `siege.py`: a plan's step holds for a round or from a state on (a wall destroyed), the
Catapult under Ballistics is aimed at the first standing target of a list (a click on its aim
hex), a hero casts the first spell of its book (`c`, then the book's first slot at 243, 125), and
the parts' hit points (+0x13F60), the stacks' flags (+0x84, dead 0x200000) and each change of
the parts (`walls.jsonl`) are read. Rounds are counted by the living creatures' acted bits
(0x4000000, cleared at a round's start): the turn flags' clear moment lasts less than a step once
the keep is down and the Catapult has no target. A hero with a spell book lights the panel's book
button, so the battle screen is recognised without its probes. The game's draws repeat exactly
from run to run of the same map. Batch 3d ran so.

**The arrow towers played by an agent** (E04c T7c): `experiments/agent/turrets.py` builds on
`catapult.py`: a turret's shot is its target's drop of hit points while the turret is the battle's
last activated stack (+0x132B8/+0x132BC: an automatic turret is the active stack +0x132C8 only for
an instant), the order within a round the order of activations; a commanded turret is clicked at
its target, a Wait pressed. With a human command in the battle the later draws varied between runs
(AT: two variants in four runs, every command at the same game time), so such rows want two runs.
Batch 3e ran so.

**A town's other rules played by an agent** (E04c T8): `experiments/agent/town.py` builds on
`catapult.py`: it reads each stack's morale and luck (+0x4E8, +0x4EC) and the battle manager's
town, fort, moat and tactics flags at the opening, and at every turn it commands the battle
window's surrender (2001) and retreat (2002) buttons (the window +0x132FC, its items a list from
+0x2C, next +0x8, id +0x10; a disabled button's word +0x16 has the bit 0x8); a planned cast clicks
the stack's own hex if the plan says so (Teleport's first target) and then tries hexes in order
until one starts something within 1.5 s (a refused click changes nothing read), else Escape. Both
human players get a random starting bonus: a Blue hero's artifact is retried as Red's. Batch 3g
ran so.

**The automatic Ballista played by an agent** (E04c T7e): `experiments/agent/ballista.py` runs
`turrets.py`'s play on the open map: the Red hero steps east into the Blue hero
(`h3drive.into_battle`), every stack defends (a commanded Ballista too: the plans list a Defend for
every machine record), and Red's Ballista's shot is its target's drop of hit points while it is the
last activated stack, printed one line per shot. Batch 3h ran so.

## Spec format (`h3-experiment`, `crates/h3-experiment/src/lib.rs`)

- `[map]`: `name`, `description`, `size` (36, 72, 108 or 144), `terrain` (any land terrain; the
  battlefield of every encounter on it), `instructions` (the template's numbered introduction).
  `[[map.patch]]` (E08 T10a, the adventure map's experiments): `terrain`, `from = [column, row]`,
  `to` (inclusive), a rectangle of another terrain (also `water`, E04c T3: the coast, and `rock`,
  E08 T22a1); a later patch covers an earlier one. The
  writer gives every tile an interior picture of its terrain (the ranges the game's own maps use,
  `docs/formats/h3m.md`), so the game draws a hard edge between two terrains.
  `[[map.road]]` (E08 T10a): `kind` (dirt, gravel, cobblestone), `row`, `from` and `to` (the
  west and east end's columns): a straight road of at least two tiles with the pictures the
  game's own maps give such a road.
  `[[map.pile]]` (E08 T16b): `resource` (wood, mercury, ore, sulfur, crystal, gems, gold),
  `row`, `from` and `to` (columns, inclusive; equal for one pile) and `amount` (as the map stores
  it: gold in hundreds, at most 524 287 (`0x7FFFF`, refused above); 0, the default, is drawn when
  the game starts): a resource pile on every tile of the row
  from `from` to `to`, the template from `Objects.txt` (class 79). A pile is visited from a
  neighbouring tile and the visit ends the move; the template does not list piles, the
  encounters' procedures say where they are. Since E08 T22a1 a row may lie on the underground
  (`level = 1`) and be guarded: `guards` (`[creature, count]`, one to seven stacks, every pile
  of the row with the same) and `question` (the text the game shows before the guards' battle;
  absent, the guards fight at once; only with guards).
  `[[map.ground]]` (E04c T3): `picture` (a special ground's picture in `Objects.txt`: `AVXcrsd0`
  Cursed Ground 21, `AVXplns0` Magic Plains 46, `AVXcf0` Clover Field 222, `AVXcg1`–`7` and
  `AVXmp1`–`7` the SoD Cursed Ground 223 and Magic Plains 230, `AVXef0`, `AVXff0`, `AVXhg0`,
  `AVXlp0`, `AVXmc0`, `AVXrk0` the other fields, `AVXfw0` Favorable Winds) and `at = [column,
  row]`, its bottom-right tile. The picture covers its width leftwards and its height upwards, every
  tile of it (its `.msk`): the `0` pictures 6 × 4, `AVXcg5` and `AVXmp5` 3 × 3. Heroes and
  monsters stand on it; the grounds are written first, so they are drawn below; two may overlap
  (the game takes the last, E08 T32a's batch E4b checks it).
  `[[map.object]]` (E08 T32a): an object without a body, `picture` (the Cartographers:
  `AVXmapw0` water, `AVXmaps0` land, `AVXmapu0` subterranean; since E08 T36a the trees, classes
  135, 137 and 155 in `Objects.txt` (`AVLtr3d0`, `avlautr0`, `AVLPNTR0`: one tile), and the
  Border Gates, `AVXbgt00` light blue … `AVXbgt70` black) and `at`, its bottom-right tile, on
  the surface (the land Cartographer covers `at` and its entrance, the tile west of it; a Border
  Gate `at` and the two tiles west of it, the middle one its entrance). No hero, monster, pile or
  town stands on a tile it covers: a hero comes to it in play.
  `level_limit` (E08 T32a): the scenario's level limit (0, the default: none).
  `banned_skills` (E08 T36b): the secondary skills the map bans, by `SSTRAITS.TXT` name (none,
  the default): no level-up offers a banned skill the hero does not know (HER-LVL-003), so a map
  banning all but two forces a level-up's offers for heroes who know neither (batch E5e). The
  scenario's information does not show them: the spec's instructions say so.
  `difficulty` (E08 T22a1: `easy`, `normal`, `hard`, `expert` or `impossible`): the template
  says to choose it in the scenario list (the map does not hold it; the harness's recipe names
  it too). `[map.underground]` (E08 T22a1): a second level, `terrain` (a land terrain) and its
  own `[[map.underground.patch]]` rectangles (`rock` for walls); no roads or grounds on it.
- `[[encounter]]`: `id` (also the hero's name in the game), `title`, `questions`, `level` (E08
  T22a1: 1 puts the encounter's heroes and opponent on the underground; the template says so),
  `procedure`
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
- `[encounter.opponent]`: `monster` + `count` (a wandering monster one column east of the hero; savage
  and never fleeing unless E08 T22a1's `disposition`, `compliant` … `savage`, and `flees = true`
  say otherwise; it does not grow),
  `[encounter.opponent.hero]` (a Blue hero) or `[encounter.opponent.town]` (E04c T5b: a siege): its
  entrance is the tile north of the encounter's hero, its picture needs 5 columns west of the
  entrance, 2 east and 5 rows north (give the hero `at`); `faction` (castle … conflux), `owner`
  (`blue`, the default, or `neutral`), `buildings` (the built ones by name: `fort`, `citadel`,
  `castle`, `tavern`, `town hall`, `mage guild 1`, `dwelling 1`, `upgraded dwelling 1`, `horde 1`,
  … and the faction's special buildings, `escape tunnel`, `brotherhood of the sword`, …; the
  Village Hall always; absent, the editor's standard set without a fort), `garrison`
  (`[creature, count]`, at most seven; empty, the editor's default) and `tight`; with an
  `[encounter.opponent.hero]` beside it, that hero is the town's visiting hero, on its entrance.
  `[[encounter.also]]` (E08 T10a): more heroes of the
  encounter, each with its own `at`, named as their hero type unless `name` is given (scouts that
  explore a route, inspections of many heroes). Encounters are laid out in a grid; a hero type
  may appear once per map, a player may have eight heroes, two objects never share a tile.

**One encounter's map** (E04c T3): `--only <id>` instead of `--template` writes a map with that
encounter alone, in its place (the others are still checked; grounds, patches and piles stay): the
capture harness steps the only hero into its opponent. `experiments/agent/opening.py <out>
<maps…>` opens each map's battle under Wine and prints, as one JSON line per map, what the battle
manager and each stack hold at the opening (the special field +0x53C0, the terrain, the anti-magic
and ship flags; per stack creature, count, speed, native flag, morale +0x4E8 and luck +0x4EC, set
by 0x469750 at the start). `experiments/agent/crosscheck_r1.py` replays batch 1a's R1 (six attacks,
damage from the Golems' hit points) as the harness's cross-check.

Running a spec in our engine (to print predictions automatically) is not built (BACKLOG; Q37:
revisit with the next batches, E04b); the predictions are computed by hand or with a script and
written into the spec.

**Traps of a batch** (from the battle lane's HANDOFF, 2026-10-19): a predicted count needs a
throwaway test in `crates/h3-cli/tests` on the base pack (every stack defends). Wandering monsters
act on their own: a Blue hero holds still. Check each move against the creature's speed (the
armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra
turn spoils a count. Every hero has a Catapult. A rerun is no new sample (the draws repeat); a
random starting bonus may arm a hero: play again. Count rounds by the acted bits
(`catapult.py`). A worktree runs the harness through a symlinked `original-data`. The idle
fidgets draw from the rules' generator (`docs/epics/E17-design.md` §7.2), so how long a player
waits changes later rolls: a batch that compares random outcomes keeps its timing fixed (the
capture harness's frozen clock does; the client lane, E17 M2 T3).

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
| 3a (`E04c-batch-3a`) | the special battlefields (E04c T3): morale, luck and the native bonus on Cursed Ground against plain grass with an army of morale −3 (D-1, S-28; BAT-TER-003, 006), the SoD pictures of Cursed Ground and Magic Plains (D-4), Clover Field with Stronghold, Fortress, Conflux and town-less creatures (D-2), Holy Ground, Magic Plains and Lucid Pools as controls | **evaluated** (2026-10-13), **run by an agent** on the capture harness (the first battle batch so; batch 1a's R1 replayed first as the cross-check): every value as the code reading: Cursed Ground morale and luck 0 for every stack, −3 included, and no native bonus; the SoD pictures make the fields 2 and 1; Clover Field +2 for towns 6–8 only (VCMI refuted); S-28 not a quirk (BAT-Q39). `docs/epics/E04c-log.md`, T3 |
| 3b (`E04c-batch-3b`) | which tile decides the battlefield, the attacker's or the defender's, for a Clover Field and for the shore (the adventure lane's derivation, survey §4) | **evaluated** (2026-10-13), run by an agent: the defender's tile for both; the first run found no shore at all, as our writer did not set the coast flag `0x40` that every game map has (now written, `docs/formats/h3m.md`). `docs/epics/E04c-log.md`, T3 |
| 3c (`E04c-batch-3c`) | a siege's walls, gate and moat (E04c T5e): the moat's stop, damage on entering, landing and defending in place and its −3 defense below 0 at an Inferno with a Citadel (D-10; BAT-SIE-004 to 006), the gate's opening, closing and passage and the shots through the closed and the open gate at a Castle with a Fort (D-8; BAT-SIE-003, 008), the Tower's mines (D-9; BAT-SIE-007) | **evaluated** (2026-10-14), **run by an agent** in hot seat with both armies commanded (`experiments/agent/siege.py`): every value as the code reading and the engine: the moat 90 each time, the strike in the moat 127 (VCMI 125), the walk around a moat hex, the gate open with a stack on it and closed after the walk out, refused with a stack on 94; the shot through the open gate half (46), through the closed one a quarter (17); ten hidden mines of 150, a walker over three of them losing 450 and walking on. `docs/epics/E04c-log.md`, T5 |
| 3d (`E04c-batch-3d`) | the Catapult, Ballistics and Earthquake (E04c T6d): the automatic Catapult's aim and a destroyed wall's and gate's passage at a Castle with a Citadel (D-11, D-13; BAT-SIE-010 to 012, 014), the misses and re-aims under Advanced Ballistics at a Fort (D-12; BAT-SIE-011, 014), Earthquake by both sides at a Citadel (D-14, D-15, S-08; BAT-SIE-015) | **evaluated** (2026-10-14), **run by an agent** (`experiments/agent/catapult.py`): every value as the code reading: the walls first, then the gate, then the keep; misses on the nearest other wall; 78 opened, 62 not; the destroyed gate passable without the moat; the defender's Earthquake drew the absent towers and their destruction marked its first stack dead (the engine harms none, BAT-QUIRK-22). `docs/epics/E04c-log.md`, T6 |
| 3e (`E04c-batch-3e`) | the arrow towers (E04c T7c): the turrets' counts and order and the keep's and towers' damage at Castle towns (BAT-SIE-016, BAT-SIE-018, D-18), under the target's Air Shield and Armorer (S-06, D-16) and the defender's Archery (S-07, D-17), a keep commanded with Artillery, the automatic keep's target between ten Marksmen and one (D-19) | **evaluated** (2026-10-14), **run by an agent** (`experiments/agent/turrets.py`): every value as the code: N = 5, the keep, the upper and the lower tower first every round, then the Catapult; the shots within 2N..4N, lowered by Air Shield (9, 7, 7, 7, 8) and Armorer (mean 12.3), unchanged by Archery: S-06 and S-07 refuted in the user's version; the commanded keep one shot a turn, a Wait accepted. **D-19: the keep shot the single Marksman first**, as the wiki's computer rule, not the engine's (BAT-Q45). `docs/epics/E04c-log.md`, T7 |
| 3f (`E04c-batch-3f`) | the automatic turret's target (E04c T7d): the Fight Value against the AI value, the turns to attack, desperate mode, a war machine's share (BAT-SIE-020 to 023) | **evaluated** (2026-10-14), **run by an agent** (`turrets.py`): every target as the code. `docs/epics/E04c-log.md`, T7d |
| 3g (`E04c-batch-3g`) | the siege's other rules (E04c T8): the battle window's Surrender and Retreat buttons at a Castle, a Rampart and a Stronghold with the Escape Tunnel (BAT-SIE-024), the defenders' morale and luck (BAT-SIE-025), Teleport at Basic, Advanced and Expert across the moat and the walls (BAT-SIE-026, D-20), Force Field beside the walls (BAT-SIE-027) | **evaluated** (2026-10-15), **run by an agent** (`experiments/agent/town.py`): every value as the code; the Brotherhood of the Sword replaces the Tavern's morale (+2 in all, D-21: the wiki refuted, VCMI agrees). `docs/epics/E04c-log.md`, T8 |
| 3h (`E04c-batch-3h`) | the automatic Ballista's target (E04c T7e): a tie to the later stack, the turns against "can attack next turn", desperate mode, the value of a small stack, an enemy Ballista under Expert Artillery and without (BAT-WAR-008, BAT-SIE-021) | **evaluated** (2026-10-15), **run by an agent** (`experiments/agent/ballista.py`): every shot as the code reading once the estimate starts from the Ballista's record (2..3, not its hero's multiplied range: the first port's predictions for TU and DN refuted, the port fixed); VCMI's rule refuted (TI, DE, DN, AR). `docs/epics/E04c-log.md`, T7e |
| E08 1a (`E08-batch-1a`) | the adventure map's first batch (E08 T10a): when a day's movement points are set, two hot-seat battles that take a slow stack out of an army in the other player's turn, on day 1 and day 2, and a dismissed slow stack as their control (ADV-Q4, ADV-MOV-002); which tile prices a step off a road, on grass onto swamp and on swamp onto grass (ADV-Q3's D-2, ADV-MOV-003); the land table as a control (ADV-MOV-001). Paths are counted by their marks: scouts explore each walker's row beyond the farthest predicted step. Both specs were reviewed by a second agent before staging | **evaluated** (2026-10-12): as the code reading everywhere: points set at the owner's turn start on day 1 and day 2 (2000, not 1560; ADV-Q4 answered), off a road the tile left prices the step both ways (9 and 14 steps), the land table (15). The map played was a stale build without Q4c (the review's control); the user's statement stands in for it (no change of the day's points when a stack is dismissed or gained). `docs/epics/E08-log.md`, T10c |
| E08 1b (`E08-batch-1b`) | the starting army of sixteen heroes placed with everything their hero type's own, over up to eight loads: the independence of the second and third slots' rolls and the war machine's chance (ADV-Q6, GAM-SET-001) | **evaluated** (2026-10-12): one load sufficed: a hero with its third stack but not its second (VCMI's single roll refuted) and a war-machine hero without its war machine (the wiki's 100 % refuted), as the code reading; ADV-Q6 answered; `docs/epics/E08-log.md`, T10c |
| E08 2 (`E08-batch-2`) | the adventure map's second batch (E08 T16b), the first with the stamp: the points left after a step (D-3, ADV-Q3; ADV-MOV-006): sand onto a cobblestone road with 200 points (the survey's example: 50 left and one more road step by the reading and the wiki, 0 and no step by VCMI), onto a dirt road as the control (no step for any source; 0 left by the reading and VCMI, 50 by the wiki), 100 and 50 points left off a road (100 by the reading and the wiki, 0 by VCMI; 0, 0 and 50); the amounts of random resource piles (D-5, ADV-Q10; ADV-PIK-001): 16 wood and 16 mercury piles picked up one by one (a wood pile of 5 or a mercury pile of 6 refutes VCMI's ranges; neither in 32 refutes the reading, 0.054 % under it). Hot seat, one day: Red walks, Blue collects | **evaluated** (2026-10-13): as the code reading everywhere: D3a 50 points kept on a cobblestone road and a further step (VCMI: 0), D3b 0 on a dirt road and no step (the wiki: 50), D3c 100 kept on snow (VCMI: 0), D3d 50 dropped to 0 (the wiki: 50); D-5's piles: wood 5–9 with three of 5, mercury 3–6 with four of 6 (VCMI's ranges refuted). The stamp matched. `docs/epics/E08-log.md`, T16c |
| E08 3a–3c (`E08-batch-3a`, `-3b`, `-3c`) | increment 3's monsters and battles (E08 T22a2a, T22a2b): the monster's answer at r = 0.5 exactly, with its own type for sympathy and on Easy (D-8 to D-10; ADV-MON-007), the split's upgraded stack and its survivors after a retreat (D-11, A-6; ADV-MON-009, -010), the default experience of the heroes off the map (D-12), a custom experience of 5 000 at load, a surrender's gold between two players (D-13) and the defender's tight formation from the map | **evaluated** (2026-10-20), **run by an agent** (the capture harness, E08 T22a3; Easy for 3a, Normal for 3b and 3c): as the code reading everywhere: the flee offers on Easy and the battles on Normal (D-9, D-10), the join offer at Diplomacy 3 only for aggression 3 twice (D-8), Halberdiers at both draws of 50 (D-11), 23 Pikemen kept of 30 after a retreat (A-6, which the engine fixes: ADV-QUIRK-7), a 90 among 152 heroes (D-12), 1 200 gold from the surrendering attacker's player to the winner's (D-13); VCMI's way: four level-ups' gains at load and the map's tight formation reset to loose (ADV-QUIRK-8, fixed). The stamps matched. `docs/epics/E08-log.md`, T22b |
| E08 4a–4c (`E08-batch-4a`, `-4b`, `-4c`) | increment 4's hero progression (E08 T32a, decision 67; survey `E08-progression.md` §7): the levels at load chosen by the computer's evaluation or always left (D-20), level 10's primary row (D-15), the Elementalist's Wisdom at level 3 (D-16, A-10), three dialogs after one battle (§2.1), a known school's weight (D-17), each predicted from the hero's seed byte by `crates/h3-map/examples/e4_score.rs`; the level limit at load and at a gain (D-21, D-19; 4b, a map of its own); decision 45's rows (4c: the topmost of two grounds, a full army's raised Skeletons, a destroyed Ballista's artifact, an attack on a hero on a Cartographer) | **evaluated** (2026-10-21), **run by an agent** (the capture harness, E08 T32b2; 4a four times, 4b and 4c once as committed; Easy; scored in T33): as the code reading in every row but one: 65 of 66 heroes fit `e4_score`'s reading whole (the 66th in its skills, a starting bonus's Centaur's Axe in its attack); conclusive against the other source, none for it: always-left 16 of 20 (D-20), VCMI's row by the old level 7 of 12 (D-15), intervals 3 and 3 12 of 12 (D-16, A-10), the levels drawn up front 8 of 8 (§2.1), VCMI's school weight 1 of 12 (D-17, settled by the one: the user, ADV-Q18); the limit at load and at the cap (D-21, D-19); the topmost ground, 6 Skeleton Warriors for 9 Skeletons (VCMI's 9 refuted), the destroyed Ballista's slot empty; **E4-10 against the engine**: no path and no battle from above onto a hero on a Cartographer's entrance, the original's test reading the object under the hero (0x4FCBD0): the engine follows it (ADV-OBJ-004, the user, ADV-Q18); `docs/epics/E08-log.md`, T32b2 and T33 |
| E08 5a–5e (`E08-batch-5a` to `-5e`) | increment 5's movement and scouting (E08 T36a, T36b, decisions 78, 80, 81; survey `E08-hero-movement.md` §9): 5a the day's points by Logistics, Boots of Speed, Equestrian's Gloves and a creature specialty (E5-1, a check), flying's price on an inland and a coastal cobblestone road (D-24, E5-4), water walking's price across a lake (E5-5, a check), a Fortress army on swamp under the RoE Cursed Ground (D-23, E5-6a) and the SoD one, class 223 (E5-6b, the agent's call: whether the loader gives it class 21); 5b where a move may end, a tree beyond a tree line and a water tile (D-25, E5-7), a flight through a strip of unexplored tiles with a walker as the control (A-13, E5-9), a flight over a Border Gate without its key (E5-11); 5c and 5d a Logistics specialist's day's points in single precision or exact (D-22, E5-2: Gunnar, Kyrre and Dessa levelled at load to 2, 3, 6 and 12, scored from the seed byte and the recorded skills by `e4_score`); 5e, on a map banning every skill but Logistics and Scouting, a level-up's Logistics and Boots of Speed put on in the day (E5-3, a check) and a level-up's Scouting and the Spyglass put on in the day, with probe tiles at distance 6 (D-26, E5-10) | **specified** (2026-10-22, E08 T36a, T36b): every map imports and validates in the engine, E5-9's strip and E5-10's probes unexplored there; the recipes and the run wait for the harness's explored-tile read and a hero-screen equip from the backpack (T37) |

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
whose natives died still sees Quicksand cast later); a two-hex walker stopped by its rear hex; whether
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
(the reading and the engine since 2026-10-13) or 80 (VCMI), its Magic Arrow 70 either way (BAT-Q28
row 2); batch 2e's OI set-up again with F pressed on the dragons' turn: cast (the reading and the
engine since 2026-10-13) or refused (the batch tried only the cursor; BAT-Q32, BAT-Q21 point 1). *One row each:* Chain Lightning's tie: an enemy stack A as the first target, one
stack of each side of one creature type (Q43's offset cancels) two hexes from A in A's row: the
attacker's stack is struck second (the reading) or the defender's (row 1); defense 12, Defend,
Expert Stone Skin, right-click: 20 (the reading) or 21, and a defending stack of defense 0 struck
by Rust Dragons: Acid Breath lands (the reading) or not (row 3); an Archangel with 5 hit points
left attacks 20 Efreet Sultans and dies of the reflection, then a second stack attacks the Sultans
in that round: not struck back (the reading) or struck back (row 4, S-23); Cavaliers with their
front hex on a seen Quicksand pit ordered three hexes along the row: one hex (the reading) or three
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

**BAT-Q28's record** (moved from OPEN-QUESTIONS on 2026-10-19; the open rows stay there). Row 2, built after the wiki sided with the reading (the user, 2026-10-13: a stack's area spell or chain takes its controller's hero's bonuses, BAT-MAG-048); the review's row was:

| # | The reading (deciding address) | VCMI; the wiki | The engine then | The reading in play; the experiment |
|---|---|---|---|---|
| 2 (r1 F-12, r5a F-3) | A creature's area spell or Chain Lightning takes the damage bonuses of the acting side's hero (Sorcery, an Orb, a specialty); its bolts take none (0x5A4970, 0x5A6360) | no bonus for any creature's cast; not checked | none (BAT-MAG-048, 049) | 46 of 100 Faerie Dragon draws under such a hero; one dragon's Fireball under the Orb of Tempestuous Fire: 120 (reading) or 80 |

**On record, no decision now** (row 13: the engine follows the reading, VCMI differs: reading / VCMI; wiki checks and statistics rows can settle them). After a strike (BAT-MAG-040 to 047; r4b F-5): the Black Knights' Curse at 25 / 20 %, the Mummies' at 25 / 50 %; the Rust Dragon's extra damage at 20 / 30 %; a stack blinded by Unicorns retaliates at half / never; the Dragon Flies' Weakness −6 / −3; the Thunderbirds' bolt is Lightning Bolt / a level-0 spell; Stone Gaze can be resisted / cannot; Poison: a tenth per round, kept when it ends / per turn to −30 %, given back; Age halves 25 to 13 / 12; Bind ends when the binder moves or dies / at the bound stack's turn. Chain Lightning (006; r1 F-11): a candidate rolls resistance at every jump / once per cast. Frenzy (019; r2b F-4) lasts by the stack's turns / to its first strike. The Ogre Mage's Bloodlust (048; r5a F-5): 6 / 3 rounds; the aimed stack of a creature's cast never rolls resistance / rolls. The Master Genie (050): 6 / 5 rounds, with Frenzy / without. The Phoenix (059; r5b F-6): a failed rebirth uses it up / does not. Fear (060): Gargoyles can be frightened / cannot; no roll in the wait phase / one. Familiars the cast killed (055) channel nothing / channel. The Orb of Vulnerability (063, 064, 066; r6 F-3) leaves the Pendants', Armageddon's Blade's and the Dragon Father's immunities / lifts them (not "agreed by all", as BAT-Q21 has it). Hypnotize and Berserk (025, 026, BAT-RNG-001, BAT-ATK-003; the relations the review's fix took from the reading): a hypnotized shooter is blocked by its own side's stacks / by the controlling side's; a multi-headed strike hits a stack its own side hypnotized / spares it; hypnotized Unicorns protect the side that controls them / their own; two stacks are hostile whenever either is berserk / VCMI has that only for a berserk shooter's blocking.

**The wiki checks (2026-10-13;** quotes and pages: `docs/experiments/README.md`, "The wiki checks"**).** Row 2: **the wiki sides with the reading** (Sorcery: "Sorcery also boosts damage spells cast by creatures under the hero's control"; Faerie Dragon: Magic Arrow, Ice Bolt and Lightning Bolt "are not affected by damage increases from the hero's Sorcery secondary skill or elemental orbs equipped"), so the reading and the wiki stand against VCMI and the engine. Rows 4 and 10: **the Horn of the Abyss changelog fixed both as bugs** (1.7.1: "when attacking stack died from Fire Shield, retaliation strike into nowhere still happened"; 1.5.4: a two-hex corpse whose hind hex a one-hex corpse covers could not be targeted by Resurrection and Raise Dead "even when pointing at its front hex"), so the original has them: survey S-23 and S-24 have a second source. Row 11: (b) the Orb of Vulnerability page lists the Unicorns' aura among what the Orb negates, as the engine; (a) on casting the page fits the engine ("the opponent can cast spells … but only on the troops of the owner of the Orb"), the chain is not mentioned. Row 13: the wiki agrees with the engine (the reading) on the Mummies' 25 %, the Rust Dragon's 20 %, the Unicorns' Basic Blind (half), the Dragon Flies' Advanced Weakness (−6), Poison per round and kept, Age rounded up (13), Bind ended by the binder's move or death, Frenzy until the stack's turn in the next round, the Ogre Magi's and Master Genies' power 6 (6 rounds) with Frenzy in the Genies' list, no Gargoyles among the Fear-immune, the Orb leaving artifact immunities and a hypnotized shooter blocked by its previous owner's stacks; **it contradicts it twice:** the Black and Dread Knights' Curse at 20 % (as VCMI; the re-read of 0x440702 confirms one draw at 25 % for creatures 66, 67 and 141) and the Curse after a strike "Advanced" for the knights and the Mummies (the reading: every strike spell at mastery 0, power 3, 0x468990; Curse's value at Advanced is 1, so the cursed stack would deal its minimum − 1 instead of its minimum). Rows 1, 3, 5 to 9, 12 and the rest of row 13: the wiki is silent.

**From E04c** (moved there at E04c T10a, the closure's triage; none staged). From T5a (BAT-Q41,
survey S-29): a Stronghold with a Citadel on an AB-format map in the Complete game (a moat or
natural obstacles: the game's version; the writer writes SoD maps only). From T7e
(`installation.md` §7.33): a computer player's automatic Ballista on Easy (0x435CB0 divides
nothing there: the engine's default setup; hot seat makes both sides human), the / 10 of a
candidate that loses its turns, a hypnotized candidate. From T8 (§7.32): a hypnotized stack
between armies of different morale, its morale read before and after the next action (the
original's controller, the engine's owner). From T9 (survey D-22): a defending hero's Ballista at
a town without a fort (the code sets it up, VCMI does not). From T10a (survey D-23): a tie
between a First Aid Tent and an Ammo Cart in the computer's aim (the later record: the Ammo Cart
in the original's order). From T10b (the review, BAT-Q50): whether the original offers the
commanded Catapult a Wait (the engine refuses it, as Defend, which batch 3d B saw do nothing);
which side's stack acts after the Catapult at equal top speeds in rounds 1 and 2 (the engine, by
the pick 0x4650BB: the attacker's; `installation.md` §7.29); the defender's Fire Elemental summoned
with columns 12–15 of the inner yard filled, so that its place is the gate's hex (BAT-SIE-028,
review F-r6-3): whether the gate opens (+0x53A4 1 after the cast: the code reading, the engine) or
stays closed (3: VCMI).

**The wiki checks** (the review's 49 wanted checks, done 2026-10-13 with the user's approval:
raw pages of heroes.thelazy.net through WebFetch, verbatim quotes; SoD arguments of `swh`). Each
outcome names the rule or question; pages without a link are `https://heroes.thelazy.net/index.php/<Page>`.
- *The wiki agrees with the engine* (a wiki source for the rule; no change): Dispel (Basic own,
  Advanced any, Expert all units; "Spells/Abilities unaffected by Dispel: Disrupting Ray, Acid
  Breath"; Hypnotize lists Dispel among the spells that end it: BAT-MAG-016, 026); Disrupting Ray
  ("the target was killed and resurrected the disrupting ray effect still remains", BAT-MAG-010);
  Slayer unskilled ("only ... when fighting Dragons, Hydras and Behemoths", BAT-MAG-014); Frenzy
  ("Until target troop's action turn in the next round", BAT-MAG-019; VCMI: to its first strike);
  Anti-Magic ("removes negative spell effects (e.g. Curse) but not positive", BAT-MAG-021; its own
  side blocked only in user commentary); Adela ("3% for every N hero levels, where N is the level of
  the target creature": the levels divided first, BAT-MAG-020); Hypnotize ("a ranged hypnotised
  unit can be blocked by its previous owner's troops", "retaliates as normal", BAT-MAG-026);
  Sacrifice (Efreet, Efreet Sultans and Phoenixes among the creatures "immune to Sacrifice (or
  resurrection via Sacrifice)"; Gold Dragons sacrificed, not raised; the victim "non-summoned,
  non-hypnotized and non-cloned", BAT-MAG-036); the strike's spells: Mummies 25 %, Rust Dragon 20 %,
  the Unicorns' Basic Blind (retaliation at 50 %), the Dragon Flies' Advanced Weakness, Paralyze's
  retaliation at "a quarter", a petrified stack halved "both melee and ranged" and striking back at
  full strength, Poison 10 % "for every combat round", at most 50 %, "the maximum health remains
  decreased", ended by Cure only, Age "down to 50% (rounded up)", Bind until the Dendroids "move or
  perish" or Dispel or Teleport, the Thunderbird's bolt raised by neither Sorcery nor the Orb
  (BAT-MAG-040 to 047; BAT-Q28 row 13); BAT-ATK-004's two quotations (both verbatim); the Ogre Magi's
  and Master Genies' power 6 at Advanced, Frenzy in the Genies' list, no Slayer without level 7
  troops (BAT-MAG-048, 050); the Enchanters' eight spells at Expert every three rounds for three
  rounds (BAT-MAG-053; the page's Stone Skin "11.9%" is a slip: its column then sums to 94.2 %, the
  engine's weight 15 gives 17.9 %), and "the spell relevance is calculated based on stats of the
  first stack only (fixed in HotA)" (BAT-QUIRK-13); Fear's immune list without Gargoyles, 10 %, "only
  at the beginning of enemy stack's turn" (BAT-MAG-060); the Phoenixes' rebirth "divided by 5" plus
  20 % for each remaining one, once per combat (BAT-MAG-059); the elementals' half damage ("not
  halved against targets affected by Anti-Magic", BAT-DMG-010's first wiki source); Familiars'
  20 % "rounded down", once whatever the stacks; Mages and Pegasi counting "if ... present at the
  beginning of the combat" (BAT-MAG-055, BAT-QUIRK-15); the Orb of Vulnerability ("Does not affect
  ... the immunities given by artifacts"; it negates the Unicorns' aura: BAT-MAG-063, 064, 066;
  row 11); the Paralyze and Petrifying attack pages ("negated by the Recanter's Cloak and Cursed
  Ground, but not by the Orb of Inhibition", BAT-MAG-062); the Badge of Courage ("Immunity to mind
  spells (removed in Horn of the Abyss)", BAT-MAG-065); Fire Shield ("only the stronger Fire Shield
  takes effect", BAT-MAG-058); the Ammo Cart ("until the Ammo Cart is destroyed, after which their
  shots will deplete as normal", BAT-WAR-004, against the review's r3 reading); the HotA changelog's
  1.7.1 "if you summon 20 stacks and then destroy some of them, you still can't summon new stacks"
  (dead ones counted, BAT-MAG-037) and 1.7.0 "AI-controlled heroes' spells were not reflected by
  the Magic Mirror" (BAT-MAG-022's note).
- *The wiki sides with the reading against the engine* (decisions for the user, OPEN-QUESTIONS):
  the Faerie Dragons' area spells and Chain Lightning take the hero's Sorcery and orbs, their bolts
  do not (Sorcery, Faerie Dragon; BAT-Q28 row 2); the Orb of Inhibition and the anti-magic garrison
  do not stop them ("must press F or G", BAT-Q32); the retaliation at a striker the Fire Shield
  killed and the raise of the corpse on a two-hex corpse's rear hex, fixed as bugs in HotA 1.7.1 and
  1.5.4 (rows 4 and 10, S-23 and S-24); the natives' sight as the original times it (Land Mine
  "started the battle with"; Quicksand "has a creature fighting on native terrain": BAT-Q34).
- *The wiki contradicts the engine and the reading:* the Black and Dread Knights' Curse at 20 %
  (as VCMI; the code draws 25 % for knights and Mummies alike, re-read at 0x440702); the strike's
  Curse "Advanced" (the code: mastery 0, power 3; a cursed stack's minimum − 1 or its minimum).
- *Silent* (the experiments stay): Chain Lightning's ties beyond "randomly chosen" and its roll per
  jump (row 1); a worked Sorcery number (BAT-MAG-004); the Unicorns' aura for a hypnotized stack
  (BAT-MAG-008); a mass or battle-wide spell that affects nobody (BAT-MAG-003; HotA 1.8.0 made
  Berserk refuse an area without a creature); the Faerie Dragons' area spells at an empty hex in
  SoD (the page has it under HotA, BAT-Q32); Magic Mirror and war machines (BAT-MAG-022); a morale
  or luck base beyond ±3 under the four spells (BAT-MAG-011); computer and neutral sides' morale
  (BAT-Q33); Acid Breath after death and a raise (BAT-MAG-046); the Defend bonus under later
  spells (row 3); the Behemoths' rounding (BAT-DMG-006); a retaliation ending Frenzy; the Fire
  Shield page itself (row 4); who may attack an own berserk stack (BAT-MAG-025); a two-hex stack
  in a pit and an attack from a pit (row 5); a Harpy returning onto a Fire Wall (BAT-ATK-004); a
  timed obstacle removed early (S-26); a raised stack's turn (row 6); the surrender price of raised
  creatures (row 7: "half the recruitment cost of the remaining creatures"); expired clones and the
  20 stacks (BAT-MAG-037); the Thunderbird's bolt against immunities; Stone Gaze against
  resistance; a failed rebirth (BAT-MAG-059); the wait phase's order and a frozen stack in the
  speed tie (rows 8, 9: "The units naturally cannot wait again" is about the second phase); mana
  above the maximum (BAT-MAG-055); the Dragon Father against creature abilities (BAT-MAG-064);
  Eagle Eye's 43 % and 64 % (classic 30, 40, 50 %; BAT-MAG-069); the Enchanters' first round; the
  First Aid Tent's patient ("the top creature of an allied stack", chosen by the computer).

**From BAT-Q36** (2026-10-13; `installation.md` §7.25): a two-hex shooter's shot at a target behind
it, 10 hexes from its rear hex and 11 from its front hex: full damage (the reading, the engine since
BAT-Q36) or half (VCMI); e.g. the attacker's Medusas on (12, 5) shooting a stack on (2, 5), a
control on (1, 5). **From the wiki checks:** Swordsmen (damage 6 to 9) cursed by Black Knights strike for 6
per creature before the attack and defense factor (the reading: Curse at mastery 0) or 5 (the
wiki's Advanced Curse, value 1, BAT-MAG-011); the statistics row on the knights' 25 % above carries
the wiki's 20 % too.

The smoke map is generated by `cargo run -p h3-map --example smoke_map --
original-data/extracted/H3bitmap/Objects.txt experiments/out/E04a-smoke.h3m`.
