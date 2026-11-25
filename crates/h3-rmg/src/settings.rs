//! The settings (ADR-0029 point 2; survey `docs/rules/surveys/E14-generator.md` §1.1 to §1.3,
//! §3.2.1): the Random Map tab's choices as the generator takes them, every one resolved but water,
//! whose Random the generator draws itself after its seed (RMG-SET-007).
//!
//! The original draws the tab's other Random choices from the game's earlier `rand` state
//! (`random_map_generate` 0x5860E0, before the generator's seed), which no port reproduces:
//! [`resolve_random`] applies the original's formulas to a stream the caller seeds (RMG-SET-001 to
//! -003). [`Settings::check`] refuses what the tab cannot give (the input is untrusted).

use h3_kernel::rng::CrtRand;

/// The number of player colours (red to pink).
pub const COLOURS: usize = 8;

/// The number of town types a colour can be given (Castle to Conflux; the original's 0 to 8).
pub const TOWN_TYPES: u8 = 9;

/// The map's size: the tab's four, the map square (RMG-SET-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapSize {
    /// S, 36 × 36.
    Small,
    /// M, 72 × 72.
    Medium,
    /// L, 108 × 108.
    Large,
    /// XL, 144 × 144.
    ExtraLarge,
}

impl MapSize {
    /// The map's width and height in tiles.
    pub const fn side(self) -> i32 {
        match self {
            Self::Small => 36,
            Self::Medium => 72,
            Self::Large => 108,
            Self::ExtraLarge => 144,
        }
    }
}

/// The tab's water choice (the generator's +0x10B8: 0 to 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Water {
    /// No water, 0.
    None,
    /// Normal water, 1.
    Normal,
    /// Islands, 2: the templates' size index halved (RMG-TPL-005).
    Islands,
    /// Random, 3: drawn after the seed, the generation's first draw (RMG-SET-007).
    Random,
}

impl Water {
    /// The original's number.
    pub const fn value(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Normal => 1,
            Self::Islands => 2,
            Self::Random => 3,
        }
    }
}

/// The tab's monster strength (the dialog's 0 to 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Monsters {
    /// Weak: the parameters' -1, the generator's 2.
    Weak,
    /// Normal: 0, the generator's 3.
    Normal,
    /// Strong: 1, the generator's 4.
    Strong,
}

impl Monsters {
    /// The parameters' value (`rmg_parameters` +0x48): the dialog's choice less 1.
    pub const fn parameter(self) -> i32 {
        match self {
            Self::Weak => -1,
            Self::Normal => 0,
            Self::Strong => 1,
        }
    }

    /// The generator's strength: the parameters' value plus 3, clamped to 1..=5
    /// (`rmg_generate_and_write` 0x54BF8C).
    // rule: RMG-SET-005
    pub const fn strength(self) -> i32 {
        let s = self.parameter() + 3;
        if s < 1 {
            1
        } else if s > 5 {
            5
        } else {
            s
        }
    }
}

/// The map's format, the generator's version (RMG-SET-004): the setup's 1 AB, 2 and 3 SoD, any
/// other RoE (`random_map_generate` 0x58622D).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapFormat {
    /// The Restoration of Erathia, version 0 (format 0x0E).
    RoE,
    /// Armageddon's Blade, version 1 (format 0x15).
    AB,
    /// The Shadow of Death, version 2 (format 0x1C).
    SoD,
}

impl MapFormat {
    /// The generator's version (+0x8).
    pub const fn version(self) -> i32 {
        match self {
            Self::RoE => 0,
            Self::AB => 1,
            Self::SoD => 2,
        }
    }

    /// The format from the setup's expansion value (+0x1898).
    // rule: RMG-SET-004
    pub const fn from_setup(value: i32) -> Self {
        match value {
            1 => Self::AB,
            2 | 3 => Self::SoD,
            _ => Self::RoE,
        }
    }
}

/// The settings a map is generated with: the tab's choices, each resolved but [`Water::Random`]
/// (RMG-SET-004). The counts are as the original takes them after `random_map_generate`: the
/// teams 1 to the players, the computer teams 1 to the computers (0 without computers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The map's size.
    pub size: MapSize,
    /// A second level, underground.
    pub two_levels: bool,
    /// Each colour's human flag, red first (the setup's slots).
    pub humans: [bool; COLOURS],
    /// Each colour's town type, 0 to 8 (Castle to Conflux); `None`: Random (the parameters' -1).
    pub towns: [Option<u8>; COLOURS],
    /// The tab's "human or computer" players, the humans among them: 1 to 8.
    pub players: u8,
    /// Their teams: 1 to the players.
    pub teams: u8,
    /// The tab's "computer only" players: at most 8 less the players.
    pub computers: u8,
    /// Their teams: 1 to the computers, 0 without computers.
    pub computer_teams: u8,
    /// Water.
    pub water: Water,
    /// Monster strength.
    pub monsters: Monsters,
    /// The map's format.
    pub format: MapFormat,
}

/// Why settings are refused (an untrusted input, `docs/06-security.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsError {
    /// Players outside 1 to 8.
    Players(u8),
    /// More humans than players.
    Humans {
        /// The colours marked human.
        humans: u8,
        /// The players.
        players: u8,
    },
    /// Teams outside 1 to the players.
    Teams {
        /// The teams.
        teams: u8,
        /// The players.
        players: u8,
    },
    /// More computers than 8 less the players.
    Computers {
        /// The computers.
        computers: u8,
        /// The players.
        players: u8,
    },
    /// Computer teams above the computers (RMG-QUIRK-6: the original turns them into the players'
    /// count), or teams without computers.
    ComputerTeams {
        /// The computer teams.
        teams: u8,
        /// The computers.
        computers: u8,
    },
    /// A colour's town type past Conflux.
    Town {
        /// The colour, 0 red.
        colour: u8,
        /// The town type.
        town: u8,
    },
}

impl core::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Players(p) => write!(f, "{p} players: 1 to 8"),
            Self::Humans { humans, players } => {
                write!(f, "{humans} humans among {players} players")
            }
            Self::Teams { teams, players } => {
                write!(f, "{teams} teams of {players} players: 1 to the players")
            }
            Self::Computers { computers, players } => {
                write!(
                    f,
                    "{computers} computers beside {players} players: 8 in all at most"
                )
            }
            Self::ComputerTeams { teams, computers } => write!(
                f,
                "{teams} computer teams of {computers} computers: 1 to the computers, none without"
            ),
            Self::Town { colour, town } => write!(f, "colour {colour}'s town type {town}: 0 to 8"),
        }
    }
}

impl Settings {
    /// The colours marked human.
    pub fn human_count(&self) -> u8 {
        human_count(&self.humans)
    }

    /// Checks what the tab can give: 1 to 8 players, the humans among them, 1 to the players
    /// teams, at most 8 players in all, 1 to the computers computer teams (RMG-QUIRK-6), town
    /// types 0 to 8.
    ///
    /// # Errors
    ///
    /// The first [`SettingsError`] in that order.
    // rule: RMG-SET-004
    pub fn check(&self) -> Result<(), SettingsError> {
        let players = self.players;
        if !(1..=8).contains(&players) {
            return Err(SettingsError::Players(players));
        }
        let humans = self.human_count();
        if humans > players {
            return Err(SettingsError::Humans { humans, players });
        }
        if !(1..=players).contains(&self.teams) {
            return Err(SettingsError::Teams {
                teams: self.teams,
                players,
            });
        }
        if self.computers > 8 - players {
            return Err(SettingsError::Computers {
                computers: self.computers,
                players,
            });
        }
        let teams_fit = if self.computers == 0 {
            self.computer_teams == 0
        } else {
            (1..=self.computers).contains(&self.computer_teams)
        };
        if !teams_fit {
            return Err(SettingsError::ComputerTeams {
                teams: self.computer_teams,
                computers: self.computers,
            });
        }
        for (colour, town) in (0u8..).zip(self.towns) {
            if let Some(town) = town.filter(|&t| t >= TOWN_TYPES) {
                return Err(SettingsError::Town { colour, town });
            }
        }
        Ok(())
    }
}

fn human_count(humans: &[bool; COLOURS]) -> u8 {
    humans.iter().fold(0, |n, &h| n + u8::from(h))
}

/// The tab's choices, `None` a Random one ([`resolve_random`]). The teams' 0 is the tab's "none".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choices {
    /// The map's size.
    pub size: MapSize,
    /// A second level.
    pub two_levels: bool,
    /// Each colour's human flag.
    pub humans: [bool; COLOURS],
    /// Each colour's town type (`None` Random, which the generator draws).
    pub towns: [Option<u8>; COLOURS],
    /// The players.
    pub players: Option<u8>,
    /// Their teams; 0 or more than the players: the players.
    pub teams: Option<u8>,
    /// The computers; more than 8 less the players: 8 less the players.
    pub computers: Option<u8>,
    /// Their teams; 0: the computers.
    pub computer_teams: Option<u8>,
    /// Water (its Random goes on, RMG-SET-007).
    pub water: Water,
    /// Monster strength.
    pub monsters: Option<Monsters>,
    /// The map's format.
    pub format: MapFormat,
}

/// The original's draws for the tab's Random choices and its adjustments of the counts, in
/// `random_map_generate`'s order, from `rand` (the caller's stream; the original's is the game's
/// state before the generator's seed): the players `h + rand() % (9 - h)`, `h` the humans
/// (0x586168); their teams `rand() % players + 1` (0x586183), 0 or more than the players the
/// players; with 8 players no computers; else the computers `rand() % (9 - players)` (0x5861B1),
/// at most 8 less the players; with computers their teams `rand() % computers + 1` (0x5861E4), 0
/// the computers; the monsters `rand() % 3` (0x58621D) weak, normal, strong. The result is
/// checked ([`Settings::check`]).
///
/// # Errors
///
/// A [`SettingsError`] where the original would go wrong or the tab cannot give the choices: no
/// players (the original's teams draw divides by them, RMG-QUIRK-3), computer teams above the
/// computers (it gives them the players' count, RMG-QUIRK-6), or what [`Settings::check`] refuses.
// rule: RMG-SET-001, RMG-SET-002, RMG-SET-003
pub fn resolve_random(choices: &Choices, rand: &mut CrtRand) -> Result<Settings, SettingsError> {
    let h = human_count(&choices.humans);
    let players = match choices.players {
        Some(p) => p,
        None => h.saturating_add(draw(rand, 9u8.saturating_sub(h))),
    };
    if players == 0 {
        return Err(SettingsError::Players(0));
    }
    let teams = match choices.teams {
        Some(t) => t,
        None => draw(rand, players) + 1,
    };
    let teams = if teams == 0 || teams > players {
        players
    } else {
        teams
    };
    let (computers, computer_teams) = if players >= 8 {
        (0, 0)
    } else {
        let computers = match choices.computers {
            Some(c) => c,
            None => draw(rand, 9 - players),
        }
        .min(8 - players);
        let computer_teams = if computers == 0 {
            0
        } else {
            match choices.computer_teams {
                Some(0) => computers,
                Some(t) => t,
                None => draw(rand, computers) + 1,
            }
        };
        (computers, computer_teams)
    };
    let monsters = match choices.monsters {
        Some(m) => m,
        None => match draw(rand, 3) {
            0 => Monsters::Weak,
            1 => Monsters::Normal,
            _ => Monsters::Strong,
        },
    };
    let settings = Settings {
        size: choices.size,
        two_levels: choices.two_levels,
        humans: choices.humans,
        towns: choices.towns,
        players,
        teams,
        computers,
        computer_teams,
        water: choices.water,
        monsters,
        format: choices.format,
    };
    settings.check()?;
    Ok(settings)
}

/// `rand() % n` for `n` of 1 to 9 (the callers' moduli are never 0).
fn draw(rand: &mut CrtRand, n: u8) -> u8 {
    let r = rand.rand() % u32::from(n.max(1));
    // `r` is below `n`, a `u8`.
    u8::try_from(r).unwrap_or(0)
}
