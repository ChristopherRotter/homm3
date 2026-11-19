//! The map's situations (E24 M1 T3b2, *provisional*): what the capture set shows the original
//! and compares with the viewer's still (`clients/map/situations/`, the harness's `h3map.py`).
//! A situation is a map built with the adventure lane's writer (`h3_map::writer::write_with`), a
//! view and the tiles the viewer has not explored. Its file (TOML) gives the map's side, its
//! terrain, patches and roads as the writer takes them, tiles copied from a rectangle of a
//! shipped map (its transitions, rivers and roads as the editor wrote them; the file names the
//! map, the harness's copy of the installation holds it) and tiles given one by one (a later
//! one replaces an earlier on its tile), objects (E24 M1 T4a: a copy's objects with its tiles,
//! and objects given by their template's DEF, its line of the game's `Objects.txt`), the heroes
//! of up to eight colours and the boats without a hero (E24 M1 T5a: each with the facing the
//! harness turns it to), the view's top-left and the rectangles left unexplored. The view on the
//! surface; since E24 M3 T6b2 an underground level of terrains alone. A situation that names a
//! shipped map whole (`map`, E24 M1 T4b)
//! is the harness's alone: it loads that map, this module does not read it.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_map::RawMap;
use h3_map::model::{Mission, Rewards, Stack};
use h3_map::writer::{
    ARMY_SLOTS, ARTIFACT_OBJECT, ARTIFACT_SLOTS, AnyObject, AnyTown, EVENT_OBJECT, Event, Extras,
    GivenTile, Guards, Hero, HeroArtifacts, Map, MapObject, Message, Monster, ObjectTemplate,
    PANDORA_OBJECT, Patch, Pile, Player, RewardBody, RewardObject, RiverKind, Road, RoadKind,
    SHRINE_OBJECTS, Terrain, Town, TownBuildings, Underground, WITCH_HUT_OBJECT,
};

use serde::Deserialize;

/// The spell book's artifact and its worn slot (`ArtSlots.txt`: 17), and the spells a book holds
/// (E24 M4 T2d3).
const SPELL_BOOK: u16 = 0;
const SPELL_BOOK_SLOT: usize = 17;
const SPELLS: u8 = 70;

/// One situation as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Situation {
    /// The map's side (36, 72, 108 or 144).
    pub size: u8,
    /// The terrain of every tile nothing else covers.
    pub terrain: Ground,
    /// Rectangles of other terrains, as the writer takes them (corners inclusive).
    #[serde(default)]
    pub patches: Vec<PatchSpec>,
    /// The underground level (E24 M3 T6b2, design §4.14: the underground's button and the
    /// minimap's level), its terrain and patches as the writer takes them, no objects; none: the
    /// surface only. The harness explores both levels but `hidden`'s rectangles on the surface.
    #[serde(default)]
    pub underground: Option<UndergroundSpec>,
    /// Straight roads along a row, as the writer takes them.
    #[serde(default)]
    pub roads: Vec<RoadSpec>,
    /// Rectangles copied from shipped maps.
    #[serde(default)]
    pub copy: Vec<CopySpec>,
    /// Tiles given one by one, after the copies.
    #[serde(default)]
    pub tiles: Vec<TileSpec>,
    /// Objects given one by one, after the copies' (in this order: the map's order).
    #[serde(default)]
    pub objects: Vec<ObjectSpec>,
    /// The towns (E24 M1 T5b1), after the heroes and before the objects in the map's order.
    #[serde(default)]
    pub towns: Vec<TownSpec>,
    /// The heroes (E24 M1 T5a), the map's first objects in this order: Red's (the human player)
    /// and another colour's at least, at most eight of a colour; every colour with a hero a
    /// player.
    pub heroes: Vec<HeroSpec>,
    /// The boats without a hero (E24 M1 T5a), the map's last objects in this order.
    #[serde(default)]
    pub boats: Vec<BoatSpec>,
    /// The view's top-left tile and level (0).
    pub view: [i32; 3],
    /// Rectangles of tiles Red has not explored (corners inclusive); every other tile explored.
    #[serde(default)]
    pub hidden: Vec<[i32; 4]>,
    /// The explored tiles as the new game left them (`"start"`, E24 M4 T3a: the engine's start
    /// compared with the original's), not written by the harness; none: `hidden`'s.
    #[serde(default)]
    pub explored: Option<Explored>,
    /// What the harness selects from its list after its walks and writes (E24 M3 T4a; design
    /// §4.5): the selection scrolls its list to show it and redraws both lists (and builds the
    /// status window again, M3 T5b1), so a situation whose heroes or towns the harness writes
    /// into needs one, of another hero or town than the turn's start selects.
    #[serde(default)]
    pub select: Option<SelectSpec>,
    /// Red's resources the harness writes before its selection (E24 M3 T5a, design §4.7), in the
    /// original's order: a hero's selection draws the resource bar again (`hero_select`
    /// 0x417F48), a town's does not, so a situation that gives them selects a hero.
    #[serde(default)]
    pub resources: Option<[i32; 7]>,
    /// The game's date the harness writes before its selection (E24 M3 T5a): [month, week, day],
    /// a week from 1 to 4 and a day from 1 to 7; as the resources, with a hero's selection.
    #[serde(default)]
    pub date: Option<[u16; 3]>,
    /// The hero of Red's (on its tile) the harness selects back after `select` (E24 M4 T3b1,
    /// design §5.10): the turn start's, so that the resources written show with the start's
    /// selection, as the engine's start has it; with a hero's `select` only.
    #[serde(default)]
    pub reselect: Option<[u8; 2]>,
    /// The players' starting bonus the harness chooses (E24 M4 T3b1): `"gold"`, so that no
    /// artifact changes a hero (the engine has no starting bonus yet, E10's: the situation writes
    /// the resources); none the scenario's random.
    #[serde(default)]
    pub bonus: Option<Bonus>,
    /// The status window's kind the harness asks for after its selection (E24 M3 T5b2, design
    /// §4.9): 1 the date's, 2 the kingdom's, until long after the still (the manager's +0x398 and
    /// +0x39C, as `status_window_set_kind` 0x415D60 sets them); the next view draw builds it.
    #[serde(default)]
    pub status: Option<u8>,
    /// The players' teams by colour, which the harness writes into the game (E24 M3 T5b2, design
    /// §4.9: the kingdom's window shows the players of Red's team and the others; the writer's
    /// maps have none, each player its own), each from 0 to 7.
    #[serde(default)]
    pub teams: Option<[u8; 8]>,
    /// What the viewer knows of objects, which the harness writes into the game after the view
    /// is set (E24 M3 T6a2a, design §4.11): the hint over a map object of a class with visit
    /// marks shows them (the viewer takes them from the live part's `[hover]`, not from here).
    #[serde(default)]
    pub visits: Vec<VisitSpec>,
    /// The harness chooses each of Red's heroes and towns in turn after the path is shown, and
    /// then the staging's selection again (E24 M3 T6b1, design §4.13): what each choice shows,
    /// the live part's `[chosen]` (the viewer takes it from there, not from here).
    #[serde(default)]
    pub choose: bool,
    /// The point of the screen the harness moves the mouse to last, over the view (E24 M4 T3b2,
    /// design §5.11): the still shows the map's hint and cursor there, the selected hero's cost of
    /// that tile read as a sequence's (`look_over_view`); none the parking over the right panel.
    #[serde(default)]
    pub pointer: Option<[i32; 2]>,
}

/// What the viewer knows of the object anchored on a tile (E24 M3 T6a2a): whether Red knows its
/// kind's bonus, and whether it was visited (by the selected hero, where the class keeps it so)
/// or as the new game left it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisitSpec {
    /// The object's anchor.
    pub at: [u8; 2],
    /// Red knows its kind's bonus.
    #[serde(default)]
    pub known: bool,
    /// Visited or not; none as the new game left it.
    #[serde(default)]
    pub visited: Option<bool>,
    /// The selected hero's own (E24 M3 T6a2b, design §4.12): a Tree of Knowledge's visit, a
    /// Shrine's spell known, a Witch Hut's skill learned; none as the new game left it.
    #[serde(default)]
    pub hero: Option<bool>,
    /// A mine's or a creature bank's guards as [creature, count] stacks (E24 M3 T6a2b), in its
    /// army's slots; none as the new game left them, empty none.
    #[serde(default)]
    pub guards: Option<Vec<[u32; 2]>>,
    /// A wandering monster's aggression (class 54: its tile word's bits 12 to 16, signed; E24 M4
    /// T2b1, design §5.4), which the map's load draws from its disposition; none as drawn.
    #[serde(default)]
    pub aggression: Option<i8>,
    /// A treasure chest's gold (class 101: its word's bits 11 to 14 in 500s, the artifact's bit
    /// 10 cleared; T2b1), which the load draws; none as drawn.
    #[serde(default)]
    pub gold: Option<u32>,
    /// A Tree of Knowledge's price (class 102: its word's bits 13 to 15, 0 free, 1 2 000 gold, 2
    /// ten gems; T2b1), which the load draws; none as drawn.
    #[serde(default)]
    pub price: Option<u8>,
}

/// A Seer's Hut's or a Quest Guard's quest (E24 M3 T6a2b, design §4.12), the writer's mission
/// (`h3_map::writer::Extras::quests`): the hero and the monster to defeat named by their tiles.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum QuestSpec {
    /// Reach this level.
    Level(u32),
    /// Reach these primary skills (attack, defense, power, knowledge).
    Skills([u8; 4]),
    /// Defeat the hero standing on this tile, one of the situation's.
    Hero([u8; 2]),
    /// Defeat the monster anchored on this tile, one of the situation's objects.
    Monster([u8; 2]),
    /// Bring these artifacts.
    Artifacts(Vec<u16>),
    /// Bring these [creature, count] stacks.
    Creatures(Vec<[u16; 2]>),
    /// Bring these resources (in the original's order).
    Resources([u32; 7]),
    /// Be the hero of this type.
    BeHero(u8),
    /// Belong to this colour.
    Player(u8),
}

/// What a situation selects (E24 M3 T4a): Red's hero standing on a tile (after its walk) or Red's
/// town anchored on one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum SelectSpec {
    /// The hero on this tile.
    Hero([u8; 2]),
    /// The town anchored on this tile.
    Town([u8; 2]),
}

/// A terrain by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ground {
    /// Dirt.
    Dirt,
    /// Sand.
    Sand,
    /// Grass.
    Grass,
    /// Snow.
    Snow,
    /// Swamp.
    Swamp,
    /// Rough.
    Rough,
    /// Subterranean.
    Subterranean,
    /// Lava.
    Lava,
    /// Water.
    Water,
    /// Rock.
    Rock,
}

/// A river by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiverName {
    /// Clear.
    Clear,
    /// Icy.
    Icy,
    /// Muddy.
    Muddy,
    /// Lava.
    Lava,
}

/// A road by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoadName {
    /// Dirt.
    Dirt,
    /// Gravel.
    Gravel,
    /// Cobblestone.
    Cobblestone,
}

/// A patch of terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchSpec {
    /// Its terrain.
    pub terrain: Ground,
    /// Its top-left tile.
    pub from: [u8; 2],
    /// Its bottom-right tile.
    pub to: [u8; 2],
}

impl PatchSpec {
    /// The writer's patch.
    fn writer(&self) -> Patch {
        Patch {
            terrain: self.terrain.writer(),
            from: (self.from[0], self.from[1]),
            to: (self.to[0], self.to[1]),
        }
    }
}

/// The underground level of a situation (E24 M3 T6b2): a base terrain and rectangles of others.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndergroundSpec {
    /// The terrain of every tile no patch covers.
    pub terrain: Ground,
    /// Rectangles of other terrains (corners inclusive).
    #[serde(default)]
    pub patches: Vec<PatchSpec>,
}

/// A road along a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoadSpec {
    /// Its kind.
    pub kind: RoadName,
    /// The row.
    pub row: u8,
    /// Its first column.
    pub from: u8,
    /// Its last column.
    pub to: u8,
}

/// Which explored tiles a situation keeps (E24 M4 T3a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Explored {
    /// The new game's: the harness writes none (`h3map.py`'s `explored = "start"`).
    Start,
}

/// A rectangle of a shipped map's tiles copied onto the map.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopySpec {
    /// The shipped map's file name (in the installation's `Maps`).
    pub map: String,
    /// The rectangle's top-left tile there.
    pub from: [u8; 2],
    /// Its bottom-right tile.
    pub to: [u8; 2],
    /// Its level there (0 the surface).
    #[serde(default)]
    pub level: u8,
    /// Where its top-left goes on the situation's map.
    pub at: [u8; 2],
    /// Also the objects anchored in the rectangle (on its level), moved with its tiles, in the
    /// shipped map's order, each with its template and the editor's body, neutral. Heroes,
    /// prisons, random heroes and hero placeholders are left out (their bodies name a hero: the
    /// writer writes only the situation's two).
    #[serde(default)]
    pub objects: bool,
}

/// An object given by its template: the first line of the game's `Objects.txt` with its DEF
/// (ignoring case) and, when given, its class and subclass.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSpec {
    /// The template's DEF.
    pub def: String,
    /// The class, to choose among the DEF's templates.
    #[serde(default)]
    pub class: Option<u32>,
    /// The subclass, likewise.
    #[serde(default)]
    pub subclass: Option<u32>,
    /// Its anchor (the bottom-right corner of its picture), which may lie past the map's right
    /// and bottom edges (the writer's limits).
    pub at: [u8; 2],
    /// Its owner's colour (0 Red … 7 Pink; E24 M1 T5b1), for a class whose body names one (the
    /// writer's `AnyObject`): a colour with a hero, which makes it a player. None is neutral.
    #[serde(default)]
    pub owner: Option<u8>,
    /// A Seer's Hut's or a Quest Guard's quest (E24 M3 T6a2b); none without one.
    #[serde(default)]
    pub quest: Option<QuestSpec>,
    /// A resource pile's amount (class 79; E24 M4 T2a1, design §5.3: gold in units, as the
    /// writer's `Pile`); none the game's draw.
    #[serde(default)]
    pub amount: Option<u32>,
    /// A Shrine's spell (88 to 90; T2a1); none the game's draw.
    #[serde(default)]
    pub spell: Option<u8>,
    /// The skills a Witch Hut may teach (113; T2a1): one of them drawn; none the editor's.
    #[serde(default)]
    pub skills: Option<Vec<u8>>,
    /// An event's rewards (26; T2a1): it fires for every colour, a human's hero alone, once.
    /// Since E24 M4 T2b1 Pandora's Box's (6) too, without a message.
    #[serde(default)]
    pub rewards: Option<RewardsSpec>,
    /// A wandering monster's body (54; E24 M4 T2b1, design §5.4): none the editor's (a random
    /// count, aggressive).
    #[serde(default)]
    pub monster: Option<MonsterSpec>,
    /// The guards of a resource pile (79, with its `amount` or a drawn one) or an artifact (5),
    /// and their message's text (T2b1): the game asks it before their battle; an empty one makes
    /// a pile's guards fight at once and an artifact's question the game's own.
    #[serde(default)]
    pub guards: Option<GuardsSpec>,
}

/// A wandering monster's body (E24 M4 T2b1; the writer's `Monster`): no message or treasure,
/// it grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonsterSpec {
    /// Its count (1 to 4 095).
    pub count: u16,
    /// Its disposition: 0 compliant, 1 friendly, 2 aggressive, 3 hostile, 4 savage.
    pub disposition: u8,
    /// It never flees.
    #[serde(default)]
    pub never_flees: bool,
}

/// An object's guards and their message (E24 M4 T2b1; the writer's `Guards` and `Message`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardsSpec {
    /// The stacks, [creature, count] by slot: one to seven.
    pub army: Vec<[u16; 2]>,
    /// The message's text (our own words, Latin-1): empty none.
    #[serde(default)]
    pub text: String,
}

impl GuardsSpec {
    /// The army by slot, checked: one to seven stacks, none empty.
    fn army(&self) -> Result<[Option<(u16, u16)>; ARMY_SLOTS], String> {
        if self.army.is_empty() || self.army.len() > ARMY_SLOTS {
            return Err(format!(
                "guards of {} stacks: one to seven",
                self.army.len()
            ));
        }
        let mut army = [None; ARMY_SLOTS];
        for (slot, &[creature, count]) in army.iter_mut().zip(&self.army) {
            if count == 0 {
                return Err(format!("guards of creature {creature}: a count of 0"));
            }
            *slot = Some((creature, count));
        }
        Ok(army)
    }
}

impl ObjectSpec {
    /// Whether it has a body the writer writes (E24 M4 T2a1): then it is one of the map's placed
    /// objects, not the extras' (which take a template alone).
    fn bodied(&self) -> bool {
        self.amount.is_some()
            || self.spell.is_some()
            || self.skills.is_some()
            || self.rewards.is_some()
            || self.monster.is_some()
            || self.guards.is_some()
    }

    /// Its body as the writer takes it (E24 M4 T2a1), on its template: a pile's amount, a
    /// Shrine's spell, a Witch Hut's skills or an event's rewards, each on its own class.
    fn placed(&self, template: &ObjectTemplate) -> Result<MapObject, String> {
        let tile = (self.at[0], self.at[1]);
        let class = template.class;
        let reward = |body| {
            Ok(MapObject::Reward(RewardObject {
                tile,
                template: template.clone(),
                body,
            }))
        };
        let guards = self.guards.as_ref().map(GuardsSpec::army).transpose()?;
        let text = || {
            self.guards
                .as_ref()
                .map(|g| g.text.clone())
                .unwrap_or_default()
        };
        match (
            &self.amount,
            &self.spell,
            &self.skills,
            &self.rewards,
            &self.monster,
        ) {
            (amount, None, None, None, None) if class == PILE_CLASS => Ok(MapObject::Pile(Pile {
                tile,
                template: template.clone(),
                amount: amount.unwrap_or(0),
                guards: guards.map(|army| Guards {
                    army,
                    question: text(),
                }),
            })),
            (None, None, None, None, None) if class == ARTIFACT_OBJECT && guards.is_some() => {
                reward(RewardBody::Artifact(Some(Message {
                    text: text(),
                    guards,
                })))
            }
            (None, None, None, None, Some(m)) if class == MONSTER_CLASS && guards.is_none() => {
                if m.disposition > 4 || !(1..=0xFFF).contains(&m.count) {
                    return Err(format!(
                        "{} at {:?}: a monster of {} creatures, disposition {}",
                        self.def, self.at, m.count, m.disposition
                    ));
                }
                Ok(MapObject::Monster(Monster {
                    tile,
                    template: template.clone(),
                    count: m.count,
                    disposition: m.disposition,
                    never_flees: m.never_flees,
                    fixed_count: false,
                    treasure: None,
                }))
            }
            (None, Some(spell), None, None, None)
                if SHRINE_OBJECTS.contains(&class) && guards.is_none() =>
            {
                reward(RewardBody::Shrine(Some(*spell)))
            }
            (None, None, Some(skills), None, None)
                if class == WITCH_HUT_OBJECT && guards.is_none() =>
            {
                let mut mask = [0u8; 4];
                for &s in skills {
                    let byte = mask
                        .get_mut(usize::from(s / 8))
                        .filter(|_| s < 28)
                        .ok_or_else(|| {
                            format!("{} at {:?}: skill {s} past 27", self.def, self.at)
                        })?;
                    *byte |= 1 << (s % 8);
                }
                reward(RewardBody::WitchHut(Some(mask)))
            }
            (None, None, None, Some(r), None) if class == EVENT_OBJECT && guards.is_none() => {
                reward(RewardBody::Event(Event {
                    message: None,
                    rewards: r.writer(),
                    players: 0xFF,
                    computer: false,
                    removed_after_visit: true,
                }))
            }
            (None, None, None, Some(r), None) if class == PANDORA_OBJECT && guards.is_none() => {
                reward(RewardBody::Pandora {
                    message: None,
                    rewards: r.writer(),
                })
            }
            _ => Err(format!(
                "{} at {:?}: a body its class {class} does not take, or more than one",
                self.def, self.at
            )),
        }
    }
}

/// An event's rewards (E24 M4 T2a1; the writer's `Rewards`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardsSpec {
    /// Experience.
    #[serde(default)]
    pub experience: u32,
    /// Spell points.
    #[serde(default)]
    pub mana: i32,
    /// Morale (−3 to 3).
    #[serde(default)]
    pub morale: i8,
    /// Luck (−3 to 3).
    #[serde(default)]
    pub luck: i8,
    /// Resources, in resource order.
    #[serde(default)]
    pub resources: [i32; 7],
    /// Attack, defence, spell power, knowledge.
    #[serde(default)]
    pub primary: [u8; 4],
    /// Secondary skills, [skill, level] each.
    #[serde(default)]
    pub skills: Vec<[u8; 2]>,
    /// Artifacts.
    #[serde(default)]
    pub artifacts: Vec<u16>,
    /// Spells.
    #[serde(default)]
    pub spells: Vec<u8>,
    /// Creatures, [creature, count] each.
    #[serde(default)]
    pub creatures: Vec<[u16; 2]>,
}

impl RewardsSpec {
    /// The writer's rewards.
    fn writer(&self) -> Rewards {
        Rewards {
            experience: self.experience,
            mana: self.mana,
            morale: self.morale,
            luck: self.luck,
            resources: self.resources,
            primary: self.primary,
            skills: self.skills.iter().map(|&[s, l]| (s, l)).collect(),
            artifacts: self.artifacts.clone(),
            spells: self.spells.clone(),
            creatures: self
                .creatures
                .iter()
                .map(|&[creature, count]| Stack { creature, count })
                .collect(),
        }
    }
}

/// A town (E24 M1 T5b1): its faction's template (`Objects.txt`'s first of class 98 and the
/// faction), its owner and buildings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownSpec {
    /// Its anchor; its entrance is two columns left of it.
    pub at: [u8; 2],
    /// Its faction (0 Castle … 8 Conflux).
    pub faction: u32,
    /// Its owner's colour (a colour with a hero), none for a neutral town.
    #[serde(default)]
    pub owner: Option<u8>,
    /// Its buildings, which give its picture (design §2.9).
    pub buildings: TownBuilt,
    /// A building built in it today (+0x2), written by the harness before the still (E24 M3 T4a;
    /// design §4.5: the town list's picture): Red's towns only.
    #[serde(default)]
    pub built: bool,
    /// A Resource Silo among its buildings (E24 M3 T5b1, design §4.8: the status window's silo;
    /// the map's building 9): with chosen buildings only (a Citadel, Castle or Capitol).
    #[serde(default)]
    pub silo: bool,
    /// Its garrison's slots, [creature, count] each, written by the harness into its record
    /// before the selection (E24 M3 T5b1: the status window's army): Red's towns only, at most
    /// seven.
    #[serde(default)]
    pub garrison: Vec<[i32; 2]>,
    /// A hero of Red's (on its tile, after its walk) the harness makes its garrison hero in its
    /// record (+0xC) before the selection (E24 M3 T5b1: only the status window reads it, its mark
    /// and that hero's army): Red's towns only.
    #[serde(default)]
    pub garrison_hero: Option<[u8; 2]>,
    /// Its own name, written by the writer into the map (E24 M4 T3b1, design §5.10: the status
    /// window names it so); none the new game's draw among its faction's names.
    #[serde(default)]
    pub name: Option<String>,
}

impl TownSpec {
    /// The writer's buildings: its set's, with the Resource Silo (the map's building 9) when it
    /// has one (E24 M3 T5b1; the checks refuse a silo with a standard set).
    fn writer_buildings(&self) -> TownBuildings {
        match self.buildings.writer() {
            TownBuildings::Custom {
                mut built,
                forbidden,
            } if self.silo => {
                built[1] |= 1 << 1;
                TownBuildings::Custom { built, forbidden }
            }
            other => other,
        }
    }
}

/// A starting bonus the harness chooses (E24 M4 T3b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bonus {
    /// Gold, for every player.
    Gold,
}

/// A town's buildings as a situation names them: the editor's standard set without or with a
/// Fort, or chosen ones up to a Citadel, a Castle or a Capitol (with the Town and City Halls).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TownBuilt {
    /// The standard set without a Fort.
    Village,
    /// The standard set with a Fort.
    Fort,
    /// A Fort and a Citadel.
    Citadel,
    /// The Town Hall and a Fort (E24 M3 T5b2: the kingdom's window counts the towns by their
    /// halls).
    TownHall,
    /// The Town and City Halls and a Fort (E24 M3 T5b2).
    CityHall,
    /// A Fort, a Citadel and a Castle.
    Castle,
    /// The Town, City Hall and Capitol, a Fort, a Citadel and a Castle.
    Capitol,
}

impl TownBuilt {
    /// The writer's buildings (the map's bits: 0 Town Hall, 1 City Hall, 2 Capitol, 3 Fort,
    /// 4 Citadel, 5 Castle; `docs/formats/h3m.md`).
    #[must_use]
    pub fn writer(self) -> TownBuildings {
        let custom = |byte: u8| TownBuildings::Custom {
            built: [byte, 0, 0, 0, 0, 0],
            forbidden: [0; 6],
        };
        match self {
            Self::Village => TownBuildings::Standard { fort: false },
            Self::Fort => TownBuildings::Standard { fort: true },
            Self::Citadel => custom(0b1_1000),
            Self::TownHall => custom(0b1001),
            Self::CityHall => custom(0b1011),
            Self::Castle => custom(0b11_1000),
            Self::Capitol => custom(0b11_1111),
        }
    }
}

/// The towns' class.
const TOWN_CLASS: u32 = 98;

/// A resource pile's class (E24 M4 T2a1; the writer's `Pile`).
const PILE_CLASS: u32 = 79;

/// A hero (E24 M1 T5a).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSpec {
    /// The tile it stands on (the writer puts its anchor one column right).
    pub at: [u8; 2],
    /// Its owner's colour (0 Red … 7 Pink).
    #[serde(default)]
    pub owner: u8,
    /// Its class (0–17), which gives its map picture: its hero type is the class's first not
    /// taken by an earlier hero ([`hero_type`]).
    #[serde(default)]
    pub class: u8,
    /// The tiles the harness walks it to in turn before the still, by play (Red's heroes only:
    /// the human player's; a hero boards a boat by walking onto it, design §2.8).
    #[serde(default)]
    pub walk: Vec<[u8; 2]>,
    /// Where the harness turns it before the still, after its walk (0–7, the hero record's +0x47;
    /// design §2.8): a new game faces every hero right (2).
    #[serde(default)]
    pub facing: Option<u8>,
    /// The tile its path leads to, shown by the harness after the view is set: the hero selected
    /// and the tile clicked once (E24 M1 T5b2; design §2.10, passes 6 and 8). Red's heroes only,
    /// one at most (the selected hero's path is the one shown), but in a situation that has each
    /// hero chosen (`choose`, E24 M3 T6b1: the last shown at the still).
    #[serde(default)]
    pub path: Option<[u8; 2]>,
    /// Its movement points left (+0x4D), written by the harness after its walk (E24 M3 T4a;
    /// design §4.5: the hero list's movement bar): Red's heroes only.
    #[serde(default)]
    pub points: Option<i32>,
    /// Its mana (+0x18), likewise (the mana bar).
    #[serde(default)]
    pub mana: Option<i32>,
    /// Its attack, defense, power and knowledge as stored (i8 +0x476), likewise (E24 M3 T5b1,
    /// design §4.8: the status window).
    #[serde(default)]
    pub skills: Option<[i8; 4]>,
    /// Its morale until its next battle (i8 +0x11A), likewise: the status window shows its
    /// morale with it.
    #[serde(default)]
    pub morale: Option<i8>,
    /// Its luck until its next battle (i8 +0x11B), likewise.
    #[serde(default)]
    pub luck: Option<i8>,
    /// Its army's slots, [creature, count] each (+0x91, +0xAD), likewise; at most seven.
    #[serde(default)]
    pub army: Vec<[i32; 2]>,
    /// Its experience as the map gives it (E24 M4 T2a1, design §5.3: the writer's; the game
    /// gives the levels it reaches at the start, choosing for the human), none the type's.
    #[serde(default)]
    pub experience: Option<u32>,
    /// Its Diplomacy's level (0 to 3; hero +0xCD, the level alone: no skill slot), written by
    /// the harness after its walk (E24 M4 T2b1, design §5.4: a monster's offer to join for
    /// gold); none the type's. It redraws nothing, so it needs no selection.
    #[serde(default)]
    pub diplomacy: Option<u8>,
    /// Its secondary skills, [skill, level 1 to 3] in the list's order (at most eight, each
    /// skill once; hero +0xC9, +0xE5, +0x101), written by the harness after its walk in place of
    /// the type's (E24 M4 T2c, design §5.5: a level-up's offers, one or none with eight skills
    /// known); empty the type's. The status window at 0 was built before it is written and is
    /// built again at the walk, so the skills written and the type's cleared must not change the
    /// morale or luck shown (Leadership, Luck): TRAPS. It needs no selection.
    #[serde(default)]
    pub secondary: Vec<[u8; 2]>,
    /// The spells in its spell book (0 to 69, E24 M4 T2d3, design §5.8: a cast on the map),
    /// written by the writer with the book worn in slot 17 (its artifacts then these alone: the
    /// type's war machines left out); empty the type's spells and artifacts.
    #[serde(default)]
    pub spells: Vec<u8>,
    /// Its own name, written by the writer into the map (E24 M4 T3b1, design §5.10: the status
    /// window names it so); none the type's.
    #[serde(default)]
    pub name: Option<String>,
    /// Its army as the map gives it, [creature, count] by slot (E24 M4 T3b1: the new game then
    /// draws none, so the engine's start and the original's hold the same; `army` is the
    /// harness's write after the load); empty the type's drawn.
    #[serde(default)]
    pub map_army: Vec<[u16; 2]>,
}

/// An army as the writer takes it: [creature, count] by slot.
type MapArmy = [Option<(u16, u16)>; ARMY_SLOTS];

impl HeroSpec {
    /// Its army in the map (E24 M4 T3b1): none for an empty `map_army`, else its stacks by slot,
    /// at most seven, each of at least one creature (the writer checks the creature).
    fn file_army(&self) -> Result<Option<MapArmy>, String> {
        if self.map_army.is_empty() {
            return Ok(None);
        }
        if self.map_army.len() > ARMY_SLOTS || self.map_army.iter().any(|&[_, n]| n == 0) {
            return Err(format!(
                "hero at {:?}: its map army has more than seven stacks or an empty one",
                self.at
            ));
        }
        let mut army = [None; ARMY_SLOTS];
        for (slot, &[creature, count]) in army.iter_mut().zip(&self.map_army) {
            *slot = Some((creature, count));
        }
        Ok(Some(army))
    }

    /// Its worn spell book alone when it has spells (E24 M4 T2d3): artifact 0 in slot 17.
    fn book(&self) -> Option<HeroArtifacts> {
        if self.spells.is_empty() {
            return None;
        }
        let mut worn = [None; ARTIFACT_SLOTS];
        worn[SPELL_BOOK_SLOT] = Some(SPELL_BOOK);
        Some(HeroArtifacts {
            worn,
            backpack: Vec::new(),
        })
    }

    /// Its spells as the writer's bits (E24 M4 T2d3), each spell once and below 70.
    fn spell_bits(&self) -> Result<Option<[u8; 9]>, String> {
        if self.spells.is_empty() {
            return Ok(None);
        }
        let mut bits = [0u8; 9];
        for &spell in &self.spells {
            let (byte, bit) = (usize::from(spell / 8), spell % 8);
            if spell >= SPELLS || bits[byte] & (1 << bit) != 0 {
                return Err(format!(
                    "hero at {:?}: the spell {spell} past 69 or given twice",
                    self.at
                ));
            }
            bits[byte] |= 1 << bit;
        }
        Ok(Some(bits))
    }

    /// Whether the harness writes anything into its record (E24 M3 T4a, T5b1).
    fn written(&self) -> bool {
        self.points.is_some()
            || self.mana.is_some()
            || self.skills.is_some()
            || self.morale.is_some()
            || self.luck.is_some()
            || !self.army.is_empty()
    }
}

/// A boat without a hero (E24 M1 T5a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoatSpec {
    /// The tile it lies on: its template's visitable cell, the anchor one column right.
    pub at: [u8; 2],
    /// Its type (0–2): the template `AVXboat0.def` … `AVXboat2.def` of class 8.
    pub kind: u8,
    /// Where the harness turns it before the still (0–7, the boat record's +0x1B; design §2.8).
    #[serde(default)]
    pub facing: Option<u8>,
    /// The flag's colour the harness writes into its record once a Red hero boarded it (0–7, the
    /// boat record's +0x1C; E24 M1 T5b2): a still's staging, as the facings (boarding gives the
    /// boat its hero's owner, design §2.8), so that the boats' flags of every colour are drawn.
    #[serde(default)]
    pub colour: Option<u8>,
}

/// The secondary skills (SSTRAITS.TXT's 28).
const SECONDARY_SKILLS: usize = 28;
/// The boats' class.
const BOAT_CLASS: u32 = 8;
/// The monsters' class (54), whose object a quest may name (E24 M3 T6a2b).
const MONSTER_CLASS: u32 = 54;

/// The hero type of the `nth` hero (from 0) of `class` on a map: the class's eight types in
/// order (classes 0–15 the types 8 × class onwards, the AB classes 16 and 17 the types 128 and
/// 136 onwards: the packs' hero list), `None` past the eighth or for no class.
#[must_use]
pub fn hero_type(class: u8, nth: usize) -> Option<u8> {
    let first = match class {
        0..=15 => class * 8,
        16 | 17 => 128 + (class - 16) * 8,
        _ => return None,
    };
    u8::try_from(nth).ok().filter(|&n| n < 8).map(|n| first + n)
}

/// The classes whose bodies name a hero (the writer's `AnyObject` takes none of them).
const HERO_CLASSES: [u32; 4] = [34, 62, 70, 214];

/// A tile as given: the writer's [`GivenTile`] on the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileSpec {
    /// The tile.
    pub at: [u8; 2],
    /// Its terrain.
    pub terrain: Ground,
    /// The terrain's picture.
    pub picture: u8,
    /// Its river and picture.
    #[serde(default)]
    pub river: Option<(RiverName, u8)>,
    /// Its road and picture.
    #[serde(default)]
    pub road: Option<(RoadName, u8)>,
    /// The mirror bits (and the coast bit) as stored.
    #[serde(default)]
    pub flags: u8,
}

impl Situation {
    /// Parses a situation and checks its view's level (the surface only).
    ///
    /// # Errors
    /// The parser's message, a view on another level or one more than the map's side off it.
    pub fn parse(text: &str) -> Result<Self, String> {
        // A shipped map loaded whole (`map`, E24 M1 T4b) is the harness's (E24 M1 T6b, R2-12).
        let table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        if table.contains_key("map") {
            return Err("it names a shipped map whole (`map`): the harness loads it".into());
        }
        let situation: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        if situation.view[2] != 0 {
            return Err("a situation's view is on the surface (level 0)".into());
        }
        // The view's corner near the map (M2 T6a, the reviewer's R1-8: one far off overflowed the
        // tile under a click); the original's own range, -9 to the side less 10, is the harness's
        // check (`render --live`).
        let side = i32::from(situation.size);
        if situation.view[..2]
            .iter()
            .any(|v| !(-side..=side).contains(v))
        {
            return Err(format!(
                "the view's corner {:?} lies more than the map's side ({side}) off the map",
                &situation.view[..2]
            ));
        }
        Ok(situation)
    }

    /// The shipped maps the copies read.
    #[must_use]
    pub fn sources(&self) -> BTreeSet<String> {
        self.copy.iter().map(|c| c.map.clone()).collect()
    }

    /// Whether objects or boats are given by their DEF (then [`Situation::build`] needs the
    /// game's templates).
    #[must_use]
    pub fn needs_templates(&self) -> bool {
        !self.objects.is_empty() || !self.boats.is_empty() || !self.towns.is_empty()
    }

    /// The map, its given tiles and its objects, the copies read from `sources` (the shipped maps
    /// by file name; [`Situation::sources`]), the given objects' and boats' templates from
    /// `templates` (the game's `Objects.txt`, in its order). Every colour with a hero is a player
    /// that may be human or computer (the scenario list makes Red the human one); the heroes come
    /// first, then the towns, the objects, the boats.
    ///
    /// # Errors
    /// A source missing, a copied rectangle outside its map or the situation's, a code with no
    /// name, a given object's, town's or boat's template not among `templates`; no hero of Red's or of
    /// another colour, an owner past 7, more than eight heroes of a colour or of a class, a
    /// facing or a boat's colour past 7, a walk or a path of a hero not Red's, a second path.
    pub fn build(
        &self,
        sources: &BTreeMap<String, RawMap>,
        templates: &[ObjectTemplate],
    ) -> Result<(Map, Extras), String> {
        let mut given: BTreeMap<(u8, u8), GivenTile> = BTreeMap::new();
        let mut objects = Vec::new();
        for c in &self.copy {
            let raw = sources
                .get(&c.map)
                .ok_or_else(|| format!("{}: not read", c.map))?;
            for tile in copied(c, raw, self.size)? {
                given.insert(tile.tile, tile);
            }
            if c.objects {
                objects.extend(copied_objects(c, raw));
            }
        }
        let first = objects.len();
        // The objects with a body are placed objects (E24 M4 T2a1), after the heroes and towns;
        // the quests count the extras' objects by the situation's order, so none goes with them.
        let mut bodied = Vec::new();
        if self.objects.iter().any(ObjectSpec::bodied)
            && self.objects.iter().any(|o| o.quest.is_some())
        {
            return Err("a situation with both objects' bodies and quests".into());
        }
        for o in &self.objects {
            let template = templates
                .iter()
                .find(|t| {
                    t.def.eq_ignore_ascii_case(&o.def)
                        && o.class.is_none_or(|c| c == t.class)
                        && o.subclass.is_none_or(|s| s == t.subclass)
                })
                .ok_or_else(|| format!("{}: no such template in Objects.txt", o.def))?;
            if o.bodied() {
                bodied.push(o.placed(template)?);
                continue;
            }
            objects.push(AnyObject {
                level: 0,
                tile: (o.at[0], o.at[1]),
                template: template.clone(),
                owner: o.owner,
            });
        }
        for t in &self.tiles {
            let tile = GivenTile {
                level: 0,
                tile: (t.at[0], t.at[1]),
                terrain: t.terrain.writer(),
                picture: t.picture,
                river: t.river.map(|(r, p)| (r.writer(), p)),
                road: t.road.map(|(r, p)| (r.writer(), p)),
                flags: t.flags,
            };
            given.insert(tile.tile, tile);
        }
        objects.extend(self.boats(templates)?);
        let (players, mut heroes) = self.heroes()?;
        self.selection()?;
        for t in &self.towns {
            let template = templates
                .iter()
                .find(|o| (o.class, o.subclass) == (TOWN_CLASS, t.faction))
                .ok_or_else(|| {
                    format!("town at {:?}: no template of faction {}", t.at, t.faction)
                })?;
            let town = Town {
                tile: (t.at[0], t.at[1]),
                template: template.clone(),
                owner: t.owner,
                garrison: None,
                tight_formation: false,
                buildings: t.writer_buildings(),
            };
            // A town named by the map (E24 M4 T3b1) is the writer's town of any kind.
            heroes.push(match &t.name {
                Some(name) => MapObject::AnyTown(AnyTown {
                    name: Some(name.clone()),
                    ..AnyTown::from(&town)
                }),
                None => MapObject::Town(town),
            });
        }
        heroes.extend(bodied);
        let map = Map {
            name: "situation".into(),
            description: String::new(),
            size: self.size,
            terrain: self.terrain.writer(),
            patches: self.patches.iter().map(PatchSpec::writer).collect(),
            roads: self
                .roads
                .iter()
                .map(|r| Road {
                    kind: r.kind.writer(),
                    row: r.row,
                    from: r.from,
                    to: r.to,
                })
                .collect(),
            players,
            objects: heroes,
            underground: self.underground.as_ref().map(|u| Underground {
                terrain: u.terrain.writer(),
                patches: u.patches.iter().map(PatchSpec::writer).collect(),
                objects: Vec::new(),
            }),
            level_limit: 0,
            banned_skills: vec![],
            banned_artifacts: Vec::new(),
            events: Vec::new(),
            teams: None,
        };
        let quests = self.quests(&map.objects, &objects, first)?;
        let extras = Extras {
            tiles: given.into_values().collect(),
            objects,
            quests,
        };
        Ok((map, extras))
    }

    /// The players and the heroes: every colour with a hero a player (human or computer), each
    /// hero of the next free type of its class.
    fn heroes(&self) -> Result<([Option<Player>; 8], Vec<MapObject>), String> {
        let mut players: [Option<Player>; 8] = Default::default();
        let mut of_owner = [0usize; 8];
        let mut of_class = [0usize; 18];
        let mut paths = None;
        let mut heroes = Vec::new();
        for h in &self.heroes {
            let facing = h.facing.unwrap_or(0);
            let owner = usize::from(h.owner);
            let (Some(count), true) = (of_owner.get_mut(owner), facing < 8) else {
                return Err(format!(
                    "hero at {:?}: owner {} or facing {facing} past 7",
                    h.at, h.owner
                ));
            };
            // Everything the harness writes into a record is Red's alone (E24 M3 T7b2, F5-6: the
            // status window's facts too, as `HeroSpec`'s docs give them).
            if h.owner != 0 && (!h.walk.is_empty() || h.path.is_some() || h.written()) {
                return Err(format!(
                    "hero at {:?}: only Red's heroes walk, show a path or have anything written",
                    h.at
                ));
            }
            // With `choose` each hero's path shows when it is chosen (E24 M3 T6b1), the staging's
            // selection's at the still.
            if h.path.is_some() && paths.replace(h.at).is_some() && !self.choose {
                return Err(format!("hero at {:?}: a second path shown", h.at));
            }
            *count += 1;
            if *count > 8 {
                return Err(format!("hero at {:?}: a ninth of colour {owner}", h.at));
            }
            let nth = of_class.get(usize::from(h.class)).copied().unwrap_or(0);
            let hero_type = hero_type(h.class, nth).ok_or_else(|| {
                if h.class < 18 {
                    format!("hero at {:?}: a ninth of class {}", h.at, h.class)
                } else {
                    format!(
                        "hero at {:?}: class {} is no hero class (0-17)",
                        h.at, h.class
                    )
                }
            })?;
            of_class[usize::from(h.class)] += 1;
            players[owner] = Some(Player {
                human: true,
                computer: true,
                towns: 0x1ff,
            });
            heroes.push(MapObject::Hero(Hero {
                tile: (h.at[0], h.at[1]),
                owner: h.owner,
                hero_type,
                class: h.class,
                name: h.name.clone(),
                experience: h.experience,
                skills: None,
                army: h.file_army()?,
                tight_formation: false,
                artifacts: h.book(),
                spells: h.spell_bits()?,
                primary: None,
            }));
        }
        if of_owner[0] == 0 || of_owner[1..].iter().all(|&n| n == 0) {
            return Err("a situation needs a hero of Red's and of another colour".into());
        }
        Ok((players, heroes))
    }

    /// The given objects' quests as the writer takes them (E24 M3 T6a2b), by their index among
    /// the extras' `objects` (the given ones from `first`): a hero's mission names the identifier
    /// of the hero standing on its tile (its index among the map's `placed` objects, plus one),
    /// a monster's that of the monster object anchored there (after them).
    fn quests(
        &self,
        placed: &[MapObject],
        objects: &[AnyObject],
        first: usize,
    ) -> Result<BTreeMap<usize, Mission>, String> {
        let identifier = |i: usize| u32::try_from(i + 1).unwrap_or(u32::MAX);
        let mut quests = BTreeMap::new();
        for (k, o) in self.objects.iter().enumerate() {
            let Some(quest) = &o.quest else {
                continue;
            };
            let mission = match quest {
                QuestSpec::Level(n) => Mission::Level(*n),
                QuestSpec::Skills(s) => Mission::PrimarySkills(*s),
                QuestSpec::Hero(at) => {
                    let j = placed
                        .iter()
                        .position(|p| matches!(p, MapObject::Hero(h) if h.tile == (at[0], at[1])))
                        .ok_or_else(|| format!("the quest at {:?}: no hero on {at:?}", o.at))?;
                    Mission::KillHero(identifier(j))
                }
                QuestSpec::Monster(at) => {
                    let i = (0..self.objects.len())
                        .find(|&i| {
                            self.objects[i].at == *at
                                && objects
                                    .get(first + i)
                                    .is_some_and(|m| m.template.class == MONSTER_CLASS)
                        })
                        .ok_or_else(|| {
                            format!("the quest at {:?}: no monster anchored on {at:?}", o.at)
                        })?;
                    Mission::KillCreature(identifier(placed.len() + first + i))
                }
                QuestSpec::Artifacts(a) => Mission::Artifacts(a.clone()),
                QuestSpec::Creatures(c) => Mission::Army(
                    c.iter()
                        .map(|&[creature, count]| Stack { creature, count })
                        .collect(),
                ),
                QuestSpec::Resources(r) => Mission::Resources(*r),
                QuestSpec::BeHero(h) => Mission::Hero(*h),
                QuestSpec::Player(p) => Mission::Player(*p),
            };
            quests.insert(first + k, mission);
        }
        Ok(quests)
    }

    /// The harness's writes into the lists and its selection (E24 M3 T4a): a town's day written
    /// only for Red's; the selection Red's hero on its tile (after its walk) or Red's town on its
    /// anchor; a selection whenever something is written, so that the lists show it; a hero's
    /// with the resources or the date (T5a), a day of the game's; since T5b1 the status window's
    /// writes into Red's heroes and towns, a garrison hero of Red's, seven slots at most, a silo
    /// with chosen buildings.
    fn selection(&self) -> Result<(), String> {
        if let Some(t) = self.towns.iter().find(|t| t.built && t.owner != Some(0)) {
            return Err(format!(
                "town at {:?}: only Red's towns have their day written",
                t.at
            ));
        }
        if let Some([month, week, day]) = self.date
            && (month == 0 || !(1..=4).contains(&week) || !(1..=7).contains(&day))
        {
            return Err(format!("the date {:?} is no day of the game's", self.date));
        }
        let bar = self.resources.is_some() || self.date.is_some();
        let hero_selected = matches!(self.select, Some(SelectSpec::Hero(_)));
        if bar && !hero_selected {
            return Err("the resources or the date without a hero's selection".into());
        }
        if self.reselect.is_some() && !hero_selected {
            return Err("`reselect` without a hero's selection".into());
        }
        // The status window's kind asked (E24 M3 T5b2): the date's or the kingdom's; the teams.
        if let Some(kind) = self.status
            && !(1..=2).contains(&kind)
        {
            return Err(format!("the status window's kind {kind}: not 1 or 2"));
        }
        if let Some(teams) = self.teams
            && teams.iter().any(|&t| t > 7)
        {
            return Err(format!("the teams {teams:?}: one past 7"));
        }
        // The visit marks (E24 M3 T6a2a): each of an object the situation gives, once.
        for (i, v) in self.visits.iter().enumerate() {
            if !self.objects.iter().any(|o| o.at == v.at) {
                return Err(format!(
                    "the visits at {:?}: no object anchored there",
                    v.at
                ));
            }
            if self.visits[..i].iter().any(|w| w.at == v.at) {
                return Err(format!("the visits at {:?}: given twice", v.at));
            }
            // The words the load draws (E24 M4 T2b1): each in its bits' range.
            let fits = v.aggression.is_none_or(|a| (-16..=15).contains(&a))
                && v.gold
                    .is_none_or(|g| g % 500 == 0 && (500..=7_500).contains(&g))
                && v.price.is_none_or(|p| p <= 2);
            if !fits {
                return Err(format!(
                    "the visits at {:?}: an aggression, a chest's gold or a Tree's price \
                     its word cannot hold",
                    v.at
                ));
            }
        }
        if let Some(h) = self
            .heroes
            .iter()
            .find(|h| h.diplomacy.is_some_and(|d| d > 3))
        {
            return Err(format!("the hero at {:?}: Diplomacy past Expert", h.at));
        }
        // A level-up's offers (E24 M4 T2c): eight skills at most, each a skill once at a level.
        if let Some(h) = self.heroes.iter().find(|h| {
            h.secondary.len() > 8
                || h.secondary.iter().enumerate().any(|(i, &[skill, level])| {
                    usize::from(skill) >= SECONDARY_SKILLS
                        || !(1..=3).contains(&level)
                        || h.secondary[..i].iter().any(|k| k[0] == skill)
                })
        }) {
            return Err(format!(
                "the hero at {:?}: secondary skills past eight, unknown, twice or at no level",
                h.at
            ));
        }
        // The status window's (E24 M3 T5b1): Red's towns' garrisons and garrison heroes, of
        // Red's; seven slots at most; a silo with chosen buildings.
        for t in &self.towns {
            let garrisoned = !t.garrison.is_empty() || t.garrison_hero.is_some();
            if garrisoned && t.owner != Some(0) {
                return Err(format!(
                    "town at {:?}: only Red's towns have their garrison written",
                    t.at
                ));
            }
            if let Some(at) = t.garrison_hero
                && !self
                    .heroes
                    .iter()
                    .any(|h| h.owner == 0 && h.walk.last().copied().unwrap_or(h.at) == at)
            {
                return Err(format!(
                    "town at {:?}: its garrison hero is none of Red's",
                    t.at
                ));
            }
            if t.silo && matches!(t.buildings, TownBuilt::Village | TownBuilt::Fort) {
                return Err(format!(
                    "town at {:?}: a silo with the standard buildings",
                    t.at
                ));
            }
        }
        let slots = self
            .heroes
            .iter()
            .map(|h| h.army.len())
            .chain(self.towns.iter().map(|t| t.garrison.len()));
        if slots.into_iter().any(|n| n > 7) {
            return Err("an army of more than seven slots".into());
        }
        let written = self.heroes.iter().any(HeroSpec::written)
            || self
                .towns
                .iter()
                .any(|t| t.built || !t.garrison.is_empty() || t.garrison_hero.is_some());
        let found = match self.select {
            None if written => return Err("a write into the lists without a selection".into()),
            None => true,
            Some(SelectSpec::Hero(at)) => self
                .heroes
                .iter()
                .any(|h| h.owner == 0 && h.walk.last().copied().unwrap_or(h.at) == at),
            Some(SelectSpec::Town(at)) => {
                self.towns.iter().any(|t| t.owner == Some(0) && t.at == at)
            }
        };
        if found {
            Ok(())
        } else {
            Err(format!("the selection {:?} is none of Red's", self.select))
        }
    }

    /// The boats as objects: each of its type's template, anchored one column right of its tile.
    fn boats(&self, templates: &[ObjectTemplate]) -> Result<Vec<AnyObject>, String> {
        self.boats
            .iter()
            .map(|b| {
                let template = templates
                    .iter()
                    .find(|t| (t.class, t.subclass) == (BOAT_CLASS, u32::from(b.kind)))
                    .filter(|_| b.facing.unwrap_or(0) < 8 && b.colour.unwrap_or(0) < 8)
                    .ok_or_else(|| {
                        format!(
                            "boat at {:?}: no template of type {} or a facing or colour past 7",
                            b.at, b.kind
                        )
                    })?;
                Ok(AnyObject {
                    level: 0,
                    tile: (b.at[0].saturating_add(1), b.at[1]),
                    template: template.clone(),
                    owner: None,
                })
            })
            .collect()
    }
}

/// The tiles of a copy: each of the rectangle's tiles of `raw` as given, moved to `at`.
fn copied(c: &CopySpec, raw: &RawMap, side: u8) -> Result<Vec<GivenTile>, String> {
    let (src, levels) = (raw.terrain.size, raw.terrain.levels);
    let [x0, y0] = c.from.map(usize::from);
    let [x1, y1] = c.to.map(usize::from);
    let inside = x0 <= x1 && y0 <= y1 && x1 < src && y1 < src && usize::from(c.level) < levels;
    let fits = |at: u8, from: usize, to: usize| usize::from(at) + (to - from) < usize::from(side);
    if !inside || !fits(c.at[0], x0, x1) || !fits(c.at[1], y0, y1) {
        return Err(format!(
            "{}: the rectangle {:?}-{:?} does not fit (the map's side {src}, the situation's {side})",
            c.map, c.from, c.to
        ));
    }
    let mut out = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let name = |what: &str, code: u8| format!("{}: ({x}, {y}): {what} code {code}", c.map);
            let t = raw
                .terrain
                .tiles
                .get((usize::from(c.level) * src + y) * src + x)
                .ok_or_else(|| name("no tile", 0))?;
            let terrain = terrain(t.terrain).ok_or_else(|| name("terrain", t.terrain))?;
            let river = match t.river {
                0 => None,
                code => Some((river(code).ok_or_else(|| name("river", code))?, t.river_dir)),
            };
            let road = match t.road {
                0 => None,
                code => Some((road(code).ok_or_else(|| name("road", code))?, t.road_dir)),
            };
            let at = |base: u8, v: usize, from: usize| {
                u8::try_from(usize::from(base) + v - from).unwrap_or(u8::MAX)
            };
            out.push(GivenTile {
                level: 0,
                tile: (at(c.at[0], x, x0), at(c.at[1], y, y0)),
                terrain,
                picture: t.view,
                river,
                road,
                flags: t.flags,
            });
        }
    }
    Ok(out)
}

/// The objects of a copy (it fitted: [`copied`]): those anchored in the rectangle on its level,
/// but the hero classes, moved to `at`, with their templates as the shipped map gives them.
fn copied_objects(c: &CopySpec, raw: &RawMap) -> Vec<AnyObject> {
    let [x0, y0] = c.from;
    let [x1, y1] = c.to;
    raw.objects
        .iter()
        .filter(|o| {
            o.pos.z == c.level && (x0..=x1).contains(&o.pos.x) && (y0..=y1).contains(&o.pos.y)
        })
        .filter_map(|o| {
            let index = usize::try_from(o.template).unwrap_or(usize::MAX);
            let template = raw.templates.get(index).map(|t| ObjectTemplate {
                def: t.def.0.iter().map(|&b| char::from(b)).collect(),
                passable: t.passable,
                visitable: t.visitable,
                terrains: t.terrains,
                menu: t.editor_terrains,
                class: t.class,
                subclass: t.subclass,
                group: t.group,
                priority: t.priority,
            });
            template
                .filter(|t| !HERO_CLASSES.contains(&t.class))
                .map(|template| AnyObject {
                    level: 0,
                    tile: (c.at[0] + (o.pos.x - x0), c.at[1] + (o.pos.y - y0)),
                    template,
                    owner: None,
                })
        })
        .collect()
}

impl Ground {
    fn writer(self) -> Terrain {
        match self {
            Self::Dirt => Terrain::Dirt,
            Self::Sand => Terrain::Sand,
            Self::Grass => Terrain::Grass,
            Self::Snow => Terrain::Snow,
            Self::Swamp => Terrain::Swamp,
            Self::Rough => Terrain::Rough,
            Self::Subterranean => Terrain::Subterranean,
            Self::Lava => Terrain::Lava,
            Self::Water => Terrain::Water,
            Self::Rock => Terrain::Rock,
        }
    }
}

impl RiverName {
    fn writer(self) -> RiverKind {
        match self {
            Self::Clear => RiverKind::Clear,
            Self::Icy => RiverKind::Icy,
            Self::Muddy => RiverKind::Muddy,
            Self::Lava => RiverKind::Lava,
        }
    }
}

impl RoadName {
    fn writer(self) -> RoadKind {
        match self {
            Self::Dirt => RoadKind::Dirt,
            Self::Gravel => RoadKind::Gravel,
            Self::Cobblestone => RoadKind::Cobblestone,
        }
    }
}

/// The writer's terrain of a `.h3m` code.
fn terrain(code: u8) -> Option<Terrain> {
    [
        Terrain::Dirt,
        Terrain::Sand,
        Terrain::Grass,
        Terrain::Snow,
        Terrain::Swamp,
        Terrain::Rough,
        Terrain::Subterranean,
        Terrain::Lava,
        Terrain::Water,
        Terrain::Rock,
    ]
    .get(usize::from(code))
    .copied()
}

/// The writer's river of a `.h3m` code (1–4).
fn river(code: u8) -> Option<RiverKind> {
    [
        RiverKind::Clear,
        RiverKind::Icy,
        RiverKind::Muddy,
        RiverKind::Lava,
    ]
    .get(usize::from(code).wrapping_sub(1))
    .copied()
}

/// The writer's road of a `.h3m` code (1–3).
fn road(code: u8) -> Option<RoadKind> {
    [RoadKind::Dirt, RoadKind::Gravel, RoadKind::Cobblestone]
        .get(usize::from(code).wrapping_sub(1))
        .copied()
}

#[cfg(test)]
mod tests;
