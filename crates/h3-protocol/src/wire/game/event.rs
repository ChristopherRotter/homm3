//! The whole game's events (E16 M5 T3b): what happened, as one viewer may see it
//! (`h3_game::project`). The engine's `h3_game::GameEvent`, `h3_adventure::AdventureEventKind`
//! and `h3_heroes::HeroChange` one to one but for who sees an event: a projected event is the
//! viewer's alone, so its sets of viewers (`seen_by`, a step's `seen_from` and `seen_to`) are not
//! sent, and a step's or a jump's end the viewer does not see is `None`. A battle's event is the
//! battle wire's [`Event`].
//!
//! The same rules as [`super`]'s; the variants and fields are documented on the engine's types.
//! Provisional (E16 M5 part 1's review): the shapes, the viewer sets left out.

use serde::{Deserialize, Serialize};

use super::{ArmySlot, ArmyStack, Artifact, ArtifactPlace, Formation, HireTarget, Month};
use super::{GarrisonStack, OfferLevels, Pos, PrimarySkill, TownBuildings};
use crate::wire::{Event, Outcome};

/// One game event, projected for its viewer (`h3_game::GameEvent`).
#[allow(missing_docs)] // the variants and fields are documented on `h3_game::GameEvent`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GameEvent {
    TurnEnded {
        player: u8,
    },
    DayStarted {
        day: u32,
    },
    WeekStarted {
        week: u32,
    },
    MonthStarted {
        month: u32,
        kind: Month,
    },
    TurnStarted {
        player: u8,
    },
    TimedEventFired {
        player: u8,
        event: u16,
    },
    Adventure {
        event: AdventureEvent,
    },
    ResourceGained {
        player: u8,
        resource: u16,
        amount: u32,
    },
    OfferOpened {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    OfferRefused {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    OfferAnswered {
        player: u8,
        object_type: u16,
        anchor: Pos,
        accepted: bool,
    },
    /// A battle started: its heroes by side, attacker first, and its tile.
    BattleStarted {
        heroes: [Option<u16>; 2],
        tile: Pos,
    },
    Battle {
        event: Event,
    },
    BattleEnded {
        heroes: [Option<u16>; 2],
        result: Outcome,
    },
    HeroChanged {
        player: u8,
        hero: u16,
        change: HeroChange,
    },
    CreaturesRaised {
        player: u8,
        hero: u16,
        creature: u16,
        count: u32,
    },
    LevelUpOpened {
        player: u8,
        hero: u16,
        level: u8,
        primary: PrimarySkill,
        offers: [u16; 2],
    },
    LevelGained {
        player: u8,
        hero: u16,
        level: u8,
        primary: PrimarySkill,
        skill: Option<u16>,
    },
    ResourceLost {
        player: u8,
        resource: u16,
        amount: u32,
    },
    ResourcePaid {
        player: u8,
        resource: u16,
        amount: u32,
    },
    /// A slot of an army changed: the stack there now, none when it is empty.
    StackChanged {
        player: u8,
        at: ArmySlot,
        stack: Option<ArmyStack>,
    },
    FormationSet {
        player: u8,
        hero: u16,
        formation: Formation,
    },
    TacticsSet {
        player: u8,
        hero: u16,
        on: bool,
    },
    MeetingOpened {
        player: u8,
        heroes: [u16; 2],
    },
    ArmyDialogOpened {
        player: u8,
        hero: u16,
        stack: ArmyStack,
    },
    LeaveGuardsOpened {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    ChoiceOpened {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    UniversityOpened {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    HireOpened {
        player: u8,
        target: HireTarget,
        object_type: u16,
        anchor: Pos,
    },
    DecisionClosed {
        player: u8,
    },
    GroupLost {
        player: u8,
        hero: u16,
        stack: ArmyStack,
    },
    TownChanged {
        player: u8,
        town: u16,
        owner: Option<u8>,
        fort: u8,
        capitol: bool,
        built_today: bool,
        level: u8,
        slots: bool,
        hall: Option<u8>,
        buildings: Option<TownBuildings>,
    },
    TownStackSeen {
        player: u8,
        town: u16,
        stack: GarrisonStack,
        slot: Option<u8>,
    },
    TownIncomeSeen {
        player: u8,
        town: u16,
        resource: u16,
        amount: u32,
    },
    TownBuilt {
        player: u8,
        town: u16,
        building: u8,
        event: Option<u16>,
    },
    TownEventFired {
        player: u8,
        town: u16,
        event: u16,
    },
}

/// Something that happened on the adventure map, projected for its viewer
/// (`h3_adventure::AdventureEventKind`).
#[allow(missing_docs)] // the variants and fields are documented on `h3_adventure::AdventureEventKind`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AdventureEvent {
    /// A step: the tile left and the tile entered, each `None` where the viewer does not see
    /// the hero.
    HeroStepped {
        hero: u16,
        from: Option<Pos>,
        to: Option<Pos>,
    },
    ObjectRemoved {
        object_type: u16,
        anchor: Pos,
    },
    TilesRevealed {
        object_type: u16,
        count: u32,
    },
    KeyGained {
        key: u8,
    },
    HeroRemoved {
        hero: u16,
        pos: Pos,
    },
    CreaturesJoined {
        hero: u16,
        creature: u16,
        count: u32,
    },
    EventFired {
        hero: u16,
        tile: Pos,
    },
    MonsterChanged {
        object_type: u16,
        anchor: Pos,
        count_word: u8,
    },
    Boarded {
        hero: u16,
        boat: u8,
        at: Pos,
    },
    Landed {
        hero: u16,
        boat: u8,
        at: Pos,
        facing: u8,
    },
    MessageRead {
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    BoatRemoved {
        hero: u16,
        boat: u8,
        at: Pos,
    },
    ObjectFlagged {
        object_type: u16,
        anchor: Pos,
        owner: u8,
    },
    TownCaptured {
        town: u16,
        anchor: Pos,
        owner: u8,
    },
    GuardsChanged {
        object_type: u16,
        anchor: Pos,
        creature: Option<u16>,
        count_word: Option<u8>,
    },
    ObjectMarked {
        object_type: u16,
        anchor: Pos,
    },
    ObjectEmpty {
        object_type: u16,
        anchor: Pos,
    },
    ShipyardBlocked {
        object_type: u16,
        anchor: Pos,
        reason: ShipyardBlock,
    },
    BoatBuilt {
        tile: Pos,
        boat_type: u8,
    },
    NoneToRecruit {
        object_type: u16,
        anchor: Pos,
        creature: u16,
    },
    JoinRefused {
        hero: u16,
        creature: u16,
        count: u32,
    },
    MonsterAdded {
        object_type: u16,
        anchor: Pos,
        creature: u16,
        count_word: u8,
    },
    /// A jump: its two ends, each `None` where the viewer does not see the hero.
    HeroTeleported {
        hero: u16,
        from: Option<Pos>,
        to: Option<Pos>,
    },
    LeadsNowhere {
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    WhirlpoolLoss {
        hero: u16,
        slot: u8,
        creature: u16,
        count: u32,
    },
}

/// Why a Shipyard builds no boat (`h3_adventure::ShipyardBlock`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipyardBlock {
    /// It has no boat tile.
    NoTile,
    /// A boat or a hero is on its boat tile.
    Taken,
}

/// A change of a hero's details (`h3_heroes::HeroChange`).
#[allow(missing_docs)] // the variants and fields are documented on `h3_heroes::HeroChange`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HeroChange {
    GainExperience {
        amount: u32,
    },
    SetSpellPoints {
        points: u16,
    },
    AddArtifact {
        artifact: Artifact,
    },
    RemoveArtifact {
        place: ArtifactPlace,
    },
    PlaceArtifact {
        artifact: Artifact,
        at: ArtifactPlace,
    },
    LearnSpell {
        spell: u16,
    },
    GainLevel {
        primary: PrimarySkill,
        learned: Option<u16>,
        offered: OfferLevels,
    },
    /// Primary skills gained: attack, defense, power, knowledge.
    GainPrimary {
        primary: [u8; 4],
    },
    RaiseSkill {
        skill: u16,
        level: u8,
    },
    Bonus {
        morale: i8,
        luck: i8,
        mark: Option<u8>,
    },
    ClearBonuses {},
}
