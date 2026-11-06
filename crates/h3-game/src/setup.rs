//! The whole game's setup (ADR-0015 point 4, E08 T6a): plain owned data a game is built and
//! validated from. A map is imported into it (`h3-map`); `h3-game` checks its own part (the
//! players, the heroes' types and starting armies) and hands the adventure context its setup,
//! which the context validates with the checks its commands use.

use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_adventure::{AdventureSetup, Bans, HeroSetup, MapSetup, Pos, Seat, SetupError};
use h3_content::{
    ArtifactTypeId, CreatureTypeId, Formation, HeroEffect, HeroType, HeroTypeId, ResourceId,
    Ruleset, SkillId, SpellId, StartingStack,
};
use h3_contracts::{ArmyStack, HeroId, HeroSex, PlayerId, TownId};
use h3_heroes::{ArtifactInstance, DetailsError, HeroDetails, OfferLevels};
use h3_kernel::Rng;
use h3_towns::{Town, TownSetup, TownStatic};

use crate::town::{self, HeroPool, PlayerStart};
use crate::{GameRuleset, GameState};

/// A player of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSetup {
    /// The player's colour in the original's order (0 red to 7 pink): what a map's ownership
    /// and conditions name it by. Players act in colour order (sequential play).
    pub colour: u8,
    /// Whether the player counts as human (GAM-BAT-010, E08 T54): a map's import sets it from the
    /// map's player kinds (a colour a human may play is human, a computer-only colour is not;
    /// ADV-Q27's answer), and whoever seats the players (an env, the server) may change it before
    /// the game is built. Read by the battles from the map alone (`BattleSetup.human`): the
    /// original lets a side without a human player off one bad-morale freeze in four
    /// (BAT-MOR-004, BAT-QUIRK-27). It does not decide who sends a player's commands.
    pub human: bool,
    /// Its start (E09 T7a1, design note `E09-design.md` §13): its factions, choice, pick, main
    /// town and random-hero flag; the default for a setup without towns.
    pub start: PlayerStart,
}

/// Where a hero's army comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeroArmy {
    /// The stacks the setup gives.
    Custom(Vec<ArmyStack>),
    /// The hero type's starting army, drawn when the game is built (E08 T6b): a random
    /// placeholder of the setup (ADR-0015 point 4).
    Starting,
}

/// A hero on the map as the game's setup places it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameHeroSetup {
    /// The hero type; `None` for a random hero (the map's class 70, E09 T7a1), whose type the game
    /// draws when it is built (TWN-SET-008, E09 T7b1).
    pub hero_type: Option<HeroTypeId>,
    /// The player owning it.
    pub owner: PlayerId,
    /// The tile it stands on.
    pub pos: Pos,
    /// Its army.
    pub army: HeroArmy,
    /// Its army's formation in battle (BAT-SET-001; the map's flag).
    pub formation: Formation,
    /// What the setup customises of its details (GAM-HER-001).
    pub details: HeroDetailsSetup,
}

/// What a setup customises of a hero's details (GAM-HER-001, E08 T18b): each field the map's, or
/// `None` for the default the game builds when the game is built.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroDetailsSetup {
    /// Experience; `None`: 40 to 90, drawn from the setup stream (a placeholder). A custom
    /// experience is gained when the game is built, from level 1 and experience 0 under the
    /// scenario's level limit, with the levels it gives (GAM-SET-002, E08 T29).
    pub experience: Option<u32>,
    /// Base primary skills (attack, defense, power, knowledge); `None`: the class's.
    pub primary: Option<[u8; 4]>,
    /// Secondary skills with their levels; `None`: the hero type's.
    pub skills: Option<Vec<(SkillId, u8)>>,
    /// Worn artifacts by position and the backpack; `None`: the spell book if the hero type starts
    /// with one.
    pub artifacts: Option<HeroArtifactsSetup>,
    /// Learned spells; `None`: the hero type's.
    pub spells: Option<Vec<SpellId>>,
    /// The hero's sex (an AB or SoD map's; E08 T54); `None`: the hero type's own. Only a battle's
    /// viewers read it (GAM-BAT-012: the hero's picture).
    pub sex: Option<HeroSex>,
}

/// A hero's artifacts as a setup gives them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroArtifactsSetup {
    /// Worn artifacts by position (E08 T49a; [`h3_heroes::positions`], one entry per position;
    /// a map's import keeps the map's positions). [`h3_heroes::wearing`] places a list.
    pub worn: Vec<Option<ArtifactInstance>>,
    /// The backpack, in order.
    pub backpack: Vec<ArtifactInstance>,
}

/// The difficulty a map's import sets: 1, normal in the original packs. The game's difficulty is
/// chosen when a game starts; a map's header holds only the map's rating (survey `E08-objects.md`
/// §7.2; E08 decision 16).
pub const DEFAULT_DIFFICULTY: u8 = 1;

/// Everything a game is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSetup {
    /// The players in colour order; a player's [`PlayerId`] is its index here.
    pub players: Vec<PlayerSetup>,
    /// The game's difficulty: a row of the ruleset's starting resources, from 0 (easy; 4 is
    /// impossible in the original packs). Every player starts with its human row (GAM-RES-002).
    pub difficulty: u8,
    /// The static map.
    pub map: MapSetup,
    /// The heroes on the map; a hero's [`HeroId`] is its index here (the game's store of heroes).
    /// Since E09 T7b2b the main towns' heroes the game generates follow them, in player order
    /// (TWN-SET-009).
    pub heroes: Vec<GameHeroSetup>,
    /// The scenario's level limit (HER-EXP-003; the map header's byte, AB and SoD): no hero gains
    /// experience beyond its threshold; 0 none, as is a limit above the ruleset's highest level
    /// (`ADV-QUIRK-12`). E08 T29.
    pub level_limit: u8,
    /// The secondary skills the scenario bans (HER-LVL-003; the map's set, SoD): no level offers
    /// them. Each must be the ruleset's; their order and repeats do not matter. E08 T29.
    pub banned_skills: Vec<SkillId>,
    /// The spells the scenario bans (GAM-SET-003; the map's set, SoD): no draw when the game is
    /// built gives them (a Shrine's, a Scholar's, the Pyramid's). Each must be the ruleset's; their
    /// order and repeats do not matter. E08 T61b.
    pub banned_spells: Vec<SpellId>,
    /// The artifacts the scenario bans (GAM-SET-003; the map's set, AB and SoD): no draw when the
    /// game is built gives them (a random artifact's, a chest's, a find's). Each must be the
    /// ruleset's; their order and repeats do not matter. E08 T61b.
    pub banned_artifacts: Vec<ArtifactTypeId>,
    /// The map's timed events in the map's order (GAM-CAL-004, E08 T83a; increment 9's design
    /// note §4): checked when the game is built, fired at their players' turn start from T85b.
    /// At most 65 536 (an event's [`TimedEventSetup::message`] is a `u16`).
    pub events: Vec<TimedEventSetup>,
    /// The towns as the map gives them, by `TownId` (the map's classes 77 and 98 in object order;
    /// E09 T7a1, design note `E09-design.md` §5 and §13); built with the game since E09 T7b2a
    /// (their factions, garrisons and buildings) and T7b2b (the main towns' heroes).
    pub towns: Vec<TownSetup>,
    /// The hero types the map allows in the pool (E09 T7a1; the pool: E09 T7b1, TWN-SET-005).
    pub allowed_heroes: Vec<HeroTypeId>,
    /// The hero types the map restricts to some colours, each with its colour mask (bit `c` for
    /// colour `c`; SoD's disposed heroes; E09 T7a1): the pool's masks (TWN-SET-005).
    pub hero_masks: Vec<(HeroTypeId, u8)>,
    /// The prisoners' hero types (the map's class 62; E09 T7a1): taken in the pool (TWN-SET-005).
    pub prisoners: Vec<HeroTypeId>,
}

/// A timed event as the map gives it (GAM-CAL-004, E08 T83a; survey `E08-economy.md` §11): on its
/// days, at the turn start of each of its players that is human or computer as it says, its
/// resources to that player's treasury, never below 0 (T85b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedEventSetup {
    /// The players it fires for: players of the game, in id order, each once (the map's colours
    /// that play; a colour nobody plays is left out, so an event may fire for none).
    pub players: Vec<PlayerId>,
    /// Whether it fires for a human player (the map's flag; before SoD always).
    pub human: bool,
    /// Whether it fires for a player that is not human.
    pub computer: bool,
    /// The first day it fires, from 1 (the file's day + 1, `map_timed_events` 0x4FC2E0).
    pub first_day: u16,
    /// It fires again every this many days; 0 once.
    pub interval: u16,
    /// What it gives (a negative amount takes): distinct resources of the ruleset, none 0.
    pub resources: Vec<(ResourceId, i32)>,
    /// Its index in the map's list, which names its message (the text is the client's).
    pub message: u16,
}

impl TimedEventSetup {
    /// Whether it fires on `day` at the turn start of `player`, human or not (GAM-CAL-004; survey
    /// `E08-economy.md` §11, `map_timed_events` 0x4CD910): `player` is one of its players, its
    /// flag for the player's kind is set, and `day` is its first day or, with an interval, a
    /// multiple of it after the first day.
    // rule: GAM-CAL-004
    pub fn fires(&self, player: PlayerId, human: bool, day: u32) -> bool {
        let first = u32::from(self.first_day);
        // An interval of 0 makes only the first day a multiple (`is_multiple_of(0)` is `== 0`).
        (if human { self.human } else { self.computer })
            && self.players.contains(&player)
            && day >= first
            && (day - first).is_multiple_of(u32::from(self.interval))
    }
}

/// What [`GameSetup::adventure_setup`] prepares for [`GameState::new`]: the adventure's setup,
/// the heroes' details by id, the hero pool, each town's record and static part by `TownId` and
/// the main towns' heroes (TWN-SET-009), whose ids follow the setup's heroes'.
pub(crate) type Prepared = (
    AdventureSetup,
    Vec<HeroDetails>,
    HeroPool,
    Vec<(Town, TownStatic)>,
    Vec<GameHeroSetup>,
);

/// Why a [`GameSetup`] cannot start a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSetupError {
    /// The ruleset has no starting resources for the difficulty.
    Difficulty(u8),
    /// More players than a [`PlayerId`] can name; the number.
    Players(usize),
    /// The players' colours are not strictly increasing or reach past the ruleset's players;
    /// the index of the first offending player.
    Colour(usize),
    /// More heroes than a [`HeroId`] can name; the number (since E09 T7b2b the setup's heroes
    /// and one per main town that generates a hero, TWN-SET-009).
    Heroes(usize),
    /// The hero's type is not part of the ruleset.
    UnknownHeroType(HeroId),
    /// The hero takes its type's starting army, which has no stacks or more stacks than the
    /// army's slots (the core checks each stack's counts).
    StartingArmy(HeroId),
    /// The hero's details break a rule of the hero model (`h3_heroes::validate`).
    Hero(HeroId, DetailsError),
    /// A banned skill is not the ruleset's (E08 T29).
    BannedSkill(SkillId),
    /// A banned spell is not the ruleset's (E08 T61b).
    BannedSpell(SpellId),
    /// A banned artifact is not the ruleset's (E08 T61b).
    BannedArtifact(ArtifactTypeId),
    /// More timed events than a `u16` names; the number (E08 T83a).
    Events(usize),
    /// The timed event at this index names a player twice, out of order or not of the game,
    /// a resource twice, not the ruleset's or of 0, or a first day of 0 (GAM-CAL-004, E08 T83a).
    Event(usize),
    /// The setup's towns do not give one per town of the static map, the objects whose type is of
    /// kind `Town` (E09 T8a1, the town epic's design note §13); their number.
    TownCount(usize),
    /// The town's faction is fixed and has no town record in the town part (TWN-SET-002, E09
    /// T7a2), or since E09 T7b2a the town is random and no faction has a town record (a random
    /// town takes a faction with one: its aligned colour's, its owner's or, neutral, one drawn
    /// among them).
    TownFaction(TownId),
    /// A random dwelling is tied to this town (E09 T11, ADV-DWL-005), which may take a faction
    /// without a random dwelling table in the adventure part: a fixed town its own, a random town
    /// any with a town record (whichever the build resolves or draws), so whether the setup is
    /// valid does not depend on a draw. The first such town in object order of the dwellings.
    LinkedTown(TownId),
    /// The random town is aligned to a colour no player has (TWN-SET-002, E09 T7a2): refused,
    /// though the original falls through to the town's owner (no shipped map has one).
    TownAlignment(TownId),
    /// The player's factions hold one without a town record, or its choice is a fixed faction
    /// outside them, random among none, or missing while its start needs a faction: a main town,
    /// the random-hero flag, a random hero of its own, a random town aligned to its colour
    /// (TWN-SET-001, E09 T7a2) or, since E09 T7b2a, a random town it owns at the start, which
    /// takes its faction (TWN-SET-002).
    Choice(PlayerId),
    /// The player's main town is not one of the setup's towns (TWN-SET-001, E09 T7a2), or since
    /// E09 T8a1 a town the player does not own at the start (another player's or neutral), or
    /// since E09 T7b2b it generates a hero and no tile of the map visits it (TWN-SET-009: the
    /// hero has no entrance to stand on).
    MainTown(PlayerId),
    /// The player's pick is not in its offer ([`GameSetup::offer`]; TWN-SET-006, E09 T7a2).
    Pick(PlayerId),
    /// The player's pick is an earlier player's too (TWN-SET-006, E09 T7a2).
    PickTaken(PlayerId),
    /// The random hero's owner is no player of the game (TWN-SET-008, E09 T7a2).
    RandomHeroOwner(HeroId),
    /// The draw finds no free hero type this player's colour may take for its random hero
    /// (TWN-SET-007, `TWN-QUIRK-4`; E09 T7b1) or, since E09 T7b2b, its main town's hero
    /// (TWN-SET-009).
    NoFreeHero(PlayerId),
    /// The adventure context rejects its part.
    Adventure(SetupError),
}

impl fmt::Display for GameSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Difficulty(d) => write!(f, "the ruleset has no difficulty {d}"),
            Self::Players(n) => write!(f, "{n} players are too many"),
            Self::Colour(player) => write!(f, "player {player}'s colour is out of order"),
            Self::Heroes(n) => write!(f, "{n} heroes are too many"),
            Self::UnknownHeroType(hero) => {
                write!(f, "hero {} has an unknown hero type", hero.raw())
            }
            Self::StartingArmy(hero) => {
                write!(
                    f,
                    "hero {}'s starting army does not fit an army",
                    hero.raw()
                )
            }
            Self::Hero(hero, error) => write!(f, "hero {}: {error}", hero.raw()),
            Self::BannedSkill(skill) => {
                write!(f, "banned skill {} is not the ruleset's", skill.raw())
            }
            Self::BannedSpell(spell) => {
                write!(f, "banned spell {} is not the ruleset's", spell.raw())
            }
            Self::BannedArtifact(artifact) => {
                write!(f, "banned artifact {} is not the ruleset's", artifact.raw())
            }
            Self::Events(n) => write!(f, "{n} timed events are too many"),
            Self::Event(event) => write!(f, "timed event {event} is invalid"),
            Self::TownCount(n) => write!(f, "{n} towns do not give one per town of the map"),
            Self::TownFaction(town) => {
                write!(f, "town {}'s faction has no town record", town.raw())
            }
            Self::LinkedTown(town) => write!(
                f,
                "town {}'s random dwellings may take a faction without a table",
                town.raw()
            ),
            Self::TownAlignment(town) => {
                write!(f, "town {} is aligned to no player's colour", town.raw())
            }
            Self::Choice(player) => {
                write!(f, "player {}'s faction choice is invalid", player.index())
            }
            Self::MainTown(player) => {
                write!(
                    f,
                    "player {}'s main town is not a town of its own",
                    player.index()
                )
            }
            Self::Pick(player) => write!(f, "player {}'s pick is not offered", player.index()),
            Self::PickTaken(player) => {
                write!(f, "player {}'s pick is an earlier player's", player.index())
            }
            Self::RandomHeroOwner(hero) => {
                write!(f, "random hero {}'s owner is no player", hero.raw())
            }
            Self::NoFreeHero(player) => {
                write!(f, "player {} draws a hero with none free", player.index())
            }
            Self::Adventure(error) => write!(f, "adventure: {error}"),
        }
    }
}

impl core::error::Error for GameSetupError {}

impl GameSetup {
    /// Checks the setup: the game's part (a difficulty the ruleset has, players in colour order
    /// within the ruleset's players, banned skills, then spells and artifacts of the ruleset (E08
    /// T29, T61b), the timed events (E08 T83a), the towns and the players' starts (E09 T7a2: since
    /// E09 T8a1 one town record per town of the static map, then a fixed town's faction with a
    /// town record, a random town's alignment to a player's colour, each player's faction choice,
    /// main town (since E09 T8a1 one it owns; since E09 T7b2b with an entrance on the map when it
    /// generates a hero), pick from its offer and a pick of no earlier player, a random hero's
    /// owner), the placed heroes' known hero types, a free type for each random hero's draw (E09
    /// T7b1, TWN-SET-007) and since E09 T7b2b for each main town's hero (TWN-SET-009), starting
    /// armies that can be drawn, the heroes' details by the hero model, E08 T18b), then the
    /// adventure context's part with its own checks (a main town's hero on an entrance a placed
    /// hero already stands on: `SetupError::HeroTile`, `TWN-QUIRK-6`). It builds the game as
    /// [`GameState::new`] does, with the seed 0: whether a setup is valid does not depend on the
    /// seed, since every starting army it can draw is valid when its hero type's army has a first
    /// stack and no more stacks than the army's slots (the core checks each stack's counts), and
    /// neither a drawn experience nor a war machine's artifact (worn only where its slot is free)
    /// makes the details invalid, nor do the levels at load (the details are checked at
    /// experience 0 before a custom experience is gained under the bound, and every level the
    /// hero model gives keeps them valid, GAM-SET-002). **Two exceptions, both from the heroes'
    /// draws** (E09 T7b1; since E09 T7b2b the main towns' heroes' too): [`GameSetupError::NoFreeHero`]
    /// when a colour mask ([`GameSetup::hero_masks`]) leaves out a drawing player's colour
    /// (without one every draw takes one free type whatever it draws, so the number of drawn
    /// heroes against the free types decides; with one, an earlier draw may take the last free
    /// type a later player's colour may take, or leave it), and [`GameSetupError::StartingArmy`]
    /// for a random hero with the starting army, or a main town's hero, when the pool holds a type
    /// without a valid one (no original hero type). A setup valid with the seed 0 may then fail with another. The towns'
    /// draws (E09 T7b2a: a random town's faction, a neutral town's garrison and buildings) keep
    /// it valid with the original packs; a town part whose neutral garrison gives more stacks than
    /// the army's slots, or counts past a stack's most, would make the adventure's
    /// `SetupError::Garrison` depend on them.
    ///
    /// # Errors
    ///
    /// The first [`GameSetupError`] found, in that order.
    pub fn validate(&self, rules: &GameRuleset) -> Result<(), GameSetupError> {
        GameState::new(rules, self, 0).map(drop)
    }

    /// The adventure context's setup, the heroes' details, the hero pool, the towns' records by
    /// `TownId` and the main towns' heroes: the pool, the players' random factions and the random
    /// heroes' types drawn first from `draws` (TWN-SET-005, TWN-SET-001, TWN-SET-007, TWN-SET-008:
    /// [`town::draw_heroes`], E09 T7b1), then the random towns' factions (TWN-SET-002:
    /// [`town::draw_towns`], E09 T7b2a), then each main town's hero's type in player order
    /// (TWN-SET-009: [`town::main_town_heroes`], E09 T7b2b; their ids after the setup's heroes),
    /// then each town's garrison and buildings in `TownId` order (TWN-SET-004, TWN-SET-003:
    /// [`town::build_towns`]; the garrisons and, since E09 T11, the towns' factions into the
    /// adventure's setup, for its random dwellings tied to a town), then each starting army in
    /// hero id order, then each default experience in hero id order (GAM-HER-001), the main towns'
    /// heroes last in both; a hero with a custom experience at experience 0, which
    /// [`GameState::new`] gives it with its levels (GAM-SET-002).
    pub(crate) fn adventure_setup(
        &self,
        rules: &GameRuleset,
        draws: &mut Rng,
    ) -> Result<Prepared, GameSetupError> {
        let count = self.players.len();
        let players = u8::try_from(count).map_err(|_| GameSetupError::Players(count))?;
        let params = rules.adventure().params();
        for (index, player) in self.players.iter().enumerate() {
            let after_previous = index
                .checked_sub(1)
                .is_none_or(|previous| self.players[previous].colour < player.colour);
            if !(after_previous && player.colour < params.max_players()) {
                return Err(GameSetupError::Colour(index));
            }
        }
        if let Some(&skill) = self
            .banned_skills
            .iter()
            .find(|&&skill| rules.core().skill(skill).is_none())
        {
            return Err(GameSetupError::BannedSkill(skill));
        }
        // rule: GAM-SET-003
        if let Some(&spell) = self
            .banned_spells
            .iter()
            .find(|&&spell| rules.core().spell(spell).is_none())
        {
            return Err(GameSetupError::BannedSpell(spell));
        }
        if let Some(&artifact) = self
            .banned_artifacts
            .iter()
            .find(|&&artifact| rules.core().artifact(artifact).is_none())
        {
            return Err(GameSetupError::BannedArtifact(artifact));
        }
        if self.events.len() > usize::from(u16::MAX) + 1 {
            return Err(GameSetupError::Events(self.events.len()));
        }
        // rule: GAM-CAL-004
        if let Some(index) = self
            .events
            .iter()
            .position(|event| !check_event(rules.core(), count, event))
        {
            return Err(GameSetupError::Event(index));
        }
        // E09 T7b2b: one hero more per main town that generates one (TWN-SET-009).
        let generating = self
            .players
            .iter()
            .filter(|player| player.start.main_town.is_some_and(|t| t.generates_hero))
            .count();
        let count = self.heroes.len() + generating;
        if count > usize::from(u16::MAX) + 1 {
            return Err(GameSetupError::Heroes(count));
        }
        let (owners, entrances) = town::check_starts(self, rules)?;
        let mut drawn = town::draw_heroes(self, rules, &owners, draws)?;
        let factions = town::draw_towns(self, rules, &owners, &drawn.factions, draws);
        let generated = town::main_town_heroes(self, rules, &entrances, &mut drawn, draws)?;
        let (garrisons, towns): (Vec<_>, Vec<_>) =
            town::build_towns(self, rules, &owners, &factions, draws)
                .into_iter()
                .map(|built| (built.garrison, (built.town, built.town_static)))
                .unzip();
        let all = || self.heroes.iter().chain(&generated);
        let mut heroes = Vec::with_capacity(count);
        let mut machines = Vec::with_capacity(count);
        for ((hero, &(type_id, hero_type)), index) in all().zip(&drawn.types).zip(0..=u16::MAX) {
            let id = HeroId::new(index);
            let (army, given) = match &hero.army {
                HeroArmy::Custom(stacks) => (stacks.clone(), Vec::new()),
                HeroArmy::Starting => {
                    let first = hero_type
                        .army
                        .split_first()
                        .filter(|_| hero_type.army.len() <= usize::from(params.army_slots()))
                        .ok_or(GameSetupError::StartingArmy(id))?;
                    draw_starting_army(first, hero_type, params.starting_stack_chances(), draws)
                }
            };
            heroes.push(HeroSetup {
                id,
                owner: hero.owner,
                pos: hero.pos,
                army,
                formation: hero.formation,
            });
            machines.push(((type_id, hero_type), given));
        }
        let mut details = Vec::with_capacity(count);
        for ((hero, (hero_type, given)), index) in all().zip(machines).zip(0..=u16::MAX) {
            let built = hero_details(rules.core(), hero, hero_type, &given, draws);
            h3_heroes::validate(&built, rules.core())
                .map_err(|error| GameSetupError::Hero(HeroId::new(index), error))?;
            details.push(built);
        }
        let setup = AdventureSetup {
            players,
            map: self.map.clone(),
            heroes,
            difficulty: self.difficulty,
            bans: Bans {
                skills: self.banned_skills.clone(),
                spells: self.banned_spells.clone(),
                artifacts: self.banned_artifacts.clone(),
            },
            // E08 T62b2: what an event tests (ADV-OBJ-007).
            seats: self
                .players
                .iter()
                .map(|player| Seat {
                    colour: player.colour,
                    human: player.human,
                })
                .collect(),
            // E09 T8a1: one garrison per town, by `TownId` (the setup's towns are the static
            // map's, `TownCount`); since E09 T7b2a each town's as built (TWN-SET-004).
            garrisons,
            // E09 T11: each town's faction from step 2, which a random dwelling tied to a town
            // takes (ADV-DWL-005). After the checks every town has one ([`town::draw_towns`]);
            // were one missing, none is given and the adventure refuses the count.
            town_factions: factions
                .iter()
                .copied()
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default(),
        };
        Ok((setup, details, drawn.pool, towns, generated))
    }
}

/// Whether a timed event fits the game (GAM-CAL-004, E08 T83a): its players those of the game in
/// id order, each once; its resources distinct and the ruleset's, none 0; its first day from 1.
fn check_event(core: &Ruleset, players: usize, event: &TimedEventSetup) -> bool {
    let players_fit = event.players.windows(2).all(|pair| pair[0] < pair[1])
        && event.players.iter().all(|player| player.index() < players);
    let mut seen = Vec::with_capacity(event.resources.len());
    let resources_fit = event.resources.iter().all(|&(resource, amount)| {
        let fresh = !seen.contains(&resource);
        seen.push(resource);
        fresh && amount != 0 && usize::from(resource.raw()) < core.resources().len()
    });
    players_fit && resources_fit && event.first_day >= 1
}

/// The lowest default experience, and the width of its draw (GAM-HER-001: 40 to 90).
const DEFAULT_EXPERIENCE: (u32, u32) = (40, 50);

/// A hero's details when the game is built (GAM-HER-001): the setup's customisations, else the
/// defaults (survey `E08-battles.md` §6.2): level 1; an experience of 40 + a draw of 0 to 50
/// from the setup stream, or 0 with a custom experience, which the levels at load give later
/// (GAM-SET-002, E08 T29); the class's primary skills; the hero type's secondary skills and
/// spells; worn, the spell book when the type starts with one (the first artifact type with the
/// `spell_book` effect, at its first position); then the artifact of each war machine its
/// starting army gave (GAM-SET-001) at the first position that takes it, where one does (no worn
/// artifact takes that slot); the maximum spell points (HER-MAN-001).
// rule: GAM-HER-001
fn hero_details(
    core: &Ruleset,
    hero: &GameHeroSetup,
    (type_id, hero_type): (HeroTypeId, &HeroType),
    machines: &[CreatureTypeId],
    draws: &mut Rng,
) -> HeroDetails {
    let custom = &hero.details;
    let (worn, backpack) = custom.artifacts.as_ref().map_or_else(
        || {
            let book = core
                .artifacts()
                .iter()
                .position(|a| a.effects.contains(&HeroEffect::SpellBook))
                .and_then(ArtifactTypeId::from_index)
                .filter(|_| hero_type.spell_book);
            let book: Vec<ArtifactInstance> = book.into_iter().map(Into::into).collect();
            (h3_heroes::wearing(core, &book), Vec::new())
        },
        |a| (a.worn.clone(), a.backpack.clone()),
    );
    let mut details = HeroDetails {
        hero_type: type_id,
        level: 1,
        experience: custom.experience.map_or_else(
            || {
                let (low, width) = DEFAULT_EXPERIENCE;
                low + draws.range_inclusive(0, width)
            },
            |_| 0,
        ),
        offered: OfferLevels::default(),
        primary: custom.primary.unwrap_or_else(|| {
            core.hero_class(hero_type.class)
                .map_or([0; 4], |class| class.primary)
        }),
        skills: custom
            .skills
            .clone()
            .unwrap_or_else(|| hero_type.skills.clone()),
        worn,
        backpack,
        spells: custom
            .spells
            .clone()
            .unwrap_or_else(|| hero_type.spells.clone()),
        mana: 0,
        // None at a map's start (HER-ADV-008): a map holds no hero's bonuses.
        bonuses: Default::default(),
    };
    for &machine in machines {
        let artifact = core
            .artifacts()
            .iter()
            .position(|a| a.war_machine == Some(machine))
            .and_then(ArtifactTypeId::from_index);
        let at = artifact.and_then(|id| {
            h3_heroes::first_position(&details, core, id).map(|at| (at, ArtifactInstance::from(id)))
        });
        at.and_then(|(at, item)| details.worn.get_mut(at.0).map(|place| (place, item)))
            .into_iter()
            .for_each(|(place, item)| *place = Some(item));
    }
    details.mana = h3_heroes::max_mana(&details, core);
    details
}

/// Draws a hero's starting army from its type's starting slots (GAM-SET-001): the first slot's
/// stack always; each later slot with its chance, a roll of 1 to 100 at most the chance, slots
/// past the chances never. The given stacks take the army's slots from 0 in order, each with a
/// count drawn between its bounds. The original's starting slots are the first stack, the war
/// machines, then the other stacks (its hero table has every war machine in the second of three
/// slots; the packs keep the war machines apart, `HeroType::war_machines`). A war machine's slot
/// is rolled like the others but gives no stack: the given war machines are returned, and the
/// hero wears their artifacts (GAM-HER-001, E08 T18b). Draws in the original's order
/// (0x4C9730): the first count, then per later slot its roll and, when given, its count.
// rule: GAM-SET-001
fn draw_starting_army(
    (first, rest): (&StartingStack, &[StartingStack]),
    hero_type: &HeroType,
    chances: &[u8],
    rng: &mut Rng,
) -> (Vec<ArmyStack>, Vec<CreatureTypeId>) {
    let mut army = vec![draw_stack(0, first, rng)];
    let mut machines = Vec::new();
    let later = hero_type
        .war_machines
        .iter()
        .map(|&machine| Err(machine))
        .chain(rest.iter().map(Ok));
    for (slot, &chance) in later.zip(chances) {
        if rng.range_inclusive(1, 100) > u32::from(chance) {
            continue;
        }
        match slot {
            Ok(stack) => {
                let next = u8::try_from(army.len()).unwrap_or(u8::MAX);
                army.push(draw_stack(next, stack, rng));
            }
            Err(machine) => machines.push(machine),
        }
    }
    (army, machines)
}

/// One stack of a starting army in army slot `slot`, its count drawn between its bounds.
fn draw_stack(slot: u8, stack: &StartingStack, rng: &mut Rng) -> ArmyStack {
    let [low, high] = stack.count;
    ArmyStack {
        slot,
        creature: stack.creature,
        count: rng.range_inclusive(u32::from(low), u32::from(high)),
    }
}
