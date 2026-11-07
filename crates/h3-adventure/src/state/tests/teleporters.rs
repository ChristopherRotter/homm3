//! Increment 11's teleporters on the map before their jump (E08 T103a, increment 11's design note
//! §1, §7): typed objects whose visit is not built yet, so a step onto an entrance is refused as
//! any such visit's (`Visit`) and the reachable tiles end before it, until T105a.

use super::*;
use crate::ruleset::{MonolithWay, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{PIKEMAN, hero_at, hero_details, key, movement_rules_with, started, world};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const HERO: HeroId = HeroId::new(1);

/// [`movement_rules_with`] (faithful) with a two-way monolith `t:monolith`, a Subterranean Gate
/// `t:underground_gate` (the test rules hold a border gate `t:gate`) and a Whirlpool
/// `t:whirlpool`, all entered, the Whirlpool from above.
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
    for k in ["t:monolith", "t:underground_gate", "t:whirlpool"] {
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
            state.apply(&rules.0, &rules.1, &heroes, RED, &walk, &mut events),
            Err(AdventureCommandError::Visit { step: 1 }),
            "{k}"
        );
        assert_eq!(hash_of(&state), before, "{k}");
        assert!(events.is_empty(), "{k}");
    }
}
