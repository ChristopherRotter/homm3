use super::*;
use alloc::string::ToString;
use alloc::vec;

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
        ui::Draw::Button { def, frame: 15, mirrored: true, x: 557, y: 57, .. } if def == "TwCrPort.def"
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
        Draw::Dialog(ui::Draw::Darken {
            rect: [0, 0, 800, 600]
        })
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
