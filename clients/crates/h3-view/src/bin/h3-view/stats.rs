//! A stack's stats dialog in the viewer (E17 M6 T2a1; `docs/epics/E17-design.md` §12.1): what it
//! shows of a stack, from a seat's observation, its rules and the asset tables, for the capture
//! set's stills (`h3_view::stats` the dialog itself). *Provisional* (M6 T2a1's review list): the
//! numbers the executable reads from its own records are the observation's where it carries
//! them, the rest derived here as noted.

use std::collections::BTreeMap;

use h3_content_battle::BattleRuleset;
use h3_kernel::Handle;
use h3_protocol::wire::{Manifest, Observation, Side, Stack};
use h3_ui::text::{Args, Catalogue};
use h3_view::message::MessageBox;
use h3_view::stats::{Effect, Hints, Memory, Modal, Pair, Stats, StatsLayout, Steps, Texts};
use h3_view::tables::Tables;

/// The creatures the dialog tells by number (their original ids): the Ballista's damage, the war
/// machines' steps, the Faerie Dragon's cast button, the arrow tower's shots.
const BALLISTA: u32 = 146;
const ARROW_TOWER: u32 = 149;
const FAERIE_DRAGON: u32 = 134;
const WAR_MACHINES: core::ops::RangeInclusive<u32> = 145..=148;
/// The spells it tells by number (their original ids): those that change morale and luck, and
/// Forgetfulness, which at Advanced stops a shot.
const MIRTH: u32 = 49;
const SORROW: u32 = 50;
const FORTUNE: u32 = 51;
const MISFORTUNE: u32 = 52;
const FORGETFULNESS: u32 = 61;

/// What the dialog needs to name and number a stack: the record's manifest (the handles' keys),
/// the rules, the asset tables and the catalogue.
pub struct Sources<'a> {
    /// The record's manifest.
    pub manifest: &'a Manifest,
    /// The rules the battle is played by.
    pub ruleset: &'a BattleRuleset,
    /// The asset tables.
    pub tables: &'a Tables,
    /// The catalogue.
    pub catalogue: &'a Catalogue,
}

/// The dialog a left press (or the key T) opened on a stack: the modal dialog with its OK's
/// state, status line and cursor (E17 M6 T2a2a), its creature's first steps, and the point the
/// mouse was at (the battle's cursor and status line keep it: §12.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The dialog and what its input did to it.
    pub modal: Modal,
    /// Its creature as the dialog opened.
    pub steps: Steps,
    /// Where the mouse was as it opened.
    pub looked: [i32; 2],
    /// The right button's popups held over it (E17 M6 T2a2b1), each `[start, end]` in ms since
    /// it opened (`u32::MAX` while held): its creature does not step meanwhile
    /// ([`Steps::made_around`]).
    pub holds: Vec<[u32; 2]>,
}

impl Open {
    /// The dialog `since` ms after it opened, its creature's first step `first` ms after it
    /// (`creature_dialog_proc` 0x5F4CFF: once the clock reaches the frame deadline, then every
    /// 100 ms; none while a popup is held, E17 M6 T2a2b1).
    pub fn at(&self, first: u32, since: u32) -> MessageBox {
        let mut steps = self.steps.clone();
        for _ in 0..Steps::made_around(first, &self.holds, since) {
            steps.step();
        }
        let mut window = self.modal.open.message.clone();
        for item in &mut window.dialog.items {
            if let h3_ui::ui::Item::Def(d) = item
                && d.id == h3_view::stats::CREATURE
            {
                (d.group, d.frame) = (steps.group, steps.frame);
            }
        }
        window
    }
}

/// The cursor over the open dialog at the mouse's `point` (E17 M6 T2a2a): the default cursor's
/// frame its moves set (the spell book's cursor DEF, `crdeflt.def`), until then the combat
/// cursor's pointer the press or the key set as it opened (0x476E97, 0x475D72).
pub fn cursor(
    layout: &h3_view::layout::BattleLayout,
    modal: &Modal,
    point: [i32; 2],
) -> h3_view::scene::Draw {
    match modal.cursor {
        Some(frame) => h3_view::scene::Draw::Cursor {
            def: layout.book.cursor.clone(),
            frame,
            x: point[0],
            y: point[1],
            hot: usize::try_from(frame)
                .ok()
                .and_then(|f| layout.book.hot.get(f).copied())
                .unwrap_or_default(),
        },
        None => h3_view::mouse::cursor(layout, layout.mouse.panel_frame, point),
    }
}

/// The original id of a spell handle (none: not in the manifest or the tables).
fn spell_original(sources: &Sources<'_>, handle: u16) -> Option<u32> {
    let key = sources.manifest.keys.spells.get(usize::from(handle))?;
    sources
        .tables
        .spells
        .get(key)
        .map(|s| u32::from(s.original))
}

/// Whether `stack` can shoot now (`stack_can_shoot_now` 0x4428F0): a Ballista or an arrow tower
/// always; else a shooter with shots, no enemy beside it unless it shoots then too, not forgetful
/// at Advanced or Expert.
fn can_shoot_now(
    sources: &Sources<'_>,
    observation: &Observation,
    stack: &Stack,
    original: u32,
) -> bool {
    if original == BALLISTA || original == ARROW_TOWER {
        return true;
    }
    if !stack.shooter || stack.shots == 0 {
        return false;
    }
    let forgetful = stack
        .effects
        .iter()
        .any(|e| spell_original(sources, e.spell) == Some(FORGETFULNESS) && e.mastery >= 2);
    if forgetful {
        return false;
    }
    if stack.shoots_when_adjacent {
        return true;
    }
    let own = hexes(stack);
    let blocked = observation.stacks.iter().any(|other| {
        other.alive
            && other.controller != stack.controller
            && hexes(other)
                .iter()
                .any(|&h| own.iter().any(|&o| beside(o, h)))
    });
    !blocked
}

/// A stack's hexes: its position, and a two-hex stack's other (the attacker's to the right, the
/// defender's to the left).
fn hexes(stack: &Stack) -> Vec<u8> {
    let mut out = vec![stack.hex];
    if stack.two_hex {
        let other = if stack.side == Side::Attacker {
            stack.hex.checked_add(1)
        } else {
            stack.hex.checked_sub(1)
        };
        out.extend(other);
    }
    out
}

/// Whether hexes `a` and `b` are neighbours.
fn beside(a: u8, b: u8) -> bool {
    (0..6).any(|d| h3_view::mouse::neighbour(a, d) == Some(b))
}

/// What the dialog shows of stack `id` in `observation` (`stack_stats_dialog_create` 0x5F3360),
/// `modal` for a left press's; none when the observation, the rules or the tables lack it.
pub fn of(sources: &Sources<'_>, observation: &Observation, id: u8, modal: bool) -> Option<Stats> {
    let stack = observation.stacks.iter().find(|s| s.id == id)?;
    let handle = Handle::from_raw(stack.creature);
    let creature = sources.ruleset.creature(handle)?;
    let core = &**sources.ruleset;
    let info = core.creature(handle)?;
    let key = sources
        .manifest
        .keys
        .creatures
        .get(usize::from(stack.creature))?;
    let entry = sources.tables.creatures.get(key)?;
    let original = u32::from(entry.original);
    let count = i64::from(stack.count);
    let name = sources
        .catalogue
        .text(&format!("creature.{key}"), &Args::new().count(count));
    let ability = sources
        .catalogue
        .text(&format!("creature-ability.{key}"), &Args::new());
    // The attack: the ranged one when it can shoot now, the higher of it and the melee one
    // (0x5F345B); the observation's attack holds its timed effects but the melee and ranged
    // bonuses (Bloodlust, Precision).
    let melee = stack.attack + i32::from(stack.melee_attack_bonus);
    let ranged = stack.attack + i32::from(stack.ranged_attack_bonus);
    let attack = if can_shoot_now(sources, observation, stack, original) {
        ranged.max(melee)
    } else {
        melee
    };
    let controller = usize::from(stack.controller == Side::Defender);
    let hero = observation
        .sides
        .get(controller)
        .and_then(|s| s.hero.as_ref());
    // A Ballista under a hero: each end times its attack, clamped to 0..99, plus 1 (0x5F58FD).
    let times = match hero {
        Some(h) if original == BALLISTA => i32::from(h.attack.min(99)) + 1,
        _ => 1,
    };
    let damage = [stack.damage_min, stack.damage_max].map(|d| i32::from(d) * times);
    // Morale and luck: the stack's own, with what Mirth, Sorrow, Fortune and Misfortune change
    // (the observation's are before them; the executable's +0x4E8 and +0x4EC after).
    let by = |spell: u32| -> i32 {
        stack
            .effects
            .iter()
            .filter(|e| spell_original(sources, e.spell) == Some(spell))
            .map(|e| i32::from(e.value))
            .sum()
    };
    let morale = i32::from(stack.morale) + by(MIRTH) - by(SORROW);
    let luck = i32::from(stack.luck) + by(FORTUNE) - by(MISFORTUNE);
    let effects = stack
        .effects
        .iter()
        .filter_map(|e| {
            let key = sources.manifest.keys.spells.get(usize::from(e.spell))?;
            Some(Effect {
                spell: spell_original(sources, e.spell)?,
                name: sources
                    .catalogue
                    .text(&format!("spell.{key}"), &Args::new()),
                rounds: e.rounds,
            })
        })
        .collect();
    // The cast button (0x5F3974): a Faerie Dragon of the side acting now, with casts left,
    // outside the tactics phase, in the modal dialog.
    let acting = observation
        .active
        .and_then(|a| observation.stacks.iter().find(|s| s.id == a))
        .map(|s| s.controller);
    let cast = modal
        && original == FAERIE_DRAGON
        && acting == Some(stack.controller)
        && stack.casts_left > 0
        && observation.tactics.is_none();
    let war_machine = WAR_MACHINES.contains(&original);
    let speed_now = if stack.war_machine {
        0
    } else {
        i32::from(stack.speed)
    };
    Some(Stats {
        name,
        def: entry.def.clone(),
        town: info.faction.and_then(|f| u8::try_from(f.index()).ok()),
        count: stack.count,
        attack: Pair {
            own: i32::from(creature.attack),
            now: attack,
        },
        defense: Pair {
            own: i32::from(creature.defense),
            now: stack.defense,
        },
        shots: stack.shooter.then(|| Pair {
            own: i32::from(creature.shots),
            now: i32::from(stack.shots),
        }),
        damage,
        health: Pair {
            own: i32::try_from(creature.hit_points).unwrap_or(i32::MAX),
            now: i32::try_from(stack.max_hp).unwrap_or(i32::MAX),
        },
        health_left: i32::try_from(stack.top_hp).unwrap_or(i32::MAX),
        speed: Pair {
            own: i32::from(info.speed),
            now: if war_machine { 0 } else { speed_now },
        },
        morale,
        luck,
        effects,
        ability: Some(ability),
        cast,
        modal,
    })
}

/// The player whose colours the dialog's background takes (0x5F3550): the owner of the hero who
/// controls the stack, else the viewing player (`viewing`).
pub fn player(observation: &Observation, stack: &Stack, viewing: u8) -> u8 {
    let index = usize::from(stack.controller == Side::Defender);
    let side = &observation.sides[index];
    match side.hero {
        Some(_) => side
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0)),
        None => viewing,
    }
}

/// The dialog a left press (or the key T) opens on stack `id` (its place next to the stack's hex,
/// its creature's steps from the frame counts `frames` of its DEF's groups; `seed` the viewer's
/// generator's), the mouse at `looked`; drawn and hinted in `memory`'s profile, which its opening
/// may make forget the item last hovered.
pub fn open(
    sources: &Sources<'_>,
    observation: &Observation,
    (id, looked): (u8, [i32; 2]),
    (layout, anchor): (&StatsLayout, [i32; 2]),
    (frames, viewing, seed): (BTreeMap<u32, u32>, u8, u32),
    memory: &mut Memory,
) -> Option<Open> {
    let mut stats = of(sources, observation, id, true)?;
    // The cast button only for the acting stack in the client's profile (CLT-Q38 (4), answered
    // at E17 M6 T2a2a's start: its cast is the acting stack's own drawn spell); the original's
    // shows it for any Faerie Dragon of the side acting, for the comparisons.
    if memory.bugs() == h3_render::Bugs::Fixed && observation.active != Some(id) {
        stats.cast = false;
    }
    let stack = observation.stacks.iter().find(|s| s.id == id)?;
    let key = sources
        .manifest
        .keys
        .creatures
        .get(usize::from(stack.creature))?;
    let original = sources
        .tables
        .creatures
        .get(key)
        .map(|e| u32::from(e.original));
    let war_machine = original.is_some_and(|o| WAR_MACHINES.contains(&o));
    let texts = Texts::of(sources.catalogue);
    let bugs = memory.bugs();
    let window = h3_view::stats::layout(
        layout,
        &texts,
        &stats,
        (
            h3_view::stats::place(anchor),
            player(observation, stack, viewing),
        ),
        bugs,
    );
    let hints = Hints::of(sources.catalogue, &stats, bugs);
    Some(Open {
        modal: Modal::new(window, hints, memory),
        steps: Steps::new(frames, war_machine, seed),
        looked,
        holds: Vec::new(),
    })
}
