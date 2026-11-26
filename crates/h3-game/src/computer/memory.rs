//! The computer player's memory (ADR-0030 point 5; CPU-TRN-005; survey §1.6, §4.6): what the
//! policy keeps between its calls, outside the game state, one per computer player. A plain value
//! with a canonical encoding ([`Memory::encode`], [`Memory::decode`]) and a hash
//! ([`Memory::hash`]): the host keeps it beside the journal and saves it with a saved game; a
//! replay rebuilds it by running the policy at the journal's computer commands.
//!
//! Since E31 T2a it holds the turn's frame: the day of the turn it plays, the pass, the turn's
//! flag, the hero a garrison just sent out and the heroes done (the original's sleep, +0x11C,
//! which the computer's turn start clears for every hero and garrison hero). Since T2d the
//! records of its turn start ([`Record`]: the reserve, the weights' inputs, the mean weight, the
//! heroes' values per unit, the threats), kept until its next turn: its answers in the other
//! players' turns read the last start's (survey §4.6). The heroes' destinations, the only fields
//! the original carries into the next turn (+0x35, +0x41), come with the planning (T2i).

use alloc::vec::Vec;

use h3_contracts::{HeroId, TownId};
use h3_kernel::hash::{CanonicalHash, StateHasher, hash_of};

use super::spells::SpellGains;
use super::start::{HeroValues, Record};

/// Version of [`Memory`]'s canonical encoding (`docs/08-work-process.md`, "Versions"; owned by
/// the ai lane): bumped with any change to what it holds or how it is written. v1 (E31 T2a): the
/// turn's frame; v2 (T2d): its start's records; v3 (T2e): the heroes' spell gains beside their
/// experience weights; v4 (T2f): the artifacts' mean after the mean weight.
pub const MEMORY_ENCODING_VERSION: u32 = 4;

/// A computer player's memory (ADR-0030 point 5; CPU-TRN-005): empty at the player's first turn
/// in a game (the original's records survive from an earlier game, a quirk candidate not
/// reproduced, §4.6), then the frame of the turn it plays. [`super::choose`] reads and writes it;
/// a host keeps one per computer player and hands the same one to every call for that player.
// rule: CPU-TRN-005
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    turn: Option<Turn>,
}

/// Why [`Memory::decode`] refused its bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    /// Written in another encoding version than [`MEMORY_ENCODING_VERSION`]: the version read.
    Version(u32),
    /// The bytes end early, hold a value outside its range (a tag, a pass, a flag, the done
    /// heroes not in ascending order, a rate's row outside 1 to 10, a record's heroes or towns
    /// not in ascending order, a threat or a town's count of 0) or go on after the memory.
    Malformed,
}

/// The frame of the turn the memory's player plays (`ai_player_turn` 0x525E80; CPU-TRN-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Turn {
    /// The day of the turn: a call on another day starts a new turn.
    pub(super) day: u32,
    /// The pass the turn is in.
    pub(super) pass: Pass,
    /// The turn's flag F (survey §1.1 step 4): the difficulty above Easy and a town owned, at the
    /// start and again after the middle; a reveal clears it (T2j).
    pub(super) flag: bool,
    /// The hero a garrison sent out (`ai_next_hero` 0x526A90's last branch), which plays next
    /// without the alone flag.
    pub(super) next: Option<HeroId>,
    /// The heroes done this turn (the original's sleep, +0x11C), in ascending order.
    pub(super) asleep: Vec<HeroId>,
    /// The records of the turn's start (CPU-STA-001).
    pub(super) record: Record,
}

/// A turn's pass (survey §1.1): the first pass of heroes, then after the middle the second, then
/// the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pass {
    /// The first pass of heroes (step 5).
    First,
    /// The second pass, after the middle (step 7).
    Second,
    /// The end (step 8): the turn ends.
    Over,
}

impl Pass {
    /// The pass's code in the encoding.
    const fn code(self) -> u8 {
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Over => 2,
        }
    }

    /// The pass of `code`, if it is one's.
    const fn of_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::First),
            1 => Some(Self::Second),
            2 => Some(Self::Over),
            _ => None,
        }
    }
}

impl Turn {
    /// Whether `hero` is done this turn.
    pub(super) fn is_asleep(&self, hero: HeroId) -> bool {
        self.asleep.binary_search(&hero).is_ok()
    }

    /// Marks `hero` done this turn (the original's +0x11C set), once.
    pub(super) fn sleep(&mut self, hero: HeroId) {
        let _ = self
            .asleep
            .binary_search(&hero)
            .map_err(|at| self.asleep.insert(at, hero));
    }
}

impl Memory {
    /// An empty memory: no turn played yet.
    pub const fn new() -> Self {
        Self { turn: None }
    }

    /// The frame of the turn of `day`, started by `start` when the memory holds another day's or
    /// none (the computer's turn start, survey §1.5: every done mark cleared).
    pub(super) fn turn_of(&mut self, day: u32, start: impl FnOnce() -> Turn) -> &mut Turn {
        let turn = self.turn.take().filter(|turn| turn.day == day);
        self.turn.insert(turn.unwrap_or_else(start))
    }

    /// The canonical encoding (v4, [`MEMORY_ENCODING_VERSION`]), little-endian: the version
    /// (u32); 0 without a turn, else 1, the turn's day (u32), its pass (u8: 0 the first, 1 the
    /// second, 2 the end), its flag (u8: 0 or 1), the hero sent out (u8 0, or 1 and the hero's
    /// id as u16), the number of heroes done (u32) and each one's id (u16) in ascending order;
    /// then the start's record: the Eye value (i32), the number of resources (u32) and per
    /// resource the reserve, then the means, then the demand (i32 each), the rate's row (u8, 1 to
    /// 10), the mean weight (i32), the artifacts' mean (its f32 bits, u32), and three lists, each its length (u32) then its entries in
    /// ascending id order: the heroes' values per unit (the hero's id u16, the experience weight's
    /// f32 bits u32, then the spell gains as i32: one Spell Power, one round, a Knowledge point,
    /// the mana doubled, the mana refilled),
    /// the threats (the hero's id u16, the threat i32, not 0) and the towns' counts (the town's
    /// id u16, the count u8, not 0).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16);
        out.extend_from_slice(&MEMORY_ENCODING_VERSION.to_le_bytes());
        match &self.turn {
            None => out.push(0),
            Some(turn) => {
                out.push(1);
                out.extend_from_slice(&turn.day.to_le_bytes());
                out.push(turn.pass.code());
                out.push(u8::from(turn.flag));
                match turn.next {
                    None => out.push(0),
                    Some(hero) => {
                        out.push(1);
                        out.extend_from_slice(&hero.raw().to_le_bytes());
                    }
                }
                write_count(&mut out, turn.asleep.len());
                for hero in &turn.asleep {
                    out.extend_from_slice(&hero.raw().to_le_bytes());
                }
                encode_record(&mut out, &turn.record);
            }
        }
        out
    }

    /// The memory [`Self::encode`] wrote into `bytes`: the only memory whose encoding they are.
    ///
    /// # Errors
    ///
    /// [`MemoryError::Version`] for another version's bytes, [`MemoryError::Malformed`] for bytes
    /// that are no memory's encoding.
    pub fn decode(bytes: &[u8]) -> Result<Self, MemoryError> {
        let mut reader = Reader(bytes);
        let version = u32::from_le_bytes(reader.take().ok_or(MemoryError::Malformed)?);
        if version != MEMORY_ENCODING_VERSION {
            return Err(MemoryError::Version(version));
        }
        let turn = match reader.take::<1>().ok_or(MemoryError::Malformed)? {
            [0] => None,
            [1] => Some(reader.turn().ok_or(MemoryError::Malformed)?),
            _ => return Err(MemoryError::Malformed),
        };
        if reader.0.is_empty() {
            Ok(Self { turn })
        } else {
            Err(MemoryError::Malformed)
        }
    }

    /// The canonical hash: FNV-1a over [`Self::encode`]'s bytes (`h3_kernel::hash`).
    pub fn hash(&self) -> u64 {
        hash_of(self)
    }
}

impl CanonicalHash for Memory {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bytes(&self.encode());
    }
}

/// The bytes [`Memory::decode`] has not read yet.
struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    /// The next `N` bytes, if there are as many.
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, rest) = self.0.split_first_chunk::<N>()?;
        self.0 = rest;
        Some(*head)
    }

    /// A hero's id.
    fn hero(&mut self) -> Option<HeroId> {
        self.take()
            .map(|bytes| HeroId::new(u16::from_le_bytes(bytes)))
    }

    /// A turn's frame (after its tag 1).
    fn turn(&mut self) -> Option<Turn> {
        let day = u32::from_le_bytes(self.take()?);
        let [pass] = self.take()?;
        let pass = Pass::of_code(pass)?;
        let flag = match self.take()? {
            [0] => false,
            [1] => true,
            _ => return None,
        };
        let next = match self.take()? {
            [0] => None,
            [1] => Some(self.hero()?),
            _ => return None,
        };
        let asleep = self.ascending(|reader| reader.hero().map(|hero| (hero, ())))?;
        let asleep = asleep.into_iter().map(|(hero, ())| hero).collect();
        let record = self.record()?;
        Some(Turn {
            day,
            pass,
            flag,
            next,
            asleep,
            record,
        })
    }

    /// An `i32`.
    fn int(&mut self) -> Option<i32> {
        self.take().map(i32::from_le_bytes)
    }

    /// A list's length, then its entries by `entry`, their keys in strictly ascending order. No
    /// room reserved by the length: a forged one ends at the first missing entry.
    fn ascending<K: Ord + Copy, V>(
        &mut self,
        mut entry: impl FnMut(&mut Self) -> Option<(K, V)>,
    ) -> Option<Vec<(K, V)>> {
        let count = u32::from_le_bytes(self.take()?);
        let mut list: Vec<(K, V)> = Vec::new();
        for _ in 0..count {
            let (key, value) = entry(self)?;
            if list.last().is_some_and(|&(last, _)| last >= key) {
                return None;
            }
            list.push((key, value));
        }
        Some(list)
    }

    /// A start's record ([`encode_record`]).
    fn record(&mut self) -> Option<Record> {
        let eyes = self.int()?;
        let resources = u32::from_le_bytes(self.take()?);
        let mut arrays: [Vec<i32>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for array in &mut arrays {
            for _ in 0..resources {
                array.push(self.int()?);
            }
        }
        let [markets] = self.take()?;
        if !(1..=10).contains(&markets) {
            return None;
        }
        let mean = self.int()?;
        let artifacts = u32::from_le_bytes(self.take()?);
        let heroes = self.ascending(|reader| {
            let hero = reader.hero()?;
            let experience = u32::from_le_bytes(reader.take()?);
            let spells = SpellGains {
                power: reader.int()?,
                rounds: reader.int()?,
                knowledge: reader.int()?,
                doubled: reader.int()?,
                refilled: reader.int()?,
            };
            Some((hero, HeroValues { experience, spells }))
        })?;
        let threats = self.ascending(|reader| {
            let hero = reader.hero()?;
            reader
                .int()
                .filter(|&threat| threat != 0)
                .map(|threat| (hero, threat))
        })?;
        let towns = self.ascending(|reader| {
            let town = TownId::new(u16::from_le_bytes(reader.take()?));
            let [count] = reader.take()?;
            (count != 0).then_some((town, count))
        })?;
        let [reserve, means, demand] = arrays;
        Some(Record {
            eyes,
            reserve,
            means,
            demand,
            markets,
            mean,
            artifacts,
            heroes,
            threats,
            towns,
        })
    }
}

/// A list's length as the encoding writes it (u32).
fn write_count(out: &mut Vec<u8>, len: usize) {
    out.extend_from_slice(&u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes());
}

/// A start's record as [`Memory::encode`] writes it.
fn encode_record(out: &mut Vec<u8>, record: &Record) {
    out.extend_from_slice(&record.eyes.to_le_bytes());
    write_count(out, record.reserve.len());
    for array in [&record.reserve, &record.means, &record.demand] {
        for value in array {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    out.push(record.markets);
    out.extend_from_slice(&record.mean.to_le_bytes());
    out.extend_from_slice(&record.artifacts.to_le_bytes());
    write_count(out, record.heroes.len());
    for (hero, values) in &record.heroes {
        out.extend_from_slice(&hero.raw().to_le_bytes());
        out.extend_from_slice(&values.experience.to_le_bytes());
        let g = values.spells;
        for value in [g.power, g.rounds, g.knowledge, g.doubled, g.refilled] {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    write_count(out, record.threats.len());
    for (hero, threat) in &record.threats {
        out.extend_from_slice(&hero.raw().to_le_bytes());
        out.extend_from_slice(&threat.to_le_bytes());
    }
    write_count(out, record.towns.len());
    for (town, count) in &record.towns {
        out.extend_from_slice(&town.raw().to_le_bytes());
        out.push(*count);
    }
}
