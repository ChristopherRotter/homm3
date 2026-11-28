//! A stack transformed at a town's Skeleton Transformer (TWN-TRF-001; E09 T59b2, the town epic's
//! design note §113; survey `E09-trades.md` §7): the stack's creature replaced in place, its count
//! kept, in the army the game names (a hero's, the visitor or the garrison hero, or the town's
//! garrison). The town part checks the Transformer and names the target
//! (`h3_towns::check_transformer`); the game checks the town, its owner, the army and the stack
//! and tells the slot (`h3_game`'s `town/transform.rs`).

use h3_content::CreatureTypeId;
use h3_contracts::{ArmyStack, HeroId, TownId};

use crate::army::{ArmyRef, ArmySlot};
use crate::state::AdventureState;

impl AdventureState {
    /// Transforms the stack in `slot` of `hero`'s army, a hero on the map or a town's garrison
    /// hero, or with `hero` none of `town`'s garrison, into `creature` (TWN-TRF-001; design §113;
    /// survey §7.5, the original's `transformer_convert` 0x566C20 setting the slot's type, its
    /// count unchanged): which the game checked for this state as it stands (the army, a stack
    /// in the slot, a creature other than its own). Nothing merged, the army's last stack
    /// allowed (§7.7). The slot the game tells (`StackChanged`); no adventure event.
    // rule: TWN-TRF-001
    pub fn transform_stack(
        &mut self,
        town: TownId,
        (hero, slot): (Option<HeroId>, u8),
        creature: CreatureTypeId,
    ) -> ArmySlot {
        let set = |army: &mut [ArmyStack]| {
            army.iter_mut()
                .filter(|stack| stack.slot == slot)
                .for_each(|stack| stack.creature = creature);
        };
        if let Some(hero) = hero {
            // The checked hero: none only defensively, so no early return.
            self.hero_or_held_mut(hero)
                .into_iter()
                .for_each(|on| set(&mut on.army));
            return ArmySlot {
                army: ArmyRef::Hero(hero),
                slot,
            };
        }
        // The checked town's garrison (the map's town: written as a captured one's, E09 T8a2);
        // none only defensively, so no branch.
        let index = self.map().town_object(town);
        index.into_iter().for_each(|index| {
            let mut army = self.town_garrison(index).unwrap_or_default().to_vec();
            set(&mut army);
            self.set_army_left(index, army);
        });
        ArmySlot {
            army: ArmyRef::Garrison(town),
            slot,
        }
    }
}
