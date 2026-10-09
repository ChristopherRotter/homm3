//! The mana creatures on the harvested packs (E04b T7c1; BAT-MAG-054, BAT-MAG-055): batch 2a R6
//! (`experiments/results/E04b-batch-2a.md`, SoD 4.0 Complete, plain HD, 2026-10-06) replayed.
//! Red leads Familiars and Mages, Blue a Pegasus and Gnolls; each hero casts at no mastery with
//! power 1. In the game: Red's Magic Arrow cost 5 (5 + 2 − 2), Blue's Ice Bolt 8 with 1 channelled
//! to Red, and Red's next Magic Arrows 5 again, also after the Pegasus died (the armies count,
//! BAT-QUIRK-15).

use h3_battle::{
    BattleState, CastTarget, Command, LegalCommands, NoopSink, StackId, apply, legal_commands,
};
use h3_content::{Formation, Ruleset, SpellId};
use h3_contracts::{Army, ArmyStack, BattleHero, BattleSetup, BattleSide};

fn caster(r: &Ruleset, hero: &str, mana: u16, spell: SpellId) -> BattleHero {
    let mut h = BattleHero::new(r.hero_type_by_key(hero).unwrap());
    h.power = 1;
    h.magic.mana = mana;
    h.magic.max_mana = mana;
    h.magic.spell_book = Some(vec![spell]);
    h
}

fn army(r: &Ruleset, stacks: &[(u8, &str, u32)]) -> Army {
    Army {
        stacks: stacks
            .iter()
            .map(|&(slot, key, count)| ArmyStack {
                slot,
                creature: r.creature_by_key(key).unwrap(),
                count,
            })
            .collect(),
    }
}

fn mana(s: &BattleState, side: BattleSide) -> u16 {
    s.hero(side).unwrap().mana
}

/// The side to act casts `spell` at `target`, then its active stack defends.
fn cast_then_defend(s: &mut BattleState, r: &Ruleset, spell: SpellId, target: u8) {
    let side = s.side_to_act().unwrap();
    let cast = Command::Cast {
        spell,
        target: CastTarget::Stack(StackId(target)),
        second: CastTarget::None,
    };
    apply(s, r, side, cast, &mut NoopSink).unwrap();
    apply(s, r, side, Command::Defend, &mut NoopSink).unwrap();
}

// covers: BAT-MAG-054, BAT-MAG-055
#[test]
fn batch_2a_r6_spell_costs_and_the_familiars_channel() {
    let r = h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        "base:faithful",
    )
    .unwrap();
    let arrow = r.spell_by_key("base:magic_arrow").unwrap();
    let ice = r.spell_by_key("base:ice_bolt").unwrap();
    let setup = BattleSetup {
        attacker: army(&r, &[(0, "base:familiar", 10), (1, "base:mage", 10)]),
        defender: army(&r, &[(0, "base:pegasus", 1), (1, "base:gnoll", 20)]),
        obstacles: vec![],
        heroes: [
            Some(caster(&r, "base:calid", 20, arrow)),
            Some(caster(&r, "base:orrin", 10, ice)),
        ],
        terrain: None,
        war_machines: [vec![], vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
    };
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    // Stacks: 0 Familiars (speed 7), 1 Mages (5), 2 Pegasus (8), 3 Gnolls (4).
    // Round 1, the Pegasus' turn: Blue's Ice Bolt at the Familiars, 8; Red's Familiars channel 1.
    assert_eq!(s.active(), Some(StackId(2)));
    cast_then_defend(&mut s, &r, ice, 0);
    assert_eq!(mana(&s, BattleSide::Defender), 2);
    assert_eq!(mana(&s, BattleSide::Attacker), 21);
    // The Familiars' turn: Red's Magic Arrow at the Gnolls, 5 + 2 − 2.
    assert_eq!(s.active(), Some(StackId(0)));
    cast_then_defend(&mut s, &r, arrow, 3);
    assert_eq!(mana(&s, BattleSide::Attacker), 16);
    // The Mages kill the Pegasus (10 × 7..9 × 1.15 ≥ 80 of its 75 hit points).
    assert_eq!(s.active(), Some(StackId(1)));
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    let shot = *legal
        .iter()
        .find(|c| matches!(c, Command::Shoot { target, .. } if *target == StackId(2)))
        .unwrap();
    apply(&mut s, &r, BattleSide::Attacker, shot, &mut NoopSink).unwrap();
    assert!(!s.stacks()[2].is_alive());
    apply(
        &mut s,
        &r,
        BattleSide::Defender,
        Command::Defend,
        &mut NoopSink,
    )
    .unwrap();
    // Round 2, the dead Pegasus still counts: 5 again.
    assert_eq!(s.round(), 2);
    assert_eq!(s.active(), Some(StackId(0)));
    cast_then_defend(&mut s, &r, arrow, 3);
    assert_eq!(mana(&s, BattleSide::Attacker), 11);
}
