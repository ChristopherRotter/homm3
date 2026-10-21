//! A hero's levels in the game (E08 T29): what they read that never changes after the build (the
//! scenario's level limit and banned skills, each hero's level seed; GAM-HER-002), where a level's
//! draws come from by the profile (`ADV-QUIRK-9`), and the levels at load (GAM-SET-002). Design
//! `docs/epics/E08-design.md` §11 and §12; survey `docs/rules/surveys/E08-progression.md` §2.2,
//! §3.2.

use alloc::boxed::Box;
use alloc::vec::Vec;

use h3_content::{LEVEL_UP_FROM_SEED, Ruleset, SkillId};
use h3_contracts::HeroId;
use h3_heroes::{
    ArmyWorth, HeroChange, HeroDetails, LevelUp, OriginalDraws, TerrainShares,
    apply_change_limited, draw_level_up, max_mana, original_choice, pending_levels,
};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

/// The lowest and the highest level seed: the original's `random_between(1, 255)` (0x4BFEC2).
const SEEDS: (u32, u32) = (1, 255);

/// What a hero's levels read that never changes after the build (E08 T29): kept once per game,
/// so a fork shares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Progression {
    /// The scenario's level limit, 0 none (HER-EXP-003).
    level_limit: u8,
    /// The skills the scenario bans, in id order, each once (HER-LVL-003).
    banned: Box<[SkillId]>,
    /// Each hero's level seed, by id (GAM-HER-002): server-only state (ADR-0017 point 5), never
    /// in an observation or event.
    seeds: Box<[u8]>,
}

impl Progression {
    /// The progression of a game whose scenario has `level_limit` and bans `banned` (in any order,
    /// repeats allowed), each hero's level seed drawn from `draws` in hero id order, one per hero
    /// of `heroes` (GAM-HER-002).
    pub(crate) fn new(level_limit: u8, banned: &[SkillId], heroes: usize, draws: &mut Rng) -> Self {
        // rule: GAM-HER-002
        let seeds = (0..heroes)
            .map(|_| {
                let (low, high) = SEEDS;
                u8::try_from(draws.range_inclusive(low, high)).unwrap_or(u8::MAX)
            })
            .collect();
        let mut banned: Vec<SkillId> = banned.to_vec();
        banned.sort_unstable();
        banned.dedup();
        Self {
            level_limit,
            banned: banned.into(),
            seeds,
        }
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
    /// drawn; off (`strict-fixed`) from `stream`, the setup stream at load (in play the owner's,
    /// E08 T30).
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
    /// each).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.level_limit);
        h.write_len(self.banned.len());
        for skill in &self.banned {
            skill.hash_into(h);
        }
        h.write_len(self.seeds.len());
        h.write_bytes(&self.seeds);
    }
}
