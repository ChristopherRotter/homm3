//! Increment 9's objects and the map's timed events (E08 T78, batch E9's maps; increment 9's
//! design note §4, survey `docs/rules/surveys/E08-economy.md` §2.1, §3.4, §11): the ownable
//! objects with their bodies ([`OwnedObject`]: the seven mines and the Lighthouse and the Shipyard
//! with their owners, the abandoned mine with the resources it may draw) and the timed events
//! ([`TimedEvent`]). The weekly objects (the Windmill, the Water Wheel, the Mystical Garden) have
//! no body: they are [`Plain`](super::Plain) objects.

use super::{Codes, Format, Map, ObjectTemplate, Out, WriteError, out_of_range};

/// The object class of a mine (subclass 0 to 6 the resource it produces, `RESTYPES.TXT` order;
/// subclass 7 the abandoned mine).
pub const MINE_OBJECT: u32 = 53;
/// SoD's class of the abandoned mine (subclass 7; the game loads it as 53.7, survey §2.1).
pub const ABANDONED_MINE_OBJECT: u32 = 220;
/// The subclass of an abandoned mine (of [`MINE_OBJECT`] or [`ABANDONED_MINE_OBJECT`]).
pub const ABANDONED_MINE_SUBCLASS: u32 = 7;
/// The object class of a Lighthouse.
pub const LIGHTHOUSE_OBJECT: u32 = 42;
/// The object class of a Shipyard.
pub const SHIPYARD_OBJECT: u32 = 87;
/// The resources an abandoned mine may draw: bits 0 (wood) to 6 (gold) of its body's first byte;
/// bit 7 names no resource of the game's (survey §3.4, A-31).
const ABANDONED_MINE_RESOURCES: u8 = 0x7f;

/// What an ownable object's body holds (`docs/formats/h3m.md`, "Object bodies by class").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnedBody {
    /// A mine ([`MINE_OBJECT`] below subclass 7), a Lighthouse or a Shipyard: its owner (0 red …
    /// 7 pink, a colour the map's players play); `None` is neutral (`0xFF`).
    Owner(Option<u8>),
    /// An abandoned mine ([`MINE_OBJECT`] or [`ABANDONED_MINE_OBJECT`], subclass 7): the resources
    /// it may draw when the map loads, a bit per resource (bit 0 wood … bit 6 gold), at least one
    /// (the editor's rule). Its body holds no owner: it starts neutral.
    Resources(u8),
}

/// An ownable object (E08 T78): its template and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedObject {
    /// The object's position: the bottom-right tile of its picture (its masks say which tiles it
    /// blocks and where it is visited).
    pub tile: (u8, u8),
    /// Template (from `Objects.txt`; its class must fit the body).
    pub template: ObjectTemplate,
    /// The body.
    pub body: OwnedBody,
}

/// A timed event of the map (`docs/formats/h3m.md`, "Shared parts of the bodies"; survey §11):
/// on its days it gives or takes resources from its players at their turn's start.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimedEvent {
    /// The editor's name for it (the game never shows it).
    pub name: String,
    /// The text the game shows when it fires.
    pub message: String,
    /// Resources it gives (negative: takes), `RESTYPES.TXT` order (wood … gold).
    pub resources: [i32; 7],
    /// The players it fires for, a bit per colour (bit 0 red … bit 7 pink).
    pub players: u8,
    /// It fires for a human player. An RoE map stores no such flag (the game reads every event as
    /// a human's), so it writes only `true`.
    pub human: bool,
    /// It fires for a computer player.
    pub computer: bool,
    /// The file's first day: 0 is the game's day 1 (survey §11).
    pub first_day: u16,
    /// Its repeat interval in days; 0 fires once.
    pub interval: u16,
}

pub(super) fn validate(map: &Map, object: &OwnedObject, codes: &Codes) -> Result<(), WriteError> {
    let template = &object.template;
    match object.body {
        OwnedBody::Owner(owner) => {
            let fits = match template.class {
                MINE_OBJECT => template.subclass < ABANDONED_MINE_SUBCLASS,
                LIGHTHOUSE_OBJECT | SHIPYARD_OBJECT => true,
                _ => false,
            };
            if !fits {
                return Err(out_of_range("owned object class", template.class));
            }
            if let Some(owner) = owner
                && map
                    .players
                    .get(usize::from(owner))
                    .and_then(Option::as_ref)
                    .is_none()
            {
                return Err(out_of_range("object owner (no playable player)", owner));
            }
        }
        OwnedBody::Resources(resources) => {
            let class = template.class;
            if !matches!(class, MINE_OBJECT | ABANDONED_MINE_OBJECT)
                || template.subclass != ABANDONED_MINE_SUBCLASS
            {
                return Err(out_of_range("abandoned mine class", class));
            }
            if class == ABANDONED_MINE_OBJECT {
                codes.refuse("SoD's abandoned mine in an RoE map", class)?;
            }
            if resources == 0 || resources & !ABANDONED_MINE_RESOURCES != 0 {
                return Err(out_of_range("abandoned mine resources", resources));
            }
        }
    }
    Ok(())
}

pub(super) fn write_body(out: &mut Out, body: OwnedBody) {
    match body {
        OwnedBody::Owner(owner) => out.u32(u32::from(owner.unwrap_or(0xff))),
        OwnedBody::Resources(resources) => out.bytes(&[resources, 0, 0, 0]),
    }
}

pub(super) fn validate_event(event: &TimedEvent, codes: &Codes) -> Result<(), WriteError> {
    if !event.human {
        codes.refuse("timed event without human players in an RoE map", 0u8)?;
    }
    Ok(())
}

/// The map's timed events: their count, then each (an RoE map without the human flag).
pub(super) fn write_events(out: &mut Out, events: &[TimedEvent]) -> Result<(), WriteError> {
    out.u32(super::len32(events.len()));
    for event in events {
        out.string(&event.name)?;
        out.string(&event.message)?;
        for amount in event.resources {
            out.bytes(&amount.to_le_bytes());
        }
        out.u8(event.players);
        if out.1 == Format::Sod {
            out.bool(event.human);
        }
        out.bool(event.computer);
        out.u16(event.first_day);
        out.u16(event.interval);
        out.bytes(&[0; 16]);
    }
    Ok(())
}
