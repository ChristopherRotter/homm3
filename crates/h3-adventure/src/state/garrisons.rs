//! The map's garrisons (ADV-GAR-002, ADV-GAR-004, ADV-GAR-005; E08 T114a1; increment 12's design
//! note §5 to §7 and §14; survey `E08-garrisons-bonuses.md` §3 to §5 and §7): a garrison's army as
//! it stands, the visit of another team's or a neutral one (its army fought at once, unsplit, else
//! its flag), its battle's end (the survivors kept, or the flag after the winner's levels) and the
//! anti-magic battle on its entrance; since E08 T114a2 the exchange its visit opens (ADV-GAR-003,
//! increment 12's design note §13). A garrison of the mover's team is crossed on the way
//! (`AdventureState::visit_at`) and visited where the move ends or by a revisit.

use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::{AdventureState, allied};
use crate::command::{AdventureOutcome, Defender, Exchange, Sink};
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::{Holds, Pos};

impl AdventureState {
    /// The army of the garrison at object index `index` as it stands (ADV-GAR-002, E08 T114a1;
    /// increment 12's design note §5): what its last battle left ([`Self::army_left`]: none after
    /// a won battle or a draw), else the map's (`Holds::Garrison`); none for an object that is no
    /// map garrison. Its stacks by slot.
    pub fn garrison_army(&self, index: usize) -> &[ArmyStack] {
        match (&self.object_at(index).holds, self.army_left(index)) {
            (Holds::Garrison { .. }, Some(left)) => left,
            (Holds::Garrison { army, .. }, None) => army,
            _ => &[],
        }
    }

    /// The removable flag of the map garrison at object index `index` (ADV-GAR-001, ADV-GAR-003;
    /// the map's, `Holds::Garrison`): whether its owner may take its troops back in its exchange
    /// (the one owner rule: a non-removable one's bar owner is nobody). None for an object that
    /// is no map garrison.
    pub fn garrison_removable(&self, index: usize) -> Option<bool> {
        match self.object(index)?.holds {
            Holds::Garrison { removable, .. } => Some(removable),
            _ => None,
        }
    }

    /// Whether a battle at `tile` is fought without magic (ADV-GAR-005, E08 T114a1; increment 12's
    /// design note §14; D-60 the original): the tile is an anti-magic garrison's entrance, so
    /// the garrison's own battle and that of a hero standing in it whom another attacks (the
    /// original's `tile_special_ground` 0x4FCE20 answers both, survey §4; the battle's tile is the
    /// defender's, batch E12's E12-4). The game sets the battle's `anti_magic` from it
    /// (BAT-TER-002, BAT-TER-004: neither side casts). A hero attacking out of one fights with
    /// magic: its tile is not the battle's.
    // rule: ADV-GAR-005
    pub fn anti_magic(&self, part: &AdventureRuleset, tile: Pos) -> bool {
        self.tile_object(part, tile)
            .is_some_and(|(_, _, ty)| matches!(ty.kind, ObjectKind::Garrison { anti_magic: true }))
    }

    /// The visit of the garrison at object index `index`, of type `object_type`, by `player`'s
    /// hero `hero` on its entrance (ADV-GAR-002, E08 T114a1; increment 12's design note §6; survey
    /// §3, the original's `visit_object` case 33, 0x4A9061): a garrison of the visitor's team
    /// opens the exchange (ADV-GAR-003, E08 T114a2: [`Self::exchange`]); another team's or a
    /// neutral one holding a creature is fought at once, **with no question**
    /// ([`Self::garrison_battle`]); one holding none is flagged for the visitor's owner
    /// ([`Self::flag`], ADV-OWN-001: its reveal of the flag radius), then the exchange.
    // rule: ADV-GAR-002, ADV-GAR-003, ADV-OWN-001
    pub(super) fn visit_garrison<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        object: (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        match self.owner(object.0) {
            Some(owner) if allied(owner, player) => self.exchange(hero, object),
            _ if self.garrison_army(object.0).is_empty() => {
                self.flag(rules, object, player, sink);
                self.exchange(hero, object)
            }
            _ => self.garrison_battle(rules.1, hero, object.0),
        }
    }

    /// The exchange of the hero `hero` with the garrison at object index `index`, of type
    /// `object_type` (ADV-GAR-003, GAM-DEC-009; E08 T114a2, increment 12's design note §13; the
    /// original's `garrison_exchange_dialog` 0x5D1130 at the visit, survey §6): the outcome the
    /// game opens as the hero's owner's decision, the garrison by index, type and anchor.
    // rule: ADV-GAR-003
    pub(super) fn exchange(
        &self,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
    ) -> AdventureOutcome {
        AdventureOutcome {
            exchange: Some(Exchange {
                hero,
                garrison: index,
                object_type,
                anchor: self.object_at(index).anchor,
            }),
            ..AdventureOutcome::default()
        }
    }

    /// The battle of the hero `hero` against the garrison at object index `index` (GAM-BAT-001
    /// amended, ADV-GAR-002; E08 T114a1; increment 12's design note §6 and §14; survey §4): its
    /// army as it stands, **unsplit** (the original's case 33 calls `adventure_battle` with the
    /// record's army, never `monster_battle`: a neutral single stack is not split, batch E12's
    /// E12-6), an owned garrison's as its owner's side without a hero ([`Defender::Garrison`],
    /// played by its owner outside its turn, as an owned mine's guards), a neutral one's unowned
    /// ([`Defender::Guards`]). Fought on the entrance, where its visitor stands, as any battle
    /// there ([`Self::battle_at`]: its terrain and special ground, no case of its own). None when
    /// the hero is not on the map.
    // rule: GAM-BAT-001, ADV-GAR-002
    fn garrison_battle(
        &self,
        part: &AdventureRuleset,
        hero: HeroId,
        index: usize,
    ) -> AdventureOutcome {
        let army = self.garrison_army(index).to_vec();
        let defender = match self.owner(index) {
            Some(owner) => Defender::Garrison { owner, army },
            None => Defender::Guards { army },
        };
        AdventureOutcome {
            battle: self
                .hero(hero)
                .map(|attacker| self.battle_at(part, attacker, defender, attacker.pos)),
            ..AdventureOutcome::default()
        }
    }

    /// The end of the battle against the garrison at object index `index`, fought by its visitor
    /// `hero` at `tile` (ADV-GAR-002, E08 T114a1; increment 12's design note §6; survey §4):
    /// after the visitor's side `won` the garrison's army is gone and its flag comes after the
    /// winner's levels ([`AdventureOutcome::reward_after`], GAM-DEC-007: the original's visit
    /// flags it once its battle, levels included, returns; [`Self::give_won`], which then opens
    /// the exchange, E08 T114a2); else the defender's `survivors` stay as its army, by slot (none
    /// after a draw: its next visitor flags it), its owner unchanged. The army's view is told
    /// with T114b (ADV-GAR-006).
    // rule: ADV-GAR-002
    pub(super) fn end_garrison_battle(
        &mut self,
        index: usize,
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
    ) -> AdventureOutcome {
        let left = if won { Vec::new() } else { survivors.to_vec() };
        self.army_left.set(index, Some(left));
        AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        }
    }

    /// The garrison visited at `tile`, with its type, if one is ([`Self::give_won`]).
    pub(super) fn garrison_at(
        &self,
        part: &AdventureRuleset,
        tile: Pos,
    ) -> Option<(usize, ObjectTypeId)> {
        self.tile_object(part, tile)
            .filter(|(_, _, ty)| matches!(ty.kind, ObjectKind::Garrison { .. }))
            .map(|(index, id, _)| (index, id))
    }
}
