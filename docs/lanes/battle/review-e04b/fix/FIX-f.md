# List F: two quirk flags are retired (the user's decisions BAT-Q29 and BAT-Q30, 2026-10-12)

Worker name: `f`. Worktree: `/home/claude/wt-a`. Branch: `battle/e04b-t11b-f` (starts at the
joined review branch, head `bc2d951`, rebased onto today's `master`: it has every fix and test of
both review rounds so far). Logs: `<SCRATCH>/wf/`. Notes: `<SCRATCH>/review/fix-notes-f.md`.
`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.

Read `<SCRATCH>/review/BRIEF-fix.md` first. All of it applies (where you work, test first, the hard
limits, the closing steps, the form of the notes and of the final message), except where this
list says otherwise.

## The two decisions

The review re-read the original's executable on two quirks and found that neither exists in the
original. The user decided on 2026-10-12 to retire both flags. The questions with the evidence:
`docs/OPEN-QUESTIONS.md`, BAT-Q29 and BAT-Q30; the readings: `docs/formats/installation.md`, the
paragraphs "Review re-read (E04b T11b, 2026-10-12): Magic Mirror never redirects a creature's
spell" (§7.21) and "… no attack is offered from a seen pit" (§7.16); the findings:
`<SCRATCH>/review/r5a-creature-casters.md` (F-r5a-1) and `r3-control-battlefield.md` (F-…-4).

1. **BAT-QUIRK-9, `quirk.battle.creature_mirror_untested` (BAT-Q29).** Magic Mirror never
   redirects a creature's spell. The executable (the redirect is drawn only in the player's
   targeting of a hero's spell and never handed to a creature's action), VCMI 1.7.5
   (`BattleSpellMechanics.cpp` L514–L523: only a hero's cast is reflected) and the wiki ("Spells
   cast by creatures … will never be reflected") agree. **Intended:** only a hero's spell is ever
   redirected by a Magic Mirror, the spell's or a creature's own (the Faerie Dragons' 20 %), in
   every profile. A creature's spell (a stack's cast of any kind) always meets the stack it was
   aimed at, with the tests it has today for its aimed stack. The flag and everything only it
   reaches go.
2. **BAT-QUIRK-2, `quirk.battle.attack_from_seen_pit` (BAT-Q30).** The original never offers an
   attack from a position with a hex on a Quicksand pit that the stack's side sees. **Intended:**
   such an attack is illegal in every profile, which is exactly what the engine does today with
   the flag off (`legal.rs`, `command.rs` and `control.rs` read the flag). The flag and everything
   only it reaches go.

## How a flag is retired (there is no precedent: decide inside this frame and say what you chose)

ADR-0007 fixes the registry: a closed list, a flag's index is its position, never reordered
(`crates/h3-content/src/quirks.rs`). So:

- Positions 10 and 14 stay in `h3_content::REGISTRY` as reserved entries that no profile can set
  and no code reads. `lookup` never returns a reserved position, so a profile that names a retired
  flag fails to compile with `CompileError::UnknownQuirk` (test it). Every other flag keeps its
  index (the registry's test already pins them).
- One simple form: an empty name with a comment that gives the retired flag's name, its quirk id,
  the date and the question, and `lookup` refusing the empty name. If you find a cleaner form with
  the same properties (stable positions, not settable, 100 % coverage), take it.
- The constants `ATTACK_FROM_SEEN_PIT` and `CREATURE_MIRROR_UNTESTED` and their re-exports go.
- `h3-pack`'s test `the_profiles_differ_exactly_by_the_quirk_flags` must still say what its name
  says: `faithful` sets exactly the flags that exist, `strict-fixed` none.

## Packs

- `packs/synthetic/pack.toml`: both flags leave both profiles (and the id lists in the comments).
- `crates/h3-harvest/src/lib.rs` writes the harvested packs' profiles: change it there, then
  regenerate the packs from your worktree (never edit `packs/base|ab|sod` by hand):
  `cargo run -p h3-harvest -- --install /home/claude/homm3/original-data/install --out packs`.
  The resulting diff must be the two profile lines of `packs/base/pack.toml` and nothing else; if
  anything else changes, stop and report.
- `crates/h3-pack/src/lib.rs` has test text that names the flags. The test profiles `t:pits` and
  `t:mirrored` in `crates/h3-battle/src/tests/support.rs` go.
- `packs/README.md` (the pack format's description) is yours if it needs a word. Pack format
  version: follow the precedent (was it bumped when a flag was added?) and say what you found; no
  bump if adding a flag needed none.

## Tests

- **Test first** where behaviour changes. BAT-QUIRK-9: a stack's cast at an enemy stack under a
  mirror that always reflects (and at a stack with an innate mirror) is never sent away, in
  `t:faithful` and in a profile without quirks: such a test fails today. A hero's cast is still
  redirected exactly as before (the existing tests of BAT-MAG-022 and BAT-MAG-057 stay green
  unchanged where they concern a hero's cast).
- BAT-QUIRK-2: the tests of the flag-on behaviour go or are restated. What stays pinned: such an
  attack is not listed and is refused with state untouched, in every profile; a berserk stack's
  automatic turn does not attack from a seen pit (today's flag-off branch of `control.rs`).
- `crates/h3-battle/src/tests/properties.rs`: `hidden_obstacles_do_not_show` draws one of five
  profiles since this review (BAT-QUIRK-1, BAT-QUIRK-2, BAT-QUIRK-3, none, all three): BAT-QUIRK-2
  leaves it. Run that property 20 times and once with `PROPTEST_CASES=960`. Change nothing else in
  that file: another worker's last two commits to it (the invariant property, the robustness
  test) are still to be joined.
- Coverage is gated at 100 % for `h3-battle`, `h3-content` and `h3-heroes`: leave no branch that
  only the retired flags reached, and give every new branch a fixed test.

## What else moves: report it, do not regenerate it

- Golden replays pin ruleset hashes, and the faithful profiles' hashes change (bits 10 and 14 are
  no longer set): `fixtures/replays/*.toml` (the battle's eight, `synthetic:faithful`) and
  `fixtures/replays/adventure/duel-s1-d28.toml` and `duel-s2-d7.toml` (`base:faithful`; they
  belong to the adventure lane). The coordinator regenerates them. Expected failures for this
  reason only: `golden_replays_reproduce_their_hashes` and
  `golden_adventure_replays_reproduce_their_hashes` in `h3-cli`. Report any other test that pins a
  ruleset hash or a profile's flags.
- Other lanes' paths (`h3-adventure`, `h3-game`, `h3-map`, `h3-pack-adventure`, `h3-protocol`,
  `h3-server`, `clients/`, `fixtures/replays/adventure/`, `benches/`): change nothing. If one of
  them fails to build or to pass its tests because of this change (other than the adventure
  replays above), stop and report: that is a cross-lane matter for the user.
- No version bump (state hash, ruleset hash version, replay format, encodings, C ABI).

## Where the common brief does not apply

- "No new quirk flag and no change to which profile has which flag": this list is that change,
  for these two flags only.
- `docs/` stays off limits (the catalog, the ledger, `installation.md`, `OPEN-QUESTIONS.md`, the
  epic files): write your wording proposals into the notes (rule id or file, old clause verbatim
  from this branch, new clause) for BAT-MAG-022, BAT-MAG-049, BAT-MAG-057, BAT-MAG-032, the
  ledger's BAT-QUIRK-2 and BAT-QUIRK-9, every `quirks = [...]` field that names one of the two,
  and every sentence of `installation.md` that says what the engine does about them.

## Closing steps

As the common brief, plus: `cargo test --workspace --no-fail-fast` once, so that every lane's
crates are built and tested against the change (the two golden tests fail as said); the battle
crate's tests three times; `cargo xtask check-arch`, `check-rules`, `check-stubs`, run with your
own target directory (`/home/claude/wt-a/target`; never a directory another worktree uses: xtask
takes the workspace root from its build). Keep `target` below 7 GB (11 GB of disk are free for
everybody; nobody else builds heavily right now, but property runs use one core until about 21:15).

Your final message, at most 20 lines: each of the two retirements with done / not done and commit
ids; how the registry keeps the positions; the pack format precedent; the tests that fail for the
expected reason and any that fail for another; anything in another lane's paths that this change
touches; open points.
