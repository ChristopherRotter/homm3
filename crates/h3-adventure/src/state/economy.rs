//! The owned and weekly objects' state (ADR-0016 points 1 and 5; E08 T83b1, increment 9's design
//! note §6): each ownable object's owner by its place among the static map's ownable objects, a
//! mine's guards (the armies on objects, else an abandoned mine's drawn Troglodytes), each weekly
//! object's contents and visited marks by its place among the weekly ones, with the first week's
//! drawn when the game is built, and the start's reveal around every owned object. The owners are
//! one block the forks share (a write copies it: a few hundred bytes on the largest shipped map),
//! the weekly contents one too, none for a map without weekly objects, so a fork costs a reference
//! count or two.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::ResourceId;
use h3_contracts::{ArmyStack, PlayerId};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

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
    /// The players that visited it, a bit by player id: a Windmill's and a Water Wheel's the
    /// visitor's team's (the visitor's own until E10), a Garden's the visitor's own; never cleared
    /// (the original's week keeps them, §9.4), so they say "visited once", not "emptied".
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

/// The kind of the object at `index` of `map`, if it has a type.
fn kind_at<'p>(
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

/// A weekly object's first week's contents by its kind (ADV-WEK-001 to -003): a Windmill draws its
/// resource among its kind's, then its amount between the bounds; a Water Wheel takes its first
/// week's amount; a Garden draws one of its gifts, with equal chances. None for every other kind
/// (the part keeps a Windmill's resources and a Garden's gifts from being empty).
// rule: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003
fn first_contents(kind: Option<&ObjectKind>, draws: &mut Rng) -> Option<(ResourceId, u32)> {
    match kind {
        Some(ObjectKind::Windmill { resources, amount }) => pick(resources, draws)
            .map(|&resource| (resource, draws.range_inclusive(amount[0], amount[1]))),
        Some(&ObjectKind::WaterWheel {
            resource, first, ..
        }) => Some((resource, first)),
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
/// (the setup checked it, [`crate::SetupError::ObjectOwner`]).
// rule: ADV-OWN-002
pub(super) fn owners_at_start(map: &StaticMap, setup: &[ObjectSetup]) -> Arc<[Option<PlayerId>]> {
    map.ownables()
        .iter()
        .map(|&index| setup.get(index).and_then(|object| object.owner))
        .collect()
}

/// The first week's weekly contents (ADV-WEK-001 to -003; increment 9's design note §6, §10), drawn
/// from the setup stream after the static map's draws, in object order (the weekly objects'
/// order): each one's [`first_contents`], no marks. None for a map without weekly objects.
pub(super) fn first_week(
    part: &AdventureRuleset,
    map: &StaticMap,
    draws: &mut Rng,
) -> Option<Arc<[Weekly]>> {
    let weekly: Vec<Weekly> = (0..map.objects().len())
        .filter_map(|index| first_contents(kind_at(part, map, index), draws))
        .map(|(resource, amount)| Weekly {
            resource,
            amount,
            marks: 0,
        })
        .collect();
    (!weekly.is_empty()).then(|| weekly.into())
}

impl AdventureState {
    /// The owner of the object at object index `index` (ADV-OWN-002, E08 T83b1): none for a
    /// neutral object and for an object that is not ownable.
    pub fn owner(&self, index: usize) -> Option<PlayerId> {
        let place = self.map.ownable_index(index)?;
        self.owners.get(place).copied().flatten()
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
    // rule: ADV-MIN-003
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

    /// The start's reveal around every owned object (ADR-0016 point 5, ADV-OWN-002, E08 T83b1):
    /// the tiles within the part's flag radius of its entrance, to its owner (its team from E10),
    /// a Lighthouse's and a Shipyard's too, as the original's `reveal_owned` 0x4C7C50 from
    /// `adventure_manager_start` (batch E9's E9-4 and E9-6; A-33 withdrawn) and VCMI's
    /// `initFogOfWar`. No event, as the start's other reveals.
    // rule: ADV-OWN-002
    pub(super) fn reveal_owned(&mut self, part: &AdventureRuleset) {
        let radius = part.params().flag_radius();
        for (owner, &index) in self.owners.iter().zip(self.map.ownables()) {
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
        self.owners.len() == self.map.ownables().len()
            && self
                .owners
                .iter()
                .flatten()
                .all(|owner| owner.index() < players)
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
        h.write_len(self.owners.len());
        for owner in self.owners.iter() {
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
