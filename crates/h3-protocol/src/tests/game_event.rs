//! The whole game's events on the wire (E16 M5 T3b): every event a viewer may see maps
//! losslessly (a test-only inverse gives the engine's projected event back), the hidden ones are
//! dropped, for a hand-built list with every variant and for every viewer of games played at
//! random on the fixture maps; the events round-trip.

use std::collections::BTreeSet;
use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_adventure::{AdventureEvent, AdventureEventKind as K, ShipyardBlock, Viewers};
use h3_battle::BattleSide;
use h3_content::Formation as EngineFormation;
use h3_contracts::{ArmyStack as EngineStack, BattleResult, FortLevel, HeroId, PlayerId, TownId};
use h3_game::{
    ArmyRef, ArmySlot, ArtifactPlace, BuildingId, BuildingSet, GameEvent, GarrisonStackView,
    GuildHero, GuildPart, GuildPlayer, GuildRanking, GuildRow, GuildSlots, HeroChange, HireSeen,
    HireTarget, MonthKind, Position, ThievesGuildView, TownBuildingsView, Viewer, WeekKind,
};
use h3_heroes::{ArtifactInstance, OfferLevels, PrimarySkill as EnginePrimary};
use h3_kernel::handle::Handle;
use h3_kernel::rng::{Rng, StreamId};
use h3_sim::adventure::AdventureEnv;
use h3_sim::adventure::rollout::choose;

use super::game::{RULES, guild_back, hire_seen_back, hire_target_back, round_trips, set_back};
use crate::map::from_engine;
use crate::map::game::events;
use crate::wire::game as w;
use crate::wire::{Outcome, Side};

const P0: PlayerId = PlayerId::new(0);
const P1: PlayerId = PlayerId::new(1);

fn both() -> Viewers {
    Viewers::NONE.with(P0).with(P1)
}

// The inverse: the wire back to the engine's projected event for `viewer`, field by field (test
// code, independent of the mapping it checks). A battle's event is compared on the wire (its
// mapping is the battle wire's, `tests/mapping.rs`): the engine's is passed in.

fn pos_back(p: w::Pos) -> h3_adventure::Pos {
    h3_adventure::Pos::new(p.x, p.y, p.level)
}

fn stack_back(s: w::ArmyStack) -> EngineStack {
    EngineStack {
        slot: s.slot,
        creature: Handle::from_raw(s.creature),
        count: s.count,
    }
}

fn primary_back(p: w::PrimarySkill) -> EnginePrimary {
    match p {
        w::PrimarySkill::Attack => EnginePrimary::Attack,
        w::PrimarySkill::Defense => EnginePrimary::Defense,
        w::PrimarySkill::Power => EnginePrimary::Power,
        w::PrimarySkill::Knowledge => EnginePrimary::Knowledge,
    }
}

fn place_back(place: w::ArtifactPlace) -> ArtifactPlace {
    match place {
        w::ArtifactPlace::Worn { position } => ArtifactPlace::Worn(Position(position.into())),
        w::ArtifactPlace::Backpack { index } => ArtifactPlace::Backpack(index.into()),
    }
}

fn artifact_back(a: w::Artifact) -> ArtifactInstance {
    ArtifactInstance {
        artifact: Handle::from_raw(a.artifact),
        spell: a.spell.map(Handle::from_raw),
    }
}

fn slot_back(s: w::ArmySlot) -> ArmySlot {
    ArmySlot {
        army: match s.army {
            w::ArmyRef::Hero { hero } => ArmyRef::Hero(HeroId::new(hero)),
            w::ArmyRef::Group {} => ArmyRef::Group,
            w::ArmyRef::Garrison { town } => ArmyRef::Garrison(TownId::new(town)),
            w::ArmyRef::MapGarrison { garrison } => ArmyRef::MapGarrison(garrison),
        },
        slot: s.slot,
    }
}

fn side_back(side: Side) -> BattleSide {
    match side {
        Side::Attacker => BattleSide::Attacker,
        Side::Defender => BattleSide::Defender,
    }
}

fn result_back(o: Outcome) -> BattleResult {
    match o {
        Outcome::Victory { side } => BattleResult::Victory(side_back(side)),
        Outcome::Draw {} => BattleResult::Draw,
        Outcome::Retreat { side } => BattleResult::Retreat(side_back(side)),
        Outcome::Surrender { side } => BattleResult::Surrender(side_back(side)),
        other => panic!("not an engine result: {other:?}"),
    }
}

fn change_back(c: w::HeroChange) -> HeroChange {
    use w::HeroChange as C;
    match c {
        C::GainExperience { amount } => HeroChange::GainExperience(amount),
        C::SetSpellPoints { points } => HeroChange::SetSpellPoints(points),
        C::AddArtifact { artifact } => HeroChange::AddArtifact(artifact_back(artifact)),
        C::RemoveArtifact { place } => HeroChange::RemoveArtifact(place_back(place)),
        C::PlaceArtifact { artifact, at } => HeroChange::PlaceArtifact {
            artifact: artifact_back(artifact),
            at: place_back(at),
        },
        C::LearnSpell { spell } => HeroChange::LearnSpell(Handle::from_raw(spell)),
        C::GainLevel {
            primary,
            learned,
            offered,
        } => HeroChange::GainLevel {
            primary: primary_back(primary),
            learned: learned.map(Handle::from_raw),
            offered: OfferLevels {
                wisdom: offered.wisdom,
                school: offered.school,
            },
        },
        C::GainPrimary { primary } => HeroChange::GainPrimary(primary),
        C::RaiseSkill { skill, level } => HeroChange::RaiseSkill {
            skill: Handle::from_raw(skill),
            level,
        },
    }
}

/// The viewer alone where it sees a step's end, none where it does not.
fn seen(end: Option<w::Pos>, alone: Viewers) -> Viewers {
    if end.is_some() { alone } else { Viewers::NONE }
}

fn adventure_back(e: w::AdventureEvent, alone: Viewers) -> K {
    use w::AdventureEvent as A;
    let id = HeroId::new;
    let ty = Handle::from_raw;
    match e {
        A::HeroStepped { hero, from, to } => K::HeroStepped {
            hero: id(hero),
            from: from.map(pos_back),
            to: to.map(pos_back),
            seen_from: seen(from, alone),
            seen_to: seen(to, alone),
        },
        A::ObjectRemoved {
            object_type,
            anchor,
        } => K::ObjectRemoved {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        A::TilesRevealed { object_type, count } => K::TilesRevealed {
            object_type: ty(object_type),
            count,
        },
        A::KeyGained { key } => K::KeyGained { key },
        A::HeroRemoved { hero, pos } => K::HeroRemoved {
            hero: id(hero),
            pos: pos_back(pos),
        },
        A::GarrisonEntered { hero, town, pos } => K::GarrisonEntered {
            hero: id(hero),
            town: TownId::new(town),
            pos: pos_back(pos),
            named: alone,
        },
        A::GarrisonLeft { hero, town, pos } => K::GarrisonLeft {
            hero: id(hero),
            town: TownId::new(town),
            pos: pos_back(pos),
            named: alone,
        },
        A::CreaturesJoined {
            hero,
            creature,
            count,
        } => K::CreaturesJoined {
            hero: id(hero),
            creature: Handle::from_raw(creature),
            count,
        },
        A::EventFired { hero, tile } => K::EventFired {
            hero: id(hero),
            tile: pos_back(tile),
        },
        A::MonsterChanged {
            object_type,
            anchor,
            count_word,
        } => K::MonsterChanged {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            count_word,
        },
        A::Boarded { hero, boat, at } => K::Boarded {
            hero: id(hero),
            boat,
            at: pos_back(at),
        },
        A::Landed {
            hero,
            boat,
            at,
            facing,
        } => K::Landed {
            hero: id(hero),
            boat,
            at: pos_back(at),
            facing,
        },
        A::MessageRead {
            hero,
            object_type,
            anchor,
        } => K::MessageRead {
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        A::BoatRemoved { hero, boat, at } => K::BoatRemoved {
            hero: id(hero),
            boat,
            at: pos_back(at),
        },
        A::ObjectFlagged {
            object_type,
            anchor,
            owner,
        } => K::ObjectFlagged {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            owner: PlayerId::new(owner),
        },
        A::TownCaptured {
            town,
            anchor,
            owner,
        } => K::TownCaptured {
            town: TownId::new(town),
            anchor: pos_back(anchor),
            owner: PlayerId::new(owner),
        },
        A::GuardsChanged {
            object_type,
            anchor,
            creature,
            count_word,
        } => K::GuardsChanged {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            creature: creature.map(Handle::from_raw),
            count_word,
        },
        A::ObjectMarked {
            object_type,
            anchor,
        } => K::ObjectMarked {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        A::ObjectEmpty {
            object_type,
            anchor,
        } => K::ObjectEmpty {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        A::ShipyardBlocked {
            object_type,
            anchor,
            reason,
        } => K::ShipyardBlocked {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            reason: match reason {
                w::ShipyardBlock::NoTile => ShipyardBlock::NoTile,
                w::ShipyardBlock::Taken => ShipyardBlock::Taken,
            },
        },
        A::BoatBuilt { tile, boat_type } => K::BoatBuilt {
            tile: pos_back(tile),
            boat_type,
        },
        A::NoneToRecruit {
            object_type,
            anchor,
            creature,
        } => K::NoneToRecruit {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            creature: Handle::from_raw(creature),
        },
        A::JoinRefused {
            hero,
            creature,
            count,
        } => K::JoinRefused {
            hero: id(hero),
            creature: Handle::from_raw(creature),
            count,
        },
        A::MonsterAdded {
            object_type,
            anchor,
            creature,
            count_word,
        } => K::MonsterAdded {
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            creature: Handle::from_raw(creature),
            count_word,
        },
        A::HeroTeleported { hero, from, to } => K::HeroTeleported {
            hero: id(hero),
            from: from.map(pos_back),
            to: to.map(pos_back),
            seen_from: seen(from, alone),
            seen_to: seen(to, alone),
        },
        A::LeadsNowhere {
            hero,
            object_type,
            anchor,
        } => K::LeadsNowhere {
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        A::WhirlpoolLoss {
            hero,
            slot,
            creature,
            count,
        } => K::WhirlpoolLoss {
            hero: id(hero),
            slot,
            creature: Handle::from_raw(creature),
            count,
        },
        A::MovementChanged {
            hero,
            movement,
            max_movement,
        } => K::MovementChanged {
            hero: id(hero),
            movement,
            max_movement,
        },
        A::DaySpellsChanged { hero, spells } => K::DaySpellsChanged {
            hero: id(hero),
            spells: h3_adventure::DaySpells {
                fly: spells.fly,
                water_walk: spells.water_walk,
                doors: spells.doors,
            },
        },
        A::BoatSummoned {
            boat,
            from,
            to,
            facing,
        } => K::BoatSummoned {
            boat,
            from: from.map(pos_back),
            to: to.map(pos_back),
            facing,
            seen_from: seen(from, alone),
            seen_to: seen(to, alone),
        },
        A::BoatCreated { tile, boat_type } => K::BoatCreated {
            tile: pos_back(tile),
            boat_type,
        },
        A::BoatScuttled { tile, boat } => K::BoatScuttled {
            tile: pos_back(tile),
            boat,
        },
        A::BonusChanged { hero, morale, luck } => K::BonusChanged {
            hero: id(hero),
            morale,
            luck,
        },
        A::OnceMarkChanged { hero, bonus, held } => K::OnceMarkChanged {
            hero: id(hero),
            bonus: h3_heroes::VisitBonus {
                mark: bonus.mark,
                morale: bonus.morale,
                luck: bonus.luck,
            },
            held,
        },
        A::TimedMarkChanged {
            hero,
            object_type,
            held,
        } => K::TimedMarkChanged {
            hero: id(hero),
            object_type: Handle::from_raw(object_type),
            held,
        },
        A::NothingGained {
            hero,
            object_type,
            anchor,
            why,
        } => K::NothingGained {
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            why: match why {
                w::NothingWhy::Visited => h3_adventure::NothingWhy::Visited,
                w::NothingWhy::Full => h3_adventure::NothingWhy::Full,
                w::NothingWhy::Emptied => h3_adventure::NothingWhy::Emptied,
            },
        },
        A::SirensLoss {
            hero,
            slot,
            creature,
            count,
        } => K::SirensLoss {
            hero: id(hero),
            slot,
            creature: Handle::from_raw(creature),
            count,
        },
        A::CreaturesUpgraded {
            hero,
            slot,
            from,
            to,
        } => K::CreaturesUpgraded {
            hero: id(hero),
            slot,
            from: Handle::from_raw(from),
            to: Handle::from_raw(to),
        },
        A::KindKnown { object_type } => K::KindKnown {
            object_type: ty(object_type),
        },
    }
}

/// The engine's projected event for `viewer`; `battle` is the engine's battle event, whose wire
/// form `e` must hold.
/// The Thieves' Guild's rows on the wire and in the engine, pairwise (test code, independent of
/// the mapping it checks).
pub(super) const GUILD_ROWS: [(w::GuildRow, GuildRow); 9] = [
    (w::GuildRow::Towns, GuildRow::Towns),
    (w::GuildRow::Heroes, GuildRow::Heroes),
    (w::GuildRow::Gold, GuildRow::Gold),
    (w::GuildRow::Common, GuildRow::Common),
    (w::GuildRow::Rare, GuildRow::Rare),
    (w::GuildRow::Obelisks, GuildRow::Obelisks),
    (w::GuildRow::Artifacts, GuildRow::Artifacts),
    (w::GuildRow::Army, GuildRow::Army),
    (w::GuildRow::Income, GuildRow::Income),
];

/// A row of the Thieves' Guild back to the engine's.
fn guild_row_back(row: w::GuildRow) -> GuildRow {
    GUILD_ROWS.iter().find(|(wire, _)| *wire == row).unwrap().1
}

/// The engine's Thieves' Guild as the wire should carry it (test code, independent of the
/// mapping it checks): the rows by [`GUILD_ROWS`], the handles raw.
pub(super) fn guild_wire(view: &ThievesGuildView) -> w::ThievesGuild {
    let row = |row| {
        GUILD_ROWS
            .iter()
            .find(|(_, engine)| *engine == row)
            .unwrap()
            .0
    };
    w::ThievesGuild {
        level: view.level,
        rows: (view.rows.iter())
            .map(|ranking| w::GuildRanking {
                row: row(ranking.row),
                places: ranking.places.clone(),
            })
            .collect(),
        players: (view.players.iter())
            .map(|shown| w::GuildPlayer {
                hero: shown.hero.map(|h| w::GuildHero {
                    hero_type: h.hero_type.raw(),
                    skills: h.skills,
                }),
                creature: shown.creature.map(Handle::raw),
            })
            .collect(),
    }
}

/// What the guild shows of a player back to the engine's.
fn guild_player_back(shown: w::GuildPlayer) -> GuildPlayer {
    GuildPlayer {
        hero: shown.hero.map(|h| GuildHero {
            hero_type: Handle::from_raw(h.hero_type),
            skills: h.skills,
        }),
        creature: shown.creature.map(Handle::from_raw),
    }
}

/// A part of the Thieves' Guild back to the engine's.
fn guild_part_back(part: w::GuildPart) -> GuildPart {
    match part {
        w::GuildPart::Place { row, of, place } => GuildPart::Place {
            row: guild_row_back(row),
            of: PlayerId::new(of),
            place,
        },
        w::GuildPart::Player { of, shown } => GuildPart::Player {
            of: PlayerId::new(of),
            shown: guild_player_back(shown),
        },
    }
}

fn tavern_head_back(h: w::TavernHeroHead) -> h3_game::TavernHero {
    h3_game::TavernHero {
        id: HeroId::new(h.id),
        hero_type: Handle::from_raw(h.hero_type),
        level: h.level,
        experience: h.experience,
        primary: h.primary,
        mana: h.mana,
        carried: h.carried,
        formation: match h.formation {
            w::Formation::Loose => h3_content::Formation::Loose,
            w::Formation::Tight => h3_content::Formation::Tight,
        },
        tactics: h.tactics,
    }
}

fn tavern_part_back(part: w::TavernPart) -> h3_game::TavernPart {
    match part {
        w::TavernPart::Stack { stack } => h3_game::TavernPart::Stack(stack_back(stack)),
        w::TavernPart::Skill { skill } => h3_game::TavernPart::Skill {
            skill: Handle::from_raw(skill.skill),
            level: skill.level,
        },
        w::TavernPart::Artifact { place, artifact } => h3_game::TavernPart::Artifact {
            place: place_back(place),
            artifact: artifact_back(artifact),
        },
    }
}

fn event_back(e: &w::GameEvent, viewer: PlayerId, battle: Option<h3_battle::Event>) -> GameEvent {
    use w::GameEvent as G;
    let alone = Viewers::NONE.with(viewer);
    let id = HeroId::new;
    let ty = Handle::from_raw;
    let p = PlayerId::new;
    let heroes = |h: [Option<u16>; 2]| h.map(|h| h.map(HeroId::new));
    match e.clone() {
        G::TurnEnded { player } => GameEvent::TurnEnded { player: p(player) },
        G::DayStarted { day } => GameEvent::DayStarted { day },
        G::WeekStarted { week, kind } => GameEvent::WeekStarted {
            week,
            kind: super::game::inverse_week(kind),
        },
        G::MonthStarted { month, kind } => GameEvent::MonthStarted {
            month,
            kind: match kind {
                w::Month::Normal {} => MonthKind::Normal,
                w::Month::Creature { creature } => MonthKind::Creature(Handle::from_raw(creature)),
                w::Month::Plague {} => MonthKind::Plague,
            },
        },
        G::TurnStarted { player } => GameEvent::TurnStarted { player: p(player) },
        G::TimedEventFired { player, event } => GameEvent::TimedEventFired {
            player: p(player),
            event,
        },
        G::Adventure { event } => GameEvent::Adventure(AdventureEvent {
            kind: adventure_back(event, alone),
            seen_by: alone,
        }),
        G::ResourceGained {
            player,
            resource,
            amount,
        } => GameEvent::ResourceGained {
            player: p(player),
            resource: Handle::from_raw(resource),
            amount,
        },
        G::OfferOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::OfferOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::OfferRefused {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::OfferRefused {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::OfferAnswered {
            player,
            object_type,
            anchor,
            accepted,
        } => GameEvent::OfferAnswered {
            player: p(player),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            accepted,
        },
        G::BattleStarted { heroes: h, tile } => GameEvent::BattleStarted {
            heroes: heroes(h),
            tile: pos_back(tile),
            seen_by: alone,
        },
        G::Battle { event } => {
            let battle = battle.expect("a battle event");
            assert_eq!(event, from_engine::event(&battle.kind));
            GameEvent::Battle {
                event: battle,
                seen_by: alone,
            }
        }
        G::BattleEnded { heroes: h, result } => GameEvent::BattleEnded {
            heroes: heroes(h),
            result: result_back(result),
            seen_by: alone,
        },
        G::HeroChanged {
            player,
            hero,
            change,
        } => GameEvent::HeroChanged {
            player: p(player),
            hero: id(hero),
            change: change_back(change),
        },
        G::CreaturesRaised {
            player,
            hero,
            creature,
            count,
        } => GameEvent::CreaturesRaised {
            player: p(player),
            hero: id(hero),
            creature: Handle::from_raw(creature),
            count,
        },
        G::LevelUpOpened {
            player,
            hero,
            level,
            primary,
            offers,
        } => GameEvent::LevelUpOpened {
            player: p(player),
            hero: id(hero),
            level,
            primary: primary_back(primary),
            offers: offers.map(Handle::from_raw),
        },
        G::LevelGained {
            player,
            hero,
            level,
            primary,
            skill,
        } => GameEvent::LevelGained {
            player: p(player),
            hero: id(hero),
            level,
            primary: primary_back(primary),
            skill: skill.map(Handle::from_raw),
        },
        G::ResourceLost {
            player,
            resource,
            amount,
        } => GameEvent::ResourceLost {
            player: p(player),
            resource: Handle::from_raw(resource),
            amount,
        },
        G::ResourcePaid {
            player,
            resource,
            amount,
        } => GameEvent::ResourcePaid {
            player: p(player),
            resource: Handle::from_raw(resource),
            amount,
        },
        G::StackChanged { player, at, stack } => GameEvent::StackChanged {
            player: p(player),
            at: slot_back(at),
            stack: stack.map(stack_back),
        },
        G::FormationSet {
            player,
            hero,
            formation,
        } => GameEvent::FormationSet {
            player: p(player),
            hero: id(hero),
            formation: match formation {
                w::Formation::Loose => EngineFormation::Loose,
                w::Formation::Tight => EngineFormation::Tight,
            },
        },
        G::TacticsSet { player, hero, on } => GameEvent::TacticsSet {
            player: p(player),
            hero: id(hero),
            on,
        },
        G::MeetingOpened { player, heroes } => GameEvent::MeetingOpened {
            player: p(player),
            heroes: heroes.map(HeroId::new),
        },
        G::ArmyDialogOpened {
            player,
            hero,
            stack,
        } => GameEvent::ArmyDialogOpened {
            player: p(player),
            hero: id(hero),
            stack: stack_back(stack),
        },
        G::LeaveGuardsOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::LeaveGuardsOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::ExchangeOpened {
            player,
            hero,
            garrison,
            object_type,
            anchor,
            removable,
        } => GameEvent::ExchangeOpened {
            player: p(player),
            hero: id(hero),
            garrison,
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            removable,
        },
        G::TavernOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::TavernOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::MarketOpened {
            player,
            hero,
            place,
        } => GameEvent::MarketOpened {
            player: p(player),
            hero: hero.map(id),
            place: match place {
                w::MarketSeen::Object { object } => h3_game::MarketSeen::Object {
                    object_type: ty(object.object_type),
                    anchor: pos_back(object.anchor),
                },
                w::MarketSeen::Town { town } => h3_game::MarketSeen::Town(TownId::new(town)),
            },
        },
        G::AltarOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::AltarOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::HillFortOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::HillFortOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::ChoiceOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::ChoiceOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::UniversityOpened {
            player,
            hero,
            object_type,
            anchor,
        } => GameEvent::UniversityOpened {
            player: p(player),
            hero: id(hero),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
        },
        G::HireOpened {
            player,
            target,
            source,
        } => GameEvent::HireOpened {
            player: p(player),
            target: hire_target_back(target),
            source: hire_seen_back(source),
        },
        G::DecisionClosed { player } => GameEvent::DecisionClosed { player: p(player) },
        G::GroupLost {
            player,
            hero,
            stack,
        } => GameEvent::GroupLost {
            player: p(player),
            hero: id(hero),
            stack: stack_back(stack),
        },
        G::TownChanged {
            player,
            town,
            owner,
            fort,
            capitol,
            built_today,
            level,
            slots,
            hall,
            garrison_hero,
            buildings,
        } => GameEvent::TownChanged {
            player: p(player),
            town: TownId::new(town),
            owner: owner.map(p),
            fort: [
                FortLevel::None,
                FortLevel::Fort,
                FortLevel::Citadel,
                FortLevel::Castle,
            ][usize::from(fort)],
            capitol,
            built_today,
            level,
            slots,
            hall,
            garrison_hero,
            buildings: buildings.as_ref().map(|b| TownBuildingsView {
                built: set_back(b.built.0),
                forbidden: set_back(b.forbidden.0),
                available: b.available,
                portal: b.portal.map(|p| h3_game::Summoned {
                    creature: Handle::from_raw(p.creature),
                    count: p.count,
                }),
                guild: guild_back(&b.guild),
                aurora: b.aurora,
                vortex: b.vortex,
            }),
        },
        G::TownStackSeen {
            player,
            town,
            stack,
            slot,
        } => GameEvent::TownStackSeen {
            player: p(player),
            town: TownId::new(town),
            stack: GarrisonStackView {
                creature: Handle::from_raw(stack.creature),
                count_word: stack.count_word,
                count: stack.count,
            },
            slot,
        },
        G::TownIncomeSeen {
            player,
            town,
            resource,
            amount,
        } => GameEvent::TownIncomeSeen {
            player: p(player),
            town: TownId::new(town),
            resource: Handle::from_raw(resource),
            amount,
        },
        G::TavernOffered { player, slot, hero } => GameEvent::TavernOffered {
            player: p(player),
            slot,
            hero: hero.map(tavern_head_back),
        },
        G::TavernPartSeen { player, slot, part } => GameEvent::TavernPartSeen {
            player: p(player),
            slot,
            part: tavern_part_back(part),
        },
        G::HeroHired {
            player,
            hero,
            hero_type,
            slot,
            pos,
        } => GameEvent::HeroHired {
            player: p(player),
            hero: id(hero),
            hero_type: Handle::from_raw(hero_type),
            slot,
            pos: pos_back(pos),
            seen_by: alone,
        },
        G::HiredPartSeen { player, hero, part } => GameEvent::HiredPartSeen {
            player: p(player),
            hero: id(hero),
            part: match part {
                w::HiredPart::Details { offered, positions } => h3_game::HiredPart::Details {
                    offered: OfferLevels {
                        wisdom: offered.wisdom,
                        school: offered.school,
                    },
                    positions,
                },
                w::HiredPart::Carried { artifact } => {
                    h3_game::HiredPart::Carried(artifact_back(artifact))
                }
                w::HiredPart::Spell { spell } => h3_game::HiredPart::Spell(Handle::from_raw(spell)),
            },
        },
        // E08 T141a (v81).
        G::PrisonerFreed {
            player,
            hero,
            hero_type,
            pos,
            view,
        } => GameEvent::PrisonerFreed {
            player: p(player),
            hero: id(hero),
            hero_type: Handle::from_raw(hero_type),
            pos: pos_back(pos),
            seen_by: alone,
            view: view.map(tavern_head_back),
        },
        G::PrisonerPartSeen { player, hero, part } => GameEvent::PrisonerPartSeen {
            player: p(player),
            hero: id(hero),
            part: tavern_part_back(part),
        },
        G::PrisonFull { player, hero } => GameEvent::PrisonFull {
            player: p(player),
            hero: id(hero),
        },
        G::PrisonEmptied { player, hero } => GameEvent::PrisonEmptied {
            player: p(player),
            hero: id(hero),
        },
        G::SpellCast {
            player,
            hero,
            spell,
            mastery,
            cost,
        } => GameEvent::SpellCast {
            player: p(player),
            hero: id(hero),
            spell: Handle::from_raw(spell),
            mastery,
            cost,
        },
        G::CastFailed {
            player,
            hero,
            spell,
            reason,
        } => GameEvent::CastFailed {
            player: p(player),
            hero: id(hero),
            spell: Handle::from_raw(spell),
            reason: match reason {
                w::CastFailure::Chance => h3_adventure::CastFailure::Chance,
                w::CastFailure::NoBoat => h3_adventure::CastFailure::NoBoat,
                w::CastFailure::Target => h3_adventure::CastFailure::Target,
            },
        },
        G::ThievesGuildSeen { player, level } => GameEvent::ThievesGuildSeen {
            player: p(player),
            level,
        },
        G::GuildPartSeen { player, part } => GameEvent::GuildPartSeen {
            player: p(player),
            part: guild_part_back(part),
        },
        G::GarrisonChanged {
            player,
            object_type,
            anchor,
            owner,
            level,
            slots,
            removable,
        } => GameEvent::GarrisonChanged {
            player: p(player),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            owner: owner.map(p),
            level,
            slots,
            removable,
        },
        G::GarrisonStackSeen {
            player,
            object_type,
            anchor,
            stack,
            slot,
        } => GameEvent::GarrisonStackSeen {
            player: p(player),
            object_type: ty(object_type),
            anchor: pos_back(anchor),
            stack: GarrisonStackView {
                creature: Handle::from_raw(stack.creature),
                count_word: stack.count_word,
                count: stack.count,
            },
            slot,
        },
        G::TownBuilt {
            player,
            town,
            building,
            event,
        } => GameEvent::TownBuilt {
            player: PlayerId::new(player),
            town: TownId::new(town),
            building: BuildingId::from_index(usize::from(building)).unwrap(),
            event,
        },
        G::TownEventFired {
            player,
            town,
            event,
        } => GameEvent::TownEventFired {
            player: PlayerId::new(player),
            town: TownId::new(town),
            event,
        },
    }
}

/// Checks `wire`, the events `viewer` gets of `engine`, against the engine's projections.
fn check(engine: &[GameEvent], viewer: PlayerId, wire: &[w::GameEvent]) {
    let projected: Vec<GameEvent> = engine
        .iter()
        .filter_map(|e| h3_game::project(e, Viewer::Player(viewer)))
        .collect();
    assert_eq!(wire.len(), projected.len());
    for (w, p) in wire.iter().zip(&projected) {
        let battle = match p {
            GameEvent::Battle { event, .. } => Some(*event),
            _ => None,
        };
        assert_eq!(&event_back(w, viewer, battle), p);
    }
}

fn kinds<T: core::fmt::Debug>(value: &T) -> String {
    std::format!("{value:?}")
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .find(|word| word.starts_with(|c: char| c.is_ascii_uppercase()))
        .unwrap_or_default()
        .into()
}

/// An event of every kind, with every variant of the parts they hold, seen by both players;
/// those of one owner are player 0's.
pub(super) fn every_event() -> Vec<GameEvent> {
    let hero = HeroId::new(3);
    let at = h3_adventure::Pos::new(4, 5, 1);
    let next = h3_adventure::Pos::new(5, 5, 1);
    let ty = Handle::from_raw(17);
    let creature = Handle::from_raw(40);
    let stack = EngineStack {
        slot: 2,
        creature,
        count: 11,
    };
    let seen_by = both();
    let mut events = vec![
        GameEvent::TurnEnded { player: P1 },
        GameEvent::DayStarted { day: 9 },
        GameEvent::WeekStarted {
            week: 2,
            kind: WeekKind::Normal,
        },
        GameEvent::WeekStarted {
            week: 3,
            kind: WeekKind::Creature(creature),
        },
        GameEvent::WeekStarted {
            week: 4,
            kind: WeekKind::Of {
                creatures: [Some(creature), None],
                bonus: 15,
            },
        },
        GameEvent::MonthStarted {
            month: 2,
            kind: MonthKind::Normal,
        },
        GameEvent::MonthStarted {
            month: 3,
            kind: MonthKind::Creature(creature),
        },
        GameEvent::MonthStarted {
            month: 4,
            kind: MonthKind::Plague,
        },
        GameEvent::TurnStarted { player: P0 },
        GameEvent::TimedEventFired {
            player: P0,
            event: 3,
        },
        GameEvent::ResourceGained {
            player: P0,
            resource: Handle::from_raw(6),
            amount: 500,
        },
        GameEvent::OfferOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        GameEvent::OfferRefused {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        GameEvent::OfferAnswered {
            player: P0,
            object_type: ty,
            anchor: at,
            accepted: true,
        },
        GameEvent::BattleStarted {
            heroes: [Some(hero), None],
            tile: at,
            seen_by,
        },
        GameEvent::Battle {
            event: h3_battle::Event::public(h3_battle::EventKind::RoundStarted { round: 2 }),
            seen_by,
        },
        GameEvent::CreaturesRaised {
            player: P0,
            hero,
            creature,
            count: 4,
        },
        GameEvent::LevelUpOpened {
            player: P0,
            hero,
            level: 5,
            primary: EnginePrimary::Attack,
            offers: [Handle::from_raw(1), Handle::from_raw(2)],
        },
        GameEvent::ResourceLost {
            player: P0,
            resource: Handle::from_raw(0),
            amount: 3,
        },
        GameEvent::ResourcePaid {
            player: P0,
            resource: Handle::from_raw(6),
            amount: 1000,
        },
        GameEvent::StackChanged {
            player: P0,
            at: ArmySlot {
                army: ArmyRef::Hero(hero),
                slot: 2,
            },
            stack: Some(stack),
        },
        GameEvent::StackChanged {
            player: P0,
            at: ArmySlot {
                army: ArmyRef::Group,
                slot: 0,
            },
            stack: None,
        },
        GameEvent::StackChanged {
            player: P0,
            at: ArmySlot {
                army: ArmyRef::Garrison(TownId::new(1)),
                slot: 6,
            },
            stack: None,
        },
        GameEvent::StackChanged {
            player: P0,
            at: ArmySlot {
                army: ArmyRef::MapGarrison(41),
                slot: 3,
            },
            stack: Some(stack),
        },
        GameEvent::FormationSet {
            player: P0,
            hero,
            formation: EngineFormation::Loose,
        },
        GameEvent::FormationSet {
            player: P0,
            hero,
            formation: EngineFormation::Tight,
        },
        GameEvent::TacticsSet {
            player: P0,
            hero,
            on: true,
        },
        GameEvent::MeetingOpened {
            player: P0,
            heroes: [hero, HeroId::new(5)],
        },
        GameEvent::ArmyDialogOpened {
            player: P0,
            hero,
            stack,
        },
        GameEvent::LeaveGuardsOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        GameEvent::ExchangeOpened {
            player: P0,
            hero,
            garrison: 41,
            object_type: ty,
            anchor: at,
            removable: true,
        },
        GameEvent::TavernOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        // E08 T133b (v75); a town's window without a hero since E09 T58a1 (v85).
        GameEvent::MarketOpened {
            player: P0,
            hero: Some(hero),
            place: h3_game::MarketSeen::Object {
                object_type: ty,
                anchor: at,
            },
        },
        GameEvent::MarketOpened {
            player: P0,
            hero: None,
            place: h3_game::MarketSeen::Town(TownId::new(2)),
        },
        // E08 T134a (v76).
        GameEvent::AltarOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        // E08 T134b1 (v77).
        GameEvent::HillFortOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        GameEvent::ChoiceOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        GameEvent::UniversityOpened {
            player: P0,
            hero,
            object_type: ty,
            anchor: at,
        },
        GameEvent::HireOpened {
            player: P0,
            target: HireTarget::Hero(hero),
            source: HireSeen::Object {
                object_type: ty,
                anchor: at,
            },
        },
        // E09 T27 (protocol v59): a town's recruitment.
        GameEvent::HireOpened {
            player: P0,
            target: HireTarget::Garrison(TownId::new(1)),
            source: HireSeen::Town {
                town: TownId::new(1),
                index: 7,
            },
        },
        GameEvent::HireOpened {
            player: P0,
            target: HireTarget::Garrison(TownId::new(1)),
            source: HireSeen::Portal {
                town: TownId::new(1),
            },
        },
        GameEvent::DecisionClosed { player: P0 },
        GameEvent::GroupLost {
            player: P0,
            hero,
            stack,
        },
        // A town's whole view for its owner (its stacks by slot, its income), then another's.
        GameEvent::TownChanged {
            player: P0,
            town: TownId::new(2),
            owner: Some(P0),
            fort: FortLevel::Citadel,
            capitol: true,
            built_today: true,
            level: 3,
            slots: true,
            hall: Some(4),
            garrison_hero: true,
            buildings: Some(TownBuildingsView {
                built: BuildingSet::EMPTY
                    .with(BuildingId::from_index(0).unwrap())
                    .with(BuildingId::from_index(63).unwrap()),
                forbidden: BuildingSet::EMPTY.with(BuildingId::from_index(12).unwrap()),
                available: [7, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 1],
                portal: Some(h3_game::Summoned {
                    creature: Handle::from_raw(30),
                    count: 4,
                }),
                // E09 T47b: a spell at level 3, the Vortex's flag set.
                guild: GuildSlots::EMPTY.with(2, 0, Handle::from_raw(41)),
                aurora: false,
                vortex: true,
            }),
        },
        GameEvent::TownStackSeen {
            player: P0,
            town: TownId::new(2),
            stack: GarrisonStackView {
                creature,
                count_word: Some(2),
                count: Some(11),
            },
            slot: Some(4),
        },
        GameEvent::TownIncomeSeen {
            player: P0,
            town: TownId::new(2),
            resource: Handle::from_raw(6),
            amount: 4000,
        },
        // E09 T37a: a slot's tavern hero, each kind of part, an emptied slot.
        GameEvent::TavernOffered {
            player: P0,
            slot: 1,
            hero: Some(h3_game::TavernHero {
                id: HeroId::new(40),
                hero_type: Handle::from_raw(12),
                level: 2,
                experience: 1200,
                primary: [1, 2, 3, 4],
                mana: 40,
                carried: 6,
                formation: h3_content::Formation::Loose,
                tactics: true,
            }),
        },
        GameEvent::TavernPartSeen {
            player: P0,
            slot: 1,
            part: h3_game::TavernPart::Stack(EngineStack {
                slot: 2,
                creature,
                count: 9,
            }),
        },
        GameEvent::TavernPartSeen {
            player: P0,
            slot: 1,
            part: h3_game::TavernPart::Skill {
                skill: Handle::from_raw(5),
                level: 3,
            },
        },
        GameEvent::TavernPartSeen {
            player: P0,
            slot: 1,
            part: h3_game::TavernPart::Artifact {
                place: ArtifactPlace::Backpack(2),
                artifact: ArtifactInstance {
                    artifact: Handle::from_raw(7),
                    spell: Some(Handle::from_raw(3)),
                },
            },
        },
        GameEvent::TavernOffered {
            player: P0,
            slot: 0,
            hero: None,
        },
        // E09 T38a1 (v71): a hire both see, its slot player 0's alone, and its parts.
        GameEvent::HeroHired {
            player: P0,
            hero: HeroId::new(6),
            hero_type: Handle::from_raw(12),
            slot: Some(1),
            pos: at,
            seen_by,
        },
        GameEvent::HiredPartSeen {
            player: P0,
            hero: HeroId::new(6),
            part: h3_game::HiredPart::Details {
                offered: OfferLevels {
                    wisdom: 2,
                    school: 3,
                },
                positions: 19,
            },
        },
        GameEvent::HiredPartSeen {
            player: P0,
            hero: HeroId::new(6),
            part: h3_game::HiredPart::Carried(ArtifactInstance {
                artifact: Handle::from_raw(7),
                spell: None,
            }),
        },
        GameEvent::HiredPartSeen {
            player: P0,
            hero: HeroId::new(6),
            part: h3_game::HiredPart::Spell(Handle::from_raw(30)),
        },
        // E08 T141a (v81): a freed prisoner both see, its view and its parts player 0's alone;
        // a full player's and an emptied Prison's visits player 0's.
        GameEvent::PrisonerFreed {
            player: P0,
            hero: HeroId::new(8),
            hero_type: Handle::from_raw(14),
            pos: at,
            seen_by,
            view: Some(h3_game::TavernHero {
                id: HeroId::new(8),
                hero_type: Handle::from_raw(14),
                level: 5,
                experience: 5000,
                primary: [2, 1, 1, 3],
                mana: 30,
                carried: 0,
                formation: h3_content::Formation::Tight,
                tactics: true,
            }),
        },
        GameEvent::PrisonerPartSeen {
            player: P0,
            hero: HeroId::new(8),
            part: h3_game::TavernPart::Stack(stack),
        },
        GameEvent::PrisonFull { player: P0, hero },
        GameEvent::PrisonEmptied { player: P0, hero },
        // E08 T142a (v82): a cast on the map, player 0's alone.
        GameEvent::SpellCast {
            player: P0,
            hero,
            spell: Handle::from_raw(6),
            mastery: 2,
            cost: 15,
        },
    ];
    // E08 T143a (v83): a charged failure of each reason, player 0's alone.
    events.extend(
        [
            h3_adventure::CastFailure::Chance,
            h3_adventure::CastFailure::NoBoat,
            h3_adventure::CastFailure::Target,
        ]
        .map(|reason| GameEvent::CastFailed {
            player: P0,
            hero,
            spell: Handle::from_raw(0),
            reason,
        }),
    );
    events.extend([
        // E09 T39b2 (v74): a Den of Thieves' guild, player 0's alone, a part of each kind.
        GameEvent::ThievesGuildSeen {
            player: P0,
            level: 5,
        },
        GameEvent::GuildPartSeen {
            player: P0,
            part: GuildPart::Place {
                row: GuildRow::Income,
                of: P1,
                place: 1,
            },
        },
        GameEvent::GuildPartSeen {
            player: P0,
            part: GuildPart::Player {
                of: P1,
                shown: GuildPlayer {
                    hero: Some(GuildHero {
                        hero_type: Handle::from_raw(14),
                        skills: Some([5, 4, 99, 0]),
                    }),
                    creature: Some(Handle::from_raw(8)),
                },
            },
        },
        GameEvent::GuildPartSeen {
            player: P0,
            part: GuildPart::Player {
                of: P0,
                shown: GuildPlayer {
                    hero: None,
                    creature: None,
                },
            },
        },
        GameEvent::TownChanged {
            player: P0,
            town: TownId::new(1),
            owner: None,
            fort: FortLevel::None,
            capitol: false,
            built_today: false,
            level: 1,
            slots: false,
            hall: None,
            garrison_hero: false,
            buildings: None,
        },
        GameEvent::TownStackSeen {
            player: P0,
            town: TownId::new(1),
            stack: GarrisonStackView {
                creature,
                count_word: None,
                count: None,
            },
            slot: None,
        },
        // E08 T114b: a map garrison's whole view for its owner's team (by slot, its removable
        // flag), then another's at a tavern level.
        GameEvent::GarrisonChanged {
            player: P0,
            object_type: ty,
            anchor: at,
            owner: Some(P0),
            level: 3,
            slots: true,
            removable: Some(false),
        },
        GameEvent::GarrisonStackSeen {
            player: P0,
            object_type: ty,
            anchor: at,
            stack: GarrisonStackView {
                creature,
                count_word: Some(2),
                count: Some(11),
            },
            slot: Some(4),
        },
        GameEvent::GarrisonChanged {
            player: P0,
            object_type: ty,
            anchor: at,
            owner: None,
            level: 2,
            slots: false,
            removable: None,
        },
        GameEvent::GarrisonStackSeen {
            player: P0,
            object_type: ty,
            anchor: at,
            stack: GarrisonStackView {
                creature,
                count_word: Some(2),
                count: None,
            },
            slot: None,
        },
        GameEvent::TownBuilt {
            player: P0,
            town: TownId::new(2),
            building: BuildingId::from_index(12).unwrap(),
            event: None,
        },
        GameEvent::TownBuilt {
            player: P0,
            town: TownId::new(1),
            building: BuildingId::from_index(63).unwrap(),
            event: Some(4),
        },
        GameEvent::TownEventFired {
            player: P0,
            town: TownId::new(1),
            event: 4,
        },
    ]);
    events.extend(
        [
            BattleResult::Victory(BattleSide::Attacker),
            BattleResult::Draw,
            BattleResult::Retreat(BattleSide::Defender),
            BattleResult::Surrender(BattleSide::Attacker),
        ]
        .map(|result| GameEvent::BattleEnded {
            heroes: [Some(hero), Some(HeroId::new(4))],
            result,
            seen_by,
        }),
    );
    let worn = ArtifactPlace::Worn(Position(3));
    let scroll = ArtifactInstance {
        artifact: Handle::from_raw(1),
        spell: Some(Handle::from_raw(9)),
    };
    events.extend(
        [
            HeroChange::GainExperience(1200),
            HeroChange::SetSpellPoints(30),
            HeroChange::AddArtifact(scroll),
            HeroChange::RemoveArtifact(worn),
            HeroChange::PlaceArtifact {
                artifact: ArtifactInstance {
                    artifact: Handle::from_raw(7),
                    spell: None,
                },
                at: ArtifactPlace::Backpack(4),
            },
            HeroChange::LearnSpell(Handle::from_raw(12)),
            HeroChange::GainLevel {
                primary: EnginePrimary::Power,
                learned: Some(Handle::from_raw(3)),
                offered: OfferLevels {
                    wisdom: 4,
                    school: 6,
                },
            },
            HeroChange::GainLevel {
                primary: EnginePrimary::Knowledge,
                learned: None,
                offered: OfferLevels {
                    wisdom: 2,
                    school: 0,
                },
            },
            HeroChange::GainPrimary([1, 0, 2, 0]),
            HeroChange::RaiseSkill {
                skill: Handle::from_raw(8),
                level: 2,
            },
        ]
        .map(|change| GameEvent::HeroChanged {
            player: P0,
            hero,
            change,
        }),
    );
    events.extend(
        [EnginePrimary::Defense, EnginePrimary::Attack].map(|primary| GameEvent::LevelGained {
            player: P0,
            hero,
            level: 6,
            primary,
            skill: (primary == EnginePrimary::Defense).then(|| Handle::from_raw(5)),
        }),
    );
    let step = |from, to, seen_from, seen_to| K::HeroStepped {
        hero,
        from,
        to,
        seen_from,
        seen_to,
    };
    let jump = |from, to, seen_from, seen_to| K::HeroTeleported {
        hero,
        from,
        to,
        seen_from,
        seen_to,
    };
    let summon = |from, to, seen_from, seen_to| K::BoatSummoned {
        boat: 1,
        from,
        to,
        facing: 6,
        seen_from,
        seen_to,
    };
    let only_p1 = Viewers::NONE.with(P1);
    let adventure = [
        // Seen at both ends by both, by player 0 at one end only.
        step(Some(at), Some(next), seen_by, seen_by),
        step(Some(at), Some(next), only_p1, seen_by),
        step(Some(at), Some(next), seen_by, only_p1),
        jump(Some(at), Some(next), seen_by, seen_by),
        jump(Some(at), Some(next), only_p1, seen_by),
        jump(Some(at), Some(next), seen_by, only_p1),
        K::ObjectRemoved {
            object_type: ty,
            anchor: at,
        },
        K::TilesRevealed {
            object_type: ty,
            count: 21,
        },
        K::KeyGained { key: 4 },
        K::HeroRemoved { hero, pos: at },
        // E09 T28a (v64). E09 T31: named to both, then to player 0 alone (player 1 is told a
        // hero leaving the map and one coming into sight).
        K::GarrisonEntered {
            hero,
            town: TownId::new(2),
            pos: at,
            named: seen_by,
        },
        K::GarrisonLeft {
            hero,
            town: TownId::new(2),
            pos: at,
            named: seen_by,
        },
        K::GarrisonEntered {
            hero,
            town: TownId::new(2),
            pos: at,
            named: Viewers::NONE.with(P0),
        },
        K::GarrisonLeft {
            hero,
            town: TownId::new(2),
            pos: at,
            named: Viewers::NONE.with(P0),
        },
        K::CreaturesJoined {
            hero,
            creature,
            count: 8,
        },
        K::EventFired { hero, tile: at },
        K::MonsterChanged {
            object_type: ty,
            anchor: at,
            count_word: 3,
        },
        K::Boarded {
            hero,
            boat: 1,
            at: next,
        },
        K::Landed {
            hero,
            boat: 1,
            at,
            facing: 5,
        },
        K::MessageRead {
            hero,
            object_type: ty,
            anchor: at,
        },
        K::BoatRemoved { hero, boat: 0, at },
        K::ObjectFlagged {
            object_type: ty,
            anchor: at,
            owner: P1,
        },
        K::TownCaptured {
            town: TownId::new(1),
            anchor: at,
            owner: P0,
        },
        K::GuardsChanged {
            object_type: ty,
            anchor: at,
            creature: Some(creature),
            count_word: Some(2),
        },
        K::GuardsChanged {
            object_type: ty,
            anchor: at,
            creature: None,
            count_word: None,
        },
        K::ObjectMarked {
            object_type: ty,
            anchor: at,
        },
        K::ObjectEmpty {
            object_type: ty,
            anchor: at,
        },
        K::ShipyardBlocked {
            object_type: ty,
            anchor: at,
            reason: ShipyardBlock::NoTile,
        },
        K::ShipyardBlocked {
            object_type: ty,
            anchor: at,
            reason: ShipyardBlock::Taken,
        },
        K::BoatBuilt {
            tile: next,
            boat_type: 2,
        },
        K::NoneToRecruit {
            object_type: ty,
            anchor: at,
            creature,
        },
        K::JoinRefused {
            hero,
            creature,
            count: 30,
        },
        K::MonsterAdded {
            object_type: ty,
            anchor: at,
            creature,
            count_word: 4,
        },
        K::LeadsNowhere {
            hero,
            object_type: ty,
            anchor: at,
        },
        K::WhirlpoolLoss {
            hero,
            slot: 3,
            creature,
            count: 2,
        },
        K::MovementChanged {
            hero,
            movement: 1_234,
            max_movement: 2_000,
        },
        K::DaySpellsChanged {
            hero,
            spells: h3_adventure::DaySpells {
                fly: Some(2),
                water_walk: Some(0),
                doors: 3,
            },
        },
        K::BonusChanged {
            hero,
            morale: -1,
            luck: 2,
        },
        K::OnceMarkChanged {
            hero,
            bonus: h3_heroes::VisitBonus {
                mark: 13,
                morale: 0,
                luck: -2,
            },
            held: true,
        },
        K::TimedMarkChanged {
            hero,
            object_type: ty,
            held: false,
        },
        // E08 T115b: the bonus visitables' events, each reason of `NothingGained`.
        K::NothingGained {
            hero,
            object_type: ty,
            anchor: at,
            why: h3_adventure::NothingWhy::Visited,
        },
        K::NothingGained {
            hero,
            object_type: ty,
            anchor: at,
            why: h3_adventure::NothingWhy::Full,
        },
        K::NothingGained {
            hero,
            object_type: ty,
            anchor: at,
            why: h3_adventure::NothingWhy::Emptied,
        },
        K::SirensLoss {
            hero,
            slot: 1,
            creature,
            count: 28,
        },
        K::CreaturesUpgraded {
            hero,
            slot: 0,
            from: creature,
            to: Handle::from_raw(41),
        },
        K::KindKnown { object_type: ty },
        // E08 T143a (v83): a summoned boat seen at both ends by both, by player 0 at one end
        // only; a made boat; a sunk one.
        summon(Some(at), Some(next), seen_by, seen_by),
        summon(Some(at), Some(next), only_p1, seen_by),
        summon(Some(at), Some(next), seen_by, only_p1),
        K::BoatCreated {
            tile: next,
            boat_type: 0,
        },
        K::BoatScuttled { tile: at, boat: 1 },
    ];
    events.extend(adventure.map(|kind| GameEvent::Adventure(AdventureEvent { kind, seen_by })));
    events
}

#[test]
fn every_event_maps_back_for_its_viewers_and_round_trips() {
    let all = every_event();
    let distinct: BTreeSet<String> = all.iter().map(kinds).collect();
    // The game's 49 kinds of event (the adventure's as one; E08 T114a2: `ExchangeOpened`; T114b:
    // `GarrisonChanged`, `GarrisonStackSeen`; E09 T37a: `TavernOffered`, `TavernPartSeen`; T38a1:
    // `HeroHired`, `HiredPartSeen`; T39a: `TavernOpened`; T39b2: `ThievesGuildSeen`,
    // `GuildPartSeen`; E08 T133b: `MarketOpened`; T134a: `AltarOpened`; T134b1:
    // `HillFortOpened`; T141a: `PrisonerFreed`, `PrisonerPartSeen`, `PrisonFull`,
    // `PrisonEmptied`; T142a: `SpellCast`; T143a: `CastFailed`).
    assert_eq!(distinct.len(), 55, "{distinct:?}");
    for viewer in [P0, P1] {
        let wire = events(&all, Viewer::Player(viewer));
        check(&all, viewer, &wire);
        for e in &wire {
            round_trips(e);
        }
    }
    // Player 0 sees them all; player 1 the public ones and those it is among the viewers of,
    // none of player 0's own.
    assert_eq!(events(&all, Viewer::Player(P0)).len(), all.len());
    let theirs: BTreeSet<String> = events(&all, Viewer::Player(P1)).iter().map(kinds).collect();
    let seen: BTreeSet<String> = [
        "TurnEnded",
        "DayStarted",
        "WeekStarted",
        "MonthStarted",
        "TurnStarted",
        "Adventure",
        "BattleStarted",
        "Battle",
        "BattleEnded",
        "HeroHired",
        "PrisonerFreed",
    ]
    .map(String::from)
    .into();
    assert_eq!(theirs, seen);
    // A hire's slot is its owner's alone.
    let slots = |viewer| -> Vec<Option<u8>> {
        events(&all, Viewer::Player(viewer))
            .iter()
            .filter_map(|e| match e {
                w::GameEvent::HeroHired { slot, .. } => Some(*slot),
                _ => None,
            })
            .collect()
    };
    assert_eq!((slots(P0), slots(P1)), (vec![Some(1)], vec![None]));
    // A step player 0 sees at one end only: the other is `None` on its wire.
    let mine = events(&all, Viewer::Player(P0));
    let steps: Vec<_> = mine
        .iter()
        .filter_map(|e| match e {
            w::GameEvent::Adventure {
                event: w::AdventureEvent::HeroStepped { from, to, .. },
            } => Some((from.is_some(), to.is_some())),
            _ => None,
        })
        .collect();
    assert_eq!(steps, [(true, true), (false, true), (true, false)]);
}

#[test]
fn a_dens_guild_on_the_wire_is_the_guild_its_parts_make() {
    // E09 T39b2 (v74): a guild at everything for two players, its events as a Den of Thieves'
    // visit tells them (`ThievesGuildView::parts`), map back for its viewer alone and round-trip;
    // seen in order from the told level (`ThievesGuild::see`), the parts make the whole guild's
    // wire view, which is the query's mapping of it.
    let hero = |ty, skills| {
        Some(GuildHero {
            hero_type: Handle::from_raw(ty),
            skills,
        })
    };
    let view = ThievesGuildView {
        level: 5,
        rows: (GuildRow::ALL.iter().zip(0_u8..))
            .map(|(&row, k)| GuildRanking {
                row,
                places: vec![k % 2, (k + 1) % 2],
            })
            .collect(),
        players: vec![
            GuildPlayer {
                hero: hero(3, Some([1, 2, 3, 99])),
                creature: Some(Handle::from_raw(5)),
            },
            GuildPlayer {
                hero: hero(7, None),
                creature: None,
            },
        ],
    };
    let head = GameEvent::ThievesGuildSeen {
        player: P0,
        level: 5,
    };
    let parts = view
        .parts()
        .map(|part| GameEvent::GuildPartSeen { player: P0, part });
    let engine: Vec<GameEvent> = core::iter::once(head).chain(parts).collect();
    assert_eq!(engine.len(), 1 + 9 * 2 + 2);
    let wire = events(&engine, Viewer::Player(P0));
    check(&engine, P0, &wire);
    wire.iter().for_each(round_trips);
    assert!(events(&engine, Viewer::Player(P1)).is_empty());
    let mut told = w::ThievesGuild::at(5);
    (wire.iter())
        .filter_map(|e| match *e {
            w::GameEvent::GuildPartSeen { part, .. } => Some(part),
            _ => None,
        })
        .for_each(|part| told.see(part));
    assert_eq!(told, guild_wire(&view));
    assert_eq!(told, crate::map::game::thieves_guild(&view));
    round_trips(&told);
}

#[test]
fn the_events_json_is_serdes_default_form() {
    let engine = vec![
        GameEvent::DayStarted { day: 2 },
        GameEvent::Adventure(AdventureEvent {
            kind: K::HeroStepped {
                hero: HeroId::new(1),
                from: Some(h3_adventure::Pos::new(3, 1, 0)),
                to: Some(h3_adventure::Pos::new(4, 1, 0)),
                seen_from: both(),
                seen_to: Viewers::NONE.with(P1),
            },
            seen_by: both(),
        }),
    ];
    let wire = events(&engine, Viewer::Player(P0));
    let text = serde_json::to_string(&wire).unwrap();
    assert_eq!(
        text,
        r#"[{"DayStarted":{"day":2}},{"Adventure":{"event":{"HeroStepped":{"hero":1,"from":{"x":3,"y":1,"level":0},"to":null}}}}]"#
    );
    let extra = r#"{"DayStarted":{"day":2,"seen_by":1}}"#;
    assert!(serde_json::from_str::<w::GameEvent>(extra).is_err());
}

/// Plays `steps` random steps of `map` and checks every viewer's events of each.
fn check_played(map: &[u8], seed: u64, steps: usize, met: &mut BTreeSet<String>) -> usize {
    let rules = &*RULES;
    let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
    let setup = std::sync::Arc::new(h3_map::import(&m, rules).unwrap());
    let mut env = AdventureEnv::new(rules.clone(), setup, seed, 28).unwrap();
    let mut rng = Rng::for_stream(seed, StreamId::AGENT);
    let mut checked = 0;
    for step in 0..steps {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        let mut sink: Vec<GameEvent> = Vec::new();
        env.step_with(seat, action, &mut sink).unwrap();
        for player in (0..=u8::MAX).take(env.state().players()) {
            let viewer = PlayerId::new(player);
            let wire = events(&sink, Viewer::Player(viewer));
            check(&sink, viewer, &wire);
            for e in &wire {
                let kind = match e {
                    w::GameEvent::Adventure { event } => std::format!("Adventure.{}", kinds(event)),
                    other => kinds(other),
                };
                if met.insert(kind) || step % 50 == 0 {
                    round_trips(e);
                }
            }
            checked += wire.len();
        }
    }
    checked
}

#[test]
fn a_played_games_events_map_back_for_every_viewer() {
    let maps: [(&[u8], u64); 5] = [
        (
            include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m"),
            21,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e09-towns-36.h3m"),
            22,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-rewards-36.h3m"),
            23,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-economy-36.h3m"),
            24,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-sea-36.h3m"),
            25,
        ),
    ];
    let mut met = BTreeSet::new();
    for (map, seed) in maps {
        let checked = check_played(map, seed, 400, &mut met);
        assert!(checked >= 500, "{checked}");
    }
    assert!(met.len() >= 25, "{met:?}");
}
