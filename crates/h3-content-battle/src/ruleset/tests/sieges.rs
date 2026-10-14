//! Sieges (E04c T5): the siege's field in the battle parameters, the siege records, their checks
//! and hash.

use alloc::vec;

use h3_content::{CompileError, FactionDef, SpellCategory, SpellDef, SpellSchoolDef};

use super::{Pack, Spell, base, compile, content, expect, key};
use crate::siege::{BallisticsRecord, SiegeDef, SiegeFieldDef, WallKind, WallTarget};
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
        catapult: 119,
        turret_count: 4,
        targets: [
            (WallKind::Tower, None, Some(255)),
            (WallKind::Wall, Some(29), None),
            (WallKind::Wall, Some(78), None),
            (WallKind::Gate, Some(96), None),
            (WallKind::Wall, Some(130), None),
            (WallKind::Wall, Some(182), None),
            (WallKind::Tower, None, Some(251)),
            (WallKind::Keep, None, Some(254)),
        ]
        .map(|(kind, wall, turret)| WallTarget { kind, wall, turret })
        .to_vec(),
        missed: vec![1, 2, 5, 4],
        automatic: vec![1, 2, 4, 5],
        fallback: vec![3, 7, 0, 6],
        castle_bonus: 1,
        ballistics: vec![
            record_of([5, 10, 25, 50], 1, [10, 60, 30]),
            record_of([7, 15, 30, 60], 1, [0, 50, 50]),
            record_of([7, 15, 30, 60], 2, [0, 50, 50]),
            record_of([10, 20, 40, 75], 2, [0, 0, 100]),
        ],
    }
}

/// A Ballistics record.
const fn record_of(hit: [u8; 4], shots: u8, damage: [u8; 3]) -> BallisticsRecord {
    BallisticsRecord { hit, shots, damage }
}

fn record(faction: &str, damage: u32, second_moat: bool, mines: Option<&str>) -> SiegeDef {
    SiegeDef {
        faction: key(faction),
        moat_damage: damage,
        second_moat,
        mines: mines.map(key),
        hit_points: vec![2; 8],
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
    // The wall shots (E04c T6).
    assert_eq!((f.catapult, f.targets.len(), f.castle_bonus), (119, 8, 1));
    assert_eq!(
        f.targets[2],
        WallTarget {
            kind: WallKind::Wall,
            wall: Some(78),
            turret: None,
        }
    );
    // The turrets (E04c T7).
    assert_eq!(
        (f.turret_count, f.targets[7].turret, f.targets[6].turret),
        (4, Some(254), Some(251))
    );
    assert_eq!(f.ballistics[3], record_of([10, 20, 40, 75], 2, [0, 0, 100]));
    assert_eq!(c.hit_points, [2; 8]);
    // No siege field, no records: a ruleset without sieges.
    let r = compile(&[base()], "base:faithful").unwrap();
    assert!(r.siege_field().is_none() && r.sieges().is_empty());
    assert_eq!(r.siege(castle), None);
}

#[test]
fn the_hash_covers_the_sieges() {
    let hash = |p: Pack| compile(&[p], "base:faithful").unwrap().content_hash();
    let with = hash(sieges());
    // Golden value: pins the sieges' encoding (battle part v5; bump BATTLE_RULESET_HASH_VERSION
    // when it changes).
    assert_eq!(with, GOLDEN, "got {with:#018x}");
    assert_eq!(crate::BATTLE_RULESET_HASH_VERSION, 5);
    assert_ne!(with, hash(base()));
    let mut p = sieges();
    p.battle.sieges[0].moat_damage = 71;
    assert_ne!(hash(p), with);
    let mut p = sieges();
    p.battle.sieges.clear();
    assert_ne!(hash(p), with);
    // The wall shots' data (battle part v4).
    let mut p = sieges();
    p.battle.sieges[0].hit_points[0] = 3;
    assert_ne!(hash(p), with);
    let edits: [&dyn Fn(&mut SiegeFieldDef); 9] = [
        &|f| f.catapult = 102,
        // The turrets (battle part v5).
        &|f| f.turret_count = 5,
        &|f| f.targets[0].turret = Some(250),
        &|f| f.targets[0].kind = WallKind::Keep,
        &|f| f.targets[7].kind = WallKind::Tower,
        &|f| f.missed.swap(0, 1),
        &|f| f.fallback.pop().map_or((), drop),
        &|f| f.castle_bonus = 2,
        &|f| f.ballistics[0].shots = 2,
    ];
    for edit in edits {
        let mut p = sieges();
        if let Some(f) = p.battle.battle.as_mut().and_then(|b| b.siege.as_mut()) {
            edit(f);
        }
        assert_ne!(hash(p), with);
    }
}

const GOLDEN: u64 = 0x5dd3_f4b1_3ebb_111a;

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
    // The wall shots (E04c T6).
    let targets = "the siege field needs 1 to 8 wall-shot targets";
    assert_eq!(with(&|f| f.targets.clear()), targets);
    assert_eq!(with(&|f| f.targets.push(f.targets[0])), targets);
    let gate = "one wall-shot target must be the gate, on the gate's hex";
    assert_eq!(with(&|f| f.targets[3].kind = WallKind::Wall), gate);
    assert_eq!(with(&|f| f.targets[1].kind = WallKind::Gate), gate);
    assert_eq!(with(&|f| f.targets[3].wall = Some(78)), gate);
    let placed = "a wall target needs a wall hex other than the gate's, a turret none";
    assert_eq!(with(&|f| f.targets[1].wall = None), placed);
    assert_eq!(with(&|f| f.targets[1].wall = Some(96)), placed);
    assert_eq!(with(&|f| f.targets[1].wall = Some(30)), placed);
    assert_eq!(with(&|f| f.targets[0].wall = Some(12)), placed);
    assert_eq!(with(&|f| f.targets[1].turret = Some(250)), placed);
    assert_eq!(with(&|f| f.targets[3].turret = Some(250)), placed);
    // The turrets (E04c T7): one per tower and the keep, off the field, distinct; a keep.
    let turrets = "the towers and the keep need turrets at distinct off-field positions, at most \
                   three, the keep at least one creature";
    assert_eq!(with(&|f| f.targets[7].turret = None), turrets);
    assert_eq!(with(&|f| f.targets[7].turret = Some(186)), turrets);
    assert_eq!(with(&|f| f.targets[6].turret = Some(255)), turrets);
    assert_eq!(with(&|f| f.turret_count = 0), turrets);
    assert_eq!(
        with(&|f| {
            f.targets[1] = WallTarget {
                kind: WallKind::Tower,
                wall: None,
                turret: Some(250),
            };
        }),
        turrets
    );
    let lists = "the wall-shot lists name targets, each once";
    assert_eq!(with(&|f| f.missed.push(8)), lists);
    assert_eq!(with(&|f| f.automatic.push(1)), lists);
    assert_eq!(with(&|f| f.fallback[0] = 9), lists);
    let place = "the Catapult stands in the side column, its front hex standable";
    assert_eq!(with(&|f| f.catapult = 120), place);
    assert_eq!(with(&|f| f.catapult = 16), place);
    assert_eq!(with(&|f| f.catapult = 187), place);
    assert_eq!(
        with(&|f| {
            f.ballistics.pop();
        }),
        "the Ballistics table needs one record per level, none to Expert"
    );
    let record = "a Ballistics record needs chances of at most 100, damage chances summing to 100 \
                  and 1 to 8 shots";
    assert_eq!(with(&|f| f.ballistics[0].hit[3] = 101), record);
    assert_eq!(with(&|f| f.ballistics[1].damage = [0, 50, 49]), record);
    assert_eq!(with(&|f| f.ballistics[2].shots = 0), record);
    assert_eq!(with(&|f| f.ballistics[3].shots = 9), record);
    // A record's hit points for another number of targets.
    let mut p = sieges();
    p.battle.sieges[0].hit_points.pop();
    assert_eq!(
        param_error(p),
        "a siege record needs hit points for each of the field's wall targets"
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
    for points in [vec![2, 0], vec![11], vec![2; 9]] {
        let mut p = sieges();
        p.battle.sieges[0].hit_points = points;
        assert_eq!(
            refused(p),
            invalid(
                "base:castle",
                "the wall targets' hit points are 1 to 10, at most eight of them"
            )
        );
    }
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
