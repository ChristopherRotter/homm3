//! The heroes beside the battlefield and their flags over time (E17 M3 T3a;
//! `docs/epics/E17-design.md` §4.5, §7.14). Each runs on the battle's idle steps (0x4960D0), never
//! while a command plays:
//!
//! - **The flags** wave: each step shows the next of group 0's frames, the left one from frame 0,
//!   the right one from 3 (`battle_start_setup` 0x463F29), so the opening's step shows 1 and 4.
//! - **A hero** stands (group 0, frame 0, `battle_load_heroes` 0x463370). At the first step more
//!   than 4 500 ms after its idle clock (0x4965AB) it starts its idle motion (group 1) at frame 0,
//!   and each later step shows the next frame; after the last it stands again and its clock starts
//!   then. The clock also starts with the battle (`battle_start_idle_clocks` 0x479F43) and after
//!   every action (the action dispatcher), as the stacks' fidget clocks do.
//! - **Defeat and victory** (groups 2 and 3): when a stack dies (`stack_death` 0x4441E0), a draw of
//!   1 to 100 below 60 asks its side's hero for defeat, else one below 80 the other hero for
//!   victory, from the rules' generator; a standing hero takes the request at the next step, once
//!   per round each (`round_start` 0x4760DF clears requests and the once), but only while the
//!   side whose turn it is has a human player (0x4CE970), else the requests lapse. The viewer
//!   draws them from a generator of its own (as the stacks' fidgets, `anim::Fidgets`), or none:
//!   the reference checks take the original's poses from its memory.
//!
//! Not read: Berserk's and Chain Lightning's strikes on a stack of the own side (0x422391,
//! 0x4457FC, 0x5A6545 ask for defeat too), a clone's death (none asked), the cast (group 4,
//! `cast_spell` 0x5A01D5; E17 M3 T7).

use alloc::collections::BTreeMap;

use h3_protocol::wire::Side;

use crate::anim::Timeline;
use crate::tables::HeroGroups;

/// The flags' frames at the battle's start, the left one's and the right one's (0x463F29).
pub const FLAG_START: [u32; 2] = [0, 3];
/// How long a hero stands before its idle motion, ms (0x4965AB: more than this).
const IDLE_AFTER: i64 = 4_500;
/// A dying stack's draws (0x4441E5, 0x44420E): below these of 1 to 100.
const DEFEAT_BELOW: u32 = 60;
const VICTORY_BELOW: u32 = 80;

/// A side's hero as drawn: the frames of its DEF's groups and of its flag's group 0.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeroLook {
    /// The frames of each group of the hero's DEF.
    pub frames: BTreeMap<u32, u32>,
    /// The frames of the flag's group 0.
    pub flag_frames: u32,
}

/// What a hero and its flag show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroPose {
    /// The hero's DEF group.
    pub group: u32,
    /// The frame in it.
    pub frame: u32,
    /// The flag's frame in its group 0.
    pub flag: u32,
}

/// What an action did that the heroes answer, in order (E17 M3 T3a): a stack of a side died, a
/// round started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Happening {
    /// A stack of this side died.
    Died(Side),
    /// A round started.
    Round,
}

/// An action: when it started and ended (the heroes' idle clocks start again then), what it did,
/// and what its start does to the stacks the mouse or a fidget left in group 1 (E17 M3 T6c-2).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Action {
    /// Its start (`action_dispatcher` 0x478D80's head).
    pub start: u32,
    /// Its end.
    pub end: u32,
    /// What it did, in order.
    pub happenings: alloc::vec::Vec<Happening>,
    /// Its start stands every stack showing group 1 (`battle_stand_fidgets` 0x479DE0: a walk, a
    /// strike, a shot, a cast, a creature's cast, the Catapult's shot).
    pub stands: bool,
    /// Its start unmarks the mouse's stack (`battle_clear_highlight` 0x477E10: a walk, a strike,
    /// a shot, a cast, the Catapult's shot).
    pub unmarks: bool,
}

/// The battle's heroes: each side's look (none: no hero), whose players are human, the groups,
/// and the viewer's generator for defeat and victory (none: no reactions of its own).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heroes {
    looks: [Option<HeroLook>; 2],
    human: [bool; 2],
    groups: HeroGroups,
    seed: Option<u32>,
}

/// One hero's state (manager +0x53E4 group, +0x53EC frame, +0x53FC clock; +0x53DC, +0x53DE the
/// requests and +0x53E0, +0x53E2 their once per round).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    group: u32,
    frame: u32,
    clock: i64,
    defeat: bool,
    victory: bool,
    defeated: bool,
    won: bool,
}

impl Heroes {
    /// The heroes of a battle: the attacker's and the defender's looks (none: that side has no
    /// hero), whether each side's player is human, the DEF groups, and a seed for the viewer's
    /// own reactions (none: no reactions).
    #[must_use]
    pub fn new(
        looks: [Option<HeroLook>; 2],
        human: [bool; 2],
        groups: HeroGroups,
        seed: Option<u32>,
    ) -> Self {
        Self {
            looks,
            human,
            groups,
            seed,
        }
    }

    /// Whether a side has a hero.
    #[must_use]
    pub fn any(&self) -> bool {
        self.looks.iter().any(Option::is_some)
    }

    /// What each side's hero and flag show at `t` of `timeline` (none: no hero).
    #[must_use]
    pub fn at(&self, timeline: &Timeline, t: u32) -> [Option<HeroPose>; 2] {
        let standing = u32::from(self.groups.standing);
        let mut states = [State {
            group: standing,
            ..State::default()
        }; 2];
        let mut rng = self.seed;
        let mut actions = timeline.actions.iter().peekable();
        let mut steps = 0u32;
        let mut step = |now: u32| {
            while let Some(action) = actions.next_if(|a| a.end <= now) {
                self.act(&mut states, action, &mut rng);
            }
            let human = timeline
                .active(now)
                .and_then(|stack| timeline.members.iter().find(|m| m.stack == stack))
                .is_some_and(|m| self.human[usize::from(m.side == Side::Defender)]);
            for (state, look) in states.iter_mut().zip(&self.looks) {
                if let Some(look) = look {
                    self.step(state, look, now, human);
                }
            }
            steps += 1;
        };
        for &now in timeline.steps.iter().filter(|s| **s <= t) {
            step(now);
        }
        let mut idle = timeline.idle;
        idle.run(t, step);
        let mut out = [None; 2];
        for (side, ((out, look), state)) in out.iter_mut().zip(&self.looks).zip(&states).enumerate()
        {
            if let Some(look) = look {
                *out = Some(HeroPose {
                    group: state.group,
                    frame: state.frame,
                    flag: (FLAG_START[side] + steps) % look.flag_frames.max(1),
                });
            }
        }
        out
    }

    /// An action ended: the clocks start again; its deaths ask for reactions (with the viewer's
    /// generator), a round start clears them.
    fn act(&self, states: &mut [State; 2], action: &Action, rng: &mut Option<u32>) {
        for state in states.iter_mut() {
            state.clock = i64::from(action.end);
        }
        for happening in &action.happenings {
            match *happening {
                Happening::Died(side) => {
                    let Some(seed) = rng.as_mut() else {
                        continue;
                    };
                    let own = usize::from(side == Side::Defender);
                    if between(seed, 1, 100) < DEFEAT_BELOW {
                        states[own].defeat = true;
                    } else if between(seed, 1, 100) < VICTORY_BELOW {
                        states[1 - own].victory = true;
                    }
                }
                Happening::Round => {
                    for state in states.iter_mut() {
                        state.defeat = false;
                        state.victory = false;
                        state.defeated = false;
                        state.won = false;
                    }
                }
            }
        }
    }

    /// One idle step at `now` for a hero (0x49642B to 0x4967B3); `human`: the side whose turn it
    /// is has a human player.
    fn step(&self, s: &mut State, look: &HeroLook, now: u32, human: bool) {
        let g = self.groups;
        let count = |group: u32| look.frames.get(&group).copied().unwrap_or(0);
        let playing = [g.idle, g.defeat, g.victory].map(u32::from);
        let (mut next, mut dirty) = (None, playing.contains(&s.group));
        if s.group == u32::from(g.standing) {
            let asked = if !s.defeated && s.defeat {
                Some((g.defeat, true))
            } else if !s.won && s.victory {
                Some((g.victory, false))
            } else {
                None
            };
            match asked {
                Some((group, defeat)) => {
                    s.defeat = false;
                    s.victory = false;
                    if human {
                        if defeat {
                            s.defeated = true;
                        } else {
                            s.won = true;
                        }
                        if count(u32::from(group)) > 0 {
                            (next, dirty) = (Some(u32::from(group)), true);
                        }
                    }
                }
                None if i64::from(now) - s.clock > IDLE_AFTER => {
                    (next, dirty) = (Some(u32::from(g.idle)), true);
                }
                None => {}
            }
        }
        if !dirty {
            return;
        }
        match next {
            Some(group) => (s.group, s.frame) = (group, 0),
            None => {
                s.frame += 1;
                if s.frame >= count(s.group) {
                    (s.group, s.frame) = (u32::from(g.standing), 0);
                    s.clock = i64::from(now);
                }
            }
        }
    }
}

/// The C library's `rand()` (as `anim::Fidgets`): a draw from `low` to `high`.
fn between(seed: &mut u32, low: u32, high: u32) -> u32 {
    *seed = seed.wrapping_mul(214_013).wrapping_add(2_531_011);
    low + ((*seed >> 16) & 0x7FFF) % (high - low + 1)
}

#[cfg(test)]
mod tests;
