//! The owned and weekly objects' state (ADR-0016 points 1 and 5; E08 T83b1, increment 9's design
//! note §6): each ownable object's owner by its place among the static map's ownable objects, a
//! mine's guards (the armies on objects, else an abandoned mine's drawn Troglodytes), each weekly
//! object's contents and visited marks by its place among the weekly ones, with the first week's
//! drawn when the game is built (since E08 T94b in one pass with the Refugee Camps' draws, in
//! object order, [`in_object_order`]), and the start's reveal around every owned object. The owners are
//! one block the forks share (a write copies it: a few hundred bytes on the largest shipped map),
//! the weekly contents one too, each none for a map without such objects, so a fork of one costs
//! nothing more and of another a reference count or two (`bench-check`).

use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{CreatureAbility, CreatureTypeId, ResourceId, Ruleset};
use h3_contracts::{ArmyStack, PlayerId};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::dwellings::{Camp, draw_camp};
use super::{AdventureState, StaticMap, pick};
use crate::fog::reveal;
use crate::ruleset::{AdventureRuleset, ObjectKind};
use crate::setup::ObjectSetup;

/// A weekly object's contents this week and its visited marks (ADV-WEK-001 to -003, E08 T83b1;
/// increment 9's design note §6; survey `E08-economy.md` §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weekly {
    /// The resource it gives: a Windmill's drawn one, a Water Wheel's own, a Garden's drawn gift's.
    pub resource: ResourceId,
    /// The amount its next visit gives; 0 once a visit emptied it, until the week's step.
    pub amount: u32,
    /// The players that visited it, a bit by player id: the visitor's team's (the visitor's own
    /// until E10), a Garden's too (`ADV-QUIRK-35`, not reproduced); never cleared (the original's
    /// week keeps them, §9.4), so they say "visited once", not "emptied".
    pub marks: u8,
}

impl CanonicalHash for Weekly {
    /// The resource, the amount, the marks.
    fn hash_into(&self, h: &mut StateHasher) {
        self.resource.hash_into(h);
        h.write_u32(self.amount);
        h.write_u8(self.marks);
    }
}

/// Whether `a` and `b` are of one team: the player alone until E10's teams (increment 9's design
/// note §19, "Teams"; an allied map plays its allies as enemies, as before).
pub(crate) fn allied(a: PlayerId, b: PlayerId) -> bool {
    a == b
}

/// The kind of the object at `index` of `map`, if it has a type: none for an object added in
/// play, a monster and never an ownable or weekly object (E08 T85a's audit), so the owned and
/// weekly objects' reads by index stay the static map's.
pub(super) fn kind_at<'p>(
    part: &'p AdventureRuleset,
    map: &StaticMap,
    index: usize,
) -> Option<&'p ObjectKind> {
    map.objects()
        .get(index)
        .and_then(|object| object.object_type)
        .and_then(|id| part.object_type(id))
        .map(|ty| &ty.kind)
}

/// A weekly object's contents for a week by its kind (ADV-WEK-001 to -003): a Windmill draws its
/// resource among its kind's, then its amount between the bounds; a Water Wheel takes its first
/// week's amount in the `first` week, its later one in every other (E08 T85a, the week's step); a
/// Garden draws one of its gifts, with equal chances. None for every other kind (the part keeps a
/// Windmill's resources and a Garden's gifts from being empty).
// rule: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003
pub(super) fn contents(
    kind: Option<&ObjectKind>,
    first_week: bool,
    draws: &mut Rng,
) -> Option<(ResourceId, u32)> {
    match kind {
        Some(ObjectKind::Windmill { resources, amount }) => pick(resources, draws)
            .map(|&resource| (resource, draws.range_inclusive(amount[0], amount[1]))),
        Some(&ObjectKind::WaterWheel {
            resource,
            first,
            later,
        }) => Some((resource, if first_week { first } else { later })),
        Some(ObjectKind::Garden { gifts }) => pick(gifts, draws).copied(),
        _ => None,
    }
}

/// Whether `weekly` fits its object's kind (increment 9's design note §6): a Windmill's resource
/// one of its kind's and its amount within the bounds or 0; a Water Wheel's its kind's resource
/// and its first or later amount or 0; a Garden's one of its gifts, or that gift's resource with
/// 0. Never for another kind.
pub(super) fn fits(kind: Option<&ObjectKind>, weekly: &Weekly) -> bool {
    let amount = weekly.amount;
    match kind {
        Some(ObjectKind::Windmill {
            resources,
            amount: [low, high],
        }) => {
            resources.contains(&weekly.resource)
                && (amount == 0 || (*low..=*high).contains(&amount))
        }
        Some(ObjectKind::WaterWheel {
            resource,
            first,
            later,
        }) => weekly.resource == *resource && [0, *first, *later].contains(&amount),
        Some(ObjectKind::Garden { gifts }) => gifts
            .iter()
            .any(|&(r, a)| r == weekly.resource && (amount == 0 || amount == a)),
        _ => false,
    }
}

/// The owners at the start, by ownable place (ADV-OWN-002): each ownable object's setup owner
/// (the setup checked it, [`crate::SetupError::ObjectOwner`]); none for a map without ownable
/// objects.
// rule: ADV-OWN-002
pub(super) fn owners_at_start(
    map: &StaticMap,
    setup: &[ObjectSetup],
) -> Option<Arc<[Option<PlayerId>]>> {
    (!map.ownables().is_empty()).then(|| {
        map.ownables()
            .iter()
            .map(|&index| setup.get(index).and_then(|object| object.owner))
            .collect()
    })
}

/// A weekly object or a Refugee Camp, by its place among its own (E08 T94b): what the week's
/// draws walk ([`in_object_order`]).
#[derive(Clone, Copy)]
pub(super) enum Drawn {
    /// The weekly object at this place among the static map's weekly objects.
    Weekly(usize),
    /// The camp at this camp index.
    Camp(usize),
}

/// The weekly objects and the camps of `map` in object order, in one pass (E08 T94b, increment
/// 10's design note §9: the original's map scan holds case 78 beside the weekly objects' cases;
/// *agent's call:* one pass, so the stream's order follows the object order).
pub(super) fn in_object_order(map: &StaticMap) -> impl Iterator<Item = Drawn> + '_ {
    let (weeklies, camps) = (map.weeklies(), map.camps());
    let (mut weekly, mut camp) = (0, 0);
    core::iter::from_fn(move || {
        let next_camp = camps
            .get(camp)
            .is_some_and(|c| weeklies.get(weekly).is_none_or(|w| c < w));
        if next_camp {
            camp += 1;
            return Some(Drawn::Camp(camp - 1));
        }
        let place = weekly;
        weekly += 1;
        weeklies.get(place).map(|_| Drawn::Weekly(place))
    })
}

/// A block of the state the forks share, by place: none for a map without its objects.
pub(super) type Block<T> = Option<Arc<[T]>>;

/// A block of `entries`, none for none.
pub(super) fn block<T>(entries: Vec<T>) -> Block<T> {
    (!entries.is_empty()).then(|| entries.into())
}

/// The first week's weekly contents and camps (ADV-WEK-001 to -003, ADV-RFG-001; increment 9's
/// design note §6, §10, increment 10's §9), drawn from the setup stream after the static map's
/// draws, in one pass in object order (since E08 T94b, [`in_object_order`]): each weekly object's
/// first week's [`contents`], no marks; each camp's creature and count ([`draw_camp`]). Each block
/// none for a map without its objects.
// rule: ADV-RFG-001
pub(super) fn first_week(
    core: &Ruleset,
    part: &AdventureRuleset,
    map: &StaticMap,
    draws: &mut Rng,
) -> (Block<Weekly>, Block<Camp>) {
    let mut weekly = Vec::with_capacity(map.weeklies().len());
    let mut camps = Vec::with_capacity(map.camps().len());
    for drawn in in_object_order(map) {
        match drawn {
            Drawn::Weekly(place) => {
                let kind = kind_at(part, map, map.weeklies()[place]);
                weekly.extend(
                    contents(kind, true, draws).map(|(resource, amount)| Weekly {
                        resource,
                        amount,
                        marks: 0,
                    }),
                );
            }
            Drawn::Camp(place) => {
                camps.extend(draw_camp(core, part, map.camp_levels[place], draws));
            }
        }
    }
    (block(weekly), block(camps))
}

impl AdventureState {
    /// The owner of the object at object index `index` (ADV-OWN-002, E08 T83b1): none for a
    /// neutral object and for an object that is not ownable.
    pub fn owner(&self, index: usize) -> Option<PlayerId> {
        self.owners
            .as_deref()
            .zip(self.map.ownable_index(index))
            .and_then(|(owners, place)| owners.get(place).copied().flatten())
    }

    /// The sea points `player`'s Lighthouses add to each of its heroes' day's sea points
    /// (ADV-OWN-005, HER-ADV-009's input; E08 T84b1, increment 9's design note §7; survey
    /// `E08-economy.md` §4, the original's `hero_max_land_movement` 0x4E4A18: the count of the
    /// records the hero's owner holds times 500): the sum of the kind's `sea_points` over the
    /// Lighthouses it owns, saturating; an ally's do not count. 0 on a map without ownable
    /// objects.
    // rule: ADV-OWN-005
    pub fn lighthouse_points(&self, part: &AdventureRuleset, player: PlayerId) -> u32 {
        let Some(owners) = self.owners.as_deref() else {
            return 0;
        };
        self.map
            .ownables()
            .iter()
            .zip(owners)
            .filter(|&(_, &owner)| owner == Some(player))
            .filter_map(|(&index, _)| match kind_at(part, &self.map, index) {
                Some(&ObjectKind::Lighthouse { sea_points }) => Some(sea_points),
                _ => None,
            })
            .fold(0, u32::saturating_add)
    }

    /// The contents and marks of the weekly object at object index `index` (E08 T83b1), if it is
    /// one.
    pub fn weekly(&self, index: usize) -> Option<&Weekly> {
        self.weekly
            .as_deref()
            .zip(self.map.weekly_index(index))
            .and_then(|(weekly, place)| weekly.get(place))
    }

    /// The guards of the mine at object index `index` as they stand (ADV-MIN-002, ADV-MIN-003, E08
    /// T83b1; increment 9's design note §6), by slot: the army written on it (a battle's
    /// survivors, a Leave Guards dialog's group; from T84), else an abandoned mine's drawn count
    /// of its kind's creature in slot 0; none for a mine of the seven, for an army written empty
    /// (the drawn Troglodytes do not come back) and for an object that is no mine.
    // rule: ADV-MIN-001, ADV-MIN-003
    pub fn mine_guards(&self, part: &AdventureRuleset, index: usize) -> Vec<ArmyStack> {
        let kind = kind_at(part, &self.map, index);
        if !matches!(
            kind,
            Some(ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. })
        ) {
            return Vec::new();
        }
        if let Some(army) = self.army_left(index) {
            return army.to_vec();
        }
        match (kind, self.map.objects()[index].resolved.mine()) {
            (Some(&ObjectKind::AbandonedMine { guards, .. }), Some((_, count))) => {
                vec![ArmyStack {
                    slot: 0,
                    creature: guards,
                    count,
                }]
            }
            _ => Vec::new(),
        }
    }

    /// The production of the mine at object index `index`, its resource and its amount a day
    /// (GAM-RES-003, E08 T83b2b): a mine's own; an abandoned mine's drawn resource with the part's
    /// mine of that resource's amount; none for every other object.
    pub fn production(&self, part: &AdventureRuleset, index: usize) -> Option<(ResourceId, u32)> {
        match kind_at(part, &self.map, index)? {
            &ObjectKind::Mine {
                resource, daily, ..
            } => Some((resource, daily)),
            ObjectKind::AbandonedMine { .. } => self.map.objects()[index]
                .resolved
                .mine()
                .and_then(|(resource, _)| part.mine_daily(resource).map(|daily| (resource, daily))),
            _ => None,
        }
    }

    /// The day's income of `player`'s objects and armies on the map (ADR-0017 point 10; E08 T85b,
    /// increment 9's design note §14; survey `E08-economy.md` §8.1, the original's
    /// `player_daily_income` 0x4B8AF0): by resource, dense in the core's order (a treasury's
    /// layout), the production of every mine it owns ([`Self::production`]: a mine's own, an
    /// abandoned mine's drawn resource at that resource's mine's amount), and on a week's first
    /// day (`week_start`) each creature type with a `weekly_resource` ability in the army of one
    /// of its heroes on the map, that amount once however many stacks and heroes hold it (the
    /// Crystal Dragons' 3 crystal, `player_daily_income` 0x4B8C32). Saturating per resource; an
    /// ally's do not count, nor a mine's guards. The heroes' own income is
    /// `h3_heroes::daily_income`, the game's to add, as the towns' (E09).
    // rule: GAM-RES-003, GAM-RES-005
    pub fn income(
        &self,
        core: &Ruleset,
        part: &AdventureRuleset,
        player: PlayerId,
        week_start: bool,
    ) -> Vec<u32> {
        let mut income = vec![0_u32; core.resources().len()];
        // A mine's and an ability's resources are the core's (their compiles' checks).
        let mut add = |resource: ResourceId, amount: u32| {
            let entry = &mut income[resource.index()];
            *entry = entry.saturating_add(amount);
        };
        if let Some(owners) = self.owners.as_deref() {
            let owned = self.map.ownables().iter().zip(owners);
            let mines = owned
                .filter(|&(_, &owner)| owner == Some(player))
                .filter_map(|(&index, _)| self.production(part, index));
            for (resource, daily) in mines {
                add(resource, daily);
            }
        }
        if week_start {
            let creatures: BTreeSet<CreatureTypeId> = self
                .heroes
                .iter()
                .filter(|hero| hero.owner == player)
                .flat_map(|hero| &hero.army)
                .map(|stack| stack.creature)
                .collect();
            for info in creatures.into_iter().filter_map(|c| core.creature(c)) {
                for ability in &info.abilities {
                    let CreatureAbility::WeeklyResource { resource, amount } = *ability;
                    add(resource, amount);
                }
            }
        }
        income
    }

    /// Whether `player` holds the visited mark of the weekly object at object index `index` (E08
    /// T83b2b; survey `E08-economy.md` §9.4): a visit of its team's set it (a Garden's too since
    /// T84b2, `ADV-QUIRK-35`); never cleared. False for every other object.
    pub fn visited_by(&self, index: usize, player: PlayerId) -> bool {
        self.weekly(index).is_some_and(|weekly| {
            weekly
                .marks
                .checked_shr(u32::from(player.raw()))
                .is_some_and(|marks| marks & 1 == 1)
        })
    }

    /// The start's reveal around every owned object (ADR-0016 point 5, ADV-OWN-002, E08 T83b1):
    /// the tiles within the part's flag radius of its entrance, to its owner (its team from E10),
    /// a Lighthouse's and a Shipyard's too, as the original's `reveal_owned` 0x4C7C50 from
    /// `adventure_manager_start` (batch E9's E9-4 and E9-6; A-33 withdrawn) and VCMI's
    /// `initFogOfWar`. No event, as the start's other reveals.
    // rule: ADV-OWN-002
    pub(super) fn reveal_owned(&mut self, part: &AdventureRuleset) {
        let radius = part.params().flag_radius();
        let owners = self.owners.as_deref().unwrap_or(&[]);
        for (owner, &index) in owners.iter().zip(self.map.ownables()) {
            let entrance = self.map.objects()[index].entrance(&self.map);
            if let (Some(owner), Some(entrance)) = (owner, entrance) {
                reveal(
                    &mut self.explored[usize::from(owner.raw())],
                    entrance,
                    radius,
                );
            }
        }
    }

    /// The owners' invariants (E08 T83b1; increment 9's design note §6), which every command
    /// keeps: an owner for each ownable object and none other, each a player of the game; a mine's
    /// guards within the army's slots, in distinct slots in order, each of one creature to a
    /// stack's most.
    // rule: ADV-MIN-001
    pub fn check_owners(&self, part: &AdventureRuleset) -> bool {
        let params = part.params();
        let players = usize::from(self.players);
        let guards = |index: usize| {
            let army = self.mine_guards(part, index);
            army.windows(2).all(|pair| pair[0].slot < pair[1].slot)
                && army.iter().all(|stack| {
                    stack.slot < params.army_slots()
                        && (1..=params.max_stack_count()).contains(&stack.count)
                })
        };
        let owners = self.owners.as_deref().unwrap_or(&[]);
        owners.len() == self.map.ownables().len()
            && owners.iter().flatten().all(|owner| owner.index() < players)
            && self.map.ownables().iter().all(|&index| guards(index))
    }

    /// The weekly objects' invariants (E08 T83b1; increment 9's design note §6), which every
    /// command keeps: contents for each weekly object, none for a map without them; a Windmill's
    /// resource one of its kind's and its amount within the bounds or 0; a Water Wheel's its
    /// kind's resource, its first or later amount or 0; a Garden's one of its gifts or that gift's
    /// resource with 0; marks only of the game's players.
    pub fn check_weekly(&self, part: &AdventureRuleset) -> bool {
        let Some(weekly) = self.weekly.as_deref() else {
            return self.map.weeklies().is_empty();
        };
        let all_marks = u8::try_from((1_u16 << self.players) - 1).unwrap_or(u8::MAX);
        weekly.len() == self.map.weeklies().len()
            && self.map.weeklies().iter().zip(weekly).all(|(&index, w)| {
                fits(kind_at(part, &self.map, index), w) && w.marks & !all_marks == 0
            })
    }

    /// The owners' and weekly objects' part of the state hash (since adventure state hash v18,
    /// E08 T83b1): the owners as their number and each, 0 for none, else 1 and the player id; the
    /// weekly contents as 0 for none, else 1, their number and each one's encoding.
    pub(super) fn hash_economy(&self, h: &mut StateHasher) {
        let owners = self.owners.as_deref().unwrap_or(&[]);
        h.write_len(owners.len());
        for owner in owners {
            match owner {
                None => h.write_u8(0),
                Some(player) => {
                    h.write_u8(1);
                    h.write_u8(player.raw());
                }
            }
        }
        match self.weekly.as_deref() {
            None => h.write_u8(0),
            Some(weekly) => {
                h.write_u8(1);
                h.write_len(weekly.len());
                for w in weekly {
                    w.hash_into(h);
                }
            }
        }
    }
}
