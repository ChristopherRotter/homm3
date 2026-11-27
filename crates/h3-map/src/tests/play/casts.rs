//! The adventure spells in the plans (E08 T144b; increment 15's design note §17): casters among
//! the plans' heroes ([`Caster`], [`spell_plan`]), runs of casts ([`spell_steps`], `Step::Cast`,
//! [`seek_cast`]), the oracles of a step's casts and new day ([`check_casts_step`]: the door's
//! points, the day spells cleared) and the cover test [`the_plans_cast_on_the_map`]. The
//! listing's oracles are T144a1's (`check_casts`, `check_cast_targets`); a refused cast changes
//! nothing by the properties' atomicity.

use std::collections::BTreeMap;

use super::*;

/// The five adventure spells the adventure part binds, by code: Summon Boat, Scuttle Boat, Fly,
/// Water Walk, Dimension Door.
pub(super) const ADVENTURE_SPELLS: [u8; 5] = [0, 1, 6, 7, 8];

/// A plan's caster (E08 T144b): a hero (modulo the plan's) wearing a Spell Book, with some of the
/// [`ADVENTURE_SPELLS`] of its own (on a SoD map; on an AB map the first, its one spell byte, the
/// type's own beside it; none on an RoE map, which stores no spells) and, on a SoD map, primary
/// skills 1, 1, 1 and its knowledge, so that random play casts them.
#[derive(Debug, Clone)]
pub(in crate::tests) struct Caster {
    /// The hero, modulo the plan's.
    pub(super) hero: u8,
    /// Indices into [`ADVENTURE_SPELLS`].
    pub(super) spells: Vec<usize>,
    /// Its knowledge on a SoD map (ten spell points each).
    pub(super) knowledge: u8,
}

/// The casters of a plan: none in one plan of two, else one to three.
fn casters() -> impl Strategy<Value = Vec<Caster>> {
    let spells = subsequence((0..ADVENTURE_SPELLS.len()).collect::<Vec<_>>(), 1..=5);
    let caster = (any::<u8>(), spells, 2u8..=12).prop_map(|(hero, spells, knowledge)| Caster {
        hero,
        spells,
        knowledge,
    });
    prop_oneof![1 => Just(Vec::new()), 1 => vec(caster, 1..=3)]
}

/// [`trade_plan`]'s plans with casters (E08 T144b): the properties of `play.rs` and `view.rs`
/// draw these, the cover tests [`plan`]'s (the trades' precedent).
pub(in crate::tests) fn spell_plan() -> impl Strategy<Value = Plan> {
    (trade_plan(), casters()).prop_map(|(mut plan, casters)| {
        plan.casters = casters;
        plan
    })
}

/// [`trade_steps`]' runs with (E08 T144b) runs of casts among them in one case of three: 16 to 64
/// `Step::Cast`s with moves and ends of turn among them, which cast the heroes' spells, the
/// targets drawn from the queries, and walk, fly and water-walk after them. The properties draw
/// these.
pub(in crate::tests) fn spell_steps() -> impl Strategy<Value = Vec<Step>> {
    let casts = vec(
        prop_oneof![
            6 => any::<u16>().prop_map(Step::Cast),
            3 => any::<u16>().prop_map(Step::Legal),
            1 => Just(Step::EndTurn),
        ],
        16..64,
    );
    prop_oneof![
        2 => trade_steps(),
        1 => (trade_steps(), casts).prop_map(|(mut steps, casts)| {
            let at = steps.len() / 2;
            steps.splice(at..at, casts);
            steps
        }),
    ]
}

/// The plan's caster at the hero `index` of `heroes` (E08 T144b), if one is: the first one whose
/// hero this is (modulo).
pub(super) fn caster_of(plan: &Plan, index: usize, heroes: usize) -> Option<&Caster> {
    plan.casters
        .iter()
        .find(|c| usize::from(c.hero) % heroes.max(1) == index)
}

/// The step `Step::Cast(n)`: in a battle as `fight`, an open decision settled as the trades'
/// steps settle it; in one step of eight a cast drawn from `n` alone ([`drawn_cast`]), accepted
/// or refused; else the `n`-th of the actor's casts, those `legal` lists and each of its heroes'
/// targeted casts at the targets `legal_cast_targets` gives, or, with none, the `n`-th of
/// `legal`.
pub(super) fn seek_cast(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    n: u16,
) -> GameCommand {
    let who = actor(state);
    if battle_of(state, who).is_some() {
        return fight(state, rules, legal, n);
    }
    if let Some(answer) = settle(state, legal, n) {
        return answer;
    }
    if n % 8 == 7
        && let Some(cast) = drawn_cast(state, n / 8)
    {
        return cast;
    }
    let mut casts: Vec<GameCommand> = legal
        .iter()
        .filter(|c| matches!(c, GameCommand::Cast { .. }))
        .cloned()
        .collect();
    let player = acting(state);
    let part = rules.adventure();
    for hero in state.adventure().hero_list(player) {
        for &(spell, effect) in part.spells() {
            if !effect.targets() {
                continue;
            }
            let targets = legal_cast_targets(state, rules, who, (hero, spell));
            casts.extend(targets.into_iter().map(|target| GameCommand::Cast {
                hero,
                spell,
                target: Some(target),
            }));
        }
    }
    match casts.len() {
        0 => legal[usize::from(n) % legal.len()].clone(),
        len => casts.swap_remove(usize::from(n) % len),
    }
}

/// A cast of the acting player's drawn from `n` alone, listed or not (E08 T144b): one of its
/// heroes, one of the [`ADVENTURE_SPELLS`], Scuttle Boat's and the door's target up to ten
/// columns and nine rows from the hero (its own tile where that leaves the map), so that refusals
/// and unlisted casts meet the properties' atomicity and `check_unlisted`; none without a hero.
fn drawn_cast(state: &GameState, n: u16) -> Option<GameCommand> {
    let heroes = state.adventure().hero_list(acting(state));
    let hero = *heroes.get(usize::from(n) % heroes.len().max(1))?;
    let n = usize::from(n) / heroes.len();
    let code = ADVENTURE_SPELLS[n % ADVENTURE_SPELLS.len()];
    let spell = h3_content::SpellId::from_raw(u16::from(code));
    let target = matches!(code, 1 | 8).then(|| {
        let at = state.adventure().hero(hero).unwrap().pos;
        let (dx, dy) = ((n / 5) % 21, (n / 105) % 19);
        let x = (at.x + u16::try_from(dx).unwrap()).checked_sub(10);
        let y = (at.y + u16::try_from(dy).unwrap()).checked_sub(9);
        let map = state.adventure().map();
        x.zip(y)
            .map(|(x, y)| TilePos::new(x, y, at.level))
            .filter(|&pos| map.tile(pos).is_some())
            .unwrap_or(at)
    });
    Some(GameCommand::Cast {
        hero,
        spell,
        target,
    })
}

/// The step's casts and new day (E08 T144b; increment 15's design note §17, exit criteria 4 and
/// 7): an accepted Dimension Door (its `SpellCast` of the door, arrived or failed) leaves its hero
/// the points before less the door's points at the mastery it tells, at least 0 (ADV-SPL-007), on
/// the arrival's tile or its own; and a new day's start leaves no hero with a day spell
/// (ADV-SPL-002).
pub(super) fn check_casts_step(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    command: &GameCommand,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let door = h3_content::SpellId::from_raw(8);
    if let GameCommand::Cast {
        hero,
        spell,
        target,
    } = *command
        && spell == door
        && let Some(mastery) = events.iter().find_map(|e| match e {
            GameEvent::SpellCast { mastery, .. } => Some(*mastery),
            _ => None,
        })
    {
        let points = rules
            .adventure()
            .params()
            .spells()
            .map_or(0, |p| p.door_points[usize::from(mastery)]);
        let (was, is) = (
            before.adventure().hero(hero).unwrap(),
            after.adventure().hero(hero).unwrap(),
        );
        prop_assert_eq!(is.movement, was.movement.saturating_sub(points));
        let failed = events
            .iter()
            .any(|e| matches!(e, GameEvent::CastFailed { .. }));
        prop_assert_eq!(is.pos, if failed { was.pos } else { target.unwrap() });
    }
    let new_day = events
        .iter()
        .any(|e| matches!(e, GameEvent::DayStarted { .. }));
    if new_day {
        for hero in after.adventure().heroes() {
            prop_assert_eq!(
                after.adventure().day_spells(hero.id),
                h3_adventure::DaySpells::default(),
                "{:?}",
                hero.id
            );
        }
    }
    Ok(())
}

/// What the cover test's casts met (E08 T144b): the casters laid out, the casts by spell
/// ([`ADVENTURE_SPELLS`]' order) and their charged failures by reason, the refused ones, the boats
/// summoned, made and sunk, the doors' arrivals, the new days with a day spell to clear.
#[derive(Debug, Default)]
struct CastsMet {
    casters: usize,
    casts: [usize; 5],
    failures: BTreeMap<String, usize>,
    refused: usize,
    summoned: usize,
    made: usize,
    sunk: usize,
    arrived: usize,
    cleared: usize,
}

fn cast_plans() -> impl Strategy<Value = (Plan, bool, u64, Vec<Step>)> {
    let steps = vec(
        prop_oneof![
            8 => any::<u16>().prop_map(Step::Cast),
            3 => any::<u16>().prop_map(Step::Legal),
            1 => Just(Step::EndTurn),
        ],
        0..160,
    );
    let plan = spell_plan().prop_filter("casters on a SoD map", |p| {
        !p.casters.is_empty() && p.version == Version::Sod
    });
    (plan, any::<bool>(), any::<u64>(), steps)
}

/// The plans cast on the map (E08 T144b; increment 15's design note §17, exit criteria 3 to 8):
/// over a fixed set of cases (proptest's deterministic runner), random play with the casts' steps
/// ([`cast_plans`], `Step::Cast`) casts each of the five spells, accepted and failing on its
/// chance or its target, refused, summons, makes and sinks boats, arrives by the door and clears
/// the day spells at a new day, in both profiles, so that the properties' checks (`check_legal`'s
/// `check_casts` and `check_cast_targets`, [`check_casts_step`], the atomicity of a refusal) see
/// each.
// covers: GAM-SPL-001, ADV-SPL-001, ADV-SPL-002, ADV-SPL-003, ADV-SPL-004, ADV-SPL-005
// covers: ADV-SPL-006, ADV-SPL-007, ADV-SPL-008, ADV-SPL-009
#[test]
fn the_plans_cast_on_the_map() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = cast_plans();
    let mut met = CastsMet::default();
    for _ in 0..96 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        met.casters += plan.casters.len();
        for (i, step) in steps.iter().enumerate() {
            let legal = options(&state, rules);
            if i % 4 == 0 {
                let reaches = reaches(&state, rules);
                check_legal(&state, rules, &legal, &reaches).unwrap();
            }
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command.clone(), &mut events);
            if result.is_err() {
                assert_eq!(state, before);
                met.refused += usize::from(matches!(command, GameCommand::Cast { .. }));
                continue;
            }
            check_casts_step(rules, (&before, &state), &command, &events).unwrap();
            let carried =
                before.adventure().heroes().iter().any(|h| {
                    before.adventure().day_spells(h.id) != h3_adventure::DaySpells::default()
                });
            for e in &events {
                match e {
                    GameEvent::SpellCast { spell, .. } => {
                        let code = u8::try_from(spell.raw()).unwrap();
                        if let Some(at) = ADVENTURE_SPELLS.iter().position(|&s| s == code) {
                            met.casts[at] += 1;
                        }
                    }
                    GameEvent::CastFailed { spell, reason, .. } => {
                        let key = format!("{} {reason:?}", spell.raw());
                        *met.failures.entry(key).or_default() += 1;
                    }
                    GameEvent::DayStarted { .. } => met.cleared += usize::from(carried),
                    GameEvent::Adventure(a) => match a.kind {
                        AdventureEventKind::BoatSummoned { .. } => met.summoned += 1,
                        AdventureEventKind::BoatCreated { .. } => met.made += 1,
                        AdventureEventKind::BoatScuttled { .. } => met.sunk += 1,
                        AdventureEventKind::HeroTeleported { .. }
                            if matches!(command, GameCommand::Cast { .. }) =>
                        {
                            met.arrived += 1;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
    let m = &met;
    eprintln!("{m:?}");
    assert!(m.casts.iter().all(|&n| n > 0), "{m:?}");
    for failure in ["0 Chance", "1 Chance", "1 Target", "8 Target"] {
        assert!(m.failures.contains_key(failure), "{failure}: {m:?}");
    }
    assert!(m.refused > 0 && m.summoned > 0 && m.sunk > 0, "{m:?}");
    assert!(m.arrived > 0 && m.cleared > 0, "{m:?}");
}
