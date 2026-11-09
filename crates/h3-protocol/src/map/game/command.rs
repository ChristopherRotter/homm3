//! The whole game's commands both ways and its rejections on the wire (E16 M5 T3a). Every enum
//! is matched without a wildcard and every struct's fields named: a new variant or field of the
//! engine's fails this crate's build until it is mapped (ADR-0018 point 5 as amended). A client's
//! command maps to the engine whatever its values: the engine validates them against the state
//! and the trusted actor (ADR-0008); an index the wire holds narrower than the engine (a step, an
//! artifact's place) is the engine's own echo of a client's value or a value of the game's
//! bounds, so it never saturates in play.

use h3_adventure::{AdventureCommand as EngineAdventure, AdventureCommandError, StandError};
use h3_content::{CreatureTypeId, Formation as EngineFormation};
use h3_contracts::{HeroId, TownId};
use h3_game::{
    Answer as EngineAnswer, ArenaChoice as EngineArena, ArmyCommand as EngineArmy,
    ArmyCommandError, ArmyRef as EngineArmyRef, ArmySlot as EngineSlot,
    ArtifactPlace as EnginePlace, ChestChoice as EngineChest, Consent as EngineConsent,
    GameCommand as EngineCommand, GameCommandError, HireAnswer as EngineHire, HireError,
    Manage as EngineManage, ManageError, Position, SchoolChoice as EngineSchool,
    SkillOffer as EngineOffer, UniversityAnswer as EngineUniversity,
};
use h3_heroes::{CombineError, MoveError};
use h3_kernel::handle::Handle;

use super::pos;
use crate::map::{command_to_engine as battle_to_engine, from_engine};
use crate::wire::game::{
    AdventureCommand, AdventureReject, Answer, ArenaChoice, ArmyCommand, ArmyRef, ArmyReject,
    ArmySlot, ArtifactAt, ArtifactPlace, ChestChoice, CombineReject, Consent, Formation,
    GameCommand, GameReject, HireAnswer, HireReject, Manage, ManageReject, MoveReject, Pos,
    SchoolChoice, SkillOffer, StandReject, UniversityAnswer,
};

/// An engine command on the wire (a legal list's, the echo of an applied command).
pub fn command(command: &EngineCommand) -> GameCommand {
    match command {
        EngineCommand::EndTurn => GameCommand::EndTurn {},
        EngineCommand::Adventure(c) => GameCommand::Adventure {
            command: match c {
                EngineAdventure::MoveHero { hero, path } => AdventureCommand::MoveHero {
                    hero: hero.raw(),
                    path: path.iter().map(|&p| pos(p)).collect(),
                },
                EngineAdventure::BuildBoat {
                    object_type,
                    anchor,
                } => AdventureCommand::BuildBoat {
                    object_type: object_type.raw(),
                    anchor: pos(*anchor),
                },
            },
        },
        EngineCommand::Answer(a) => GameCommand::Answer { answer: answer(*a) },
        EngineCommand::Battle(c) => GameCommand::Battle {
            command: from_engine::command(*c),
        },
        EngineCommand::Manage(m) => GameCommand::Manage { manage: manage(*m) },
    }
}

/// An answer on the wire.
const fn answer(a: EngineAnswer) -> Answer {
    match a {
        EngineAnswer::Offer(consent) => Answer::Offer {
            consent: match consent {
                EngineConsent::Accept => Consent::Accept,
                EngineConsent::Decline => Consent::Decline,
            },
        },
        EngineAnswer::LevelUp(offer) => Answer::LevelUp {
            offer: match offer {
                EngineOffer::First => SkillOffer::First,
                EngineOffer::Second => SkillOffer::Second,
            },
        },
        EngineAnswer::Chest(choice) => Answer::Chest {
            choice: match choice {
                EngineChest::Gold => ChestChoice::Gold,
                EngineChest::Experience => ChestChoice::Experience,
            },
        },
        EngineAnswer::Arena(choice) => Answer::Arena {
            choice: match choice {
                EngineArena::Attack => ArenaChoice::Attack,
                EngineArena::Defense => ArenaChoice::Defense,
            },
        },
        EngineAnswer::School(choice) => Answer::School {
            choice: match choice {
                EngineSchool::First => SchoolChoice::First,
                EngineSchool::Second => SchoolChoice::Second,
                EngineSchool::Leave => SchoolChoice::Leave,
            },
        },
        EngineAnswer::University(answer) => Answer::University {
            answer: match answer {
                EngineUniversity::Buy(slot) => UniversityAnswer::Buy { slot },
                EngineUniversity::Leave => UniversityAnswer::Leave {},
            },
        },
        EngineAnswer::Hire(answer) => Answer::Hire {
            answer: match answer {
                EngineHire::Buy { creature, amount } => HireAnswer::Buy {
                    creature: creature.raw(),
                    amount,
                },
                EngineHire::Close => HireAnswer::Close {},
            },
        },
        EngineAnswer::Close => Answer::Close {},
    }
}

/// A management command on the wire.
fn manage(m: EngineManage) -> Manage {
    match m {
        EngineManage::Army(command) => Manage::Army {
            command: match command {
                EngineArmy::Move { from, to } => ArmyCommand::Move {
                    from: slot(from),
                    to: slot(to),
                },
                EngineArmy::Split { from, to, count } => ArmyCommand::Split {
                    from: slot(from),
                    to: slot(to),
                    count,
                },
                EngineArmy::Dismiss { at } => ArmyCommand::Dismiss { at: slot(at) },
            },
        },
        EngineManage::Formation { hero, formation } => Manage::Formation {
            hero: hero.raw(),
            formation: match formation {
                EngineFormation::Loose => Formation::Loose,
                EngineFormation::Tight => Formation::Tight,
            },
        },
        EngineManage::Tactics { hero, on } => Manage::Tactics {
            hero: hero.raw(),
            on,
        },
        EngineManage::Artifact { from, to } => Manage::Artifact {
            from: artifact_at(from),
            to: artifact_at(to),
        },
        EngineManage::Assemble { hero, at } => Manage::Assemble {
            hero: hero.raw(),
            at: position(at),
        },
        EngineManage::Disassemble { hero, at } => Manage::Disassemble {
            hero: hero.raw(),
            at: position(at),
        },
    }
}

/// An army slot on the wire.
pub(super) const fn slot(s: EngineSlot) -> ArmySlot {
    let EngineSlot { army, slot } = s;
    ArmySlot {
        army: match army {
            EngineArmyRef::Hero(hero) => ArmyRef::Hero { hero: hero.raw() },
            EngineArmyRef::Group => ArmyRef::Group {},
            EngineArmyRef::Garrison(town) => ArmyRef::Garrison { town: town.raw() },
        },
        slot,
    }
}

/// A hero's artifact place on the wire.
fn artifact_at((hero, place): (HeroId, EnginePlace)) -> ArtifactAt {
    ArtifactAt {
        hero: hero.raw(),
        place: artifact_place(place),
    }
}

/// An artifact place on the wire.
pub(super) fn artifact_place(place: EnginePlace) -> ArtifactPlace {
    match place {
        EnginePlace::Worn(at) => ArtifactPlace::Worn {
            position: position(at),
        },
        EnginePlace::Backpack(index) => ArtifactPlace::Backpack {
            index: narrow(index),
        },
    }
}

/// A worn position on the wire.
fn position(Position(at): Position) -> u16 {
    narrow(at)
}

/// An engine index as the wire's `u16` (a client's own value echoed, or one below the ruleset's
/// bounds: never saturated in play).
fn narrow(index: usize) -> u16 {
    u16::try_from(index).unwrap_or(u16::MAX)
}

/// A path's step as the wire's `u32` (a path is bounded by its frame).
fn step(step: usize) -> u32 {
    u32::try_from(step).unwrap_or(u32::MAX)
}

/// A client's command in the engine. The engine validates every value against the state and the
/// actor, which the session gives (ADR-0008).
pub fn command_to_engine(command: GameCommand) -> EngineCommand {
    match command {
        GameCommand::EndTurn {} => EngineCommand::EndTurn,
        GameCommand::Adventure { command } => EngineCommand::Adventure(match command {
            AdventureCommand::MoveHero { hero, path } => EngineAdventure::MoveHero {
                hero: HeroId::new(hero),
                path: path.into_iter().map(pos_to_engine).collect(),
            },
            AdventureCommand::BuildBoat {
                object_type,
                anchor,
            } => EngineAdventure::BuildBoat {
                object_type: Handle::from_raw(object_type),
                anchor: pos_to_engine(anchor),
            },
        }),
        GameCommand::Answer { answer } => EngineCommand::Answer(answer_to_engine(answer)),
        GameCommand::Battle { command } => EngineCommand::Battle(battle_to_engine(command)),
        GameCommand::Manage { manage } => EngineCommand::Manage(manage_to_engine(manage)),
    }
}

/// A wire tile in the engine.
const fn pos_to_engine(p: Pos) -> h3_adventure::Pos {
    let Pos { x, y, level } = p;
    h3_adventure::Pos::new(x, y, level)
}

/// A wire answer in the engine.
const fn answer_to_engine(a: Answer) -> EngineAnswer {
    match a {
        Answer::Offer { consent } => EngineAnswer::Offer(match consent {
            Consent::Accept => EngineConsent::Accept,
            Consent::Decline => EngineConsent::Decline,
        }),
        Answer::LevelUp { offer } => EngineAnswer::LevelUp(match offer {
            SkillOffer::First => EngineOffer::First,
            SkillOffer::Second => EngineOffer::Second,
        }),
        Answer::Chest { choice } => EngineAnswer::Chest(match choice {
            ChestChoice::Gold => EngineChest::Gold,
            ChestChoice::Experience => EngineChest::Experience,
        }),
        Answer::Arena { choice } => EngineAnswer::Arena(match choice {
            ArenaChoice::Attack => EngineArena::Attack,
            ArenaChoice::Defense => EngineArena::Defense,
        }),
        Answer::School { choice } => EngineAnswer::School(match choice {
            SchoolChoice::First => EngineSchool::First,
            SchoolChoice::Second => EngineSchool::Second,
            SchoolChoice::Leave => EngineSchool::Leave,
        }),
        Answer::University { answer } => EngineAnswer::University(match answer {
            UniversityAnswer::Buy { slot } => EngineUniversity::Buy(slot),
            UniversityAnswer::Leave {} => EngineUniversity::Leave,
        }),
        Answer::Hire { answer } => EngineAnswer::Hire(match answer {
            HireAnswer::Buy { creature, amount } => EngineHire::Buy {
                creature: CreatureTypeId::from_raw(creature),
                amount,
            },
            HireAnswer::Close {} => EngineHire::Close,
        }),
        Answer::Close {} => EngineAnswer::Close,
    }
}

/// A wire management command in the engine.
fn manage_to_engine(m: Manage) -> EngineManage {
    match m {
        Manage::Army { command } => EngineManage::Army(match command {
            ArmyCommand::Move { from, to } => EngineArmy::Move {
                from: slot_to_engine(from),
                to: slot_to_engine(to),
            },
            ArmyCommand::Split { from, to, count } => EngineArmy::Split {
                from: slot_to_engine(from),
                to: slot_to_engine(to),
                count,
            },
            ArmyCommand::Dismiss { at } => EngineArmy::Dismiss {
                at: slot_to_engine(at),
            },
        }),
        Manage::Formation { hero, formation } => EngineManage::Formation {
            hero: HeroId::new(hero),
            formation: match formation {
                Formation::Loose => EngineFormation::Loose,
                Formation::Tight => EngineFormation::Tight,
            },
        },
        Manage::Tactics { hero, on } => EngineManage::Tactics {
            hero: HeroId::new(hero),
            on,
        },
        Manage::Artifact { from, to } => EngineManage::Artifact {
            from: artifact_at_to_engine(from),
            to: artifact_at_to_engine(to),
        },
        Manage::Assemble { hero, at } => EngineManage::Assemble {
            hero: HeroId::new(hero),
            at: Position(usize::from(at)),
        },
        Manage::Disassemble { hero, at } => EngineManage::Disassemble {
            hero: HeroId::new(hero),
            at: Position(usize::from(at)),
        },
    }
}

/// A wire army slot in the engine.
const fn slot_to_engine(s: ArmySlot) -> EngineSlot {
    let ArmySlot { army, slot } = s;
    EngineSlot {
        army: match army {
            ArmyRef::Hero { hero } => EngineArmyRef::Hero(HeroId::new(hero)),
            ArmyRef::Group {} => EngineArmyRef::Group,
            ArmyRef::Garrison { town } => EngineArmyRef::Garrison(TownId::new(town)),
        },
        slot,
    }
}

/// A wire artifact place of a hero in the engine.
fn artifact_at_to_engine(at: ArtifactAt) -> (HeroId, EnginePlace) {
    let ArtifactAt { hero, place } = at;
    (
        HeroId::new(hero),
        match place {
            ArtifactPlace::Worn { position } => EnginePlace::Worn(Position(usize::from(position))),
            ArtifactPlace::Backpack { index } => EnginePlace::Backpack(usize::from(index)),
        },
    )
}

/// The engine's reason for a rejection on the wire.
pub fn reject(error: GameCommandError) -> GameReject {
    use GameCommandError as E;
    match error {
        E::UnknownPlayer(player) => GameReject::UnknownPlayer {
            player: player.raw(),
        },
        E::NotYourTurn(player) => GameReject::NotYourTurn {
            player: player.raw(),
        },
        E::DecisionOpen => GameReject::DecisionOpen {},
        E::NoDecision => GameReject::NoDecision {},
        E::OfferGone => GameReject::OfferGone {},
        E::Adventure(error) => GameReject::Adventure {
            error: adventure_reject(error),
        },
        E::BattleOpen => GameReject::BattleOpen {},
        E::NoBattle => GameReject::NoBattle {},
        E::NotYourSide => GameReject::NotYourSide {},
        E::Battle(error) => GameReject::Battle {
            code: from_engine::reject(error),
        },
        E::NotForSystem => GameReject::NotForSystem {},
        E::WrongAnswer => GameReject::WrongAnswer {},
        E::WaitingForLevelUp(player) => GameReject::WaitingForLevelUp {
            player: player.raw(),
        },
        E::Manage(error) => GameReject::Manage {
            error: manage_reject(error),
        },
        E::SlotClosed => GameReject::SlotClosed {},
        E::CannotPay => GameReject::CannotPay {},
        E::Hire(error) => GameReject::Hire {
            error: match error {
                HireError::NoSource => HireReject::NoSource {},
                HireError::NotOffered(creature) => HireReject::NotOffered {
                    creature: creature.raw(),
                },
                HireError::Amount(amount) => HireReject::Amount { amount },
                HireError::NoRoom => HireReject::NoRoom {},
            },
        },
    }
}

/// The adventure's reason on the wire.
fn adventure_reject(error: AdventureCommandError) -> AdventureReject {
    use AdventureCommandError as E;
    match error {
        E::NotYourHero(hero) => AdventureReject::NotYourHero { hero: hero.raw() },
        E::EmptyPath => AdventureReject::EmptyPath {},
        E::NotNeighbour { step: s } => AdventureReject::NotNeighbour { step: step(s) },
        E::Unexplored { step: s } => AdventureReject::Unexplored { step: step(s) },
        E::Blocked { step: s, error } => AdventureReject::Blocked {
            step: step(s),
            error: match error {
                StandError::Outside => StandReject::Outside,
                StandError::Terrain => StandReject::Terrain,
                StandError::Blocked => StandReject::Blocked,
                StandError::Occupied => StandReject::Occupied,
                StandError::Visited => StandReject::Visited,
            },
        },
        E::Visit { step: s } => AdventureReject::Visit { step: step(s) },
        E::FromAbove { step: s } => AdventureReject::FromAbove { step: step(s) },
        E::PastVisit { step: s } => AdventureReject::PastVisit { step: step(s) },
        E::NoStop { step: s } => AdventureReject::NoStop { step: step(s) },
        E::TooFewPoints {
            step: s,
            cost,
            points,
        } => AdventureReject::TooFewPoints {
            step: step(s),
            cost,
            points,
        },
        E::Boat { step: s } => AdventureReject::Boat { step: step(s) },
        E::Landing { step: s } => AdventureReject::Landing { step: step(s) },
        E::Coast { step: s } => AdventureReject::Coast { step: step(s) },
        E::UnknownObject => AdventureReject::UnknownObject {},
        E::NotOwner => AdventureReject::NotOwner {},
        E::NoBoatTile => AdventureReject::NoBoatTile {},
        E::BoatTileTaken => AdventureReject::BoatTileTaken {},
    }
}

/// A management command's reason on the wire.
fn manage_reject(error: ManageError) -> ManageReject {
    match error {
        ManageError::NotYourHero(hero) => ManageReject::NotYourHero { hero: hero.raw() },
        ManageError::NotTogether => ManageReject::NotTogether {},
        ManageError::Army(error) => ManageReject::Army {
            error: match error {
                ArmyCommandError::NotYourHero(hero) => ArmyReject::NotYourHero { hero: hero.raw() },
                ArmyCommandError::NoGroup => ArmyReject::NoGroup {},
                ArmyCommandError::NoSlot(slot) => ArmyReject::NoSlot { slot },
                ArmyCommandError::NoStack(slot) => ArmyReject::NoStack { slot },
                ArmyCommandError::SameSlot => ArmyReject::SameSlot {},
                ArmyCommandError::TooManyCreatures => ArmyReject::TooManyCreatures {},
                ArmyCommandError::LastStack => ArmyReject::LastStack {},
                ArmyCommandError::OtherCreature => ArmyReject::OtherCreature {},
                ArmyCommandError::SplitCount(count) => ArmyReject::SplitCount { count },
                ArmyCommandError::Garrison(town) => ArmyReject::Garrison { town: town.raw() },
            },
        },
        ManageError::Artifact(error) => ManageReject::Artifact {
            error: match error {
                MoveError::NoArtifact(place) => MoveReject::NoArtifact {
                    place: artifact_place(place),
                },
                MoveError::Unmovable(artifact) => MoveReject::Unmovable {
                    artifact: artifact.raw(),
                },
                MoveError::SamePlace => MoveReject::SamePlace {},
                MoveError::CannotPlace(place) => MoveReject::CannotPlace {
                    place: artifact_place(place),
                },
                MoveError::CannotReturn(place) => MoveReject::CannotReturn {
                    place: artifact_place(place),
                },
            },
        },
        ManageError::Combine(error) => ManageReject::Combine {
            error: match error {
                CombineError::NoArtifact(at) => CombineReject::NoArtifact { at: position(at) },
                CombineError::Incomplete(at) => CombineReject::Incomplete { at: position(at) },
                CombineError::NotCombination(at) => {
                    CombineReject::NotCombination { at: position(at) }
                }
            },
        },
    }
}
