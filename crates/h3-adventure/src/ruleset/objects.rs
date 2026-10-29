//! Object types (ADR-0016 point 4, E08 T12a): what the import maps the original's classes to. A
//! type names its kind, a closed enum with typed parameters, and two visit properties; the packs
//! define the types (`adventure.toml`, `[[object_type]]`) and this module compiles them into the
//! ruleset part. The rules that read a kind come with the tasks that build them (pickups T13, the
//! Cartographer T14, the observatories, tents and guards T15a, the gates T15b); the setup draws
//! random contents (T12b). Since T20a2a the wandering monsters and the special grounds, which no
//! visit reads yet.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{
    ArtifactClass, ContentKey, CreatureFlag, CreatureTypeId, ResourceId, Ruleset, SkillId,
    TerrainId,
};
use h3_kernel::{CanonicalHash, Handle, StateHasher};

use super::rewards::{
    Chest, ChestDef, Find, FindDef, Gift, GiftDef, Pyramid, PyramidDef, Resources, check_pyramid,
    compile_artifact_classes, compile_chest, compile_find, compile_gift, compile_pyramid,
    compile_resources, hash_classes, hash_resources,
};
use super::{AdventureCompileError, check_namespace, insert_unique};

/// The key colours of Keymaster's tents, border guards and border gates (survey `E08-objects.md`
/// §6: eight tents' colours, VCMI 1.7.5 `config/objects/moddables.json`); a key is one of them.
pub const KEY_COLOURS: u8 = 8;

/// The largest amount of a pickup before its type's multiplier: a map's fixed amount and a random
/// amount's bounds. The original keeps a pile's amount in the low 19 bits of its tile's word (the
/// setup tests `& 0x7FFFF`, 0x4C1AE4; survey `E08-objects.md` §1); the shipped maps' largest is
/// 1 000.
pub const MAX_PICKUP_AMOUNT: u32 = 0x7_FFFF;

/// The largest single gain or price: a treasury's cap, 2³¹ − 1 (ADR-0017 point 1). A pickup's
/// largest amount times its multiplier stays within it.
pub const MAX_GAIN: u32 = 0x7FFF_FFFF;

/// How a hero visits an object's visitable tile (ADR-0016 point 4, ADV-OBJ-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visit {
    /// A step onto the visitable tile enters it, visits the object and ends the move.
    Enter,
    /// A step toward the visitable tile visits the object from the tile the hero stands on.
    Neighbour,
}

/// What a pickup holds, as its type draws it when the game is built (ADR-0016 point 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pickup {
    /// One resource: the map's amount, or one drawn between the bounds, times the multiplier (a
    /// resource pile, ADV-PIK-001).
    Resource {
        /// The resource.
        resource: ResourceId,
        /// The bounds of a drawn amount, inclusive.
        amount: [u32; 2],
        /// What an amount is multiplied by (gold counts in hundreds).
        multiplier: NonZeroU32,
    },
    /// Becomes one of these types, each a [`Pickup::Resource`], with equal chances; the map's
    /// amount carries over (a random resource, ADV-PIK-002).
    OneOf(Vec<ObjectTypeId>),
    /// An amount drawn between the bounds, then one of the resources: that many of it, and the
    /// amount times the multiplier of the paired resource (a campfire, ADV-PIK-003).
    Paired {
        /// The bounds of the amount, inclusive.
        amount: [u32; 2],
        /// The resources one is drawn from, with equal chances.
        resources: Vec<ResourceId>,
        /// The resource given beside it.
        paired: ResourceId,
        /// What the amount is multiplied by for the paired resource.
        multiplier: NonZeroU32,
    },
}

/// An object type's behaviour: a closed set of kinds with typed parameters (ADR-0016 point 4),
/// growing by variants with their rules. A class without a type is a blocker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKind {
    /// Resources gained, then the object leaves the map.
    Pickup(Pickup),
    /// Reveals every tile of these terrains on every level for a price, once per player and type
    /// (a Cartographer, ADV-VIS-001).
    RevealTerrain {
        /// The terrains, in the core's order.
        terrains: Vec<TerrainId>,
        /// The price, by resource in the core's order; each amount at least 1.
        price: Vec<(ResourceId, u32)>,
    },
    /// Reveals the tiles within the radius of the visited tile, every visit (the Redwood
    /// Observatory, the Pillar of Fire; ADV-VIS-002).
    RevealArea {
        /// The radius.
        radius: u8,
    },
    /// Gives its visitor's owner the key of its colour (ADV-VIS-003).
    KeyTent {
        /// The key colour, below [`KEY_COLOURS`].
        key: u8,
    },
    /// Leaves the map for everyone when a holder of its key accepts to pass (ADV-VIS-004).
    KeyGuard {
        /// The key colour, below [`KEY_COLOURS`].
        key: u8,
    },
    /// Lets the heroes of its key's holders pass (ADV-VIS-005).
    KeyGate {
        /// The key colour, below [`KEY_COLOURS`].
        key: u8,
    },
    /// A wandering monster (ADV-MON-001, E08 T20a2a): its creature the map's or, with levels,
    /// drawn when the game is built among the creatures of those levels (ADV-MON-003). Its visit
    /// is its answer to the hero (ADV-MON-007, E08 T20b1).
    Monster {
        /// The creature levels, inclusive, a random monster's creature is drawn from; none for
        /// a monster whose creature the map names.
        levels: Option<[u8; 2]>,
    },
    /// A special ground (ADV-GRD-001, E08 T20a2a): a battle on a tile it lies on is fought on
    /// this battlefield; never visited.
    Ground {
        /// The battlefield (a battle part's key, checked when the game bundles the parts).
        battlefield: ContentKey,
        /// Whether a step from a tile it lies on, as the tile's top ground, costs no army the
        /// native price (Cursed Ground, ADV-MOV-004, E08 T40).
        no_native: bool,
    },
    /// A treasure chest (ADV-PIK-005, E08 T61a): gold or an artifact, drawn when the game is
    /// built.
    Chest(Chest),
    /// An artifact as a pickup (ADV-PIK-006, ADV-PIK-007, E08 T61a): the map's artifact, or one
    /// drawn among these classes when the game is built (a random artifact, ADV-OBJ-006).
    Artifact {
        /// The classes of a random artifact; none for a placed one, whose artifact the map names.
        classes: Option<Vec<ArtifactClass>>,
    },
    /// A Spell Scroll as a pickup, its spell the map's (ADV-PIK-008, E08 T61a).
    Scroll,
    /// A once-per-hero gift (ADV-TCH-001 to -003, E08 T61a).
    Gift(Gift),
    /// A Shrine (ADV-TCH-004, E08 T61a): a spell of this level, the map's or drawn.
    Shrine {
        /// The spell's level.
        level: u8,
    },
    /// A Witch Hut (ADV-TCH-005, E08 T61a): a skill drawn among the map's mask; an RoE map has
    /// none, and its huts draw among these skills by the profile (A-21, `ADV-QUIRK-21`).
    WitchHut {
        /// The skills under `faithful`: the original's fixed mask 0xEFBF (14 skills).
        roe: Vec<SkillId>,
        /// The skills under `strict-fixed`: every skill but Leadership and Necromancy.
        roe_fixed: Vec<SkillId>,
    },
    /// A Scholar (ADV-TCH-006, E08 T61a): its gift the map's or drawn.
    Scholar,
    /// A University (ADV-TCH-007, E08 T61a): skills drawn when the game is built, each sold at
    /// the price.
    University {
        /// How many skills it offers.
        offers: u8,
        /// The price of each.
        price: Resources,
    },
    /// A find, emptied by its first visit (ADV-PIK-009 to -012, E08 T61a).
    Find(Find),
    /// Pandora's Box (ADV-PIK-013, E08 T61a): the map's reward record behind a question.
    PandoraBox,
    /// An event (ADV-OBJ-007, E08 T61a): the map's reward record, fired by a step onto its tile;
    /// never visited through [`ObjectKind::effect`] (design §12).
    Event,
    /// The Pyramid (ADV-TCH-008, E08 T61a).
    Pyramid(Pyramid),
    /// A message (E08 T72, design §5: the Ocean Bottle, ADV-PIK-017): its visit tells the
    /// visitor's owner which object's message it read (the map's text is the client's); its
    /// visit is T74's.
    Message {
        /// It leaves the map at its visit (the Ocean Bottle; the Sign, a later increment's, stays).
        leaves: bool,
    },
    /// A ground of the sea (E08 T72, design §5: Favorable Winds): lies on its picture's tiles,
    /// never visited, no battlefield; a boat's step from a tile it tops costs the part's
    /// `sea_ground` share of its price (ADV-SEA-004, T73).
    SeaGround,
    /// A hole (E08 T72, design §5): lies on its picture's tiles, never visited, blocks nothing;
    /// no landing onto a tile it lies on (ADV-SEA-003, T73).
    Hole,
}

/// What a visit of an object does, for the kinds whose visit is built ([`ObjectKind::effect`]):
/// the kind's parameters a visit reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Effect<'k> {
    /// Its contents gained, then it leaves the map (a pickup, ADV-PIK-001 to ADV-PIK-003).
    Pickup,
    /// An offer of its terrains' reveal for this price, once per player and type (a
    /// Cartographer, ADV-VIS-001).
    Chart(&'k [(ResourceId, u32)]),
    /// The tiles within this radius of the visited tile revealed (an observatory, ADV-VIS-002).
    Reveal(u8),
    /// This key for the visitor's owner (a Keymaster's tent, ADV-VIS-003).
    Tent(u8),
    /// For a holder of this key, an offer to pass (a border guard, ADV-VIS-004).
    Guard(u8),
    /// For a holder of this key, a tile like any other, crossed without a visit; for another
    /// player, a visit that does nothing (a border gate, ADV-VIS-005).
    Gate(u8),
    /// The monster's answer to the hero (a wandering monster, ADV-MON-007, E08 T20b1).
    Monster,
    /// Its gold or artifact (a treasure chest, ADV-PIK-005, E08 T62a).
    Chest(&'k Chest),
    /// Its artifact, picked up (an artifact or a Spell Scroll, ADV-PIK-006 to -008, E08 T62a).
    Artifact,
    /// Its gift, once per hero (ADV-TCH-001 to -003, E08 T62a).
    Gift(&'k Gift),
    /// Its spell (a Shrine, ADV-TCH-004, E08 T62a).
    Shrine,
    /// Its skill (a Witch Hut, ADV-TCH-005, E08 T62a).
    WitchHut,
    /// Its gift, then it leaves the map (a Scholar, ADV-TCH-006, E08 T62a).
    Scholar,
    /// Its skills, each for this price (a University, ADV-TCH-007, E08 T62a).
    University(&'k Resources),
    /// Its contents, once (a find, ADV-PIK-009 to -012, E08 T62a).
    Find(&'k Find),
    /// Its question, then its guards or its reward record (Pandora's Box, ADV-PIK-013, E08
    /// T62b1).
    PandoraBox,
    /// Its question at every visit, then its guards or a looted one's luck (the Pyramid,
    /// ADV-TCH-008, E08 T62b1).
    Pyramid,
}

impl ObjectKind {
    /// What a visit of this kind does, if its visit is built (ADR-0016 point 4; ADV-OBJ-003):
    /// the one place a task that builds a kind's visit extends. A step onto the visitable tile of
    /// an object whose visit is not built is rejected (`Visit`). E08 T13b: the pickups; T14a and
    /// T14b: the Cartographer, entered, with its offer; T15a: the observatories, the tents and the
    /// guards with their offer; T15b: the gates, which are not visited by their key's holders;
    /// T20b1: the wandering monsters, whose visit is their answer; T62a: the chests, artifacts,
    /// scrolls, gifts, Shrines, Witch Huts, Scholars, Universities and finds; T62b1: Pandora's
    /// Box and the Pyramid.
    pub(crate) fn effect(&self) -> Option<Effect<'_>> {
        match self {
            Self::Pickup(_) => Some(Effect::Pickup),
            Self::RevealTerrain { price, .. } => Some(Effect::Chart(price)),
            Self::RevealArea { radius } => Some(Effect::Reveal(*radius)),
            Self::KeyTent { key } => Some(Effect::Tent(*key)),
            Self::KeyGuard { key } => Some(Effect::Guard(*key)),
            Self::KeyGate { key } => Some(Effect::Gate(*key)),
            Self::Monster { .. } => Some(Effect::Monster),
            Self::Chest(chest) => Some(Effect::Chest(chest)),
            Self::Artifact { .. } | Self::Scroll => Some(Effect::Artifact),
            Self::Gift(gift) => Some(Effect::Gift(gift)),
            Self::Shrine { .. } => Some(Effect::Shrine),
            Self::WitchHut { .. } => Some(Effect::WitchHut),
            Self::Scholar => Some(Effect::Scholar),
            Self::University { price, .. } => Some(Effect::University(price)),
            Self::Find(find) if !find.leaves => Some(Effect::Find(find)),
            Self::PandoraBox => Some(Effect::PandoraBox),
            Self::Pyramid(_) => Some(Effect::Pyramid),
            // E08 T61a: typed; an event is fired by a step, never visited (T62b2). E08 T72: the
            // grounds of the sea and the holes are never visited; the sea's pickups (a find that
            // leaves) and the Ocean Bottle are visited from a boat only, built by T74 (design
            // §11), so until then a step onto them is refused as any unbuilt visit's.
            Self::Ground { .. }
            | Self::Event
            | Self::Find(_)
            | Self::Message { .. }
            | Self::SeaGround
            | Self::Hole => None,
        }
    }

    /// The terrains and price of a [`Self::RevealTerrain`] (a Cartographer, ADV-VIS-001); none
    /// for every other kind.
    pub(crate) fn reveal_terrain(&self) -> Option<TerrainReveal<'_>> {
        match self {
            Self::RevealTerrain { terrains, price } => Some((terrains, price)),
            _ => None,
        }
    }
}

/// A Cartographer's terrains and price ([`ObjectKind::reveal_terrain`]).
pub(crate) type TerrainReveal<'a> = (&'a [TerrainId], &'a [(ResourceId, u32)]);

/// A compiled object type (ADV-OBJ-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// How it is visited.
    pub visit: Visit,
    /// Whether it may be visited from the three tiles above its visitable tile; when not, a
    /// walking hero also leaves that tile toward none of them (ADV-OBJ-004).
    pub from_above: bool,
    /// Its behaviour.
    pub kind: ObjectKind,
}

impl ObjectType {
    /// Whether a map may fix this type's amount: a resource pile's or a random resource's.
    pub const fn takes_amount(&self) -> bool {
        matches!(
            self.kind,
            ObjectKind::Pickup(Pickup::Resource { .. } | Pickup::OneOf(_))
        )
    }
}

/// Handle of an object type in an [`super::AdventureRuleset`].
pub type ObjectTypeId = Handle<ObjectType>;

/// A pickup as a pack defines it: [`Pickup`] with keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickupDef {
    /// [`Pickup::Resource`].
    Resource {
        /// A core resource.
        resource: ContentKey,
        /// The bounds of a drawn amount, from 1 to [`MAX_PICKUP_AMOUNT`].
        amount: [u32; 2],
        /// The multiplier; [`MAX_PICKUP_AMOUNT`] times it stays within [`MAX_GAIN`].
        multiplier: NonZeroU32,
    },
    /// [`Pickup::OneOf`]: types of resource pickups, distinct, at least one.
    OneOf(Vec<ContentKey>),
    /// [`Pickup::Paired`].
    Paired {
        /// The bounds of the amount, as for a resource.
        amount: [u32; 2],
        /// Distinct core resources, at least one.
        resources: Vec<ContentKey>,
        /// A core resource.
        paired: ContentKey,
        /// The multiplier, as for a resource.
        multiplier: NonZeroU32,
    },
}

/// An object kind as a pack defines it: [`ObjectKind`] with keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKindDef {
    /// [`ObjectKind::Pickup`].
    Pickup(PickupDef),
    /// [`ObjectKind::RevealTerrain`]: distinct core terrains, at least one; a price of distinct
    /// resources, each amount from 1 to [`MAX_GAIN`].
    RevealTerrain {
        /// The terrains.
        terrains: Vec<ContentKey>,
        /// The price.
        price: Vec<(ContentKey, u32)>,
    },
    /// [`ObjectKind::RevealArea`]: a radius of at least 1.
    RevealArea {
        /// The radius.
        radius: u8,
    },
    /// [`ObjectKind::KeyTent`].
    KeyTent {
        /// The key colour.
        key: u8,
    },
    /// [`ObjectKind::KeyGuard`].
    KeyGuard {
        /// The key colour.
        key: u8,
    },
    /// [`ObjectKind::KeyGate`]: its type visited from above (its key's holders cross it).
    KeyGate {
        /// The key colour.
        key: u8,
    },
    /// [`ObjectKind::Monster`]: its type visited from a neighbour; levels from 1, low to high.
    Monster {
        /// The levels of a random monster's creature.
        levels: Option<[u8; 2]>,
    },
    /// [`ObjectKind::Ground`]: a battlefield key of the pack's dependency closure.
    Ground {
        /// The battlefield.
        battlefield: ContentKey,
        /// Whether it takes the native price away (E08 T40).
        no_native: bool,
    },
    /// [`ObjectKind::Chest`].
    Chest(ChestDef),
    /// [`ObjectKind::Artifact`]: a random artifact's classes distinct, at least one, never
    /// Special.
    Artifact {
        /// The classes.
        classes: Option<Vec<ArtifactClass>>,
    },
    /// [`ObjectKind::Scroll`].
    Scroll,
    /// [`ObjectKind::Gift`].
    Gift(GiftDef),
    /// [`ObjectKind::Shrine`]: a level from 1.
    Shrine {
        /// The level.
        level: u8,
    },
    /// [`ObjectKind::WitchHut`]: core skills, each list distinct.
    WitchHut {
        /// Under `faithful`.
        roe: Vec<ContentKey>,
        /// Under `strict-fixed`.
        roe_fixed: Vec<ContentKey>,
    },
    /// [`ObjectKind::Scholar`].
    Scholar,
    /// [`ObjectKind::University`]: offers from 1; a price of distinct core resources, each
    /// amount from 1 to [`MAX_GAIN`].
    University {
        /// The offers.
        offers: u8,
        /// The price.
        price: Vec<(ContentKey, u32)>,
    },
    /// [`ObjectKind::Find`].
    Find(FindDef),
    /// [`ObjectKind::PandoraBox`].
    PandoraBox,
    /// [`ObjectKind::Event`].
    Event,
    /// [`ObjectKind::Pyramid`].
    Pyramid(PyramidDef),
    /// [`ObjectKind::Message`] (E08 T72).
    Message {
        /// Whether it leaves at its visit.
        leaves: bool,
    },
    /// [`ObjectKind::SeaGround`] (E08 T72).
    SeaGround,
    /// [`ObjectKind::Hole`] (E08 T72).
    Hole,
}

/// An object type as a pack defines it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectTypeDef {
    /// Key, in the pack's namespace.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// How it is visited.
    pub visit: Visit,
    /// Whether it may be visited from above.
    pub from_above: bool,
    /// Its behaviour.
    pub kind: ObjectKindDef,
}

/// The object types being compiled, by handle and by key.
#[derive(Debug, Default)]
pub(super) struct ObjectTables {
    pub(super) types: Vec<ObjectType>,
    pub(super) index: BTreeMap<String, ObjectTypeId>,
}

/// What a pack's definitions may reference: the core, the pack and its dependency closure.
pub(super) struct Scope<'a> {
    pub(super) core: &'a Ruleset,
    pub(super) pack: &'a str,
    pub(super) closure: &'a BTreeSet<String>,
}

impl Scope<'_> {
    /// Rejects a key outside the pack's dependency closure.
    fn reach(&self, key: &ContentKey) -> Result<(), AdventureCompileError> {
        if self.closure.contains(key.pack()) {
            Ok(())
        } else {
            Err(AdventureCompileError::ReferenceOutsideDependencies {
                key: String::from(key.as_str()),
                pack: String::from(self.pack),
            })
        }
    }

    pub(super) fn resource(&self, key: &ContentKey) -> Result<ResourceId, AdventureCompileError> {
        self.reach(key)?;
        self.core
            .resources()
            .iter()
            .position(|r| r.key == *key)
            .and_then(ResourceId::from_index)
            .ok_or_else(|| AdventureCompileError::UnknownResource(String::from(key.as_str())))
    }

    fn terrain(&self, key: &ContentKey) -> Result<TerrainId, AdventureCompileError> {
        self.reach(key)?;
        self.core
            .terrain_by_key(key.as_str())
            .ok_or_else(|| AdventureCompileError::UnknownTerrain(String::from(key.as_str())))
    }

    pub(super) fn creature(
        &self,
        key: &ContentKey,
    ) -> Result<CreatureTypeId, AdventureCompileError> {
        self.reach(key)?;
        self.core
            .creature_by_key(key.as_str())
            .ok_or_else(|| AdventureCompileError::UnknownCreature(String::from(key.as_str())))
    }

    /// Whether the creature is a war machine.
    pub(super) fn war_machine(&self, creature: CreatureTypeId) -> bool {
        self.core
            .creature(creature)
            .is_some_and(|c| c.flags.contains(CreatureFlag::WarMachine))
    }

    fn skill(&self, key: &ContentKey) -> Result<SkillId, AdventureCompileError> {
        self.reach(key)?;
        self.core
            .skill_by_key(key.as_str())
            .ok_or_else(|| AdventureCompileError::UnknownSkill(String::from(key.as_str())))
    }

    /// Distinct skills, in the list's order.
    fn skills(
        &self,
        keys: &[ContentKey],
    ) -> Result<Result<Vec<SkillId>, &'static str>, AdventureCompileError> {
        let list = keys
            .iter()
            .map(|key| self.skill(key))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            if list.iter().collect::<BTreeSet<_>>().len() == list.len() {
                Ok(list)
            } else {
                Err("skills name a skill twice")
            },
        )
    }
}

/// Adds one pack's object types: first every key (a type may name a later type of its own pack),
/// then each kind.
pub(super) fn compile_object_types(
    core: &Ruleset,
    pack: &str,
    closure: &BTreeSet<String>,
    defs: &[ObjectTypeDef],
    tables: &mut ObjectTables,
) -> Result<(), AdventureCompileError> {
    for (offset, def) in defs.iter().enumerate() {
        check_namespace(&def.key, pack)?;
        let handle = ObjectTypeId::from_index(tables.types.len() + offset)
            .ok_or(AdventureCompileError::TooManyEntries("object_type"))?;
        insert_unique(&mut tables.index, &def.key, handle)?;
    }
    let scope = Scope {
        core,
        pack,
        closure,
    };
    for def in defs {
        let invalid = |reason| AdventureCompileError::InvalidObjectType {
            key: String::from(def.key.as_str()),
            reason,
        };
        let kind = compile_kind(&scope, &tables.index, &def.kind)?.map_err(invalid)?;
        // Its key's holders cross a gate (ADV-VIS-005), so a walking hero also leaves its tile
        // upward: the original's class 212 is on the "visitable from above" list (survey
        // `E08-objects.md` §2.3), and the reachable-tiles query tests ADV-OBJ-004 on a path's
        // first tile only.
        if matches!(kind, ObjectKind::KeyGate { .. }) && !def.from_above {
            return Err(invalid("a key gate is not visited from above"));
        }
        // A guard or a gate blocks the players without its key: an entered one would let such a
        // hero stand on it and walk on past it (survey `E08-objects.md` §2.1: classes 9 and 212
        // are visited from a neighbour; the review of E08 T15).
        if matches!(
            kind,
            ObjectKind::KeyGuard { .. } | ObjectKind::KeyGate { .. }
        ) && def.visit != Visit::Neighbour
        {
            return Err(invalid(
                "a key guard or gate is not visited from a neighbour",
            ));
        }
        // A monster is attacked from the tile beside it, which the hero keeps (survey
        // `E08-battles.md` §2.3: class 54 is on the original's list of objects visited from a
        // neighbour, as VCMI's `CGCreature` is blocking-visitable).
        if matches!(kind, ObjectKind::Monster { .. }) && def.visit != Visit::Neighbour {
            return Err(invalid("a monster is not visited from a neighbour"));
        }
        tables.types.push(ObjectType {
            key: def.key.clone(),
            name: def.name.clone(),
            visit: def.visit,
            from_above: def.from_above,
            kind,
        });
    }
    Ok(())
}

/// Checks, once every pack is in, that each [`Pickup::OneOf`] names resource pickups only, that
/// each random monster has a creature to draw ([`random_monsters`]) and that a Pyramid's second
/// creature leaves its first a slot of the army's (E08 T61a).
pub(super) fn check_object_types(
    core: &Ruleset,
    types: &[ObjectType],
    excluded: &BTreeSet<CreatureTypeId>,
    army_slots: u8,
) -> Result<(), AdventureCompileError> {
    for ty in types {
        if let ObjectKind::Pyramid(pyramid) = &ty.kind {
            check_pyramid(pyramid, army_slots).map_err(|reason| {
                AdventureCompileError::InvalidObjectType {
                    key: String::from(ty.key.as_str()),
                    reason,
                }
            })?;
        }
        if let ObjectKind::Monster {
            levels: Some(levels),
        } = &ty.kind
            && random_monsters(core, excluded, *levels).next().is_none()
        {
            return Err(AdventureCompileError::InvalidObjectType {
                key: String::from(ty.key.as_str()),
                reason: "no creature of its levels may be drawn",
            });
        }
        if let ObjectKind::Pickup(Pickup::OneOf(choices)) = &ty.kind {
            let resource = |id: &ObjectTypeId| {
                types
                    .get(usize::from(id.raw()))
                    .is_some_and(|t| matches!(t.kind, ObjectKind::Pickup(Pickup::Resource { .. })))
            };
            if !choices.iter().all(resource) {
                return Err(AdventureCompileError::InvalidObjectType {
                    key: String::from(ty.key.as_str()),
                    reason: "one_of names a type that is not a resource pickup",
                });
            }
        }
    }
    Ok(())
}

/// The creatures a random monster of these levels is drawn from (ADV-MON-003), in the core's
/// order: each creature whose level lies between them, inclusive, neither a war machine nor
/// excluded by a pack (`not_random_monsters`; the original draws among the creature ids 0 to 144
/// without the expansions' six, 132 to 137, survey `E08-battles.md` §1.4).
pub(crate) fn random_monsters<'a>(
    core: &'a Ruleset,
    excluded: &'a BTreeSet<CreatureTypeId>,
    [low, high]: [u8; 2],
) -> impl Iterator<Item = CreatureTypeId> + 'a {
    (0..core.creature_count())
        .filter_map(CreatureTypeId::from_index)
        .filter(move |&id| {
            core.creature(id).is_some_and(|c| {
                (low..=high).contains(&c.level) && !c.flags.contains(CreatureFlag::WarMachine)
            }) && !excluded.contains(&id)
        })
}

/// A kind with its references resolved: an unknown reference is an error of its own; an invalid
/// parameter is the inner error's reason.
fn compile_kind(
    scope: &Scope<'_>,
    index: &BTreeMap<String, ObjectTypeId>,
    def: &ObjectKindDef,
) -> Result<Result<ObjectKind, &'static str>, AdventureCompileError> {
    Ok(match def {
        ObjectKindDef::Pickup(PickupDef::Resource {
            resource,
            amount,
            multiplier,
        }) => {
            let resource = scope.resource(resource)?;
            check_amount(*amount, *multiplier).map(|()| {
                ObjectKind::Pickup(Pickup::Resource {
                    resource,
                    amount: *amount,
                    multiplier: *multiplier,
                })
            })
        }
        ObjectKindDef::Pickup(PickupDef::OneOf(keys)) => {
            let mut choices = Vec::with_capacity(keys.len());
            for key in keys {
                scope.reach(key)?;
                let id = index.get(key.as_str()).copied().ok_or_else(|| {
                    AdventureCompileError::UnknownObjectType(String::from(key.as_str()))
                })?;
                choices.push(id);
            }
            let distinct = choices.iter().collect::<BTreeSet<_>>().len() == choices.len();
            match (choices.is_empty(), distinct) {
                (true, _) => Err("one_of is empty"),
                (false, false) => Err("one_of names a type twice"),
                (false, true) => Ok(ObjectKind::Pickup(Pickup::OneOf(choices))),
            }
        }
        ObjectKindDef::Pickup(PickupDef::Paired {
            amount,
            resources,
            paired,
            multiplier,
        }) => {
            let list = resources
                .iter()
                .map(|r| scope.resource(r))
                .collect::<Result<Vec<_>, _>>()?;
            let paired = scope.resource(paired)?;
            let distinct = list.iter().collect::<BTreeSet<_>>().len() == list.len();
            match (
                check_amount(*amount, *multiplier),
                list.is_empty(),
                distinct,
            ) {
                (Err(reason), _, _) => Err(reason),
                (Ok(()), true, _) => Err("resources is empty"),
                (Ok(()), false, false) => Err("resources names a resource twice"),
                (Ok(()), false, true) => Ok(ObjectKind::Pickup(Pickup::Paired {
                    amount: *amount,
                    resources: list,
                    paired,
                    multiplier: *multiplier,
                })),
            }
        }
        ObjectKindDef::RevealTerrain { terrains, price } => {
            let mut set = BTreeSet::new();
            for key in terrains {
                set.insert(scope.terrain(key)?);
            }
            let mut costs = BTreeMap::new();
            for (key, amount) in price {
                costs.insert(scope.resource(key)?, *amount);
            }
            if terrains.is_empty() {
                Err("terrains is empty")
            } else if set.len() != terrains.len() {
                Err("terrains names a terrain twice")
            } else if costs.len() != price.len() {
                Err("price names a resource twice")
            } else if costs.values().any(|&a| !(1..=MAX_GAIN).contains(&a)) {
                Err("a price is 0 or above MAX_GAIN")
            } else {
                Ok(ObjectKind::RevealTerrain {
                    terrains: set.into_iter().collect(),
                    price: costs.into_iter().collect(),
                })
            }
        }
        ObjectKindDef::RevealArea { radius } => {
            if *radius == 0 {
                Err("radius is 0")
            } else {
                Ok(ObjectKind::RevealArea { radius: *radius })
            }
        }
        ObjectKindDef::KeyTent { key } => check_key(*key).map(|key| ObjectKind::KeyTent { key }),
        ObjectKindDef::KeyGuard { key } => check_key(*key).map(|key| ObjectKind::KeyGuard { key }),
        ObjectKindDef::KeyGate { key } => check_key(*key).map(|key| ObjectKind::KeyGate { key }),
        ObjectKindDef::Monster { levels } => match levels {
            Some([low, high]) if !(1 <= *low && low <= high) => {
                Err("levels is not 1 <= low <= high")
            }
            _ => Ok(ObjectKind::Monster { levels: *levels }),
        },
        ObjectKindDef::Ground {
            battlefield,
            no_native,
        } => {
            scope.reach(battlefield)?;
            Ok(ObjectKind::Ground {
                battlefield: battlefield.clone(),
                no_native: *no_native,
            })
        }
        ObjectKindDef::Chest(def) => compile_chest(scope, def)?.map(ObjectKind::Chest),
        ObjectKindDef::Artifact { classes } => compile_artifact_classes(classes.as_ref())
            .map(|classes| ObjectKind::Artifact { classes }),
        ObjectKindDef::Scroll => Ok(ObjectKind::Scroll),
        ObjectKindDef::Gift(def) => compile_gift(scope, def)?.map(ObjectKind::Gift),
        ObjectKindDef::Shrine { level } => {
            if *level == 0 {
                Err("a Shrine's level is 0")
            } else {
                Ok(ObjectKind::Shrine { level: *level })
            }
        }
        ObjectKindDef::WitchHut { roe, roe_fixed } => {
            match (scope.skills(roe)?, scope.skills(roe_fixed)?) {
                (Ok(roe), Ok(roe_fixed)) => Ok(ObjectKind::WitchHut { roe, roe_fixed }),
                (Err(reason), _) | (_, Err(reason)) => Err(reason),
            }
        }
        ObjectKindDef::Scholar => Ok(ObjectKind::Scholar),
        ObjectKindDef::University { offers, price } => {
            match (compile_resources(scope, price)?, *offers) {
                (Err(reason), _) => Err(reason),
                (Ok(_), 0) => Err("a University offers nothing"),
                (Ok(price), offers) => Ok(ObjectKind::University { offers, price }),
            }
        }
        ObjectKindDef::Find(def) => compile_find(scope, def)?.map(ObjectKind::Find),
        ObjectKindDef::PandoraBox => Ok(ObjectKind::PandoraBox),
        ObjectKindDef::Event => Ok(ObjectKind::Event),
        ObjectKindDef::Pyramid(def) => compile_pyramid(scope, def)?.map(ObjectKind::Pyramid),
        ObjectKindDef::Message { leaves } => Ok(ObjectKind::Message { leaves: *leaves }),
        ObjectKindDef::SeaGround => Ok(ObjectKind::SeaGround),
        ObjectKindDef::Hole => Ok(ObjectKind::Hole),
    })
}

/// An amount's bounds: from 1 to [`MAX_PICKUP_AMOUNT`], in order; the largest amount times the
/// multiplier within [`MAX_GAIN`].
fn check_amount([low, high]: [u32; 2], multiplier: NonZeroU32) -> Result<(), &'static str> {
    if !(1 <= low && low <= high && high <= MAX_PICKUP_AMOUNT) {
        Err("amount is not 1 <= low <= high <= MAX_PICKUP_AMOUNT")
    } else if MAX_PICKUP_AMOUNT
        .checked_mul(multiplier.get())
        .is_none_or(|g| g > MAX_GAIN)
    {
        Err("MAX_PICKUP_AMOUNT times the multiplier is above MAX_GAIN")
    } else {
        Ok(())
    }
}

fn check_key(key: u8) -> Result<u8, &'static str> {
    if key < KEY_COLOURS {
        Ok(key)
    } else {
        Err("key is not below KEY_COLOURS")
    }
}

impl CanonicalHash for ObjectType {
    /// Key, name, visit (0 enter, 1 neighbour), from above, then the kind: its tag (0 pickup,
    /// 1 reveal terrain, 2 reveal area, 3 key tent, 4 key guard, 5 key gate; since part v7 6
    /// monster, 7 ground; since part v12 8 chest, 9 artifact, 10 scroll, 11 gift, 12 Shrine,
    /// 13 Witch Hut, 14 Scholar, 15 University, 16 find, 17 Pandora's Box, 18 event, 19 Pyramid;
    /// since part v13 20 message, 21 ground of the sea, 22 hole) and parameters (a pickup's own tag, 0 resource, 1 one of, 2 paired, then its fields in
    /// order; a monster's levels as 0 for none, else 1, low, high; a ground's battlefield key,
    /// since part v10 whether it takes the native price away; an artifact's classes as 0 for
    /// none, else 1 and the classes; a Witch Hut's two skill lists; a University's offers and
    /// price; a message's whether it leaves; the others' fields in order).
    fn hash_into(&self, h: &mut StateHasher) {
        self.key.as_str().hash_into(h);
        self.name.as_str().hash_into(h);
        h.write_u8(match self.visit {
            Visit::Enter => 0,
            Visit::Neighbour => 1,
        });
        h.write_u8(u8::from(self.from_above));
        match &self.kind {
            ObjectKind::Pickup(pickup) => {
                h.write_u8(0);
                pickup.hash_into(h);
            }
            ObjectKind::RevealTerrain { terrains, price } => {
                h.write_u8(1);
                terrains.hash_into(h);
                h.write_len(price.len());
                for (resource, amount) in price {
                    resource.hash_into(h);
                    h.write_u32(*amount);
                }
            }
            ObjectKind::RevealArea { radius } => {
                h.write_u8(2);
                h.write_u8(*radius);
            }
            ObjectKind::KeyTent { key } => {
                h.write_u8(3);
                h.write_u8(*key);
            }
            ObjectKind::KeyGuard { key } => {
                h.write_u8(4);
                h.write_u8(*key);
            }
            ObjectKind::KeyGate { key } => {
                h.write_u8(5);
                h.write_u8(*key);
            }
            ObjectKind::Monster { levels } => {
                h.write_u8(6);
                match levels {
                    None => h.write_u8(0),
                    Some([low, high]) => {
                        h.write_u8(1);
                        h.write_u8(*low);
                        h.write_u8(*high);
                    }
                }
            }
            ObjectKind::Ground {
                battlefield,
                no_native,
            } => {
                h.write_u8(7);
                battlefield.as_str().hash_into(h);
                h.write_bool(*no_native);
            }
            ObjectKind::Chest(chest) => {
                h.write_u8(8);
                chest.hash_into(h);
            }
            ObjectKind::Artifact { classes } => {
                h.write_u8(9);
                match classes {
                    None => h.write_u8(0),
                    Some(classes) => {
                        h.write_u8(1);
                        hash_classes(classes, h);
                    }
                }
            }
            ObjectKind::Scroll => h.write_u8(10),
            ObjectKind::Gift(gift) => {
                h.write_u8(11);
                gift.hash_into(h);
            }
            ObjectKind::Shrine { level } => {
                h.write_u8(12);
                h.write_u8(*level);
            }
            ObjectKind::WitchHut { roe, roe_fixed } => {
                h.write_u8(13);
                roe.hash_into(h);
                roe_fixed.hash_into(h);
            }
            ObjectKind::Scholar => h.write_u8(14),
            ObjectKind::University { offers, price } => {
                h.write_u8(15);
                h.write_u8(*offers);
                hash_resources(price, h);
            }
            ObjectKind::Find(find) => {
                h.write_u8(16);
                find.hash_into(h);
            }
            ObjectKind::PandoraBox => h.write_u8(17),
            ObjectKind::Event => h.write_u8(18),
            ObjectKind::Pyramid(pyramid) => {
                h.write_u8(19);
                pyramid.hash_into(h);
            }
            ObjectKind::Message { leaves } => {
                h.write_u8(20);
                h.write_bool(*leaves);
            }
            ObjectKind::SeaGround => h.write_u8(21),
            ObjectKind::Hole => h.write_u8(22),
        }
    }
}

impl CanonicalHash for Pickup {
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Resource {
                resource,
                amount,
                multiplier,
            } => {
                h.write_u8(0);
                resource.hash_into(h);
                h.write_u32(amount[0]);
                h.write_u32(amount[1]);
                h.write_u32(multiplier.get());
            }
            Self::OneOf(choices) => {
                h.write_u8(1);
                choices.hash_into(h);
            }
            Self::Paired {
                amount,
                resources,
                paired,
                multiplier,
            } => {
                h.write_u8(2);
                h.write_u32(amount[0]);
                h.write_u32(amount[1]);
                resources.hash_into(h);
                paired.hash_into(h);
                h.write_u32(multiplier.get());
            }
        }
    }
}
