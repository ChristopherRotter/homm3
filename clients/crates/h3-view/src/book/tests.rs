use super::*;
use alloc::vec;

use crate::text::pack::Pack;

const PACK: &str = r#"{
    "book": {
        "place": "<{{spell}}>{{level}}/{{mastery}};{{points}}={{cost}}",
        "place-unskilled": "<{{spell}}>{{level}};{{points}}={{cost}}",
        "points": "SP",
        "level": { "1": "L1", "2": "L2", "3": "L3", "4": "L4", "5": "L5" },
        "mastery": { "basic": "Bas", "advanced": "Adv", "expert": "Exp" }
    }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

/// A spell of `schools` at `level`, its handle and number `n`, named `name`.
fn spell(n: u16, name: &str, level: u8, schools: u8) -> Spell {
    Spell {
        handle: n,
        original: u32::from(n),
        name: name.into(),
        description: format!("[{name}]\n\nIt does things."),
        level,
        schools,
        kind: Kind::Combat,
        mastery: 1,
        cost: 5,
    }
}

/// A hero with 20 spell points, Expert Fire and Basic Air, and `spells`.
fn hero(spells: Vec<Spell>) -> Hero {
    Hero {
        id: 7,
        mana: 20,
        schools: [1, 3, 0, 0],
        spells,
    }
}

/// Fourteen fire spells of level 1 (handles 0 to 13, names `f00` to `f13`) and an air one.
fn many() -> Hero {
    let mut spells: Vec<Spell> = (0..14)
        .map(|n| spell(n, &format!("f{n:02}"), 1, 2))
        .collect();
    spells.push(spell(20, "Gust", 2, 1));
    hero(spells)
}

#[test]
fn the_first_opening_shows_page_0_of_all_schools_and_the_memory_keeps_the_rest() {
    let mut memory = Memory::default();
    let mut book = Book::open(&mut memory, 7);
    assert_eq!(
        book,
        Book {
            tab: ALL,
            kind: Kind::Combat,
            page: 0
        }
    );
    // The fire tab's second page, remembered for the same hero (0x684B40, 0x6A34F4).
    let hero = many();
    let page = book.page(&hero, &catalogue());
    assert_eq!(
        book.click(&mut memory, TAB_AREAS + 1, (&hero, &page)),
        Outcome::Turned
    );
    assert_eq!(
        book.click(&mut memory, NEXT, (&hero, &page)),
        Outcome::Turned
    );
    assert_eq!(Book::open(&mut memory, 7), book);
    assert_eq!((book.tab, book.page), (2, 1));
    // An adventure bookmark last: the next opening shows combat spells of all schools, page 0.
    assert_eq!(book.key(&mut memory, "A", &page), Outcome::Turned);
    assert_eq!(Book::open(&mut memory, 7).tab, ALL);
    // Another hero forgets the last one's page and tab.
    let mut book = Book::open(&mut memory, 7);
    book.click(&mut memory, TAB_AREAS, (&hero, &page));
    assert_eq!(
        Book::open(&mut memory, 8),
        Book::open(&mut Memory::default(), 8)
    );
}

#[test]
fn the_page_lists_the_tab_s_spells_by_level_school_and_name() {
    let mut a = spell(1, "Bless", 1, 8);
    a.kind = Kind::Adventure;
    let hero = hero(vec![
        spell(2, "zap", 2, 2),
        spell(3, "Arrow", 1, ALL),
        spell(4, "bolt", 1, 2),
        spell(5, "Aim", 1, 2),
        spell(6, "Haste", 1, 1),
        a,
    ]);
    let book = Book::open(&mut Memory::default(), 7);
    // All schools: the arrow under the school its hero knows best (fire: Expert); fire's bit 2
    // after air's 1.
    let names: Vec<(&str, u8)> = book
        .entries(&hero)
        .into_iter()
        .map(|(s, school)| (s.name.as_str(), school))
        .collect();
    assert_eq!(
        names,
        [
            ("Haste", 1),
            ("Aim", 2),
            ("Arrow", 2),
            ("bolt", 2),
            ("zap", 2)
        ]
    );
    // A school's tab: its spells under its bit.
    let fire = Book { tab: 2, ..book };
    assert_eq!(fire.entries(&hero).len(), 4);
    let adventure = Book {
        kind: Kind::Adventure,
        ..book
    };
    assert_eq!(adventure.entries(&hero).len(), 1);
}

#[test]
fn the_best_school_is_tried_air_fire_earth_water() {
    assert_eq!(best_school([0, 0, 0, 0], ALL), 1);
    assert_eq!(best_school([0, 1, 0, 0], ALL), 2);
    assert_eq!(best_school([0, 0, 0, 1], ALL), 8);
    assert_eq!(best_school([0, 0, 2, 1], ALL), 4);
    assert_eq!(best_school([3, 3, 3, 3], 4 | 8), 8);
    assert_eq!(best_school([3, 3, 3, 3], 0), 0);
}

#[test]
fn a_page_holds_twelve_places_a_school_s_first_its_emblem() {
    let hero = many();
    let catalogue = catalogue();
    let all = Book::open(&mut Memory::default(), 7);
    let page = all.page(&hero, &catalogue);
    assert!(page.places.iter().all(Option::is_some));
    assert_eq!(page.emblem, None);
    assert!(!page.previous && page.next);
    let first = page.places[0].as_ref().unwrap();
    assert_eq!(first.text, "<f00>L1/Bas;SP=5");
    assert_eq!(first.hint, "[f00] (L1)");
    assert_eq!((first.school, first.mastery, first.greyed), (1, 1, false));
    let second = Book { page: 1, ..all }.page(&hero, &catalogue);
    let shown: Vec<bool> = second.places.iter().map(Option::is_some).collect();
    assert_eq!(&shown[..3], [true, true, true]);
    assert!(shown[3..].iter().all(|s| !s));
    assert!(second.previous && !second.next);
    // Fire's first page: the emblem over places 0 and 1, ten spells; its second starts two earlier.
    let fire = Book { tab: 2, ..all };
    let page = fire.page(&hero, &catalogue);
    assert_eq!(page.emblem, Some(1));
    assert!(page.places[..2].iter().all(Option::is_none));
    assert_eq!(page.places[2].as_ref().unwrap().handle, 0);
    let page = Book { page: 1, ..fire }.page(&hero, &catalogue);
    assert_eq!(page.places[0].as_ref().unwrap().handle, 10);
    assert_eq!(page.places[3].as_ref().unwrap().handle, 13);
    assert!(page.places[4].is_none() && !page.next);
}

#[test]
fn a_place_s_text_names_the_mastery_and_greys_a_spell_above_the_mana() {
    let mut dear = spell(1, "Dear", 3, 4);
    dear.cost = 21;
    dear.mastery = 0;
    let mut sure = spell(2, "Sure", 5, 4);
    sure.mastery = 3;
    let hero = hero(vec![dear, sure]);
    let page = Book::open(&mut Memory::default(), 7).page(&hero, &catalogue());
    let [Some(a), Some(b)] = [&page.places[0], &page.places[1]] else {
        panic!("two places");
    };
    assert_eq!((a.text.as_str(), a.greyed), ("<Dear>L3;SP=21", true));
    assert_eq!((b.text.as_str(), b.mastery), ("<Sure>L5/Exp;SP=5", 3));
}

#[test]
fn the_items_show_the_page() {
    let hero = many();
    let mut book = Book::open(&mut Memory::default(), 7);
    book.tab = 2;
    let mut page = book.page(&hero, &catalogue());
    page.places[3].as_mut().unwrap().greyed = true;
    let ribbons: Vec<String> = ["a.def", "f.def", "w.def", "e.def"]
        .map(String::from)
        .to_vec();
    let shown = book.shown(&page, &hero, (&ribbons, 3), "hi");
    assert_eq!(shown.players[&BACKGROUND], 3);
    // Places 0 and 1 under the emblem: nothing.
    assert!(shown.state(RIBBONS).hidden && shown.state(ICONS + 1).hidden);
    assert_eq!(shown.defs[&(RIBBONS + 2)], "f.def");
    assert_eq!(shown.frames[&(RIBBONS + 2)], 1);
    assert_eq!(shown.frames[&(ICONS + 2)], 0);
    assert!(shown.texts[&(TEXTS + 2)].starts_with("<f00>"));
    assert!(shown.state(TEXTS + 3).disabled && !shown.state(TEXTS + 2).disabled);
    assert_eq!(shown.frames[&EMBLEM], 1);
    assert_eq!(shown.frames[&TABS], 1);
    assert!(shown.state(PREVIOUS).hidden && !shown.state(NEXT).hidden);
    assert_eq!(
        (shown.texts[&MANA].as_str(), shown.texts[&STATUS].as_str()),
        ("20", "hi")
    );
    // All schools, the last page: no emblem, the tabs' frame 4, the next corner hidden; a school
    // without a ribbon DEF shows its frame alone.
    book.tab = ALL;
    book.page = 1;
    let page = book.page(&hero, &catalogue());
    let shown = book.shown(&page, &hero, (&ribbons[..1], 0), "");
    assert!(shown.state(EMBLEM).hidden && shown.state(NEXT).hidden);
    assert!(!shown.state(PREVIOUS).hidden);
    assert_eq!(shown.frames[&TABS], 4);
    // The fire spells' ribbons (places 0 and 1) have no DEF in the one given, the air one's has.
    assert!(!shown.defs.contains_key(&RIBBONS) && shown.defs.contains_key(&(RIBBONS + 2)));
}

#[test]
fn a_place_gives_its_spell_or_refuses_it() {
    let mut dear = spell(1, "Dear", 3, 4);
    dear.cost = 21;
    let hero = hero(vec![spell(0, "Cheap", 1, 4), dear]);
    let mut memory = Memory::default();
    let mut book = Book::open(&mut memory, 7);
    let page = book.page(&hero, &catalogue());
    assert_eq!(
        book.click(&mut memory, ICONS, (&hero, &page)),
        Outcome::Chosen(0)
    );
    assert_eq!(
        book.click(&mut memory, ICONS + 1, (&hero, &page)),
        Outcome::Refused { cost: 21, mana: 20 }
    );
    assert_eq!(
        book.click(&mut memory, ICONS + 2, (&hero, &page)),
        Outcome::Nothing
    );
    // An adventure spell in battle: its description.
    let mut boat = spell(9, "Boat", 1, 4);
    boat.kind = Kind::Adventure;
    let hero = self::hero(vec![boat]);
    book.click(&mut memory, ADVENTURE, (&hero, &page));
    let page = book.page(&hero, &catalogue());
    assert_eq!(
        book.click(&mut memory, ICONS, (&hero, &page)),
        Outcome::Described(9)
    );
    assert_eq!(
        book.click(&mut memory, ICONS + 1, (&hero, &page)),
        Outcome::Nothing
    );
}

#[test]
fn the_tabs_bookmarks_corners_and_close_turn_or_close_the_book() {
    let hero = many();
    let mut memory = Memory::default();
    let mut book = Book::open(&mut memory, 7);
    let page = book.page(&hero, &catalogue());
    let mut click = |book: &mut Book, id| book.click(&mut memory, id, (&hero, &page));
    // The tab already open, the bookmark already taken: nothing.
    assert_eq!(click(&mut book, TAB_AREAS + 4), Outcome::Nothing);
    assert_eq!(click(&mut book, COMBAT), Outcome::Nothing);
    assert_eq!(click(&mut book, TAB_AREAS + 3), Outcome::Turned);
    assert_eq!(book.tab, 8);
    assert_eq!(click(&mut book, NEXT), Outcome::Turned);
    assert_eq!(click(&mut book, PREVIOUS), Outcome::Turned);
    assert_eq!(book.page, 0);
    assert_eq!(click(&mut book, ADVENTURE), Outcome::Turned);
    assert_eq!(click(&mut book, COMBAT), Outcome::Turned);
    assert_eq!(book.kind, Kind::Combat);
    assert_eq!(click(&mut book, CLOSE), Outcome::Closed);
    assert_eq!(click(&mut book, BACKGROUND), Outcome::Nothing);
}

#[test]
fn the_keys_close_take_bookmarks_turn_pages_and_go_round_the_tabs() {
    let hero = many();
    let catalogue = catalogue();
    let mut memory = Memory::default();
    let mut book = Book::open(&mut memory, 7);
    let first = book.page(&hero, &catalogue);
    assert_eq!(book.key(&mut memory, "Escape", &first), Outcome::Closed);
    assert_eq!(book.key(&mut memory, "Return", &first), Outcome::Closed);
    assert_eq!(book.key(&mut memory, "C", &first), Outcome::Nothing);
    assert_eq!(book.key(&mut memory, "Left", &first), Outcome::Nothing);
    assert_eq!(book.key(&mut memory, "Right", &first), Outcome::Turned);
    let last = book.page(&hero, &catalogue);
    assert_eq!(book.key(&mut memory, "Right", &last), Outcome::Nothing);
    assert_eq!(book.key(&mut memory, "Left", &last), Outcome::Turned);
    assert_eq!(book.page, 0);
    // From all schools up: water, fire, earth, air, all again; down from all: air.
    let tabs: Vec<u8> = ["Up", "Up", "Up", "Up", "Up"]
        .into_iter()
        .map(|k| {
            book.key(&mut memory, k, &first);
            book.tab
        })
        .collect();
    assert_eq!(tabs, [4, 2, 8, 1, ALL]);
    book.key(&mut memory, "Down", &first);
    assert_eq!(book.tab, 1);
    assert_eq!(book.key(&mut memory, "A", &first), Outcome::Turned);
    assert_eq!(book.key(&mut memory, "C", &first), Outcome::Turned);
    assert_eq!(book.key(&mut memory, "W", &first), Outcome::Nothing);
}

fn layout() -> BookLayout {
    crate::layout::parse_battle(include_str!("../../../../layouts/battle.toml"))
        .unwrap()
        .book
}

const HINTS: &str = r#"{
    "book": {
        "level": { "1": "L1", "2": "L2" },
        "hint": { "fire": "Fire!", "close": "Shut" }
    }
}"#;

#[test]
fn the_open_book_takes_presses_keys_and_the_mouse() {
    let layout = layout();
    let catalogue = Catalogue {
        packs: vec![
            Pack::from_json("en", HINTS).unwrap(),
            Pack::from_json("en", PACK).unwrap(),
        ],
    };
    let mut memory = Memory::default();
    let mut open = Open::new(&mut memory, many(), &catalogue);
    assert_eq!((open.status.as_str(), open.cursor), ("", None));
    // The fire tab's area on the screen (the dialog at (90, 2), the area at its (523, 205)).
    assert_eq!(
        open.press(&mut memory, (&layout, &catalogue), [654, 228]),
        Outcome::Turned
    );
    assert_eq!((open.book.tab, open.page.emblem), (2, Some(1)));
    // Outside the book nothing takes the press.
    assert_eq!(
        open.press(&mut memory, (&layout, &catalogue), [20, 300]),
        Outcome::Nothing
    );
    assert_eq!(open.key(&mut memory, "Right", &catalogue), Outcome::Turned);
    assert_eq!(open.page.places[0].as_ref().map(|p| p.handle), Some(10));
    assert_eq!(open.key(&mut memory, "W", &catalogue), Outcome::Nothing);
    // The mouse over place 0's icon (its centre (246, 123)): its hint, the cursor over an item.
    assert!(open.hover(&mut memory, (&layout, &catalogue), [246, 123]));
    assert_eq!((open.status.as_str(), open.cursor), ("[f10] (L1)", Some(1)));
    // The same item again: nothing changes.
    assert!(!open.hover(&mut memory, (&layout, &catalogue), [247, 124]));
    // The fire tab's own hint, the close's; the book's picture has none; outside: nothing.
    assert!(open.hover(&mut memory, (&layout, &catalogue), [654, 228]));
    assert_eq!(open.status, "Fire!");
    assert!(open.hover(&mut memory, (&layout, &catalogue), [585, 430]));
    assert_eq!(open.status, "Shut");
    assert!(open.hover(&mut memory, (&layout, &catalogue), [300, 30]));
    assert_eq!((open.status.as_str(), open.cursor), ("", Some(1)));
    assert!(open.hover(&mut memory, (&layout, &catalogue), [20, 300]));
    assert_eq!((open.status.as_str(), open.cursor), ("", Some(0)));
    // On the air tab the emblem takes the mouse over place 0, whose icon is hidden.
    let mut open = Open::new(&mut memory, many(), &catalogue);
    open.key(&mut memory, "Down", &catalogue);
    assert!(open.hover(&mut memory, (&layout, &catalogue), [246, 123]));
    // Reopened, the mouse resting on the item it was over last: no hint (0x684B4C is never
    // reset; CLT-Q26).
    let mut again = Open::new(&mut memory, many(), &catalogue);
    assert!(!again.hover(&mut memory, (&layout, &catalogue), [246, 123]));
    assert_eq!(again.status, "");
    // The cursor it set, at the mouse: none yet in the reopened book; over an item frame 1.
    assert_eq!(again.cursor_draw(&layout, [1, 2]), None);
    assert!(matches!(
        open.cursor_draw(&layout, [246, 123]),
        Some(crate::scene::Draw::Cursor {
            frame: 1,
            x: 246,
            y: 123,
            hot: [0, 0],
            ..
        })
    ));
    // Its draws: the dialog's shadow, then its items as shown.
    let draws = again.draws(&layout, "PLAYERS.PAL", 2);
    assert!(
        draws[..4]
            .iter()
            .all(|d| matches!(d, crate::scene::Draw::Darken { .. }))
    );
    assert!(matches!(
        &draws[4],
        crate::scene::Draw::Picture { picture, player: Some(p), .. }
            if picture == "Spelback.pcx" && p.player == 2
    ));
}
