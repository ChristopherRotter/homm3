//! Batch 2b's stack spells on the harvested SoD packs (E04b T11b review, r2b F-6): R3 of
//! `experiments/results/E04b-batch-2b.md` (SoD 4.0 Complete, plain HD, 2026-10-07) replayed with
//! the heroes the batch used, derived in the Heroes context: the Fire Shield's reflection
//! (BAT-MAG-018, quirk Q-0010).

use h3_battle::{
    BattleState, CastTarget, Command, Event, EventKind, Field, LegalCommands, StackId, apply,
    legal_commands,
};
use h3_content::Formation;
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};
use h3_heroes::{HeroDetails, derive_battle_hero};

fn sod(profile: &str) -> BattleRuleset {
    h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
        profile,
    )
    .unwrap()
}

/// `hero` at level 1 with the primary skills `primary`, no skills, knowing `spells` (with a
/// worn Spell Book if any), with full mana.
fn details(r: &BattleRuleset, hero: &str, primary: [u8; 4], spells: &[&str]) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary,
        skills: vec![],
        worn: h3_heroes::wearing(
            r,
            &spells
                .iter()
                .take(1)
                .map(|_| r.artifact_by_key("base:spell_book").unwrap().into())
                .collect::<Vec<_>>(),
        ),
        backpack: vec![],
        spells: spells.iter().map(|k| r.spell_by_key(k).unwrap()).collect(),
        mana: u16::from(primary[3]) * 10,
    }
}

fn army(r: &BattleRuleset, creature: &str, count: u32) -> Army {
    Army {
        stacks: vec![ArmyStack {
            slot: 0,
            creature: r.creature_by_key(creature).unwrap(),
            count,
        }],
    }
}

/// The side whose turn it is does `command`, with stack `stack` active; returns the events.
fn turn(s: &mut BattleState, r: &BattleRuleset, stack: u8, command: Command) -> Vec<Event> {
    assert_eq!(s.active(), Some(StackId(stack)));
    let side = s.side_to_act().unwrap();
    let mut events = Vec::new();
    apply(s, r, side, command, &mut events).unwrap();
    events
}

// covers: BAT-MAG-018, BAT-DMG-002
/// Batch 2b R3: Aine (attack 0) leads ten Naga Queens against Sephinroth (defense 35, power 5,
/// Fire Shield at no mastery) with 1000 Peasants. Round 1: the Naga Queens walk towards the
/// Peasants; in the Peasants' turn Blue casts Fire Shield on them and they walk one hex (no
/// Defend). Round 2: the Naga Queens attack: 150 damage, 150 Peasants killed (attack 16 against
/// defense 36: half of the first stage's 300), and the log's "The Fire Shield does 60 damage.":
/// 20 % of the first stage (quirk Q-0010), not of the 150 dealt; ten Naga Queens left, the top
/// one at 50 of 110. Without the quirk (strict-fixed) the shield returns 20 % of the damage
/// dealt: 30.
#[test]
fn batch_2b_r3_the_fire_shield_reflects_a_fifth_of_the_first_stage() {
    for (profile, reflected) in [("base:faithful", 60), ("base:strict-fixed", 30)] {
        let r = sod(profile);
        let field = Field::new(r.battle().field);
        let aine = details(&r, "base:aine", [0, 0, 1, 1], &[]);
        let sephinroth = details(&r, "base:sephinroth", [0, 35, 5, 2], &["base:fire_shield"]);
        let setup = BattleSetup {
            attacker: army(&r, "base:naga_queen", 10),
            defender: army(&r, "ab:peasant", 1000),
            obstacles: vec![],
            heroes: [&aine, &sephinroth].map(|d| Some(derive_battle_hero(d, &r).unwrap())),
            terrain: None,
            battlefield: None,
            anti_magic: false,
            siege: None,
            war_machines: [vec![], vec![]],
            formations: [Formation::Loose; 2],
            funds: [0; 2],
            ..BattleSetup::default()
        };
        let mut s = BattleState::new(&r, &setup, 1).unwrap();
        // Stacks: 0 the Naga Queens (speed 7, two hexes), 1 the Peasants (speed 3).
        let (nagas, peasants) = (0, 1);
        let (_, row) = field.coords(s.stacks()[0].hex);
        // Round 1: the Naga Queens walk seven hexes along their row (from their position, the
        // rear hex in column 1).
        let to = field.hex(8, row).unwrap();
        turn(&mut s, &r, nagas, Command::Move { to });
        // The Peasants' turn: Blue's Fire Shield on them (16 of its 20 spell points), then one
        // hex towards the Naga Queens.
        let shield = Command::Cast {
            spell: r.spell_by_key("base:fire_shield").unwrap(),
            target: CastTarget::Stack(StackId(peasants)),
            second: CastTarget::None,
        };
        turn(&mut s, &r, peasants, shield);
        assert_eq!(s.hero(BattleSide::Defender).unwrap().mana, 4);
        let to = field.hex(14, row).unwrap();
        turn(&mut s, &r, peasants, Command::Move { to });
        // Round 2: the Naga Queens attack.
        assert_eq!((s.round(), s.active()), (2, Some(StackId(nagas))));
        let mut legal = LegalCommands::new();
        legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
        let attack = *legal
            .iter()
            .find(|c| matches!(c, Command::Attack { target, .. } if *target == StackId(peasants)))
            .unwrap();
        let events = turn(&mut s, &r, nagas, attack);
        let report: Vec<(u8, u64, u32)> = events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Struck {
                    target,
                    damage,
                    kills,
                    ..
                } => Some((target.0, damage, kills)),
                EventKind::Reflected {
                    stack,
                    damage,
                    kills,
                } => Some((stack.0, damage, kills)),
                _ => None,
            })
            .collect();
        // The strike on the Peasants, then the shield's damage to the Naga Queens; no
        // retaliation against them.
        assert_eq!(
            report,
            [(peasants, 150, 150), (nagas, reflected, 0)],
            "{profile}"
        );
        let queens = s.stacks()[usize::from(nagas)];
        assert_eq!(
            (queens.count, queens.top_hp, queens.max_hp),
            (10, 110 - u32::try_from(reflected).unwrap(), 110),
            "{profile}"
        );
        assert_eq!(s.stacks()[usize::from(peasants)].count, 850, "{profile}");
    }
}
