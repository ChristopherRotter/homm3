# Review robustness

E04b T11b, lens: robustness. Base: `battle/e04b-t11b` at `4cd7e2e` (baseline verify green). Everything ran in the worktree `/home/claude/wt-robust` (branch `review/robustness`), x86-64 Linux, release builds with the workspace's overflow checks unless said otherwise. No engine code was changed.

Files next to this report (`<SCRATCH>/review/`):

- `robustness-failing-tests.patch`: the failing tests of the findings (applies to `4cd7e2e`): `crates/h3-battle/src/tests/review_repro.rs` (9 failing tests, 1 passing control) with a small hook in `tests/support.rs`, and `crates/h3-pack/tests/review_repro.rs` (2 failing tests on the harvested packs). Run: `cargo test -p h3-battle --lib review_repro`, `cargo test -p h3-pack --test review_repro`.
- `robustness-hidden-obstacles-regression.patch`: one line for `proptest-regressions/tests/properties.txt` that makes the existing property `hidden_obstacles_do_not_show` fail every time (F-robust-1).
- `robustness-stress-harness.patch`: the throwaway harness (applies on top of the first patch), so that the seeds below reproduce.
- `robust-logs/`: the logs of every run named below; `hidden-new-cases.txt` and `hidden-classes.txt` for F-robust-1.

## What the existing tests exercise of E04b (and what they do not)

- **Directed unit tests** (`crates/h3-battle/src/tests/*.rs`, most of the crate's 442 tests): each rule on small fixed setups of the in-code test pack. They are the rule coverage; they do not search.
- **`every_step_preserves_the_invariants`** (`tests/properties.rs`, 96 random cases of at most 160 steps): the one property that casts. It probes every listed hero cast and creature cast for acceptance, but only in profile `t:quirky` (exact rates, no quirk of E04b), with armies of 1 to 59 creatures a stack, one fixed hero (power 2, at most 63 mana, four mild magic effects), a `side_stack_limit` of 12 and a book of 35 spells that leaves out Berserk, Hypnotize, Blind, Forgetfulness, Frenzy, Anti-Magic, Magic Mirror, Cure, the shields and protections, Slayer, Counterstrike and Fire Shield.
- **`hidden_obstacles_do_not_show`** (same file): hidden pits and mines in three profiles (`t:sight`, `t:anchors`, `t:quirky`). It is the test that fails at random (F-robust-1).
- **`extreme_heroes_and_counts_play_without_failures`** (E04a T12c): extreme heroes and armies up to `u32::MAX`, but its heroes have no spell book, its effects are the E04a kinds only, its rates have no single-precision value of their own, and `finish()` plays `legal_commands`, which lists no cast. **No hero and no stack casts in it**, and it runs with exact rates only.
- **`random_legal_sod_heroes_derive_and_play`** (`crates/h3-pack/tests/harvested_heroes.rs`): derived SoD heroes with `mana: 0`, `legal_commands` only, profile `base:faithful` only. **No cast**; of the magic only what happens by itself (strike spells, Enchanters, opening spells).
- **`h3-sim`** rollouts, golden replays and benchmarks cast (`rollout::choose` takes the two cast entries), on the three synthetic scenarios with ordinary heroes. `h3-ffi` has five tests (two cast head by head, one feeds wrong arguments).

Not exercised by anything before this review: casts under quirk Q-0005 (single-precision rates) with large values; the E04b hero effects at the limits of their types; `Rate::single` bit patterns that are not ordinary numbers; casts on the harvested packs outside the directed rule tests; field shapes other than 17 × 11; the stack capacity of 42; unsound setups in bulk; the C ABI under wrong and random input; mutated packs.

## What I ran (configuration, battles, seeds, result)

Three kinds of generated heroes ("tiers"): **0** what a derived hero could have; **1** the full ranges of the seam's types (rates up to `u32::MAX` basis points, counts up to `u32::MAX`, opening powers up to 65 535, up to 44 effects), no single-precision patterns; **2** tier 1 plus arbitrary `Rate::single` bit patterns (NaN, infinities, negative, denormal). "safe": the spell damage bonuses of every spell and of a school at most 65 535 bp, to get past F-robust-3.

Every step of every battle: the other side has no legal command and no cast; the chosen command (40 % a hero's cast, 35 % a stack's cast when possible, else a legal command) must be accepted; invariants (effects sorted, unique, on living stacks; over ⇔ no active stack; no two stacks on a hex, none on an obstacle; hit point, count, morale and luck ranges; side stack limit; clone links; observations for both sides and the spectator; at sampled steps the recomputed effects equal the stored ones). At sampled steps **every** listed command (each castable spell at each target and second target) is applied to a clone and checked, and arbitrary commands must be rejected without a change or be listed. Panics are caught per battle.

| Run | Configuration | Battles, seeds | Result |
|---|---|---|---|
| S1 synthetic pack (`stress_synthetic`), 19 profiles × three (morale, stack limit) variants | tier 0 | 300 (1..), 20 000 (1000..), 60 000 (300000..), 30 000 (400000..) | F-robust-2 only (50 battles) |
| | tier 1, tier 2 | 20 000 each (1000..) | F-robust-3 (169 battles), F-robust-2 (13) |
| | tier 1 safe, tier 2 safe | 40 000 each (100000.., 200000..), 20 000 each (430000.., 450000..) | F-robust-2 only (56) |
| | dev profile (the one `debug_assert!`, in `outcome`): tier 0, 1 safe, 2 safe | 4 000, 2 500, 2 500 (700000.., 710000.., 720000..) | F-robust-2 only (2) |
| S1 totals | | 279 300 battles, 2.1 million hero casts, 368 million probed commands | two failure groups |
| S1 field shapes (`stress_fields`): 20 shapes from 4 × 11 to 255 × 1, 17 × 15, 15 × 17, 8 × 16 with 16 army slots | tier 0; tiers 1 and 2 safe | 27 000 (1000..); 10 800 each (500000.., 600000..) | 2 shapes refused by the compiler, `NoRoom` refusals on narrow fields, F-robust-2 only (112) |
| S1 unsound setups (`stress_setups`): 1 to 3 corruptions of a setup | | 20 000 (1..) | 18 781 refused (28 kinds of refusal), none panicked; 1 219 accepted and played (6 × F-robust-3) |
| S1 capacity (`stress_capacity`): both sides summon to the limit of 21 | | 1 260 | 1 017 reached all 42 stacks; no failure |
| S2 harvested packs through `h3-sim` (`stress_harvested`): every action chosen from the masks, decoded, encoded back and stepped; observations encoded; half on generated battlefields | `base:faithful` and `base:strict-fixed`, tier 0; tiers 1 and 2 (safe in `faithful`) | 44 100 (seeds in the logs `s2-*.log`) | no failure; 59 of the 60 real combat spells cast (not Earthquake) |
| | `base:faithful`, tiers 1 and 2 not safe | 2 000 each (70000.., 80000..) | F-robust-3 (170 battles) |
| S2 Remove Obstacle on generated battlefields (`remove_obstacle_on_generated_battlefields`) | each profile, 30 000 battlefields per terrain | 300 000 each | `faithful`: 71 end in F-robust-2; `strict-fixed`: 0 |
| S3 C ABI (`ffi_fuzz`): wrong sides, short buffers, canaries, random actions on forks | three synthetic scenarios | 240 games, 95 425 legal steps, 3.8 million random actions | no panic code, accepted ⇔ listed, a refusal changes nothing |
| S4 pack mutation (`fuzz_pack`): 1 to 3 values of the base pack's spells, creatures, obstacles or battle parameters changed, then battles that use them | both profiles | 60 000 trials (5001..) | 28 029 refused by the compiler without a panic, 31 971 compiled and played (722 000 hero casts): no failure |
| Existing property `hidden_obstacles_do_not_show`, fresh cases | as committed | 3 000 runs of 96 cases | 36 runs failed (1.2 %): F-robust-1 |
| The two kept tests' logic at scale | | 6 000 and 8 000 battles | no failure |

Observations that are not findings: one command may run many rounds while no stack can act (worst seen in play: 253 rounds, 2 031 events; forced on the test pack with a hero of power 255 and 60 `SpellDuration` effects: 15 555 rounds, 77 778 events, 4 ms); the lists of effects and spell obstacles grow past their reserved 32 and 8 entries (96 and 210 seen), which only reallocates.

## Findings

### F-robust-1: the automatic Ballista's target shows its side a Quicksand pit hidden from it, and the existing property test for it fails in 1.2 % of its runs

- kind: other (hidden information in a public event; observation hygiene, BAT-MAG-031, BAT-WAR-002). None of the brief's kinds.
- severity: high (reachable in normal play: default rules and packs, a hero's Quicksand against a side whose Ballista shoots on its own). The effect on play is small; the effect on the repository is that `cargo test -p h3-battle`, and so the verify, is red about once in 83 runs.
- reproduction:
  - Directed, `the_automatic_ballista_does_not_show_hidden_pits`: profile `t:faithful`, seed 1; attacker two stacks of 10 `fixed` placed at (3,5) and (3,9); defender two stacks of 10 `tank` at (9,5) and (9,9) and a Ballista; one obstacle over (4..=8, 4) and (4..=8, 6); the attacker's Quicksand pit on (6,5), hidden from the defender. Every stack defends; the Ballista shoots stack 1. In the same battle without the pit, which the defender observes identically (asserted), it shoots stack 0.
  - The existing property: `robustness-hidden-obstacles-regression.patch` (`cc c047ff82…`, profile `t:anchors`) fails every time. With fresh cases 36 of 3 000 runs failed; in all 36 the first difference is a war machine's shot at another target.
- where: `crates/h3-battle/src/war.rs:197` (`shooting_target` prefers a candidate that `can_attack_next_turn`, `war.rs:100`), which asks `movement::reachable` for the candidate; `crates/h3-battle/src/movement.rs:109` takes `obstacles::seen_pits(state, mover.controller())`, the pits the **candidate's** controller sees. Those include its own side's hidden pits, which the Ballista's side does not see, and the shot is public.
- fix sketch: answer "could attack next turn" from what both sides see (the revealed pits only, or none), then add the regression line. By reading, `control::berserk` walks with the berserk stack's own controller's pits (`reachable_within`); none of the 36 failures was such a case, but it deserves the same look.
- also reported, from reading: F-sec-1 (`security.md`, which expected the property to fail at random) and F-r3-control-battlefield-11. New here: the directed failing test, the saved case and the failure rate.

### F-robust-2: with quirk BAT-QUIRK-3, Remove Obstacle stays castable when no hex leads to an obstacle any more

- kind: legal command rejected. `castable_spells` lists the spell while `cast_targets` is empty; in `h3-sim` the entry `HERO_CAST` and the spell head are set and the target head is empty; `rollout::choose` then builds a cast without a target, which `apply` rejects (its comment says a castable spell has a target).
- severity: high by the brief's scale (reachable in normal play with the harvested packs in the default profile), rare and mild: 71 of 300 000 generated battlefields (grass 48, swamp 22, snow 1 of 30 000 each) once a hero removes obstacles; never in `base:strict-fixed`.
- reproduction:
  - `a_castable_remove_obstacle_has_a_target`: profile `t:anchors`, seed 1, 10 `fixed` each, `h` = hex (7,5); obstacles `{hexes [h], anchor h}` and `{hexes [h+1], anchor h}`; a hero with `t:remove` casts at `Hex(h+1)`; `Defend`, `Defend`; in round 2 the spell is castable and has no target.
  - Harvested, `remove_obstacle_keeps_a_target_while_it_is_castable`: `base:faithful`, grass, `generate_obstacles` with `Rng::for_stream(888, StreamId::OBSTACLES)`: the fourth obstacle blocks 109 and 110 from its anchor 108, the fifth is one hex on 108. Casts at 109, 122, 73, 80, one a round.
  - Stress: 233 battles, for example tier 0 seeds 5454 and 300374.
- where: `crates/h3-battle/src/natural.rs:166` (`any_removable` counts every obstacle with a blocked hex; its comment says "a removal never clears the blocked hexes of another"). `remove` (`natural.rs:189`) clears the removed obstacle's anchor hex under the quirk, and its own comment says that a hex another obstacle blocks there "stays blocked but leads to none". `magic::has_target` (`magic.rs:1615`) uses `any_removable`; `cast_targets` asks `removable_at` (`natural.rs:176`), which answers none on a cleared hex.
- fix sketch: `any_removable` = some obstacle has a blocked hex that is not cleared, or an anchor that still belongs to it. The two kept tests can then take every obstacle.
- also reported, from reading: F-r3-control-battlefield-7. New here: the failing tests, the generated battlefield of the harvested packs and the frequency.

### F-robust-3: `magic::spell_damage` overflows `u128` under quirk Q-0005 for a sound hero with huge spell damage bonuses, or for a Fire Shield that reflects a huge strike

- kind: overflow (a panic: overflow checks are on in every profile).
- severity: medium (extreme but valid input). The battle's soundness check (BAT-HER-001) accepts the setup. It can come from code, from a user of `h3-sim` or the C ABI, or from a scenario file (`crates/h3-pack/src/lib.rs:2303` reads `bp: u32` and `single` for `spell_damage`). By reading, not through `h3_heroes::derive_battle_hero` with the harvested packs: its rates stop at 65 535 bp, and the largest value there, a Fire Shield's reflection on `u32::MAX` Azure Dragons, stays about three times below the threshold. Not with exact rates (`strict-fixed`, `t:faithful`).
- condition: `value × (1 + school bonus) × (1 + bonus of every spell) ≥ 2^48` (about 2.8 · 10^14) at line 1711; two bonuses of 2^24 or more each (as `f32` values of their own) already at line 1710. With both bonuses at `u32::MAX` bp a value of 1 526 is enough.
- reproduction (each a test of the patch):
  - `huge_spell_damage_rates_do_not_overflow_the_spell_damage`: `t:single`, seed 1, 10 `fixed` each; hero power 255, mana 100, book `[t:bolt]`, effects `SpellDamage{All, u32::MAX bp}` and `SpellDamage{School(air), u32::MAX bp}`; `Cast{bolt, Stack(1)}` panics at `magic.rs:1711:27`. Control with `t:faithful` passes.
  - `spell_damage_rates_of_millions_of_percent_do_not_overflow` (both 3 316 000 000 bp); `saturated_single_precision_rates_do_not_overflow_the_factor` (both `single = 0x7100_0000`, panics at `magic.rs:1710:18`).
  - `a_fire_shield_reflects_a_huge_strike_without_overflow`: no spell damage bonus at all; `u32::MAX` `flyer` (hero attack 255, melee +100 %, damage raised by `StackStat`) strike `u32::MAX` `sultan` (innate Fire Shield) whose hero adds 40 × 32 767 hit points.
  - Harvested, `implosion_with_huge_spell_damage_rates_does_not_overflow`: `base:faithful`, power 20, Implosion, both bonuses `u32::MAX` bp.
  - Stress: tier 1 seeds 1318 (`t:single`), 9188, 5084 (`t:original`); tier 2 seeds 1094, 1205; harvested tier 1 seed 70008, tier 2 seed 80000. Reached through a hero's cast, an attack on a shielded stack and a move into a Fire Wall or a mine.
- where: `crates/h3-battle/src/magic.rs:1710-1711`. With Q-0005 the rates are fixed point over 2^40, so `(2^40 + s) × (2^40 + a)` alone takes up to 2^129, and `value` times it more. The `unwrap_or(u64::MAX)` on line 1711 shows that saturation was meant.
- fix sketch: saturate both products (`checked_mul`, else `u64::MAX`), or divide between the two factors (`mul_div_floor` twice); optionally bound the hero's rates in `hero::check`.
- also reported, from arithmetic: F-r1-5 (marked needs-run there). New here: it runs and panics, also with plain basis points (no `f32` pattern), also without any spell damage bonus through Fire Shield, and on the harvested packs.

### F-robust-4: a pack with a permanent turn-losing spell compiles, and one command then never returns when no stack can ever act

- kind: unbounded loop.
- severity: low (only a pack that validation should refuse). With the harvested packs the loop is bounded: only Disrupting Ray and Acid Breath are permanent.
- reproduction: `a_command_returns_when_no_stack_can_ever_act`: the test pack with `t:blind` set to `EffectDuration::Permanent`; attacker 10 `fast` with a hero (book `[t:blind]`, mana 100), defender 1 000 `azure` (`fear=100`); seed 3; `Cast{blind, Stack(1)}`, then `Defend` does not return (the test waits 10 seconds).
- where: `crates/h3-battle/src/turn.rs:207` (`activate`: the loop passes a stack that loses its turns and a frightened one, `pick` starts round after round, the round counter saturates). The compiler accepts `duration = "permanent"` for Blind, Paralyze and Petrify and a `fear` of 100 (`crates/h3-content/src/ability.rs:718`), while it refuses a certain bad-morale freeze for this very reason (`crates/h3-content/src/ruleset.rs:1378`).
- fix sketch: the compiler refuses a permanent duration for the changes that lose turns (as for the bad-morale chance); or `activate` stops after a full round in which no stack could act and nothing that stops them counts down.

### F-robust-5: an obstacle type with more than eight blocked hexes compiles, the generator places it, and the battle refuses the generated battlefield

- kind: validation gap.
- severity: low (only a pack that validation should refuse; the harvested types block at most eight).
- reproduction: `the_battle_accepts_every_generated_battlefield`: the test pack plus a usual obstacle type on `t:void` that blocks (0..9, 0); `generate_obstacles(&r, void, Rng::from_seed(1))`; `BattleState::new` answers `Err(TooManyObstacles)`.
- where: `crates/h3-content/src/ruleset.rs:1303` (`validate_obstacle` has no bound on the blocked hexes); `crates/h3-battle/src/natural.rs:25` and `:122` (`MAX_REMOVABLE_HEXES = 8`).
- fix sketch: the compiler refuses a usual obstacle type with more than `MAX_REMOVABLE_HEXES` blocked hexes.

### F-robust-6: Slayer's specialist bonus is clamped to `i32::MAX` and then added in `i32`

- kind: overflow (panic).
- severity: low (a pack whose Slayer has a value of about 20 000 or more and no specialist rule of its own, together with the largest specialty a sound hero can carry).
- reproduction: `a_huge_slayer_specialty_does_not_overflow_the_attack`: the test pack with `t:slayer` at value 30 000 and `specialist = None`; attacker 10 `archer` with a hero of 64 × `SpellSpecialty{slayer, 65 535, 255}`, defender 10 `lord`; seed 1; cast Slayer on the archers, shoot: panic at `combat.rs:246:19`.
- where: `crates/h3-battle/src/combat.rs:326` clamps the bonus into `i32`; `combat.rs:246` adds `i32::from(slayer.0) + slayer_bonus` to the attack.
- fix sketch: `saturating_add` at line 246.

## Tests kept (file, test names, commit ids on review/robustness; run time)

Both on the local branch `review/robustness` (not pushed), `cargo fmt` and `cargo clippy -p <crate> --all-targets -- -D warnings` clean, `cargo xtask check-rules` and `check-arch` ok, fixed seeds, no engine code.

| Commit | File | Test | Run time |
|---|---|---|---|
| `5ad9475` | `crates/h3-battle/src/tests/properties.rs` | `extreme_casting_heroes_play_without_failures` (`// covers: BAT-HER-001, BAT-MAG-001, BAT-MAG-003, BAT-MAG-048`): 200 battles in `t:quirky` and `t:original` of sound casting heroes at the limits of the seam's types; about 1 000 hero casts and 400 stack casts, each listed command accepted | 2.2 s dev, 0.16 s release |
| `46fd5b9` | `crates/h3-pack/tests/harvested_heroes.rs` | `random_legal_sod_heroes_cast_and_play` (`// covers: HER-BAT-011, BAT-HER-001, BAT-MAG-001, BAT-MAG-003, BAT-MAG-048, BAT-GEO-004`): 80 battles in `base:faithful` and `base:strict-fixed` of derived heroes with spell books and mana on generated battlefields; about 2 000 hero casts and 200 stack casts | 1.9 s dev, 0.15 s release |

Both are bounded by open findings, as their comments say, and should be widened with the fixes: the first keeps the Q-0005 half below F-robust-3 (spell damage bonuses of at most 50 % each, at most 200 000 creatures a stack) and anchors its obstacles on their own hex; the second drops generated obstacles that share a hex (F-robust-2).

The whole battle suite passed on the branch (443 tests, 49 s dev); an earlier run of the same tree failed once in `hidden_obstacles_do_not_show`, which is how F-robust-1 was found.

## Not exercised

- Sieges and adventure spells (outside E04b; Earthquake is the one real combat spell never cast).
- Scenario files and `HeroDetails` were not fuzzed: that a scenario file can carry F-robust-3's hero is from reading the parser. The pack mutation changed one to three values of the base pack per trial with ordinary heroes; findings 4 to 6 need combinations and came from reading.
- The server, protocol and client crates and the `h3` tools.
- Other targets (32-bit, WASM, `target-cpu=native`) and the determinism matrix; replays were not compared across my runs.
- Coverage-guided fuzzing (not set up in the repository).
- Battles longer than 450 commands (the harness's bound); `max_steps` truncation only in the C ABI run.
- The other automatic choices against hidden obstacles (the berserk turn): by reading only.
- F-robust-3's sibling sites were read, not proven exhaustively: the other `u128` products in `flee.rs`, `war.rs` and `magic.rs` fit by their operands' types.
