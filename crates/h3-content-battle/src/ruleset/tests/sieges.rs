//! Sieges (E04c T5): the siege's field in the battle parameters, the siege records, their checks
//! and hash.

use alloc::vec;

use h3_content::{CompileError, FactionDef, SpellCategory, SpellDef, SpellSchoolDef};

use super::{Pack, Spell, base, compile, content, expect, key};
use crate::siege::{SiegeDef, SiegeFieldDef};
use crate::spell::{ObstacleKind, ObstacleShape, SpellEffect, SpellMagic, SpellPolarity};
use crate::{BattleCompileError, BattleSpellDef};

/// The original's field (`installation.md` §7.27, §7.28).
fn field() -> SiegeFieldDef {
    SiegeFieldDef {
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        gate: 96,
        moat: vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
        second_moat: vec![10, 27, 43, 60, 76, 94, 110, 128, 145, 163, 180],
        moat_defense_penalty: 3,
        mine_power: 3,
    }
}

fn record(faction: &str, damage: u32, second_moat: bool, mines: Option<&str>) -> SiegeDef {
    SiegeDef {
        faction: key(faction),
        moat_damage: damage,
        second_moat,
        mines: mines.map(key),
    }
}

/// A spell placing obstacles of `kind`.
fn obstacle_spell(k: &str, kind: ObstacleKind) -> Spell {
    Spell {
        core: SpellDef {
            key: key(k),
            name: "Mine".into(),
            category: SpellCategory::Combat,
            level: 3,
            schools: vec![key("base:fire")],
            cost: [5, 4, 4, 4],
            power: 10,
            effect: [25, 25, 50, 100],
            granted_by_name_only: false,
        },
        battle: BattleSpellDef {
            key: key(k),
            polarity: SpellPolarity::Neutral,
            magic: SpellMagic {
                obstacle: Some(ObstacleShape {
                    steps: vec![],
                    hexes: [4, 4, 6, 8],
                    random: true,
                    mirrored: false,
                    separate: true,
                    rounds: 0,
                }),
                effects: vec![SpellEffect::Obstacle(kind)],
                ..SpellMagic::default()
            },
            counters: vec![],
            dispels: vec![],
            spares: vec![],
            tests_as: None,
            summons: None,
        },
    }
}

/// `base()` with a second faction, a fire school, a mine and a pit spell, the field and two
/// records: the castle's moat, the dungeon's mines.
fn sieges() -> Pack {
    let mut p = base();
    p.core.factions.push(FactionDef {
        key: key("base:dungeon"),
        name: "D".into(),
        native_terrain: None,
    });
    p.core.spell_schools = vec![SpellSchoolDef {
        key: key("base:fire"),
        name: "Fire".into(),
        protection_rank: 0,
    }];
    p.add_spells([
        obstacle_spell("base:mine", ObstacleKind::Mine),
        obstacle_spell("base:pit", ObstacleKind::Pit),
    ]);
    if let Some(b) = p.battle.battle.as_mut() {
        b.siege = Some(field());
    }
    p.battle.sieges = vec![
        record("base:castle", 70, true, None),
        record("base:dungeon", 150, false, Some("base:mine")),
    ];
    p
}

/// The compiler's refusal of `p` as a battle parameter reason.
fn param_error(p: Pack) -> &'static str {
    match expect(&[p], "base:faithful") {
        BattleCompileError::InvalidBattleParams(reason) => reason,
        other => panic!("not a parameter error: {other:?}"),
    }
}

#[test]
fn the_field_and_the_records_compile() {
    let r = compile(&[sieges()], "base:faithful").unwrap();
    let f = r.siege_field().unwrap();
    assert_eq!((f.gate, f.bridge(), f.outer_bridge()), (96, 95, 94));
    assert_eq!((f.moat_defense_penalty, f.mine_power), (3, 3));
    assert_eq!(r.sieges().len(), 2);
    let castle = r.faction_by_key("base:castle").unwrap();
    let dungeon = r.faction_by_key("base:dungeon").unwrap();
    let c = r.siege(castle).unwrap();
    assert_eq!((c.moat_damage, c.second_moat, c.mines), (70, true, None));
    let mine = r.spell_by_key("base:mine");
    assert_eq!(r.siege(dungeon).unwrap().mines, mine);
    // No siege field, no records: a ruleset without sieges.
    let r = compile(&[base()], "base:faithful").unwrap();
    assert!(r.siege_field().is_none() && r.sieges().is_empty());
    assert_eq!(r.siege(castle), None);
}

#[test]
fn the_hash_covers_the_sieges() {
    let hash = |p: Pack| compile(&[p], "base:faithful").unwrap().content_hash();
    let with = hash(sieges());
    assert_ne!(with, hash(base()));
    let mut p = sieges();
    p.battle.sieges[0].moat_damage = 71;
    assert_ne!(hash(p), with);
    let mut p = sieges();
    p.battle.sieges.clear();
    assert_ne!(hash(p), with);
}

#[test]
fn field_errors() {
    let with = |edit: &dyn Fn(&mut SiegeFieldDef)| {
        let mut p = sieges();
        if let Some(f) = p.battle.battle.as_mut().and_then(|b| b.siege.as_mut()) {
            edit(f);
        }
        param_error(p)
    };
    let rows = "the siege field needs one wall, moat and second moat hex per row";
    assert_eq!(
        with(&|f| {
            f.walls.pop();
        }),
        rows
    );
    assert_eq!(with(&|f| f.moat.push(1)), rows);
    assert_eq!(with(&|f| f.second_moat.clear()), rows);
    let order = "each siege row needs standable hexes: the second moat, the moat, the wall, left \
                 to right";
    // A wall left of its moat, a moat on the wrong row, a hex in a side column.
    assert_eq!(with(&|f| f.walls[0] = 10), order);
    assert_eq!(with(&|f| f.moat[0] = 28), order);
    assert_eq!(with(&|f| f.second_moat[0] = 0), order);
    assert_eq!(with(&|f| f.walls[0] = 16), order);
    assert_eq!(
        with(&|f| f.gate = 95),
        "the gate must be one of the wall hexes"
    );
    // Records without the field.
    let mut p = sieges();
    if let Some(b) = p.battle.battle.as_mut() {
        b.siege = None;
    }
    assert_eq!(
        param_error(p),
        "siege records need the battle parameters' siege field"
    );
    // Mines of a spell that places pits.
    let mut p = sieges();
    p.battle.sieges[1].mines = Some(key("base:pit"));
    assert_eq!(
        param_error(p),
        "a siege's mines must be a spell that places mines"
    );
}

#[test]
fn record_errors() {
    let refused = |p: Pack| content(expect(&[p], "base:faithful"));
    let invalid = |faction: &str, reason| CompileError::InvalidContent {
        kind: "siege",
        key: faction.into(),
        reason,
    };
    let mut p = sieges();
    p.battle.sieges[0].moat_damage = 10_001;
    assert_eq!(
        refused(p),
        invalid("base:castle", "the moat's damage is at most 10 000")
    );
    let mut p = sieges();
    p.battle.sieges[1].second_moat = true;
    assert_eq!(
        refused(p),
        invalid(
            "base:dungeon",
            "land mines take the moat's place: no second moat row"
        )
    );
    let mut p = sieges();
    p.battle.sieges.push(record("base:castle", 90, false, None));
    assert_eq!(refused(p), CompileError::DuplicateKey("base:castle".into()));
    let mut p = sieges();
    p.battle.sieges.push(record("base:elves", 90, false, None));
    assert_eq!(
        refused(p),
        CompileError::UnknownReference {
            kind: "faction",
            key: "base:elves".into()
        }
    );
    let mut p = sieges();
    p.battle.sieges[1].mines = Some(key("base:nothing"));
    assert_eq!(
        refused(p),
        CompileError::UnknownReference {
            kind: "spell",
            key: "base:nothing".into()
        }
    );
}
