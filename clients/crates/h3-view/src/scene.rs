//! The scene: from a record's snapshot and the asset tables to the draw list of one frame (E17 M1
//! T7: the battle at time 0, `docs/epics/E17-design.md` §4; animations come with M2).
//!
//! The list names what is drawn (a picture, an obstacle's sprite, a creature's sprite with the
//! shadow its frame holds and its outline, a hero and its flag) in the executable's order; [`crate::compose`] decides
//! how each is drawn, so a later graphics option changes only the compositor (the user,
//! 2026-10-13).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_protocol::wire::{Manifest, Observation, Side, Stack};

use crate::anim::{Magic, Pose, Projectile, Ray, Timeline};
use crate::hero::{FLAG_START, HeroPose};
use crate::layout::BattleLayout;
use crate::tables::{ObstacleArt, Tables};

/// The row passes of the battle's redraw (0x494440, §4.6): for each row, passes 0–7, each over
/// the row's columns left to right.
const PASSES: u8 = 8;
/// The pass that draws the layer-0 obstacle anchored on a hex.
const OBSTACLE_PASS: u8 = 2;
/// A standing stack's draw layer, the pass it is drawn in (+0x8 = 4 from the setup, 0x43D750);
/// a pose names its own (`crate::anim`: walks, strikes, the corpses' pass 1).
const STANDING: u8 = 4;

/// The battle at a moment of its timeline (`crate::anim`): each stack's pose, the active stack,
/// the idle steps taken and the projectile in flight.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Moment {
    /// The stacks' poses, by id; a living stack without one stands on its hex.
    pub poses: BTreeMap<u8, Pose>,
    /// The active stack (its outline).
    pub active: Option<u8>,
    /// The idle steps taken: the outline's colour rotates with them (§4.2).
    pub pulse: u32,
    /// The projectile in flight, drawn over everything (§7.9).
    pub projectile: Option<Projectile>,
    /// The ray drawn so far, over everything (§7.12).
    pub ray: Option<Ray>,
    /// The attacker's and the defender's hero and flag (E17 M3 T3a; none: the hero stands, its
    /// flag at its start).
    pub heroes: [Option<HeroPose>; 2],
    /// The magic animation drawn over stacks (E17 M3 T3b).
    pub magic: Option<Magic>,
    /// The counts the boxes show, by id, of the stacks whose count changed since the snapshot
    /// (E17 M3 T5b); the others show the snapshot's.
    pub counts: BTreeMap<u8, u32>,
}

impl Moment {
    /// The battle at `t` of `timeline` (E17 M2 T4b-2a).
    pub fn at(timeline: &Timeline, t: u32) -> Self {
        Self {
            poses: timeline.at(t),
            active: timeline.outlined(t),
            pulse: timeline.pulse(t),
            projectile: timeline.projectile(t),
            ray: timeline.ray(t),
            heroes: timeline
                .heroes
                .as_ref()
                .map_or([None; 2], |h| h.at(timeline, t)),
            magic: timeline.magic(t),
            counts: timeline.counts(t),
        }
    }
}

/// One thing drawn, in the order of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draw {
    /// The battlefield's picture, opaque, its top-left corner at (`x`, `y`) (§4.4).
    Background {
        /// The picture.
        picture: String,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
    },
    /// The absolute obstacle's picture, index 0 transparent, into the background (§4.4).
    AbsoluteObstacle {
        /// The picture.
        picture: String,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
    },
    /// A usual obstacle's sprite: never mirrored, without a keyed shadow (§4.4).
    Obstacle {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
    },
    /// A projectile's frame, drawn plain over the screen as it stands, runs skipped (§7.9).
    Projectile {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Reflected across its canvas.
        mirrored: bool,
    },
    /// A ray's strokes, each pixel in its colour straight into the screen (§7.12).
    Ray(Ray),
    /// A hero's flag beside the battlefield, in its player's colours, with the shadow its frame
    /// holds (§4.5; E17 M3 T3a).
    Flag {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// The palette of the players' colours.
        palette: String,
        /// The player's colour (0 red to 7 pink): its 32 entries of that palette.
        player: u8,
    },
    /// A hero beside the battlefield, with the shadow its frame holds, no outline (§4.5).
    Hero {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Reflected across its canvas (the right side's).
        mirrored: bool,
    },
    /// A magic animation's frame over a stack, right after its sprite (`draw_battle_stack`'s
    /// tail 0x43E795, §7.15; E17 M3 T3b): group 0, by the plain drawer with runs skipped or the
    /// translucent one (0x4953B0, which clips it to the battlefield, 0x694F18: the panel below,
    /// not drawn yet, is the only part of the screen outside it).
    Magic {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Reflected across its canvas (the stack faces left).
        mirrored: bool,
        /// Drawn half and half with what is under it.
        translucent: bool,
    },
    /// A stack's count box, right after its sprite and any magic animation over it
    /// (`draw_battle_stack`'s part 1 0x43E568, §4.3; E17 M3 T4a): the layout's picture with index
    /// 0 transparent, its inside tinted, the count written over it in the layout's font.
    CountBox {
        /// Whose box it is (the stack's id).
        stack: u8,
        /// The box's left edge.
        x: i32,
        /// Its top edge.
        y: i32,
        /// The count written (`%d` of the stack's +0x50 while it is not −1, else +0x4C: §4.3).
        count: u32,
        /// The tint's hue and saturation, the bits of the f32 values (the layout's without
        /// spells; by the stack's spells not yet).
        tint: [u32; 2],
    },
    /// A creature's sprite with the shadow its frame holds, standing on a hex (§4.2).
    Creature {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// Where it stands: its position hex's anchor.
        anchor: [i32; 2],
        /// It faces left (the defender): drawn mirrored.
        mirrored: bool,
        /// Its outline: the layout's outline palette's entry, none for no outline.
        outline: Option<u8>,
    },
    /// The battle log's item in the panel (0x46BCBE; §9.5; E17 M3 T4b-2b-1): the layout's picture,
    /// then the text in its font over it.
    Log {
        /// The entries it shows, one per line (0x472C30: the first, then a line break and the
        /// next if there is one; the status line's text instead when it shows).
        text: String,
    },
}

/// What the panel below the battlefield draws, after the battlefield (E17 M3 T4b-2b-1): so far its
/// log's item, showing `log` (empty at a battle's start); the panel's picture and buttons come with
/// M3 T6.
#[must_use]
pub fn panel(log: &str) -> Vec<Draw> {
    vec![Draw::Log { text: log.into() }]
}

/// Why a snapshot has no draw list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneError {
    /// A handle the manifest's key tables do not hold: the table and the handle.
    Handle(&'static str, u16),
    /// A content key the asset tables have no entry for.
    Key(String),
    /// An obstacle type drawn the other way: a usual one as the absolute obstacle, or an absolute
    /// one among the usual ones.
    Art(String),
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handle(table, h) => write!(f, "the manifest's {table} have no handle {h}"),
            Self::Key(key) => write!(f, "the asset tables have no entry for `{key}`"),
            Self::Art(key) => write!(f, "`{key}` is not drawn that way"),
        }
    }
}

/// A usual obstacle, placed.
struct Placed {
    def: String,
    before_stacks: bool,
    at: (u8, u8),
    x: i32,
    y: i32,
}

/// The players' colours of the attacker's and the defender's flags: red and blue, as the capture
/// set's maps give them (*provisional*: a battle's record names no player colours, CLT-Q6).
const PLAYERS: [u8; 2] = [0, 1];

/// The background variant 0x41ADB0 returns when no tree, mountain or dead vegetation stands
/// around the battle (§4.4).
const EMPTY_SURROUNDINGS: usize = 2;

/// The draw list of the battle `observation` shows, at time 0: the background with the absolute
/// obstacle, the layer-1 obstacles, each side's flag and hero (E17 M3 T3a), then each row's
/// passes with its layer-0 obstacles and its living stacks, each with its count box (§4.6; M3
/// T4a). A natural obstacle without a type is not drawn; a removed one is no longer in the
/// observation's list (the engine drops it), so every listed one is drawn, whatever hexes a
/// removal cleared. Left out until later tasks: corpses, spell obstacles, the grid.
///
/// # Errors
/// A handle the manifest does not resolve, a key the tables lack, an obstacle type of the wrong
/// kind.
pub fn battle(
    observation: &Observation,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
) -> Result<Vec<Draw>, SceneError> {
    let moment = Moment {
        active: observation.active,
        ..Moment::default()
    };
    battle_at(observation, &moment, manifest, tables, layout)
}

/// The draw list of the battle `observation` shows at `moment` (E17 M2 T4a): as [`battle`],
/// each stack in its pose, facing its way, in its layer's pass of its position's row (T4b-2a:
/// a walk's, a strike's; a dead stack's is the corpses' pass), a flier between its hexes after
/// every row (T5a, `anim::FLYING`); the outline on the moment's active stack in the colour its
/// idle steps rotated to; the projectile last.
///
/// # Errors
/// As [`battle`].
pub fn battle_at(
    observation: &Observation,
    moment: &Moment,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
) -> Result<Vec<Draw>, SceneError> {
    let keys = &manifest.keys;
    let (width, height) = (manifest.field.width, manifest.field.height);
    let at = |hex: u8| (hex / width.max(1), hex % width.max(1));
    let mut list = Vec::new();
    // The background buffer (§4.4): the battlefield's picture, then the absolute obstacle. The
    // picture (0x4646D0, §5): a special battlefield's own (the shore, a special ground's field, a
    // ship's deck, two boats; E17 M3 T2); else the terrain's. Which of a terrain's three pictures
    // a battle gets depends on the adventure map around it (0x41ADB0, §4.4), which a battle's
    // record does not carry: an empty surrounding gives the third, which the reference captures
    // confirm (*provisional*, E17 M2 T1); a field with one picture has it. Water and rock have
    // none (the original fights on water on a deck or between boats only): a key the tables lack.
    let field = match (observation.battlefield, observation.terrain) {
        (Some(handle), _) => Some(key(&keys.battlefields, "battlefields", handle)?),
        (None, Some(handle)) => Some(key(&keys.terrains, "terrains", handle)?),
        (None, None) => None,
    };
    if let Some(key) = field {
        let field = tables
            .battlefields
            .get(key)
            .ok_or_else(|| SceneError::Key(key.to_string()))?;
        let variant = EMPTY_SURROUNDINGS.min(field.backgrounds.len().saturating_sub(1));
        list.push(Draw::Background {
            picture: field.backgrounds[variant].clone(),
            x: 0,
            y: 0,
        });
    }
    if let Some(handle) = observation.absolute_obstacle {
        let (key, art) = obstacle(manifest, tables, handle)?;
        let ObstacleArt::Absolute { pcx, at: point } = art else {
            return Err(SceneError::Art(key));
        };
        list.push(Draw::AbsoluteObstacle {
            picture: pcx.clone(),
            x: i32::from(point[0]),
            y: i32::from(point[1]),
        });
    }
    let mut placed = Vec::new();
    for o in &observation.removable_obstacles {
        let Some(handle) = o.kind else {
            continue;
        };
        let (key, art) = obstacle(manifest, tables, handle)?;
        let ObstacleArt::Usual {
            def,
            rows,
            before_stacks,
        } = art
        else {
            return Err(SceneError::Art(key));
        };
        // The canvas's top-left: the anchor hex's box, raised by the rows above it (§4.4).
        let (row, column) = at(o.anchor);
        let [x, y] = layout.hex_box(row, column);
        placed.push(Placed {
            def: def.clone(),
            before_stacks: *before_stacks,
            at: (row, column),
            x,
            y: y - layout.hex_step[1] * (i32::from(*rows) - 1),
        });
    }
    let obstacle_draw = |o: &Placed| Draw::Obstacle {
        def: o.def.clone(),
        group: 0,
        frame: 0,
        x: o.x,
        y: o.y,
    };
    // Layer-1 obstacles, by anchor: rows top to bottom, the layout's columns left to right.
    let [first, last] = layout.before_stacks_columns;
    for row in 0..height {
        for column in first..=last {
            let here = placed
                .iter()
                .filter(|o| o.before_stacks && o.at == (row, column));
            list.extend(here.map(obstacle_draw));
        }
    }
    // The heroes (§4.5, `battle_redraw` 0x494701, after the siege's parts): each side's flag,
    // then its hero, the left side first; the right hero mirrored in its canvas.
    let standing = u32::from(tables.battle.hero_groups.standing);
    for (side, view) in observation.sides.iter().enumerate() {
        let Some(hero) = &view.hero else {
            continue;
        };
        let key = key(&keys.hero_types, "hero types", hero.hero_type)?;
        let def = tables
            .heroes
            .get(key)
            .ok_or_else(|| SceneError::Key(key.to_string()))?;
        let pose = moment.heroes[side].unwrap_or(HeroPose {
            group: standing,
            frame: 0,
            flag: FLAG_START[side],
        });
        let ([fx, fy], [hx, hy]) = (layout.flag_canvas[side], layout.hero_canvas[side]);
        list.push(Draw::Flag {
            def: tables.battle.flags[side].clone(),
            frame: pose.flag,
            x: fx,
            y: fy,
            palette: layout.player_palette.clone(),
            player: PLAYERS[side],
        });
        list.push(Draw::Hero {
            def: def.clone(),
            group: pose.group,
            frame: pose.frame,
            x: hx,
            y: hy,
            mirrored: side == 1,
        });
    }
    // The hexes a count box cannot take (0x43E605): a living stack's (each of its poses' hexes)
    // while it holds them (not dead since, its box not hidden: E17 M3 T5a), and the blocked ones.
    let mut taken: BTreeSet<u8> = observation.obstacles.iter().copied().collect();
    for s in observation.stacks.iter().filter(|s| s.alive) {
        let pose = pose_of(s, moment, tables);
        if !pose.holds_hex() {
            continue;
        }
        taken.insert(pose.hex);
        if s.two_hex {
            taken.extend(ahead(pose.hex, !pose.mirrored, 0));
        }
    }
    let stacks = observation
        .stacks
        .iter()
        .filter(|s| s.alive || moment.poses.contains_key(&s.id))
        .map(|s| {
            let pose = pose_of(s, moment, tables);
            let sprite = creature(s, pose, moment, manifest, tables, layout, at)?;
            // A magic animation over the stack is drawn right after it (E17 M3 T3b).
            let magic = moment.magic.as_ref().and_then(|m| {
                let &(_, [x, y], mirrored) = m.over.iter().find(|o| o.0 == s.id)?;
                Some(Draw::Magic {
                    def: m.def.clone(),
                    frame: m.frame,
                    x,
                    y,
                    mirrored,
                    translucent: m.translucent,
                })
            });
            let shown = moment.counts.get(&s.id).copied().unwrap_or(s.count);
            let count = count_box(s, pose, shown, &sprite, &taken, layout);
            Ok((sprite.at, pose.layer, [Some(sprite.draw), magic, count]))
        })
        .collect::<Result<Vec<_>, SceneError>>()?;
    for row in 0..height {
        for pass in 0..PASSES {
            for column in 0..width {
                if pass == OBSTACLE_PASS {
                    let here = placed
                        .iter()
                        .filter(|o| !o.before_stacks && o.at == (row, column));
                    list.extend(here.map(obstacle_draw));
                }
                let here = stacks
                    .iter()
                    .filter(|(at, layer, _)| *layer == pass && *at == (row, column));
                list.extend(here.flat_map(|(_, _, draws)| draws.iter().flatten().cloned()));
            }
        }
    }
    // A flier between its hexes: off them, drawn by itself over everything (anim::FLYING).
    list.extend(
        stacks
            .iter()
            .filter(|(_, layer, _)| *layer >= PASSES)
            .flat_map(|(_, _, draws)| draws.iter().flatten().cloned()),
    );
    if let Some(p) = &moment.projectile {
        list.push(Draw::Projectile {
            def: p.def.clone(),
            frame: p.frame,
            x: p.at[0],
            y: p.at[1],
            mirrored: p.mirrored,
        });
    }
    if let Some(ray) = &moment.ray {
        list.push(Draw::Ray(ray.clone()));
    }
    Ok(list)
}

/// A stack's pose at `moment`: its own, else standing on its hex, facing its side's way.
fn pose_of(stack: &Stack, moment: &Moment, tables: &Tables) -> Pose {
    moment.poses.get(&stack.id).copied().unwrap_or(Pose {
        group: u32::from(tables.battle.groups.standing),
        frame: 0,
        hex: stack.hex,
        offset: [0, 0],
        mirrored: stack.side == Side::Defender,
        layer: STANDING,
        boxless: false,
    })
}

/// The hex `1 + extra` ahead of `hex` facing right or left, if there is one.
fn ahead(hex: u8, right: bool, extra: u8) -> Option<u8> {
    let step = 1 + extra;
    if right {
        hex.checked_add(step)
    } else {
        hex.checked_sub(step)
    }
}

/// A stack's count box, if it shows one (§4.3, 0x43E568): a living stack, not a war machine,
/// standing or under the mouse (groups 2 and 1), its box not hidden by a walk or a flight (its
/// +0x30, [`Pose::boxless`]: E17 M3 T5a). Beside its sprite's anchor by its facing, its width and
/// whether the hex ahead is `taken`; its number the count `shown` (E17 M3 T5b: the snapshot's,
/// then each strike's from its end).
fn count_box(
    stack: &Stack,
    pose: Pose,
    shown: u32,
    sprite: &Sprite,
    taken: &BTreeSet<u8>,
    layout: &BattleLayout,
) -> Option<Draw> {
    if !stack.alive || stack.war_machine || !sprite.boxed || pose.boxless {
        return None;
    }
    let right = !pose.mirrored;
    let wide = stack.two_hex;
    let blocked = ahead(pose.hex, right, u8::from(wide)).is_some_and(|h| taken.contains(&h));
    let [x, y] = layout
        .count_box
        .corner(sprite.anchor, right, wide, blocked, sprite.troop_offset);
    Some(Draw::CountBox {
        stack: stack.id,
        x,
        y,
        count: shown,
        tint: layout.count_box.tint,
    })
}

/// A stack's sprite as the scene places it.
struct Sprite {
    /// Its position (row, column).
    at: (u8, u8),
    /// Where it stands: its anchor moved by its pose's offset.
    anchor: [i32; 2],
    /// Its creature's troop-count offset (§4.3).
    troop_offset: i32,
    /// Its pose's group shows a count box: standing or under the mouse (groups 2 and 1).
    boxed: bool,
    /// Its draw.
    draw: Draw,
}

/// A stack's sprite in its pose at `moment`.
fn creature(
    stack: &Stack,
    pose: Pose,
    moment: &Moment,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
    at: impl Fn(u8) -> (u8, u8),
) -> Result<Sprite, SceneError> {
    let key = key(&manifest.keys.creatures, "creatures", stack.creature)?;
    let entry = tables
        .creatures
        .get(key)
        .ok_or_else(|| SceneError::Key(key.to_string()))?;
    // Drawn once, from its position (§4.2): the engine's position is the executable's, a two-hex
    // stack's rear hex (BAT-GEO-002 since PR #150, protocol v23); moved by its pose's offset.
    let (row, column) = at(pose.hex);
    let [x, y] = layout.anchor(row, column);
    let anchor = [x + pose.offset[0], y + pose.offset[1]];
    let draw = Draw::Creature {
        def: entry.def.clone(),
        group: pose.group,
        frame: pose.frame,
        anchor,
        mirrored: pose.mirrored,
        outline: (moment.active == Some(stack.id)).then(|| layout.outline_after(moment.pulse)),
    };
    let groups = tables.battle.groups;
    Ok(Sprite {
        at: (row, column),
        anchor,
        troop_offset: entry.troop_count_offset,
        boxed: [groups.standing, groups.mouse_over]
            .map(u32::from)
            .contains(&pose.group),
        draw,
    })
}

/// The key of `handle` in a manifest's key table.
fn key<'a>(table: &'a [String], name: &'static str, handle: u16) -> Result<&'a str, SceneError> {
    table
        .get(usize::from(handle))
        .map(String::as_str)
        .ok_or(SceneError::Handle(name, handle))
}

/// An obstacle type's key and graphics.
fn obstacle<'t>(
    manifest: &Manifest,
    tables: &'t Tables,
    handle: u16,
) -> Result<(String, &'t ObstacleArt), SceneError> {
    let key = key(&manifest.keys.obstacles, "obstacles", handle)?;
    let art = tables
        .obstacles
        .get(key)
        .ok_or_else(|| SceneError::Key(key.to_string()))?;
    Ok((key.to_string(), art))
}

#[cfg(test)]
pub(crate) mod tests;
