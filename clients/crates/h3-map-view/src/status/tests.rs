use super::*;
use alloc::collections::BTreeMap;
use alloc::vec;

use h3_ui::text::pack::Pack;
use h3_ui::ui::Draw;

use crate::live::{Explored, Kingdom, KingdomHero, KingdomTown, Moment, Walking};

fn layout() -> AdventureLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap()
}

pub(crate) fn art() -> StatusArt {
    let text = include_str!("../../../../map/assets/base.toml");
    crate::tables::parse(text).unwrap().map.unwrap().status
}

fn lists() -> ListArt {
    crate::lists::tests::art()
}

/// A catalogue whose messages show their keys' arguments.
fn texts() -> Catalogue {
    let json = concat!(
        r#"{ "map.status.number": "{{number}}", "map.status.thousands": "{{thousands}}k", "#,
        r#""map.status.date": "{{word}} {{number}}", "map.status.side": "{{word}}:", "#,
        r#""word": { "day": "Day", "week": "Week", "allies": "Allies", "enemies": "Enemies" }, "#,
        r#""hero": { "48": "Fiona" }, "town-name": { "37": "Arcton" } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

/// A map's own hero names: type 48's.
fn names() -> BTreeMap<u8, String> {
    BTreeMap::from([(48, "Own".into())])
}

fn live(kingdom: Kingdom) -> Live {
    Live {
        objects: Vec::new(),
        heroes: Vec::new(),
        boats: Vec::new(),
        viewer: 0,
        explored: Explored::all(1, 1, true),
        path: Vec::new(),
        moment: Moment::default(),
        walking: Walking::default(),
        kingdom,
        buttons: None,
        minimap: None,
        hover: None,
        hint: None,
        chosen: crate::live::Chosen::default(),
        pressed: Vec::new(),
    }
}

fn stack(creature: u16, count: i32) -> Option<Stack> {
    Some(Stack { creature, count })
}

fn fiona() -> HeroStatus {
    HeroStatus {
        hero: 48,
        own_name: false,
        skills: [120, -5, 0, -1],
        morale: -3,
        luck: 3,
        army: [
            stack(42, 22),
            None,
            stack(46, 9_999),
            stack(0, 10_000),
            stack(149, 123_456),
            None,
            stack(1, 1),
        ],
    }
}

fn hero_kingdom(status: HeroStatus) -> Kingdom {
    Kingdom {
        heroes: vec![
            KingdomHero::default(),
            KingdomHero {
                portrait: 48,
                mana: 115,
                ..KingdomHero::default()
            },
        ],
        selected: Selected::Hero(1),
        status: Some(Status::Hero(status)),
        ..Kingdom::default()
    }
}

fn town() -> TownStatus {
    TownStatus {
        name: 37,
        buildings: 1 << 12 | 1 << 8,
        garrison: true,
        army: [None, stack(13, 5), None, None, None, None, None],
        silo: vec![0, 2],
        income: 1_500,
    }
}

fn town_kingdom(status: TownStatus) -> Kingdom {
    Kingdom {
        towns: vec![KingdomTown {
            faction: 8,
            fort: true,
            built: true,
        }],
        selected: Selected::Town(0),
        status: Some(Status::Town(status)),
        ..Kingdom::default()
    }
}

fn texts_of(shown: &Shown) -> Vec<(u16, &str)> {
    shown
        .texts
        .iter()
        .map(|(&id, t)| (id, t.as_str()))
        .collect()
}

fn hidden(shown: &Shown) -> Vec<u16> {
    shown
        .states
        .iter()
        .filter(|(_, s)| s.hidden)
        .map(|(&id, _)| id)
        .collect()
}

#[test]
fn a_hero_s_window() {
    // 0x451AB0 (design §4.8): the large portrait, the name, the skills cut to 99 and to 0 (attack,
    // defense) or 1 (power, knowledge), the spell points, morale and luck + 3, the filled slots.
    let layout = layout();
    let (panel, shown) = window(
        &layout,
        &art(),
        &lists(),
        &live(hero_kingdom(fiona())),
        &names(),
        &texts(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(panel, &layout.status_hero);
    assert_eq!(shown.pictures[&id::PORTRAIT], "HPL048Hr.PCX");
    assert_eq!(
        texts_of(&shown),
        [
            (2002, "Fiona"),
            (2003, "99"),
            (2004, "0"),
            (2005, "1"),
            (2006, "1"),
            (2010, "115"),
            (2021, "22"),
            (2023, "9999"),
            (2024, "10k"),
            (2025, "123k"),
            (2027, "1"),
        ]
    );
    let frames: Vec<(u16, u32)> = shown.frames.iter().map(|(&i, &f)| (i, f)).collect();
    assert_eq!(
        frames,
        [
            (2008, 0),
            (2009, 6),
            (2011, 44),
            (2013, 48),
            (2014, 2),
            (2015, 151),
            (2017, 3)
        ]
    );
    assert_eq!(hidden(&shown), [2012, 2016, 2022, 2026]);
    // Drawn from the panel's corner (614, 400): the name at (680, 402), the fifth slot's count at
    // (668, 555), the pictures at the slots' places (the table 0x660BD0).
    let draws = panel.draws(&shown, &layout.player_palette);
    let at = |id_text: &str| {
        draws.iter().find_map(|d| match d {
            Draw::Label { x, y, text, .. } if text == id_text => Some((*x, *y)),
            _ => None,
        })
    };
    assert_eq!(at("Fiona"), Some((680, 402)));
    assert_eq!(at("123k"), Some((668, 555)));
    let pictures: Vec<(i32, i32, u32)> = draws
        .iter()
        .filter_map(|d| match d {
            Draw::Button {
                def, x, y, frame, ..
            } if def == "cprsmall.def" => Some((*x, *y, *frame)),
            _ => None,
        })
        .collect();
    assert_eq!(
        pictures,
        [
            (650, 473, 44),
            (722, 473, 48),
            (632, 521, 2),
            (668, 521, 151),
            (740, 521, 3)
        ]
    );
}

#[test]
fn a_hero_s_own_name_the_map_s() {
    // A record's name not its type's (a shipped map's hero): the map's; one the map lacks, the
    // type's.
    let name = |hero: u8| {
        let status = HeroStatus {
            hero,
            own_name: true,
            ..fiona()
        };
        let (_, shown) = window(
            &layout(),
            &art(),
            &lists(),
            &live(hero_kingdom(status)),
            &names(),
            &texts(),
        )
        .unwrap()
        .unwrap();
        shown.texts[&id::NAME].clone()
    };
    assert_eq!(name(48), "Own");
    assert_eq!(name(7), "hero.7");
}

#[test]
fn a_bonus_past_the_frames_shows_the_first() {
    let status = HeroStatus {
        morale: -7,
        luck: i8::MIN,
        ..fiona()
    };
    let (_, shown) = window(
        &layout(),
        &art(),
        &lists(),
        &live(hero_kingdom(status)),
        &names(),
        &texts(),
    )
    .unwrap()
    .unwrap();
    assert_eq!((shown.frames[&2008], shown.frames[&2009]), (0, 0));
}

#[test]
fn a_town_s_window() {
    // 0x4521F0 (design §4.8): the picture the list's less 2, the name, the City Hall (2) and the
    // Citadel (1), the garrison hero's mark, two silo resources, the income, the army packed.
    let layout = layout();
    let (panel, shown) = window(
        &layout,
        &art(),
        &lists(),
        &live(town_kingdom(town())),
        &names(),
        &texts(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(panel, &layout.status_town);
    let frames: Vec<(u16, u32)> = shown.frames.iter().map(|(&i, &f)| (i, f)).collect();
    // The list's frame of a Conflux with a Fort built in today: 2 + 2 x 8 + 1.
    assert_eq!(
        frames,
        [
            (2001, 17),
            (2003, 2),
            (2004, 1),
            (2007, 0),
            (2011, 15),
            (2018, 2)
        ]
    );
    // The filled slots one after another from the first place (0x452A38), not at their slots'.
    assert_eq!(
        texts_of(&shown),
        [(2002, "Arcton"), (2006, "1500"), (2021, "5")]
    );
    assert_eq!(
        hidden(&shown),
        [
            2012, 2013, 2014, 2015, 2016, 2017, 2019, 2022, 2023, 2024, 2025, 2026, 2027
        ]
    );
}

#[test]
fn a_town_s_hall_fort_garrison_and_silo_by_what_stands() {
    let shown = |status: TownStatus| {
        window(
            &layout(),
            &art(),
            &lists(),
            &live(town_kingdom(status)),
            &names(),
            &texts(),
        )
        .unwrap()
        .unwrap()
        .1
    };
    let art = art();
    // The halls tested in order, the Town Hall first; the forts so, none the fourth frame.
    assert_eq!(art.hall(0), 0);
    assert_eq!(art.hall(1 << 10), 0);
    assert_eq!(art.hall(1 << 11), 1);
    assert_eq!(art.hall(1 << 13), 3);
    assert_eq!(art.hall(1 << 11 | 1 << 13), 1);
    assert_eq!(art.fort(0), 3);
    assert_eq!(art.fort(1 << 7), 0);
    assert_eq!(art.fort(1 << 9), 2);
    assert_eq!(art.fort(1 << 7 | 1 << 9), 0);
    // No garrison hero: its mark hidden. One silo resource at the middle place; three, none.
    let one = shown(TownStatus {
        garrison: false,
        silo: vec![6],
        ..town()
    });
    assert!(one.state(id::GARRISON).hidden);
    assert_eq!(one.frames[&id::SILO_ONLY], 6);
    assert!(one.state(id::SILO).hidden && one.state(id::SILO_SECOND).hidden);
    let three = shown(TownStatus {
        silo: vec![0, 2, 4],
        ..town()
    });
    for id in [id::SILO, id::SILO_SECOND, id::SILO_ONLY] {
        assert!(three.state(id).hidden, "{id}");
    }
    assert!(!three.state(id::GARRISON).hidden);
    let none = shown(TownStatus {
        silo: Vec::new(),
        ..town()
    });
    assert!(none.state(id::SILO_ONLY).hidden);
    // The income's low 16 bits, signed (MCL-Q16): 40 000 shows -25 536.
    let rich = shown(TownStatus {
        income: 40_000,
        ..town()
    });
    assert_eq!(rich.texts[&id::INCOME], "-25536");
}

#[test]
fn nothing_drawn_but_a_hero_s_or_a_town_s_window_of_the_selection() {
    let layout = layout();
    let draw = |kingdom: Kingdom| {
        window(
            &layout,
            &art(),
            &lists(),
            &live(kingdom),
            &names(),
            &texts(),
        )
        .unwrap()
        .is_some()
    };
    assert!(!draw(Kingdom {
        status: None,
        ..hero_kingdom(fiona())
    }));
    assert!(!draw(Kingdom {
        status: Some(Status::Other(5)),
        ..hero_kingdom(fiona())
    }));
    // A hero's window with a town selected, a town's with a hero, a selection past its list.
    assert!(!draw(Kingdom {
        selected: Selected::Town(0),
        ..hero_kingdom(fiona())
    }));
    assert!(!draw(Kingdom {
        selected: Selected::Hero(0),
        ..town_kingdom(town())
    }));
    assert!(!draw(Kingdom {
        selected: Selected::Hero(5),
        ..hero_kingdom(fiona())
    }));
    assert!(!draw(Kingdom {
        selected: Selected::Town(5),
        ..town_kingdom(town())
    }));
    // A portrait the tables lack.
    let mut kingdom = hero_kingdom(fiona());
    kingdom.heroes[1].portrait = 200;
    assert_eq!(
        window(
            &layout,
            &art(),
            &lists(),
            &live(kingdom),
            &names(),
            &texts()
        )
        .unwrap_err(),
        "a hero's portrait is past the tables'"
    );
}

#[test]
fn a_key_no_pack_has_shows_as_itself() {
    let (_, shown) = window(
        &layout(),
        &art(),
        &lists(),
        &live(hero_kingdom(fiona())),
        &names(),
        &Catalogue::default(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(shown.texts[&id::NAME], "hero.48");
    assert_eq!(shown.texts[&2025], "map.status.thousands");
    assert_eq!(hero_key(0), "hero.0");
    assert_eq!(town_key(143), "town-name.143");
}

#[test]
fn the_tables_rules_checked() {
    let art = art();
    assert_eq!(art.check(), Ok(()));
    assert_eq!(art.portraits.len(), 163);
    let broken = |change: fn(&mut StatusArt)| {
        let mut broken = art.clone();
        change(&mut broken);
        broken.check().unwrap_err()
    };
    assert_eq!(broken(|a| a.portraits.clear()), "no portraits");
    assert_eq!(
        broken(|a| a.skills.bottom[2] = 100),
        "a skill's lowest above its highest"
    );
    assert_eq!(broken(|a| a.thousands = 0), "thousands below 1");
    // The date's (E24 M3 T5b2): a picture's frames for each picture, none 0.
    let unmatched = "the date's pictures and their frames do not match";
    assert_eq!(broken(|a| a.date.frames.truncate(4)), unmatched);
    assert_eq!(broken(|a| a.date.frames[2] = 0), unmatched);
}

fn date_kingdom(frame: u32, [month, week, day]: [u16; 3]) -> Kingdom {
    Kingdom {
        status: Some(Status::Date(DateStatus { frame })),
        date: Date { month, week, day },
        ..hero_kingdom(fiona())
    }
}

#[test]
fn the_date_s_window_by_the_day_or_the_week() {
    let layout = layout();
    let shown = |kingdom: Kingdom| {
        let (panel, shown) = window(
            &layout,
            &art(),
            &lists(),
            &live(kingdom),
            &names(),
            &texts(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(panel.place, layout.status_date.place);
        assert_eq!(panel.items.len(), 3);
        shown
    };
    // A day: the day's picture at its frame and the day's text, whatever the selection.
    let day = shown(date_kingdom(4, [1, 2, 3]));
    assert_eq!(day.defs[&id::DATE_PICTURE], "NewDay.def");
    assert_eq!(day.frames[&id::DATE_PICTURE], 4);
    assert_eq!(texts_of(&day), [(id::DATE_TEXT, "Day 3")]);
    // A week's first day: the week's picture and text; the game's first day the day's.
    let week = shown(date_kingdom(0, [2, 3, 1]));
    assert_eq!(week.defs[&id::DATE_PICTURE], "NewWeek3.def");
    assert_eq!(texts_of(&week), [(id::DATE_TEXT, "Week 3")]);
    let first = shown(Kingdom {
        selected: Selected::None,
        ..date_kingdom(0, [1, 1, 1])
    });
    assert_eq!(first.defs[&id::DATE_PICTURE], "NewDay.def");
    assert_eq!(texts_of(&first), [(id::DATE_TEXT, "Day 1")]);
    assert_eq!(
        shown(date_kingdom(0, [1, 2, 1])).defs[&id::DATE_PICTURE],
        "NewWeek2.def"
    );
}

#[test]
fn what_the_date_shows() {
    let art = art().date;
    let at = |month, week, day| art.shows(Date { month, week, day });
    assert_eq!(
        at(1, 1, 1),
        DateShows {
            def: "NewDay.def",
            frames: 10,
            week: false
        }
    );
    assert_eq!(
        at(3, 4, 1),
        DateShows {
            def: "NewWeek4.def",
            frames: 16,
            week: true
        }
    );
    assert_eq!(at(2, 1, 1).def, "NewWeek1.def");
    assert_eq!(at(2, 1, 2).def, "NewDay.def");
    // A week past the table's (or 0): the day's.
    assert_eq!(at(2, 5, 1).def, "NewDay.def");
    assert_eq!(at(2, 0, 1).def, "NewDay.def");
    // A picture without its frames given: one.
    let mut short = art.clone();
    short.frames.truncate(1);
    assert_eq!(
        short
            .shows(Date {
                month: 2,
                week: 3,
                day: 1
            })
            .frames,
        1
    );
}

#[test]
fn the_kingdom_s_window() {
    let layout = layout();
    let realm = KingdomStatus {
        halls: [0, 2, 1, 12],
        allies: vec![0, 2],
        enemies: vec![1, 3, 4, 5, 6, 7],
    };
    let kingdom = Kingdom {
        status: Some(Status::Kingdom(realm)),
        ..town_kingdom(town())
    };
    let (panel, shown) = window(
        &layout,
        &art(),
        &lists(),
        &live(kingdom),
        &names(),
        &texts(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(panel.place, layout.status_kingdom.place);
    // The halls' frames 0 to 3; the counts where there are towns; the labels.
    let frames: Vec<(u16, u32)> = shown.frames.iter().map(|(&i, &f)| (i, f)).collect();
    assert_eq!(frames[..4], [(2001, 0), (2002, 1), (2003, 2), (2004, 3)]);
    assert_eq!(
        texts_of(&shown),
        [
            (2022, "2"),
            (2023, "1"),
            (2024, "12"),
            (id::ALLIES, "Allies:"),
            (id::ENEMIES, "Enemies:"),
        ]
    );
    // Each row's flags from its first place, the colour the frame; the rest hidden.
    assert_eq!(
        frames[4..],
        [
            (2041, 0),
            (2042, 2),
            (2051, 1),
            (2052, 3),
            (2053, 4),
            (2054, 5),
            (2055, 6),
            (2056, 7),
        ]
    );
    let mut hidden_ids = vec![2021];
    hidden_ids.extend(2043..2049);
    hidden_ids.extend([2057, 2058]);
    assert_eq!(hidden(&shown), hidden_ids);
    // Every item the window gives is the panel's.
    for id in shown
        .frames
        .keys()
        .chain(shown.texts.keys())
        .chain(shown.states.keys())
    {
        assert!(panel.item(*id).is_some(), "{id}");
    }
}
