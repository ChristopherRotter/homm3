//! The War Machine Factories on the map (E08 T134b2; increment 14's design note §5, §10, §11, §16;
//! survey `docs/rules/surveys/E08-trades.md` §14): the visit's hire question naming the machines
//! (ADV-WMF-001), and the hire dialog's factory side (ADV-HIR-002 amended): the stock, one of each
//! machine the dialog's hero lacks (the original's `recruit_dialog_select` 0x5503A0: the count 1
//! less whether the hero holds its artifact, at least 0), the room of a machine's artifact, and
//! the purchase of one, whose artifact the game gives (`HeroChange::AddArtifact`: the hero store is
//! the game's, GAM-DEC-008, as a market's purchase, ADV-MKT-007). The dialog, its check and its
//! purchase are `dwellings.rs`' (one hire dialog for every source). *Agent's call:* the factory
//! keeps no count; the original's three machines share one count of 4 000, lowered by each
//! purchase, which no game reaches.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, CreatureTypeId, Ruleset};
use h3_contracts::HeroId;
use h3_heroes::{HeroChange, HeroDetails, check_change};

use super::AdventureState;
use crate::command::{AdventureOutcome, HireTarget, OfferKind};
use crate::movement::details_of;
use crate::ruleset::{ObjectTypeId, WarMachineFactory};

impl AdventureState {
    /// The visit of the War Machine Factory at object index `index`, of type `object_type`, by
    /// hero `hero` (ADV-WMF-001; E08 T134b2, increment 14's design note §10; survey §14, the
    /// original's `visit_object` case 106 0x4AA622): at every visit, by any player's hero, the hire
    /// question naming its machines in its order ([`OfferKind::Hire`]: the original's ADVEVENT
    /// 157), whose yes opens the hire dialog of them (`accept_hire`, ADV-HIR-002). No mark, no
    /// flag, nothing free; declined, nothing changes.
    // rule: ADV-WMF-001
    pub(super) fn ask_factory(
        &self,
        hero: HeroId,
        object: (usize, ObjectTypeId),
        factory: &WarMachineFactory,
    ) -> AdventureOutcome {
        let creatures = factory
            .machines
            .iter()
            .map(|&(machine, _)| machine)
            .collect();
        self.ask(hero, object, OfferKind::Hire { creatures })
    }
}

/// The hero a factory's dialog sells to: its target's, none for a town's garrison (no factory's
/// dialog has one).
pub(super) fn target_hero(target: &HireTarget) -> Option<HeroId> {
    match *target {
        HireTarget::Hero(hero) => Some(hero),
        HireTarget::Garrison(_) => None,
    }
}

/// Whether `details` holds `artifact`, worn or carried (the original's `hero_has_artifact`
/// 0x4D91B0).
fn holds(details: &HeroDetails, artifact: ArtifactTypeId) -> bool {
    let worn = details.worn.iter().flatten();
    worn.chain(&details.backpack)
        .any(|item| item.artifact == artifact)
}

/// A factory's stock for the hero `target` names (ADV-WMF-001, ADV-HIR-002; E08 T134b2, increment
/// 14's design note §11; survey §14, the original's `recruit_dialog_select` 0x5503A0): each of its
/// machines in its order, 1 while the hero holds none of its artifact, else 0. None for a target
/// that is no hero of the store.
pub(super) fn machine_stock(
    heroes: &[Arc<HeroDetails>],
    target: &HireTarget,
    factory: &WarMachineFactory,
) -> Option<Vec<(CreatureTypeId, u32)>> {
    let details = target_hero(target).and_then(|hero| details_of(heroes, hero))?;
    let stock = factory
        .machines
        .iter()
        .map(|&(machine, artifact)| (machine, u32::from(!holds(details, artifact))));
    Some(stock.collect())
}

/// The room of a factory's machine for the hero `target` names (ADV-HIR-002; E08 T134b2,
/// increment 14's design note §11): 1 when the hero can receive the machine's `artifact`
/// ([`HeroChange::AddArtifact`]'s check, HER-CHG-002: a position that takes it free; a war
/// machine's is never carried, so in the original packs its own position), else 0.
pub(super) fn machine_room(
    (core, heroes): (&Ruleset, &[Arc<HeroDetails>]),
    target: &HireTarget,
    artifact: ArtifactTypeId,
) -> u32 {
    let details = target_hero(target).and_then(|hero| details_of(heroes, hero));
    let change = HeroChange::AddArtifact(artifact.into());
    u32::from(details.is_some_and(|details| check_change(details, core, change).is_ok()))
}
