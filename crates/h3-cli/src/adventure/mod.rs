//! The adventure side of the `h3` tool (E08 T9b; adventure-owned, `lanes.toml`): loading packs
//! with their adventure files and `.h3m` maps from disk, recording random games and the adventure
//! replay file format (TOML). The commands themselves are in [`cli`].
//!
//! Replay file format v7 (`fixtures/replays/adventure/*.toml` are examples; v2 added the
//! difficulty and the answers, E08 T14b: a v1 file has no difficulty, reads as normal and is
//! refused by its format version; a later file without one is malformed; v3 added the battle
//! commands, E08 T19b1, and names each command's actor: a player by its number; v4 the system
//! actor, `system`, whose commands are a battle's unowned side's, E08 T20b2; v5 a level-up's
//! answers, `answer first` and `answer second`, E08 T30; v6 the management commands, E08 T51a:
//! `army move|split|dismiss` naming slots as `<army>:<slot>`, the army a hero's id or `g` the
//! army dialog's group; `formation <hero> loose|tight`; `tactics <hero> on|off`; `artifact
//! <hero>:<place> <hero>:<place>`, a place `w<position>` worn or `b<index>` in the backpack;
//! `assemble` and `disassemble <hero> <position>`; v7 the answers typed by the decision's kind,
//! E08 T63a: `answer <kind> <answer>`, the kinds `offer accept|decline`, `level-up
//! first|second`, `chest gold|experience`, `arena attack|defense`, `school first|second|leave`,
//! `university buy <slot>|leave`, and `answer close` for a meeting or an army dialog):
//!
//! ```toml
//! commands = [                        # the game commands in order, each after its actor:
//!     "0 move 1 5,7,0 6,8,0",         #   `<player> move <hero> <x>,<y>,<level> …` (the path
//!     "0 end-turn",                   #   without the hero's own tile), `<player> end-turn`
//!     "1 answer offer accept",        #   `<player> answer <kind> …`: an answer to the
//!     "1 answer level-up first",      #   player's open decision, of its kind
//!     "1 battle attack 3 41",         #   `<player> battle <command>`: a command in its battle,
//!     "system battle wait",           #   the battle replays' text without the side letter;
//!                                     #   `system battle …`: the unowned side's
//!     "0 army split 1:0 1:3 5",       #   `<player> army …`, `formation …`, `tactics …`,
//!     "0 artifact 1:b0 1:w2",         #   `artifact …`, `assemble …`, `disassemble …`: a
//! ]                                   #   management command
//! checkpoints = ["0x…"]               # state hash after every `checkpoint_every` commands
//!
//! [replay]
//! format = 7
//! engine = "0.1.0"
//! seed = "1"                          # u64 as a decimal string
//! difficulty = 1                      # the game's (GameSetup::difficulty), 1 the import's
//! packs = [["base", "1.0.0"], ["ab", "1.0.0"], ["sod", "1.0.0"]]
//! profile = "base:faithful"
//! ruleset_hash = "0x…"                # the core ruleset's content hash
//! adventure_ruleset_hash = "0x…"      # the adventure part's
//! rng = "xoshiro256starstar+splitmix64"
//! rng_version = 1
//! hash = "fnv1a64-le"
//! hash_version = 1
//! game_state_hash_version = 3
//! adventure_state_hash_version = 5
//! checkpoint_every = 25
//!
//! [map]                               # how the replay names its map (ADV-Q2)
//! path = "fixtures/maps/e08-duel-36.h3m"  # relative, resolved against the tool's root
//! hash = "0x…"                        # the file's content hash
//! import_version = 3                  # h3_map::MAP_IMPORT_VERSION
//!
//! [result]
//! day = 29                            # the day after the last command
//! final_hash = "0x…"
//! ```

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub mod bench;
pub mod cli;

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use h3_adventure::{AdventureCommand, Pos};
use h3_content::Formation;
use h3_contracts::{BattleSide, HeroId, PlayerId};
use h3_game::{
    Actor, Answer, ArenaChoice, ArmyCommand, ArmyRef, ArmySlot, ArtifactPlace, ChestChoice,
    Consent, GameCommand, GameRuleset, GameSetup, Manage, Position, SchoolChoice, SkillOffer,
    UniversityAnswer,
};
use h3_sim::adventure::replay::{
    MapRef, RecordError, RecordOptions, Replay, ReplayError, ReplayHeader, map_content_hash,
};
use serde::{Deserialize, Serialize};

/// The packs an adventure game loads by default: the original three, RoE to SoD.
pub const DEFAULT_PACKS: [&str; 3] = ["base", "ab", "sod"];

/// The default profile.
pub const DEFAULT_PROFILE: &str = "base:faithful";

/// Errors of the adventure commands.
#[derive(Debug)]
pub enum AdventureCliError {
    /// File system error.
    Io(PathBuf, std::io::Error),
    /// A pack, its adventure file or the profile does not compile.
    Pack(PathBuf, String),
    /// The map file does not read or import.
    Map(PathBuf, String),
    /// The replay file is malformed.
    Format(String),
    /// The replay does not verify.
    Replay(ReplayError),
    /// The setup does not start a game.
    Setup(h3_game::GameSetupError),
    /// A game could not be recorded.
    Record(RecordError),
}

impl fmt::Display for AdventureCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(p, e) => write!(f, "{}: {e}", p.display()),
            Self::Pack(p, e) | Self::Map(p, e) => write!(f, "{}: {e}", p.display()),
            Self::Format(e) => write!(f, "replay format: {e}"),
            Self::Replay(e) => write!(f, "{e}"),
            Self::Setup(e) => write!(f, "setup: {e}"),
            Self::Record(e) => write!(f, "record: {e}"),
        }
    }
}

impl std::error::Error for AdventureCliError {}

type Result<T> = std::result::Result<T, AdventureCliError>;

fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| AdventureCliError::Io(path.to_path_buf(), e))
}

fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| AdventureCliError::Io(path.to_path_buf(), e))
}

/// Loads `<packs_dir>/<id>/core.toml` and, where they exist, `<id>/adventure.toml` and
/// `<id>/battle.toml` for each id and compiles them with `profile` into the game's rules (the
/// battle part since E08 T19a). Ids come from replay files, so they must be valid pack ids (no
/// paths).
///
/// # Errors
///
/// An invalid id, an unreadable file, or the packs' compile error.
pub fn load_rules(packs_dir: &Path, ids: &[String], profile: &str) -> Result<GameRuleset> {
    let mut texts = Vec::new();
    for id in ids {
        h3_content::ContentKey::new(id, "pack")
            .map_err(|_| AdventureCliError::Format(format!("invalid pack id `{id}`")))?;
        let dir = packs_dir.join(id);
        let optional = |name: &str| {
            let path = dir.join(name);
            if path.is_file() {
                read_text(&path)
            } else {
                Ok(String::new())
            }
        };
        texts.push((
            read_text(&dir.join("core.toml"))?,
            optional("adventure.toml")?,
            optional("battle.toml")?,
        ));
    }
    let adventure: Vec<(&str, &str)> = texts
        .iter()
        .map(|(p, a, _)| (p.as_str(), a.as_str()))
        .collect();
    let battle: Vec<(&str, &str)> = texts
        .iter()
        .map(|(p, _, b)| (p.as_str(), b.as_str()))
        .collect();
    let pack_error = |e: String| AdventureCliError::Pack(packs_dir.to_path_buf(), e);
    let (core, part) =
        h3_pack_adventure::compile(&adventure, profile).map_err(|e| pack_error(e.to_string()))?;
    let battle =
        h3_pack_battle::compile(&battle, profile).map_err(|e| pack_error(e.to_string()))?;
    GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle))
        .map_err(|e| pack_error(e.to_string()))
}

/// Checks that a map path from the command line or a replay file is relative and stays below the
/// directory it is resolved against (a replay file is untrusted input).
fn check_map_path(path: &str) -> Result<()> {
    let ok = !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if ok {
        Ok(())
    } else {
        Err(AdventureCliError::Format(format!(
            "map path `{path}` must be relative, without `..`"
        )))
    }
}

/// A map file below `root`: its bytes and how a replay names it. The path must be relative and
/// stay below `root` once links are resolved, and the file within `h3-map`'s size limit, checked
/// before it is read (a replay file is untrusted input).
///
/// # Errors
///
/// A path that is not relative or leaves `root`, an unreadable or too large file.
pub fn read_map(root: &Path, path: &str) -> Result<(Vec<u8>, MapRef)> {
    check_map_path(path)?;
    let joined = root.join(path);
    let canonical = |p: &Path| {
        p.canonicalize()
            .map_err(|e| AdventureCliError::Io(p.to_path_buf(), e))
    };
    let file = canonical(&joined)?;
    if !file.starts_with(canonical(root)?) {
        return Err(AdventureCliError::Format(format!(
            "map path `{path}` leads outside {}",
            root.display()
        )));
    }
    let max = h3_map::Limits::default().max_file_bytes;
    let size = std::fs::metadata(&file)
        .map_err(|e| AdventureCliError::Io(file.clone(), e))?
        .len();
    if usize::try_from(size).map_or(true, |size| size > max) {
        return Err(AdventureCliError::Map(
            joined,
            format!("{size} bytes, over the limit of {max}"),
        ));
    }
    let bytes = read_bytes(&file)?;
    let map = MapRef {
        path: path.to_owned(),
        content_hash: map_content_hash(&bytes),
        import_version: h3_map::MAP_IMPORT_VERSION,
    };
    Ok((bytes, map))
}

/// Reads a map file's bytes ([`read_map`]) and imports them with `rules`.
fn import_map(root: &Path, path: &str, bytes: &[u8], rules: &GameRuleset) -> Result<GameSetup> {
    let map_error = |e: String| AdventureCliError::Map(root.join(path), e);
    let raw =
        h3_map::read(bytes, &h3_map::Limits::default()).map_err(|e| map_error(e.to_string()))?;
    h3_map::import(&raw, rules).map_err(|e| map_error(e.to_string()))
}

/// Reads the map at `path` (relative to `root`, [`read_map`]) and imports it with `rules`: how a
/// replay names it, and the game's setup.
///
/// # Errors
///
/// [`read_map`]'s errors, or the map's read or import error.
pub fn load_map(root: &Path, path: &str, rules: &GameRuleset) -> Result<(MapRef, GameSetup)> {
    let (bytes, map) = read_map(root, path)?;
    let setup = import_map(root, path, &bytes, rules)?;
    Ok((map, setup))
}

fn hex64(v: u64) -> String {
    format!("{v:#018x}")
}

fn parse_hex64(s: &str) -> Result<u64> {
    s.strip_prefix("0x")
        .and_then(|h| u64::from_str_radix(h, 16).ok())
        .ok_or_else(|| AdventureCliError::Format(format!("bad hash `{s}`")))
}

/// A command as a replay file writes it (format v7, the module's docs): its actor, then the
/// command; a battle command (E08 T19b1) as the battle replays write it ([`crate::format_command`])
/// without the side, which is the actor's in its battle; a management command (E08 T51a) by its
/// own words ([`format_manage`]); an answer (E08 T63a) by its kind and words ([`format_answer`]).
pub fn format_command(actor: Actor, command: &GameCommand) -> String {
    let player = match actor {
        Actor::Player(player) => player.raw().to_string(),
        // Format v4 (E08 T20b2): the system actor's commands, a monster's side's.
        Actor::System => "system".to_owned(),
    };
    match command {
        GameCommand::Battle(command) => {
            let text = crate::format_command(BattleSide::Attacker, *command);
            let (_, text) = text.split_once(' ').unwrap_or_default();
            format!("{player} battle {text}")
        }
        GameCommand::EndTurn => format!("{player} end-turn"),
        // E08 T14b: an answer to the player's open decision (format v2; typed since v7, T63a).
        GameCommand::Answer(answer) => format!("{player} answer {}", format_answer(*answer)),
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => {
            let mut text = format!("{player} move {}", hero.raw());
            for pos in path {
                text.push_str(&format!(" {},{},{}", pos.x, pos.y, pos.level));
            }
            text
        }
        GameCommand::Manage(command) => format!("{player} {}", format_manage(command)),
    }
}

/// An answer's words (format v7, E08 T63a): its kind, then the kind's answer.
fn format_answer(answer: Answer) -> String {
    let words = match answer {
        Answer::Offer(Consent::Accept) => "offer accept",
        Answer::Offer(Consent::Decline) => "offer decline",
        Answer::LevelUp(SkillOffer::First) => "level-up first",
        Answer::LevelUp(SkillOffer::Second) => "level-up second",
        Answer::Chest(ChestChoice::Gold) => "chest gold",
        Answer::Chest(ChestChoice::Experience) => "chest experience",
        Answer::Arena(ArenaChoice::Attack) => "arena attack",
        Answer::Arena(ArenaChoice::Defense) => "arena defense",
        Answer::School(SchoolChoice::First) => "school first",
        Answer::School(SchoolChoice::Second) => "school second",
        Answer::School(SchoolChoice::Leave) => "school leave",
        Answer::University(UniversityAnswer::Buy(slot)) => {
            return format!("university buy {slot}");
        }
        Answer::University(UniversityAnswer::Leave) => "university leave",
        Answer::Close => "close",
    };
    words.to_owned()
}

/// Parses the words of an answer after `answer` (format v7, [`format_answer`]); none for others.
fn parse_answer<'w>(mut words: impl Iterator<Item = &'w str>) -> Option<Answer> {
    let answer = match (words.next()?, words.next()) {
        ("offer", Some("accept")) => Answer::Offer(Consent::Accept),
        ("offer", Some("decline")) => Answer::Offer(Consent::Decline),
        ("level-up", Some("first")) => Answer::LevelUp(SkillOffer::First),
        ("level-up", Some("second")) => Answer::LevelUp(SkillOffer::Second),
        ("chest", Some("gold")) => Answer::Chest(ChestChoice::Gold),
        ("chest", Some("experience")) => Answer::Chest(ChestChoice::Experience),
        ("arena", Some("attack")) => Answer::Arena(ArenaChoice::Attack),
        ("arena", Some("defense")) => Answer::Arena(ArenaChoice::Defense),
        ("school", Some("first")) => Answer::School(SchoolChoice::First),
        ("school", Some("second")) => Answer::School(SchoolChoice::Second),
        ("school", Some("leave")) => Answer::School(SchoolChoice::Leave),
        ("university", Some("buy")) => {
            Answer::University(UniversityAnswer::Buy(words.next()?.parse().ok()?))
        }
        ("university", Some("leave")) => Answer::University(UniversityAnswer::Leave),
        ("close", None) => return Some(Answer::Close),
        _ => return None,
    };
    words.next().is_none().then_some(answer)
}

/// An army slot as `<army>:<slot>`: the army a hero's id, or `g` for the group.
fn format_slot(at: ArmySlot) -> String {
    match at.army {
        ArmyRef::Hero(hero) => format!("{}:{}", hero.raw(), at.slot),
        ArmyRef::Group => format!("g:{}", at.slot),
    }
}

/// A hero's artifact place as `<hero>:w<position>` or `<hero>:b<index>`.
fn format_place((hero, place): (HeroId, ArtifactPlace)) -> String {
    match place {
        ArtifactPlace::Worn(at) => format!("{}:w{}", hero.raw(), at.0),
        ArtifactPlace::Backpack(index) => format!("{}:b{index}", hero.raw()),
    }
}

/// A management command's words (format v6, E08 T51a), without its actor.
fn format_manage(command: &Manage) -> String {
    match *command {
        Manage::Army(ArmyCommand::Move { from, to }) => {
            format!("army move {} {}", format_slot(from), format_slot(to))
        }
        Manage::Army(ArmyCommand::Split { from, to, count }) => {
            format!(
                "army split {} {} {count}",
                format_slot(from),
                format_slot(to)
            )
        }
        Manage::Army(ArmyCommand::Dismiss { at }) => format!("army dismiss {}", format_slot(at)),
        Manage::Formation { hero, formation } => {
            let word = match formation {
                Formation::Loose => "loose",
                Formation::Tight => "tight",
            };
            format!("formation {} {word}", hero.raw())
        }
        Manage::Tactics { hero, on } => {
            format!("tactics {} {}", hero.raw(), if on { "on" } else { "off" })
        }
        Manage::Artifact { from, to } => {
            format!("artifact {} {}", format_place(from), format_place(to))
        }
        Manage::Assemble { hero, at } => format!("assemble {} {}", hero.raw(), at.0),
        Manage::Disassemble { hero, at } => format!("disassemble {} {}", hero.raw(), at.0),
    }
}

/// Parses an army slot written by [`format_slot`].
fn parse_slot(word: &str) -> Option<ArmySlot> {
    let (army, slot) = word.split_once(':')?;
    let army = match army {
        "g" => ArmyRef::Group,
        id => ArmyRef::Hero(HeroId::new(id.parse().ok()?)),
    };
    Some(ArmySlot {
        army,
        slot: slot.parse().ok()?,
    })
}

/// Parses a place written by [`format_place`].
fn parse_place(word: &str) -> Option<(HeroId, ArtifactPlace)> {
    let (hero, place) = word.split_once(':')?;
    let hero = HeroId::new(hero.parse().ok()?);
    let place = match place.split_at_checked(1)? {
        ("w", at) => ArtifactPlace::Worn(Position(at.parse().ok()?)),
        ("b", index) => ArtifactPlace::Backpack(index.parse().ok()?),
        _ => return None,
    };
    Some((hero, place))
}

/// Parses a management command's words written by [`format_manage`], its kind's word first.
fn parse_manage<'a>(kind: &str, words: &mut impl Iterator<Item = &'a str>) -> Option<Manage> {
    let mut next = || words.next();
    let hero = |word: Option<&str>| word?.parse().ok().map(HeroId::new);
    let command = match kind {
        "army" => {
            let army = match next()? {
                "move" => ArmyCommand::Move {
                    from: parse_slot(next()?)?,
                    to: parse_slot(next()?)?,
                },
                "split" => ArmyCommand::Split {
                    from: parse_slot(next()?)?,
                    to: parse_slot(next()?)?,
                    count: next()?.parse().ok()?,
                },
                "dismiss" => ArmyCommand::Dismiss {
                    at: parse_slot(next()?)?,
                },
                _ => return None,
            };
            Manage::Army(army)
        }
        "formation" => Manage::Formation {
            hero: hero(next())?,
            formation: match next()? {
                "loose" => Formation::Loose,
                "tight" => Formation::Tight,
                _ => return None,
            },
        },
        "tactics" => Manage::Tactics {
            hero: hero(next())?,
            on: match next()? {
                "on" => true,
                "off" => false,
                _ => return None,
            },
        },
        "artifact" => Manage::Artifact {
            from: parse_place(next()?)?,
            to: parse_place(next()?)?,
        },
        "assemble" => Manage::Assemble {
            hero: hero(next())?,
            at: Position(next()?.parse().ok()?),
        },
        _ => Manage::Disassemble {
            hero: hero(next())?,
            at: Position(next()?.parse().ok()?),
        },
    };
    Some(command)
}

/// Parses a command written by [`format_command`].
///
/// # Errors
///
/// [`AdventureCliError::Format`] naming the text.
pub fn parse_command(text: &str) -> Result<(Actor, GameCommand)> {
    let bad = || AdventureCliError::Format(format!("bad command `{text}`"));
    let mut words = text.split_whitespace();
    let actor = match words.next() {
        Some("system") => Actor::System,
        word => word
            .and_then(|w| w.parse().ok())
            .map(|p| Actor::Player(PlayerId::new(p)))
            .ok_or_else(bad)?,
    };
    let command = match words.next() {
        Some("end-turn") => GameCommand::EndTurn,
        Some("answer") => GameCommand::Answer(parse_answer(words.by_ref()).ok_or_else(bad)?),
        Some("battle") => {
            let rest: Vec<&str> = words.by_ref().collect();
            let (_, command) =
                crate::parse_command(&format!("A {}", rest.join(" "))).map_err(|_| bad())?;
            GameCommand::Battle(command)
        }
        Some("move") => {
            let hero = words
                .next()
                .and_then(|w| w.parse().ok())
                .map(HeroId::new)
                .ok_or_else(bad)?;
            let path = words
                .by_ref()
                .map(|w| {
                    let mut n = w.split(',');
                    let (x, y, level) = (n.next()?, n.next()?, n.next()?);
                    if n.next().is_some() {
                        return None;
                    }
                    Some(Pos::new(
                        x.parse().ok()?,
                        y.parse().ok()?,
                        level.parse().ok()?,
                    ))
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(bad)?;
            GameCommand::Adventure(AdventureCommand::MoveHero { hero, path })
        }
        // Format v6 (E08 T51a): the management commands.
        Some(
            kind @ ("army" | "formation" | "tactics" | "artifact" | "assemble" | "disassemble"),
        ) => GameCommand::Manage(parse_manage(kind, &mut words).ok_or_else(bad)?),
        _ => return Err(bad()),
    };
    if words.next().is_some() {
        return Err(bad());
    }
    Ok((actor, command))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayFile {
    commands: Vec<String>,
    checkpoints: Vec<String>,
    replay: Meta,
    map: MapEntry,
    result: ResultEntry,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Meta {
    format: u32,
    engine: String,
    seed: String,
    /// Since format v2 ([`DIFFICULTY_SINCE`]); a v1 file has none.
    #[serde(default)]
    difficulty: Option<u8>,
    packs: Vec<[String; 2]>,
    profile: String,
    ruleset_hash: String,
    adventure_ruleset_hash: String,
    rng: String,
    rng_version: u32,
    hash: String,
    hash_version: u32,
    game_state_hash_version: u32,
    adventure_state_hash_version: u32,
    checkpoint_every: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapEntry {
    path: String,
    hash: String,
    import_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultEntry {
    day: u32,
    final_hash: String,
}

/// The first format version whose files name the game's difficulty: an earlier file has none and
/// reads as the import's (it is refused by its version), a later one without it is malformed.
const DIFFICULTY_SINCE: u32 = 2;

/// The replay as TOML (format v4).
///
/// # Errors
///
/// The serializer's error (none for a replay this tool recorded).
pub fn replay_to_toml(replay: &Replay) -> Result<String> {
    let h = &replay.header;
    let file = ReplayFile {
        commands: replay
            .commands
            .iter()
            .map(|(p, c)| format_command(*p, c))
            .collect(),
        checkpoints: replay.checkpoints.iter().map(|h| hex64(*h)).collect(),
        replay: Meta {
            format: h.format_version,
            engine: h.engine_version.clone(),
            seed: replay.seed.to_string(),
            difficulty: Some(replay.difficulty),
            packs: h
                .packs
                .iter()
                .map(|(id, v)| [id.clone(), v.clone()])
                .collect(),
            profile: h.profile.clone(),
            ruleset_hash: hex64(h.ruleset_hash),
            adventure_ruleset_hash: hex64(h.adventure_ruleset_hash),
            rng: h.rng_algorithm.clone(),
            rng_version: h.rng_version,
            hash: h.hash_algorithm.clone(),
            hash_version: h.hash_version,
            game_state_hash_version: h.game_state_hash_version,
            adventure_state_hash_version: h.adventure_state_hash_version,
            checkpoint_every: replay.checkpoint_every,
        },
        map: MapEntry {
            path: replay.map.path.clone(),
            hash: hex64(replay.map.content_hash),
            import_version: replay.map.import_version,
        },
        result: ResultEntry {
            day: replay.final_day,
            final_hash: hex64(replay.final_hash),
        },
    };
    toml::to_string_pretty(&file).map_err(|e| AdventureCliError::Format(e.to_string()))
}

/// Parses a replay file (format v4; an earlier file without battle commands parses, and
/// [`ReplayHeader::check`] refuses it).
/// Its rules and map are loaded by [`verify_file`].
///
/// # Errors
///
/// [`AdventureCliError::Format`]: malformed TOML, an unknown key, a bad number, hash or command,
/// no difficulty in a file of format v2 or later.
pub fn parse_replay(text: &str) -> Result<Replay> {
    let f: ReplayFile =
        toml::from_str(text).map_err(|e| AdventureCliError::Format(e.to_string()))?;
    let m = f.replay;
    let difficulty = match m.difficulty {
        Some(difficulty) => difficulty,
        None if m.format < DIFFICULTY_SINCE => h3_game::DEFAULT_DIFFICULTY,
        None => {
            return Err(AdventureCliError::Format(format!(
                "format {} needs a difficulty",
                m.format
            )));
        }
    };
    Ok(Replay {
        header: ReplayHeader {
            format_version: m.format,
            engine_version: m.engine,
            packs: m.packs.into_iter().map(|[id, v]| (id, v)).collect(),
            profile: m.profile,
            ruleset_hash: parse_hex64(&m.ruleset_hash)?,
            adventure_ruleset_hash: parse_hex64(&m.adventure_ruleset_hash)?,
            rng_algorithm: m.rng,
            rng_version: m.rng_version,
            hash_algorithm: m.hash,
            hash_version: m.hash_version,
            game_state_hash_version: m.game_state_hash_version,
            adventure_state_hash_version: m.adventure_state_hash_version,
        },
        map: MapRef {
            path: f.map.path,
            content_hash: parse_hex64(&f.map.hash)?,
            import_version: f.map.import_version,
        },
        seed: m
            .seed
            .parse()
            .map_err(|_| AdventureCliError::Format(format!("bad seed `{}`", m.seed)))?,
        difficulty,
        commands: f
            .commands
            .iter()
            .map(|c| parse_command(c))
            .collect::<Result<_>>()?,
        checkpoint_every: m.checkpoint_every,
        checkpoints: f
            .checkpoints
            .iter()
            .map(|h| parse_hex64(h))
            .collect::<Result<_>>()?,
        final_day: f.result.day,
        final_hash: parse_hex64(&f.result.final_hash)?,
    })
}

/// Result of verifying a replay file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    /// The map's path.
    pub map: String,
    /// Number of commands.
    pub commands: usize,
    /// The final day.
    pub day: u32,
    /// Final state hash.
    pub final_hash: u64,
}

/// Loads a replay file, its packs from `packs_dir` and its map below `root`, and verifies it at
/// the replay's difficulty.
///
/// # Errors
///
/// The first [`AdventureCliError`].
pub fn verify_file(path: &Path, packs_dir: &Path, root: &Path) -> Result<Verified> {
    let replay = parse_replay(&read_text(path)?)?;
    let ids: Vec<String> = replay
        .header
        .packs
        .iter()
        .map(|(id, _)| id.clone())
        .collect();
    let rules = load_rules(packs_dir, &ids, &replay.header.profile)?;
    // The header first: a replay of other rules is refused before its map is read.
    replay
        .header
        .check(&rules)
        .map_err(AdventureCliError::Replay)?;
    // The map by its hash before its bytes are parsed: an untrusted file is read only when it is
    // the one the replay names.
    let (bytes, map) = read_map(root, &replay.map.path)?;
    if map.content_hash != replay.map.content_hash {
        return Err(AdventureCliError::Replay(ReplayError::Incompatible(
            "map.hash",
        )));
    }
    let setup = import_map(root, &replay.map.path, &bytes, &rules)?;
    replay
        .verify(&rules, &map, &setup)
        .map_err(AdventureCliError::Replay)?;
    Ok(Verified {
        map: replay.map.path,
        commands: replay.commands.len(),
        day: replay.final_day,
        final_hash: replay.final_hash,
    })
}

/// What [`run_random`] plays.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// The map's path, relative to the root.
    pub map: String,
    /// The profile key.
    pub profile: String,
    /// The game's difficulty (setup input, checked by the game's build: an unknown one is the
    /// setup's error).
    pub difficulty: u8,
    /// The game and the agent ([`RecordOptions`]).
    pub record: RecordOptions,
}

/// Plays a game of uniformly random legal agents on a map at the options' difficulty and returns
/// the replay and its TOML.
///
/// # Errors
///
/// The first [`AdventureCliError`].
pub fn run_random(packs_dir: &Path, root: &Path, options: &RunOptions) -> Result<(Replay, String)> {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(packs_dir, &ids, &options.profile)?;
    let (map, mut setup) = load_map(root, &options.map, &rules)?;
    setup.difficulty = options.difficulty;
    let replay = Replay::record_random(&rules, Arc::new(setup), map, options.record)
        .map_err(AdventureCliError::Record)?;
    let text = replay_to_toml(&replay)?;
    Ok((replay, text))
}

/// What [`run_script`] plays (E08 T52).
#[derive(Debug, Clone)]
pub struct ScriptOptions {
    /// The map's path, relative to the root.
    pub map: String,
    /// The profile key.
    pub profile: String,
    /// The game's difficulty (setup input, checked by the game's build).
    pub difficulty: u8,
    /// The game's seed.
    pub seed: u64,
    /// Checkpoint interval (0 = none).
    pub checkpoint_every: u32,
}

/// Reads a script (E08 T52): a replay file's commands (a path ending in `.toml`), or a text file
/// of commands, one per line as a replay writes them ([`format_command`]), blank lines and lines
/// starting with `#` skipped.
///
/// # Errors
///
/// The file's, or [`AdventureCliError::Format`] naming a malformed line or replay.
pub fn read_script(path: &Path) -> Result<Vec<(Actor, GameCommand)>> {
    let text = read_text(path)?;
    if path.extension().is_some_and(|e| e == "toml") {
        return Ok(parse_replay(&text)?.commands);
    }
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(parse_command)
        .collect()
}

/// Plays `commands` on a map at the options' difficulty and returns the replay and its TOML (E08
/// T52: the scripted golden replays, of what the random agent does not play; a replay's own
/// commands re-record it when a hash moves).
///
/// # Errors
///
/// The first [`AdventureCliError`]: the rules', the map's, the setup's or a rejected command's.
pub fn run_script(
    packs_dir: &Path,
    root: &Path,
    options: &ScriptOptions,
    commands: Vec<(Actor, GameCommand)>,
) -> Result<(Replay, String)> {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(packs_dir, &ids, &options.profile)?;
    let (map, mut setup) = load_map(root, &options.map, &rules)?;
    setup.difficulty = options.difficulty;
    let replay = Replay::record_commands(
        &rules,
        (&setup, map),
        options.seed,
        commands,
        options.checkpoint_every,
    )
    .map_err(AdventureCliError::Record)?;
    let text = replay_to_toml(&replay)?;
    Ok((replay, text))
}

#[cfg(test)]
mod tests;
