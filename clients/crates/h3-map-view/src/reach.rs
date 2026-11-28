//! The engine's reach in play (E24 M5 T4a, *provisional*; `docs/epics/E24-design.md` §5.26): the
//! facts a live view asks of the hot seat's host while the viewer walks, in place of a record's or
//! a recording's.
//!
//! - **A stand** ([`Stand`]): the selected hero standing between two of the engine's commands, by
//!   its place among the map's heroes and the walks and turns the loop took so far. The engine's
//!   answer for it holds until the next walk or turn, so the loop asks it once per stand.
//! - **The reach** ([`Reach`]): the selected hero's moves today (`GameQuery::Moves`, the engine's
//!   `h3_adventure::Reach`): each tile it reaches with the path there as a click shows it (every
//!   arrow green: a step the points pay today, `timeline::arrows`) and what the path costs (the
//!   cursor's, `Hover::cursor`). Only today's: the original's cursor counts the days of a longer
//!   path and its red arrows show it, which the engine's query does not answer (MCL-Q37): a tile
//!   beyond today's reach shows the arrow and a click there only moves the pointer.
//! - **The host's facts** ([`Hosted`]): the reaches given so far and the walks the engine refused
//!   (a direction key's step into an obstacle: nothing happens, as the original's
//!   `hero_step_allowed` refuses it), so that a loop run again from before them plays them alike.
//!   A refusal holds for its stand: the same walk asked again there (a held key's repeats against
//!   a wall) is refused at once, not asked again.
//! - **A walk asked** ([`WalkAsked`]): a path's walk or a key's step whose facts the loop lacks:
//!   the host plays its `MoveHero` and gives its facts (`Loop::add_walks`) or its refusal
//!   (`Loop::refuse_walk`).
//!
//! Since E24 M5 T4b (*provisional*; design §5.27) the boxes the player answers:
//!
//! - **An answer asked** ([`AnswerAsked`]): a visit's question whose answers are the host's
//!   (`timeline::Question::hosted`: the engine's decision open) closed by one of its buttons
//!   ([`Pick`]); the host plays the session's `Answer` and gives what it shows (`Loop::add_answer`),
//!   kept by the visit's find, the question's place among its visit's and the button.
//! - **A revisit asked** ([`RevisitAsked`]): Space with the selected hero on an object's
//!   entrance; the host plays the engine's `Revisit` and gives the find its visit tells
//!   (`Loop::add_revisit`), kept by the stand (which counts the revisits taken).
//! - **A level-up asked** ([`LevelAsked`]): a level-up's dialog whose offer is the host's
//!   (`timeline::LevelBox::hosted`) closed by its OK with an offer chosen; the host plays it and
//!   gives the levels the engine opens after it (`Loop::add_level`), kept by the hero and the level.
//!
//! Since E24 M5 T4c (*provisional*; design §5.28) the screens' notices a live view shows where a
//! screen of M6 or M7 has no decision ([`Hosted::notices`], `crate::notice`): laid out by the
//! window for the view, not the host's answers.
//!
//! Since E24 M5 T6a (*provisional*; design §5.33) the viewer's battles:
//!
//! - **A battle asked** ([`BattleAsked`]): a battle of the viewer's walk or answer the map hands
//!   over where the record player would (a walk's end as a plain cut, a switch where it shows the
//!   battle whole) whose result the host has not given: the host hosts the battle in the window
//!   (the battle client on the session's seat) and gives its result as the record laid out again
//!   tells it ([`Fought`], `Loop::add_battle`), kept by the battle's number; a switch's battle is
//!   then closed in the same millisecond (`timeline::Action::BattleClosed`), as the record player
//!   closes a record's.

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_protocol::wire::game::{Moves, Pos as WirePos};
use h3_ui::message::MessageBox;

use crate::live::Arrow;
use crate::notice::Notice;
use crate::timeline::{self, Answer, LevelBox};

/// The selected hero standing between two of the engine's commands (E24 M5 T4a): its place among
/// the map's heroes (`Start::heroes`), the engine's walks and the turns' starts the loop took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stand {
    /// The hero's place among the map's heroes.
    pub hero: usize,
    /// The engine's walks taken so far.
    pub walks: usize,
    /// The turns' starts taken so far.
    pub turns: usize,
    /// The host's answers taken so far (E24 M5 T4b: a reach asked under a decision the engine
    /// held open is none, and an answer may change it).
    pub answers: usize,
}

/// The selected hero's reach today, as the engine answers it (E24 M5 T4a).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reach {
    /// The path to each tile it reaches, its arrows in order from the hero's tile.
    pub paths: Vec<Vec<Arrow>>,
    /// The points each path costs, by the tile's column and row on the hero's level.
    pub costs: BTreeMap<(i32, i32), u32>,
}

impl Reach {
    /// The reach of `moves` (the engine's answer), each path's arrows by the walk's table
    /// `table` (`path_arrow_frames`, `timeline::arrows`); none for a tree that does not read (a
    /// malformed answer, a tile off the map's 256 columns or rows).
    #[must_use]
    pub fn of(moves: &Moves, table: &[u8]) -> Option<Self> {
        moves.nodes()?;
        let origin = tile(moves.origin)?;
        let mut reach = Self::default();
        for (to, left) in moves.tiles() {
            // Each tile the tree reaches has its path.
            let path: Vec<Pos> = moves
                .path(to)
                .unwrap_or_default()
                .into_iter()
                .map(tile)
                .collect::<Option<_>>()?;
            reach.paths.push(timeline::arrows(table, origin, &path)?);
            let cost = moves.points.saturating_sub(left);
            reach.costs.insert((i32::from(to.x), i32::from(to.y)), cost);
        }
        Some(reach)
    }
}

/// What the host gave a live view so far (E24 M5 T4a): each stand's reach and the walks the engine
/// refused, by their stands and tiles; since E24 M5 T4b the questions' answers and the level-ups'
/// next levels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hosted {
    /// The reaches given, by their stands.
    pub reaches: Vec<(Stand, Arc<Reach>)>,
    /// The walks refused: each one's stand and tiles.
    pub refused: Vec<(Stand, Vec<Pos>)>,
    /// The answers given (E24 M5 T4b): what each shows, by its question ([`AnswerKey`]).
    pub answers: Vec<(AnswerKey, Answer)>,
    /// The level-ups answered (E24 M5 T4b): the levels the engine opened after each, by its hero
    /// (its place among the map's heroes) and level.
    pub levels: Vec<((usize, u8), Vec<LevelBox>)>,
    /// Space's revisits given (E24 M5 T4b), by their stands: the find each tells (its place among
    /// the loop's finds), none where the engine refused it or it tells nothing.
    pub revisits: Vec<(Stand, Option<usize>)>,
    /// The seed of the client's own picks (E24 M5 T4b, `crate::picks`: a pickup's sound, a
    /// computer turn's music); none: the picks in turn.
    pub picks: Option<u64>,
    /// The notices of the screens without a decision (E24 M5 T4c, design §5.28: the hero screen,
    /// the town screen, the spell book, [`Notice::SCREENS`]), each its box (style 1); one not
    /// given: nothing shown, as before.
    pub notices: Vec<(Notice, Arc<MessageBox>)>,
    /// The battles the host hosted and gave the result of (E24 M5 T6a, design §5.33), by the
    /// record's number of the `GameEvents` whose `BattleStarted` began each: none is asked again.
    pub battles: Vec<u32>,
}

impl Hosted {
    /// The reach given for `stand`.
    #[must_use]
    pub fn reach(&self, stand: Stand) -> Option<&Arc<Reach>> {
        self.reaches
            .iter()
            .find(|(s, _)| *s == stand)
            .map(|(_, r)| r)
    }

    /// Whether the walk along `tiles` from `stand` was refused.
    #[must_use]
    pub fn refuses(&self, stand: Stand, tiles: &[Pos]) -> bool {
        self.refused.iter().any(|(s, p)| *s == stand && p == tiles)
    }

    /// What the answer `asked` shows, given.
    #[must_use]
    pub fn answer(&self, asked: AnswerKey) -> Option<&Answer> {
        self.answers
            .iter()
            .find(|(a, _)| *a == asked)
            .map(|(_, answer)| answer)
    }

    /// The revisit of `stand` given: the find it tells, if any.
    #[must_use]
    pub fn revisit(&self, stand: Stand) -> Option<Option<usize>> {
        self.revisits
            .iter()
            .find(|(s, _)| *s == stand)
            .map(|&(_, told)| told)
    }

    /// Whether the battle the record's `GameEvents` numbered `battle` began was hosted and its
    /// result given (E24 M5 T6a).
    #[must_use]
    pub fn fought(&self, battle: u32) -> bool {
        self.battles.contains(&battle)
    }

    /// The box of `notice` (E24 M5 T4c), given.
    #[must_use]
    pub fn notice(&self, notice: Notice) -> Option<&Arc<MessageBox>> {
        self.notices
            .iter()
            .find(|(n, _)| *n == notice)
            .map(|(_, laid)| laid)
    }

    /// The levels the engine opened after hero `hero`'s level-up to `level`, given.
    #[must_use]
    pub fn leveled(&self, hero: usize, level: u8) -> Option<&[LevelBox]> {
        self.levels
            .iter()
            .find(|(key, _)| *key == (hero, level))
            .map(|(_, next)| next.as_slice())
    }
}

/// A question's button as the engine's answer takes it (E24 M5 T4b): its OK (a choice's first
/// picture), a choice's second picture, its Cancel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    /// The OK, or a choice's first picture.
    Yes,
    /// A choice's second picture.
    Second,
    /// The Cancel.
    No,
}

/// A question answered (E24 M5 T4b): its visit's find (its place among the loop's finds), its
/// place among the questions the visit asks (0 the first; a monster's flight after its offer
/// declined the second) and the button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnswerKey {
    /// The find.
    pub find: usize,
    /// The question's place among its visit's.
    pub nth: u8,
    /// The button.
    pub pick: Pick,
}

/// A question's answer the loop lacks, asked of the host (E24 M5 T4b): the millisecond of the
/// button's click and the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnswerAsked {
    /// The click's millisecond.
    pub at: u32,
    /// The question and the button.
    pub asked: AnswerKey,
}

/// A level-up's offer the loop lacks the next of, asked of the host (E24 M5 T4b): the millisecond
/// of the OK's click, the hero (its place among the map's heroes), the level and the offer chosen
/// (the dialog's left first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelAsked {
    /// The click's millisecond.
    pub at: u32,
    /// The hero.
    pub hero: usize,
    /// The level.
    pub level: u8,
    /// The offer chosen.
    pub offer: usize,
}

/// Space's revisit the loop lacks, asked of the host (E24 M5 T4b): the millisecond of the key and
/// the selected hero's stand (on an object's entrance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevisitAsked {
    /// The key's millisecond.
    pub at: u32,
    /// The stand.
    pub stand: Stand,
}

/// A battle whose result the loop lacks, asked of the host where the map hands it over (E24 M5
/// T6a, design §5.33): the millisecond of the hand-over (a walk's end, or the switch's battle
/// whole), the record's number of the `GameEvents` whose `BattleStarted` began it and whether a
/// switch hands it over (closed by the host in that millisecond, `timeline::Action::BattleClosed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleAsked {
    /// The hand-over's millisecond.
    pub at: u32,
    /// The battle.
    pub battle: u32,
    /// A switch hands it over (else a plain cut).
    pub switch: bool,
}

/// A battle's result as the host gives it (E24 M5 T6a, design §5.33): the visit's fight as the
/// record laid out again after the battle tells it (its winner, what the map shows after it, the
/// experience, the monster it takes off), the answer that began it as it shows after it (its
/// boxes, its flag: a mine's, Pandora's Box's rewards) and the visit's level-ups (the engine's,
/// open: their offers the host's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fought {
    /// The fight.
    pub fight: timeline::Fight,
    /// The answer that began it, its fight left out; none for a walk's.
    pub answer: Option<Answer>,
    /// The visit's level-ups.
    pub levels: Vec<LevelBox>,
}

/// A walk whose facts the loop lacks, asked of the host (E24 M5 T4a): the millisecond it starts
/// at, the stand it starts from (its hero's place among the map's heroes) and the tiles it steps
/// onto (a key's step's one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkAsked {
    /// The walk's millisecond.
    pub at: u32,
    /// The stand it starts from.
    pub stand: Stand,
    /// The tiles, in order, as `MoveHero` takes them.
    pub tiles: Vec<Pos>,
}

impl WalkAsked {
    /// The tiles as the wire's (`MoveHero`'s path).
    #[must_use]
    pub fn path(&self) -> Vec<WirePos> {
        self.tiles
            .iter()
            .map(|p| WirePos {
                x: u16::from(p.x),
                y: u16::from(p.y),
                level: p.z,
            })
            .collect()
    }
}

/// The map's tile of the wire's `at`, none off the map's 256 columns or rows.
fn tile(at: WirePos) -> Option<Pos> {
    Some(Pos {
        x: u8::try_from(at.x).ok()?,
        y: u8::try_from(at.y).ok()?,
        z: at.level,
    })
}

#[cfg(test)]
mod tests;
