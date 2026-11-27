//! Tests of the army plan (E31 T2g): the alignments and the army's base morale with and without
//! the Angelic Alliance (and `AI-QUIRK-2`'s neutral), a stack's worth and its slot (the speed's
//! loss, morale-bad, the replacement's rules), the put and the shed, the exchange and the purchase
//! with the weighted cost. On the crate's test core with nine factions (the base and AB packs'
//! towns: the Necropolis 4), the creatures to the Minotaur King's id (79: the Angel 12 and the
//! Minotaur 78 at their original ids) and the Alliance at 129 joining 0, 1, 2, 6 and 7. The
//! numbers with doubles come from an IEEE double model of the reading (`E31-log.md` T2g); every
//! other number is worked by hand in the comments.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::AdventureRuleset;
use h3_content::{
    ArtifactClass, ArtifactDef, ContentPack, CreatureDef, FactionDef, HeroEffectDef, Ruleset,
};
use h3_content_battle::BattleCreatureDef;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use super::*;
use crate::tests::{adventure_pack, battle_part_with, core_pack, key};

/// A creature the tests name: its id, faction, AI value, speed and flags.
type Named = (u16, Option<u16>, u32, u8, &'static [&'static str]);

/// The creatures the tests name.
const CREATURES: [Named; 13] = [
    (0, Some(0), 10, 5, &[]),             // the imp
    (1, Some(0), 40, 2, &[]),             // the gog
    (3, None, 20, 3, &[]),                // a neutral
    (4, Some(1), 30, 3, &[]),             // of the Alliance's second town
    (5, Some(4), 60, 3, &["undead"]),     // of the Necropolis
    (6, Some(3), 25, 1, &["shooter"]),    // slow, outside the Alliance
    (7, Some(2), 15, 3, &["shooter"]),    // of the Alliance's third town
    (8, Some(5), 100, 3, &["no_morale"]), // feels no morale
    (9, Some(6), 100, 3, &[]),            // feels morale
    (10, Some(2), 1, 3, &["shooter"]),    // worth little
    (11, Some(2), 1, 3, &["shooter"]),    // worth little
    (12, Some(0), 500, 3, &[]),           // the Angel
    (78, Some(5), 80, 3, &[]),            // the Minotaur
];

/// The rules of the module's doc, with the profile `profile`.
fn rules_on(profile: &str) -> GameRuleset {
    let mut pack: ContentPack = core_pack(false);
    pack.factions.extend((1..9).map(|i| FactionDef {
        key: key(&alloc::format!("t:f{i}")),
        name: "Town".into(),
        native_terrain: None,
    }));
    for i in pack.creatures.len()..80 {
        let mut creature: CreatureDef = pack.creatures[1].clone();
        // The battle part's fourth row is the elf's.
        creature.key = key(&if i == 3 {
            "t:elf".into()
        } else {
            alloc::format!("t:c{i}")
        });
        pack.creatures.push(creature);
    }
    let town = |f: u16| {
        if f == 0 {
            key("t:town")
        } else {
            key(&alloc::format!("t:f{f}"))
        }
    };
    for (id, f, ai, speed, flags) in CREATURES {
        let creature = &mut pack.creatures[usize::from(id)];
        creature.info.faction = f.map(town);
        creature.info.ai_value = ai;
        creature.speed = speed;
        creature.info.flags = flags.iter().map(|&f| f.into()).collect();
    }
    for i in pack.artifacts.len()..130 {
        pack.artifacts.push(ArtifactDef {
            key: key(&alloc::format!("t:a{i}")),
            name: "Artifact".into(),
            class: ArtifactClass::Treasure,
            cost: vec![],
            slot: Some(key("t:ring_slot")),
            war_machine: None,
            needs_skill: None,
            effects: vec![],
            parts: vec![],
        });
    }
    pack.artifacts[129].effects = vec![
        HeroEffectDef::Morale {
            amount: 1,
            affects: h3_content::Affects::Own,
        },
        HeroEffectDef::AlignmentGroup {
            factions: [0, 1, 2, 6, 7].map(town).to_vec(),
        },
    ];
    let core = Arc::new(Ruleset::compile(&[pack], profile).unwrap());
    let part = AdventureRuleset::compile(&core, &[adventure_pack(vec![88, 25], 2)]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        let row: BattleCreatureDef = pack.creatures[1].clone();
        for i in pack.creatures.len()..80 {
            pack.creatures.push(BattleCreatureDef {
                key: key(&if i == 3 {
                    "t:elf".into()
                } else {
                    alloc::format!("t:c{i}")
                }),
                ..row.clone()
            });
        }
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(h3_towns::TownRuleset::empty(&core)),
    )
    .unwrap()
}

/// [`rules_on`] the faithful profile.
fn rules() -> GameRuleset {
    rules_on("t:faithful")
}

/// The creature `raw`.
const fn c(raw: u16) -> CreatureTypeId {
    CreatureTypeId::from_raw(raw)
}

/// An army of `stacks` (creature, count) in slots 0 up.
fn army(stacks: &[(u16, i32)]) -> Army {
    let mut slots = [None; SLOTS];
    for (slot, &(raw, n)) in slots.iter_mut().zip(stacks) {
        *slot = Some((c(raw), n));
    }
    Army(slots)
}

/// `stacks` as the game's stacks, slots 0 up.
fn stacks(stacks: &[(u16, u32)]) -> Vec<ArmyStack> {
    (0..)
        .zip(stacks)
        .map(|(slot, &(raw, count))| ArmyStack {
            slot,
            creature: c(raw),
            count,
        })
        .collect()
}

/// 0.3 as a double.
fn tenths(n: u32) -> SignedX87 {
    SignedX87::from(X87::ratio(n, 10, DOUBLE_BITS).unwrap())
}

/// A plan of `army` (no spare, no Alliance, the hero's morale 0), recounted.
fn plan_of<'a>(rules: &'a GameRuleset, weights: &'a [SignedX87], of: Army) -> Plan<'a> {
    let mut plan = Plan::new(rules, weights, vec![]);
    plan.army = of;
    plan.recount();
    plan
}

// covers: CPU-ARM-001
#[test]
fn the_alignments_and_the_armys_base_morale() {
    let rules = rules();
    let joined = alliance_factions(&rules);
    assert_eq!(
        joined.iter().map(|f| f.raw()).collect::<Vec<_>>(),
        [0, 1, 2, 6, 7]
    );
    // The imp (town 0), the second town's, a neutral and the cart (a war machine, not counted).
    let mut plan = plan_of(&rules, &[], army(&[(0, 1), (4, 1), (3, 1), (2, 1)]));
    assert_eq!(plan.alignments.0, 3);
    assert_eq!(plan.alignments.1[..3], [1, 1, 1]);
    assert_eq!(plan.base_morale(), -1);
    // With the Alliance the second town joins the first: two alignments, morale 0.
    plan.join = &joined;
    plan.recount();
    assert_eq!(plan.alignments.0, 2);
    assert_eq!(plan.alignments.1[..3], [1, 2, 0]);
    assert_eq!(plan.base_morale(), 0);
    assert_eq!(plan.alignment(c(4)), 1);
    // The second town alone joins nothing to count: the first town's place takes it, no fewer.
    plan.army = army(&[(4, 1), (3, 1)]);
    plan.recount();
    assert_eq!(
        (plan.alignments.0, &plan.alignments.1[..3]),
        (2, &[1, 1, 0][..])
    );
    // An undead stack 1 less, an Angel 1 more; the Necropolis outside the Alliance.
    plan.army = army(&[(0, 1), (4, 1), (3, 1), (5, 1)]);
    assert_eq!(plan.base_morale(), -2);
    plan.join = &[];
    assert_eq!(plan.base_morale(), -3);
    plan.army = army(&[(12, 1), (0, 1)]);
    assert_eq!(plan.base_morale(), 2);
}

// covers: CPU-ARM-001
#[test]
fn a_neutral_under_the_alliance_is_its_own_alignment_in_every_profile() {
    // AI-QUIRK-2: the original's position of −1 in the nine-town set throws; fixed, the neutral
    // is the neutral alignment. The plan reads no profile: the same values under each.
    for profile in ["t:faithful", "t:tile"] {
        let rules = rules_on(profile);
        let joined = alliance_factions(&rules);
        let weights = [tenths(3)];
        // A purchase of neutrals into an army of the Alliance's towns.
        let offers = vec![Offer {
            creature: c(3),
            count: 5,
            free: true,
        }];
        let mut plan = Plan::new(&rules, &weights, offers);
        let hero = stacks(&[(0, 10), (4, 10)]);
        let bought = plan.purchase_value((&hero, None), vec![0], (&joined, 0));
        assert_eq!(plan.alignment(c(3)), 0);
        // Morale: the two towns joined, one alignment, 1; the neutral new: 1 is not below 1.
        assert_eq!(bought, 100);
        assert_eq!(plan.alignments.1[..2], [1, 2]);
        // An exchange taking neutrals; the last imp only by a swap of its own creature: none.
        let mut plan = Plan::new(&rules, &weights, vec![]);
        let other = stacks(&[(3, 5), (0, 1)]);
        let taken = plan.exchange_value((&hero, &other), (&joined, 0, 40));
        assert_eq!(taken, 100);
        // A replacement under morale-bad asks a neutral's alignment too.
        plan.army = army(&[(3, 1), (0, 1)]);
        plan.recount();
        assert_eq!(plan.to_replace(false, true), Some(1));
    }
}

// covers: CPU-ARM-002
#[test]
fn a_stacks_worth_its_slot_and_the_speeds_loss() {
    let rules = rules();
    let plan = plan_of(&rules, &[], army(&[(1, 10)]));
    // Its own creature's slot; forced, −1.
    assert_eq!(plan.marginal(c(1), 5, false), (200, Some(0)));
    assert_eq!(plan.marginal(c(1), 5, true), (-1, Some(0)));
    // A faster creature: the first empty slot, no loss.
    assert_eq!(plan.marginal(c(0), 5, false), (50, Some(1)));
    // A slower one: 250 + trunc(1 360 × 3 500 / 1 500 − 3 500), the model's −326.
    let mut slow = plan_of(&rules, &[], army(&[(4, 100)]));
    assert_eq!(slow.marginal(c(6), 10, false), (250 - 326, Some(1)));
    // The word: 65 546 creatures count 10.
    assert_eq!(slow.marginal(c(4), short(65_546), false), (300, Some(0)));
    // Six stacks with a spare: a replacement of the least (the first of two equal).
    slow.army = army(&[(4, 1), (10, 1), (11, 1), (3, 1), (0, 1), (1, 1)]);
    slow.spare = Some(army(&[]));
    slow.recount();
    assert_eq!(slow.marginal(c(9), 1, false), (100 - 1, Some(1)));
    // Without the spare, the free seventh slot.
    slow.spare = None;
    assert_eq!(slow.marginal(c(9), 1, false), (100, Some(6)));
    // Seven stacks: a replacement; none may go: 0.
    slow.army = army(&[(4, 1), (10, 1), (11, 1), (3, 1), (0, 1), (1, 1), (12, 1)]);
    slow.recount();
    assert_eq!(slow.marginal(c(9), 1, false), (99, Some(1)));
    slow.army = army(&[(6, 1), (7, 1), (10, 1), (11, 1), (3, 1), (0, 1), (1, 1)]);
    slow.recount();
    assert_eq!(slow.to_replace(true, false), Some(2));
    assert_eq!(slow.marginal(c(7), 1, true), (-1, Some(1)));
}

// covers: CPU-ARM-002
#[test]
fn a_new_alignment_below_the_morale_is_morale_bad() {
    let rules = rules();
    // Three alignments: base morale −1. The stacks that feel morale are worth 950.
    let mut plan = plan_of(&rules, &[], army(&[(0, 10), (3, 20), (7, 30)]));
    assert_eq!(plan.base_morale(), -1);
    // A feeling creature worth 100: 1 050 ≥ 1 000, morale-bad: only a stack alone in its
    // alignment, never the only shooter (the 15 × 30): the imp's 100.
    assert_eq!(plan.marginal(c(9), 1, false), (100 - 100, Some(0)));
    // One that feels none: 950 < 1 000, the free slot.
    assert_eq!(plan.marginal(c(8), 1, false), (100, Some(3)));
    // A Minotaur in the army does not count: 100 + 450 + 100 < 1 000.
    plan.army = army(&[(0, 10), (78, 5), (7, 30)]);
    plan.recount();
    assert_eq!(plan.marginal(c(9), 1, false), (100, Some(3)));
    // A Necropolis creature needs 2: one alignment (1) is below for it alone.
    plan.army = army(&[(0, 100)]);
    plan.recount();
    assert_eq!(plan.marginal(c(5), 1, false), (60 - 1000, Some(0)));
    assert_eq!(plan.marginal(c(4), 2, false), (60, Some(1)));
    // The hero's morale counts: −1 makes the second town's morale-bad too.
    plan.morale = -1;
    assert_eq!(plan.marginal(c(4), 2, false), (60 - 1000, Some(0)));
    // An empty army: nothing is new against it.
    let empty = plan_of(&rules, &[], army(&[]));
    assert_eq!(empty.marginal(c(5), 1, false), (60, Some(0)));
}

// covers: CPU-ARM-002
#[test]
fn the_replacements_rules_for_shooters() {
    let rules = rules();
    // Four shooting stacks: a new shooter replaces only a shooter (the 1s, the first).
    let plan = plan_of(
        &rules,
        &[],
        army(&[(0, 1), (6, 1), (7, 1), (10, 1), (11, 1)]),
    );
    assert_eq!(plan.to_replace(true, false), Some(3));
    // A non-shooter takes the least of all.
    assert_eq!(plan.to_replace(false, false), Some(3));
    // One shooter: a non-shooter never takes it; a shooter may.
    let plan = plan_of(&rules, &[], army(&[(10, 1), (0, 1)]));
    assert_eq!(plan.to_replace(false, false), Some(1));
    assert_eq!(plan.to_replace(true, false), Some(0));
    let plan = plan_of(&rules, &[], army(&[(10, 1)]));
    assert_eq!(plan.to_replace(false, false), None);
}

// covers: CPU-ARM-003
#[test]
fn a_put_moves_the_displaced_stack_to_the_spare() {
    let rules = rules();
    let mut plan = plan_of(&rules, &[], army(&[(1, 10)]));
    plan.spare = Some(army(&[(1, 3)]));
    // Over the gogs: +50 − 400, the gogs into the spare's.
    plan.put(c(0), 5, 0);
    assert_eq!(plan.gain, 50 - 400);
    assert_eq!(plan.army.0[0], Some((c(0), 5)));
    assert_eq!(plan.spare.unwrap().0[0], Some((c(1), 13)));
    // Into its own creature's slot: added.
    plan.put(c(0), 2, 0);
    assert_eq!(
        (plan.gain, plan.army.0[0]),
        (50 - 400 + 20, Some((c(0), 7)))
    );
    // A full spare: over its least stack below the displaced one (the first of two 1s).
    plan.spare = Some(army(&[
        (4, 1),
        (10, 1),
        (11, 1),
        (3, 1),
        (9, 1),
        (12, 1),
        (6, 1),
    ]));
    plan.put(c(1), 1, 0);
    assert_eq!(plan.spare.unwrap().0[1], Some((c(0), 7)));
    // None below it: the displaced stack is lost.
    let spare = army(&[(4, 1), (7, 1), (12, 1), (3, 1), (9, 1), (8, 1), (6, 1)]);
    plan.spare = Some(spare);
    plan.army = army(&[(10, 2)]);
    plan.put(c(1), 1, 0);
    assert_eq!(plan.spare, Some(spare));
    // No spare: lost too; a count above a word: its low 16 bits (70 000: 4 464).
    plan.spare = None;
    plan.gain = 0;
    plan.army = army(&[(0, 70_000)]);
    plan.put(c(1), 1, 0);
    assert_eq!(plan.gain, 40 - 10 * 4464);
    plan.spare = Some(army(&[]));
    plan.army = army(&[(0, 70_000)]);
    plan.put(c(1), 1, 0);
    assert_eq!(plan.spare.unwrap().0[0], Some((c(0), 4464)));
}

// covers: CPU-ARM-003
#[test]
fn the_shed_moves_worthless_stacks_to_the_spare() {
    let rules = rules();
    // The slow shooter is worth 25 − 326 to the 3 000 of the second town's: shed; the rest kept.
    let mut plan = Plan::new(&rules, &[], vec![]);
    let hero = stacks(&[(4, 100), (6, 1)]);
    let spare = stacks(&[]);
    assert_eq!(
        plan.purchase_value((&hero, Some(&spare)), vec![], (&[], 0)),
        0
    );
    assert_eq!(plan.army, army(&[(4, 100)]));
    assert_eq!(plan.spare, Some(army(&[(6, 1)])));
    // In slot order, until one stack is left; a full spare: nothing is shed.
    let hero = stacks(&[(6, 1), (4, 100)]);
    assert_eq!(
        plan.purchase_value((&hero, Some(&spare)), vec![], (&[], 0)),
        0
    );
    assert_eq!(plan.spare, Some(army(&[(6, 1)])));
    let full = stacks(&[(0, 1), (1, 1), (3, 1), (7, 1), (9, 1), (10, 1), (11, 1)]);
    let hero = stacks(&[(4, 100), (6, 1)]);
    plan.purchase_value((&hero, Some(&full)), vec![], (&[], 0));
    assert_eq!(plan.army.stacks(), 2);
    // No spare: nothing to shed into.
    plan.purchase_value((&hero, None), vec![], (&[], 0));
    assert_eq!(plan.army.stacks(), 2);
    // Two stacks of one creature merge first.
    let twice = stacks(&[(0, 3), (1, 1), (0, 4)]);
    plan.purchase_value((&twice, None), vec![], (&[], 0));
    assert_eq!(plan.army, army(&[(0, 7), (1, 1)]));
}

// covers: CPU-ARM-004
#[test]
fn an_exchange_takes_the_best_stacks_by_the_advantage() {
    let rules = rules();
    let hero = stacks(&[(1, 10)]);
    let other = stacks(&[(0, 20), (4, 5)]);
    // 40: the imps (200) first, then of the five the four (120) the one left keeps; the last
    // one forced is the hero's own creature's (−1 × 40 / 40, not above 0).
    let mut plan = Plan::new(&rules, &[], vec![]);
    assert_eq!(plan.exchange_value((&hero, &other), (&[], 0, 40)), 320);
    assert_eq!(plan.gain, 320);
    assert_eq!(plan.army, army(&[(1, 10), (0, 20), (4, 4)]));
    let mut left = army(&[]);
    left.0[1] = Some((c(4), 1));
    assert_eq!(plan.spare, Some(left));
    // Half the advantage, half of each step (truncated); none (or below 0): nothing.
    let mut plan = Plan::new(&rules, &[], vec![]);
    assert_eq!(plan.exchange_value((&hero, &other), (&[], 0, 20)), 100 + 60);
    let mut plan = Plan::new(&rules, &[], vec![]);
    assert_eq!(plan.exchange_value((&hero, &other), (&[], 0, -5)), 0);
    // A full army and one Angel stack. The shed moves the first 1-worth shooter out first: with
    // six stacks left and a spare it could only replace the other (1 − 1). Then, with two stacks
    // there, both Angels over the other 1-worth shooter (1 000 − 1), which goes there too; neither
    // shooter is then worth a place (1 less the imp's 10).
    let full = stacks(&[(0, 1), (1, 1), (3, 1), (4, 1), (7, 1), (10, 1), (11, 1)]);
    let angels = stacks(&[(12, 2)]);
    let mut plan = Plan::new(&rules, &[], vec![]);
    assert_eq!(plan.exchange_value((&full, &angels), (&[], 0, 40)), 999);
    assert_eq!(plan.army.0[5..], [None, Some((c(12), 2))]);
    assert_eq!(plan.spare, Some(army(&[(11, 1), (10, 1)])));
}

// covers: CPU-ARM-005
#[test]
fn a_purchase_weighs_its_costs_but_keeps_a_quarter() {
    let rules = rules();
    let weights = [tenths(3)];
    assert_eq!(weighted_cost(&weights, &[50]), 15);
    // Truncated at each resource (the model: 1.5, then 2.5): 2, not 3; toward 0.
    let two = [tenths(3), tenths(3)];
    assert_eq!(weighted_cost(&two, &[5, 5]), 2);
    assert_eq!(weighted_cost(&two, &[-5, -5]), -2);
    let hero = stacks(&[(1, 10)]);
    let offers = |free| {
        vec![
            Offer {
                creature: c(0),
                count: 30,
                free,
            },
            Offer {
                creature: c(1),
                count: 10,
                free: false,
            },
        ]
    };
    // 1 000 gold: 20 imps worth 200 less their cost 300 capped at 150; the 10 gogs worth 400
    // less 300: the gogs (100); then nothing is paid.
    let mut plan = Plan::new(&rules, &weights, offers(false));
    assert_eq!(
        plan.purchase_value((&hero, None), vec![1000], (&[], 0)),
        100
    );
    assert_eq!(plan.gain, 400);
    assert_eq!(plan.resources, [0]);
    assert_eq!(plan.offers[1].count, 0);
    assert_eq!(plan.offers[0].count, 30);
    // Free imps whole (300), then the gogs (100).
    let mut plan = Plan::new(&rules, &weights, offers(true));
    assert_eq!(
        plan.purchase_value((&hero, None), vec![1000], (&[], 0)),
        400
    );
    assert_eq!(plan.gain, 700);
    assert_eq!((plan.offers[0].count, plan.offers[1].count), (0, 0));
    // No gold: nothing but the free.
    let mut plan = Plan::new(&rules, &weights, offers(false));
    assert_eq!(plan.purchase_value((&hero, None), vec![], (&[], 0)), 0);
    // An offer worth nothing to the army is not bought.
    let mut plan = Plan::new(
        &rules,
        &weights,
        vec![Offer {
            creature: c(6),
            count: 1,
            free: true,
        }],
    );
    let big = stacks(&[(4, 100)]);
    assert_eq!(plan.purchase_value((&big, None), vec![], (&[], 0)), 0);
    assert_eq!(plan.offers[0].count, 1);
    // A count without a creature the core has: worth 0.
    assert_eq!(creature_costs(&rules, c(999)), [0]);
}
