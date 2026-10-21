//! The whole game's setup (ADR-0015 point 4, E08 T6a): plain owned data a game is built and
//! validated from. A map is imported into it (`h3-map`); `h3-game` checks its own part (the
//! players, the heroes' types and starting armies) and hands the adventure context its setup,
//! which the context validates with the checks its commands use.

use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_adventure::{AdventureSetup, HeroSetup, MapSetup, Pos, SetupError};
use h3_content::{
    CreatureTypeId, Formation, HeroEffect, HeroType, HeroTypeId, Ruleset, SkillId, SpellId,
    StartingStack,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{ArtifactInstance, DetailsError, HeroDetails, OfferLevels};
use h3_kernel::Rng;

use crate::{GameRuleset, GameState};

/// A player of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSetup {
    /// The player's colour in the original's order (0 red to 7 pink): what a map's ownership
    /// and conditions name it by. Players act in colour order (sequential play).
    pub colour: u8,
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
    /// The hero type.
    pub hero_type: HeroTypeId,
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
    /// Worn artifacts and the backpack; `None`: the spell book if the hero type starts with one.
    pub artifacts: Option<HeroArtifactsSetup>,
    /// Learned spells; `None`: the hero type's.
    pub spells: Option<Vec<SpellId>>,
}

/// A hero's artifacts as a setup gives them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroArtifactsSetup {
    /// Worn artifacts, in any order.
    pub worn: Vec<ArtifactInstance>,
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
    pub heroes: Vec<GameHeroSetup>,
    /// The scenario's level limit (HER-EXP-003; the map header's byte, AB and SoD): no hero gains
    /// experience beyond its threshold; 0 none, as is a limit above the ruleset's highest level
    /// (`ADV-QUIRK-12`). E08 T29.
    pub level_limit: u8,
    /// The secondary skills the scenario bans (HER-LVL-003; the map's set, SoD): no level offers
    /// them. Each must be the ruleset's; their order and repeats do not matter. E08 T29.
    pub banned_skills: Vec<SkillId>,
}

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
    /// More heroes than a [`HeroId`] can name; the number.
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
            Self::Adventure(error) => write!(f, "adventure: {error}"),
        }
    }
}

impl core::error::Error for GameSetupError {}

impl GameSetup {
    /// Checks the setup: the game's part (a difficulty the ruleset has, players in colour order
    /// within the ruleset's players, banned skills of the ruleset (E08 T29), known hero types,
    /// starting armies that can be drawn, the heroes' details by the hero model, E08 T18b), then
    /// the adventure context's part with its own checks. It builds the game as
    /// [`GameState::new`] does: whether a setup is valid does not depend on the seed, since every
    /// starting army it can draw is valid when its hero type's army has a first stack and no more
    /// stacks than the army's slots (the core checks each stack's counts), and neither a drawn
    /// experience nor a war machine's artifact (worn only where its slot is free) makes the
    /// details invalid, nor do the levels at load (the details are checked at experience 0 before
    /// a custom experience is gained under the bound, and every level the hero model gives keeps
    /// them valid, GAM-SET-002).
    ///
    /// # Errors
    ///
    /// The first [`GameSetupError`] found, in that order.
    pub fn validate(&self, rules: &GameRuleset) -> Result<(), GameSetupError> {
        GameState::new(rules, self, 0).map(drop)
    }

    /// The adventure context's setup and the heroes' details: each starting army drawn from
    /// `draws` in hero order, then each default experience in hero order (GAM-HER-001); a hero
    /// with a custom experience at experience 0, which [`GameState::new`] gives it with its
    /// levels (GAM-SET-002).
    pub(crate) fn adventure_setup(
        &self,
        rules: &GameRuleset,
        draws: &mut Rng,
    ) -> Result<(AdventureSetup, Vec<HeroDetails>), GameSetupError> {
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
        if self.heroes.len() > usize::from(u16::MAX) + 1 {
            return Err(GameSetupError::Heroes(self.heroes.len()));
        }
        let mut heroes = Vec::with_capacity(self.heroes.len());
        let mut machines = Vec::with_capacity(self.heroes.len());
        for (hero, index) in self.heroes.iter().zip(0..=u16::MAX) {
            let id = HeroId::new(index);
            let hero_type = rules
                .core()
                .hero_type(hero.hero_type)
                .ok_or(GameSetupError::UnknownHeroType(id))?;
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
            machines.push((hero_type, given));
        }
        let mut details = Vec::with_capacity(self.heroes.len());
        for ((hero, (hero_type, given)), index) in
            self.heroes.iter().zip(machines).zip(0..=u16::MAX)
        {
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
        };
        Ok((setup, details))
    }
}

/// The lowest default experience, and the width of its draw (GAM-HER-001: 40 to 90).
const DEFAULT_EXPERIENCE: (u32, u32) = (40, 50);

/// A hero's details when the game is built (GAM-HER-001): the setup's customisations, else the
/// defaults (survey `E08-battles.md` §6.2): level 1; an experience of 40 + a draw of 0 to 50
/// from the setup stream, or 0 with a custom experience, which the levels at load give later
/// (GAM-SET-002, E08 T29); the class's primary skills; the hero type's secondary skills and
/// spells; worn, the spell book when the type starts with one (the first artifact type with the
/// `spell_book` effect); then the artifact of each war machine its starting army gave
/// (GAM-SET-001), where no worn artifact takes that slot; the maximum spell points
/// (HER-MAN-001).
// rule: GAM-HER-001
fn hero_details(
    core: &Ruleset,
    hero: &GameHeroSetup,
    hero_type: &HeroType,
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
                .and_then(h3_content::ArtifactTypeId::from_index)
                .filter(|_| hero_type.spell_book);
            (
                book.into_iter().map(ArtifactInstance::from).collect(),
                Vec::new(),
            )
        },
        |a| (a.worn.clone(), a.backpack.clone()),
    );
    let mut details = HeroDetails {
        hero_type: hero.hero_type,
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
    };
    for &machine in machines {
        let artifact = core
            .artifacts()
            .iter()
            .position(|a| a.war_machine == Some(machine))
            .and_then(h3_content::ArtifactTypeId::from_index);
        let slot = artifact
            .and_then(|id| core.artifact(id))
            .and_then(|a| a.slot);
        let taken = details
            .worn
            .iter()
            .filter_map(|item| core.artifact(item.artifact))
            .any(|a| a.slot == slot);
        if let Some(id) = artifact.filter(|_| !taken) {
            details.worn.push(id.into());
        }
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
