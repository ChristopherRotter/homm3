//! The mana creatures on the harvested packs (E04b T7c1; BAT-MAG-054, BAT-MAG-055): batch 2a R6
//! (`experiments/results/E04b-batch-2a.md`, SoD 4.0 Complete, plain HD, 2026-10-06) replayed.
//! Red leads Familiars and Mages, Blue a Pegasus and Gnolls; each hero casts at no mastery with
//! power 1. In the game: Red's Magic Arrow cost 5 (5 + 2 − 2), Blue's Ice Bolt 8 with 1 channelled
//! to Red, and Red's next Magic Arrows 5 again, also after the Pegasus died (the armies count,
//! BAT-QUIRK-15). The second test replays the batch's own order of casts with Red's Ballista,
//! in whose turn the hero casts (BAT-MAG-001).

use h3_battle::{
    BattleState, CastTarget, Command, LegalCommands, NoopSink, StackId, StackTraits, apply,
    castable_spells, legal_commands,
};
use h3_content::{Formation, SpellId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleHero, BattleSetup, BattleSide};
use h3_heroes::{HeroDetails, derive_battle_hero, war_machines};

fn caster(r: &BattleRuleset, hero: &str, mana: u16, spell: SpellId) -> BattleHero {
    let mut h = BattleHero::new(r.hero_type_by_key(hero).unwrap());
    h.power = 1;
    h.magic.mana = mana;
    h.magic.max_mana = mana;
    h.magic.spell_book = Some(vec![spell]);
    h
}

fn army(r: &BattleRuleset, stacks: &[(u8, &str, u32)]) -> Army {
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
fn cast_then_defend(s: &mut BattleState, r: &BattleRuleset, spell: SpellId, target: u8) {
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
    let r = h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
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
        battlefield: None,
        anti_magic: false,
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

// covers: BAT-MAG-001, BAT-WAR-006, BAT-MAG-054, BAT-MAG-055
/// Batch 2a R6 as it was played (E04b T11b review, r1 F-10), with the heroes derived in the
/// Heroes context: Calid (power 1, knowledge 2, Basic Artillery, a Ballista, Magic Arrow) leads
/// ten Familiars and ten Mages against Grindan (power 1, knowledge 1, Ice Bolt) with a Pegasus
/// and twenty Gnolls. The batch's spell points: Red 20 to 15 in the Familiars' turn of round 1;
/// Blue 10 to 2 in the Gnolls' turn, Red 16 afterwards; Red 16 to 11 in his Ballista's turn of
/// round 2, the Pegasus still alive; Red 11 to 6 in the Familiars' turn of round 3, the Pegasus
/// dead (the Mages shot it in round 2 after the Ballista's turn: they wait here).
#[test]
fn batch_2a_r6_the_hero_casts_in_his_ballistas_turn() {
    let r = h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
        "base:faithful",
    )
    .unwrap();
    let arrow = r.spell_by_key("base:magic_arrow").unwrap();
    let ice = r.spell_by_key("base:ice_bolt").unwrap();
    let artifact = |key: &str| r.artifact_by_key(key).unwrap().into();
    let details = |hero: &str, knowledge: u8, skills, worn: &[&str], spell| HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level: 1,
        experience: 0,
        primary: [0, 0, 1, knowledge],
        skills,
        worn: worn.iter().map(|k| artifact(k)).collect(),
        backpack: vec![],
        spells: vec![spell],
        mana: u16::from(knowledge) * 10,
    };
    let artillery = vec![(r.skill_by_key("base:artillery").unwrap(), 1)];
    let calid = details(
        "base:calid",
        2,
        artillery,
        &["base:spell_book", "base:ballista"],
        arrow,
    );
    let grindan = details("ab:grindan", 1, vec![], &["base:spell_book"], ice);
    let setup = BattleSetup {
        attacker: army(&r, &[(0, "base:familiar", 10), (1, "base:mage", 10)]),
        defender: army(&r, &[(0, "base:pegasus", 1), (1, "base:gnoll", 20)]),
        obstacles: vec![],
        heroes: [&calid, &grindan].map(|d| Some(derive_battle_hero(d, &r).unwrap())),
        terrain: None,
        battlefield: None,
        anti_magic: false,
        war_machines: [war_machines(&calid, &r), vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
    };
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    // Stacks: 0 Familiars (speed 7), 1 Mages (5), 2 the Ballista (last); 3 Pegasus (8), 4 Gnolls
    // (4).
    let (familiars, mages, ballista, pegasus, gnolls) = (0, 1, 2, 3, 4);
    assert!(s.stacks()[usize::from(ballista)].has(StackTraits::WAR_MACHINE));
    assert_eq!(
        (
            mana(&s, BattleSide::Attacker),
            mana(&s, BattleSide::Defender)
        ),
        (20, 10)
    );
    let turn = |s: &mut BattleState, stack: u8, command: Command| {
        assert_eq!(s.active(), Some(StackId(stack)));
        let side = s.side_to_act().unwrap();
        apply(s, &r, side, command, &mut NoopSink).unwrap();
    };
    // Round 1. The Familiars' turn: Red's Magic Arrow at the Gnolls, 20 to 15.
    turn(&mut s, pegasus, Command::Defend);
    assert_eq!(s.active(), Some(StackId(familiars)));
    cast_then_defend(&mut s, &r, arrow, gnolls);
    assert_eq!(mana(&s, BattleSide::Attacker), 15);
    turn(&mut s, mages, Command::Defend);
    // The Gnolls' turn: Blue's Ice Bolt at the Familiars, 10 to 2; Red's Familiars channel 1.
    assert_eq!(s.active(), Some(StackId(gnolls)));
    cast_then_defend(&mut s, &r, ice, familiars);
    assert_eq!(
        (
            mana(&s, BattleSide::Defender),
            mana(&s, BattleSide::Attacker)
        ),
        (2, 16)
    );
    // Red commands his Ballista (Basic Artillery); it defends.
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    turn(&mut s, ballista, Command::Defend);
    // Round 2: nobody casts before the Ballista's turn; the Mages wait.
    assert_eq!(s.round(), 2);
    turn(&mut s, pegasus, Command::Defend);
    turn(&mut s, familiars, Command::Defend);
    turn(&mut s, mages, Command::Wait);
    turn(&mut s, gnolls, Command::Defend);
    // The Ballista's turn: the hero may cast, and does: 16 to 11, the Pegasus alive.
    assert_eq!(s.active(), Some(StackId(ballista)));
    assert!(castable_spells(&s, &r, BattleSide::Attacker).contains(arrow));
    assert!(s.stacks()[usize::from(pegasus)].is_alive());
    cast_then_defend(&mut s, &r, arrow, gnolls);
    assert_eq!(mana(&s, BattleSide::Attacker), 11);
    // The Mages shoot the Pegasus dead.
    assert_eq!(s.active(), Some(StackId(mages)));
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    let shot = *legal
        .iter()
        .find(|c| matches!(c, Command::Shoot { target, .. } if *target == StackId(pegasus)))
        .unwrap();
    turn(&mut s, mages, shot);
    assert!(!s.stacks()[usize::from(pegasus)].is_alive());
    // Round 3, the Familiars' turn: 11 to 6, the dead Pegasus still counting.
    assert_eq!((s.round(), s.active()), (3, Some(StackId(familiars))));
    cast_then_defend(&mut s, &r, arrow, gnolls);
    assert_eq!(mana(&s, BattleSide::Attacker), 6);
}
