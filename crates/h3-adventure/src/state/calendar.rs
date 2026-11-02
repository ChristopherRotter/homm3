//! The adventure's steps of the day's start (ADR-0016 point 4, ADR-0017 point 3; E08 T85a,
//! increment 9's design note §10, §11; survey `E08-economy.md` §7.3, §7.4, §10): the week's step,
//! which fills the weekly objects again and grows the wandering monsters, and a creature month's
//! step, which adds the month's monsters on free land tiles as objects added in play. The game
//! calls them on a week's and a month's first day after day 1, with its stream (design §14,
//! T85b); they read the static map's flags and the live presence, never a hero's tile but to keep
//! a monster off it.

use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::ArmyStack;
use h3_kernel::Rng;

use super::economy::{contents, kind_at};
use super::{Added, AdventureState, MapObject, Monster, Resolved, Weekly, aggression_between};
use crate::command::{AdventureEvent, AdventureEventKind, Sink};
use crate::ruleset::{AdventureRuleset, MAX_MONSTER_COUNT, ObjectTypeId, Passage};
use crate::setup::{Holds, Pos, Scenery, TileMask};

/// A creature month's monster on `anchor` (ADV-MON-014; the original's `place_object` 0x4C9890):
/// of the months' monster type, one tile blocked and visited from its neighbours, no treasure,
/// growing, fleeing as any.
fn month_monster(object_type: ObjectTypeId, anchor: Pos, monster: Monster) -> MapObject {
    MapObject {
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(object_type),
        contents: Vec::new(),
        drawn: TileMask::default(),
        monster: Some(monster),
        guards: None,
        scenery: Scenery::Other,
        holds: Holds::Nothing,
        resolved: Resolved::Nothing,
    }
}

impl AdventureState {
    /// The week's step (ADR-0017 point 3; E08 T85a, increment 9's design note §10; survey
    /// `E08-economy.md` §7.3, the original's `new_day_week_events` 0x4C8AE0 to 0x4C8D19), called
    /// by the game on a week's first day after the first with the game's stream:
    ///
    /// 1. **The weekly objects** (ADV-WEK-001 to -003), in the order of the static map's weekly
    ///    objects, visited or not, whoever stands on them: a Windmill draws its resource, then
    ///    its amount; a Water Wheel takes its later amount; a Garden draws its gift. The marks
    ///    stay (the original's week keeps them).
    /// 2. **The growth** (ADV-MON-013; D-49 and A-32 as batch E9's E9-9 and E9-10 confirmed), in
    ///    object index order over the wandering monsters still on the map, the static map's then
    ///    those added in play, but those that do not grow: the value `count × unit + fraction`
    ///    (the original's sixteenths) gains `⌊value × growth_bp ÷ 10 000⌋`, capped at
    ///    `growth_cap × unit` (a count from the map above the cap falls to it); the new count is
    ///    written where it changed (one stack of the monster's creature, as a battle's survivors),
    ///    the rest the fraction (none for 0). Where the word of the count a viewer sees changed,
    ///    [`AdventureEventKind::MonsterChanged`] to the players that see the monster; a change
    ///    within the word stays hidden, as the count and the fraction do (ADV-FOG-006).
    ///
    /// No hero is lifted (the original lifts one off an entrance only to read the object under
    /// it). Draws only for the weekly objects.
    // rule: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003, ADV-MON-013
    pub fn start_week<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        draws: &mut Rng,
        sink: &mut S,
    ) {
        self.refill_weekly(part, draws);
        self.grow_monsters(core, part, sink);
    }

    /// The week's step's first part ([`Self::start_week`]): each weekly object's contents for a
    /// later week, its marks kept. One write of the block the forks share; none without weekly
    /// objects.
    // rule: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003
    fn refill_weekly(&mut self, part: &AdventureRuleset, draws: &mut Rng) {
        let Some(block) = self.weekly.as_deref() else {
            return;
        };
        let map = &self.map;
        let weekly: Vec<Weekly> = map
            .weeklies()
            .iter()
            .zip(block)
            .map(|(&index, &old)| {
                let (resource, amount) = contents(kind_at(part, map, index), false, draws)
                    .unwrap_or((old.resource, old.amount));
                Weekly {
                    resource,
                    amount,
                    marks: old.marks,
                }
            })
            .collect();
        self.weekly = Some(weekly.into());
    }

    /// The week's step's growth ([`Self::start_week`]; ADV-MON-013). Nothing for a part without
    /// the monsters' parameters.
    // rule: ADV-MON-013
    fn grow_monsters<S: Sink>(&mut self, core: &Ruleset, part: &AdventureRuleset, sink: &mut S) {
        let Some(params) = part.params().monsters() else {
            return;
        };
        let unit = u64::from(params.growth_unit());
        let most = u64::from(params.growth_cap()) * unit;
        let rate = u64::from(params.growth_bp());
        let objects = self.map.objects().len() + self.added_objects().len();
        for index in 0..objects {
            let Some((object_type, creature, count)) = self.growing(index) else {
                continue;
            };
            let fraction = self.fraction(index);
            let value = u64::from(count) * unit + u64::from(fraction);
            let grown = (value + value * rate / 10_000).min(most);
            let now = u32::try_from(grown / unit).unwrap_or(MAX_MONSTER_COUNT);
            let left = u8::try_from(grown % unit).unwrap_or(u8::MAX);
            if left != fraction {
                self.fractions.set(index, (left > 0).then_some(left));
            }
            if now == count {
                continue;
            }
            self.army_left.set(
                index,
                Some(vec![ArmyStack {
                    slot: 0,
                    creature,
                    count: now,
                }]),
            );
            let word = params.count_word(now);
            if word != params.count_word(count) {
                let object = self.object_at(index);
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::MonsterChanged {
                        object_type,
                        anchor: object.anchor,
                        count_word: word,
                    },
                    seen_by: self.object_viewers(core, object),
                });
            }
        }
    }

    /// The monster at object index `index` if it grows this week: a wandering monster still on
    /// the map that may grow, with its type, creature and count as it stands.
    fn growing(&self, index: usize) -> Option<(ObjectTypeId, CreatureTypeId, u32)> {
        let object = self.object_at(index);
        match (object.object_type, &object.monster) {
            (Some(object_type), Some(monster)) if !monster.no_growth && !self.is_removed(index) => {
                Some((object_type, monster.creature, self.count_of(index, monster)))
            }
            _ => None,
        }
    }

    /// A creature month's step (ADR-0017 point 3; ADV-MON-014, E08 T85a, increment 9's design
    /// note §11; D-50 as read; survey `E08-economy.md` §7.4, §10.2, the original's `new_month`
    /// 0x4C90D0 to 0x4C9236), called by the game in a month of `creature` (one of the months'
    /// parameters' creatures) with the game's stream: every tile of the map in level, row and
    /// column order that is land (the part's passage: no water, no rock), that no object still on
    /// the map blocks or is visited from (one the walk added before too), with no event still on
    /// the map and no hero, draws `1..=spawn_one_in`; a 1 adds a monster on it, an object added
    /// in play ([`month_monster`]): `creature`, a count of `spawn_factor` times a draw from the
    /// creature's adventure-map range (ADV-MON-004; at least 1, at most [`MAX_MONSTER_COUNT`]),
    /// then an aggression drawn between `spawn_aggression`'s bounds. Each is told as
    /// [`AdventureEventKind::MonsterAdded`] to the players that see it, as a placed monster
    /// (ADV-FOG-004), and guards, answers and fights as one from then on. Nothing tests a guard
    /// zone, a hero beside the tile or a viewer, as the original's walk; its guarded tiles,
    /// refreshed after the walk (`refresh_guard_flags` 0x4CA410), are read live here, which no
    /// test of the walk reads. Nothing for a part without the months' parameters.
    // rule: ADV-MON-014
    pub fn start_month<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        creature: CreatureTypeId,
        draws: &mut Rng,
        sink: &mut S,
    ) {
        let Some(months) = part.params().months() else {
            return;
        };
        let [low, high] = core
            .creature(creature)
            .map_or([1, 1], |info| info.adventure_count);
        let one_in = u32::from(months.spawn_one_in.get());
        let factor = u32::from(months.spawn_factor.get());
        let map = Arc::clone(&self.map);
        let heroes: BTreeSet<usize> = self
            .heroes
            .iter()
            .filter_map(|h| map.index(h.pos))
            .collect();
        for level in 0..map.levels() {
            for y in 0..map.height() {
                for x in 0..map.width() {
                    let pos = Pos::new(x, y, level);
                    let tile = map.at(pos);
                    let free = matches!(part.passage(tile.terrain), Some(Passage::Land(_)))
                        && {
                            let live = self.live(pos, tile);
                            !live.blocked && !live.visitable
                        }
                        && self.event_at(pos).is_none()
                        && map.index(pos).is_some_and(|index| !heroes.contains(&index));
                    if !free || draws.range_inclusive(1, one_in) != 1 {
                        continue;
                    }
                    let count = draws
                        .range_inclusive(u32::from(low), u32::from(high))
                        .max(1)
                        .saturating_mul(factor)
                        .min(MAX_MONSTER_COUNT);
                    let aggression = aggression_between(months.spawn_aggression, draws);
                    let monster = Monster {
                        creature,
                        count,
                        aggression,
                        never_flees: false,
                        no_growth: false,
                        treasure: Vec::new(),
                        artifact: None,
                    };
                    let index = self.add_object(month_monster(months.monster, pos, monster));
                    let count_word = part.params().monsters().map_or(0, |w| w.count_word(count));
                    sink.emit(AdventureEvent {
                        kind: AdventureEventKind::MonsterAdded {
                            object_type: months.monster,
                            anchor: pos,
                            creature,
                            count_word,
                        },
                        seen_by: self.object_viewers(core, self.object_at(index)),
                    });
                }
            }
        }
    }

    /// Adds `object` to the map after every object before it (E08 T83b2a, increment 9's design
    /// note §6; a creature month's monster since T85a) and returns its index: the removed
    /// objects' grid makes room for it, the added objects' block takes it with its tiles. The
    /// caller has checked that it is a monster on land tiles no other object still on the map
    /// covers ([`Self::check_added`]).
    pub(crate) fn add_object(&mut self, object: MapObject) -> usize {
        let index = self.map.objects().len() + self.added_objects().len();
        if let Some(occupancy) = &mut self.occupancy {
            Arc::make_mut(occupancy).make_room(index + 1);
        }
        let map = &self.map;
        let added = self.added.get_or_insert_with(|| Arc::new(Added::new(map)));
        Arc::make_mut(added).push(map, object);
        index
    }
}
