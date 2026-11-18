//! The advanced options' tests (E27 M3 T3a): the rows as `adv-single-town` recorded them, the
//! town's and the bonus's arrows, the popups, the hiding; the hero's arrows, portraits, names and
//! popups (T3b1, as `adv-single-hero` and `adv-single-maps` recorded them).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::ui::{Draw, Item, State};

use super::super::tests::{scenario_of, texts};
use super::super::{CustomHero, Entry, Format, Heroes, MainHero, Scenario, Seat, Texts};
use super::*;
use crate::shell::look::parse_layout;

const LAYOUT: &str = include_str!("../../../../../../layouts/shell.toml");

/// The advanced options' texts of the shell's layout: numbered texts for the tables.
fn advanced_texts() -> AdvancedTexts {
    let look = parse_layout(LAYOUT).unwrap().scenario.advanced;
    let numbered = |what: &str, n: usize| (0..n).map(|k| format!("{what} {k}")).collect();
    let creature_names = look
        .town_popup
        .creatures
        .iter()
        .flatten()
        .map(|&c| (c, format!("creature {c}")))
        .collect();
    AdvancedTexts {
        look,
        hero_names: numbered("hero", 156),
        portraits: (0..156).map(|k| format!("HPS{k:03}.pcx")).collect(),
        random_hero: [String::from("Random Hero"), String::from("A hero")],
        kinds: ["both", "human", "computer"].map(String::from),
        computer: String::from("Computer"),
        random: String::from("Random"),
        none: String::from("None"),
        towns: numbered("town", 9),
        bonuses: ["Artifact", "Gold", "Resource"].map(String::from).into(),
        durations: numbered("duration", 11),
        alignment: String::from("Town Alignment"),
        creatures: String::from("Associated Creatures"),
        creature_names,
        random_town: [String::from("Random Town"), String::from("A town")],
        bonus_popups: [
            ["Artifact Bonus", "", "An artifact"],
            ["Gold Bonus", "500-1000", "Gold"],
            ["Resource Bonus", "", ""],
            ["Random Bonus", "", "Anything"],
        ]
        .map(|p| p.map(String::from)),
        resources: (0..9)
            .map(|f| [format!("caption {f}"), format!("text {f}")])
            .collect(),
    }
}

/// A seat both may play, every faction offered at random, its main town generating its hero (an
/// Arrogance colour).
const fn open() -> Seat {
    Seat {
        human: true,
        computer: true,
        factions: 0x1FF,
        random_town: true,
        generates: true,
        random_hero: false,
        main_hero: None,
    }
}

/// A map of `seats` (none for a colour no one plays).
fn map_of(file: &str, seats: [Option<Seat>; 8]) -> Entry {
    Entry {
        file: file.to_string(),
        format: Some(Format::SoD),
        name: file.to_string(),
        description: String::from("A map."),
        size: 72,
        difficulty: 1,
        victory: None,
        standard: false,
        loss: None,
        seats: seats.map(Option::unwrap_or_default),
        teams: [0, 1, 2, 3, 4, 5, 6, 7],
        heroes: Heroes::default(),
    }
}

/// Arrogance with red's offers: Castle's heroes 0 to 2, Rampart's none.
fn arrogance_offered() -> Entry {
    let mut map = arrogance();
    map.heroes.offers[0][0] = vec![0, 1, 2];
    map
}

/// Arrogance: red, blue, purple and teal, each [`open`].
fn arrogance() -> Entry {
    let mut seats = [None; 8];
    for c in [0, 1, 5, 6] {
        seats[c] = Some(open());
    }
    map_of("Arrogance.h3m", seats)
}

/// The screen over `entries` with the advanced options.
fn with_options(entries: Vec<Entry>) -> Scenario {
    with_options_in(entries, Bugs::Fixed)
}

/// The screen over `entries` with the advanced options in the profile `bugs`.
fn with_options_in(entries: Vec<Entry>, bugs: Bugs) -> Scenario {
    let layout = parse_layout(LAYOUT).unwrap();
    Scenario::new(
        layout.scenario.dialog,
        (layout.scenario.list, layout.scenario.description),
        Texts {
            advanced: Some(advanced_texts()),
            ..texts(bugs)
        },
        entries,
        bugs,
    )
}

fn click(s: &mut Scenario, point: [i32; 2]) -> bool {
    s.moved(point);
    s.press(point);
    s.release().changed
}

/// The options shown over `entries` (Show Advanced Options clicked).
fn shown(entries: Vec<Entry>) -> Scenario {
    shown_in(entries, Bugs::Fixed)
}

/// The options shown over `entries` in the profile `bugs`.
fn shown_in(entries: Vec<Entry>, bugs: Bugs) -> Scenario {
    let mut s = with_options_in(entries, bugs);
    assert!(click(&mut s, [514, 519]));
    assert!(s.advanced_showing());
    s
}

/// The pictures drawn by name and corner.
fn pictures(draws: &[Draw]) -> Vec<(String, [i32; 2])> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Picture { picture, x, y, .. } => Some((picture.clone(), [*x, *y])),
            _ => None,
        })
        .collect()
}

/// The DEF frames drawn of `def` (any case) by corner.
fn frames(draws: &[Draw], def: &str) -> Vec<([i32; 2], u32)> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Button {
                def: name,
                frame,
                x,
                y,
                ..
            } if name.eq_ignore_ascii_case(def) => Some(([*x, *y], *frame)),
            _ => None,
        })
        .collect()
}

/// The texts drawn at `at`.
fn texts_at(draws: &[Draw], at: [i32; 2]) -> Vec<String> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label { x, y, text, .. } if [*x, *y] == at => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// Row `r`'s y less its row 0's.
const fn dy(r: i32) -> i32 {
    50 * r
}

#[test]
fn the_options_show_a_row_for_each_colour_that_plays() {
    let mut s = shown(vec![arrogance()]);
    let draws = s.draws("PLAYERS.PAL");
    let pictures = pictures(&draws);
    // The panel, each colour's bar on its row (purple's on row 2, teal's on row 3), Random towns
    // and heroes.
    assert!(pictures.contains(&(String::from("AdvOptBk.pcx"), [3, 6])));
    for (r, letter) in [(0, 'r'), (1, 'b'), (2, 'p'), (3, 't')] {
        let bar = format!("adop{letter}pnl.pcx");
        assert!(pictures.contains(&(bar, [57, 128 + dy(r)])), "{letter}");
        assert!(pictures.contains(&(String::from("hpsrand0.pcx"), [176, 130 + dy(r)])));
        assert!(pictures.contains(&(String::from("hpsrand1.pcx"), [252, 130 + dy(r)])));
        assert_eq!(texts_at(&draws, [164, 162 + dy(r)]), ["Random"]);
        assert_eq!(texts_at(&draws, [316, 162 + dy(r)]), ["Random"]);
    }
    // The flags (the original's frame 0) where a human may play; the bonus's picture Random.
    let flags = frames(&draws, "AOFLGBp.DEF");
    assert!(
        flags
            .iter()
            .all(|&(at, f)| at == [14, 130 + dy(2)] && f == 0)
    );
    assert!(!flags.is_empty());
    assert!(frames(&draws, "ScnrStar.def").iter().all(|&(_, f)| f == 10));
    // Who may play, the names (the single player's empty), the handicap drawn up.
    assert_eq!(texts_at(&draws, [62, 151 + dy(1)]), ["both", "both"]);
    assert_eq!(texts_at(&draws, [62, 130]), ["", ""]);
    assert_eq!(
        texts_at(&draws, [62, 130 + dy(3)]),
        ["Computer", "Computer"]
    );
    assert!(
        frames(&draws, "adopb2r.def")
            .iter()
            .all(|&(at, f)| at == [110, 151] && f == 0)
    );
    // The arrows: the town's and the bonus's on every row, the hero's on none (a Random town);
    // a right arrow up is its frame 1.
    let rights = frames(&draws, "adoprta.def");
    assert!(rights.contains(&([225, 133 + dy(3)], 1)));
    assert!(rights.contains(&([377, 133 + dy(2)], 1)));
    assert!(!rights.iter().any(|&(at, _)| at[0] == 301));
    // The headers, the title and the duration's text and bar.
    assert_eq!(texts_at(&draws, [58, 24]), ["advanced-title"]);
    assert_eq!(texts_at(&draws, [256, 556]), ["duration 10"]);
    assert_eq!(frames(&draws, "SlideBuH.def").len(), 3);
    // The records: the single player red's, every town Random.
    let records = s.records().unwrap();
    assert_eq!(records[0], Record::NEW.with_colour(0));
    assert_eq!(records[5], Record::NEW);
    // The handicap takes no press.
    s.moved([135, 163]);
    assert!(!s.press([135, 163]).changed);
}

impl Record {
    /// The record holding `colour`.
    const fn with_colour(self, colour: u8) -> Self {
        Self {
            colour: Some(colour),
            ..self
        }
    }
}

#[test]
fn the_town_s_arrows_step_through_the_allowed_factions_and_random() {
    let mut s = shown(vec![arrogance()]);
    let town = |s: &Scenario, r: i32| texts_at(&s.draws("P"), [164, 162 + dy(r)]);
    for t in 0..9 {
        assert!(click(&mut s, [230, 145]));
        assert_eq!(town(&s, 0), [format!("town {t}")]);
        let draws = s.draws("P");
        assert!(frames(&draws, "itpa.def").contains(&([176, 130], 2 * t + 2)));
        // The hero's arrows show under a town (the single player's, its main town generating).
        assert!(frames(&draws, "adoprta.def").contains(&([301, 133], 1)));
    }
    assert!(click(&mut s, [230, 145]));
    assert_eq!(town(&s, 0), ["Random"]);
    // From Random the left arrow goes to the far end, then back.
    assert!(click(&mut s, [169, 145]));
    assert_eq!(town(&s, 0), ["town 8"]);
    assert!(click(&mut s, [169, 145]));
    assert_eq!(town(&s, 0), ["town 7"]);
    // A computer's row (blue) too; the left arrow from Castle to Random.
    assert!(click(&mut s, [230, 195]));
    assert_eq!(town(&s, 1), ["town 0"]);
    assert!(click(&mut s, [169, 195]));
    assert_eq!(town(&s, 1), ["Random"]);
    assert_eq!(s.records().unwrap()[0].town, Some(7));
}

/// The hero's text and picture of row `r`.
fn hero_of(s: &Scenario, r: i32) -> (Vec<String>, Option<String>) {
    let draws = s.draws("P");
    let picture = pictures(&draws)
        .into_iter()
        .find(|(_, at)| *at == [252, 130 + dy(r)])
        .map(|(p, _)| p);
    (texts_at(&draws, [240, 162 + dy(r)]), picture)
}

#[test]
fn the_hero_s_arrows_step_through_the_offer_and_random() {
    let mut s = shown(vec![arrogance_offered()]);
    // Castle: right from Random to the offer's first, on to its last, past it to Random.
    click(&mut s, [230, 145]);
    let random = (
        vec![String::from("Random")],
        Some(String::from("hpsrand1.pcx")),
    );
    assert_eq!(hero_of(&s, 0), random);
    for k in 0..3 {
        assert!(click(&mut s, [306, 145]));
        let portrait = format!("HPS{k:03}.pcx");
        assert_eq!(hero_of(&s, 0), (vec![format!("hero {k}")], Some(portrait)));
        assert_eq!(s.records().unwrap()[0].hero, Some(k));
    }
    click(&mut s, [306, 145]);
    assert_eq!(hero_of(&s, 0), random);
    // Left: from Random to the last, back to the first, past it to Random.
    click(&mut s, [245, 145]);
    assert_eq!(hero_of(&s, 0).0, ["hero 2"]);
    click(&mut s, [245, 145]);
    click(&mut s, [245, 145]);
    assert_eq!(hero_of(&s, 0).0, ["hero 0"]);
    click(&mut s, [245, 145]);
    assert_eq!(hero_of(&s, 0), random);
    // A pick, then the town changed: Random again; Rampart offers none: its arrows do nothing.
    click(&mut s, [306, 145]);
    click(&mut s, [230, 145]);
    assert_eq!(hero_of(&s, 0), random);
    assert!(click(&mut s, [306, 145]));
    assert_eq!(hero_of(&s, 0), random);
    assert_eq!(s.records().unwrap()[0].hero, None);
    // The pick has no popup of its own yet (T3b2).
    click(&mut s, [169, 145]);
    click(&mut s, [306, 145]);
    assert_eq!(hero_of(&s, 0).0, ["hero 0"]);
    assert!(s.right([276, 146]).changed);
    assert!(s.box_draws("P").is_empty());
}

#[test]
fn a_pick_shows_the_map_s_customised_portrait_and_name() {
    let mut map = arrogance_offered();
    map.heroes.custom = vec![
        CustomHero {
            hero_type: 1,
            portrait: Some(40),
            name: String::from("Sir One"),
        },
        CustomHero {
            hero_type: 2,
            portrait: None,
            name: String::new(),
        },
    ];
    let mut s = shown(vec![map.clone()]);
    click(&mut s, [230, 145]);
    click(&mut s, [306, 145]);
    click(&mut s, [306, 145]);
    let one = (
        vec![String::from("Sir One")],
        Some(String::from("HPS040.pcx")),
    );
    assert_eq!(hero_of(&s, 0), one);
    click(&mut s, [306, 145]);
    let two = (
        vec![String::from("hero 2")],
        Some(String::from("HPS002.pcx")),
    );
    assert_eq!(hero_of(&s, 0), two);
    // A main hero's own name of the map names a pick in the original alone
    // (`scenario_shown_hero_name` 0x58D33A, CLT-Q142 (b)).
    map.seats[0].main_hero = Some(MainHero {
        hero_type: 7,
        portrait: Some(19),
    });
    map.heroes.main_names[0] = String::from("Tovar");
    for (bugs, name) in [(Bugs::Fixed, "hero 0"), (Bugs::Original, "Tovar")] {
        let mut s = shown_in(vec![map.clone()], bugs);
        click(&mut s, [230, 145]);
        click(&mut s, [306, 145]);
        assert_eq!(hero_of(&s, 0).0, [name]);
    }
}

/// Adventures of Jared Haret's kinds of rows (`adv-single-maps` map-2), and two more: red's main
/// hero of a portrait of its own and a name, blue's (a fixed town) and tan's (a Random town) of
/// their types' own portraits (CLT-Q141), a colour with no hero at all, a portrait past the table.
fn main_heroes() -> Entry {
    let computer = Seat {
        human: false,
        random_town: false,
        generates: false,
        ..open()
    };
    let main = |hero_type, portrait| {
        Some(MainHero {
            hero_type,
            portrait,
        })
    };
    let mut seats = [None; 8];
    seats[0] = Some(Seat {
        human: true,
        factions: 1,
        main_hero: main(2, Some(0)),
        ..computer
    });
    seats[1] = Some(Seat {
        factions: 1 << 5,
        main_hero: main(85, None),
        ..computer
    });
    seats[2] = Some(Seat {
        random_town: true,
        main_hero: main(33, None),
        ..computer
    });
    seats[3] = Some(Seat {
        factions: 1,
        ..computer
    });
    seats[4] = Some(Seat {
        factions: 1,
        main_hero: main(5, Some(200)),
        ..computer
    });
    let mut map = map_of("Jared.h3m", seats);
    map.heroes.main_names[0] = String::from("Jared Haret");
    map.heroes.main_names[2] = String::from("Thenith");
    map
}

#[test]
fn a_main_hero_shows_its_portrait_and_name() {
    let mut s = shown(vec![main_heroes()]);
    let hero = |k: &str, p: &str| (vec![String::from(k)], Some(String::from(p)));
    assert_eq!(hero_of(&s, 0), hero("Jared Haret", "HPS000.pcx"));
    // CLT-Q141 (b): the type's own portrait; the map's name, else the type's.
    assert_eq!(hero_of(&s, 1), hero("hero 85", "HPS085.pcx"));
    assert_eq!(hero_of(&s, 2), hero("Thenith", "HPS033.pcx"));
    assert_eq!(hero_of(&s, 3), hero("None", "hpsrand6.pcx"));
    // A portrait past the table: its name alone.
    assert_eq!(hero_of(&s, 4), (vec![String::from("hero 5")], None));
    // No popup over a None (CLT-Q112 (b)), none yet over a hero (T3b2).
    for r in 0..4 {
        assert!(s.right([276, 146 + dy(r)]).changed);
        assert!(s.box_draws("P").is_empty(), "{r}");
        s.right_release();
    }
}

#[test]
fn the_original_shows_no_main_hero_of_its_type_s_portrait() {
    let mut s = shown_in(vec![main_heroes()], Bugs::Original);
    let hero = |k: &str, p: &str| (vec![String::from(k)], Some(String::from(p)));
    assert_eq!(hero_of(&s, 0), hero("Jared Haret", "HPS000.pcx"));
    assert_eq!(hero_of(&s, 1), hero("None", "hpsrand6.pcx"));
    assert_eq!(hero_of(&s, 2), hero("Random", "hpsrand1.pcx"));
    assert_eq!(hero_of(&s, 3), hero("None", "hpsrand6.pcx"));
    // Random's popup over blue's None and tan's Random (`map-2-computer-hero-popup`), none over
    // a colour with no hero at all (`map-1-hero-popup`).
    for (r, random) in [(1, true), (2, true), (3, false)] {
        s.right([276, 146 + dy(r)]);
        let labels = popup(&s).0;
        assert_eq!(!labels.is_empty(), random, "{r}");
        s.right_release();
    }
}

#[test]
fn a_mask_s_factions_alone_are_stepped_to() {
    let rampart_and_dungeon = Seat {
        factions: 0b10_0010,
        random_town: false,
        ..open()
    };
    let mut seats = [None; 8];
    seats[0] = Some(rampart_and_dungeon);
    let mut s = shown(vec![map_of("Two.h3m", seats)]);
    let town = |s: &Scenario| texts_at(&s.draws("P"), [164, 162]);
    for expected in ["town 1", "town 5", "Random", "town 1"] {
        click(&mut s, [230, 145]);
        assert_eq!(town(&s), [expected]);
    }
    click(&mut s, [169, 145]);
    click(&mut s, [169, 145]);
    assert_eq!(town(&s), ["town 5"]);
}

#[test]
fn the_bonus_s_arrows_pass_over_what_the_colour_cannot_take() {
    let mut s = shown(vec![arrogance()]);
    let bonus = |s: &Scenario| {
        let draws = s.draws("P");
        let frame = frames(&draws, "ScnrStar.def")
            .into_iter()
            .find(|&(at, _)| at == [328, 130])
            .map(|(_, f)| f);
        (texts_at(&draws, [316, 162]), frame)
    };
    // Under a Random town: right from Random to the artifact and gold, past the resource.
    click(&mut s, [382, 145]);
    assert_eq!(bonus(&s), (vec![String::from("Artifact")], Some(9)));
    click(&mut s, [382, 145]);
    assert_eq!(bonus(&s), (vec![String::from("Gold")], Some(8)));
    click(&mut s, [382, 145]);
    assert_eq!(bonus(&s), (vec![String::from("Random")], Some(10)));
    // Left from Random under a Random town: past the resource to gold.
    click(&mut s, [321, 145]);
    assert_eq!(bonus(&s), (vec![String::from("Gold")], Some(8)));
    // Under Conflux the resource shows Inferno's picture (3); a town's own otherwise.
    for _ in 0..9 {
        click(&mut s, [230, 145]);
    }
    click(&mut s, [382, 145]);
    assert_eq!(bonus(&s), (vec![String::from("Resource")], Some(3)));
    click(&mut s, [169, 145]);
    assert_eq!(bonus(&s).1, Some(7));
    // The town Random again: the resource made Random.
    for _ in 0..8 {
        click(&mut s, [169, 145]);
    }
    assert_eq!(bonus(&s), (vec![String::from("Random")], Some(10)));
    assert_eq!(s.records().unwrap()[0].bonus, Bonus::Random);
}

#[test]
fn a_colour_with_no_hero_to_take_it_passes_over_the_artifact() {
    let fixed = Seat {
        factions: 0b100,
        random_town: false,
        generates: false,
        ..open()
    };
    let mut seats = [None; 8];
    seats[0] = Some(fixed);
    seats[1] = Some(Seat {
        main_hero: Some(MainHero {
            hero_type: 3,
            portrait: Some(4),
        }),
        ..fixed
    });
    seats[2] = Some(Seat {
        human: false,
        random_town: true,
        generates: false,
        ..open()
    });
    seats[3] = Some(Seat {
        computer: false,
        generates: false,
        random_hero: true,
        ..open()
    });
    let mut s = shown(vec![map_of("Fixed.h3m", seats)]);
    let draws = s.draws("P");
    // A fixed town (Tower) shows without its arrows; no hero at all shows none; the main hero its
    // portrait and its type's name (T3b1); a Random town with neither flag shows none too.
    assert_eq!(texts_at(&draws, [164, 162]), ["town 2"]);
    assert!(!frames(&draws, "adoprta.def").contains(&([225, 133], 1)));
    assert!(pictures(&draws).contains(&(String::from("hpsrand6.pcx"), [252, 130])));
    assert_eq!(texts_at(&draws, [240, 162]), ["None"]);
    assert!(pictures(&draws).contains(&(String::from("HPS004.pcx"), [252, 130 + dy(1)])));
    assert_eq!(texts_at(&draws, [240, 162 + dy(1)]), ["hero 3"]);
    assert_eq!(texts_at(&draws, [240, 162 + dy(2)]), ["None"]);
    // A human-only colour (green, the single player's: the first only a human may play) says so;
    // under a town its random-hero flag gives it the hero's arrows.
    assert_eq!(texts_at(&draws, [62, 151 + dy(3)]), ["human", "human"]);
    assert_eq!(s.records().unwrap()[3].colour, Some(3));
    click(&mut s, [230, 145 + dy(3)]);
    assert!(frames(&s.draws("P"), "adoprta.def").contains(&([301, 133 + dy(3)], 1)));
    let draws = s.draws("P");
    // A computer-only colour has no flag; its town may change, so its bonus may too.
    assert!(frames(&draws, "AOFLGBy.DEF").is_empty());
    assert!(frames(&draws, "adoprta.def").contains(&([377, 133 + dy(2)], 1)));
    // Red right from Random: past the artifact to gold; left from gold to Random.
    click(&mut s, [382, 145]);
    assert_eq!(texts_at(&s.draws("P"), [316, 162]), ["Gold"]);
    click(&mut s, [321, 145]);
    assert_eq!(texts_at(&s.draws("P"), [316, 162]), ["Random"]);
    // Blue (a main hero) takes the artifact.
    click(&mut s, [382, 195]);
    assert_eq!(texts_at(&s.draws("P"), [316, 162 + dy(1)]), ["Artifact"]);
    click(&mut s, [321, 195]);
    assert_eq!(texts_at(&s.draws("P"), [316, 162 + dy(1)]), ["Random"]);
    // A heroless left from the resource of a fixed town: the artifact passed over to Random.
    click(&mut s, [321, 145]);
    assert_eq!(texts_at(&s.draws("P"), [316, 162]), ["Resource"]);
    click(&mut s, [321, 145]);
    click(&mut s, [321, 145]);
    assert_eq!(texts_at(&s.draws("P"), [316, 162]), ["Random"]);
}

/// The popup's labels' texts and its DEF frames and pictures.
fn popup(s: &Scenario) -> (Vec<String>, Vec<(String, u32)>, Vec<String>) {
    let draws = s.box_draws("P");
    let labels = draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    let defs = draws
        .iter()
        .filter_map(|d| match d {
            Draw::Button { def, frame, .. } if !def.eq_ignore_ascii_case("dialgbox.def") => {
                Some((def.clone(), *frame))
            }
            _ => None,
        })
        .collect();
    let pictures = pictures(&draws)
        .into_iter()
        .map(|(p, _)| p)
        .filter(|p| p != "diboxbck.pcx")
        .collect();
    (labels, defs, pictures)
}

#[test]
fn the_pictures_right_press_shows_their_popups_while_held() {
    let mut s = shown(vec![arrogance()]);
    // A Random town's.
    assert!(s.right([200, 146]).changed);
    assert!(s.cursor());
    let (labels, defs, pictures) = popup(&s);
    assert_eq!(labels, ["Random Town", "Random", "A town"]);
    assert!(defs.is_empty());
    assert_eq!(pictures, ["hpsrand0.pcx"]);
    assert!(s.right_release().changed);
    assert!(s.box_draws("P").is_empty());
    // Conflux's: its picture, name and creatures by level, three then four.
    for _ in 0..9 {
        click(&mut s, [230, 145]);
    }
    s.right([200, 146]);
    let (labels, defs, _) = popup(&s);
    assert_eq!(
        labels[..3],
        ["Town Alignment", "town 8", "Associated Creatures"]
    );
    assert_eq!(labels[3], "creature 118");
    assert_eq!(labels[9], "creature 130");
    assert_eq!(defs[0], (String::from("itpa.def"), 18));
    assert_eq!(defs[1], (String::from("cprsmall.def"), 120));
    let places: Vec<[i32; 2]> = s
        .box_draws("P")
        .iter()
        .filter_map(|d| match d {
            Draw::Button { def, x, y, .. } if def == "cprsmall.def" => Some([*x, *y]),
            _ => None,
        })
        .collect();
    assert_eq!(places[0], [272 + 60, 140 + 159]);
    assert_eq!(places[3], [272 + 40, 140 + 235]);
    s.right_release();
    // The bonuses': Random, then the resource of Conflux's town, gold, the artifact.
    s.right([352, 146]);
    assert_eq!(popup(&s).0, ["Random Bonus", "", "Anything"]);
    assert_eq!(popup(&s).1, [(String::from("ScnrStar.def"), 10)]);
    s.right_release();
    click(&mut s, [321, 145]);
    s.right([352, 146]);
    assert_eq!(popup(&s).0, ["Resource Bonus", "caption 8", "text 8"]);
    assert_eq!(popup(&s).1, [(String::from("ScnrStar.def"), 3)]);
    s.right_release();
    click(&mut s, [321, 145]);
    s.right([352, 146]);
    assert_eq!(popup(&s).0, ["Gold Bonus", "500-1000", "Gold"]);
    s.right_release();
    click(&mut s, [321, 145]);
    s.right([352, 146]);
    assert_eq!(popup(&s).0, ["Artifact Bonus", "", "An artifact"]);
    s.right_release();
    // The hero's picture, Random under a town with its arrows: Random's popup (T3b1).
    assert!(s.right([276, 146]).changed);
    assert_eq!(popup(&s).0, ["Random Hero", "Random", "A hero"]);
    assert_eq!(popup(&s).2, ["hpsrand1.pcx"]);
}

#[test]
fn the_options_hide_with_their_button_and_the_list() {
    let mut s = shown(vec![arrogance(), map_of("Other.h3m", [Some(open()); 8])]);
    let hidden = |s: &Scenario| {
        !pictures(&s.draws("P"))
            .iter()
            .any(|(p, _)| p == "AdvOptBk.pcx")
    };
    // A town chosen, the options hidden by their button and shown again: the choice kept.
    click(&mut s, [230, 145]);
    assert!(click(&mut s, [514, 519]));
    assert!(hidden(&s));
    assert!(!s.advanced_showing());
    click(&mut s, [514, 519]);
    assert_eq!(texts_at(&s.draws("P"), [164, 162]), ["town 0"]);
    // The list hides them; the options hide the list.
    assert!(click(&mut s, [514, 91]));
    assert!(s.showing());
    assert!(hidden(&s));
    // Another map selected (its row under the kept one), its rows made anew at the showing: eight
    // Random rows.
    click(&mut s, [216, 160]);
    click(&mut s, [514, 519]);
    assert!(!s.showing());
    let draws = s.draws("P");
    assert_eq!(texts_at(&draws, [164, 162]), ["Random"]);
    assert_eq!(texts_at(&draws, [164, 162 + dy(7)]), ["Random"]);
    // With the options hidden, the list's column under their header takes its press.
    click(&mut s, [514, 519]);
    click(&mut s, [514, 91]);
    assert!(click(&mut s, [216, 100]));
}

#[test]
fn opened_anew_the_screen_keeps_one_row_of_items_a_colour() {
    let s = shown(vec![arrogance()]);
    let opened = s.opened(crate::shell::menu::Hovers::default());
    let flags = |s: &Scenario| s.dialog.items.iter().filter(|i| i.id() == FLAG).count();
    assert_eq!((flags(&s), flags(&opened)), (1, 1));
    // Its options hidden and its records new.
    assert!(!opened.advanced_showing());
    assert_eq!(opened.records().unwrap()[0], Record::NEW);
}

#[test]
fn over_no_map_the_options_show_no_row() {
    let mut s = with_options(Vec::new());
    assert!(click(&mut s, [514, 519]));
    let draws = s.draws("P");
    assert!(pictures(&draws).iter().any(|(p, _)| p == "AdvOptBk.pcx"));
    assert!(texts_at(&draws, [164, 162]).is_empty());
    // A right press on a row's place opens nothing.
    s.right([200, 146]);
    assert!(s.box_draws("P").is_empty());
}

#[test]
fn without_their_texts_there_are_no_options() {
    let mut s = scenario_of(vec![arrogance()], Bugs::Fixed);
    assert!(!click(&mut s, [514, 519]) || !s.advanced_showing());
    assert_eq!(s.records(), None);
}

#[test]
fn the_items_and_names_of_every_colour() {
    let look = advanced_texts().look;
    let items = Advanced::items(&look);
    assert_eq!(items.len(), 8 * 13);
    // Pink's flag on row 7, its DEF's letter s.
    let pink = items.iter().find(|i| i.id() == FLAG + 7).unwrap();
    assert!(matches!(pink, Item::Button(b) if b.def == "AOFLGBs.DEF" && b.at == [14, 480]));
    let names = Advanced::names(&look);
    assert!(names.iter().any(|n| n == "adopspnl.pcx"));
    assert!(names.iter().any(|n| n == "cprsmall.def"));
    assert!(Advanced::owns(BONUS_AREA + 7) && !Advanced::owns(BONUS_AREA + 8));
    assert!(Advanced::takes(TOWN_LEFT) && !Advanced::takes(HANDICAP + 3));
    // Out of the letters: none; a mask with no faction: Castle; codes modulo 4.
    let short = AdvancedLook {
        letters: String::from("r"),
        ..look
    };
    assert_eq!(letter(&short, 3), "");
    assert_eq!(first_faction(0), 0);
    assert_eq!(Bonus::of(7), Bonus::Random);
    // The options' own state: shown nowhere until shown; a click or a right press then nothing.
    let mut advanced = Advanced::new(advanced_texts(), Bugs::Fixed);
    let mut shown = Shown::default();
    let map = Map {
        seats: [open(); 8],
        heroes: Heroes::default(),
        local: Some(0),
    };
    assert!(!advanced.click(TOWN_RIGHT, &map, &mut shown));
    assert_eq!(advanced.popup(TOWN_AREA, &map), None);
    let mut dialog = Dialog {
        place: [0, 0],
        shadow: None,
        items: Vec::new(),
    };
    advanced.show(&map, &mut dialog, &mut shown);
    assert!(!advanced.click(PLAYER, &map, &mut shown));
    // A hero's arrow of a colour no human record holds (its arrows hidden): no list, no step
    // (`scenario_dialog_click` 0x586A93).
    assert!(advanced.click(HERO_RIGHT + 1, &map, &mut shown));
    assert_eq!(advanced.record(1).hero, None);
    assert_eq!(advanced.popup(PLAYER, &map), None);
    assert_eq!(shown.state(HANDICAP), State::default());
}
