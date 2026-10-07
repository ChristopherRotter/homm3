//! Natural obstacles as pieces and Remove Obstacle (E04b T5b; OPEN-QUESTIONS Q55): the anchors
//! (BAT-GEO-003, quirk BAT-QUIRK-3), Remove Obstacle (BAT-MAG-034) and the random draws of
//! Quicksand and Land Mine (BAT-MAG-032, quirk BAT-QUIRK-4), after the original's code
//! (`docs/formats/installation.md` §7.15, §7.16).

use std::vec;
use std::vec::Vec;

use h3_content::{ObstacleKind, Ruleset};
use h3_contracts::{BattleHero, BattleSide, Obstacle};

use super::spells::{mage, spell};
use super::support::*;
use crate::{
    BattleSetup, BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind,
    Hex, HexSet, SetupError, Visibility, cast_targets, castable_spells, observe,
};

/// A mage with `spells`, air mastery `air` and 100 mana.
fn caster(r: &Ruleset, spells: &[&str], air: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = air;
    h.magic.mana = 100;
    h
}

/// A battle with the attacker's hero `hero`, a serf per side at (2, 5) and (14, 5), and the
/// natural obstacles `obstacles` given as `(hexes, anchor)` cells; the attacker is active.
fn field_with(
    r: &Ruleset,
    hero: BattleHero,
    obstacles: &[(&[(i32, i32)], Option<(i32, i32)>)],
) -> BattleState {
    let setup = BattleSetup {
        heroes: [Some(hero), None],
        obstacles: obstacles
            .iter()
            .map(|&(hexes, anchor)| Obstacle {
                hexes: hexes.iter().map(|&(x, y)| at(r, x, y).0).collect(),
                anchor: anchor.map(|(x, y)| at(r, x, y).0),
            })
            .collect(),
        ..setup(r, &[(0, "serf", 1)], &[(0, "serf", 1)])
    };
    let mut s = BattleState::new(r, &setup, 1).unwrap();
    place(&mut s, r, 0, 2, 5);
    place(&mut s, r, 1, 14, 5);
    activate(&mut s, 0);
    s
}

/// The hexes `name` may be cast at by the attacker now.
fn targets(s: &BattleState, r: &Ruleset, name: &str) -> HexSet {
    let mut out = CastTargets::new();
    cast_targets(s, r, BattleSide::Attacker, spell(r, name), &mut out);
    out.iter().fold(HexSet::new(), |mut set, t| {
        if let CastTarget::Hex(h) = *t {
            set.insert(h);
        }
        set
    })
}

/// The hexes at `cells` as a set.
fn cells(r: &Ruleset, cells: &[(i32, i32)]) -> HexSet {
    cells.iter().fold(HexSet::new(), |mut set, &(x, y)| {
        set.insert(at(r, x, y));
        set
    })
}

/// The attacker casts `name` at `target` (its hero may cast again); returns the events.
fn cast_on(s: &mut BattleState, r: &Ruleset, name: &str, target: CastTarget) -> Vec<Event> {
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(s, 0);
    act(
        s,
        r,
        Command::Cast {
            spell: spell(r, name),
            target,
            second: CastTarget::None,
        },
    )
    .unwrap()
}

/// The attacker casts `name` at the hex `(x, y)`; returns the events.
fn cast_at(s: &mut BattleState, r: &Ruleset, name: &str, (x, y): (i32, i32)) -> Vec<Event> {
    cast_on(s, r, name, CastTarget::Hex(at(r, x, y)))
}

/// Places one obstacle of `kind` for `side` on `(x, y)` (as the spell would).
fn put(s: &mut BattleState, r: &Ruleset, side: BattleSide, kind: ObstacleKind, (x, y): (i32, i32)) {
    let key = match kind {
        ObstacleKind::Wall => "wall",
        ObstacleKind::Fire => "firewall",
        ObstacleKind::Pit => "quicksand",
        ObstacleKind::Mine => "landmine",
    };
    let id = spell(r, key);
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let hexes = [at(r, x, y)];
    crate::obstacles::place(s, (id, kind, &shape), side, (&hexes, 40), &mut Vec::new());
}

const STONE: &[(i32, i32)] = &[(7, 4), (8, 4)];
const CLIFF: &[(i32, i32)] = &[(10, 2), (10, 3)];

// covers: BAT-MAG-034, BAT-GEO-003
#[test]
fn remove_obstacle_takes_away_a_whole_natural_obstacle() {
    let r = ruleset();
    // A stone anchored beside its hexes and an absolute cliff (no anchor), without BAT-QUIRK-3.
    let mut s = field_with(
        &r,
        caster(&r, &["remove"], 0),
        &[(STONE, Some((6, 4))), (CLIFF, None)],
    );
    assert_eq!(s.removable_obstacles().len(), 1);
    assert_eq!(s.removable_obstacles()[0].anchor(), at(&r, 6, 4));
    // Without the quirk only the stone's own hexes are targets, at any mastery; never the cliff.
    assert_eq!(targets(&s, &r, "remove"), cells(&r, STONE));
    assert_eq!(
        act(
            &mut s.clone(),
            &r,
            Command::Cast {
                spell: spell(&r, "remove"),
                target: CastTarget::Hex(at(&r, 10, 3)),
                second: CastTarget::None,
            }
        ),
        Err(CommandError::InvalidSpellTarget)
    );
    // The cast takes the whole stone away; both sides are told and see it gone.
    let events = cast_at(&mut s, &r, "remove", (8, 4));
    assert!(
        events.contains(&Event::public(EventKind::NaturalObstacleRemoved {
            anchor: at(&r, 6, 4),
            hexes: cells(&r, STONE),
        }))
    );
    assert_eq!(s.obstacles(), cells(&r, CLIFF));
    assert!(s.removable_obstacles().is_empty());
    for viewer in BattleSide::BOTH {
        let o = observe(&s, &r, viewer);
        assert_eq!(o.obstacles, cells(&r, CLIFF));
        assert!(o.removable_obstacles.is_empty());
    }
    // Its hexes are free; nothing is left to remove, so the spell cannot be cast.
    activate(&mut s, 0);
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "remove")));
    place(&mut s, &r, 0, 7, 5);
    assert!(act(&mut s, &r, Command::Move { to: at(&r, 7, 4) }).is_ok());
}

// covers: BAT-GEO-003, BAT-MAG-034, BAT-MAG-027, BAT-MAG-032
#[test]
fn anchors_belong_to_their_obstacles_with_bat_quirk_3() {
    for quirk in [false, true] {
        let r = if quirk {
            ruleset_with("t:anchors", |_| {})
        } else {
            ruleset()
        };
        let hero = caster(&r, &["remove", "firewall", "wall", "quicksand"], 1);
        let mut s = field_with(&r, hero, &[(STONE, Some((6, 4)))]);
        let anchor = at(&r, 6, 4);
        // With the quirk the anchor leads Remove Obstacle to the stone and takes no Fire Wall or
        // Force Field; a stack may still stand on it.
        assert_eq!(targets(&s, &r, "remove").contains(anchor), quirk);
        assert_eq!(targets(&s, &r, "firewall").contains(anchor), !quirk);
        assert_eq!(targets(&s, &r, "wall").contains(anchor), !quirk);
        assert!(act(&mut s.clone(), &r, Command::Move { to: anchor }).is_ok());
        // Quicksand's pits never land on it with the quirk: only it and one other hex are free.
        let mut drawn = s.clone();
        let open = [anchor, at(&r, 12, 8)];
        for h in (0u8..187).map(Hex) {
            let taken = drawn.stacks().iter().any(|st| st.hex == h);
            if field(&r).is_standable(h) && !open.contains(&h) && !taken {
                drawn.natural.block(h);
            }
        }
        cast_on(&mut drawn, &r, "quicksand", CastTarget::None);
        let pits: Vec<HexSet> = drawn.spell_obstacles().iter().map(|o| o.hexes).collect();
        let expected: Vec<HexSet> = if quirk {
            vec![cells(&r, &[(12, 8)])]
        } else {
            vec![cells(&r, &[(6, 4)]), cells(&r, &[(12, 8)])]
        };
        let mut sorted = pits.clone();
        sorted.sort_by_key(|set| set.iter().next());
        assert_eq!(sorted, expected);
        if quirk {
            // Aimed at the anchor, Remove Obstacle takes the stone away, anchor and all.
            let events = cast_at(&mut s, &r, "remove", (6, 4));
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e.kind, EventKind::NaturalObstacleRemoved { .. }))
            );
            assert!(s.obstacles().is_empty() && s.removable_obstacles().is_empty());
            s.heroes[0].as_mut().unwrap().view.casts = 0;
            assert!(targets(&s, &r, "firewall").contains(anchor));
        }
    }
}

// covers: BAT-MAG-034
#[test]
fn an_anchor_hex_leads_to_the_obstacle_placed_last() {
    let r = ruleset_with("t:anchors", |_| {});
    // Two stones share the anchor (6, 4); a cliff (absolute) covers the anchor (9, 6) of a third.
    let mut s = field_with(
        &r,
        caster(&r, &["remove"], 0),
        &[
            (&[(7, 4)], Some((6, 4))),
            (&[(5, 4)], Some((6, 4))),
            (&[(9, 6)], None),
            (&[(10, 6)], Some((9, 6))),
        ],
    );
    let gone = |events: &[Event]| -> Vec<HexSet> {
        events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::NaturalObstacleRemoved { hexes, .. } => Some(hexes),
                _ => None,
            })
            .collect()
    };
    // The original's hex holds the index of the object placed last: the second stone, then the
    // first. The absolute cliff has no object, so its hex still leads to the third stone.
    let events = cast_at(&mut s, &r, "remove", (6, 4));
    assert_eq!(gone(&events), [cells(&r, &[(5, 4)])]);
    let events = cast_at(&mut s, &r, "remove", (6, 4));
    assert_eq!(gone(&events), [cells(&r, &[(7, 4)])]);
    let events = cast_at(&mut s, &r, "remove", (9, 6));
    assert_eq!(gone(&events), [cells(&r, &[(10, 6)])]);
    assert_eq!(s.obstacles(), cells(&r, &[(9, 6)]));
    assert!(targets(&s, &r, "remove").is_empty());
}

// covers: BAT-MAG-034
#[test]
fn remove_obstacle_finds_spell_obstacles_by_mastery_and_only_those_its_side_sees() {
    let r = ruleset();
    let (att, def) = (BattleSide::Attacker, BattleSide::Defender);
    for air in 0..=3u8 {
        let mut s = field_with(&r, caster(&r, &["remove"], air), &[]);
        put(&mut s, &r, att, ObstacleKind::Fire, (6, 2));
        put(&mut s, &r, att, ObstacleKind::Wall, (6, 4));
        put(&mut s, &r, att, ObstacleKind::Pit, (6, 6));
        put(&mut s, &r, def, ObstacleKind::Pit, (8, 6));
        put(&mut s, &r, def, ObstacleKind::Mine, (8, 8));
        // A pit of the other side that a walker ran into is seen.
        put(&mut s, &r, def, ObstacleKind::Pit, (10, 6));
        s.spell_obstacles.reveal_for_tests(at(&r, 10, 6));
        // Below Advanced no spell obstacle; Advanced Fire Wall; Expert every one it sees. The
        // other side's unseen pit and mine are never targets (Q-0002).
        let expected: &[(i32, i32)] = match air {
            0 | 1 => &[],
            2 => &[(6, 2)],
            _ => &[(6, 2), (6, 4), (6, 6), (10, 6)],
        };
        assert_eq!(
            targets(&s, &r, "remove"),
            cells(&r, expected),
            "mastery {air}"
        );
        assert_eq!(
            castable_spells(&s, &r, att).contains(spell(&r, "remove")),
            air >= 2
        );
    }
    // Expert: the cast takes the obstacle on its hex; its own pit is reported to its side only.
    let mut s = field_with(&r, caster(&r, &["remove"], 3), &[]);
    put(&mut s, &r, att, ObstacleKind::Pit, (6, 6));
    put(&mut s, &r, def, ObstacleKind::Mine, (6, 6));
    let events = cast_at(&mut s, &r, "remove", (6, 6));
    let removed: Vec<Visibility> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::ObstacleRemoved { .. }))
        .map(|e| e.visibility)
        .collect();
    assert_eq!(removed, [Visibility::Side(att)]);
    // The other side's mine on the same hex stays, unseen.
    assert_eq!(s.spell_obstacles().len(), 1);
    assert_eq!(s.spell_obstacles()[0].side, def);
}

// covers: BAT-MAG-032
#[test]
fn random_draws_avoid_hidden_obstacles_only_with_bat_quirk_4() {
    for quirk in [false, true] {
        let r = if quirk {
            ruleset_with("t:draws", |_| {})
        } else {
            ruleset()
        };
        let mut s = field_with(&r, caster(&r, &["quicksand"], 1), &[]);
        let open = [at(&r, 6, 6), at(&r, 12, 8)];
        for h in (0u8..187).map(Hex) {
            let taken = s.stacks().iter().any(|st| st.hex == h);
            if field(&r).is_standable(h) && !open.contains(&h) && !taken {
                s.natural.block(h);
            }
        }
        // The other side's unseen pit on (6, 6).
        put(&mut s, &r, BattleSide::Defender, ObstacleKind::Pit, (6, 6));
        cast_on(&mut s, &r, "quicksand", CastTarget::None);
        let own: Vec<HexSet> = s
            .spell_obstacles()
            .iter()
            .filter(|o| o.side == BattleSide::Attacker)
            .map(|o| o.hexes)
            .collect();
        // With the quirk (the original) the draw skips it and so gives it away; without it the
        // caster's pit may go over it.
        assert_eq!(own.len(), if quirk { 1 } else { 2 });
        assert!(own.iter().any(|h| *h == cells(&r, &[(12, 8)])));
    }
}

// covers: BAT-GEO-003
#[test]
fn obstacle_pieces_are_validated() {
    let r = ruleset();
    let start = |obstacles: Vec<Obstacle>| {
        let s = BattleSetup {
            obstacles,
            ..setup(&r, &[(0, "serf", 1)], &[(0, "serf", 1)])
        };
        BattleState::new(&r, &s, 1)
    };
    let piece = |hexes: Vec<u8>, anchor| Obstacle { hexes, anchor };
    // An anchor in a side column or off the field.
    assert_eq!(
        start(vec![piece(vec![40], Some(0))]).unwrap_err(),
        SetupError::InvalidAnchor(0)
    );
    assert_eq!(
        start(vec![piece(vec![40], Some(200))]).unwrap_err(),
        SetupError::InvalidAnchor(200)
    );
    // A hex blocked by two obstacles.
    assert_eq!(
        start(vec![piece(vec![40], None), piece(vec![40], Some(41))]).unwrap_err(),
        SetupError::InvalidObstacle(40)
    );
    // A removable obstacle of more than 8 hexes, or more than 16 removable obstacles.
    let row: Vec<u8> = (2..11).map(|x| at(&r, x, 2).0).collect();
    assert_eq!(
        start(vec![piece(row.clone(), Some(row[0]))]).unwrap_err(),
        SetupError::TooManyObstacles
    );
    assert!(start(vec![piece(row.clone(), None)]).is_ok());
    let many: Vec<Obstacle> = (2..15)
        .chain(2..6)
        .zip([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 8, 8, 8, 8])
        .map(|(x, y)| piece(vec![at(&r, x, y).0], Some(at(&r, x, y).0)))
        .collect();
    assert_eq!(many.len(), 17);
    assert_eq!(
        start(many.clone()).unwrap_err(),
        SetupError::TooManyObstacles
    );
    assert!(start(many[..16].to_vec()).is_ok());
    // An anchor may lie on another obstacle's hex, and a piece may block nothing.
    assert!(start(vec![piece(vec![40], None), piece(vec![], Some(40))]).is_ok());
}
