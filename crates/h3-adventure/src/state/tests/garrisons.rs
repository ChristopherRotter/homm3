//! Increment 12's kinds on the map before their visits (E08 T113a, increment 12's design note
//! §19's round 5): a garrison's and each bonus kind's tile is refused as any unbuilt visit's
//! (ADV-OBJ-003), entered or visited from a neighbour, and the reach ends before it. The
//! garrisons' visits come with T114a, the bonus visitables' with T115b.

use std::sync::Arc;

use super::teleporters::{HERO, ONTO, Rules, jump_state, p, walk};
use super::*;
use crate::ruleset::{BONUS_DAYS, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{hero_details, key, movement_rules_with, stream};

/// [`movement_rules_with`] (faithful) with one type of each of increment 12's seven kinds: the
/// garrison `t:garrison` (anti-magic), entered and from above, a bonus `t:buoy` visited from a
/// neighbour, and `t:fountain`, `t:well`, `t:spring`, `t:stables` and `t:sirens` entered.
fn bonus_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |_| {},
        |part| {
            let ty = |k: &str, visit, from_above, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit,
                from_above,
                kind_mark: true,
                kind,
            };
            part.object_types.extend([
                ty(
                    "t:garrison",
                    Visit::Enter,
                    true,
                    ObjectKindDef::Garrison { anti_magic: true },
                ),
                ty(
                    "t:buoy",
                    Visit::Neighbour,
                    true,
                    ObjectKindDef::Bonus {
                        mark: 2,
                        days: [(1, 0); BONUS_DAYS],
                        points: 0,
                        drain: false,
                    },
                ),
                ty(
                    "t:fountain",
                    Visit::Enter,
                    false,
                    ObjectKindDef::FountainOfFortune {
                        mark: 5,
                        values: vec![-1, 1, 2, 3],
                    },
                ),
                ty("t:well", Visit::Enter, false, ObjectKindDef::MagicWell),
                ty("t:spring", Visit::Enter, false, ObjectKindDef::MagicSpring),
                ty(
                    "t:stables",
                    Visit::Enter,
                    false,
                    ObjectKindDef::Stables {
                        points: 400,
                        from: key("t:pikeman"),
                        to: key("t:gnoll"),
                    },
                ),
                ty(
                    "t:sirens",
                    Visit::Neighbour,
                    false,
                    ObjectKindDef::Sirens {
                        mark: 13,
                        keep: (7, 10),
                    },
                ),
            ]);
        },
    )
}

// covers: ADV-GAR-001, ADV-BON-001
#[test]
fn a_step_onto_a_garrison_or_a_bonus_visitable_is_refused_until_its_visit() {
    let rules = bonus_rules();
    let heroes = [Arc::new(hero_details(&[]))];
    for k in [
        "t:garrison",
        "t:buoy",
        "t:fountain",
        "t:well",
        "t:spring",
        "t:stables",
        "t:sirens",
    ] {
        // The object on (1, 1), visited from it; the hero on (0, 1) beside it.
        let mut state = jump_state(&rules, (1, None), &[], &[(k, ONTO[0])]);
        assert_eq!(
            walk(&mut state, &rules, &ONTO[..1], &mut stream()).err(),
            Some(AdventureCommandError::Visit { step: 0 }),
            "{k}"
        );
        // The reach ends before it: no path onto or through its tile.
        let reach = state.reach(&rules.0, &rules.1, &heroes, HERO).unwrap();
        assert_eq!(reach.path(ONTO[0]), None, "{k}");
        assert!(reach.path(p(2, 1)).is_some(), "{k}");
    }
}
