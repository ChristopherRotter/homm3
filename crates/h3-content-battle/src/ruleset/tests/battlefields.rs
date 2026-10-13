//! Special battlefields (E04c T2b): their compilation, checks and hash, and obstacle types on
//! them.

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FactionDef, SpellSchoolDef};

use super::{base, compile, content, expect, key, obstacle};
use crate::battlefield::{BattlefieldDef, FactionChangeDef};
use h3_content::CompileError;

use crate::{BattleCompileError, Battlefield};

/// A battlefield without rules.
fn field(k: &str) -> BattlefieldDef {
    BattlefieldDef {
        key: key(k),
        own_obstacles: false,
        blocked: vec![],
        morale: vec![],
        luck: vec![],
        no_morale: false,
        no_luck: false,
        no_native: false,
        max_spell_level: None,
        no_creature_casts: false,
        expert_schools: vec![],
        expert_other_casts: false,
    }
}

fn change(factions: &[&str], amount: i8) -> FactionChangeDef {
    FactionChangeDef {
        factions: factions.iter().map(|f| key(f)).collect(),
        amount,
    }
}

/// `base()` with a second faction, two schools, a battlefield with every rule set, one with
/// fixed hexes, and an obstacle type of the first battlefield only.
fn fields() -> super::Pack {
    let mut p = base();
    p.core.factions.push(FactionDef {
        key: key("base:dungeon"),
        name: "D".into(),
        native_terrain: None,
    });
    p.core.spell_schools = ["base:air", "base:fire"]
        .iter()
        .map(|k| SpellSchoolDef {
            key: key(k),
            name: "S".into(),
            protection_rank: 0,
        })
        .collect();
    p.battle.battlefields = vec![
        BattlefieldDef {
            own_obstacles: true,
            morale: vec![change(&["base:castle"], 1), change(&["base:dungeon"], -1)],
            luck: vec![change(&["base:castle", "base:dungeon"], 2)],
            no_morale: true,
            no_luck: true,
            no_native: true,
            max_spell_level: Some(1),
            no_creature_casts: true,
            expert_schools: vec![key("base:fire")],
            expert_other_casts: true,
            ..field("base:cursed")
        },
        BattlefieldDef {
            blocked: vec![20, 21],
            ..field("base:boats")
        },
    ];
    let mut reef = obstacle("base:reef", &[], &[(0, 0)], false);
    reef.battlefields = vec![key("base:cursed")];
    p.battle.obstacles.push(reef);
    p
}

/// The compiler's refusal of `p` as the core error it raised.
fn refused(p: super::Pack) -> CompileError {
    content(expect(&[p], "base:faithful"))
}

fn invalid(reason: &'static str) -> CompileError {
    CompileError::InvalidContent {
        kind: "battlefield",
        key: "base:cursed".into(),
        reason,
    }
}

#[test]
fn battlefields_compile_with_their_rules() {
    let r = compile(&[fields()], "base:faithful").unwrap();
    let keys: Vec<&str> = r.battlefields().iter().map(|b| b.key.as_str()).collect();
    assert_eq!(keys, ["base:cursed", "base:boats"]);
    let id = r.battlefield_by_key("base:cursed").unwrap();
    assert_eq!(r.battlefield_by_key("base:nowhere"), None);
    let b = r.battlefield(id).unwrap();
    let castle = r.faction_by_key("base:castle");
    let dungeon = r.faction_by_key("base:dungeon");
    assert_eq!(Battlefield::change(&b.morale, castle), 1);
    assert_eq!(Battlefield::change(&b.morale, dungeon), -1);
    assert_eq!(Battlefield::change(&b.luck, dungeon), 2);
    assert_eq!(Battlefield::change(&b.morale, None), 0);
    let air = r
        .spell_schools()
        .iter()
        .position(|s| s.key.as_str() == "base:air");
    let fire = r
        .spell_schools()
        .iter()
        .position(|s| s.key.as_str() == "base:fire");
    let school = |i: Option<usize>| h3_content::SchoolId::from_index(i.unwrap()).unwrap();
    assert!(b.raises(&[school(air), school(fire)]));
    assert!(!b.raises(&[school(air)]));
    assert!(b.own_obstacles && b.no_morale && b.no_luck && b.no_native);
    assert!(b.no_creature_casts && b.expert_other_casts && b.max_spell_level == 1);
    let boats = r
        .battlefield(r.battlefield_by_key("base:boats").unwrap())
        .unwrap();
    assert_eq!(
        (boats.blocked.as_slice(), boats.max_spell_level),
        (&[20, 21][..], u8::MAX)
    );
    let reef = r.obstacle(r.obstacle_by_key("base:reef").unwrap()).unwrap();
    assert_eq!(
        (reef.terrains.len(), reef.battlefields.as_slice()),
        (0, &[id][..])
    );
    // Golden value: pins the battlefields' encoding (bump BATTLE_RULESET_HASH_VERSION when it
    // changes).
    assert_eq!(r.content_hash(), GOLDEN, "got {:#018x}", r.content_hash());
    // Each rule changes the hash.
    let edits: [fn(&mut BattlefieldDef); 6] = [
        |b| b.no_luck = false,
        |b| b.max_spell_level = Some(2),
        |b| b.expert_schools.clear(),
        |b| b.morale[0].amount = 2,
        |b| b.own_obstacles = false,
        |b| b.expert_other_casts = false,
    ];
    for edit in edits {
        let mut p = fields();
        edit(&mut p.battle.battlefields[0]);
        let other = compile(&[p], "base:faithful").unwrap();
        assert_ne!(other.content_hash(), r.content_hash());
    }
    let mut p = fields();
    p.battle.obstacles[2].battlefields.clear();
    p.battle.obstacles[2].terrains = vec![key("base:grass")];
    assert_ne!(
        compile(&[p], "base:faithful").unwrap().content_hash(),
        r.content_hash()
    );
}

const GOLDEN: u64 = 0xe419_6f07_7ccb_80b5;

#[test]
fn battlefields_are_checked() {
    type Edit = (fn(&mut BattlefieldDef), &'static str);
    let edits: [Edit; 8] = [
        (
            |b| b.morale[0].amount = 0,
            "a morale or luck change is -3..=3 and not 0",
        ),
        (
            |b| b.luck[0].amount = -4,
            "a morale or luck change is -3..=3 and not 0",
        ),
        (
            |b| b.morale[0].factions.clear(),
            "a change needs at least one faction",
        ),
        (
            |b| b.morale[1].factions = vec![key("base:castle")],
            "a faction is listed twice in one list",
        ),
        (
            |b| b.luck[0].factions[1] = key("base:castle"),
            "a faction is listed twice in one list",
        ),
        (
            |b| b.expert_schools.push(key("base:fire")),
            "a school is listed twice",
        ),
        (
            |b| b.max_spell_level = Some(6),
            "the spell level limit is 1..=5",
        ),
        (
            |b| b.blocked = vec![20],
            "fixed blocked hexes replace random obstacles",
        ),
    ];
    for (edit, reason) in edits {
        let mut p = fields();
        edit(&mut p.battle.battlefields[0]);
        assert_eq!(refused(p), invalid(reason), "{reason}");
    }
    let mut p = fields();
    p.battle.battlefields[0].max_spell_level = Some(0);
    assert_eq!(refused(p), invalid("the spell level limit is 1..=5"));
    let boats = |hexes: Vec<u8>, reason| {
        let mut p = fields();
        p.battle.battlefields[1].blocked = hexes;
        assert_eq!(
            refused(p),
            CompileError::InvalidContent {
                kind: "battlefield",
                key: "base:boats".into(),
                reason,
            }
        );
    };
    boats(vec![20, 20], "blocked hexes must be distinct");
    // Column 0 and 16 are side columns; row 11 is off the field.
    for hex in [17, 33, 187] {
        boats(
            vec![hex],
            "blocked hexes must be standable hexes of the field",
        );
    }
    // References: unknown faction and school, a key twice or outside the pack.
    let mut p = fields();
    p.battle.battlefields[0].morale[0].factions[0] = key("base:tower");
    assert!(matches!(
        refused(p),
        CompileError::UnknownReference {
            kind: "faction",
            ..
        }
    ));
    let mut p = fields();
    p.battle.battlefields[0].expert_schools[0] = key("base:water");
    assert!(matches!(
        refused(p),
        CompileError::UnknownReference {
            kind: "spell school",
            ..
        }
    ));
    let mut p = fields();
    p.battle.battlefields[1].key = key("base:cursed");
    assert_eq!(refused(p), CompileError::DuplicateKey("base:cursed".into()));
    let mut p = fields();
    p.battle.battlefields[1].key = key("ext:boats");
    assert!(matches!(refused(p), CompileError::KeyOutsidePack { .. }));
}

#[test]
fn obstacles_on_battlefields_are_checked() {
    // An unknown battlefield, and an obstacle on neither a terrain nor a battlefield.
    let mut p = fields();
    p.battle.obstacles[2].battlefields = vec![key("base:shore")];
    assert!(matches!(
        refused(p),
        CompileError::UnknownReference {
            kind: "battlefield",
            ..
        }
    ));
    let mut p = fields();
    p.battle.obstacles[2].battlefields.clear();
    assert_eq!(
        expect(&[p], "base:faithful"),
        BattleCompileError::InvalidObstacle {
            key: "base:reef".into(),
            reason: "an obstacle needs a terrain or a battlefield and one blocked hex",
        }
    );
}
