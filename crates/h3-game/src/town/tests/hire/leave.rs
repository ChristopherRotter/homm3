//! Tests of a hero that leaves the game (E09 T38b1, design note `E09-design.md` §67; TWN-TAV-006):
//! a beaten hero's type free with one creature, its spell points full and its points over the
//! cleared army (`TWN-QUIRK-12`, both settings); a retreated or surrendered hero back into its
//! owner's tavern, its slot drawn from the owner's stream, the marked slot's other (TD-18,
//! TD-19), told to its owner last; a draw's and a created-stacks winner's leaves. On the hire
//! tests' game: red hires at T0, then in blue's turn blue at T1, and a battle between the two
//! heroes ends ([`GameState::end_battle`]).

use h3_battle::BattleResult;
use h3_content::Formation;
use h3_contracts::{Army, BattleOutcome, BattleSide, HeroId, HeroOutcome};

use super::*;
use crate::{Manage, Viewer};

/// The hero `player` hires from its slot 0 (its type's record).
fn slot_hero(game: &GameState, player: PlayerId) -> HeroId {
    let hero_type = slots(game, player)[0].unwrap();
    game.pool().record(hero_type).unwrap()
}

/// The game after red's hire at T0 (slot 0) and blue's at T1 (slot 0) in its turn, red's hero in
/// the tight formation: with red's and blue's hired heroes (the setup's two heroes, both of type
/// 0, stay apart).
fn hired(rules: &GameRuleset) -> (GameState, HeroId, HeroId) {
    let mut game = game(rules, &towns_setup(rules), 5000);
    let red = slot_hero(&game, RED);
    run(rules, &mut game, RED, hire(T0, 0)).unwrap();
    let tight = Manage::Formation {
        hero: red,
        formation: Formation::Tight,
    };
    run(rules, &mut game, RED, GameCommand::Manage(tight)).unwrap();
    run(rules, &mut game, RED, GameCommand::EndTurn).unwrap();
    game.set_resource(BLUE, GOLD, 5000);
    let blue = slot_hero(&game, BLUE);
    run(rules, &mut game, BLUE, hire(T1, 0)).unwrap();
    (game, red, blue)
}

/// An outcome of `result` with these army survivors (by side), each hero left `mana` spell points.
fn outcome(result: BattleResult, survivors: [&[ArmyStack]; 2], mana: u16) -> BattleOutcome {
    BattleOutcome {
        result,
        survivors: survivors.map(|stacks| Army {
            stacks: stacks.to_vec(),
        }),
        war_machines: [vec![], vec![]],
        stacks: [vec![], vec![]],
        surrender_cost: None,
        heroes: [0, 1].map(|_| {
            Some(HeroOutcome {
                mana,
                learned: vec![],
            })
        }),
        rounds: 1,
    }
}

/// `game` after the battle of red's `red` (attacking) and blue's `blue` ended with `outcome`, and
/// its events.
fn ended(
    rules: &GameRuleset,
    game: &GameState,
    (red, blue): (HeroId, HeroId),
    outcome: &BattleOutcome,
) -> (GameState, Vec<GameEvent>) {
    let mut after = game.clone();
    let tile = game.adventure().hero(blue).unwrap().pos;
    let sides = ([Some(RED), Some(BLUE)], [Some(red), Some(blue)], tile);
    let mut events = Vec::new();
    after.end_battle(rules, sides, None, outcome, &mut events);
    (after, events)
}

/// One creature of `hero`'s type's first starting stack.
fn one_of(rules: &GameRuleset, game: &GameState, hero: HeroId) -> ArmyStack {
    let ty = rules.core().hero_type(game.hero(hero).unwrap().hero_type);
    ArmyStack {
        slot: 0,
        creature: ty.unwrap().army[0].creature,
        count: 1,
    }
}

// covers: TWN-TAV-006, GAM-BAT-003
#[test]
fn a_beaten_hero_leaves_with_one_creature_full_spell_points_and_its_land_maximum() {
    for profile in ["t:faithful", "t:leave"] {
        let rules = rules_in((profile, true), |_| {});
        let (game, red, blue) = hired(&rules);
        let red_type = game.hero(red).unwrap().hero_type;
        let survivor = game.adventure().hero(blue).unwrap().army[0];
        let won = outcome(
            BattleResult::Victory(BattleSide::Defender),
            [&[], &[survivor]],
            0,
        );
        let (after, events) = ended(&rules, &game, (red, blue), &won);
        // Off the map, its type free, nothing told of the pool and no draw.
        assert!(after.adventure().hero(red).is_none());
        let pool = after.pool();
        assert_eq!(pool.status(red_type), Some(HeroStatus::Free));
        assert_eq!(pool.record(red_type), Some(red));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, GameEvent::TavernOffered { .. }))
        );
        assert_eq!(after.stream(RED), game.stream(RED));
        assert_eq!(slots(&after, RED), slots(&game, RED));
        // Its record's spell points full (the battle left none); its away record one creature,
        // its formation and tactics flag as on the map.
        let (core, part) = (rules.core(), rules.adventure());
        let details = after.hero(red).unwrap();
        assert_eq!(details.mana, max_mana(details, core));
        let away = pool.away(red_type).unwrap();
        let one = one_of(&rules, &after, red);
        assert_eq!(away.army, [one]);
        assert_eq!((away.formation, away.tactics), (Formation::Tight, true));
        assert_eq!((away.stale, away.full), (false, true));
        // T-16: in `t:leave` the land maximum over the cleared army (the table's most), else
        // over the one creature, which is slower.
        let cleared = h3_adventure::day_points(core, part, Some(details), &[], 0);
        let kept = h3_adventure::day_points(core, part, Some(details), &[one], 0);
        assert_eq!(cleared, *part.params().land_points_table().last().unwrap());
        assert!(kept < cleared, "{kept} {cleared}");
        let points = if profile == "t:leave" { cleared } else { kept };
        assert_eq!((away.movement, away.max_movement), (points, points));
        // The day's regain skips it: its record is full.
        assert!(pool.full_records().contains(&red));
    }
}

// covers: TWN-TAV-006, GAM-BAT-003
#[test]
fn after_a_draw_both_leave_and_a_winner_with_created_stacks_only_leaves_too() {
    let rules = rules();
    let (game, red, blue) = hired(&rules);
    let types = [red, blue].map(|hero| game.hero(hero).unwrap().hero_type);
    let draw = outcome(BattleResult::Draw, [&[], &[]], 0);
    let (after, _) = ended(&rules, &game, (red, blue), &draw);
    for ty in types {
        assert_eq!(after.pool().status(ty), Some(HeroStatus::Free));
    }
    assert_eq!(after.stream(RED), game.stream(RED));
    assert_eq!(after.stream(BLUE), game.stream(BLUE));
    // Red retreats; blue wins with created stacks only (BAT-END-001): it leaves without return.
    let fled = outcome(BattleResult::Retreat(BattleSide::Attacker), [&[], &[]], 0);
    let (after, _) = ended(&rules, &game, (red, blue), &fled);
    assert_eq!(after.pool().status(types[0]), Some(HeroStatus::Offered));
    assert_eq!(after.pool().status(types[1]), Some(HeroStatus::Free));
    assert_eq!(after.stream(BLUE), game.stream(BLUE));
    assert!(
        [red, blue]
            .iter()
            .all(|&h| after.adventure().hero(h).is_none())
    );
}

// covers: TWN-TAV-006, TWN-TAV-005
#[test]
fn a_retreated_hero_returns_into_a_slot_drawn_from_its_owners_stream() {
    let rules = rules();
    let (game, red, blue) = hired(&rules);
    let red_type = game.hero(red).unwrap().hero_type;
    let on_map = game.adventure().hero(red).unwrap().clone();
    let before = slots(&game, RED);
    let survivor = game.adventure().hero(blue).unwrap().army[0];
    let fled = outcome(
        BattleResult::Retreat(BattleSide::Attacker),
        [&[], &[survivor]],
        3,
    );
    let (after, events) = ended(&rules, &game, (red, blue), &fled);
    // The slot: one draw of 0 to 1 from red's stream (neither slot marked).
    let mut stream = game.stream(RED);
    let slot = usize::try_from(stream.range_inclusive(0, 1)).unwrap();
    assert_eq!(after.stream(RED), stream);
    let pool = after.pool();
    let tavern = pool.tavern(RED).unwrap();
    assert_eq!(tavern.slots[slot], Some(red_type));
    assert_eq!(tavern.slots[1 - slot], before[1 - slot]);
    assert!(tavern.returning[slot] && !tavern.returning[1 - slot]);
    assert_eq!(pool.status(red_type), Some(HeroStatus::Offered));
    // The slot's hero goes free (its record and away record kept).
    if let Some(freed) = before[slot] {
        assert_eq!(pool.status(freed), Some(HeroStatus::Free));
        assert!(pool.away(freed).is_some());
    }
    // One creature; its points, the day's maximum and the spell points as the battle left them,
    // the points stale for red's next turn start.
    let away = pool.away(red_type).unwrap();
    assert_eq!(away.army, [one_of(&rules, &after, red)]);
    assert_eq!(
        (away.movement, away.max_movement),
        (on_map.movement, on_map.max_movement)
    );
    assert_eq!((away.formation, away.tactics), (Formation::Tight, true));
    assert_eq!(after.hero(red).unwrap().mana, 3);
    assert!(away.stale && !away.full);
    // Told to red last: the slot's view, then its parts; the view shows it.
    let view = views(&after, RED)[slot].clone().unwrap();
    let told: Vec<GameEvent> = core::iter::once(GameEvent::TavernOffered {
        player: RED,
        slot: u8::try_from(slot).unwrap(),
        hero: Some(view.hero),
    })
    .chain(view.parts().map(|part| GameEvent::TavernPartSeen {
        player: RED,
        slot: u8::try_from(slot).unwrap(),
        part,
    }))
    .collect();
    assert!(events.ends_with(&told), "{events:?}");
    assert_eq!(view.hero.id, red);
    assert_eq!(
        observe(&after, &rules, Viewer::Player(RED)).tavern[slot],
        Some(view)
    );
    assert_eq!(
        observe(&after, &rules, Viewer::Player(BLUE)).tavern,
        views(&after, BLUE)
    );
    // At red's next turn start its points are derived again (TWN-TAV-005: they were stale), and
    // a hire places it with them.
    let mut again = after.clone();
    run(&rules, &mut again, BLUE, GameCommand::EndTurn).unwrap();
    let away = again.pool().away(red_type).unwrap().clone();
    let details = again.hero(red).unwrap();
    let (core, part) = (rules.core(), rules.adventure());
    let points = h3_adventure::day_points(core, part, Some(details), &away.army, 0);
    assert_eq!(
        (away.movement, away.max_movement, away.stale),
        (points, points, false)
    );
    again.set_resource(RED, GOLD, 5000);
    run(
        &rules,
        &mut again,
        RED,
        hire(T0, u8::try_from(slot).unwrap()),
    )
    .unwrap();
    let placed = again.adventure().hero(red).unwrap();
    assert_eq!((&placed.army, placed.movement), (&away.army, points));
    assert_eq!(placed.formation, Formation::Tight);
}

// covers: TWN-TAV-006
#[test]
fn a_surrendered_hero_returns_with_its_survivors_and_a_marked_slot_is_spared() {
    let rules = rules();
    let (game, red, blue) = hired(&rules);
    let red_type = game.hero(red).unwrap().hero_type;
    let kept = ArmyStack {
        count: 1,
        ..game.adventure().hero(red).unwrap().army[0]
    };
    let survivor = game.adventure().hero(blue).unwrap().army[0];
    let given = outcome(
        BattleResult::Surrender(BattleSide::Attacker),
        [&[kept], &[survivor]],
        0,
    );
    // Slot 0 marked: the return goes into slot 1 whatever the draw; both marked, the drawn one's
    // other, so a third return replaces one of two (TD-18).
    for marks in [[true, false], [true, true]] {
        let mut marked = game.clone();
        let pool = Arc::make_mut(marked.taverns().pool);
        pool.tavern_mut(RED).unwrap().returning = marks;
        let (after, _) = ended(&rules, &marked, (red, blue), &given);
        let mut stream = game.stream(RED);
        let drawn = usize::try_from(stream.range_inclusive(0, 1)).unwrap();
        let slot = if marks == [true, false] { 1 } else { 1 - drawn };
        let tavern = after.pool().tavern(RED).unwrap();
        assert_eq!(tavern.slots[slot], Some(red_type));
        assert_eq!(tavern.returning, [true, true]);
        assert_eq!(after.pool().away(red_type).unwrap().army, [kept]);
        assert_eq!(after.stream(RED), stream);
    }
    // A surrender whose survivors are none (its war machines or created stacks alone paid for
    // it): one creature, as a retreat (the agent's call).
    let none = outcome(
        BattleResult::Surrender(BattleSide::Attacker),
        [&[], &[survivor]],
        0,
    );
    let (after, _) = ended(&rules, &game, (red, blue), &none);
    let away = after.pool().away(red_type).unwrap();
    assert_eq!(away.army, [one_of(&rules, &after, red)]);
}

// covers: TWN-TAV-006
#[test]
fn a_repeated_types_other_hero_leaves_an_offered_record_as_it_is() {
    let rules = rules();
    let game = game(&rules, &towns_setup(&rules), 5000);
    // The setup's two heroes, red's 0 and blue's 1, are both of type 0, whose record is red's.
    let (red, blue) = (HeroId::new(0), HeroId::new(1));
    let ty = game.hero(red).unwrap().hero_type;
    assert_eq!(game.hero(blue).unwrap().hero_type, ty);
    assert_eq!(game.pool().record(ty), Some(red));
    // Red retreats: its record returns, offered; blue, left with created stacks only, leaves
    // after it and changes nothing of the type's (decision 146).
    let fled = outcome(BattleResult::Retreat(BattleSide::Attacker), [&[], &[]], 0);
    let (after, _) = ended(&rules, &game, (red, blue), &fled);
    let pool = after.pool();
    assert_eq!(pool.status(ty), Some(HeroStatus::Offered));
    assert_eq!(pool.record(ty), Some(red));
    assert!(pool.tavern(RED).unwrap().slots.contains(&Some(ty)));
    let away = pool.away(ty).unwrap();
    assert_eq!(away.army, [one_of(&rules, &after, red)]);
    let on_map = game.adventure().hero(red).unwrap();
    assert_eq!(away.movement, on_map.movement);
    assert!(away.stale);
}
