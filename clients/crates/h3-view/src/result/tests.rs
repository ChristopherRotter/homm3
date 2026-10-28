use super::*;
use alloc::string::ToString;
use alloc::vec;
use h3_kernel::Handle;

fn assets() -> ResultLayout {
    ResultLayout {
        picture: "CPResult.pcx".to_string(),
        title_font: "BigFont.fnt".to_string(),
        font: "smalfont.fnt".to_string(),
        font_palette: "game.pal".to_string(),
        creatures: "TwCrPort.def".to_string(),
        losses: "cprsmall.def".to_string(),
        button_back: "Box64x30.pcx".to_string(),
        ok: "iOkay.def".to_string(),
    }
}

fn texts() -> Texts {
    Texts::of(&Catalogue::default(), "the result".to_string())
}

fn hero(name: &str) -> Face {
    Face::Hero {
        name: name.to_string(),
        portrait: "HPL000Kn.PCX".to_string(),
    }
}

fn labels(window: &MessageBox) -> Vec<(u16, [i32; 2], u32, u32, &str)> {
    window
        .dialog
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(l) => Some((
                l.id,
                l.at,
                l.colour,
                l.align,
                window.shown.texts.get(&l.id).map_or("", String::as_str),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn the_window_is_laid_out_as_the_executable_builds_it() {
    let sides = [
        Side {
            face: hero("Orrin"),
            victorious: false,
            losses: Vec::new(),
        },
        Side {
            face: hero("Crag Hack"),
            victorious: true,
            losses: Vec::new(),
        },
    ];
    let window = layout(&assets(), &texts(), &sides, 0);
    assert_eq!(window.dialog.place, [165, 19]);
    assert_eq!(window.dialog.shadow, Some([470, 561]));
    // The background first, in the viewing player's colours.
    assert!(matches!(
        &window.dialog.items[0],
        Item::Picture(p) if p.picture == "CPResult.pcx" && p.at == [0, 0] && p.size == Some([470, 561])
    ));
    assert_eq!(window.shown.players.get(&BACKGROUND), Some(&0));
    assert_eq!(
        labels(&window),
        vec![
            (0x100, [0, 290], 8, 1, "result.title"),
            (0x101, [0, 320], 4, 1, "result.attacker"),
            (0x102, [0, 412], 4, 1, "result.defender"),
            (ATTACKER_NAME, [89, 37], 4, 0, "Orrin"),
            (0xCB, [83, 112], 4, 2, ""),
            (DEFENDER_NAME, [266, 37], 4, 2, "Crag Hack"),
            (0xCE, [266, 38], 4, 0, ""),
            (0x103, [17, 116], 1, 1, "result.defeated"),
            (0x104, [367, 116], 1, 1, "result.victorious"),
            (TEXT, [68, 202], 4, 5, "the result"),
            (LOSSES, [42, 353], 1, 1, "result.none"),
            (LOSSES + 0x10, [42, 450], 1, 1, "result.none"),
        ]
    );
    // The heroes' portraits as pictures; the OK with its back, Enter and Escape.
    for (id, at) in [
        (ATTACKER_PORTRAIT, [21, 38]),
        (DEFENDER_PORTRAIT, [392, 38]),
    ] {
        assert!(matches!(
            window.dialog.item(id),
            Some(Item::Picture(p)) if p.at == at && p.picture == "HPL000Kn.PCX"
        ));
    }
    let n = window.dialog.items.len();
    assert!(matches!(
        &window.dialog.items[n - 2],
        Item::Picture(p) if p.picture == "Box64x30.pcx" && p.at == [384, 506]
    ));
    assert!(matches!(
        &window.dialog.items[n - 1],
        Item::Button(b) if b.id == OK && b.at == [385, 507] && b.keys == ["Return", "Escape"]
    ));
    assert_eq!(window.dialog.button_for_key("Escape"), Some(OK));
}

#[test]
fn a_side_without_a_hero_shows_its_creature_the_defender_s_mirrored() {
    let creature = |original| Face::Creature {
        name: "Pikemen".to_string(),
        original,
    };
    let sides = [
        Side {
            face: creature(0),
            victorious: true,
            losses: Vec::new(),
        },
        Side {
            face: creature(13),
            victorious: false,
            losses: Vec::new(),
        },
    ];
    let window = layout(&assets(), &texts(), &sides, 1);
    let def = |id| match window.dialog.item(id) {
        Some(Item::Def(d)) => Some((d.def.as_deref(), d.frame, d.mirrored)),
        _ => None,
    };
    assert_eq!(
        def(ATTACKER_PORTRAIT),
        Some((Some("TwCrPort.def"), 2, false))
    );
    assert_eq!(
        def(DEFENDER_PORTRAIT),
        Some((Some("TwCrPort.def"), 15, true))
    );
    // The mirror reaches the draw list.
    let draws = window.dialog.draws(&window.shown, "PLAYERS.PAL");
    assert!(draws.iter().any(|d| matches!(
        d,
        Draw::Button { def, frame: 15, mirrored: true, x: 557, y: 57, .. } if def == "TwCrPort.def"
    )));
}

#[test]
fn losses_are_centred_seven_at_most_their_counts_under_them() {
    let losses = |n: u32| (0..n).map(|k| (k, 10 + k)).collect::<Vec<_>>();
    let sides = [
        Side {
            face: hero("A"),
            victorious: true,
            losses: losses(1),
        },
        Side {
            face: hero("B"),
            victorious: false,
            losses: losses(9),
        },
    ];
    let window = layout(&assets(), &texts(), &sides, 0);
    let pictures: Vec<(u16, [i32; 2], u32)> = window
        .dialog
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Def(d) if d.def.as_deref() == Some("cprsmall.def") => Some((d.id, d.at, d.frame)),
            _ => None,
        })
        .collect();
    // One loss: (468 − 42) / 2 + 11; seven of nine: (468 − 294) / 2 + 11, then by 42.
    assert_eq!(pictures[0], (LOSSES, [224, 343], 2));
    assert_eq!(pictures.len(), 1 + LOSSES_SHOWN);
    assert_eq!(pictures[1], (LOSSES + 0x10, [98, 440], 2));
    assert_eq!(pictures[7], (LOSSES + 0x10 + 12, [98 + 6 * 42, 440], 8));
    let count = labels(&window)
        .into_iter()
        .find(|l| l.0 == LOSSES + 1)
        .map(|l| (l.1, l.4.to_string()));
    assert_eq!(count, Some(([219, 381], "10".to_string())));
}

#[test]
fn the_backdrop_darkens_the_whole_screen() {
    assert_eq!(
        backdrop([800, 600]),
        Draw::Darken {
            rect: [0, 0, 800, 600]
        }
    );
}

#[test]
fn the_shown_side_is_a_human_winner_s_else_the_other_one() {
    assert_eq!(shown_side(Some(0), [true, false]), 0);
    assert_eq!(shown_side(Some(1), [true, false]), 0);
    assert_eq!(shown_side(Some(0), [false, false]), 1);
    assert_eq!(shown_side(Some(1), [true, true]), 1);
    assert_eq!(shown_side(None, [true, true]), 1);
}

#[test]
fn the_text_follows_the_shown_side_s_fate() {
    let c = Catalogue::default();
    assert_eq!(
        text(&c, true, None, Some("Orrin"), 600),
        "result.wonresult.experience"
    );
    assert_eq!(text(&c, true, None, None, 600), "result.won");
    assert_eq!(
        text(&c, true, Some(GaveUp::Retreat), None, 0),
        "result.won.fled"
    );
    assert_eq!(
        text(&c, true, Some(GaveUp::Surrender), None, 0),
        "result.won.surrendered"
    );
    assert_eq!(text(&c, false, None, Some("Orrin"), 0), "result.lost");
    assert_eq!(
        text(&c, false, Some(GaveUp::Retreat), Some("Orrin"), 0),
        "result.lost.fled"
    );
    assert_eq!(
        text(&c, false, Some(GaveUp::Surrender), Some("Orrin"), 0),
        "result.lost.surrendered"
    );
    assert_eq!(text(&c, false, None, None, 0), "result.lost-army");
    assert_eq!(
        text(&c, false, Some(GaveUp::Retreat), None, 0),
        "result.lost.fled-army"
    );
    assert_eq!(
        text(&c, false, Some(GaveUp::Surrender), None, 0),
        "result.lost.surrendered-army"
    );
}

fn record(origin: StackOrigin, creature: u16, start: u32, end: u32) -> StackRecord {
    StackRecord {
        origin,
        creature: Handle::from_raw(creature),
        start,
        end,
    }
}

#[test]
fn losses_sum_by_creature_in_the_order_they_first_lost() {
    let stacks = vec![
        record(StackOrigin::Army(0), 5, 10, 10),
        record(StackOrigin::Army(1), 3, 10, 4),
        record(StackOrigin::Army(2), 5, 8, 0),
        record(StackOrigin::WarMachine, 7, 1, 0),
        record(StackOrigin::Army(3), 3, 2, 1),
        record(StackOrigin::Created, 9, 4, 0),
    ];
    let original = |c: CreatureTypeId| (c.raw() != 9).then_some(u32::from(c.raw()) * 10);
    assert_eq!(losses(&stacks, original), vec![(30, 7), (50, 8), (70, 1)]);
}

#[test]
fn the_strongest_creature_is_the_first_of_the_highest_value_without_the_towers() {
    let stacks = vec![
        record(StackOrigin::Army(0), 1, 10, 0),
        record(StackOrigin::Army(1), 2, 10, 0),
        record(StackOrigin::Army(2), 3, 10, 0),
        record(StackOrigin::WarMachine, 149, 1, 1),
    ];
    let original = |c: CreatureTypeId| Some(u32::from(c.raw()));
    let value = |c: CreatureTypeId| match c.raw() {
        2 | 3 => 50,
        149 => 1000,
        _ => 10,
    };
    assert_eq!(
        strongest(&stacks, original, value),
        Some((Handle::from_raw(2), true))
    );
    assert_eq!(
        strongest(&stacks[..1], original, value),
        Some((Handle::from_raw(1), false))
    );
    assert_eq!(strongest(&[], original, value), None);
}

#[test]
fn experience_counts_the_loser_s_army_and_the_heroes() {
    let loser = vec![
        record(StackOrigin::Army(0), 1, 10, 0),
        record(StackOrigin::Raised(1), 2, 4, 1),
        record(StackOrigin::WarMachine, 3, 1, 0),
        record(StackOrigin::Created, 4, 5, 0),
    ];
    let hp = |c: CreatureTypeId| u32::from(c.raw()) * 10;
    // 10 × 10 + 3 × 20; the machine and the created stack give nothing.
    assert_eq!(experience(&loser, hp, (false, false, false)), 160);
    assert_eq!(experience(&loser, hp, (true, false, false)), 660);
    assert_eq!(experience(&loser, hp, (true, true, false)), 160);
    assert_eq!(experience(&loser, hp, (true, false, true)), 1160);
    assert_eq!(experience(&[], hp, (false, true, false)), 0);
}

#[test]
fn behind_the_window_the_flags_stay_as_shown() {
    use crate::hero::HeroPose;
    let pose = |group, flag| HeroPose {
        group,
        frame: 0,
        flag,
    };
    let mut now = Moment {
        heroes: [Some(pose(3, 5)), None],
        ..Moment::default()
    };
    let before = Moment {
        heroes: [Some(pose(0, 4)), Some(pose(0, 2))],
        ..Moment::default()
    };
    behind(&mut now, &before);
    assert_eq!(now.heroes, [Some(pose(3, 4)), None]);
}
