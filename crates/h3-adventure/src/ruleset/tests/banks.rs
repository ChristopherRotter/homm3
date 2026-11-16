//! Increment 13's kind in the ruleset part (E08 T123a, increment 13's design note §1): the
//! creature bank with its question, levels, looted bonus, deck and coast, compiled, checked and in
//! the hash (part v21). Its draw, visit, battle and reward come with T124b: until then it has no
//! effect.

use h3_content::ArtifactClass;

use super::super::banks::bank_stacks;
use super::*;

/// [`packs`] with `t`'s type `t:obj` of `kind`, visited as `visit`.
fn kind_packs(visit: Visit, kind: ObjectKindDef) -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "t")
            .object_types
            .push(object("t:obj", visit, false, kind));
    })
}

/// `t:obj`'s compile error with `bank`, visited as `visit`.
fn refused_as(visit: Visit, bank: BankDef) -> &'static str {
    match compile_err(&kind_packs(visit, ObjectKindDef::Bank(bank))) {
        AdventureCompileError::InvalidObjectType { key, reason } => {
            assert_eq!(key, "t:obj");
            reason
        }
        other => panic!("{other:?}"),
    }
}

/// `t:obj`'s compile error with `bank`, entered.
fn refused(bank: BankDef) -> &'static str {
    refused_as(Visit::Enter, bank)
}

/// A level of this chance guarded by `count` imps, upgraded at 50 %, giving 4 wood and 2 gogs.
fn level(chance: u8, count: u32) -> BankLevelDef {
    BankLevelDef {
        chance,
        guards: vec![(key("t:imp"), count)],
        upgrade: 50,
        resources: vec![(key("t:wood"), 4)],
        creature: Some((key("t:gog"), 2)),
        artifacts: [0; 4],
    }
}

/// An attack bank of these levels, not on the deck, no coast.
fn bank(levels: Vec<BankLevelDef>) -> BankDef {
    BankDef {
        question: BankQuestion::Attack,
        levels,
        looted: None,
        deck: false,
        coast: false,
    }
}

/// A search bank like the Shipwreck: four levels, looted −1 morale under mark 14, on the deck,
/// from the coast.
fn shipwreck() -> BankDef {
    let mut levels = vec![level(30, 10), level(30, 15), level(30, 25), level(10, 50)];
    for (l, gold) in levels.iter_mut().zip([2000, 3000, 4000, 5000]) {
        l.guards = vec![(key("t:gog"), l.guards[0].1)];
        l.upgrade = 0;
        l.resources = vec![(key("t:gold"), gold)];
        l.creature = None;
    }
    levels[2].artifacts = [1, 0, 0, 0];
    levels[3].artifacts = [0, 1, 0, 0];
    BankDef {
        question: BankQuestion::Search,
        levels,
        looted: Some(OnceBonus {
            morale: -1,
            luck: 0,
            mark: 14,
        }),
        deck: true,
        coast: true,
    }
}

/// The compiled `t:obj` of `bank`, visited as `visit`.
fn compiled(visit: Visit, bank: BankDef) -> ObjectType {
    let part = compile(&kind_packs(visit, ObjectKindDef::Bank(bank))).unwrap();
    part.object_type(part.object_type_by_key("t:obj").unwrap())
        .unwrap()
        .clone()
}

// covers: ADV-BNK-001
#[test]
fn a_creature_bank_compiles_with_its_levels_and_no_visit_yet() {
    let core = core();
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    let resource = |k: &str| {
        let i = core.resources().iter().position(|r| r.key.as_str() == k);
        ResourceId::from_index(i.unwrap()).unwrap()
    };
    let mut def = bank(vec![level(60, 20), level(40, 30)]);
    def.levels[1].guards.push((key("t:gog"), 5));
    def.levels[1].resources = vec![(key("t:gold"), 2500), (key("t:wood"), 2)];
    def.levels[1].creature = None;
    def.levels[1].artifacts = [1, 2, 3, 4];
    let ty = compiled(Visit::Enter, def);
    let expected = Bank {
        question: BankQuestion::Attack,
        levels: vec![
            BankLevel {
                chance: 60,
                guards: vec![(creature("t:imp"), 20)],
                upgrade: 50,
                resources: vec![(resource("t:wood"), 4)],
                creature: Some((creature("t:gog"), 2)),
                artifacts: [0; 4],
            },
            BankLevel {
                chance: 40,
                // The guards in the table's order, the resources in the core's.
                guards: vec![(creature("t:imp"), 30), (creature("t:gog"), 5)],
                upgrade: 50,
                resources: vec![(resource("t:wood"), 2), (resource("t:gold"), 2500)],
                creature: None,
                artifacts: [1, 2, 3, 4],
            },
        ],
        looted: None,
        deck: false,
        coast: false,
    };
    assert_eq!(ty.kind, ObjectKind::Bank(expected));
    // No visit until T124b: a step onto one is refused as any unbuilt visit's.
    assert_eq!(ty.kind.effect(), None);
    assert!(!ty.kind.ownable());
    assert!(!ty.kind.weekly());
    assert_eq!(ty.kind.timed_mark(), None);
    assert_eq!(ty.kind.battle_terrain(), None);
    // A search bank with its looted bonus, on the deck, its coast visited from a neighbour.
    let ty = compiled(Visit::Neighbour, shipwreck());
    let ObjectKind::Bank(sea) = &ty.kind else {
        panic!("{:?}", ty.kind)
    };
    assert_eq!(sea.question, BankQuestion::Search);
    assert_eq!(sea.looted.map(|b| (b.morale, b.mark)), Some((-1, 14)));
    assert!(sea.deck && sea.coast);
    assert_eq!(sea.levels.len(), BANK_LEVELS);
    assert_eq!(sea.levels[3].artifacts, [0, 1, 0, 0]);
    assert_eq!(BANK_ARTIFACT_CLASSES[1], ArtifactClass::Minor);
    // A search bank is entered too (the Crypt), a deck without a coast (the Derelict Ship).
    let mut crypt = shipwreck();
    crypt.coast = false;
    compiled(Visit::Enter, crypt);
}

// covers: ADV-BNK-001
#[test]
fn a_creature_banks_parameters_are_checked() {
    let one = |l: BankLevelDef| bank(vec![l]);
    assert_eq!(
        refused(bank(vec![])),
        "a bank has no level or more than four"
    );
    assert_eq!(
        refused(bank(vec![level(20, 10); 5])),
        "a bank has no level or more than four"
    );
    assert_eq!(
        refused(bank(vec![level(60, 10), level(30, 10)])),
        "the chances do not add up to 100"
    );
    assert_eq!(
        refused(bank(vec![level(100, 10), level(0, 10)])),
        "a chance is 0"
    );
    let mut none = level(100, 10);
    none.guards.clear();
    assert_eq!(
        refused(one(none)),
        "a bank's level has no guard or more than four"
    );
    let mut five = level(100, 10);
    five.guards = vec![(key("t:imp"), 1); 5];
    assert_eq!(
        refused(one(five)),
        "a bank's level has no guard or more than four"
    );
    let mut twice = level(100, 10);
    twice.guards.push((key("t:imp"), 3));
    assert_eq!(refused(one(twice)), "a bank's level names a guard twice");
    let mut machine = level(100, 10);
    machine.guards.push((key("t:cart"), 3));
    assert_eq!(refused(one(machine)), "a bank's guard is a war machine");
    assert_eq!(refused(one(level(100, 0))), "a bank's guard count is 0");
    let mut upgrade = level(100, 10);
    upgrade.upgrade = 101;
    assert_eq!(
        refused(one(upgrade)),
        "a bank's upgrade chance is above 100"
    );
    let mut all = level(100, 10);
    all.upgrade = 100;
    compiled(Visit::Enter, one(all));
    let reward = |creature: &str, count: u32| {
        let mut l = level(100, 10);
        l.creature = Some((key(creature), count));
        one(l)
    };
    assert_eq!(
        refused(reward("t:cart", 1)),
        "a bank's reward creature is a war machine"
    );
    for count in [0, 128] {
        assert_eq!(
            refused(reward("t:gog", count)),
            "a bank's reward count is 0 or above 127"
        );
    }
    compiled(Visit::Enter, reward("t:gog", 127));
    let mut gift = level(100, 10);
    gift.resources = vec![(key("t:wood"), 1), (key("t:wood"), 2)];
    assert_eq!(refused(one(gift)), "a price or gift names a resource twice");
    // No resources is a level's choice (the Griffin Conservatory's).
    let mut bare = level(100, 10);
    bare.resources.clear();
    compiled(Visit::Enter, one(bare));
    let mut attack = shipwreck();
    attack.question = BankQuestion::Attack;
    assert_eq!(
        refused_as(Visit::Neighbour, attack),
        "a bank's looted bonus is not a search's"
    );
    assert_eq!(
        refused(shipwreck()),
        "a bank's coast is not a neighbour-visited one's"
    );
    // The references: an unknown creature (a guard, the reward), an unknown resource.
    let unknown = [
        {
            let mut l = level(100, 10);
            l.guards[0].0 = key("t:nobody");
            (l, AdventureCompileError::UnknownCreature("t:nobody".into()))
        },
        {
            let mut l = level(100, 10);
            l.creature = Some((key("t:nobody"), 1));
            (l, AdventureCompileError::UnknownCreature("t:nobody".into()))
        },
        {
            let mut l = level(100, 10);
            l.resources[0].0 = key("t:ore");
            (l, AdventureCompileError::UnknownResource("t:ore".into()))
        },
    ];
    for (l, error) in unknown {
        assert_eq!(
            compile_err(&kind_packs(Visit::Enter, ObjectKindDef::Bank(one(l)))),
            error
        );
    }
}

// covers: ADV-BNK-001
#[test]
fn a_creature_banks_guards_fit_a_stack_and_the_army_once_split() {
    let with = |max_stack_count: u32, army_slots: u8, guards: usize| {
        let mut l = level(100, 5000);
        l.guards = ["t:imp", "t:gog"][..guards]
            .iter()
            .map(|c| (key(c), 5000))
            .collect();
        let mut packs = kind_packs(Visit::Enter, ObjectKindDef::Bank(bank(vec![l])));
        let params = pack(&mut packs, "t").params.as_mut().unwrap();
        params.max_stack_count = NonZeroU32::new(max_stack_count).unwrap();
        params.army_slots = NonZeroU8::new(army_slots).unwrap();
        compile(&packs).map(|_| ())
    };
    let refused = |reason| {
        Err(AdventureCompileError::InvalidObjectType {
            key: "t:obj".into(),
            reason,
        })
    };
    assert_eq!(with(5000, 7, 1), Ok(()));
    assert_eq!(
        with(4999, 7, 1),
        refused("a bank's guard count is above max_stack_count")
    );
    // One to three stacks split into five: the army needs five slots; four stand as named.
    assert_eq!(with(5000, 5, 2), Ok(()));
    assert_eq!(
        with(5000, 4, 1),
        refused("a bank's guards stand as more stacks than the army's slots")
    );
    assert_eq!(
        (1..=BANK_GUARDS).map(bank_stacks).collect::<Vec<_>>(),
        [5, 5, 5, 4]
    );
}

// covers: ADV-BNK-001
#[test]
fn a_creature_banks_parameters_are_in_the_hash() {
    let hash = |visit: Visit, def: BankDef| {
        compile(&kind_packs(visit, ObjectKindDef::Bank(def)))
            .unwrap()
            .content_hash()
    };
    let base = || bank(vec![level(60, 20), level(40, 30)]);
    let mut changes = vec![base()];
    let mut edit_level = |f: fn(&mut BankLevelDef)| {
        let mut def = base();
        f(&mut def.levels[1]);
        changes.push(def);
    };
    edit_level(|l| l.guards[0].1 = 31);
    edit_level(|l| l.guards[0].0 = key("t:gog"));
    edit_level(|l| l.guards.push((key("t:gog"), 1)));
    edit_level(|l| l.upgrade = 0);
    edit_level(|l| l.resources[0].1 = 5);
    edit_level(|l| l.resources.push((key("t:gold"), 1)));
    edit_level(|l| l.creature = None);
    edit_level(|l| l.creature = Some((key("t:imp"), 2)));
    edit_level(|l| l.creature = Some((key("t:gog"), 3)));
    for i in 0..4 {
        let mut def = base();
        def.levels[1].artifacts[i] = 1;
        changes.push(def);
    }
    // The levels' chances moved with their order kept, and a level more.
    let mut def = base();
    def.levels[0].chance = 40;
    def.levels[1].chance = 60;
    changes.push(def);
    let mut def = base();
    def.levels[1].chance = 10;
    def.levels.push(level(30, 30));
    changes.push(def);
    let mut search = base();
    search.question = BankQuestion::Search;
    changes.push(search.clone());
    let mut looted = search.clone();
    looted.looted = Some(OnceBonus {
        morale: -1,
        luck: 0,
        mark: 15,
    });
    changes.push(looted.clone());
    looted.looted = Some(OnceBonus {
        morale: -1,
        luck: 0,
        mark: 16,
    });
    changes.push(looted);
    let mut deck = base();
    deck.deck = true;
    changes.push(deck);
    let count = changes.len();
    let mut hashes = changes
        .into_iter()
        .map(|def| hash(Visit::Neighbour, def))
        .collect::<BTreeSet<_>>();
    assert_eq!(hashes.len(), count);
    // The coast, and a bank against another kind without parameters.
    let mut coast = base();
    coast.coast = true;
    hashes.insert(hash(Visit::Neighbour, coast));
    hashes.insert(
        compile(&kind_packs(Visit::Neighbour, ObjectKindDef::MagicWell))
            .unwrap()
            .content_hash(),
    );
    assert_eq!(hashes.len(), count + 2);
}
