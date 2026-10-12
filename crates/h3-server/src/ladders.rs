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
//! Each season's table keeps its leaderboard in order (E16 M4 T4): the accounts with a rated game
//! in the season, by rating, the highest first, ties by login name. A client reads it a page of
//! [`PAGE`] rows at a time, any season's, and its own row in the current one with its rank.
//!
//! Provisional (E16 M4 T1, T2, T4): the layout, the names, one lock for every ladder; a season
//! starts at the start of a server whose content changed, and only then; a leaderboard goes by the
//! rating itself, not by a bound below it (the rating less twice its deviation).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use h3_protocol::account;
use h3_protocol::session::Grace;
use h3_protocol::wire::{Points, Refusal};
use serde::{Deserialize, Serialize};

use crate::game::Offer;
use crate::rating::{self, Rating, Score, Standing};
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

/// How a ranked game ended, for its ladder (E16 M4 T3b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum End {
    /// Who won, and whether the loser forfeited.
    Result { outcome: Outcome, forfeit: bool },
    /// Both players were absent beyond the grace: void, a no-show each.
    NoShow,
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
    /// Its leaderboard (E16 M4 T4): the accounts with a rated game in it, by rating, the highest
    /// first, ties by login name. Whatever changes a row's rating in `table` moves its entry here
    /// too (`Results::update`), or a page and one's own rank disagree with the table.
    order: BTreeSet<(Ahead, String)>,
}

/// A rating as a leaderboard orders it (E16 M4 T4): the highest first. A line's ratings are
/// finite (checked), so the order is the numbers'.
#[derive(Debug, Clone, Copy)]
struct Ahead(f64);

impl Ord for Ahead {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0)
    }
}

impl PartialOrd for Ahead {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Ahead {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Ahead {}

/// The rows a leaderboard's page lists at most (E16 M4 T4).
pub(crate) const PAGE: usize = 50;

/// A season's leaderboard as a request reads it (E16 M4 T4).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Board {
    /// The season.
    pub season: u32,
    /// How many accounts it lists.
    pub total: u32,
    /// The rows asked for: each one's rank (the first 1), login name and row.
    pub rows: Vec<(u32, String, Standing)>,
}

/// A count or a place on the wire: saturated (no ladder holds 2³² accounts).
fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// What a ladder's results log holds, read back.
#[derive(Debug, Clone, Default, PartialEq)]
struct Results {
    /// The seasons, the first first.
    seasons: Vec<Season>,
    /// Every account's latest rating, across the seasons.
    ratings: BTreeMap<String, Rating>,
    /// The games a line names (E16 M4 T3b): game ids are unique for the server's life, so a game
    /// is rated once.
    games: BTreeSet<u32>,
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
                game,
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
                (game, accounts)
            }
            Line::NoShow { game, accounts, .. } => (game, accounts),
        };
        let (game, accounts) = accounts;
        if self.seasons.is_empty() {
            return fail("a game before the first season");
        }
        if self.games.contains(game) {
            return fail("a game named twice");
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
                    order: BTreeSet::new(),
                });
            }
            Line::Rated {
                game,
                accounts,
                outcome,
                ratings,
                ..
            } => {
                self.games.insert(game);
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
                    let row = (season.table.entry(login.clone()))
                        .or_insert_with(|| Standing::new(rating));
                    // Its place on the leaderboard moves with its rating (E16 M4 T4); a row
                    // without a rated game has none yet.
                    season
                        .order
                        .remove(&(Ahead(row.rating.rating), login.clone()));
                    season.order.insert((Ahead(rating.rating), login));
                    row.rating = rating;
                    row.games += 1;
                    match won {
                        Some(true) => row.wins += 1,
                        Some(false) => row.losses += 1,
                        None => row.draws += 1,
                    }
                }
            }
            Line::NoShow { game, accounts, .. } => {
                self.games.insert(game);
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

    /// The season `number` (the current one for `None`) and its number, if there is one.
    fn season(&self, number: Option<u32>) -> Option<(u32, &Season)> {
        let number = number.unwrap_or_else(|| count(self.seasons.len()));
        let index = usize::try_from(number).ok()?.checked_sub(1)?;
        Some((number, self.seasons.get(index)?))
    }
}

/// A rating as the wire carries it (E16 M4 T3b): rounded to whole points.
pub(crate) fn points(rating: Rating) -> Points {
    // A rating is a few thousand points, a deviation at most 350: the casts saturate far from
    // where they would.
    Points {
        rating: rating.rating.round() as i32,
        deviation: rating.deviation.round() as u32,
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

    /// The highest game id the ladders' logs name (0 for none; E16 M4 T7): the server hands out
    /// higher ones, so a game whose `next-game` was lost is still rated once, never taken for one
    /// the log names already.
    pub(crate) fn last_game(&self) -> u32 {
        let ladders = self.lock();
        let games = ladders.values().filter_map(|l| l.results.games.last());
        games.copied().max().unwrap_or(0)
    }

    /// The ladder `id`'s current season.
    pub(crate) fn season(&self, id: &str) -> Option<u32> {
        let ladders = self.lock();
        let seasons = ladders.get(id)?.results.seasons.len();
        u32::try_from(seasons).ok()
    }

    /// The ladders as their queues know them (E16 M4 T2), with their bots as rated now (T5).
    pub(crate) fn queues(&self) -> Vec<h3_protocol::ladder::Ladder> {
        let ladders = self.lock();
        (ladders.iter())
            .map(|(id, ladder)| h3_protocol::ladder::Ladder {
                id: id.clone(),
                name: ladder.config.name.clone(),
                season: u32::try_from(ladder.results.seasons.len()).unwrap_or(u32::MAX),
                accept_ms: ladder.config.accept_ms,
                bots: (ladder.config.bots.iter())
                    .map(|bot| h3_protocol::ladder::LadderBot {
                        login: bot.login.clone(),
                        display: bot.display.clone(),
                        rating: points(ladder.results.rating(&bot.login)).rating,
                    })
                    .collect(),
                bot_after_ms: ladder.config.bot_after_ms,
            })
            .collect()
    }

    /// Every ladder's bots and their ratings as they stand, in whole points (E16 M4 T5): the
    /// matcher's turn goes by them.
    pub(crate) fn bots(&self) -> Vec<(String, String, i32)> {
        let ladders = self.lock();
        (ladders.iter())
            .flat_map(|(id, ladder)| {
                (ladder.config.bots.iter()).map(|bot| {
                    let rating = points(ladder.results.rating(&bot.login)).rating;
                    (id.clone(), bot.login.clone(), rating)
                })
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
        points(rating.unwrap_or_default()).rating
    }

    /// Records how the ladder `id`'s game `game` ended (`end`; E16 M4 T3b), its replay's id
    /// `replay` and its seats' accounts `accounts` (the attacker's first, each known to the player
    /// log as `known` says: asked before, outside this lock): a `Rated` line with both ratings
    /// after the game, as Glicko-2 rates one game between them, or a `NoShow` line, written and
    /// synced before the tables change. Each account's rating before and after the game; `None` for
    /// a no-show and for a game the log names already (a restart re-ran a game whose journal
    /// outlived its line). Why not, for a line that does not follow or does not write.
    pub(crate) fn rate(
        &self,
        id: &str,
        (game, replay): (u32, &str),
        accounts: [String; 2],
        (end, known): (End, &dyn Fn(&str) -> bool),
    ) -> Result<Option<[(Rating, Rating); 2]>, String> {
        let mut ladders = self.lock();
        let ladder = ladders
            .get_mut(id)
            .ok_or_else(|| format!("no ladder `{id}`"))?;
        if ladder.results.games.contains(&game) {
            return Ok(None);
        }
        let replay = replay.to_owned();
        let End::Result { outcome, forfeit } = end else {
            let line = Line::NoShow {
                game,
                replay,
                accounts,
            };
            return ladder.record(line, known).map(|()| None);
        };
        let before = accounts
            .each_ref()
            .map(|login| ladder.results.rating(login));
        let score = match outcome {
            Outcome::Attacker => Score::Win,
            Outcome::Defender => Score::Loss,
            Outcome::Draw => Score::Draw,
        };
        let (attacker, defender) = rating::game(before[0], before[1], score);
        let line = Line::Rated {
            game,
            replay,
            accounts,
            outcome,
            forfeit,
            ratings: [attacker, defender],
        };
        ladder.record(line, known)?;
        Ok(Some([(before[0], attacker), (before[1], defender)]))
    }

    /// The account `login`'s row on the ladder `id` in the season `season`, if it played in it.
    pub(crate) fn standing(&self, id: &str, season: u32, login: &str) -> Option<Standing> {
        let ladders = self.lock();
        let (_, season) = ladders.get(id)?.results.season(Some(season))?;
        season.table.get(login).copied()
    }

    /// A page of the ladder `id`'s leaderboard in the season `season` (the current one for `None`;
    /// E16 M4 T4): [`PAGE`] rows from the `page`-th page (the first 0), none past the last. Refused
    /// for a ladder or a season the server does not have.
    pub(crate) fn page(&self, id: &str, season: Option<u32>, page: u32) -> Result<Board, Refusal> {
        let ladders = self.lock();
        let ladder = ladders.get(id).ok_or(Refusal::NoSuchLadder)?;
        let (number, season) = ladder.results.season(season).ok_or(Refusal::NoSuchSeason)?;
        let from = usize::try_from(page).map_or(usize::MAX, |page| page.saturating_mul(PAGE));
        let rows = (season.order.iter().enumerate().skip(from).take(PAGE))
            .filter_map(|(i, (_, login))| {
                let row = season.table.get(login)?;
                Some((count(i.saturating_add(1)), login.clone(), *row))
            })
            .collect();
        Ok(Board {
            season: number,
            total: count(season.order.len()),
            rows,
        })
    }

    /// The account `login`'s own row on the ladder `id`'s leaderboard in its current season (E16
    /// M4 T4): none before its first rated game of the season. Refused for a ladder the server
    /// does not have.
    pub(crate) fn own(&self, id: &str, login: &str) -> Result<Board, Refusal> {
        let ladders = self.lock();
        let ladder = ladders.get(id).ok_or(Refusal::NoSuchLadder)?;
        let (number, season) = ladder.results.season(None).ok_or(Refusal::NoSuchSeason)?;
        let row = (season.table.get(login))
            .filter(|row| row.games > 0)
            .map(|row| {
                let place = (Ahead(row.rating.rating), login.to_owned());
                let ahead = season.order.range(..place).count();
                (count(ahead.saturating_add(1)), login.to_owned(), *row)
            });
        Ok(Board {
            season: number,
            total: count(season.order.len()),
            rows: row.into_iter().collect(),
        })
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
                            pauses: config.pauses,
                            pause_ms: config.pause_ms,
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
        let no_show = Line::NoShow {
            game: 4,
            replay: String::from("arena-game4-0123456789abcdef"),
            accounts: [String::from("bob"), String::from("alice")],
        };
        append(&data, &line(&no_show));
        let ladders = open(&data, Some("synthetic:strict-fixed")).unwrap();
        let carried = ladders.standing("arena", 2, "alice").unwrap();
        assert_eq!(carried.rating, alice.rating.new_season());
        assert_eq!((carried.games, carried.no_shows), (0, 1));
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn a_game_is_rated_once_as_glicko_2_rates_it() {
        // E16 M4 T3b.
        let data = data();
        let ladders = open(&data, None).unwrap();
        let known = |login: &str| ["alice", "bob"].contains(&login);
        let names = |a: &str, b: &str| [String::from(a), String::from(b)];
        let new = Rating::default();
        // Bob forfeits game 5: Alice wins it, as Glicko-2 rates one game between new players.
        let forfeit = End::Result {
            outcome: Outcome::Attacker,
            forfeit: true,
        };
        let rate = |game, accounts, end| {
            let replay = format!("arena.1-game{game}-0123456789abcdef");
            ladders.rate("arena", (game, &replay), accounts, (end, &known))
        };
        let (alice, bob) = rating::game(new, new, Score::Win);
        assert_eq!(
            rate(5, names("alice", "bob"), forfeit),
            Ok(Some([(new, alice), (new, bob)]))
        );
        // Once: a restart's re-run of the same game changes nothing.
        assert_eq!(rate(5, names("alice", "bob"), forfeit), Ok(None));
        // Bob, attacking, loses again; a draw; a no-show, unrated.
        let defender = End::Result {
            outcome: Outcome::Defender,
            forfeit: false,
        };
        let (bob_after, alice_after) = rating::game(bob, alice, Score::Loss);
        assert_eq!(
            rate(6, names("bob", "alice"), defender),
            Ok(Some([(bob, bob_after), (alice, alice_after)]))
        );
        let draw = End::Result {
            outcome: Outcome::Draw,
            forfeit: false,
        };
        assert!(rate(7, names("alice", "bob"), draw).unwrap().is_some());
        assert_eq!(rate(8, names("alice", "bob"), End::NoShow), Ok(None));
        // Refused, nothing written: an account the player log does not know, another ladder.
        assert_eq!(
            rate(9, names("alice", "carol"), forfeit),
            Err(String::from("no such account"))
        );
        let other = ladders.rate("pit", (9, "r"), names("alice", "bob"), (draw, &known));
        assert_eq!(other, Err(String::from("no ladder `pit`")));
        let row = ladders.standing("arena", 1, "alice").unwrap();
        assert_eq!(
            (row.games, row.wins, row.losses, row.draws, row.no_shows),
            (3, 2, 0, 1, 1)
        );
        // The log reads back to the same tables.
        drop(ladders);
        let again = open(&data, None).unwrap();
        assert_eq!(again.standing("arena", 1, "alice"), Some(row));
        let text = fs::read_to_string(path(&data, "arena")).unwrap();
        assert!(text.contains("\"forfeit\":true"), "{text}");
        assert_eq!(text.lines().count(), 5, "a season, three rated, a no-show");
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn a_leaderboard_pages_by_rating_ties_by_login_name() {
        // E16 M4 T4.
        let data = data();
        let known = |_: &str| true;
        let ladders = Ladders::open(&[arena(None)], &root().join("packs"), &data, &known).unwrap();
        let won = End::Result {
            outcome: Outcome::Attacker,
            forfeit: false,
        };
        let rate = |game: u32, winner: &str, loser: &str, end| {
            let accounts = [String::from(winner), String::from(loser)];
            let replay = format!("arena.1-game{game}-0123456789abcdef");
            ladders
                .rate("arena", (game, &replay), accounts, (end, &known))
                .unwrap();
        };
        // Sixty winners, each over a new account: the winners tie, and so do the losers.
        for i in 0..60 {
            rate(i, &format!("won{i:02}"), &format!("lost{i:02}"), won);
        }
        let logins = |board: &Board| -> Vec<String> {
            board
                .rows
                .iter()
                .map(|(_, login, _)| login.clone())
                .collect()
        };
        let names = |prefix: &str, range: std::ops::Range<u32>| -> Vec<String> {
            range.map(|i| format!("{prefix}{i:02}")).collect()
        };
        let first = ladders.page("arena", None, 0).unwrap();
        assert_eq!((first.season, first.total), (1, 120));
        assert_eq!(logins(&first), names("won", 0..50));
        assert_eq!(
            first
                .rows
                .iter()
                .map(|(rank, ..)| *rank)
                .collect::<Vec<_>>(),
            (1..=50).collect::<Vec<_>>()
        );
        let second = ladders.page("arena", Some(1), 1).unwrap();
        assert_eq!(
            logins(&second),
            [names("won", 50..60), names("lost", 0..40)].concat()
        );
        assert_eq!(second.rows[10].0, 61);
        assert_eq!(logins(&ladders.page("arena", None, 2).unwrap()).len(), 20);
        for past in [3, u32::MAX] {
            let page = ladders.page("arena", None, past).unwrap();
            assert_eq!((page.total, page.rows), (120, vec![]));
        }
        // A second win lifts `won59` to the top: its row moves, the count stays.
        rate(60, "won59", "lost59", won);
        let own = |login: &str| ladders.own("arena", login).unwrap();
        let top = own("won59");
        assert_eq!((top.season, top.total), (1, 120));
        assert_eq!(
            top.rows
                .iter()
                .map(|(rank, login, row)| (*rank, login.as_str(), row.games, row.wins))
                .collect::<Vec<_>>(),
            [(1, "won59", 2, 2)]
        );
        assert_eq!(own("won00").rows[0].0, 2);
        assert_eq!(own("lost59").rows[0].0, 120);
        assert_eq!(
            logins(&ladders.page("arena", None, 0).unwrap())[..2],
            ["won59", "won00"]
        );
        // A no-show is no rated game: neither account is listed.
        rate(61, "xavier", "yves", End::NoShow);
        assert_eq!((own("xavier").total, own("xavier").rows), (120, vec![]));
        assert_eq!(own("nobody").rows, vec![]);
        // A ladder or a season the server does not have.
        assert_eq!(ladders.page("pit", None, 0), Err(Refusal::NoSuchLadder));
        assert_eq!(ladders.own("pit", "won00"), Err(Refusal::NoSuchLadder));
        for season in [0, 2] {
            let page = ladders.page("arena", Some(season), 0);
            assert_eq!(page, Err(Refusal::NoSuchSeason));
        }
        let before = [0, 1, 2].map(|page| ladders.page("arena", None, page).unwrap());
        drop(ladders);
        // Read back, the same pages; the next season's leaderboard is empty, the last one's kept.
        let ladders = Ladders::open(
            &[arena(Some("synthetic:strict-fixed"))],
            &root().join("packs"),
            &data,
            &known,
        )
        .unwrap();
        assert_eq!(
            [0, 1, 2].map(|page| ladders.page("arena", Some(1), page).unwrap()),
            before
        );
        let next = ladders.page("arena", None, 0).unwrap();
        assert_eq!((next.season, next.total, next.rows), (2, 0, vec![]));
        let own = ladders.own("arena", "won59").unwrap();
        assert_eq!((own.season, own.rows), (2, vec![]));
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
            // E16 M4 T3b: game ids are unique for the server's life.
            (
                format!("{}{rated}{rated}", season(1)),
                "line 3: a game named twice",
            ),
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
