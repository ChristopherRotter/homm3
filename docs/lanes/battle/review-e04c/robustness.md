# Review robustness

E04c (sieges, special battlefields) at `b539054`, worktree `/home/claude/wt-robust`, branch
`review/robustness`. Scope: inputs that make a battle at a town or on a special battlefield panic,
overflow, loop without a bound, corrupt its state or reject its own legal commands. No rule
reading.

## What the existing tests exercise of E04c (and what they do not)

`crates/h3-battle/src/tests/properties.rs`:

- `arb_town` draws the three test siege records (`t:castle`, `t:inferno` with the second moat
  row, `t:necropolis` with mines), fort levels none/Fort/Citadel/Castle (weights 1:2:3:3), the
  moat absent/harmless/damaging (only from a Citadel), structures 0..12, an Escape Tunnel, morale
  0..=2, luck 0..=1, the attacker's Ballistics 0..=3 (a `CatapultControl` from 1) and the
  defender's Artillery 0..=3. `arb_ground` puts three games in seven at a town, one on one of
  the eight test battlefields, one at an anti-magic garrison. `situate` clears the natural
  obstacles at a walled town. A town is never combined with a special battlefield or anti-magic.
- `every_step_preserves_the_invariants` (96 cases, ≤160 steps, armies of 1..60): every legal
  command and every cast target × second target accepted; at a siege it asserts the turret/part
  pairing (each turret held by one target, alive exactly while it stands, no corpse) and that a
  turret's death is explained by `WallDestroyed`. Half the games run the quirk mix with
  BAT-QUIRK-23 (no defender Ballista at a fort).
- `walls_and_the_moat_agree_with_movement` (forced town with a fort) and its reach test
  `the_walls_property_s_games_reach_the_gate_the_moat_and_the_breaches` (96 fixed games): the
  gate opens/closes, moat entries and damage, the gate, a wall and a keep/tower destroyed, a
  breach crossed, a mine set off. Small armies only.
- `extreme_heroes_and_counts_play_without_failures` (48 proptest cases): armies up to
  `u32::MAX`, extreme heroes (attack 0..=255, up to 24 effects, any Ballista/Tent control), half
  the games at an `arb_town` with structures any byte, 5 000 steps with an agent that prefers
  `Attack` commands. It never casts (no Earthquake, no Teleport), never prefers the wall shots
  (`ShootWall` is picked only when no attack is listed), never draws a special battlefield, and
  does not probe the legal set against `apply`.
- `extreme_casting_heroes_play_without_failures` (200 fixed games): extreme casting heroes with
  every spell (Earthquake and Teleport among them), armies up to `u32::MAX`, half the games at a
  town with every fort level, moat, structures any byte, Ballistics and Artillery 0..=3, 400
  steps; casts are half the commands; every listed command accepted. No special battlefields, no
  wall-shot preference, no legal-set probe, no siege invariants checked.
- `crates/h3-pack-battle/tests/harvested_heroes.rs`: every setup has `siege: None` and no
  battlefield. No test on the harvested packs besieged a town before this review.
- `h3-sim`: the siege scenario was exercised only by the `random-rollouts-siege` benchmark (no
  mask/step agreement, no out-of-range actions). `h3-ffi` exposes only scenarios 0..2 (duel,
  skirmish, full): **a siege cannot be created through the C ABI at all**, so the eight wall-shot
  entries of the action encoding are never set in a mask the ABI hands out.

So before this review: towns with huge stacks and extreme heroes were played, but never with the
wall shots preferred, never with every legal command probed at a siege, never on a special
battlefield with extreme heroes, never with the harvested factions' records (Fortress's second
row and the Tower's mines in real data), never with the real Ballistics/Artillery derivation.

## What I ran (configuration, battles, seeds, result)

All in release (overflow checks on in every profile), one job at a time, `-j 2`.

1. **Synthetic extreme sieges** (`run_extreme_sieges` in `properties.rs`; the throwaway driver
   `stress_extreme_sieges` is in `robustness-throwaway.patch`). Per seed: three rulesets
   (`lively`, `lively_with("t:original")`, `lively_mix(no_luck_without_hero,
   clone_on_its_original, no_defender_ballista_at_fort)`), place = a town three in four (faction
   0..3, fort 0..4, moat 0..3, structures any byte, Escape Tunnel, morale and luck in
   {i8::MIN, i8::MAX, −4..=3}, Ballistics 0..=3, Artillery 0..=3) else a special battlefield; a
   town on a special battlefield one in eight, anti-magic one in sixteen; armies of 1..7 stacks,
   counts 1..100 / 60k..200k / any / `u32::MAX`, half the stacks Cyclopes, flyers, the
   teleporter, two-hex stacks or casters; `extreme_caster` heroes (attack/defense/power/knowledge
   in {0,1,7,30,99,255}, every spell, Ballista with 255 shots, extreme rates, 100 % damage
   reduction possible) on the attacker, 7/8 on the defender; a quarter of the games with counts
   ≤100 and mild heroes (so that sieges last); difficulty 0..5, human flags random. Agent: 40 %
   hero casts (half of them Earthquake/Teleport when castable), 15 % creature casts, else
   `ShootWall` two times in three when listed, strikes, or any legal command. Every 16th/25th step
   a probe: every legal command applied to a copy, every Earthquake/Teleport cast target × second
   target validated (one in eight applied), `ShootWall` at parts {0,3,7,8,9,200,255,random} that
   are not listed rejected by both sides with the hash unchanged. After every command: no living
   non-turret stack on a standing wall hex, turret ⇔ one target held, alive ⇔ stands,
   `top_hp ≤ max_hp`, `alive ⇔ top_hp > 0`. At the end: `observe` for both sides, `outcome ⇔ over`.
   - seeds 0..40 (kept test, 400 steps): 120 games, 120 over, 346 wall shots, 72 quakes, 76
     teleports, 85 moat damages, 252 parts destroyed, 21 keep/tower parts.
   - seeds 1000..6000, 1 000 steps: 15 000 games, 14 930 over, 41 365 wall shots, 12 874
     quakes, 13 659 teleports, 2 875 moat damages, 28 345 parts, 3 955 turrets; 54 843 probes.
   - seeds 100000..120000, 1 000 steps: 60 000 games, 59 674 over, 172 024 wall shots, 49 869
     quakes, 58 138 teleports, 116 763 parts, 16 258 turrets; 225 821 probes; 163 s.
   - seeds 200000..202000 (with the small-count quarter): 6 000 games, 5 993 over.
   - seeds 300000..310000 (with town-on-battlefield and anti-magic draws): 30 000 games, 29 865
     over, 83 980 wall shots, 29 275 quakes, 45 985 teleports, 58 198 parts, 8 160 turrets.
   - **Result: no panic, no rejected legal command, no invariant violation in 111 120 games /
     ~9.3 M commands / ~426 k probes.** The games not over at the bound were examined
     (`H3_STRESS_SHOW`, greedy replay in the patch): they are battles a hero with 100 % damage
     reduction (a valid `DamageBonuses.reduction` of 10 000 bp) or billions of creatures make
     endless, not stuck states (the side to act always had Move/Attack commands).
2. **Harvested packs** (`random_legal_sod_heroes_besiege_every_town`, SoD+AB+base packs, both
   profiles `base:faithful` and `base:strict-fixed`): derived heroes of random type, level,
   primary skills, Ballistics and Artillery 0..=3 each, up to six more skills, random worn
   artifacts (scrolls with random spells), a Spell Book, half the spells, mana; three games in
   four at the town of siege record `game % 9` (castle, rampart, tower with its mines, inferno,
   necropolis, dungeon, stronghold, fortress with its second moat row, conflux) at a random fort
   level, moat, structures any byte, morale −1..=2, luck 0..=2; else a random special
   battlefield; armies as in 1 with the packs' Cyclopes, flyers, teleporters, two-hex stacks and
   casters; loose/tight formations; difficulty and human flags random; 400 steps; a probe every
   20 steps. Seeds: base 77, 60 games per profile (kept): 120 games, 120 over, 262 wall shots,
   107 quakes, 127 teleports, 163 parts destroyed. Volume: base 5, 4 000 games per profile:
   8 000 games, 7 989 over, 19 653 wall shots, 6 694 quakes, 7 189 teleports, 13 343 parts;
   14.5 s. **No failure.**
3. **The sim** (`the_siege_scenario_s_mask_agrees_with_step_for_every_entry`,
   `packs/synthetic/scenarios/siege.toml`, seeds 0..4, 250 steps, the baseline agent of
   `random_rollout`): for every entry 0..ACTION_COUNT+8 but the two cast heads, `step` on a fork
   succeeds exactly when the mask says so and an unmasked entry leaves the hash unchanged; four
   garbage actions per step (random entry up to `u32::MAX`, random spell/target/second) are
   refused or accepted without a panic, a refusal changing nothing; the observation encodes for
   both viewers and a buffer one short is refused; the Catapult's and the Hurlers' wall shots are
   played (>10 per run). An earlier run with 6 seeds × 300 steps × every entry every step passed
   too. **No failure.**
4. **The C ABI**: read `crates/h3-ffi/src/lib.rs`; its size and null checks are covered by
   `errors_are_codes_not_panics`; no siege reachable (see "Not exercised").
5. **Setup validation probe** (throwaway `probe_siege_setup_errors` in the patch): a walled
   town with a natural obstacle on the Catapult's rear hex 119 → `InvalidObstacle(119)` (a side
   column), on its front hex 120 → `NoRoom { Attacker, slot 19 }`, on a moat or wall hex →
   `InvalidSiege`; seven attacker stacks in tight and in loose formation at a Castle with a hero
   set up fine (the Catapult takes its hexes before the army).

## Findings

No panic, overflow, unbounded loop, state corruption or rejected legal command was found, by
reading or by running. What the reading established, for the record:

- Arithmetic: the turret's damage is a per-creature draw summed in `u64` with `saturating_add`
  (count ≤ 255 + 255 + 4); moat damage ≤ 10 000 (compiler) applied through `apply_damage`
  (saturating); wall hit points `u8` with `saturating_add`/`saturating_sub`; Earthquake's loop is
  bounded by its `u16` hit count (and usually by the parts' hit points); the computer's worth and
  scores run in `X87`/`i32` with `ftol` and `wrapping_add`, the only `i64` additions
  (`kill_value`) fed from `i32` or from an estimate bounded by a shot's damage; `LineAxis` steps
  at most 17 × 16 times in `u64` numerators; the aim's Dial queue pushes each position once (costs
  pop in nondecreasing order, a stop's cost is the next multiple of the speed), within
  `MAX_FIELD_HEXES + 1` entries and 128 buckets.
- Indexing: `part` from a command is checked by `stands(part)` (`t < targets`) before any use;
  `turret(t)` uses `get`; a siege record's hit points are cross-checked against the field's
  targets (`ruleset.rs:608`); the Ballistics table is a fixed array of four with 1..=8 shots and
  chances ≤ 100; the compiler checks the moat rows, the gate among the walls, the turrets'
  off-field positions and lists of distinct target indices.
- Capacity: the mines go into a `Vec`; the turrets take at most 3 of the defender's 21 records
  (`room`); a side's stacks ≤ `side_stack_limit` was asserted in every probe.
- Turrets stay out of every target search (`may_attack`, `affects_stack`, `living`, the aim's
  loops), their footprint covers no field hex, and their death leaves `NO_CORPSE`.

### F-robust-1: a natural obstacle on the Catapult's front hex refuses a walled town as `NoRoom`, not as an invalid siege
- kind: validation gap
- severity: low (an explicit obstacle list at a walled town; the engine's own generator places
  none there, `terrain::generate_setup_obstacles`)
- reproduction: `lively()` ruleset, attacker with a hero, `siege = Some(Siege { fort: Fort, .. })`,
  `obstacles = pieces(&[120])` (the siege field's `catapult + 1`): `BattleState::new` →
  `Err(NoRoom { side: Attacker, slot: 19 })`. Probe `probe_siege_setup_errors` in
  `robustness-throwaway.patch`.
- where: `crates/h3-battle/src/siege.rs:296-301` (`kept` = walls ∪ moat ∪ mines; the Catapult's
  hexes are not in it) and `state.rs:445-473` (`catapult` → `NoRoom`).
- fix sketch: add the siege field's `catapult` and `catapult + 1` to `kept` when the town is
  walled and the attacker has a hero (or document `NoRoom` as the answer). Refused either way:
  no state is built.

### F-robust-2: no siege reaches the C ABI
- kind: validation gap (coverage)
- severity: low (nothing fails; the Python/RL side cannot train on sieges and the eight
  wall-shot action entries are dead through the ABI)
- reproduction: `h3_env_create_synthetic(scenario, ..)` accepts 0..2 only
  (`crates/h3-ffi/src/lib.rs:235-243`); `SYNTHETIC_SIEGE` has no code.
- fix sketch: scenario 3 = siege (and a sentence in the ABI doc); the sim test added here is the
  template for its mask/step check.

Observed, not a defect: a hero whose `DamageBonuses.reduction` is 10 000 bp (100 %) makes its
stacks invulnerable, so a battle against it never ends on its own (the step bound of any runner
is the only end; the server's `CommandLimit` covers hosted games). BAT-HER-001 accepts the value;
whether the original caps it is a rules question for the hero lane (OPEN-QUESTIONS Q31 context).

## Tests kept (file, test names, commit ids on review/robustness; run time)

Commit `6b9f5e9` on `review/robustness` (worktree `/home/claude/wt-robust`, not pushed):

- `crates/h3-battle/src/tests/properties.rs`: `extreme_sieges_play_without_failures` (seeds
  0..40 × three rulesets, 400 steps, probe every 16 steps; asserts the reach: >50 wall shots,
  >20 quakes, >10 teleports, >10 moat damages, >20 parts, >3 turrets' parts, >20 games over) —
  0.4 s release, 5.9 s debug. Helpers `extreme_place`, `siege_army`, `SiegeReach`,
  `standing_walls`, `siege_invariants`, `siege_probe`, `play_siege`, `extreme_siege_setup`,
  `siege_rulesets`, `run_extreme_sieges`. `// covers: BAT-SIE-005, BAT-SIE-011, BAT-SIE-013,
  BAT-SIE-015, BAT-SIE-017, BAT-SIE-026`.
- `crates/h3-pack-battle/tests/harvested_sieges.rs` (new): `random_legal_sod_heroes_besiege_every_town`
  (60 games per profile, seed 77; asserts >50 wall shots, >10 quakes, >10 teleports, >20 parts,
  >20 over) — 0.3 s release, 3.7 s debug. `// covers: BAT-SIE-001, BAT-SIE-007, BAT-SIE-011,
  BAT-SIE-013, BAT-SIE-015, BAT-SIE-026, BAT-TER-002`.
- `crates/h3-sim/src/env.rs`: `the_siege_scenario_s_mask_agrees_with_step_for_every_entry`
  (4 seeds, 250 steps, every entry every fifth step) — 4 s release, 5.3 s debug. `// covers:
  BAT-SIE-011, BAT-SIE-013`.

`cargo fmt --check` clean; `cargo clippy -p h3-battle -p h3-pack-battle -p h3-sim --all-targets
-- -D warnings` clean; no engine code touched (no coverage impact). `cargo xtask check-rules` ok
(265 rules, 1286 covers markers).

Throwaway material, not committed: `<SCRATCH>/review/robustness-throwaway.patch` (the working
tree before the clean-up, on top of `b539054`: the `H3_STRESS_SEEDS`/`H3_STRESS_FROM`/
`H3_STRESS_SHOW` volume driver, the greedy replay `greedy_extreme_siege` with `H3_GREEDY_SEED`,
the pack test's `H3_STRESS_SEED`/`H3_STRESS_GAMES`, `probe_siege_setup_errors`). Logs under
`<SCRATCH>/review/logs/` (`stress1..4.log`, `packstress1.log`, `clippy.log`).

## Not exercised

- The C ABI with a siege (F-robust-2): no scenario reaches one.
- The original-data packs' `h3 bench random-rollouts-siege` workload was not re-run (its
  baseline is `bench-check`'s; the sim test plays the same scenario with the same agent).
- Earthquake with a pack whose spell value is near `u16::MAX` (65 535 iterations: bounded, not
  run). Siege records with fewer hit points than targets are refused by the compiler, not run.
- A `BattleSetup` whose `siege.faction` has no record (`UnknownSiege`) and a moat below a
  Citadel (`InvalidSiege`) are refused by code I read, not run here (the existing
  `tests/sieges.rs` covers them).
- Hypnotized wall shooters (a Cyclops controlled by the defender, the Catapult controlled by
  the defender): legality consistency held wherever the random games reached them, but no game
  was steered there; the rule itself (who may shoot which walls) is a rules reviewer's.
- Long sieges with the gate cycling for hundreds of rounds under small armies ran only in the
  small-count quarter of runs 3 and 4 (≈9 000 games), not under the `walls` property's
  per-event assertions.
- `h3-protocol`/`h3-server` over the wire with a siege: out of the brief.
