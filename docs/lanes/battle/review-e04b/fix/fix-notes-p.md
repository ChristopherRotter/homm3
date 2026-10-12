# Fix notes of worker `p` (E04b T11b, second round, list P)

Worktree `/home/claude/wt-b`, branch `battle/e04b-t11b-p`, base `64fe5cf`. Logs: `<SCRATCH>/wp/`.
No engine code is changed by this list: only `crates/h3-battle/src/tests/properties.rs` (P1 to P5)
and `crates/h3-pack/tests/harvested_heroes.rs` (P5).

Status (kept current; a restart continues from here):

- **Second worker `p` since the restart of 2026-10-12 18:00** (the first one's work: everything
  above "P1 reviewed"). Helper scripts in `wp/`: `build.sh` (test binary into `wp/bin/<tag>`),
  `runprop.sh`, `runall.sh` (20 runs of each property and one with ten times its cases, one test
  thread), `mutate.sh` + `mut/` (mutation checks: one engine file edited, built, always restored),
  `with-throwaway.sh` (a statistics harness appended for one build and stripped again).
- **Where the work is (19:55):** P1 is committed and pushed (`c641ef1`). The test file of each
  later item's end is a full file in `wp/stage/`: `S2.rs` (P2), `S3.rs` (P3), `S4.rs` (P4),
  `S5.rs` (P5; the worktree has it); each is formatted and passes
  `cargo clippy -p h3-battle --all-targets -- -D warnings`; its test binary is `wp/bin/s2` to
  `s5` (rebuilt 19:51 after the last addition to P2, "the limits are obeyed"). A commit of item
  N is `cp wp/stage/SN.rs` over `crates/h3-battle/src/tests/properties.rs`, nothing else, with
  the message in `wp/msg/pN.txt`.
- **Runs:** stage 1 (P1) passed. Stages 2, 3, 4 are to run (`wp/runall.sh`, one test thread per
  stream; logs `wp/s<N>-<property>.log` and `-10x.log`, `wp/s<N>-done.flag` at the end). Nothing
  is committed before its stage's runs passed.
- P2, P3, P4, P5: written, tried, mutation-checked (below); P2 to P4 await their stage's runs,
  P5 is a fixed-seed test (no runs to wait for) and is committed after P4.

## Before anything (baseline at `64fe5cf`)

- `cargo test -p h3-battle`: 479 tests, 77.7 s (`wp/base-test-1.log`; the binary was built before).
- Each property alone, one test thread (`wp/base-props-time.log`): `every_step_preserves_the_invariants`
  95.5 s in that run (another worker was building; 77 s inside the whole suite), every other
  property 0.2 to 3.9 s. The suite's wall time is this one property's: two test threads, and the
  rest together takes about 20 s on the other.
- Where its time went (a throwaway harness over 96 generated games, `wp/p1-stats-probe-1.log`,
  with Berserk already in the book): 22.5 s for the listed legal commands (370 000 clone-and-apply
  probes), 11.3 s each for `nova`, `ring` and (new) `berserk` (a hex spell has 165 targets, each
  applied to a clone at every step), 6.4 s for Teleport (669 000 probes), 2 s each for the two
  wall spells, under 0.5 s for each other spell.
- `PROPTEST_CASES` does override the `cases` of `proptest_config` (proptest 1.11's `proptest!`
  passes the given config through `contextualize_config`), so the long runs below use it.
  `docs/05-testing.md` does not name the variable; `docs/lanes/server/HANDOFF.md` does.
- The ten saved seeds of `proptest-regressions/tests/properties.txt` no longer reproduce the
  cases they were saved for once the strategies change (a `cc` line is the seed of the whole
  strategy): they still run, as ten more random cases per property. The file is unchanged.

## P1. Controllers that differ from sides — in progress (not committed yet)

What is written (all in `tests/properties.rs`; the source as built for the runs is saved in
`wp/snap/properties-p1c.rs`, with a throwaway block at its end that is not committed):

- Generators. The heroes' book has `t:berserk`, `t:hypnotize`, `t:blind` (38 spells). A new step
  kind `Control(n)` (weight 1 of 11; `Legal` 6, `Arbitrary` 2, `Cast` 2) casts the n-th castable
  one of the three at its n-th target, Berserk at a hex a living stack stands on. Power and
  masteries from bits of the setup word rotated by 29: the attacker's hero has power 10 instead
  of 2 one time in four, the defender's 6 instead of 1, and the defender's hero has air and fire
  masteries (it had none). Seeds near 0 keep the old values.
- `arb_command` (sec F-4's fourth point): `Shoot`, any spell of the ruleset as a cast (it named
  only the first 12 of 61), stack ids up to 43 and sometimes any byte, a stack or a hex as a
  cast's second target.
- What a run of 192 generated games plays (`wp/p1-stats-1.log`, a throwaway harness with a fixed
  seed): 832 hero casts, of them 146 Berserk (101 stacks made berserk), 145 Blind, 40 Hypnotize;
  153 accepted commands for a hypnotized stack (9 strikes, 10 hero casts in its turn); 100
  automatic turns of berserk stacks with 93 strikes and 59 moves; 16 casts that made the active
  stack berserk. So one run of a 96-case property has about 75 commands of a controller that is
  not the stack's side and about 50 automatic berserk turns.
- Assertions restated in `every_step_preserves_the_invariants`:
  - exactly the side that controls the active stack has legal commands (was: "the opponent has
    none or it is its turn");
  - a `SpellCast` event appears exactly for an accepted `Cast`, with the actor's side;
  - a cast keeps the turn unless it killed the active stack **or made it berserk** (BAT-MAG-025);
  - one walk over the command's events replaces "the mover's strikes hit the commanded target,
    every other strike is a retaliation at the mover or a war machine's": the acting stack is the
    mover until a `TurnStarted`; a stack moves and strikes on its own turn only; the commanded
    strikes hit the commanded target, which is never of the actor's side; a retaliation is by the
    last struck stack at the acting stack; a turn after the command is automatic, a war
    machine's (its target of the other side) or a berserk stack's (any other stack); `Teleported`
    only by the actor's cast at a stack of the actor's side;
  - good morale per `GoodMorale` event (was: the mover's): the stack takes the turn (the next
    event is its `TurnStarted`, not followed by its `Frightened`) and then, unless it is still
    berserk, it is the active stack at the command's end and no round started after;
  - "the actor cannot move enemy stacks" became "a stack's hex changes only with a `Moved` or
    `Teleported` event that names it" (the walk above says when those may occur);
  - an enemy stack's count and hit points change only with an event that names it: `Reflected`
    and `LifeDrained` added (a stack of the other side that strikes on the actor's command while
    hypnotized, or on its own while berserk);
  - new state invariants: `Stack::controller()` is the other side exactly while a spell with the
    Hypnotize change is in the effect list for the stack (read from the list, not from the
    stack); `side_to_act()` is the active stack's controller by that rule; the stack that awaits a
    command is not berserk.
- `observations_do_not_depend_on_hidden_state`: the drawn spell is shown exactly to the side
  that commands the active stack, and only that side has cast candidates.
- `hidden_obstacles_do_not_show`: `t:berserk` is taken out of both books, with a comment that
  names BAT-Q31.
- **A change of how the invariant property probes casts (to pay for Berserk's 165 targets).**
  Every listed cast (spell, target, second target) is checked with `validate` at every step (the
  crate's public entry for exactly this, the validation `apply` runs before it changes anything);
  it is applied to a clone as before unless it is aimed at a hex: of those (the area spells, the
  wall spells, Remove Obstacle, Teleport's places) every second hex is applied, alternating with
  the step. Before, every one was applied at every step. Legal commands and creature casts are all
  applied as before.

### BAT-Q31, confirmed by runs (not a new finding; the exception P3 names)

- The security reviewer's sketch for F-sec-1 part a, run as a throwaway test (`zz_berserk_pits`,
  not committed; output in these notes only): the defender's berserk `fixed` at (8,5), the
  attacker's at (4,5), the defender's own hidden pits at (6,5) and (5,5).
  `t:faithful`: with the pits it ends on hex 106 and strikes from there, without them on hex 90:
  the attacker, who sees no pit, is told another `Moved`. `t:pits` (BAT-QUIRK-2): with the pits it
  walks to hex 90, stops there and does not strike; without them it strikes: the attacker reads
  the pit's hex. Exactly as the reviewer traced by hand.
- The property itself with Berserk in the book: 30 runs of 96 cases passed; of 10 runs of 960
  cases one failed (`wp/p1-hid-berserk-10x-fail-9.log`, seed line
  `cc c42b19a96a87c7b2cc77144d9e5378b8d3d1fcc53a2d1281be4ae4206a13a62f`, reproducible only with
  the binary `wp/bin/p1a`). There the berserk stack of the other side makes the same move and
  strikes the same target in both worlds, but its strike's damage differs (4 against 1): the
  number of nearest candidates differs with its side's hidden pits, so the tie draw
  (`control.rs:84`) happens in one world only and every later roll moves. A third form of the
  same exception, besides the two the reviewers describe (another position, no strike).
  With Berserk in this property's book the suite would fail about once in a hundred runs.

## P1 reviewed (second worker `p`, after the restart of 18:00)

Baseline measured again at a quiet moment, the test binary of `64fe5cf` with two test threads
(`wp/bin/base`, `wp/base2-suite-1.log`): **479 tests in 72.2 s**; the invariant property is the
last to finish and decides the wall time (the other 478 tests take about 20 s of the second
thread).

I read the first worker's diff line by line against BAT-MAG-025, BAT-MAG-026, item A10 and the
engine (`turn.rs`, `control.rs`, `command.rs`, `legal.rs`, `combat.rs`, `war.rs`, `effects.rs`,
`observe.rs`). The generators, the legality assertions (now "exactly the side that controls the
active stack has commands"), the `SpellCast` assertion, the hex check and the state invariants
were right. Four assertions said less than the rules, and I restated them; the throwaway block
(`zz_stats`, `zz_berserk_pits`) is out of the file (kept as `wp/throwaway-*.rs`).

- **Weaker than the assertion it replaced: good morale.** The old assertion was "after
  `GoodMorale` the mover is active again unless it lost its turns or Fear froze it". The first
  worker's version asked nothing unless a `TurnStarted` for the stack followed: a turn that
  silently went missing would have passed. Now, per `GoodMorale` event: the stack is the one that
  just acted; `TurnStarted` for it follows **exactly when it does not lose its turns** (BAT-MOR-003,
  BAT-MAG-024: blind, petrified or paralysed by a retaliation's spell, read from the events up to
  that point, not from the state at the command's end, which a later automatic strike may have
  changed); then as before (Fear, the command ends there unless the stack is still berserk, no new
  round, the round's mark).
- **An escape, not a statement: "or made it berserk".** `enraged` only excused a cast after which
  the active stack changed. Now a berserk stack whose turn starts (`TurnStarted`, or the cast
  that makes the active stack berserk) **must act or pass before the next turn starts or the
  command ends**: a `Moved` or a strike of its own, `TurnPassed`, `BadMorale`, `Frightened`, or
  its Berserk ending first (BAT-MAG-025: "takes its automatic turn at once", "with no stack to
  attack, or unable to move, its turn passes").
- **A hole in "who strikes":** a commanded stack that struck without a strike command passed if
  it was a war machine hitting the other side. Now: a berserk stack strikes any other stack; a
  commanded one only the commanded target, of a side that is not the actor's; any other striker
  must be a war machine on its own turn hitting the other side. `Moved` likewise: berserk, or the
  commanded stack under a `Move` or `Attack` command.
- **The model of "who is berserk" was never checked against the state.** The walk follows the
  timed effects by the events alone (`EffectApplied`, `EffectEnded`, `Reborn`, `StackCreated`,
  `RoseFromCorpse`); at the command's end what the events told must equal the state's effect list
  (a dead stack keeps only the effects without a duration: BAT-MAG-009, BAT-MAG-010). This is a
  new invariant of its own: the public events tell every change of the effect list but a death's.
- Added, cheap: `TurnPassed` only for the stack whose turn it is, which loses its turns or is
  berserk; `BadMorale` and `Frightened` only at an automatic turn start; `Waited`, `Defended`,
  `StackCast`, `CreatureRaise` only by the actor's command (an automatic turn does none).
- `controller_by_rule` (the effect list, not the stack's status bit) is now also what the
  observation property uses for "the side that commands"; the awaiting stack's "not berserk"
  reads the effect list too.

**The restated assertions bite** (mutation checks: one engine line changed, the test binary built
in `wp/bin/`, the engine file restored by the script; nothing of it is committed):

| mutation | caught by | after |
|---|---|---|
| `command.rs`: no `turn::berserk_now` after a cast | "a berserk stack's turn left no trace" (end of the command) | 0.8 s |
| `turn.rs` `begins`: a berserk stack's turn is skipped and marked acted | the same, at the next `TurnStarted` | 0.1 s |
| `effects.rs` `attacked`: Berserk ends without its `EffectEnded` | the events' effect list against the state's | 0.8 s |
| `turn.rs` `morale_turn`: a stack that loses its turns takes its good-morale turn | `TurnStarted` follows exactly when it does not lose its turns | 54 s |
| `state.rs` `may_attack` by side instead of controller | "a commanded strike hits no stack of the actor's side" | 15 s |

What a run plays now (`wp/p1-stats-2.log`, 960 generated games, a fixed seed; per run of 96 cases a
tenth of it): 450 accepted commands for a hypnotized stack (45 strikes, 44 hero casts during its
turn), 501 automatic turns of berserk stacks (476 strikes, 292 moves), 89 casts that made the
active stack berserk, 3 good-morale turns of a berserk stack, 9 good-morale rolls of a stack that
had lost its turns.

### Observation P1-F1 (not asserted, nothing left out): a stack that loses its turns while it awaits a command keeps that turn

- Seen by the statistics harness: in 5 of 79 287 steps the stack that awaits a command is blind.
  Two ways, both confirmed by a directed throwaway test (`wp/throwaway-blind.rs`,
  `wp/p1-f1-blind.log`; profile `t:faithful`):
  - (a) the attacker hypnotizes the defender's stack 1 (5 `fixed`); on that stack's turn in round
    2, which the attacker commands, the attacker's hero casts Blind at it (an enemy stack by its
    side: BAT-MAG-026). The stack is blind, stays the active stack with 95 legal commands, and
    `Attack { target: 2 }` is accepted: a blind stack strikes (and is retaliated against, which
    ends its Blind).
  - (b) the attacker's hero casts Blind at the defender's `pixie` (Magic Mirror 100 %); the mirror
    sends it to the attacker's own active stack (BAT-MAG-022). It is blind, stays active with 86
    legal commands and walks from hex 89 to 91.
- The rule: BAT-MAG-024 says "While active the stack has no turns: when its turn comes in the
  order its turn starts and passes". It says nothing about a stack that is blinded during its own
  turn (BAT-MAG-043 shares the mechanism).
- The original, by my reading (own words): the action dispatcher's case for a hero's cast
  (0x479036, the cast at 0x479056) ends the active stack's turn only when the stack has no
  creatures left after the cast (the test at 0x47905B); the battle loop re-tests the active stack
  only for Berserk (0x4745DF); the Blind duration (+0x290) is read by the pick (0x465222), the
  melee routine, the return, the Enchanter and the battle AI, but neither by the cursor
  (0x476490) nor by the input handler (0x474D80) nor elsewhere in the dispatcher. So a stack
  blinded on its own turn keeps the turn and can be commanded there too.
- Judgement: **no defect; a gap in the rule's text** (and perhaps a quirk to classify: "a blind
  stack acts" is the original's behaviour, reachable in normal play through Hypnotize or Magic
  Mirror). Proposed sentence for BAT-MAG-024: "A stack that becomes blind while it awaits a
  command keeps that turn and may act in it (its controller's hero blinds the hypnotized stack it
  commands, or a Magic Mirror sends the hero's Blind to its own active stack): the original ends
  the active stack's turn after a hero's cast only when the stack is dead (0x47905B) and re-tests
  only Berserk (0x4745DF)." No directed rule test exists for it (`tests/control.rs` or
  `tests/timed.rs` would be the place: worker `c`'s files). An invariant "the stack that awaits a
  command does not lose its turns" would be wrong and is not in the property.

### P1, amended twice after its first runs (both in the invariant property; both in every later stage)

- **A wrong old assertion, reachable through P1's generators: two rebirths in one command.** The
  assertion "a stack's casts change only by its own cast or by one rebirth" (`was == is + 1`)
  assumed one death per command. With Berserk two automatic turns may kill the same stack one
  after the other. Found by chance in a mutation run (case 53 of a run of the stage-4 binary),
  replayed from the printed input (`wp/replay.py` turns a "minimal failing input" into a throwaway
  test): the attacker's hero casts Expert Berserk at hex 87 on its own active `wight` (stack 1),
  its `firebird` (stack 2, 4 creatures, two casts, rebirth by thirds) and its `wide` (stack 3). The
  wight strikes at once, kills the four Firebirds, `Reborn { stack: 2, creatures: 1 }` (casts
  2 → 1); `TurnStarted { stack: 3 }`, the berserk `wide` kills the one, `Reborn` again (1 → 0).
  The engine is right (BAT-MAG-059: after every action that may kill, a stack that died in it
  with a cast left uses one). Restated: the casts used are the stack's own cast, one for each
  `Reborn` event, and at most one more if it lies dead (a draw of none). In 27 000 generated
  games of my statistics harness the case did not come up once; it is rare, so the old assertion
  would have failed the suite about once in several hundred runs.
- **Added: its own attack ends a stack's Berserk** (BAT-MAG-025; the first version checked only
  that a berserk stack acts). A berserk stack that struck must no longer be berserk when the next
  turn starts or the command ends, unless it died. And as state invariants: no stack is berserk
  and hypnotized at once (each ends the other), and no war machine is either.
  Mutation check: `effects::attacked` never ends an effect → "attacked and is still berserk"
  after 4 s.

## P2. The heroes' hidden magic in force — done (commit below)

- **Generators.** A generated game has a sixth part, the hidden magic of each hero (`Hidden`,
  `arb_hidden`): half the heroes have none; the others have each part with a chance of its own:
  `NoCasting` 10 %, `SpellLevelLimit` 1 to 3 20 %, a `SpellImmunity` 30 % (half of them to one
  of Berserk, Hypnotize, Blind: the Pendants), `SpellLevelImmunity` 1 or 2 20 %, `MindImmunity`
  20 %, `NegateImmunities` 20 %, one or two opening spells 33 % (any spell and mastery
  `magic::opens_with` accepts, power 1 to 3), Eagle Eye 30 % (chance 0, 40 or 100 %, a random
  part of the book). `game_setup` = `make_setup` plus `conceal`; all seven properties over
  `arb_game` use it (`make_setup` alone stays for the three fixed-seed tests, whose games do not
  change). Explicit parts, not bits of the seed: a failing case shrinks to "no hidden magic"
  when it does not need any.
- What 960 generated games have (`wp/p2-stats-1.log`): 1 469 heroes; no casting 71, level limit
  129, spell immunity 223, level immunity 120, mind immunity 150, negation 153, opening spells
  225 (160 `OpeningSpell` events), Eagle Eye 224. P1's numbers stay: 509 commands for a
  hypnotized stack, 394 berserk turns.
- **The invariant property** needed no change to pass: legal casts and `apply` agree and a refusal
  changes nothing with all of them in force. **Added: the magic in force is obeyed**, read from
  the setup (`Limits::of`) and held against each command's events, so by nothing the engine
  derives:
  - BAT-MAG-061: while a hero forbids casting no spell is castable, and no `SpellCast`,
    `PeriodicCast`, `CreatureRaise` or `StackCast` comes, but a `StackCast` of a creature with
    `ignores_magic_block`; opening spells and the strikes' spells are no casts.
  - BAT-MAG-062: no spell above the smallest level limit is castable, cast (`SpellCast`,
    `StackCast`, `PeriodicCast`, `OpeningSpell`), placed (`ObstaclePlaced`) or takes effect
    (`EffectApplied`).
  - BAT-MAG-063 to 066, at every `EffectApplied`: the spell is not one the controlling hero's
    `SpellImmunity` names (no exemption for the spells that skip immunities: the rule says
    Dispel is stopped too); it is not up to that hero's `SpellLevelImmunity` unless it skips the
    immunities (`ignores_immunity`, BAT-MAG-016); it is no mind spell if that hero has
    `MindImmunity`, unless it skips the immunities or the hero of the side that cast or the
    controlling hero negates them (so the other side's negation does not count for a side's
    spell at a stack it controls). "Controlling hero": of the side that controlled the stack
    when the spell was cast, by the walk's own effect list at the last `SpellCast`, `StackCast`,
    `PeriodicCast`, `OpeningSpell` or `StrikeSpell` event (a stack casts for the side that
    controls it). The one exemption from the level limit and the spell immunity is what a strike
    puts on a stack without a test (an `ignores_immunity` spell right after its `StrikeSpell`
    at that stack: Bind, Acid Breath; BAT-MAG-045/046, "ask nothing").
  - First written with one exemption for both immunities (`ignores_immunity`); restated as
    above after reading BAT-MAG-063 again: the rule exempts only the strike's untested spells
    from the named immunity. In the test pack the difference cannot show (the only spells that
    skip immunities and leave an effect are the strike's `t:tether` and `t:acid`).
  - Not asserted: the creatures' own immunities and their negation (BAT-MAG-005; rule tests
    hold them), and the level limit for an Archangel's raise (no spell in its event).
  - Mutation checks (960 to 1 500 cases asked, stopped at the first failure): a creature casts
    although a hero forbids it (`casters::ready` without the block) → "a cast while a hero
    forbids casting" (54 s, about 110 cases); `Guard::of` reading the defences of the stack's
    side instead of its controller's → "is immune to mind spells" (190 s, about 400 cases);
    `affects` without the named immunity → "is immune to #39" (157 s, about 330 cases);
    `affects` without the level immunity → "is immune to the level of #12" (5 s);
    `Guard::of` counting the other side's negation too for a side's spell at a stack it
    controls → "is immune to mind spells" (1 094 s of a long run: a rare case; the result was
    read from `wp/mut/run-m18.log` by the coordinator after the worker was interrupted).
- **`observations_do_not_depend_on_hidden_state`** changed the other hero's private parts in a
  copy and kept the battle's derived limits, with a comment that "the rules read the limits and
  the defences through `BattleState::limits`, which the twin keeps". That was only true while no
  hero had defences: `magic::Guard::of` reads the controlling hero's `defences` and `negates`
  directly once `limits.guards` is set. Now two copies per side:
  1. what no rule reads at the state (tactics, damage, machine control, funds, book, masteries,
     bonuses, resistance, duration, opening spells, Eagle Eye, the drawn spell): the other side's
     and the spectator's observations are equal, as before;
  2. on top, the hidden magic in force: `no_casting`, the level limit, the defences and the
     negation change **and the limits are derived again**, so the rules do read the change: the
     other side's observation is equal but for `cast_candidates`; the spectator's is equal.
- **`cast_candidates` is BAT-Q26's "casts its commands may not make", inside an observation.**
  Without the exemption the property fails in 11 of 12 runs of 960 cases (tried in a scratch
  build): the viewer's Master Genie has candidates `[.., 6, 7, ..]` and none in the copy where the
  other hero forbids casting. The security review classed this as allowed ("same class, not
  through a refused cast"); I followed it. For the coordinator: it means an agent reads the other
  hero's Orb of Inhibition, Recanter's Cloak or negation off its own observation as soon as it
  commands a helpful caster, without issuing a command.
- **New property `a_heros_eagle_eye_and_unopened_spells_do_not_show`** (96 cases, 2.3 s): twin
  games, the same commands in both.
  - Eagle Eye (BAT-MAG-069) at chance 0 against 100 % (no draw either way), every spell of the
    book: every command has the same result and **every** event is the same (nobody is told
    during the battle); the other side's and the spectator's observations are the same after
    every command; its own side's differ in the picks only; at the end the hero with 100 % has
    picked exactly the spells of the other hero's `SpellCast` events, the one with 0 none.
  - Opening spells (BAT-MAG-068), with against without: until the hero's `OpeningSpell` event
    every result and every event is the same and the other side and the spectator observe the
    same (also right after the start, and for the whole game when no opening spell affects a
    stack: the engine skips those without an event).
- Mutation checks: an extra RNG draw in `eagle_eye` at 100 % → the twin games' events differ
  (0.1 s); `observe_as` mixing the other hero's `opening_due` into `wait_phase` → the opening
  twins' observations differ (0.2 s); `observe_as` mixing `limits.guards` into `wait_phase` →
  copy 2 of the observation property (0.2 s; copy 1 does not see it).
- Not done, and why: twin games for the casting limits, the defences and the negation. They
  change what the other side's own casts may do and achieve from the first cast on (refusals,
  `SpellResisted`, a mass spell passing over a stack, a strike's spell failing): all of that is
  what BAT-Q26 lets the other side infer, so two such games may differ in everything public and
  there is no window in which they must be equal.

## P3. Hidden obstacles with everything that walks, casts or aims — done (commit below)

- `hidden_obstacles_do_not_show` draws one of five profiles (was three, by two flags):
  `t:sight` (BAT-QUIRK-1), **`t:pits` (BAT-QUIRK-2)**, `t:anchors` (BAT-QUIRK-3), `t:quirky`
  (none of the three) and **the three together**, as `faithful` has them. The test pack has one
  quirk per profile, so `lively_mix` compiles the pack with a profile of its own
  (`support::ruleset_from` with a patch that adds profile `t:mix`); no change to `support.rs`.
- **The active stack's cast**: `creature_cast_targets` must be the same with and without the
  obstacles the viewer does not see, and each target's `CreatureCast` is applied in both worlds
  and compared like the other commands (events told to the viewer, the spectator's, the
  observation after).
- Berserk stays out of the heroes' books in this one property, with the comment that names
  BAT-Q31 (the first worker's; unchanged).
- Mutation check: `magic::creature_targets` leaving out a stack that stands on any pit → the
  new target comparison differs after 17 s (about 500 cases: the viewer must command a caster
  while a target stands on a pit it does not see). No mutation found for BAT-QUIRK-2 alone: its
  only reader on an automatic turn is the berserk stack, which is BAT-Q31's.
- No failure in 30 runs of 96 and 6 runs of 960 cases before the stage runs.

## P4. The battle's end as an invariant — done (commit below)

- **The invariant**, at every step of `every_step_preserves_the_invariants`: while the battle
  goes on each side has a living stack that is no war machine; once it ended without a side
  leaving it, a side has none, and the result is `Victory` of the other side exactly if a stack of
  its army lives (not only created ones), else `Draw` (BAT-END-001, BAT-END-002, BAT-MAG-037).
  And in the accepted branch: the result is `Retreat` or `Surrender` of a side exactly when the
  command was one, and then of the trusted actor's side (BAT-FLE-001/002; this is the assertion
  F-sec-3 would have failed).
- **The profile.** Half the games (a flag `own_clone`) run `lively_mix` with Q-0003 and
  **BAT-QUIRK-7**. That alone changed nothing: in 1 920 generated games of that profile not one
  stack became its own clone (`wp/` statistics, `TurnedSummoned` 0), because the clone's search
  finds a free place long before the original's own hexes. So two generator changes:
  - `crowd`: in those games a living two-hex stack that is no war machine (of a side with a hero
    if there is one) is walled in before the first step: every free hex up to two hexes from it
    is blocked (`NaturalObstacles::block`, the tests' own hook) but the one ahead of its head;
  - a step kind `Clone` (1 of 13) casts Clone at a two-hex target if it may.
  Now in 1 920 such games: 673 have such a stack, **119 `TurnedSummoned`**, 74 of those stacks
  vanish when their rounds run out, **5 times as their side's last stack, and the battle is over
  right then** (r4a F-3's case). Per run of the property (48 such games): 3 stacks become their
  own clones, and F-3's case comes up about once in eight runs.
- **A wrong old assertion found by it:** "exactly the living clones have rounds left"
  (`clone_rounds > 0` ⇔ alive with trait CLONE) fails for a stack that became its own clone: it
  has the clone's rounds without the trait (BAT-MAG-038). Restated: rounds are left to exactly
  the living clones and the living stacks linked to themselves, which exist only with the quirk,
  are created and no clones; any other link joins a living clone and its living original, both
  ways.
- Mutation checks: `created::turn` giving the turned stack no rounds → the restated assertion
  (18 s). `turn::start_round` without its `check_end` (A6's fix taken out) and `check_end`
  counting created stacks as an army: see "Runs" below.

## P5. The two bounded robustness tests — done (commit below)

- `extreme_casting_heroes_play_without_failures` (h3-battle): the bound "natural obstacles
  anchored on their own hex" (F-robust-2, fixed by A2) is out. Its profiles had no BAT-QUIRK-3, so
  the bound never did anything there and taking it out alone would not either. Now: the
  exact-rates half runs `lively_mix` with Q-0003 and **BAT-QUIRK-3** (was `t:quirky`), the
  other half `t:original` as before; the obstacles are a random half of eight one-hex pieces of
  every kind of `pieces`, three pairs side by side whose right piece is anchored on the left
  one's hex (F-robust-2's layout), one without an anchor; and `play_casting` lets one hero cast
  in four be the spell that removes obstacles while it is castable, so that the removals do
  leave pieces no hex leads to. Still 200 games, 2.7 s (was 2.7 s), 1 121 hero casts and 369
  stack casts (was 1 217 and 373).
  Mutation check: `NaturalObstacles::any_removable` as before A2 → "castable #47 without a
  target" after 0.2 s; the test as it was (and the widened obstacles without the agent's
  removals) passes on that mutant.
- `random_legal_sod_heroes_cast_and_play` (h3-pack): **nothing to change.** Its one bound was the
  helper `apart` (generated obstacles that share a hex dropped, F-robust-2), which A2's commit
  `c010257` took out; it never had a bound for F-robust-3 (a derived hero's rates stop at
  65 535 bp). Its doc comments and the module's say what it does (both profiles, every generated
  obstacle, armies up to `u32::MAX`); its counts are 2 083 hero casts and 230 stack casts against
  a comment of "about two thousand, two hundred" (the first worker's log
  `wp/p5-harvested-base.log`). The file is untouched.
