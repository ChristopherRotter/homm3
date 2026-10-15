# E04c T10b fold-in: independent check of the behaviour fixes

Branch `battle/e04c-t10b`, commits 4865233..HEAD. Read only; nothing built or run.

## Verdict per fix

- 29e7736 (F-r3a-1, tie after the Catapult): ok. A low robustness note on the slot test (P2) and a doc nit (P3).
- a094c2d (F-r3a-2, no Wait/Defend for the commanded Catapult; `turn::catapult_passes`): ok. Same slot note (P2).
- b31b10c (F-r3b-1, no Cyclops gate shot over a living stack): ok.
- 374a922 (F-r2a-1, a dead created stack holds no gate): ok.
- dfaa170 (F-r2b-1, frenzied striker in the moat): ok.
- 3063c72 (F-r4-3, Sacrifice refuses a turret): ok (no shipped pack changes).
- bf52214 (F-r5-1, a two-hex walker's new hexes only): ok.
- 1571345 (F-r5-2, the computer's moat avoidance): problem: the aim's turns follow the original, but the berserk walk, which the engine plays on its own, does not (P1).
- 60573a5 (F-r5-3, saturation): ok. Values in range are unchanged; at absurd counts the ratio saturates toward 1 instead of panicking.
- 17a0489 (F-r1-1/2/6, obstacles, Magic Plains opening spells): ok.
- 2659f1b (F-sec-1, the Escape Tunnel is hidden): ok. No other field, event, wire field or flat feature carries the tunnel.

## Problems

### P1: the moat avoidance is missing from the berserk walk (1571345; medium-low; a gap, not a regression)

The original's prologue (0x4B3420–0x4B34B2) runs inside the pathfinder 0x4B3400 for every caller. It tests the stack being pathed with 0x474BF0, which returns "automatic" for a non-war-machine whose controlling side has no human player. Two callers are a berserk stack's turn:
- the candidates: 0x445582 calls `ai_pathfinder_map(stack, -1, 0x7f, 0, -1)`, and that function passes its arg2 on to 0x4B3400 (0x4B2DF4–0x4B2E00);
- the walk: 0x42232C calls 0x4B3400 with arg2 = -1.

A berserk stack's control is not flipped (+0x288 is Hypnotize; Berserk is +0x284), so a computer player's berserk walker at a town avoids its stops by hit points. In the engine, `control::berserk` (control.rs:53) uses `movement::reachable_within` (movement.rs:87–98). That function still treats the moat as a stop the walk ends on, and never applies `aim::avoids_stops`.

Since arg2 is -1 on these paths, the "side 1" term never applies there. Only the gate term does: avoid = hp <= 4d || (gate not closed && hp <= 40d). `aim::avoids_stops` hard-codes `mover.side == Defender` (aim.rs, `avoids_stops`), so it cannot be reused unchanged.

Scenario: `human = [false, false]` (the default), a Tower siege (moat damage 150), the gate closed. The defender's hero casts Berserk on an attacker's stack of 10 Pikemen (100 hp <= 600). The stack's nearest candidate by walking cost lies beyond a moat hex.
- Original: moat hexes cannot be entered for this stack. The candidate set and costs differ, and the stack walks around the moat or picks another target.
- Engine: the walk enters the moat, stops there and takes 150 damage, which kills the stack.

The same holds for a defender's weak berserk stack once the gate is open.

Fix sketch:
- Change the signature to `avoids_stops(state, ruleset, mover, records_side: Option<BattleSide>)`, with the 40× term `(records_side == Some(Defender) || !closed)`. The aim passes `Some(mover.side)`, the berserk path passes `None`.
- In `control::berserk`, build the reach with the stops treated as not enterable when it holds. For example, a `reachable_within` variant that adds the stops to `blocked` instead of to `seen`; the start footprint stays exempt, as in mark_walk_stops (0x4B33D6–0x4B33DD).
- Test: a computer's berserk weak walker with a target only across the moat. Also the same with `human[controller] = true`, which must still wade through.

If this is out of T10b's scope, record it in BAT-SIE-022 and BAT-MAG-025 as a reading the engine does not yet follow, and put it in the queue's inbox.

### P2: the Catapult is identified by its slot number, which a raised stack can share (29e7736, a094c2d; low; unreachable with shipped packs)

`created::raise_corpse` gives a raised stack the dead stack's slot (created.rs:153, `slot: dead.slot`). If a corpse raiser raised a stack from the attacker's dead Catapult, the new stack would sit in `CATAPULT_SLOT` (10), and every slot test would take it for the Catapult. The new tests are `acted_side` (turn.rs:121), `legal_commands` (legal.rs:67, which lists no Wait, Defend or moves), `not_the_catapult` (command.rs:183) and `catapult_passes` (turn.rs:418). The older tests are turn.rs:63, war.rs:60/97 and siege.rs:755/790/807.

Consequence: a raised stack with nothing listed but Retreat/Surrender, or nothing at all when its hero has no Ballistics, which is a soft lock. The original tests creature 145, not the record (0x4650AE, 0x473C35, 0x4766B2).

This is unreachable today, because `raise::corpse_raisable` wants an `alive` corpse and no pack (base, synthetic) gives the Catapult `alive`.

Fix sketch: either
- a `Stack::is_catapult()` (`slot == CATAPULT_SLOT && !has(RAISED)`, or a trait set in `state::catapult`) used at every site, or
- a pack-compiler refusal of `alive` on a `war_machine`.

### P3: doc nit (29e7736)

`last_side_acted` is documented as "Side of the last stack that took a turn" in three places: observe.rs:82, state.rs:1418, and the flat encoding's "the side that took the last turn" (h3-sim observation.rs). After the attacker's Catapult it now reads Defender. It is the tie-break's side (the original's +0x132B8), and the docs should say so.

## What was checked (and found right)

- **29e7736.** 0x46508B–0x4650C5: the reset fires only when the last activated is the attacker's record (`[+0x132B8] == 0`) with creature 0x91; it sets side 1 and record 0. In the engine, slot 10 exists only on the attacker's side (defender machines 7–9, turrets 11–13, created stacks 255). All three writers take `acted_side`: `end_turn`, `activate` (the automatic Catapult) and `berserk_turn`. The Catapult has no morale and is immune to mind spells, so no other path reaches it.
- **a094c2d.**
  - The Catapult is active only when commanded (`war::is_commanded` needs `commanded_wall_shots`). While it is active a target stands, and the Catapult's `may_shoot_part` ignores stacks, so its list is never empty.
  - The refusal sits in `plan` (pure), so a rejected command leaves the state untouched.
  - `catapult_passes`: 0x4743A3 calls 0x473C00 on every tick of the loop. Without a standing target that function sets action 12 (0x473C9D–0x473CA2), so a hero's Earthquake that brings down the last target passes the turn, as `end_turn(Acted)` does: no Defend bonus, no morale for a war machine. Only a hero's cast (`Plan::Cast`) can bring targets down during that turn, and the call sits after `berserk_now`.
  - The client bot (`h3-view` Greedy) and the session's fallback find the ShootWall in the list.
- **b31b10c.** At 0x4766AA–0x4766B9, a hex whose record holds a stack (side byte >= 0; corpses are kept elsewhere) goes to the stack path for any creature but 0x91. That includes an own stack and the shooter itself (0x4766EA). The engine tests the living stacks covering `opens[part]`, which is 96 for the gate and None for the turret targets; a wall's hex cannot hold a stack while the wall stands. The check sits in both `legal_commands` and `plan`.
- **374a922.** `lies` now equals "has, or will have, a corpse after `mark_corpses`" (created.rs and raise.rs use the same `is_created` test). The other caller, `blocked_for`, changes in step: a defender may pass the closed gate over a dead summon on the bridge within the same command, which is consistent with no corpse in the original.
- **dfaa170.** 0x442590 (whole): +0xCC, then the Behemoth share (when a striker is passed), then -3 on either moat hex, not floored. 0x442514 calls it on the striker with the victim and flag 0, so the order matches the engine (effective defense, the quirk-gated share, minus the penalty). `frenzied` truncates a negative total toward 0, as `_ftol` does.
- **bf52214.** At 0x4B3701–0x4B3725, the new front is a stop unless it equals the old rear, and the new rear is a stop unless it equals the old front. That equals covered(to) \ covered(from). For a one-hex stack the test is unchanged. mark_walk_stops (0x4B3290) exempts the stack's own hexes, as `stops` does.
- **1571345, the aim's path.**
  - 0x4B3420–0x4B34B2: avoid = hp <= 4d || ((arg2 == 1 || [+0x53A4] != 3) && hp <= 40d).
  - hp is 0x443080 with flag 0: count × health − lost, 1 for a clone. d is 0x63BD18[town type], which equals the packs' `moat_damage` (70/70/150/90/70/90/70/90; Conflux 70).
  - [+0x53A4] is 3 at any town's setup, 1 when open, 0 when destroyed, which matches the engine's `fort == None || !gate_open`.
  - arg2 is the threat records' side (0x422B29 edi → 0x422C4F → 0x4B2DF4). The records skip hypnotized and berserk stacks, so owner equals controller there.
  - At 0x4B375C, avoid && stop takes the not-fitting branch (0x4B3772), which is the engine's `continue`. Flyers and Devils get no stops (0x4B32A9–0x4B32C8).
- **17a0489.**
  - 0x46608D rejects when height > row, matching `y < height`.
  - Usual types: sampler_init(0, 0x5A), and 0x50C740 draws uniformly among the values not drawn yet. Non-matching types are drawn again (0x46683C). A placeless type is consumed and the loop goes on (0x466858–0x466878). The loop ends on `counted >= to_block` or an empty sampler.
  - Anchors: sampler_init(0x12, 0xA8), drawn without repetition until one passes. That is uniform over the passing anchors, the engine's `pick`, and the same 18..=168 range.
  - Distributions are equal, though not draw for draw (the seed differs by design). `pick` on an empty list draws nothing.
  - The opening spells: `other_mastery` is used for the reach test, the value and the cast's mastery.
- **2659f1b.**
  - `retreat_seen` and `seen_by` cover `Observation.may_retreat` and `.siege`.
  - The flat encoding reads only `obs.may_retreat[own/other]` and the siege's faction, fort, moat, gate and parts.
  - The wire's `SideView.may_retreat` comes from the observation (h3-protocol map/mod.rs:289/328), and its `Siege` has no tunnel field.
  - `SiegeState` has no public accessor for the tunnel, and `BattleState` has none for its siege.
  - No event carries `SiegeState`. `YourTurn` goes to the acting side only. The state-hash checkpoints are server-private (session.rs:205–222).
  - A spectator sees `false`. The defender's retreat, once made, is public, as in the original.

## Addresses re-read

0x465080 (0x46508B–0x4650CF); 0x473C00 (head to 0x473CE9); 0x4743A3 ±25; 0x476490 (0x476670–0x476720); 0x442590 (whole); 0x442410 (0x4424E0–0x44254B); 0x4B3400 (0x4B3400–0x4B3570 and 0x4B3640–0x4B3790); 0x4B3290 (to 0x4B33DD); 0x474BF0 (head); 0x443080; 0x422B20 (0x422B20–0x422C60); 0x4B2DA0 (to 0x4B2E10); 0x4222C0 (0x422300–0x422340); 0x445490 (0x44555E–0x4455AE); 0x466290 (0x4667A0–0x466897); 0x466010 (to 0x4661B7); 0x50C740; the tables 0x63BD18 and 0x63BE68.
