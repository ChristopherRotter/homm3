//! The whole game's events on the wire, projected for their viewer (E16 M5 T3b): [`events`]
//! keeps what `h3_game::project` shows the viewer and maps it, every variant matched without a
//! wildcard and every field named (ADR-0018 point 5 as amended). A projected event is the
//! viewer's alone, so its sets of viewers are not sent (`seen_by: _`).

use alloc::vec::Vec;

use h3_adventure::{AdventureEvent as EngineAdventure, AdventureEventKind, ShipyardBlock as Block};
use h3_content::Formation as EngineFormation;
use h3_game::{GameEvent as EngineEvent, HeroChange as EngineChange, Viewer};
use h3_heroes::OfferLevels as EngineLevels;
use h3_kernel::handle::Handle;

use super::command::{artifact_place, slot};
use super::town_buildings;
use super::{army_stack, artifact, fort_level, garrison_stack, month_kind, pos, primary_skill};
use crate::map::{from_engine, outcome};
use crate::wire::game::{
    AdventureEvent, Formation, GameEvent, HeroChange, HireTarget, OfferLevels, ShipyardBlock,
};

/// The events `viewer` may see, in order, on the wire (`h3_game::project`); the others are
/// dropped.
pub fn events(events: &[EngineEvent], viewer: Viewer) -> Vec<GameEvent> {
    events
        .iter()
        .filter_map(|e| h3_game::project(e, viewer))
        .map(|e| event(&e))
        .collect()
}

/// A projected event on the wire.
fn event(e: &EngineEvent) -> GameEvent {
    match *e {
        EngineEvent::TurnEnded { player } => GameEvent::TurnEnded {
            player: player.raw(),
        },
        EngineEvent::DayStarted { day } => GameEvent::DayStarted { day },
        EngineEvent::WeekStarted { week } => GameEvent::WeekStarted { week },
        EngineEvent::MonthStarted { month, kind } => GameEvent::MonthStarted {
            month,
            kind: month_kind(kind),
        },
        EngineEvent::TurnStarted { player } => GameEvent::TurnStarted {
            player: player.raw(),
        },
        EngineEvent::TimedEventFired { player, event } => GameEvent::TimedEventFired {
            player: player.raw(),
            event,
        },
        EngineEvent::Adventure(EngineAdventure {
            kind,
            // The viewer alone in a projection.
            seen_by: _,
        }) => GameEvent::Adventure {
            event: adventure(kind),
        },
        EngineEvent::ResourceGained {
            player,
            resource,
            amount,
        } => GameEvent::ResourceGained {
            player: player.raw(),
            resource: resource.raw(),
            amount,
        },
        EngineEvent::OfferOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::OfferOpened {
            player: player.raw(),
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        EngineEvent::OfferRefused {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::OfferRefused {
            player: player.raw(),
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        EngineEvent::OfferAnswered {
            player,
            object_type,
            anchor,
            accepted,
        } => GameEvent::OfferAnswered {
            player: player.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
            accepted,
        },
        EngineEvent::BattleStarted {
            heroes,
            tile,
            seen_by: _,
        } => GameEvent::BattleStarted {
            heroes: heroes.map(|h| h.map(|h| h.raw())),
            tile: pos(tile),
        },
        EngineEvent::Battle { event, seen_by: _ } => GameEvent::Battle {
            event: from_engine::event(&event.kind),
        },
        EngineEvent::BattleEnded {
            heroes,
            result,
            seen_by: _,
        } => GameEvent::BattleEnded {
            heroes: heroes.map(|h| h.map(|h| h.raw())),
            result: outcome(result),
        },
        EngineEvent::HeroChanged {
            player,
            hero,
            change,
        } => GameEvent::HeroChanged {
            player: player.raw(),
            hero: hero.raw(),
            change: hero_change(change),
        },
        EngineEvent::CreaturesRaised {
            player,
            hero,
            creature,
            count,
        } => GameEvent::CreaturesRaised {
            player: player.raw(),
            hero: hero.raw(),
            creature: creature.raw(),
            count,
        },
        EngineEvent::LevelUpOpened {
            player,
            hero,
            level,
            primary,
            offers,
        } => GameEvent::LevelUpOpened {
            player: player.raw(),
            hero: hero.raw(),
            level,
            primary: primary_skill(primary),
            offers: offers.map(Handle::raw),
        },
        EngineEvent::LevelGained {
            player,
            hero,
            level,
            primary,
            skill,
        } => GameEvent::LevelGained {
            player: player.raw(),
            hero: hero.raw(),
            level,
            primary: primary_skill(primary),
            skill: skill.map(Handle::raw),
        },
        EngineEvent::ResourceLost {
            player,
            resource,
            amount,
        } => GameEvent::ResourceLost {
            player: player.raw(),
            resource: resource.raw(),
            amount,
        },
        EngineEvent::ResourcePaid {
            player,
            resource,
            amount,
        } => GameEvent::ResourcePaid {
            player: player.raw(),
            resource: resource.raw(),
            amount,
        },
        EngineEvent::StackChanged { player, at, stack } => GameEvent::StackChanged {
            player: player.raw(),
            at: slot(at),
            stack: stack.map(army_stack),
        },
        EngineEvent::FormationSet {
            player,
            hero,
            formation,
        } => GameEvent::FormationSet {
            player: player.raw(),
            hero: hero.raw(),
            formation: match formation {
                EngineFormation::Loose => Formation::Loose,
                EngineFormation::Tight => Formation::Tight,
            },
        },
        EngineEvent::TacticsSet { player, hero, on } => GameEvent::TacticsSet {
            player: player.raw(),
            hero: hero.raw(),
            on,
        },
        EngineEvent::MeetingOpened { player, heroes } => GameEvent::MeetingOpened {
            player: player.raw(),
            heroes: heroes.map(h3_contracts::HeroId::raw),
        },
        EngineEvent::ArmyDialogOpened {
            player,
            hero,
            stack,
        } => GameEvent::ArmyDialogOpened {
            player: player.raw(),
            hero: hero.raw(),
            stack: army_stack(stack),
        },
        EngineEvent::LeaveGuardsOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::LeaveGuardsOpened {
            player: player.raw(),
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        EngineEvent::ChoiceOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::ChoiceOpened {
            player: player.raw(),
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        EngineEvent::UniversityOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::UniversityOpened {
            player: player.raw(),
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        EngineEvent::HireOpened {
            player,
            target,
            object_type,
            anchor,
        } => GameEvent::HireOpened {
            player: player.raw(),
            target: match target {
                h3_game::HireTarget::Hero(hero) => HireTarget::Hero { hero: hero.raw() },
            },
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        EngineEvent::DecisionClosed { player } => GameEvent::DecisionClosed {
            player: player.raw(),
        },
        EngineEvent::GroupLost {
            player,
            hero,
            stack,
        } => GameEvent::GroupLost {
            player: player.raw(),
            hero: hero.raw(),
            stack: army_stack(stack),
        },
        EngineEvent::TownChanged {
            player,
            town,
            owner,
            fort,
            capitol,
            built_today,
            level,
            slots,
            hall,
            buildings,
        } => GameEvent::TownChanged {
            player: player.raw(),
            town: town.raw(),
            owner: owner.map(h3_contracts::PlayerId::raw),
            fort: fort_level(fort),
            capitol,
            built_today,
            level,
            slots,
            hall,
            buildings: buildings.map(town_buildings),
        },
        EngineEvent::TownStackSeen {
            player,
            town,
            stack,
            slot,
        } => GameEvent::TownStackSeen {
            player: player.raw(),
            town: town.raw(),
            stack: garrison_stack(stack),
            slot,
        },
        EngineEvent::TownIncomeSeen {
            player,
            town,
            resource,
            amount,
        } => GameEvent::TownIncomeSeen {
            player: player.raw(),
            town: town.raw(),
            resource: resource.raw(),
            amount,
        },
        EngineEvent::TownBuilt {
            player,
            town,
            building,
            event,
        } => GameEvent::TownBuilt {
            player: player.raw(),
            town: town.raw(),
            building: building.raw(),
            event,
        },
        EngineEvent::TownEventFired {
            player,
            town,
            event,
        } => GameEvent::TownEventFired {
            player: player.raw(),
            town: town.raw(),
            event,
        },
    }
}

/// A projected adventure event on the wire: a step's and a jump's ends are already `None` where
/// the viewer does not see them, their viewer sets the viewer alone.
fn adventure(kind: AdventureEventKind) -> AdventureEvent {
    use AdventureEventKind as K;
    match kind {
        K::HeroStepped {
            hero,
            from,
            to,
            seen_from: _,
            seen_to: _,
        } => AdventureEvent::HeroStepped {
            hero: hero.raw(),
            from: from.map(pos),
            to: to.map(pos),
        },
        K::ObjectRemoved {
            object_type,
            anchor,
        } => AdventureEvent::ObjectRemoved {
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        K::TilesRevealed { object_type, count } => AdventureEvent::TilesRevealed {
            object_type: object_type.raw(),
            count,
        },
        K::KeyGained { key } => AdventureEvent::KeyGained { key },
        K::HeroRemoved { hero, pos: p } => AdventureEvent::HeroRemoved {
            hero: hero.raw(),
            pos: pos(p),
        },
        K::CreaturesJoined {
            hero,
            creature,
            count,
        } => AdventureEvent::CreaturesJoined {
            hero: hero.raw(),
            creature: creature.raw(),
            count,
        },
        K::EventFired { hero, tile } => AdventureEvent::EventFired {
            hero: hero.raw(),
            tile: pos(tile),
        },
        K::MonsterChanged {
            object_type,
            anchor,
            count_word,
        } => AdventureEvent::MonsterChanged {
            object_type: object_type.raw(),
            anchor: pos(anchor),
            count_word,
        },
        K::Boarded { hero, boat, at } => AdventureEvent::Boarded {
            hero: hero.raw(),
            boat,
            at: pos(at),
        },
        K::Landed {
            hero,
            boat,
            at,
            facing,
        } => AdventureEvent::Landed {
            hero: hero.raw(),
            boat,
            at: pos(at),
            facing,
        },
        K::MessageRead {
            hero,
            object_type,
            anchor,
        } => AdventureEvent::MessageRead {
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        K::BoatRemoved { hero, boat, at } => AdventureEvent::BoatRemoved {
            hero: hero.raw(),
            boat,
            at: pos(at),
        },
        K::ObjectFlagged {
            object_type,
            anchor,
            owner,
        } => AdventureEvent::ObjectFlagged {
            object_type: object_type.raw(),
            anchor: pos(anchor),
            owner: owner.raw(),
        },
        K::TownCaptured {
            town,
            anchor,
            owner,
        } => AdventureEvent::TownCaptured {
            town: town.raw(),
            anchor: pos(anchor),
            owner: owner.raw(),
        },
        K::GuardsChanged {
            object_type,
            anchor,
            creature,
            count_word,
        } => AdventureEvent::GuardsChanged {
            object_type: object_type.raw(),
            anchor: pos(anchor),
            creature: creature.map(Handle::raw),
            count_word,
        },
        K::ObjectMarked {
            object_type,
            anchor,
        } => AdventureEvent::ObjectMarked {
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        K::ObjectEmpty {
            object_type,
            anchor,
        } => AdventureEvent::ObjectEmpty {
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        K::ShipyardBlocked {
            object_type,
            anchor,
            reason,
        } => AdventureEvent::ShipyardBlocked {
            object_type: object_type.raw(),
            anchor: pos(anchor),
            reason: match reason {
                Block::NoTile => ShipyardBlock::NoTile,
                Block::Taken => ShipyardBlock::Taken,
            },
        },
        K::BoatBuilt { tile, boat_type } => AdventureEvent::BoatBuilt {
            tile: pos(tile),
            boat_type,
        },
        K::NoneToRecruit {
            object_type,
            anchor,
            creature,
        } => AdventureEvent::NoneToRecruit {
            object_type: object_type.raw(),
            anchor: pos(anchor),
            creature: creature.raw(),
        },
        K::JoinRefused {
            hero,
            creature,
            count,
        } => AdventureEvent::JoinRefused {
            hero: hero.raw(),
            creature: creature.raw(),
            count,
        },
        K::MonsterAdded {
            object_type,
            anchor,
            creature,
            count_word,
        } => AdventureEvent::MonsterAdded {
            object_type: object_type.raw(),
            anchor: pos(anchor),
            creature: creature.raw(),
            count_word,
        },
        K::HeroTeleported {
            hero,
            from,
            to,
            seen_from: _,
            seen_to: _,
        } => AdventureEvent::HeroTeleported {
            hero: hero.raw(),
            from: from.map(pos),
            to: to.map(pos),
        },
        K::LeadsNowhere {
            hero,
            object_type,
            anchor,
        } => AdventureEvent::LeadsNowhere {
            hero: hero.raw(),
            object_type: object_type.raw(),
            anchor: pos(anchor),
        },
        K::WhirlpoolLoss {
            hero,
            slot,
            creature,
            count,
        } => AdventureEvent::WhirlpoolLoss {
            hero: hero.raw(),
            slot,
            creature: creature.raw(),
            count,
        },
        K::MovementChanged {
            hero,
            movement,
            max_movement,
        } => AdventureEvent::MovementChanged {
            hero: hero.raw(),
            movement,
            max_movement,
        },
    }
}

/// A hero's change on the wire.
fn hero_change(change: EngineChange) -> HeroChange {
    match change {
        EngineChange::GainExperience(amount) => HeroChange::GainExperience { amount },
        EngineChange::SetSpellPoints(points) => HeroChange::SetSpellPoints { points },
        EngineChange::AddArtifact(a) => HeroChange::AddArtifact {
            artifact: artifact(a),
        },
        EngineChange::RemoveArtifact(place) => HeroChange::RemoveArtifact {
            place: artifact_place(place),
        },
        EngineChange::PlaceArtifact { artifact: a, at } => HeroChange::PlaceArtifact {
            artifact: artifact(a),
            at: artifact_place(at),
        },
        EngineChange::LearnSpell(spell) => HeroChange::LearnSpell { spell: spell.raw() },
        EngineChange::GainLevel {
            primary,
            learned,
            offered: EngineLevels { wisdom, school },
        } => HeroChange::GainLevel {
            primary: primary_skill(primary),
            learned: learned.map(Handle::raw),
            offered: OfferLevels { wisdom, school },
        },
        EngineChange::GainPrimary(primary) => HeroChange::GainPrimary { primary },
        EngineChange::RaiseSkill { skill, level } => HeroChange::RaiseSkill {
            skill: skill.raw(),
            level,
        },
        EngineChange::Bonus { morale, luck, mark } => HeroChange::Bonus { morale, luck, mark },
        EngineChange::ClearBonuses => HeroChange::ClearBonuses {},
    }
}
