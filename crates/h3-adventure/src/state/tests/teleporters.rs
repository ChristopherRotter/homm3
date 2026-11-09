//! Increment 11's teleporters on the map before their jump (E08 T103a, increment 11's design note
//! §1, §7): typed objects whose visit is not built yet, so a step onto an entrance is refused as
//! any such visit's (`Visit`) and the reachable tiles end before it, until T105a. Since E08 T104
//! (design note §5) the build's channels and gates' pairs, and its refusal of a teleporter off its
//! surface (A-42).

use super::*;
use crate::ruleset::{MonolithWay, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{
    PIKEMAN, hero_at, hero_details, key, movement_rules_with, started, stream, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const HERO: HeroId = HeroId::new(1);

/// [`movement_rules_with`] (faithful) with a two-way monolith `t:monolith`, a Subterranean Gate
/// `t:underground_gate` (the test rules hold a border gate `t:gate`) and a Whirlpool
/// `t:whirlpool`, all entered, the Whirlpool from above; since E08 T104 a one-way entrance
/// `t:entrance` and exit `t:exit` of channel 0, an exit `t:exit_1` and a two-way monolith
/// `t:two_way_1` of channel 1.
fn teleporter_rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |_| {},
        |part| {
            let ty = |k: &str, from_above, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Enter,
                from_above,
                kind,
            };
            part.object_types.extend([
                ty(
                    "t:monolith",
                    false,
                    ObjectKindDef::Monolith {
                        channel: 0,
                        way: MonolithWay::TwoWay,
                    },
                ),
                ty("t:underground_gate", false, ObjectKindDef::SubterraneanGate),
                ty("t:whirlpool", true, ObjectKindDef::Whirlpool { divisor: 2 }),
            ]);
            let monolith =
                |k: &str, channel, way| ty(k, false, ObjectKindDef::Monolith { channel, way });
            part.object_types.extend([
                monolith("t:entrance", 0, MonolithWay::Entrance),
                monolith("t:exit", 0, MonolithWay::Exit),
                monolith("t:exit_1", 1, MonolithWay::Exit),
                monolith("t:two_way_1", 1, MonolithWay::TwoWay),
            ]);
        },
    )
}

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

// covers: ADV-TEL-001, ADV-OBJ-003
#[test]
fn a_step_onto_a_teleporters_entrance_is_refused_until_its_jump_is_built() {
    let rules = teleporter_rules();
    // Since E08 T104 a Whirlpool lies on water (A-42, the last test here): a boat's step onto it
    // comes with its jump (T105a).
    for k in ["t:monolith", "t:underground_gate"] {
        // Red's hero on (0, 0); the teleporter's entrance, blocked and visitable, on (2, 0).
        let mut setup = world(&["gggg"], 1, vec![hero_at(1, 0, p(0, 0), &[PIKEMAN])]);
        setup.map.objects.push(ObjectSetup {
            scenery: Scenery::default(),
            anchor: p(2, 0),
            blocked: TileMask::from_bits(1),
            visitable: TileMask::from_bits(1),
            object_type: rules.1.object_type_by_key(k),
            amount: None,
            drawn: TileMask::default(),
            monster: None,
            guards: None,
            holds: Holds::Nothing,
            owner: None,
        });
        let mut state = started(&rules, &setup).unwrap();
        let heroes = [Arc::new(hero_details(&[])), Arc::new(hero_details(&[]))];
        // The reachable tiles end before it: the entrance is no destination yet.
        let reach = state.reach(&rules.0, &rules.1, &heroes, HERO).unwrap();
        assert_eq!(reach.path(p(1, 0)), Some(vec![p(1, 0)]), "{k}");
        assert_eq!(reach.path(p(2, 0)), None, "{k}");
        // A step onto it is refused and changes nothing.
        let before = hash_of(&state);
        let walk = AdventureCommand::MoveHero {
            hero: HERO,
            path: vec![p(1, 0), p(2, 0)],
        };
        let mut events = Vec::new();
        assert_eq!(
            state.apply(
                &rules.0,
                &rules.1,
                &heroes,
                (RED, &mut stream()),
                &walk,
                &mut events
            ),
            Err(AdventureCommandError::Visit { step: 1 }),
            "{k}"
        );
        assert_eq!(hash_of(&state), before, "{k}");
        assert!(events.is_empty(), "{k}");
    }
}

/// An object of type `k` (none: an untyped blocker) anchored on `anchor`, blocking and visited
/// from the tiles of `visitable`.
fn object(rules: &Rules, k: Option<&str>, anchor: Pos, visitable: u64) -> ObjectSetup {
    ObjectSetup {
        scenery: Scenery::default(),
        anchor,
        blocked: TileMask::from_bits(visitable),
        visitable: TileMask::from_bits(visitable),
        object_type: k.map(|k| rules.1.object_type_by_key(k).unwrap()),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// The static map `setup` builds to under `rules`.
fn built(rules: &Rules, setup: &AdventureSetup) -> Arc<StaticMap> {
    Arc::clone(&started(rules, setup).unwrap().map)
}

fn at(x: u16, y: u16, level: u8) -> Pos {
    Pos::new(x, y, level)
}

// covers: ADV-TEL-002
#[test]
fn the_channels_list_each_ways_tiles_in_the_walks_order() {
    let rules = teleporter_rules();
    let mut setup = world(&["gggggg"; 8], 2, vec![]);
    let o = |k, anchor, visitable| object(&rules, k, anchor, visitable);
    setup.map.objects.extend([
        // Channel 0's exits in map order: the walk's order is level, row, column, so (5, 0) comes
        // before (4, 3) on the surface, and the underground's after both (`Pos`' order, column
        // first, would take the underground's (1, 0) first).
        o(Some("t:exit"), at(4, 3, 0), 1),
        o(Some("t:exit"), at(1, 0, 1), 1),
        o(Some("t:exit"), at(5, 0, 0), 1),
        // An entrance is in no list.
        o(Some("t:entrance"), at(0, 0, 0), 1),
        // Channel 0's two-way monoliths: (4, 1) before (1, 2).
        o(Some("t:monolith"), at(1, 2, 0), 1),
        o(Some("t:monolith"), at(4, 1, 0), 1),
        // Channel 1's: a one-way and a two-way channel of one number are distinct.
        o(Some("t:two_way_1"), at(3, 3, 1), 1),
        o(Some("t:exit_1"), at(2, 1, 1), 1),
        // An exit under a blocker visited from its tile, placed after it: the tile is the
        // blocker's (the original's tile class, the last object placed), in no list.
        o(Some("t:exit"), at(3, 1, 0), 1),
        o(None, at(3, 1, 0), 1),
        // An exit visited from a tile off the map alone (one column left of the first): none.
        o(Some("t:exit"), at(0, 2, 0), 2),
    ]);
    let map = built(&rules, &setup);
    let channels = map.channels();
    assert!(!channels.is_empty());
    assert_eq!(channels.exits(0), [at(5, 0, 0), at(4, 3, 0), at(1, 0, 1)]);
    assert_eq!(channels.two_way(0), [at(4, 1, 0), at(1, 2, 0)]);
    assert_eq!(channels.exits(1), [at(2, 1, 1)]);
    assert_eq!(channels.two_way(1), [at(3, 3, 1)]);
    assert!(channels.exits(2).is_empty() && channels.two_way(2).is_empty());
    let one_way: Vec<_> = channels.one_way_channels().collect();
    assert_eq!(
        one_way,
        [
            (0, &[at(5, 0, 0), at(4, 3, 0), at(1, 0, 1)][..]),
            (1, &[at(2, 1, 1)][..])
        ]
    );
    let two_way: Vec<_> = channels.two_way_channels().collect();
    assert_eq!(
        two_way,
        [
            (0, &[at(4, 1, 0), at(1, 2, 0)][..]),
            (1, &[at(3, 3, 1)][..])
        ]
    );
    assert!(channels.whirlpools().is_empty() && channels.gates().is_empty());
    // Built without a draw: the setup's stream stands where a map without them leaves it.
    let mut draws = draws();
    AdventureState::new(&rules.0, &rules.1, &setup, &mut draws).unwrap();
    let mut bare = crate::testing::draws();
    AdventureState::new(
        &rules.0,
        &rules.1,
        &world(&["gggggg"; 8], 2, vec![]),
        &mut bare,
    )
    .unwrap();
    assert_eq!(draws, bare);
}

// covers: ADV-TEL-002
#[test]
fn the_whirlpools_list_every_tile_with_its_whirlpool() {
    let rules = teleporter_rules();
    let mut setup = world(&["~~~~~~~~~~", "~~~~~~~~~~", "gggggggggg"], 1, vec![]);
    // Two Whirlpools of 3 × 2 tiles side by side, the second first in map order: the walk takes
    // row 0 of both, then row 1.
    let six = 0x0707;
    setup.map.objects.extend([
        object(&rules, Some("t:whirlpool"), at(8, 1, 0), six),
        object(&rules, Some("t:whirlpool"), at(3, 1, 0), six),
    ]);
    let map = built(&rules, &setup);
    let tiles: Vec<(Pos, usize)> = map
        .channels()
        .whirlpools()
        .iter()
        .map(|w| (w.tile, w.object))
        .collect();
    let row = |y, from: u16, object| (from..from + 3).map(move |x| (at(x, y, 0), object));
    let expected: Vec<_> = row(0, 1, 1)
        .chain(row(0, 6, 0))
        .chain(row(1, 1, 1))
        .chain(row(1, 6, 0))
        .collect();
    assert_eq!(tiles, expected);
}

// covers: ADV-TEL-002
#[test]
fn the_gates_pair_greedily_in_the_walks_order_by_the_truncated_distance() {
    let rules = teleporter_rules();
    let mut setup = world(&["gggggggggggggggg"; 8], 2, vec![]);
    let gate = |anchor| object(&rules, Some("t:underground_gate"), anchor, 1);
    // In map order, which the walk reorders: on the surface A (0, 0), B (4, 0), E (14, 3);
    // underground C (3, 0), D (10, 0), F (12, 2), G (12, 3).
    let [a, b, e] = [at(0, 0, 0), at(4, 0, 0), at(14, 3, 0)];
    let [c, d, f, g] = [at(3, 0, 1), at(10, 0, 1), at(12, 2, 1), at(12, 3, 1)];
    setup.map.objects.extend([g, e, d, a, c, f, b].map(gate));
    let map = built(&rules, &setup);
    let channels = map.channels();
    let pairs: Vec<(Pos, Option<Pos>)> =
        channels.gates().iter().map(|g| (g.tile, g.pair)).collect();
    assert_eq!(
        pairs,
        [
            // A's nearest is C (3); B's would be C too (1), but A took it: D (6), the greedy
            // order the original pairs in (survey §1: 18 shipped gates not with their nearest).
            (a, Some(c)),
            (b, Some(d)),
            // E is ⌊√5⌋ = 2 from F and ⌊√4⌋ = 2 from G: a tie, so F, the first in the walk's
            // order, though G is nearer (VCMI's and a squared distance's pick).
            (e, Some(f)),
            (c, Some(a)),
            (d, Some(b)),
            (f, Some(e)),
            // G is left over: no gate after it on another level.
            (g, None),
        ]
    );
    assert_eq!(channels.gate_pair(b), Some(d));
    assert_eq!(channels.gate_pair(f), Some(e));
    assert_eq!(channels.gate_pair(g), None);
    assert_eq!(channels.gate_pair(at(1, 0, 0)), None);
    // Gates of one level never pair.
    let mut setup = world(&["gggggggggggggggg"; 8], 2, vec![]);
    setup.map.objects.extend([a, b, e].map(gate));
    let map = built(&rules, &setup);
    assert!(map.channels().gates().iter().all(|g| g.pair.is_none()));
}

// covers: ADV-TEL-001, ADV-TEL-002
#[test]
fn a_teleporter_off_its_surface_is_refused_when_the_game_is_built() {
    let rules = teleporter_rules();
    // Grass, sea and rock.
    let rows = ["gg~~##"];
    let o = |k, x, visitable| object(&rules, Some(k), at(x, 0, 0), visitable);
    // On its surface (a tile off the map ignored: the second mask's left tile).
    let fine = [o("t:monolith", 0, 3), o("t:whirlpool", 2, 1)];
    let mut setup = world(&rows, 1, vec![]);
    setup.map.objects.extend(fine.clone());
    assert!(started(&rules, &setup).is_ok());
    for (k, x) in [
        ("t:monolith", 2),
        ("t:entrance", 4),
        ("t:underground_gate", 3),
        ("t:whirlpool", 1),
        ("t:whirlpool", 5),
    ] {
        let mut setup = world(&rows, 1, vec![]);
        setup.map.objects.extend(fine.clone());
        // Visited from its anchor and the tile left of it, one of them off its surface.
        setup.map.objects.push(o(k, x, 3));
        assert_eq!(
            started(&rules, &setup).err(),
            Some(SetupError::TeleporterSurface(2)),
            "{k} on {x}"
        );
    }
    assert_eq!(
        SetupError::TeleporterSurface(2).to_string(),
        "teleporter 2 has a visitable tile off its surface"
    );
    // Without teleporters the channels are empty.
    let map = built(&rules, &world(&rows, 1, vec![]));
    assert!(map.channels().is_empty());
    assert_eq!(*map.channels(), Channels::default());
}
