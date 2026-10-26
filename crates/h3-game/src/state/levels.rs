//! A hero's levels in the game (E08 T29): what they read that never changes after the build (the
//! scenario's level limit and banned skills, each hero's level seed; GAM-HER-002), where a level's
//! draws come from by the profile (`ADV-QUIRK-9`), and the levels at load (GAM-SET-002). Since E08
//! T30 the levels in play: given one at a time, a level with two offers a pending decision of the
//! hero's owner (GAM-DEC-002). Design: increment 4's note, §11 to §13 (`docs/epics/E08-log.md`,
//! "Increment 4's design note at its closure"); survey `docs/rules/surveys/E08-progression.md`
//! §2.2, §3.2, §4.1.

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;

use h3_content::{LEVEL_UP_FROM_SEED, Ruleset, SkillId};
use h3_contracts::{HeroId, HeroSex, PlayerId};
use h3_heroes::{
    ArmyWorth, Choice, HeroChange, HeroDetails, LevelUp, Offers, OriginalDraws, TerrainShares,
    apply_change_limited, draw_level_up, max_mana, original_choice, pending_levels,
};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{GameEvent, Sink};
use crate::observe::LevelUpView;

/// The lowest and the highest level seed: the original's `random_between(1, 255)` (0x4BFEC2).
const SEEDS: (u32, u32) = (1, 255);

/// What a hero's levels read that never changes after the build (E08 T29), and since E08 T54 each
/// hero's sex the map gave (GAM-BAT-012), which never changes either: kept once per game, so a
/// fork shares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Progression {
    /// The scenario's level limit, 0 none (HER-EXP-003).
    level_limit: u8,
    /// The skills the scenario bans, in id order, each once (HER-LVL-003).
    banned: Box<[SkillId]>,
    /// Each hero's level seed, by id (GAM-HER-002): server-only state (ADR-0017 point 5), never
    /// in an observation or event.
    seeds: Box<[u8]>,
    /// Each hero's sex the map gave, by id (`None`: its type's own; GAM-BAT-012).
    sexes: Box<[Option<HeroSex>]>,
}

impl Progression {
    /// The progression of a game whose scenario has `level_limit` and bans `banned` (in any order,
    /// repeats allowed), with each hero's sex of `sexes` (by hero id) and its level seed drawn
    /// from `draws` in hero id order (GAM-HER-002).
    pub(crate) fn new(
        level_limit: u8,
        banned: &[SkillId],
        sexes: Box<[Option<HeroSex>]>,
        draws: &mut Rng,
    ) -> Self {
        // rule: GAM-HER-002
        let seeds = (0..sexes.len())
            .map(|_| {
                let (low, high) = SEEDS;
                u8::try_from(draws.range_inclusive(low, high)).unwrap_or(u8::MAX)
            })
            .collect();
        // In id order, each once.
        let banned: BTreeSet<SkillId> = banned.iter().copied().collect();
        Self {
            level_limit,
            banned: banned.into_iter().collect(),
            seeds,
            sexes,
        }
    }

    /// The sex the map gave `hero`, if any (GAM-BAT-012).
    pub(crate) fn sex(&self, hero: HeroId) -> Option<HeroSex> {
        self.sexes.get(usize::from(hero.raw())).copied().flatten()
    }

    /// The scenario's level limit, 0 none.
    pub(crate) const fn level_limit(&self) -> u8 {
        self.level_limit
    }

    /// The skills the scenario bans, in id order.
    pub(crate) fn banned(&self) -> &[SkillId] {
        &self.banned
    }

    /// The level seed of `hero` (1 to 255; 1 for a hero the game does not have).
    pub(crate) fn seed(&self, hero: HeroId) -> u8 {
        self.seeds
            .get(usize::from(hero.raw()))
            .copied()
            .unwrap_or(1)
    }

    /// The next level of `hero` with `details`, if one is pending (HER-LVL-001 to HER-LVL-004):
    /// with `ADV-QUIRK-9` on (`quirk.adventure.level_up_from_seed`, `faithful`) drawn by the
    /// original's generator from the hero's level seed and the new level, the same whenever it is
    /// drawn; off (`strict-fixed`) from `stream`, the setup stream at load and the owner's in play
    /// (E08 T30).
    pub(crate) fn draw(
        &self,
        core: &Ruleset,
        hero: HeroId,
        details: &HeroDetails,
        stream: &mut Rng,
    ) -> Option<LevelUp> {
        // rule: GAM-HER-002
        if core.quirks().is_enabled(LEVEL_UP_FROM_SEED) {
            let mut draws = OriginalDraws::new(self.seed(hero), details.level.saturating_add(1));
            draw_level_up(details, core, &self.banned, &mut draws)
        } else {
            draw_level_up(details, core, &self.banned, stream)
        }
    }

    /// A hero's levels at load (GAM-SET-002; the original's `placed_hero_setup` 0x4D8B30 from
    /// 0x4D8E02): the custom `experience` gained by `details` (built at level 1 and experience 0)
    /// under the level limit, then each pending level drawn ([`Self::draw`]) and given, a choice
    /// between two offers made by the original's choice with the setup's army worth and the
    /// map's `shares` (HER-LVL-006; one offer is learned, none raises the primary skill only),
    /// last the spell points full (HER-MAN-001). No events: building a game emits none.
    pub(crate) fn levels_at_load(
        &self,
        core: &Ruleset,
        (hero, details): (HeroId, &mut HeroDetails),
        experience: u32,
        shares: TerrainShares,
        stream: &mut Rng,
    ) {
        // rule: GAM-SET-002
        let limit = self.level_limit;
        apply_change_limited(details, core, limit, HeroChange::GainExperience(experience));
        // Validated details: the parameters are there, and every drawn level is given (so the
        // levels end with the pending ones).
        let pending = core
            .hero_params()
            .map_or(0, |params| pending_levels(details, params));
        for _ in 0..pending {
            let drawn = self.draw(core, hero, details, stream);
            drawn.into_iter().for_each(|level_up| {
                let choice = original_choice(details, core, &level_up, ArmyWorth::SETUP, shares);
                apply_change_limited(details, core, limit, level_up.change(choice));
            });
        }
        details.mana = max_mana(details, core);
    }
}

impl CanonicalHash for Progression {
    /// The level limit; the banned skills (length, then each); the level seeds (length, then
    /// each); since game state hash v14 (E08 T54) the heroes' sexes (length, then each: 0 the
    /// type's own, 1 male, 2 female).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.level_limit);
        h.write_len(self.banned.len());
        for skill in &self.banned {
            skill.hash_into(h);
        }
        h.write_len(self.seeds.len());
        h.write_bytes(&self.seeds);
        h.write_len(self.sexes.len());
        for sex in &self.sexes {
            h.write_u8(match sex {
                None => 0,
                Some(HeroSex::Male) => 1,
                Some(HeroSex::Female) => 2,
            });
        }
    }
}

impl GameState {
    /// Gives `hero`'s pending levels in play (GAM-DEC-002, E08 T30; ADR-0017 point 6), one at a
    /// time, in order: each is drawn when it is given or its decision opens ([`Progression::draw`]:
    /// from the hero's level seed in `faithful`, from the owner's stream in `strict-fixed`;
    /// GAM-HER-002). A level with two offers opens a pending decision of the hero's `owner` after
    /// its open ones, told to it alone (`LevelUpOpened`), and the next level waits for its answer
    /// ([`Self::answer_level_up`]); one with one offer learns it and one with none raises the
    /// primary skill only, at once (`LevelGained`), and the next is drawn. The decision holds
    /// the game until its answer ([`Self::check_held`]), so no other gain reaches the hero
    /// meanwhile.
    // rule: GAM-DEC-002
    pub(crate) fn give_levels<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        hero: (HeroId, PlayerId),
        sink: &mut S,
    ) {
        let open = self.next_level_up(rules, hero, sink);
        open.into_iter()
            .for_each(|decision| self.push_decision(decision));
    }

    /// A level-up's answer (GAM-DEC-002, E08 T30): the drawn `level_up` given with the `choice`n
    /// offer, then the hero's next pending levels as [`Self::give_levels`] gives them; a next one
    /// with two offers takes the answered decision's place `at`, else the decision closes.
    pub(super) fn answer_level_up<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (hero, at): ((HeroId, PlayerId), usize),
        (level_up, choice): (&LevelUp, Choice),
        sink: &mut S,
    ) {
        self.gain_level(rules.core(), hero, level_up, choice, sink);
        match self.next_level_up(rules, hero, sink) {
            Some(decision) => self.replace(at, decision),
            None => self.close(at),
        }
    }

    /// Gives `hero`'s pending levels with one offer or none at once, until one with two offers,
    /// whose decision is returned, told to the owner (`LevelUpOpened`); none when no level is
    /// pending.
    fn next_level_up<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (hero, owner): (HeroId, PlayerId),
        sink: &mut S,
    ) -> Option<Decision> {
        let core = rules.core();
        while let Some(level_up) = self.draw_level(core, hero, owner) {
            if let Offers::Two(first, second) = level_up.offers {
                let view = LevelUpView {
                    hero,
                    level: level_up.level,
                    primary: level_up.primary,
                    offers: [first, second],
                };
                sink.emit(GameEvent::LevelUpOpened {
                    player: owner,
                    hero,
                    level: view.level,
                    primary: view.primary,
                    offers: view.offers,
                });
                return Some(Decision::LevelUp {
                    owner,
                    view,
                    offered: level_up.offered,
                });
            }
            self.gain_level(core, (hero, owner), &level_up, Choice::First, sink);
        }
        None
    }

    /// The next pending level of `hero`, drawn ([`Progression::draw`]) with `owner`'s stream.
    fn draw_level(&mut self, core: &Ruleset, hero: HeroId, owner: PlayerId) -> Option<LevelUp> {
        let stream = &mut self.players[owner.index()].rng;
        let progression = &self.progression;
        self.heroes
            .get(usize::from(hero.raw()))
            .and_then(|details| progression.draw(core, hero, details, stream))
    }

    /// Gives `level_up` to `hero` with the `choice`n offer (HER-LVL-005: `GainLevel`) and tells its
    /// `owner` (`LevelGained`).
    fn gain_level<S: Sink>(
        &mut self,
        core: &Ruleset,
        (hero, owner): (HeroId, PlayerId),
        level_up: &LevelUp,
        choice: Choice,
        sink: &mut S,
    ) {
        self.update_hero(core, hero, level_up.change(choice));
        sink.emit(GameEvent::LevelGained {
            player: owner,
            hero,
            level: level_up.level,
            primary: level_up.primary,
            skill: level_up.learned(choice),
        });
    }

    /// Applies `change` to the details of `hero` in the store (copied on its first write when a
    /// fork shares it), experience under the scenario's level limit (HER-EXP-003, E08 T29).
    pub(super) fn update_hero(&mut self, core: &Ruleset, hero: HeroId, change: HeroChange) {
        let limit = self.progression.level_limit();
        let store = Arc::make_mut(&mut self.heroes);
        // Every hero of a battle or a level-up is in the store (GAM-HER-001).
        store
            .get_mut(usize::from(hero.raw()))
            .into_iter()
            .for_each(|details| apply_change_limited(Arc::make_mut(details), core, limit, change));
    }
}
