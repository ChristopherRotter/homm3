//! A level-up's facts, levels, texts and dialog (E24 M4 T2c, design §5.5).

use super::*;

use alloc::vec;
use h3_ui::text::pack::Pack;

use crate::layout::AdventureLayout;

/// The tables' thresholds (`[map.level_up]`).
fn art() -> LevelArt {
    LevelArt {
        experience: vec![
            0, 1000, 2000, 3200, 4600, 6200, 8000, 10000, 12200, 14700, 17500, 20600,
        ],
        ..LevelArt::default()
    }
}

/// The committed layout's `[level_up]`.
fn layout() -> LevelUpLayout {
    AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml"))
        .unwrap()
        .level_up
}

/// A `[[level_up]]` table as the harness writes it.
fn table(hero: i64, level: i64, primary: i64, offers: [i64; 2]) -> toml::Value {
    let text = alloc::format!(
        "hero = {hero}\nlevel = {level}\noffers = [{}, {}]\npictures = [[\"secskill.def\", 1], \
         [\"pskil42.def\", {primary}], [\"hpl016rn.pcx\", -1], [\"lvlupbkg.pcx\", -1]]\n",
        offers[0],
        offers[1]
    );
    toml::Value::Table(text.parse().unwrap())
}

/// A catalogue of the dialog's keys and the names it uses.
fn catalogue() -> Catalogue {
    let json = concat!(
        r#"{ "map": { "level-up": { "gained": "{{name}} gained", "#,
        r#""now": "{{name}} {{level}} {{class}}", "primary": "{{skill}} +1", "#,
        r#""learned": "learned {{level}} {{skill}}", "#,
        r#""either": "either {{level}} {{skill}} or {{second_level}} {{second}}", "#,
        r#""caption": "{{level}}/{{skill}}" }, "ask": { "or": "or" } }, "#,
        r#""primary-skill": { "1": "Defense" }, "#,
        r#""secondary-skill": { "11": "Eagle Eye", "23": "Armorer" }, "#,
        r#""skill-level": { "0": "Basic", "1": "Advanced" } }"#,
    );
    Catalogue {
        packs: vec![Pack::from_json("en", json).unwrap()],
    }
}

#[test]
fn the_level_an_experience_reaches() {
    let art = art();
    assert_eq!(art.level_of(0), 1);
    assert_eq!(art.level_of(999), 1);
    assert_eq!(art.level_of(1000), 2);
    // `level-up-twice`: 900 and the chest's 1 500.
    assert_eq!(art.level_of(2400), 3);
    // `find-stone`: 3 200 and the stone's 1 000, no level.
    assert_eq!((art.level_of(3200), art.level_of(4200)), (4, 4));
    assert_eq!(art.level_of(20600), 12);
    // Beyond the table each step a fifth more: 3 720 to level 13, 4 464 to 14.
    assert_eq!(art.level_of(24319), 12);
    assert_eq!(art.level_of(24320), 13);
    assert_eq!(art.level_of(28783), 13);
    assert_eq!(art.level_of(28784), 14);
    // Held at level 255; no table, level 1.
    assert_eq!(art.level_of(u32::MAX), u8::MAX);
    assert_eq!(LevelArt::default().level_of(5000), 1);
    let flat = LevelArt {
        experience: vec![0, 0],
        ..LevelArt::default()
    };
    assert_eq!(flat.level_of(9), 2);
}

#[test]
fn an_offers_frame() {
    // 3 × the skill + 3 + its level before: Advanced Armorer 73, Basic Eagle Eye 36.
    let armorer = Offer::of_frame(73).unwrap();
    assert_eq!(
        armorer,
        Offer {
            skill: 23,
            level: 2
        }
    );
    assert_eq!(armorer.frame(), 73);
    assert_eq!(
        Offer::of_frame(36),
        Some(Offer {
            skill: 11,
            level: 1
        })
    );
    assert_eq!(Offer::of_frame(-1), None);
    assert_eq!(Offer::of_frame(2), None);
    assert_eq!(Offer::of_frame(3 * 300), None);
}

#[test]
fn reads_the_harnesss_tables() {
    let up = LevelUp::read(&table(16, 2, 1, [73, 36])).unwrap();
    assert_eq!(
        up,
        LevelUp {
            hero: 16,
            level: 2,
            primary: 1,
            offers: vec![Offer::of_frame(73).unwrap(), Offer::of_frame(36).unwrap()],
        }
    );
    assert_eq!(
        LevelUp::read(&table(16, 2, 3, [-1, -1])).unwrap().offers,
        []
    );
    assert_eq!(
        LevelUp::read(&table(16, 2, 3, [36, -1]))
            .unwrap()
            .offers
            .len(),
        1
    );
    // A primary skill past Knowledge, an offer's frame, a hero or a level out of range, no
    // table; a key missing or not a list, too few pictures, an offer not a number.
    let without = |key: &str| {
        let mut t = table(16, 2, 1, [73, 36]);
        t.as_table_mut().unwrap().remove(key);
        t
    };
    let mut few = table(16, 2, 1, [73, 36]);
    few["pictures"] = toml::Value::Array(Vec::new());
    let mut word = table(16, 2, 1, [73, 36]);
    word["offers"] = toml::Value::Array(vec![toml::Value::String("x".into())]);
    for bad in [
        table(16, 2, 4, [-1, -1]),
        table(16, 2, 1, [2, -1]),
        table(300, 2, 1, [-1, -1]),
        table(16, 300, 1, [-1, -1]),
        toml::Value::Integer(1),
        without("pictures"),
        without("offers"),
        without("hero"),
        without("level"),
        few,
        word,
    ] {
        assert_eq!(LevelUp::read(&bad), None, "{bad:?}");
    }
    let mut three = table(16, 2, 1, [36, 36]);
    three["offers"] = toml::Value::Array(vec![toml::Value::Integer(36); 3]);
    assert_eq!(LevelUp::read(&three), None);
    // One level-up a dialog: its tables appended at each change are the same dialog's.
    let tables = toml::Value::Array(vec![
        table(16, 2, 1, [73, 36]),
        table(16, 2, 1, [73, 36]),
        table(16, 3, 3, [22, 63]),
        table(16, 3, 3, [22, 63]),
    ]);
    let ups = LevelUp::read_all(&tables).unwrap();
    assert_eq!(
        ups.iter().map(|u| (u.level, u.primary)).collect::<Vec<_>>(),
        [(2, 1), (3, 3)]
    );
    let broken = toml::Value::Array(vec![toml::Value::Integer(1)]);
    assert_eq!(LevelUp::read_all(&broken), None);
    assert_eq!(LevelUp::read_all(&toml::Value::Integer(1)), None);
}

#[test]
fn the_dialogs_texts() {
    let texts = catalogue();
    let two = LevelUp::read(&table(16, 2, 1, [73, 36])).unwrap();
    let shown = two.texts(&texts, "Mephala", "Ranger");
    assert_eq!(shown[&GAINED], "Mephala gained");
    assert_eq!(shown[&NOW], "Mephala 2 Ranger");
    assert_eq!(shown[&RAISED], "Defense +1");
    assert_eq!(
        shown[&SENTENCE],
        "either Advanced Armorer or Basic Eagle Eye"
    );
    assert_eq!(shown[&OR], "or");
    assert_eq!(
        [shown[&CAPTIONS[0]].as_str(), shown[&CAPTIONS[1]].as_str()],
        ["Advanced/Armorer", "Basic/Eagle Eye"]
    );
    // One offer: GENRLTXT 204's sentence, one caption, no "or"; none: the three texts alone.
    let one = LevelUp {
        offers: vec![Offer::of_frame(36).unwrap()],
        ..two.clone()
    };
    let shown = one.texts(&texts, "Mephala", "Ranger");
    assert_eq!(shown[&SENTENCE], "learned Basic Eagle Eye");
    assert!(!shown.contains_key(&OR) && !shown.contains_key(&CAPTIONS[1]));
    let none = LevelUp {
        offers: Vec::new(),
        ..two
    };
    assert_eq!(
        none.texts(&texts, "M", "R")
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [GAINED, NOW, RAISED]
    );
}

#[test]
fn the_dialog_with_two_offers_one_or_none() {
    let layout = layout();
    let two = LevelUp::read(&table(16, 2, 1, [73, 36])).unwrap();
    let texts = BTreeMap::from([(GAINED, String::from("gained"))]);
    let laid = dialog(&layout, &two, texts.clone(), "HPL016Rn.pcx", 3);
    assert_eq!(laid.dialog.place, [205, 65]);
    assert_eq!(laid.dialog.shadow, Some([385, 470]));
    let ids: Vec<u16> = laid.dialog.items.iter().map(Item::id).collect();
    assert_eq!(
        ids,
        [
            2000, 2001, 2002, 2003, 2004, 2009, 2005, 2006, 2012, 2013, 2010, 2011, 2007, 2008, OK
        ]
    );
    let shown = &laid.shown;
    assert_eq!(shown.texts, texts);
    assert_eq!(shown.players[&BACKGROUND], 3);
    assert_eq!(shown.pictures[&PORTRAIT], "HPL016Rn.pcx");
    assert_eq!(
        [PRIMARY, PICTURES[0], PICTURES[1]].map(|id| shown.frames[&id]),
        [1, 73, 36]
    );
    assert!(FRAMES.iter().all(|&id| shown.state(id).hidden));
    assert!(shown.state(OK).disabled);
    assert_eq!(chosen(&laid, 2), None);
    assert!(frame_draws(&laid, "game.pal").is_empty());
    // One: its frame in the middle shown, chosen at once, the OK enabled.
    let one = LevelUp {
        offers: vec![Offer::of_frame(36).unwrap()],
        ..two.clone()
    };
    let laid = dialog(&layout, &one, BTreeMap::new(), "p.pcx", 0);
    let ids: Vec<u16> = laid.dialog.items.iter().map(Item::id).collect();
    assert_eq!(
        ids,
        [
            2000, 2001, 2002, 2003, 2004, 2009, 2005, 2012, 2010, 2007, OK
        ]
    );
    assert!(!laid.shown.state(OK).disabled);
    assert_eq!(chosen(&laid, 1), Some(0));
    // Its outline: 47 × 46 at (205 + 169, 65 + 325), in entry 45.
    let draws = frame_draws(&laid, "game.pal");
    assert_eq!(draws.len(), 4);
    assert_eq!(
        draws[0],
        Draw::Fill {
            rect: [374, 390, 47, 1],
            palette: "game.pal".into(),
            entry: message::FRAME_ENTRY,
        }
    );
    // None: the head and the OK.
    let none = LevelUp {
        offers: Vec::new(),
        ..two
    };
    let laid = dialog(&layout, &none, BTreeMap::new(), "p.pcx", 0);
    assert_eq!(laid.dialog.items.len(), 7);
    assert_eq!(chosen(&laid, 0), None);
}

#[test]
fn an_offer_chosen_by_its_picture_its_frame_or_its_key() {
    let layout = layout();
    let two = LevelUp::read(&table(16, 2, 1, [73, 36])).unwrap();
    let mut laid = dialog(&layout, &two, BTreeMap::new(), "p.pcx", 0);
    // The recorded clicks: the left picture (351, 413), the right (449, 413); a frame's edge;
    // the OK, the "or" and the background are no offer.
    assert_eq!(offer_at(&laid, [351, 413]), Some(0));
    assert_eq!(offer_at(&laid, [449, 413]), Some(1));
    assert_eq!(offer_at(&laid, [205 + 122, 65 + 325]), Some(0));
    assert_eq!(offer_at(&laid, [205 + 266, 65 + 370]), Some(1));
    assert_eq!(offer_at(&laid, [533, 493]), None);
    assert_eq!(offer_at(&laid, [400, 410]), None);
    assert_eq!(offer_at(&laid, [10, 10]), None);
    // The right: its frame shown, the left's hidden, the OK enabled; then the left.
    assert!(choose(&mut laid, 1, 2));
    assert_eq!(chosen(&laid, 2), Some(1));
    assert!(!laid.shown.state(OK).disabled);
    assert_eq!(frame_draws(&laid, "game.pal").len(), 4);
    assert!(choose(&mut laid, 0, 2));
    assert_eq!(chosen(&laid, 2), Some(0));
    assert!(laid.shown.state(FRAMES[1]).hidden);
    // No third offer; with one offer no right one; none with none.
    assert!(!choose(&mut laid, 2, 3));
    let one = LevelUp {
        offers: vec![Offer::of_frame(36).unwrap()],
        ..two
    };
    let mut laid = dialog(&layout, &one, BTreeMap::new(), "p.pcx", 0);
    assert_eq!(offer_at(&laid, [449, 413]), None);
    assert!(!choose(&mut laid, 1, 1));
    assert!(choose(&mut laid, 0, 1));
    assert!(!choose(&mut laid, 0, 0));
}
