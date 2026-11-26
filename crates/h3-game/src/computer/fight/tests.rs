//! Tests of the fight estimate (E31 T2c): its pieces on numbers an IEEE double model of the
//! reading gave (`E31-log.md` T2c), and whole estimates on the crate's test games, with an enemy
//! the simulated combat beats before its first round (war machines worth nothing) so that the
//! numbers are the estimate's own: the experience's value, the enemy's by its owner, the walls
//! and the loss.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureRuleset, Holds, ObjectKindDef, ObjectSetup, ObjectTypeDef, Scenery, TileMask, Visit,
};
use h3_content::{ContentPack, Ruleset};
use h3_content_battle::BattlefieldDef;
use h3_contracts::{PlayerId, Rate};
use h3_towns::TownRuleset;

use super::*;
use crate::GameSetup;
use crate::tests::{adventure_pack, battle_part_with, core_pack, key, setup};
use crate::town::tests::{towns_rules, towns_setup};

const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const CART: CreatureTypeId = CreatureTypeId::from_raw(2);

/// The crate's test core with values (the imp Fight Value 100 and AI value 10, the gog 300 and
/// 40, the cart 0 and 50: worth something, fighting with nothing) and the battle's hit points
/// (the imp 4, the gog 13, the cart 100); a ground `t:plains_ground` of the battlefield
/// `t:plains` (air at Expert).
fn valued_rules() -> GameRuleset {
    let mut pack: ContentPack = core_pack(false);
    for (creature, (fight, ai)) in pack
        .creatures
        .iter_mut()
        .zip([(100, 10), (300, 40), (0, 50)])
    {
        creature.info.fight_value = fight;
        creature.info.ai_value = ai;
    }
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 2);
    adventure.object_types.push(ObjectTypeDef {
        key: key("t:plains_ground"),
        name: "Ground".into(),
        visit: Visit::Enter,
        from_above: true,
        kind_mark: false,
        kind: ObjectKindDef::Ground {
            battlefield: key("t:plains"),
            no_native: false,
        },
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        let plain = pack.battlefields[0].clone();
        pack.battlefields.push(BattlefieldDef {
            key: key("t:plains"),
            expert_schools: vec![key("t:air")],
            ..plain
        });
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(TownRuleset::empty(&core)),
    )
    .unwrap()
}

/// A ground `t:plains_ground` on `tile`.
fn plains(rules: &GameRuleset, tile: Pos) -> ObjectSetup {
    ObjectSetup {
        anchor: tile,
        blocked: TileMask::default(),
        visitable: TileMask::default(),
        object_type: rules.adventure().object_type_by_key("t:plains_ground"),
        amount: None,
        drawn: TileMask::from_bits(1),
        monster: None,
        guards: None,
        scenery: Scenery::Other,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// The crate's test game (red's hero with 9 imps on (0, 0), blue's on (2, 2)) on `rules`, with
/// `edit` made to its setup.
fn game(rules: &GameRuleset, edit: impl FnOnce(&mut GameSetup)) -> GameState {
    let mut setup = setup();
    edit(&mut setup);
    GameState::new(rules, &setup, 3).unwrap()
}

/// `state`'s hero of `player` as a fighter.
fn fighter(state: &GameState, player: PlayerId) -> Fighter<'_> {
    let on_map = state
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == player)
        .unwrap();
    Fighter {
        on_map,
        details: state.hero(on_map.id).unwrap(),
    }
}

/// `count` carts in slot 0.
fn carts(count: u32) -> Vec<ArmyStack> {
    vec![ArmyStack {
        slot: 0,
        creature: CART,
        count,
    }]
}

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

// covers: CPU-SIM-012
#[test]
fn a_won_fight_is_worth_its_experience_less_its_losses() {
    let rules = valued_rules();
    let tile = Pos::new(1, 1, 0);
    // Red a computer player; the fight on the plains.
    let state = game(&rules, |s| {
        s.players[0].human = false;
        s.map.objects.push(plains(&rules, tile));
    });
    let enemy = carts(20);
    let opponent = Opponent {
        hero: None,
        army: &enemy,
        town: None,
    };
    // The carts fight with nothing: no losses. 20 × 100 experience points at
    // (f32(2 500) + f32(9 × 10)) / f32(40 × 1 000) = 0.064 75 each: ftol(129.5) = 129.
    assert_eq!(
        fight(&state, &rules, fighter(&state, RED), opponent, Some(tile)),
        129
    );
}

// covers: CPU-SIM-012
#[test]
fn an_enemy_heros_army_adds_its_value_weighted_by_its_owner() {
    let rules = valued_rules();
    let enemy = carts(20);
    let worth = |state: &GameState| {
        let opponent = Opponent {
            hero: Some(fighter(state, BLUE)),
            army: &enemy,
            town: None,
        };
        fight(state, &rules, fighter(state, RED), opponent, None)
    };
    // A human owner on Normal: 0.5 of the carts' 1 000 and the 129.
    assert_eq!(worth(&game(&rules, |_| {})), 629);
    // On Easy a computer owner weighs 1.0, a human one -0.4 (f32): 1 129 and ftol(-271.000 006).
    let state = game(&rules, |s| {
        s.difficulty = 0;
        s.players[0].human = false;
        s.players[1].human = false;
    });
    assert_eq!(worth(&state), 1129);
    assert_eq!(worth(&game(&rules, |s| s.difficulty = 0)), -271);
}

// covers: CPU-SIM-012
#[test]
fn a_fight_without_strength_is_lost_and_a_town_counts_its_walls_and_owner() {
    // The town tests' game: nothing fights with anything, so red loses at blue's town.
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    setup.players[0].human = false;
    let state = GameState::new(&rules, &setup, 3).unwrap();
    let garrison = [ArmyStack {
        slot: 0,
        creature: IMP,
        count: 3,
    }];
    let opponent = Opponent {
        hero: None,
        army: &garrison,
        town: Some(TownId::new(1)),
    };
    let tile = Some(Pos::new(3, 0, 0));
    assert_eq!(
        fight(&state, &rules, fighter(&state, RED), opponent, tile),
        LOST
    );
}

// covers: CPU-SIM-012
#[test]
fn the_enemys_factor_and_the_owners_weight_follow_the_difficulty() {
    let quarter = |n: u32| X87::ratio(n, 4, DOUBLE_BITS).unwrap();
    assert_eq!(enemy_factor(0, true), quarter(2));
    assert_eq!(enemy_factor(2, true), quarter(4));
    assert_eq!(enemy_factor(4, true), quarter(5));
    assert_eq!(enemy_factor(9, true), quarter(5));
    assert_eq!(enemy_factor(0, false), quarter(5));
    let quarters = |n: i64| SignedX87::int(n).over(SignedX87::int(4)).unwrap();
    assert_eq!(blend(3, None), SignedX87::ZERO);
    assert_eq!(blend(0, Some(true)), SignedX87::from_f32_bits(0xBECC_CCCD));
    assert_eq!(blend(1, Some(true)), quarters(2));
    assert_eq!(blend(4, Some(true)), quarters(5));
    assert_eq!(blend(0, Some(false)), SignedX87::int(1));
    assert_eq!(blend(2, Some(false)), quarters(1));
    assert_eq!(blend(4, Some(false)), quarters(-1));
}

// covers: CPU-SIM-012
#[test]
fn experience_is_worth_the_army_over_the_next_level_and_points_are_hit_points() {
    let rules = valued_rules();
    let f32_of = |n: i64| SignedX87::int(n).round(SINGLE_BITS);
    let value = f32_of(2590).over(f32_of(40_000));
    assert_eq!(experience_value(90, 1000), value);
    assert_eq!(experience_value(90, 0), None);
    let army = [
        ArmyStack {
            slot: 0,
            creature: IMP,
            count: 3,
        },
        ArmyStack {
            slot: 4,
            creature: GOG,
            count: 2,
        },
    ];
    assert_eq!(experience_points(&rules, &army), 3 * 4 + 2 * 13);
    assert_eq!(value_of(&rules, &army), 3 * 10 + 2 * 40);
}

// covers: CPU-SIM-012
#[test]
fn a_net_loss_under_a_quarter_of_the_army_counts_nothing() {
    // 100 gained, 30 lost of 1 000: 70.
    assert_eq!(settle(100, 1000, 970), 70);
    // 100 gained, 200 lost of 1 000 (under a quarter): 0; 300 of 1 000 (above): -200.
    assert_eq!(settle(100, 1000, 800), 0);
    assert_eq!(settle(100, 1000, 700), -200);
}

// covers: CPU-SIM-011
#[test]
fn necromancy_raises_its_share_of_the_enemys_hit_points_into_the_survivors() {
    let rules = valued_rules();
    let gogs = [ArmyStack {
        slot: 2,
        creature: GOG,
        count: 10,
    }];
    let hp = |c: CreatureTypeId| rules.battle().creature(c).map_or(0, |c| c.hit_points);
    // p = f32(f32(0.2) + f32(0.02)) = 0.219 999 998 8; min(13, 4) × 10 × p / 4 = 2.2: 2.
    let fifth = X87::ratio(2000, 10_000, SINGLE_BITS).unwrap();
    assert_eq!(necromancy_raises(fifth, 4, &gogs, hp), Some(2));
    // Nothing raised still raises 1; no share raises nothing.
    assert_eq!(necromancy_raises(fifth, 4, &[], hp), Some(1));
    assert_eq!(necromancy_raises(X87::ZERO, 4, &gogs, hp), None);
    // Into the survivors: the imps' stack, else the first empty slot, else nowhere.
    let raising = |share: Rate| {
        Some(Necromancy {
            share,
            creature: IMP,
        })
    };
    let tenth = Rate {
        bp: 2000,
        single: None,
    };
    let mut survivors = vec![ArmyStack {
        slot: 1,
        creature: IMP,
        count: 9,
    }];
    raise_into(&rules, &mut survivors, raising(tenth), &gogs);
    assert_eq!(survivors[0].count, 11);
    // The single-precision share of 70 %: 4 × 10 × 0.72 / 4 = 7.2: 7, into slot 0.
    let seventy = Rate {
        bp: 7000,
        single: core::num::NonZeroU32::new(0x3F33_3333),
    };
    let mut survivors = vec![ArmyStack {
        slot: 1,
        creature: GOG,
        count: 1,
    }];
    raise_into(&rules, &mut survivors, raising(seventy), &gogs);
    assert_eq!(
        survivors[1],
        ArmyStack {
            slot: 0,
            creature: IMP,
            count: 7,
        }
    );
    let mut full: Vec<ArmyStack> = (0..7)
        .map(|slot| ArmyStack {
            slot,
            creature: GOG,
            count: 1,
        })
        .collect();
    let before = full.clone();
    raise_into(&rules, &mut full, raising(tenth), &gogs);
    assert_eq!(full, before);
    raise_into(&rules, &mut full, None, &gogs);
    assert_eq!(full, before);
}
