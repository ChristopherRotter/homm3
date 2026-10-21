//! Batch E4's scorer (E08 T32a, decision 67; `docs/experiments/README.md`): reads a map of
//! `experiments/specs/E08-batch-4a.toml` or `-4b.toml` and its results template as the capture
//! harness fills it in, and predicts from each hero's seed byte (hero +0x8F, drawn by the new game;
//! survey `docs/rules/surveys/E08-progression.md` §2.2, §7) its levels at load and its level-up
//! dialogs under each source's rule, beside what was recorded:
//!
//! - **the reading** (the engine's `faithful`): the original's draws as `h3_heroes` makes them
//!   (D-14), the levels at load chosen by the computer's evaluation (D-20) under the scenario's
//!   level limit (D-21);
//! - **always-left** (the wiki; VCMI takes the first offer too, with draws of its own): the same
//!   draws, the left offer taken at load;
//! - **VCMI's rules on the original's draws**, one per row: the primary skill's row by the old
//!   level (E4-2, D-15), the magic classes' intervals 3 and 3 (E4-3, D-16, A-10), a known school by
//!   its class weight (E4-5, D-17); the levels at load without the limit (E4-6a, D-21);
//! - **drawn up front** (E4-4): every level of one gain drawn from the hero before the battle,
//!   before the first answer.
//!
//! A rule whose prediction equals the reading's says nothing about it (inconclusive). The
//! reading's draws and levels at load are checked against `h3_heroes` and `h3_game` by this
//! example's tests. The rows that need no seed byte (batch E4c, E4-6's experience) are scored by
//! hand from the spec's predictions.
//!
//! `cargo run -p h3-map --example e4_score -- experiments/out/E08-batch-4a.h3m experiments/results/E08-batch-4a.md`

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::ops::Range;
use std::process::ExitCode;
use std::sync::Arc;

use h3_content::{HeroClass, HeroTypeId, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId};
use h3_contracts::HeroId;
use h3_game::{GameRuleset, GameSetup, GameState};
use h3_heroes::{
    ArmyWorth, Choice, Draws, HeroChange, HeroDetails, LevelUp, OfferLevels, Offers, OriginalDraws,
    PrimarySkill, TerrainShares, apply_change_limited, experience_for, level_of, original_choice,
    pending_levels,
};
use h3_map::Limits;
use h3_pack_adventure::{ORIGINAL_PACKS, compile};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [map, template] = args.as_slice() else {
        eprintln!("usage: e4_score <batch map .h3m> <filled results template .md>");
        return ExitCode::FAILURE;
    };
    let run = || -> Result<String, String> {
        let rules = original_rules()?;
        let file = std::fs::read(map).map_err(|e| format!("{map}: {e}"))?;
        let raw = h3_map::read(&file, &Limits::default()).map_err(|e| format!("{map}: {e}"))?;
        let setup = h3_map::import(&raw, &rules).map_err(|e| format!("{map}: {e}"))?;
        let text = std::fs::read_to_string(template).map_err(|e| format!("{template}: {e}"))?;
        let batch = Batch::new(&rules, &setup)?;
        Ok(batch.report(&parse_template(&text)))
    };
    match run() {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("e4_score: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The original packs' rules, `base:faithful` (the reading).
fn original_rules() -> Result<GameRuleset, String> {
    let (core, part) = compile(&ORIGINAL_PACKS, "base:faithful").map_err(|e| format!("{e:?}"))?;
    let packs = [
        h3_pack_battle::BASE_PACK,
        h3_pack_battle::AB_PACK,
        h3_pack_battle::SOD_PACK,
    ];
    let battle = h3_pack_battle::compile(&packs, "base:faithful").map_err(|e| format!("{e:?}"))?;
    GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle)).map_err(|e| format!("{e:?}"))
}

// -- The draws under each rule -----------------------------------------------------------------

/// Which source's rule a level-up is drawn under: the default is the reading (the engine's
/// `h3_heroes::draw_level_up` with the original's draws, `faithful`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Rules {
    /// D-15, VCMI: the primary skill's row chosen by the old level, not the new one.
    row_by_old_level: bool,
    /// D-16, VCMI and HotA: these Wisdom and school intervals instead of the class's.
    intervals: Option<[u8; 2]>,
    /// D-17, VCMI: a known school drawn by its class weight, not 1.
    school_by_class_weight: bool,
}

/// The hero's next level drawn from its seed byte under `rules` (`h3_heroes::draw_level_up`'s
/// steps, `hero_level_up` 0x4DA720 and `hero_secondary_offer` 0x4DAD00, with each rule a switch);
/// `None` when no level is pending or the ruleset lacks the hero's class.
fn draw(
    details: &HeroDetails,
    core: &Ruleset,
    banned: &[SkillId],
    seed: u8,
    rules: Rules,
) -> Option<LevelUp> {
    let params = core.hero_params()?;
    let class = core
        .hero_type(details.hero_type)
        .and_then(|hero| core.hero_class(hero.class))?;
    let level = details
        .level
        .checked_add(1)
        .filter(|&level| level <= level_of(params, details.experience))?;
    let draws = &mut OriginalDraws::new(seed, level);
    let row_level = if rules.row_by_old_level {
        details.level
    } else {
        level
    };
    let row = class.primary_gain[usize::from(row_level >= 10)];
    let mut rest = draws.between(1, 100);
    let primary = PrimarySkill::ALL
        .into_iter()
        .zip(row)
        .find(|&(_, chance)| {
            let reached = rest <= u32::from(chance);
            rest = rest.saturating_sub(u32::from(chance));
            reached
        })
        .map_or(PrimarySkill::Knowledge, |(skill, _)| skill);
    let hero = Offering {
        details,
        core,
        class,
        level,
        banned,
        rules,
    };
    let mut offered = details.offered;
    let first = hero
        .offer(offered, 1..MAX_SKILL_LEVEL, None, draws)
        .or_else(|| hero.offer(offered, 0..MAX_SKILL_LEVEL, None, draws));
    if first.is_some_and(|skill| params.schools.contains(&skill)) {
        offered.school = level;
    }
    let second = first.and_then(|first| {
        hero.offer(offered, 0..1, Some(first), draws)
            .or_else(|| hero.offer(offered, 0..MAX_SKILL_LEVEL, Some(first), draws))
            .map(|second| (first, second))
    });
    let offers = match (first, second) {
        (_, Some((first, second))) => Offers::Two(first, second),
        (Some(first), None) => Offers::One(first),
        (None, None) => Offers::None,
    };
    for skill in first.into_iter().chain(second.map(|(_, s)| s)) {
        if skill == params.wisdom {
            offered.wisdom = level;
        }
        if params.schools.contains(&skill) {
            offered.school = level;
        }
    }
    Some(LevelUp {
        level,
        primary,
        offers,
        offered,
    })
}

/// A hero at its new level, for its offers under a rule.
struct Offering<'a> {
    details: &'a HeroDetails,
    core: &'a Ruleset,
    class: &'a HeroClass,
    level: u8,
    banned: &'a [SkillId],
    rules: Rules,
}

impl Offering<'_> {
    fn known(&self, skill: SkillId) -> u8 {
        let entry = self.details.skills.iter().find(|&&(s, _)| s == skill);
        entry.map_or(0, |&(_, level)| level)
    }

    fn weight(&self, skill: SkillId) -> u32 {
        let entry = self.class.skill_weights.iter().find(|&&(s, _)| s == skill);
        entry.map_or(0, |&(_, weight)| u32::from(weight))
    }

    fn offer(
        &self,
        offered: OfferLevels,
        window: Range<u8>,
        excluded: Option<SkillId>,
        draws: &mut OriginalDraws,
    ) -> Option<SkillId> {
        let params = self.core.hero_params()?;
        let full = self.details.skills.len() >= MAX_HERO_SKILLS;
        let window = if full {
            window.start.max(1)
        } else {
            window.start
        }..window.end;
        let candidate =
            |skill: SkillId| window.contains(&self.known(skill)) && excluded != Some(skill);
        let banned = |skill: SkillId| self.banned.contains(&skill);
        let [wisdom_interval, school_interval] =
            self.rules.intervals.unwrap_or(self.class.offer_intervals);
        let due =
            |last: u8, interval: u8| u16::from(self.level) >= u16::from(last) + u16::from(interval);
        if due(offered.wisdom, wisdom_interval)
            && candidate(params.wisdom)
            && !banned(params.wisdom)
        {
            return Some(params.wisdom);
        }
        let schools = &params.schools;
        if due(offered.school, school_interval) && !excluded.is_some_and(|s| schools.contains(&s)) {
            let weighted = schools
                .iter()
                .filter(|&&s| candidate(s) && !banned(s))
                .map(|&s| {
                    let known = self.known(s) > 0 && !self.rules.school_by_class_weight;
                    (s, if known { 1 } else { self.weight(s) })
                });
            if let Some(school) = draw_weighted(weighted, draws) {
                return Some(school);
            }
        }
        let count = u16::try_from(self.core.skills().len()).unwrap_or(u16::MAX);
        let weighted = (0..count)
            .map(SkillId::from_raw)
            .filter(|&s| candidate(s))
            .map(|s| {
                let known = self.known(s) > 0;
                let weight = match (banned(s), known) {
                    (true, _) => u32::from(known),
                    (false, true) => self.weight(s).max(1),
                    (false, false) => self.weight(s),
                };
                (s, weight)
            });
        draw_weighted(weighted, draws)
    }
}

fn draw_weighted(
    weighted: impl Iterator<Item = (SkillId, u32)> + Clone,
    draws: &mut OriginalDraws,
) -> Option<SkillId> {
    let total: u32 = weighted.clone().map(|(_, w)| w).sum();
    if total == 0 {
        return None;
    }
    let mut rest = draws.between(1, total);
    weighted
        .into_iter()
        .find(|&(_, weight)| {
            let reached = rest <= weight;
            rest = rest.saturating_sub(weight);
            reached
        })
        .map(|(skill, _)| skill)
}

// -- A batch map's heroes ------------------------------------------------------------------------

/// A source's levels at load: its rule, who chooses, the limit it applies, and whether the whole
/// hero is predicted (or only its experience and level: VCMI's own draws).
struct LoadVariant {
    name: &'static str,
    rule: (Rules, Chooser),
    limit: u8,
    whole: bool,
}

/// Who chooses a level's offer at load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Chooser {
    /// The computer's evaluation (the reading, D-20).
    Evaluation,
    /// The left offer (the wiki; VCMI's first).
    Left,
}

/// A batch map as the game builds it: its rules, limit, banned skills and terrain shares, and
/// each hero's type, its details before its levels at load and its custom experience.
struct Batch<'a> {
    core: &'a Ruleset,
    limit: u8,
    banned: Vec<SkillId>,
    shares: TerrainShares,
    heroes: Vec<(HeroTypeId, HeroDetails, Option<u32>)>,
}

impl<'a> Batch<'a> {
    /// The game built from `setup` (any seed: the seed bytes come from the record), and the same
    /// game with every custom experience 0, whose heroes are the details before their levels.
    fn new(rules: &'a GameRuleset, setup: &GameSetup) -> Result<Self, String> {
        let game = GameState::new(rules, setup, 1).map_err(|e| format!("{e:?}"))?;
        let mut before = setup.clone();
        for hero in &mut before.heroes {
            hero.details.experience = hero.details.experience.map(|_| 0);
        }
        let base = GameState::new(rules, &before, 1).map_err(|e| format!("{e:?}"))?;
        let heroes = setup
            .heroes
            .iter()
            .zip(0..=u16::MAX)
            .filter_map(|(hero, index)| {
                let details = base.hero(HeroId::new(index))?.clone();
                Some((hero.hero_type, details, hero.details.experience))
            })
            .collect();
        Ok(Self {
            core: rules.core(),
            limit: game.level_limit(),
            banned: game.banned_skills().to_vec(),
            shares: game.adventure().map().terrain_shares(),
            heroes,
        })
    }

    /// A hero's levels at load (GAM-SET-002 as `h3_game` gives them): `experience` gained under
    /// `limit`, then each pending level drawn under `rules` and given with `chooser`'s choice.
    fn at_load(
        &self,
        before: &HeroDetails,
        (experience, limit): (u32, u8),
        seed: u8,
        (rules, chooser): (Rules, Chooser),
    ) -> (HeroDetails, Vec<LevelUp>) {
        let mut details = before.clone();
        let mut levels = Vec::new();
        apply_change_limited(
            &mut details,
            self.core,
            limit,
            HeroChange::GainExperience(experience),
        );
        let pending = self
            .core
            .hero_params()
            .map_or(0, |params| pending_levels(&details, params));
        for _ in 0..pending {
            let Some(up) = draw(&details, self.core, &self.banned, seed, rules) else {
                break;
            };
            let choice = match chooser {
                Chooser::Evaluation => {
                    original_choice(&details, self.core, &up, ArmyWorth::SETUP, self.shares)
                }
                Chooser::Left => Choice::First,
            };
            apply_change_limited(&mut details, self.core, limit, up.change(choice));
            levels.push(up);
        }
        (details, levels)
    }

    /// `dialogs` level-ups from `start` with its experience `after`, drawn under `rules`, each
    /// answered with `answers` (left when it runs out); up front: every level drawn from `start`
    /// (its skills), the offers' levels carried from one to the next.
    fn in_play(
        &self,
        start: &HeroDetails,
        after: u32,
        seed: u8,
        (rules, up_front): (Rules, bool),
        answers: &[Choice],
    ) -> Vec<LevelUp> {
        let mut details = start.clone();
        details.experience = after;
        let mut probe = details.clone();
        let mut levels = Vec::new();
        for index in 0.. {
            let drawn = if up_front {
                draw(&probe, self.core, &self.banned, seed, rules)
            } else {
                draw(&details, self.core, &self.banned, seed, rules)
            };
            let Some(up) = drawn else { break };
            probe.level = up.level;
            probe.offered = up.offered;
            let choice = answers.get(index).copied().unwrap_or(Choice::First);
            apply_change_limited(&mut details, self.core, self.limit, up.change(choice));
            levels.push(up);
            if !up_front && details.level < up.level {
                break; // the level was not given (a change the hero refuses): draw no more
            }
        }
        levels
    }

    /// The report: for each hero of the template, its predictions under each rule its row
    /// compares, what was recorded and which predictions the record fits.
    fn report(&self, records: &[Record]) -> String {
        let mut out = String::new();
        for record in records {
            let Some((_, before, experience)) = self.heroes.iter().find(|(hero_type, ..)| {
                self.core
                    .hero_type(*hero_type)
                    .is_some_and(|h| h.name == record.hero_type)
            }) else {
                let _ = writeln!(
                    out,
                    "## {}: no hero `{}` on the map\n",
                    record.id, record.hero_type
                );
                continue;
            };
            out.push_str(&self.score(record, before, *experience));
        }
        out
    }

    /// One hero's predictions and record.
    fn score(&self, record: &Record, before: &HeroDetails, experience: Option<u32>) -> String {
        let mut out = String::new();
        let class = self
            .core
            .hero_type(before.hero_type)
            .and_then(|h| self.core.hero_class(h.class))
            .map_or("?", |c| c.name.as_str());
        let _ = writeln!(out, "## {} ({}, {class})\n", record.id, record.hero_type);
        let Some(seed) = record
            .number("seed byte")
            .and_then(|s| u8::try_from(s).ok())
        else {
            let _ = writeln!(out, "No seed byte recorded: nothing to predict.\n");
            return out;
        };
        let _ = writeln!(out, "Seed byte {seed}.\n");
        let observed_start = self.observed(record, "at the start", before);
        let mut start = observed_start.clone();
        let reading = experience.map(|e| {
            let rule = (Rules::default(), Chooser::Evaluation);
            self.at_load(before, (e, self.limit), seed, rule)
        });
        if let (Some(experience), Some((loaded, levels))) = (experience, reading)
            && !levels.is_empty()
        {
            let mut variants = vec![LoadVariant {
                name: "always-left",
                rule: (Rules::default(), Chooser::Left),
                limit: self.limit,
                whole: true,
            }];
            if self.limit > 0 {
                variants.push(LoadVariant {
                    name: "VCMI's limit not applied at load (D-21; its own draws: experience and \
                           level only)",
                    rule: (Rules::default(), Chooser::Left),
                    limit: 0,
                    whole: false,
                });
            }
            let _ = writeln!(out, "At load ({experience} experience):\n");
            let mut fits = Vec::new();
            let mut show = |name: &'static str, details: &HeroDetails, ups: &[LevelUp], whole| {
                let _ = writeln!(out, "- {name}: {}", self.hero_line(details));
                for up in ups {
                    let _ = writeln!(out, "  - {}", self.level_line(up));
                }
                let fit = observed_start.as_ref().is_some_and(|o| {
                    if whole {
                        self.same_hero(o, details)
                    } else {
                        (o.experience, o.level) == (details.experience, details.level)
                    }
                });
                if fit {
                    fits.push(name);
                }
            };
            show("the reading", &loaded, &levels, true);
            for v in &variants {
                let (details, ups) = self.at_load(before, (experience, v.limit), seed, v.rule);
                show(v.name, &details, &ups, v.whole);
            }
            match &observed_start {
                Some(o) => {
                    let _ = writeln!(out, "- recorded: {}", self.hero_line(o));
                    let _ = writeln!(out, "- fits: {}\n", list(&fits));
                }
                None => {
                    let _ = writeln!(
                        out,
                        "- recorded: incomplete; the reading's hero starts the dialogs below\n"
                    );
                }
            }
            if start.is_none() {
                start = Some(loaded);
            }
        }
        let start = start.unwrap_or_else(|| before.clone());
        let dialogs = record.dialogs(self.core);
        if dialogs.is_empty() {
            return out;
        }
        let after = record
            .number("experience after")
            .and_then(|e| u32::try_from(e).ok());
        let last_level = dialogs
            .iter()
            .filter_map(|d| d.level)
            .max()
            .unwrap_or(start.level);
        let after = after.unwrap_or_else(|| {
            self.core
                .hero_params()
                .map_or(start.experience, |p| experience_for(p, last_level))
        });
        let answers: Vec<Choice> = dialogs
            .iter()
            .map(|d| d.taken.unwrap_or(Choice::First))
            .collect();
        let row = record.id.get(..4).unwrap_or("");
        let mut variants: Vec<(&str, (Rules, bool))> =
            vec![("the reading", (Rules::default(), false))];
        match row {
            "E4-2" => variants.push((
                "VCMI's row by the old level (D-15)",
                (
                    Rules {
                        row_by_old_level: true,
                        ..Rules::default()
                    },
                    false,
                ),
            )),
            "E4-3" => variants.push((
                "intervals 3 and 3 (VCMI, HotA: D-16, A-10)",
                (
                    Rules {
                        intervals: Some([3, 3]),
                        ..Rules::default()
                    },
                    false,
                ),
            )),
            "E4-4" => variants.push(("drawn up front", (Rules::default(), true))),
            "E4-5" => variants.push((
                "a known school by its class weight (VCMI, D-17)",
                (
                    Rules {
                        school_by_class_weight: true,
                        ..Rules::default()
                    },
                    false,
                ),
            )),
            _ => {}
        }
        let _ = writeln!(
            out,
            "In play ({} dialog(s), {after} experience after):\n",
            dialogs.len()
        );
        let mut predicted = Vec::new();
        for (name, rule) in &variants {
            let levels = self.in_play(&start, after, seed, *rule, &answers);
            let shown: Vec<String> = levels.iter().map(|up| self.level_line(up)).collect();
            let _ = writeln!(out, "- {name}: {}", shown.join("; "));
            predicted.push((*name, levels));
        }
        let recorded: Vec<String> = dialogs.iter().map(|d| d.line()).collect();
        let _ = writeln!(out, "- recorded: {}", recorded.join("; "));
        let fits: Vec<&str> = predicted
            .iter()
            .filter(|(_, levels)| {
                levels.len() == dialogs.len()
                    && levels.iter().zip(&dialogs).all(|(up, d)| d.matches(up))
            })
            .map(|(name, _)| *name)
            .collect();
        let same = predicted.windows(2).all(|w| w[0].1 == w[1].1);
        let verdict = if same && predicted.len() > 1 {
            "the rules agree here: inconclusive"
        } else {
            ""
        };
        let _ = writeln!(out, "- fits: {} {verdict}\n", list(&fits));
        out
    }

    /// The hero recorded under `when` (its experience, level, primary and secondary skills and
    /// the levels last offered), on `before`'s type; `None` when a part is missing.
    fn observed(&self, record: &Record, when: &str, before: &HeroDetails) -> Option<HeroDetails> {
        let mut hero = before.clone();
        hero.experience = u32::try_from(record.number(&format!("experience {when}"))?).ok()?;
        hero.level = u8::try_from(record.number(&format!("level {when}"))?).ok()?;
        hero.primary = parse_primary(record.field(&format!("primary skills {when}"))?)?;
        hero.skills = parse_skills(
            record.field(&format!("secondary skills {when}"))?,
            self.core,
        )?;
        hero.offered = OfferLevels {
            wisdom: u8::try_from(record.number("Wisdom last offered").unwrap_or(0)).ok()?,
            school: u8::try_from(record.number("a school last offered").unwrap_or(0)).ok()?,
        };
        Some(hero)
    }

    /// Whether two heroes agree in experience, level, primary skills, skills (in any order) and
    /// the levels last offered.
    fn same_hero(&self, a: &HeroDetails, b: &HeroDetails) -> bool {
        let sorted = |h: &HeroDetails| {
            let mut skills = h.skills.clone();
            skills.sort_unstable();
            skills
        };
        (a.experience, a.level, a.primary, a.offered)
            == (b.experience, b.level, b.primary, b.offered)
            && sorted(a) == sorted(b)
    }

    fn skill_name(&self, skill: SkillId) -> &str {
        self.core.skill(skill).map_or("?", |s| s.name.as_str())
    }

    fn hero_line(&self, h: &HeroDetails) -> String {
        let [a, d, p, k] = h.primary;
        let skills: Vec<String> = h
            .skills
            .iter()
            .map(|&(s, l)| format!("{} {}", self.skill_name(s), LEVELS[usize::from(l)]))
            .collect();
        format!(
            "{} experience, level {}, {a}/{d}/{p}/{k}, {}; Wisdom last offered {}, a school {}",
            h.experience,
            h.level,
            skills.join(", "),
            h.offered.wisdom,
            h.offered.school
        )
    }

    fn level_line(&self, up: &LevelUp) -> String {
        let offers = match up.offers {
            Offers::None => "no offer".to_owned(),
            Offers::One(s) => format!("{} alone", self.skill_name(s)),
            Offers::Two(l, r) => format!("{} | {}", self.skill_name(l), self.skill_name(r)),
        };
        format!("level {}: {}, {offers}", up.level, up.primary.name())
    }
}

/// Secondary skill levels by their number, as the harness prints them.
const LEVELS: [&str; 4] = ["none", "basic", "advanced", "expert"];

fn list(names: &[&str]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

// -- The template ----------------------------------------------------------------------------

/// One encounter's section of a filled template: its id, its hero's type and its fields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Record {
    id: String,
    hero_type: String,
    fields: BTreeMap<String, String>,
}

/// A level-up dialog as recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dialog {
    level: Option<u8>,
    primary: Option<PrimarySkill>,
    left: Option<SkillId>,
    right: Option<SkillId>,
    taken: Option<Choice>,
    text: String,
}

impl Dialog {
    /// Whether a predicted level-up shows as recorded: its level, primary skill and offers.
    fn matches(&self, up: &LevelUp) -> bool {
        let offers = match up.offers {
            Offers::None => (None, None),
            Offers::One(s) => (Some(s), None),
            Offers::Two(l, r) => (Some(l), Some(r)),
        };
        self.level == Some(up.level)
            && self.primary == Some(up.primary)
            && (self.left, self.right) == offers
    }

    fn line(&self) -> String {
        self.text.clone()
    }
}

impl Record {
    /// The value of the first field whose label starts with `label`, if filled in.
    fn field(&self, label: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, v)| k.starts_with(label) && !v.trim().is_empty())
            .map(|(_, v)| v.trim())
    }

    /// The field's number (its digits; spaces between them allowed).
    fn number(&self, label: &str) -> Option<u64> {
        let value = self.field(label)?;
        let digits: String = value
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit() || *c == ' ')
            .filter(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    }

    /// The recorded dialogs, in order (`dialog 1: …` to the last one filled in).
    fn dialogs(&self, core: &Ruleset) -> Vec<Dialog> {
        let mut dialogs = Vec::new();
        for n in 1.. {
            let label = |part: &str| format!("dialog {n}, {part}");
            let Some(level) = self.number(&label("level")) else {
                break;
            };
            let left = self.field(&label("left")).and_then(|s| skill(s, core));
            let right = self.field(&label("right")).and_then(|s| skill(s, core));
            let taken = self.field(&label("taken")).and_then(|s| {
                let side = s.to_ascii_lowercase();
                match (side.contains("left"), side.contains("right")) {
                    (true, false) => Some(Choice::First),
                    (false, true) => Some(Choice::Second),
                    _ => skill(s, core).map(|t| {
                        if right == Some(t) {
                            Choice::Second
                        } else {
                            Choice::First
                        }
                    }),
                }
            });
            let primary = self.field(&label("primary")).and_then(parse_primary_skill);
            let text = format!(
                "level {level}: {}, {} | {} (taken {})",
                self.field(&label("primary")).unwrap_or("?"),
                self.field(&label("left")).unwrap_or("?"),
                self.field(&label("right")).unwrap_or("-"),
                self.field(&label("taken")).unwrap_or("?"),
            );
            dialogs.push(Dialog {
                level: u8::try_from(level).ok(),
                primary,
                left,
                right,
                taken,
                text,
            });
        }
        dialogs
    }
}

/// The template's encounters: a section starts at `## <id> — `, its hero at the line
/// `- Red hero **<id>** (<type>, …` (or Blue), its fields at `- <label>: <value>`.
fn parse_template(text: &str) -> Vec<Record> {
    let mut records: Vec<Record> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            let id = rest.split(" — ").next().unwrap_or(rest).trim().to_owned();
            records.push(Record {
                id,
                ..Record::default()
            });
            continue;
        }
        let Some(record) = records.last_mut() else {
            continue;
        };
        let Some(item) = line.strip_prefix("- ") else {
            continue;
        };
        if item.starts_with("Red hero **") || item.starts_with("Blue hero **") {
            if let Some(type_part) = item.split("** (").nth(1) {
                record.hero_type = type_part
                    .split([',', ')'])
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_owned();
            }
            continue;
        }
        if let Some((label, value)) = item.split_once(": ") {
            record
                .fields
                .insert(label.trim().to_owned(), value.trim().to_owned());
        } else if let Some(label) = item.strip_suffix(':') {
            record.fields.insert(label.trim().to_owned(), String::new());
        }
    }
    records
}

/// A skill by its name (any case).
fn skill(name: &str, core: &Ruleset) -> Option<SkillId> {
    let name = name.trim();
    let count = u16::try_from(core.skills().len()).unwrap_or(u16::MAX);
    (0..count).map(SkillId::from_raw).find(|&s| {
        core.skill(s)
            .is_some_and(|k| k.name.eq_ignore_ascii_case(name))
    })
}

/// `a/d/p/k`.
fn parse_primary(text: &str) -> Option<[u8; 4]> {
    let parts: Vec<u8> = text
        .split('/')
        .map(|p| p.trim().parse().ok())
        .collect::<Option<_>>()?;
    parts.try_into().ok()
}

/// A primary skill by its word: attack, defense, (spell) power, knowledge.
fn parse_primary_skill(text: &str) -> Option<PrimarySkill> {
    let t = text.to_ascii_lowercase();
    PrimarySkill::ALL.into_iter().find(|p| match p {
        PrimarySkill::Attack => t.contains("attack"),
        PrimarySkill::Defense => t.contains("defen"),
        PrimarySkill::Power => t.contains("power"),
        PrimarySkill::Knowledge => t.contains("knowledge"),
    })
}

/// `Leadership expert, Wisdom basic` (the level before or after the name), `none` for none.
fn parse_skills(text: &str, core: &Ruleset) -> Option<Vec<(SkillId, u8)>> {
    if text.trim().eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    text.split(',')
        .map(|item| {
            let words: Vec<&str> = item.split_whitespace().collect();
            let level = words
                .iter()
                .find_map(|w| LEVELS.iter().position(|l| l.eq_ignore_ascii_case(w)))?;
            let name: Vec<&str> = words
                .iter()
                .copied()
                .filter(|w| !LEVELS.iter().any(|l| l.eq_ignore_ascii_case(w)))
                .collect();
            Some((skill(&name.join(" "), core)?, u8::try_from(level).ok()?))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_heroes::draw_level_up;
    use h3_map::writer as w;

    /// A small generator for the test cases (a 64-bit LCG, Knuth's constants).
    struct Cases(u64);

    impl Cases {
        fn range_inclusive(&mut self, low: u32, high: u32) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let span = u64::from(high - low) + 1;
            low + u32::try_from((self.0 >> 33) % span).expect("in range")
        }
    }

    fn rules() -> GameRuleset {
        original_rules().expect("the original packs")
    }

    /// Random heroes of every class: the reading's draws are the engine's, for every seed byte.
    #[test]
    fn the_reading_draws_as_the_engine() {
        let rules = rules();
        let core = rules.core();
        let params = core.hero_params().expect("hero parameters");
        let mut rng = Cases(7);
        let skills = u32::try_from(core.skills().len()).expect("skills");
        let types = u32::try_from(core.hero_types().len()).expect("types");
        for case in 0..400u32 {
            let hero_type = HeroTypeId::from_raw(
                u16::try_from(rng.range_inclusive(0, types - 1)).expect("type"),
            );
            let count = rng.range_inclusive(0, 8);
            let mut known: Vec<(SkillId, u8)> = Vec::new();
            for _ in 0..count {
                let s = SkillId::from_raw(
                    u16::try_from(rng.range_inclusive(0, skills - 1)).expect("skill"),
                );
                if !known.iter().any(|&(k, _)| k == s) {
                    known.push((s, u8::try_from(rng.range_inclusive(1, 3)).expect("level")));
                }
            }
            let level = u8::try_from(rng.range_inclusive(1, 30)).expect("level");
            let offered = OfferLevels {
                wisdom: u8::try_from(rng.range_inclusive(0, u32::from(level))).expect("level"),
                school: u8::try_from(rng.range_inclusive(0, u32::from(level))).expect("level"),
            };
            let banned: Vec<SkillId> = (0..rng.range_inclusive(0, 2))
                .map(|_| {
                    SkillId::from_raw(
                        u16::try_from(rng.range_inclusive(0, skills - 1)).expect("skill"),
                    )
                })
                .collect();
            let details = HeroDetails {
                hero_type,
                level,
                experience: experience_for(params, level + 1),
                offered,
                primary: [1, 1, 1, 1],
                skills: known,
                worn: Vec::new(),
                backpack: Vec::new(),
                spells: Vec::new(),
                mana: 0,
            };
            for seed in [
                1,
                2,
                75,
                128,
                254,
                255,
                u8::try_from(case % 255 + 1).expect("seed"),
            ] {
                let ours = draw(&details, core, &banned, seed, Rules::default());
                let engine = draw_level_up(
                    &details,
                    core,
                    &banned,
                    &mut OriginalDraws::new(seed, level + 1),
                );
                assert_eq!(ours, engine, "case {case}, seed {seed}: {details:?}");
            }
        }
    }

    /// A map with heroes of custom experience under a level limit: for each seed of the game, each
    /// hero's levels at load are the reading's for one of the 255 seed bytes.
    #[test]
    fn levels_at_load_are_the_game_s() {
        let rules = rules();
        let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
        players[0] = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
        let hero = |tile: (u8, u8), hero_type: u8, class: u8, experience: u32| {
            w::MapObject::Hero(w::Hero {
                tile,
                owner: 0,
                hero_type,
                class,
                name: None,
                experience: Some(experience),
                skills: Some(vec![(6, 1)]),
                army: Some([Some((0, 1)), None, None, None, None, None, None]),
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: Some([2, 2, 1, 1]),
            })
        };
        for limit in [0, 5] {
            let map = w::Map {
                name: "E4".to_owned(),
                description: "Levels at load".to_owned(),
                size: 36,
                terrain: w::Terrain::Grass,
                patches: vec![],
                roads: vec![],
                players: players.clone(),
                objects: vec![
                    hero((3, 3), 0, 0, 3200),
                    hero((9, 3), 70, 8, 5000),
                    hero((15, 3), 96, 12, 10_000),
                ],
                underground: None,
                level_limit: limit,
            };
            let file = w::gzip(&w::write(&map).expect("map"));
            let raw = h3_map::read(&file, &Limits::default()).expect("read");
            let setup = h3_map::import(&raw, &rules).expect("import");
            let batch = Batch::new(&rules, &setup).expect("batch");
            assert_eq!(batch.limit, limit);
            for seed in 1..4 {
                let game = GameState::new(&rules, &setup, seed).expect("game");
                for (index, (_, before, experience)) in batch.heroes.iter().enumerate() {
                    let id = HeroId::new(u16::try_from(index).expect("id"));
                    let built = game.hero(id).expect("hero");
                    let experience = experience.expect("custom");
                    let found = (1..=255).any(|s| {
                        let (ours, _) = batch.at_load(
                            before,
                            (experience, limit),
                            s,
                            (Rules::default(), Chooser::Evaluation),
                        );
                        ours.level == built.level
                            && ours.experience == built.experience
                            && ours.primary == built.primary
                            && ours.skills == built.skills
                            && ours.offered == built.offered
                    });
                    assert!(
                        found,
                        "limit {limit}, game seed {seed}, hero {index}: {built:?}"
                    );
                }
            }
        }
    }

    /// A template filled in from the reading's own predictions fits the reading; one with
    /// another primary skill fits nothing.
    #[test]
    fn a_record_of_the_reading_fits_it() {
        let rules = rules();
        let core = rules.core();
        let params = core.hero_params().expect("hero parameters");
        let knight = core
            .hero_types()
            .iter()
            .position(|h| h.name == "Orrin")
            .and_then(|i| u16::try_from(i).ok())
            .map(HeroTypeId::from_raw)
            .expect("Orrin");
        let leadership = skill("leadership", core).expect("Leadership");
        let before = HeroDetails {
            hero_type: knight,
            level: 1,
            experience: 0,
            offered: OfferLevels::default(),
            primary: [2, 2, 1, 1],
            skills: vec![(leadership, 1)],
            worn: Vec::new(),
            backpack: Vec::new(),
            spells: Vec::new(),
            mana: 0,
        };
        let batch = Batch {
            core,
            limit: 0,
            banned: Vec::new(),
            shares: TerrainShares::default(),
            heroes: vec![(knight, before.clone(), Some(3200))],
        };
        let seed = 75;
        let (loaded, _) = batch.at_load(
            &before,
            (3200, 0),
            seed,
            (Rules::default(), Chooser::Evaluation),
        );
        let after = experience_for(params, 5) + 10;
        let ups = batch.in_play(&loaded, after, seed, (Rules::default(), false), &[]);
        assert_eq!(ups.len(), 1);
        let up = ups[0];
        let names = |s: SkillId| core.skill(s).expect("skill").name.clone();
        let (left, right) = match up.offers {
            Offers::Two(l, r) => (names(l), names(r)),
            Offers::One(l) => (names(l), String::new()),
            Offers::None => (String::new(), String::new()),
        };
        let skills: Vec<String> = loaded
            .skills
            .iter()
            .map(|&(s, l)| format!("{} {}", names(s), LEVELS[usize::from(l)]))
            .collect();
        let [a, d, p, k] = loaded.primary;
        let filled = |primary: &str| {
            format!(
                "# E08 batch E4a — results\n\n## E4-1a — Levels at load (D-20)\n\n\
                 - Blue hero **E4-1a** (Orrin, specialty skill Archery, 3200 experience): …\n\n\
                 1. Read it.\n\n**Record**\n\n\
                 - seed byte (hero +0x8F): {seed}\n\
                 - Wisdom last offered (hero +0x90): {}\n\
                 - a school last offered (hero +0x3F): {}\n\
                 - experience at the start: 3 200\n\
                 - level at the start: {}\n\
                 - primary skills at the start (attack/defense/power/knowledge): {a}/{d}/{p}/{k}\n\
                 - secondary skills at the start (in the hero's order, each with its level): {}\n\
                 - dialog 1, level: {}\n\
                 - dialog 1, primary skill raised: {primary}\n\
                 - dialog 1, left offer (or the only one): {left}\n\
                 - dialog 1, right offer: {right}\n\
                 - dialog 1, taken: {left}\n\
                 - experience after: {after}\n\
                 - anything odd: \n",
                loaded.offered.wisdom,
                loaded.offered.school,
                loaded.level,
                skills.join(", "),
                up.level,
            )
        };
        let records = parse_template(&filled(up.primary.name()));
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].hero_type, "Orrin");
        assert_eq!(records[0].number("experience at the start"), Some(3200));
        let report = batch.report(&records);
        assert!(report.contains("At load (3200 experience)"), "{report}");
        assert!(report.contains("- fits: the reading"), "{report}");
        let wrong = PrimarySkill::ALL
            .into_iter()
            .find(|&p| p != up.primary)
            .expect("another skill");
        let report = batch.report(&parse_template(&filled(wrong.name())));
        assert!(report.contains("- fits: none"), "{report}");
        assert!(
            batch
                .report(&parse_template(
                    "## X — x\n\n- Red hero **X** (Nobody, …)\n"
                ))
                .contains("no hero `Nobody`")
        );
        assert_eq!(parse_skills("none", core), Some(Vec::new()));
        assert_eq!(
            parse_skills("expert Leadership", core),
            Some(vec![(leadership, 3)])
        );
        assert_eq!(parse_primary("1/2/3"), None);
        assert_eq!(
            parse_primary_skill("Spell Power"),
            Some(PrimarySkill::Power)
        );
    }
}
