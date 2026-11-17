//! The scenario screen's maps (E27 M3 T1a; `docs/epics/E27-design.md` §3.3, decision 44): every
//! map file of the installation copy's `Maps` read whole (`h3_map::read`) into the list's entries
//! (`h3_view::shell::scenario::Entry`), in the list's order. A file that fails to read is left
//! out, as the original leaves out one whose header does not read (`scenario_header_read`
//! 0x583170). Read once, at the shell's start (*provisional*: the original reads them at each
//! opening; M3's review list).

use std::fs;
use std::path::Path;

use h3_map::model::{Loss, Version, Victory, VictoryGoal};
use h3_map::{Limits, RawMap};
use h3_ui::text::decode_1252;
use h3_view::shell::scenario::{self, Entry, Format, Seat};

/// The maps' folder of an installation (`maps`, as the original names it: the copy's `Maps`).
const MAPS: &str = "Maps";

/// The entries of `install`'s maps (`*.h3m`, any case), in the list's order; none without the
/// folder.
#[must_use]
pub fn entries(install: &Path) -> Vec<Entry> {
    let Ok(dir) = fs::read_dir(install.join(MAPS)) else {
        return Vec::new();
    };
    let limits = Limits::default();
    let mut entries: Vec<Entry> = dir
        .filter_map(Result::ok)
        .filter_map(|e| {
            let file = e.file_name().into_string().ok()?;
            let h3m = Path::new(&file)
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("h3m"));
            let bytes = h3m.then(|| fs::read(e.path()).ok()).flatten()?;
            let map = h3_map::read(&bytes, &limits).ok()?;
            Some(entry(file, &map))
        })
        .collect();
    scenario::sort(&mut entries);
    entries
}

/// `entries` with the files `names` names alone (any case), in their order; all of them without
/// names (E27 M3 T1b1: a replay's `--maps`, a recipe's `maps` as the harness's run copies them).
#[must_use]
pub fn named(entries: Vec<Entry>, names: Option<&[String]>) -> Vec<Entry> {
    match names {
        Some(names) => entries
            .into_iter()
            .filter(|e| names.iter().any(|n| n.eq_ignore_ascii_case(&e.file)))
            .collect(),
        None => entries,
    }
}

/// The list's entry of `map`, read from the file `file`.
#[must_use]
pub fn entry(file: String, map: &RawMap) -> Entry {
    let header = &map.header;
    let (victory, standard) = match map.victory {
        Victory::Standard => (None, false),
        Victory::Special {
            allow_standard,
            goal,
            ..
        } => {
            let code = goal_code(goal);
            (Some(code), allow_standard || STANDARD_ALWAYS.contains(&code))
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
        name: decode_1252(header.name.as_bytes()),
        description: decode_1252(header.description.as_bytes()),
        size: header.size,
        difficulty: header.difficulty,
        victory,
        standard,
        loss,
        seats: map.players.each_ref().map(|p| Seat {
            human: p.human,
            computer: p.computer,
        }),
        teams,
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
        let entries = entries(&install);
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
        let named = named(entries.clone(), Some(&names));
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].file, "Arrogance.h3m");
        assert_eq!(super::named(entries, None).len(), 159);
        assert!(super::named(Vec::new(), Some(&[])).is_empty());
    }

    #[test]
    fn no_maps_folder_lists_nothing() {
        assert!(entries(Path::new("/nonexistent-h3-install")).is_empty());
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
