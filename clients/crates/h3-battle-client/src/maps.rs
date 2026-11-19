//! The scenario screen's maps (E27 M3 T1a; `docs/epics/E27-design.md` §3.3, decision 44): every
//! map file of the installation copy's `Maps` read whole (`h3_map::read`) into the list's entries
//! (`h3_view::shell::scenario::Entry`), in the list's order. A file that fails to read is left
//! out, as the original leaves out one whose header does not read (`scenario_header_read`
//! 0x583170). Read once, at the shell's start (*provisional*: the original reads them at each
//! opening; M3's review list). Since E27 M3 T3b1 each map is imported with the game's rules
//! ([`rules`]) for the heroes the engine offers its colours (`GameSetup::offer`, decision 4).

use std::fs;
use std::path::Path;

use h3_content::FactionId;
use h3_contracts::PlayerId;
use h3_game::{FactionChoice, GameRuleset};
use h3_map::model::{Loss, Player, Version, Victory, VictoryGoal};
use h3_map::{Limits, RawMap};
use h3_ui::text::decode_1252;
use h3_view::shell::scenario::{self, CustomHero, Entry, Format, Heroes, MainHero, Seat};

/// The packs of the game's rules the shell imports the maps with, and their profile (as `h3
/// adventure`'s defaults: Complete's content, the faithful profile).
const PACKS: [&str; 3] = ["base", "ab", "sod"];
const PROFILE: &str = "base:faithful";

/// The game's rules of the repository's packs (`packs/`, beside the client workspace) that the
/// maps are imported with (E27 M3 T3b1).
///
/// # Errors
///
/// The packs do not load.
pub fn rules() -> Result<GameRuleset, String> {
    let packs = crate::assets::clients_dir().join("../packs");
    let ids = PACKS.map(String::from);
    h3_files::adventure::load_rules(&packs, &ids, PROFILE)
        .map_err(|e| format!("{}: {e}", packs.display()))
}

/// The maps' folder of an installation (`maps`, as the original names it: the copy's `Maps`).
pub(crate) const MAPS: &str = "Maps";

/// The entries of `install`'s maps (`*.h3m`, any case) under `rules`, in the list's order; none
/// without the folder.
#[must_use]
pub fn entries(install: &Path, rules: &GameRuleset) -> Vec<Entry> {
    let Ok(dir) = fs::read_dir(install.join(MAPS)) else {
        return Vec::new();
    };
    let mut entries: Vec<Entry> = dir
        .filter_map(Result::ok)
        .filter_map(|e| read(&e.path(), rules))
        .collect();
    scenario::sort(&mut entries);
    entries
}

/// The entry of the map file `path` (`*.h3m`, any case) under `rules`, by its file's name; none
/// for another file or one that does not read.
fn read(path: &Path, rules: &GameRuleset) -> Option<Entry> {
    let file = path.file_name()?.to_str()?.to_owned();
    let h3m = Path::new(&file)
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("h3m"));
    let bytes = h3m.then(|| fs::read(path).ok()).flatten()?;
    let map = h3_map::read(&bytes, &Limits::default()).ok()?;
    Some(entry(file, &map, rules))
}

/// `entries` with the files `names` names alone (any case), in their order; all of them without
/// names (E27 M3 T1b1: a replay's `--maps`, a recipe's `maps` as the harness's run copies them).
/// A name with a `/` is a map file's path (E27 M3 T2c: a map of the harness's own, written for
/// the run): read and listed by its file's name, the list in its order again.
#[must_use]
pub fn named(entries: Vec<Entry>, names: Option<&[String]>, rules: &GameRuleset) -> Vec<Entry> {
    let Some(names) = names else {
        return entries;
    };
    let mut named: Vec<Entry> = entries
        .into_iter()
        .filter(|e| names.iter().any(|n| n.eq_ignore_ascii_case(&e.file)))
        .collect();
    let own = names.iter().filter(|n| n.contains('/'));
    named.extend(own.filter_map(|path| read(Path::new(path), rules)));
    scenario::sort(&mut named);
    named
}

/// The list's entry of `map`, read from the file `file`, its heroes under `rules`.
#[must_use]
pub fn entry(file: String, map: &RawMap, rules: &GameRuleset) -> Entry {
    let header = &map.header;
    let (victory, standard) = match map.victory {
        Victory::Standard => (None, false),
        Victory::Special {
            allow_standard,
            goal,
            ..
        } => {
            let code = goal_code(goal);
            (
                Some(code),
                allow_standard || STANDARD_ALWAYS.contains(&code),
            )
        }
    };
    let loss = match map.loss {
        Loss::Standard => None,
        Loss::LoseTown { .. } => Some(0),
        Loss::LoseHero { .. } => Some(1),
        Loss::TimeExpires { .. } => Some(2),
    };
    let teams = map.teams.map_or([0, 1, 2, 3, 4, 5, 6, 7], |t| t.of_player);
    Entry {
        file,
        format: Some(match header.version {
            Version::RoE => Format::RoE,
            Version::Ab => Format::Ab,
            Version::Sod => Format::SoD,
        }),
        // Whole: the screen cuts them to the list's record's 60 and 300 bytes in the original's
        // profile alone (E27 M3 T7, CLT-Q151 (b)).
        name: decode_1252(header.name.as_bytes()),
        description: decode_1252(header.description.as_bytes()),
        size: header.size,
        difficulty: header.difficulty,
        victory,
        standard,
        loss,
        seats: map.players.each_ref().map(seat),
        teams,
        heroes: heroes(map, rules),
    }
}

/// What `map`'s import under `rules` gives the advanced options of its heroes (E27 M3 T3b1;
/// design §24): each colour's offer by faction (its choice made each faction in turn,
/// `GameSetup::offer`; none for a map that does not import), the main heroes' names, the map's
/// customised heroes (its disposed heroes: a portrait of 0xFF and an empty name the type's).
fn heroes(map: &RawMap, rules: &GameRuleset) -> Heroes {
    let mut heroes = Heroes::default();
    for (c, player) in map.players.iter().enumerate() {
        if let Some(h) = player.details.as_ref().and_then(|d| d.main_hero.as_ref()) {
            heroes.main_names[c] = decode_1252(h.name.as_bytes());
        }
    }
    heroes.custom = map
        .disposed_heroes
        .iter()
        .map(|h| CustomHero {
            hero_type: h.hero_type,
            portrait: (h.portrait != 0xFF).then_some(h.portrait),
            name: decode_1252(h.name.as_bytes()),
        })
        .collect();
    let Ok(mut setup) = h3_map::import(map, rules) else {
        return heroes;
    };
    for p in 0..setup.players.len() {
        let colour = usize::from(setup.players[p].colour & 7);
        let player = PlayerId::new(u8::try_from(p).unwrap_or(u8::MAX));
        for (f, offer) in heroes.offers[colour].iter_mut().enumerate() {
            let Some(faction) = FactionId::from_index(f) else {
                continue;
            };
            setup.players[p].start.choice = Some(FactionChoice::Fixed(faction));
            *offer = setup
                .offer(rules, player)
                .iter()
                .filter_map(|h| u8::try_from(h.index()).ok())
                .collect();
        }
    }
    heroes
}

/// A colour's seat (E27 M3 T3a: what the advanced options offer it): who may play it, its
/// factions and flags, its main hero.
fn seat(player: &Player) -> Seat {
    let details = player.details.as_ref();
    Seat {
        human: player.human,
        computer: player.computer,
        factions: details.map_or(0, |d| d.allowed_towns),
        random_town: details.is_some_and(|d| d.random_town),
        generates: details
            .and_then(|d| d.main_town)
            .is_some_and(|t| t.generate_hero),
        random_hero: details.is_some_and(|d| d.random_hero),
        main_hero: details
            .and_then(|d| d.main_hero.as_ref())
            .map(|h| MainHero {
                hero_type: h.hero_type,
                // The file's 0xFF (its type's own) kept apart (CLT-Q141).
                portrait: (h.portrait != 0xFF).then_some(h.portrait),
            }),
    }
}

/// The special goals whose standard victory the header's read always allows (E27 M3 T2a, design
/// §21.1: `victory_standard_set` 0x4BF780, from the conditions' read 0x4C3555 with no colour
/// defeated yet, outside a campaign): an artifact (0), the Grail (4) and an artifact's transport
/// (10) unless a single colour is left (never at the read), a hero's defeat (5) always (VCMI
/// 1.7.5 agrees for 5, `MapFormatH3M.cpp`: "H3 behavior"). The file's flag counts for the others
/// (recorded: `scn-info` row-08 and row-15, two maps of goal 5 whose flag is clear).
const STANDARD_ALWAYS: [u8; 4] = [0, 4, 5, 10];

/// A special goal's code in the format (0 to 10).
const fn goal_code(goal: VictoryGoal) -> u8 {
    match goal {
        VictoryGoal::AcquireArtifact { .. } => 0,
        VictoryGoal::AccumulateCreatures { .. } => 1,
        VictoryGoal::AccumulateResources { .. } => 2,
        VictoryGoal::UpgradeTown { .. } => 3,
        VictoryGoal::BuildGrail { .. } => 4,
        VictoryGoal::DefeatHero { .. } => 5,
        VictoryGoal::CaptureTown { .. } => 6,
        VictoryGoal::DefeatMonster { .. } => 7,
        VictoryGoal::FlagDwellings => 8,
        VictoryGoal::FlagMines => 9,
        VictoryGoal::TransportArtifact { .. } => 10,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The staged installation copy, if the original data are staged (`H3_INSTALL` empty: the
    /// staged-data tests skip, as on CI).
    fn staged() -> Option<PathBuf> {
        let dir = std::env::var_os("H3_INSTALL").map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../original-data/install"),
            PathBuf::from,
        );
        dir.join(MAPS).is_dir().then_some(dir)
    }

    /// Every shipped map reads (decision 44: all import and validate since E08 T97b1), in the
    /// list's order: the formats in turn, the names case-insensitively. The copy's `Maps` holds
    /// 160 files: 159 maps (`*.h3m`, the new game's pattern) and the tutorial's `Tutorial.tut`.
    #[test]
    fn the_shipped_maps_read_into_the_list() {
        let Some(install) = staged() else {
            return;
        };
        let rules = rules().unwrap();
        let entries = entries(&install, &rules);
        assert_eq!(entries.len(), 159);
        assert!(entries.windows(2).all(|w| w[0].format <= w[1].format));
        let arrogance = entries.iter().find(|e| e.file == "Arrogance.h3m").unwrap();
        assert_eq!(arrogance.name, "Arrogance");
        assert_eq!(arrogance.size, 36);
        assert_eq!(arrogance.difficulty, 1);
        assert_eq!((arrogance.victory, arrogance.loss), (None, None));
        assert_eq!(arrogance.local(), Some(0));
        // E27 M3 T1b1: the list's players column, "4/4" in `scn-list-keys` list.
        assert_eq!((arrogance.players(), arrogance.humans()), (4, 4));
        // E27 M3 T2a: a hero's defeat allows the standard victory whatever the file's flag
        // (`scn-info` row-08), a town's capture takes the file's.
        let goal = |file: &str| {
            let e = entries.iter().find(|e| e.file == file).unwrap();
            (e.victory, e.standard)
        };
        assert_eq!(goal("Good Witch, Bad Witch.h3m"), (Some(5), true));
        assert_eq!(goal("Thousand Islands.h3m"), (Some(5), true));
        assert_eq!(goal("For Sale.h3m"), (Some(6), false));
        // A recipe's maps alone (`--maps`), any case; all without names.
        let names = [String::from("arrogance.H3M"), String::from("no such.h3m")];
        let named = named(entries.clone(), Some(&names), &rules);
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].file, "Arrogance.h3m");
        assert_eq!(super::named(entries.clone(), None, &rules).len(), 159);
        assert!(super::named(Vec::new(), Some(&[]), &rules).is_empty());
        // E27 M3 T2c: a name with a `/` is a map file of its own (the harness's), read and listed
        // in the list's order; Jihad's description whole, the record's 300 bytes exactly (E27 M3
        // T7, CLT-Q151 (b): a longer one kept whole, the screen cutting it in the original's
        // profile).
        let path = install.join(MAPS).join("Jihad.h3m");
        let names = [
            String::from("Arrogance.h3m"),
            path.to_string_lossy().into_owned(),
        ];
        let own = super::named(Vec::from([entries[0].clone()]), Some(&names), &rules);
        let files: Vec<&str> = own.iter().map(|e| e.file.as_str()).collect();
        assert_eq!(files, ["Jihad.h3m"]);
        let own = super::named(entries, Some(&names), &rules);
        let files: Vec<&str> = own.iter().map(|e| e.file.as_str()).collect();
        assert_eq!(files, ["Jihad.h3m", "Arrogance.h3m"]);
        assert_eq!(
            own[0].description.chars().count(),
            scenario::DESCRIPTION_BYTES
        );
    }

    #[test]
    fn no_maps_folder_lists_nothing() {
        let rules = rules().unwrap();
        assert!(entries(Path::new("/nonexistent-h3-install"), &rules).is_empty());
    }

    /// E27 M3 T3b1: the engine's offers, the main heroes and the map's customised heroes as the
    /// advanced options read them (`adv-single-hero`, `adv-single-maps`).
    #[test]
    fn the_shipped_maps_offer_their_heroes() {
        let Some(install) = staged() else {
            return;
        };
        let rules = rules().unwrap();
        let entries = entries(&install, &rules);
        let map = |file: &str| entries.iter().find(|e| e.file == file).unwrap();
        // Arrogance's red as Castle (`hero-1` to `hero-last`): Orrin first, 16 at most; none of
        // Rampart's; a colour that does not play none.
        let arrogance = map("Arrogance.h3m");
        let castle = &arrogance.heroes.offers[0][0];
        assert_eq!(castle.first(), Some(&0));
        assert_eq!(castle.len(), 16);
        let rampart = &arrogance.heroes.offers[0][1];
        assert_eq!(rampart.first(), Some(&16));
        assert!(rampart.iter().all(|h| !castle.contains(h)));
        assert!(arrogance.heroes.offers[2].iter().all(Vec::is_empty));
        // Adventures of Jared Haret (`map-2`): red's main hero Orrin's portrait named "Jared
        // Haret"; blue's of its type's own portrait (CLT-Q141); tan's named "Thenith".
        let jared = map("Adventures of Jared Haret.h3m");
        let main = |c: usize| jared.seats[c].main_hero.unwrap();
        assert_eq!((main(0).hero_type, main(0).portrait), (2, Some(0)));
        assert_eq!((main(1).hero_type, main(1).portrait), (85, None));
        assert_eq!(jared.heroes.main_names[0], "Jared Haret");
        assert_eq!(jared.heroes.main_names[1], "");
        assert_eq!(jared.heroes.main_names[2], "Thenith");
        // The disposed heroes the shipped maps list keep their types' portraits and names.
        let custom: Vec<_> = entries.iter().flat_map(|e| &e.heroes.custom).collect();
        assert!(!custom.is_empty());
        assert!(
            custom
                .iter()
                .all(|h| h.portrait.is_none() && h.name.is_empty())
        );
    }

    #[test]
    fn the_goals_take_their_codes() {
        use h3_map::model::Pos;
        let pos = Pos::default();
        let goals = [
            VictoryGoal::AcquireArtifact { artifact: 1 },
            VictoryGoal::AccumulateCreatures {
                creature: 1,
                count: 1,
            },
            VictoryGoal::AccumulateResources {
                resource: 0,
                amount: 1,
            },
            VictoryGoal::UpgradeTown {
                pos,
                hall: 0,
                castle: 0,
            },
            VictoryGoal::BuildGrail { pos },
            VictoryGoal::DefeatHero { pos },
            VictoryGoal::CaptureTown { pos },
            VictoryGoal::DefeatMonster { pos },
            VictoryGoal::FlagDwellings,
            VictoryGoal::FlagMines,
            VictoryGoal::TransportArtifact { artifact: 1, pos },
        ];
        let codes: Vec<u8> = goals.into_iter().map(goal_code).collect();
        assert_eq!(codes, (0..11).collect::<Vec<u8>>());
    }
}
