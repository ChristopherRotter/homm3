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

use super::{
    ArmySlot, ArmyStack, Artifact, ArtifactPlace, DaySpells, Formation, HireSeen, HireTarget,
    MarketSeen, Month, Week,
};
use super::{
    GarrisonStack, GuildPart, HiredPart, MeetingPart, OfferLevels, Pos, PrimarySkill,
    TavernHeroHead, TavernPart, TownBuildings, VisitBonus,
};
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
        kind: Week,
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
    // E10 T3b1 (v88): the meeting's ally's hero as the visitor's owner sees it (GAM-TEM-003),
    // after its `MeetingOpened`, one part each in `MeetingPart`'s order.
    MeetingPartSeen {
        player: u8,
        hero: u16,
        part: MeetingPart,
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
    ExchangeOpened {
        player: u8,
        hero: u16,
        garrison: u32,
        object_type: u16,
        anchor: Pos,
        removable: bool,
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
        source: HireSeen,
    },
    /// E09 T39a (v73).
    TavernOpened {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    /// E08 T133b (v75); the hero optional and the place a town's too since E09 T58a1 (v85).
    MarketOpened {
        player: u8,
        hero: Option<u16>,
        place: MarketSeen,
    },
    /// E08 T134a (v76).
    AltarOpened {
        player: u8,
        hero: u16,
        object_type: u16,
        anchor: Pos,
    },
    /// E08 T134b1 (v77).
    HillFortOpened {
        player: u8,
        hero: u16,
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
        garrison_hero: bool,
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
    TavernOffered {
        player: u8,
        slot: u8,
        hero: Option<TavernHeroHead>,
    },
    TavernPartSeen {
        player: u8,
        slot: u8,
        part: TavernPart,
    },
    // E09 T38a1 (v71).
    HeroHired {
        player: u8,
        hero: u16,
        hero_type: u16,
        slot: Option<u8>,
        pos: Pos,
    },
    HiredPartSeen {
        player: u8,
        hero: u16,
        part: HiredPart,
    },
    // E08 T141a (v81): a Prison's prisoner freed (as a hire; its view for its owner alone, its
    // lists as `PrisonerPartSeen` and its other parts as `HiredPartSeen` following), the eight
    // heroes, an emptied Prison.
    PrisonerFreed {
        player: u8,
        hero: u16,
        hero_type: u16,
        pos: Pos,
        view: Option<TavernHeroHead>,
    },
    PrisonerPartSeen {
        player: u8,
        hero: u16,
        part: TavernPart,
    },
    PrisonFull {
        player: u8,
        hero: u16,
    },
    PrisonEmptied {
        player: u8,
        hero: u16,
    },
    // E09 T39b2 (v74): a Den of Thieves' Thieves' Guild, its parts following.
    ThievesGuildSeen {
        player: u8,
        level: u8,
    },
    GuildPartSeen {
        player: u8,
        part: GuildPart,
    },
    GarrisonChanged {
        player: u8,
        object_type: u16,
        anchor: Pos,
        owner: Option<u8>,
        level: u8,
        slots: bool,
        removable: Option<bool>,
    },
    GarrisonStackSeen {
        player: u8,
        object_type: u16,
        anchor: Pos,
        stack: GarrisonStack,
        slot: Option<u8>,
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
    // E08 T142a (v82): a cast on the map, before the spell points' `HeroChanged`.
    SpellCast {
        player: u8,
        hero: u16,
        spell: u16,
        mastery: u8,
        cost: u16,
    },
    // E08 T143a (v83): an accepted cast's charged failure, after its `SpellCast`.
    CastFailed {
        player: u8,
        hero: u16,
        spell: u16,
        reason: CastFailure,
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
    /// E09 T28a (v64).
    GarrisonEntered {
        hero: u16,
        town: u16,
        pos: Pos,
    },
    /// E09 T28a (v64).
    GarrisonLeft {
        hero: u16,
        town: u16,
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
    MovementChanged {
        hero: u16,
        movement: u32,
        max_movement: u32,
    },
    BonusChanged {
        hero: u16,
        morale: i8,
        luck: i8,
    },
    OnceMarkChanged {
        hero: u16,
        bonus: VisitBonus,
        held: bool,
    },
    TimedMarkChanged {
        hero: u16,
        object_type: u16,
        held: bool,
    },
    NothingGained {
        hero: u16,
        object_type: u16,
        anchor: Pos,
        why: NothingWhy,
    },
    SirensLoss {
        hero: u16,
        slot: u8,
        creature: u16,
        count: u32,
    },
    CreaturesUpgraded {
        hero: u16,
        slot: u8,
        from: u16,
        to: u16,
    },
    KindKnown {
        object_type: u16,
    },
    // E08 T142a (v82): a hero's day spells, told after the command.
    DaySpellsChanged {
        hero: u16,
        spells: DaySpells,
    },
    // E08 T143a (v83): Summon Boat's and Scuttle Boat's boats. A summoned boat's tile left and
    // tile now, each `None` where the viewer does not see heroes there.
    BoatSummoned {
        boat: u8,
        from: Option<Pos>,
        to: Option<Pos>,
        facing: u8,
    },
    BoatCreated {
        tile: Pos,
        boat_type: u8,
    },
    BoatScuttled {
        tile: Pos,
        boat: u8,
    },
}

/// Why a bonus visitable's visit gave nothing (`h3_adventure::NothingWhy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NothingWhy {
    /// The hero holds the kind's mark.
    Visited,
    /// The hero's spell points are at or above what the object gives.
    Full,
    /// The Magic Spring's entrance is emptied this week.
    Emptied,
}

/// Why an accepted cast on the map failed, its spell points spent (`h3_adventure::CastFailure`;
/// E08 T143a, v83).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastFailure {
    /// The chance draw came above the spell's chance.
    Chance,
    /// Summon Boat found no boat to bring and made none.
    NoBoat,
    /// The target, once revealed, held nothing the spell acts on.
    Target,
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
}
