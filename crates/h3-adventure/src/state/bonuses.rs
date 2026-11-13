//! The battle bonuses and the timed marks on the hero on the map (E08 T115a; increment 12's
//! design note §12; ADR-0016 point 3, HER-ADV-008 amended; decisions 306, 307): every visit's
//! effect lives on the hero on the map, not in its details, so none carries over to a hero that
//! leaves the map and is hired again. The battle bonuses are the morale and luck no once-mark
//! names (a Pandora's Box's, an event's) and a short list of once-marks with the amounts their
//! visits gave (the Warrior's Tomb's, a looted Pyramid's; increment 12's ten bonus kinds and the
//! Fountain of Fortune from T115b); a battle's end clears them (GAM-BAT-014). The timed marks are
//! the object types whose day's or week's mark the hero holds (the Magic Well's, the Stables'):
//! the adventure's day step drops the day's, its week step the week's, and the week's land points
//! are derived from them. The drops and the week's points are functions of one record, so a hero
//! held off the map (the town lane's garrison hero, E09 T28b) keeps and drops them so too.

use alloc::vec::Vec;

use h3_contracts::HeroId;
use h3_heroes::{BattleBonuses, VisitBonus};
use h3_kernel::StateHasher;

use super::AdventureState;
use crate::ruleset::{AdventureRuleset, MarkLifetime, ObjectTypeId};

/// A hero on the map's battle bonuses (E08 T115a; increment 12's design note §12.2, decision
/// 306): what its visits gave it until its next battle's end.
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

    /// Adds `bonus` under its once-mark while the hero does not hold the mark, which it then
    /// holds with the bonus's amounts; `false`, and nothing changed, when it does (a second
    /// Warrior's Tomb before a battle: E7-11; the visits' `NothingGained` from T115b).
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

    /// The hash of the bonuses (the adventure state hash v23): the morale, the luck, then the
    /// once-marks' number and each mark with its amounts.
    pub(crate) fn hash_into(&self, h: &mut StateHasher) {
        h.write_i32(i32::from(self.morale));
        h.write_i32(i32::from(self.luck));
        h.write_len(self.once.len());
        for bonus in &self.once {
            h.write_u8(bonus.mark);
            h.write_i32(i32::from(bonus.morale));
            h.write_i32(i32::from(bonus.luck));
        }
    }
}

/// Adds the timed mark of the object type `object_type` to `timed`, a hero's (sorted, unique):
/// `false`, and nothing changed, when it holds it (a Magic Well's visit the same day, the Stables'
/// the same week; ADV-BON-005, -006 from T115b). The type's kind names how long it holds it
/// ([`crate::ObjectKind::timed_mark`]).
pub fn add_timed(timed: &mut Vec<ObjectTypeId>, object_type: ObjectTypeId) -> bool {
    match timed.binary_search(&object_type) {
        Ok(_) => false,
        Err(at) => {
            timed.insert(at, object_type);
            true
        }
    }
}

/// Drops from `timed`, a hero's timed marks, every mark whose type's kind lives for `lifetime`
/// (E08 T115a; increment 12's design note §12.4): the adventure's day step drops the day's, its
/// week step the week's. A function of one record (the town lane's garrison hero, E09 T28b, calls
/// it for the heroes it holds off the map). A type the part lacks is kept (a validated mark is
/// always a kind's).
pub fn drop_timed(part: &AdventureRuleset, timed: &mut Vec<ObjectTypeId>, lifetime: MarkLifetime) {
    timed.retain(|&ty| {
        part.object_type(ty)
            .and_then(|t| t.kind.timed_mark())
            .is_none_or(|kind| kind != lifetime)
    });
}

/// The week's land points of a hero holding the timed marks `timed` (E08 T115a; increment 12's
/// design note §12.4, HER-ADV-003's input): the points of every week-marked type's kind (the
/// Stables' 400), saturating; derived, not stored.
pub fn week_land_points(part: &AdventureRuleset, timed: &[ObjectTypeId]) -> u32 {
    timed
        .iter()
        .filter_map(|&ty| part.object_type(ty))
        .map(|t| t.kind.week_points())
        .fold(0, u32::saturating_add)
}

impl AdventureState {
    /// The end of a battle for its heroes on the map (GAM-BAT-014 amended, E08 T115a; increment
    /// 12's design note §14): each of `heroes` still on the map loses its battle bonuses and their
    /// once-marks, the game calling it after the battle's capture and the heroes' part, as the
    /// original's `hero_battle_bonuses_clear` from `battle_end` (0x477792). A hero that left the
    /// map left with them. The marks a part names as outliving a battle under the profile (the
    /// Sirens', `ADV-QUIRK-46` off) come with their visits (T115b). Told as the command's
    /// [`crate::AdventureEventKind::BonusChanged`] and [`crate::AdventureEventKind::OnceMarkChanged`].
    // rule: GAM-BAT-014
    pub fn clear_battle_bonuses(&mut self, heroes: &[HeroId]) {
        for &id in heroes {
            if let Some(hero) = self.hero_mut(id) {
                hero.bonuses = HeroBonuses::NONE;
            }
        }
    }

    /// Every hero on the map drops its timed marks of `lifetime` ([`drop_timed`]): the day step's
    /// and the week step's (increment 12's design note §15).
    pub(crate) fn drop_timed_marks(&mut self, part: &AdventureRuleset, lifetime: MarkLifetime) {
        for hero in &mut self.heroes {
            if !hero.timed.is_empty() {
                drop_timed(part, &mut hero.timed, lifetime);
            }
        }
    }
}
