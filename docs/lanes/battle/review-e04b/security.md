# Review security

E04b (battle magic) against `docs/06-security.md`, at commit `4cd7e2e`. Read only: nothing was
built or run, every claim below is a code reading; where a run would settle it the item says
`needs-run`. Paths are relative to `crates/` unless they start with `docs/` or `packs/`.

Short result: the commands and the per-viewer projections hold up. Validation is one pure
function over `&BattleState`, every reference is checked, no error and no legal list reads the
other side's hidden obstacles. One real leak in two places (F-sec-1: the two automatic turns
that plan with pits), one refusal oracle in the sim layer (F-sec-2), one unreachable validation
slip (F-sec-3), the test gap they went through (F-sec-4) and one residual side channel (F-sec-5).

## What is hidden from whom (table: fact | hidden from | where it lives in the state | ways out checked)

| fact | hidden from | where it lives | ways out checked |
|---|---|---|---|
| RNG state, seed | everyone | `BattleState.rng` (`h3-battle/src/state.rs:826`) | Not in `Observation` (`observe.rs:55-141`); no event; no hash or seed in `h3-protocol/src/wire.rs` (only `ruleset_hash`, 925/952); the state hash covers it and stays with the host (`state.rs:1279`; C ABI `h3_env_state_hash` is the trusted local ABI). Residual: F-sec-5. |
| Spell book contents, masteries, spell bonuses | other side, spectators | `HeroMagic.{book,masteries,bonuses}` (`magic.rs:283-289`) | `observe_as`: only in `own_hero` (`observe.rs:209-211`, `220-250`), none for a spectator. Flat encoding and wire take the projected `Observation` only (`h3-sim/src/observation.rs:305`, `h3-protocol/src/map/mod.rs:130-132`). Legal lists take the actor and are empty off turn (`magic.rs:1539-1541`, `1554`, `1586`, `1669`). Off-turn errors: `NotYourTurn` first (`command.rs:190-192`). Summon templates built from the book (`state.rs:376-417`) are in no observation. Events: a cast names its spell, and `SpellDamage.value/caster` (`magic.rs:2019-2025`) give that cast's mastery effect and bonus factor: public by ADR-0014 point 8 ("each cast with its spell, targets and results"). No other way out found. |
| Mana, casts this round, "carries a spell book" | nobody (public, ADR-0014 pt 8, BAT-Q12) | `HeroView` (`hero.rs:30-54`) | as designed |
| Hidden magic of T9a: `no_casting`, `level_limit`, `defences`, `negates`, `resistance`, `duration_bonus` | other side, spectators; "the other side infers it from the casts it may not make" (BAT-Q26) | `HeroMagic` (`magic.rs:290-324`), derived `BattleState.limits` (`magic.rs:353-374`) | `observe_as`: `own_hero` only. Shows, as BAT-Q26 allows, through the acting side's own casts: `CastingForbidden`, `SpellLevelBarred`, `InvalidSpellTarget`, `CannotCast` (`magic.rs:619-625`, `1193`; `casters.rs:391-399`), `castable_spells`/`cast_targets` (via `Guard::of`, `magic.rs:676-691`, and `share`, `1893-1914`). Same class, not through a refused cast: `SpellResisted`, a mass cast passing over a stack, the Enchanter's choice (`casters.rs:354-377`), a mine that does not go off (`magic::hurts`, `2431-2436`), `cast_candidates` for an own stack the enemy hypnotized (`casters.rs:498-518` → `useful` → `Guard::of` reads the controller's hero). Nothing beyond that class found. |
| Opening spells | other side until cast | `HeroMagic.opening`, `opening_due` | `own_hero` only; public `OpeningSpell` when cast (`magic.rs:959-962`); a spell that affects no stack is skipped without an event (`956-958`). |
| Eagle Eye: chance, spells, picks | other side | `HeroMagic.eagle_chance/eagle_spells/learned` | `own_hero` only; no event (`magic.rs:991-1008`); one hidden draw per enemy hero cast when the chance is 1..=99 (`1000-1004`), before the cast's own draws; `outcome().heroes[].learned` (`state.rs:1249-1254`) goes to the orchestrator, the wire's `GameEnded` carries the result only (`h3-protocol/src/session.rs:934-938`). |
| A Faerie Dragon's drawn spell | the side that does not control it, spectators, until cast | `BattleState.casts.drawn` (`casters.rs:163`) | `Observation.drawn` only for the controller (`observe.rs:192-196`); `SpellDrawn` is `Visibility::Side(controller)` (`casters.rs:250-257`); sim masks need the controller (`ready`, `casters.rs:397`; `h3-sim/src/action.rs:373-378`, `415`). **Leaks through `h3_sim::action::decode` → `BattleEnv::step` → `h3_env_step`: F-sec-2.** Wire: not reachable (typed commands, off turn `NotYourTurn`). |
| Quicksand pits, Land Mines | the other side unless revealed or seen by natives (BAT-QUIRK-1); a spectator unless both sides see | `BattleState.spell_obstacles` (`obstacles.rs:75-86`) | Observation: `obstacles::observed` (`268-279`). Events: `ObstaclePlaced/Removed/Triggered` by `visibility` (`257-263`, `445-452`, `481-492`, `815-822`), `ObstacleRevealed` public. Legality: walkers' reach uses `seen_pits(controller)` (`movement.rs:109`), obstacle spells `seen_hexes(side)` (`obstacles.rs:350-360`), Remove Obstacle `sees(side)` (`529-539`), Teleport, Clone, Summon and raises `blocked_for` (walls only, `state.rs:1049-1057`). Mines never enter legality or any plan. Events of a hidden obstacle that does nothing: none, no split `Moved` (`acts_on`, `613-616`). Every read of the list is in `obstacles.rs` or `blocked_for` (grep). Results: a walker is stopped only by a pit it runs into, which is then revealed (`stuck`, `799-824`), as the original; random draws avoid the other side's hidden objects under BAT-QUIRK-4, which `faithful` has on (`obstacles.rs:378-382`, `packs/base/pack.toml:7082`): a decided leak (BAT-Q4), also the original's. **The two rule-driven readers of `seen_pits`, a berserk walker's turn and the automatic Ballista's target: F-sec-1.** |
| Master Genie's draw counts | shown to the commanding side only | computed, `casters::candidates` | zero for the other side and a spectator (`observe.rs:198-200`). |

Events. Four emit sites are not public, all of E04b: `SpellDrawn` (`casters.rs:251`) and the
obstacle events at `obstacles.rs:445`, `486`, `815`. Every other event E04b added is public, and
`project_as` drops or passes, it never redacts (`observe.rs:259-265`). I read each public kind
for a hidden count, id, position or draw: `Moved` is not split by a hidden obstacle that does
nothing (`acts_on`), `SpellCast` for Quicksand or Land Mine carries no hex and no count,
`Teleported` and `StackCreated` use no hidden obstacle for their places. The public events that
imply a hidden fact are those of F-sec-1: a berserk stack's `Moved`/`Struck` and an automatic
Ballista's `Struck`.

## Commands (table: command | intent only | actor check | references | validated before change and draw | errors)

| command | intent only | actor check | references | validated before change and draw | errors |
|---|---|---|---|---|---|
| `Cast { spell, target, second }` (hero) | Yes. Cost, mastery, value, rounds, power, bonuses come from the actor's `HeroRecord` (`magic.rs:1247-1273`); wire and sim carry the same three fields (`h3-protocol/src/wire.rs:126-133`, `h3-sim/src/action.rs:288-297`). | Active stack's controller is the actor (`command.rs:190-192`), so a hypnotized stack's turn is its controller's and a berserk one never waits; hero is `state.heroes[actor]` (`magic.rs:606`); tactics phase refused (`command.rs:378-379`). | Spell: in the book and the ruleset (`magic.rs:606-611`, handles ≥ 128 never match, `51-54`). First target: stack exists before any use (`1126`), hex standable / shape fits / removable (`1129-1143`), out-of-field hexes fail `is_standable` (`geometry.rs:233-239`). Second: only after the first holds (`1188-1189`); Teleport hex via `fits` (`1165-1169`), victim exists (`raise.rs:229`). | `plan` takes `&BattleState` and returns a `Plan` (`command.rs:180-185`, `392`); no interior mutability in the crate (grep: none); first draw is Eagle Eye's in `apply_cast` (`magic.rs:2222`). Property: state with RNG, hash and events unchanged on `Err` (`tests/properties.rs:452-456`). | To the issuing seat only (`h3-protocol/src/session.rs:737`, `978-980`). Own facts: `NotInSpellBook`, `NotACombatSpell`, `AlreadyCast`, `NotCastable`, `NotEnoughMana`. The other hero's hidden magic, as BAT-Q26 allows: `CastingForbidden`, `SpellLevelBarred`, `InvalidSpellTarget`. None depends on a hidden obstacle. |
| `CreatureCast { target }`, a spell | Yes: the target only; spell, mastery and power from the creature's ability or the drawn spell (`casters.rs:106-153`). | `ready`: active stack, a cast left, not tactics, controller is the actor (`casters.rs:382-401`), after `plan`'s own check. | Stack exists (`magic.rs:1379`, `1126`, `1321`, `1447`); a hex must be a hex of a target stack (`1382-1385`). Content check keeps a caster's spell at a stack (or, drawn, a hex): `h3-content/src/ruleset.rs:1174-1256`. | Same `plan`; the Master Genie's draw is in `draw_useful`, after validation (`magic.rs:1510-1536`, `2190-2194`). | `CannotCast`, `InvalidSpellTarget`. |
| `CreatureCast { target }`, a raise (Archangel, Pit Lord) | Yes; hit points and count from the raiser (`casters.rs:588-602`, `609-624`). | as above | Target exists and is not the caster (`casters.rs:582-587`); a slot for the Demons is checked (`created.rs:101-114`) before `add` (`407-434`). | Same. | `CannotCast`, `InvalidSpellTarget`. |
| `Move`, `Attack` as E04b changed them (pits, mines, Fire Wall; a hypnotized stack) | unchanged | controller (`command.rs:190`) | unchanged | Reach from the pits the controller sees only (`movement.rs:109`); what the walk meets is resolution (`obstacles.rs:698-794`). | `Unreachable` only for what the controller sees (`command.rs:264-268`). |
| `Retreat`, `Surrender` | unchanged | outside tactics the actor (`command.rs:199-200`); in tactics the active stack's side (`372-373`): F-sec-3. | - | - | unchanged |
| Sim `Action { entry, spell, target, second }`, C `H3Action` | Same fields. | `BattleEnv::step(actor, ..)` passes the actor to `apply` (`h3-sim/src/env.rs:220-233`), but decodes first without it: F-sec-2. | Heads range-checked (`h3-sim/src/action.rs:142-151`, `289`, `399`, `443`, `460`). | `decode` is pure. | `InvalidAction` / `Rejected`: F-sec-2. |

Legal lists and `apply` share one implementation (`first_target`, `second_target`,
`creature_target`; `cast_targets` and `plan_cast` both go through `check_spell`).

## Findings

### F-sec-1: Two automatic turns are planned with pits one side does not see, and their public events show it: a berserk walker (its own side's pits) and the automatic Ballista's choice of target (the target side's pits)
- kind: leak
- severity: medium. By the letter of the scale it is "read hidden state in normal play". I rate
  it medium because what comes out is partial: the pits on one stack's routes, or one yes/no per
  Ballista shot. (a) needs Berserk on a walker of the side that owns the pits and, under
  BAT-QUIRK-2 (on in `faithful`), can give one pit's exact hex. (b) needs nothing from the
  observer: any Ballista without Artillery against a side with Quicksand.
- confidence: medium to high. Both paths are traced line by line and both sketches are traced by
  hand; nothing was run. Whether the original does the same is not known for (a)
  (`control.rs:5-7`: the engine's own reachability, survey S-11 unconfirmed); (b) is a rule
  ported from VCMI (`war.rs:98-99`, `153-161`), so the original does not decide it. Neither is a
  recorded quirk.
- evidence, common: for a walker, `movement::reachable_within` takes
  `obstacles::seen_pits(state, mover.controller())` (`h3-battle/src/movement.rs:109`) and stops
  the walk at those pits (`133-138`). A side sees its own unrevealed pits (`obstacles.rs:252-254`).
  That is right for a commanded stack: its controller chooses. Two callers are rules, not
  players, and report in public:
- evidence (a), the berserk turn:
  - `control.rs:52` computes the berserk stack's reach that way; a berserk stack's controller is
    its own side (`state.rs:707-713`). Target and position come from it: `attack_position`
    (`control.rs:62`, `122-141`), `towards` (`107`, `146-172`, a second reach at `162`), "no
    candidate" (`86`).
  - All it reports is public: `Moved` (`obstacles.rs:716-723`), `Struck`, `TurnPassed`
    (`control.rs:86`, `113`). When the walk ends in the owner's own pit, the pit is not revealed
    (`stuck`, `obstacles.rs:806`: the controller sees it) and `ObstacleTriggered` goes to the
    owner only (`815-822`): the other side gets a bare `Moved`.
  - Scenario. B's hero has cast Quicksand; A sees none of it. A casts Berserk on B's walker X
    (or B's own area Berserk covers X). At X's activation (`turn.rs:279-291`) X goes for the
    nearest stack. A knows the rule and every input but B's pits (and the tie draw,
    `control.rs:84`), so A knows the moves X can make on a field without hidden pits. X ending
    elsewhere, striking from another position or another stack, or passing its turn, proves pits
    on the shorter routes. With BAT-QUIRK-2, `attack_position` accepts a position whose walk
    stops in a seen pit (`control.rs:136`): X walks next to its target, stops in B's pit and does
    not strike (`102-104`). A berserk stack that arrives always strikes, so A reads the pit's hex
    off X's position. Spectators get the same events.
- evidence (b), the automatic Ballista:
  - `war::act` → `shooting_target` (`war.rs:85-96`, `162-203`) takes the best candidate "that
    could attack next turn" (`197`). `can_attack_next_turn` computes the candidate's reach with
    `reachable(state, field, id, me)` (`100-117`, the reach at `105`), so with the pits the
    candidate's controller sees: for A's Ballista judging B's stacks, B's own unrevealed pits.
  - The result is a public `Struck { attacker: the Ballista, target }`.
  - Scenario. A's Ballista is automatic. B's most valuable walker stands exactly its speed away
    from its only attack position, and one of B's own hidden pits lies on that straight way: it
    "cannot attack next turn", so the Ballista shoots B's next stack. Without the pit it shoots
    the valuable one. A learns, shot by shot, whether B's own pits cut B's stacks off.
  - `hidden_obstacles_do_not_show` covers this case: it compares what the viewer is told after
    its command, automatic turns included (`tests/properties.rs:791-825`), and its games have
    Ballistas and hidden pits. It can therefore fail at random on such a layout. Its games never
    contain Berserk, so (a) is outside it (F-sec-4).
- note: BAT-MAG-031 ("what a side does not see never changes what it may do",
  `docs/rules/catalog/battle.toml:2574`) holds; it says nothing about what an automatic turn
  shows. A commanded walk's route has the same dependency (`movement::path`, `movement.rs:167-191`),
  but there the side that knows the pits picks the move.
- fix or test sketch:
  - Decide (a) first (AGENTS.md, quirk classification): read what the original's berserk AI
    plans with, then record it as a Q-0002 instance with its class, or fix it. A fix that keeps
    the hygiene property for both: rule-driven plans use only the pits both sides see; the walk
    still stops in any pit (`travel_through` does, `obstacles.rs:737-741`); a pit an automatic
    walk stops in is revealed, since the stop shows it. Using the Ballista's own side's pits
    instead would only turn the leak round.
  - Test (a), `needs-run`, in the style of `tests/pits.rs`. By hand it fails today in both
    profiles: without the pits the walker goes to (5, 5) in 3 steps and strikes; with them and
    BAT-QUIRK-2 off it strikes from (4, 6) (the first position at 4 steps), with BAT-QUIRK-2 on
    it walks round the first pit to (5, 5), stops in the second and does not strike. One pit at
    (6, 5) alone changes only the route, which `Moved` does not show.
    ```rust
    #[test]
    fn a_berserk_walkers_turn_does_not_show_its_sides_unseen_pits() {
        for profile in ["t:faithful", "t:pits"] { // BAT-QUIRK-2 off, on
            let r = ruleset_with(profile, |_| {});
            let mut s = arena(&r, [None, None], &[("fixed", 5, 4, 5)], &[("fixed", 5, 8, 5)], 1);
            let mut twin = s.clone();
            // The defender's own pits, unseen by the attacker, on its walker's way.
            pits(&mut s, &r, BattleSide::Defender, &[(6, 5), (5, 5)]);
            for state in [&mut s, &mut twin] {
                let st = &mut state.stacks.as_mut_slice()[1];
                st.spells.status = st.spells.status.with(Status::BERSERK, true);
            }
            // The attacker's stack defends; the berserk walker then acts on its own.
            let told = |state: &mut BattleState| -> Vec<Event> {
                let events = act(state, &r, Command::Defend).unwrap();
                events.iter().filter_map(|e| project(e, BattleSide::Attacker)).collect()
            };
            assert_eq!(told(&mut s), told(&mut twin));
        }
    }
    ```
  - Test (b), `needs-run`, in the style of `tests/war.rs` (with `pits` from `tests/pits.rs`).
    By hand: the valuable stack is 5 steps, its speed, from (3, 5), the only attack position in
    its reach, along its row; with the pit the detour takes 6. Today the targets are 3 and 2.
    ```rust
    #[test]
    fn the_ballistas_target_does_not_show_the_other_sides_unseen_pits() {
        let r = ruleset();
        // Ids: 0 serf, 1 ballista (attacker); 2 valuable, 3 fixed (defender).
        let setup = with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "valuable", 1), (1, "fixed", 1)],
            [&["ballista"], &[]],
        );
        let mut s = start(&r, &setup, 1);
        place(&mut s, &r, 0, 2, 5);
        place(&mut s, &r, 2, 8, 5);
        place(&mut s, &r, 3, 4, 3);
        let mut twin = s.clone();
        // The defender's own pit, unseen by the attacker.
        pits(&mut s, &r, BattleSide::Defender, &[(5, 5)]);
        let target = |s: &mut BattleState| shots_by(&defend_round(s, &r), 1)[0].0;
        assert_eq!(target(&mut s), target(&mut twin));
    }
    ```
    The lasting form of (a) is F-sec-4's: put `t:berserk` into the property's spell book.

### F-sec-2: `h3_sim::action::decode` has no actor, so `BattleEnv::step` and `h3_env_step` tell the side that does not control a Faerie Dragon its drawn spell by the kind of refusal
- kind: leak (an error tells the issuer a hidden fact)
- severity: low. The hosted wire is not affected (it has its own typed commands, and the seat not
  to act gets `NotYourTurn`); the sim and the C ABI are documented as trusted local surfaces
  (`h3-ffi/src/lib.rs:9-10`). It still breaks ADR-0014 point 8 for anything built on the env that
  lets an agent act out of turn, and nothing warns the next surface.
- confidence: high (the two refusals are separate enum values and separate ABI codes).
- evidence:
  - `h3-sim/src/action.rs:213-218`: `active_spell` asks `h3_battle::stack_spell` for the active
    stack, whoever asks. `stack_spell` (`h3-battle/src/casters.rs:406-408`, exported in
    `lib.rs:55`) returns a random caster's drawn spell while it is active (`casters.rs:138-150`).
  - `action.rs:298-304`: a `CREATURE_CAST` action decodes only if `action.spell` equals that
    spell (the tests pin it: `854-864`, `943-947`).
  - `h3-sim/src/env.rs:224-233`: `step` decodes first (`InvalidAction`), then applies with the
    actor (`Rejected(NotYourTurn)`, `h3-battle/src/command.rs:190-192`). The C ABI keeps them
    apart: `H3_ERR_INVALID_ACTION` −3, `H3_ERR_REJECTED` −4 (`h3-ffi/src/lib.rs:520-521`).
  - Scenario. A's Faerie Dragon is active with a drawn spell. B calls
    `step(B, Action { entry: CREATURE_CAST, spell: s, target: 0, second: 0 })` for s in 0..128.
    The one value answered `Rejected` is the drawn spell; nothing changes, so the probes are
    free. `Observation.drawn` hides the same fact from B (`observe.rs:192-196`).
  - Same root, no leak today: `encode` reads `active_spell` too (`action.rs:238-245`);
    `BattleState::spell_obstacles()` returns every obstacle, hidden ones included
    (`state.rs:995-997`), and the server's `Bot` trait hands a bot the whole state with a comment
    asking it to use per-side functions (`h3-protocol/src/session.rs:90-104`).
- fix or test sketch:
  - `BattleEnv::step`: refuse `Some(actor) != self.to_move()` with
    `Rejected(CommandError::NotYourTurn)` before decoding. Or give `decode`/`encode` the actor
    and take the spell from `creature_cast_spell(state, ruleset, actor)`. Then drop `stack_spell`
    from the public API or give it a viewer.
  - Test, `needs-run`, in `h3-sim/src/env.rs` (a scenario with the synthetic pack's random
    caster, `packs/synthetic/pack.toml:268`):
    ```rust
    // ... play until a random caster with a drawn spell awaits its controller's command
    let controller = env.to_move().unwrap();
    assert!(env.observation(controller).drawn.is_some());
    assert_eq!(env.observation(controller.opponent()).drawn, None);
    for spell in 0..SPELL_COUNT as u16 {
        let probe = Action { entry: CREATURE_CAST as u32, spell, target: 0, second: 0 };
        // Today: Rejected(NotYourTurn) for the drawn spell, InvalidAction for the others.
        assert_eq!(
            env.step(controller.opponent(), probe),
            Err(StepError::Rejected(CommandError::NotYourTurn))
        );
    }
    ```

### F-sec-3: In the tactics phase Retreat and Surrender act for the active stack's side, not for the trusted actor
- kind: validation
- severity: low: not reachable today. No timed effect can exist in the tactics phase (casts are
  refused there, `command.rs:378-379`; opening spells come at the first activation outside it,
  `magic.rs:922-929`), so controller and side are the same, stack for stack.
- confidence: high for the code; the unreachability rests on the two lines just cited.
- evidence: `h3-battle/src/command.rs:372-373` call `retreat(state, mover.side)` and
  `surrender(state, ruleset, mover.side)`; outside tactics the same commands use `actor`
  (`199-200`), and `legal_commands` uses `actor` in both phases (`legal.rs:65-71`). `plan` has
  only checked `mover.controller() == actor` (`190`). If a later task lets a status exist during
  tactics (effects carried into a battle, a spell before the phase), the side controlling a
  hypnotized stack could issue `Retreat` and the stack's owner would be the one who retreats
  (`Plan::Retreat { side }` → `BattleResult::Retreat(side)`, `449`), and the legal list would
  disagree with `apply`.
- fix or test sketch: pass `actor` to `plan_tactics` and use it there. Test (`needs-run`): a
  tactics state with `Status::HYPNOTIZED` set by hand on the active stack; `apply(controller,
  Command::Retreat)` must give `Retreat(controller)` or an error, never `Retreat(owner)`.

### F-sec-4: The invariant and hygiene properties never play Berserk, Hypnotize, a hero's hidden magic in force, BAT-QUIRK-2 or a creature's cast under hidden obstacles
- kind: test-gap
- severity: medium. `docs/06-security.md` and ADR-0014 ("Consequences") name these properties as
  what enforces hiding and ownership for the epic; F-sec-1 (a) sits exactly in what they leave
  out.
- confidence: high (read from the generators).
- evidence:
  - The heroes' book in `make_setup` has no `t:berserk`, `t:hypnotize` or `t:blind`
    (`tests/properties.rs:221-257`; the spells exist, `tests/support.rs:2252-2351`), and no
    creature in `CREATURES` casts Berserk or Hypnotize (Blind comes from a strike only,
    `blinder`). So no generated game has a stack whose controller is not its side, or an
    automatic berserk turn. The ownership assertions (`565-580`) are written for side, not
    controller, which is probably why.
  - The heroes' effects are `SpellDamage`, `SpellSpecialty`, `MagicResistance`, `SpellDuration`
    (`265-279`); `extreme_effect` has only E04a kinds (`1058-1081`). No game has `NoCasting`,
    `SpellLevelLimit`, the three immunities, `NegateImmunities`, an opening spell or Eagle Eye in
    force, so "legal casts and `apply` agree" and atomicity are untested with them.
  - `hidden_obstacles_do_not_show` runs `t:sight`, `t:anchors`, `t:quirky` (`725-729`): never
    `t:pits` (BAT-QUIRK-2, `tests/support.rs:1504-1507`), never the combination `faithful`
    ships. It replays `legal_commands` and hero casts (`768-790`), no `CreatureCast`.
  - `arb_command` (`67-97`) has no `Shoot`, no stack as a second target, and stack ids below 40
    only (`STACK_CAPACITY` is 42, a payload id goes to 255).
- fix or test sketch: add the three spells to the book (ownership assertions by controller, the
  move assertion relaxed for controlled and berserk stacks); a hero strategy with the E04b
  effects; `t:pits` and a profile with quirks 1 to 3 together in the hidden-obstacle property,
  plus `creature_cast_targets`; `Shoot`, `CastTarget::Stack` seconds and `any::<u8>()` stack ids
  in `arb_command`.

### F-sec-5: Hidden draws come from the same stream as the public rolls; they show only to someone who recovers the generator's state, which the design does not rule out
- kind: side channel
- severity: low (residual; the generator is the kernel's, older than E04b, but E04b is what makes
  past draws worth knowing: pit and mine hexes, a drawn spell).
- confidence: low. The code facts are traced; the cost of a recovery is an estimate, nothing was
  run or measured.
- evidence:
  - Hidden draws: pit and mine hexes `state.rng.below(len)` (`h3-battle/src/obstacles.rs:404`),
    the Faerie Dragon's spell (`casters.rs:241`), Eagle Eye (`magic.rs:1003`). Public rolls
    (morale, luck, damage, resistance, the mirror, a summon's row) use the same `state.rng`.
  - Counting does not help a viewer: a hidden draw leaves no event and no gap anyone can see.
  - The generator is xoshiro256\*\*, not a cryptographic one, and its 256-bit state is expanded
    from a 64-bit seed (`h3-kernel/src/rng.rs:83-97`). Each public `below(n)` result shows the
    top bits of one output (`124-143`). So the work to learn the state is at most 2^64 seed
    trials offline against a battle's public rolls; I did not look for a cheaper recovery from
    truncated outputs. With the state, every hidden draw of the battle so far and every roll to
    come can be computed.
  - `docs/06-security.md` asks that the seed and state never reach clients and gets that; it
    does not say the outputs may not give them away.
- fix or test sketch: none in the battle lane. For the user and the shared lane: either add it to
  "Not covered yet" in `docs/06-security.md` with the estimate, or let a hosted game use a keyed
  stream behind the same `Rng` calls (ADR-0003 territory). A second named stream for the hidden
  draws would not help: every stream derives from the one seed (`rng.rs:94-97`).

## Gaps in the hygiene properties

`observations_do_not_depend_on_hidden_state` (`tests/properties.rs:633-707`)
- Varies: the RNG; the other side's hero's private fields, T9a's among them; the drawn spell of a
  stack that side controls. Compares `observe` and `observe_as` at one state.
- It mutates a clone instead of playing a twin game, and keeps `limits` (comment at `667-668`).
  The generated heroes have none of the hidden magic, so `limits.guards` is false and
  `Guard::of` returns early (`magic.rs:678`): the changed defences, negation and limits are read
  by no rule. `cast_candidates`, which does depend on the other hero's defences for a hypnotized
  own stack, is therefore not exercised.
- Not varied: the other side's hidden obstacles (the other property), the templates, a
  controller that differs from the side.
- Not compared: events, legal lists, the flat encoding (it derives from `Observation`, so it
  follows), anything after a further command. No check that a battle with the other hero's
  Eagle Eye at chance 0 and at 100 (no draw either way) tells the viewer the same events.

`hidden_obstacles_do_not_show` (`719-828`)
- Varies: pits and mines of both sides, placed directly; BAT-QUIRK-1 or 3 on, or neither.
- Compares for the side to act only: observation, `legal_commands`, `castable_spells`,
  `cast_targets`, `second_targets` for three targets, then each command's projected events and
  the observation after, the automatic turns that follow in the same command included. The
  other viewer has no commands, so what it is told about the acting side's command is never
  looked at.
- Skips a command entirely when anything is revealed (`796-798`) or a raised native gives sight
  (`810-813`): the events before the reveal are not compared.
- Not covered: BAT-QUIRK-2, BAT-QUIRK-4 (by design), the `faithful` combination, creature casts,
  Berserk and Hypnotize (F-sec-4), the sim's masks.
- Covered but not stable: the automatic Ballista (F-sec-1 b) makes it fail on a rare layout.
- The spectator part checks the observation; `public(&ex) == shown` (`816-824`) restates
  `project_as`.

`tests/observe.rs`: three example tests (what is kept per stack, `own_hero` with the hidden
magic, a spectator with one pit and a drawn spell). They show that the fields are filled and
dropped; they vary nothing.

`h3-protocol`: `hidden_obstacles_never_reach_the_other_viewers_bytes` compares frames across
seeds up to and just after the first hidden cast (`tests/session.rs:1552-1648`); it does not play
on, so no later consequence of a hidden object is compared on the wire.

## Not reviewed, left open

- No finding in: the wire mapping (`h3-protocol/src/map/`: observation and events go through
  `observe_as` and `project_as`, `mod.rs:130-143`; exhaustive matches; the echo of an applied
  command goes to its seat only, `session.rs:739-748`); the flat encoding; the C ABI's `unsafe`
  (null, side and length checked before a slice is formed, `h3-ffi/src/lib.rs:314-328`,
  `469-479`, `499-504`; panics caught; the header matches). Not checked by the code, left to the
  caller: the alignment of `out` in `h3_env_observe`.
- Bounded work: no finding. Target lists are at most 298 entries, a Teleport query at most one
  fit test per stack and hex of the field; a chain strikes each stack once
  (`magic.rs:2360-2373`); one periodic cast per activation; the berserk loop runs at most twice
  (`morale.rs:344`). Storage: stacks fixed at 42 with `has_room`/`has_slot` before every `add`
  (at the bound Clone and Summon are no cast, a Pit Lord's raise is refused); effects one entry
  per stack and spell; spell obstacles bounded by the free hexes a caster sees; binds a bit
  mask; casts and rounds saturate. Two things are bounded by data, not by a constant: the
  automatic turns inside one command (by the durations of what takes turns away; Berserk's 255
  rounds is the largest in the packs) and the battle itself (no round limit, already under "Not
  covered yet").
- For the server lane, not ours to change: the wire has no query for a creature's cast targets
  and `YourTurn` lists none (`session.rs:967-975`, `wire.rs:990-1003`), so a client must probe
  with commands; the sample scenario hosts `synthetic:faithful`
  (`packs/synthetic/scenarios/full.toml:7`), which has BAT-QUIRK-4 and BAT-QUIRK-2 on, so hosted
  games carry the decided Quicksand/Land Mine draw leak and the sharper form of F-sec-1 (a).
- Not read: `h3-server`'s I/O shell and journal, `h3-heroes` (the derivation that fills
  `BattleSetup`, trusted input here), the adventure seam, timing. Not verified: the claim in
  `event.rs:312-314` that the original's log shows the three `SpellDamage` amounts (batch 2a R5);
  what the original's berserk AI plans with (decides how F-sec-1 (a) is classified).
