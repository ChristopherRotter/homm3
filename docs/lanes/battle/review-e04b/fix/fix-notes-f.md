# Fix notes F: the two retired quirk flags (BAT-Q29, BAT-Q30)

Worker `f`, worktree `/home/claude/wt-a`, branch `battle/e04b-t11b-f` (from `bc2d951`).
Logs: `<SCRATCH>/wf/`. Kept current after each step.

## State (update this first)

All done; nothing half-done. Branch `battle/e04b-t11b-f`, pushed, head `0b24be4`, tree clean.

- [done, pushed] Retirement 1: BAT-QUIRK-9 `quirk.battle.creature_mirror_untested` (BAT-Q29):
  commit `18fa9d6`.
- [done, pushed] Retirement 2: BAT-QUIRK-2 `quirk.battle.attack_from_seen_pit` (BAT-Q30):
  commit `0b24be4`.
- [done] Closing steps on the final tree: `cargo fmt --all -- --check` clean; clippy
  `-D warnings --all-targets` clean on h3-content, h3-battle, h3-pack, h3-harvest;
  `cargo test --workspace --no-fail-fast` once (`<SCRATCH>/wf/ws.log`): only the two golden
  tests fail; the battle crate's tests three times (502 passed each: `q2-battle.log`, `ws.log`,
  `final-battle-3.log`); `hidden_obstacles_do_not_show` 20 times and once with 960 cases;
  `cargo xtask check-rules`, `check-arch`, `check-stubs`, `check-exe-map` ok
  (`final-xtask.log`). No coverage run (the brief); every new branch has a fixed test. `target`
  is 4.6 GB (I removed its stale `debug/incremental`, 3.7 GB, before the workspace run).
- Not done on purpose: no golden replay or benchmark baseline regenerated, nothing under `docs/`
  changed, no version bumped.

## How a flag is retired (the form chosen)

`crates/h3-content/src/quirks.rs`:

- The registry stays a list of names by position. A retired position holds the private constant
  `RETIRED` (the empty name) with a comment that gives the retired flag's name, its quirk id, the
  date and the question. `REGISTRY.len()` stays 23 (positions).
- `QuirkRegistry::lookup` never finds a retired position: its closure asks `*n != RETIRED` before
  comparing, so neither the old name nor the empty name is found. A profile that names either
  fails with `CompileError::UnknownQuirk`, whether it sets the flag on or off (the compiler looks
  up every flag the selected profile names); a gated override that names it fails with
  `NotADataQuirk` as any unknown name.
- New accessor `QuirkRegistry::flags()`: the flags that exist, by index (every position but the
  retired ones). The one user outside the crate is `h3-pack`'s
  `the_profiles_differ_exactly_by_the_quirk_flags`, which before computed `(1 << len) - 1`.
  `len()` and `is_empty()` are kept and documented as counting positions.
- The constants and their re-exports are gone.
- Why this form and not a typed entry (`Option<&str>`, an enum): `QuirkRegistry::new` keeps its
  signature (the test registries in `h3-content` are unchanged), and the three properties hold:
  stable positions, not settable, 100 % coverage (the retired branch of both closures is reached
  by every lookup past the position and by `flags()`).

## Pack format: the precedent

No bump. The pack format's version (v34 since E04b T8e, the battle HANDOFF) counts new keys of
the pack files, not flags: the adventure lane added four flags to the registry, the harvester and
`packs/base/pack.toml`'s two profile lines on 2026-10-11 (commits `42c8cbd`, three
`quirk.adventure.*` flags, and `53bf939`, `quirk.adventure.fringe_shows_objects`) without a pack
format bump and without a ruleset hash version bump (v42 before and after). The battle flags
before them came in tasks that bumped the format for new keys of their own (for example `4468ec6`,
"periodic casters in packs (pack format v26), quirk flag 16"). `packs/README.md` now says so in
its "Profiles" paragraph.

## Retirement 1: BAT-QUIRK-9, `quirk.battle.creature_mirror_untested` (done, `18fa9d6`)

**Test first.** On the unchanged code (`bc2d951`) the two new battle tests failed for the
finding's reason, a `SpellRedirected` event for a stack's cast, at seed 0: in `t:faithful`
(`<SCRATCH>/wf/q9-first.log`) and, with the profile list swapped for the run, in `t:mirrored`
(`q9-first-mirrored.log`). The harvested test failed at `base:faithful` seed 1 with the redirect
put back for one run (`q9-harvested-first.log`); it passes with the fix.

**The profiles of the tests.** FIX-f asks for "`t:faithful` and a profile without quirks". In the
battle crate's test pack `t:faithful` *is* the profile without quirks (its name misleads; each
other `t:` profile has one or a few flags), and the only profile with the flag, `t:mirrored`, is
gone with it. So the battle tests run in `t:faithful` (no quirks) and `t:original` (three other
flags on: nothing reads a flag here any more, the second profile only shows that), and the real
profiles are covered where they exist: `h3-pack`'s `harvested_defences.rs` runs Faerie Dragons
against Faerie Dragons in `base:faithful` and `base:strict-fixed`.

**Code.**
- `crates/h3-battle/src/magic.rs` `reach_stacks`: the mirror is asked only for `cast.by.is_none()`
  (a match guard); `mirrored`, `untested` and the flag read are gone; `rolls` is
  `cast.by.is_none()`. A stack's cast at a mirrored stack now draws nothing from the generator
  (before: one draw for the chance, one for the new stack). `mirror`'s and `reach_stacks`' doc
  comments say so; the `// rule: BAT-MAG-049` marker sits on the guard.
- `crates/h3-content/src/quirks.rs`: the registry form (above), position 14 `RETIRED`, the
  constant gone; `lib.rs` re-exports. `spell.rs`: `TimedChange::MagicMirror`'s doc comment ("the
  other side's hero ... a creature's spell never does").
- `crates/h3-battle/src/tests/support.rs`: profile `t:mirrored` gone.
- `crates/h3-harvest/src/lib.rs`, `packs/synthetic/pack.toml`: the flag out of both profiles and
  out of the id lists in the comments. `packs/base/pack.toml` regenerated with
  `cargo run -p h3-harvest -- --install /home/claude/homm3/original-data/install --out packs`:
  the diff is its two profile lines and nothing else (`packs/ab`, `packs/sod` unchanged).
- `packs/README.md`: `magic_mirror` ("by the other side's hero ...; a creature's spell is never
  sent away") and the "Profiles" paragraph (a named flag must be registered; a retired flag is
  not; no format bump when a flag comes or goes; the retired flags by name).

**Tests** (all in the commit):
- `h3-battle` `tests/casters.rs` `a_magic_mirror_never_sends_a_creatures_spell_away` (replaces
  `a_mirrored_random_cast_is_untested_with_quirk_bat_quirk_9`): 20 seeds × 2 profiles; a random
  caster's bolt beside its own immune wyrm or resistant dwarves, its chain (70 on the aimed stack,
  35 on the next), and a fixed caster's Weaken, each at a stack under an Expert mirror (100 %): no
  `SpellRedirected`, no `SpellResisted`, the effect on the aimed stack.
- `h3-battle` `tests/creature_defences.rs` `a_mirror_of_its_own_never_sends_a_creatures_spell_away`
  (replaces `a_creatures_spell_at_a_dragon_is_sent_away_untested_with_quirk_bat_quirk_9`): the
  pixie's own 100 % mirror, 24 seeds × 2 profiles.
- `h3-pack` `tests/harvested_defences.rs`
  `a_faerie_dragons_spell_is_never_sent_away_by_the_other_dragons_mirror`: 80 seeds × the two real
  profiles; the drawn spell at the other dragons (their own 20 %); never sent away, the aimed
  stack struck; 30 to 65 of the 80 draws are stack spells.
- `h3-content` `quirks.rs` `a_retired_flag_keeps_its_position_and_has_no_name`; `ruleset/tests.rs`
  `quirk_flags_come_from_the_profile` (the retired name and the empty name, on and off:
  `UnknownQuirk`); `h3-pack` `strict_parsing` (the synthetic pack with the old flag set does not
  compile) and `the_profiles_differ_exactly_by_the_quirk_flags` (the flags that exist; bit 14 not
  among them).
- Unchanged and green: the hero's mirror tests of BAT-MAG-022 (`tests/defences.rs`) and
  BAT-MAG-057 (`a_faerie_dragons_mirror_sends_a_heros_spell_away_at_its_chance`,
  `an_innate_mirror_gives_its_chance_unless_the_spell_gives_more`).

**Runs after the change** (`<SCRATCH>/wf/q9-all.log`): `cargo test -p h3-battle -p h3-content
-p h3-heroes -p h3-pack -p h3-harvest -p h3-sim -p h3-cli -p h3-ffi -p h3-protocol`: everything
passes (h3-battle 501) but the two golden tests, both on `replay header mismatch: ruleset_hash`
(`fixtures/replays/duel-s1.toml` first; `fixtures/replays/adventure/duel-s1-d28.toml` first).
clippy `-D warnings` on the four touched crates clean; `check-rules`, `check-arch`,
`check-exe-map` ok.

## Retirement 2: BAT-QUIRK-2, `quirk.battle.attack_from_seen_pit` (done, `0b24be4`)

**Test first.** Behaviour changes only where the flag was on. On the code with the flag
(`18fa9d6`) the two new tests passed in the flag-off profiles (`<SCRATCH>/wf/q2-first-off.log`:
they pin what stays) and failed in `t:pits` for the finding's reason (`q2-first-on.log`): the
attack from the seen pit was listed (`pits.rs`), and the berserk stack walked into the pit and
struck nobody (`control.rs`: `strikes.first()` was `None`, not `(0, 2)`).

**Code** (today's flag-off branch is now the only one):
- `crates/h3-battle/src/legal.rs` `turn_commands`: an attack position is listed if it is the
  stack's own or `reach.can_move_to(from) && !reach.stops_at(from)`; the `pit_attacks` parameter
  is gone.
- `crates/h3-battle/src/command.rs` `plan`: `reach.steps(from).filter(|_| !reach.stops_at(from))`,
  else `CommandError::Unreachable` (before validation changes anything: the state is untouched).
- `crates/h3-battle/src/control.rs` `berserk`, `attack_position`: no position where the walk
  stops in a pit the stack's side sees.
- `crates/h3-battle/src/error.rs`: `Unreachable`'s doc comment without the quirk.
- `crates/h3-content/src/quirks.rs`: position 10 `RETIRED`, the constant gone; `lib.rs`.
- `crates/h3-battle/src/tests/support.rs`: profile `t:pits` gone.
- `crates/h3-harvest/src/lib.rs`, `packs/synthetic/pack.toml`: the flag out of both profiles; the
  id lists in the comments now read "BAT-QUIRK-1, BAT-QUIRK-3 to BAT-QUIRK-5, BAT-QUIRK-7,
  BAT-QUIRK-13" (base: "..., BAT-QUIRK-19, ADV-QUIRK-1 to ADV-QUIRK-4"). `packs/base/pack.toml`
  regenerated: one hunk, its two profile lines, nothing else. `packs/README.md`: the retired
  flags by name.
- Nothing else was reachable only through the flag: a walk that ends in a seen pit is still made
  by a plain `Move` (`obstacles::travel_through`, `stuck` without a reveal), and a walk with an
  attack still stops in a pit the side does not see.

**Tests** (all in the commit):
- `tests/pits.rs` `an_attack_whose_walk_stops_in_a_seen_pit_is_neither_listed_nor_accepted`
  (restates `an_attack_from_a_seen_pit_is_legal_only_with_bat_quirk_2`), in `t:faithful` (no
  quirks) and `t:sight` (BAT-QUIRK-1 on): a one-hex walker and a two-hex walker whose tail would
  enter the pit (the premise is asserted: the position is within reach and its walk stops): not
  listed, `Err(Unreachable)`, the state equal to its clone; the other positions and the move into
  the pit stay listed; a stack standing in the pit attacks from there; a flyer lands on the pit
  and strikes.
- `tests/control.rs` `a_berserk_stack_does_not_attack_from_a_pit_its_side_sees` (new; no fixed
  test pinned this branch of `control.rs` before): the nearer stack's only free neighbouring hex
  holds a pit. Its own side's pit: the berserk stack goes for the farther stack and strikes it.
  The other side's hidden pit: it goes for the nearer one, is stopped in the pit (revealed) and
  does not strike.
- `tests/properties.rs` `hidden_obstacles_do_not_show`: four profiles (BAT-QUIRK-1, BAT-QUIRK-3,
  neither, both); nothing else in that file changed. 20 runs and one with `PROPTEST_CASES=960`
  passed (`<SCRATCH>/wf/q2-prop.log`; proptest 1.11 applies the variable over the block's
  `cases: 96`).
- `h3-content` and `h3-pack`: the retired-name tests now name both flags; the profiles test pins
  bits 10 and 14 as unset.

**Runs after the change:** h3-battle 502 passed (`q2-battle.log`); h3-content, h3-pack, h3-harvest
pass (`q2-pack.log`); h3-heroes, h3-sim, h3-ffi, h3-protocol pass and h3-cli fails only the two
golden tests on `replay header mismatch: ruleset_hash` (`q2-all.log`); clippy clean.

## Golden replays, benchmarks, other lanes (nothing regenerated, nothing of theirs changed)

**The two tests that fail** (`cargo test --workspace --no-fail-fast`, `<SCRATCH>/wf/ws.log`; every
other target of the workspace passes: h3-adventure 71, h3-battle 502, h3-content 80, h3-game 21,
h3-map 42, h3-pack 17 + 27, h3-pack-adventure 8, h3-protocol 86, h3-server 94, h3-sim 40,
xtask 69, the rest green):
- `h3-cli` `golden_replays_reproduce_their_hashes`: `fixtures/replays/duel-s1.toml: replay header
  mismatch: ruleset_hash`.
- `h3-cli` `golden_adventure_replays_reproduce_their_hashes`:
  `fixtures/replays/adventure/duel-s1-d28.toml: replay header mismatch: ruleset_hash`.

New ruleset hashes (both retirements in): `synthetic:faithful` `0xf365519700129800` (was
`0xca39e2af5fabd024`), `base:faithful` `0xbff0f037828b990a` (was `0x742c5a43297a1b06`);
`base:strict-fixed` unchanged (`0x85731ab661e80677`). No other test pins a ruleset hash, and the
only test of a profile's flags outside `h3-content` is `h3-pack`'s
`the_profiles_differ_exactly_by_the_quirk_flags` (changed here).

**The adventure lane's replays change in one header line each, nothing else.** Checked on copies
in `<SCRATCH>/wf/replays/adv/` with `ruleset_hash` set to the new value: `duel-s1-d28` (136
commands, final hash `0x15638d883911a64c`) and `duel-s2-d7` (42 commands, final hash
`0xe421b1e52810b5b0`) verify with `h3 adventure verify` on the new code, commands, checkpoints
and final hashes as recorded; `duel-s5-strict-d7` verifies untouched. So
`fixtures/replays/adventure/duel-s1-d28.toml` L166 and `duel-s2-d7.toml` L66 need
`ruleset_hash = "0xbff0f037828b990a"`; a re-record gives the same.

**The battle's eight replays: a failure for another reason came before this branch.** I had not
expected it, so I checked it with the starting commit's own binary (a throwaway worktree of
`bc2d951` in the scratch folder, removed again): at `bc2d951` `h3 verify` already fails 7 of the 8
committed replays, from the review's earlier fixes, and only `full-s3-a1367` verifies:
- `duel-s1`: state hash after 371 commands (the final hash only: the last stack's death);
  `skirmish-s7`: after 225; `full-s3`: after 225; `full-s3-a1766`: after 125; `full-s3-a2378`:
  after 375; `full-s3-a303`: after 175 (each with the recorded commands still accepted up to
  there); `full-s3-a955`: command 132 rejected ("the target is not a hurt allied stack").
- So `golden_replays_reproduce_their_hashes` was red at `bc2d951` already; my header mismatch now
  hides that. The three adventure replays verified at `bc2d951`.

**What the two retirements change in the eight recordings** (same seeds, recorded into
`<SCRATCH>/wf/replays/oldgen/` with `bc2d951`'s binary and into `<SCRATCH>/wf/replays/new/` with
`0b24be4`'s; compared without the `ruleset_hash` line):
- Identical games (commands, checkpoints, final hash): `duel-s1`, `skirmish-s7`, `full-s3`,
  `full-s3-a955`, `full-s3-a1766`.
- Different games: `full-s3-a1367` (307 → 250 commands, from command 126), `full-s3-a2378`
  (535 → 497, from command 136, the move after a Quicksand cast) and `full-s3-a303` (531 → 541,
  from command 496); the winners stay. Why I think so: BAT-QUIRK-2. The side that sees pits has
  fewer legal commands (no attacks from them), and the random agent's draw over a shorter list
  picks another command: the first differing command is each time a move one hex off
  (`D move 55` → `D move 54`) of the defender, who has cast Mire, the synthetic Quicksand
  (`D cast 32 none`), before it in all three (in `full-s3-a2378` as the command just before). No
  recording shows an effect of BAT-QUIRK-9.
- `fixtures/README.md` describes what each `full` replay exercises ("mines or pits go off",
  "three rebirths", ...). Those lines were written for the T9b recordings; after the review's
  fixes and these two, a re-record with the same seeds gives other games for three of them, so
  the census of events per agent seed has to be redone and the seeds perhaps chosen anew.
  The scratch recordings are there to look at; they are not meant to be copied in.

**Benchmark baselines** (not measured: no valgrind run here). `benches/battle/
random-rollouts-full.toml` counts 20 random battles of the `full` scenario: its games change
with BAT-QUIRK-2 as the replays do, so the count will move and may pass the 2 % threshold;
compare `h3 bench --games 20` step counts before re-baselining (lane trap). `benches/server/
session-random-play-full.toml` (the server lane's) plays the same scenario through the server;
its README calls it lane-only (the full verify does not measure it, and a drift from `master` is
re-baselined at the server lane's next session). The adventure workloads run on `base:faithful`:
not measured; the two faithful replays play as before, which is no proof for the workloads' own
games.

**Other lanes' paths.** Nothing changed in them. They build and pass their tests. What this
change touches of theirs: the two adventure replays' header line (above; `fixtures/replays/
adventure/` is the adventure lane's); the server lane's bench baseline may drift (lane-only);
`docs/lanes/battle/QUEUE.md`'s note for the server lane ("the sample scenario hosts
`synthetic:faithful`, which has BAT-QUIRK-4 and BAT-QUIRK-2 on") is overtaken for BAT-QUIRK-2.
`clients/` holds only its README on this branch. No wire type changed; `h3-protocol` and
`h3-server` did not need a line.

## Open points for the coordinator

1. `golden_replays_reproduce_their_hashes` was already red at `bc2d951` (7 of 8 replays, the
   review's earlier fixes), not only through this list; the regeneration has more to carry than
   two flags, and three `full` replays change again with BAT-QUIRK-2 (above).
2. The battle crate's test profile `t:faithful` has no quirks, so "in `t:faithful` and in a
   profile without quirks" could not be taken literally: the battle tests use `t:faithful` (none)
   and a second profile with other flags; the real `base:faithful` and `base:strict-fixed` are
   covered for BAT-QUIRK-9 in `h3-pack`'s harvested tests. For BAT-QUIRK-2 there is no test on the
   harvested packs: a pit cannot be placed on a chosen hex through the public interface (the cast
   draws its hexes), and no code reads a flag there any more.
3. `QuirkRegistry::flags()` is a new public function of `h3-content` (a battle-lane crate), and
   `len()` now counts positions, retired ones included. If ADR-0007 should say how a flag is
   retired, that is a shared change for someone else (proposal under "Docs").
4. A berserk stack whose only attack position at the nearest stack lies on a pit its side sees:
   the engine takes another stack or passes (pinned now); what the original's walk does there was
   not read to its end (`installation.md` §7.16, BAT-Q31). Unchanged by this list.
5. The same stops hold the moat in a siege (0x4B3290): by the same reading the original offers no
   attack from a moat hex either. E04c's matter; nothing here models the moat.
6. An old pack or a recorded scenario that names a retired flag now fails to compile with
   "unknown quirk"; the message does not say "retired". The two golden-replay headers and the
   packs in the repository are the only users.

## Docs: what is now wrong, and the wording proposed

Nothing under `docs/` was changed. Old clauses are verbatim from this branch (`0b24be4`). After
the catalog's `quirks` fields change, `cargo xtask check-rules` still wants the two ledger
headings to exist only if a catalog entry names them; with the proposals below none does, and the
ledger keeps both entries (ids are not reused).

### `docs/rules/catalog/battle.toml`

**BAT-MAG-022 (Magic Mirror), summary.**
- Old: "a spell aimed at the stack by the other side's hero or by its random caster
  (BAT-MAG-049; a stack spell, not a mass version, an area or a battle-wide spell; Dispel and any
  spell that ignores immunity never) goes, if a draw from 0 to 99 is below v, to a living stack of
  the caster's side drawn with equal chances (war machines included; SpellRedirected)."
  New: "a spell aimed at the stack by the other side's hero (a stack spell, not a mass version, an
  area or a battle-wide spell; Dispel and any spell that ignores immunity never) goes, if a draw
  from 0 to 99 is below v, to a living stack of the caster's side drawn with equal chances (war
  machines included; SpellRedirected)."
- Old: "A creature's spell sent away is tested alike, but not at all with quirk BAT-QUIRK-9
  (BAT-MAG-049)."
  New: "A creature's spell is never sent away: a stack's cast always meets the stack it is aimed
  at, in every profile, and draws nothing for a mirror (BAT-MAG-048, BAT-MAG-049; the original
  hands the redirect to a hero's cast alone, installation.md §7.21; the user, BAT-Q29)."
- Old: "A creature's own mirror sends spells away alike (the Faerie Dragons' 20 %, BAT-MAG-057)."
  New: "A creature's own mirror sends a hero's spells away alike (the Faerie Dragons' 20 %,
  BAT-MAG-057)."
- Field, old: `quirks   = ["BAT-QUIRK-9"]`. New: the line goes (rules without quirks have none).
- Source `original-data`, optional addition at the end of `ref`: "; only the hero's action hands
  the second hex to 0x59FE30 (the dispatcher's case for action 1, 0x479036), a creature's action
  never (0x47936E; §7.21)". The `vcmi` source already says "a hero's direct negative spell".

**BAT-MAG-049 (the Faerie Dragon), summary.**
- Old: "Magic Mirror sends its single-target spells away as a hero's (BAT-MAG-022); the stack it
  is sent to is tested and rolls, but with quirk BAT-QUIRK-9 it takes the spell untested
  (immunities, Anti-Magic and resistance do not apply)."
  New: "A Magic Mirror never sends its spell away, neither the spell on the aimed stack nor that
  stack's own (BAT-MAG-022, BAT-MAG-057): only a hero's spell is redirected, in every profile (the
  user, BAT-Q29)."
- Source `original-data`, old tail: "the creature caster's button enters the cast through
  0x59EC50 (action 10), which decides Magic Mirror's redirect, and 0x59FE30 tests no stack of
  caster kind 1 (0x5A02CB), the redirected one included"
  New tail: "the F key enters the cast through 0x59EC50 (action 10), which draws Magic Mirror's
  redirect into the manager's +0x48, but the creature's action passes the target hex alone
  (0x47936E) and every cast of 0x448260 gives 0x59FE30 the second hex -1, so the redirect is
  dropped and the mouse path draws none (the review's re-read); 0x59FE30 tests no stack of caster
  kind 1 (0x5A02CB), which is only ever the aimed one"
- Source `vcmi`, old tail: "isReflected: only a hero's spell is reflected, a unit's never (the
  rule redirects it: BAT-Q29)". New tail: "isReflected: only a hero's spell is reflected, a
  unit's never (as the rule, BAT-Q29)".
- A `wiki` source for the decision's quote ("Spells cast by creatures ... will never be
  reflected", the Magic Mirror page) would fit here or at BAT-MAG-022; I did not fetch the page.

**BAT-MAG-057 (the Faerie Dragons' Magic Mirror), summary.**
- Old: "A hero's or a creature's single-target spell aimed at it by the other side is sent away as
  by the spell (BAT-MAG-022; a creature's spell sent away untested with quirk BAT-QUIRK-9,
  BAT-MAG-049)."
  New: "A single-target spell the other side's hero aims at it is sent away as by the spell
  (BAT-MAG-022); a creature's spell never is (BAT-MAG-049)."

**BAT-MAG-048 (a creature's cast), summary: optional addition** after "and the stack it is aimed
at never rolls for resistance (an area spell's stacks would)": "; no Magic Mirror sends it away
(BAT-MAG-022)".

**BAT-MAG-032 (Quicksand), summary.**
- Old: "An attack ordered with a move happens only if the stack arrives: a move and attack into an
  unseen pit stops there without the attack; with quirk BAT-QUIRK-2 (faithful) an attack from a
  position whose walk stops in a seen pit is legal and the attack is skipped, as the original's,
  without it (strict-fixed) it is not legal (CommandError::Unreachable)."
  New: "An attack ordered with a move happens only if the stack arrives: a move and attack into an
  unseen pit stops there without the attack. An attack from a position where the walk stops in a
  pit the stack's controller sees (with its head or, for a two-hex stack, its tail; not on the
  hexes it starts on) is not legal in any profile (CommandError::Unreachable; not listed), as the
  original offers none (installation.md §7.16; the user, BAT-Q30); the move into the pit stays
  legal, and a berserk stack's automatic turn takes no such position either (BAT-MAG-025)."
- Field, old: `quirks   = ["BAT-QUIRK-2", "BAT-QUIRK-3", "BAT-QUIRK-4"]`.
  New: `quirks   = ["BAT-QUIRK-3", "BAT-QUIRK-4"]`.
- Source `original-data`, addition at the end of `ref`: "; the cursor's attack positions
  (0x474690, from-hex only from them, 0x474A00) and the pathfinder's marking of attack targets
  within one move (0x4B378D) leave out a position with a hex on a stop, so no attack is offered
  from a seen pit (the review's re-read)".

**BAT-MAG-025 (Berserk), summary: optional addition** after "to the nearest of the target's
attack positions (the first in canonical order on ties)": ", never one where its walk stops in a
pit its side sees (BAT-MAG-032)". The summary says nothing about pits today; the new test
`a_berserk_stack_does_not_attack_from_a_pit_its_side_sees` covers BAT-MAG-025 and BAT-MAG-032.

### `docs/rules/quirks-ledger.md`

**BAT-QUIRK-2.** The entry stays under its id as a record of something considered and found to
be no quirk. Proposed fields:
- Title, old: "BAT-QUIRK-2 — An attack from a seen Quicksand pit is accepted and skipped".
  New: "BAT-QUIRK-2 — An attack from a seen Quicksand pit (retired: not a quirk)".
- "Class: fix-optional (user, 2026-10-07, BAT-Q1)" → "Class: none: considered, not a quirk
  (retired by the user, 2026-10-12, BAT-Q30; before: fix-optional, user, 2026-10-07, BAT-Q1)".
- "Original behavior: ... A command to attack from a seen pit therefore moves the stack into the
  pit and does nothing else." → keep the walker's part and replace the last sentence: "No path of
  the original issues such a command: the cursor's attack positions (0x474690, 0x4747BE to
  0x474810; the from-hex only from them, 0x474A00) and the pathfinder's marking of attack targets
  within one move (0x4B378D to 0x4B37D1) leave out every position with a hex on one of the
  stack's stops, so the original offers no attack from a pit the stack's side sees."
- "Intended/expected behavior: such an attack is not legal (strict-fixed), as it can never
  happen; the plain move into the pit stays legal." → "Engine behavior: such an attack is not
  legal in any profile (CommandError::Unreachable, not listed), as in the original; the plain
  move into the pit stays legal."
- "Impact: cosmetic for players (the original's interface may not offer it; not read), relevant
  for agents (an action that never does what it names)" → "Impact: none (the engine follows the
  original)".
- "Flag: `quirk.battle.attack_from_seen_pit` (E04b T5c1)" → "Flag: none; retired:
  `quirk.battle.attack_from_seen_pit` (E04b T5c1 to 2026-10-12). Its position 10 in
  `h3_content::REGISTRY` stays reserved and a profile that names it does not compile
  (`CompileError::UnknownQuirk`)".
- "Defaults: faithful = on, strict-fixed = off" → "Defaults: none (no flag)".
- "Regression test: `crates/h3-battle/src/tests/pits.rs`,
  `an_attack_from_a_seen_pit_is_legal_only_with_bat_quirk_2`" → "Regression test:
  `crates/h3-battle/src/tests/pits.rs`,
  `an_attack_whose_walk_stops_in_a_seen_pit_is_neither_listed_nor_accepted`;
  `crates/h3-battle/src/tests/control.rs`,
  `a_berserk_stack_does_not_attack_from_a_pit_its_side_sees`; the retired name:
  `crates/h3-content/src/quirks.rs`, `a_retired_flag_keeps_its_position_and_has_no_name`".
- "Decided by / date" gets "; retired: user, 2026-10-12 (BAT-Q30)".
- The "Review (E04b T11b, 2026-10-12)" line's end, old: "(`docs/formats/installation.md` §7.16);
  open as BAT-Q30." New: "(`docs/formats/installation.md` §7.16). The user retired the flag
  (BAT-Q30): the engine offers and accepts no such attack in any profile."

**BAT-QUIRK-9.** Likewise:
- Title, old: "BAT-QUIRK-9 — A creature's spell that Magic Mirror sends away is not tested (E04b
  T7b2a)". New: "BAT-QUIRK-9 — A creature's spell that Magic Mirror sends away (retired: not a
  quirk; E04b T7b2a)".
- "Class: fix-optional (user, 2026-10-08, BAT-Q11)" → "Class: none: considered, not a quirk
  (retired by the user, 2026-10-12, BAT-Q29; before: fix-optional, user, 2026-10-08, BAT-Q11)".
- "Original behavior: a Faerie Dragon's cast enters through the same routine as a hero's ... A
  hero's mirrored spell is tested and rolls (BAT-MAG-022)." → "Original behavior: no creature's
  spell is ever sent away. The redirect is drawn only in the player's targeting routine (0x59EC50,
  0x59EE7A to 0x59EF8B) and kept in the manager's +0x48, which only the hero's action hands to
  the cast routine (0x479036 to 0x479056); the creature's action passes the target hex alone
  (0x47936E to 0x479382) and every cast of 0x448260 gives the second hex -1. The cast routine's
  skipped test for caster kind 1 (0x5A02CB) therefore only ever meets the aimed stack. A hero's
  mirrored spell is tested and rolls (BAT-MAG-022)."
- "Intended/expected behavior: the redirected stack is tested and rolls as the new target of a
  hero's cast (strict-fixed)." → "Engine behavior: only a hero's spell is redirected, by the spell
  or by a creature's own mirror (BAT-MAG-057), in every profile; a stack's cast always meets the
  stack it is aimed at, as in the original and in VCMI."
- "Impact: gameplay-altering (small: ...)" → "Impact: none (the engine follows the original)".
- "Sources: original-data, `docs/formats/installation.md` §7.21 ("The Faerie Dragon, re-read for
  T7b2a")" → add "and the review re-read ("Magic Mirror never redirects a creature's spell");
  VCMI 1.7.5 `lib/spells/BattleSpellMechanics.cpp` L514–L523; the wiki's Magic Mirror page".
- "Flag: `quirk.battle.creature_mirror_untested` (E04b T7b2a)" → "Flag: none; retired:
  `quirk.battle.creature_mirror_untested` (E04b T7b2a to 2026-10-12). Its position 14 in
  `h3_content::REGISTRY` stays reserved and a profile that names it does not compile
  (`CompileError::UnknownQuirk`)".
- "Defaults: faithful = on, strict-fixed = off" → "Defaults: none (no flag)".
- "Regression test: `crates/h3-battle/src/tests/casters.rs`,
  `a_mirrored_random_cast_is_untested_with_quirk_bat_quirk_9`" → "Regression test:
  `crates/h3-battle/src/tests/casters.rs`, `a_magic_mirror_never_sends_a_creatures_spell_away`;
  `crates/h3-battle/src/tests/creature_defences.rs`,
  `a_mirror_of_its_own_never_sends_a_creatures_spell_away`;
  `crates/h3-pack/tests/harvested_defences.rs`,
  `a_faerie_dragons_spell_is_never_sent_away_by_the_other_dragons_mirror`".
- "Decided by / date" gets "; retired: user, 2026-10-12 (BAT-Q29)".
- The "Review" line's end, old: "The engine still redirects a creature's spell in both variants
  of the flag (`docs/formats/installation.md` §7.21); open as BAT-Q29." New: "The user retired
  the flag (BAT-Q29): the engine redirects only a hero's spell, in every profile."

**BAT-QUIRK-4's review line** (same file), old: "so a detour, a farther target, a passed turn or,
with BAT-QUIRK-2 on, a stop beside its target can show the other side where a hidden pit lies."
New: "so a detour, a farther target or a passed turn can show the other side where a hidden pit
lies." (The stop beside the target needed the retired flag; BAT-Q31's "sharper form" is gone.)

If the ledger's head or `docs/04-rules-and-sources.md` lists the classes, "retired: not a quirk"
is a new state of an entry; ADR-0007 does not name it. Whether ADR-0007 gets an amendment (a
retired flag's position stays reserved as an entry without a name; `lookup` never finds it;
`flags()` lists the flags that exist) is for the coordinator and the user: ADRs are shared.

### `docs/formats/installation.md`

- §7.16, "The walker's stops", old: "Either way the walk returns false, and a move and attack
  (0x478971) then skips the attack, also when the pit was the move's destination and seen
  (BAT-QUIRK-2)."
  New: "Either way the walk returns false, and a move and attack (0x478971) then skips the
  attack; a walk that ends in a seen pit is only ever a plain move, as no attack is offered from
  there (the review re-read below)."
- §7.16, "Review re-read ...: no attack is offered from a seen pit", old: "The engine lists and
  accepts such attacks while `quirk.battle.attack_from_seen_pit` is on, as it is in the faithful
  profile (BAT-QUIRK-2): it does not follow this yet: BAT-Q30."
  New: "The engine follows this since the user's decision of 2026-10-12 (BAT-Q30): it lists and
  accepts no attack whose walk stops in a pit the stack's side sees, in any profile, and the flag
  `quirk.battle.attack_from_seen_pit` of BAT-QUIRK-2 is retired (BAT-MAG-032)."
- §7.16, "Review re-read ...: a berserk stack's turn and its own side's pits", old: "a berserk
  stack whose attack position lies on a pit its side sees does not strike that turn, and whether
  it ends its walk on that pit, as the engine's does while BAT-QUIRK-2 is on, is not known."
  New: "a berserk stack whose attack position lies on a pit its side sees does not strike that
  turn, and whether it ends its walk on that pit is not known; the engine's takes no such
  position and goes for another stack or passes (BAT-MAG-025; the flag of BAT-QUIRK-2, under
  which it walked into the pit, is retired, BAT-Q30)."
- §7.21, "The Faerie Dragon, re-read for T7b2a", old: "So in the original no own Black Dragon or
  Earth Elemental takes the dragon's mirrored Magic Arrow or Lightning Bolt; quirk BAT-QUIRK-9
  and the engine still have it so (BAT-Q29)."
  New: "So in the original no own Black Dragon or Earth Elemental takes the dragon's mirrored
  Magic Arrow or Lightning Bolt, nor in the engine since BAT-QUIRK-9's flag was retired
  (2026-10-12, BAT-Q29)."
- §7.21, "Review re-read ...: Magic Mirror never redirects a creature's spell", old: "The engine
  still redirects a creature's single-target spell at a stack under Magic Mirror, or at Faerie
  Dragons with their own 20 %, untested while `quirk.battle.creature_mirror_untested` is on and
  tested without it: it does not follow this yet: BAT-Q29."
  New: "The engine follows this since the user's decision of 2026-10-12 (BAT-Q29): only a hero's
  spell is redirected, by the spell or by the Faerie Dragons' own 20 %, in every profile, and the
  flag `quirk.battle.creature_mirror_untested` of BAT-QUIRK-9 is retired (BAT-MAG-022,
  BAT-MAG-049, BAT-MAG-057)."
- §7.21, "The creature defences, re-read for T7c2a", old: "a creature's cast is never sent away
  (the review re-read above; the engine still redirects it, BAT-QUIRK-9, BAT-Q29)."
  New: "a creature's cast is never sent away (the review re-read above; nor in the engine since
  2026-10-12, BAT-Q29)."

### Other mentions (not asked for; for whoever closes the questions)

- `docs/OPEN-QUESTIONS.md`: BAT-Q29 and BAT-Q30 to "Answered" (option (a) each; implemented in
  `18fa9d6` and `0b24be4`). BAT-Q31's text says "(with BAT-QUIRK-2 on it may stop in the pit: the
  exact hex)": that case is gone; BAT-Q30's last sentence ("It also takes the sharper form out of
  BAT-Q31") is now fact. BAT-Q32's "(a) ... it carries ... BAT-Q29's check too" still names the
  Faerie Dragon session as a confirmation of BAT-Q29.
- `docs/exe/map.toml`: 0x5A02CB "though BAT-QUIRK-9 describes one (the review's re-read,
  BAT-Q29)" and 0x5A098A "(BAT-Q29)": true as history; could say "BAT-QUIRK-9, retired".
- `docs/experiments/README.md` L181 "(the engine today; BAT-Q29)", L209 "(BAT-Q30); one look at
  the cursor confirms it", L236 to L238: the engine now is as the reading on both; the rows stay
  as confirmations.
- `docs/lanes/battle/QUEUE.md` L28 "whether the interface offers an attack from a seen pit
  (BAT-QUIRK-2's faithful variant accepts it)" and L32 "`synthetic:faithful`, which has
  BAT-QUIRK-4 and BAT-QUIRK-2 on (... and the sharper form of BAT-Q31)": both overtaken.
- `docs/epics/E04b-battle-magic.md` L90, L136, L137 and `docs/epics/E04b-log.md`: history of T5c1
  and T7b2a (the flags' introduction); a T11b line should say both were retired.
- `packs/README.md` (mine, changed): the "Profiles" paragraph still says "the packs' `faithful`
  profiles turn Q-0003 and Q-0005 on" and the next paragraph "encoding v15,
  `RULESET_HASH_VERSION`" (the constant is 42): both stale before this change, left alone.
- The battle crate's test profile `t:faithful` has no quirks (its name misleads); a rename would
  touch most test files, not done.

