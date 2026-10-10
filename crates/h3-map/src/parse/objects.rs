//! Parsing the object templates, the objects and the timed events (E08 T3b). The body of an
//! object follows from its template's class as in VCMI 1.7.5's `CMapLoaderH3M::readObject` and
//! the readers it calls (HotA's branches left out: the reader rejects HotA maps).

use super::{artifact, artifacts, id, secondary_skills};
use crate::ReadError;
use crate::cursor::Cursor;
use crate::model::{
    Army, Body, Buildings, EventBody, HeroBody, Message, Mission, MonsterBody, MonsterTreasure,
    Object, Quest, RandomDwelling, Reward, Rewards, SeerTask, Stack, Template, Text, TimedEvent,
    TownBody, TownEvent, Version,
};

/// Army slots of a creature set.
const ARMY_SLOTS: usize = 7;
/// Smallest template, object, timed event and town event record, in bytes.
const TEMPLATE_MIN: usize = 46;
const OBJECT_MIN: usize = 12;
const EVENT_MIN: usize = 57;
const TOWN_EVENT_MIN: usize = EVENT_MIN + 24;

pub(super) fn templates(c: &mut Cursor<'_>) -> Result<Vec<Template>, ReadError> {
    let n = c.count("templates", c.limits.max_templates, TEMPLATE_MIN)?;
    (0..n)
        .map(|_| {
            let t = Template {
                def: c.text("template name")?,
                passable: c.array("template passability")?,
                visitable: c.array("template visitability")?,
                terrains: c.u16("template terrains")?,
                editor_terrains: c.u16("template editor terrains")?,
                class: c.u32("template class")?,
                subclass: c.u32("template subclass")?,
                group: c.u8("template group")?,
                priority: c.u8("template priority")?,
            };
            c.skip(16, "template unused")?;
            Ok(t)
        })
        .collect()
}

pub(super) fn objects(
    c: &mut Cursor<'_>,
    v: Version,
    templates: &[Template],
) -> Result<Vec<Object>, ReadError> {
    let n = c.count("objects", c.limits.max_objects, OBJECT_MIN)?;
    (0..n)
        .map(|_| {
            let pos = c.pos("object position")?;
            let offset = c.offset();
            let index = c.u32("object template")?;
            let template = usize::try_from(index)
                .ok()
                .and_then(|i| templates.get(i))
                .ok_or(ReadError::InvalidValue {
                    field: "object template",
                    value: u64::from(index),
                    offset,
                })?;
            c.skip(5, "object unused")?;
            Ok(Object {
                pos,
                template: index,
                body: body(c, v, template)?,
            })
        })
        .collect()
}

pub(super) fn timed_events(c: &mut Cursor<'_>, v: Version) -> Result<Vec<TimedEvent>, ReadError> {
    let n = c.count("timed events", c.limits.max_events, EVENT_MIN)?;
    (0..n).map(|_| timed_event(c, v)).collect()
}

/// The body the template's class carries.
fn body(c: &mut Cursor<'_>, v: Version, t: &Template) -> Result<Body, ReadError> {
    Ok(match t.class {
        26 => Body::Event(event(c, v)?),
        34 | 62 | 70 => Body::Hero(Box::new(hero(c, v)?)),
        54 | 71..=75 | 162..=164 => Body::Monster(monster(c, v)?),
        59 | 91 => {
            let text = c.text("sign message")?;
            c.skip(4, "sign unused")?;
            Body::Sign(text)
        }
        83 => Body::SeerHut(seer_hut(c, v)?),
        113 => Body::WitchHut(if v == Version::RoE {
            Default::default()
        } else {
            c.bits(4, "witch hut skills")?
        }),
        81 => {
            let kind = c.i8("scholar bonus kind")?;
            let code = c.u8("scholar bonus")?;
            c.skip(6, "scholar unused")?;
            Body::Scholar { kind, code }
        }
        33 | 219 => {
            let owner = c.u32("garrison owner")?;
            let army = army(c, v)?;
            let removable = v == Version::RoE || c.bool("garrison removable")?;
            c.skip(8, "garrison unused")?;
            Body::Garrison {
                owner,
                army,
                removable,
            }
        }
        5 | 65..=69 => Body::Artifact(message(c, v)?),
        93 => Body::Scroll {
            message: message(c, v)?,
            spell: c.u32("scroll spell")?,
        },
        76 | 79 => {
            let message = message(c, v)?;
            let amount = c.u32("resource amount")?;
            c.skip(4, "resource unused")?;
            Body::Resource { message, amount }
        }
        77 | 98 => Body::Town(Box::new(town(c, v)?)),
        53 | 220 if t.subclass < 7 => Body::Mine {
            owner: c.u32("mine owner")?,
        },
        53 | 220 => Body::AbandonedMine(c.bits(4, "abandoned mine resources")?),
        17..=20 | 42 | 87 => Body::Owned {
            owner: c.u32("owner")?,
        },
        216..=218 => Body::RandomDwelling(random_dwelling(c, v, t.class)?),
        88..=90 => Body::Shrine {
            spell: c.u32("shrine spell")?,
        },
        6 => {
            let message = message(c, v)?;
            let rewards = rewards(c, v)?;
            Body::Pandora { message, rewards }
        }
        36 if t.subclass < 1000 => Body::Grail {
            radius: c.i32("grail radius")?,
        },
        215 => Body::QuestGuard(quest(c, v)?),
        214 => {
            let owner = c.u8("placeholder owner")?;
            let hero_type = c.u8("placeholder hero")?;
            let power_rank = if hero_type == u8::MAX {
                Some(c.u8("placeholder power rank")?)
            } else {
                None
            };
            Body::HeroPlaceholder {
                owner,
                hero_type,
                power_rank,
            }
        }
        _ => Body::None,
    })
}

/// Seven slots of (creature code, count); the format's "none" code is an empty slot.
fn army(c: &mut Cursor<'_>, v: Version) -> Result<Army, ReadError> {
    let none = if v == Version::RoE {
        u16::from(u8::MAX)
    } else {
        u16::MAX
    };
    (0..ARMY_SLOTS)
        .map(|_| {
            let creature = id(c, v, "army creature")?;
            let count = c.u16("army count")?;
            Ok((creature != none).then_some(Stack { creature, count }))
        })
        .collect()
}

fn stack(c: &mut Cursor<'_>, v: Version) -> Result<Stack, ReadError> {
    Ok(Stack {
        creature: id(c, v, "creature")?,
        count: c.u16("creature count")?,
    })
}

/// Seven values, one per resource.
fn resources<T: Copy + Default>(
    c: &mut Cursor<'_>,
    mut read: impl FnMut(&mut Cursor<'_>) -> Result<T, ReadError>,
) -> Result<[T; 7], ReadError> {
    let mut out = [T::default(); 7];
    for r in &mut out {
        *r = read(c)?;
    }
    Ok(out)
}

/// A guards' message behind its flag: the text, the guards behind theirs, four unused bytes.
fn message(c: &mut Cursor<'_>, v: Version) -> Result<Option<Message>, ReadError> {
    if !c.bool("has message")? {
        return Ok(None);
    }
    let text = c.text("message")?;
    let guards = if c.bool("has guards")? {
        Some(army(c, v)?)
    } else {
        None
    };
    c.skip(4, "message unused")?;
    Ok(Some(Message { text, guards }))
}

/// A `u8` count of records read by `item`.
fn list<T>(
    c: &mut Cursor<'_>,
    field: &'static str,
    min_record: usize,
    mut item: impl FnMut(&mut Cursor<'_>) -> Result<T, ReadError>,
) -> Result<Vec<T>, ReadError> {
    let n = c.count_u8(field, min_record)?;
    (0..n).map(|_| item(c)).collect()
}

fn rewards(c: &mut Cursor<'_>, v: Version) -> Result<Rewards, ReadError> {
    let r = Rewards {
        experience: c.u32("reward experience")?,
        mana: c.i32("reward mana")?,
        morale: c.i8("reward morale")?,
        luck: c.i8("reward luck")?,
        resources: resources(c, |c| c.i32("resources"))?,
        primary: c.array("reward primary skills")?,
        skills: list(c, "reward skills", 2, |c| {
            Ok((c.u8("reward skill")?, c.u8("reward skill level")?))
        })?,
        artifacts: list(c, "reward artifacts", v.id_bytes(), |c| {
            id(c, v, "reward artifact")
        })?,
        spells: list(c, "reward spells", 1, |c| c.u8("reward spell"))?,
        creatures: list(c, "reward creatures", v.id_bytes() + 2, |c| stack(c, v))?,
    };
    c.skip(8, "reward unused")?;
    Ok(r)
}

fn event(c: &mut Cursor<'_>, v: Version) -> Result<EventBody, ReadError> {
    let message = message(c, v)?;
    let rewards = rewards(c, v)?;
    let e = EventBody {
        message,
        rewards,
        players: c.u8("event players")?,
        computer: c.bool("event computer")?,
        remove_after_visit: c.bool("event removed after visit")?,
    };
    c.skip(4, "event unused")?;
    Ok(e)
}

fn hero(c: &mut Cursor<'_>, v: Version) -> Result<HeroBody, ReadError> {
    let identifier = if v == Version::RoE {
        None
    } else {
        Some(c.u32("hero identifier")?)
    };
    let owner = c.u8("hero owner")?;
    let hero_type = c.u8("hero type")?;
    let name = if c.bool("hero has name")? {
        Some(c.text("hero name")?)
    } else {
        None
    };
    let experience = if v != Version::Sod || c.bool("hero has experience")? {
        Some(c.u32("hero experience")?)
    } else {
        None
    };
    let portrait = if c.bool("hero has portrait")? {
        Some(c.u8("hero portrait")?)
    } else {
        None
    };
    let secondary_skills = if c.bool("hero has skills")? {
        Some(secondary_skills(c)?)
    } else {
        None
    };
    let army = if c.bool("hero has army")? {
        Some(army(c, v)?)
    } else {
        None
    };
    let formation = c.u8("hero formation")?;
    let artifacts = artifacts(c, v)?;
    let patrol = c.u8("hero patrol")?;
    let (biography, sex) = if v == Version::RoE {
        (None, u8::MAX)
    } else {
        let biography = if c.bool("hero has biography")? {
            Some(c.text("hero biography")?)
        } else {
            None
        };
        (biography, c.u8("hero sex")?)
    };
    let spell = if v == Version::Ab {
        Some(c.u8("hero spell")?)
    } else {
        None
    };
    let (spells, primary_skills) = if v == Version::Sod {
        let spells = if c.bool("hero has spells")? {
            Some(c.bits(9, "hero spells")?)
        } else {
            None
        };
        let primary = if c.bool("hero has primary skills")? {
            Some(c.array("hero primary skills")?)
        } else {
            None
        };
        (spells, primary)
    } else {
        (None, None)
    };
    c.skip(16, "hero unused")?;
    Ok(HeroBody {
        identifier,
        owner,
        hero_type,
        name,
        experience,
        portrait,
        secondary_skills,
        army,
        formation,
        artifacts,
        patrol,
        biography,
        sex,
        spell,
        spells,
        primary_skills,
    })
}

fn monster(c: &mut Cursor<'_>, v: Version) -> Result<MonsterBody, ReadError> {
    let identifier = if v == Version::RoE {
        None
    } else {
        Some(c.u32("monster identifier")?)
    };
    let count = c.u16("monster count")?;
    let disposition = c.u8("monster disposition")?;
    let treasure = if c.bool("monster has message")? {
        Some(MonsterTreasure {
            text: c.text("monster message")?,
            resources: resources(c, |c| c.i32("resources"))?,
            artifact: artifact(c, v)?,
        })
    } else {
        None
    };
    let m = MonsterBody {
        identifier,
        count,
        disposition,
        treasure,
        never_flees: c.bool("monster never flees")?,
        no_growth: c.bool("monster does not grow")?,
    };
    c.skip(2, "monster unused")?;
    Ok(m)
}

fn seer_hut(c: &mut Cursor<'_>, v: Version) -> Result<Option<SeerTask>, ReadError> {
    let quest = if v == Version::RoE {
        // RoE's seer hut asks for one artifact, without a deadline or texts.
        artifact(c, v)?.map(|a| Quest {
            mission: Mission::Artifacts(vec![a]),
            deadline: -1,
            first_visit: Text::default(),
            next_visit: Text::default(),
            completed: Text::default(),
        })
    } else {
        quest(c, v)?
    };
    let task = match quest {
        Some(quest) => Some(SeerTask {
            quest,
            reward: reward(c, v)?,
        }),
        None => {
            c.skip(1, "seer hut reward")?;
            None
        }
    };
    c.skip(2, "seer hut unused")?;
    Ok(task)
}

/// A quest in the AB and SoD layout; `None` for mission code 0 (no quest, nothing follows).
fn quest(c: &mut Cursor<'_>, v: Version) -> Result<Option<Quest>, ReadError> {
    let offset = c.offset();
    let mission = match c.u8("quest mission")? {
        0 => return Ok(None),
        1 => Mission::Level(c.u32("quest level")?),
        2 => Mission::PrimarySkills(c.array("quest primary skills")?),
        3 => Mission::KillHero(c.u32("quest hero")?),
        4 => Mission::KillCreature(c.u32("quest monster")?),
        5 => Mission::Artifacts(list(c, "quest artifacts", v.id_bytes(), |c| {
            id(c, v, "quest artifact")
        })?),
        6 => Mission::Army(list(c, "quest creatures", v.id_bytes() + 2, |c| {
            stack(c, v)
        })?),
        7 => Mission::Resources(resources(c, |c| c.u32("quest resources"))?),
        8 => Mission::Hero(c.u8("quest hero type")?),
        9 => Mission::Player(c.u8("quest player")?),
        code => {
            return Err(ReadError::InvalidValue {
                field: "quest mission",
                value: u64::from(code),
                offset,
            });
        }
    };
    Ok(Some(Quest {
        mission,
        deadline: c.i32("quest deadline")?,
        first_visit: c.text("quest first visit")?,
        next_visit: c.text("quest next visit")?,
        completed: c.text("quest completed")?,
    }))
}

fn reward(c: &mut Cursor<'_>, v: Version) -> Result<Reward, ReadError> {
    let offset = c.offset();
    Ok(match c.u8("seer hut reward")? {
        0 => Reward::Nothing,
        1 => Reward::Experience(c.u32("reward experience")?),
        2 => Reward::Mana(c.u32("reward mana")?),
        3 => Reward::Morale(c.i8("reward morale")?),
        4 => Reward::Luck(c.i8("reward luck")?),
        5 => Reward::Resource {
            resource: c.u8("reward resource")?,
            amount: c.u32("reward amount")?,
        },
        6 => Reward::PrimarySkill {
            skill: c.u8("reward primary skill")?,
            amount: c.u8("reward amount")?,
        },
        7 => Reward::SecondarySkill {
            skill: c.u8("reward skill")?,
            level: c.u8("reward skill level")?,
        },
        8 => Reward::Artifact(id(c, v, "reward artifact")?),
        9 => Reward::Spell(c.u8("reward spell")?),
        10 => Reward::Creatures(stack(c, v)?),
        code => {
            return Err(ReadError::InvalidValue {
                field: "seer hut reward",
                value: u64::from(code),
                offset,
            });
        }
    })
}

fn town(c: &mut Cursor<'_>, v: Version) -> Result<TownBody, ReadError> {
    let identifier = if v == Version::RoE {
        None
    } else {
        Some(c.u32("town identifier")?)
    };
    let owner = c.u8("town owner")?;
    let name = if c.bool("town has name")? {
        Some(c.text("town name")?)
    } else {
        None
    };
    let garrison = if c.bool("town has garrison")? {
        Some(army(c, v)?)
    } else {
        None
    };
    let formation = c.u8("town formation")?;
    let buildings = if c.bool("town has custom buildings")? {
        Buildings::Custom {
            built: c.array("town buildings")?,
            forbidden: c.array("town forbidden buildings")?,
        }
    } else {
        Buildings::Standard {
            fort: c.bool("town has fort")?,
        }
    };
    let required_spells = if v == Version::RoE {
        None
    } else {
        Some(c.bits(9, "town required spells")?)
    };
    let banned_spells = c.bits(9, "town banned spells")?;
    let n = c.count("town events", c.limits.max_events, TOWN_EVENT_MIN)?;
    let events = (0..n)
        .map(|_| {
            let event = timed_event(c, v)?;
            let buildings = c.array("town event buildings")?;
            let mut creatures = [0; 7];
            for n in &mut creatures {
                *n = c.u16("town event creatures")?;
            }
            c.skip(4, "town event unused")?;
            Ok(TownEvent {
                event,
                buildings,
                creatures,
            })
        })
        .collect::<Result<_, ReadError>>()?;
    let alignment = if v == Version::Sod {
        c.u8("town alignment")?
    } else {
        u8::MAX
    };
    c.skip(3, "town unused")?;
    Ok(TownBody {
        identifier,
        owner,
        name,
        garrison,
        formation,
        buildings,
        required_spells,
        banned_spells,
        events,
        alignment,
    })
}

fn timed_event(c: &mut Cursor<'_>, v: Version) -> Result<TimedEvent, ReadError> {
    let e = TimedEvent {
        name: c.text("event name")?,
        message: c.text("event message")?,
        resources: resources(c, |c| c.i32("resources"))?,
        players: c.u8("event players")?,
        human: v != Version::Sod || c.bool("event human")?,
        computer: c.bool("event computer")?,
        first_day: c.u16("event first day")?,
        interval: c.u16("event interval")?,
    };
    c.skip(16, "event unused")?;
    Ok(e)
}

fn random_dwelling(
    c: &mut Cursor<'_>,
    v: Version,
    class: u32,
) -> Result<RandomDwelling, ReadError> {
    let owner = c.u32("dwelling owner")?;
    let faction = if class == 216 || class == 217 {
        let town = c.u32("dwelling town")?;
        let set = if town == 0 {
            Some(c.bits(v.faction_bytes(), "dwelling factions")?)
        } else {
            None
        };
        Some((town, set))
    } else {
        None
    };
    let levels = if class == 216 || class == 218 {
        Some((
            c.u8("dwelling lowest level")?,
            c.u8("dwelling highest level")?,
        ))
    } else {
        None
    };
    Ok(RandomDwelling {
        owner,
        faction,
        levels,
    })
}
