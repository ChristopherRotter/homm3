//! The templates kept and drawn, and the positions (survey `docs/rules/surveys/E14-generator.md`
//! §1.5, §2.1, §3.2.1 and §3.2.2): `rmg_templates_read` 0x537FF0 with `rmg_template_zones_read`
//! 0x538480 keep the templates whose size range and zones fit the settings, `rmg_generate_map`
//! 0x549930 draws one and gives the colours their start positions.

use alloc::vec::Vec;

use crate::part::{Connection, Template, Zone, ZoneKind};
use crate::settings::{COLOURS, MapFormat, Water};

/// The town types' bit of Conflux, the ninth (`rmg_zone` +0x49, column 31).
const CONFLUX: u16 = 1 << 8;

/// The monster towns' bit of Fortress, the eighth (`rmg_zone` +0x9D, column 65).
const FORTRESS: u16 = 1 << 7;

/// What the templates are kept for: the map's size and the counts the generator runs with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fit {
    /// The size index: width × height × levels / 1296, halved for islands (at least 1).
    pub size: i32,
    /// The players (the tab's "human or computer").
    pub players: i32,
    /// The players and the computers.
    pub total: i32,
}

impl Fit {
    /// The fit of a `side` × `side` map of `levels` levels with `water`, `players` and
    /// `computers` (`rmg_templates_read` 0x53801D to 0x538076: the product divided as a signed
    /// 32-bit number, islands halved towards zero and raised to 1).
    // rule: RMG-TPL-005
    pub const fn new(side: i32, levels: i32, water: Water, players: i32, computers: i32) -> Self {
        let mut size = side * side * levels / 1296;
        if matches!(water, Water::Islands) {
            size /= 2;
            if size < 1 {
                size = 1;
            }
        }
        Self {
            size,
            players,
            total: players + computers,
        }
    }

    /// Whether a row's least and most humans and least and most players hold the players and
    /// the total (columns 9 to 12 of a zone, 81 to 84 of a connection; inclusive, as read).
    // rule: RMG-TPL-004
    const fn holds(self, humans: [i32; 2], players: [i32; 2]) -> bool {
        humans[0] <= self.players
            && humans[1] >= self.players
            && players[0] <= self.total
            && players[1] >= self.total
    }

    /// Whether `template` is kept: its range holds the size, and of its zones that fit, the human
    /// starts are at least the players and the human and computer starts at least the total.
    // rule: RMG-TPL-005
    pub fn keeps(self, template: &Template) -> bool {
        if self.size < template.sizes[0] || self.size > template.sizes[1] {
            return false;
        }
        let (mut humans, mut starts) = (0, 0);
        for z in template
            .zones
            .iter()
            .filter(|z| self.holds(z.humans, z.players))
        {
            match z.kind {
                ZoneKind::HumanStart => {
                    humans += 1;
                    starts += 1;
                }
                ZoneKind::ComputerStart => starts += 1,
                ZoneKind::Treasure | ZoneKind::Junction => {}
            }
        }
        humans >= self.players && starts >= self.total
    }
}

/// A connection kept with its template: the two zones' indices among the kept zones and the row as
/// read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeptConnection {
    /// The zones at its ends, the first named first (columns 76 and 77).
    pub zones: [usize; 2],
    /// The row.
    pub row: Connection,
    /// +0xA: set when a way is made (step 6); step 2's connections between two water zones are
    /// made set (no way to make).
    pub created: bool,
}

/// A template as the generator keeps it (RMG-TPL-003, -004): its zones that fit, in row order,
/// their ids renumbered 0, 1, … (a zone's id is its index); the connections between two of
/// them that fit, in row order (the original appends each to both zones' lists, the first zone's
/// first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeptTemplate {
    /// Its index in the rmg part's table.
    pub index: usize,
    /// The zones.
    pub zones: Vec<Zone>,
    /// The connections.
    pub connections: Vec<KeptConnection>,
}

impl KeptTemplate {
    /// The template's zones and connections that `fit` keeps, for `format`: RoE's zones read
    /// eight town types (Conflux's bit cleared) and lose a monster town, Fortress's
    /// (`roe_fortress`, RMG-QUIRK-11's flag) or else Conflux's. A connection's zones are the
    /// first kept zones of its ids as read (`rmg_template_zone_find` 0x5330A0); one naming a
    /// dropped zone is dropped.
    // rule: RMG-TPL-003, RMG-TPL-004
    pub fn new(
        index: usize,
        template: &Template,
        fit: Fit,
        format: MapFormat,
        roe_fortress: bool,
    ) -> Self {
        let fitting: Vec<&Zone> = template
            .zones
            .iter()
            .filter(|z| fit.holds(z.humans, z.players))
            .collect();
        let find = |id: i32| fitting.iter().position(|z| z.id == id);
        let connections = template
            .connections
            .iter()
            .filter_map(|c| {
                let zones = [find(c.zones[0])?, find(c.zones[1])?];
                fit.holds(c.humans, c.players).then(|| KeptConnection {
                    zones,
                    row: c.clone(),
                    created: false,
                })
            })
            .collect();
        let zones = (0i32..)
            .zip(&fitting)
            .map(|(id, z)| {
                let mut zone = (*z).clone();
                zone.id = id;
                if format == MapFormat::RoE {
                    zone.town_types &= !CONFLUX;
                    zone.monster_towns &= if roe_fortress { !FORTRESS } else { !CONFLUX };
                }
                zone
            })
            .collect();
        Self {
            index,
            zones,
            connections,
        }
    }
}

/// The indices of the templates `fit` keeps, in the table's order (`rmg_templates_read`'s list).
pub fn kept_templates(templates: &[Template], fit: Fit) -> Vec<usize> {
    (0..)
        .zip(templates)
        .filter(|(_, t)| fit.keeps(t))
        .map(|(i, _)| i)
        .collect()
}

/// Why the positions cannot be given: the original writes outside its eight-entry arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionError {
    /// A start zone's position (Ownership less 1) outside 0 to 7.
    Outside {
        /// The zone's index among the kept zones.
        zone: usize,
        /// The position.
        position: i32,
    },
    /// The free starts ran out: fewer distinct human starts than the players, or starts than
    /// the players and computers.
    Exhausted,
}

/// The start positions' colours (RMG-TPL-008): the start zones mark their positions (Ownership
/// less 1), a human start as a human start and a start, a computer start as a start; the colours
/// listed humans first, then the others, each in colour order; the first `players` take the
/// human starts' positions in position order, then the next `computers` the remaining starts
/// (human or computer) in position order. Entry `p` is position `p`'s colour (the generator's
/// +0xEE4 + 4p), `None` for no colour (-1).
///
/// A start zone without Ownership has position -1: the original's mark of its start lands on the
/// human starts' eighth entry (`start[-1]` is `human[7]` in `rmg_generate_map`'s frame, 0x5499E5
/// and 0x5499F4), a human start at position 7 that no zone has (RMG-QUIRK-24: with
/// `unowned_seventh`, its flag; without, such a zone marks nothing).
///
/// # Errors
///
/// A [`PositionError`] where the original's arrays would be indexed outside the frame's two
/// eight-entry arrays (RMG-QUIRK-4): a position below -1 or above 7, or the starts running out
/// (its write past them clears the drawn template's index).
// rule: RMG-TPL-008
pub fn positions(
    zones: &[Zone],
    humans: &[bool; COLOURS],
    players: usize,
    computers: usize,
    unowned_seventh: bool,
) -> Result<[Option<u8>; COLOURS], PositionError> {
    let mut human_start = [false; COLOURS];
    let mut start = [false; COLOURS];
    for (zone, z) in zones.iter().enumerate() {
        let human = match z.kind {
            ZoneKind::HumanStart => true,
            ZoneKind::ComputerStart => false,
            ZoneKind::Treasure | ZoneKind::Junction => continue,
        };
        let position = z.ownership.wrapping_sub(1);
        if position == -1 {
            // RMG-QUIRK-24: `start[-1]`, the human starts' last entry.
            human_start[COLOURS - 1] |= unowned_seventh;
            continue;
        }
        let p = usize::try_from(position)
            .ok()
            .filter(|&p| p < COLOURS)
            .ok_or(PositionError::Outside { zone, position })?;
        human_start[p] |= human;
        start[p] = true;
    }
    let colours = (0u8..)
        .zip(humans)
        .filter(|(_, h)| **h)
        .chain((0u8..).zip(humans).filter(|(_, h)| !**h))
        .map(|(c, _)| c);
    let mut colour_of = [None; COLOURS];
    let mut taken = [false; COLOURS];
    let mut colours = colours.take(players + computers);
    let mut next = 0;
    for colour in colours.by_ref().take(players) {
        while next < COLOURS && !human_start[next] {
            next += 1;
        }
        let p = (next < COLOURS)
            .then_some(next)
            .ok_or(PositionError::Exhausted)?;
        taken[p] = true;
        colour_of[p] = Some(colour);
        next += 1;
    }
    next = 0;
    for colour in colours {
        while next < COLOURS && (!start[next] || taken[next]) {
            next += 1;
        }
        let p = (next < COLOURS)
            .then_some(next)
            .ok_or(PositionError::Exhausted)?;
        colour_of[p] = Some(colour);
        next += 1;
    }
    Ok(colour_of)
}
