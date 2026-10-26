//! The objects that hand a visiting hero something (E08 T56, batch E7's maps; survey
//! `docs/rules/surveys/E08-rewards.md` §11): artifacts and random artifacts with their guards,
//! Spell Scrolls, Shrines, Witch Huts, Pandora's Boxes and events, each with the body the
//! format gives it (`docs/formats/h3m.md`, "Rewards"), and a monster's treasure. The objects
//! without a body among increment 7's (a chest, a Corpse, the Pyramid, …) are
//! [`Plain`](super::Plain) objects.

use super::{ARMY_SLOTS, Codes, Format, ObjectTemplate, Out, WriteError, out_of_range};
use crate::model::Rewards;

/// Spells a map may name: the 70 of `SPTRAITS.TXT` (codes 0–69, the bits of the format's 9-byte
/// spell masks), in every version.
pub const SPELL_COUNT: u8 = 70;
/// The object class of an artifact (subclass = the artifact).
pub const ARTIFACT_OBJECT: u32 = 5;
/// The object classes of the random artifacts: any (65), a treasure (66), a minor (67), a major
/// (68) and a relic (69).
pub const RANDOM_ARTIFACT_OBJECTS: [u32; 5] = [65, 66, 67, 68, 69];
/// The object class of Pandora's Box.
pub const PANDORA_OBJECT: u32 = 6;
/// The object class of an event.
pub const EVENT_OBJECT: u32 = 26;
/// The object classes of the Shrines of the first, second and third circle.
pub const SHRINE_OBJECTS: [u32; 3] = [88, 89, 90];
/// The object class of a Spell Scroll.
pub const SCROLL_OBJECT: u32 = 93;
/// The object class of a Witch Hut.
pub const WITCH_HUT_OBJECT: u32 = 113;
/// A Witch Hut's possible skills as the editor sets them: all 28 but Leadership (6) and
/// Necromancy (12); 213 of the 259 witch huts in the game's 56 SoD maps (E24 M1 T3a).
pub const WITCH_HUT_SKILLS: [u8; 4] = [0xbf, 0xef, 0xff, 0x0f];
/// The largest morale or luck a reward record gives or takes (the editor's range, −3 to 3).
const MAX_MORALE: i8 = 3;

/// A message record (`docs/formats/h3m.md`, "Shared parts of the bodies"): the text the game
/// shows at the visit and the guards that fight for the object. With guards, the text is the
/// question asked before the battle (empty: the guards fight at once).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Message {
    /// The text (Windows-1252, at most Latin-1).
    pub text: String,
    /// The guards (creature id, count) by slot; `Some` holds at least one stack.
    pub guards: Option<[Option<(u16, u16)>; ARMY_SLOTS]>,
}

/// An event's body beyond its message and reward record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Event {
    /// The message record.
    pub message: Option<Message>,
    /// What it gives (or takes: negative resources, mana, morale and luck).
    pub rewards: Rewards,
    /// The players it fires for, a bit per colour (bit 0 red … bit 7 pink).
    pub players: u8,
    /// A computer player's hero fires it too.
    pub computer: bool,
    /// It is removed after it fired once; otherwise it fires at every entry.
    pub removed_after_visit: bool,
}

/// What a reward object's body holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewardBody {
    /// An artifact (class 5, the artifact as subclass) or a random artifact (65–69): its message
    /// record.
    Artifact(Option<Message>),
    /// A Spell Scroll (93): its message record and its spell.
    Scroll {
        /// The message record.
        message: Option<Message>,
        /// The spell (below [`SPELL_COUNT`]).
        spell: u8,
    },
    /// A Shrine (88–90): its spell; `None` (`0xFF`) is drawn when the game starts.
    Shrine(Option<u8>),
    /// A Witch Hut (113): the skills it may teach, a bit per skill (bits 0–27); `None` is the
    /// editor's [`WITCH_HUT_SKILLS`]. An RoE map stores none (its huts draw from a fixed mask,
    /// A-21), so it writes only `None`.
    WitchHut(Option<[u8; 4]>),
    /// Pandora's Box (6): its message record and reward record.
    Pandora {
        /// The message record.
        message: Option<Message>,
        /// What it gives.
        rewards: Rewards,
    },
    /// An event (26).
    Event(Event),
}

/// An object that hands a visiting hero something (E08 T56): its template and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardObject {
    /// The object's position: the bottom-right tile of its picture (its masks say which tiles it
    /// blocks and where it is visited).
    pub tile: (u8, u8),
    /// Template (from `Objects.txt`; its class must fit the body).
    pub template: ObjectTemplate,
    /// The body.
    pub body: RewardBody,
}

/// A wandering monster's message and treasure (`docs/formats/h3m.md`, "Monster body"): the text
/// shown when it is beaten, the resources and the artifact it gives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Treasure {
    /// The text.
    pub text: String,
    /// Resources in `RESTYPES.TXT` order (wood … gold), as the map stores them.
    pub resources: [i32; 7],
    /// The artifact, if any.
    pub artifact: Option<u16>,
}

pub(super) fn validate(object: &RewardObject, codes: &Codes) -> Result<(), WriteError> {
    let class = object.template.class;
    let fits = match &object.body {
        RewardBody::Artifact(_) => {
            class == ARTIFACT_OBJECT || RANDOM_ARTIFACT_OBJECTS.contains(&class)
        }
        RewardBody::Scroll { .. } => class == SCROLL_OBJECT,
        RewardBody::Shrine(_) => SHRINE_OBJECTS.contains(&class),
        RewardBody::WitchHut(_) => class == WITCH_HUT_OBJECT,
        RewardBody::Pandora { .. } => class == PANDORA_OBJECT,
        RewardBody::Event(_) => class == EVENT_OBJECT,
    };
    if !fits {
        return Err(out_of_range("reward object class", class));
    }
    if class == ARTIFACT_OBJECT {
        codes.artifact(object.template.subclass)?;
    }
    match &object.body {
        RewardBody::Artifact(message) => validate_message(message.as_ref(), codes),
        RewardBody::Scroll { message, spell } => {
            validate_spell(*spell)?;
            validate_message(message.as_ref(), codes)
        }
        RewardBody::Shrine(spell) => spell.map_or(Ok(()), validate_spell),
        RewardBody::WitchHut(None) => Ok(()),
        RewardBody::WitchHut(Some(skills)) => {
            if codes.format == Format::RoE {
                return Err(out_of_range("witch hut skills in an RoE map", skills[0]));
            }
            if skills[3] >> 4 != 0 {
                return Err(out_of_range("witch hut skill (bits of byte 3)", skills[3]));
            }
            Ok(())
        }
        RewardBody::Pandora { message, rewards } => {
            validate_message(message.as_ref(), codes)?;
            validate_rewards(rewards, codes)
        }
        RewardBody::Event(event) => {
            validate_message(event.message.as_ref(), codes)?;
            validate_rewards(&event.rewards, codes)
        }
    }
}

pub(super) fn validate_treasure(treasure: &Treasure, codes: &Codes) -> Result<(), WriteError> {
    treasure.artifact.map_or(Ok(()), |a| codes.artifact(a))
}

fn validate_spell(spell: u8) -> Result<(), WriteError> {
    if spell < SPELL_COUNT {
        Ok(())
    } else {
        Err(out_of_range("spell", spell))
    }
}

fn validate_message(message: Option<&Message>, codes: &Codes) -> Result<(), WriteError> {
    match message.and_then(|m| m.guards.as_ref()) {
        Some(guards) if guards.iter().all(Option::is_none) => {
            Err(out_of_range("message guards (no stack)", 0u8))
        }
        Some(guards) => codes.army(guards),
        None => Ok(()),
    }
}

fn validate_rewards(rewards: &Rewards, codes: &Codes) -> Result<(), WriteError> {
    for (what, value) in [
        ("reward morale", rewards.morale),
        ("reward luck", rewards.luck),
    ] {
        if !(-MAX_MORALE..=MAX_MORALE).contains(&value) {
            return Err(out_of_range(what, value.unsigned_abs()));
        }
    }
    let lists = [
        rewards.skills.len(),
        rewards.artifacts.len(),
        rewards.spells.len(),
        rewards.creatures.len(),
    ];
    if let Some(&len) = lists.iter().find(|&&len| len > usize::from(u8::MAX)) {
        return Err(out_of_range("reward list length", len));
    }
    for &(skill, level) in &rewards.skills {
        codes.skill(skill, level)?;
    }
    for &artifact in &rewards.artifacts {
        codes.artifact(artifact)?;
    }
    for &spell in &rewards.spells {
        validate_spell(spell)?;
    }
    for stack in &rewards.creatures {
        codes.stack(stack.creature, stack.count)?;
    }
    Ok(())
}

/// The body of a reward object, as the format lays it out for its class.
pub(super) fn write_body(out: &mut Out, body: &RewardBody) -> Result<(), WriteError> {
    match body {
        RewardBody::Artifact(message) => write_message(out, message.as_ref()),
        RewardBody::Scroll { message, spell } => {
            write_message(out, message.as_ref())?;
            out.u32(u32::from(*spell));
            Ok(())
        }
        RewardBody::Shrine(spell) => {
            out.u32(spell.map_or(0xff, u32::from));
            Ok(())
        }
        RewardBody::WitchHut(skills) => {
            if out.1 != Format::RoE {
                out.bytes(skills.as_ref().unwrap_or(&WITCH_HUT_SKILLS));
            }
            Ok(())
        }
        RewardBody::Pandora { message, rewards } => {
            write_message(out, message.as_ref())?;
            write_rewards(out, rewards);
            Ok(())
        }
        RewardBody::Event(event) => {
            write_message(out, event.message.as_ref())?;
            write_rewards(out, &event.rewards);
            out.u8(event.players);
            out.bool(event.computer);
            out.bool(event.removed_after_visit);
            out.bytes(&[0; 4]);
            Ok(())
        }
    }
}

/// A message record behind its flag: the text, the guards behind theirs, 4 unused bytes.
pub(super) fn write_message(out: &mut Out, message: Option<&Message>) -> Result<(), WriteError> {
    out.bool(message.is_some());
    if let Some(message) = message {
        out.string(&message.text)?;
        out.bool(message.guards.is_some());
        if let Some(guards) = &message.guards {
            out.army(guards);
        }
        out.bytes(&[0; 4]);
    }
    Ok(())
}

/// A reward record (`docs/formats/h3m.md`, "Rewards"): experience, mana, morale, luck, the seven
/// resources, the four primary skills, the lists of skills, artifacts, spells and creatures (a
/// `u8` count each), 8 unused bytes.
fn write_rewards(out: &mut Out, rewards: &Rewards) {
    out.u32(rewards.experience);
    out.u32(rewards.mana.cast_unsigned());
    out.u8(rewards.morale.cast_unsigned());
    out.u8(rewards.luck.cast_unsigned());
    for amount in rewards.resources {
        out.u32(amount.cast_unsigned());
    }
    out.bytes(&rewards.primary);
    // The lengths were checked to fit a byte.
    let count = |len: usize| u8::try_from(len).unwrap_or(u8::MAX);
    out.u8(count(rewards.skills.len()));
    for &(skill, level) in &rewards.skills {
        out.bytes(&[skill, level]);
    }
    out.u8(count(rewards.artifacts.len()));
    for &artifact in &rewards.artifacts {
        out.code(Some(artifact));
    }
    out.u8(count(rewards.spells.len()));
    out.bytes(&rewards.spells);
    out.u8(count(rewards.creatures.len()));
    for stack in &rewards.creatures {
        out.code(Some(stack.creature));
        out.u16(stack.count);
    }
    out.bytes(&[0; 8]);
}

/// A monster's message and treasure behind its flag: the text, the seven resources, the artifact.
pub(super) fn write_treasure(out: &mut Out, treasure: Option<&Treasure>) -> Result<(), WriteError> {
    out.bool(treasure.is_some());
    if let Some(treasure) = treasure {
        out.string(&treasure.text)?;
        for amount in treasure.resources {
            out.u32(amount.cast_unsigned());
        }
        out.code(treasure.artifact);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::smoke_like_map;
    use super::super::{
        Format, Hero, HeroArtifacts, Map, MapObject, Monster, Pile, Plain, Player, Town,
        TownBuildings, write, write_in,
    };
    use super::*;
    use crate::model::Stack;

    fn template(class: u32, subclass: u32) -> ObjectTemplate {
        ObjectTemplate {
            def: format!("o{class}s{subclass}.def"),
            passable: [0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            visitable: [0, 0, 0, 0, 0, 0x80],
            terrains: 0x1ff,
            menu: 0x1ff,
            class,
            subclass,
            group: 0,
            priority: 0,
        }
    }

    fn reward(class: u32, subclass: u32, body: RewardBody) -> MapObject {
        MapObject::Reward(RewardObject {
            tile: (5, 5),
            template: template(class, subclass),
            body,
        })
    }

    fn guarded(text: &str) -> Message {
        let mut guards = [None; ARMY_SLOTS];
        guards[1] = Some((0, 12));
        Message {
            text: text.into(),
            guards: Some(guards),
        }
    }

    /// The smoke map with `object` added, written in `format`.
    fn written(object: MapObject, format: Format) -> Result<Vec<u8>, WriteError> {
        let mut map = smoke_like_map();
        if format == Format::RoE {
            map.objects.retain(|o| !matches!(o, MapObject::Hero(_)));
        }
        map.objects.push(object);
        write_in(&map, format)
    }

    fn refused(object: MapObject, what: &'static str, value: u64) {
        assert_eq!(
            written(object, Format::Sod),
            Err(WriteError::OutOfRange { what, value })
        );
    }

    /// The body of the last object: the bytes after its 12-byte record, up to the timed events'
    /// count.
    fn last_body(bytes: &[u8], len: usize) -> &[u8] {
        &bytes[bytes.len() - 4 - len..bytes.len() - 4]
    }

    #[test]
    fn each_body_has_the_format_s_layout() {
        let artifact = written(reward(5, 7, RewardBody::Artifact(None)), Format::Sod).unwrap();
        assert_eq!(last_body(&artifact, 1), [0]);
        let message = Some(Message {
            text: "Hi".into(),
            guards: None,
        });
        let scroll = written(
            reward(93, 0, RewardBody::Scroll { message, spell: 15 }),
            Format::Sod,
        )
        .unwrap();
        // The message flag, its text, no guards, 4 unused bytes; the spell as a u32.
        assert_eq!(
            last_body(&scroll, 16),
            [1, 2, 0, 0, 0, b'H', b'i', 0, 0, 0, 0, 0, 15, 0, 0, 0]
        );
        let shrine = written(reward(88, 0, RewardBody::Shrine(None)), Format::Sod).unwrap();
        assert_eq!(last_body(&shrine, 4), [0xff, 0, 0, 0]);
        let shrine = written(reward(90, 0, RewardBody::Shrine(Some(69))), Format::Sod).unwrap();
        assert_eq!(last_body(&shrine, 4), [69, 0, 0, 0]);
        let hut = written(reward(113, 0, RewardBody::WitchHut(None)), Format::Sod).unwrap();
        assert_eq!(last_body(&hut, 4), WITCH_HUT_SKILLS);
        let hut = written(
            reward(113, 0, RewardBody::WitchHut(Some([1, 2, 3, 4]))),
            Format::Sod,
        )
        .unwrap();
        assert_eq!(last_body(&hut, 4), [1, 2, 3, 4]);
        let rewards = Rewards {
            experience: 0x0102,
            mana: -2,
            morale: -3,
            luck: 3,
            resources: [1, 0, 0, 0, 0, 0, -1],
            primary: [1, 2, 3, 4],
            skills: vec![(7, 2)],
            artifacts: vec![0x83],
            spells: vec![15, 16],
            creatures: vec![Stack {
                creature: 0x90,
                count: 0x0205,
            }],
        };
        let mut expected = vec![0, 2, 1, 0, 0, 0xfe, 0xff, 0xff, 0xff, 0xfd, 3];
        expected.extend([1, 0, 0, 0]);
        expected.extend([0; 20]);
        expected.extend([0xff; 4]);
        expected.extend([1, 2, 3, 4, 1, 7, 2, 1, 0x83, 0, 2, 15, 16, 1, 0x90, 0, 5, 2]);
        expected.extend([0; 8]);
        let pandora = RewardBody::Pandora {
            message: None,
            rewards: rewards.clone(),
        };
        let bytes = written(reward(6, 0, pandora), Format::Sod).unwrap();
        assert_eq!(last_body(&bytes, expected.len()), expected);
        let event = RewardBody::Event(Event {
            message: None,
            rewards,
            players: 0b101,
            computer: true,
            removed_after_visit: false,
        });
        let bytes = written(reward(26, 0, event), Format::Sod).unwrap();
        expected.extend([0b101, 1, 0, 0, 0, 0, 0]);
        assert_eq!(last_body(&bytes, expected.len()), expected);
    }

    #[test]
    fn guards_follow_the_text_and_monsters_carry_their_treasure() {
        let bytes = written(
            reward(65, 0, RewardBody::Artifact(Some(guarded("")))),
            Format::Sod,
        )
        .unwrap();
        // The message flag, an empty text, the guards' flag and seven slots, 4 unused bytes.
        let body = last_body(&bytes, 38);
        assert_eq!(&body[..6], &[1, 0, 0, 0, 0, 1]);
        assert_eq!(&body[6..14], &[0xff, 0xff, 0, 0, 0, 0, 12, 0]);
        assert_eq!(&body[34..], &[0; 4]);
        let mut map = smoke_like_map();
        let MapObject::Monster(monster) = &mut map.objects[1] else {
            panic!("the smoke map's monster")
        };
        monster.treasure = Some(Treasure {
            text: "T".into(),
            resources: [0, 0, 0, 0, 0, 0, 500],
            artifact: Some(7),
        });
        let bytes = write(&map).unwrap();
        // Count, disposition, the treasure's flag, its text, seven resources and the artifact;
        // never flees, does not grow, 2 unused bytes.
        let body = last_body(&bytes, 3 + 1 + 5 + 28 + 2 + 4);
        assert_eq!(&body[..9], &[10, 0, 4, 1, 1, 0, 0, 0, b'T']);
        assert_eq!(&body[33..37], &[0xf4, 1, 0, 0]);
        assert_eq!(&body[37..], &[7, 0, 1, 1, 0, 0]);
    }

    #[test]
    fn bodies_and_classes_are_checked() {
        refused(
            reward(6, 0, RewardBody::Artifact(None)),
            "reward object class",
            6,
        );
        refused(
            reward(93, 0, RewardBody::Shrine(None)),
            "reward object class",
            93,
        );
        refused(
            reward(5, 0, RewardBody::WitchHut(None)),
            "reward object class",
            5,
        );
        refused(
            reward(5, 0, RewardBody::Shrine(None)),
            "reward object class",
            5,
        );
        refused(
            reward(5, 0, RewardBody::Event(Event::default())),
            "reward object class",
            5,
        );
        let pandora = |rewards| RewardBody::Pandora {
            message: None,
            rewards,
        };
        refused(
            reward(26, 0, pandora(Rewards::default())),
            "reward object class",
            26,
        );
        let scroll = |spell| RewardBody::Scroll {
            message: None,
            spell,
        };
        refused(reward(88, 0, scroll(0)), "reward object class", 88);
        refused(reward(5, 144, RewardBody::Artifact(None)), "artifact", 144);
        refused(reward(93, 0, scroll(SPELL_COUNT)), "spell", 70);
        refused(reward(89, 0, RewardBody::Shrine(Some(70))), "spell", 70);
        refused(
            reward(113, 0, RewardBody::WitchHut(Some([0, 0, 0, 0x10]))),
            "witch hut skill (bits of byte 3)",
            0x10,
        );
        let empty = Message {
            text: String::new(),
            guards: Some([None; ARMY_SLOTS]),
        };
        refused(
            reward(65, 0, RewardBody::Artifact(Some(empty))),
            "message guards (no stack)",
            0,
        );
        let mut message = guarded("");
        message.guards = message.guards.map(|mut g| {
            g[0] = Some((145, 1));
            g
        });
        refused(
            reward(65, 0, RewardBody::Artifact(Some(message))),
            "creature",
            145,
        );
        let with = |f: fn(&mut Rewards)| {
            let mut rewards = Rewards::default();
            f(&mut rewards);
            reward(6, 0, pandora(rewards))
        };
        refused(with(|r| r.morale = 4), "reward morale", 4);
        refused(with(|r| r.luck = -4), "reward luck", 4);
        refused(with(|r| r.skills = vec![(28, 1)]), "secondary skill", 28);
        refused(
            with(|r| r.skills = vec![(0, 0)]),
            "secondary skill level",
            0,
        );
        refused(with(|r| r.artifacts = vec![144]), "artifact", 144);
        refused(with(|r| r.spells = vec![70]), "spell", 70);
        refused(
            with(|r| {
                r.creatures = vec![Stack {
                    creature: 0,
                    count: 0,
                }];
            }),
            "stack size",
            0,
        );
        refused(with(|r| r.spells = vec![0; 256]), "reward list length", 256);
        assert!(written(with(|r| r.spells = vec![0; 255]), Format::Sod).is_ok());
        let event = |message| {
            reward(
                26,
                0,
                RewardBody::Event(Event {
                    message,
                    ..Event::default()
                }),
            )
        };
        assert!(written(event(Some(guarded("Fight?"))), Format::Sod).is_ok());
        let mut message = guarded("");
        message.guards = message.guards.map(|mut g| {
            g[1] = Some((0, 0));
            g
        });
        refused(event(Some(message)), "stack size", 0);
        let event_rewards = RewardBody::Event(Event {
            rewards: Rewards {
                artifacts: vec![200],
                ..Rewards::default()
            },
            ..Event::default()
        });
        refused(reward(26, 0, event_rewards), "artifact", 200);
        let monster = |f: fn(&mut Monster)| {
            let mut map = smoke_like_map();
            if let MapObject::Monster(monster) = &mut map.objects[1] {
                f(monster);
            }
            write(&map)
        };
        let refusal = |what, value| Err(WriteError::OutOfRange { what, value });
        assert_eq!(
            monster(|m| {
                m.treasure = Some(Treasure {
                    artifact: Some(144),
                    ..Treasure::default()
                });
            }),
            refusal("artifact", 144)
        );
        assert_eq!(
            monster(|m| m.template.subclass = 145),
            refusal("creature", 145)
        );
        assert!(monster(|m| m.template.subclass = 144).is_ok());
    }

    /// A map of every kind RoE can hold, and the refusals of what it cannot.
    #[test]
    fn roe_maps_hold_what_roe_can() {
        let map = smoke_like_map();
        assert_eq!(write_in(&map, Format::Sod), write(&map));
        // The smoke map's hero has primary skills; without them it is an RoE hero.
        let roe = |f: &dyn Fn(&mut Map)| {
            let mut map = smoke_like_map();
            if let MapObject::Hero(hero) = &mut map.objects[0] {
                hero.primary = None;
            }
            f(&mut map);
            write_in(&map, Format::RoE)
        };
        let bytes = roe(&|_| {}).unwrap();
        assert_eq!(&bytes[..4], &[0x0e, 0, 0, 0]);
        let refusal = |what, value| Err(WriteError::OutOfRange { what, value });
        assert_eq!(
            roe(&|m| m.level_limit = 5),
            refusal("level limit in an RoE map", 5)
        );
        assert_eq!(
            roe(&|m| m.banned_skills = vec![3]),
            refusal("banned skill in an RoE map", 3)
        );
        assert_eq!(
            roe(&|m| m.players[1] = Some(Player {
                human: true,
                computer: true,
                towns: 0x100,
            })),
            refusal("allowed towns in an RoE map (eight factions)", 0x100)
        );
        let hero = |f: fn(&mut Hero)| {
            move |m: &mut Map| {
                if let MapObject::Hero(hero) = &mut m.objects[0] {
                    f(hero);
                }
            }
        };
        assert_eq!(roe(&hero(|h| h.hero_type = 128)), refusal("hero type", 128));
        assert_eq!(
            roe(&hero(|h| h.spells = Some([0; 9]))),
            refusal("hero spells in an RoE map", 0)
        );
        assert_eq!(
            roe(&hero(|h| h.primary = Some([0; 4]))),
            refusal("hero primary skills in an RoE map", 0)
        );
        assert_eq!(
            roe(&hero(|h| h.artifacts = Some(HeroArtifacts {
                worn: [None; 19].map(|_: Option<u16>| Some(0)),
                backpack: vec![],
            }))),
            refusal("artifact at misc 5 in an RoE map", 0)
        );
        assert_eq!(
            roe(&hero(|h| h.artifacts = Some(HeroArtifacts {
                worn: [None; 19],
                backpack: vec![127],
            }))),
            refusal("artifact", 127)
        );
        assert_eq!(
            roe(&hero(
                |h| h.army = Some([Some((118, 1)), None, None, None, None, None, None])
            )),
            refusal("creature", 118)
        );
        assert_eq!(
            roe(&|m| m
                .objects
                .push(reward(113, 0, RewardBody::WitchHut(Some(WITCH_HUT_SKILLS))))),
            refusal("witch hut skills in an RoE map", 0xbf)
        );
        assert_eq!(
            roe(&|m| m.objects.push(MapObject::Town(Town {
                tile: (20, 10),
                template: template(98, 0),
                owner: None,
                garrison: None,
                tight_formation: false,
                buildings: TownBuildings::Standard { fort: true },
            }))),
            refusal("town in an RoE map", 0)
        );
        assert_eq!(
            roe(
                &|m| if let MapObject::Monster(monster) = &mut m.objects[1] {
                    monster.template.subclass = 118;
                }
            ),
            refusal("creature", 118)
        );
        assert_eq!(
            roe(&|m| m.objects.push(MapObject::Pile(Pile {
                tile: (20, 10),
                template: template(79, 0),
                amount: 0,
                guards: Some(super::super::Guards {
                    army: [Some((118, 1)), None, None, None, None, None, None],
                    question: String::new(),
                }),
            }))),
            refusal("creature", 118)
        );
        // RoE's codes are one byte: a guarded pile's slots take three bytes each.
        let pile = roe(&|m| {
            m.objects.push(MapObject::Pile(Pile {
                tile: (20, 10),
                template: template(79, 0),
                amount: 0,
                guards: Some(super::super::Guards {
                    army: [Some((117, 2)), None, None, None, None, None, None],
                    question: String::new(),
                }),
            }));
        })
        .unwrap();
        let body = last_body(&pile, 1 + 4 + 1 + 21 + 4 + 8);
        assert_eq!(&body[5..12], &[1, 117, 2, 0, 0xff, 0, 0]);
        // The increment's objects without a body, in either format.
        for class in [22, 63, 100, 101, 104, 105, 108] {
            let plain = MapObject::Plain(Plain {
                tile: (20, 10),
                template: template(class, 0),
            });
            assert!(roe(&|m| m.objects.push(plain.clone())).is_ok(), "{class}");
        }
        let monster = MapObject::Monster(Monster {
            tile: (20, 10),
            template: template(54, 117),
            count: 1,
            disposition: 0,
            never_flees: false,
            fixed_count: false,
            treasure: Some(Treasure {
                artifact: Some(127),
                ..Treasure::default()
            }),
        });
        assert_eq!(
            roe(&|m| m.objects.push(monster.clone())),
            refusal("artifact", 127)
        );
    }
}
