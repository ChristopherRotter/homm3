//! A search agent's worlds (E06 T8, ADR-0025 point 12a, `docs/epics/E06-design.md` §14): copies of
//! a battle in which everything a side may not know ([`observe_as`](crate::observe_as)'s list) is
//! replaced from a seed and a guess of the other side's hero, the rest as it is. Trusted tooling
//! beside `observe`: a change of what a side may not know changes this file in the same change.
//!
//! The other side's hero keeps its public part ([`HeroView`]: type, primary skills, mana, the
//! round's casts, the war machines it commands, whether it carries a book) and whatever the
//! battle shows of it (no retreat, its spell-cost change); its private part comes from the
//! guess. Its stream is re-seeded, its hidden spell obstacles are placed again by their rule, a
//! spell a random caster the viewer does not control drew is drawn again, its Escape Tunnel
//! and its retreat facts are what the viewer may know of them (none). The templates of the
//! stacks its spells would create follow the guessed hero.
//!
//! Known gaps: the state keeps no list of the spells a hero cast, so a guessed book may lack
//! one the hero cast before; whether its opening spells are still due follows the public turns
//! (a side's first stack activated outside the tactics phase); the number and value of its
//! hidden obstacles stay (the cast is public, the hexes are not; their number follows the
//! caster's mastery).

use alloc::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleSide, RetreatFacts};
use h3_kernel::{Rng, StreamId};

use crate::error::SetupError;
use crate::event::NoopSink;
use crate::hero::{self, HeroView};
use crate::magic::MagicLimits;
use crate::state::{self, BattleState, StackFlags};
use crate::{casters, obstacles};

impl BattleState {
    /// A world of the battle for `viewer` (ADR-0025 point 12a; the module's documentation): its
    /// stream from `seed`, the other side's hero's private part from `guess` (`None`: a hero of
    /// its type alone), whose public part is taken from the battle. It reads nothing `viewer` may
    /// not know: two battles that differ only there give the same world for a seed, and
    /// `viewer`'s observation of it is its observation of the battle. `ruleset` must be the
    /// battle's.
    ///
    /// # Errors
    /// [`SetupError::InvalidHero`] for a guess the battle refuses (BAT-HER-001).
    pub fn determinized(
        &self,
        ruleset: &BattleRuleset,
        viewer: BattleSide,
        guess: Option<&BattleHero>,
        seed: u64,
    ) -> Result<Self, SetupError> {
        let mut world = self.clone();
        world.reseed(seed);
        let other = viewer.opponent();
        // rule: BAT-SIE-024
        world.siege = world.siege.map(|s| s.seen_by(Some(viewer)));
        world.retreat[other.index()] = RetreatFacts::default();
        if let Some(record) = self.heroes[other.index()] {
            let hero = public(guess, &record.view);
            let mut guessed = hero::check(ruleset, other, &hero, 0)?;
            guessed.view = record.view;
            guessed.no_retreat = record.no_retreat;
            guessed.magic.cost_change = record.magic.cost_change;
            // rule: BAT-MAG-068
            guessed.magic.opening_due &= !first_turn_taken(self, other);
            world.heroes[other.index()] = Some(guessed);
            world.limits = MagicLimits::of(
                &world.heroes,
                world.battlefield.and_then(|b| ruleset.battlefield(b)),
                world.anti_magic,
            );
            let templates = state::world_templates(&world, ruleset, other, (&hero, &guessed));
            world.created.set_templates(other, templates);
        }
        // rule: BAT-MAG-031
        obstacles::place_unseen_again(&mut world, ruleset, viewer);
        // rule: BAT-MAG-049
        if let Some((drawer, _)) = world
            .casts
            .drawn
            .filter(|&(s, _)| world.at(s).controller() != viewer)
        {
            casters::activated(&mut world, ruleset, drawer, &mut NoopSink);
        }
        Ok(world)
    }

    /// Re-seeds the battle's stream from `seed`, as a new battle's (stream `BATTLE`): a world's
    /// simulations each draw their own rolls (E06 T8).
    pub fn reseed(&mut self, seed: u64) {
        self.rng = Rng::for_stream(seed, StreamId::BATTLE);
    }
}

/// `guess` (or a hero of the view's type alone) with the public part `view` shows: its primary
/// skills, mana, sex, its book only if it carries one, and no control of a war machine it does
/// not command.
fn public(guess: Option<&BattleHero>, view: &HeroView) -> BattleHero {
    let mut hero = guess
        .cloned()
        .unwrap_or_else(|| BattleHero::new(view.hero_type));
    hero.hero_type = view.hero_type;
    (hero.attack, hero.defense, hero.power, hero.knowledge) =
        (view.attack, view.defense, view.power, view.knowledge);
    (hero.magic.mana, hero.magic.max_mana) = (view.mana, view.max_mana);
    hero.sex = view.sex;
    if !view.spell_book {
        hero.magic.spell_book = None;
    } else if hero.magic.spell_book.is_none() {
        hero.magic.spell_book = Some(Vec::new());
    }
    let machines = &mut hero.war_machines;
    if !view.commands_ballista {
        machines.ballista = None;
    }
    if !view.commands_tent {
        machines.tent = None;
    }
    if !view.commands_catapult {
        machines.catapult = None;
    }
    hero
}

/// Whether `side` has had its first turn outside the tactics phase (when its hero casts its
/// opening spells, BAT-MAG-068), as the public turns show it: past the first round, or a stack it
/// controls started its turn in the first.
fn first_turn_taken(state: &BattleState, side: BattleSide) -> bool {
    state.tactics.is_none()
        && (state.round > 1
            || state
                .stacks
                .iter()
                .any(|s| s.controller() == side && s.flags.contains(StackFlags::STARTED)))
}
