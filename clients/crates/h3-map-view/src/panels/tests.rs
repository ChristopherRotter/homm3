use super::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_map::model::Pos;

use crate::live::{Arrow, Explored, Kingdom, KingdomHero, Moment, Walking};

fn layout() -> AdventureLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap()
}

const AWAKE: KingdomHero = KingdomHero {
    asleep: false,
    movable: true,
};

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
    assert!(moved(vec![], Some(Buttons { move_hero: true, ..off })));
}

#[test]
fn a_town_selected_disables_the_hero_s_buttons() {
    let layout = layout();
    let kingdom = Kingdom {
        heroes: vec![AWAKE],
        towns: 1,
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
        towns: 6,
        selected: Selected::Hero(9),
        tops: [1, 0],
        quests: false,
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
    let shown = resource_bar(&layout, &live(Kingdom::default(), vec![]));
    assert_eq!(shown.players.into_iter().collect::<Vec<_>>(), [(1000, 2)]);
}
