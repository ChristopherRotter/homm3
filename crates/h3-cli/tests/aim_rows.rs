//! The computer's aim on the base pack against the rows of batches 3f (the automatic keep,
//! `experiments/results/E04c-batch-3f.md`) and 3h (the automatic Ballista, `E04c-batch-3h.md`):
//! BAT-SIE-020 to BAT-SIE-023, BAT-WAR-008. Each encounter's armies, heroes' primary skills and
//! town, no obstacles, both players human (hot seat), every commanded stack defends; the stacks
//! the keep or Red's Ballista shoots in rounds 1 and 2 (the army's slot, or a war machine) are
//! the runs'. The heroes' types and resource specialties change no battle value, so one type
//! serves. (E04c T10b, review F-r5-4.)

use std::path::PathBuf;

use h3_battle::{BattleState, Command, Event, EventKind, Stack, StackTraits, apply};
use h3_content_battle::BattleRuleset;
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleHero, BattleSetup, BattleSide, FortLevel,
    MachineControl, Moat, Siege,
};

fn base() -> BattleRuleset {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    h3_cli::load_ruleset(&root.join("packs"), &["base".into()], "base:faithful").unwrap()
}

fn army(r: &BattleRuleset, stacks: &[(&str, u32)]) -> Army {
    Army {
        stacks: (0u8..)
            .zip(stacks)
            .map(|(slot, &(name, count))| ArmyStack {
                slot,
                creature: r.creature_by_key(&format!("base:{name}")).unwrap(),
                count,
            })
            .collect(),
    }
}

fn hero(r: &BattleRuleset, attack: u8) -> BattleHero {
    let mut hero = BattleHero::new(r.hero_type_by_key("base:damacon").unwrap());
    [hero.attack, hero.defense, hero.power, hero.knowledge] = [attack, 0, 1, 1];
    hero
}

/// What `shooter` shoots in rounds 1 and 2, every commanded stack defending: the target's army
/// slot, or `None` for a war machine.
fn shots(r: &BattleRuleset, setup: &BattleSetup, shooter: fn(&Stack) -> bool) -> Vec<Option<u8>> {
    let mut events = Vec::<Event>::new();
    let mut s = BattleState::start(r, setup, 1, &mut events).unwrap();
    let mut out = Vec::new();
    while out.len() < 2 {
        for e in events.drain(..) {
            if let EventKind::Struck {
                attacker, target, ..
            } = e.kind
                && shooter(&s.stacks()[attacker.index()])
            {
                let t = &s.stacks()[target.index()];
                out.push((!t.has(StackTraits::WAR_MACHINE)).then_some(t.slot));
            }
        }
        let side = s.side_to_act().unwrap();
        apply(&mut s, r, side, Command::Defend, &mut events).unwrap();
    }
    out.truncate(2);
    out
}

/// Batch 3h's encounter: Red's hero of `attack` with a Ballista and `red`'s army, Blue's hero of
/// attack 0 with `blue`'s army (and a Ballista under Expert Artillery when `artillery`).
fn field(
    r: &BattleRuleset,
    (attack, red): (u8, &[(&str, u32)]),
    blue: &[(&str, u32)],
    artillery: bool,
) -> BattleSetup {
    let ballista = r.creature_by_key("base:ballista").unwrap();
    let mut defender = hero(r, 0);
    let mut machines = [vec![ballista], vec![]];
    if artillery {
        defender.war_machines = MachineControl {
            ballista: Some(BallistaControl {
                level: 3,
                shots: 2,
                double_damage_chance_bp: 10_000,
            }),
            ..MachineControl::default()
        };
        machines[1].push(ballista);
    }
    BattleSetup {
        attacker: army(r, red),
        defender: army(r, blue),
        heroes: [Some(hero(r, attack)), Some(defender)],
        war_machines: machines,
        human: [true, true],
        ..BattleSetup::default()
    }
}

fn red_ballista(st: &Stack) -> bool {
    st.side == BattleSide::Attacker && st.has(StackTraits::WAR_MACHINE)
}

// covers: BAT-WAR-008, BAT-SIE-021, BAT-SIE-022, BAT-SIE-023
#[test]
fn the_automatic_ballista_shoots_as_batch_3h() {
    let r = base();
    let pikemen = |n| [("pikeman", n)];
    let titans = [("archer", 5), ("titan", 2)];
    // TI: two like stacks, the later; TU: the Archer (24) before the Behemoths three turns away
    // (13); DE: desperate, every creature 1000; DN: the Archers (24) before the Titans (16).
    let rows = [
        (
            (10, &pikemen(20)[..]),
            &[("archer", 10), ("archer", 10)][..],
            [Some(1), Some(1)],
        ),
        (
            (30, &pikemen(100)[..]),
            &[("archer", 1), ("behemoth", 3)][..],
            [Some(0), Some(1)],
        ),
        ((30, &pikemen(1)[..]), &titans[..], [Some(0), Some(1)]),
        ((30, &pikemen(100)[..]), &titans[..], [Some(0), Some(1)]),
    ];
    for (red, blue, expected) in rows {
        let setup = field(&r, red, blue, false);
        assert_eq!(shots(&r, &setup, red_ballista), expected, "{blue:?}");
    }
    // AR: Blue's Ballista under Expert Artillery before the thirty Zombies; AN: without it, the
    // Zombies.
    let zombies = [("zombie", 30)];
    let ar = field(&r, (10, &pikemen(50)), &zombies, true);
    assert_eq!(shots(&r, &ar, red_ballista), [None, None]);
    let mut an = ar;
    an.heroes[1].as_mut().unwrap().war_machines = MachineControl::default();
    assert_eq!(shots(&r, &an, red_ballista), [Some(0), Some(0)]);
}

// covers: BAT-SIE-020, BAT-SIE-021, BAT-SIE-022, BAT-SIE-023
#[test]
fn the_automatic_keep_shoots_as_batch_3f() {
    // A Castle with a Fort and a Citadel (a damaging moat) and one structure: a keep of 5. Red's
    // hero has attack 0 and the siege's Catapult (and in BA a Ballista); Blue's garrison no hero.
    let r = base();
    let siege = |red: &[(&str, u32)], blue: &[(&str, u32)], ballista: bool| BattleSetup {
        attacker: army(&r, red),
        defender: army(&r, blue),
        heroes: [Some(hero(&r, 0)), None],
        war_machines: [
            if ballista {
                vec![r.creature_by_key("base:ballista").unwrap()]
            } else {
                vec![]
            },
            vec![],
        ],
        siege: Some(Siege {
            faction: r.faction_by_key("base:castle").unwrap(),
            fort: FortLevel::Citadel,
            moat: Moat::Damaging,
            structures: 1,
            escape_tunnel: false,
            morale: 0,
            luck: 0,
        }),
        human: [true, true],
        ..BattleSetup::default()
    };
    let keep = |st: &Stack| st.is_turret();
    let gremlins = [("master_gremlin", 10), ("magog", 10)];
    // FV: the Magogs (242) before the Medusas (227); TU: the Royal Griffins two turns away (114)
    // before the Giant behind the walls (62); DE: desperate, the Master Gremlins (3750); DN: the
    // Magogs (242) before the Master Gremlins (206); BA: the Ballista before the Zombies.
    let rows = [
        (&[("magog", 5), ("medusa", 5)][..], 50, false, Some(0)),
        (
            &[("royal_griffin", 5), ("giant", 1)][..],
            50,
            false,
            Some(0),
        ),
        (&gremlins[..], 1, false, Some(0)),
        (&gremlins[..], 50, false, Some(1)),
        (&[("zombie", 100)][..], 30, true, None),
    ];
    for (red, pikemen, ballista, expected) in rows {
        let blue = if ballista { "swordsman" } else { "pikeman" };
        let setup = siege(red, &[(blue, pikemen)], ballista);
        assert_eq!(shots(&r, &setup, keep), [expected; 2], "{red:?}");
    }
}
