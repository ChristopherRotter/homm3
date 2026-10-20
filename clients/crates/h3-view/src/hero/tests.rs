use super::*;
use alloc::vec;
use alloc::vec::Vec;

use crate::anim::tests::magic;
use crate::anim::{self, Member, Pose, Stage, Timeline};
use crate::record::tests::observation;
use crate::scene::Moment;
use crate::scene::tests::{layout, tables};

/// The groups of the asset tables.
fn groups() -> HeroGroups {
    tables().battle.hero_groups
}

/// A hero DEF's look: 1 standing frame, 4 of the idle motion, 11 of defeat and victory, 8 of the
/// cast, and a flag of 6 frames (the original's, design §4.5).
fn look() -> HeroLook {
    HeroLook {
        frames: [(0, 1), (1, 4), (2, 11), (3, 11), (4, 8)]
            .into_iter()
            .collect(),
        flag_frames: 6,
    }
}

/// An idle battle's timeline (its steps every 100 ms from 0), the attacker's stack 0 and the
/// defender's stack 1 taking turns as `turns` say, and `actions`.
fn idle(turns: &[(u32, u8)], actions: Vec<Action>) -> Timeline {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic(),
    };
    let mut timeline = anim::timeline(
        &observation(),
        &[],
        &BTreeMap::new(),
        stage,
        anim::Speed::new(0),
    )
    .unwrap();
    let start = Pose {
        group: 2,
        frame: 0,
        hex: 0,
        offset: [0, 0],
        mirrored: false,
        layer: 4,
        boxless: false,
    };
    timeline.members = [(0, Side::Attacker), (1, Side::Defender)]
        .map(|(stack, side)| Member {
            stack,
            side,
            record: 0,
            start,
        })
        .to_vec();
    timeline.turns = turns.to_vec();
    timeline.actions = actions;
    timeline
}

fn pose(group: u32, frame: u32, flag: u32) -> Option<HeroPose> {
    Some(HeroPose { group, frame, flag })
}

#[test]
fn the_flags_wave_and_the_hero_moves_after_standing_long_enough() {
    let heroes = Heroes::new([Some(look()), Some(look())], [true, false], groups(), None);
    assert!(heroes.any());
    let timeline = idle(&[], Vec::new());
    // The opening's step shows each flag's next frame (from 0 and 3), each step one more.
    assert_eq!(heroes.at(&timeline, 0), [pose(0, 0, 1), pose(0, 0, 4)]);
    assert_eq!(heroes.at(&timeline, 250), [pose(0, 0, 3), pose(0, 0, 0)]);
    // More than 4 500 ms after the start: the idle motion, a frame per step, then standing.
    let at = |t| heroes.at(&timeline, t)[0].map(|p| (p.group, p.frame));
    assert_eq!(at(4_500), Some((0, 0)));
    assert_eq!(at(4_600), Some((1, 0)));
    assert_eq!(at(4_900), Some((1, 3)));
    assert_eq!(at(5_000), Some((0, 0)));
    // Its clock starts again at its end.
    assert_eq!(at(9_500), Some((0, 0)));
    assert_eq!(at(9_600), Some((1, 0)));
    // An action starts the clocks again.
    let timeline = idle(
        &[],
        vec![Action {
            end: 3_000,
            happenings: Vec::new(),
        }],
    );
    let at = |t| heroes.at(&timeline, t)[1].map(|p| (p.group, p.frame));
    assert_eq!(at(7_500), Some((0, 0)));
    assert_eq!(at(7_600), Some((1, 0)));
    // A side without a hero shows none; no hero at all: none drawn.
    let one = Heroes::new([None, Some(look())], [true, true], groups(), None);
    assert_eq!(one.at(&timeline, 0)[0], None);
    let none = Heroes::new([None, None], [true, true], groups(), None);
    assert!(!none.any());
    assert_eq!(none.at(&timeline, 0), [None, None]);
    // A flag without frames stays at 0.
    let flat = HeroLook {
        flag_frames: 0,
        ..look()
    };
    let flat = Heroes::new([Some(flat), None], [true, true], groups(), None);
    assert_eq!(flat.at(&timeline, 300)[0], pose(0, 0, 0));
}

/// A seed whose first draws ask for `wanted`: defeat (true), victory (false) or none.
fn seed_for(wanted: Option<bool>) -> u32 {
    (0..1_000)
        .find(|&s| {
            let mut r = s;
            let got = if between(&mut r, 1, 100) < DEFEAT_BELOW {
                Some(true)
            } else if between(&mut r, 1, 100) < VICTORY_BELOW {
                Some(false)
            } else {
                None
            };
            got == wanted
        })
        .unwrap()
}

#[test]
fn a_death_asks_one_hero_for_defeat_or_the_other_for_victory() {
    // The defender's stack dies in an action ending at 1 000 ms; the attacker's turn (a human's)
    // runs: at the next step its hero plays victory, or the defender's defeat, or neither.
    let died = vec![Action {
        end: 1_000,
        happenings: vec![Happening::Died(Side::Defender)],
    }];
    let timeline = idle(&[(0, 0)], died.clone());
    let at = |seed, t| {
        let heroes = Heroes::new([Some(look()), Some(look())], [true, false], groups(), seed);
        heroes
            .at(&timeline, t)
            .map(|p| p.map(|p| (p.group, p.frame)))
    };
    let defeat = Some(seed_for(Some(true)));
    assert_eq!(at(defeat, 900), [Some((0, 0)), Some((0, 0))]);
    assert_eq!(at(defeat, 1_000), [Some((0, 0)), Some((2, 0))]);
    assert_eq!(at(defeat, 2_000), [Some((0, 0)), Some((2, 10))]);
    assert_eq!(at(defeat, 2_100), [Some((0, 0)), Some((0, 0))]);
    let victory = Some(seed_for(Some(false)));
    assert_eq!(at(victory, 1_000), [Some((3, 0)), Some((0, 0))]);
    let neither = Some(seed_for(None));
    assert_eq!(at(neither, 1_000), [Some((0, 0)), Some((0, 0))]);
    // No generator of its own: no reactions (the reference checks take the original's).
    assert_eq!(at(None, 1_000), [Some((0, 0)), Some((0, 0))]);
    // While a computer's turn runs (or none) the request lapses.
    for turns in [&[(0, 1)][..], &[]] {
        let timeline = idle(turns, died.clone());
        let heroes = Heroes::new(
            [Some(look()), Some(look())],
            [true, false],
            groups(),
            defeat,
        );
        for t in [1_000, 1_500] {
            let shown = heroes.at(&timeline, t)[1].map(|p| (p.group, p.frame));
            assert_eq!(shown, Some((0, 0)));
        }
    }
}

#[test]
fn each_reaction_plays_once_a_round() {
    let seed = seed_for(Some(true));
    let defeat = Some(seed);
    // Two deaths of the defender's stacks, each asking for its defeat: the second waits for the
    // next round, which clears the requests asked before it.
    let death = |end, round| Action {
        end,
        happenings: if round {
            vec![Happening::Round, Happening::Died(Side::Defender)]
        } else {
            vec![Happening::Died(Side::Defender)]
        },
    };
    // The second death's draws come after the first's one (below 60: defeat).
    let mut after = seed;
    between(&mut after, 1, 100);
    let heroes = Heroes::new([Some(look()), Some(look())], [true, true], groups(), defeat);
    let group = |timeline: &Timeline, t| heroes.at(timeline, t)[1].map(|p| p.group);
    let timeline = idle(&[(0, 0)], vec![death(1_000, false), death(3_000, false)]);
    assert_eq!(group(&timeline, 1_000), Some(2));
    assert_eq!(group(&timeline, 3_000), Some(0));
    // A round between: the second may play (if its draw asks for defeat).
    let timeline = idle(&[(0, 0)], vec![death(1_000, false), death(3_000, true)]);
    let second = if between(&mut after, 1, 100) < DEFEAT_BELOW {
        Some(2)
    } else {
        Some(0)
    };
    assert_eq!(group(&timeline, 3_000), second);
    // A round after a death clears its request.
    let cleared = vec![Action {
        end: 1_000,
        happenings: vec![Happening::Died(Side::Defender), Happening::Round],
    }];
    assert_eq!(group(&idle(&[(0, 0)], cleared), 1_000), Some(0));
    // A hero whose DEF lacks the group: the reaction counts as played, nothing shows.
    let bare = HeroLook {
        frames: [(0, 1), (1, 4)].into_iter().collect(),
        flag_frames: 6,
    };
    let heroes = Heroes::new(
        [Some(bare.clone()), Some(bare)],
        [true, true],
        groups(),
        defeat,
    );
    let timeline = idle(&[(0, 0)], vec![death(1_000, false)]);
    assert_eq!(heroes.at(&timeline, 1_000)[1].map(|p| p.group), Some(0));
    let victory = Some(seed_for(Some(false)));
    let heroes = Heroes::new(
        [Some(look()), Some(look())],
        [true, true],
        groups(),
        victory,
    );
    let timeline = idle(&[(0, 0)], vec![death(1_000, false), death(2_000, false)]);
    // The victory plays once; the second death's draw may ask again, but not this round.
    assert_eq!(heroes.at(&timeline, 1_000)[0].map(|p| p.group), Some(3));
    assert_eq!(heroes.at(&timeline, 2_200)[0].map(|p| p.group), Some(0));
}

#[test]
fn a_timeline_carries_its_heroes() {
    // The idle steps before the last command's start count as the ones after it.
    let mut timeline = idle(&[], Vec::new());
    timeline.steps = vec![0, 100];
    timeline.idle = anim::Idle {
        from: 1_000,
        next: 1_000,
    };
    assert!(timeline.still());
    let none =
        timeline
            .clone()
            .with_heroes(Heroes::new([None, None], [true, true], groups(), None));
    assert!(none.still());
    assert_eq!(Moment::at(&none, 0).heroes, [None, None]);
    let heroes = Heroes::new([Some(look()), None], [true, true], groups(), None);
    let timeline = timeline.with_heroes(heroes);
    // The flags wave: nothing is ever still.
    assert!(!timeline.still());
    assert_eq!(Moment::at(&timeline, 900).heroes, [pose(0, 0, 2), None]);
    assert_eq!(Moment::at(&timeline, 1_000).heroes, [pose(0, 0, 3), None]);
}
