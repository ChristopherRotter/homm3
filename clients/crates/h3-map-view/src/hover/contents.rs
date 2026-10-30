//! What the map hint shows of an object's owner and contents (E24 M3 T6a2b, *provisional*;
//! `docs/epics/E24-design.md` §4.12): the 20 classes of `map_hint` 0x40B150 whose case names an
//! owner, what an object holds or what the viewer learnt of it, through seven sub-functions.
//!
//! What only the game knows (a mine's guards, a bank's visit and guards, a Shrine's spell, a
//! Tree's price, a Witch Hut's skill, a quest) is the live part's [`Content`] of an object: the
//! original's by its records and the players' fields (`h3mem.Mem.contents`), the engine's from
//! M4. Its four players (the viewing, the shown, the current one and the selected hero's owner)
//! are one in a human's turn of a single-player game, so a content holds what the hint reads
//! already decided for them.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

use super::{CLASSES, Hint, LAST_CREATURE, SIZES};
use crate::status::hero_key;

/// An object's contents as the hint reads them (E24 M3 T6a2b, design §4.12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// A mine (53): the resource it produces (its record's +1, an abandoned mine's drawn at the
    /// new game) and its guards' stacks (+0x4, [creature, count] in slot order).
    Mine {
        /// The resource (0 to 6).
        resource: u8,
        /// The guards.
        guards: Vec<(u32, u32)>,
    },
    /// A creature bank (16, 24, 25, 84, 85): visited by the shown player (its word's bit 5 + the
    /// player); its guards' stacks (none once looted, the word's bit 25, or beaten).
    Bank {
        /// Visited.
        visited: bool,
        /// The guards.
        guards: Vec<(u32, u32)>,
    },
    /// A Shrine (88 to 90): the current player knows its kind; once that player visited it its
    /// spell and whether the selected hero knows it.
    Shrine {
        /// Its kind known.
        known: bool,
        /// Its spell, and the selected hero knows it.
        spell: Option<(u16, bool)>,
    },
    /// A Tree of Knowledge (102): its kind known; once the current player visited it its price
    /// (0 free, 1 gold, 2 gems); the selected hero's mark (none without one).
    Tree {
        /// Its kind known.
        known: bool,
        /// Its price.
        price: Option<u8>,
        /// The selected hero visited it.
        visited: Option<bool>,
    },
    /// A Witch Hut (113) with a skill: its kind known; once the current player visited it its
    /// skill and whether the selected hero has it (a hut without a skill shows its name alone:
    /// neither known nor a skill).
    WitchHut {
        /// Its kind known.
        known: bool,
        /// Its skill, and the selected hero has it.
        skill: Option<(u8, bool)>,
    },
    /// A Seer's Hut (83) or a Quest Guard (215): visited (the record's +4 bit of the viewing
    /// player, a guard's of the shown one), the seer's name (+0x11, SeerHut.txt's), its quest.
    Quest {
        /// Visited.
        visited: bool,
        /// The seer's name.
        seer: u8,
        /// The quest.
        quest: Option<Arc<Quest>>,
    },
}

/// A quest as its hint names it (E24 M3 T6a2b): its texts' variant (+0x38) and its mission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    /// The variant (0 to 2): SeerHut.txt's column after the seer's or the guard's first.
    pub variant: u8,
    /// The mission.
    pub mission: Mission,
}

/// A quest's mission (the quest object's type, 1 to 9: its vtable `quest_vtables` 0x64174C +
/// 0x3C a type) and what its hint's text names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mission {
    /// 1: reach a level (+0x40).
    Level(u32),
    /// 2: the primary skills (+0x40, four signed bytes): those above 0 named.
    Skills([i8; 4]),
    /// 3: defeat a hero (+0x44): its record's index and whether its name is the map's own.
    DefeatHero(u8, bool),
    /// 4: defeat a monster (+0x48): its creature.
    DefeatMonster(u32),
    /// 5: bring artifacts (+0x44's vector).
    Artifacts(Vec<u32>),
    /// 6: bring creatures (+0x54's types and +0x44's counts).
    Creatures(Vec<(u32, u32)>),
    /// 7: bring resources (+0x40, seven): those above 0 named.
    Resources([i32; 7]),
    /// 8: be a hero (+0x40): its index and own name.
    BeHero(u8, bool),
    /// 9: belong to a colour (+0x40).
    BePlayer(u8),
}

impl Mission {
    /// Its type (1 to 9), SeerHut.txt's rows 5 x type (the "rollover" texts).
    #[must_use]
    pub fn kind(&self) -> u8 {
        match self {
            Self::Level(_) => 1,
            Self::Skills(_) => 2,
            Self::DefeatHero(..) => 3,
            Self::DefeatMonster(_) => 4,
            Self::Artifacts(_) => 5,
            Self::Creatures(_) => 6,
            Self::Resources(_) => 7,
            Self::BeHero(..) => 8,
            Self::BePlayer(_) => 9,
        }
    }
}

/// The contents' lists of the live part's `[hover]` as written (left out: none).
#[derive(Debug, Default, Deserialize)]
pub(super) struct Written {
    /// `[[object, resource, [[creature, count], ...]], ...]`.
    #[serde(default)]
    pub mines: Vec<(usize, u8, Vec<(u32, u32)>)>,
    /// `[[object, visited, [[creature, count], ...]], ...]`.
    #[serde(default)]
    pub banks: Vec<(usize, u8, Vec<(u32, u32)>)>,
    /// `[[object, known, spell or -1, learned], ...]`.
    #[serde(default)]
    pub shrines: Vec<(usize, u8, i32, u8)>,
    /// `[[object, known, price or -1, mark -1, 0 or 1], ...]`.
    #[serde(default)]
    pub trees: Vec<(usize, u8, i8, i8)>,
    /// `[[object, known, skill or -1, learned], ...]`.
    #[serde(default)]
    pub witch_huts: Vec<(usize, u8, i32, u8)>,
    /// `[[object, visited, seer, type (0 none), variant, [values], [[a, b], ...]], ...]`: the
    /// values of a level [n], skills [a, d, p, k], a hero [index, own], a monster [creature],
    /// artifacts [ids], resources [seven], a hero to be [index, own], a colour [c]; a creature
    /// quest's stacks the pairs.
    #[serde(default)]
    pub quests: Vec<QuestRow>,
}

/// One quest's row as written.
type QuestRow = (usize, u8, u8, u8, u8, Vec<i64>, Vec<(u32, u32)>);

impl Written {
    /// The contents by object, each object's index below `objects`; none when a row is wrong.
    pub(super) fn contents(self, objects: usize) -> Option<BTreeMap<usize, Content>> {
        let flag = |n: u8| (n < 2).then_some(n == 1);
        let mut out = BTreeMap::new();
        let mut put = |object: usize, content: Content| {
            (object < objects && out.insert(object, content).is_none()).then_some(())
        };
        for (o, resource, guards) in self.mines {
            put(o, Content::Mine { resource, guards })?;
        }
        for (o, visited, guards) in self.banks {
            let visited = flag(visited)?;
            put(o, Content::Bank { visited, guards })?;
        }
        for (o, known, spell, learned) in self.shrines {
            let spell = match spell {
                -1 => None,
                s => Some((u16::try_from(s).ok()?, flag(learned)?)),
            };
            let known = flag(known)?;
            put(o, Content::Shrine { known, spell })?;
        }
        for (o, known, price, mark) in self.trees {
            let price = (price != -1)
                .then(|| u8::try_from(price).ok())
                .map_or(Some(None), |p| p.map(Some))?;
            let visited = match mark {
                -1 => None,
                m => Some(flag(u8::try_from(m).ok()?)?),
            };
            let known = flag(known)?;
            put(
                o,
                Content::Tree {
                    known,
                    price,
                    visited,
                },
            )?;
        }
        for (o, known, skill, learned) in self.witch_huts {
            let skill = match skill {
                -1 => None,
                s => Some((u8::try_from(s).ok()?, flag(learned)?)),
            };
            let known = flag(known)?;
            put(o, Content::WitchHut { known, skill })?;
        }
        for (o, visited, seer, kind, variant, values, pairs) in self.quests {
            let quest = match kind {
                0 => None,
                k => Some(Arc::new(Quest {
                    variant: (variant < 3).then_some(variant)?,
                    mission: mission(k, &values, pairs)?,
                })),
            };
            let visited = flag(visited)?;
            put(
                o,
                Content::Quest {
                    visited,
                    seer,
                    quest,
                },
            )?;
        }
        Some(out)
    }
}

/// A quest's mission of type `kind` from its row's `values` and `pairs`.
fn mission(kind: u8, values: &[i64], pairs: Vec<(u32, u32)>) -> Option<Mission> {
    let one = |values: &[i64]| match values {
        [v] => Some(*v),
        _ => None,
    };
    let byte = |v: i64| u8::try_from(v).ok();
    let hero = |values: &[i64]| match values {
        &[id, own] => Some((byte(id)?, (0..2).contains(&own).then_some(own == 1)?)),
        _ => None,
    };
    Some(match kind {
        1 => Mission::Level(u32::try_from(one(values)?).ok()?),
        2 => Mission::Skills(
            values
                .iter()
                .map(|&v| i8::try_from(v).ok())
                .collect::<Option<Vec<_>>>()?
                .try_into()
                .ok()?,
        ),
        3 => {
            let (id, own) = hero(values)?;
            Mission::DefeatHero(id, own)
        }
        4 => Mission::DefeatMonster(u32::try_from(one(values)?).ok()?),
        5 => Mission::Artifacts(
            values
                .iter()
                .map(|&v| u32::try_from(v).ok())
                .collect::<Option<_>>()?,
        ),
        6 => Mission::Creatures(pairs),
        7 => Mission::Resources(
            values
                .iter()
                .map(|&v| i32::try_from(v).ok())
                .collect::<Option<Vec<_>>>()?
                .try_into()
                .ok()?,
        ),
        8 => {
            let (id, own) = hero(values)?;
            Mission::BeHero(id, own)
        }
        9 => Mission::BePlayer(byte(one(values)?)?),
        _ => return None,
    })
}

/// An army as a hint names it (`army_description_text` 0x40ABE0 without its details): its stacks
/// merged by creature (`army_add_stack` 0x44ACE0), the creature of the only one (none for
/// several: "creatures") and the total count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guards {
    /// The creature of the only merged stack; none for several.
    pub creature: Option<u32>,
    /// The total count.
    pub count: u32,
}

impl Guards {
    /// The guards of `stacks` ([creature, count] in slot order); none without one.
    #[must_use]
    pub fn of(stacks: &[(u32, u32)]) -> Option<Self> {
        let first = stacks.first()?.0;
        let one = stacks.iter().all(|&(c, _)| c == first);
        Some(Self {
            creature: one.then_some(first),
            count: stacks
                .iter()
                .fold(0u32, |sum, &(_, n)| sum.wrapping_add(n)),
        })
    }
}

/// What a creature bank's hint ends with (0x40D3F0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankMark {
    /// Not visited by the shown player.
    NotVisited,
    /// Visited and empty.
    Visited,
    /// Visited and guarded.
    Guarded(Guards),
}

/// What a Shrine, a Tree of Knowledge or a Witch Hut shows once visited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Teach {
    /// A Shrine's spell and whether the selected hero knows it.
    Spell(u16, bool),
    /// A Tree's price.
    Price(u8),
    /// A Witch Hut's skill and whether the selected hero has it.
    Skill(u8, bool),
}

/// The catalogue's keys the contents' hints show texts by (the families `keymaster`,
/// `player-owned`, `dwelling1`, `dwelling4`, `resource`, `mine`, `bank`, `spell`, `tree-price`,
/// `secondary-skill`, `seer-name`, `primary-skill`, `artifact` and `player-colour` besides).
pub const KEYS: [&str; 34] = [
    "map.hint.keymaster",
    "map.hint.owned",
    "map.hint.owner",
    "map.hint.open",
    "map.hint.close",
    "map.hint.guarded",
    "map.hint.creatures",
    "map.hint.learn-spell",
    "map.hint.spell-known",
    "map.hint.learn-skill",
    "map.hint.skill-known",
    "map.hint.seer",
    "map.hint.comma",
    "map.hint.and",
    "map.quest.skill",
    "map.quest.count",
    "map.quest.1",
    "map.quest.2",
    "map.quest.3",
    "map.quest.4",
    "map.quest.5",
    "map.quest.6",
    "map.quest.7",
    "map.quest.8",
    "map.quest.9",
    "map.guards.0",
    "map.guards.1",
    "map.guards.2",
    "map.guards.3",
    "map.guards.4",
    "map.guards.5",
    "map.guards.6",
    "map.guards.7",
    "map.guards.8",
];

/// The colours a family by colour names (eight).
const COLOURS: u32 = 8;
/// The resources a family names (seven, the eighth an abandoned mine's own name).
const RESOURCES: u32 = 8;

/// A family's text of entry `n`, empty past its `count` (the original reads past its tables).
fn entry(texts: &Catalogue, family: &str, n: u32, count: u32) -> String {
    if n < count {
        texts.text(&alloc::format!("{family}.{n}"), &Args::new())
    } else {
        String::new()
    }
}

/// `text` after a space (`map.hint.more`, the executable's " %s"; the sub-functions' separator
/// arguments are a space, 0x660330).
fn more(texts: &Catalogue, text: &str) -> String {
    texts.text("map.hint.more", &Args::new().text("text", text))
}

/// A creature's name: its plural, or its singular for one (`count`); empty past the last named.
fn creature(texts: &Catalogue, id: u32, count: u32) -> String {
    if id <= LAST_CREATURE {
        let args = Args::new().count(i64::from(count));
        texts.text(&alloc::format!("creature.{id}"), &args)
    } else {
        String::new()
    }
}

/// A hero's name: the map's own when it has one, else its type's.
fn hero_name(names: &BTreeMap<u8, String>, texts: &Catalogue, id: u8, own: bool) -> String {
    match names.get(&id) {
        Some(name) if own => name.clone(),
        _ => texts.text(&hero_key(id), &Args::new()),
    }
}

/// The guards' text (`army_description_text` 0x40ABE0 without its details, GENRLTXT 203, the
/// size word's form 2 and the creature's plural or GENRLTXT 44), each part after a space.
fn guards(texts: &Catalogue, g: Guards) -> String {
    let plain = |key: &str| texts.text(key, &Args::new());
    let size = SIZES.iter().take_while(|&&s| g.count >= s).count();
    let name = match g.creature {
        Some(c) => creature(texts, c, 2),
        None => plain("map.hint.creatures"),
    };
    let mut text = plain("map.hint.guarded");
    text.push_str(&more(texts, &plain(&alloc::format!("map.guards.{size}"))));
    text.push_str(&more(texts, &name));
    text
}

/// A list's items joined (0x56C960): ", " between them, " and " (GENRLTXT 142) before the last.
fn joined(texts: &Catalogue, items: &[String]) -> String {
    let mut text = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            let key = if i + 1 == items.len() {
                "map.hint.and"
            } else {
                "map.hint.comma"
            };
            text.push_str(&texts.text(key, &Args::new()));
        }
        text.push_str(item);
    }
    text
}

/// A quest's text in its hint (the quest's vtable +0x1C): SeerHut.txt's row 5 x its type in the
/// column of the seer's (`seer`) or the guard's variant, with what its mission names.
#[must_use]
pub fn quest_text(
    names: &BTreeMap<u8, String>,
    texts: &Catalogue,
    quest: &Quest,
    seer: bool,
) -> String {
    let side = if seer { "seer" } else { "guard" };
    let context = alloc::format!("{side}-{}", quest.variant + 1);
    let key = alloc::format!("map.quest.{}", quest.mission.kind());
    let count = |n: i64, name: &str| {
        let args = Args::new().number("amount", n).text("name", name);
        texts.text("map.quest.count", &args)
    };
    let what = match &quest.mission {
        Mission::Level(n) => {
            let args = Args::new()
                .context(&context)
                .number("level", i64::from(*n));
            return texts.text(&key, &args);
        }
        Mission::Skills(s) => {
            let items: Vec<String> = (0u32..)
                .zip(s)
                .filter(|&(_, &v)| v > 0)
                .map(|(n, &v)| {
                    let args = Args::new()
                        .text("skill", &entry(texts, "primary-skill", n, 4))
                        .number("value", i64::from(v));
                    texts.text("map.quest.skill", &args)
                })
                .collect();
            joined(texts, &items)
        }
        Mission::DefeatHero(id, own) | Mission::BeHero(id, own) => {
            hero_name(names, texts, *id, *own)
        }
        Mission::DefeatMonster(c) => creature(texts, *c, 2),
        Mission::Artifacts(a) => {
            let items: Vec<String> = a
                .iter()
                .map(|&id| texts.text(&alloc::format!("artifact.{id}"), &Args::new()))
                .collect();
            joined(texts, &items)
        }
        Mission::Creatures(c) => {
            let items: Vec<String> = c
                .iter()
                .map(|&(id, n)| count(i64::from(n), &creature(texts, id, n)))
                .collect();
            joined(texts, &items)
        }
        Mission::Resources(r) => {
            let items: Vec<String> = (0u32..)
                .zip(r)
                .filter(|&(_, &v)| v > 0)
                .map(|(n, &v)| count(i64::from(v), &entry(texts, "resource", n, RESOURCES)))
                .collect();
            joined(texts, &items)
        }
        Mission::BePlayer(c) => {
            entry(texts, "player-colour", u32::from(*c), COLOURS).to_ascii_lowercase()
        }
    };
    texts.text(&key, &Args::new().context(&context).text("what", &what))
}

/// The text of a contents' hint (E24 M3 T6a2b, design §4.12); the other hints' texts are
/// [`Hint::text`]'s.
#[must_use]
pub fn text(hint: &Hint, names: &BTreeMap<u8, String>, texts: &Catalogue) -> String {
    let plain = |key: &str| texts.text(key, &Args::new());
    let object = |class: u32| {
        if class < CLASSES {
            plain(&alloc::format!("object.{class}"))
        } else {
            String::new()
        }
    };
    let owned = |owner: u8| entry(texts, "player-owned", u32::from(owner), COLOURS);
    let mark = |visited: bool| {
        plain(if visited {
            "map.hint.visited"
        } else {
            "map.hint.not-visited"
        })
    };
    match hint {
        Hint::Keymaster {
            class,
            colour,
            visited,
        } => {
            let args = Args::new()
                .text("colour", &entry(texts, "keymaster", *colour, COLOURS))
                .text("name", &object(*class));
            let mut text = texts.text("map.hint.keymaster", &args);
            if let Some(v) = visited {
                text.push_str(&more(texts, &mark(*v)));
            }
            text
        }
        Hint::Dwelling {
            class,
            kind,
            owner,
        } => {
            let (family, count) = if *class == 20 {
                ("dwelling4", 2)
            } else {
                ("dwelling1", 80)
            };
            let name = entry(texts, family, *kind, count);
            match owner {
                Some(o) => {
                    let args = Args::new().text("name", &name).text("owner", &owned(*o));
                    texts.text("map.hint.owned", &args)
                }
                None => name,
            }
        }
        Hint::Lighthouse { owner } => {
            let mut text = object(42);
            if let Some(o) = owner {
                let args = Args::new().text("owner", &owned(*o));
                text.push_str(&texts.text("map.hint.owner", &args));
            }
            text
        }
        Hint::Resource(n) => entry(texts, "resource", *n, RESOURCES),
        Hint::Mine {
            name,
            owner,
            produces,
            guards: g,
        } => {
            let mut text = entry(texts, "mine", u32::from(*name), RESOURCES);
            if let Some(o) = owner {
                text.push_str(&more(texts, &owned(*o)));
            }
            if let Some(r) = produces {
                let mut part = plain("map.hint.open");
                part.push_str(&entry(texts, "resource", u32::from(*r), RESOURCES));
                part.push_str(&plain("map.hint.close"));
                text.push_str(&more(texts, &part));
            }
            if let Some(g) = g {
                text.push_str(&more(texts, &guards(texts, *g)));
            }
            text
        }
        Hint::Bank { bank, mark: m } => {
            let tail = match m {
                BankMark::NotVisited => mark(false),
                BankMark::Visited => mark(true),
                BankMark::Guarded(g) => {
                    let mut part = plain("map.hint.open");
                    part.push_str(&guards(texts, *g));
                    part.push_str(&plain("map.hint.close"));
                    part
                }
            };
            let mut text = entry(texts, "bank", u32::from(*bank), BANKS);
            text.push_str(&more(texts, &tail));
            text
        }
        Hint::Teaches {
            class,
            bonus,
            teach,
            visited,
        } => {
            let mut text = object(*class);
            match teach {
                Some(Teach::Spell(spell, known)) => {
                    let name = plain(&alloc::format!("spell.{spell}"));
                    let args = Args::new().text("name", &name);
                    text.push_str(&more(texts, &texts.text("map.hint.learn-spell", &args)));
                    if *known {
                        text.push_str(&more(texts, &plain("map.hint.spell-known")));
                    }
                }
                Some(Teach::Skill(skill, known)) => {
                    let name = plain(&alloc::format!("secondary-skill.{skill}"));
                    let args = Args::new().text("name", &name);
                    text.push_str(&more(texts, &texts.text("map.hint.learn-skill", &args)));
                    if *known {
                        text.push_str(&more(texts, &plain("map.hint.skill-known")));
                    }
                }
                Some(Teach::Price(p)) => {
                    text.push_str(&more(texts, &entry(texts, "tree-price", u32::from(*p), 3)));
                }
                None => {
                    if let Some(n) = bonus {
                        text.push_str(&more(texts, &plain(&alloc::format!("map-bonus.{n}"))));
                    }
                }
            }
            if let Some(v) = visited {
                text.push_str(&more(texts, &mark(*v)));
            }
            text
        }
        Hint::Quest {
            class,
            seer,
            quest,
        } => {
            let mut text = match seer {
                Some(n) => {
                    let name = entry(texts, "seer-name", u32::from(*n), SEER_NAMES);
                    texts.text("map.hint.seer", &Args::new().text("name", &name))
                }
                None => object(*class),
            };
            if let Some(q) = quest {
                text.push_str(&more(texts, &quest_text(names, texts, q, *class == SEER)));
            }
            text
        }
        _ => String::new(),
    }
}

/// The creature banks' kinds (CrBanks.txt's).
pub const BANKS: u32 = 11;
/// The seers' names (SeerHut.txt's records 50 on).
pub const SEER_NAMES: u32 = 48;
/// The Seer's Huts' class (83).
pub const SEER: u32 = 83;

#[cfg(test)]
mod tests;
