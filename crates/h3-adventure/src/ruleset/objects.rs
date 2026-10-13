//! Object types (ADR-0016 point 4, E08 T12a): what the import maps the original's classes to. A
//! type names its kind, a closed enum with typed parameters, and two visit properties; the packs
//! define the types (`adventure.toml`, `[[object_type]]`) and this module compiles them into the
//! ruleset part. The rules that read a kind come with the tasks that build them (pickups T13, the
//! Cartographer T14, the observatories, tents and guards T15a, the gates T15b); the setup draws
//! random contents (T12b).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{ContentKey, ResourceId, Ruleset, TerrainId};
use h3_kernel::{CanonicalHash, Handle, StateHasher};

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
}

impl ObjectKind {
    /// What a visit of this kind does, if its visit is built (ADR-0016 point 4; ADV-OBJ-003):
    /// the one place a task that builds a kind's visit extends. A step onto the visitable tile of
    /// an object whose visit is not built is rejected (`Visit`). E08 T13b: the pickups; T14a and
    /// T14b: the Cartographer, entered, with its offer; T15a: the observatories, the tents and the
    /// guards with their offer; T15b: the gates, which are not visited by their key's holders.
    pub(crate) fn effect(&self) -> Option<Effect<'_>> {
        match self {
            Self::Pickup(_) => Some(Effect::Pickup),
            Self::RevealTerrain { price, .. } => Some(Effect::Chart(price)),
            Self::RevealArea { radius } => Some(Effect::Reveal(*radius)),
            Self::KeyTent { key } => Some(Effect::Tent(*key)),
            Self::KeyGuard { key } => Some(Effect::Guard(*key)),
            Self::KeyGate { key } => Some(Effect::Gate(*key)),
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

/// Checks, once every pack is in, that each [`Pickup::OneOf`] names resource pickups only.
pub(super) fn check_object_types(types: &[ObjectType]) -> Result<(), AdventureCompileError> {
    for ty in types {
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
    /// 1 reveal terrain, 2 reveal area, 3 key tent, 4 key guard, 5 key gate) and parameters (a
    /// pickup's own tag, 0 resource, 1 one of, 2 paired, then its fields in order).
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
