//! Increment 10's kinds and parameters in the ruleset part (E08 T93a; increment 10's design note
//! §1, §2): the dwellings, their derived guards and shift, the random dwelling, the Refugee Camp,
//! `[params.dwellings]`, the random dwellings' tables, every check and the hash.

use h3_content::FactionDef;

use super::*;
use crate::setup::TileMask;

/// A creature of `faction` (none: neutral) of this level and growth, upgrading into `upgrades`.
fn line(k: &str, faction: Option<&str>, level: u8, growth: u16, upgrades: &[&str]) -> CreatureDef {
    let mut c = creature(k, level, false);
    c.info.faction = faction.map(key);
    c.info.growth = growth;
    c.info.upgrades = upgrades.iter().map(|u| key(u)).collect();
    c
}

fn faction(k: &str) -> FactionDef {
    FactionDef {
        key: key(k),
        name: "Faction".into(),
        native_terrain: None,
    }
}

/// [`core`]'s packs with factions: `t:castle`, its line `t:pike` (level 1, upgrading into
/// `t:halberd`, level 1 too), `t:archer` 2, `t:griffin` 3, `t:sword` 4, `t:monk` 5 (growth 3),
/// `t:knight` 6 (growth 2), `t:angel` 7 (growth 1); `t:keep` without creatures; `t:imp` (growth
/// 10) and `t:gog` (growth 0) neutral, `t:cart` a war machine; `u:fen` in `u`.
fn dwelling_core() -> Ruleset {
    let castle = Some("t:castle");
    let t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        resources: vec![resource("t:wood"), resource("t:gold")],
        factions: vec![faction("t:castle"), faction("t:keep")],
        creatures: vec![
            line("t:imp", None, 1, 10, &[]),
            line("t:gog", None, 2, 0, &[]),
            creature("t:cart", 1, true),
            line("t:pike", castle, 1, 14, &["t:halberd"]),
            line("t:halberd", castle, 1, 14, &[]),
            line("t:archer", castle, 2, 9, &[]),
            line("t:griffin", castle, 3, 7, &[]),
            line("t:sword", castle, 4, 4, &[]),
            line("t:monk", castle, 5, 3, &[]),
            line("t:knight", castle, 6, 2, &[]),
            line("t:angel", castle, 7, 1, &[]),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    let u = ContentPack {
        terrains: vec![terrain("u:swamp")],
        resources: vec![resource("u:gem")],
        factions: vec![faction("u:fen")],
        creatures: vec![creature("u:wisp", 1, false)],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let v = ContentPack::new(manifest("v", &[]));
    Ruleset::compile(&[t, u, v], "t:faithful").unwrap()
}

fn dwelling(creatures: &[&str], blocked: &[(u8, u8)], visitable: &[(u8, u8)]) -> ObjectKindDef {
    ObjectKindDef::Dwelling {
        creatures: creatures.iter().map(|c| key(c)).collect(),
        blocked: blocked.to_vec(),
        visitable: visitable.to_vec(),
    }
}

/// The castle's seven dwellings `t:d1` to `t:d7`, each its level's creature, the third the
/// halberd's upgraded line beside the griffin, the seventh the Angel Portal's footprint (its
/// anchor free, its entrance one left).
fn castle_dwellings() -> Vec<(String, ObjectKindDef)> {
    let footprint = [(0, 0), (1, 0), (1, 1)];
    let creatures: [&[&str]; 7] = [
        &["t:pike"],
        &["t:archer"],
        &["t:halberd", "t:griffin"],
        &["t:sword"],
        &["t:monk"],
        &["t:knight"],
        &["t:angel"],
    ];
    creatures
        .iter()
        .enumerate()
        .map(|(i, list)| {
            let kind = if i == 6 {
                dwelling(list, &[(1, 0), (2, 0), (1, 1)], &[(1, 0)])
            } else {
                dwelling(list, &footprint, &[(0, 0)])
            };
            (format!("t:d{}", i + 1), kind)
        })
        .collect()
}

fn castle_table() -> RandomDwellingDef {
    RandomDwellingDef {
        faction: key("t:castle"),
        dwellings: core::array::from_fn(|i| key(&format!("t:d{}", i + 1))),
    }
}

fn dwelling_params() -> DwellingParams {
    DwellingParams {
        guard_level: 5,
        guard_factor: NonZeroU16::new(3).unwrap(),
        free_level: 1,
    }
}

/// [`monster_packs`] with `t`'s castle dwellings, a two-creature `t:factory` (the knight and the
/// imp), a free-bottomed `t:spire`, the random dwelling `t:random_dwelling`, the camp `t:camp`,
/// the castle's table and the dwellings' parameters.
fn dwelling_packs() -> Vec<AdventurePack> {
    let mut p = monster_packs();
    let t = pack(&mut p, "t");
    for (k, kind) in castle_dwellings() {
        t.object_types.push(object(&k, Visit::Enter, false, kind));
    }
    t.object_types.extend([
        object(
            "t:factory",
            Visit::Enter,
            false,
            dwelling(&["t:knight", "t:imp", "t:angel"], &[(0, 0)], &[(0, 0)]),
        ),
        object(
            "t:spire",
            Visit::Enter,
            true,
            dwelling(&["t:imp"], &[(0, 1), (3, 2)], &[(0, 1)]),
        ),
        object(
            "t:random_dwelling",
            Visit::Enter,
            false,
            ObjectKindDef::RandomDwelling,
        ),
        object(
            "t:camp",
            Visit::Enter,
            false,
            ObjectKindDef::RefugeeCamp { levels: [1, 7] },
        ),
    ]);
    t.random_dwellings = vec![castle_table()];
    t.params.as_mut().unwrap().dwellings = Some(dwelling_params());
    p
}

fn compile_dwellings(packs: &[AdventurePack]) -> Result<AdventureRuleset, AdventureCompileError> {
    AdventureRuleset::compile(&dwelling_core(), packs)
}

/// [`dwelling_packs`] changed by `f`.
fn with_dwellings(f: impl FnOnce(&mut Vec<AdventurePack>)) -> Vec<AdventurePack> {
    let mut p = dwelling_packs();
    f(&mut p);
    p
}

/// [`dwelling_packs`] with the kind of `t`'s type `k` changed by `f`.
fn with_type(k: &str, f: impl FnOnce(&mut ObjectTypeDef)) -> Vec<AdventurePack> {
    with_dwellings(|p| {
        let ty = pack(p, "t")
            .object_types
            .iter_mut()
            .find(|t| t.key.as_str() == k)
            .unwrap();
        f(ty);
    })
}

fn dwelling_params_of(p: &mut [AdventurePack]) -> &mut DwellingParams {
    pack(p, "t")
        .params
        .as_mut()
        .unwrap()
        .dwellings
        .as_mut()
        .unwrap()
}

fn invalid_type(k: &str, reason: &'static str) -> AdventureCompileError {
    AdventureCompileError::InvalidObjectType {
        key: k.into(),
        reason,
    }
}

fn table_error(level: u8, reason: &'static str) -> AdventureCompileError {
    AdventureCompileError::RandomDwelling {
        faction: "t:castle".into(),
        level,
        reason,
    }
}

// covers: ADV-OBJ-002
#[test]
fn compiles_the_dwellings() {
    let core = dwelling_core();
    let part = compile_dwellings(&dwelling_packs()).unwrap();
    let id = |k: &str| part.object_type_by_key(k).unwrap();
    let kind = |k: &str| part.object_type(id(k)).unwrap().kind.clone();
    let c = |k: &str| core.creature_by_key(k).unwrap();
    let footprint = TileMask::from_bits(0b11 | 1 << 9);
    // The creatures in the pack's order; no guards below the guard level; the masks as given.
    assert_eq!(
        kind("t:d3"),
        ObjectKind::Dwelling {
            creatures: vec![c("t:halberd"), c("t:griffin")],
            blocked: footprint,
            visitable: TileMask::from_bits(1),
            guards: vec![],
            shift: 0,
        }
    );
    // Guards from the guard level, three times the growth; the anchor free: a shift of 1.
    assert_eq!(
        kind("t:d5"),
        ObjectKind::Dwelling {
            creatures: vec![c("t:monk")],
            blocked: footprint,
            visitable: TileMask::from_bits(1),
            guards: vec![(c("t:monk"), 9)],
            shift: 0,
        }
    );
    assert_eq!(
        kind("t:d7"),
        ObjectKind::Dwelling {
            creatures: vec![c("t:angel")],
            blocked: TileMask::from_bits(0b110 | 1 << 9),
            visitable: TileMask::from_bits(0b10),
            guards: vec![(c("t:angel"), 3)],
            shift: 1,
        }
    );
    // One stack for each creature of the guard level or more, in creature order.
    assert_eq!(
        kind("t:factory"),
        ObjectKind::Dwelling {
            creatures: vec![c("t:knight"), c("t:imp"), c("t:angel")],
            blocked: TileMask::from_bits(1),
            visitable: TileMask::from_bits(1),
            guards: vec![(c("t:knight"), 6), (c("t:angel"), 3)],
            shift: 0,
        }
    );
    // A bottom row free of every cell: the shift runs over the whole block.
    assert_eq!(
        kind("t:spire"),
        ObjectKind::Dwelling {
            creatures: vec![c("t:imp")],
            blocked: TileMask::from_bits(1 << 8 | 1 << 19),
            visitable: TileMask::from_bits(1 << 8),
            guards: vec![],
            shift: 8,
        }
    );
    assert_eq!(kind("t:random_dwelling"), ObjectKind::RandomDwelling);
    assert_eq!(kind("t:camp"), ObjectKind::RefugeeCamp { levels: [1, 7] });
    // Typed, no visit yet (T95); the dwellings and the random dwelling ownable, not the camp.
    for k in ["t:d1", "t:factory", "t:random_dwelling", "t:camp"] {
        let kind = kind(k);
        assert_eq!(kind.effect(), None, "{k}");
        assert_eq!(kind.ownable(), k != "t:camp", "{k}");
        assert!(!kind.weekly(), "{k}");
        assert_eq!(kind.battle_terrain(), None, "{k}");
    }
    // The parameters and the castle's table; none for a faction without one.
    assert_eq!(part.params().dwellings(), Some(dwelling_params()));
    let castle = core.faction_by_key("t:castle").unwrap();
    let row: [ObjectTypeId; DWELLING_LEVELS] =
        core::array::from_fn(|i| id(&format!("t:d{}", i + 1)));
    assert_eq!(part.random_dwelling(castle), Some(&row));
    assert_eq!(
        part.random_dwelling(core.faction_by_key("t:keep").unwrap()),
        None
    );
    assert_eq!(part.random_dwellings(), [(castle, row)]);
    // Without dwellings the parameters may be left out, and tables are not needed.
    let plain = AdventureRuleset::compile(&dwelling_core(), &monster_packs()).unwrap();
    assert_eq!(plain.params().dwellings(), None);
    assert!(plain.random_dwellings().is_empty());
    // A table may name a type of a dependency, from a later pack; the tables by faction.
    let p = with_dwellings(|p| {
        pack(p, "t").random_dwellings.clear();
        let u = pack(p, "u");
        u.object_types.push(object(
            "u:fen_house",
            Visit::Enter,
            false,
            dwelling(&["t:pike"], &[(0, 0)], &[(0, 0)]),
        ));
        let mut table = castle_table();
        table.dwellings[0] = key("u:fen_house");
        u.random_dwellings = vec![table];
    });
    let part = compile_dwellings(&p).unwrap();
    let fen_house = part.object_type_by_key("u:fen_house").unwrap();
    assert_eq!(part.random_dwelling(castle).unwrap()[0], fen_house);
}

// covers: ADV-OBJ-002
#[test]
fn the_dwellings_are_checked() {
    let kind = |k: &str, kind: ObjectKindDef| {
        compile_dwellings(&with_type(k, |t| t.kind = kind)).unwrap_err()
    };
    let d1 = |creatures: &[&str], blocked: &[(u8, u8)], visitable: &[(u8, u8)]| {
        kind("t:d1", dwelling(creatures, blocked, visitable))
    };
    let at = [(0, 0)];
    assert_eq!(
        d1(&["t:nobody"], &at, &at),
        AdventureCompileError::UnknownCreature("t:nobody".into())
    );
    assert_eq!(
        d1(&["u:wisp"], &at, &at),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:wisp".into(),
            pack: "t".into(),
        }
    );
    let count = invalid_type("t:d1", "a dwelling's creatures are not 1 to 4");
    assert_eq!(d1(&[], &at, &at), count);
    let five = ["t:pike", "t:archer", "t:griffin", "t:sword", "t:monk"];
    assert_eq!(d1(&five, &at, &at), count);
    assert!(
        compile_dwellings(&with_type("t:d1", |t| t.kind = dwelling(&five[..4], &at, &at))).is_ok()
    );
    assert_eq!(
        d1(&["t:pike", "t:pike"], &at, &at),
        invalid_type("t:d1", "a dwelling names a creature twice")
    );
    assert_eq!(
        d1(&["t:pike", "t:cart"], &at, &at),
        invalid_type("t:d1", "a dwelling's creature is a war machine")
    );
    assert_eq!(
        d1(&["t:gog"], &at, &at),
        invalid_type("t:d1", "a dwelling's creature has no growth")
    );
    let outside = invalid_type("t:d1", "a dwelling's cell is outside the 8 by 6 block");
    assert_eq!(d1(&["t:pike"], &[(8, 0)], &at), outside);
    assert_eq!(d1(&["t:pike"], &at, &[(0, 6)]), outside);
    let twice = invalid_type("t:d1", "a dwelling names a cell twice");
    assert_eq!(d1(&["t:pike"], &[(1, 1), (1, 1)], &at), twice);
    assert_eq!(d1(&["t:pike"], &at, &[(7, 5), (7, 5)]), twice);
    assert_eq!(
        d1(&["t:pike"], &at, &[]),
        invalid_type("t:d1", "a dwelling has no visitable cell")
    );
    // The corners of the block, no blocked cell and overlapping lists are fine.
    assert!(
        compile_dwellings(&with_type("t:d1", |t| {
            t.kind = dwelling(&["t:pike"], &[], &[(7, 5), (0, 0)]);
        }))
        .is_ok()
    );
    // Entered: the dwellings, the random dwelling (ownable), the camp.
    for k in ["t:d1", "t:random_dwelling"] {
        assert_eq!(
            compile_dwellings(&with_type(k, |t| t.visit = Visit::Neighbour)).unwrap_err(),
            invalid_type(k, "an ownable object is not entered")
        );
    }
    assert_eq!(
        compile_dwellings(&with_type("t:camp", |t| t.visit = Visit::Neighbour)).unwrap_err(),
        invalid_type("t:camp", "a refugee camp is not entered")
    );
    // The camp's levels, and a creature of them to draw.
    let levels = invalid_type(
        "t:camp",
        "a refugee camp's levels are not 1 <= low <= high <= DWELLING_LEVELS",
    );
    for bad in [[0, 1], [2, 1], [1, 8]] {
        assert_eq!(
            kind("t:camp", ObjectKindDef::RefugeeCamp { levels: bad }),
            levels
        );
    }
    let p = with_type("t:camp", |t| {
        t.kind = ObjectKindDef::RefugeeCamp { levels: [6, 6] };
    });
    assert!(compile_dwellings(&p).is_ok());
    let mut p = p;
    pack(&mut p, "u").not_random_monsters.push(key("t:knight"));
    assert_eq!(
        compile_dwellings(&p).unwrap_err(),
        invalid_type("t:camp", "no creature of its levels may be drawn")
    );
}

// covers: ADV-OBJ-002
#[test]
fn the_guards_and_the_dwellings_parameters_are_checked() {
    // A dwelling needs the parameters, which need the monsters'.
    let p = with_dwellings(|p| pack(p, "t").params.as_mut().unwrap().dwellings = None);
    assert_eq!(
        compile_dwellings(&p).unwrap_err(),
        invalid_type("t:d1", "a dwelling without [params.dwellings]")
    );
    let p = with_dwellings(|p| pack(p, "t").params.as_mut().unwrap().monsters = None);
    assert_eq!(
        compile_dwellings(&p).unwrap_err(),
        AdventureCompileError::InvalidParams("dwellings without the monsters' parameters")
    );
    let params = |f: fn(&mut DwellingParams)| {
        compile_dwellings(&with_dwellings(|p| f(dwelling_params_of(p))))
    };
    let guard_level =
        AdventureCompileError::InvalidParams("dwellings.guard_level is not 1 to DWELLING_LEVELS");
    assert_eq!(params(|d| d.guard_level = 0).unwrap_err(), guard_level);
    assert_eq!(params(|d| d.guard_level = 8).unwrap_err(), guard_level);
    assert_eq!(
        params(|d| d.free_level = 8).unwrap_err(),
        AdventureCompileError::InvalidParams("dwellings.free_level is above DWELLING_LEVELS")
    );
    // The bounds themselves: guards from level 7 (the angels' only), every creature joins free.
    let part = params(|d| {
        d.guard_level = 7;
        d.free_level = 7;
    })
    .unwrap();
    let guards = |k: &str| {
        let mut stacks = usize::MAX;
        let ty = part
            .object_type(part.object_type_by_key(k).unwrap())
            .unwrap();
        if let ObjectKind::Dwelling { guards, .. } = &ty.kind {
            stacks = guards.len();
        }
        stacks
    };
    assert_eq!(
        (guards("t:d5"), guards("t:d7"), guards("t:factory")),
        (0, 1, 1)
    );
    assert!(
        params(|d| {
            d.guard_level = 1;
            d.free_level = 0;
        })
        .is_ok()
    );
    // Each stack within max_stack_count (the monks' 3 × 1 365 = 4 095 fits, 4 098 not)...
    let factor = |f: u16| {
        compile_dwellings(&with_dwellings(|p| {
            let t = pack(p, "t").params.as_mut().unwrap();
            t.max_stack_count = NonZeroU32::new(MAX_MONSTER_COUNT).unwrap();
            t.dwellings.as_mut().unwrap().guard_factor = NonZeroU16::new(f).unwrap();
        }))
    };
    assert!(factor(1365).is_ok());
    assert_eq!(
        factor(1366).unwrap_err(),
        invalid_type("t:d5", "a dwelling's guards are above max_stack_count")
    );
    // ... and the stacks within an army's slots.
    let slots = |n: u8| {
        compile_dwellings(&with_dwellings(|p| {
            pack(p, "t").params.as_mut().unwrap().army_slots = NonZeroU8::new(n).unwrap();
        }))
    };
    assert!(slots(2).is_ok());
    assert_eq!(
        slots(1).unwrap_err(),
        invalid_type(
            "t:factory",
            "a dwelling's guards need more slots than an army has"
        )
    );
}

// covers: ADV-OBJ-002
#[test]
fn the_random_dwellings_tables_are_checked() {
    let table = |f: fn(&mut RandomDwellingDef)| {
        compile_dwellings(&with_dwellings(
            |p| f(&mut pack(p, "t").random_dwellings[0]),
        ))
        .unwrap_err()
    };
    assert_eq!(
        table(|t| t.faction = key("t:nobody")),
        AdventureCompileError::UnknownFaction("t:nobody".into())
    );
    assert_eq!(
        table(|t| t.faction = key("u:fen")),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:fen".into(),
            pack: "t".into(),
        }
    );
    assert_eq!(
        table(|t| t.dwellings[3] = key("t:nothing")),
        AdventureCompileError::UnknownObjectType("t:nothing".into())
    );
    assert_eq!(
        table(|t| t.dwellings[3] = key("u:something")),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:something".into(),
            pack: "t".into(),
        }
    );
    assert_eq!(
        table(|t| t.dwellings[0] = key("t:camp")),
        table_error(1, "its type is not a dwelling")
    );
    let unfit = "its dwelling has no creature of the faction and level that is no upgrade";
    // Another level's dwelling.
    assert_eq!(table(|t| t.dwellings.swap(1, 2)), table_error(2, unfit));
    // Neutral creatures of the level only.
    assert_eq!(
        table(|t| t.dwellings[0] = key("t:spire")),
        table_error(1, unfit)
    );
    // The level's creature an upgrade only (the halberd, the pike's).
    let p = with_dwellings(|p| {
        let t = pack(p, "t");
        t.object_types.push(object(
            "t:halberd_house",
            Visit::Enter,
            false,
            dwelling(&["t:halberd"], &[(0, 0)], &[(0, 0)]),
        ));
        t.random_dwellings[0].dwellings[0] = key("t:halberd_house");
    });
    assert_eq!(compile_dwellings(&p).unwrap_err(), table_error(1, unfit));
    // Another faction's creatures.
    let p = with_dwellings(|p| pack(p, "t").random_dwellings[0].faction = key("t:keep"));
    assert_eq!(
        compile_dwellings(&p).unwrap_err(),
        AdventureCompileError::RandomDwelling {
            faction: "t:keep".into(),
            level: 1,
            reason: unfit,
        }
    );
    // One table per faction in the whole part.
    let p = with_dwellings(|p| pack(p, "u").random_dwellings = vec![castle_table()]);
    assert_eq!(
        compile_dwellings(&p).unwrap_err(),
        AdventureCompileError::DuplicateRandomDwelling("t:castle".into())
    );
}

#[test]
fn the_dwellings_errors_explain_themselves() {
    let cases = [
        (
            AdventureCompileError::UnknownFaction("t:x".into()),
            "unknown faction `t:x`",
        ),
        (
            AdventureCompileError::DuplicateRandomDwelling("t:castle".into()),
            "faction `t:castle` has a second random dwelling table",
        ),
        (
            table_error(3, "its type is not a dwelling"),
            "random dwelling table of `t:castle`, level 3: its type is not a dwelling",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}

#[test]
fn the_hash_sees_the_dwellings() {
    let base = compile_dwellings(&dwelling_packs()).unwrap().content_hash();
    let mut changes: Vec<Vec<AdventurePack>> = Vec::new();
    let mut kind = |k: &str, kind: ObjectKindDef| changes.push(with_type(k, |t| t.kind = kind));
    let footprint = [(0, 0), (1, 0), (1, 1)];
    // The creatures, their order, the masks, the shift.
    kind(
        "t:d3",
        dwelling(&["t:griffin", "t:halberd"], &footprint, &[(0, 0)]),
    );
    kind("t:d3", dwelling(&["t:griffin"], &footprint, &[(0, 0)]));
    kind("t:d1", dwelling(&["t:pike"], &[(0, 0), (1, 0)], &[(0, 0)]));
    kind("t:d1", dwelling(&["t:pike"], &footprint, &[(0, 0), (1, 1)]));
    kind(
        "t:d1",
        dwelling(&["t:pike"], &[(1, 0), (2, 0), (1, 1)], &[(1, 0)]),
    );
    kind("t:camp", ObjectKindDef::RefugeeCamp { levels: [2, 7] });
    kind("t:camp", ObjectKindDef::RefugeeCamp { levels: [1, 6] });
    kind("t:camp", ObjectKindDef::RandomDwelling);
    let mut params = |f: fn(&mut DwellingParams)| {
        changes.push(with_dwellings(|p| f(dwelling_params_of(p))));
    };
    // The guards through the parameters, and the parameters themselves.
    params(|d| d.guard_level = 6);
    params(|d| d.guard_factor = NonZeroU16::new(4).unwrap());
    params(|d| d.free_level = 2);
    // The tables.
    changes.push(with_dwellings(|p| pack(p, "t").random_dwellings.clear()));
    changes.push(with_dwellings(|p| {
        pack(p, "t").object_types.push(object(
            "t:other_d1",
            Visit::Enter,
            false,
            dwelling(&["t:pike"], &[(0, 0)], &[(0, 0)]),
        ));
        pack(p, "t").random_dwellings[0].dwellings[0] = key("t:other_d1");
    }));
    let other = object(
        "t:other_d1",
        Visit::Enter,
        false,
        dwelling(&["t:pike"], &[(0, 0)], &[(0, 0)]),
    );
    let mut with_other = dwelling_packs();
    pack(&mut with_other, "t").object_types.push(other);
    let with_other = compile_dwellings(&with_other).unwrap().content_hash();
    let last = changes.len() - 1;
    for (i, packs) in changes.iter().enumerate() {
        let expected = if i == last { with_other } else { base };
        assert_ne!(
            compile_dwellings(packs).unwrap().content_hash(),
            expected,
            "change {i}"
        );
    }
    // The parameters alone, without a dwelling.
    let plain = compile_dwellings(&monster_packs()).unwrap().content_hash();
    let mut p = monster_packs();
    pack(&mut p, "t").params.as_mut().unwrap().dwellings = Some(dwelling_params());
    assert_ne!(compile_dwellings(&p).unwrap().content_hash(), plain);
}
