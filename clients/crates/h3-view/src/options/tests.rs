//! The combat options dialog's tests (E17 M6 T4a), on an invented pack (the original's text never
//! enters git, ADR-0009).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;
use h3_ui::ui::{Def, Dialog, Item};

use super::{
    AUTO, BACKGROUND, BOXES, DEFAULT, Did, EFFECTS, LEVELS, MUSIC, NO_SOUND, Open, OptionsLayout,
    SPEEDS, Settings, TEXTS, help_key, sound_volume,
};
use crate::layout::parse_battle;
use crate::message::OK;

/// The committed native layout.
const NATIVE: &str = include_str!("../../../../layouts/battle.toml");

const PACK: &str = r#"{
    "battle": {
        "options": {
            "title": "Options",
            "auto-ballista": "B",
            "help": { "ok": "back", "music-3": "m3", "speed-2": "fast", "spell-book": "pages" }
        }
    }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

fn layout() -> OptionsLayout {
    parse_battle(NATIVE).unwrap().options
}

/// The dialog open with `settings`, the audio device opened or not, in the client's profile.
fn open(settings: Settings, audio: bool) -> Open {
    Open::new(&layout(), &catalogue(), settings, (3, audio, Bugs::Fixed))
}

/// [`open`] in the original's profile (the comparisons').
fn original(settings: Settings, audio: bool) -> Open {
    Open::new(
        &layout(),
        &catalogue(),
        settings,
        (3, audio, Bugs::Original),
    )
}

/// Item `id`'s middle on the screen.
fn centre(open: &Open, id: u16) -> [i32; 2] {
    let item = open.window.dialog.item(id).unwrap();
    let ([x, y], [w, h]) = item.area().unwrap();
    let [px, py] = open.window.dialog.place;
    [px + x + w / 2, py + y + h / 2]
}

/// A press and a release at item `id`'s middle.
fn click(open: &mut Open, id: u16) -> (Did, Did) {
    let point = centre(open, id);
    let pressed = open.press(point);
    (pressed, open.release())
}

const CHANGED: Did = Did {
    changed: true,
    button_sound: false,
    box_sound: false,
    no_sound: false,
    closed: false,
};

/// [`CHANGED`] with a DEF button's own press sample.
const PRESSED: Did = Did {
    button_sound: true,
    ..CHANGED
};

#[test]
fn the_layout_holds_the_executables_items() {
    // Design §12.12: at (159, 84) with its shadow, the picture, Default, OK, the 20 bars, the
    // three speeds, the 11 boxes and the 17 texts, in the build's order.
    let layout = layout();
    let dialog = &layout.dialog;
    assert_eq!(layout.sound, "button.wav");
    assert_eq!((dialog.place, dialog.shadow), ([159, 84], Some([481, 431])));
    let ids: Vec<u16> = dialog.items.iter().map(Item::id).collect();
    let mut expected = vec![BACKGROUND, DEFAULT, OK];
    expected.extend(MUSIC..EFFECTS + 10);
    expected.extend(SPEEDS..SPEEDS + 3);
    expected.extend(BOXES..BOXES + 4);
    expected.extend(AUTO..AUTO + 5);
    expected.extend([LEVELS, LEVELS + 1]);
    expected.extend(TEXTS..TEXTS + 17);
    assert_eq!(ids, expected);
    // The bars 19 apart, the Auto Combat boxes 30 apart.
    let at = |id: u16| dialog.item(id).unwrap().area().unwrap().0;
    assert_eq!((at(MUSIC + 9), at(EFFECTS + 1)), ([200, 303], [48, 369]));
    assert_eq!((at(AUTO + 4), at(LEVELS + 1)), ([246, 204], [246, 313]));
    assert_eq!((at(TEXTS + 13), at(TEXTS + 16)), ([283, 204], [283, 313]));
}

#[test]
fn the_fill_shows_the_settings() {
    // A first run's: speed 0 chosen (disabled: its frame 2), the spell book's animation and the
    // Auto Combat boxes on, the rest off, each column's bar 5 alone shown with frame 5; the
    // background in the player's colours; the texts from the catalogue.
    let o = open(Settings::FIRST_RUN, true);
    let shown = &o.window.shown;
    assert_eq!(shown.players.get(&BACKGROUND), Some(&3));
    assert_eq!(shown.texts.get(&TEXTS).map(String::as_str), Some("Options"));
    assert_eq!(
        shown.texts.get(&(TEXTS + 12)).map(String::as_str),
        Some("B")
    );
    assert!(shown.state(SPEEDS).disabled);
    assert!(!shown.state(SPEEDS + 1).disabled);
    let frame = |id: u16| shown.frames.get(&id).copied();
    assert_eq!(frame(BOXES), Some(0));
    assert_eq!(frame(BOXES + 3), Some(1));
    assert_eq!(frame(AUTO + 4), Some(1));
    assert_eq!((frame(LEVELS), frame(LEVELS + 1)), (Some(0), Some(0)));
    for column in [MUSIC, EFFECTS] {
        for k in 0..10 {
            let hidden = shown.state(column + k).hidden;
            assert_eq!(hidden, k != 5, "{column} + {k}");
        }
        assert_eq!(frame(column + 5), Some(5));
    }
    // The draws: the shadow, then the shown items (18 bars hidden).
    let draws = o.window.dialog.draws(shown, "PLAYERS.PAL");
    assert_eq!(draws.len(), 4 + o.window.dialog.items.len() - 18);
}

#[test]
fn a_box_acts_on_its_release_over_it_alone() {
    // CLT-Q43 (2), fixed in the client's profile (E17 M7 T1a): a box or a bar released off it
    // does nothing, as a DEF button; back over it, its release clicks it.
    let mut o = open(Settings::FIRST_RUN, true);
    for id in [BOXES, MUSIC + 3] {
        let (point, before) = (centre(&o, id), o.settings);
        o.press(point);
        assert_eq!(o.moved([0, 0]), Did::default());
        assert_eq!(o.release(), Did::default());
        assert_eq!(o.settings, before);
        assert_eq!(o.clicked, id != BOXES);
        o.press(point);
        o.moved([0, 0]);
        assert_eq!(o.moved(point), Did::default());
        assert_eq!(o.release(), CHANGED);
    }
    assert!(o.settings.grid && o.clicked);
    assert_eq!(o.settings.music, 3);
    // Released over another bar of its column: not that bar's click either.
    let (from, to) = (centre(&o, MUSIC + 1), centre(&o, MUSIC + 6));
    o.press(from);
    o.moved(to);
    assert_eq!(o.release(), Did::default());
    assert_eq!(o.settings.music, 3);
    // A key's hold notes no mouse.
    o.key("Return");
    assert_eq!(o.moved([0, 0]), Did::default());
}

#[test]
fn a_box_acts_on_its_release_anywhere_and_sounds_as_pressed() {
    // Oddity 2, the original's profile: a box's press holds it, unchanged, with the procedure's
    // sample; its release anywhere toggles it and redraws; the dialog then counts as clicked.
    let mut o = original(Settings::FIRST_RUN, true);
    let pressed = o.press(centre(&o, BOXES));
    assert_eq!(
        pressed,
        Did {
            box_sound: true,
            ..Did::default()
        }
    );
    assert_eq!(o.moved([0, 0]), Did::default());
    assert!(!o.clicked);
    assert_eq!(o.release(), CHANGED);
    assert!(o.settings.grid && o.clicked);
    assert_eq!(o.window.shown.frames.get(&BOXES), Some(&1));
    for id in BOXES + 1..=BOXES + 3 {
        click(&mut o, id);
    }
    for id in AUTO..AUTO + 5 {
        click(&mut o, id);
    }
    let s = o.settings;
    assert!(s.movement_shade && s.cursor_shade && !s.spell_book);
    assert_eq!(s.auto, [false; 5]);
    // A second release, nothing held: nothing.
    assert_eq!(o.release(), Did::default());
}

#[test]
fn the_levels_pair_can_be_emptied() {
    // Oddity 5: a click on the level's box set clears the level; on the other sets its level.
    let mut o = open(Settings::FIRST_RUN, true);
    click(&mut o, LEVELS);
    assert_eq!(o.settings.army_info, 1);
    click(&mut o, LEVELS + 1);
    assert_eq!(o.settings.army_info, 2);
    assert_eq!(o.window.shown.frames.get(&LEVELS), Some(&0));
    assert_eq!(o.window.shown.frames.get(&(LEVELS + 1)), Some(&1));
    click(&mut o, LEVELS + 1);
    assert_eq!(o.settings.army_info, 0);
    click(&mut o, LEVELS);
    click(&mut o, LEVELS);
    assert_eq!(o.settings.army_info, 0);
}

#[test]
fn a_speed_is_chosen_by_its_button_sounding_once() {
    // CLT-Q74 (E17 M7 T1a; design §12.12's oddity 1 read again): a speed's press plays the
    // button's own sample alone, in both profiles (its loop sends the procedure no press); its
    // release over it chooses it (the three enabled, the new one disabled); the chosen one takes
    // no press.
    for mut o in [
        open(Settings::FIRST_RUN, true),
        original(Settings::FIRST_RUN, true),
    ] {
        let (pressed, released) = click(&mut o, SPEEDS + 2);
        assert_eq!(pressed, PRESSED);
        assert_eq!(released, CHANGED);
        assert_eq!(o.settings.speed, 2);
        let shown = &o.window.shown;
        assert!(shown.state(SPEEDS + 2).disabled && !shown.state(SPEEDS).disabled);
        assert!(!shown.state(SPEEDS + 2).pressed);
        assert_eq!(o.press(centre(&o, SPEEDS + 2)), Did::default());
        assert_eq!(o.release(), Did::default());
    }
}

#[test]
fn a_button_needs_its_release_over_it() {
    // A DEF button follows the mouse on and off; released off it, it goes up without its click.
    let mut o = open(Settings::FIRST_RUN, true);
    let point = centre(&o, SPEEDS + 1);
    o.press(point);
    assert!(o.window.shown.state(SPEEDS + 1).pressed);
    assert_eq!(o.moved(point), Did::default());
    assert_eq!(o.moved([0, 0]), CHANGED);
    assert!(!o.window.shown.state(SPEEDS + 1).pressed);
    // Back over it: pressed again with its own sample alone (0x4563A2; E17 M6 T7b, R4-3).
    assert_eq!(o.moved(point), PRESSED);
    assert_eq!(o.moved([0, 0]), CHANGED);
    assert_eq!(o.release(), CHANGED);
    assert_eq!(o.settings.speed, 0);
    assert!(!o.clicked);
    // A press over nothing that takes it (the background, a text, off the dialog): nothing.
    for point in [[170, 95], centre(&o, TEXTS), [0, 0]] {
        assert_eq!(o.press(point), Did::default());
        assert_eq!(o.release(), Did::default());
    }
}

#[test]
fn default_sets_the_combat_options_and_keeps_the_volumes() {
    let changed = Settings {
        speed: 2,
        music: 7,
        effects: 1,
        grid: true,
        movement_shade: true,
        cursor_shade: true,
        spell_book: false,
        auto: [false; 5],
        army_info: 2,
        ..Settings::FIRST_RUN
    };
    let mut o = open(changed, true);
    let (pressed, released) = click(&mut o, DEFAULT);
    assert!(pressed.button_sound && !pressed.box_sound);
    assert_eq!(released, CHANGED);
    assert_eq!(
        o.settings,
        Settings {
            music: 7,
            effects: 1,
            ..Settings::FIRST_RUN
        }
    );
    assert!(o.clicked);
    assert!(o.window.shown.state(SPEEDS).disabled);
}

#[test]
fn a_bar_sets_its_columns_volume_hidden_bars_too() {
    // Oddity 3: a hidden bar takes the click; oddity 8: the effects bars write the last effects
    // volume, the music bars leave the last music volume.
    let mut o = open(Settings::FIRST_RUN, true);
    assert!(o.window.shown.state(MUSIC + 2).hidden);
    let (pressed, released) = click(&mut o, MUSIC + 2);
    assert_eq!((pressed, released), (Did::default(), CHANGED));
    assert_eq!((o.settings.music, o.settings.last_music), (2, 5));
    assert!(!o.window.shown.state(MUSIC + 2).hidden);
    assert!(o.window.shown.state(MUSIC + 5).hidden);
    click(&mut o, EFFECTS + 9);
    assert_eq!((o.settings.effects, o.settings.last_effects), (9, 9));
    // The same volume again still counts as a click.
    let mut again = open(Settings::FIRST_RUN, true);
    click(&mut again, MUSIC + 5);
    assert!(again.clicked);
    // At volume 0 the audio device opened: the bars work.
    let silent = Settings {
        music: 0,
        effects: 0,
        ..Settings::FIRST_RUN
    };
    let mut o = open(silent, true);
    click(&mut o, MUSIC);
    click(&mut o, EFFECTS + 4);
    assert_eq!((o.settings.music, o.settings.effects), (0, 4));
}

#[test]
fn without_the_audio_device_no_bar_is_refused() {
    // CLT-Q44 (c), the client's profile (E17 M7 T1a): every bar works without the audio device,
    // a column at volume 0 too, its setting kept for a device later.
    let settings = Settings {
        music: 0,
        effects: 0,
        ..Settings::FIRST_RUN
    };
    let mut o = open(settings, false);
    let (_, released) = click(&mut o, MUSIC + 4);
    assert_eq!(released, CHANGED);
    click(&mut o, EFFECTS + 2);
    assert_eq!((o.settings.music, o.settings.effects), (4, 2));
    assert_eq!(o.settings.last_effects, 2);
    assert!(o.clicked);
}

#[test]
fn without_the_audio_device_a_silent_column_tells_so() {
    // Oddity 7, the original's profile (`options-no-sound`): a column at volume 0 without the
    // audio device opens the no-sound box (GENRLTXT 151) and changes nothing; a column above 0
    // works, silently.
    let settings = Settings {
        music: 0,
        ..Settings::FIRST_RUN
    };
    let mut o = original(settings, false);
    let (_, released) = click(&mut o, MUSIC + 4);
    assert_eq!(
        released,
        Did {
            no_sound: true,
            ..Did::default()
        }
    );
    assert_eq!(o.settings, settings);
    assert!(!o.clicked);
    click(&mut o, EFFECTS + 2);
    assert_eq!(o.settings.effects, 2);
    assert_eq!(NO_SOUND, "battle.options.no-sound");
}

#[test]
fn the_oks_click_or_its_key_closes() {
    let mut o = open(Settings::FIRST_RUN, true);
    let (pressed, released) = click(&mut o, OK);
    assert!(pressed.button_sound && !pressed.box_sound);
    assert!(released.closed && !o.clicked);
    // Its key: down presses it (nothing else meanwhile), the other key's release does nothing,
    // its own clicks it.
    let mut o = open(Settings::FIRST_RUN, true);
    // Its press sample as the key goes down (0x4565E7; E17 M6 T7b, R4-3).
    assert_eq!(o.key("Escape"), PRESSED);
    assert!(o.window.shown.state(OK).pressed);
    assert_eq!(o.key("Return"), Did::default());
    assert_eq!(o.press(centre(&o, BOXES)), Did::default());
    assert_eq!(o.release(), Did::default());
    assert_eq!(o.key_up("Return"), Did::default());
    let closed = o.key_up("Escape");
    assert!(closed.closed && closed.changed);
    assert!(!o.window.shown.state(OK).pressed);
}

#[test]
fn the_right_button_shows_each_controls_help() {
    // Design §12.12: Help 392 for the OK, else 392 + (id - 200), the chosen speed's and the
    // hidden bars' too; nothing over the picture, a text or off the dialog.
    let o = open(Settings::FIRST_RUN, true);
    let help = |id: u16| o.help_at(centre(&o, id));
    assert_eq!(help(OK).as_deref(), Some("battle.options.help.ok"));
    assert_eq!(
        help(DEFAULT).as_deref(),
        Some("battle.options.help.default")
    );
    assert_eq!(
        help(MUSIC + 3).as_deref(),
        Some("battle.options.help.music-3")
    );
    assert_eq!(
        help(EFFECTS).as_deref(),
        Some("battle.options.help.effects-0")
    );
    assert_eq!(
        help(AUTO + 3).as_deref(),
        Some("battle.options.help.auto-ballista")
    );
    assert_eq!(help(SPEEDS).as_deref(), Some("battle.options.help.speed-0"));
    assert_eq!(
        help(LEVELS + 1).as_deref(),
        Some("battle.options.help.info-spells")
    );
    assert_eq!(
        help(BOXES + 1).as_deref(),
        Some("battle.options.help.movement-shade")
    );
    assert_eq!(help(TEXTS), None);
    assert_eq!(o.help_at([170, 95]), None);
    assert_eq!(o.help_at([0, 0]), None);
    // The procedure's ids 222 to 224 name nothing (oddity 9); the picture none.
    assert_eq!(help_key(222), None);
    assert_eq!(help_key(BACKGROUND), None);
    let texts = catalogue();
    let text = |key: Option<String>| texts.text(&key.unwrap(), &h3_ui::text::Args::new());
    assert_eq!(text(help(BOXES + 3)), "pages");
    assert_eq!(text(help(SPEEDS + 2)), "fast");
}

#[test]
fn an_item_the_procedure_does_not_know_does_nothing() {
    // A crafted layout's DEF picture of an id no click handles (222): held, released, nothing.
    let mut layout = layout();
    layout.dialog = Dialog {
        place: [0, 0],
        shadow: None,
        items: vec![Item::Def(Def {
            id: 222,
            def: Some(String::from("syslb.def")),
            at: [0, 0],
            size: [10, 10],
            frame: 0,
            mirrored: false,
            group: 0,
            creature: false,
            frames_of: None,
            repeat: None,
        })],
    };
    let mut o = Open::new(
        &layout,
        &catalogue(),
        Settings::FIRST_RUN,
        (0, true, Bugs::Fixed),
    );
    assert_eq!(o.press([5, 5]), Did::default());
    assert_eq!(o.release(), Did::default());
    assert!(!o.clicked);
}

#[test]
fn a_volume_is_scaled_by_its_setting() {
    // `sound_volume` 0x5996C0: v (s + 1) / 10, at least 1, at most 127, for s of 1 to 10.
    assert_eq!(sound_volume(127, 0), 0);
    assert_eq!(sound_volume(127, 9), 127);
    assert_eq!(sound_volume(127, 5), 76);
    assert_eq!(sound_volume(64, 1), 12);
    assert_eq!(sound_volume(4, 1), 1);
    assert_eq!(sound_volume(0, 3), 1);
    assert_eq!(sound_volume(127, 10), 127);
    assert_eq!(sound_volume(127, 11), 0);
}

#[test]
fn the_key_f5_cycles_the_army_info_level_and_its_file_keeps_it() {
    // E17 M6 T4b2a (design §12.14; 0x475902): 0, 1, 2, 0; the rest untouched; the file written
    // at once reads the level back.
    let mut s = Settings::FIRST_RUN;
    let levels: Vec<u8> = (0..4).map(|_| s.cycle_army_info()).collect();
    assert_eq!(levels, [1, 2, 0, 1]);
    assert_eq!(
        s,
        Settings {
            army_info: 1,
            ..Settings::FIRST_RUN
        }
    );
    assert_eq!(Settings::read(Some(&s.write())), Ok((s, Vec::new())));
}

#[test]
fn the_settings_file_is_read_as_the_registry() {
    // No file, or no music volume: a first run's.
    let first = Ok((Settings::FIRST_RUN, Vec::new()));
    assert_eq!(Settings::read(None), first);
    assert_eq!(Settings::read(Some("")), first);
    let other = "[map]\nscroll = 3\n[battle]\ncombat_speed = 2\n";
    assert_eq!(Settings::read(Some(other)), first);
    // Written and read back.
    let set = Settings {
        speed: 1,
        music: 3,
        effects: 8,
        last_music: 3,
        last_effects: 8,
        grid: true,
        cursor_shade: true,
        auto: [true, false, true, false, true],
        army_info: 2,
        ..Settings::CAPTURE
    };
    assert_eq!(Settings::read(Some(&set.write())), Ok((set, Vec::new())));
    // Missing values 0; the flags their low bit, the ranges as `settings_check`'s; the last
    // volumes the volumes.
    let odd = "[battle]\nmusic_volume = 12\nsound_volume = -1\ncombat_speed = 3\n\
               combat_army_info_level = 2\nshow_combat_grid = 3\ncombat_ballista = 2\n\
               last_music_volume = 1\n";
    let checked = Settings {
        music: 5,
        effects: 5,
        last_music: 5,
        last_effects: 5,
        grid: true,
        army_info: 2,
        auto: [false; 5],
        ..Settings::CAPTURE
    };
    assert_eq!(Settings::read(Some(odd)), Ok((checked, Vec::new())));
    // A name it does not know or a value not an integer is missing, told in a line (E17 M6 T7b,
    // R4-5: another version's file, a hand edit): the rest is read.
    let later = "[battle]\nmusic_volume = 3\nwalk_speed = 1\nshow_combat_grid = true\n";
    let (read, mut notes) = Settings::read(Some(later)).unwrap();
    assert_eq!(
        read,
        Settings {
            music: 3,
            last_music: 3,
            auto: [false; 5],
            ..Settings::CAPTURE
        }
    );
    notes.sort_unstable();
    assert_eq!(
        notes,
        [
            "battle.show_combat_grid: not an integer, taken as missing",
            "battle: no setting `walk_speed`, left out"
        ]
    );
    let (read, notes) = Settings::read(Some("[battle]\nmusic_volume = \"5\"")).unwrap();
    assert_eq!((read, notes.len()), (Settings::FIRST_RUN, 1));
    // Not TOML (a file cut in a name), `battle` not a table: refused.
    for text in [
        "[battle",
        "battle = 1",
        "[battle]\nmusic_volume = 5\ncombat_army_info_le",
    ] {
        assert!(Settings::read(Some(text)).is_err(), "{text}");
    }
}

#[test]
fn the_help_never_opens_over_a_button_the_mouse_holds() {
    // E17 M6 T7b (R4-2): a DEF button the mouse holds runs its own loop (0x45641C), which drops
    // the right press: no help, the chosen (disabled) speed's and a box's held aside.
    let mut o = open(Settings::FIRST_RUN, true);
    let default = centre(&o, DEFAULT);
    o.press(default);
    assert_eq!(o.help_at(default), None);
    assert_eq!(o.help_at(centre(&o, OK)), None);
    o.moved([0, 0]);
    assert_eq!(o.help_at(centre(&o, OK)), None);
    assert!(o.release().changed && !o.clicked);
    assert!(o.help_at(default).is_some());
    o.press(centre(&o, SPEEDS));
    assert!(o.help_at(centre(&o, SPEEDS)).is_some());
    o.press(centre(&o, BOXES));
    assert!(o.help_at(centre(&o, BOXES)).is_some());
}

#[test]
fn what_the_help_s_popup_took_the_release_of_goes_without_its_click() {
    // E17 M6 T7b (R4-2, CLT-Q39's fix): the popup took the left release or the key's: what was
    // held goes, nothing clicked, the next press and release the dialog's again.
    let mut o = open(Settings::FIRST_RUN, true);
    o.press(centre(&o, BOXES));
    assert_eq!(o.let_go(Some("Return")), Did::default());
    assert_eq!(o.let_go(None), Did::default());
    assert_eq!(o.release(), Did::default());
    assert!(!o.settings.grid && !o.clicked);
    // A DEF button by the mouse: its own loop opens no help over it, so nothing lets it go
    // (E17 M7 T2): its release clicks.
    o.press(centre(&o, DEFAULT));
    assert_eq!(o.let_go(Some("Return")), Did::default());
    assert_eq!(o.let_go(None), Did::default());
    assert!(o.window.shown.state(DEFAULT).pressed);
    assert!(o.release().changed && o.clicked);
    o.clicked = false;
    // The OK by its key: its other key and the mouse let nothing go; its own key does.
    o.key("Return");
    assert_eq!(o.let_go(None), Did::default());
    assert_eq!(o.let_go(Some("Escape")), Did::default());
    assert_eq!(o.let_go(Some("Return")), CHANGED);
    assert!(!o.window.shown.state(OK).pressed);
    assert_eq!(o.key_up("Return"), Did::default());
    let (pressed, released) = click(&mut o, BOXES);
    assert!(pressed.box_sound && released.changed && o.settings.grid && o.clicked);
    assert_eq!(o.let_go(None), Did::default());
}

#[test]
fn the_right_release_ends_a_def_button_s_hold_with_its_click() {
    // E17 M7 T2 (CLT-Q72 (a); `def_button_message` 0x456338): the right button's release ends
    // the loop of the DEF button the mouse holds as the left's does, clicking it over it; the
    // left release after it finds nothing held.
    let mut o = open(Settings::FIRST_RUN, true);
    let speed = centre(&o, SPEEDS + 2);
    o.press(speed);
    let did = o.right_release();
    assert!(did.changed && o.clicked && o.settings.speed == 2);
    assert_eq!(o.release(), Did::default());
    // Off the button: it goes up, nothing clicked.
    let mut o = open(Settings::FIRST_RUN, true);
    o.press(centre(&o, DEFAULT));
    o.moved([0, 0]);
    let did = o.right_release();
    assert!(did.changed && !did.closed && !o.clicked);
    assert!(!o.window.shown.state(DEFAULT).pressed);
    // The OK over it closes the dialog.
    let mut o = open(Settings::FIRST_RUN, true);
    o.press(centre(&o, OK));
    assert!(o.right_release().closed);
    // A box the mouse holds (a DEF picture, no loop of its own) and the OK its key holds stay
    // held; nothing held, nothing.
    let mut o = open(Settings::FIRST_RUN, true);
    o.press(centre(&o, BOXES));
    assert_eq!(o.right_release(), Did::default());
    assert!(o.release().changed && o.settings.grid);
    o.key("Return");
    assert_eq!(o.right_release(), Did::default());
    assert!(o.key_up("Return").closed);
    let mut o = open(Settings::FIRST_RUN, true);
    assert_eq!(o.right_release(), Did::default());
}
