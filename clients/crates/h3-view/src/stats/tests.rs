use super::*;
use alloc::vec;

fn stats_layout() -> StatsLayout {
    StatsLayout {
        picture: "CrStkPU.pcx".into(),
        towns: ["Neu", "Cas", "Ram"]
            .iter()
            .map(|t| format!("CrBkg{t}.pcx"))
            .collect(),
        font: "smalfont.fnt".into(),
        count_font: "Verd10B.fnt".into(),
        font_palette: "game.pal".into(),
        morale: "imrl42.def".into(),
        luck: "ilck42.def".into(),
        morale_frames: Some("imrlB.def".into()),
        luck_frames: None,
        effects: "spellint.def".into(),
        status: "VARBack.pcx".into(),
        button_back: "Box64x30.pcx".into(),
        ok: "iOKAY.def".into(),
        cast_back: "Box46x32.pcx".into(),
        cast: "icm005.def".into(),
    }
}

fn texts() -> Texts {
    Texts {
        attack: "A".into(),
        defense: "D".into(),
        shots: "S".into(),
        damage: "Dm".into(),
        health: "H".into(),
        health_left: "HL".into(),
        speed: "Sp".into(),
    }
}

fn marksmen() -> Stats {
    Stats {
        name: "Marksmen".into(),
        def: "CHCROS.def".into(),
        town: Some(0),
        count: 12,
        attack: Pair { own: 6, now: 9 },
        defense: Pair { own: 3, now: 3 },
        shots: Some(Pair { own: 24, now: 20 }),
        damage: [2, 3],
        health: Pair { own: 10, now: 10 },
        health_left: 7,
        speed: Pair { own: 6, now: -1 },
        morale: 5,
        luck: -1,
        effects: vec![41, 43, 53, 46],
        ability: Some("Shoots twice".into()),
        cast: false,
        modal: true,
    }
}

#[test]
fn the_dialog_sits_next_to_the_hex_inside_the_screen() {
    // 0x46888A: the anchor less (149, 140), clamped to 0..502 and 0..275.
    assert_eq!(place([300, 300]), [151, 160]);
    assert_eq!(place([100, 100]), [0, 0]);
    assert_eq!(place([700, 500]), [502, 275]);
    assert_eq!(place([i32::MIN, i32::MAX]), [0, 275]);
}

#[test]
fn a_pair_shows_its_own_number_and_the_stacks_when_they_differ() {
    assert_eq!(Pair { own: 6, now: 6 }.text(), "6");
    assert_eq!(Pair { own: 6, now: 9 }.text(), "6(9)");
}

#[test]
fn the_items_come_in_the_executables_order() {
    let m = layout(&stats_layout(), &texts(), &marksmen(), [151, 160], 2);
    assert_eq!(m.dialog.place, [151, 160]);
    assert_eq!(m.dialog.shadow, Some([298, 311]));
    let ids: Vec<u16> = m.dialog.items.iter().map(Item::id).collect();
    assert_eq!(
        ids,
        [
            200, 203, 202, 201, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216,
            217, 218, 219, 220, 221, 222, 223, 225, 0x7802, 224, ABILITY
        ]
    );
    let text = |id| m.shown.texts.get(&id).map(String::as_str);
    assert_eq!(text(NAME), Some("Marksmen"));
    assert_eq!(text(COUNT), Some("12"));
    assert_eq!(text(206), Some("6(9)"));
    assert_eq!(text(208), Some("3"));
    assert_eq!(text(210), Some("24(20)"));
    assert_eq!(text(212), Some("2 - 3"));
    assert_eq!(text(216), Some("7"));
    // A speed below 0 shows as 0.
    assert_eq!(text(218), Some("6(0)"));
    assert_eq!(text(ABILITY), Some("Shoots twice"));
    assert_eq!(m.shown.players.get(&BACKGROUND), Some(&2));
    // The damage's value a row above its label (0x5F59A2).
    let at = |id| m.dialog.item(id).and_then(Item::area).map(|(at, _)| at);
    assert_eq!(at(211), Some([154, 104]));
    assert_eq!(at(212), Some([154, 103]));
    assert_eq!(at(217), Some([154, 161]));
    // The town's frame by the town + 1, the creature's group 2 drawn as a creature.
    let Some(Item::Picture(town)) = m.dialog.item(TOWN) else {
        panic!("the town's frame");
    };
    assert_eq!(town.picture, "CrBkgCas.pcx");
    let Some(Item::Def(creature)) = m.dialog.item(CREATURE) else {
        panic!("the creature");
    };
    assert_eq!(
        (
            creature.group,
            creature.creature,
            creature.at,
            creature.size
        ),
        (2, true, [21, 48], [100, 130])
    );
    // Morale in another DEF's frames (CLT-Q36), luck in its own.
    let shared = |id| match m.dialog.item(id) {
        Some(Item::Def(d)) => d.frames_of.clone(),
        _ => None,
    };
    assert_eq!(shared(MORALE).as_deref(), Some("imrlB.def"));
    assert_eq!(shared(LUCK), None);
    // Morale and luck clamped to ±3 (+ 3), the effects the list's last three (frame + 1).
    let frame = |id| match m.dialog.item(id) {
        Some(Item::Def(d)) => Some((d.frame, d.at)),
        _ => None,
    };
    assert_eq!(frame(MORALE), Some((6, [23, 189])));
    assert_eq!(frame(LUCK), Some((2, [77, 189])));
    assert_eq!(frame(221), Some((44, [127, 186])));
    assert_eq!(frame(222), Some((54, [179, 186])));
    assert_eq!(frame(223), Some((47, [231, 186])));
}

#[test]
fn what_a_stack_lacks_is_left_out() {
    // No shots, no count, one effect, a single damage, no ability, not modal, a neutral town.
    let stats = Stats {
        count: 0,
        shots: None,
        damage: [4, 4],
        effects: vec![15],
        ability: None,
        modal: false,
        town: None,
        ..marksmen()
    };
    let m = layout(&stats_layout(), &texts(), &stats, [0, 0], 0);
    let ids: Vec<u16> = m.dialog.items.iter().map(Item::id).collect();
    assert_eq!(
        ids,
        [
            200, 203, 202, 201, 205, 206, 207, 208, 211, 212, 213, 214, 215, 216, 217, 218, 219,
            220, 221, 224
        ]
    );
    assert_eq!(m.shown.texts.get(&212).map(String::as_str), Some("4"));
    let Some(Item::Picture(town)) = m.dialog.item(TOWN) else {
        panic!("the town's frame");
    };
    assert_eq!(town.picture, "CrBkgNeu.pcx");
    // A town past the layout's list: no picture named.
    let far = Stats {
        town: Some(8),
        ..stats
    };
    let m = layout(&stats_layout(), &texts(), &far, [0, 0], 0);
    let Some(Item::Picture(town)) = m.dialog.item(TOWN) else {
        panic!("the town's frame");
    };
    assert_eq!(town.picture, "");
}

#[test]
fn a_faerie_dragons_cast_button_takes_the_abilitys_place() {
    let stats = Stats {
        cast: true,
        ..marksmen()
    };
    let m = layout(&stats_layout(), &texts(), &stats, [0, 0], 0);
    let ids: Vec<u16> = m.dialog.items.iter().map(Item::id).collect();
    assert_eq!(ids[ids.len() - 3..], [224, CAST_BACK, CAST]);
    assert!(!m.shown.texts.contains_key(&ABILITY));
    let Some(Item::Button(cast)) = m.dialog.item(CAST) else {
        panic!("the cast button");
    };
    assert_eq!((cast.def.as_str(), cast.at), ("icm005.def", [75, 237]));
}

#[test]
fn the_labels_come_from_the_catalogue() {
    let texts = Texts::of(&h3_ui::text::Catalogue::default());
    // Without a pack every key shows itself.
    assert_eq!(texts.attack, "stats.attack");
    assert_eq!(texts.health_left, "stats.health-left");
    assert_eq!(texts.speed, "stats.speed");
}

fn frames(counts: &[(u32, u32)]) -> BTreeMap<u32, u32> {
    counts.iter().copied().collect()
}

#[test]
fn the_creature_plays_its_group_then_draws_the_next() {
    // Group 2 of three frames, then a group drawn by the weights: each a group the DEF has.
    let mut steps = Steps::new(frames(&[(1, 2), (2, 3), (4, 2)]), false, 7);
    assert_eq!((steps.group, steps.frame), (2, 0));
    steps.step();
    steps.step();
    assert_eq!((steps.group, steps.frame), (2, 2));
    let mut seen = alloc::collections::BTreeSet::new();
    for _ in 0..400 {
        steps.step();
        assert!([1, 2, 4].contains(&steps.group), "{}", steps.group);
        if steps.frame == 0 {
            seen.insert(steps.group);
        }
    }
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), [1, 2, 4]);
}

#[test]
fn a_walk_starts_and_ends_with_its_groups() {
    // A DEF with the walk (0), its start (20) and end (21): a walk drawn from standing starts
    // with 20, and a group drawn after the walk comes after 21.
    let mut steps = Steps::new(frames(&[(0, 1), (2, 1), (20, 1), (21, 1)]), false, 1);
    let mut groups = vec![steps.group];
    for _ in 0..200 {
        steps.step();
        groups.push(steps.group);
    }
    for w in groups.windows(2) {
        match (w[0], w[1]) {
            (2, 0) => panic!("a walk without its start"),
            (0, 2) => panic!("a walk's end without its group"),
            (20, g) => assert_eq!(g, 0),
            _ => {}
        }
    }
    assert!(groups.contains(&20) && groups.contains(&21));
}

#[test]
fn a_war_machine_draws_from_its_own_weights() {
    let mut steps = Steps::new(frames(&[(2, 1), (15, 1), (0, 1)]), true, 3);
    for _ in 0..300 {
        steps.step();
        assert!([2, 15].contains(&steps.group));
    }
    // A DEF without any group drawn stands.
    let mut lone = Steps::new(frames(&[(7, 1)]), true, 3);
    lone.step();
    assert_eq!((lone.group, lone.frame), (2, 0));
}

#[test]
fn the_steps_come_every_100_ms_from_the_first() {
    assert_eq!(Steps::made(40, 0), 0);
    assert_eq!(Steps::made(40, 40), 1);
    assert_eq!(Steps::made(40, 139), 1);
    assert_eq!(Steps::made(40, 140), 2);
    assert_eq!(Steps::made(0, 0), 1);
}

#[test]
fn a_seed_gives_the_same_steps() {
    let run = |seed| {
        let mut s = Steps::new(frames(&[(1, 2), (2, 1), (3, 2), (4, 1)]), false, seed);
        (0..50)
            .map(|_| {
                s.step();
                (s.group, s.frame)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(run(9), run(9));
    assert_ne!(run(9), run(10));
}
