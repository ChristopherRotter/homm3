//! A7: a sampled hero. Its level-ups, spells and artifacts are the distribution's approximation
//! of a human main hero (design §4.4), not the game's rules (`h3_heroes::draw_level_up`, E08 T27);
//! its experience is its level's (HER-EXP-002, E08 T27); its battle values are the hero model's
//! (`h3_heroes::derive_battle_hero`, `max_mana`, `war_machines`), as the game derives them
//! (GAM-BAT-001).

use h3_battle::HeroView;
use h3_content::CompiledHeroEffect;
use h3_content::{
    ArtifactClass, ArtifactTypeId, CreatureTypeId, HeroClassId, HeroEffect, HeroTypeId,
    MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId, SpellCategory, SpellId,
};
use h3_contracts::BattleHero;
use h3_heroes::{
    ArtifactInstance, HeroChange, HeroDetails, apply_change, derive_battle_hero, experience_for,
    learn_level, max_mana, war_machines, wearing,
};
use h3_kernel::Rng;

use super::{EncounterError, EncounterTable, Sampler, WEEKS, chance, pick};

/// The artifact classes a hero draws from, in the order of `artifact_from_level`.
const CLASSES: [ArtifactClass; 4] = [
    ArtifactClass::Treasure,
    ArtifactClass::Minor,
    ArtifactClass::Major,
    ArtifactClass::Relic,
];

/// What the hero draws need, resolved once.
#[derive(Debug, Clone)]
pub(super) struct HeroData {
    /// The artifacts a hero may draw, by class: worn in a slot, no combination, scroll, war
    /// machine or artifact that needs a skill.
    artifacts: [Vec<ArtifactTypeId>; 4],
    /// The hero spells a guild may offer, by level (1 to 5): no creature ability and none that only
    /// a grant by name gives.
    spells: [Vec<SpellId>; 5],
    /// The war machines' artifacts.
    war_machines: Vec<ArtifactTypeId>,
    /// The skills whose Basic level commands the Ballista, the First Aid Tent and the Catapult
    /// (Artillery, First Aid, Ballistics: found by their effects), if the ruleset has them.
    machine_skills: [Option<SkillId>; 3],
    /// The spell book's artifact.
    spell_book: ArtifactTypeId,
    /// The primary skills' maximum.
    primary_max: u8,
}

impl HeroData {
    /// The hero data of `table` on `core`.
    pub(super) fn new(core: &Ruleset, table: &EncounterTable) -> Result<Self, EncounterError> {
        let artifact = |key: &str| {
            core.artifact_by_key(key)
                .ok_or_else(|| EncounterError::Unknown {
                    what: "artifact",
                    key: key.to_owned(),
                })
        };
        let war_machines = table
            .war_machines
            .iter()
            .map(|key| artifact(key))
            .collect::<Result<Vec<_>, _>>()?;
        if war_machines
            .iter()
            .any(|&id| core.artifact(id).is_none_or(|a| a.war_machine.is_none()))
        {
            return Err(EncounterError::Rules(
                "a war machine's artifact places none",
            ));
        }
        let artifacts = CLASSES.map(|class| {
            core.artifacts()
                .iter()
                .enumerate()
                .filter(|(_, a)| {
                    a.class == class
                        && a.slot.is_some()
                        && a.war_machine.is_none()
                        && a.needs_skill.is_none()
                        && !a.is_combination()
                        && !a.effects.iter().any(|e| matches!(e, HeroEffect::Scroll))
                })
                .filter_map(|(index, _)| ArtifactTypeId::from_index(index))
                .collect::<Vec<_>>()
        });
        let spells = std::array::from_fn(|index| {
            core.spells()
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    usize::from(s.level) == index + 1
                        && s.category != SpellCategory::Ability
                        && !s.granted_by_name_only
                })
                .filter_map(|(index, _)| SpellId::from_index(index))
                .collect::<Vec<_>>()
        });
        let gives = |test: fn(&CompiledHeroEffect) -> bool| {
            (0..)
                .zip(core.skills())
                .find(|(_, s)| s.effects(1).iter().any(test))
                .and_then(|(i, _)| SkillId::from_index(i))
        };
        let machine_skills = [
            gives(|e| matches!(e, HeroEffect::Artillery { .. })),
            gives(|e| matches!(e, HeroEffect::FirstAid { .. })),
            gives(|e| matches!(e, HeroEffect::Ballistics { .. })),
        ];
        Ok(Self {
            artifacts,
            spells,
            war_machines,
            machine_skills,
            spell_book: artifact(&table.spell_book)?,
            primary_max: core.hero_params().map_or(u8::MAX, |p| p.primary_max),
        })
    }

    /// A hero of `faction` (an index of the sampler's factions) by `week`.
    pub(super) fn details(
        &self,
        sampler: &Sampler,
        rng: &mut Rng,
        faction: usize,
        week: u8,
    ) -> HeroDetails {
        let data = &sampler.factions[faction];
        let (class_id, types) = &data.classes[rng.below(data.classes.len() as u32) as usize];
        let hero_type = types[rng.below(types.len() as u32) as usize];
        self.details_of(sampler, rng, (faction, *class_id, hero_type), week)
    }

    /// A hero of type `hero_type`, of class `class_id` and faction `faction` (an index of the
    /// sampler's factions), by `week`: its level, primary and secondary skills, artifacts and
    /// spells as [`details`](Self::details) draws them.
    pub(super) fn details_of(
        &self,
        sampler: &Sampler,
        rng: &mut Rng,
        (faction, class_id, hero_type): (usize, HeroClassId, HeroTypeId),
        week: u8,
    ) -> HeroDetails {
        let core = sampler.core();
        let table = sampler.table();
        let data = &sampler.factions[faction];
        let w = usize::from(week).min(WEEKS) - 1;
        let spread = table.level_spread;
        let level = (table.hero_level[w] + rng.range_inclusive(0, 2 * u32::from(spread)) as u8)
            .saturating_sub(spread)
            .max(1);
        let (primary, skills) = match (core.hero_class(class_id), core.hero_type(hero_type)) {
            (Some(class), Some(hero)) => {
                let mut primary = class.primary;
                let mut skills = hero.skills.clone();
                for reached in 2..=level {
                    let gains = class.primary_gain[usize::from(reached > 9)];
                    let skill = pick(rng, gains.iter().map(|&g| u32::from(g)));
                    if primary[skill] < self.primary_max {
                        primary[skill] += 1;
                    }
                    level_up_skill(rng, &class.skill_weights, &mut skills);
                }
                (primary, skills)
            }
            _ => ([0; 4], Vec::new()),
        };
        let mut details = HeroDetails {
            hero_type,
            level,
            experience: core
                .hero_params()
                .map_or(0, |params| experience_for(params, level)),
            offered: Default::default(),
            primary,
            skills,
            worn: wearing(core, &[]),
            backpack: Vec::new(),
            spells: core
                .hero_type(hero_type)
                .map(|h| h.spells.clone())
                .unwrap_or_default(),
            mana: 0,
            bonuses: Default::default(),
        };
        let add = |details: &mut HeroDetails, artifact: ArtifactTypeId| {
            apply_change(
                details,
                core,
                HeroChange::AddArtifact(ArtifactInstance::from(artifact)),
            );
        };
        add(&mut details, self.spell_book);
        for &machine in &self.war_machines {
            if chance(rng, table.war_machine) {
                add(&mut details, machine);
            }
        }
        let reached: Vec<usize> = (0..CLASSES.len())
            .filter(|&c| table.artifact_from_level[c] <= level && !self.artifacts[c].is_empty())
            .collect();
        if !reached.is_empty() {
            for _ in 0..level / table.levels_per_artifact {
                let class = &self.artifacts[reached[rng.below(reached.len() as u32) as usize]];
                add(&mut details, class[rng.below(class.len() as u32) as usize]);
            }
        }
        let guild = table.guild[w]
            .min(data.guild_max)
            .min(learn_level(&details, core));
        for spell_level in 1..=usize::from(guild) {
            let mut offered: Vec<SpellId> = self.spells[spell_level - 1]
                .iter()
                .copied()
                .filter(|s| !details.spells.contains(s))
                .collect();
            let n = usize::from(table.spells_per_level[spell_level - 1]).min(offered.len());
            for _ in 0..n {
                let spell = offered.remove(rng.below(offered.len() as u32) as usize);
                details.spells.push(spell);
            }
        }
        details.mana = max_mana(&details, core);
        details
    }

    /// A hero's battle hero (with its maximum spell points, HER-MAN-001) and war machines, as the
    /// game derives them (GAM-BAT-001); a hero the model refuses (never sampled: the tests check
    /// it) fights with its type alone.
    pub(super) fn battle(
        &self,
        core: &Ruleset,
        details: &HeroDetails,
    ) -> (BattleHero, Vec<CreatureTypeId>) {
        let hero = derive_battle_hero(details, core).map_or_else(
            |_| BattleHero::new(details.hero_type),
            |mut hero| {
                hero.magic.max_mana = max_mana(details, core);
                hero
            },
        );
        (hero, war_machines(details, core))
    }
}

impl HeroData {
    /// A guess of the hero `view` shows (`Sampler::guess`): of `candidates` heroes of its type
    /// by weeks drawn by `weeks`, the one whose primary skills are nearest the view's (L1, ties to
    /// the first drawn), then the skill of each war machine the view shows it commands learnt at
    /// Basic where it does not command it (in place of its last skills when its slots are full). `None` for a hero type none
    /// of the sampler's factions has.
    pub(super) fn guess(
        &self,
        sampler: &Sampler,
        rng: &mut Rng,
        view: &HeroView,
        weeks: &[u32],
        candidates: usize,
    ) -> Option<BattleHero> {
        let core = sampler.core();
        let (faction, class) = (0..).zip(&sampler.factions).find_map(|(i, f)| {
            f.classes
                .iter()
                .find(|(_, types)| types.contains(&view.hero_type))
                .map(|&(c, _)| (i, c))
        })?;
        let target = [view.attack, view.defense, view.power, view.knowledge];
        let draw = |rng: &mut Rng| {
            let week = pick(rng, weeks.iter().copied()) as u8 + 1;
            let details = self.details_of(sampler, rng, (faction, class, view.hero_type), week);
            let (hero, _) = self.battle(core, &details);
            let primary = [hero.attack, hero.defense, hero.power, hero.knowledge];
            let distance = primary
                .iter()
                .zip(target)
                .map(|(&a, b)| u32::from(a.abs_diff(b)))
                .sum::<u32>();
            (distance, details)
        };
        let (mut nearest, mut details) = draw(rng);
        for _ in 1..candidates {
            let (distance, other) = draw(rng);
            if distance < nearest {
                (nearest, details) = (distance, other);
            }
        }
        let (hero, _) = self.battle(core, &details);
        let machines = &hero.war_machines;
        let has = [
            machines.ballista.is_some(),
            machines.tent.is_some(),
            machines.catapult.is_some(),
        ];
        let commands = [
            view.commands_ballista,
            view.commands_tent,
            view.commands_catapult,
        ];
        let commanded: Vec<SkillId> = self
            .machine_skills
            .iter()
            .zip(commands)
            .filter_map(|(skill, commanded)| skill.filter(|_| commanded))
            .collect();
        let learn: Vec<SkillId> = self
            .machine_skills
            .iter()
            .zip(commands)
            .zip(has)
            .filter_map(|((skill, commanded), has)| skill.filter(|_| commanded && !has))
            .collect();
        let learnt = !learn.is_empty();
        // Room from the last skills that command no machine the view shows.
        let (kept, others): (Vec<_>, Vec<_>) = details
            .skills
            .iter()
            .filter(|(s, _)| !learn.contains(s))
            .partition(|(s, _)| commanded.contains(s));
        let room = MAX_HERO_SKILLS.saturating_sub(kept.len() + learn.len());
        details.skills = others
            .into_iter()
            .take(room)
            .chain(kept)
            .chain(learn.iter().map(|&s| (s, 1)))
            .collect();
        Some(if learnt {
            self.battle(core, &details).0
        } else {
            hero
        })
    }
}

/// One level-up's secondary skill: a skill drawn by the class's weights among the known ones
/// below Expert and, while a slot is free, the unknown ones; raised a level, or learnt at Basic.
fn level_up_skill(rng: &mut Rng, weights: &[(SkillId, u8)], skills: &mut Vec<(SkillId, u8)>) {
    let weight =
        |&(skill, w): &(SkillId, u8)| match skills.iter().find(|(known, _)| *known == skill) {
            Some(&(_, level)) if level < MAX_SKILL_LEVEL => u32::from(w),
            Some(_) => 0,
            None if skills.len() < MAX_HERO_SKILLS => u32::from(w),
            None => 0,
        };
    if weights.iter().map(weight).sum::<u32>() == 0 {
        return;
    }
    let (skill, _) = weights[pick(
        rng,
        weights.iter().map(weight).collect::<Vec<_>>().into_iter(),
    )];
    match skills.iter_mut().find(|(known, _)| *known == skill) {
        Some((_, level)) => *level += 1,
        None => skills.push((skill, 1)),
    }
}
