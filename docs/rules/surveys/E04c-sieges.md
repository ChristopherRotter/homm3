# E04c — sieges: survey (T4)

The survey behind the siege seam (E04c T4, 2026-10-13): what the original's battle takes from a
town and gives back, the rule areas of the siege with their sources compared, the disputes, and
the cut of the siege increments. The detail readings of each area come with its task (T5–T8).

Sources: `code` (the executable, `docs/formats/installation.md` §7.27, T5's §7.28, earlier §7.15
to §7.26); `original-data` (`CRTRAITS.TXT`, `BALLIST.TXT`, `WALLS.TXT`); `vcmi` (VCMI 1.7.5:
`lib/battle/BattleInfo.cpp`, `CBattleInfoEssentials.cpp`, `CBattleInfoCallback.cpp`,
`server/battles/BattleProcessor.cpp`, `config/spells/moats.json`, `DamageCalculator.cpp`, `lib/mapObjects/CGTownInstance.cpp`,
`config/buildingsLibrary.json`, `config/factions/*.json`); `wiki` (heroes.thelazy.net, read
2026-10-13 with the user's approval: `Combat` (its Siege section), `Moat`, `Arrow tower`,
`Ballistics`).

## 1. A battle at a town in the original

**Every battle at a town is "at a town"**, with or without a fort (`code`: the town is stored
first, the fort level beside it). The fort level, the highest of Fort, Citadel and Castle built,
adds the siege's parts:

| Part | Fort | Citadel | Castle | Source |
|---|---|---|---|---|
| Walls (11 wall hexes blocked, 12 to 182), the gate | yes | yes | yes | `code` (hexes), `vcmi`, `wiki` |
| Walls' hit points | 2 | 2 | 3 | `vcmi`; `code`: 18 parts from a per-town table, four +1 with a Castle (T5) |
| The keep (an Arrow Tower stack at 254) | — | yes (2 hit points, `vcmi`) | yes | `code`, `vcmi` |
| Upper and lower towers (255, 251) | — | — | yes (2 hit points each, `vcmi`) | `code`, `vcmi` |
| The moat | — | yes | yes | `code`, `vcmi`, `wiki` |
| The attacker's Catapult; no defender's Ballista | yes | yes | yes | `code`, `vcmi` (no Ballista "in a town") |

Without a fort the battle is fought on the town's tile like any other, but still at the town: the
town's native terrain, the defending hero's building bonuses, the defenders' morale from the
Tavern, no surrender or retreat for the defender, +500 experience for the attacker (`code`). The
special buildings (the Escape Tunnel, Brimstone Stormclouds, Blood Obelisk, Glyphs of Fear, the
Brotherhood of the Sword) are tested in a second building set of the town (`code`, +0x158), the
Tavern, the fort and the Grail in its built set: which buildings the second set holds is the town
context's to read (E09).

**Moat.** Not in a Tower (land mines instead), not in a Stronghold in a game under RoE or AB
rules (`code`: the game's version, H3API's `mapKind`, BAT-Q41; `wiki`: "Stronghold does not have a
moat in RoE"). The Fortress's is two
hexes wide. Damage on entering: 70 (Castle, Rampart, Necropolis, Stronghold, Conflux) or 90
(Inferno, Dungeon, Fortress), straight to the hit points, and **only under SoD rules** (`code`: the
game's version; `wiki`: "That effect was introduced in SoD"); the Tower's mines 150 (D-9). The moat is a stop
(`code` 0x4B3290); the `wiki` adds −3 defense and damage for each round a stack stays (T5).

**Turrets.** The keep's count is 4 + the town's built buildings among a mask (`code`), each
creature dealing 2–4 (`CRTRAITS.TXT`); a tower has half the keep's count, rounded up. With the
`wiki`'s count N of "structures" (upgrades, Citadel and Castle not counted) and the Village Hall
always built, the keep's count is 5 + N: the `wiki`'s keep damage (10 + 2N to 20 + 4N) and tower
damage (6..12 + (2..4)·⌊N/2⌋) follow. `vcmi`: keep (10 + 2n)..(20 + 4n), tower (6 + n)..(12 + 2n).

**The defending hero's bonuses** for the battle (`code`, restored at its end; mana capped at its
value before): Tower's Grail +150 mana; Inferno's Brimstone Stormclouds +2 power; Dungeon's Grail
+12 power; Stronghold's Grail +20 attack; Fortress's Blood Obelisk +2 attack, Glyphs of Fear +2
defense, its Grail +10 attack and defense. **The defenders' stacks**: Tavern +1 morale, Castle's
Brotherhood of the Sword +2 in its place, not more (`code`, T8's batch 3g; D-21); luck from the town (Rampart's Fountain of Fortune +2:
`vcmi`, the `wiki`, `code` §7.32, batch 3g RL).

## 2. The rule areas compared

| Area | `code` | `original-data` | `vcmi` | `wiki` | Increment |
|---|---|---|---|---|---|
| Fort levels, parts | as §1 | — | as §1 (`fortifications` per building) | Fort needed; Citadel, Castle improve | T5 |
| Walls, gate, who passes | 18 parts, hit points (`WALLS.TXT`), 11 wall hexes; the gate closed or open, the defender's passage, opening and closing (§7.28) | `WALLS.TXT` | wall parts, gate states; the same passage ("bridge" hex 94) | "flying creatures can fly over walls" | T5 |
| Moat | as §1; a stop, damage on entering and on acting in place, −3 defense not floored (§7.28) | — | `moats.json`, `moatAbility`; defense floored at 0 (D-10) | as §1; stop, −3 defense, per-round damage | T5 |
| Tower's land mines | ten hidden mines, at least 150 or the hero's Land Mine (§7.28, D-9) | — | `towerMoat`: 150 | "minimum 150 points of damage" | T5 |
| Shooting through walls | halved, a grid line test, the open gate free (§7.28, D-8) | — | halved, the shortest hex path | towers bypass the penalties | T5 |
| Catapult and Ballistics | the Catapult on hex 119; its turn, automatic aim, shot and the parts' damage (§7.29, D-11 to D-13) | `BALLIST.TXT`: hit chances keep/tower/gate/wall none 5/10/25/50 %, Basic and Advanced 7/15/30/60 %, Expert 10/20/40/75 %; shots 1, 1, 2, 2; damage 0/1/2 by 10/60/30 % (none), 0/50/50 % (Basic, Advanced), always 2 (Expert) | from the same table | Basic: control; Advanced: two shots; Expert: "always hits the intended target" | T6 |
| Earthquake | 2, 2, 3, 4 hits of 1 by mastery; the draw (§7.29, D-14, D-15; S-08) | `SPTRAITS.TXT` | `other.json`: uniform among the standing parts, the attacker only | "one point of damage to two (three, four) random walls/arrow towers"; S-08 | T6 |
| Arrow towers | as §1; first in the round, the keep before the towers; Artillery controls; the shot: per creature `random(2, 4)`, summed, times the target's protection factor (Air Shield, petrification, Armorer), nothing else; immune to every spell, off the field (§7.30, D-16 to D-19) | stats in `CRTRAITS.TXT` (2–4, 100 hit points) | one uniform draw of the range through the full formula (attack 0, defense ignored); first in the siege phase; Artillery controls | as §1; no luck, defense ignored, no modifiers but the bugs S-06 and S-07; Artillery controls; the computer's target by classes A–C | T7 (S-06, S-07) |
| Defender's bonuses | as §1; the town's morale and luck last in the base routines, before the clamp, to the stacks the defender controls (§7.32) | — | town buildings' bonuses | Morale, Luck: "when defending against siege" | T8 |
| Surrender, retreat | surrender never for the besieged; retreat only with the Escape Tunnel (§7.32) | — | the same | Combat, Escape Tunnel: the same | T8 |
| Experience | +500 for the attacker at a town; −500 after a retreat or a surrender (§7.32; the orchestrator's) | — | — | +500 for a capture, none after an escape or a surrender | T8 |
| Teleport, summoning, spell cost in sieges | Teleport by mastery: the moat, the walls' line (D-20); a created stack placed through the closed gate for the defender, a summon there opens it; no spell obstacle on a wall's hex; the cost's armies as the setup's (§7.32) | — | Teleport: the masteries (D-20) | Teleport: the masteries | T8 |
| Tactics | the phase as in any battle: only a creature bank has none (§7.32) | — | `tacticsAllowed`, off in banks only | Tactics: "especially vital when sieging a town" | T8 |

## 3. What the battle setup must receive, and who derives it

The battle needs, for a battle at a town: that it is at a town (the controls, the experience), the
town's faction (the moat's hexes and damage, the turrets' look, the battlefield picture), the fort
level, whether the moat exists and whether it does damage (the map's format), the turrets'
strength (the count of the town's buildings), whether the defender may retreat (the Escape
Tunnel), and the defenders' morale and luck from the town. Everything else in §1 is the defending
hero's (its primary skills and mana for the battle, derived before the seam like its other
bonuses, ADR-0013 point 6) or the battle's own rules. The town's buildings belong to the town
context (E09, not refined); its owner derives these facts, as the adventure lane derives the
battlefield (ADR-0022). The battle reports nothing about the town: the walls are not kept (`code`:
nothing written back to the town but the hero's restored skills; `vcmi` the same), and a capture
follows the result (the adventure lane).

## 4. Disputes and quirk candidates

- **D-5 A tower's damage for an odd N.** `code` and `wiki`: half the keep's count, rounded up
  (6 + 2⌊N/2⌋ at the least); `vcmi`: 6 + N. Recommendation: the code, a confirmation row (T7).
  **The user (BAT-Q40): the code.**
- **D-6 Expert Ballistics.** `BALLIST.TXT`: Expert hits the wall 75 % of the time; the `wiki`'s
  Ballistics page: "always hits the intended target" (likely HotA's, the page does not say).
  Recommendation: the table, read in T6 where the Catapult's shot reads it. **The user (BAT-Q40):
  the table.**
- **D-7 A battle at a town without a fort.** `code`: at a town all the same (no surrender, no
  retreat without the tunnel, +500 experience, the hero's and the stacks' town bonuses, the town's
  terrain); `vcmi`: also "defended town" for surrender and retreat; the `wiki`: "a usual
  battlefield". Recommendation: the code; the seam keeps the town without a fort.
- **S-29 The game version's branch** (candidate). A game under RoE or AB rules: the moat does no
  damage, the Stronghold has none and its siege with a Citadel or Castle gets the usual natural
  obstacles instead (`code`; the `wiki` on the versions). Under SoD rules no siege has natural
  obstacles (`code`, `vcmi`). Not a bug: the original keeps the old rules for its old versions.
  Recommendation: the derivation honours it (the seam's moat says "absent", "harmless" or
  "damaging"; the obstacles are drawn before the seam as always); no flag. **The user (BAT-Q40):
  honoured; not a quirk. (BAT-Q41)** The deciding value is the game's version (the original's
  `mapKind`), which the original probably takes from the map's format (its new-game setter is not
  read): how a game gets its version is the adventure lane's; an experiment candidate, a Stronghold
  with a Citadel on an AB-format map in the Complete game.
- **D-8 The shot through walls: the line** (T5; `code` 0x467510, 0x467840; `vcmi`
  `battleHasWallPenalty`, `battleHasPenaltyOnLine`). Both halve a shot from outside the walls on
  the shooter's row at a target inside on its row, except the Mage, Arch Mage, Enchanter,
  Sharpshooter and Arrow Tower and a hero's Golden Bow or Bow of the Sharpshooter, and both multiply
  the halving with the distance's (a quarter). The `code` asks a straight line on the column and row
  grid (1/17 steps, truncated) for an intact wall hex or the closed gate's hex 96; `vcmi` the hexes
  of the shortest hex path ("to avoid various OH3 bugs"), where the gate's hex blocks while the gate
  stands, open or closed. With intact walls they differ only through the **open gate**: the `code`
  does not halve a shot whose line passes it, `vcmi` does; with destroyed parts (T6) the two
  geometries differ more. Recommendation: the code; a confirmation row. **The user (BAT-Q42): the code.**
  **Confirmed** (batch 3c G, 2026-10-14): a Titan's shot through the gate held open by a stack on 96
  at Trolls on 97 did 46, half (the distance only; VCMI's quarter would be 18 to 27); through the
  closed gate 17, a quarter.
- **D-9 The Tower's mines** (T5; `code` 0x4663BE): under SoD rules at least 150, the defending hero's
  Land Mine damage when larger (its power and mastery, no damage bonuses); under older rules the
  hero's, or without a hero the spell's at power 3 and no mastery. `wiki` ("minimum 150 points of
  damage") agrees, `vcmi` always 150. The code (two sources agree). **Observed** (batch 3c T): ten
  hidden mines of the defender's worth 150 on the moat hexes but 95, no defending hero.
- **D-10 The moat's defense below 0** (T5; `code` 0x442590: −3 after the other reductions, not
  floored; `vcmi`: −3 as a bonus, a stack's defense floored at 0). They differ for a stack with
  defense below 3 in the moat (the Peasants have 1): the code gives its attacker the larger
  difference. Recommendation: the code; a confirmation row. **The user (BAT-Q42): the code.**
  **Confirmed** (batch 3c M): an Archangel striking Sprites (defense 2) in the moat did 127, the
  defense -1 (VCMI's floor: 125).
- **Agreements** (T5): the closed gate's passage for the defender needs hexes 95 and 94 free of
  living stacks and corpses in every town (`code` 0x469A10; `vcmi`'s "bridge" hex 94); a defender
  entering 94 to 96 opens it, and it closes when 95 and 96 (94 in the Fortress) are free; the moat's
  hexes (95 only while the gate is closed), its stop, its damage by town and its −3 defense; the
  Tower's ten hidden mines instead of a moat. Small differences, the code followed without a
  question: `vcmi` lets a Force Field on 94 keep the gate from opening and only living stacks keep
  it open (the `code` tests stacks and corpses, no Force Field).
- **D-11 The Mid-Upper Wall's hex** (T6; `code` 0x63BE60: target 2, the Mid-Upper Wall, is aimed
  at hex 62 and its destruction opens wall hex 78, its record's index 4; `vcmi`: `OVER_GATE` at 78,
  62 indestructible). **No dispute:** the session's question (BAT-Q43) read the index as the hex
  and asked about 62 against 78; corrected the same day, the code and VCMI agree on 78. The user's
  answer, the code, stands. **Batch 3d:** 78 opened, 62 stayed a wall.
- **D-12 A missed shot** (T6; `code` 0x445EC0: the nearest other standing wall of 1, 2, 5, 4 by the
  difference of the target indices, a tie by `random(1, 100)` ≤ 50, the aimed target if no other
  wall stands; `vcmi`: a random standing wall, the gate only without walls, the towers and keep
  only without either). Both roll the damage (0, 1, 2) apart from the hit. Recommendation: the
  code. **The user (BAT-Q43): the code.** **Batch 3d:** every miss landed on the nearest other
  standing wall (two of them where VCMI could have picked another).
- **D-13 The automatic Catapult's aim** (T6; `code` 0x473C00: the weakest standing wall, ties at
  random, then the gate, the keep, the upper and the lower tower; `vcmi`: not read, its automatic
  shot names no target and lands like a miss; the `wiki`: only that Ballistics gives control).
  Batch 3c's six automatic shots hit walls only (both readings allow it). Recommendation: the code;
  a confirmation row. **The user (BAT-Q43): the code.** **Batch 3d confirms it:** nine shots, the
  gate only after the four walls fell, the keep after the gate.
- **D-14 Earthquake's draw** (T6; `code` 0x5A7C80: per hit `n` = the targets with hit points above
  their hits drawn, then the n-th target with hit points above 0, so a doomed target can be drawn
  again and the last ones spared; `vcmi`: uniform among the parts standing at the cast, each hit
  1; the `wiki`: "random walls/arrow towers"). Recommendation: the code (not a quirk: the odds
  only). **The user (BAT-Q43): the code.** **Batch 3d:** the absent towers were drawn and a doomed
  part drawn again (three hit points for four hits), as the code.
- **D-15 The defender's Earthquake** (T6; `code` 0x59ED29: any side in a battle with a fort; `vcmi`:
  refused to the defender (its smart target); the `wiki`: silent). Recommendation: the code. **The
  user (BAT-Q43): the code.** **Batch 3d:** the defender's hero cast it.
- **S-08 Earthquake disables the first defending stack** (exploit-fix, `E04-quirks.md`).
  **Confirmed by the code** (T6, §7.29): Earthquake's draw has no fort test, so at a Fort or
  Citadel the absent towers and keep (2 hit points from `WALLS.TXT`) are drawn too, and an absent
  turret's destruction marks dead the stack its turret record names, the defender's first. **The
  user (BAT-Q43):** the fix keeps the absent parts in the draw (they soak hits, so the real walls
  keep the original's odds) and harms no stack (ledger BAT-QUIRK-22). **Observed in batch 3d:**
  the defender's own Earthquake destroyed both absent towers and its first stack was marked dead.
- **D-16 Armorer, Air Shield and petrification against a turret** (T7, S-06; `code` 0x465AD0 →
  0x443D90: the shot's sum times the target's protection factor, the same as any shot's stage 3:
  Air Shield's factor, × 0.5 petrified, × (1 − Armorer), so each **lowers** the damage; `vcmi`:
  they lower it (the full formula); the `wiki`: Armorer ("the damage from arrow towers during siege
  combat is not reduced by armorer secondary skill but actually increased the corresponding
  amount"), Air Shield ("it actually significantly increases their damage"), Arrow tower ("Troops,
  which are buffed by Air Shield spell or are petrified take additional damage as well"), each
  "(fixed in HotA)"). The code has no path that raises it (a division would). A confirmation row
  decides: under Advanced Air Shield the keep's shot is at most `2N` by the code, at least `4N`
  by the wiki (`N` the keep's count). **The user (BAT-Q44): the code**, with the row in batch 3e.
  **Batch 3e confirms the code:** N = 5; under Expert Air Shield 9, 7, 7, 7, 8 (N..2N); under
  Expert Armorer ten shots of 10 to 14, mean 12.3 (× 0.85; none raised).
- **D-17 Archery and a turret** (T7, S-07; `code`: the defending hero's Archery is not read; `vcmi`:
  not read for a turret either; the `wiki`: its Arrow tower page "Archery or the Artillery
  secondary skills do not affect the damage", its Archery page "Bug: The archery skill reduces
  damage done by arrow towers (fixed in HotA)"). Recommendation: the code; a confirmation row
  (Expert Archery: every shot within `2N..4N`). **The user (BAT-Q44): the code**, with the row.
  **Batch 3e confirms it:** ten shots of 10 to 17, mean 15.2, as without Archery.
- **D-18 The turret's damage roll** (T7; `code`: one `random(2, 4)` per creature, summed: the keep's
  `N` creatures give `2N..4N`, bell-shaped; `vcmi`: one uniform draw of `(10 + 2n)..(20 + 4n)`; the
  `wiki`: the same ranges, "Base damage plus d × N"). The ranges agree (`N = 5 + n`); the code's
  sum is the original. Recommendation: the code. **The user (BAT-Q44): the code.** Batch 3e: every
  shot within the ranges (eight samples cannot tell the two rolls apart).
- **D-19 The automatic turret's target** (T7; `code`: the computer player's battle AI, not read, as
  for the automatic Ballista; `vcmi`: the Ballista's choice (`tryMakeAutomaticActionOfRangedUnit`:
  not a war machine, then a stack that can attack next turn, then the highest expected damage
  value), which the engine ported for its Ballista (BAT-WAR-002); the `wiki`: classes A (can attack
  at its turn), B (next turn), C (later), each stack's priority `(AI value / health) × 100 %, 50 %,
  33 %` and `1 / health` by the same weights, "The AI considers the Arrow tower damage equal to 1
  (bug)", ties to "the rightmost slot of the hero's army"). Recommendation: the Ballista's rule now
  (one automatic choice for both machines), observation rows in the batch where the wiki and VCMI
  pick different stacks; the original's AI is read with the battle AI's task (the battle queue's
  inbox). **The user (BAT-Q44): the Ballista's rule**, with observation rows. **Batch 3e (AI): the
  original shot the single Marksman first, as the wiki's rule and not the engine's** (one sample;
  BAT-Q45 asks the user). **T7d's reading (§7.31, BAT-Q46):** the turret has its own routine
  (0x41E190), not the Ballista's: the mean damage `3n` through the target's protection as kills
  times the computer's worth of a creature (a Fight Value estimate), divided by the turns until the
  stack can attack (1 to 5), ties to the later record, and a desperate mode; batch 3e's choice
  follows (a tie, the later record). The wiki's classes and tie agree, its "damage 1" and AI value
  do not; VCMI has no turret routine.
- **D-20 Teleport in a siege** (T8; `code` §7.32: below Advanced not across the moat, the
  moat's side by its row's moat hex, so the moat hex itself counts as inside; below Expert only
  where the shot's line test meets no standing wall and not the closed gate, outside the walls
  too; `vcmi`: the same masteries, but tested only while a wall stands and for two hexes on
  different sides of the walls, along the shortest hex path; the `wiki`: the masteries, no
  detail). The masteries agree; the code's tests are the original's. Recommendation: the code,
  with confirmation rows (the user's standing answer for E04c, BAT-Q42 to BAT-Q44: the code with
  a batch). **Batch 3g: as the code** (Basic refused the moat hex 61, Advanced accepted it and
  refused the inside hex 64, Expert accepted 64).
- **D-21 The Tavern and the Brotherhood of the Sword** (T8; `code` §7.32: the Tavern's +1 is read in
  the town's set +0x150, the Brotherhood's +2 in +0x158; batch 3g CM read both sets back: in
  +0x150 the Brotherhood replaces the Tavern, its predecessor, so a Castle with it gives +2 in
  all; `vcmi`: the Brotherhood upgrades the Tavern and replaces its bonus, +2; the `wiki` (Morale):
  "+2 (in addition to Tavern)"). The code, VCMI and the batch agree: +2. A fact for whoever derives
  `Siege.morale` (the town context; the adventure lane's inbox); the battle adds what it is given.
- **D-22 The defender's Ballista at a town** (T9, BAT-SIE-029; `code` §7.27: 0x463600 sets up no
  Ballista for the defender in a battle with a fort level above 0; `vcmi`: `BattleInfo::setupBattle`
  gives a defending hero none at any town, "(bug #551)"). They agree with a fort, which the faithful
  profile now follows (the setup may list the hero's Ballista; the battle leaves it out): a
  fix-optional quirk, BAT-QUIRK-23 (the user, BAT-Q48), fixed the Ballista fights at every town;
  without a fort the engine keeps the code's Ballista. Unobserved: a batch row (a defending hero
  with a Ballista in a town without a fort) would settle it.
- **D-23 The war machines' record order** (T10a, BAT-WAR-001; `code` §7.27: 0x463600 gives the
  records after the army to the Catapult, the Ballista, the First Aid Tent, the Ammo Cart; `vcmi`:
  `BattleInfo::setupBattle` numbers the war machines before the army, Ballista, Ammo Cart, Tent,
  then the Catapult; the `wiki`: silent). Only tie-breaks and draws over records see it: the
  computer's ties go to the later record (the turrets', the automatic Ballista's), a Magic Mirror's
  or a Faerie Dragon's draw among stacks. The engine numbered them in VCMI's order after the army;
  T10a follows the code (stack ids: the army, then the code's order). **The user (BAT-Q49): the
  original's order.** Unobserved: a tie between a First Aid Tent and an Ammo Cart in the
  computer's aim would show it.
- **S-06, S-07** (`E04-quirks.md`: the towers) are D-16 and D-17: the code reading finds neither
  bug in the user's executable (T7, §7.30), and batch 3e confirms it (Air Shield and Armorer lower
  the damage, Archery does not touch it): **not quirks** of the user's version.

## 5. Experiments

Siege batches need towns on the experiment maps (the writer places heroes, monsters, piles and
grounds today, `docs/formats/h3m.md`): a town object with its buildings is the next writer step
(`h3-map`, a `cross-lane` change as T3's). An agent's run reads the opening from memory as batch
3 did: the turrets' counts (D-5, the count of buildings), the fort level, the moat flags, the
defending hero's raised skills and the stacks' morale; the Catapult's hit table needs many shots
and the shots' outcomes (T6).

**Batch 3c** (T5e, 2026-10-14; `experiments/results/E04c-batch-3c.md`): the walls, the gate and the
moat, played by an agent in hot seat with both armies commanded (`experiments/agent/siege.py`):
the moat's stop, damage and defense (D-10), the gate's opening, closing and passage, the shots
through the closed and the open gate (D-8) and the Tower's mines (D-9), every value as the code.
Seen on the way, for T6 and T7: at every round's start the keep shoots first, then the Catapult
(without Ballistics on its own, one shot); it destroyed wall parts but did not hit the gate in
these six shots. The gate is one of its targets (`BALLIST.TXT`'s gate column; the user expects
hits on it): T6 investigates its choice of target and what a destroyed gate does. **Read in T6
(§7.29):** without Ballistics the Catapult (one shot) aims at the weakest standing wall while one
stands and a miss lands on a wall, so it hits the gate only once every wall is down; with
Ballistics the side aims, the gate too, and a second shot (Advanced, Expert, the Cyclops King)
moves on to a neighbour of a fallen target, the gate among them. A destroyed gate lets both sides
pass, and its drawbridge's hex is no longer moat.

**Batch 3g** (T8, 2026-10-15; `experiments/results/E04c-batch-3g.md`, `experiments/agent/town.py`):
the battle window's buttons at a Castle, a Rampart and a Stronghold with the Escape Tunnel, the
defenders' morale and luck, Teleport at Basic, Advanced and Expert Water Magic from outside to the
moat and inside a Citadel (D-20), Force Field beside the walls: every value as the code. The
Brotherhood of the Sword replaces the Tavern's morale (D-21: +2 in all, the wiki's "in addition"
refuted).
