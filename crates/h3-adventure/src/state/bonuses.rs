//! The battle bonuses and the timed marks on the hero on the map (E08 T115a; increment 12's
//! design note §12; ADR-0016 point 3, HER-ADV-008 amended; decisions 306, 307): every visit's
//! effect lives on the hero on the map, not in its details, so none carries over to a hero that
//! leaves the map and is hired again. The battle bonuses are the morale and luck no once-mark
//! names (a Pandora's Box's, an event's) and a short list of once-marks with the amounts their
//! visits gave (the Warrior's Tomb's, a looted Pyramid's; increment 12's ten bonus kinds, the
//! Fountain of Fortune and the Sirens since T115b, `visitables.rs`); a battle's end clears them
//! (GAM-BAT-014; the Sirens' kept with `ADV-QUIRK-46` off). The timed marks are
//! the object types whose day's or week's mark the hero holds (the Magic Well's, the Stables'):
//! the adventure's day step drops the day's, its week step the week's, and the week's land points
//! are derived from them. Both are one record's ([`VisitMarks`]), so a hero held off the map (the
//! town lane's garrison hero, E09 T28b) keeps and drops them so too.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_contracts::HeroId;
use h3_heroes::{BattleBonuses, VisitBonus};
use h3_kernel::{CanonicalHash, StateHasher};

use super::AdventureState;
use crate::ruleset::{AdventureRuleset, MarkLifetime, ObjectTypeId};

/// A hero's battle bonuses (E08 T115a; increment 12's design note §12.2, decision 306): what its
/// visits gave it until its next battle's end.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroBonuses {
    /// The morale no once-mark names (a Pandora's Box's, an event's: the battle's temporary
    /// source), held saturating (the original's byte at +0x11A wraps past 127: A-9's class,
    /// fixed).
    pub morale: i8,
    /// The luck no once-mark names (the original's +0x11B), held saturating.
    pub luck: i8,
    /// The once-marks with the morale and luck each visit gave, sorted by mark and unique; no
    /// fixed count (the user, ADV-Q50 (3): content beyond SoD may add visitables). The mark is the
    /// content's (the Tomb's 0, the Pyramid's 1, increment 12's twelve from 2 in the original's
    /// bit order); the amounts are kept per mark, since a Fountain's value is the week's and the
    /// Idol's and the Temple's the day's, so the content cannot give them back.
    pub once: Vec<VisitBonus>,
}

impl HeroBonuses {
    /// No bonus: a hero placed on the map, or after its battle.
    pub const NONE: Self = Self {
        morale: 0,
        luck: 0,
        once: Vec::new(),
    };

    /// Whether the hero holds the once-mark `mark`.
    pub fn has_mark(&self, mark: u8) -> bool {
        self.once.binary_search_by_key(&mark, |b| b.mark).is_ok()
    }

    /// Whether the hero holds no bonus: no morale, no luck, no once-mark.
    pub fn is_empty(&self) -> bool {
        self.morale == 0 && self.luck == 0 && self.once.is_empty()
    }

    /// The bonuses as `h3_heroes::derive_battle_hero`'s input (ADR-0013 point 15.1 amended).
    pub fn battle(&self) -> BattleBonuses<'_> {
        BattleBonuses {
            morale: self.morale,
            luck: self.luck,
            once: &self.once,
        }
    }

    /// Adds `morale` and `luck` to the sums no once-mark names, saturating (a reward record's
    /// bonus half: a Pandora's Box's, an event's; ADR-0013 point 15.2's place in its order).
    // rule: HER-ADV-008
    pub fn add(&mut self, morale: i8, luck: i8) {
        self.morale = self.morale.saturating_add(morale);
        self.luck = self.luck.saturating_add(luck);
    }

    /// Whether a battle's end that keeps the once-marks `kept` changes these bonuses (E08 T115b):
    /// some morale or luck, or a once-mark not kept.
    pub fn clears(&self, kept: &[u8]) -> bool {
        self.morale != 0 || self.luck != 0 || self.once.iter().any(|b| !kept.contains(&b.mark))
    }

    /// Adds `bonus` under its once-mark while the hero does not hold the mark, which it then
    /// holds with the bonus's amounts; `false`, and nothing changed, when it does (a second
    /// Warrior's Tomb before a battle: E7-11; since T115b the bonus visitables' second visit,
    /// [`crate::AdventureEventKind::NothingGained`]).
    // rule: HER-ADV-008
    pub fn add_once(&mut self, bonus: VisitBonus) -> bool {
        match self.once.binary_search_by_key(&bonus.mark, |b| b.mark) {
            Ok(_) => false,
            Err(at) => {
                self.once.insert(at, bonus);
                true
            }
        }
    }
}

/// What a hero holds of its visits (E08 T115a; increment 12's design note §12): its battle
/// bonuses ([`HeroBonuses`]) and its timed marks, the object types whose day's or week's mark it
/// holds, sorted. A record of its own, so a hero held off the map (the town lane's garrison
/// hero, E09 T28b) keeps and drops them as a hero on the map does; the state holds them for the
/// heroes on the map that hold any ([`AdventureState::visit_marks`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VisitMarks {
    bonuses: HeroBonuses,
    timed: Vec<ObjectTypeId>,
}

/// Nothing held, for a hero that holds nothing ([`AdventureState::visit_marks`]).
static NO_MARKS: VisitMarks = VisitMarks::NONE;

impl VisitMarks {
    /// Nothing held: a hero placed on the map.
    pub const NONE: Self = Self {
        bonuses: HeroBonuses::NONE,
        timed: Vec::new(),
    };

    /// The battle bonuses.
    pub const fn bonuses(&self) -> &HeroBonuses {
        &self.bonuses
    }

    /// The timed marks: the object types whose day's or week's mark the hero holds, sorted.
    pub fn timed(&self) -> &[ObjectTypeId] {
        &self.timed
    }

    /// Whether nothing is held: no bonus and no timed mark.
    pub fn is_empty(&self) -> bool {
        self.bonuses.is_empty() && self.timed.is_empty()
    }

    /// [`HeroBonuses::add`].
    // rule: HER-ADV-008
    pub fn add(&mut self, morale: i8, luck: i8) {
        self.bonuses.add(morale, luck);
    }

    /// [`HeroBonuses::add_once`].
    // rule: HER-ADV-008
    pub fn add_once(&mut self, bonus: VisitBonus) -> bool {
        self.bonuses.add_once(bonus)
    }

    /// Adds the timed mark of the object type `object_type` (the type's kind names how long it is
    /// held, [`crate::ObjectKind::timed_mark`]): `false`, and nothing changed, when it is held (a
    /// Magic Well's visit the same day, the Stables' the same week; E08 T115b).
    pub fn add_timed(&mut self, object_type: ObjectTypeId) -> bool {
        match self.timed.binary_search(&object_type) {
            Ok(_) => false,
            Err(at) => {
                self.timed.insert(at, object_type);
                true
            }
        }
    }

    /// The battle's end (GAM-BAT-014): the bonuses cleared but the once-marks `kept` (since E08
    /// T115b the Sirens' with `ADV-QUIRK-46` off), the timed marks kept.
    // rule: GAM-BAT-014
    pub fn clear_bonuses(&mut self, kept: &[u8]) {
        self.bonuses.morale = 0;
        self.bonuses.luck = 0;
        self.bonuses.once.retain(|bonus| kept.contains(&bonus.mark));
    }

    /// Drops every timed mark whose type's kind lives for `lifetime` (E08 T115a; increment 12's
    /// design note §12.4): the adventure's day step drops the day's, its week step the week's. A
    /// function of one record (the town lane's garrison hero, E09 T28b, calls it for the heroes it
    /// holds off the map). A type the part lacks is kept (a validated mark is always a kind's).
    // rule: ADV-BON-001
    pub fn drop_timed(&mut self, part: &AdventureRuleset, lifetime: MarkLifetime) {
        self.timed.retain(|&ty| !lives_for(part, ty, lifetime));
    }

    /// The week's land points (E08 T115a; increment 12's design note §12.4, HER-ADV-003's input):
    /// the points of every week-marked type's kind (the Stables' 400), saturating; derived, not
    /// stored.
    // rule: ADV-BON-001
    pub fn week_land_points(&self, part: &AdventureRuleset) -> u32 {
        self.timed
            .iter()
            .filter_map(|&ty| part.object_type(ty))
            .map(|t| t.kind.week_points())
            .fold(0, u32::saturating_add)
    }

    /// The hash of what is held (the adventure state hash v23): the morale, the luck, the
    /// once-marks' number and each mark with its amounts, then the timed marks' number and types.
    pub(crate) fn hash_into(&self, h: &mut StateHasher) {
        h.write_i32(i32::from(self.bonuses.morale));
        h.write_i32(i32::from(self.bonuses.luck));
        h.write_len(self.bonuses.once.len());
        for bonus in &self.bonuses.once {
            h.write_u8(bonus.mark);
            h.write_i32(i32::from(bonus.morale));
            h.write_i32(i32::from(bonus.luck));
        }
        h.write_len(self.timed.len());
        for ty in &self.timed {
            ty.hash_into(h);
        }
    }
}

/// Whether the timed mark of `ty` lives for `lifetime`: a type the part lacks is never dropped (a
/// validated mark is always a kind's).
fn lives_for(part: &AdventureRuleset, ty: ObjectTypeId, lifetime: MarkLifetime) -> bool {
    part.object_type(ty)
        .and_then(|t| t.kind.timed_mark())
        .is_some_and(|kind| kind == lifetime)
}

/// The visit marks of the heroes on the map that hold any (E08 T115a): by hero id, sorted, none
/// empty ([`AdventureState`]'s block).
pub(crate) type HeldMarks = Vec<(HeroId, VisitMarks)>;

/// The marks `hero` holds in `block`, nothing when it holds none.
pub(crate) fn marks_in(block: Option<&HeldMarks>, hero: HeroId) -> &VisitMarks {
    block
        .and_then(|all| {
            all.binary_search_by_key(&hero, |&(id, _)| id)
                .ok()
                .map(|at| &all[at].1)
        })
        .unwrap_or(&NO_MARKS)
}

impl AdventureState {
    /// The battle bonuses and timed marks of the hero on the map `hero` (E08 T115a; increment 12's
    /// design note §12, ADR-0016 point 3: the hero on the map's, not its details'); nothing for a
    /// hero that holds none or is not on the map.
    pub fn visit_marks(&self, hero: HeroId) -> &VisitMarks {
        marks_in(self.visited.as_deref(), hero)
    }

    /// Changes the marks of the hero on the map `hero` with `change` (nothing for a hero not on
    /// the map): the block written first if there is none (copied if the forks share it), the
    /// hero's entry dropped when it is left empty and the block when no hero holds any.
    pub(crate) fn change_marks<R: Default>(
        &mut self,
        hero: HeroId,
        change: impl FnOnce(&mut VisitMarks) -> R,
    ) -> R {
        if self.hero(hero).is_none() {
            return R::default();
        }
        let all = Arc::make_mut(self.visited.get_or_insert_with(Arc::default));
        let at = all
            .binary_search_by_key(&hero, |&(id, _)| id)
            .unwrap_or_else(|at| {
                all.insert(at, (hero, VisitMarks::NONE));
                at
            });
        let result = change(&mut all[at].1);
        if all[at].1.is_empty() {
            all.remove(at);
        }
        if all.is_empty() {
            self.visited = None;
        }
        result
    }

    /// Adds `morale` and `luck` to the battle bonuses of the hero on the map `hero`, unmarked
    /// ([`VisitMarks::add`]; nothing written for nothing added).
    pub(crate) fn give_bonus(&mut self, hero: HeroId, morale: i8, luck: i8) {
        if morale != 0 || luck != 0 {
            self.change_marks(hero, |marks| marks.add(morale, luck));
        }
    }

    /// Gives the hero on the map `hero` `bonus` under its once-mark ([`VisitMarks::add_once`]):
    /// `false`, nothing written, when it holds the mark or is not on the map.
    pub(crate) fn give_once(&mut self, hero: HeroId, bonus: VisitBonus) -> bool {
        !self.visit_marks(hero).bonuses().has_mark(bonus.mark)
            && self.change_marks(hero, |marks| marks.add_once(bonus))
    }

    /// The block of the heroes' marks, for [`crate::OwnHeroes`]' snapshot (a pointer's copy).
    pub(crate) fn held_marks(&self) -> Option<Arc<HeldMarks>> {
        self.visited.clone()
    }

    /// A hero that leaves the map leaves with its marks (decision 307: none carries over to a
    /// hero hired again).
    pub(crate) fn leave_marks(&mut self, hero: HeroId) {
        let holds = |all: &&mut Arc<HeldMarks>| all.iter().any(|&(id, _)| id == hero);
        let Some(block) = self.visited.as_mut().filter(holds) else {
            return;
        };
        let all = Arc::make_mut(block);
        all.retain(|&(id, _)| id != hero);
        if all.is_empty() {
            self.visited = None;
        }
    }

    /// Every hero on the map drops its timed marks of `lifetime` ([`VisitMarks::drop_timed`]):
    /// the day step's and the week step's (increment 12's design note §15). Nothing written
    /// while no hero holds a mark.
    pub(crate) fn drop_timed_marks(&mut self, part: &AdventureRuleset, lifetime: MarkLifetime) {
        let Some(block) = self.visited.as_deref() else {
            return;
        };
        let held: Vec<HeroId> = block
            .iter()
            .filter(|(_, marks)| marks.timed.iter().any(|&ty| lives_for(part, ty, lifetime)))
            .map(|&(id, _)| id)
            .collect();
        for id in held {
            self.change_marks(id, |marks| marks.drop_timed(part, lifetime));
        }
    }
}
