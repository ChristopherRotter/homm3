//! The parameters of increment 7's object kinds (E08 T61a; ADR-0016 point 4, increment 7's design
//! note §8 in `E08-log.md`): what a type of treasure chest, once-per-hero gift, find or Pyramid
//! holds as data, and their compilation from the packs' keys. The visits that read them come with
//! T62 (ADV-PIK-005 to -013, ADV-TCH-001 to -008), the draws when the game is built with T61b
//! (ADR-0016 point 4, GAM-SET-001's stream). The kinds without parameters of their own (a Spell
//! Scroll, a Scholar, Pandora's Box, an event) and the small ones (a Shrine's level, a Witch Hut's
//! RoE skills, a University's offers and price) are written inline in [`super::ObjectKind`].

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use h3_content::{ArtifactClass, ContentKey, CreatureTypeId, ResourceId};
use h3_heroes::{BattleBonuses, PrimarySkill};
use h3_kernel::{CanonicalHash, StateHasher};

use super::{AdventureCompileError, MAX_GAIN, MAX_MONSTER_COUNT, Scope};

/// A price or gift of resources: distinct resources in the core's order, each amount from 1 to
/// [`MAX_GAIN`].
pub type Resources = Vec<(ResourceId, u32)>;

/// A battle bonus an object gives once until the hero's next battle (HER-ADV-008): the morale
/// and luck it adds and the once-mark, below [`BattleBonuses::MARKS`], that gates and records it
/// (the Warrior's Tomb: −3 morale, mark 0; a looted Pyramid: −2 luck, mark 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnceBonus {
    /// The morale added.
    pub morale: i8,
    /// The luck added.
    pub luck: i8,
    /// The once-mark.
    pub mark: u8,
}

/// A treasure chest's type (ADV-PIK-005, survey `E08-rewards.md` §2): its contents drawn when
/// the game is built, gold by these chances and an artifact with the rest of 100.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chest {
    /// The resource its gold is.
    pub resource: ResourceId,
    /// The gold amounts, each with its chance in percent (32, 32, 31: 1 000, 1 500, 2 000), in
    /// the draw's order.
    pub gold: Vec<(u8, u32)>,
    /// The classes the artifact is drawn from with the rest of 100 (5 %: treasure).
    pub artifact: Vec<ArtifactClass>,
    /// The gold an artifact turns into for a hero who cannot receive it (1 000).
    pub full_backpack: u32,
    /// What comes off the gold before the hero's Learning for the experience choice (500).
    pub experience_offset: u32,
}

/// [`Chest`] as a pack defines it: the gold's resource by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChestDef {
    /// A core resource.
    pub resource: ContentKey,
    /// The gold amounts and chances: at least one, each chance from 1, together at most 100, each
    /// amount from the offset to [`MAX_GAIN`].
    pub gold: Vec<(u8, u32)>,
    /// The artifact's classes: distinct, never Special; at least one when the chances leave room.
    pub artifact: Vec<ArtifactClass>,
    /// The full backpack's gold, from the offset to [`MAX_GAIN`].
    pub full_backpack: u32,
    /// The experience offset.
    pub experience_offset: u32,
}

/// What a once-per-hero gift gives (ADV-TCH-001 to -003, survey `E08-rewards.md` §6.2 to §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gives {
    /// These gains to the primary skills, in [`PrimarySkill`] order (Marletto Tower +1 Defense,
    /// the Library +2 each).
    Primary([u8; 4]),
    /// One of two primary skills, the hero's owner's choice, raised by this much (the Arena +2
    /// Attack or Defense, the Schools +1).
    Choice {
        /// The two skills, in the dialog's order.
        skills: [PrimarySkill; 2],
        /// The gain.
        amount: u8,
    },
    /// This experience times the hero's Learning (the Learning Stone's 1 000).
    Experience(u32),
    /// The experience to the hero's next level, without Learning (the Tree of Knowledge).
    Level,
}

/// A once-per-hero gift's type (ADV-TCH-001 to -003): what it gives, at what price, to whom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gift {
    /// What it gives.
    pub gives: Gives,
    /// Its prices: none is free; one is the price; several are drawn from when the game is built,
    /// one each with equal chances (the Tree of Knowledge: free, 2 000 gold or 10 gems). An empty
    /// price is free.
    pub prices: Vec<Resources>,
    /// The Library's condition: a hero is admitted when its level plus twice its Diplomacy level
    /// reaches this (`h3_heroes::library_admits`, 10).
    pub admits: Option<u8>,
    /// Whether the visitor may leave its choice without the gift (the Schools; the Arena's choice
    /// has no way out).
    pub leave: bool,
}

/// [`Gift`] as a pack defines it: the prices' resources by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GiftDef {
    /// What it gives: primary gains not all 0; a choice of two distinct skills with a gain from
    /// 1; an experience from 1 to [`MAX_GAIN`].
    pub gives: Gives,
    /// Its prices, each of distinct core resources with amounts from 1 to [`MAX_GAIN`].
    pub prices: Vec<Vec<(ContentKey, u32)>>,
    /// The Library's condition, from 1.
    pub admits: Option<u8>,
    /// Whether the visitor may leave its choice.
    pub leave: bool,
}

/// What a find may hold (ADV-PIK-009 to -012, survey `E08-rewards.md` §6.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindContent {
    /// Nothing.
    Nothing,
    /// An artifact drawn among these classes as one pool.
    Artifact(Vec<ArtifactClass>),
    /// An amount drawn between the bounds, inclusive, of one of the resources, drawn with equal
    /// chances.
    Resources {
        /// The bounds.
        amount: [u32; 2],
        /// The resources.
        resources: Vec<ResourceId>,
    },
    /// These resources, and beside them an artifact drawn among these classes as one pool if any
    /// (E08 T72, increment 8's design note §5: the sea's finds, ADV-PIK-014, -015; Flotsam's 5 wood
    /// and 200 gold, the Sea Chest's 1 000 gold and a treasure artifact): given together, the
    /// artifact through the full backpack's turn.
    Fixed {
        /// The resources, sorted by resource: distinct, at least one.
        resources: Resources,
        /// The classes of the artifact beside them; none for resources alone.
        artifact: Option<Vec<ArtifactClass>>,
    },
}

/// [`FindContent`] as a pack defines it: resources by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindContentDef {
    /// [`FindContent::Nothing`].
    Nothing,
    /// [`FindContent::Artifact`]: distinct classes, at least one, never Special.
    Artifact(Vec<ArtifactClass>),
    /// [`FindContent::Resources`]: bounds from 1 to [`MAX_GAIN`] in order; distinct core
    /// resources, at least one.
    Resources {
        /// The bounds.
        amount: [u32; 2],
        /// The resources.
        resources: Vec<ContentKey>,
    },
    /// [`FindContent::Fixed`] (E08 T72): distinct core resources, at least one, each amount from 1
    /// to [`MAX_GAIN`]; the artifact's classes as [`FindContentDef::Artifact`]'s.
    Fixed {
        /// The resources.
        resources: Vec<(ContentKey, u32)>,
        /// The artifact's classes.
        artifact: Option<Vec<ArtifactClass>>,
    },
}

/// What a find's artifact turns into for a hero who cannot receive it (survey §8.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Turn {
    /// These resources to the hero's owner instead of the whole entry, its artifact and the
    /// resources beside it (the Corpse's 1 000 gold; since E08 T72 the Sea Chest's 1 500 gold
    /// instead of its 1 000 and the artifact, survey `E08-sea.md` §8.3).
    Give(Resources),
    /// The find's own resources, drawn with the artifact when the game is built (the Wagon).
    Drawn,
    /// Nothing: the artifact stays in the find (the Warrior's Tomb).
    Stays,
    /// Nothing: the artifact is lost and the find emptied all the same (E08 T72: the Shipwreck
    /// Survivor, which leaves anyway; survey `E08-sea.md` §8.4).
    Lost,
}

/// [`Turn`] as a pack defines it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnDef {
    /// [`Turn::Give`]: distinct core resources, at least one, amounts from 1 to [`MAX_GAIN`].
    Give(Vec<(ContentKey, u32)>),
    /// [`Turn::Drawn`]: the find has a resources entry.
    Drawn,
    /// [`Turn::Stays`].
    Stays,
    /// [`Turn::Lost`].
    Lost,
}

/// A find's type (ADV-PIK-009 to -012): its contents drawn when the game is built, emptied by
/// its first visit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Find {
    /// What it may hold, each with its chance in percent, in the draw's order (the Wagon: 10
    /// nothing, 40 an artifact, 50 resources).
    pub draws: Vec<(u8, FindContent)>,
    /// What its artifact turns into for a hero who cannot receive it; none for a find that holds
    /// no artifact.
    pub full_backpack: Option<Turn>,
    /// The visit asks first (the Warrior's Tomb).
    pub question: bool,
    /// The bonus every visit gives (the Warrior's Tomb's −3 morale), held or not.
    pub bonus: Option<OnceBonus>,
    /// It leaves the map at its visit instead of being emptied in place (E08 T72: the sea's
    /// pickups, ADV-PIK-014 to -016; their visits E08 T74a).
    pub leaves: bool,
}

/// [`Find`] as a pack defines it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindDef {
    /// The draws: at least one, each chance from 1, together 100; at most one resources entry.
    pub draws: Vec<(u8, FindContentDef)>,
    /// The full backpack's turn: given exactly when an entry may hold an artifact;
    /// [`TurnDef::Drawn`] only beside a resources entry and no fixed entry's artifact.
    pub full_backpack: Option<TurnDef>,
    /// The question.
    pub question: bool,
    /// The bonus, its mark below [`BattleBonuses::MARKS`].
    pub bonus: Option<OnceBonus>,
    /// Whether it leaves the map at its visit (E08 T72); one that does has no
    /// [`TurnDef::Stays`] (E08 T74a).
    pub leaves: bool,
}

/// The Pyramid's type (ADV-TCH-008, survey `E08-rewards.md` §7.5): its guards, its spell's level
/// and what a looted one costs a visitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pyramid {
    /// The first creature and its count, split as a monster's (40 Gold Golems; ADV-MON-012).
    pub split: (CreatureTypeId, u32),
    /// The second creature and its stacks' counts, unsplit (20 Diamond Golems as 2 stacks of 10).
    pub fixed: (CreatureTypeId, Vec<u32>),
    /// The level of the spell it holds, drawn when the game is built (5).
    pub level: u8,
    /// The bonus a looted Pyramid gives its visitor (−2 luck, mark 1).
    pub looted: OnceBonus,
}

/// [`Pyramid`] as a pack defines it: creatures by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyramidDef {
    /// The first creature, a core creature and no war machine, with a count from 1 to
    /// [`MAX_MONSTER_COUNT`].
    pub split: (ContentKey, u32),
    /// The second creature, as the first, with its stacks' counts: at least one stack, fewer than
    /// the army's slots, each count from 1 to [`MAX_MONSTER_COUNT`].
    pub fixed: (ContentKey, Vec<u32>),
    /// The spell's level, from 1.
    pub level: u8,
    /// The looted bonus, its mark below [`BattleBonuses::MARKS`].
    pub looted: OnceBonus,
}

/// A kind's parameters with their references resolved: an unknown reference is an error of its
/// own (the outer error), an invalid parameter the inner error's reason.
pub(super) type Compiled<T> = Result<Result<T, &'static str>, AdventureCompileError>;

/// A price or gift of resources, sorted by resource: distinct core resources, each amount from 1
/// to [`MAX_GAIN`].
pub(super) fn compile_resources(
    scope: &Scope<'_>,
    list: &[(ContentKey, u32)],
) -> Compiled<Resources> {
    let mut map = BTreeMap::new();
    for (key, amount) in list {
        map.insert(scope.resource(key)?, *amount);
    }
    Ok(if map.len() != list.len() {
        Err("a price or gift names a resource twice")
    } else if map.values().any(|&a| !(1..=MAX_GAIN).contains(&a)) {
        Err("a price or gift is 0 or above MAX_GAIN")
    } else {
        Ok(map.into_iter().collect())
    })
}

/// Artifact classes a draw takes: distinct, at least one, never Special.
fn check_classes(classes: &[ArtifactClass]) -> Result<(), &'static str> {
    let set = classes.iter().collect::<BTreeSet<_>>();
    if classes.is_empty() {
        Err("artifact classes are empty")
    } else if set.len() != classes.len() {
        Err("artifact classes name a class twice")
    } else if set.contains(&ArtifactClass::Special) {
        Err("artifact classes name Special")
    } else {
        Ok(())
    }
}

/// Artifact classes, if any: [`check_classes`] (an artifact type, ADV-OBJ-006).
pub(super) fn compile_artifact_classes(
    classes: Option<&Vec<ArtifactClass>>,
) -> Result<Option<Vec<ArtifactClass>>, &'static str> {
    classes
        .map(|list| check_classes(list).map(|()| list.clone()))
        .transpose()
}

/// A once-bonus's mark below [`BattleBonuses::MARKS`].
fn check_bonus(bonus: OnceBonus) -> Result<OnceBonus, &'static str> {
    if bonus.mark < BattleBonuses::MARKS {
        Ok(bonus)
    } else {
        Err("a bonus's mark is not below BattleBonuses::MARKS")
    }
}

/// Chances in percent: each from 1, together `total` at most (`exact`: exactly).
fn check_chances(chances: impl Iterator<Item = u8>, exact: bool) -> Result<u32, &'static str> {
    let mut sum = 0_u32;
    for chance in chances {
        if chance == 0 {
            return Err("a chance is 0");
        }
        sum += u32::from(chance);
    }
    if sum > 100 || exact && sum != 100 {
        Err("the chances do not add up to 100")
    } else {
        Ok(sum)
    }
}

/// [`ChestDef`]'s checks.
pub(super) fn compile_chest(scope: &Scope<'_>, def: &ChestDef) -> Compiled<Chest> {
    let resource = scope.resource(&def.resource)?;
    let check = || {
        let sum = check_chances(def.gold.iter().map(|&(chance, _)| chance), false)?;
        let amounts = def.gold.iter().map(|&(_, a)| a).chain([def.full_backpack]);
        if def.gold.is_empty() {
            Err("a chest has no gold")
        } else if amounts
            .clone()
            .any(|a| !(def.experience_offset..=MAX_GAIN).contains(&a) || a == 0)
        {
            Err("a chest's gold is 0, below the experience offset or above MAX_GAIN")
        } else if sum < 100 {
            check_classes(&def.artifact)
        } else if def.artifact.is_empty() {
            Ok(())
        } else {
            Err("a chest's chances leave no room for its artifact")
        }
    };
    Ok(check().map(|()| Chest {
        resource,
        gold: def.gold.clone(),
        artifact: def.artifact.clone(),
        full_backpack: def.full_backpack,
        experience_offset: def.experience_offset,
    }))
}

/// [`GiftDef`]'s checks.
pub(super) fn compile_gift(scope: &Scope<'_>, def: &GiftDef) -> Compiled<Gift> {
    let mut prices = Vec::with_capacity(def.prices.len());
    for price in &def.prices {
        match compile_resources(scope, price)? {
            Ok(price) => prices.push(price),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    let gives = match def.gives {
        Gives::Primary(gains) if gains == [0; 4] => Err("a gift's primary gains are all 0"),
        Gives::Choice { skills, .. } if skills[0] == skills[1] => {
            Err("a gift's choice names one skill twice")
        }
        Gives::Choice { amount: 0, .. } => Err("a gift's choice gives 0"),
        Gives::Experience(amount) if !(1..=MAX_GAIN).contains(&amount) => {
            Err("a gift's experience is 0 or above MAX_GAIN")
        }
        gives => Ok(gives),
    };
    Ok(match (gives, def.admits) {
        (Err(reason), _) => Err(reason),
        (Ok(_), Some(0)) => Err("a gift's condition is 0"),
        (Ok(gives), admits) => Ok(Gift {
            gives,
            prices,
            admits,
            leave: def.leave,
        }),
    })
}

/// [`FindDef`]'s checks.
pub(super) fn compile_find(scope: &Scope<'_>, def: &FindDef) -> Compiled<Find> {
    let mut draws = Vec::with_capacity(def.draws.len());
    for (chance, content) in &def.draws {
        let content = match content {
            FindContentDef::Nothing => Ok(FindContent::Nothing),
            FindContentDef::Artifact(classes) => {
                check_classes(classes).map(|()| FindContent::Artifact(classes.clone()))
            }
            FindContentDef::Resources { amount, resources } => {
                let list = resources
                    .iter()
                    .map(|r| scope.resource(r))
                    .collect::<Result<Vec<_>, _>>()?;
                let [low, high] = *amount;
                if !(1 <= low && low <= high && high <= MAX_GAIN) {
                    Err("a find's amount is not 1 <= low <= high <= MAX_GAIN")
                } else if list.is_empty() {
                    Err("a find's resources are empty")
                } else if list.iter().collect::<BTreeSet<_>>().len() != list.len() {
                    Err("a find's resources name a resource twice")
                } else {
                    Ok(FindContent::Resources {
                        amount: *amount,
                        resources: list,
                    })
                }
            }
            FindContentDef::Fixed {
                resources,
                artifact,
            } => match (
                compile_resources(scope, resources)?,
                compile_artifact_classes(artifact.as_ref()),
            ) {
                (Err(reason), _) | (_, Err(reason)) => Err(reason),
                (Ok(list), Ok(_)) if list.is_empty() => Err("a find's fixed resources are empty"),
                (Ok(resources), Ok(artifact)) => Ok(FindContent::Fixed {
                    resources,
                    artifact,
                }),
            },
        };
        match content {
            Ok(content) => draws.push((*chance, content)),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    let turn = match &def.full_backpack {
        None => None,
        Some(TurnDef::Give(list)) => match compile_resources(scope, list)? {
            Ok(list) if list.is_empty() => return Ok(Err("a find's turn gives nothing")),
            Ok(list) => Some(Turn::Give(list)),
            Err(reason) => return Ok(Err(reason)),
        },
        Some(TurnDef::Drawn) => Some(Turn::Drawn),
        Some(TurnDef::Stays) => Some(Turn::Stays),
        Some(TurnDef::Lost) => Some(Turn::Lost),
    };
    // A fixed entry's artifact comes with its resources, which a drawn turn would give instead.
    let fixed_artifact = draws.iter().any(|(_, c)| {
        matches!(
            c,
            FindContent::Fixed {
                artifact: Some(_),
                ..
            }
        )
    });
    let artifact = fixed_artifact
        || draws
            .iter()
            .any(|(_, c)| matches!(c, FindContent::Artifact(_)));
    let resources = draws
        .iter()
        .filter(|(_, c)| matches!(c, FindContent::Resources { .. }))
        .count();
    let check = || {
        check_chances(draws.iter().map(|&(chance, _)| chance), true)?;
        if resources > 1 {
            Err("a find has two resources entries")
        } else if artifact != turn.is_some() {
            Err("a find has a full backpack's turn exactly when it may hold an artifact")
        } else if turn == Some(Turn::Drawn) && resources == 0 {
            Err("a find's drawn turn has no resources entry")
        } else if turn == Some(Turn::Drawn) && fixed_artifact {
            Err("a find's drawn turn is beside a fixed entry's artifact")
        } else if def.leaves && turn == Some(Turn::Stays) {
            // E08 T74a: a find that leaves takes what it holds with it.
            Err("a find that leaves keeps its artifact")
        } else {
            def.bonus.map(check_bonus).transpose()
        }
    };
    Ok(check().map(|bonus| Find {
        draws,
        full_backpack: turn,
        question: def.question,
        bonus,
        leaves: def.leaves,
    }))
}

/// [`PyramidDef`]'s checks; its stacks against the army's slots come when every pack is in
/// ([`check_pyramid`]).
pub(super) fn compile_pyramid(scope: &Scope<'_>, def: &PyramidDef) -> Compiled<Pyramid> {
    let split = scope.creature(&def.split.0)?;
    let fixed = scope.creature(&def.fixed.0)?;
    let war_machine = scope.war_machine(split) || scope.war_machine(fixed);
    let counts = || core::iter::once(def.split.1).chain(def.fixed.1.iter().copied());
    Ok(if war_machine {
        Err("a Pyramid's guard is a war machine")
    } else if counts().any(|count| !(1..=MAX_MONSTER_COUNT).contains(&count)) {
        Err("a Pyramid's count is 0 or above MAX_MONSTER_COUNT")
    } else if def.fixed.1.is_empty() {
        Err("a Pyramid's second creature has no stack")
    } else if def.level == 0 {
        Err("a Pyramid's spell level is 0")
    } else {
        check_bonus(def.looted).map(|looted| Pyramid {
            split: (split, def.split.1),
            fixed: (fixed, def.fixed.1.clone()),
            level: def.level,
            looted,
        })
    })
}

/// A Pyramid's stacks, once the army's slots are known: the second creature's stacks leave the
/// first at least one slot.
pub(super) fn check_pyramid(pyramid: &Pyramid, army_slots: u8) -> Result<(), &'static str> {
    if pyramid.fixed.1.len() < usize::from(army_slots) {
        Ok(())
    } else {
        Err("a Pyramid's second creature fills every army slot")
    }
}

/// An artifact class's code in the hash: its place in [`ArtifactClass::ALL`].
const fn class_code(class: ArtifactClass) -> u8 {
    class as u8
}

/// Artifact classes in the hash: the length, then each class's code.
pub(super) fn hash_classes(classes: &[ArtifactClass], h: &mut StateHasher) {
    h.write_len(classes.len());
    for &class in classes {
        h.write_u8(class_code(class));
    }
}

/// Resources in the hash: the length, then each resource and amount.
pub(super) fn hash_resources(list: &[(ResourceId, u32)], h: &mut StateHasher) {
    h.write_len(list.len());
    for (resource, amount) in list {
        resource.hash_into(h);
        h.write_u32(*amount);
    }
}

impl CanonicalHash for OnceBonus {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_i32(i32::from(self.morale));
        h.write_i32(i32::from(self.luck));
        h.write_u8(self.mark);
    }
}

impl CanonicalHash for Chest {
    fn hash_into(&self, h: &mut StateHasher) {
        self.resource.hash_into(h);
        h.write_len(self.gold.len());
        for &(chance, amount) in &self.gold {
            h.write_u8(chance);
            h.write_u32(amount);
        }
        hash_classes(&self.artifact, h);
        h.write_u32(self.full_backpack);
        h.write_u32(self.experience_offset);
    }
}

/// A primary skill's code in the hash: its place in [`PrimarySkill::ALL`].
const fn primary_code(skill: PrimarySkill) -> u8 {
    skill as u8
}

impl CanonicalHash for Gift {
    /// What it gives (0 primary gains, 1 a choice: its skills' codes and gain, 2 experience,
    /// 3 a level), the prices as length and each, the condition (0 none, else 1 and it), whether
    /// one may leave.
    fn hash_into(&self, h: &mut StateHasher) {
        match self.gives {
            Gives::Primary(gains) => {
                h.write_u8(0);
                for gain in gains {
                    h.write_u8(gain);
                }
            }
            Gives::Choice { skills, amount } => {
                h.write_u8(1);
                h.write_u8(primary_code(skills[0]));
                h.write_u8(primary_code(skills[1]));
                h.write_u8(amount);
            }
            Gives::Experience(amount) => {
                h.write_u8(2);
                h.write_u32(amount);
            }
            Gives::Level => h.write_u8(3),
        }
        h.write_len(self.prices.len());
        for price in &self.prices {
            hash_resources(price, h);
        }
        match self.admits {
            None => h.write_u8(0),
            Some(level) => {
                h.write_u8(1);
                h.write_u8(level);
            }
        }
        h.write_bool(self.leave);
    }
}

impl CanonicalHash for Find {
    /// The draws as length and each (the chance, then 0 nothing, 1 an artifact's classes,
    /// 2 resources: the bounds and the resources; since part v13 3 fixed: the resources, then 0
    /// for no artifact, else 1 and its classes), the turn (0 none, 1 give and its resources,
    /// 2 drawn, 3 stays; since part v13 4 lost), the question, the bonus (0 none, else 1 and it);
    /// since part v13 whether it leaves.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.draws.len());
        for (chance, content) in &self.draws {
            h.write_u8(*chance);
            match content {
                FindContent::Nothing => h.write_u8(0),
                FindContent::Artifact(classes) => {
                    h.write_u8(1);
                    hash_classes(classes, h);
                }
                FindContent::Resources { amount, resources } => {
                    h.write_u8(2);
                    h.write_u32(amount[0]);
                    h.write_u32(amount[1]);
                    resources.hash_into(h);
                }
                FindContent::Fixed {
                    resources,
                    artifact,
                } => {
                    h.write_u8(3);
                    hash_resources(resources, h);
                    match artifact {
                        None => h.write_u8(0),
                        Some(classes) => {
                            h.write_u8(1);
                            hash_classes(classes, h);
                        }
                    }
                }
            }
        }
        match &self.full_backpack {
            None => h.write_u8(0),
            Some(Turn::Give(list)) => {
                h.write_u8(1);
                hash_resources(list, h);
            }
            Some(Turn::Drawn) => h.write_u8(2),
            Some(Turn::Stays) => h.write_u8(3),
            Some(Turn::Lost) => h.write_u8(4),
        }
        h.write_bool(self.question);
        match &self.bonus {
            None => h.write_u8(0),
            Some(bonus) => {
                h.write_u8(1);
                bonus.hash_into(h);
            }
        }
        h.write_bool(self.leaves);
    }
}

impl CanonicalHash for Pyramid {
    fn hash_into(&self, h: &mut StateHasher) {
        self.split.0.hash_into(h);
        h.write_u32(self.split.1);
        self.fixed.0.hash_into(h);
        h.write_len(self.fixed.1.len());
        for &count in &self.fixed.1 {
            h.write_u32(count);
        }
        h.write_u8(self.level);
        self.looted.hash_into(h);
    }
}
