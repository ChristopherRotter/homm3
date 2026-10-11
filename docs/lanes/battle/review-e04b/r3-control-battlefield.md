# Review r3-control-battlefield: BAT-MAG-025..034, BAT-QUIRK-1..4, BAT-GEO-003 and BAT-MOV-003 (E04b parts)

Method: every rule printed with `rules.py`; the implementing Rust read in full (`control.rs`,
`obstacles.rs`, `natural.rs`, `movement.rs`, `turn.rs`, `legal.rs`, `observe.rs`, the cast paths of
`magic.rs`, the attack paths of `command.rs`/`combat.rs`, `war.rs`'s users of the enemy relation,
`terrain.rs`'s generator); `installation.md` §7.14 (Berserk, Hypnotize), §7.15, §7.16, §7.17; batch 2c
whole; the tests `control.rs`, `pits.rs`, `mines.rs`, `remove.rs`, `obstacles.rs`, the property
`hidden_obstacles_do_not_show`. No cargo run, no repository change. Test sketches are `needs-run`.

Executable re-read with `scripts/exe.py` (own words below, no listings):
0x442880, 0x442690, 0x469600, 0x523A70, 0x523B20, 0x523BB0, 0x523D90, 0x523DF0, 0x441610 (start),
0x476490 (whole: 0x476490-0x4769BA), 0x474690, 0x474A00, 0x4937D0, 0x465330 (0x465348-0x46540E),
0x4B3290, 0x4B3400 (0x4B3400-0x4B37D8 and 0x4B37A0-0x4B3960: the stop and attack-target logic, not the
cost/tie order), 0x445490, 0x4456D0, 0x4222C0, 0x4740D0 (0x474507-0x47467B), 0x441CB0 (0x441EED-end),
0x440160 (0x440230-end), 0x445A60 (start), 0x59FE30 cases 0x5A0354-0x5A0531 (Quicksand),
0x5A0536-0x5A0754 (Land Mine), 0x5A0759-0x5A0978 (Force Field, Fire Wall), 0x5A15CE-0x5A1779 (Dispel,
single, mass, objects), 0x5A1820-0x5A18A0 (Expert Cure), 0x5A1B4D-0x5A1D16 (Teleport, Remove Obstacle),
0x5A1D80-0x5A1E6B (Berserk), 0x5A4810, 0x5A66D0, 0x5A8090 (whole), 0x5A39C0 (whole), 0x5A3700,
0x467840 (start), 0x446C40, 0x4466A0, 0x446720, 0x4669B0, 0x466B30, 0x469A10, 0x475ED0
(0x4760C6-0x4761C5), 0x59EC50 (0x59EC50-0x59ED5A), 0x463C60 (0x463FC7-0x46406D), 0x4448F0 (duration
block and cases 59, 60), 0x465830, 0x43EA70, 0x59FCD0, 0x59FB40, 0x46A520, 0x46A570, 0x441FA0 (whole),
0x478900 (0x478900-0x478A6F), 0x445950, 0x445D10, 0x473EA0, 0x43F900 (0x43F998-0x43FA09); tables
0x642264, 0x642214, 0x642224, 0x5A3C64, descriptors 0x63CEE8, 0x63CF00, 0x63CF18, 0x63CF2C; callers
of 0x442880, 0x46A570, 0x441F70, 0x476490, 0x523B20; every read of +0x288 (list) and of +0x1329C/D.

VCMI 1.7.5 opened: `lib/battle/CBattleInfoEssentials.cpp` 100-127, 433-472;
`lib/battle/CBattleInfoCallback.cpp` 889-908, 984-1010, 1542-1602; `lib/battle/CObstacleInstance.cpp`
152-162; `lib/spells/effects/Obstacle.cpp` 119-124; `server/battles/BattleFlowProcessor.cpp` 400-475;
`server/battles/BattleActionProcessor.cpp` 338-351, 759-763, 955-1033; `config/spells/other.json`
(quicksand, landMine, forceField, fireWall, dispel, teleport, removeObstacle), `config/spells/timed.json`
(hypnotize), `config/gameConfig.json` 613-614.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-025 Berserk | cast, area, 255 rounds, ends Hypnotize, automatic turn, ties drawn, ends after own attack, at-once turn: as stated. "Its own side may attack it" and "the automatic Ballista may shoot it": in the code, not in the original | 0x5A1D80, 0x5A4810, 0x642264, 0x5A8090 (polarity exception for 59), 0x4448F0, 0x442880 + its two callers, 0x4745A8-0x4745FA, 0x4456D0, 0x445490, 0x4222C0, 0x441F55, 0x4402F7, cursor 0x476490; VCMI CBattleInfoCallback 889-908, 984-1010, 1542-1602, BattleFlowProcessor 457-475; batch B1 (lower, upper, lower: a draw) | B1's draw pinned (`a_berserk_shooter_...ties_are_drawn`); `control.rs:150-152` pins the own-side attack | F-2 |
| BAT-MAG-026 Hypnotize | limit (`<=`, full hit points, specialist bonus), Magic Mirror, duration, controller commands and casts, side tests by owner, battle end by owner, Q-0012: as stated and as the code reads. The "controller's friend, owner's enemy" relation, the aura, Ammo Cart, First Aid Tent and "mass versions pass over" (Dispel) are not what the original does | 0x5A8363-0x5A83C2, 0x4448F0 (59/60), 0x442690, 0x442880, 0x469600, 0x465389, 0x443D90, 0x5A82F8, 0x5A66D0, 0x5A1847, 0x5A163C-0x5A16D0, 0x465830, 0x43EA70, 0x473EA0, 0x43F9E1, cursor; VCMI CBattleInfoEssentials 433-472, timed.json; batch H1 and C1 (limit 60: 70 refused, 35 allowed, 40 of 70 refused; Expert Cure leaves it; Basic Cure ends it; Red commands in rounds 2 and 3) and H1's "anything odd" line | limit, mirror, command, Q-0012 both ways pinned; nothing on who may attack a hypnotized stack; no exact-boundary case | F-1, F-3, F-10 |
| BAT-MAG-027 spell obstacles, Force Field | as stated | 0x5A0759, descriptors (0, -16 / 0, -16, -34), 0x5A39C0 (Force Field branch), 0x4669B0, 0x476152 (countdown), 0x469A10; VCMI other.json forceField (2 rounds, mirrored there); F1 (defender's wall up-right; gone in round 3), A2 | shape, mirror, free hexes, rounds, anchors pinned | ok |
| BAT-MAG-028 Fire Wall | as stated, except the strike-and-return flight, which enters no hex | 0x5A0821 (value, hexes by row and side), 0x59FCD0, 0x46A520, 0x46A570, 0x441FA0, 0x445950, 0x478900, callers of 0x441F70; VCMI other.json fireWall, BattleActionProcessor 338-351; F1 (20, 48/46/44; defender up-left) | enter, once per move, stand after defend/wait/attack/shot, dies in fire, bonuses pinned (synthetic numbers) | F-6 |
| BAT-MAG-029 Teleport | as stated | 0x5A1B4D, 0x5A3700, 0x446C40 (shift only with its flag, which the test passes as 0), 0x467840 (returns true outside sieges); no observation; VCMI other.json teleport (siege limits only) | `teleport_moves_an_own_stack_to_a_free_position` | ok |
| BAT-MAG-030 Expert Dispel clears | as stated | 0x5A16FB-0x5A1777 (skips removed objects, any bit of 0x3C on the anchor hex) | `expert_dispel_removes_every_spell_obstacle`, visibility in `pits.rs:496-513` | ok |
| BAT-MAG-031 hidden obstacles, sight | as stated; the VCMI source is misdescribed | 0x46402C (bytes written once), 0x5A045C, 0x5A0676, 0x59ECF8 (the only readers), 0x4B3384-0x4B33C4 (stops by the real side), 0x5A39C0; VCMI CObstacleInstance 152-162, CBattleInfoEssentials 100-127 | sight both variants, events' visibility, property test over observation, legal commands, targets and reports | F-9, F-11 |
| BAT-MAG-032 Quicksand | count, draw, reveal, flyers, move-and-attack into an unseen pit: as stated. Three points differ from the original: the attack from a seen pit (BAT-QUIRK-2), a two-hex walker over its own pit hex, dead stacks without a corpse | 0x5A0354-0x5A04F4, 0x642214, 0x4B3290, 0x441FA0 (step tests 0x442266-0x442334), 0x478971, 0x474690, 0x474A00, pathfinder 0x4B36ED-0x4B37D1 | counts, corpses, reveal, seen pit, tail, both variants of quirks 2 and 4 | F-4, F-5, F-8 |
| BAT-MAG-033 Land Mine | as stated | 0x59ECEE, 0x5A0536-0x5A0716, 0x642224, 0x59FB40 (dead, owner's side by +0xF4, revealed, f <= 0; damage; removal), 0x46A570 (after Fire Wall's), 0x44237E; VCMI Obstacle.cpp 119-124 (refused while the other side has a native stack) | value, refusal both variants, walk on, attack still happens, own/immune cross unseen, flyer, stand, hypnotized | ok (the wiki's "castable when the natives are immune" stays the open experiment the rule names) |
| BAT-MAG-034 Remove Obstacle | as stated; `castable_spells` can list it without a target | 0x5A1C04-0x5A1D16, 0x466B30, 0x5A3A4E-0x5A3AA4 with the jump table 0x5A3C64 (none/Basic: no bit of 0x3C; Advanced: also 0x10; Expert: any object); A1 ("will not affect anything"), A2 (anchor hex removes the rock) | natural, anchors, placed-last, cleared hexes, mastery, sight pinned | F-7 |
| BAT-GEO-003 (E04b part) | as stated | 0x4669B0 (anchor bit and index always), 0x466B30, 0x469A10; A2 | `obstacle_pieces_are_validated`, anchors tests | ok (F-7 touches it) |
| BAT-MOV-003 (E04b part) | as stated | 0x441FA0, 0x4B3290; F2 (via the up-right neighbour for both sides: the engine's tie rule gives the same) | the tie rule that F2 supports is pinned by no test | F-10 (F-5 touches the stops) |
| BAT-QUIRK-1 natives' sight at the start | original confirmed: second read of 0x46402C and of every reader of the bytes; the wiki's Land Mine line ("started the battle with"); VCMI counts dead natives too | as above | each variant has a test that fails for the other (`pits.rs:434`, `pits.rs:467`; `mines.rs:200` both) | flag ok; F-9 (the "intended" side cites VCMI wrongly) |
| BAT-QUIRK-2 attack from a seen pit | **not confirmed**: the walker and 0x478971 do what the entry says, but the original's cursor never offers such an attack | 0x474690, 0x474A00, 0x4B378D-0x4B37D1 | `pits.rs:373` covers both variants | F-4 |
| BAT-QUIRK-3 anchor hex | original confirmed: 0x4669B0, 0x466B30, 0x5A39C0 re-read; batch A2 | as above | `remove.rs:160` (both variants), `remove.rs:212`, `remove.rs:266` | ok; class fits |
| BAT-QUIRK-4 draws avoid hidden obstacles | original confirmed: 0x5A03DE and 0x5A05F6 test all six object bits | as above | `remove.rs:336` (both variants; Land Mine shares `random_hexes`) | ok. Note: the same kind of leak as the three Q-0002 instances that are always fixed; kept toggleable by the user's decision (BAT-Q4) |

## Findings

### F-r3-control-battlefield-1: to other stacks a hypnotized stack stays its owner's; the engine treats it as its controller's (who may attack it, blocking, multi-headed strikes, Unicorn aura, First Aid Tent, Ammo Cart)
- rules: BAT-MAG-026 (also BAT-MAG-008, BAT-RNG-001, BAT-ATK-003, BAT-WAR-002/003/004/007 through it)
- kind: behaviour-bug (with rule-text and source: §7.14 and the catalog misread 0x442880)
- severity: high (wrong legal commands for both sides whenever Hypnotize is active)
- confidence: high for "the owner's stacks cannot attack it" (observation + reading + VCMI); medium-high
  for the rest (reading + VCMI, no observation). An experiment row "Red's stack attacks the stack Red
  hypnotized; Red's archers stand next to it" would settle the second half.
- evidence:
  - Engine: `Stack::is_hostile_to` (`state.rs:717-735`) compares both stacks' controllers. Its users:
    `command.rs:174` and `legal.rs:98` (attack and shot targets), `combat.rs:42` (a shooter is
    blocked), `combat.rs:627` (multi-headed secondaries), `war.rs:108`, `war.rs:172` (automatic
    Ballista). Same idea in `magic.rs:1936-1940` (aura: "of the same controller"), `war.rs:64` (tent
    patient: same controller), `combat.rs:847` (the controller's Ammo Cart).
  - Original, enemy test 0x442880: yes if either stack is berserk; otherwise it takes the first
    stack's side flipped by its own Hypnotize (0x4428AE-0x4428C8) and compares it with the second
    stack's plain side field +0xF4 (0x4428CE-0x4428D9). The second stack's Hypnotize is never read. Its
    only callers are the blocked-shooter test (0x46963E: first = the shooter, second = the adjacent
    stack) and the adjacent-stack test of multi-headed attacks (0x523D53 via 0x523B20, called by the
    melee strike 0x44165B: first = the attacker).
  - Original, the player's cursor 0x476490: the stack under the cursor is an attack or shot target
    only if its side field +0xF4 differs from the acting side +0x132C0 (0x4766CC, 0x476728,
    0x476753-0x476799); the acting side is the active stack's controller (0x465389-0x4653AE). There is
    no Hypnotize test on the target. The melee part then asks the pathfinder with the mover's
    controller as its side (0x523A70), and the pathfinder counts a stack as attackable when its +0xF4
    differs from that side (0x4B3801, 0x4B3910). The movement shadow does the same (0x4938D0-0x4938F9).
    The strike itself tests no side (0x445A60).
  - Original, aura lists 0x43EA70: a Unicorn protects a neighbour when the Unicorn's controller equals
    the neighbour's +0xF4 (0x43EAEE-0x43EB13, and the mirror case 0x43EB9C-0x43EBC3).
  - Original, automatic First Aid Tent 0x473EA0: loops over the record array of the tent's side
    (0x473F12-0x473F7A; dead stacks and war machines skipped; no Hypnotize test), so its own side's
    hypnotized stack is a patient and an enemy stack its side controls is not; the commanded tent's
    cursor wants the patient's +0xF4 equal to the acting side (0x476759-0x476784).
  - Original, shot routine: the shot is not counted when the hero of the shooter's +0xF4 side (no
    Hypnotize flip) wears artifact 5, the Ammo Cart (0x43F9E1-0x43FA03).
  - VCMI 1.7.5 agrees: `battleMatchOwner(attacker, defender)` compares `battleGetOwner(attacker)`
    (flipped while hypnotized) with the defender's initial side (`CBattleInfoEssentials.cpp:454-472`),
    used by `battleCanAttackUnit` (`CBattleInfoCallback.cpp:902`), `battleCanShoot` (1007) and the
    Ammo Cart (`CStack.cpp:312`); the automatic Ballista and tent compare `unitOwner()`
    (`BattleFlowProcessor.cpp:472, 712`).
  - Observation: batch 2c H1, last line: "While hypnotized, that stack can attack it's original side
    (Blue), but Blue's stacks cannot attack the hypnotized stack." The fold-in (E04b-log, "Batch 2c
    folded in") did not take this line up.
  - The same pattern is already a rule for Fear (BAT-QUIRK-18: the frightened stack by controller, the
    Azure Dragons by real side). What the catalog says of 0x443898/0x443DE6 (Offense, Armorer), of
    0x5A8090 (hero resistance, Pendants, Sphere) and of the side tests holds; I re-read them.
  - Scenario: Red hypnotizes Blue's Swordsman H. (a) Blue's Pikemen act: the original offers no attack
    on H; the engine lists `Attack { target: H }` and accepts it (and Blue's automatic Ballista may
    shoot H). (b) Red's Wolf Riders act: the original lets them attack H; the engine answers
    `NotAnEnemy`. (c) Blue's archers next to H: free to shoot in the original, blocked in the engine;
    Red's archers next to H: blocked in the original, free in the engine. (d) A Blue Unicorn next to H
    gives H its aura in the original, a Red one does not; the engine has it the other way round.
    (e) Blue's tent may heal H, Red's may not; a hypnotized Blue shooter is supplied by the Ammo Cart
    of Blue's hero, not of Red's.
- fix or test sketch: two relations instead of one. May attack or shoot: `mover.controller() !=
  target.side` (no Berserk term, see F-2). Counts as enemy for blocking and multi-headed strikes:
  either berserk, or `me.controller() != other.side`. Aura: `unicorn.controller() == target.side`.
  Tent: `patient.side == tent.side`. Ammo: the shooter's own side. Rule text and §7.14 to match. Test
  (`tests/control.rs`, needs-run):
  ```rust
  let mut s = arena(&r, [Some(adept(&r, &["hypnotize"], 3)), None],
      &[("fixed", 5, 8, 5)], &[("fixed", 5, 9, 5), ("fixed", 5, 10, 5)], 1);
  act(&mut s, &r, cast(&r, "hypnotize", stack(1))).unwrap();
  assert!(attacks(&legal(&s, &r, BattleSide::Attacker), 1));   // fails today
  activate(&mut s, 2);
  assert!(!attacks(&legal(&s, &r, BattleSide::Defender), 1));  // fails today (batch 2c H1)
  ```

### F-r3-control-battlefield-2: a berserk stack cannot be attacked by its own side in the original; the engine lets it, and a test pins that
- rules: BAT-MAG-025
- kind: behaviour-bug (rule-text: "its own side may attack it", "the automatic Ballista may shoot it")
- severity: medium (an action the original never offers; an agent can kill its own berserk stack)
- confidence: medium-high (cursor reading + VCMI; no observation). An experiment row (Berserk on the
  caster's own stack, then the cursor of another own stack on it) or the wiki would raise it.
- evidence: the rule's source for "it is every stack's enemy" is 0x442880, whose only callers are the
  blocked-shooter test and the multi-headed strike (F-1); both parts of the rule that rest on them are
  right (it blocks shooters, a berserk Hydra strikes every neighbour: 0x44163F picks the "any stack"
  mode). The attack permission is the cursor's: target's +0xF4 against the acting side
  (0x476759), with no Berserk test, and the pathfinder's side test (0x4B3801) is the same; a berserk
  stack's own turn passes side -1 (0x422329), nobody else's does. VCMI: `battleCanAttackUnit` has no
  Berserk exception (`CBattleInfoCallback.cpp:889-908`), `battleCanShoot` only for a berserk
  *attacker* (1006-1007), the automatic Ballista compares owners (`BattleFlowProcessor.cpp:472`).
  Engine: `state.rs:728-729` makes any berserk stack hostile to all, so `legal.rs:98` and
  `command.rs:174` offer the attack; `tests/control.rs:150-152` asserts `attacks(&commands, 1)` for
  the side's own berserk stack; `war.rs:172` lets the automatic Ballista shoot it (the VCMI port it
  names does not).
- fix or test sketch: as F-1 (no Berserk term in the attack permission; keep it for blocking and
  secondaries); flip `control.rs:152` to `!attacks(&commands, 1)`.

### F-r3-control-battlefield-3: Expert Dispel reaches hypnotized stacks in the original; with Q-0012 on (faithful) the engine passes over them
- rules: BAT-MAG-026 ("Mass versions pass over hypnotized stacks"), BAT-MAG-003, Q-0012
- kind: behaviour-bug (rule-text)
- severity: medium (the natural answer to Hypnotize, a mass Dispel, does nothing in `faithful`)
- confidence: high (the loop is short and complete; VCMI agrees: it has no skip of hypnotized units at all)
- evidence: Dispel's Expert branch is its own loop over both sides' records (0x5A163C-0x5A16D0): for
  each stack whose effect factor is above 0 it ends every spell but 71. It reads no Hypnotize field,
  unlike the mass handler (0x5A6713) and Expert Cure's loop (0x5A186E), which the rule and Q-0012
  rightly describe. Engine: `skips_hypnotized` (`magic.rs:854-874`) treats every mass version alike
  and returns true whenever the flag is on; Dispel is `mass = "expert"` in the pack
  (`packs/base/pack.toml:3443`), so `reachable` (`magic.rs:775-786`) drops the hypnotized stack.
  Scenario: Red hypnotizes Blue's stack; Blue's hero with Expert Water casts Dispel: original: the
  Hypnotize ends and Blue commands the stack again; engine (faithful): it stays hypnotized.
- fix or test sketch: let the quirk cover only what the original skips (the mass timed handler and
  Expert Cure); a mass dispel whose loop is its own (Dispel) reaches hypnotized stacks in every
  profile. Test (`tests/control.rs`, needs-run): `ruleset_with("t:hypnosis", ..)`, defender's hero
  `adept(&r, &["dispel"], 3)`, `give(&mut s, &r, (1, "hypnotize"), 1)`, `activate(&mut s, 2)`, cast
  Dispel at `CastTarget::None`, `assert!(!has(&s, 1, Status::HYPNOTIZED))` (fails today).

### F-r3-control-battlefield-4: BAT-QUIRK-2 does not hold for a player: the original never offers an attack from a seen pit, so the faithful default (legal and skipped) is the unfaithful one
- rules: BAT-QUIRK-2, BAT-MAG-032
- kind: source (a quirk kept without confirmation of its deciding path); behaviour-bug in `faithful`
- severity: medium (extra legal Attack commands in the default profile that only walk into the pit)
- confidence: high for the player's path; the computer player's single-turn query refuses it too
- evidence: the ledger says "the original's interface may not offer it; not read". I read it. When the
  cursor is on an attackable stack, 0x474690 builds the twelve attack slots: a neighbouring position
  is entered only if the reach map marks it, the stack fits, and neither its head hex nor (for a
  two-hex stack) its tail hex is in the stops array (0x4747BE-0x474810; the shifted two-hex position
  0x47485F-0x4748B5). 0x474A00 takes the attack's from-hex only from these slots (0x474B4F). The
  pathfinder's own attack marking skips a node on a stop when the range is within one turn
  (0x4B378D-0x4B37D1), so the cursor shows no attack at all if only stop positions touch the target.
  The stops array holds the pits the stack's side sees and the moat, minus the stack's own hexes
  (0x4B3290). So "accepted and skipped" needs a command no interface path issues. Engine:
  `legal.rs:121-123`, `command.rs:265-267`, `control.rs:53,136` add such attacks when
  `quirk.battle.attack_from_seen_pit` is on, which `base:faithful` has (`packs/base/pack.toml:7082`).
  (A case of an accepted attack that is skipped does exist, by another road: F-5.)
- fix or test sketch: drop the flag and keep today's strict behaviour (`Unreachable`) in every profile,
  or swap its defaults; move the ledger entry to "considered, not a quirk". `pits.rs:373` then keeps
  only its `false` row.

### F-r3-control-battlefield-5: a two-hex walker with one hex on a pit that moves along its row is stopped again after one step in the original; the engine lets it walk on
- rules: BAT-MAG-032 ("the hexes a stack starts on never stop it, so it leaves a pit freely"), BAT-MOV-003
- kind: question (reading against VCMI's default; behaviour-bug if the reading stands)
- severity: medium (Quicksand holds cavalry for two turns instead of one; an attack ordered with that
  step is skipped)
- confidence: medium. The walker's code is unambiguous; no observation; VCMI's default differs. A wiki
  line (below) or one experiment row settles it.
- evidence: at every step the walker 0x441FA0 tests the new head hex and, for a two-hex stack, the new
  tail hex (new head plus or minus 1 by facing, 0x4422C6-0x4422E8): in the stops array, or else
  carrying the Quicksand bit, in which case the pit is revealed; either makes the step the last and
  the walk return false (0x442266-0x442334). Only the stops array leaves out the hexes the stack
  started on (0x4B33D6-0x4B33F5); the direct Quicksand test has no exemption. For a step along the
  row the new tail hex is the old head hex. The pathfinder does exempt a hex the stack's other half
  already covers (0x4B36ED-0x4B3725), so the path and the cursor allow the move and the walker then
  cuts it. The catalog's own source sentence says it ("stops at a step whose new head or tail hex ...
  has Quicksand"). Engine: `movement.rs:109-119` removes the start hexes from the pits for the whole
  walk and `obstacles.rs:743-771` skips visited hexes. VCMI: start hexes are "passed"
  (`BattleActionProcessor.cpp:759-763`); its option `oneHexTriggersObstacles` ("double-wide creatures
  will trigger obstacle effect when moving one tile forward or backwards", `gameConfig.json:613`) is
  off by default.
  Scenario: Cavaliers (attacker, head on pit P, tail on P-1) are ordered three hexes to the right.
  Reading: they advance one hex (tail now on P), stop; a move-and-attack with that step loses its
  attack; next turn they are free. Engine: they walk the three hexes.
- fix or test sketch: if confirmed, a walker stops at a position where any hex of its footprint has a
  pit, except a pit its own half covered at the start only for the stops used in planning; reveal as
  now. Test (`tests/pits.rs`, needs-run): `arena(.., &[("wide", 5, 6, 5)], ..)`,
  `pits(&mut s, &r, BattleSide::Defender, &[(6, 5)])`, `Command::Move { to: at(&r, 9, 5) }` → expect
  `moves == [(0, at(6,5), at(7,5))]` under the reading (today: one move to (9, 5)).

### F-r3-control-battlefield-6: the strike-and-return flight enters no hex; in the original it is a move that enters the hexes it lands on
- rules: BAT-MAG-028 (a hex it enters during a move), BAT-MAG-033, BAT-ATK-004
- kind: behaviour-bug
- severity: low (needs a Harpy that starts its turn on a burning hex)
- confidence: medium-high (reading + VCMI)
- evidence: the return is the ordinary move handler (0x478A2B-0x478A30 → 0x445950): it clears the
  entered-hex array and marks the hexes the stack stands on now (0x4459B2 → 0x46A520), flies, and
  enters the landing hexes (0x4B4A1E → 0x441F70 → 0x46A570: Fire Wall, then Land Mine). The start hex
  is no longer marked, so a Fire Wall there burns the returning stack. VCMI returns with `moveStack`,
  which handles obstacles on the final hex (`BattleActionProcessor.cpp:338-351`, 1033). Engine:
  `combat.rs:756-768` sets the hex and reports `Moved`; `command.rs:423-425` calls no `stand` after a
  moved attack.
- fix or test sketch: after the return, enter the footprint as a flyer's landing does. Test
  (`tests/obstacles.rs`, needs-run): `burning(&r, &[("harpy", 20, 6, 5)], &[("fixed", 20, 9, 5)])`,
  attack stack 1 from (8, 5): expect the Harpies back on (6, 5) and one burn of 40 there (today none).

### F-r3-control-battlefield-7: `castable_spells` lists Remove Obstacle when no hex is a target (an obstacle orphaned by a removal)
- rules: BAT-MAG-034, BAT-GEO-003
- kind: robustness (legal-command consistency)
- severity: low (rare field, but generated ones can have it)
- confidence: high that the two functions disagree; needs-run for the test
- evidence: `NaturalObstacles::any_removable` (`natural.rs:166-170`) answers yes for any removable
  obstacle with a blocked hex ("a removal never clears the blocked hexes of another"), but with
  BAT-QUIRK-3 a removal clears its anchor hex (`natural.rs:198-200`), which another obstacle may
  block, and `removable_at` (`natural.rs:176-183`) then finds nothing there. The original has the same
  orphan (0x466B30 clears the index on the anchor hex) and, by A1, refuses the spell outright. The
  generator can produce it: an obstacle whose anchor it does not block (28 of 55 usual types), then a
  one-hex obstacle anchored on its own hex (`obstacle_21`, `obstacle_34`) on that anchor
  (`terrain.rs:59-73` only refuses a blocked anchor). After the first is removed through its own
  hexes, `has_target` (`magic.rs:1622-1624`) is true and `cast_targets` is empty. `h3-sim` relies on
  "a castable spell has at least one target" (`rollout.rs:99-110` then builds a cast at a default
  target, which `apply` rejects; an agent gets a spell with an all-false target mask).
- fix or test sketch: `any_removable`: some blocked hex not in `cleared`, or the anchor in `anchors`.
  Test (`tests/remove.rs`, needs-run): `ruleset_with("t:anchors", ..)`,
  `field_with(&r, caster(&r, &["remove"], 0), &[(&[(12, 2)], Some((11, 4))), (&[(11, 4)], Some((11, 4)))])`,
  `cast_at(&mut s, &r, "remove", (12, 2))`, reset the casts, then
  `assert!(targets(&s, &r, "remove").is_empty())` and
  `assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "remove")))` (fails today).

### F-r3-control-battlefield-8: Quicksand's and Land Mine's draw avoids the hexes of dead stacks that left no corpse
- rules: BAT-MAG-032 ("a stack, living or dead"), BAT-MAG-033
- kind: behaviour-bug (rule-text)
- severity: low (which hexes the random pits and mines can take)
- confidence: medium-high (the draw's test read; that created and sacrificed stacks leave no corpse is
  the project's own reading, §7.18)
- evidence: the draw skips a hex with a living stack or a corpse count above 0 (0x5A03E3-0x5A03F3,
  0x5A05FB-0x5A060D). A dead summoned stack or clone, a sacrificed stack and a corpse the Pit Lords
  used leave none. Engine: `obstacles.rs:388-390` takes the footprint of every stack in the list,
  dead ones without a corpse included, although the state knows them (`raised.corpse == NO_CORPSE`,
  `raise.rs:41-43, 362`, `created.rs:134, 477`).
- fix or test sketch: skip dead stacks whose corpse is `NO_CORPSE`. Test: as `pits.rs:190`, with the
  dead stack marked `SUMMONED`: its hex takes a pit.

### F-r3-control-battlefield-9: BAT-QUIRK-1's "intended behaviour" and BAT-MAG-031's VCMI source say VCMI tests the living natives; VCMI counts dead ones too
- rules: BAT-QUIRK-1, BAT-MAG-031, BAT-MAG-033 (and a label in BAT-MAG-026)
- kind: source
- severity: low (documents; but the user's classification at BAT-Q1 was given this description)
- confidence: high
- evidence: the ledger ("VCMI's `nativeVisible`; ... while it has a living stack"), §7.16 ("VCMI tests
  the living natives each time") and the catalog source of BAT-MAG-031 ("a side with a living native
  stack ... VCMI's living-natives rule is the fix") against VCMI 1.7.5: `battleHasNativeStack`
  (`CBattleInfoEssentials.cpp:107-118`) walks `battleGetAllStacks()`, which leaves out only ghosts
  (120-127), so a dead native stack still gives sight, and Land Mine stays refused
  (`Obstacle.cpp:121`). VCMI differs from the original only for natives that appear later. The wiki
  line the catalog quotes under BAT-MAG-033 ("if the enemy started the battle with a unit on its
  native terrain") supports the original's timing. So the strict-fixed variant (sight ends when the
  natives die) follows neither VCMI nor the wiki; it is the engine's own. Also: BAT-MAG-026's VCMI
  source names `config/spells/ability.json`; Hypnotize is in `config/spells/timed.json` (its URL is
  right), and VCMI agrees on Magic Mirror (`spell.magicMirror` in `noneOf`), which the entry does not say.
- fix or test sketch: correct the three texts; tell the user that VCMI and the wiki side with the
  original on dying natives, in case that changes the class or the strict-fixed variant.

### F-r3-control-battlefield-10: no test replays batch 2c with its numbers on the harvested packs; the path tie F2 supports and Hypnotize's exact limit are unpinned
- rules: BAT-MAG-025, 026, 027, 028, 034 (verified), BAT-MOV-003
- kind: test-gap
- severity: low
- confidence: high
- evidence: batch 2a has `crates/h3-pack/tests/harvested_spell_damage.rs`; nothing replays 2c (grep
  for the spells in `crates/h3-pack/tests`: only the Pendants). The battle tests use the synthetic
  pack with other numbers (Fire Wall 40 with power 3; Hypnotize 85 and 125). F1's "20 damage, 50 → 48
  → 46 → 44" and H1's "limit 60: 2 Swordsmen refused, 1 allowed, 40 of 70 refused" are reproduced
  nowhere. BAT-MOV-003's test (`obstacles.rs:734`) checks a shortest path around a wall, not the
  tie: reversing `Direction::ALL` or choosing from the start would pass, though F2 (burnt via the
  up-right neighbour, for attacker and defender alike) is the rule's only observation. Hypnotize's
  `<=` (0x5A83BA) has no case with full hit points equal to the limit.
- fix or test sketch: `harvested_control_and_field.rs` with F1 (power 1, 50 Pikemen: walk in, Wait,
  Defend), H1 (power 2, mastery none: targets), F2 (Fire Wall up-right / up-left, two rows straight
  up, both sides), A2's shape; in `control.rs` a stack of exactly the limit (power 2 at Basic: 60 = 6
  "fixed").

### F-r3-control-battlefield-11: a berserk stack's automatic turn follows its own side's hidden pits, and the other side watches it
- rules: BAT-MAG-025, BAT-MAG-031 ("what a side does not see never changes what it may do" covers
  commands, not what it can infer from an automatic turn)
- kind: security (inference from public events; the same kind as BAT-QUIRK-4), question for the ledger
- severity: low (needs Berserk on a stack of the side that owns hidden pits)
- confidence: high that the code does it; the original does the same (its pathfinder uses the stops of
  the stack's side, 0x4B3384-0x4B33C4)
- evidence: `control::berserk` builds its reach with `reachable_within` (`control.rs:52`), whose stops
  are the pits the stack's controller sees (`movement.rs:109`), its own side's hidden ones among
  them; `attack_position` (`control.rs:122-141`) ranks targets by the steps around those stops and
  drops positions on them, and `towards`/`travel` walk around them. The choice is the rule's, not the
  owner's, and every `Moved` and strike is public. Scenario: Red has a hidden pit on the short way
  from its stack R to Blue's B1; Blue casts Berserk on R; R, three steps from B1 without the pit, goes
  for B2 at four steps. Blue, who knows the rule and the field, learns that the short way holds a Red
  pit. The property `hidden_obstacles_do_not_show` cannot see this (it compares what commands report
  with and without the obstacles the *viewer* does not see, for the viewer's own commands).
- fix or test sketch: no clean fix (ignoring the pits would make the stack stop in them, which shows
  them too). Record it beside BAT-QUIRK-4 as a known inference, or let the user decide whether a
  berserk stack should plan with the revealed pits only.

## Wiki checks wanted

1. Hypnotize: who may attack a hypnotized stack (its owner's troops, the hypnotizer's), whether it
   blocks the hypnotizer's shooters, First Aid Tent and Ammo Cart for it (F-1).
2. Berserk: can a player attack or shoot his own berserk stack (F-2).
3. Dispel: does the Expert (mass) Dispel free the caster's own hypnotized stack (F-3).
4. Quicksand: is a two-hex creature stopped a second time when its rear hex enters the pit; can a
   creature standing in a pit attack by stepping one hex (F-5).
5. Quicksand, Land Mine: the wording on native terrain ("started the battle with" or living) (F-9).
6. Harpy, Fire Wall: is a Harpy burnt when it returns onto a Fire Wall hex (F-6).
7. Remove Obstacle, Dispel, Fire Wall, Force Field: any note of a crash or of a later obstacle
   vanishing after a timed obstacle was removed early (see the last point below).

## Not reviewed, left open

- Sieges: the moat's stops, the wall and moat limits of Teleport (0x5A3769-0x5A37B1 read only as far
  as "true outside sieges"; VCMI gives the moat at Advanced and the wall at Expert, the note in §7.15
  has the wall first: for E04c).
- The pathfinder 0x4B3400 beyond its stop and target logic (costs, tie order) and 0x41F580 (a berserk
  stack's walk towards a far target): the engine's own choices stay unchecked against it, as the rule
  says. The flight 0x4B49C0 and the teleporting move 0x4B4E90 only through their calls.
- The computer player's routines (0x422B20, 0x41E190) for a hypnotized or berserk target: not read;
  the automatic Ballista part of F-1/F-2 rests on VCMI.
- Hypnotize ending on the active stack in mid-turn (its controller dispels it): the original keeps
  the acting side until the next activation (it is set at 0x4653AE and otherwise only flipped around
  a retaliation, 0x441E35 and 0x441E52); the engine hands the turn to the owner at once
  (`side_to_act`). Rare; not followed further.
- Pointers outside my slice, met on the way (for the war-machine rules' reviewer): the shot routine's
  Ammo Cart test is "the hero of the shooter's side wears artifact 5", no living-cart test
  (0x43F9E1-0x43FA03; BAT-WAR-004 says "a living Ammo Cart"); the automatic First Aid Tent heals the
  stack with the most lost hit points among its side's records, not a random one (0x473F3C-0x473F67;
  BAT-WAR-003 says uniformly chosen).
- An original bug the engine rightly does not have, not in the ledger: a removed battlefield object
  stays in the list with its rounds; the round start (0x476162-0x47618F) has no "already removed"
  test (Expert Dispel's loop has one, 0x5A170F), so a Fire Wall or Force Field removed early by
  Remove Obstacle or Dispel is removed a second time when its rounds run out: the hex bits and index
  of whatever was placed there since are cleared, and 0x466C39 calls through the object's graphic,
  which the first removal set to 0. Read only, not observed; worth a ledger line (exploit-fix class)
  if an experiment or the wiki confirms it.
