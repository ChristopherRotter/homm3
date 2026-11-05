//! The original's computer's decision taken apart (E17 M6 T5b): the parts [`crate::unchecked`]
//! chains, each asked alone, on the battle or on the side's world ([`Sight`]). The original asks
//! them apart for a human's side under Auto Combat (E17 design §7.22, §12.13): the hero's cast at
//! each stack's turn start only with the option Combat Auto Spells (0x422E42; on the turns the
//! human plays too), the stack's action only for a stack that plays itself (0x474BF0), and on a
//! yes of its retreat test a question to the human instead of the retreat (0x477F56). Chained
//! (the cast; else Retreat on the test's yes; else the action) they give [`crate::choose`]'s
//! command with [`Sight::AllSeeing`] and [`crate::choose_fair`]'s with [`Sight::Fair`], and leave
//! the stream where those leave it (the property tests).

use h3_battle::{BattleSide, BattleState, Command, StackTraits, validate};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;

use crate::{acting, checked, hero, retreat, stack, tactics};

/// Which battle the computer decides on (ADR-0027 point 4 as amended).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sight {
    /// Its side's world of the battle, as the fair system actor ([`crate::choose_fair`];
    /// BAT-AI-025): everything the side may not know replaced.
    #[default]
    Fair,
    /// The whole battle, as the original's computer ([`crate::choose`]).
    AllSeeing,
}

/// One decision of the original's computer for a side, asked part by part (E17 M6 T5b): the
/// battle it decides on is fixed when the decision is made, so the parts of one decision see the
/// same world; each part draws from the caller's stream as [`crate::unchecked`] does at that
/// point.
#[derive(Debug, Clone)]
pub struct Decision<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    /// The side's world ([`Sight::Fair`]), else none: the battle itself.
    world: Option<BattleState>,
}

impl<'a> Decision<'a> {
    /// The decision for `side` in `state` on the battle `sight` names. The fair world's seed is
    /// drawn from a copy of `rng`, as [`crate::choose_fair`] draws it. None for a world the
    /// battle refuses ([`crate::choose_fair`]'s Defend; none is known).
    pub fn new(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        side: BattleSide,
        sight: Sight,
        rng: &Rng,
    ) -> Option<Self> {
        let world = match sight {
            Sight::AllSeeing => Ok(None),
            Sight::Fair => state
                .determinized(ruleset, side, None, rng.clone().next_u64())
                .map(Some),
        };
        world.ok().map(|world| Self {
            state,
            ruleset,
            side,
            world,
        })
    }

    /// The battle the decision is made on.
    fn seen(&self) -> &BattleState {
        self.world.as_ref().unwrap_or(self.state)
    }

    /// The hero's cast at the active stack's turn (0x422DA0; BAT-AI-017, [`hero`]'s): none when
    /// `side` is not to act, in the tactics phase, at a war machine's turn, when the hero cast
    /// this round, cannot cast or decides not to, or when the engine refuses the cast (none is
    /// known). The original asks it of a human's side only under Auto Combat with Combat Auto
    /// Spells set: the caller's test.
    // rule: BAT-AI-017
    pub fn hero_cast(&self, rng: &mut Rng) -> Option<Command> {
        let seen = self.seen();
        acting(seen, self.side)
            .filter(|_| seen.tactics().is_none())
            .and_then(|id| hero::cast(seen, self.ruleset, (id, self.side), rng))
            .filter(|&cast| validate(self.state, self.ruleset, self.side, cast).is_ok())
    }

    /// The retreat test before the active stack's action (0x477EE0's 0x41E570; BAT-AI-024,
    /// [`retreat::test`]): never when `side` is not to act, in the tactics phase, or at a
    /// commanded Catapult's or First Aid Tent's turn (their own turns ask none). A yes is the
    /// computer's Retreat; the original asks a human's side instead (0x477F56): the caller's.
    // rule: BAT-AI-024
    pub fn retreat_test(&self, rng: &mut Rng) -> bool {
        let seen = self.seen();
        acting(seen, self.side)
            .filter(|_| seen.tactics().is_none())
            .is_some_and(|id| {
                let me = &seen.stacks()[id.index()];
                !me.is_catapult() && !me.has(StackTraits::FIRST_AID)
            })
            && retreat::test(seen, self.ruleset, self.side, rng, false).retreat
    }

    /// The active stack's action without the hero's cast and the retreat test (0x4221F0 and the
    /// machines' turns; BAT-AI-001): in the tactics phase the phase's turn (BAT-AI-023), else the
    /// Catapult's shot, the Tent's heal, a shooter's shot or a walker's turn, checked as
    /// [`crate::choose`]'s (Defend for a side not to act).
    // rule: BAT-AI-001
    pub fn action(&self, rng: &mut Rng) -> Command {
        let seen = self.seen();
        let command = match (acting(seen, self.side), seen.tactics()) {
            (None, _) => Command::Defend,
            (Some(id), Some(phase)) => {
                tactics::turn(seen, self.ruleset, (id, self.side, phase), rng)
            }
            (Some(id), None) => stack(seen, self.ruleset, (id, self.side), rng, false),
        };
        checked(self.state, self.ruleset, self.side, command)
    }
}
