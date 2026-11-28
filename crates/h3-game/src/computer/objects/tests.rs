//! Tests of the object value (E31 T2h) on the crate's test core with values (the imp Fight Value
//! 100 and AI value 10, the gog 300 and 40, the cart 0 and 50; the battle's hit points 4, 13,
//! 100), four more resources and gems (5) beside the gold (0), a level-5 spell, a market and an
//! object type of each kind the values read, on a 12 × 12 map of grass: red's hero (its 9 imps)
//! on (0, 0), blue's on (2, 2). The weights, the hero's values and the record are set by hand.
//! Where a value adds the fight estimate (T2c), a spell's gain (T2e), a skill's (T2e), an
//! artifact's (T2f) or T2g's values, the test asks that tested function for its part; every
//! number of this task's own comes from an IEEE double model (`E31-log.md` T2h) or is worked by
//! hand.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureRuleset, Altar, BankDef, BankLevelDef, BankQuestion, FindContent, FindContentDef,
    FindDef, Gift, GiftDef, Guards, HeroOnMap, HillFort, Holds, Market, MarketParamsDef,
    MarketScreen, MonolithWay, MonsterSetup, ObjectKind, ObjectKindDef, ObjectSetup, ObjectTypeDef,
    PickupDef, TileMask, TileSetup, Turn, TurnDef, Visit, WarMachineFactoryDef,
};
use h3_content::{
    ArtifactClass, ContentPack, CreatureTypeId, ResourceDef, ResourceId, Ruleset, SpellDef,
    TerrainId,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, PrimarySkill};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87, X87};
use h3_towns::TownRuleset;

use super::super::artifacts::Valued;
use super::super::fight::{self, Fighter, Opponent};
use super::super::spells::SpellGains;
use super::super::start::{HeroValues, Record};
use super::*;
use crate::tests::{adventure_pack, battle_part_with, core_pack, key, setup};
use crate::{Actor, GameCommand, GameEvent, GameRuleset, GameSetup, GameState, apply};

const RED: PlayerId = PlayerId::new(0);
const HERO: HeroId = HeroId::new(0);
const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const GOLD: ResourceId = ResourceId::from_raw(0);
/// The tile beside red's hero the tests' objects stand on.
const AT: Pos = Pos::new(1, 0, 0);

/// A type `k` entered from its tile.
fn entered(k: &str, kind: ObjectKindDef) -> ObjectTypeDef {
    ObjectTypeDef {
        key: key(k),
        name: "Object".into(),
        visit: Visit::Enter,
        from_above: true,
        kind_mark: false,
        kind,
    }
}

/// A bonus visitable of mark `mark` with these days, points and drain.
fn bonus(k: &str, mark: u8, days: [(i8, i8); 7], points: u32, drain: bool) -> ObjectTypeDef {
    entered(
        k,
        ObjectKindDef::Bonus {
            mark,
            days,
            points,
            drain,
        },
    )
}

/// A once-per-hero gift.
fn gift(k: &str, gives: Gives, prices: Vec<Vec<(h3_content::ContentKey, u32)>>) -> ObjectTypeDef {
    entered(
        k,
        ObjectKindDef::Gift(GiftDef {
            gives,
            prices,
            admits: None,
            leave: !matches!(gives, Gives::Primary(_)),
        }),
    )
}

/// A find.
fn find(
    k: &str,
    draws: Vec<(u8, FindContentDef)>,
    full: Option<TurnDef>,
    leaves: bool,
) -> ObjectTypeDef {
    entered(
        k,
        ObjectKindDef::Find(FindDef {
            draws,
            full_backpack: full,
            question: false,
            bonus: None,
            leaves,
        }),
    )
}

/// The resources beside the gold, in order: their keys.
const MORE: [&str; 5] = ["t:r1", "t:r2", "t:r3", "t:r4", "t:gems"];

/// The fixture's rules.
pub(in crate::computer) fn rules() -> GameRuleset {
    let mut pack: ContentPack = core_pack(false);
    for (creature, (fight, ai)) in pack
        .creatures
        .iter_mut()
        .zip([(100, 10), (300, 40), (0, 50)])
    {
        creature.info.fight_value = fight;
        creature.info.ai_value = ai;
        creature.info.growth = 4;
    }
    // The imp level 2, upgraded to the gog; the cart costs 100 gold and a gem.
    pack.creatures[0].info.level = 2;
    pack.creatures[0].info.upgrades = vec![key("t:gog")];
    pack.creatures[2].info.cost = vec![(key("t:gold"), 100), (key("t:gems"), 1)];
    // The book costs 500 gold, the cart's artifact 1 000; the cart's machine is the tent's (6, the
    // original's First Aid Tent), after a fifth artifact.
    pack.artifacts[0].cost = vec![(key("t:gold"), 500)];
    pack.artifacts[1].cost = vec![(key("t:gold"), 1000)];
    pack.artifacts[1].war_machine = None;
    pack.artifacts[2].cost = vec![(key("t:gold"), 1000)];
    pack.artifacts[3].cost = vec![(key("t:gold"), 500)];
    let mut fifth = pack.artifacts[4].clone();
    fifth.key = key("t:fifth");
    let mut tent = pack.artifacts[1].clone();
    tent.key = key("t:aid");
    tent.war_machine = Some(key("t:cart"));
    pack.artifacts.extend([fifth, tent]);
    // Hero types enough for eight heroes beside a prisoner.
    for n in 5..11 {
        let mut hero = pack.heroes[1].clone();
        hero.key = key(&alloc::format!("t:h{n}"));
        pack.heroes.push(hero);
    }
    // Eight skills and more.
    for n in 4..9 {
        let mut skill = pack.skills[0].clone();
        skill.key = key(&alloc::format!("t:s{n}"));
        pack.skills.push(skill);
    }
    for k in MORE {
        pack.resources.push(ResourceDef {
            key: key(k),
            name: "Resource".into(),
        });
    }
    let storm = SpellDef {
        key: key("t:storm"),
        level: 5,
        ..pack.spells[0].clone()
    };
    pack.spells.push(storm);
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 7);
    for k in MORE {
        adventure.object_types.push(entered(
            &alloc::format!("{k}_mine"),
            ObjectKindDef::Mine {
                resource: key(k),
                daily: 1,
                battle_terrain: None,
            },
        ));
    }
    let days = |pair| [pair; 7];
    let gold = |n| vec![(key("t:gold"), n)];
    adventure.object_types.extend([
        bonus("t:buoy", 2, days((1, 0)), 0, false),
        bonus("t:swan", 3, days((0, 2)), 0, true),
        bonus("t:faerie", 9, days((0, 1)), 0, false),
        bonus("t:youth", 10, days((1, 0)), 400, false),
        bonus("t:rally", 12, days((1, 1)), 400, false),
        bonus(
            "t:temple",
            8,
            [(1, 0), (1, 0), (1, 0), (1, 0), (1, 0), (1, 0), (2, 0)],
            0,
            false,
        ),
        entered(
            "t:fortune",
            ObjectKindDef::FountainOfFortune {
                mark: 5,
                values: vec![-1, 1, 2, 3],
            },
        ),
        entered("t:well", ObjectKindDef::MagicWell),
        entered("t:spring", ObjectKindDef::MagicSpring),
        entered(
            "t:stables",
            ObjectKindDef::Stables {
                points: 400,
                from: key("t:imp"),
                to: key("t:gog"),
            },
        ),
        entered(
            "t:bank",
            ObjectKindDef::Bank(BankDef {
                question: BankQuestion::Attack,
                levels: vec![BankLevelDef {
                    chance: 100,
                    guards: vec![(key("t:imp"), 2)],
                    upgrade: 0,
                    resources: vec![(key("t:gold"), 500), (key("t:gems"), 2)],
                    creature: Some((key("t:gog"), 2)),
                    artifacts: [0; 4],
                }],
                looted: None,
                deck: false,
                coast: false,
            }),
        ),
        entered(
            "t:black_market",
            ObjectKindDef::Market(Market {
                screens: vec![MarketScreen::Buy],
                count: 5,
                stock: vec![ArtifactClass::Treasure; 2],
            }),
        ),
        entered(
            "t:hill_fort",
            ObjectKindDef::HillFort(HillFort {
                gold: [0, 25, 50, 75, 100, 100, 100],
            }),
        ),
        entered(
            "t:factory",
            ObjectKindDef::WarMachineFactory(WarMachineFactoryDef {
                machines: vec![key("t:cart")],
            }),
        ),
        entered("t:scholar", ObjectKindDef::Scholar),
        entered("t:shrine", ObjectKindDef::Shrine { level: 1 }),
        entered(
            "t:witch",
            ObjectKindDef::WitchHut {
                roe: vec![key("t:lore")],
                roe_fixed: vec![],
            },
        ),
        gift("t:garden", Gives::Primary([0, 0, 0, 1]), vec![]),
        gift("t:axis", Gives::Primary([0, 0, 1, 0]), vec![]),
        gift("t:marletto", Gives::Primary([0, 1, 0, 0]), vec![]),
        entered(
            "t:library",
            ObjectKindDef::Gift(GiftDef {
                gives: Gives::Primary([2, 2, 2, 2]),
                prices: vec![],
                admits: Some(10),
                leave: false,
            }),
        ),
        gift(
            "t:war_school",
            Gives::Choice {
                skills: [PrimarySkill::Attack, PrimarySkill::Defense],
                amount: 1,
            },
            vec![gold(1000)],
        ),
        find(
            "t:corpse",
            vec![
                (20, FindContentDef::Artifact(vec![ArtifactClass::Treasure])),
                (80, FindContentDef::Nothing),
            ],
            Some(TurnDef::Give(gold(1000))),
            false,
        ),
        find(
            "t:wagon",
            vec![
                (50, FindContentDef::Artifact(vec![ArtifactClass::Treasure])),
                (
                    50,
                    FindContentDef::Resources {
                        amount: [2, 5],
                        resources: vec![key("t:r1")],
                    },
                ),
            ],
            Some(TurnDef::Drawn),
            false,
        ),
        find(
            "t:lean_to",
            vec![(
                100,
                FindContentDef::Resources {
                    amount: [1, 5],
                    resources: vec![key("t:r1")],
                },
            )],
            None,
            false,
        ),
        find(
            "t:flotsam",
            vec![(
                100,
                FindContentDef::Fixed {
                    resources: vec![(key("t:gold"), 100)],
                    artifact: None,
                },
            )],
            None,
            true,
        ),
        find(
            "t:sea_chest",
            vec![(
                100,
                FindContentDef::Fixed {
                    resources: vec![(key("t:gold"), 1000)],
                    artifact: Some(vec![ArtifactClass::Treasure]),
                },
            )],
            Some(TurnDef::Give(gold(1500))),
            true,
        ),
        find(
            "t:survivor",
            vec![(100, FindContentDef::Artifact(vec![ArtifactClass::Treasure]))],
            Some(TurnDef::Lost),
            true,
        ),
        entered(
            "t:campfire",
            ObjectKindDef::Pickup(PickupDef::Paired {
                amount: [4, 6],
                resources: vec![key("t:r1")],
                paired: key("t:gold"),
                multiplier: core::num::NonZeroU32::new(100).unwrap(),
            }),
        ),
        entered(
            "t:windmill",
            ObjectKindDef::Windmill {
                resources: vec![key("t:r1")],
                amount: [3, 6],
            },
        ),
        entered(
            "t:mystic",
            ObjectKindDef::Garden {
                gifts: vec![(key("t:gems"), 5)],
            },
        ),
        entered(
            "t:lighthouse",
            ObjectKindDef::Lighthouse { sea_points: 500 },
        ),
        entered("t:observatory", ObjectKindDef::RevealArea { radius: 20 }),
        entered("t:camp", ObjectKindDef::RefugeeCamp { levels: [1, 2] }),
        entered(
            "t:hut_dwelling",
            ObjectKindDef::Dwelling {
                creatures: vec![key("t:imp")],
                blocked: vec![(0, 0)],
                visitable: vec![(0, 0)],
            },
        ),
        entered("t:scroll_object", ObjectKindDef::Scroll),
        entered(
            "t:random_art",
            ObjectKindDef::Artifact {
                classes: Some(vec![ArtifactClass::Treasure]),
            },
        ),
        bonus("t:mermaid", 11, days((0, 1)), 0, false),
        gift(
            "t:drawn_tree",
            Gives::Level,
            vec![vec![], gold(2000), vec![(key("t:gems"), 10)]],
        ),
    ]);
    let params = adventure.params.as_mut().unwrap();
    params.max_map_size = core::num::NonZeroU16::new(12).unwrap();
    params.dwellings = Some(h3_adventure::DwellingParams {
        guard_level: 5,
        guard_factor: core::num::NonZeroU16::new(3).unwrap(),
        free_level: 1,
    });
    params.market = Some(MarketParamsDef {
        cost: key("t:gold"),
        values: core::iter::once(("t:gold", 1))
            .chain(MORE.iter().map(|&k| (k, 250)))
            .map(|(k, v)| (key(k), v))
            .collect(),
        artifact_values: core::iter::once(("t:gold", 1))
            .chain(MORE.iter().map(|&k| (k, 125)))
            .map(|(k, v)| (key(k), v))
            .collect(),
        resources: vec![10, 20, 30, 40, 50],
        artifacts: vec![20, 25, 30, 35, 40],
        creatures: vec![30, 35, 40, 45, 50],
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        let storm = h3_content_battle::BattleSpellDef {
            key: key("t:storm"),
            ..pack.spells[0].clone()
        };
        pack.spells.push(storm);
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(TownRuleset::empty(&core)),
    )
    .unwrap()
}

/// An object of type `k` at `pos` (one tile, visited from it).
pub(in crate::computer) fn object(rules: &GameRuleset, k: &str, pos: Pos) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: pos,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// The crate's test setup on a 12 × 12 map of grass.
pub(in crate::computer) fn wide() -> GameSetup {
    let mut setup = setup();
    setup.map.width = 12;
    setup.map.height = 12;
    setup.map.tiles = vec![
        TileSetup {
            terrain: TerrainId::from_raw(0),
            road: None,
            river: None,
            coast: false,
        };
        144
    ];
    setup
}

/// The game of [`wide`] with `objects`, `edit` made to its setup.
pub(in crate::computer) fn game(
    rules: &GameRuleset,
    objects: Vec<ObjectSetup>,
    edit: impl FnOnce(&mut GameSetup),
) -> GameState {
    let mut setup = wide();
    setup.map.objects = objects;
    edit(&mut setup);
    GameState::new(rules, &setup, 7).unwrap()
}

/// Red's hero on the map with its details.
pub(in crate::computer) fn red(state: &GameState) -> (HeroOnMap, HeroDetails) {
    let on_map = state.adventure().hero(HERO).unwrap().clone();
    let details = state.hero(HERO).unwrap().clone();
    (on_map, details)
}

/// `stacks` (creature, count) in slots 0 up.
pub(in crate::computer) fn army(stacks: &[(CreatureTypeId, u32)]) -> Vec<ArmyStack> {
    (0..)
        .zip(stacks)
        .map(|(slot, &(creature, count))| ArmyStack {
            slot,
            creature,
            count,
        })
        .collect()
}

/// `n` tenths as a double.
pub(in crate::computer) fn tenths(n: u32) -> SignedX87 {
    SignedX87::from(X87::ratio(n, 10, DOUBLE_BITS).unwrap())
}

/// The f32 bits of `num / den`.
pub(in crate::computer) fn f32_bits(num: u32, den: u32) -> u32 {
    X87::ratio(num, den, SINGLE_BITS).unwrap().to_f32_bits()
}

/// A record with the hero's experience weight `w` (f32 bits), its spell values `gains`, the
/// mean artifact value `mean` (f32 bits) and the mean weight `weight`.
pub(in crate::computer) fn record(w: u32, gains: SpellGains, (mean, weight): (u32, i32)) -> Record {
    Record {
        artifacts: mean,
        mean: weight,
        heroes: vec![(
            HERO,
            HeroValues {
                experience: w,
                spells: gains,
            },
        )],
        ..Record::default()
    }
}

/// What a test asks: the game, the rules, the hero, its record and weights, its destination.
pub(in crate::computer) struct Case<'a> {
    pub(in crate::computer) state: &'a GameState,
    pub(in crate::computer) rules: &'a GameRuleset,
    pub(in crate::computer) hero: (HeroOnMap, HeroDetails),
    pub(in crate::computer) record: Record,
    pub(in crate::computer) weights: Vec<SignedX87>,
    pub(in crate::computer) destination: Option<Pos>,
}

impl<'a> Case<'a> {
    /// Red's hero on `state` with a weight of 0.5, gains of 100 and 200, a mean of 1 000 and a
    /// mean weight of 3, the weights 0.1 (gold) and 1.0 to 1.4 (the others).
    pub(in crate::computer) fn new(state: &'a GameState, rules: &'a GameRuleset) -> Self {
        let gains = SpellGains {
            power: 100,
            knowledge: 200,
            rounds: 0,
            doubled: 300,
            refilled: 400,
        };
        Self {
            state,
            rules,
            hero: red(state),
            record: record(f32_bits(1, 2), gains, (f32_bits(1000, 1), 3)),
            weights: (0..6)
                .map(|n| if n == 0 { tenths(1) } else { tenths(9 + n) })
                .collect(),
            destination: None,
        }
    }

    /// `f` of the case's [`Asking`].
    pub(in crate::computer) fn ask<T>(&self, f: impl FnOnce(Asking<'_, '_>) -> T) -> T {
        let (on_map, details) = &self.hero;
        let hero = Fighter { on_map, details };
        let v = Valued::new(self.state, self.rules, (&self.record, &self.weights), hero);
        f(Asking {
            asked: Asked {
                hero: &v,
                record: &self.record,
            },
            destination: self.destination,
        })
    }

    /// The object value of `pos` at `cost`: the value and the cost after.
    pub(in crate::computer) fn value(&self, pos: Pos, cost: i32) -> (i32, i32) {
        self.ask(|a| {
            let mut cost = cost;
            let v = value(a, pos, &mut cost);
            (v, cost)
        })
    }

    /// The fight estimate of the hero against `army` on `pos`.
    pub(in crate::computer) fn fight(&self, army: &[ArmyStack], pos: Pos) -> i32 {
        let (on_map, details) = &self.hero;
        let opponent = Opponent {
            hero: None,
            army,
            town: None,
        };
        fight::fight(
            self.state,
            self.rules,
            Fighter { on_map, details },
            opponent,
            Some(pos),
        )
    }
}

/// `command` of red, applied.
pub(in crate::computer) fn play(state: &mut GameState, rules: &GameRuleset, command: GameCommand) {
    let mut events: Vec<GameEvent> = Vec::new();
    apply(state, rules, Actor::Player(RED), command, &mut events).unwrap();
}

/// Red's hero's move along `path`.
pub(in crate::computer) fn walk(path: &[Pos]) -> GameCommand {
    GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    })
}

/// The kind of the fixture's type `k`.
pub(in crate::computer) fn kind(rules: &GameRuleset, k: &str) -> ObjectKind {
    let id = rules.adventure().object_type_by_key(k).unwrap();
    rules.adventure().object_type(id).unwrap().kind.clone()
}

// covers: CPU-OBJ-001
#[test]
fn the_class_comes_from_the_kind() {
    let rules = rules();
    let class = |k: &str| Class::of(&kind(&rules, k));
    let pairs = [
        ("t:pile", Class::Resource),
        ("t:campfire", Class::Campfire),
        ("t:chart", Class::Cartographer),
        ("t:observatory", Class::Observatory),
        ("t:tent", Class::Keymaster),
        ("t:guard", Class::Other),
        ("t:holy_ground", Class::Other),
        ("t:monster", Class::Monster),
        ("t:chest", Class::TreasureChest),
        ("t:artifact", Class::Artifact),
        ("t:scroll_object", Class::Scroll),
        ("t:stone", Class::LearningStone),
        ("t:tree", Class::Tree),
        ("t:arena", Class::Arena),
        ("t:school", Class::SchoolOfMagic),
        ("t:war_school", Class::SchoolOfWar),
        ("t:library", Class::Library),
        ("t:axis", Class::StarAxis),
        ("t:garden", Class::GardenOfRevelation),
        ("t:marletto", Class::OneLevel),
        ("t:shrine", Class::Shrine),
        ("t:witch", Class::WitchHut),
        ("t:scholar", Class::Scholar),
        ("t:university", Class::University),
        ("t:tomb", Class::WarriorsTomb),
        ("t:corpse", Class::Corpse),
        ("t:wagon", Class::Wagon),
        ("t:lean_to", Class::LeanTo),
        ("t:flotsam", Class::Flotsam),
        ("t:sea_chest", Class::SeaChest),
        ("t:survivor", Class::Survivor),
        ("t:box", Class::Pandora),
        ("t:event", Class::Other),
        ("t:pyramid", Class::Pyramid),
        ("t:mine", Class::Mine),
        ("t:abandoned", Class::Mine),
        ("t:lighthouse", Class::Lighthouse),
        ("t:shipyard", Class::Shipyard),
        ("t:windmill", Class::Windmill),
        ("t:wheel", Class::WaterWheel),
        ("t:mystic", Class::MysticalGarden),
        ("t:hut_dwelling", Class::Dwelling),
        ("t:camp", Class::RefugeeCamp),
        ("t:entrance", Class::MonolithEntrance),
        ("t:exit", Class::Other),
        ("t:garrison", Class::Garrison),
        ("t:buoy", Class::Buoy),
        ("t:swan", Class::SwanPond),
        ("t:faerie", Class::FaerieRing),
        ("t:youth", Class::Oasis),
        ("t:rally", Class::RallyFlag),
        ("t:temple", Class::Temple),
        ("t:idol", Class::IdolOfFortune),
        ("t:fortune", Class::FountainOfFortune),
        ("t:well", Class::MagicWell),
        ("t:spring", Class::MagicSpring),
        ("t:stables", Class::Stables),
        ("t:sirens", Class::Sirens),
        ("t:bank", Class::Bank),
        ("t:black_market", Class::BlackMarket),
        ("t:hill_fort", Class::HillFort),
        ("t:factory", Class::WarMachineFactory),
        ("t:prison", Class::Prison),
        ("t:market_of_time", Class::Other),
    ];
    for (k, expected) in pairs {
        assert_eq!(class(k), expected, "{k}");
    }
    // The kinds the fixture lacks, made here.
    let market = |screens| {
        ObjectKind::Market(Market {
            screens,
            count: 5,
            stock: vec![],
        })
    };
    let made = [
        (ObjectKind::Message { leaves: false }, Class::Other),
        (ObjectKind::SeaGround, Class::Other),
        (ObjectKind::Hole, Class::Other),
        (ObjectKind::RandomDwelling, Class::Other),
        (ObjectKind::SubterraneanGate, Class::Other),
        (ObjectKind::Tavern, Class::Other),
        (ObjectKind::DenOfThieves, Class::Other),
        (ObjectKind::Whirlpool { divisor: 2 }, Class::Whirlpool),
        (ObjectKind::KeyGate { key: 0 }, Class::Other),
        (
            ObjectKind::Monolith {
                channel: 0,
                way: MonolithWay::TwoWay,
            },
            Class::MonolithTwoWay,
        ),
        (market(vec![MarketScreen::Trade]), Class::Other),
        (
            ObjectKind::Altar(Altar {
                experience: [0; 5],
                keeps: vec![],
                places: 1,
                creature_factor: 1,
                creature_divisor: core::num::NonZeroU32::MIN,
            }),
            Class::Altar,
        ),
        (
            ObjectKind::Town {
                boat_offsets: vec![],
            },
            Class::Town,
        ),
    ];
    for (kind, expected) in made {
        assert_eq!(Class::of(&kind), expected, "{kind:?}");
    }
    // A gift of primary skills of no class; a find that leaves with an artifact that stays.
    let other = Gift {
        gives: Gives::Primary([1, 0, 1, 0]),
        prices: vec![],
        admits: None,
        leave: false,
    };
    assert_eq!(Class::of(&ObjectKind::Gift(other)), Class::Other);
    let leaving = ObjectKind::Find(Find {
        draws: vec![(100, FindContent::Nothing)],
        full_backpack: Some(Turn::Stays),
        question: false,
        bonus: None,
        leaves: true,
    });
    assert_eq!(Class::of(&leaving), Class::Survivor);
    // The scouting's class numbers.
    assert_eq!(Class::Observatory.number(), 58);
    assert_eq!(Class::Other.number(), 0);
}

/// The shared game's objects: a key and a tile.
pub(in crate::computer) const OBJECTS: [(&str, (u16, u16)); 56] = [
    ("t:arena", (0, 4)),
    ("t:marletto", (1, 4)),
    ("t:scholar", (2, 4)),
    ("t:garden", (3, 4)),
    ("t:axis", (4, 4)),
    ("t:library", (5, 4)),
    ("t:stone", (6, 4)),
    ("t:school", (7, 4)),
    ("t:war_school", (8, 4)),
    ("t:tree", (9, 4)),
    ("t:university", (10, 4)),
    ("t:witch", (11, 4)),
    ("t:shrine", (0, 5)),
    ("t:buoy", (1, 5)),
    ("t:faerie", (2, 5)),
    ("t:swan", (3, 5)),
    ("t:youth", (4, 5)),
    ("t:rally", (5, 5)),
    ("t:temple", (6, 5)),
    ("t:idol", (7, 5)),
    ("t:fortune", (8, 5)),
    ("t:stables", (9, 5)),
    ("t:sirens", (10, 5)),
    ("t:well", (11, 5)),
    ("t:spring", (0, 6)),
    ("t:observatory", (1, 6)),
    ("t:tent", (2, 6)),
    ("t:lighthouse", (3, 6)),
    ("t:chart", (4, 6)),
    ("t:campfire", (5, 6)),
    ("t:flotsam", (6, 6)),
    ("t:sea_chest", (7, 6)),
    ("t:survivor", (8, 6)),
    ("t:tomb", (9, 6)),
    ("t:corpse", (10, 6)),
    ("t:lean_to", (11, 6)),
    ("t:wagon", (0, 7)),
    ("t:chest", (1, 7)),
    ("t:mystic", (2, 7)),
    ("t:windmill", (3, 7)),
    ("t:wheel", (4, 7)),
    ("t:black_market", (5, 7)),
    ("t:factory", (6, 7)),
    ("t:hill_fort", (7, 7)),
    ("t:camp", (8, 7)),
    ("t:hut_dwelling", (9, 7)),
    ("t:mine", (10, 7)),
    ("t:bank", (11, 7)),
    ("t:pyramid", (0, 8)),
    ("t:monster", (2, 8)),
    ("t:box", (3, 8)),
    ("t:artifact", (4, 8)),
    ("t:scroll_object", (5, 8)),
    ("t:pile", (6, 8)),
    ("t:garrison", (7, 8)),
    ("t:random_art", (8, 8)),
];

/// The tile of the shared game's object `k`.
pub(in crate::computer) fn tile(k: &str) -> Pos {
    let &(_, (x, y)) = OBJECTS.iter().find(|&&(o, _)| o == k).unwrap();
    Pos::new(x, y, 0)
}

/// The index of the shared game's object `k`.
pub(in crate::computer) fn index(k: &str) -> usize {
    OBJECTS.iter().position(|&(o, _)| o == k).unwrap()
}

/// The ring (the core's artifact 4).
pub(in crate::computer) const RING: h3_content::ArtifactTypeId =
    h3_content::ArtifactTypeId::from_raw(4);
/// The scroll (the core's artifact 3) and the gust (its spell 0).
pub(in crate::computer) const SCROLL: h3_content::ArtifactTypeId =
    h3_content::ArtifactTypeId::from_raw(3);
pub(in crate::computer) const GUST: h3_content::SpellId = h3_content::SpellId::from_raw(0);

/// Pandora's Box's record: 1 000 experience, Attack +1 and Spell Power +2, Lore at Advanced, a
/// ring, the gust, 1 000 gold and 2 gems, 5 imps.
pub(in crate::computer) fn box_record() -> h3_adventure::RewardRecord {
    h3_adventure::RewardRecord {
        hero: h3_adventure::HeroReward {
            experience: 1000,
            primary: [1, 0, 2, 0],
            skills: vec![(h3_content::SkillId::from_raw(1), 2)],
            mana: 0,
            morale: 0,
            luck: 0,
            artifacts: vec![RING],
            spells: vec![GUST],
        },
        resources: vec![(GOLD, 1000), (ResourceId::from_raw(5), 2)],
        creatures: army(&[(IMP, 5)]),
    }
}

/// The shared game: [`OBJECTS`] on [`wide`], `edit` made to its setup.
pub(in crate::computer) fn shared(
    rules: &GameRuleset,
    edit: impl FnOnce(&mut GameSetup),
) -> GameState {
    let objects = OBJECTS
        .iter()
        .map(|&(k, (x, y))| {
            let mut o = object(rules, k, Pos::new(x, y, 0));
            match k {
                "t:monster" => {
                    o.monster = Some(MonsterSetup {
                        creature: Some(IMP),
                        count: 30,
                        disposition: 0,
                        never_flees: false,
                        no_growth: false,
                        treasure: vec![],
                        artifact: None,
                    });
                }
                "t:box" => o.holds = Holds::Record(alloc::boxed::Box::new(box_record())),
                "t:artifact" => o.holds = Holds::Artifact(RING.into()),
                "t:scroll_object" => {
                    o.holds = Holds::Artifact(h3_heroes::ArtifactInstance {
                        artifact: SCROLL,
                        spell: Some(GUST),
                    });
                }
                "t:scholar" => o.holds = Holds::Gift(None),
                "t:shrine" => o.holds = Holds::Spell(None),
                "t:witch" => o.holds = Holds::Skills(Some(vec![h3_content::SkillId::from_raw(1)])),
                "t:garrison" => {
                    o.holds = Holds::Garrison {
                        army: army(&[(GOG, 3)]),
                        removable: true,
                    };
                }
                _ => {}
            }
            o
        })
        .collect();
    game(rules, objects, edit)
}

/// [`Case::new`] with red's hero's army `stacks`.
pub(in crate::computer) fn with_army<'a>(
    state: &'a GameState,
    rules: &'a GameRuleset,
    stacks: &[(CreatureTypeId, u32)],
) -> Case<'a> {
    let mut case = Case::new(state, rules);
    case.hero.0.army = army(stacks);
    case
}

/// The big army: 400 imps and 100 gogs (8 000; its strength 8 800).
pub(in crate::computer) const BIG: [(CreatureTypeId, u32); 2] = [(IMP, 400), (GOG, 100)];

// covers: CPU-OBJ-002
#[test]
fn the_units_are_the_level_the_strength_and_the_gains() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let case = Case::new(&state, &rules);
    // One level: f32(1 000) × 0.5; the strength of 9 imps: 90 × 44 / 40.
    assert_eq!(case.ask(|a| level(a.asked)), 500);
    assert_eq!(case.ask(|a| strength(a.asked)), 99);
    let big = with_army(&state, &rules, &BIG);
    assert_eq!(big.ask(|a| strength(a.asked)), 8800);
    // One morale: 8 800 × 0.0173 (152.24); one and two luck: × f32(0.0173), × 2.
    assert_eq!(big.ask(|a| morale_gain(a.asked, 1)), 152);
    assert_eq!(big.ask(|a| luck_gain(a.asked, 1)), 152);
    assert_eq!(big.ask(|a| luck_gain(a.asked, 2)), 304);
    // An empty army is undead: no morale.
    let none = with_army(&state, &rules, &[]);
    assert_eq!(none.ask(|a| morale_gain(a.asked, 1)), 0);
    // Resources: each step truncated: 1 000 gold at 0.1 (100), then 2 gems at 1.4 (102.8).
    let amounts = [(GOLD, 1000), (ResourceId::from_raw(5), 2)];
    assert_eq!(case.ask(|a| resources_value(a.asked, &amounts)), 102);
    assert_eq!(low16(0x0001_8000), -32768);
}

// covers: CPU-OBJ-001
#[test]
fn a_tile_holds_a_hero_a_boat_or_the_object_a_visit_meets() {
    let rules = rules();
    let state = shared(&rules, |s| {
        s.map.tiles[143].terrain = TerrainId::from_raw(2);
        s.map.boats.push(h3_adventure::BoatSetup {
            tile: Pos::new(11, 11, 0),
            kind: 0,
        });
    });
    let case = Case::new(&state, &rules);
    let class = |pos| at(&state, &rules, pos).map(|a| a.class());
    assert_eq!(class(Pos::new(0, 0, 0)), Some(Class::Hero));
    assert_eq!(class(Pos::new(11, 11, 0)), Some(Class::Boat));
    assert_eq!(class(tile("t:arena")), Some(Class::Arena));
    assert_eq!(class(Pos::new(0, 1, 0)), None);
    // Nothing, a boat and a class of no value are worth 0.
    assert_eq!(case.value(Pos::new(0, 1, 0), 0), (0, 0));
    assert_eq!(case.value(Pos::new(11, 11, 0), 0), (0, 0));
    assert_eq!(case.value(tile("t:chart"), 0), (0, 0));
    // The fight answer is the value with a cost of 0.
    let answer = case.ask(|a| fight_answer(a, tile("t:arena")));
    assert_eq!(answer, case.value(tile("t:arena"), 0).0);
    // A picked pile leaves the map: its tile is no entrance any more.
    let mut picked = game(&rules, vec![object(&rules, "t:pile", AT)], |_| {});
    play(&mut picked, &rules, walk(&[AT]));
    assert!(at(&picked, &rules, AT).is_none());
}

/// The value of the shared game's object `k` for `case` at `cost`.
fn of(case: &Case<'_>, k: &str, cost: i32) -> (i32, i32) {
    case.value(tile(k), cost)
}

// covers: CPU-OBJ-009
#[test]
fn the_experience_and_skill_objects_value_their_gifts() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let case = Case::new(&state, &rules);
    // Two levels; one; one (the Scholar unmarked as always); the knowledge and power values.
    assert_eq!(of(&case, "t:arena", 0).0, 1000);
    assert_eq!(of(&case, "t:marletto", 0).0, 500);
    assert_eq!(of(&case, "t:scholar", 0).0, 500);
    assert_eq!(of(&case, "t:garden", 0).0, 200);
    assert_eq!(of(&case, "t:axis", 0).0, 100);
    // The Library below its condition (level 1); at level 10: 360 / 40 + 2 × 300.
    assert_eq!(of(&case, "t:library", 0).0, 0);
    let mut high = Case::new(&state, &rules);
    high.hero.1.level = 10;
    assert_eq!(of(&high, "t:library", 0).0, 609);
    // The Learning Stone: 0.5 × 1 000; the School of Magic: 200 − 0.1 × 1 000; of War: 500 − 100.
    assert_eq!(of(&case, "t:stone", 0).0, 500);
    assert_eq!(of(&case, "t:school", 0).0, 100);
    assert_eq!(of(&case, "t:war_school", 0).0, 400);
    // The Tree's price unknown: 500 − min(trunc((200 + 14) / 3), 1000 / 3).
    assert_eq!(of(&case, "t:tree", 0).0, 429);
    // The Witch Hut's skill unknown: one level.
    assert_eq!(of(&case, "t:witch", 0).0, 500);
    // The University: its drawn skills by the skills' value.
    let university = case.ask(|a| {
        let asked = a.asked;
        let gains = asked.values().spells;
        let reads = super::super::skills::reads(
            asked.state(),
            &rules,
            asked.fighter(),
            &gains,
            asked.record,
        );
        let offered = state
            .adventure()
            .object(index("t:university"))
            .unwrap()
            .resolved
            .skills();
        super::super::skills::university_value(
            asked.state(),
            &rules,
            asked.fighter(),
            &reads,
            (offered, true),
        )
    });
    assert_eq!(of(&case, "t:university", 0).0, university);
    // The Shrine's spell: no book, nothing; with the book its gain.
    assert_eq!(of(&case, "t:shrine", 0).0, 0);
    let mut booked = Case::new(&state, &rules);
    let book = h3_content::ArtifactTypeId::from_raw(0);
    let at = h3_heroes::first_position(&booked.hero.1, rules.core(), book).unwrap();
    booked.hero.1.worn[at.0] = Some(book.into());
    let spell = match state
        .adventure()
        .object(index("t:shrine"))
        .unwrap()
        .resolved
    {
        h3_adventure::Resolved::Spell(spell) => spell,
        _ => panic!("the shrine drew no spell"),
    };
    let gain = booked.ask(|a| super::super::spells::gain(&rules, &a.asked.hero.caster, spell));
    assert_eq!(of(&booked, "t:shrine", 0).0, gain);
    // A spell above Wisdom + 2 is worth nothing to it.
    let storm = h3_content::SpellId::from_raw(2);
    assert_eq!(booked.ask(|a| learnable_spell(a.asked, storm)), 0);
}

// covers: CPU-OBJ-009
#[test]
fn a_marked_gift_and_a_known_tree_or_hut() {
    let rules = rules();
    // Marletto marks its visitor: nothing more.
    let mut state = game(&rules, vec![object(&rules, "t:marletto", AT)], |_| {});
    let fresh = Case::new(&state, &rules).ask(|a| {
        visits::value(
            a,
            (
                0,
                ty(&rules, "t:marletto"),
                &kind(&rules, "t:marletto"),
                Class::OneLevel,
            ),
            AT,
            &mut 0,
        )
    });
    assert_eq!(fresh, 500);
    play(&mut state, &rules, walk(&[AT]));
    let marked = Case::new(&state, &rules).ask(|a| {
        visits::value(
            a,
            (
                0,
                ty(&rules, "t:marletto"),
                &kind(&rules, "t:marletto"),
                Class::OneLevel,
            ),
            AT,
            &mut 0,
        )
    });
    assert_eq!(marked, 0);
    // The drawn trees, known after a visit (declined): free the level, 2 000 gold 500 − 200, 10
    // gems 0 without them, else 500 − 14.
    let trees: Vec<ObjectSetup> = (1..12)
        .map(|x| object(&rules, "t:drawn_tree", Pos::new(x, 0, 0)))
        .collect();
    let mut state = game(&rules, trees, |s| s.heroes[1].pos = Pos::new(11, 11, 0));
    let before = red(&state);
    let mut events: Vec<GameEvent> = Vec::new();
    for x in 1..12 {
        play(&mut state, &rules, walk(&[Pos::new(x, 0, 0)]));
        while let Some(answer) = crate::default_answer(&state, &rules, RED) {
            let answer = GameCommand::Answer(answer);
            apply(&mut state, &rules, Actor::Player(RED), answer, &mut events).unwrap();
        }
    }
    let prices: Vec<u8> = (0..11)
        .map(|i| match state.adventure().object(i).unwrap().resolved {
            h3_adventure::Resolved::Price(p) => p,
            _ => 0,
        })
        .collect();
    let first = |p| prices.iter().position(|&q| q == p).unwrap();
    // Valued for the hero as it was before its visits (level 1), the free tree for another hero
    // of red's (its levels taken, the visitor marked: 0).
    let tree_for = |state: &GameState, i: usize, hero: HeroId| {
        let pos = Pos::new(u16::try_from(i + 1).unwrap(), 0, 0);
        let k = "t:drawn_tree";
        let mut case = Case::new(state, &rules);
        case.hero = before.clone();
        case.hero.0.id = hero;
        case.record.heroes[0].0 = hero;
        case.ask(|a| {
            visits::value(
                a,
                (i, ty(&rules, k), &kind(&rules, k), Class::Tree),
                pos,
                &mut 0,
            )
        })
    };
    let tree = |state: &GameState, i: usize| tree_for(state, i, HERO);
    assert_eq!(tree(&state, first(0)), 0);
    assert_eq!(tree_for(&state, first(0), HeroId::new(5)), 500);
    assert_eq!(tree(&state, first(1)), 300);
    assert_eq!(tree(&state, first(2)), 0);
    state.set_resource(RED, ResourceId::from_raw(5), 20);
    assert_eq!(tree(&state, first(2)), 486);
    state.set_resource(RED, GOLD, 1999);
    assert_eq!(tree(&state, first(1)), 0);
    // The Witch Hut known: the hero lacking its skill values it by the skills' value.
    let mut state = game(
        &rules,
        vec![{
            let mut o = object(&rules, "t:witch", AT);
            o.holds = Holds::Skills(Some(vec![h3_content::SkillId::from_raw(1)]));
            o
        }],
        |_| {},
    );
    play(&mut state, &rules, walk(&[AT]));
    let mut case = Case::new(&state, &rules);
    let skill = match state.adventure().object(0).unwrap().resolved {
        h3_adventure::Resolved::Skill(skill) => skill,
        _ => panic!("the hut drew no skill"),
    };
    let hut = |case: &Case<'_>| {
        case.ask(|a| {
            visits::value(
                a,
                (
                    0,
                    ty(&rules, "t:witch"),
                    &kind(&rules, "t:witch"),
                    Class::WitchHut,
                ),
                AT,
                &mut 0,
            )
        })
    };
    // It learned the skill at the visit: known and had, nothing.
    assert_eq!(hut(&case), 0);
    case.hero.1.skills.retain(|&(s, _)| s != skill);
    let expected = case.ask(|a| {
        let reads = visits_reads(a.asked);
        if h3_heroes::computer_takes_new_skill(&case.hero.1, rules.core(), &reads, skill) {
            h3_heroes::computer_skill_value(&case.hero.1, rules.core(), &reads, skill)
        } else {
            0
        }
    });
    assert_eq!(hut(&case), expected);
}

/// The skills' reads of `asked`.
fn visits_reads(asked: Asked<'_, '_>) -> h3_heroes::ComputerReads {
    let gains = asked.values().spells;
    super::super::skills::reads(
        asked.state(),
        asked.hero.rules,
        asked.fighter(),
        &gains,
        asked.record,
    )
}

/// The fixture's type `k`.
fn ty(rules: &GameRuleset, k: &str) -> ObjectTypeId {
    rules.adventure().object_type_by_key(k).unwrap()
}

// covers: CPU-OBJ-010
#[test]
fn the_morale_and_luck_objects_value_the_change_by_the_strength() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let big = with_army(&state, &rules, &BIG);
    // Reachable today: the Buoy's one morale, the Faerie Ring's one luck, the Temple's two morale
    // (152.24, 152.24, 304.48); beyond the points left (1 500), nothing.
    assert_eq!(of(&big, "t:buoy", 1500), (152, 1500));
    assert_eq!(of(&big, "t:buoy", 1501), (0, 1501));
    assert_eq!(of(&big, "t:faerie", 100).0, 152);
    assert_eq!(of(&big, "t:temple", 100).0, 304);
    // The Swan Pond only within 200 of the points left, the cost then the day's points.
    assert_eq!(of(&big, "t:swan", 1299), (0, 1299));
    assert_eq!(of(&big, "t:swan", 1300), (304, 1500));
    // The Fountain and the Idol of Fortune: a bare fraction, truncated (survey §5.5 #3).
    assert_eq!(of(&big, "t:fortune", 0).0, 0);
    assert_eq!(of(&big, "t:idol", 0).0, 0);
    // The Fountain of Youth: 200 off the cost and one morale; a smaller cost covered (10 000).
    assert_eq!(of(&big, "t:youth", 300), (152, 100));
    assert_eq!(of(&big, "t:youth", 199), (10_000, 0));
    let none = with_army(&state, &rules, &[]);
    assert_eq!(of(&none, "t:youth", 300), (0, 100));
    // The Rally Flag: its part plus the bare fractions (152 + 0.0346: 152); none beyond reach.
    assert_eq!(of(&big, "t:rally", 300), (152, 100));
    assert_eq!(of(&big, "t:rally", 1501), (0, 1501));
    assert_eq!(of(&none, "t:rally", 300), (0, 100));
}

// covers: CPU-OBJ-002, CPU-OBJ-010
#[test]
fn the_luck_clamp_values_a_hero_at_three_as_from_zero() {
    let rules = rules();
    // The Faerie Ring, the Mermaids and the Rally Flag give luck 3, then the Swan Pond's.
    let path = [Pos::new(1, 0, 0), Pos::new(2, 0, 0), Pos::new(3, 0, 0)];
    let objects = vec![
        object(&rules, "t:faerie", path[0]),
        object(&rules, "t:mermaid", path[1]),
        object(&rules, "t:rally", path[2]),
        object(&rules, "t:swan", Pos::new(5, 0, 0)),
    ];
    let mut state = game(&rules, objects, |s| s.heroes[1].pos = Pos::new(11, 11, 0));
    for step in path {
        play(&mut state, &rules, walk(&[step]));
    }
    let mut big = Case::new(&state, &rules);
    big.hero.0.army = army(&BIG);
    assert_eq!(big.ask(|a| luck(a.asked)), 3);
    // 3 + 2 passes 3: valued as two points from 0 (8 800 × 2 × 0.0173), not 0.
    let k = "t:swan";
    let swan = big.ask(|a| {
        let mut cost = 1400;
        visits::value(
            a,
            (3, ty(&rules, k), &kind(&rules, k), Class::SwanPond),
            Pos::new(5, 0, 0),
            &mut cost,
        )
    });
    assert_eq!(swan, 304);
    // The Faerie Ring's mark: worth nothing to its visitor any more.
    let k = "t:faerie";
    let ring = big.ask(|a| {
        visits::value(
            a,
            (0, ty(&rules, k), &kind(&rules, k), Class::FaerieRing),
            path[0],
            &mut 0,
        )
    });
    assert_eq!(ring, 0);
}

// covers: CPU-OBJ-010
#[test]
fn the_stables_the_sirens_and_the_magic_springs() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let big = with_army(&state, &rules, &BIG);
    // Day 1: 7 × 400 / 2 = 1 400 off the cost (base 50), or the cost covered (10 000); one imp
    // stack upgraded (40 − 10, × 1.2 with a gog stack: 36).
    assert_eq!(of(&big, "t:stables", 1500), (86, 100));
    assert_eq!(of(&big, "t:stables", 1399), (10_036, 0));
    let imps = with_army(&state, &rules, &[(IMP, 4), (IMP, 5)]);
    assert_eq!(of(&imps, "t:stables", 1500), (110, 100));
    let gogs = with_army(&state, &rules, &[(GOG, 4)]);
    assert_eq!(of(&gogs, "t:stables", 1500), (50, 100));
    // The Sirens: 280 imps and 70 gogs kept, 870 hit points lost, 870 experience at
    // (2 500 + 5 600) / 40 000, less 2 400 × 44 / 40; with a weight of 0, less the strength alone.
    assert_eq!(of(&big, "t:sirens", 0).0, -2463);
    let mut weightless = with_army(&state, &rules, &BIG);
    weightless.record.heroes[0].1.experience = 0;
    assert_eq!(of(&weightless, "t:sirens", 0).0, -2640);
    // A stack of one loses nothing.
    let one = with_army(&state, &rules, &[(IMP, 1)]);
    assert_eq!(of(&one, "t:sirens", 0).0, 0);
    // A Well's extra mana value, a Spring's doubled, wherever the hero is headed within 300.
    let case = Case::new(&state, &rules);
    assert_eq!(of(&case, "t:well", 301).0, 400);
    assert_eq!(of(&case, "t:spring", 301).0, 300);
    let mut headed = Case::new(&state, &rules);
    headed.destination = Some(tile("t:spring"));
    assert_eq!(of(&headed, "t:well", 301).0, 400);
    assert_eq!(of(&headed, "t:well", 300).0, 400);
    headed.destination = Some(tile("t:arena"));
    assert_eq!(of(&headed, "t:well", 301).0, 0);
    assert_eq!(of(&headed, "t:spring", 301).0, 0);
}

// covers: CPU-OBJ-010
#[test]
fn a_drunk_well_an_emptied_spring_and_the_stables_mark() {
    let rules = rules();
    let objects = vec![
        object(&rules, "t:well", AT),
        object(&rules, "t:spring", Pos::new(2, 0, 0)),
        object(&rules, "t:stables", Pos::new(3, 0, 0)),
    ];
    let mut state = game(&rules, objects, |s| s.heroes[1].pos = Pos::new(11, 11, 0));
    state.set_mana(HERO, 0);
    for x in 1..4 {
        play(&mut state, &rules, walk(&[Pos::new(x, 0, 0)]));
    }
    let mut case = Case::new(&state, &rules);
    case.hero.0.army = army(&BIG);
    let value = |i: usize, k: &str, class| {
        let pos = Pos::new(u16::try_from(i + 1).unwrap(), 0, 0);
        case.ask(|a| {
            let mut cost = 1500;
            let v = visits::value(
                a,
                (i, ty(&rules, k), &kind(&rules, k), class),
                pos,
                &mut cost,
            );
            (v, cost)
        })
    };
    assert_eq!(value(0, "t:well", Class::MagicWell).0, 0);
    assert_eq!(value(1, "t:spring", Class::MagicSpring).0, 0);
    // The week's mark: no bonus off the cost, the upgrade alone.
    assert_eq!(value(2, "t:stables", Class::Stables), (36, 1500));
}

/// A full backpack of rings.
fn full_backpack(case: &mut Case<'_>) {
    case.hero.1.backpack = vec![RING.into(); 64];
}

// covers: CPU-OBJ-011
#[test]
fn the_finds_and_the_flat_values() {
    let rules = rules();
    let state = shared(&rules, |s| {
        s.map.objects[index("t:lighthouse")].owner = None
    });
    let case = Case::new(&state, &rules);
    let mut full = Case::new(&state, &rules);
    full_backpack(&mut full);
    // The Keymaster's 5 000 while its key is missing; the Lighthouse's 1 000 while not the team's.
    assert_eq!(of(&case, "t:tent", 0).0, 5000);
    assert_eq!(of(&case, "t:lighthouse", 0).0, 1000);
    let owned = shared(&rules, |s| {
        s.map.objects[index("t:lighthouse")].owner = Some(RED)
    });
    assert_eq!(of(&Case::new(&owned, &rules), "t:lighthouse", 0).0, 0);
    // The Campfire's 5 of a resource at 1.0 and 500 gold at 0.1; the Flotsam's 175 gold and 5
    // wood (the core's resource 0, the gold here).
    assert_eq!(of(&case, "t:campfire", 0).0, 55);
    assert_eq!(of(&case, "t:flotsam", 0).0, 18);
    // The Sea Chest: a tenth of the mean and 1 200 gold, the gold alone with a full backpack.
    assert_eq!(of(&case, "t:sea_chest", 0).0, 220);
    assert_eq!(of(&full, "t:sea_chest", 0).0, 120);
    // The Survivor and the Warrior's Tomb the mean, nothing with a full backpack.
    assert_eq!(of(&case, "t:survivor", 0).0, 1000);
    assert_eq!(of(&full, "t:survivor", 0).0, 0);
    assert_eq!(of(&case, "t:tomb", 0).0, 1000);
    assert_eq!(of(&full, "t:tomb", 0).0, 0);
    // The Corpse a fifth of the mean, 200 gold with a full backpack; the Lean-To three mean
    // weights; the Wagon two fifths of the mean and 7 / 4 of them.
    assert_eq!(of(&case, "t:corpse", 0).0, 200);
    assert_eq!(of(&full, "t:corpse", 0).0, 20);
    assert_eq!(of(&case, "t:lean_to", 0).0, 9);
    assert_eq!(of(&case, "t:wagon", 0).0, 405);
    // The Treasure Chest: max(80, 32) + max(160, 48) + max(232, 62) + 1 000 / 20.
    assert_eq!(of(&case, "t:chest", 0).0, 522);
    let mut golden = Case::new(&state, &rules);
    golden.weights[0] = tenths(20);
    assert_eq!(of(&golden, "t:chest", 0).0, 640 + 960 + 1240 + 50);
}

// covers: CPU-OBJ-011
#[test]
fn a_known_find_a_taken_key_and_the_weekly_objects() {
    let rules = rules();
    let objects = vec![
        object(&rules, "t:tent", AT),
        object(&rules, "t:corpse", Pos::new(2, 0, 0)),
        object(&rules, "t:windmill", Pos::new(3, 0, 0)),
        object(&rules, "t:mystic", Pos::new(4, 0, 0)),
        object(&rules, "t:wheel", Pos::new(5, 0, 0)),
    ];
    let mut state = game(&rules, objects, |s| s.heroes[1].pos = Pos::new(11, 11, 0));
    let k = |i: usize| ["t:tent", "t:corpse", "t:windmill", "t:mystic", "t:wheel"][i];
    let value = |state: &GameState, i: usize| {
        let pos = Pos::new(u16::try_from(i + 1).unwrap(), 0, 0);
        let class = Class::of(&kind(&rules, k(i)));
        Case::new(state, &rules).ask(|a| {
            visits::value(
                a,
                (i, ty(&rules, k(i)), &kind(&rules, k(i)), class),
                pos,
                &mut 0,
            )
        })
    };
    // Unvisited: the windmill 3 × 9 / 2, the garden (50 + 7) / 2, the wheel 1 000 gold at 0.1.
    assert_eq!(value(&state, 2), 13);
    assert_eq!(value(&state, 3), 28);
    assert_eq!(value(&state, 4), 100);
    for x in 1..6 {
        play(&mut state, &rules, walk(&[Pos::new(x, 0, 0)]));
    }
    // The key held, the corpse known, the windmill visited and empty, the garden and the wheel
    // emptied (the wheel's 0 gold at its weight).
    for i in 0..5 {
        assert_eq!(value(&state, i), 0, "{}", k(i));
    }
    // A known object asked as a tree without a drawn price is the level; as a hut without a
    // skill, nothing.
    let corpse = (1, ty(&rules, "t:corpse"), &kind(&rules, "t:corpse"));
    let as_class = |class| {
        Case::new(&state, &rules).ask(|a| {
            visits::value(
                a,
                (corpse.0, corpse.1, corpse.2, class),
                Pos::new(2, 0, 0),
                &mut 0,
            )
        })
    };
    assert_eq!(as_class(Class::Tree), 500);
    assert_eq!(as_class(Class::WitchHut), 0);
}

// covers: CPU-OBJ-011
#[test]
fn the_black_market_and_the_war_machine_factory() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let case = Case::new(&state, &rules);
    // The stock: the charm (its 1 000 at the rate 0.4: 2 499 gold, beyond the 2 000) and the
    // scroll (1 249 gold at 0.1: its value less 124.9).
    let entries = state
        .adventure()
        .object(index("t:black_market"))
        .unwrap()
        .resolved
        .stock()
        .to_vec();
    let charm = h3_content::ArtifactTypeId::from_raw(2);
    let scroll = h3_content::ArtifactTypeId::from_raw(3);
    assert_eq!(entries, vec![charm, scroll]);
    let worth = |case: &Case<'_>, artifact: h3_content::ArtifactTypeId| {
        case.ask(|a| {
            let asked = a.asked;
            let heroes = artifacts::players_heroes(
                asked.state(),
                &rules,
                (asked.record, asked.hero.weights),
                (RED, &[]),
            );
            artifacts::value_to_player(&heroes, artifact.into())
        })
    };
    let book = worth(&case, entries[1]);
    let less = |v: i32, x: SignedX87| SignedX87::int(v.into()).minus(x).ftol().max(0);
    let price = SignedX87::int(1249).times(tenths(1));
    assert_eq!(of(&case, "t:black_market", 0).0, less(book, price));
    let mut full = Case::new(&state, &rules);
    full_backpack(&mut full);
    assert_eq!(of(&full, "t:black_market", 0).0, 0);
    // With 100 of the first resource and 20 gems: the cart's for 19 of the first (19 × 1.0, the
    // gems' 19 × 1.4 dearer), the book's for 9 of it.
    let mut paid = state.clone();
    paid.set_resource(RED, ResourceId::from_raw(1), 100);
    paid.set_resource(RED, ResourceId::from_raw(5), 20);
    let rich = Case::new(&paid, &rules);
    let cart = worth(&rich, entries[0]);
    let book = worth(&rich, entries[1]);
    let expected = less(cart, tenths(190)).wrapping_add(less(book, tenths(90)));
    assert_eq!(of(&rich, "t:black_market", 0).0, expected);
    // The factory's tent (the original's 6): its value counted, less its cart's cost weighed
    // (100 gold at 0.1, a gem at 1.4: 11) beyond 500, whole within; none without a gem.
    let tent = h3_content::ArtifactTypeId::from_raw(6);
    let mut machine = Case::new(&paid, &rules);
    machine.hero.0.army = army(&BIG);
    let value = machine.ask(|a| artifacts::hero_value(a.asked.hero, tent.into(), false, true));
    assert!(value > 11);
    assert_eq!(of(&machine, "t:factory", 501).0, value - 11);
    assert_eq!(of(&machine, "t:factory", 500).0, value);
    // A price above the value: nothing.
    let mut dear = Case::new(&paid, &rules);
    dear.hero.0.army = army(&BIG);
    dear.weights[0] = tenths(100);
    assert_eq!(of(&dear, "t:factory", 501).0, 0);
    assert_eq!(of(&case, "t:factory", 500).0, 0);
    let mut carrying = Case::new(&paid, &rules);
    carrying.hero.0.army = army(&BIG);
    carrying.hero.1.backpack = vec![tent.into()];
    assert_eq!(of(&carrying, "t:factory", 500).0, 0);
    let easy = shared(&rules, |s| s.difficulty = 0);
    let mut easy_case = Case::new(&easy, &rules);
    easy_case.hero.0.army = army(&BIG);
    assert_eq!(of(&easy_case, "t:factory", 500).0, 0);
}

/// Plays red's battle to its end (each side's last listed command that stays in it), then
/// answers what it opened by default.
fn fight_out(state: &mut GameState, rules: &GameRuleset) {
    let mut events: Vec<GameEvent> = Vec::new();
    for _ in 0..4000 {
        let Some(battle) = state.battle(RED) else {
            break;
        };
        let actor = battle.owner_to_act().map_or(Actor::System, Actor::Player);
        let listed = crate::legal_commands(state, rules, actor);
        let stays = |c: &&GameCommand| !matches!(c, GameCommand::Battle(c) if c.leaves_battle());
        let command = listed.iter().rfind(stays).unwrap().clone();
        apply(state, rules, actor, command, &mut events).unwrap();
    }
    while let Some(answer) = crate::default_answer(state, rules, RED) {
        apply(
            state,
            rules,
            Actor::Player(RED),
            GameCommand::Answer(answer),
            &mut events,
        )
        .unwrap();
    }
}

/// Red's hero visits `pos`, accepts the question and fights it out.
fn take(state: &mut GameState, rules: &GameRuleset, pos: Pos) {
    play(state, rules, walk(&[pos]));
    let accept = GameCommand::Answer(crate::Answer::Offer(crate::Consent::Accept));
    play(state, rules, accept);
    fight_out(state, rules);
}

// covers: CPU-OBJ-003
#[test]
fn a_guarded_pickup_adds_its_guards_fight() {
    let rules = rules();
    let guards = army(&[(GOG, 1)]);
    let state = shared(&rules, |s| {
        s.map.objects[index("t:pile")].guards = Some(Guards {
            army: guards.clone(),
            question: false,
        });
    });
    let case = Case::new(&state, &rules);
    let mut full = Case::new(&state, &rules);
    full_backpack(&mut full);
    let to_player = |case: &Case<'_>, item: h3_heroes::ArtifactInstance| {
        case.ask(|a| {
            let asked = a.asked;
            let heroes = artifacts::players_heroes(
                asked.state(),
                &rules,
                (asked.record, asked.hero.weights),
                (RED, &[]),
            );
            artifacts::value_to_player(&heroes, item)
        })
    };
    // The ring: its value to the player (at least 10); the random artifact's draw likewise.
    assert_eq!(
        of(&case, "t:artifact", 0).0,
        to_player(&case, RING.into()).max(10)
    );
    let drawn = state
        .adventure()
        .object(index("t:random_art"))
        .unwrap()
        .resolved
        .artifact()
        .unwrap();
    assert_eq!(
        of(&case, "t:random_art", 0).0,
        to_player(&case, drawn.into()).max(10)
    );
    assert_eq!(of(&full, "t:artifact", 0).0, 0);
    // The scroll: 10 and the original's scroll (1) of its spell as a candidate.
    let gain = case.ask(|a| {
        let item = h3_heroes::ArtifactInstance {
            artifact: h3_content::ArtifactTypeId::from_raw(1),
            spell: Some(GUST),
        };
        artifacts::hero_value(a.asked.hero, item, false, false)
    });
    assert_eq!(of(&case, "t:scroll_object", 0).0, 10 + gain);
    assert_eq!(of(&full, "t:scroll_object", 0).0, 0);
    let mut holding = Case::new(&state, &rules);
    holding.hero.1.backpack = vec![h3_heroes::ArtifactInstance {
        artifact: SCROLL,
        spell: Some(GUST),
    }];
    assert_eq!(of(&holding, "t:scroll_object", 0).0, 10);
    // The pile: its gold at 0.1 plus its guards' fight.
    let (resource, amount) = state.adventure().object(index("t:pile")).unwrap().contents[0];
    assert_eq!(resource, GOLD);
    let fight = case.fight(&guards, tile("t:pile"));
    let expected = SignedX87::int(amount.into())
        .times(tenths(1))
        .plus(SignedX87::int(fight.into()))
        .ftol();
    assert_eq!(of(&case, "t:pile", 0).0, expected);
    // Unguarded: the gold alone.
    let bare = game(&rules, vec![object(&rules, "t:pile", AT)], |_| {});
    let (_, amount) = bare.adventure().object(0).unwrap().contents[0];
    let gold = SignedX87::int(amount.into()).times(tenths(1)).ftol();
    assert_eq!(Case::new(&bare, &rules).value(AT, 0).0, gold);
    // Guarded too: the artifact and the scroll.
    let state = shared(&rules, |s| {
        for k in ["t:artifact", "t:scroll_object"] {
            s.map.objects[index(k)].guards = Some(Guards {
                army: guards.clone(),
                question: false,
            });
        }
    });
    let case = Case::new(&state, &rules);
    let fight = case.fight(&guards, tile("t:artifact"));
    assert_eq!(
        of(&case, "t:artifact", 0).0,
        fight.wrapping_add(to_player(&case, RING.into()).max(10))
    );
    let fight = case.fight(&guards, tile("t:scroll_object"));
    assert_eq!(of(&case, "t:scroll_object", 0).0, fight + 10 + gain);
}

// covers: CPU-OBJ-004
#[test]
fn pandoras_box_adds_its_record_up() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let case = Case::new(&state, &rules);
    // 500 experience, 102 of resources, 3 primary levels (1 500), Lore's two (1 000), the ring at
    // the mean (1 000), 5 imps (50).
    assert_eq!(of(&case, "t:box", 0).0, 4152);
    // Lore at Basic: one level; at Expert or with eight skills: none.
    let lore = h3_content::SkillId::from_raw(1);
    let mut basic = Case::new(&state, &rules);
    basic.hero.1.skills = vec![(lore, 1)];
    assert_eq!(of(&basic, "t:box", 0).0, 3652);
    basic.hero.1.skills = vec![(lore, 3)];
    assert_eq!(of(&basic, "t:box", 0).0, 3152);
    basic.hero.1.skills = (4..12)
        .map(|s| (h3_content::SkillId::from_raw(s), 1))
        .collect();
    assert_eq!(of(&basic, "t:box", 0).0, 3152);
    // A full army of gogs takes no imps; a book adds the gust's gain.
    let gogs = with_army(&state, &rules, &[(GOG, 1); 7]);
    assert_eq!(of(&gogs, "t:box", 0).0, 4102);
    let mut booked = Case::new(&state, &rules);
    let book = h3_content::ArtifactTypeId::from_raw(0);
    let at = h3_heroes::first_position(&booked.hero.1, rules.core(), book).unwrap();
    booked.hero.1.worn[at.0] = Some(book.into());
    let gain = booked.ask(|a| super::super::spells::gain(&rules, &a.asked.hero.caster, GUST));
    assert_eq!(of(&booked, "t:box", 0).0, 4152 + gain);
    // Guarded, with experience past 2^31 (none): the fight, then the rest.
    let guards = army(&[(GOG, 1)]);
    let state = shared(&rules, |s| {
        let o = &mut s.map.objects[index("t:box")];
        o.guards = Some(Guards {
            army: guards.clone(),
            question: false,
        });
        let mut record = box_record();
        record.hero.experience = 3_000_000_000;
        o.holds = Holds::Record(alloc::boxed::Box::new(record));
    });
    let case = Case::new(&state, &rules);
    let fight = case.fight(&guards, tile("t:box"));
    let before = fight.wrapping_add(102 + 1500 + 1000);
    let with = SignedX87::int(1)
        .times(SignedX87::from_f32_bits(f32_bits(1000, 1)))
        .plus(SignedX87::int(before.into()).round(SINGLE_BITS))
        .ftol();
    assert_eq!(of(&case, "t:box", 0).0, with + 50);
}

// covers: CPU-OBJ-005
#[test]
fn a_bank_is_its_fight_and_its_rewards_until_looted() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let case = Case::new(&state, &rules);
    let guards = state.adventure().guards_left(index("t:bank")).to_vec();
    let fight = case.fight(&guards, tile("t:bank"));
    // 500 gold at 0.1 and 2 gems at 1.4 (52), 2 gogs (80), no artifact.
    assert_eq!(of(&case, "t:bank", 0).0, fight + 52 + 80);
    // A lost fight (no army) alone.
    let weak = with_army(&state, &rules, &[]);
    let lost = weak.fight(&guards, tile("t:bank"));
    assert!(lost <= -500_000_000);
    assert_eq!(of(&weak, "t:bank", 0).0, lost);
    // Looted: nothing.
    let mut state = game(&rules, vec![object(&rules, "t:bank", AT)], |s| {
        s.heroes[0].army = h3_game_army(&BIG);
    });
    take(&mut state, &rules, AT);
    assert!(state.adventure().is_emptied(0));
    let case = Case::new(&state, &rules);
    let k = "t:bank";
    let looted = case.ask(|a| fights::bank(a.asked, (0, &kind(&rules, k)), AT));
    assert_eq!(looted, 0);
}

/// A setup's custom army of `stacks`.
fn h3_game_army(stacks: &[(CreatureTypeId, u32)]) -> crate::HeroArmy {
    crate::HeroArmy::Custom(army(stacks))
}

// covers: CPU-OBJ-006
#[test]
fn a_monster_a_mine_a_prison_and_the_pyramid() {
    let rules = rules();
    let setup_prisoner = |s: &mut GameSetup| {
        crate::tests::imprison(
            &rules,
            s,
            (1, 8),
            crate::tests::held(h3_content::HeroTypeId::from_raw(1)),
        );
    };
    let state = shared(&rules, setup_prisoner);
    let case = Case::new(&state, &rules);
    // The monster's 30 imps over seven slots: 4, 4, 4, 4, 4, 5, 5.
    let spread = fights::fill_split(IMP, 30);
    assert_eq!(
        spread.iter().map(|s| s.count).collect::<Vec<_>>(),
        [4, 4, 4, 4, 4, 5, 5]
    );
    assert_eq!(
        fights::fill_split(IMP, 3)
            .iter()
            .map(|s| s.slot)
            .collect::<Vec<_>>(),
        [4, 5, 6]
    );
    assert_eq!(
        of(&case, "t:monster", 0).0,
        case.fight(&spread, tile("t:monster"))
    );
    // The neutral gold mine: two days of its income at 0.1, times 1.
    let (resource, daily) = state
        .adventure()
        .production(rules.adventure(), index("t:mine"))
        .unwrap();
    assert_eq!(resource, GOLD);
    let income = SignedX87::int(daily.into())
        .times(tenths(1))
        .times(SignedX87::int(2))
        .ftol();
    assert_eq!(of(&case, "t:mine", 0).0, income);
    // Red's own: nothing; blue's (a human on Normal: 0.5): 1.5 times.
    let owned = |owner| {
        let state = shared(&rules, |s| {
            s.map.objects[index("t:mine")].owner = Some(owner)
        });
        Case::new(&state, &rules).value(tile("t:mine"), 0).0
    };
    assert_eq!(owned(RED), 0);
    let half = SignedX87::ONE
        .plus(tenths(5))
        .times(SignedX87::int(income.into()))
        .ftol();
    assert_eq!(owned(PlayerId::new(1)), half);
    // The abandoned mine's guards: a lost fight alone.
    let mut abandoned = object(&rules, "t:abandoned", AT);
    abandoned.holds = Holds::Resources(vec![GOLD]);
    let state_mine = game(&rules, vec![abandoned], |_| {});
    let case_mine = with_army(&state_mine, &rules, &[]);
    let guards = state_mine.adventure().mine_guards(rules.adventure(), 0);
    assert!(!guards.is_empty());
    let lost = case_mine.fight(&guards, AT);
    assert!(lost <= -500_000_000);
    assert_eq!(case_mine.value(AT, 0).0, lost);
    let strong = with_army(&state_mine, &rules, &[(GOG, 4000)]);
    let won = strong.fight(&guards, AT);
    assert!(won > -500_000_000);
    let (resource, daily) = state_mine
        .adventure()
        .production(rules.adventure(), 0)
        .unwrap();
    let weight = strong.weights[resource.index()];
    let income = SignedX87::int(daily.into())
        .times(weight)
        .times(SignedX87::int(2))
        .ftol();
    assert_eq!(strong.value(AT, 0).0, won + income);
    // The Prison: 2 500 gold at 0.1 and its prisoner's imp; none with eight heroes.
    let prison = state.adventure().map().objects().len() - 1;
    let prisoner = Case::new(&state, &rules).ask(|a| fights::prison(a.asked, prison));
    assert_eq!(prisoner, 250 + 10);
    assert_eq!(case.value(Pos::new(1, 8, 0), 0).0, prisoner);
    let crowded = shared(&rules, |s| {
        setup_prisoner(s);
        for x in 3..10 {
            let mut hero = s.heroes[0].clone();
            hero.pos = Pos::new(x, 10, 0);
            hero.hero_type = h3_content::HeroTypeId::from_index(usize::from(x) + 1);
            s.heroes.push(hero);
        }
        s.allowed_heroes = (0..rules.core().hero_types().len())
            .filter_map(h3_content::HeroTypeId::from_index)
            .collect();
    });
    let crowd = Case::new(&crowded, &rules).ask(|a| fights::prison(a.asked, prison));
    assert_eq!(crowd, 0);
    // The Pyramid: its 4 imps and 2 gogs in one stack each; with Expert Wisdom and a book the
    // mean gain of its one level-5 spell, the storm.
    let golems = army(&[(IMP, 4), (GOG, 2)]);
    let fight = case.fight(&golems, tile("t:pyramid"));
    assert_eq!(of(&case, "t:pyramid", 0).0, fight);
    let mut wise = Case::new(&state, &rules);
    let sense = rules.core().hero_params().unwrap().wisdom;
    wise.hero.1.skills = vec![(sense, 3)];
    assert_eq!(of(&wise, "t:pyramid", 0).0, fight);
    let book = h3_content::ArtifactTypeId::from_raw(0);
    let at = h3_heroes::first_position(&wise.hero.1, rules.core(), book).unwrap();
    wise.hero.1.worn[at.0] = Some(book.into());
    let storm = h3_content::SpellId::from_raw(2);
    let gain = wise.ask(|a| super::super::spells::gain(&rules, &a.asked.hero.caster, storm));
    assert_eq!(of(&wise, "t:pyramid", 0).0, fight + gain);
    // Looted and known: nothing.
    let mut state = game(&rules, vec![object(&rules, "t:pyramid", AT)], |s| {
        s.heroes[0].army = h3_game_army(&BIG);
    });
    take(&mut state, &rules, AT);
    let case = Case::new(&state, &rules);
    let looted = case.ask(|a| fights::pyramid(a.asked, (0, &kind(&rules, "t:pyramid")), AT));
    assert_eq!(looted, 0);
}

/// The exchange of `army` into `hero`'s with `morale` and `advantage`, on a cleared plan, and the
/// plan's gain after it.
fn exchange(
    case: &Case<'_>,
    (army, other): (&[ArmyStack], &[ArmyStack]),
    (morale, advantage): (i16, i16),
) -> (i32, i32) {
    case.ask(|a| {
        let factions = super::super::armies::alliance_factions(&rules_of(case));
        let mut plan =
            super::super::armies::Plan::new(case.rules, a.asked.hero.weights, Vec::new());
        let v = plan.exchange_value((army, other), (a.asked.join(&factions), morale, advantage));
        (v, plan.gain)
    })
}

/// The case's rules.
fn rules_of(case: &Case<'_>) -> GameRuleset {
    case.rules.clone()
}

// covers: CPU-OBJ-007
#[test]
fn a_garrison_and_a_hero_are_an_exchange_or_a_fight() {
    let rules = rules();
    let gogs = army(&[(GOG, 3)]);
    let with_owner = |owner, removable| {
        shared(&rules, |s| {
            let o = &mut s.map.objects[index("t:garrison")];
            o.owner = owner;
            o.holds = Holds::Garrison {
                army: gogs.clone(),
                removable,
            };
        })
    };
    // Red's, removable: the exchange with its primary sum (4) the advantage.
    let state = with_owner(Some(RED), true);
    let case = Case::new(&state, &rules);
    let (expected, _) = exchange(&case, (&case.hero.0.army.clone(), &gogs), (0, 4));
    assert_eq!(of(&case, "t:garrison", 0).0, expected);
    // Red's kept: nothing; neutral or blue's: the fight.
    let state = with_owner(Some(RED), false);
    assert_eq!(of(&Case::new(&state, &rules), "t:garrison", 0).0, 0);
    for owner in [None, Some(PlayerId::new(1))] {
        let state = with_owner(owner, true);
        let case = Case::new(&state, &rules);
        assert_eq!(
            of(&case, "t:garrison", 0).0,
            case.fight(&gogs, tile("t:garrison"))
        );
    }
    // Blue's hero on (2, 2): the fight, plus within reach the blend (0.5) times its threat and
    // 10 000.
    let state = shared(&rules, |_| {});
    let mut case = Case::new(&state, &rules);
    let blue = state.adventure().hero(HeroId::new(1)).unwrap().clone();
    let blue_details = state.hero(blue.id).unwrap().clone();
    let fight = {
        let (on_map, details) = &case.hero;
        let opponent = Opponent {
            hero: Some(Fighter {
                on_map: &blue,
                details: &blue_details,
            }),
            army: &blue.army,
            town: None,
        };
        fight::fight(
            &state,
            &rules,
            Fighter { on_map, details },
            opponent,
            Some(blue.pos),
        )
    };
    let with = |threat: i32| {
        tenths(5)
            .times(SignedX87::int((threat + 10_000).into()).round(SINGLE_BITS))
            .plus(SignedX87::int(fight.into()).round(SINGLE_BITS))
            .ftol()
    };
    assert_eq!(case.value(blue.pos, 1500).0, with(0));
    assert_eq!(case.value(blue.pos, 1501).0, fight);
    case.record.threats = vec![(blue.id, 2000)];
    assert_eq!(case.value(blue.pos, 0).0, with(2000));
}

// covers: CPU-OBJ-007
#[test]
fn meeting_ones_own_hero_is_the_stronger_ones_exchange() {
    let rules = rules();
    let second = |s: &mut GameSetup| {
        let mut hero = s.heroes[0].clone();
        hero.pos = Pos::new(3, 3, 0);
        hero.hero_type = Some(h3_content::HeroTypeId::from_raw(2));
        hero.army = h3_game_army(&[(GOG, 2)]);
        s.heroes.push(hero);
        s.allowed_heroes = (0..rules.core().hero_types().len())
            .filter_map(h3_content::HeroTypeId::from_index)
            .collect();
    };
    let state = shared(&rules, second);
    let other = state
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == RED && h.id != HERO)
        .unwrap()
        .clone();
    let theirs = other.army.clone();
    // Of equal primary sums: nothing.
    let case = Case::new(&state, &rules);
    assert_eq!(case.value(other.pos, 0).0, 0);
    // Stronger (20 against 4): half the exchange into its army, kept within the day's maximum.
    let mut strong = Case::new(&state, &rules);
    strong.hero.1.primary = [5, 5, 5, 5];
    let mine = strong.hero.0.army.clone();
    let (whole, gain) = exchange(&strong, (&mine, &theirs), (0, 16));
    assert_eq!(strong.value(other.pos, 1499).0, whole / 2);
    let army_value = fight::value_of(&rules, &mine);
    let beyond = if gain >= army_value { whole / 2 } else { 0 };
    assert_eq!(strong.value(other.pos, 1500).0, beyond);
    // Weaker (2 against 4): the whole exchange from the other's side.
    let mut weak = Case::new(&state, &rules);
    weak.hero.1.primary = [0, 0, 1, 1];
    let (whole, _) = exchange(&weak, (&theirs, &weak.hero.0.army.clone()), (0, 2));
    assert_eq!(weak.value(other.pos, 0).0, whole);
    // On Easy: nothing.
    let easy = shared(&rules, |s| {
        second(s);
        s.difficulty = 0;
    });
    let mut strong = Case::new(&easy, &rules);
    strong.hero.1.primary = [5, 5, 5, 5];
    assert_eq!(strong.value(other.pos, 0).0, 0);
}

// covers: CPU-OBJ-008
#[test]
fn the_hill_fort_the_camp_and_a_dwelling() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    // 20 imps upgraded: (100 − 50) × 20 gold at 25 %, 250 of the 2 000; 30 × 20 gained; the gogs
    // have no upgrade.
    let some = with_army(&state, &rules, &[(IMP, 20), (GOG, 100)]);
    assert_eq!(of(&some, "t:hill_fort", 500).0, 600);
    // Beyond 500 it needs a third of the army (1 800 against 4 200).
    assert_eq!(of(&some, "t:hill_fort", 501).0, 0);
    // Two stacks: 250 and 750 paid.
    let two = with_army(&state, &rules, &[(IMP, 20), (IMP, 60)]);
    assert_eq!(of(&two, "t:hill_fort", 0).0, 2400);
    // 200 imps: 2 500 gold, beyond the 2 000.
    let dear = with_army(&state, &rules, &[(IMP, 200)]);
    assert_eq!(of(&dear, "t:hill_fort", 0).0, 0);
    // The camp: its 4 imps' purchase on the army with the player's resources.
    let case = Case::new(&state, &rules);
    let (creature, count) = state.adventure().camp(index("t:camp")).unwrap();
    let expected = case.ask(|a| {
        let asked = a.asked;
        let factions = super::super::armies::alliance_factions(&rules);
        let offer = super::super::armies::Offer {
            creature,
            count: i16::try_from(count).unwrap(),
            free: false,
        };
        let mut plan = super::super::armies::Plan::new(&rules, asked.hero.weights, vec![offer]);
        plan.purchase_value(
            (&asked.fighter().on_map.army, None),
            asked.treasury(),
            (asked.join(&factions), asked.morale()),
        )
    });
    assert_eq!(of(&case, "t:camp", 0).0, expected);
    // The dwelling: T2g's value with the cost.
    let dwelling = case.ask(|a| {
        super::super::towns::dwelling_value(
            a.asked,
            index("t:hut_dwelling"),
            (700, tile("t:hut_dwelling")),
        )
    });
    assert_eq!(of(&case, "t:hut_dwelling", 700).0, dwelling);
}

// covers: CPU-OBJ-012
#[test]
fn the_vision_circle_counts_the_unexplored_tiles_and_their_entrances() {
    let rules = rules();
    let state = game(
        &rules,
        vec![object(&rules, "t:observatory", Pos::new(10, 11, 0))],
        |s| {
            s.map.tiles[143].terrain = TerrainId::from_raw(2);
            s.map.boats.push(h3_adventure::BoatSetup {
                tile: Pos::new(11, 11, 0),
                kind: 0,
            });
            s.heroes[1].pos = Pos::new(11, 10, 0);
        },
    );
    let value = |radius, pos| super::super::vision::value(&state, &rules, RED, radius, pos);
    // Radius 1 in the corner: four tiles, the observatory's 100 and the boat's 100 (the hero's
    // class 34 none).
    assert_eq!(value(1, Pos::new(11, 11, 0)), 204);
    // Radius 2: nine tiles but (9, 9) at 8 > 2² + 2.
    assert_eq!(value(2, Pos::new(11, 11, 0)), 208);
    // Around red's hero everything is explored.
    assert_eq!(value(1, Pos::new(0, 0, 0)), 0);
    // The Observatory's value is its circle of 20.
    let case = Case::new(&state, &rules);
    let circle = value(20, Pos::new(10, 11, 0));
    assert!(circle > 0);
    assert_eq!(case.value(Pos::new(10, 11, 0), 0).0, circle);
    assert_eq!(super::super::vision::SCOUT.len(), 75);
}

// covers: CPU-OBJ-007, CPU-OBJ-008
#[test]
fn a_town_and_a_hero_at_a_towns_entrance() {
    use crate::town::tests::{rules_with_adventure, town_pack, towns_core_pack, towns_setup};
    // The fixtures' creatures worth their AI values in a fight (the imp 10, the gog 40).
    let mut core = towns_core_pack(false);
    for (creature, ai) in core.creatures.iter_mut().zip([10, 40]) {
        creature.info.ai_value = ai;
        creature.info.fight_value = ai;
    }
    let rules = rules_with_adventure(core, &town_pack(false), |_| {});
    let first = GameState::new(&rules, &towns_setup(&rules), 3).unwrap();
    let t0 = h3_contracts::TownId::new(0);
    let t1 = h3_contracts::TownId::new(1);
    let map = first.adventure().map();
    let (gate0, gate1) = (
        map.town_entrance(t0).unwrap(),
        map.town_entrance(t1).unwrap(),
    );
    // Red's own town: T2g's value with the cost's low word.
    let case = Case::new(&first, &rules);
    let own = case.ask(|a| super::super::towns::town_value(a.asked, t0, (-32768, gate0)));
    assert_eq!(case.value(gate0, 0x0001_8000).0, own);
    // Blue's hero at T1's entrance: its fight, and T1's capture with the cost's low word.
    let edited = |no_town: bool| {
        let mut setup = towns_setup(&rules);
        setup.heroes[1].pos = gate1;
        if no_town {
            setup.map.objects[0].owner = None;
            setup.players[0].start = crate::PlayerStart::default();
        }
        GameState::new(&rules, &setup, 3).unwrap()
    };
    let state = edited(false);
    let blue = state.adventure().hero(HeroId::new(1)).unwrap().clone();
    let details = state.hero(blue.id).unwrap().clone();
    let estimate = |case: &Case<'_>| {
        let (on_map, own) = &case.hero;
        let opponent = Opponent {
            hero: Some(Fighter {
                on_map: &blue,
                details: &details,
            }),
            army: &blue.army,
            town: None,
        };
        fight::fight(
            case.state,
            &rules,
            Fighter {
                on_map,
                details: own,
            },
            opponent,
            Some(gate1),
        )
    };
    // A won fight beyond the hero's reach (30 000): the fight and the capture.
    let mut case = Case::new(&state, &rules);
    case.hero.0.army = army(&BIG);
    let fight = estimate(&case);
    assert!(fight > -500_000_000);
    let capture = case.ask(|a| super::super::towns::capture_value(a.asked, t1, (30_000, gate1)));
    assert_eq!(case.value(gate1, 30_000).0, capture.wrapping_add(fight));
    // Lost: alone while red owns a town; else −1 250 000 and the capture.
    let mut none = Case::new(&state, &rules);
    none.hero.0.army = vec![];
    let lost = estimate(&none);
    assert!(lost <= -500_000_000);
    assert_eq!(none.value(gate1, 30_000).0, lost);
    let state = edited(true);
    let mut none = Case::new(&state, &rules);
    none.hero.0.army = vec![];
    let capture = none.ask(|a| super::super::towns::capture_value(a.asked, t1, (30_000, gate1)));
    assert_eq!(
        none.value(gate1, 30_000).0,
        capture.wrapping_add(-1_250_000)
    );
}

// covers: CPU-OBJ-001
#[test]
fn a_class_asked_of_an_object_of_another_kind_finds_nothing_to_read() {
    let rules = rules();
    let state = shared(&rules, |_| {});
    let case = Case::new(&state, &rules);
    let (i, pos) = (index("t:arena"), tile("t:arena"));
    let arena = kind(&rules, "t:arena");
    assert_eq!(case.ask(|a| fights::pandora(a.asked, i, pos)), 0);
    assert_eq!(case.ask(|a| fights::scroll(a.asked, i, pos)), 10);
    let nothing = case.fight(&[], pos);
    let as_bank = case.ask(|a| fights::bank(a.asked, (i, &arena), pos));
    assert_eq!(
        as_bank,
        SignedX87::int(nothing.into()).round(SINGLE_BITS).ftol()
    );
    assert_eq!(
        case.ask(|a| fights::pyramid(a.asked, (i, &arena), pos)),
        nothing
    );
    // Every rate whole: 9 imps upgraded for 450 gold, 30 × 9 gained.
    assert_eq!(case.ask(|a| fights::hill_fort(a.asked, &arena, 0)), 270);
    let ty = ty(&rules, "t:arena");
    let as_class = |class| case.ask(|a| visits::value(a, (i, ty, &arena, class), pos, &mut 0));
    for class in [
        Class::Stables,
        Class::WarMachineFactory,
        Class::BlackMarket,
        Class::Keymaster,
        Class::Shrine,
    ] {
        assert_eq!(as_class(class), 0, "{class:?}");
    }
    assert_eq!(visits::bonus(&arena), (None, 0));
}
