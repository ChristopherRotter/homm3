//! The raw model of a map's object templates, objects and timed events (E08 T3b). As in the rest
//! of the raw model, codes stay numbers and texts stay bytes; which body an object has follows
//! from its template's class (and for mines and the grail, its subclass), as in the game.

use super::{Artifacts, Bits, Pos, Text};

/// An object template ("def"): picture, footprint and class.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Template {
    /// Picture file name.
    pub def: Text,
    /// Passability (bit = passable), 6 rows of 8 tiles; bit `j` of byte `i` is the tile `7 − j`
    /// columns left of and `5 − i` rows above the bottom right corner (`docs/formats/h3m.md`).
    pub passable: [u8; 6],
    /// Visitability (bit = visitable), same order.
    pub visitable: [u8; 6],
    /// The first terrain mask (bit per terrain).
    pub terrains: u16,
    /// The second terrain mask (the editor's menu in our writer's reading; VCMI 1.7.5 takes it as
    /// the allowed terrains and ignores the first).
    pub editor_terrains: u16,
    /// Object class (34 hero, 54 monster, 98 town, …).
    pub class: u32,
    /// Object subclass (hero class, creature, faction, …).
    pub subclass: u32,
    /// Editor group (the game reads it to tell objects visited from above).
    pub group: u8,
    /// Drawing priority.
    pub priority: u8,
}

/// An object on the map: where its template's bottom right corner is, which template, and the
/// body its class carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    /// The template's bottom right corner (an object may reach past the map's top and left).
    pub pos: Pos,
    /// Index into [`super::RawMap::templates`].
    pub template: u32,
    /// The class's data.
    pub body: Body,
}

/// What an object carries beyond its template, by class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    /// No data (decorations, most visitable objects, banks, markets).
    None,
    /// 26: an event.
    Event(EventBody),
    /// 34 hero, 62 prison, 70 random hero.
    Hero(Box<HeroBody>),
    /// 54 monster, 71–75 and 162–164 random monsters.
    Monster(MonsterBody),
    /// 59 ocean bottle, 91 sign: a message.
    Sign(Text),
    /// 83: a seer hut; `None` when it has no quest.
    SeerHut(Option<SeerTask>),
    /// 113: a witch hut's possible skills (bit set = possible; AB and SoD, empty in RoE).
    WitchHut(Bits),
    /// 81: a scholar's bonus kind (−1 random, 0 primary, 1 secondary, 2 spell) and code.
    Scholar {
        /// Bonus kind.
        kind: i8,
        /// Bonus code.
        code: u8,
    },
    /// 33, 219: a garrison.
    Garrison {
        /// Owner (`0xFF` neutral; four bytes in the file).
        owner: u32,
        /// The troops.
        army: Army,
        /// Whether the troops may be taken out (AB and SoD; true in RoE).
        removable: bool,
    },
    /// 5 artifact, 65–69 random artifacts: the artifact follows from the template.
    Artifact(Option<Message>),
    /// 93: a spell scroll.
    Scroll {
        /// The guards' message.
        message: Option<Message>,
        /// Spell code.
        spell: u32,
    },
    /// 76 random resource, 79 resource.
    Resource {
        /// The guards' message.
        message: Option<Message>,
        /// Amount (0 = random).
        amount: u32,
    },
    /// 77 random town, 98 town.
    Town(Box<TownBody>),
    /// 53 mine with subclass below 7: its owner (`0xFF` neutral).
    Mine {
        /// Owner.
        owner: u32,
    },
    /// 53 or 220 with subclass 7 or more: an abandoned mine's possible resources (bit per
    /// resource, 4 bytes).
    AbandonedMine(Bits),
    /// 17–20 creature dwellings, 87 shipyard, 42 lighthouse: an owner.
    Owned {
        /// Owner (`0xFF` neutral).
        owner: u32,
    },
    /// 216–218: a random dwelling.
    RandomDwelling(RandomDwelling),
    /// 88–90: a shrine's spell (255 = none; VCMI 1.7.5 then picks one of the shrine's level).
    Shrine {
        /// Spell code.
        spell: u32,
    },
    /// 6: a Pandora's box.
    Pandora {
        /// The guards' message.
        message: Option<Message>,
        /// What it gives.
        rewards: Rewards,
    },
    /// 36 with subclass below 1000: the grail's radius.
    Grail {
        /// Radius in tiles.
        radius: i32,
    },
    /// 215: a quest guard; `None` when it has no quest.
    QuestGuard(Option<Quest>),
    /// 214: a hero placeholder (campaigns).
    HeroPlaceholder {
        /// Owner.
        owner: u8,
        /// Hero type (`0xFF` = by power rank).
        hero_type: u8,
        /// The power rank when the hero type is `0xFF`.
        power_rank: Option<u8>,
    },
}

/// Seven army slots; `None` is an empty slot. Codes past the creatures are random stacks by level
/// (the import resolves them).
pub type Army = Vec<Option<Stack>>;

/// A creature stack as stored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stack {
    /// Creature code (one byte in RoE, two from AB on).
    pub creature: u16,
    /// Count.
    pub count: u16,
}

/// A guards' message: the text, and the guards if any.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Message {
    /// Text.
    pub text: Text,
    /// Guards.
    pub guards: Option<Army>,
}

/// What an event or a Pandora's box gives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rewards {
    /// Experience.
    pub experience: u32,
    /// Spell points (signed).
    pub mana: i32,
    /// Morale (−3 … 3).
    pub morale: i8,
    /// Luck (−3 … 3).
    pub luck: i8,
    /// Resources, in resource order.
    pub resources: [i32; 7],
    /// Attack, defence, spell power, knowledge.
    pub primary: [u8; 4],
    /// Secondary skills (code, level).
    pub skills: Vec<(u8, u8)>,
    /// Artifact codes.
    pub artifacts: Vec<u16>,
    /// Spell codes.
    pub spells: Vec<u8>,
    /// Creatures.
    pub creatures: Vec<Stack>,
}

/// An event object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventBody {
    /// The guards' message.
    pub message: Option<Message>,
    /// What it gives.
    pub rewards: Rewards,
    /// Players it triggers for (bit per colour).
    pub players: u8,
    /// Computer players trigger it.
    pub computer: bool,
    /// It disappears after the first visit.
    pub remove_after_visit: bool,
}

/// A hero, a prison or a random hero.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroBody {
    /// Identifier quests refer to (AB and SoD).
    pub identifier: Option<u32>,
    /// Owner (`0xFF` neutral, for prisons).
    pub owner: u8,
    /// Hero type (`0xFF` = random).
    pub hero_type: u8,
    /// Name, if customised.
    pub name: Option<Text>,
    /// Experience: behind a flag in SoD, always stored before (0 = the hero type's own).
    pub experience: Option<u32>,
    /// Portrait, if customised.
    pub portrait: Option<u8>,
    /// Secondary skills, if customised.
    pub secondary_skills: Option<Vec<(u8, u8)>>,
    /// Army, if customised.
    pub army: Option<Army>,
    /// Formation (0 loose, 1 tight).
    pub formation: u8,
    /// Artifacts, if customised.
    pub artifacts: Option<Artifacts>,
    /// Patrol radius (`0xFF` = stays).
    pub patrol: u8,
    /// Biography, if customised (AB and SoD).
    pub biography: Option<Text>,
    /// Sex code (AB and SoD; `0xFF` default, and in RoE).
    pub sex: u8,
    /// AB's one spell byte (`0xFF` none, `0xFE` the hero type's own); `None` in RoE and SoD.
    pub spell: Option<u8>,
    /// SoD's spells (bit set = known; 9 bytes), if customised.
    pub spells: Option<Bits>,
    /// SoD's primary skills, if customised.
    pub primary_skills: Option<[u8; 4]>,
}

/// A wandering monster.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MonsterBody {
    /// Identifier quests refer to (AB and SoD).
    pub identifier: Option<u32>,
    /// Count (0 = random).
    pub count: u16,
    /// Disposition code (0 compliant … 4 savage).
    pub disposition: u8,
    /// A message with a treasure the monster gives when beaten, if set.
    pub treasure: Option<MonsterTreasure>,
    /// It never flees.
    pub never_flees: bool,
    /// It does not grow.
    pub no_growth: bool,
}

/// A monster's message and treasure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MonsterTreasure {
    /// Text.
    pub text: Text,
    /// Resources, in resource order.
    pub resources: [i32; 7],
    /// Artifact code, if any.
    pub artifact: Option<u16>,
}

/// A seer hut's quest and reward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeerTask {
    /// The quest. RoE stores only an artifact: its quest has no deadline and no texts.
    pub quest: Quest,
    /// The reward.
    pub reward: Reward,
}

/// A quest (AB and SoD, and RoE's artifact quest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    /// What to do.
    pub mission: Mission,
    /// Last day (−1 = none).
    pub deadline: i32,
    /// Text on the first visit.
    pub first_visit: Text,
    /// Text on later visits.
    pub next_visit: Text,
    /// Text on completion.
    pub completed: Text,
}

/// A quest's mission (codes 1–9 of the format).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mission {
    /// 1: reach a level.
    Level(u32),
    /// 2: reach primary skills.
    PrimarySkills([u8; 4]),
    /// 3: defeat the hero with this identifier.
    KillHero(u32),
    /// 4: defeat the monster with this identifier.
    KillCreature(u32),
    /// 5: bring artifacts.
    Artifacts(Vec<u16>),
    /// 6: bring creatures.
    Army(Vec<Stack>),
    /// 7: bring resources.
    Resources([u32; 7]),
    /// 8: be this hero type.
    Hero(u8),
    /// 9: be this player.
    Player(u8),
}

/// A seer hut's reward (codes 0–10 of the format).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reward {
    /// 0: nothing.
    Nothing,
    /// 1: experience.
    Experience(u32),
    /// 2: spell points.
    Mana(u32),
    /// 3: morale.
    Morale(i8),
    /// 4: luck.
    Luck(i8),
    /// 5: a resource.
    Resource {
        /// Resource code.
        resource: u8,
        /// Amount.
        amount: u32,
    },
    /// 6: a primary skill.
    PrimarySkill {
        /// Skill code.
        skill: u8,
        /// Amount.
        amount: u8,
    },
    /// 7: a secondary skill.
    SecondarySkill {
        /// Skill code.
        skill: u8,
        /// Level code.
        level: u8,
    },
    /// 8: an artifact.
    Artifact(u16),
    /// 9: a spell.
    Spell(u8),
    /// 10: creatures.
    Creatures(Stack),
}

/// A town or a random town.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TownBody {
    /// Identifier quests refer to (AB and SoD).
    pub identifier: Option<u32>,
    /// Owner (`0xFF` neutral).
    pub owner: u8,
    /// Name, if customised.
    pub name: Option<Text>,
    /// Garrison, if customised.
    pub garrison: Option<Army>,
    /// Formation (0 loose, 1 tight).
    pub formation: u8,
    /// Buildings.
    pub buildings: Buildings,
    /// Spells the mage guild must offer (bit set = required, 9 bytes; AB and SoD).
    pub required_spells: Option<Bits>,
    /// Spells the mage guild may not offer (bit set = banned, 9 bytes).
    pub banned_spells: Bits,
    /// The town's events.
    pub events: Vec<TownEvent>,
    /// SoD's alignment byte (`0xFF` = the owner's, and before SoD).
    pub alignment: u8,
}

/// A town's buildings as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Buildings {
    /// The standard set, with or without a fort.
    Standard {
        /// It has a fort.
        fort: bool,
    },
    /// Built and forbidden buildings (bit sets of 6 bytes).
    Custom {
        /// Built.
        built: [u8; 6],
        /// Forbidden.
        forbidden: [u8; 6],
    },
}

impl Default for Buildings {
    fn default() -> Self {
        Self::Standard { fort: false }
    }
}

/// A timed event: the map's own, or a town's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimedEvent {
    /// The editor's name for it.
    pub name: Text,
    /// Its message.
    pub message: Text,
    /// Resources it gives, in resource order.
    pub resources: [i32; 7],
    /// Players it applies to (bit per colour).
    pub players: u8,
    /// Human players are affected (SoD; true before).
    pub human: bool,
    /// Computer players are affected.
    pub computer: bool,
    /// First day.
    pub first_day: u16,
    /// Repeat interval in days (0 = once).
    pub interval: u16,
}

/// A town's event: a timed event plus buildings and creatures.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TownEvent {
    /// The timed event.
    pub event: TimedEvent,
    /// Buildings it builds (bit set of 6 bytes).
    pub buildings: [u8; 6],
    /// Creatures it adds per dwelling level.
    pub creatures: [u16; 7],
}

/// A random dwelling's restrictions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RandomDwelling {
    /// Owner (`0xFF` neutral).
    pub owner: u32,
    /// 216 and 217: the town it takes its faction from (identifier, 0 = none), and the faction
    /// set when there is none.
    pub faction: Option<(u32, Option<Bits>)>,
    /// 216 and 218: the lowest and highest level codes (0–6).
    pub levels: Option<(u8, u8)>,
}
