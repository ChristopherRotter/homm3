//! The ladders (E16 M4 T1; ADR-0019 points 4–7; `E16-design.md` §11): each `[[ladder]]`, its
//! scenario pool compiled at start on one ruleset, and its results log, `<data>/ladders/<id>.log`:
//! one JSON document per line, each a [`Line`]: a season's start, a rated game, a no-show. The
//! tables are read back at start: a last line a crash cut short is dropped and cut from the file;
//! any other line that does not read, or that contradicts the lines before it or the player log,
//! stops the start (an error that names the line, never its text). A ladder whose ruleset is not
//! the last season's starts the next season: every account keeps its rating and its volatility,
//! its deviation reset (the user, SRV-Q19). A rating is written as `f64`, which `serde_json` reads
//! back exactly: no rating is computed twice (`rating`). The log holds login names and ratings,
//! nothing secret; on Unix it is owner-read-write, as the player log is.
//!
//! Each scenario of a ladder's pool is compiled as an offer of its own, `<ladder>.<n>` (`n` from
//! 1, in the configuration's order; E16 M4 T2): its games' journals and replays name it, and a
//! restart finds it again. No client creates a game of it: its games are the ladder's matches.
//!
//! Provisional (E16 M4 T1, T2): the layout, the names, one lock for every ladder; a season starts
//! at the start of a server whose content changed, and only then.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use h3_protocol::account;
use h3_protocol::session::Grace;
use serde::{Deserialize, Serialize};

use crate::game::Offer;
use crate::rating::{Rating, Standing};
use crate::{Error, config, journal};

/// Who won a rated game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Outcome {
    /// The attacker's account.
    Attacker,
    /// The defender's account.
    Defender,
    /// Neither.
    Draw,
}

/// One line of a ladder's results log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum Line {
    /// A season starts, on the ruleset of this hash (16 hex digits after `0x`).
    Season { season: u32, ruleset: String },
    /// A game was rated (E16 M4 T3b): its accounts and their ratings after it, the attacker's
    /// first; `forfeit` when the loser forfeited (E16 M4 T3a).
    Rated {
        game: u32,
        replay: String,
        accounts: [String; 2],
        outcome: Outcome,
        forfeit: bool,
        ratings: [Rating; 2],
    },
    /// Both players of a game were absent beyond the grace: void, unrated, a no-show each (E16 M4
    /// T3a).
    NoShow {
        game: u32,
        replay: String,
        accounts: [String; 2],
    },
}

/// A season: its ruleset and its table.
#[derive(Debug, Clone, PartialEq)]
struct Season {
    /// The ruleset's hash.
    ruleset: u64,
    /// The accounts that played in it, by login name.
    table: BTreeMap<String, Standing>,
}

/// What a ladder's results log holds, read back.
#[derive(Debug, Clone, Default, PartialEq)]
struct Results {
    /// The seasons, the first first.
    seasons: Vec<Season>,
    /// Every account's latest rating, across the seasons.
    ratings: BTreeMap<String, Rating>,
}

impl Results {
    /// Whether `line` follows from the results as they stand, `known` naming the player log's
    /// accounts.
    fn check(&self, line: &Line, known: &dyn Fn(&str) -> bool) -> Result<(), String> {
        let fail = |why: &str| Err(String::from(why));
        let accounts = match line {
            Line::Season { season, ruleset } => {
                if usize::try_from(*season).ok() != self.seasons.len().checked_add(1) {
                    return fail("a season out of order");
                }
                if hash(ruleset).is_none() {
                    return fail("not a ruleset hash");
                }
                return Ok(());
            }
            Line::Rated {
                accounts,
                outcome,
                forfeit,
                ratings,
                ..
            } => {
                if !ratings.iter().all(|rating| rating.is_valid()) {
                    return fail("not a rating");
                }
                if *forfeit && *outcome == Outcome::Draw {
                    return fail("a forfeit without a winner");
                }
                accounts
            }
            Line::NoShow { accounts, .. } => accounts,
        };
        if self.seasons.is_empty() {
            return fail("a game before the first season");
        }
        if !accounts.iter().all(|login| account::is_login_name(login)) {
            return fail("not a name");
        }
        if accounts[0] == accounts[1] {
            return fail("an account against itself");
        }
        if !accounts.iter().all(|login| known(login)) {
            return fail("no such account");
        }
        Ok(())
    }

    /// Applies a checked line to the tables.
    fn update(&mut self, line: Line) {
        match line {
            Line::Season { ruleset, .. } => {
                if !self.seasons.is_empty() {
                    for rating in self.ratings.values_mut() {
                        *rating = rating.new_season();
                    }
                }
                self.seasons.push(Season {
                    ruleset: hash(&ruleset).unwrap_or_default(),
                    table: BTreeMap::new(),
                });
            }
            Line::Rated {
                accounts,
                outcome,
                ratings,
                ..
            } => {
                let won = match outcome {
                    Outcome::Attacker => [Some(true), Some(false)],
                    Outcome::Defender => [Some(false), Some(true)],
                    Outcome::Draw => [None, None],
                };
                for ((login, rating), won) in accounts.into_iter().zip(ratings).zip(won) {
                    self.ratings.insert(login.clone(), rating);
                    let Some(season) = self.seasons.last_mut() else {
                        continue;
                    };
                    let row = (season.table.entry(login)).or_insert_with(|| Standing::new(rating));
                    row.rating = rating;
                    row.games += 1;
                    match won {
                        Some(true) => row.wins += 1,
                        Some(false) => row.losses += 1,
                        None => row.draws += 1,
                    }
                }
            }
            Line::NoShow { accounts, .. } => {
                for login in accounts {
                    let rating = self.rating(&login);
                    let Some(season) = self.seasons.last_mut() else {
                        continue;
                    };
                    let row = (season.table.entry(login)).or_insert_with(|| Standing::new(rating));
                    row.no_shows += 1;
                }
            }
        }
    }

    /// The account's rating as it stands: a new account's if it has none.
    fn rating(&self, login: &str) -> Rating {
        self.ratings.get(login).copied().unwrap_or_default()
    }
}

/// A ruleset hash as a line writes it: `0x` and 16 hex digits.
fn hash(text: &str) -> Option<u64> {
    let hex = text.strip_prefix("0x").filter(|hex| hex.len() == 16)?;
    u64::from_str_radix(hex, 16).ok()
}

/// A ladder's pool as its matches draw from it (E16 M4 T2).
#[derive(Debug, Clone)]
pub(crate) struct Pool {
    /// Its scenarios, each compiled as an offer of its own (`<ladder>.<n>`).
    pub offers: Vec<Arc<Offer>>,
    /// Whether its games take spectators.
    pub spectators: bool,
}

/// A ladder, compiled at start.
#[derive(Debug)]
struct Ladder {
    /// Its configuration.
    config: config::Ladder,
    /// Its pool.
    pool: Vec<Arc<Offer>>,
    /// Its results, read back.
    results: Results,
    /// The log, open for appending; `None` once a write failed (as the player log's).
    file: Option<File>,
}

/// The server's ladders, by id.
#[derive(Debug, Default)]
pub(crate) struct Ladders {
    ladders: Mutex<BTreeMap<String, Ladder>>,
}

/// A ladder's log in the data directory `data`.
fn path(data: &Path, id: &str) -> PathBuf {
    data.join("ladders").join(format!("{id}.log"))
}

impl Ladders {
    /// Compiles the configured ladders on the packs in `packs`, reads their results logs back from
    /// the data directory `data` (`known` names the player log's accounts) and starts a season
    /// where the content changed.
    pub(crate) fn open(
        ladders: &[config::Ladder],
        packs: &Path,
        data: &Path,
        known: &dyn Fn(&str) -> bool,
    ) -> Result<Self, Error> {
        let mut opened = BTreeMap::new();
        for config in ladders {
            let ladder = Ladder::open(config, packs, data, known)?;
            opened.insert(config.id.clone(), ladder);
        }
        Ok(Self {
            ladders: Mutex::new(opened),
        })
    }

    /// The ladders, locked.
    fn lock(&self) -> MutexGuard<'_, BTreeMap<String, Ladder>> {
        self.ladders.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The ladder `id`'s current season.
    pub(crate) fn season(&self, id: &str) -> Option<u32> {
        let ladders = self.lock();
        let seasons = ladders.get(id)?.results.seasons.len();
        u32::try_from(seasons).ok()
    }

    /// The ladders as their queues know them (E16 M4 T2).
    pub(crate) fn queues(&self) -> Vec<h3_protocol::ladder::Ladder> {
        let ladders = self.lock();
        (ladders.iter())
            .map(|(id, ladder)| h3_protocol::ladder::Ladder {
                id: id.clone(),
                name: ladder.config.name.clone(),
                season: u32::try_from(ladder.results.seasons.len()).unwrap_or(u32::MAX),
                accept_ms: ladder.config.accept_ms,
            })
            .collect()
    }

    /// Each ladder's pool, by the ladder's id (E16 M4 T2).
    pub(crate) fn pools(&self) -> BTreeMap<String, Pool> {
        let ladders = self.lock();
        (ladders.iter())
            .map(|(id, ladder)| {
                let pool = Pool {
                    offers: ladder.pool.clone(),
                    spectators: ladder.config.spectators,
                };
                (id.clone(), pool)
            })
            .collect()
    }

    /// The account `login`'s rating on the ladder `id` as it stands, in whole points (what its
    /// queue goes by; E16 M4 T2): a new account's for one that has none.
    pub(crate) fn points(&self, id: &str, login: &str) -> i32 {
        let ladders = self.lock();
        let rating = ladders.get(id).map(|l| l.results.rating(login));
        // A rating is a few thousand points: the cast saturates far from where it would.
        rating.unwrap_or_default().rating.round() as i32
    }

    /// The account `login`'s row on the ladder `id` in the season `season`, if it played in it.
    pub(crate) fn standing(&self, id: &str, season: u32, login: &str) -> Option<Standing> {
        let ladders = self.lock();
        let index = usize::try_from(season).ok()?.checked_sub(1)?;
        let season = ladders.get(id)?.results.seasons.get(index)?;
        season.table.get(login).copied()
    }
}

impl Ladder {
    /// Compiles the ladder, reads its log back and starts a season where the content changed.
    fn open(
        config: &config::Ladder,
        packs: &Path,
        data: &Path,
        known: &dyn Fn(&str) -> bool,
    ) -> Result<Self, Error> {
        let at = |e: &dyn std::fmt::Display| Error(format!("ladder `{}`: {e}", config.id));
        let pool = (config.scenarios.iter().enumerate())
            .map(|(i, scenario)| {
                let offer = config::Offer {
                    id: format!("{}.{}", config.id, i + 1),
                    scenario: scenario.clone(),
                    profile: config.profile.clone(),
                    bots: Vec::new(),
                };
                Offer::load(&offer, packs).map(|offer| {
                    Arc::new(Offer {
                        ladder: Some(config.id.clone()),
                        grace: Some(Grace {
                            stretch_ms: config.grace_ms,
                            total_ms: config.grace_total_ms,
                        }),
                        ..offer
                    })
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ruleset = pool.first().map(|offer| offer.ruleset.content_hash());
        let Some(ruleset) = ruleset else {
            return Err(at(&"no scenario"));
        };
        if let Some(other) = pool.iter().find(|o| o.ruleset.content_hash() != ruleset) {
            return Err(at(&format!(
                "{}: its scenarios are on different rulesets",
                other.scenario
            )));
        }
        let path = path(data, &config.id);
        let shown = |e: &dyn std::fmt::Display| at(&format!("{}: {e}", path.display()));
        let (file, text) = journal::open_log(&path).map_err(|e| shown(&e))?;
        let mut results = Results::default();
        for (i, line) in text.lines().enumerate() {
            let line = serde_json::from_str::<Line>(line).map_err(|e| journal::json(&e));
            line.and_then(|line| {
                results.check(&line, known)?;
                results.update(line);
                Ok(())
            })
            .map_err(|e| shown(&format!("line {}: {e}", i + 1)))?;
        }
        let mut ladder = Self {
            config: config.clone(),
            pool,
            results,
            file: Some(file),
        };
        let last = ladder.results.seasons.last().map(|season| season.ruleset);
        if last != Some(ruleset) {
            let season = ladder.results.seasons.len().saturating_add(1);
            let line = Line::Season {
                season: u32::try_from(season).unwrap_or(u32::MAX),
                ruleset: format!("{ruleset:#018x}"),
            };
            ladder.record(line, known).map_err(|e| shown(&e))?;
            eprintln!(
                "h3-server: ladder `{}`: season {season} starts (its ruleset is new)",
                config.id
            );
        }
        Ok(ladder)
    }

    /// Records `line`, checked against the results as they stand and the player log's accounts
    /// (`known`): written and synced first.
    fn record(&mut self, line: Line, known: &dyn Fn(&str) -> bool) -> Result<(), String> {
        self.results.check(&line, known)?;
        let text = serde_json::to_string(&line).map_err(|e| e.to_string())?;
        journal::append_line(&mut self.file, &text).map_err(|e| e.to_string())?;
        self.results.update(line);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    use super::*;
    use crate::rating::{self, Score};

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn data() -> PathBuf {
        let random = crate::game::os_u64().unwrap();
        std::env::temp_dir().join(format!("h3-ladders-{random:016x}"))
    }

    /// The ladder `arena`: the synthetic duel and full battle, on `profile` if given.
    fn arena(profile: Option<&str>) -> config::Ladder {
        let scenarios = root().join("packs/synthetic/scenarios");
        config::Ladder {
            id: String::from("arena"),
            name: String::from("Arena"),
            scenarios: vec![scenarios.join("duel.toml"), scenarios.join("full.toml")],
            profile: profile.map(String::from),
            ..config::Ladder::default()
        }
    }

    fn open(data: &Path, profile: Option<&str>) -> Result<Ladders, Error> {
        let known = |login: &str| ["alice", "bob"].contains(&login);
        Ladders::open(&[arena(profile)], &root().join("packs"), data, &known)
    }

    fn line(line: &Line) -> String {
        format!("{}\n", serde_json::to_string(line).unwrap())
    }

    fn append(data: &Path, text: &str) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path(data, "arena"))
            .unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    /// Alice beat Bob in game `game`; their ratings after it.
    fn rated(game: u32, before: [Rating; 2]) -> (Line, [Rating; 2]) {
        let (alice, bob) = rating::game(before[0], before[1], Score::Win);
        let line = Line::Rated {
            game,
            replay: format!("arena-game{game}-0123456789abcdef"),
            accounts: [String::from("alice"), String::from("bob")],
            outcome: Outcome::Attacker,
            forfeit: false,
            ratings: [alice, bob],
        };
        (line, [alice, bob])
    }

    #[test]
    fn seasons_follow_the_ladders_ruleset() {
        let data = data();
        let ladders = open(&data, None).unwrap();
        assert_eq!(ladders.season("arena"), Some(1));
        assert_eq!(ladders.season("other"), None);
        drop(ladders);
        let text = fs::read_to_string(path(&data, "arena")).unwrap();
        assert!(
            text.starts_with("{\"Season\":{\"season\":1,\"ruleset\":\"0x")
                && text.lines().count() == 1,
            "{text}"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!(
                (mode(&path(&data, "arena")), mode(&data.join("ladders"))),
                (0o600, 0o700)
            );
        }
        // The same content: the same season, nothing written.
        assert_eq!(open(&data, None).unwrap().season("arena"), Some(1));
        assert_eq!(fs::read_to_string(path(&data, "arena")).unwrap(), text);
        // Another profile is another ruleset: the next season; back again, the one after.
        assert_eq!(
            open(&data, Some("synthetic:strict-fixed"))
                .unwrap()
                .season("arena"),
            Some(2)
        );
        assert_eq!(open(&data, None).unwrap().season("arena"), Some(3));
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn rated_games_are_read_back_bit_for_bit() {
        let data = data();
        drop(open(&data, None).unwrap());
        let (first, _) = rated(1, [Rating::default(); 2]);
        let (second, after) = rated(2, {
            let Line::Rated { ratings, .. } = &first else {
                unreachable!()
            };
            *ratings
        });
        let draw = Line::NoShow {
            game: 3,
            replay: String::from("arena-game3-0123456789abcdef"),
            accounts: [String::from("bob"), String::from("alice")],
        };
        append(
            &data,
            &format!("{}{}{}", line(&first), line(&second), line(&draw)),
        );
        // A crash cut the next line short: it is dropped and cut.
        let text = fs::read_to_string(path(&data, "arena")).unwrap();
        append(&data, "{\"Rated\":{\"game\":4,");
        let ladders = open(&data, None).unwrap();
        assert_eq!(fs::read_to_string(path(&data, "arena")).unwrap(), text);
        let alice = ladders.standing("arena", 1, "alice").unwrap();
        let bob = ladders.standing("arena", 1, "bob").unwrap();
        assert_eq!(
            [alice.rating, bob.rating]
                .map(|r| [r.rating, r.deviation, r.volatility].map(f64::to_bits)),
            after.map(|r| [r.rating, r.deviation, r.volatility].map(f64::to_bits))
        );
        assert_eq!(
            (
                alice.games,
                alice.wins,
                alice.losses,
                alice.draws,
                alice.no_shows
            ),
            (2, 2, 0, 0, 1)
        );
        assert_eq!(
            (bob.games, bob.wins, bob.losses, bob.no_shows),
            (2, 0, 2, 1)
        );
        assert_eq!(ladders.standing("arena", 1, "carol"), None);
        assert_eq!(ladders.standing("arena", 2, "alice"), None);
        drop(ladders);
        // A new season: every rating carried, its deviation reset; the old table stays readable.
        let ladders = open(&data, Some("synthetic:strict-fixed")).unwrap();
        assert_eq!(ladders.standing("arena", 1, "alice"), Some(alice));
        assert_eq!(ladders.standing("arena", 2, "alice"), None);
        drop(ladders);
        append(&data, &line(&draw));
        let ladders = open(&data, Some("synthetic:strict-fixed")).unwrap();
        let carried = ladders.standing("arena", 2, "alice").unwrap();
        assert_eq!(carried.rating, alice.rating.new_season());
        assert_eq!((carried.games, carried.no_shows), (0, 1));
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn a_line_that_does_not_read_or_contradicts_stops_the_start() {
        let season = |n: u32| {
            line(&Line::Season {
                season: n,
                ruleset: format!("{:#018x}", 7),
            })
        };
        let (rated, _) = rated(1, [Rating::default(); 2]);
        let rated = line(&rated);
        let cases = [
            (season(2), "line 1: a season out of order"),
            (
                format!("{}{}", season(1), season(1)),
                "line 2: a season out of order",
            ),
            (
                season(1).replace("0x0000000000000007", "0x7"),
                "line 1: not a ruleset hash",
            ),
            (rated.clone(), "line 1: a game before the first season"),
            (
                format!("{}{}", season(1), rated.replace("bob", "carol")),
                "line 2: no such account",
            ),
            (
                format!("{}{}", season(1), rated.replace("bob", "alice")),
                "line 2: an account against itself",
            ),
            (
                format!("{}{}", season(1), rated.replace("bob", "Bob")),
                "line 2: not a name",
            ),
            (
                format!(
                    "{}{}",
                    season(1),
                    rated
                        .replace("\"forfeit\":false", "\"forfeit\":true")
                        .replace("Attacker", "Draw")
                ),
                "line 2: a forfeit without a winner",
            ),
            (
                format!(
                    "{}{}",
                    season(1),
                    rated.replacen("\"volatility\":0.0", "\"volatility\":-0.0", 1)
                ),
                "line 2: not a rating",
            ),
            (format!("{}{{\"Rated\":{{}}}}\n", season(1)), "line 2: Data"),
        ];
        for (text, expected) in cases {
            let data = data();
            journal::create_dir(&data.join("ladders")).unwrap();
            fs::write(path(&data, "arena"), &text).unwrap();
            let error = open(&data, None).unwrap_err().0;
            assert!(error.contains(expected), "{expected}: {error}");
            assert!(!error.contains("alice"), "never the line's text: {error}");
            fs::remove_dir_all(&data).unwrap();
        }
    }

    #[test]
    fn a_pool_is_on_one_ruleset() {
        let data = data();
        journal::create_dir(&data).unwrap();
        let duel = root().join("packs/synthetic/scenarios/duel.toml");
        let strict = data.join("strict.toml");
        let text = fs::read_to_string(&duel).unwrap();
        fs::write(
            &strict,
            text.replace("synthetic:faithful", "synthetic:strict-fixed"),
        )
        .unwrap();
        let ladder = config::Ladder {
            scenarios: vec![duel, strict],
            ..arena(None)
        };
        let error = Ladders::open(&[ladder], &root().join("packs"), &data, &|_| true).unwrap_err();
        assert!(error.0.contains("different rulesets"), "{error}");
        let empty = config::Ladder {
            scenarios: vec![],
            ..arena(None)
        };
        let error = Ladders::open(&[empty], &root().join("packs"), &data, &|_| true).unwrap_err();
        assert!(error.0.contains("no scenario"), "{error}");
        fs::remove_dir_all(&data).unwrap();
    }
}
