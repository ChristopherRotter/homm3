//! The game's day advance (ADR-0017 point 3; E08 T85b, increment 9's design note §14; survey
//! `E08-economy.md` §7.2 to §7.4, §8, §12, the original's `new_day` 0x4C7FE0): the calendar, the
//! week's and the month's steps on their first days, the month's kind, the players' income and
//! the heroes' spell points, then the adventure's start-of-day step. Day 1 runs none of it: the
//! build calls the adventure's start-of-day step only (the original's start runs no new day,
//! survey §7.1).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, ResourceId};
use h3_contracts::PlayerId;
use h3_kernel::{CanonicalHash, StateHasher};

use super::GameState;
use crate::GameRuleset;
use crate::command::{AdventureSink, GameEvent, Sink};

/// A month's kind (GAM-CAL-003, E08 T85b; survey `E08-economy.md` §7.4): drawn on each month's
/// first day after the first (day 29, 57, …); the first month is a normal one. Public, as the
/// original's month's message (`docs/06-security.md`). What the kinds do to the towns' growth is
/// E09's; a creature's month adds its monsters to the map (ADV-MON-014).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MonthKind {
    /// A normal month (the original's draw of 1 to 5 of 10).
    #[default]
    Normal,
    /// The month of a creature (6 to 9), drawn among the part's months' creatures.
    Creature(CreatureTypeId),
    /// The Plague (10).
    Plague,
}

impl CanonicalHash for MonthKind {
    /// 0 a normal month, 1 a creature's with its id, 2 the Plague.
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Normal => h.write_u8(0),
            Self::Creature(creature) => {
                h.write_u8(1);
                creature.hash_into(h);
            }
            Self::Plague => h.write_u8(2),
        }
    }
}

impl GameState {
    /// The day advance (ADR-0017 point 3; GAM-CAL-002; survey `E08-economy.md` §7.2), once a day
    /// for the whole game after its last group's turn, in this order:
    ///
    /// 1. the day + 1, [`GameEvent::DayStarted`];
    /// 2. on a week's first day (days 8, 15, …: `(day − 1) % week_days == 0`),
    ///    [`GameEvent::WeekStarted`], then the adventure's week step with the game's stream
    ///    (`h3_adventure::AdventureState::start_week`: the weekly objects refilled, the monsters
    ///    grown);
    /// 3. on a month's first day (days 29, 57, …: every `month_weeks`-th week's), the month's
    ///    kind drawn from the game's stream ([`Self::draw_month`]), [`GameEvent::MonthStarted`],
    ///    then in a creature's month the adventure's month step with the game's stream
    ///    (`start_month`: its monsters added on free land tiles);
    /// 4. every player's income ([`Self::pay_income`]);
    /// 5. every hero's spell points ([`Self::regain_mana`]);
    /// 6. the adventure's start-of-day step (E08 T40: in `strict-fixed` each hero's reveal,
    ///    ADV-FOG-003).
    ///
    /// The week's kind and creature (the towns' growth, E09) are not drawn: decision 197 gives
    /// them a stream of their own when they come, so the game's stream stays as it is.
    // rule: GAM-TRN-002, GAM-CAL-001, GAM-CAL-002, GAM-CAL-003
    pub(super) fn advance_day<S: Sink>(&mut self, rules: &GameRuleset, sink: &mut S) {
        let (core, part) = (rules.core(), rules.adventure());
        self.day = self.day.saturating_add(1);
        sink.emit(GameEvent::DayStarted { day: self.day });
        let calendar = part.params().calendar();
        let week_days = u32::from(calendar.week_days.get());
        let month_days = week_days.saturating_mul(u32::from(calendar.month_weeks.get()));
        let since = self.day - 1;
        let week_start = since.is_multiple_of(week_days);
        if week_start {
            sink.emit(GameEvent::WeekStarted {
                week: since / week_days + 1,
            });
            self.adventure
                .start_week(core, part, &mut self.rng, &mut AdventureSink(sink));
        }
        if since.is_multiple_of(month_days) {
            self.month = self.draw_month(rules);
            sink.emit(GameEvent::MonthStarted {
                month: since / month_days + 1,
                kind: self.month,
            });
            if let MonthKind::Creature(creature) = self.month {
                self.adventure.start_month(
                    core,
                    part,
                    creature,
                    &mut self.rng,
                    &mut AdventureSink(sink),
                );
            }
        }
        self.pay_income(rules, week_start, sink);
        self.regain_mana(rules);
        self.adventure.start_day(core, part, &self.heroes);
    }

    /// The month's kind (GAM-CAL-003; survey `E08-economy.md` §7.4, `new_month` 0x4C8F5E): a draw
    /// of `1..=` the sum of the part's month odds (the original's 1 to 10) from the game's stream,
    /// against the odds in order: up to the first weight a normal month, then a creature's, then
    /// the Plague (5, 4, 1); a creature's month then draws its creature among the months'
    /// creatures with equal chances (the original's twelve). A part without the months'
    /// parameters makes every month a normal one, without a draw.
    // rule: GAM-CAL-003
    fn draw_month(&mut self, rules: &GameRuleset) -> MonthKind {
        let Some(months) = rules.adventure().params().months() else {
            return MonthKind::Normal;
        };
        let [normal, creature, plague] = months.odds.map(u32::from);
        let total = normal + creature + plague;
        let draw = self.rng.range_inclusive(1, total);
        if draw <= normal {
            MonthKind::Normal
        } else if draw <= normal + creature {
            // The part has a creature when a creature's month has a weight (its compile's check).
            let count = u32::try_from(months.creatures.len()).unwrap_or(u32::MAX);
            let pick = usize::try_from(self.rng.below(count)).unwrap_or(usize::MAX);
            months
                .creatures
                .get(pick)
                .map_or(MonthKind::Normal, |&c| MonthKind::Creature(c))
        } else {
            MonthKind::Plague
        }
    }

    /// The day's income (GAM-RES-003 to -005; survey `E08-economy.md` §8, `new_day` 0x4C81E2), for
    /// every player in id order (none is defeated before E10): the adventure's income of its mines
    /// and, on a week's first day, its armies' weekly resources
    /// (`h3_adventure::AdventureState::income`), plus `h3_heroes::daily_income` of each of its
    /// heroes on the map (artifacts, Estates, specialists), gained saturating
    /// ([`GameState::gain`]: one [`GameEvent::ResourceGained`] per resource with a non-zero
    /// amount, in the core's order, to the player alone). A mine flagged today pays from the next
    /// day's start.
    // rule: GAM-RES-003, GAM-RES-004, GAM-RES-005
    fn pay_income<S: Sink>(&mut self, rules: &GameRuleset, week_start: bool, sink: &mut S) {
        let (core, part) = (rules.core(), rules.adventure());
        for player in (0..=u8::MAX).take(self.players.len()).map(PlayerId::new) {
            let mut income = self.adventure.income(core, part, player, week_start);
            let heroes = self.adventure.heroes().iter().filter(|h| h.owner == player);
            for details in heroes.filter_map(|h| self.heroes.get(usize::from(h.id.raw()))) {
                let daily = h3_heroes::daily_income(details, core);
                for (entry, amount) in income.iter_mut().zip(daily) {
                    *entry = entry.saturating_add(amount);
                }
            }
            let gains: Vec<(ResourceId, u32)> = (0..=u16::MAX)
                .map(ResourceId::from_raw)
                .zip(income)
                .filter(|&(_, amount)| amount > 0)
                .collect();
            self.gain(player, &gains, sink);
        }
    }

    /// Every hero's spell points after the day's regain (HER-MAN-002; survey `E08-economy.md` §12,
    /// `new_day` 0x4C8237: every hero record, on the map or not), in the store's order:
    /// `h3_heroes::day_mana`, written only where it changes (a write copies the store's list and
    /// that hero's details when a fork shares them). No event: a hero's points show in its
    /// owner's observation.
    // rule: HER-MAN-002
    fn regain_mana(&mut self, rules: &GameRuleset) {
        let core = rules.core();
        let changed: Vec<(usize, u16)> = self
            .heroes
            .iter()
            .enumerate()
            .map(|(index, details)| (index, h3_heroes::day_mana(details, core)))
            .filter(|&(index, mana)| self.heroes[index].mana != mana)
            .collect();
        if changed.is_empty() {
            return;
        }
        let store = Arc::make_mut(&mut self.heroes);
        for (index, mana) in changed {
            Arc::make_mut(&mut store[index]).mana = mana;
        }
    }
}
