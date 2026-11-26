//! The random artifacts' pool (ADV-OBJ-006; survey `E08-rewards.md` §3.1, D-31, D-32): its marks
//! and its picker, the original's `random_artifact_of_level` 0x4C94D0, which the build's draws use
//! (`draws::resolve_rewards`: the random artifacts, the chests, the finds, the banks and the Black
//! Markets) and since E09 T57 the Artifact Merchants' (the town lane's `town/merchants.rs`). In a
//! module of its own since E09 T57 (the release build's codegen units: the reach loops moved by
//! +8 to +10 % with it in `state::draws`, `TRAPS.md`).

use alloc::vec::Vec;

use h3_content::{ArtifactClass, ArtifactType, ArtifactTypeId, Ruleset};
use h3_kernel::{Rng, StateHasher};

/// The classes a random artifact widens to when its own classes have nothing left (the
/// original's level bits 0x1E): every class but Special.
const ANY_CLASS: [ArtifactClass; 4] = [
    ArtifactClass::Treasure,
    ArtifactClass::Minor,
    ArtifactClass::Major,
    ArtifactClass::Relic,
];

/// The random artifacts' pool's marks (ADV-OBJ-006; survey `E08-rewards.md` §3.1, D-31, D-32): a
/// ban and a "given" mark per artifact by index, the original's game +0x4E2B4 and +0x4E224. Since
/// E09 T57 kept in the adventure state after the build (TWN-STK-001, -002; the town epic's design
/// note §108, decision 212), so the Artifact Merchants' draws go on from the marks the build left:
/// `AdventureState::merchants`, the town lane's `town/merchants.rs`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ArtifactMarks {
    /// The banned artifacts, by index.
    banned: Vec<bool>,
    /// The marked ones, by index.
    given: Vec<bool>,
    /// Each class's artifacts by index in id order, by [`ArtifactClass`] in code order: the
    /// core's, derived when the pool is made, so a draw of one class walks its own (E09 T57: the
    /// merchants' seven a month). Not hashed (the ruleset's).
    by_class: [Vec<u16>; ArtifactClass::ALL.len()],
}

impl ArtifactMarks {
    /// Hashes the bans, then the given marks, each as its length and its bits packed 64 a word
    /// from the lowest index (E09 T57: the adventure state hash v30).
    pub(crate) fn hash_into(&self, h: &mut StateHasher) {
        for marks in [&self.banned, &self.given] {
            h.write_len(marks.len());
            for word in marks.chunks(64) {
                let bits = word
                    .iter()
                    .enumerate()
                    .fold(0u64, |bits, (bit, &mark)| bits | (u64::from(mark) << bit));
                h.write_u64(bits);
            }
        }
    }
}

/// The random artifacts' pool (ADV-OBJ-006; survey `E08-rewards.md` §3.1, D-31, D-32): the
/// original's "given" marks, a mark per artifact, starting as the bans and the map's placed
/// artifacts (`artifacts_given_init` 0x4C0AA0); each draw marks the artifact it gives, so no
/// artifact repeats until the pool is used up (`random_artifact_of_level` 0x4C94D0). Since E09 T57
/// its marks outlive the build ([`ArtifactMarks`]: [`Self::into_marks`], [`Self::resume`]).
pub(crate) struct ArtifactPool<'c> {
    core: &'c Ruleset,
    marks: ArtifactMarks,
}

impl<'c> ArtifactPool<'c> {
    /// The pool with the bans marked, then each of `placed`.
    pub(super) fn new(
        core: &'c Ruleset,
        bans: &[ArtifactTypeId],
        placed: &[ArtifactTypeId],
    ) -> Self {
        let among = |list: &[ArtifactTypeId], index| list.iter().any(|a| a.index() == index);
        let banned: Vec<bool> = (0..core.artifacts().len())
            .map(|index| among(bans, index))
            .collect();
        let given = banned
            .iter()
            .enumerate()
            .map(|(index, &ban)| ban || among(placed, index))
            .collect();
        let mut by_class: [Vec<u16>; ArtifactClass::ALL.len()] = Default::default();
        for (artifact, index) in core.artifacts().iter().zip(0..=u16::MAX) {
            by_class[artifact.class as usize].push(index);
        }
        Self {
            core,
            marks: ArtifactMarks {
                banned,
                given,
                by_class,
            },
        }
    }

    /// The pool on `marks`, kept since an earlier pool's [`Self::into_marks`] (E09 T57), of the
    /// same core.
    pub(crate) const fn resume(core: &'c Ruleset, marks: ArtifactMarks) -> Self {
        Self { core, marks }
    }

    /// The pool's marks as its draws left them (E09 T57).
    pub(crate) fn into_marks(self) -> ArtifactMarks {
        self.marks
    }

    /// The artifacts of `classes` by index in id order: one class's its list, several a walk
    /// over every artifact.
    fn of<'a>(&'a self, classes: &'a [ArtifactClass]) -> Of<'a> {
        match classes {
            [class] => Of::One(self.marks.by_class[*class as usize].iter()),
            _ => Of::Walk(self.core.artifacts().iter().zip(0..), classes),
        }
    }

    /// The number of unmarked artifacts of `classes`.
    fn count(&self, classes: &[ArtifactClass]) -> usize {
        let given = &self.marks.given;
        self.of(classes).filter(|&index| !given[index]).count()
    }

    /// The `nth` unmarked artifact of `classes` in id order, by index.
    fn nth(&self, classes: &[ArtifactClass], nth: usize) -> Option<usize> {
        let given = &self.marks.given;
        self.of(classes).filter(|&index| !given[index]).nth(nth)
    }

    /// The marks of `classes`' artifacts restarted from their bans, the other classes' kept (the
    /// original's reset of the level's records, `random_artifact_of_level` 0x4C94D0; E09 T57).
    fn restart(&mut self, classes: &[ArtifactClass]) {
        let indices: Vec<usize> = self.of(classes).collect();
        let ArtifactMarks { banned, given, .. } = &mut self.marks;
        for index in indices {
            given[index] = banned[index];
        }
    }

    /// An artifact of `classes` with equal chances among the unmarked ones, then marked; none
    /// left: the marks of `classes`' artifacts restart from the bans (since E09 T57 those alone,
    /// as the original's level, survey `E09-trades.md` §3.1; before, every class's); with still
    /// none, any class but Special among the unmarked, and with none of those either, their marks
    /// restart from the bans too. None at all (every artifact of the four classes banned): none
    /// (the original loops for ever).
    // rule: ADV-OBJ-006
    pub(crate) fn draw(
        &mut self,
        classes: &[ArtifactClass],
        draws: &mut Rng,
    ) -> Option<ArtifactTypeId> {
        // E09 T57: counted, then the drawn one found, without a list (the month's merchants).
        let mut classes = classes;
        let mut n = self.count(classes);
        if n == 0 {
            self.restart(classes);
            n = self.count(classes);
        }
        if n == 0 {
            classes = &ANY_CLASS;
            n = self.count(classes);
        }
        if n == 0 {
            self.restart(classes);
            n = self.count(classes);
        }
        // As `pick` over the list: one draw below n, the drawn one's place in id order.
        let r = draws.below(u32::try_from(n).unwrap_or(u32::MAX));
        let index = self.nth(classes, usize::try_from(r).unwrap_or(usize::MAX))?;
        self.marks.given[index] = true;
        ArtifactTypeId::from_index(index)
    }
}

/// The artifacts of some classes by index in id order ([`ArtifactPool::of`]).
enum Of<'a> {
    /// One class's list.
    One(core::slice::Iter<'a, u16>),
    /// Every artifact's class tested against the classes.
    Walk(
        core::iter::Zip<core::slice::Iter<'a, ArtifactType>, core::ops::RangeFrom<usize>>,
        &'a [ArtifactClass],
    ),
}

impl Iterator for Of<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            Self::One(list) => list.next().map(|&index| usize::from(index)),
            Self::Walk(artifacts, classes) => artifacts
                .find(|(artifact, _)| classes.contains(&artifact.class))
                .map(|(_, index)| index),
        }
    }
}
