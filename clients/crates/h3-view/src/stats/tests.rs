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
        morale_described: "imrl82.def".into(),
        luck_described: "ilck82.def".into(),
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

fn effect(spell: u32) -> Effect {
    Effect {
        spell,
        name: format!("spell {spell}"),
        rounds: Some(3),
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
        effects: [41, 43, 53, 46].map(effect).into(),
        account: Account::default(),
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
    let m = layout(
        &stats_layout(),
        &texts(),
        &marksmen(),
        ([151, 160], 2),
        Bugs::Original,
    );
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
    // The damage's value a row above its label (0x5F59A2), at its row in the client's profile
    // (CLT-Q37).
    let at = |m: &MessageBox, id| m.dialog.item(id).and_then(Item::area).map(|(at, _)| at);
    assert_eq!(at(&m, 211), Some([154, 104]));
    assert_eq!(at(&m, 212), Some([154, 103]));
    assert_eq!(at(&m, 217), Some([154, 161]));
    let fixed = layout(
        &stats_layout(),
        &texts(),
        &marksmen(),
        ([151, 160], 2),
        Bugs::Fixed,
    );
    assert_eq!(at(&fixed, 212), Some([154, 104]));
    assert_eq!(at(&fixed, 210), at(&m, 210));
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
        effects: vec![effect(15)],
        ability: None,
        modal: false,
        town: None,
        ..marksmen()
    };
    let m = layout(
        &stats_layout(),
        &texts(),
        &stats,
        ([0, 0], 0),
        Bugs::Original,
    );
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
    let m = layout(&stats_layout(), &texts(), &far, ([0, 0], 0), Bugs::Original);
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
    let m = layout(
        &stats_layout(),
        &texts(),
        &stats,
        ([0, 0], 0),
        Bugs::Original,
    );
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
    let texts = Texts::of(&Catalogue::default());
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

#[test]
fn each_item_has_its_hint_tables_index() {
    // 0x5F46C0: the lines' labels and values share their line's.
    let index = |id| hint_index(id, Bugs::Original);
    assert_eq!(index(NAME), Some(0));
    assert_eq!(index(COUNT), Some(1));
    assert_eq!(
        [205, 206, 211, 212, 216, 218].map(index),
        [2, 2, 5, 5, 7, 8].map(Some)
    );
    assert_eq!(
        [MORALE, LUCK, 221, 223].map(index),
        [9, 10, 11, 11].map(Some)
    );
    assert_eq!(
        [0x7803, 300, OK, CAST].map(index),
        [12, 13, 14, 15].map(Some)
    );
    assert_eq!(
        [BACKGROUND, TOWN, CREATURE, STATUS, OK_BACK, 224, 9].map(index),
        [None; 7]
    );
    // The speed's label: the health left's in the original, its own in the client's (CLT-Q38).
    assert_eq!(index(217), Some(7));
    assert_eq!(hint_index(217, Bugs::Fixed), Some(8));
    assert_eq!(hint_index(215, Bugs::Fixed), Some(7));
}

fn hints() -> Hints {
    Hints {
        table: (0..HINTS).map(|i| format!("hint {i}")).collect(),
        cast: "cast".into(),
        effects: vec!["e0".into(), "e1".into()],
        descriptions: (0..HINTS).map(|i| format!("help {i}")).collect(),
        cast_description: "cast help".into(),
        morale: Some(("morale help".into(), picture("imrl82.def", 4))),
        luck: None,
    }
}

/// The description pictures' DEFs and sizes (82 × 93, as the original's).
fn pictures() -> Described {
    [
        ("imrl82.def".into(), [82, 93]),
        ("ilck82.def".into(), [82, 93]),
    ]
}

fn picture(def: &str, frame: u32) -> message::Picture {
    message::Picture {
        def: def.into(),
        frame,
        size: [82, 93],
        caption: String::new(),
    }
}

/// The text of a description, its picture left out.
fn said(described: Option<(String, Option<message::Picture>)>) -> Option<String> {
    described.map(|(text, _)| text)
}

#[test]
fn the_hint_over_an_item() {
    let h = hints();
    assert_eq!(h.over(NAME, Bugs::Fixed), "hint 0");
    assert_eq!(h.over(217, Bugs::Original), "hint 7");
    assert_eq!(h.over(217, Bugs::Fixed), "hint 8");
    assert_eq!(h.over(OK, Bugs::Fixed), "hint 14");
    // Past the table's end the cast button's; an effect's its own; none elsewhere.
    assert_eq!(h.over(CAST, Bugs::Original), "cast");
    assert_eq!(h.over(221, Bugs::Fixed), "e0");
    assert_eq!(h.over(222, Bugs::Fixed), "e1");
    assert_eq!(h.over(223, Bugs::Fixed), "");
    assert_eq!(h.over(BACKGROUND, Bugs::Fixed), "");
    // A table too short (no pack): the missing entry's is empty.
    let short = Hints {
        table: Vec::new(),
        ..hints()
    };
    assert_eq!(short.over(NAME, Bugs::Fixed), "");
}

#[test]
fn the_hints_come_from_the_catalogue() {
    use h3_ui::text::pack::Pack;
    let pack = Pack::from_json(
        "en",
        r#"{ "stats": { "hint": { "name": "N", "ok": "O", "cast-original": "Mine",
            "effect": "{{spell}} {{rounds}}", "effect-until": "{{spell}}: {{until}}" },
            "until": { "bind": "B", "berserk": "Z", "disrupting-ray": "R" },
            "help": { "name": "HN", "ok": "HO", "cast-original": "Lab" } },
            "battle": { "control": { "spell-book": "Cast" } } }"#,
    )
    .unwrap();
    let catalogue = Catalogue { packs: vec![pack] };
    let stats = Stats {
        effects: vec![
            effect(41),
            Effect {
                rounds: None,
                ..effect(53)
            },
            effect(72),
            effect(59),
            effect(47),
        ],
        ..marksmen()
    };
    let fixed = Hints::of(&catalogue, &stats, &pictures(), Bugs::Fixed);
    assert_eq!(fixed.table.len(), HINTS);
    assert_eq!(
        (fixed.table[0].as_str(), fixed.table[14].as_str()),
        ("N", "O")
    );
    assert_eq!(fixed.cast, "Cast");
    // The right button's descriptions, the table's second texts (E17 M6 T2a2b1).
    assert_eq!(
        (
            fixed.descriptions[0].as_str(),
            fixed.descriptions[14].as_str()
        ),
        ("HN", "HO")
    );
    assert_eq!(fixed.cast_description, "Cast");
    // The last three: Bind, Berserk and Disrupting Ray name how long they last.
    assert_eq!(fixed.effects, ["spell 72: B", "spell 59: Z", "spell 47: R"]);
    let original = Hints::of(&catalogue, &marksmen(), &pictures(), Bugs::Original);
    assert_eq!(original.cast, "Mine");
    assert_eq!(original.cast_description, "Lab");
    assert_eq!(original.effects, ["spell 43 3", "spell 53 3", "spell 46 3"]);
    // An effect for the rest of the battle shows 0 rounds.
    let rest = Stats {
        effects: vec![Effect {
            rounds: None,
            ..effect(53)
        }],
        ..marksmen()
    };
    assert_eq!(
        Hints::of(&catalogue, &rest, &pictures(), Bugs::Fixed).effects,
        ["spell 53 0"]
    );
}

fn modal(memory: &mut Memory) -> Modal {
    let window = layout(
        &stats_layout(),
        &texts(),
        &marksmen(),
        ([100, 50], 0),
        Bugs::Fixed,
    );
    Modal::new(window, hints(), memory)
}

#[test]
fn the_mouse_over_the_dialog_sets_its_cursor_and_hint() {
    let mut memory = Memory::new(Bugs::Fixed);
    let mut m = modal(&mut memory);
    assert_eq!((m.cursor, m.status()), (None, ""));
    // Over nothing of the dialog (outside it): the last one seen, nothing changes.
    assert!(!m.hover(&mut memory, [0, 0]));
    assert_eq!(m.cursor, None);
    // The attack's line (205 and 206 in one box at 154, 48): the value, added after its label,
    // takes the mouse; its line's hint, the cursor's frame 1.
    assert!(m.hover(&mut memory, [100 + 160, 50 + 50]));
    assert_eq!((m.cursor, m.status()), (Some(1), "hint 2"));
    assert!(!m.hover(&mut memory, [100 + 270, 50 + 50]));
    // The background: an item without a hint, the cursor's frame 1 still.
    assert!(m.hover(&mut memory, [100 + 5, 50 + 5]));
    assert_eq!((m.cursor, m.status()), (Some(1), ""));
    // Off the dialog: the cursor's frame 0, an empty line.
    assert!(m.hover(&mut memory, [0, 0]));
    assert_eq!((m.cursor, m.status()), (Some(0), ""));
    // The speed's line: its value's hint (the label under it never takes the mouse).
    assert!(m.hover(&mut memory, [100 + 160, 50 + 165]));
    assert_eq!(m.status(), "hint 8");
}

#[test]
fn the_ability_text_and_the_cast_buttons_back_are_nothing_to_the_mouse() {
    // The reviewer's R2-1: the executable numbers them −1, as no item (0x5F4E57). From the morale
    // icon (219 at 23, 189) onto the ability text (20, 232, 192 × 41): the cursor's frame 0 and
    // an empty line, as off the dialog.
    let mut memory = Memory::new(Bugs::Fixed);
    let mut m = modal(&mut memory);
    assert!(m.hover(&mut memory, [100 + 40, 50 + 200]));
    assert_eq!(m.cursor, Some(1));
    assert!(m.hover(&mut memory, [100 + 40, 50 + 250]));
    assert_eq!((m.cursor, m.status()), (Some(0), ""));
    // From it off the dialog: nothing changes.
    assert!(!m.hover(&mut memory, [0, 0]));
    // A dialog opened over it: nothing changes, the pointer stays.
    let mut again = modal(&mut memory);
    assert!(!again.hover(&mut memory, [100 + 40, 50 + 250]));
    assert_eq!(again.cursor, None);
    // The cast button's back (74, 236, 48 × 34) where the button (75, 237) leaves it, in the
    // original's profile too.
    let mut memory = Memory::new(Bugs::Original);
    let stats = Stats {
        cast: true,
        ability: None,
        ..marksmen()
    };
    let window = layout(
        &stats_layout(),
        &texts(),
        &stats,
        ([0, 0], 0),
        Bugs::Original,
    );
    let mut m = Modal::new(window, hints(), &mut memory);
    assert!(m.hover(&mut memory, [80, 240]));
    assert_eq!(m.cursor, Some(1));
    assert!(m.hover(&mut memory, [74, 236]));
    assert_eq!((m.cursor, m.status()), (Some(0), ""));
}

#[test]
fn the_damages_label_takes_the_mouse_below_its_value_in_the_original() {
    // A row above its label, the value leaves the label's last row to it (CLT-Q37).
    let mut memory = Memory::new(Bugs::Original);
    let window = layout(
        &stats_layout(),
        &texts(),
        &marksmen(),
        ([0, 0], 0),
        Bugs::Original,
    );
    let mut m = Modal::new(window, hints(), &mut memory);
    assert!(m.hover(&mut memory, [160, 119]));
    assert_eq!(m.status(), "hint 5");
    assert!(m.hover(&mut memory, [160, 120]));
    assert_eq!(m.status(), "hint 5");
}

#[test]
fn the_item_last_hovered_is_kept_by_the_original_only() {
    // The original's: a dialog opened over the item last seen shows nothing until another.
    let mut memory = Memory::new(Bugs::Original);
    assert_eq!(memory.bugs(), Bugs::Original);
    let mut m = modal(&mut memory);
    assert!(m.hover(&mut memory, [100 + 160, 50 + 50]));
    let mut again = modal(&mut memory);
    assert!(!again.hover(&mut memory, [100 + 161, 50 + 50]));
    assert_eq!((again.cursor, again.status()), (None, ""));
    assert!(again.hover(&mut memory, [100 + 160, 50 + 165]));
    assert_eq!(again.status(), "hint 8");
    // The client's forgets it at each opening.
    let mut memory = Memory::new(Bugs::Fixed);
    let mut m = modal(&mut memory);
    assert!(m.hover(&mut memory, [100 + 160, 50 + 50]));
    let mut again = modal(&mut memory);
    assert!(again.hover(&mut memory, [100 + 161, 50 + 50]));
    assert_eq!(again.status(), "hint 2");
}

#[test]
fn the_ok_closes_the_dialog_by_a_click_or_its_keys() {
    let mut memory = Memory::new(Bugs::Fixed);
    let ok = [100 + 220, 50 + 250];
    let mut m = modal(&mut memory);
    // A press elsewhere does nothing; on the OK it goes down, off and on again it follows.
    assert_eq!(m.press([100 + 160, 50 + 50]), Outcome::Nothing);
    assert_eq!(m.release(), Outcome::Nothing);
    assert_eq!(m.press(ok), Outcome::Changed);
    assert_eq!(m.moved([0, 0]), Outcome::Changed);
    assert_eq!(m.moved(ok), Outcome::Changed);
    assert_eq!(m.release(), Outcome::Closed(OK));
    // Released off it: nothing clicked.
    let mut m = modal(&mut memory);
    assert_eq!(m.press(ok), Outcome::Changed);
    assert_eq!(m.moved([0, 0]), Outcome::Changed);
    assert_eq!(m.release(), Outcome::Changed);
    // Its key down, then up.
    let mut m = modal(&mut memory);
    assert_eq!(m.key("Escape"), Outcome::Changed);
    assert_eq!(m.key_up("Return"), Outcome::Nothing);
    assert_eq!(m.key_up("Escape"), Outcome::Closed(OK));
}

#[test]
fn the_right_button_describes_an_item() {
    // E17 M6 T2a2b1: an item's pair's second text; the effects their pair's; the cast button's
    // its own; morale's and luck's their own with their pictures (M6 T2a2b2); none for an item
    // without a pair or an empty text.
    let h = hints();
    assert_eq!(
        said(h.described(NAME, Bugs::Fixed)).as_deref(),
        Some("help 0")
    );
    assert_eq!(
        said(h.described(206, Bugs::Fixed)).as_deref(),
        Some("help 2")
    );
    assert_eq!(
        said(h.described(EFFECTS + 1, Bugs::Fixed)).as_deref(),
        Some("help 11")
    );
    assert_eq!(
        said(h.described(OK, Bugs::Fixed)).as_deref(),
        Some("help 14")
    );
    assert_eq!(
        h.described(CAST, Bugs::Original),
        Some(("cast help".into(), None))
    );
    assert_eq!(
        h.described(MORALE, Bugs::Fixed),
        Some(("morale help".into(), Some(picture("imrl82.def", 4))))
    );
    assert_eq!(h.described(LUCK, Bugs::Fixed), None);
    assert_eq!(h.described(BACKGROUND, Bugs::Fixed), None);
    assert_eq!(h.described(CREATURE, Bugs::Fixed), None);
    // The speed's label: the health left's in the original (CLT-Q38 (1)), its own in the client's.
    assert_eq!(
        said(h.described(217, Bugs::Original)).as_deref(),
        Some("help 7")
    );
    assert_eq!(
        said(h.described(217, Bugs::Fixed)).as_deref(),
        Some("help 8")
    );
    let empty = Hints {
        descriptions: vec![String::new(); HINTS],
        ..hints()
    };
    assert_eq!(empty.described(NAME, Bugs::Fixed), None);
    // Over the dialog: the item a move would enter; off it nothing.
    let mut memory = Memory::new(Bugs::Fixed);
    let m = modal(&mut memory);
    assert_eq!(
        said(m.describe([100 + 160, 50 + 50])).as_deref(),
        Some("help 2")
    );
    assert_eq!(m.describe([100 + 5, 50 + 5]), None);
    assert_eq!(m.describe([0, 0]), None);
}

#[test]
fn the_popup_takes_its_ends_and_lets_a_held_ok_go_in_the_clients_profile() {
    let popup = || modal(&mut Memory::new(Bugs::Fixed)).open.message;
    // A key's release while it is held: the client's lets the OK its key holds go, unclicked.
    let mut memory = Memory::new(Bugs::Fixed);
    let mut m = modal(&mut memory);
    assert_eq!(m.key("Return"), Outcome::Changed);
    m.popup = Some(popup());
    assert!(m.held_input(false, Some("Return")));
    assert!(m.popup.is_some());
    // Another key's release, nothing held: nothing changes.
    assert!(!m.held_input(false, Some("Escape")));
    assert_eq!(m.key_up("Return"), Outcome::Nothing);
    // Its end closes it.
    assert!(m.held_input(true, None));
    assert!(m.popup.is_none());
    // The original keeps the OK its key holds: the key's release clicks it after the popup.
    let mut memory = Memory::new(Bugs::Original);
    let mut m = modal(&mut memory);
    assert_eq!(m.key("Return"), Outcome::Changed);
    m.popup = Some(popup());
    assert!(!m.held_input(false, Some("Return")));
    assert!(m.held_input(true, None));
    assert_eq!(m.key_up("Return"), Outcome::Closed(OK));
    // Nothing held, nothing shown: no change.
    assert!(!m.held_input(true, None));
}

#[test]
fn the_ok_the_mouse_holds_runs_its_own_loop() {
    // E17 M7 T2 (CLT-Q72 (a); `def_button_message` 0x45641C): while the mouse holds the OK the
    // right press shows no description, the moves reach only the OK (no hint, no cursor), and
    // the right button's release ends the hold as the left's, clicking it over it.
    let ok = [100 + 220, 50 + 250];
    let name = [100 + 160, 50 + 50];
    let mut memory = Memory::new(Bugs::Fixed);
    let mut m = modal(&mut memory);
    assert!(m.describe(name).is_some());
    assert!(m.hover(&mut memory, ok));
    let status = String::from(m.status());
    assert_eq!(m.press(ok), Outcome::Changed);
    assert!(m.open.holds_mouse());
    assert_eq!(m.describe(name), None);
    assert!(!m.hover(&mut memory, name));
    assert_eq!(m.status(), status);
    assert_eq!(m.moved(name), Outcome::Changed);
    assert_eq!(m.release(), Outcome::Changed);
    assert!(!m.open.holds_mouse());
    // Back over it, the right release (the same release) closes the dialog.
    assert_eq!(m.press(ok), Outcome::Changed);
    assert_eq!(m.release(), Outcome::Closed(OK));
    // The OK its key holds runs no loop: the right press describes, the moves give hints.
    let mut m = modal(&mut memory);
    assert_eq!(m.key("Return"), Outcome::Changed);
    assert!(!m.open.holds_mouse());
    assert!(m.describe(name).is_some());
    assert!(m.hover(&mut memory, name));
}

#[test]
fn the_cast_button_closes_the_dialog_with_its_id() {
    let mut memory = Memory::new(Bugs::Fixed);
    let stats = Stats {
        cast: true,
        ability: None,
        ..marksmen()
    };
    let window = layout(&stats_layout(), &texts(), &stats, ([0, 0], 0), Bugs::Fixed);
    let mut m = Modal::new(window, hints(), &mut memory);
    // Its hint: past the table's end.
    assert!(m.hover(&mut memory, [80, 240]));
    assert_eq!(m.status(), "cast");
    // Pressed and released over it: the dialog closes with its id (301).
    assert_eq!(m.press([80, 240]), Outcome::Changed);
    assert_eq!(m.release(), Outcome::Closed(CAST));
}

#[test]
fn the_creature_stands_while_a_popup_is_held() {
    // Before any popup: the grid from the first step.
    assert_eq!(Steps::made_around(100, &[], 350), Steps::made(100, 350));
    assert_eq!(Steps::next_around(100, &[], 350), Some(400));
    // Held from 150 to 180: a step at 100, none while held, the next on the grid (200).
    let short = [[150, 180]];
    assert_eq!(Steps::made_around(100, &short, 170), 1);
    assert_eq!(Steps::next_around(100, &short, 170), None);
    assert_eq!(Steps::made_around(100, &short, 199), 1);
    assert_eq!(Steps::next_around(100, &short, 199), Some(200));
    // Held from 150 to 260: one step at its end (200 was due), the deadline then 300.
    let late = [[150, 260]];
    assert_eq!(Steps::made_around(100, &late, 260), 2);
    assert_eq!(Steps::next_around(100, &late, 260), Some(300));
    // Held from 150 to 450: two steps at its end (the deadline moved to now is reached at once).
    let long = [[150, 450]];
    assert_eq!(Steps::made_around(100, &long, 450), 3);
    assert_eq!(Steps::next_around(100, &long, 450), Some(550));
    // Still held at the end of the list.
    assert_eq!(Steps::made_around(100, &[[150, u32::MAX]], 900), 1);
    // A popup after the time asked changes nothing.
    assert_eq!(Steps::made_around(100, &[[500, 600]], 300), 3);
}

/// The steps made by `since` around `holds` as the procedure makes them, a step of the deadline
/// at a time ([`Steps::made_around`]'s and [`Steps::next_around`]'s reference).
fn stepped(first: u32, holds: &[[u32; 2]], since: u32) -> (u32, Option<u32>) {
    let (mut deadline, mut made) = (first, 0_u32);
    for &[start, end] in holds {
        if start > since {
            break;
        }
        while deadline <= start {
            made += 1;
            deadline = deadline.saturating_add(STEP_MS);
        }
        if end > since {
            return (made, None);
        }
        while deadline <= end {
            made += 1;
            deadline = end.max(deadline.saturating_add(STEP_MS));
        }
    }
    while deadline <= since {
        made += 1;
        deadline = deadline.saturating_add(STEP_MS);
    }
    (made, Some(deadline))
}

#[test]
fn the_steps_around_the_popups_come_in_closed_form() {
    // The reviewers' R1-2 and R2-2: the deadline's steps counted at once, as one at a time.
    let holds: [&[[u32; 2]]; 5] = [
        &[],
        &[[150, 180]],
        &[[150, 450], [460, 470]],
        &[[0, 99], [100, 100], [1_000, u32::MAX]],
        &[[250, 260], [900, 1_350]],
    ];
    for first in [0, 1, 99, 100, 101, 250] {
        for holds in holds {
            for since in (0..2_000).step_by(7) {
                let (made, next) = stepped(first, holds, since);
                assert_eq!(Steps::made_around(first, holds, since), made);
                assert_eq!(Steps::next_around(first, holds, since), next);
            }
        }
    }
    // Hours open: as many steps, the deadline the next on the grid; near the clock's end the
    // deadline stays at its end.
    let day = 86_400_000;
    assert_eq!(Steps::made_around(100, &[], day), Steps::made(100, day));
    assert_eq!(Steps::next_around(100, &[], day), Some(day + 100));
    assert_eq!(Steps::next_around(100, &[], u32::MAX - 50), Some(u32::MAX));
}

/// A catalogue whose morale and luck descriptions' messages show which line each is.
fn described_catalogue() -> Catalogue {
    use h3_ui::text::pack::Pack;
    let pack = Pack::from_json(
        "en",
        r#"{ "stats": {
            "morale": { "header": "{{text}}|", "good": "G", "neutral": "N", "bad": "B",
                "none": "-none", "cursed": "-cursed", "unaffected": "-unaffected",
                "alignments": "-troops {{alignments}} {{change}}", "one-alignment": "-one",
                "undead": "-undead", "oppression": "-oppressed {{artifact}}", "in-group": "-in {{creature}}",
                "facing": "-facing {{creature}}", "minimum": "-min {{creature}}",
                "spells": "-spells {{change}}", "holy-good": "-holy+", "holy-evil": "-holy-",
                "fog-good": "-fog-", "fog-evil": "-fog+",
                "artifact_pendant": "-pendant", "artifact_still-eye": "-eye",
                "artifact_badge": "-badge", "artifact_crest": "-crest", "artifact_glyph": "-glyph",
                "skill_basic": "-L1", "skill_advanced": "-L2", "skill_expert": "-L3",
                "temporary_less": "-temp -{{amount}}", "temporary_more": "-temp +{{amount}}" },
            "building": { "name_tavern": "Tavern", "name_brotherhood": "Brotherhood",
                "name_fountain": "Fountain", "name_colossus": "Colossus",
                "name_spirit-guardian": "Guardian", "line_plus-one": "\n{{name}} +1",
                "line_plus-two": "\n{{name}} +2" },
            "luck": { "header": "{{text}}|", "good": "g", "neutral": "n", "bad": "b",
                "none": "-none", "cursed": "-cursed", "clover": "-clover",
                "hourglass": "-hourglass {{artifact}}", "halflings": "{{creature}} lucky",
                "artifact_pendant": "-pendant", "artifact_still-eye": "-eye",
                "artifact_clover": "-clover4", "artifact_cards": "-cards",
                "artifact_ladybird": "-ladybird",
                "skill_basic": "-l1", "skill_advanced": "-l2", "skill_expert": "-l3",
                "temporary_less": "-temp -{{amount}}", "temporary_more": "-temp +{{amount}}" } } }"#,
    )
    .unwrap();
    Catalogue { packs: vec![pack] }
}

#[test]
fn morales_description_lists_what_the_observation_shows() {
    // E17 M6 T2a2b2 (design §12.4): the header by the value's sign, then the lines in the
    // executable's order, or the empty list's line.
    let c = described_catalogue();
    let castle = Account {
        town: Some(0),
        alignments: 1,
        ..Account::default()
    };
    let m = |account: &Account, value| describe::morale(&c, account, value);
    assert_eq!(m(&castle, 1), "G|-one");
    assert_eq!(m(&Account::default(), 0), "N|-none");
    // Holy Ground's line before the alignments'; Angels in the army, the other's dragons.
    let holy = Account {
        field: Field::Holy,
        undead: true,
        angels: Some("Archangels".into()),
        dragons: Some("Ghost Dragons".into()),
        ..castle.clone()
    };
    assert_eq!(
        m(&holy, 1),
        "G|-holy+-one-undead-in Archangels-facing Ghost Dragons"
    );
    let evil = Account {
        field: Field::Holy,
        town: Some(4),
        ..Account::default()
    };
    assert_eq!(m(&evil, -1), "B|-holy-");
    // Evil Fog, the other way round; a neutral creature has no field's line.
    let fog = |town| Account {
        field: Field::EvilFog,
        town,
        ..Account::default()
    };
    assert_eq!(m(&fog(Some(1)), -1), "B|-fog-");
    assert_eq!(m(&fog(Some(5)), 1), "G|-fog+");
    assert_eq!(m(&fog(None), 0), "N|-none");
    // Three alignments and more: 2 less the number, -3 from 5; two give no line.
    let troops = |alignments| Account {
        alignments,
        ..Account::default()
    };
    assert_eq!(m(&troops(3), -1), "B|-troops 3 -1");
    assert_eq!(m(&troops(4), -2), "B|-troops 4 -2");
    assert_eq!(m(&troops(6), -3), "B|-troops 6 -3");
    assert_eq!(m(&troops(2), 0), "N|-none");
    // Cursed Ground's line alone, then a creature without morale's alone.
    let cursed = Account {
        field: Field::Cursed,
        no_morale: true,
        ..holy.clone()
    };
    assert_eq!(m(&cursed, 0), "N|-cursed");
    let unaffected = Account {
        no_morale: true,
        ..holy.clone()
    };
    assert_eq!(m(&unaffected, 0), "N|-unaffected");
}

#[test]
fn the_spells_line_is_the_value_less_the_field_and_the_base() {
    let c = described_catalogue();
    let m = |account: &Account, value| describe::morale(&c, account, value);
    let mirth = Account {
        town: Some(0),
        alignments: 1,
        morale_spells: 2,
        ..Account::default()
    };
    assert_eq!(m(&mirth, 3), "G|-one-spells +2");
    // The field's change is the field's line's, not the spells'.
    let fog = Account {
        field: Field::EvilFog,
        morale_spells: -1,
        ..mirth.clone()
    };
    assert_eq!(m(&fog, 1), "G|-fog--one-spells -1");
    // The Minotaurs' minimum: a base below 1 is 1, so Mirth's 2 shows as 1 (0x44BF88).
    let minotaurs = Account {
        alignments: 3,
        morale_minimum: Some("Minotaurs".into()),
        morale_spells: 2,
        ..Account::default()
    };
    assert_eq!(m(&minotaurs, 1), "G|-troops 3 -1-min Minotaurs");
    let minotaurs = Account {
        alignments: 1,
        morale_spells: 2,
        ..minotaurs
    };
    assert_eq!(m(&minotaurs, 3), "G|-one-spells +2");
    let minotaurs = Account {
        alignments: 0,
        ..minotaurs
    };
    assert_eq!(m(&minotaurs, 2), "G|-min Minotaurs-spells +1");
}

#[test]
fn lucks_description_and_the_halflings_line() {
    let c = described_catalogue();
    let l = |account: &Account, value, bugs| describe::luck(&c, account, value, bugs);
    assert_eq!(l(&Account::default(), 0, Bugs::Fixed), "n|-none");
    // Clover Field for a creature of a neutral town (6 to 8) only.
    let clover = |town| Account {
        field: Field::Clover,
        town,
        ..Account::default()
    };
    assert_eq!(l(&clover(Some(6)), 2, Bugs::Fixed), "g|-clover");
    assert_eq!(l(&clover(Some(8)), 2, Bugs::Fixed), "g|-clover");
    assert_eq!(l(&clover(Some(2)), 0, Bugs::Fixed), "n|-none");
    assert_eq!(l(&clover(None), 0, Bugs::Fixed), "n|-none");
    // The other army's Devils; Fortune's spells line.
    let devils = Account {
        devils: Some("Arch Devils".into()),
        luck_spells: 2,
        ..Account::default()
    };
    assert_eq!(
        l(&devils, 1, Bugs::Fixed),
        "g|-facing Arch Devils-spells +2"
    );
    // Cursed Ground's line alone.
    let cursed = Account {
        field: Field::Cursed,
        ..devils.clone()
    };
    assert_eq!(l(&cursed, 0, Bugs::Fixed), "n|-cursed");
    // The Halflings' line: without a line break in the original's profile, on its own row in
    // the client's (CLT-Q41 (1)); their base of 1 takes Fortune's 1 into it.
    let halflings = Account {
        luck_minimum: Some("Halflings".into()),
        ..Account::default()
    };
    assert_eq!(l(&halflings, 1, Bugs::Original), "g|Halflings lucky");
    assert_eq!(l(&halflings, 1, Bugs::Fixed), "g|\nHalflings lucky");
    let fortune = Account {
        devils: Some("Devils".into()),
        luck_spells: 1,
        ..halflings
    };
    assert_eq!(
        l(&fortune, 1, Bugs::Original),
        "g|-facing DevilsHalflings lucky"
    );
}

#[test]
fn the_described_pictures_follow_the_sign() {
    assert_eq!(
        [3, 1, 0, -1, -3].map(describe::picture_frame),
        [4, 4, 3, 2, 2]
    );
    let c = described_catalogue();
    let stats = Stats {
        morale: 1,
        luck: 0,
        account: Account {
            alignments: 1,
            ..Account::default()
        },
        ..marksmen()
    };
    let h = Hints::of(&c, &stats, &pictures(), Bugs::Fixed);
    assert_eq!(
        h.described(MORALE, Bugs::Fixed),
        Some(("G|-one".into(), Some(picture("imrl82.def", 4))))
    );
    assert_eq!(
        h.described(LUCK, Bugs::Fixed),
        Some(("n|-none".into(), Some(picture("ilck82.def", 3))))
    );
}

#[test]
fn the_heros_lists_come_first_in_their_own_order() {
    // E17 M6 T3c1 (design §12.9): the hero's list after the base, before the field's and the
    // army's lines: its artifacts in the list's order (the Pendant first), the skill by its level,
    // the rest as temporary; the header by the unclamped value.
    let c = described_catalogue();
    let m = |account: &Account, value| describe::morale(&c, account, value);
    let l = |account: &Account, value| describe::luck(&c, account, value, Bugs::Fixed);
    let hero = HeroList {
        total: 7,
        artifacts: vec![51, 108, 45],
        skill: 2,
        grail: None,
    };
    let account = Account {
        town: Some(0),
        alignments: 1,
        field: Field::Holy,
        hero_morale: hero.clone(),
        ..Account::default()
    };
    assert_eq!(m(&account, 9), "G|-pendant-eye-glyph-L2-holy+-one");
    // Luck's artifacts are its own: the Glyph of Gallantry is in the rest (+1).
    let account = Account {
        hero_luck: hero,
        ..Account::default()
    };
    assert_eq!(l(&account, 7), "g|-pendant-eye-l2-temp +1");
    // The rest by its sign; an empty list writes nothing.
    let rest = |total, skill| HeroList {
        total,
        artifacts: Vec::new(),
        skill,
        grail: None,
    };
    let account = |morale, luck| Account {
        hero_morale: morale,
        hero_luck: luck,
        ..Account::default()
    };
    assert_eq!(m(&account(rest(5, 2), rest(0, 0)), 5), "G|-L2-temp +3");
    assert_eq!(l(&account(rest(0, 0), rest(-1, 0)), -1), "b|-temp -1");
    assert_eq!(l(&account(rest(1, 3), rest(1, 3)), 1), "g|-l3-temp -2");
    assert_eq!(m(&account(rest(0, 0), rest(0, 0)), 0), "N|-none");
    // Cursed Ground's line and no morale's stay alone.
    let listed = Account {
        hero_morale: rest(3, 3),
        hero_luck: rest(3, 3),
        ..Account::default()
    };
    let cursed = Account {
        field: Field::Cursed,
        ..listed.clone()
    };
    assert_eq!(m(&cursed, 0), "N|-cursed");
    assert_eq!(l(&cursed, 0), "n|-cursed");
    let unaffected = Account {
        no_morale: true,
        ..listed
    };
    assert_eq!(m(&unaffected, 0), "N|-unaffected");
}

#[test]
fn the_heros_changes_are_the_bases() {
    // The minimums test the base with the hero's changes in it, and the spells' line is the
    // value less the base (0x44BF7B, 0x44C462).
    let c = described_catalogue();
    let skill = |level| HeroList {
        total: i32::from(level),
        artifacts: Vec::new(),
        skill: level,
        grail: None,
    };
    let minotaurs = Account {
        morale_minimum: Some("Minotaurs".into()),
        morale_spells: 2,
        hero_morale: skill(1),
        ..Account::default()
    };
    assert_eq!(describe::morale(&c, &minotaurs, 3), "G|-L1-spells +2");
    let halflings = Account {
        luck_minimum: Some("Halflings".into()),
        hero_luck: skill(1),
        ..Account::default()
    };
    assert_eq!(describe::luck(&c, &halflings, 1, Bugs::Fixed), "g|-l1");
    let halflings = Account {
        devils: Some("Devils".into()),
        luck_spells: 2,
        ..halflings
    };
    assert_eq!(
        describe::luck(&c, &halflings, 2, Bugs::Fixed),
        "g|-l1-facing Devils\nHalflings lucky-spells +1"
    );
}

#[test]
fn the_caps_artifacts_lines_override_the_others() {
    // E17 M6 T3c2 (design §12.10): the Hourglass of the Evil Hour's line alone in luck's
    // description, after Cursed Ground's; the Spirit of Oppression's replaces the lines before it
    // while the base is above 0 (after the Minotaurs' minimum), and the stack's morale is cut to 0.
    let c = described_catalogue();
    let hourglass = Account {
        hourglass: Some("Hourglass".into()),
        devils: Some("Devils".into()),
        luck_spells: 1,
        hero_luck: HeroList {
            total: 1,
            artifacts: vec![46],
            skill: 0,
            grail: None,
        },
        ..Account::default()
    };
    assert_eq!(
        describe::luck(&c, &hourglass, 0, Bugs::Fixed),
        "n|-hourglass Hourglass"
    );
    let cursed = Account {
        field: Field::Cursed,
        ..hourglass
    };
    assert_eq!(describe::luck(&c, &cursed, 0, Bugs::Fixed), "n|-cursed");
    let spirit = Account {
        town: Some(0),
        alignments: 1,
        undead: true,
        oppression: Some("Spirit".into()),
        ..Account::default()
    };
    // Base 1 - 1 = 0: nothing to suppress, the lines stay.
    assert_eq!(describe::morale(&c, &spirit, 0), "N|-one-undead");
    let high = Account {
        alignments: 2,
        undead: false,
        hero_morale: HeroList {
            total: 2,
            artifacts: vec![45, 49],
            skill: 0,
            grail: None,
        },
        ..spirit.clone()
    };
    // Base 2 (two artifacts): replaced by the suppression line alone.
    assert_eq!(describe::morale(&c, &high, 0), "N|-oppressed Spirit");
    // The Minotaurs' minimum first: a base of 0 becomes 1, which the Spirit then suppresses.
    let minotaurs = Account {
        morale_minimum: Some("Minotaurs".into()),
        ..spirit
    };
    assert_eq!(describe::morale(&c, &minotaurs, 0), "N|-oppressed Spirit");
}

#[test]
fn a_siege_towns_buildings_and_the_grail_have_their_lines() {
    // E17 M6 T3c2 (design §12.4, §12.9): at a siege the defender's morale lists the Tavern (+1)
    // and a Castle's Brotherhood of the Sword (+2) after the Angels' and dragons' lines and
    // before the Minotaurs', its luck a Rampart's Fountain of Fortune (+2) after the Devils'
    // and before the Halflings'; the owner's Grail line follows the hero's skill, before the
    // rest, and leaves the rest what it does not name.
    let c = described_catalogue();
    let morale = Account {
        angels: Some("Archangels".into()),
        dragons: Some("Ghost Dragons".into()),
        siege_morale: vec![Building::Tavern, Building::Brotherhood],
        morale_minimum: Some("Minotaurs".into()),
        ..Account::default()
    };
    assert_eq!(
        describe::morale(&c, &morale, 3),
        "G|-in Archangels-facing Ghost Dragons\nTavern +1\nBrotherhood +2"
    );
    // The buildings' changes are in the base: a base of 3 lifts the Minotaurs' minimum away and
    // the spells' line is the value less the base.
    let spells = Account {
        morale_spells: 1,
        ..morale
    };
    assert_eq!(
        describe::morale(&c, &spells, 4),
        "G|-in Archangels-facing Ghost Dragons\nTavern +1\nBrotherhood +2-spells +1"
    );
    let luck = Account {
        devils: Some("Devils".into()),
        siege_luck: vec![Building::Fountain],
        luck_minimum: Some("Halflings".into()),
        ..Account::default()
    };
    assert_eq!(
        describe::luck(&c, &luck, 1, Bugs::Fixed),
        "g|-facing Devils\nFountain +2"
    );
    // The Grail: after the skill, before the rest; its +2 is out of the rest.
    let list = |grail, total| HeroList {
        total,
        artifacts: vec![108],
        skill: 1,
        grail,
    };
    let account = Account {
        hero_morale: list(Some(Building::Colossus), 9),
        hero_luck: list(Some(Building::SpiritGuardian), 6),
        ..Account::default()
    };
    assert_eq!(
        describe::morale(&c, &account, 9),
        "G|-pendant-L1\nColossus +2-temp +3"
    );
    assert_eq!(
        describe::luck(&c, &account, 6, Bugs::Fixed),
        "g|-pendant-l1\nGuardian +2"
    );
    // Without a Grail the same total is all rest.
    let account = Account {
        hero_morale: list(None, 9),
        ..Account::default()
    };
    assert_eq!(describe::morale(&c, &account, 9), "G|-pendant-L1-temp +5");
}
