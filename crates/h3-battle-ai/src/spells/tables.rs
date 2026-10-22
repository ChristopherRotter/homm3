//! The spell context's threat tables (0x43C040 with its seed 0x43BF20; `docs/formats/
//! installation.md` §7.42, §7.53; BAT-AI-016): per stack of the side, the strongest strike it can
//! meet this turn and how many such strikes, by melee and by shot, and which enemies threaten the
//! side.

use h3_battle::computer;
use h3_battle::{STACK_CAPACITY, Stack, StackId, StackTraits, Status};
use h3_content::ORIGINAL_AI_SLIPS;

use super::View;

/// One stack's entry of a threat table (16 bytes in the original): the strongest strike, its
/// damage, how many strikes and their sum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The enemy of the strongest strike (the first of equals).
    pub(crate) stack: Option<StackId>,
    /// Its damage.
    pub(crate) damage: u64,
    /// How many strikes.
    pub(crate) count: u32,
    /// Their sum.
    pub(crate) sum: u64,
}

impl Entry {
    /// The entry with one more strike of `damage` by `enemy`: the strongest kept unless the new
    /// one is stronger.
    fn add(&mut self, enemy: StackId, damage: u64) {
        self.count += 1;
        self.sum = self.sum.saturating_add(damage);
        if damage > self.damage {
            self.damage = damage;
            self.stack = Some(enemy);
        }
    }
}

/// The threat tables of a side (+0x50, +0x190, +0x2D0 of the context, by stack id) and its masks
/// (+0x14, +0x18, by the enemy's stack id bit).
#[derive(Debug, Clone)]
pub(crate) struct Tables {
    /// The melee strikes: the retaliation each stack meets at its target, then the enemies'.
    pub(crate) melee: [Entry; STACK_CAPACITY],
    /// The shots.
    pub(crate) shots: [Entry; STACK_CAPACITY],
    /// The worse of the two: the shots' when their strongest is stronger.
    pub(crate) worse: [Entry; STACK_CAPACITY],
    /// The enemies that can strike one of the side's stacks this turn.
    pub(crate) threatens: u64,
    /// Those of them that strike in melee.
    pub(crate) melee_threatens: u64,
}

/// Whether stack `s` can act for the tables (0x43BF20, 0x43C040): alive, not blind, turned to
/// stone or paralysed, no First Aid Tent or Ammo Cart, not hypnotized.
fn acts(s: &Stack) -> bool {
    s.is_alive()
        && !s.spells.status.contains(Status::NO_TURNS)
        && !s.has(StackTraits::FIRST_AID)
        && !s.has(StackTraits::AMMO_SUPPLY)
        && !s.spells.status.contains(Status::HYPNOTIZED)
}

impl<'s> View<'s> {
    /// The side's threat tables (0x43C040; BAT-AI-016), built the first time they are asked.
    pub(crate) fn tables(&self) -> &'s Tables {
        self.half
            .tables
            .get_or_init(|| self.on(self.root.state).fill())
    }

    /// The seed of stack `r`'s melee entry (0x43BF20): when `r` can act ([`acts`]), cannot shoot
    /// now and attacks its threat record's enemy `e` within a turn, `e`'s strike on it by its
    /// whole count, uncapped, without a run-up: the retaliation it meets at its target.
    fn seed(&self, r: StackId) -> Entry {
        let (state, ruleset) = (self.state, self.ruleset);
        let s = self.stack(r);
        let enemy = (acts(s) && !computer::shoots(state, ruleset, r))
            .then(|| self.within_a_turn(r))
            .flatten();
        enemy.map_or_else(Entry::default, |e| {
            let count = u64::from(self.stack(e).count);
            let d = computer::uncapped_damage(state, ruleset, (e, r), count, false, 0);
            Entry {
                stack: Some(e),
                damage: d,
                count: 1,
                sum: d,
            }
        })
    }

    /// The tables (0x43C040 with 0x43BF20; BAT-AI-016). Each stack's melee entry starts with its
    /// seed ([`Self::seed`]); with BAT-QUIRK-25 every stack's with the seed of the side's first
    /// stack, alive or not (survey slip 23: the seed's loop never moves to the next record).
    /// Then for each stack of the side but the dead and the turrets, over each stack of the
    /// other side that can act ([`acts`], no turret) with hit points left after the planned
    /// damage: one that can shoot now adds its shot (its whole count, uncapped) to the shots and
    /// its bit to the threats; another, not the stack's seeded enemy and with the stack in its
    /// threat record's one-turn mask, adds its strike (uncapped, no run-up) to the melee strikes
    /// and its bit to both masks. Last, each stack's worse entry: the melee one, or the shots'
    /// when their strongest is stronger.
    // rule: BAT-AI-016
    fn fill(&self) -> Tables {
        let (state, ruleset) = (self.state, self.ruleset);
        let mut t = Tables {
            melee: [Entry::default(); STACK_CAPACITY],
            shots: [Entry::default(); STACK_CAPACITY],
            worse: [Entry::default(); STACK_CAPACITY],
            threatens: 0,
            melee_threatens: 0,
        };
        let own = || {
            (0u8..)
                .zip(state.stacks())
                .filter(|(_, s)| s.side == self.side)
                .map(|(i, _)| StackId(i))
        };
        let first = own().next();
        let slip = self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
        // Survey slip 23.
        let first_seed = first.filter(|_| slip).map(|f| self.seed(f));
        for r in own() {
            t.melee[r.index()] = first_seed.unwrap_or_else(|| self.seed(r));
        }
        for r in own() {
            let s = self.stack(r);
            if !s.is_alive() || s.is_turret() {
                continue;
            }
            for (j, e) in (0u8..).zip(state.stacks()) {
                let enemy = StackId(j);
                if e.side == self.side
                    || !acts(e)
                    || e.is_turret()
                    || self.planned.left(state, enemy) == 0
                {
                    continue;
                }
                let count = u64::from(e.count);
                if computer::shoots(state, ruleset, enemy) {
                    let d = computer::uncapped_damage(state, ruleset, (enemy, r), count, true, 0);
                    t.shots[r.index()].add(enemy, d);
                    t.threatens |= 1 << j;
                    continue;
                }
                let reaches = self
                    .threat(enemy)
                    .is_some_and(|th| th.mask & (1 << r.index()) != 0);
                if t.melee[r.index()].stack == Some(enemy) || !reaches {
                    continue;
                }
                let d = computer::uncapped_damage(state, ruleset, (enemy, r), count, false, 0);
                t.melee[r.index()].add(enemy, d);
                t.melee_threatens |= 1 << j;
                t.threatens |= 1 << j;
            }
        }
        for r in own() {
            let (melee, shots) = (t.melee[r.index()], t.shots[r.index()]);
            t.worse[r.index()] = if shots.damage > melee.damage {
                shots
            } else {
                melee
            };
        }
        t
    }
}
