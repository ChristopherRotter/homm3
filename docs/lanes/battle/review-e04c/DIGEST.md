# E04c T10b review digest (coordinator's verified list)

Legend: V = verified by the coordinator (re-read / traced), A = accepted on the report, D = user decision needed, X = rejected.

## r1 (battlefields) — 6 rules, no quirk; 6 findings
- F-r1-2 medium, V: obstacle anchor row test `y <= height` rejects (engine, VCMI) where the original rejects only `height > row` (0x46608D `cmp edx, ebx; jg`): the original accepts row == height. Fix: `terrain.rs:75` `y < height`. Changes random obstacles → golden replays, bench baselines, the obstacle property tests. D: follow the original (rule BAT-GEO-004 says VCMI's generator; its note stale).
- F-r1-6 low, A: the original draws usual types without repetition (sampler 0..90 initialised once, 0x4667F8) and continues after a type without a place (0x466858); the port repeats and `break`s. D: same decision as F-r1-2 (fix together or record as a VCMI-port divergence).
- F-r1-1 low, A: opening spells not raised to Expert on Magic Plains (`magic.rs:1055-1073`, no `other_mastery`); 0x59FE98 raises every kind ≠ 0. No shipped pack affected. Fix + test + summary.
- F-r1-3 low, test-gap: strike spells', periodic casts' mastery raise on Magic Plains unpinned.
- F-r1-4 low, test-gap: min-join of the field's level limit with a hero's untested.
- F-r1-5 low, source: BAT-GEO-004 lacks an `original-data` source; stale "not harvested yet" note.
- Wiki check wanted: creature abilities above level 1 on Cursed Ground.

## r2a (setup, walls, gate, moat stop) — 4 rules; 5 findings
- F-r2a-1 medium, V: `siege::lies` counts a freshly dead created stack (corpse 0 ≠ NO_CORPSE until `mark_corpses`, which runs after `close_gate` on the good-morale path, `command.rs:520/524`, `turn.rs:137-146`): the gate stays open through the killer's morale turn. Fix: in `lies`, `s.is_alive() || (corpse != NO_CORPSE && leaves a corpse)`; or run `mark_corpses` before `close_gate`. Test needs-run.
- F-r2a-2 low, rule-text: BAT-SIE-004 omits "no attack from a moat hex entered in the same move".
- F-r2a-3 low, source: BAT-SIE-003's VCMI note wrong ("only living stacks" — VCMI reads dead units too, agrees); moat path is `config/spells/moats.json`.
- F-r2a-4 low, test-gap: flyer's landing opens the gate; M5's walk-around on a single-row moat; a refused attack from the moat.
- F-r2a-5 low, rule-text: a defender's stack on 94 bars itself from 95 (engine and exe agree): say so.

## r2b (moat damage/defense, mines, shot line) — 4 rules; 2 findings + 2 test gaps
- F-r2b-1 medium, V: a frenzied striker in the moat adds its full defense; the original's 0x442410 calls 0x442590 on the striker (flag 0), which subtracts 3 on a moat hex (0x442646); VCMI agrees (moats.json defence −3 bonus feeds the frenzy bonus). Fix: `combat::frenzy_defense` subtracts the moat's penalty (no floor). Test needs-run; BAT-SIE-006 summary.
- F-r2b-2 low, A: the bow's wall-halving exemption is a setup trait of the shooter's own side's hero; the original (0x46751A–0x46753A) reads the controlling side's hero (hypnotized shooter). Same pattern as the distance penalty (BAT-RNG-002/BAT-HER-006, E04b). Record as a reading the engine does not follow (queue) unless cheap.
- T-1 low, test-gap: moat re-entry after Wait/Shoot/attack in place; moat hex to moat hex; a two-hex victim and the bridge hex for −3.
- T-2 low, test-gap: the hero's mine value on Magic Plains.
- The T10a integer line: sound (independent argument).

## r3a (Catapult, targets, its turn, destroyed walls) — 5 rules; 7 findings
- F-r3a-1 HIGH, V: the pick 0x465080 (0x46508B–0x4650C5) resets `last_activated_side` to 1 (defender) and the record to 0 when the last activated record was the attacker's Catapult (creature 145): the equal-speed tie after the Catapult's turn goes to the attacker. Engine `turn.rs:139/282` records the attacker → tie to the defender. §7.21 line 1958 already names this writer. Fix: after a turn of `CATAPULT_SLOT`, `last_side_acted = Some(Defender)` (both paths). Changes `siege-s7` and the siege golden replays/bench. Confirmation row for an experiment.
- F-r3a-2 medium, A: the commanded Catapult may Wait and Defend (`legal.rs:64-73,96-99`); the rule says ShootWall is its only command; batch 3d B: Defend did nothing; Wait unobserved. Fix: no Wait/Defend for the Catapult (tests `walls.rs:286-289,457`); Wait to the experiment candidates. D (minor).
- F-r3a-3 low, rule-text: BAT-SIE-014's tie text inverted (≤ 50 takes the later entry).
- F-r3a-4 low, source: BAT-SIE-009's VCMI file wrong (BattleInfo.cpp:311-320 + gameConfig.json:630).
- F-r3a-5 low, rule-text: BAT-SIE-011 "(it defends)" → no Defend bonus (action 12).
- F-r3a-6 low, test-gap: turrets-then-Catapult order in one battle; the tie after the Catapult.
- F-r3a-7 low, question: a hypnotized Catapult's Ballistics (owner's vs controller's, 0x445D64) — moot if Hypnotize cannot take war machines (check BAT-MAG-026).
- F-r3a-7: moot (BAT-MAG-026: Hypnotize spares war machines). X

## r3b (Cyclops, wall shot, Earthquake, Ballistics, BAT-QUIRK-22) — 4 rules + 1 quirk; 2 findings
- F-r3b-1 medium, V: the cursor 0x4766AA–0x4766B9 sends a non-Catapult shooter at a hex with a living stack to the stack-target path: a Cyclops cannot shoot the open gate while a living stack stands on 96; the engine lists and accepts `ShootWall { part: 3 }` (`siege.rs:750-766`, `command.rs:313-325`). Fix: refuse/omit a gate shot for a non-Catapult shooter while a living stack stands on the gate hex; test needs-run.
- F-r3b-2 low = F-r3a-3 (the tie wording) + no test of the direction.
- BAT-SIE-015, HER-BAT-018, BAT-QUIRK-22: no finding (quirk confirmed by re-read + batch E2).

## r4 (the turrets) — 4 rules; 3 findings, all low; no behaviour bug
- F-r4-1 low, test-gap: turrets before the Catapult in one battle; a waited turret first in the wait phase; the end with turrets alive; the end/rebirth after a turret's shot; Armorer of a hypnotized target's controller; Magic Mirror never redirecting to a turret.
- F-r4-2 low, rule-text: "a turret can always shoot" rests on the pack's 99 shots (`can_shoot` needs `shots > 0`).
- F-r4-3 low, rule-text/test-gap: Sacrifice's victim path (`raise::victim`) bypasses the turret immunity in code; rests on the packs' `excludes = ["war_machine"]`; a `!is_turret()` would close it.
- Batch 3e K/AC variance supports D-18's per-creature sum.

## r5 (the computer's aim) — 5 rules; 4 findings; the worth, kill value, divisors, ties, desperate mode and the Ballista's estimate match the original
- F-r5-1 medium, V: the AI pathfinder (0x4B3701–0x4B3725) exempts the hexes the stack currently covers from the stop test; the engine's `aim::stops`/`on_stop` (`aim.rs:345-363`) exempts only the starting footprint, so a two-hex walker crossing the moat pays the stop twice: turns off by one, the turret's/Ballista's divisor and target can flip. Fix: stops to test per step = new footprint less the current one. Test needs-run.
- F-r5-2 medium, A (D): the original's pathfinder makes a *computer-controlled* walker (0x474BF0: the side's player not human) avoid the moat by hit-point thresholds (4× / 40× the moat's damage, the gate's state; 0x4B3420–0x4B34B2, read at 0x4B375C): paths end at the moat. The engine always wades through; the hot-seat batches never exercise it; the engine's default `human = [false, false]` is exactly the case. User decision: (a) follow the original keyed on `human[controller]` (BattleSetup.human exists, ADR-0023 point 7), or (b) record the deviation in BAT-SIE-022 and docs/07. A re-read of 0x4B3481's branch and an experiment (a computer attacker) would raise the confidence.
- F-r5-3 low, robustness: `worth()`'s u64 hit-point sum and `kill_value`'s `damage + lost` overflow with extreme counts × hit points (the robustness reviewer may hit it).
- F-r5-4 low, test-gap: no base-pack test reproduces a batch 3f/3h row with its numbers.
- Left open: the flyer's occupied-hex crossing limit in 0x4B3400; 0x4221F0 not re-read.

## robustness — no panic, overflow, unbounded loop, state corruption or rejected legal command
- Ran: 111 120 synthetic extreme sieges (every test siege record/fort/moat, structures and morale/luck at the contract's extremes, Ballistics/Artillery 0..3, every special battlefield, extreme casting heroes, armies up to u32::MAX; ~9.3 M commands, ~426 k probes of every legal command, every Earthquake/Teleport target × destination, unlisted wall shots rejected with the hash unchanged), 8 120 harvested-pack sieges (all nine faction records, both profiles, derived heroes), the sim's siege scenario with mask⇔step agreement for every entry and garbage actions. The robustness run with u32::MAX counts met no overflow in the worth (it reads `wrapping_add` and `i64` sums bounded by `i32` inputs): F-r5-3 is to be re-checked against that reading before any guard.
- F-robust-1 low, validation gap: a natural obstacle on the Catapult's front hex at a walled town is refused as `NoRoom{Attacker,19}` rather than `InvalidSiege` (`siege.rs:296`).
- F-robust-2 low, coverage gap: no siege is reachable through the C ABI (`h3_env_create_synthetic` takes scenarios 0..2), so the wall-shot action entries are dead there.
- Observed, not a defect: a hero with 100 % damage reduction makes a battle endless.
- Kept (branch `review/robustness`, commit 6b9f5e9, pushed as a backup): `extreme_sieges_play_without_failures` (h3-battle properties.rs), `random_legal_sod_heroes_besiege_every_town` (new `h3-pack-battle/tests/harvested_sieges.rs`), `the_siege_scenario_s_mask_agrees_with_step_for_every_entry` (h3-sim env.rs); fmt, clippy and check-rules clean; no engine code touched. Throwaway harness: `robustness-throwaway.patch`.

## r6 (the town's other rules, the war machines' record order) — 8 rules + BAT-QUIRK-23; 5 findings, all low; no behaviour bug
- Re-read: 0x463600 (whole; the D-23 order Catapult, Ballista, Tent, Ammo Cart confirmed), 0x463B32–0x463C22, 0x478573–0x478635, 0x44AE60 (end), 0x44B2D0, 0x5A3700, 0x467490 (table 0x63BCE8), parts of 0x467840, 0x5A7080, 0x467130, 0x469A10, callers of 0x4671C0. BAT-QUIRK-23 confirmed by the re-read (VCMI agrees with a fort); both flag settings have a test that fails for the other.
- F-r6-1 question, X: BAT-QUIRK-23's class (`preserve` because the skip is a deliberate branch and VCMI calls the opposite its bug #551). The user decided fix-optional at BAT-Q48 with VCMI's "bug #551" in the question's text: no new evidence; the wiki check of the community status stays listed.
- F-r6-2 low, security: at a town `Observation.may_retreat[defender]` reads `Siege.escape_tunnel`, a building the attacker cannot see; the original enables the buttons for the acting side only (0x478573). To the security review (lead given).
- F-r6-3 low, source: BAT-SIE-028 cites no VCMI source; VCMI agrees that the defender's summon passes the closed gate but never opens the gate for a summon (`BattleProcessor.cpp:228-262`), where two readings of the original (T8, r6) and the engine do. Fix: the `vcmi` source with both halves, the survey's row, an experiment candidate; the engine keeps the original's reading.
- F-r6-4 low, test-gap: Teleport's line test for a move inside the walls (D-20: VCMI tests only crossings); landing on a damaging moat hex without damage; the batch rows' numbers (86→64/61/91; morale −1+2, luck +2).
- F-r6-5 low, rule-text: `h3-heroes/src/details.rs:266` "until sieges arrive" is stale.
- Left open (to the queue's inbox): the original sets up no war machine for a side without a hero (0x46374D–0x463752) and none with the start's bank flag (+0x53C5, 0x463758–0x46375D); the battle accepts listed machines without a hero and has no bank exception (harmless while HER-WAR-001 is the only producer).

## security — commands, hidden information, bounded work, the C ABI; 3 findings
- No finding: E04c's commands carry intent only and are validated completely before any change or draw; the Tower's mines do not leak (hexes fixed by the town; `hidden_obstacles_do_not_show` covers them at a town); work and storage bounded (wall line ≤ 17 × 16 steps, ≤ 8 wall targets, the aim's Dial queue); the C ABI's checks cover the wall-shot entries and the siege fields.
- F-sec-1 medium, V (leak; = F-r6-2): `observe.rs:195` publishes `may_retreat` for both sides; at a town the defender's is `Siege.escape_tunnel` (`siege.rs:408-412`), into the flat encoding (`h3-sim/src/observation.rs:366-367`) and the wire (`SideView.may_retreat`); the observed `SiegeState` carries the tunnel bit too (`observe.rs:240`). The original enables the buttons for the acting side only (0x478573). D → BAT-Q50 (4): hidden.
- F-sec-2 low, A (side channel): the automatic turrets' and Ballista's aim reads the other side's Armorer reduction and Artillery level (`aim.rs:92-101, 201-210`), its target is public; the original's computer reads the same. D → BAT-Q50 (5): kept, documented (06-security, beside BAT-Q31).
- F-sec-3 low, test-gap: the hygiene property varies no E04c private fact (Ballistics level, Artillery level other than 1, the tunnel) and runs no automatic turn after the change; `tests/observe.rs` has no town; `h3-protocol`'s frame tests play no siege (server lane: to its inbox).
- Left open: whether the turrets' strength (4 + structures) tells the town's building count where the original's tower window does not; how long one command runs when many rounds of turret turns run inside it.

# Summary (2026-10-15, the user stopped the review after these: r6 and the security review did not run)

Slices run: r1 battlefields, r2a setup/walls/gate/moat stop, r2b moat damage/defense, mines, shot line, r3a Catapult/targets/turn/destroyed, r3b Cyclops/wall shot/Earthquake/Ballistics/BAT-QUIRK-22, r4 turrets, r5 the computer's aim, robustness. **Not run:** r6 (BAT-SIE-024 to 029, BAT-QUIRK-23, BAT-WAR-001's record order D-23, HER-WAR-001, Teleport BAT-SIE-026, §7.32, batch 3g) and the security review (BRIEF-security.md is written). 28 rules and one quirk checked; 31 findings.

| # | sev | kind | claim | status |
|---|---|---|---|---|
| F-r3a-1 | high | behaviour | after the Catapult's turn the tie at equal speed goes to the attacker (0x4650AE–0x4650C5); engine: the defender | V — fix (`turn.rs`: last side = Defender after `CATAPULT_SLOT`); siege replays/bench change; confirmation row |
| F-r1-2 | medium | behaviour/source | obstacle anchor: the original accepts row == height (0x46608D); engine/VCMI reject | V — D: follow the original? (random obstacles, golden replays, bench change) |
| F-r2a-1 | medium | behaviour | a dead created stack holds the gate open through the killer's morale turn (`lies` before `mark_corpses`) | V — fix |
| F-r2b-1 | medium | behaviour | Frenzy in the moat: the original subtracts the moat's 3 from the striker's defense (0x442514 → 0x442646) | V — fix |
| F-r3a-2 | medium | behaviour | the commanded Catapult may Wait/Defend; the original offers neither (Defend observed, batch 3d B) | A — fix; Wait to the experiment candidates |
| F-r3b-1 | medium | behaviour | a Cyclops may ShootWall the gate while a living stack stands on it; the original's cursor offers the stack (0x4766AA–0x4766B9) | V — fix |
| F-r5-1 | medium | behaviour | the AI pathfinder exempts the currently covered hexes from the stop test (0x4B3701–0x4B3725); the engine only the start's | V — fix |
| F-r5-2 | medium | behaviour/question | a computer-controlled walker avoids the moat in the original (hit-point thresholds, 0x4B3420–0x4B34B2); the engine wades through | A — D: follow (keyed on `human[controller]`) or record the deviation |
| F-r1-1 | low | behaviour | opening spells not raised to Expert on Magic Plains | A — fix |
| F-r1-6 | low | behaviour/source | obstacle types drawn without repetition, the loop continues after a placeless type | A — with F-r1-2's decision |
| F-r2b-2 | low | behaviour | the bow's wall exemption reads the owner's hero, the original the controller's (hypnotized shooter) | A — record (as the distance penalty's note) |
| F-r1-3, F-r1-4, F-r2a-4, F-r2b T-1/T-2, F-r3a-6, F-r4-1, F-r5-4 | low | test-gap | listed per slice above | tests to add |
| F-r1-5, F-r2a-3, F-r3a-4 | low | source | stale/wrong sources (BAT-GEO-004, BAT-SIE-003's VCMI note, BAT-SIE-009's VCMI file) | docs |
| F-r2a-2, F-r2a-5, F-r3a-3/F-r3b-2, F-r3a-5, F-r4-2, F-r4-3 | low | rule-text | listed per slice above | docs (+ `!is_turret()` in `raise::victim`) |
| F-r5-3 | low | robustness | `worth()`/`kill_value` overflow with extreme counts × hit points | check against the robustness run; guard |
| F-robust-1, F-robust-2 | low | validation/coverage | see above | small fixes |
| F-r3a-7 | — | question | hypnotized Catapult's Ballistics | X (Hypnotize spares war machines) |

Decisions for the user (D): F-r1-2/F-r1-6 (the obstacle generator: the original's placement and draw, or VCMI's as today), F-r5-2 (the computer's moat avoidance), F-r3a-2 (minor: refuse Wait/Defend for the Catapult).

Wiki checks wanted: see each report's section (r1: creature abilities above level 1 on Cursed Ground; r2a, r2b, r3a, r4: listed).

## Next steps (T10b's remainder)
1. Run r6 and the security review (BRIEF.md, BRIEF-security.md; the r6 prompt: BAT-SIE-024..029, BAT-QUIRK-23, BAT-WAR-001 D-23, HER-WAR-001, §7.32 lines 2843–2936, §7.27's war machine part, batch 3g, `tests/town.rs`, `flee.rs`, `morale.rs`, `state::RECORD_ORDER`).
2. The user's decisions above; then fix workers (each fix test first), the docs worker (rule texts, sources, "Review re-read" paragraphs in `installation.md`, the queue's inbox for the readings the engine does not follow), re-record the siege replays and bench baselines, the full verify, the PR `battle/e04c-t10b`.
3. Merge the kept robustness tests (`review/robustness` 6b9f5e9) into the task branch.
