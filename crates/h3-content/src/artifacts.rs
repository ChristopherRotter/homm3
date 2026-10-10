//! Artifacts (E04a T3b, ADR-0013 §3): equipment slots and artifact types, compiled from the
//! packs' definitions.
//!
//! A slot is a *kind* of equipment position with a count (two rings, five misc positions in SoD):
//! an artifact fits any position of its slot. A combination artifact names its parts; it occupies
//! its own slot and locks one position of every other part's slot (`locks`, derived here). What an
//! artifact does is a list of [`crate::HeroEffect`]s, like a skill level or a specialty; which of
//! a hero's artifacts count, and how often, is the derivation's business (`h3-heroes`, E04a T4 and
//! T10). Nothing here is read on the battle's hot path.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::effect::{self, CompiledHeroEffect};
use crate::error::CompileError;
use crate::flags::CreatureFlag;
use crate::heroes::{HeroContent, Refs, SkillId, distinct, invalid};
use crate::key::ContentKey;
use crate::pack::ContentPack;
use crate::ruleset::{CreatureType, CreatureTypeId, ResourceId};

/// Most positions one slot may have.
pub const MAX_SLOT_COUNT: u8 = 8;
/// Most equipment positions of all slots together.
pub const MAX_SLOT_POSITIONS: usize = 32;
/// Most parts of a combination artifact.
pub const MAX_PARTS: usize = 16;

/// An artifact's class (the original's S, T, N, J, R): its rarity for random placement and
/// treasure (adventure map, E08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactClass {
    /// Treasure (T).
    Treasure,
    /// Minor (N).
    Minor,
    /// Major (J).
    Major,
    /// Relic (R).
    Relic,
    /// Special (S): never placed at random (war machines, spell book, scroll, Grail).
    Special,
}

impl ArtifactClass {
    /// All values, in code order.
    pub const ALL: [Self; 5] = [
        Self::Treasure,
        Self::Minor,
        Self::Major,
        Self::Relic,
        Self::Special,
    ];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Treasure => "treasure",
            Self::Minor => "minor",
            Self::Major => "major",
            Self::Relic => "relic",
            Self::Special => "special",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// A compiled equipment slot: a kind of position on a hero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSlot {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Positions of this kind on a hero (1..=8).
    pub count: u8,
    /// Whether its artifacts may be carried in the backpack (not the war machines, the
    /// Catapult and the spell book; HER-ART-002).
    pub backpack: bool,
}

/// Handle of an artifact slot in a [`crate::Ruleset`].
pub type ArtifactSlotId = Handle<ArtifactSlot>;

/// A compiled artifact type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactType {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Class.
    pub class: ArtifactClass,
    /// Value (the original's gold cost); empty for artifacts without one.
    pub cost: Vec<(ResourceId, u32)>,
    /// The slot it is worn in; `None`: carried in the backpack only (the Grail).
    pub slot: Option<ArtifactSlotId>,
    /// The war machine the artifact places in its owner's battles.
    pub war_machine: Option<CreatureTypeId>,
    /// Its effects apply only while the hero has this secondary skill (any level).
    pub needs_skill: Option<SkillId>,
    /// Its own effects (a combination's parts add theirs).
    pub effects: Vec<CompiledHeroEffect>,
    /// A combination's parts, in pack order; empty for other artifacts.
    pub parts: Vec<ArtifactTypeId>,
    /// Derived for a combination: the slots it locks besides its own, one position per entry
    /// (its parts' slots, minus one position of its own slot).
    pub locks: Vec<ArtifactSlotId>,
}

impl ArtifactType {
    /// Whether it is a combination artifact.
    pub fn is_combination(&self) -> bool {
        !self.parts.is_empty()
    }
}

/// Handle of an artifact type in a [`crate::Ruleset`].
pub type ArtifactTypeId = Handle<ArtifactType>;

/// Resolves a cost against the closure's resources, rejecting a resource listed twice.
fn cost(
    refs: &Refs<'_>,
    list: &[(ContentKey, u32)],
) -> Result<Vec<(ResourceId, u32)>, CompileError> {
    let mut out: Vec<(ResourceId, u32)> = Vec::with_capacity(list.len());
    for (key, amount) in list {
        let id = refs.resolve(key, refs.resources, "resource")?;
        if out.iter().any(|(r, _)| *r == id) {
            return Err(CompileError::DuplicateKey(String::from(key.as_str())));
        }
        out.push((id, *amount));
    }
    Ok(out)
}

impl HeroContent {
    /// Compiles the slots, slot overrides and artifacts of one pack (after its skills).
    pub(crate) fn add_artifacts(
        &mut self,
        pack: &ContentPack,
        refs: &Refs<'_>,
    ) -> Result<(), CompileError> {
        for def in &pack.artifact_slots {
            self.slots.register(&def.key, refs.pack, "artifact slot")?;
            self.slots.items.push(ArtifactSlot {
                key: def.key.clone(),
                name: def.name.clone(),
                count: def.count,
                backpack: def.backpack,
            });
        }
        for o in &pack.artifact_slot_overrides {
            let id = refs.resolve(&o.target, &self.slots.index, "artifact slot")?;
            // Handles come from the index, which only holds pushed positions.
            self.slots
                .items
                .get_mut(id.index())
                .into_iter()
                .for_each(|slot| {
                    if let Some(name) = &o.name {
                        slot.name.clone_from(name);
                    }
                    if let Some(count) = o.count {
                        slot.count = count;
                    }
                });
        }
        // Combinations may name parts defined later in the pack, so all are registered first.
        let first = self.artifacts.items.len();
        for def in &pack.artifacts {
            self.artifacts.register(&def.key, refs.pack, "artifact")?;
        }
        for def in &pack.artifacts {
            let slot = def
                .slot
                .as_ref()
                .map(|k| refs.resolve(k, &self.slots.index, "artifact slot"))
                .transpose()?;
            let war_machine = def
                .war_machine
                .as_ref()
                .map(|k| refs.resolve(k, refs.creatures, "creature"))
                .transpose()?;
            let needs_skill = def
                .needs_skill
                .as_ref()
                .map(|k| refs.resolve(k, &self.skills.index, "secondary skill"))
                .transpose()?;
            let artifact = ArtifactType {
                key: def.key.clone(),
                name: def.name.clone(),
                class: def.class,
                cost: cost(refs, &def.cost)?,
                slot,
                war_machine,
                needs_skill,
                effects: self.effects(refs, &def.effects)?,
                parts: distinct(refs, &def.parts, &self.artifacts.index, "artifact")?,
                locks: Vec::new(),
            };
            self.artifacts.items.push(artifact);
        }
        // A combination locks one position of every part's slot except one of its own.
        let locks: Vec<Vec<ArtifactSlotId>> = self
            .artifacts
            .items
            .get(first..)
            .unwrap_or_default()
            .iter()
            .map(|a| self.locks(a))
            .collect::<Result<_, _>>()?;
        for (a, l) in self.artifacts.items.iter_mut().skip(first).zip(locks) {
            a.locks = l;
        }
        Ok(())
    }

    /// The slots the combination `a` locks (empty for other artifacts).
    fn locks(&self, a: &ArtifactType) -> Result<Vec<ArtifactSlotId>, CompileError> {
        if !a.is_combination() {
            return Ok(Vec::new());
        }
        let own = a
            .slot
            .ok_or_else(|| invalid("artifact", &a.key, "a combination needs a slot"))?;
        let mut locks = Vec::with_capacity(a.parts.len());
        let mut own_seen = false;
        for &part in &a.parts {
            let slot = self
                .artifacts
                .get(part)
                .and_then(|p| p.slot)
                .ok_or_else(|| invalid("artifact", &a.key, "combination parts need a slot"))?;
            if slot == own && !own_seen {
                own_seen = true;
            } else {
                locks.push(slot);
            }
        }
        if own_seen {
            Ok(locks)
        } else {
            Err(invalid(
                "artifact",
                &a.key,
                "a combination's slot must be the slot of one of its parts",
            ))
        }
    }

    /// Validates slots and artifacts once all packs are loaded (slot overrides of later packs
    /// may change what fits).
    pub(crate) fn validate_artifacts(
        &self,
        creatures: &[CreatureType],
    ) -> Result<(), CompileError> {
        for s in &self.slots.items {
            if !(1..=MAX_SLOT_COUNT).contains(&s.count) {
                return Err(invalid("artifact slot", &s.key, "count must be 1..=8"));
            }
        }
        let positions: usize = self.slots.items.iter().map(|s| usize::from(s.count)).sum();
        if positions > MAX_SLOT_POSITIONS {
            return Err(CompileError::TooManyEntries("artifact slot position"));
        }
        let machine = |id: CreatureTypeId| {
            creatures
                .get(id.index())
                .is_some_and(|c| c.flags.contains(CreatureFlag::WarMachine))
        };
        // Part (index) → the combination (index) that names it.
        let mut part_of: BTreeMap<usize, usize> = BTreeMap::new();
        for (i, a) in self.artifacts.items.iter().enumerate() {
            let parts: Vec<&ArtifactType> = a
                .parts
                .iter()
                .filter_map(|&p| self.artifacts.get(p))
                .collect();
            let reason = if let Some(reason) = effect::list_error(&a.effects, false) {
                Some(reason)
            } else if let Some(reason) = self.spell_effect_error(&a.effects) {
                Some(reason)
            } else if a.war_machine.is_some_and(|m| !machine(m)) {
                Some("war_machine must have flag war_machine")
            } else if a.war_machine.is_some() && (a.slot.is_none() || a.is_combination()) {
                Some("a war machine artifact needs a slot and cannot be a combination")
            } else if a.is_combination() && !(2..=MAX_PARTS).contains(&a.parts.len()) {
                Some("a combination needs 2..=16 parts")
            } else if parts.iter().any(|p| p.is_combination()) {
                Some("combination parts cannot be combinations")
            } else if parts.iter().any(|p| p.war_machine.is_some()) {
                Some("war machine artifacts cannot be combination parts")
            } else if self.slots.items.iter().enumerate().any(|(s, slot)| {
                let needed = parts
                    .iter()
                    .filter(|p| p.slot.is_some_and(|ps| ps.index() == s))
                    .count();
                needed > usize::from(slot.count)
            }) {
                Some("the parts need more positions of a slot than the slot has")
            } else {
                None
            };
            if let Some(reason) = reason {
                return Err(invalid("artifact", &a.key, reason));
            }
            for &p in &a.parts {
                if part_of.insert(p.index(), i).is_some() {
                    let key = self.artifacts.get(p).map_or(&a.key, |q| &q.key);
                    return Err(invalid(
                        "artifact",
                        key,
                        "an artifact can be a part of one combination only",
                    ));
                }
            }
        }
        Ok(())
    }

    /// The combination `part` belongs to, if any.
    pub(crate) fn combination_of(&self, part: ArtifactTypeId) -> Option<ArtifactTypeId> {
        self.artifacts
            .items
            .iter()
            .position(|a| a.parts.contains(&part))
            .and_then(Handle::from_index)
    }

    /// Slots (key, name, count, backpack flag); artifacts (key, name, class, cost, slot, war machine, needed
    /// skill, effects, parts). Locks are derived from these and not hashed.
    pub(crate) fn hash_artifacts(&self, h: &mut StateHasher) {
        h.write_len(self.slots.items.len());
        for s in &self.slots.items {
            s.key.as_str().hash_into(h);
            s.name.as_str().hash_into(h);
            h.write_u8(s.count);
            h.write_bool(s.backpack);
        }
        h.write_len(self.artifacts.items.len());
        for a in &self.artifacts.items {
            a.key.as_str().hash_into(h);
            a.name.as_str().hash_into(h);
            h.write_u8(a.class as u8);
            h.write_len(a.cost.len());
            for (r, amount) in &a.cost {
                r.hash_into(h);
                h.write_u32(*amount);
            }
            a.slot.hash_into(h);
            a.war_machine.hash_into(h);
            a.needs_skill.hash_into(h);
            a.effects.hash_into(h);
            a.parts.hash_into(h);
        }
    }
}

#[cfg(test)]
mod tests;
