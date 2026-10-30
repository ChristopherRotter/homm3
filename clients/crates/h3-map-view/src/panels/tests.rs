use super::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_map::model::Pos;

use crate::live::{Arrow, Explored, Kingdom, KingdomHero, KingdomTown, Moment, Walking};

fn layout() -> AdventureLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap()
}

fn lists() -> ListArt {
    lists::tests::art()
}

const AWAKE: KingdomHero = KingdomHero {
    asleep: false,
    movable: true,
    portrait: 0,
    points: 0,
    mana: 0,
};

/// The dialog's items for `live` on a map of `levels` with the view on `z`.
fn dialog(layout: &AdventureLayout, live: &Live, levels: usize, z: u8) -> Shown {
    super::dialog(layout, &lists(), live, levels, z).unwrap()
}

/// A live part of the viewer 2 with `kingdom` and `path`.
fn live(kingdom: Kingdom, path: Vec<Arrow>) -> Live {
    Live {
        objects: Vec::new(),
        heroes: Vec::new(),
        boats: Vec::new(),
        viewer: 2,
        explored: Explored::all(1, 1, true),
        path,
        moment: Moment::default(),
        walking: Walking::default(),
        kingdom,
        buttons: None,
        minimap: None,
        hover: None,
        hint: None,
    }
}

fn one_hero() -> Kingdom {
    Kingdom {
        heroes: vec![AWAKE],
        selected: Selected::Hero(0),
        ..Kingdom::default()
    }
}

fn arrow(arrow: u16) -> Arrow {
    Arrow {
        at: Pos { x: 1, y: 1, z: 0 },
        arrow,
    }
}

/// The ids shown disabled.
fn disabled(shown: &Shown) -> Vec<u16> {
    shown
        .states
        .iter()
        .filter(|(_, s)| s.disabled)
        .map(|(&id, _)| id)
        .collect()
}

fn def(shown: &Shown, id: u16) -> &str {
    shown.defs.get(&id).map(String::as_str).unwrap()
}

#[test]
fn a_map_start_with_its_hero_selected() {
    // heroes-objects as the original showed it (design §4.1): one level, no quest, no path.
    let layout = layout();
    let shown = dialog(&layout, &live(one_hero(), vec![]), 1, 0);
    assert_eq!(disabled(&shown), [4, 5, 7, 13, 14, 30, 31]);
    assert_eq!(def(&shown, id::UNDERGROUND), "iam010.def");
    assert_eq!(def(&shown, id::SLEEP), "iam005.def");
    let coloured: Vec<(u16, u8)> = shown.players.into_iter().collect();
    assert_eq!(coloured, (3..=12).map(|id| (id, 2)).collect::<Vec<_>>());
}

#[test]
fn move_hero_needs_a_green_arrow_or_the_timeline_s_word() {
    let layout = layout();
    let moved = |path, buttons| {
        let mut l = live(one_hero(), path);
        l.buttons = buttons;
        !disabled(&dialog(&layout, &l, 1, 0)).contains(&id::MOVE_HERO)
    };
    assert!(moved(vec![arrow(26), arrow(25)], None));
    assert!(!moved(vec![arrow(26), arrow(50)], None));
    let off = Buttons {
        move_hero: false,
        next_hero: true,
    };
    assert!(!moved(vec![arrow(1)], Some(off)));
    assert!(moved(
        vec![],
        Some(Buttons {
            move_hero: true,
            ..off
        })
    ));
}

#[test]
fn a_town_selected_disables_the_hero_s_buttons() {
    let layout = layout();
    let kingdom = Kingdom {
        heroes: vec![AWAKE],
        towns: vec![KingdomTown::default()],
        selected: Selected::Town(0),
        ..Kingdom::default()
    };
    let shown = dialog(&layout, &live(kingdom, vec![arrow(1)]), 1, 0);
    assert_eq!(disabled(&shown), [4, 5, 6, 7, 8, 13, 14, 30, 31]);
    // Nothing selected likewise.
    let none = Kingdom::default();
    let shown = dialog(&layout, &live(none, vec![]), 1, 0);
    assert_eq!(disabled(&shown), [4, 5, 6, 7, 8, 11, 13, 14, 30, 31]);
}

#[test]
fn the_underground_by_the_levels_and_sleep_by_the_hero() {
    let layout = layout();
    let mut kingdom = one_hero();
    kingdom.heroes[0].asleep = true;
    kingdom.quests = true;
    let shown = dialog(&layout, &live(kingdom, vec![]), 2, 1);
    // Enabled on two levels, the underground's DEF; the quest log enabled; next hero disabled
    // with its only hero asleep.
    assert_eq!(disabled(&shown), [7, 11, 13, 14, 30, 31]);
    assert_eq!(def(&shown, id::UNDERGROUND), "iam003.def");
    assert_eq!(def(&shown, id::SLEEP), "iam011.def");
}

#[test]
fn the_lists_arrows_by_their_tops_and_lengths() {
    let layout = layout();
    let kingdom = Kingdom {
        heroes: vec![AWAKE; 7],
        towns: vec![KingdomTown::default(); 6],
        selected: Selected::Hero(9),
        tops: [1, 0],
        quests: false,
        ..Kingdom::default()
    };
    let shown = dialog(&layout, &live(kingdom.clone(), vec![]), 1, 0);
    // A selection past the list counts as an awake hero.
    assert_eq!(disabled(&shown), [4, 5, 7, 30]);
    let bottom = Kingdom {
        tops: [2, 1],
        ..kingdom
    };
    let shown = dialog(&layout, &live(bottom, vec![]), 1, 0);
    assert_eq!(disabled(&shown), [4, 5, 7, 14, 31]);
}

#[test]
fn the_resource_bar_s_picture_in_the_viewer_s_colours() {
    let layout = layout();
    let shown = resource_bar(
        &layout,
        &live(Kingdom::default(), vec![]),
        &Catalogue::default(),
    );
    assert_eq!(shown.players.into_iter().collect::<Vec<_>>(), [(1000, 2)]);
}

#[test]
fn the_resource_bar_s_amounts_and_date_by_their_keys() {
    // E24 M3 T5a (design §4.7): 0x558F55's seven amounts by `%d`, 0x558F8B's date with the
    // month's, week's and day's words; a key no pack has shows as itself.
    let layout = layout();
    let kingdom = Kingdom {
        resources: [0, 7, 123, -4, 99_999, 1_000_000, 2_147_483_647],
        date: crate::live::Date {
            month: 12,
            week: 4,
            day: 7,
        },
        ..Kingdom::default()
    };
    let json = concat!(
        r#"{ "map.resources.amount": "<{{amount}}>", "#,
        r#""map.resources.date": "{{day_word}} {{day}} {{week_word}} {{week}} "#,
        r#"{{month_word}} {{month}}", "word.month": "M", "word.week": "W", "word.day": "D" }"#,
    );
    let pack = h3_ui::text::pack::Pack::from_json("en", json).unwrap();
    let texts = Catalogue { packs: vec![pack] };
    let shown = resource_bar(&layout, &live(kingdom.clone(), vec![]), &texts);
    let all: Vec<(u16, &str)> = shown
        .texts
        .iter()
        .map(|(&id, t)| (id, t.as_str()))
        .collect();
    assert_eq!(
        all,
        [
            (1001, "<0>"),
            (1002, "<7>"),
            (1003, "<123>"),
            (1004, "<-4>"),
            (1005, "<99999>"),
            (1006, "<1000000>"),
            (1007, "<2147483647>"),
            (1008, "D 7 W 4 M 12"),
        ]
    );
    let bare = resource_bar(&layout, &live(kingdom, vec![]), &Catalogue::default());
    assert_eq!(bare.texts[&1001], "map.resources.amount");
    assert_eq!(bare.texts[&1008], "map.resources.date");
    // Drawn as the layout's labels: the amounts at (37 + 84 i, 3), the date at (607, 3), each
    // from the bar's corner (3, 575).
    let draws = layout.resource_bar.draws(&shown, &layout.player_palette);
    let labels: Vec<(i32, i32, &str)> = draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label { x, y, text, .. } => Some((*x, *y, text.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(labels.len(), 8);
    assert_eq!(labels[0], (40, 578, "<0>"));
    assert_eq!(labels[6], (544, 578, "<2147483647>"));
    assert_eq!(labels[7], (610, 578, "D 7 W 4 M 12"));
}

#[test]
fn the_lists_slots_and_their_selection_frames_hidden() {
    // hero_list_slot 0x403560, town_list_slot 0x4037A0 (design §4.5): the portraits, the bars'
    // and the towns' frames from the lists' tops; an empty slot keeps the layout's picture.
    let layout = layout();
    let kingdom = Kingdom {
        heroes: vec![
            AWAKE,
            KingdomHero {
                portrait: 9,
                points: 2500,
                mana: 30,
                ..AWAKE
            },
        ],
        towns: vec![
            KingdomTown {
                faction: 3,
                fort: true,
                built: true,
            },
            KingdomTown::default(),
        ],
        selected: Selected::Hero(1),
        tops: [1, 1],
        quests: false,
        ..Kingdom::default()
    };
    let shown = dialog(&layout, &live(kingdom.clone(), vec![]), 1, 0);
    let pictures: Vec<(u16, &str)> = shown
        .pictures
        .iter()
        .map(|(&id, p)| (id, p.as_str()))
        .collect();
    assert_eq!(pictures, [(15, "HPS009Cl.PCX")]);
    let frames: Vec<(u16, u32)> = shown.frames.iter().map(|(&id, &f)| (id, f)).collect();
    let expected: Vec<(u16, u32)> = [(20, 24), (25, 6)]
        .into_iter()
        .chain((21..25).map(|id| (id, 0)))
        .chain((26..30).map(|id| (id, 0)))
        .chain([(32, 20)])
        .chain((33..37).map(|id| (id, 0)))
        .collect::<alloc::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(frames, expected);
    let hidden: Vec<u16> = shown
        .states
        .iter()
        .filter(|(_, s)| s.hidden)
        .map(|(&id, _)| id)
        .collect();
    assert_eq!(hidden, [39, 40, 41, 42, 43]);
    // A portrait past the tables' draws no dialog.
    let mut past = kingdom;
    past.heroes[1].portrait = 200;
    assert_eq!(
        super::dialog(&layout, &lists(), &live(past, vec![]), 1, 0),
        None
    );
}

#[test]
fn the_selections_drawn_after_the_items() {
    // lists_selection_frames 0x4038C0: hpsyyy.pcx over the selected hero's slot, itpa.def's
    // frame 1 over the selected town's picture; nothing for a selection outside the slots.
    let layout = layout();
    let art = lists();
    let kingdom = Kingdom {
        heroes: vec![AWAKE; 7],
        towns: vec![KingdomTown::default(); 7],
        selected: Selected::Hero(6),
        tops: [2, 2],
        quests: false,
        ..Kingdom::default()
    };
    let draws = |selected| {
        let k = Kingdom {
            selected,
            ..kingdom.clone()
        };
        selections(&layout, &art, &live(k, vec![]))
    };
    assert_eq!(
        draws(Selected::Hero(6)),
        [Draw::Picture {
            picture: "hpsyyy.pcx".into(),
            x: 617,
            y: 340,
            player: None,
            size: Some([48, 32]),
        }]
    );
    assert_eq!(
        draws(Selected::Town(2)),
        [Draw::Button {
            def: "itpa.def".into(),
            frame: 1,
            x: 747,
            y: 212,
            mirrored: false,
            player: None,
        }]
    );
    assert_eq!(draws(Selected::Hero(1)), []);
    assert_eq!(draws(Selected::None), []);
    // A layout without the item, or a DEF item without its DEF: nothing.
    let mut bare = layout.clone();
    bare.dialog.items.retain(|i| i.id() != 39);
    let k = Kingdom {
        selected: Selected::Hero(2),
        ..kingdom.clone()
    };
    assert_eq!(selections(&bare, &art, &live(k, vec![])), []);
    for item in &mut bare.dialog.items {
        if let Item::Def(d) = item {
            d.def = None;
        }
    }
    let k = Kingdom {
        selected: Selected::Town(2),
        ..kingdom
    };
    assert_eq!(selections(&bare, &art, &live(k, vec![])), []);
}
