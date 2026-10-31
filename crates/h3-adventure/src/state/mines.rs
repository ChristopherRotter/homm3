//! The flags and the mines' visits (ADV-OWN-001, ADV-MIN-002; E08 T84a; increment 9's design note
//! §7, §13; survey `E08-economy.md` §2.3, §2.4, §3): a visit flags a neutral object or another
//! team's for its visitor's owner, revealing around its entrance; an own mine opens its Leave
//! Guards dialog, an ally's does nothing, another's guards are asked, then fought, and a win
//! flags it. The Lighthouses', Shipyards' and the weekly objects' visits come with T84b.

use alloc::vec;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::{AdventureState, allied};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, Defender, Garrison, Offer, OfferKind,
    Sink, Viewers,
};
use crate::fog::reveal;
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::Pos;

/// Whether `kind` is a mine's or an abandoned mine's.
const fn is_mine(kind: &ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. }
    )
}

impl AdventureState {
    /// Whether the object at object index `index` is a mine or an abandoned mine.
    pub(super) fn is_mine(&self, part: &AdventureRuleset, index: usize) -> bool {
        self.object_at(index)
            .object_type
            .and_then(|id| part.object_type(id))
            .is_some_and(|ty| is_mine(&ty.kind))
    }

    /// Flags the ownable object at object index `index`, of type `object_type`, for `player`
    /// (ADV-OWN-001, E08 T84a; increment 9's design note §7; survey `E08-economy.md` §2.3, the
    /// original's `set_mine_owner` 0x4C66E0 and `set_shipyard_owner` 0x4C6A30): its owner
    /// written, told to the players that see the object and to its new owner
    /// ([`AdventureEventKind::ObjectFlagged`]), then the tiles within the part's flag radius of
    /// its entrance (E08 decision 217) explored for the new owner (its team from E10; ADV-FOG-002's
    /// disk), told to it alone with the number it had not explored, as an observatory's
    /// ([`AdventureEventKind::TilesRevealed`]). The caller has checked that the object is ownable
    /// and not of `player`'s team; nothing is paid.
    // rule: ADV-OWN-001
    pub(super) fn flag<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        (index, object_type): (usize, ObjectTypeId),
        player: PlayerId,
        sink: &mut S,
    ) {
        let place = self.map.ownable_index(index);
        if let Some((owners, place)) = self.owners.as_mut().zip(place) {
            alloc::sync::Arc::make_mut(owners)[place] = Some(player);
        }
        let object = self.object_at(index);
        let (anchor, entrance) = (object.anchor, object.entrance(&self.map));
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::ObjectFlagged {
                object_type,
                anchor,
                owner: player,
            },
            seen_by: self.object_viewers(core, object).with(player),
        });
        if let Some(entrance) = entrance {
            let grid = &mut self.explored[usize::from(player.raw())];
            let before = grid.count();
            reveal(grid, entrance, part.params().flag_radius());
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::TilesRevealed {
                    object_type,
                    count: grid.count() - before,
                },
                seen_by: Viewers::NONE.with(player),
            });
        }
    }

    /// The visit of the mine at object index `index`, of type `object_type`, by `player`'s hero
    /// `hero` on its entrance (ADV-OWN-001, ADV-MIN-002, E08 T84a; increment 9's design note §7;
    /// survey `E08-economy.md` §2.4, §3.1, the original's `visit_mine` 0x4A39A0): a mine of the
    /// visitor's team does nothing but, the visitor's own, open its Leave Guards dialog (the
    /// outcome's [`AdventureOutcome::garrison`], which the game opens; every player's hero, E08
    /// decision 204); a neutral mine or another team's asks its question while its guards stand
    /// ([`OfferKind::Fight`]: the original's ADVEVENT 84 for a neutral one, 187 for an owned one;
    /// the client's text), else is flagged at once ([`Self::fight_for_mine`]).
    // rule: ADV-OWN-001, ADV-MIN-002
    pub(super) fn visit_mine<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let anchor = self.object_at(index).anchor;
        let guards = self.mine_guards(rules.1, index);
        match self.owner(index) {
            // Until E10's teams a team is its player alone: every mine of the team is the
            // visitor's own (an ally's would do nothing, survey §2.4).
            Some(owner) if allied(owner, player) => AdventureOutcome {
                garrison: (owner == player).then_some(Garrison {
                    hero,
                    mine: index,
                    object_type,
                    anchor,
                    guards,
                }),
                ..AdventureOutcome::default()
            },
            _ if !guards.is_empty() => AdventureOutcome {
                offer: Some(Offer {
                    hero,
                    object_type,
                    anchor,
                    kind: OfferKind::Fight,
                }),
                ..AdventureOutcome::default()
            },
            _ => self.fight_for_mine(rules, (player, hero), (index, object_type), sink),
        }
    }

    /// The mine at object index `index`, of type `object_type`, fought for or flagged by
    /// `player`'s hero `hero` (ADV-MIN-002, E08 T84a; its question accepted, or a visit of a mine
    /// without guards): while its guards stand, their battle ([`Self::mine_battle`]); else the
    /// flag ([`Self::flag`]).
    // rule: ADV-MIN-002
    pub(super) fn fight_for_mine<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        object: (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        if self.mine_guards(rules.1, object.0).is_empty() {
            self.flag(rules, object, player, sink);
            return AdventureOutcome::default();
        }
        self.mine_battle(rules.1, hero, object.0)
    }

    /// The battle of the hero `hero` against the guards of the mine at object index `index` as
    /// they stand (GAM-BAT-001 amended, ADV-MIN-002; E08 T84a; increment 9's design note §13;
    /// survey `E08-economy.md` §3.2): a neutral mine's single stack defends as a wandering
    /// monster ([`Defender::Monster`], split by the game, `ADV-QUIRK-6`'s draws; D-47, batch E9's
    /// E9-1: the original's `monster_battle`), a neutral mine's other guards unsplit and unowned
    /// ([`Defender::Guards`]), an owned mine's as its owner's side ([`Defender::Garrison`]: the
    /// original's `adventure_battle` with the mine's owner, batch E9's E9-2). Fought on the mine's
    /// entrance, where its visitor stands: on its type's battle terrain when it has one (the
    /// subterranean of the crystal, gold and abandoned mines; the original's
    /// `battle_setup_battlefield` 0x4643B0 tests them after a boat and the underground, before the
    /// coast), a special ground there replacing the battlefield as always; else as any battle
    /// there ([`Self::battle_at`]). None when the hero is not on the map.
    // rule: GAM-BAT-001, ADV-MIN-002
    fn mine_battle(&self, part: &AdventureRuleset, hero: HeroId, index: usize) -> AdventureOutcome {
        let guards = self.mine_guards(part, index);
        let defender = match (self.owner(index), &guards[..]) {
            (Some(owner), _) => Defender::Garrison {
                owner,
                army: guards,
            },
            (None, [stack]) => Defender::Monster {
                creature: stack.creature,
                count: stack.count,
            },
            (None, _) => Defender::Guards { army: guards },
        };
        let terrain = self
            .object_at(index)
            .object_type
            .and_then(|id| part.object_type(id))
            .and_then(|ty| ty.kind.battle_terrain());
        let battle = self.hero(hero).map(|attacker| {
            let mut start = self.battle_at(part, attacker, defender, attacker.pos);
            if let Some(terrain) = terrain {
                start.terrain = terrain;
                start.battlefield = self.special_ground(part, attacker.pos);
            }
            start
        });
        AdventureOutcome {
            battle,
            ..AdventureOutcome::default()
        }
    }

    /// The end of the battle against the guards of the mine at object index `index`, fought by
    /// the visitor's hero `hero` at `tile` (ADV-MIN-002, E08 T84a; increment 9's design note §13;
    /// survey `E08-economy.md` §3.2): after the visitor's side `won` the guards are gone and the
    /// flag comes after the winner's levels ([`AdventureOutcome::reward_after`], GAM-DEC-007: the
    /// original's `visit_mine` flags once its battle, levels included, returns;
    /// [`Self::give_won`]); else the guards' `survivors` stay on the mine (none after a draw:
    /// the next visit flags it), a neutral single stack's summed into its own slot as its own
    /// creature (the original writes the count left into slot 0, A-29 fixed: the stack's slot;
    /// every survivor counted, as a wandering monster's, A-6 fixed), others by slot as they
    /// stand; its owner is unchanged. A change of their hint is told ([`Self::write_guards`]).
    // rule: ADV-MIN-002
    pub(super) fn end_mine_battle<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        (index, object_type): (usize, ObjectTypeId),
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
        sink: &mut S,
    ) -> AdventureOutcome {
        let guards = self.mine_guards(part, index);
        let left = match (won, self.owner(index), &guards[..]) {
            (true, ..) => Vec::new(),
            (false, None, [stack]) => {
                let count = survivors
                    .iter()
                    .fold(0u32, |sum, survivor| sum.saturating_add(survivor.count));
                if count == 0 {
                    Vec::new()
                } else {
                    vec![ArmyStack { count, ..*stack }]
                }
            }
            (false, ..) => survivors.to_vec(),
        };
        self.write_guards((core, part), (index, object_type), left, sink);
        AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        }
    }

    /// The flag of the mine at object index `index`, of type `object_type`, its guards beaten by
    /// the hero `hero` (ADV-MIN-002, E08 T84a; [`Self::give_won`], after the winner's levels):
    /// flagged for the hero's owner. Nothing when the hero is no longer on the map.
    pub(super) fn flag_won<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        hero: HeroId,
        object: (usize, ObjectTypeId),
        sink: &mut S,
    ) {
        if let Some(player) = self.hero(hero).map(|h| h.owner) {
            self.flag(rules, object, player, sink);
        }
    }

    /// Writes the guards its owner's Leave Guards dialog left on the mine at object index `index`
    /// (ADV-MIN-002, GAM-DEC-004 amended; E08 T84a; survey `E08-economy.md` §3.3): the group by
    /// slot as the dialog closed it, none at all too (an abandoned mine's drawn Troglodytes then do
    /// not come back), a change of their hint told ([`Self::write_guards`]). The game passes its
    /// own dialog's group, which the army commands kept within the army's rules (ADV-ARM-001 to
    /// -005). Nothing for an object that is no mine.
    // rule: ADV-MIN-002
    pub fn set_guards<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        index: usize,
        guards: Vec<ArmyStack>,
        sink: &mut S,
    ) {
        let mine = self
            .object_at(index)
            .object_type
            .filter(|_| self.is_mine(part, index));
        if let Some(object_type) = mine {
            self.write_guards((core, part), (index, object_type), guards, sink);
        }
    }

    /// Writes `guards` on the mine at object index `index`, of type `object_type` (E08 T84a): told
    /// to the players that see the mine when the hint every viewer sees of them changes
    /// ([`AdventureEventKind::GuardsChanged`]; ADV-OWN-003: their creature when all stacks are of
    /// one, the word of their total count).
    // rule: ADV-OWN-003
    fn write_guards<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        (index, object_type): (usize, ObjectTypeId),
        guards: Vec<ArmyStack>,
        sink: &mut S,
    ) {
        let hint = |s: &Self| {
            s.guards_view(part, index, false)
                .map(|view| (view.creature, view.count_word))
        };
        let before = hint(self);
        self.army_left.set(index, Some(guards));
        let after = hint(self);
        if after != before {
            let object = self.object_at(index);
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::GuardsChanged {
                    object_type,
                    anchor: object.anchor,
                    creature: after.and_then(|(creature, _)| creature),
                    count_word: after.map(|(_, word)| word),
                },
                seen_by: self.object_viewers(core, object),
            });
        }
    }

    /// The mine or abandoned mine visited at `tile`, with its type, if one is ([`Self::give_won`],
    /// [`Self::end_unowned_battle`]).
    pub(super) fn mine_at(
        &self,
        part: &AdventureRuleset,
        tile: Pos,
    ) -> Option<(usize, ObjectTypeId)> {
        self.tile_object(part, tile)
            .filter(|(_, _, ty)| is_mine(&ty.kind))
            .map(|(index, id, _)| (index, id))
    }
}
