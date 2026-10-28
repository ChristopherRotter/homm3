//! The draws when the game is built (E08 T61b; increment 7's design note §9 in `E08-log.md`,
//! ADR-0016 point 4, ADR-0017 point 4): what increment 7's objects hold that the map leaves to
//! chance, drawn from the setup stream after the piles' and the monsters' draws, in two walks in
//! object order, as the original resolves the random artifacts (`resolve_random_objects` 0x4C9DD0)
//! before its new game's pass over the rest (`setup_map_objects` 0x4C0CC0; survey
//! `E08-rewards.md` §3.1). Within an object the draws' order is the engine's (E08 decision 17),
//! so the original's dropped draws (a placed artifact's, §3.1) have no counterpart. No draw gives
//! a banned skill, spell or artifact (GAM-SET-003), nor a spell granted by name only (the
//! original's game bans the spells with the flag 0x2000 beside the map's).

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{
    ArtifactClass, ArtifactTypeId, ROE_WITCH_HUT_MASK, ResourceId, Ruleset, SkillId, SpellCategory,
    SpellId,
};
use h3_heroes::{PrimarySkill, ScholarGift};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::{MapObject, pick};
use crate::ruleset::{AdventureRuleset, Chest, Find, FindContent, ObjectKind, Turn};
use crate::setup::{Bans, Holds};

/// What the build drew for an object of increment 7's kinds (E08 T61b; design §9), beside what the
/// map put in it ([`MapObject::holds`]); a chest's gold and a find's resources are its
/// [`MapObject::contents`]. The visits that read it come with T62.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Resolved {
    /// Nothing drawn: every other kind, a gift of one price or none, a chest of gold, a find of
    /// resources or of nothing; or a draw that found nothing: a random artifact, chest or find
    /// with every artifact banned, a Witch Hut with every skill of its mask banned (it refuses
    /// every visit), a Shrine whose level has no spell to give (A-18 fixed), the Pyramid with
    /// every spell of its level banned (A-20 fixed: it fights and gives nothing).
    #[default]
    Nothing,
    /// The artifact drawn: a random artifact's (ADV-OBJ-006), a chest's (ADV-PIK-005), a find's
    /// (ADV-PIK-009 to -011).
    Artifact(ArtifactTypeId),
    /// The price drawn among a gift's, by index into its type's prices (the Tree of Knowledge,
    /// ADV-TCH-003).
    Price(u8),
    /// A Shrine's spell, the map's or drawn (ADV-TCH-004), or the Pyramid's (ADV-TCH-008).
    Spell(SpellId),
    /// A Witch Hut's skill, drawn among its mask (ADV-TCH-005).
    Skill(SkillId),
    /// A Scholar's gift, the map's or drawn, and the primary skill it raises instead of a skill
    /// or spell the hero cannot take (ADV-TCH-006; for a primary skill's gift, that skill).
    Scholar(ScholarGift, PrimarySkill),
    /// A University's skills, in the draw's order (ADV-TCH-007).
    Skills(Vec<SkillId>),
}

impl Resolved {
    /// The artifact drawn, if one ([`Self::Artifact`]).
    pub const fn artifact(&self) -> Option<ArtifactTypeId> {
        match self {
            Self::Artifact(artifact) => Some(*artifact),
            _ => None,
        }
    }

    /// A University's skills ([`Self::Skills`]); none for every other draw.
    pub fn skills(&self) -> &[SkillId] {
        match self {
            Self::Skills(skills) => skills,
            _ => &[],
        }
    }
}

impl CanonicalHash for Resolved {
    /// Its tag (0 nothing, 1 an artifact, 2 a price's index, 3 a spell, 4 a skill, 5 a Scholar's
    /// gift with its tag (0 a primary skill, 1 a skill, 2 a spell), its value and the fallback, 6
    /// skills as length and entries), then its values.
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Nothing => h.write_u8(0),
            Self::Artifact(artifact) => {
                h.write_u8(1);
                artifact.hash_into(h);
            }
            Self::Price(index) => {
                h.write_u8(2);
                h.write_u8(*index);
            }
            Self::Spell(spell) => {
                h.write_u8(3);
                spell.hash_into(h);
            }
            Self::Skill(skill) => {
                h.write_u8(4);
                skill.hash_into(h);
            }
            Self::Scholar(gift, fallback) => {
                h.write_u8(5);
                match gift {
                    ScholarGift::Primary(skill) => {
                        h.write_u8(0);
                        h.write_u8(*skill as u8);
                    }
                    ScholarGift::Skill(skill) => {
                        h.write_u8(1);
                        skill.hash_into(h);
                    }
                    ScholarGift::Spell(spell) => {
                        h.write_u8(2);
                        spell.hash_into(h);
                    }
                }
                h.write_u8(*fallback as u8);
            }
            Self::Skills(skills) => {
                h.write_u8(6);
                skills.hash_into(h);
            }
        }
    }
}

/// The classes a random artifact widens to when its own classes have nothing left (the
/// original's level bits 0x1E): every class but Special.
const ANY_CLASS: [ArtifactClass; 4] = [
    ArtifactClass::Treasure,
    ArtifactClass::Minor,
    ArtifactClass::Major,
    ArtifactClass::Relic,
];

/// The random artifacts' pool (ADV-OBJ-006; survey `E08-rewards.md` §3.1, D-31, D-32): the
/// original's "given" marks, a mark per artifact, starting as the bans and the map's placed
/// artifacts (`artifacts_given_init` 0x4C0AA0); each draw marks the artifact it gives, so no
/// artifact repeats until the pool is used up (`random_artifact_of_level` 0x4C94D0).
pub(super) struct ArtifactPool<'c> {
    core: &'c Ruleset,
    /// The banned artifacts, by index.
    banned: Vec<bool>,
    /// The marked ones, by index.
    marks: Vec<bool>,
}

impl<'c> ArtifactPool<'c> {
    /// The pool with the bans marked, then each of `placed`.
    pub(super) fn new(
        core: &'c Ruleset,
        bans: &[ArtifactTypeId],
        placed: &[ArtifactTypeId],
    ) -> Self {
        let among = |list: &[ArtifactTypeId], index| list.iter().any(|a| a.index() == index);
        let banned: Vec<bool> = (0..core.artifacts().len())
            .map(|index| among(bans, index))
            .collect();
        let marks = banned
            .iter()
            .enumerate()
            .map(|(index, &ban)| ban || among(placed, index))
            .collect();
        Self {
            core,
            banned,
            marks,
        }
    }

    /// The unmarked artifacts of `classes`, in id order.
    fn left(&self, classes: &[ArtifactClass]) -> Vec<ArtifactTypeId> {
        self.core
            .artifacts()
            .iter()
            .zip(&self.marks)
            .enumerate()
            .filter(|&(_, (artifact, marked))| !marked && classes.contains(&artifact.class))
            .filter_map(|(index, _)| ArtifactTypeId::from_index(index))
            .collect()
    }

    /// An artifact of `classes` with equal chances among the unmarked ones, then marked; none
    /// left: the marks restart from the bans, and with still none, from any class but Special.
    /// None at all (every artifact banned): none.
    // rule: ADV-OBJ-006
    pub(super) fn draw(
        &mut self,
        classes: &[ArtifactClass],
        draws: &mut Rng,
    ) -> Option<ArtifactTypeId> {
        let mut left = self.left(classes);
        if left.is_empty() {
            self.marks.clone_from(&self.banned);
            left = self.left(classes);
        }
        if left.is_empty() {
            left = self.left(&ANY_CLASS);
        }
        let drawn = *pick(&left, draws)?;
        self.marks[drawn.index()] = true;
        Some(drawn)
    }
}

/// The spells a draw may give (the original's spells 0 to 69 with a school, not banned): each
/// spell's level if it may, by index.
pub(super) struct Spells(Vec<Option<u8>>);

impl Spells {
    /// The hero spells (each with a school, as the core's compiler requires) that neither the
    /// bans nor the ruleset's grants by name only leave out.
    pub(super) fn new(core: &Ruleset, bans: &[SpellId]) -> Self {
        Self(
            core.spells()
                .iter()
                .enumerate()
                .map(|(index, spell)| {
                    let banned =
                        spell.granted_by_name_only || bans.iter().any(|ban| ban.index() == index);
                    (spell.category != SpellCategory::Ability && !banned).then_some(spell.level)
                })
                .collect(),
        )
    }

    /// The spells of `level` (any level for none) not in `marks` (none for no marks), in id
    /// order.
    fn left(&self, level: Option<u8>, marks: &[bool]) -> Vec<SpellId> {
        self.0
            .iter()
            .enumerate()
            .filter(|&(index, &of)| {
                of.is_some()
                    && level.is_none_or(|level| of == Some(level))
                    && marks.get(index) != Some(&true)
            })
            .filter_map(|(index, _)| SpellId::from_index(index))
            .collect()
    }
}

/// The random Shrines' marks (ADV-TCH-004; survey §6.5, D-34): a mark per spell, set by each draw,
/// so the random Shrines of a level repeat no spell until each of the level's is out; then the
/// level's marks clear and its draws start again (`random_shrine_spell` 0x4C95A0, whose marks
/// start as the bans, which [`Spells`] leaves out already).
pub(super) struct ShrineMarks(Vec<bool>);

impl ShrineMarks {
    /// No spell marked.
    pub(super) fn new(spells: &Spells) -> Self {
        Self(vec![false; spells.0.len()])
    }

    /// A spell of `level` with equal chances among the unmarked ones, then marked; none for a
    /// level without one.
    // rule: ADV-TCH-004
    pub(super) fn draw(&mut self, spells: &Spells, level: u8, draws: &mut Rng) -> Option<SpellId> {
        let mut left = spells.left(Some(level), &self.0);
        if left.is_empty() {
            for (mark, &of) in self.0.iter_mut().zip(&spells.0) {
                if of == Some(level) {
                    *mark = false;
                }
            }
            left = spells.left(Some(level), &self.0);
        }
        let drawn = *pick(&left, draws)?;
        self.0[drawn.index()] = true;
        Some(drawn)
    }
}

/// The kind of an object's type (none for a blocker).
fn kind_of<'p>(part: &'p AdventureRuleset, object: &MapObject) -> Option<&'p ObjectKind> {
    object
        .object_type
        .and_then(|id| part.object_type(id))
        .map(|ty| &ty.kind)
}

/// Draws what the objects hold that the map leaves to chance (design §9), in two walks in object
/// order: each random artifact's artifact (ADV-OBJ-006); then each other object's draws in its
/// own shape: a chest (ADV-PIK-005), a gift with several prices (ADV-TCH-003), a Shrine without
/// the map's spell (ADV-TCH-004), a Witch Hut (ADV-TCH-005, `ADV-QUIRK-21` for an RoE map's),
/// a Scholar (ADV-TCH-006), a University (ADV-TCH-007), a find (ADV-PIK-009 to -012) and the
/// Pyramid (ADV-TCH-008). The random artifacts' marks are shared by the random artifacts, the
/// chests and the finds.
pub(super) fn resolve_rewards(
    core: &Ruleset,
    part: &AdventureRuleset,
    bans: &Bans,
    objects: &mut [MapObject],
    draws: &mut Rng,
) {
    let placed = objects
        .iter()
        .filter_map(|object| match (kind_of(part, object), &object.holds) {
            (Some(ObjectKind::Artifact { classes: None }), Holds::Artifact(item)) => {
                Some(item.artifact)
            }
            _ => None,
        });
    let placed: Vec<ArtifactTypeId> = placed.collect();
    let mut artifacts = ArtifactPool::new(core, &bans.artifacts, &placed);
    for object in objects.iter_mut() {
        if let Some(ObjectKind::Artifact {
            classes: Some(classes),
        }) = kind_of(part, object)
        {
            object.resolved = artifacts
                .draw(classes, draws)
                .map_or(Resolved::Nothing, Resolved::Artifact);
        }
    }
    let spells = Spells::new(core, &bans.spells);
    let mut shrines = ShrineMarks::new(&spells);
    let skills: Vec<SkillId> = (0..core.skills().len())
        .filter_map(SkillId::from_index)
        .filter(|skill| !bans.skills.contains(skill))
        .collect();
    let any_level = spells.left(None, &[]);
    let roe_mask = core.quirks().is_enabled(ROE_WITCH_HUT_MASK);
    for object in objects.iter_mut() {
        let Some(kind) = kind_of(part, object) else {
            continue;
        };
        object.resolved = match (kind, &object.holds) {
            (ObjectKind::Chest(chest), _) => {
                let (contents, resolved) = chest_draw(chest, &mut artifacts, draws);
                object.contents = contents;
                resolved
            }
            (ObjectKind::Gift(gift), _) if gift.prices.len() > 1 => {
                // rule: ADV-TCH-003
                let n = u32::try_from(gift.prices.len()).unwrap_or(u32::MAX);
                Resolved::Price(u8::try_from(draws.below(n)).unwrap_or(u8::MAX))
            }
            (ObjectKind::Shrine { .. }, &Holds::Spell(Some(spell))) => Resolved::Spell(spell),
            (ObjectKind::Shrine { level }, _) => shrines
                .draw(&spells, *level, draws)
                .map_or(Resolved::Nothing, Resolved::Spell),
            (ObjectKind::WitchHut { roe, roe_fixed }, holds) => {
                let mask = match holds {
                    Holds::Skills(Some(mask)) => mask,
                    _ if roe_mask => roe,
                    _ => roe_fixed,
                };
                hut_draw(mask, &skills, draws)
            }
            (ObjectKind::Scholar, &Holds::Gift(gift)) => {
                scholar_draw(gift, &skills, &any_level, draws)
            }
            (ObjectKind::University { offers, .. }, _) => {
                Resolved::Skills(university_draw(&skills, *offers, draws))
            }
            (ObjectKind::Find(find), _) => {
                let (contents, resolved) = find_draw(find, &mut artifacts, draws);
                object.contents = contents;
                resolved
            }
            (ObjectKind::Pyramid(pyramid), _) => {
                // rule: ADV-TCH-008
                pick(&spells.left(Some(pyramid.level), &[]), draws)
                    .map_or(Resolved::Nothing, |&spell| Resolved::Spell(spell))
            }
            _ => continue,
        };
    }
}

/// A chest's contents (ADV-PIK-005; survey §2.1): r in 0 to 99; below the first chance its first
/// gold, below the first two together its second, and so on; else an artifact of its classes from
/// the random artifacts' pool.
// rule: ADV-PIK-005
fn chest_draw(
    chest: &Chest,
    artifacts: &mut ArtifactPool<'_>,
    draws: &mut Rng,
) -> (Vec<(ResourceId, u32)>, Resolved) {
    let r = draws.below(100);
    let mut bound = 0;
    for &(chance, gold) in &chest.gold {
        bound += u32::from(chance);
        if r < bound {
            return (vec![(chest.resource, gold)], Resolved::Nothing);
        }
    }
    let artifact = artifacts.draw(&chest.artifact, draws);
    (
        Vec::new(),
        artifact.map_or(Resolved::Nothing, Resolved::Artifact),
    )
}

/// A Witch Hut's skill (ADV-TCH-005; survey §6.6, D-33, A-21): the r-th of its mask's skills in
/// id order less the banned ones (`skills`: the unbanned skills in id order), with equal chances;
/// none left: none.
// rule: ADV-TCH-005
fn hut_draw(mask: &[SkillId], skills: &[SkillId], draws: &mut Rng) -> Resolved {
    let left: Vec<SkillId> = skills
        .iter()
        .copied()
        .filter(|skill| mask.contains(skill))
        .collect();
    pick(&left, draws).map_or(Resolved::Nothing, |&skill| Resolved::Skill(skill))
}

/// A Scholar's gift and fallback (ADV-TCH-006; survey §6.7): the map's gift, else a kind with
/// equal chances among the three, then a primary skill among the four, a skill among `skills` or
/// a spell among `spells` (the unbanned ones; an empty pool gives a primary skill instead); for a
/// skill or spell the primary skill it raises instead, among the four.
// rule: ADV-TCH-006
fn scholar_draw(
    gift: Option<ScholarGift>,
    skills: &[SkillId],
    spells: &[SpellId],
    draws: &mut Rng,
) -> Resolved {
    let primary = |draws: &mut Rng| {
        pick(&PrimarySkill::ALL, draws)
            .copied()
            .unwrap_or(PrimarySkill::Attack)
    };
    let gift = gift.unwrap_or_else(|| {
        match draws.below(3) {
            0 => None,
            1 => pick(skills, draws).copied().map(ScholarGift::Skill),
            _ => pick(spells, draws).copied().map(ScholarGift::Spell),
        }
        .unwrap_or_else(|| ScholarGift::Primary(primary(draws)))
    });
    let fallback = match gift {
        ScholarGift::Primary(skill) => skill,
        ScholarGift::Skill(_) | ScholarGift::Spell(_) => primary(draws),
    };
    Resolved::Scholar(gift, fallback)
}

/// A University's skills (ADV-TCH-007; survey §6.8, D-36, A-22): `offers` distinct skills, each
/// with equal chances among the unbanned `skills` left (Necromancy among them); fewer left, all
/// of them (A-22 fixed).
// rule: ADV-TCH-007
fn university_draw(skills: &[SkillId], offers: u8, draws: &mut Rng) -> Vec<SkillId> {
    let mut left = skills.to_vec();
    let mut drawn = Vec::with_capacity(usize::from(offers));
    while drawn.len() < usize::from(offers) && !left.is_empty() {
        let n = u32::try_from(left.len()).unwrap_or(u32::MAX);
        drawn.push(left.remove(usize::try_from(draws.below(n)).unwrap_or(0)));
    }
    drawn
}

/// A find's contents (ADV-PIK-009 to -012; survey §6.9, D-35): r in 0 to 99 picks its entry by the
/// chances in order; an artifact of the entry's classes from the random artifacts' pool, with the
/// find's resources drawn too when they are what a full backpack turns it into (the Wagon); an
/// amount between the entry's bounds, then one of its resources; or nothing.
// rule: ADV-PIK-009, ADV-PIK-010, ADV-PIK-011, ADV-PIK-012
fn find_draw(
    find: &Find,
    artifacts: &mut ArtifactPool<'_>,
    draws: &mut Rng,
) -> (Vec<(ResourceId, u32)>, Resolved) {
    let r = draws.below(100);
    let mut bound = 0;
    let entry = find.draws.iter().find_map(|(chance, content)| {
        bound += u32::from(*chance);
        (r < bound).then_some(content)
    });
    let resources = |draws: &mut Rng| {
        find.draws
            .iter()
            .find_map(|(_, content)| match content {
                FindContent::Resources { amount, resources } => {
                    let amount = draws.range_inclusive(amount[0], amount[1]);
                    pick(resources, draws).map(|&resource| vec![(resource, amount)])
                }
                _ => None,
            })
            .unwrap_or_default()
    };
    match entry {
        Some(FindContent::Artifact(classes)) => {
            let artifact = artifacts.draw(classes, draws);
            let contents = if find.full_backpack == Some(Turn::Drawn) {
                resources(draws)
            } else {
                Vec::new()
            };
            (
                contents,
                artifact.map_or(Resolved::Nothing, Resolved::Artifact),
            )
        }
        Some(FindContent::Resources { .. }) => (resources(draws), Resolved::Nothing),
        _ => (Vec::new(), Resolved::Nothing),
    }
}
