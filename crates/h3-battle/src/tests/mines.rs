//! Land Mine (E04b T5c2): hidden mines that go off under a stack of the other side that enters
//! them unseen (BAT-MAG-033), on T5c1's hidden obstacles (BAT-MAG-031, quirk BAT-QUIRK-1), after
//! the original's code (`docs/formats/installation.md` §7.17).

use std::vec::Vec;

use h3_content::{ObstacleKind, Ruleset};
use h3_contracts::{BattleHero, BattleResult, BattleSetup, BattleSide};

use super::spells::{arena, mage, spell};
use super::support::*;
use crate::{
    BattleState, CastTarget, Command, CommandError, Event, EventKind, Hex, HexSet, StackId, Status,
    Visibility, castable_spells, observe,
};

/// A mage with `spells`, the masteries `[air, fire]` and 100 mana.
fn caster(r: &Ruleset, spells: &[&str], [air, fire]: [u8; 2]) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = air;
    h.magic.schools[1] = fire;
    h.magic.mana = 100;
    h
}

/// Places `side`'s mines of the value 40 on `cells` as its cast would (test setup only; the cast
/// draws its hexes at random).
fn mines(s: &mut BattleState, r: &Ruleset, side: BattleSide, cells: &[(i32, i32)]) {
    let id = spell(r, "landmine");
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let hexes: Vec<Hex> = cells.iter().map(|&(x, y)| at(r, x, y)).collect();
    crate::obstacles::place(
        s,
        (id, ObstacleKind::Mine, &shape),
        side,
        (&hexes, 40),
        &mut Vec::new(),
    );
}

/// The command to cast Land Mine.
fn land_mine(r: &Ruleset) -> Command {
    Command::Cast {
        spell: spell(r, "landmine"),
        target: CastTarget::None,
        second: CastTarget::None,
    }
}

/// The hexes of the mines `viewer` observes.
fn seen(s: &BattleState, r: &Ruleset, viewer: BattleSide) -> HexSet {
    observe(s, r, viewer)
        .spell_obstacles
        .iter()
        .filter(|o| o.kind == ObstacleKind::Mine)
        .fold(HexSet::new(), |set, o| set.union(o.hexes))
}

/// The hexes of every mine, seen or not.
fn all(s: &BattleState) -> HexSet {
    s.spell_obstacles()
        .iter()
        .filter(|o| o.kind == ObstacleKind::Mine)
        .fold(HexSet::new(), |set, o| set.union(o.hexes))
}

/// The kinds of the obstacle events, with their visibility, in order.
fn obstacle_events(events: &[Event]) -> Vec<(&'static str, Visibility)> {
    events
        .iter()
        .filter_map(|e| {
            let name = match e.kind {
                EventKind::ObstaclePlaced { .. } => "placed",
                EventKind::ObstacleRemoved { .. } => "removed",
                EventKind::ObstacleRevealed { .. } => "revealed",
                EventKind::ObstacleTriggered { .. } => "triggered",
                _ => return None,
            };
            Some((name, e.visibility))
        })
        .collect()
}

/// What a mine that goes off reports: revealed, triggered, removed, all to both sides.
const BOOM: [(&str, Visibility); 3] = [
    ("revealed", Visibility::Public),
    ("triggered", Visibility::Public),
    ("removed", Visibility::Public),
];

/// The damage `events` report, as `(stack, hex, damage, kills)`: the hex of the obstacle that
/// did it.
fn blasts(events: &[Event]) -> Vec<(u8, Hex, u64, u32)> {
    let mut out = Vec::new();
    let mut hex = None;
    for e in events {
        match e.kind {
            EventKind::ObstacleTriggered { hex: h, .. } => hex = Some(h),
            EventKind::SpellDamage {
                target,
                damage,
                kills,
                ..
            } => out.push((target.0, hex.take().unwrap(), damage, kills)),
            _ => {}
        }
    }
    out
}

/// The moves `events` report, `(stack, from, to)`.
fn moves(events: &[Event]) -> Vec<(u8, Hex, Hex)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Moved { stack, from, to } => Some((stack.0, from, to)),
            _ => None,
        })
        .collect()
}

/// A walk of the active stack to `(x, y)`.
fn walk(r: &Ruleset, (x, y): (i32, i32)) -> Command {
    Command::Move { to: at(r, x, y) }
}

/// A battle on the plain (Castle's native terrain) with heroes and the stacks placed at `(x, y)`,
/// as [`arena`]; the first attacker stack is active.
fn on_plain(
    r: &Ruleset,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
) -> BattleState {
    let specs = |list: &[(&'static str, u32, i32, i32)]| -> Vec<Spec> {
        list.iter()
            .enumerate()
            .map(|(i, &(name, count, ..))| (u8::try_from(i).unwrap(), name, count))
            .collect()
    };
    let setup = BattleSetup {
        heroes,
        terrain: r.terrain_by_key("t:plain"),
        ..setup(r, &specs(attacker), &specs(defender))
    };
    let mut s = BattleState::new(r, &setup, 1).unwrap();
    for (i, &(_, _, x, y)) in attacker.iter().chain(defender).enumerate() {
        place(&mut s, r, u8::try_from(i).unwrap(), x, y);
    }
    activate(&mut s, 0);
    s
}

// covers: BAT-MAG-033
#[test]
fn land_mine_hides_mines_by_mastery_with_the_spells_value_for_the_whole_battle() {
    let r = ruleset();
    for (mastery, count, value) in [(0, 4, 55), (1, 4, 55), (2, 6, 80), (3, 8, 130)] {
        let hero = caster(&r, &["landmine"], [0, mastery]);
        let mut s = arena(
            &r,
            [Some(hero), None],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 12, 5)],
            u64::from(mastery) + 3,
        );
        s.natural.block(at(&r, 8, 8));
        let events = act(&mut s, &r, land_mine(&r)).unwrap();
        let field = field(&r);
        let mut hexes = HexSet::new();
        for o in s.spell_obstacles() {
            // One hex each, the caster's, hidden from the other side, with the spell's value
            // (power 3 × 10 + 25, 25, 50, 100), for the whole battle.
            assert_eq!(
                (o.kind, o.side, o.hexes.len(), o.rounds, o.value, o.revealed),
                (ObstacleKind::Mine, BattleSide::Attacker, 1, 0, value, false)
            );
            let hex = o.hexes.iter().next().unwrap();
            assert!(field.is_standable(hex) && !hexes.contains(hex));
            assert!(![at(&r, 2, 5), at(&r, 12, 5), at(&r, 8, 8)].contains(&hex));
            hexes.insert(hex);
        }
        assert_eq!(hexes.len(), count, "mastery {mastery}");
        // Told to the caster only; the other side sees none.
        assert_eq!(
            obstacle_events(&events),
            std::vec![("placed", Visibility::Side(BattleSide::Attacker)); usize::try_from(count).unwrap()]
        );
        assert_eq!(seen(&s, &r, BattleSide::Attacker), hexes);
        assert!(seen(&s, &r, BattleSide::Defender).is_empty());
        let mut later = s.clone();
        while later.round() < 4 {
            act(&mut later, &r, Command::Defend).unwrap();
        }
        assert_eq!(later.spell_obstacles(), s.spell_obstacles());
    }
}

// covers: BAT-MAG-033, BAT-MAG-031
#[test]
fn land_mine_is_refused_while_the_other_sides_natives_would_see_the_mines() {
    let castable = |r: &Ruleset, s: &BattleState| {
        castable_spells(s, r, BattleSide::Attacker).contains(spell(r, "landmine"))
    };
    let kill = |s: &mut BattleState, id: usize| {
        s.stacks.as_mut_slice()[id].count = 0;
        s.stacks.as_mut_slice()[id].top_hp = 0;
    };
    for profile in ["t:sight", "t:quirky"] {
        let r = ruleset_with(profile, |_| {});
        let hero = || Some(caster(&r, &["landmine"], [0, 1]));
        // Castle's Pikemen fight on their native plain for the defender: refused.
        let mut s = on_plain(
            &r,
            [hero(), None],
            &[("pikeman_c", 5, 2, 5)],
            &[("pikeman_c", 5, 12, 5), ("fixed", 5, 12, 7)],
        );
        assert!(!castable(&r, &s));
        assert_eq!(
            act(&mut s.clone(), &r, land_mine(&r)),
            Err(CommandError::InvalidSpellTarget)
        );
        // The native dies: with BAT-QUIRK-1 the start decides (still refused), without it the
        // living natives (cast).
        kill(&mut s, 1);
        let at_start = profile == "t:sight";
        assert_eq!(castable(&r, &s), !at_start, "{profile}");
        // The caster's own natives change nothing: without the defender's, it is cast.
        let mut s = on_plain(
            &r,
            [hero(), None],
            &[("pikeman_c", 5, 2, 5)],
            &[("fixed", 5, 12, 5)],
        );
        assert!(castable(&r, &s));
        act(&mut s, &r, land_mine(&r)).unwrap();
        assert!(seen(&s, &r, BattleSide::Defender).is_empty());
        assert_eq!(all(&s).len(), 4);
    }
}

// covers: BAT-MAG-033
#[test]
fn an_enemy_walker_sets_off_an_unseen_mine_and_walks_on() {
    let r = ruleset();
    let heroes = [None, Some(caster(&r, &["landmine"], [0, 1]))];
    let mut s = arena(
        &r,
        heroes,
        &[("fixed", 10, 4, 5)],
        &[("fixed", 5, 14, 9)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5), (9, 9)]);
    assert!(seen(&s, &r, BattleSide::Attacker).is_empty());
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    // 40 damage, four of ten creatures; reported in two moves, split where it went off.
    assert_eq!(blasts(&events), [(0, at(&r, 6, 5), 40, 4)]);
    assert_eq!(obstacle_events(&events), BOOM);
    assert_eq!(
        moves(&events),
        [
            (0, at(&r, 4, 5), at(&r, 6, 5)),
            (0, at(&r, 6, 5), at(&r, 8, 5))
        ]
    );
    assert_eq!((s.stacks()[0].count, s.stacks()[0].hex), (6, at(&r, 8, 5)));
    // That mine is gone, the other stays hidden.
    assert_eq!(all(&s), {
        let mut h = HexSet::new();
        h.insert(at(&r, 9, 9));
        h
    });
    assert!(seen(&s, &r, BattleSide::Attacker).is_empty());
    // A stack that dies on a mine stops there; the battle may end.
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 4, 4, 5)],
        &[("fixed", 5, 14, 9)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5)]);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert_eq!(blasts(&events), [(0, at(&r, 6, 5), 40, 4)]);
    assert_eq!(moves(&events), [(0, at(&r, 4, 5), at(&r, 6, 5))]);
    assert!(events.contains(&Event::public(EventKind::BattleEnded {
        result: BattleResult::Victory(BattleSide::Defender)
    })));
}

// covers: BAT-MAG-033
#[test]
fn a_stack_that_survives_a_mine_still_attacks() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 10, 4, 5)],
        &[("fixed", 5, 8, 5)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5)]);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 7, 5),
        },
    )
    .unwrap();
    assert_eq!(blasts(&events), [(0, at(&r, 6, 5), 40, 4)]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Struck { .. }))
    );
}

// covers: BAT-MAG-033
#[test]
fn own_stacks_and_immune_stacks_cross_mines_without_a_trace() {
    let r = ruleset();
    // The defender's walker over its own mine, the fire-immune Ember over the other side's:
    // one move each, nothing reported, the mine stays hidden from the attacker.
    let mut s = arena(
        &r,
        [None, None],
        &[("ember", 5, 4, 5)],
        &[("fixed", 5, 10, 7)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5), (8, 7)]);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert!(obstacle_events(&events).is_empty());
    assert_eq!(moves(&events), [(0, at(&r, 4, 5), at(&r, 8, 5))]);
    activate(&mut s, 1);
    let events = act(&mut s, &r, walk(&r, (6, 7))).unwrap();
    assert!(obstacle_events(&events).is_empty());
    assert_eq!(moves(&events), [(1, at(&r, 10, 7), at(&r, 6, 7))]);
    assert_eq!(all(&s).len(), 2);
    assert!(seen(&s, &r, BattleSide::Attacker).is_empty());
}

// covers: BAT-MAG-033
#[test]
fn a_flyer_sets_off_only_the_mine_it_lands_on() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("flyer", 10, 4, 5)],
        &[("fixed", 5, 14, 9)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5), (8, 5)]);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    // 40 damage of 8 hit points each: five of ten.
    assert_eq!(blasts(&events), [(0, at(&r, 8, 5), 40, 5)]);
    assert_eq!(moves(&events), [(0, at(&r, 4, 5), at(&r, 8, 5))]);
    assert_eq!(all(&s).len(), 1);
}

// covers: BAT-MAG-033, BAT-MAG-028
#[test]
fn a_stack_standing_on_a_mine_sets_it_off_when_it_acts_there() {
    let r = ruleset();
    // Placed on the mine without entering it (as Teleport does): defending enters it again.
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 10, 6, 5)],
        &[("fixed", 5, 14, 9)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5)]);
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(blasts(&events), [(0, at(&r, 6, 5), 40, 4)]);
    assert_eq!(obstacle_events(&events), BOOM);
    // With the attacker's Fire Wall over the defender's unseen mine, a walker entering that hex
    // burns first, then the mine goes off.
    let heroes = [Some(caster(&r, &["firewall"], [0, 1])), None];
    let mut s = arena(
        &r,
        heroes,
        &[("fixed", 20, 2, 9)],
        &[("fixed", 5, 14, 9)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5)]);
    act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "firewall"),
            target: CastTarget::Hex(at(&r, 6, 5)),
            second: CastTarget::None,
        },
    )
    .unwrap();
    place(&mut s, &r, 0, 4, 5);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    // The fire's 40 (power 3 × 10 + 10) kills four, then the mine's 40 four more.
    assert_eq!(
        blasts(&events),
        [(0, at(&r, 6, 5), 40, 4), (0, at(&r, 6, 5), 40, 4)]
    );
    let triggered: Vec<_> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::ObstacleTriggered { spell, .. } => Some(spell),
            _ => None,
        })
        .collect();
    assert_eq!(triggered, [spell(&r, "firewall"), spell(&r, "landmine")]);
}

// covers: BAT-MAG-033
#[test]
fn a_two_hex_walker_sets_off_a_mine_once() {
    let r = ruleset();
    // Its head enters (6, 5); its tail, which follows, enters no hex twice in one move.
    let mut s = arena(
        &r,
        [None, None],
        &[("wide", 4, 4, 5)],
        &[("fixed", 5, 14, 9)],
        1,
    );
    mines(&mut s, &r, BattleSide::Defender, &[(6, 5)]);
    let events = act(&mut s, &r, walk(&r, (9, 5))).unwrap();
    assert_eq!(blasts(&events), [(0, at(&r, 6, 5), 40, 1)]);
    assert_eq!(s.stacks()[0].hex, at(&r, 9, 5));
}

// covers: BAT-MAG-033
#[test]
fn a_hypnotized_stack_is_spared_by_its_own_sides_mines_only() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 9)],
        &[("fixed", 10, 12, 5)],
        1,
    );
    // The attacker commands the defender's stack: the defender's mine spares it, the attacker's
    // does not.
    mines(&mut s, &r, BattleSide::Defender, &[(10, 5)]);
    mines(&mut s, &r, BattleSide::Attacker, &[(8, 5)]);
    let st = &mut s.stacks.as_mut_slice()[1];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    activate(&mut s, 1);
    let events = act(&mut s, &r, walk(&r, (7, 5))).unwrap();
    assert_eq!(blasts(&events), [(1, at(&r, 8, 5), 40, 4)]);
    assert_eq!(all(&s).len(), 1);
}

// covers: BAT-MAG-033, BAT-MAG-031
#[test]
fn a_mine_seen_through_living_natives_does_not_go_off() {
    let r = ruleset();
    // Without BAT-QUIRK-1 the defender's living Pikemen show it the attacker's mines (placed
    // here as a cast before they came would): its walkers cross them.
    let mut s = on_plain(
        &r,
        [None, None],
        &[("fixed", 5, 2, 9)],
        &[("fixed", 10, 12, 5), ("pikeman_c", 5, 14, 9)],
    );
    mines(&mut s, &r, BattleSide::Attacker, &[(10, 5), (9, 7)]);
    assert_eq!(seen(&s, &r, BattleSide::Defender).len(), 2);
    activate(&mut s, 1);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert!(obstacle_events(&events).is_empty());
    // The natives die: the next mine goes off.
    s.stacks.as_mut_slice()[2].count = 0;
    s.stacks.as_mut_slice()[2].top_hp = 0;
    activate(&mut s, 1);
    let events = act(&mut s, &r, walk(&r, (9, 7))).unwrap();
    assert_eq!(blasts(&events), [(1, at(&r, 9, 7), 40, 4)]);
}
