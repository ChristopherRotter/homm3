//! The weekly objects' visits (ADV-WEK-001 to -003, ADV-VIS-006 amended; E08 T84b1; increment 9's
//! design note §9; survey `E08-economy.md` §9): a Windmill, a Water Wheel or a Mystical Garden is
//! entered by any hero and asks nothing; its visit sets the visited marks, then gives this week's
//! contents once, to whoever comes first, or tells the visitor's owner it is empty. The week's
//! step fills them again (`calendar.rs`, E08 T85a).

use alloc::sync::Arc;
use alloc::vec;

use h3_contracts::PlayerId;

use super::{AdventureState, allied};
use crate::command::{AdventureEvent, AdventureEventKind, AdventureOutcome, Sink, Viewers};
use crate::ruleset::ObjectTypeId;

impl AdventureState {
    /// The visit of the weekly object at object index `index`, of type `object_type`, by
    /// `player`'s hero (ADV-WEK-001 to -003, ADV-VIS-006 amended; E08 T84b1; increment 9's design
    /// note §9; survey `E08-economy.md` §9, the original's `visit_windmill` 0x4A7FC0,
    /// `visit_water_wheel` 0x4A7DE0 and `visit_garden` 0x4A3BC0). The marks first, for every
    /// player of the visitor's team (`tile_mark_visited_by_team` 0x4FBF90), a Garden's too, where
    /// the original marks its visitor alone (`ADV-QUIRK-35`, not reproduced: E08 decision 222); a
    /// mark set anew is told to its player ([`AdventureEventKind::ObjectMarked`]), hint data only,
    /// never read by the gain.
    /// Then this week's contents, the object's and not a player's (E08 decision 201): an amount
    /// left is the outcome's gain for the visitor's owner (the game's `ResourceGained`, as a
    /// pickup's) and the object is empty for everyone until the week's step; an empty one gives
    /// nothing and tells the visitor's owner so ([`AdventureEventKind::ObjectEmpty`]). No question,
    /// whoever owns the hero; the block is copied only when the visit changes it (a fork shares
    /// it until then).
    // rule: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003, ADV-VIS-006
    pub(super) fn visit_weekly<S: Sink>(
        &mut self,
        player: PlayerId,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let (Some(place), Some(&weekly)) = (self.map.weekly_index(index), self.weekly(index))
        else {
            return AdventureOutcome::default();
        };
        let anchor = self.object_at(index).anchor;
        let marked = (0..self.players)
            .map(PlayerId::new)
            .filter(|&p| allied(p, player) && !self.visited_by(index, p))
            .fold(Viewers::NONE, Viewers::with);
        let empty = weekly.amount == 0;
        if marked != Viewers::NONE || !empty {
            self.weekly.iter_mut().for_each(|block| {
                let entry = &mut Arc::make_mut(block)[place];
                entry.marks |= marked.bits();
                entry.amount = 0;
            });
        }
        if marked != Viewers::NONE {
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::ObjectMarked {
                    object_type,
                    anchor,
                },
                seen_by: marked,
            });
        }
        if empty {
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::ObjectEmpty {
                    object_type,
                    anchor,
                },
                seen_by: Viewers::NONE.with(player),
            });
            return AdventureOutcome::default();
        }
        AdventureOutcome {
            gains: vec![(weekly.resource, weekly.amount)],
            ..AdventureOutcome::default()
        }
    }
}
