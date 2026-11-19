//! The game's begin (E27 M3 T6; `docs/epics/E27-design.md` §3.9, §3.17, §32; decision 4): the
//! scenario screen's choices ([`Begun`]) made into the engine's `GameSetup` through the engine's
//! own functions and validated by it: the map read and imported again (`h3_map::import`), each
//! colour's human flag, faction choice and starting hero's pick set from its record, the
//! difficulty. What the setup has no field for (the players' names, the bonus and the handicap,
//! E10's; the turn's duration, the hot seat's clock) is kept beside it ([`Game`]) for E24 M5,
//! which builds and hosts the game (the map client lane's inbox).
//!
//! - **Who is human** (§3.9; ADV-Q27's default the import's): a colour a human record holds and
//!   the import lets a human play; the others the computer's (E13's turns: decision 6). A human
//!   record on a colour the import makes the computer's alone is refused (E27 M3 T7: the screen
//!   seats a human only where the map lets one play, so only a stale record reaches it).
//! - **The town** (TWN-SET-001): a record's town `FactionChoice::Fixed`; its Random the import's
//!   choice (`Random`, or `Fixed` where the map allows one faction: the original's offer, whose
//!   Random is that one's), drawn by the engine when the game is built (CLT-Q95, decision 13).
//! - **The hero** (TWN-SET-006): a human record's pick, by its hero type; Random none; a
//!   computer's always none.
//!
//! The steps of the loading screen's bar are the client's own (§32): the map read, the setup
//! made, the setup validated ([`STEPS`]), then the bar filled (`h3_view::shell::play::Play`).

use std::fs;
use std::path::{Path, PathBuf};

use h3_content::{FactionId, HeroTypeId};
use h3_game::{FactionChoice, GameRuleset, GameSetup};
use h3_map::{Limits, RawMap};
use h3_view::shell::scenario::{Begun, Bonus};

/// The bar's steps the client takes as its work goes: the map read, the setup made, the setup
/// validated.
pub const STEPS: u8 = 3;

/// A game begun: the engine's validated setup and what it has no field for, kept for E24 M5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// The map's file.
    pub file: String,
    /// The setup the engine validated.
    pub setup: GameSetup,
    /// Each colour's human's name, red first (the hot seat's names; the single player's edit).
    pub names: [Option<String>; 8],
    /// Each colour's starting bonus, red first (E10's: `CLT-STUB-9`).
    pub bonuses: [Bonus; 8],
    /// Each colour's handicap, red first (E10's: `CLT-STUB-10`).
    pub handicaps: [u8; 8],
    /// The turn's duration (`turndur.txt`'s record; the hot seat's clock: `CLT-STUB-11`).
    pub duration: u8,
}

/// The path of the map file `file`: the installation copy's `Maps`' (E27 M3 T7: a replay builds
/// no setup, so no map of the harness's own reaches the begin).
#[must_use]
pub fn map_path(install: &Path, file: &str) -> PathBuf {
    install.join(crate::maps::MAPS).join(file)
}

/// The map at `path`, read whole.
///
/// # Errors
/// The file does not read, or is not a map.
pub fn read(path: &Path) -> Result<RawMap, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    h3_map::read(&bytes, &Limits::default()).map_err(|e| format!("{}: {e}", path.display()))
}

/// The setup of `map` under `rules` with the choices `begun` (not validated: [`validated`]).
///
/// # Errors
/// The map does not import, or a human record holds a colour the import gives the computer
/// alone (E27 M3 T7: the import's `human` the only may-be-human mark, which the engine cannot
/// check again).
pub fn setup(map: &RawMap, begun: &Begun, rules: &GameRuleset) -> Result<GameSetup, String> {
    let mut setup = h3_map::import(map, rules).map_err(|e| format!("the import: {e}"))?;
    setup.difficulty = begun.difficulty;
    for player in &mut setup.players {
        let record = begun.records[usize::from(player.colour & 7)];
        if record.colour.is_some() && !player.human {
            return Err(format!(
                "a human holds colour {}, which the map gives the computer alone",
                player.colour
            ));
        }
        // The import's human flag cleared where no human record holds the colour.
        player.human &= record.colour.is_some();
        // rule: TWN-SET-001
        if let Some(faction) = record
            .town
            .and_then(|t| FactionId::from_index(usize::from(t)))
        {
            player.start.choice = Some(FactionChoice::Fixed(faction));
        }
        // rule: TWN-SET-006
        player.start.pick = record
            .hero
            .filter(|_| player.human)
            .and_then(|h| HeroTypeId::from_index(usize::from(h)));
    }
    Ok(setup)
}

/// `setup` validated by the engine under `rules` (`GameSetup::validate`).
///
/// # Errors
/// The engine's refusal.
pub fn validated(setup: &GameSetup, rules: &GameRuleset) -> Result<(), String> {
    setup.validate(rules).map_err(|e| e.to_string())
}

/// The game begun of `setup` (validated) and `begun`'s choices it has no field for.
#[must_use]
pub fn game(setup: GameSetup, begun: &Begun) -> Game {
    Game {
        file: begun.file.clone(),
        setup,
        names: begun.names.clone(),
        bonuses: begun.records.map(|r| r.bonus),
        handicaps: begun.records.map(|r| r.handicap),
        duration: begun.duration,
    }
}

#[cfg(test)]
mod tests {
    use h3_view::shell::scenario::Record;

    use super::*;

    /// The staged installation copy, if the original data are staged (`H3_INSTALL` empty: the
    /// staged-data tests skip, as on CI).
    fn staged() -> Option<PathBuf> {
        let dir = std::env::var_os("H3_INSTALL").map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../original-data/install"),
            PathBuf::from,
        );
        dir.join(crate::maps::MAPS).is_dir().then_some(dir)
    }

    const NONE: Record = Record {
        colour: None,
        town: None,
        hero: None,
        bonus: Bonus::Random,
        handicap: 0,
    };

    #[test]
    fn a_map_is_found_in_the_installation_s_maps() {
        assert_eq!(
            map_path(Path::new("/install"), "Jihad.h3m"),
            PathBuf::from("/install/Maps/Jihad.h3m")
        );
        assert!(read(Path::new("/nonexistent-h3-map.h3m")).is_err());
    }

    /// A committed map of the repository's (`fixtures/maps`), read whole.
    fn fixture(file: &str) -> RawMap {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/maps")
            .join(file);
        read(&path).unwrap()
    }

    /// E27 M3 T7 (the review's C8): a colour no human record holds is the computer's, the
    /// import's human flag cleared; a human record on a colour the import gives the computer
    /// alone is refused, not made human.
    #[test]
    fn a_human_record_takes_only_a_colour_a_human_may_play() {
        let rules = crate::maps::rules().unwrap();
        let mut map = fixture("e08-duel-36.h3m");
        let imported = h3_map::import(&map, &rules).unwrap();
        let first = imported.players.iter().find(|p| p.human).unwrap().colour;
        let mut records = [NONE; 8];
        records[usize::from(first)].colour = Some(first);
        let begun = Begun {
            file: String::from("e08-duel-36.h3m"),
            difficulty: 2,
            records,
            names: Default::default(),
            duration: 10,
        };
        let made = setup(&map, &begun, &rules).unwrap();
        assert_eq!(made.difficulty, 2);
        for p in &made.players {
            assert_eq!(p.human, p.colour == first, "colour {}", p.colour);
        }
        // The same colour made the computer's alone in the map: the record refused.
        let player = &mut map.players[usize::from(first)];
        player.human = false;
        player.computer = true;
        let refused = setup(&map, &begun, &rules).unwrap_err();
        assert!(refused.contains("computer alone"), "{refused}");
    }

    /// `adv-begin`'s choices on Arrogance (red Castle, its first hero, gold; Normal) and
    /// `adv-hotseat-begin`'s on A Warm and Familiar Place (two humans): each setup validates, the
    /// humans the records' colours, the rest the computer's.
    #[test]
    fn the_recipes_begins_validate() {
        let Some(install) = staged() else {
            return;
        };
        let rules = crate::maps::rules().unwrap();
        let map = read(&map_path(&install, "Arrogance.h3m")).unwrap();
        let mut records = [NONE; 8];
        records[0] = Record {
            colour: Some(0),
            town: Some(0),
            hero: Some(0),
            bonus: Bonus::Gold,
            handicap: 0,
        };
        let begun = Begun {
            file: String::from("Arrogance.h3m"),
            difficulty: 1,
            records,
            names: Default::default(),
            duration: 10,
        };
        let setup = setup(&map, &begun, &rules).unwrap();
        validated(&setup, &rules).unwrap();
        assert_eq!(setup.difficulty, 1);
        let red = &setup.players[0];
        assert!(red.human);
        assert_eq!(
            red.start.choice,
            FactionId::from_index(0).map(FactionChoice::Fixed)
        );
        assert_eq!(red.start.pick, HeroTypeId::from_index(0));
        assert!(
            setup.players[1..]
                .iter()
                .all(|p| !p.human && p.start.pick.is_none())
        );
        let game = game(setup, &begun);
        assert_eq!(game.bonuses[0], Bonus::Gold);
        assert_eq!(game.file, "Arrogance.h3m");
        // A pick of a computer's is none; a hero type no offer has is the engine's refusal.
        let mut computer = begun.clone();
        computer.records[0].colour = None;
        let made = super::setup(&map, &computer, &rules).unwrap();
        assert_eq!(made.players[0].start.pick, None);
        let mut wrong = begun.clone();
        wrong.records[0].hero = Some(16);
        let made = super::setup(&map, &wrong, &rules).unwrap();
        assert!(validated(&made, &rules).is_err());
        // The hot seat: red and green A's and B's (`adv-hotseat-begin`'s flags), Random towns.
        let map = read(&map_path(&install, "A Warm and Familiar Place.h3m")).unwrap();
        let mut records = [NONE; 8];
        records[0].colour = Some(0);
        records[3].colour = Some(3);
        let begun = Begun {
            file: String::from("A Warm and Familiar Place.h3m"),
            difficulty: 1,
            records,
            names: [
                Some("A".into()),
                None,
                None,
                Some("B".into()),
                None,
                None,
                None,
                None,
            ],
            duration: 10,
        };
        let setup = super::setup(&map, &begun, &rules).unwrap();
        validated(&setup, &rules).unwrap();
        let humans: Vec<u8> = setup
            .players
            .iter()
            .filter(|p| p.human)
            .map(|p| p.colour)
            .collect();
        assert_eq!(humans, [0, 3]);
    }
}
