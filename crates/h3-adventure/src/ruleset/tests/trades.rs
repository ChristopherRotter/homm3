//! Increment 14's kinds and tables in the ruleset part (E08 T132a2; increment 14's design note §1
//! to §5): the market, the Altar of Sacrifice, the Hill Fort and the War Machine Factory, every
//! one entered and without a built visit yet; `[params.market]` compiled into the rates
//! (`crate::MarketParams`); the Altar's factions; every check and the hash (part v23; the cost
//! resource v24, E08 T133b).

extern crate std;

use std::boxed::Box;

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, FactionDef, HeroParamsDef, SecondarySkillDef,
};

use super::*;
use crate::Lot;

fn faction(k: &str) -> FactionDef {
    FactionDef {
        key: key(k),
        name: "Faction".into(),
        native_terrain: None,
    }
}

fn artifact(k: &str, class: ArtifactClass, machine: Option<&str>) -> ArtifactDef {
    ArtifactDef {
        key: key(k),
        name: "Artifact".into(),
        class,
        cost: vec![],
        slot: Some(key("t:hand")),
        war_machine: machine.map(key),
        needs_skill: None,
        effects: vec![],
        parts: vec![],
    }
}

/// [`core`]'s packs, the gem `t:gem` in `t` (the market's parameters value every core resource,
/// so they must reach each), with the factions `t:castle`, `t:keep` and `u:fen`, a slot `t:hand`, the
/// artifacts `t:book` (special), `t:sword` (treasure), `t:cart_art` (the war machine `t:cart`'s),
/// two of the war machine `t:ram` (`t:ram_a`, `t:ram_b`) and the war machine `t:tent` without
/// one.
fn trade_core() -> Ruleset {
    trade_core_with(|_| {})
}

/// [`trade_core`] with its pack `t` edited by `edit`.
fn trade_core_with(edit: impl FnOnce(&mut ContentPack)) -> Ruleset {
    let mut t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        resources: vec![resource("t:wood"), resource("t:gold"), resource("t:gem")],
        factions: vec![faction("t:castle"), faction("t:keep")],
        creatures: vec![
            creature("t:imp", 1, false),
            creature("t:gog", 2, false),
            creature("t:cart", 1, true),
            creature("t:ram", 1, true),
            creature("t:tent", 1, true),
        ],
        artifact_slots: vec![ArtifactSlotDef {
            key: key("t:hand"),
            name: "Hand".into(),
            count: 1,
            backpack: true,
        }],
        artifacts: vec![
            artifact("t:book", ArtifactClass::Special, None),
            artifact("t:sword", ArtifactClass::Treasure, None),
            artifact("t:cart_art", ArtifactClass::Special, Some("t:cart")),
            artifact("t:ram_a", ArtifactClass::Special, Some("t:ram")),
            artifact("t:ram_b", ArtifactClass::Special, Some("t:ram")),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    edit(&mut t);
    let u = ContentPack {
        terrains: vec![terrain("u:swamp")],
        factions: vec![faction("u:fen")],
        creatures: vec![creature("u:wisp", 1, false)],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let v = ContentPack::new(manifest("v", &[]));
    Ruleset::compile(&[t, u, v], "t:faithful").unwrap()
}

/// `[params.market]` for [`trade_core`]: every resource valued, two rows.
fn market_params() -> MarketParamsDef {
    MarketParamsDef {
        cost: key("t:gold"),
        values: vec![
            (key("t:wood"), 250),
            (key("t:gold"), 1),
            (key("t:gem"), 500),
        ],
        artifact_values: vec![
            (key("t:gem"), 250),
            (key("t:gold"), 1),
            (key("t:wood"), 125),
        ],
        resources: vec![10, 30],
        artifacts: vec![20, 40],
        creatures: vec![30, 70],
    }
}

fn trading_post() -> Market {
    Market {
        screens: vec![MarketScreen::Trade, MarketScreen::Transfer],
        count: 2,
        stock: vec![],
    }
}

fn altar() -> AltarDef {
    AltarDef {
        experience: [1000, 1500, 3000, 6000, 0],
        keeps: vec![key("t:book")],
        places: 23,
        creature_factor: 5,
        creature_divisor: 40,
    }
}

fn factory() -> WarMachineFactoryDef {
    WarMachineFactoryDef {
        machines: vec![key("t:cart")],
    }
}

fn entry(k: &str, artifacts: bool, creatures: bool) -> AltarFactionDef {
    AltarFactionDef {
        faction: key(k),
        artifacts,
        creatures,
    }
}

/// [`packs`] with the market parameters in `t`, the four kinds as `t:post`, `t:altar`, `t:fort`
/// and `t:factory`, and the three factions' Altar entries (`u:fen`'s in `u`); `u`'s gem pile
/// left out.
fn trade_packs() -> Vec<AdventurePack> {
    edit(|p| {
        pack(p, "u").object_types.clear();
        let t = pack(p, "t");
        t.params.as_mut().unwrap().market = Some(market_params());
        t.object_types.extend([
            object(
                "t:post",
                Visit::Enter,
                false,
                ObjectKindDef::Market(trading_post()),
            ),
            object(
                "t:altar",
                Visit::Enter,
                false,
                ObjectKindDef::Altar(altar()),
            ),
            object(
                "t:fort",
                Visit::Enter,
                false,
                ObjectKindDef::HillFort(HillFort {
                    gold: [0, 25, 50, 75, 100, 100, 100],
                }),
            ),
            object(
                "t:factory",
                Visit::Enter,
                false,
                ObjectKindDef::WarMachineFactory(factory()),
            ),
        ]);
        t.altar_factions = vec![entry("t:castle", true, false), entry("t:keep", true, true)];
        pack(p, "u").altar_factions = vec![entry("u:fen", false, true)];
    })
}

fn compile_trades(packs: &[AdventurePack]) -> Result<AdventureRuleset, AdventureCompileError> {
    AdventureRuleset::compile(&trade_core(), packs)
}

/// The compile error of [`trade_packs`] edited by `f`.
fn trade_err(f: impl FnOnce(&mut Vec<AdventurePack>)) -> AdventureCompileError {
    let mut packs = trade_packs();
    f(&mut packs);
    compile_trades(&packs).unwrap_err()
}

/// The kind of `t:<name>` in [`trade_packs`] edited.
fn set(packs: &mut [AdventurePack], name: &str, kind: ObjectKindDef) {
    let ty = pack(packs, "t")
        .object_types
        .iter_mut()
        .find(|t| t.key.as_str() == format!("t:{name}"))
        .unwrap();
    ty.kind = kind;
}

/// The reason `t:<name>` with `kind` is refused for.
fn refused(name: &str, kind: ObjectKindDef) -> &'static str {
    match trade_err(|p| set(p, name, kind)) {
        AdventureCompileError::InvalidObjectType { key, reason } => {
            assert_eq!(key, format!("t:{name}"));
            reason
        }
        other => panic!("{other:?}"),
    }
}

// covers: ADV-MKT-001, ADV-MKT-002, ADV-ALT-001, ADV-ALT-002, ADV-ALT-003, ADV-HFT-002, ADV-WMF-001
#[test]
fn the_four_kinds_compile_entered_and_visited() {
    let core = trade_core();
    let part = AdventureRuleset::compile(&core, &trade_packs()).unwrap();
    let kind = |k: &str| {
        part.object_type(part.object_type_by_key(k).unwrap())
            .unwrap()
            .kind
            .clone()
    };
    let ObjectKind::Market(post) = kind("t:post") else {
        panic!("not a market");
    };
    assert_eq!(post, trading_post());
    assert!(post.shows(MarketScreen::Transfer) && !post.shows(MarketScreen::Buy));
    let ObjectKind::Altar(altar) = kind("t:altar") else {
        panic!("not an altar");
    };
    assert_eq!(altar.keeps, [core.artifact_by_key("t:book").unwrap()]);
    assert_eq!(
        (altar.places, altar.creature_factor, altar.creature_divisor),
        (23, 5, n(40))
    );
    assert_eq!(altar.artifact_experience(ArtifactClass::Treasure), 1000);
    assert_eq!(altar.artifact_experience(ArtifactClass::Minor), 1500);
    assert_eq!(altar.artifact_experience(ArtifactClass::Major), 3000);
    assert_eq!(altar.artifact_experience(ArtifactClass::Relic), 6000);
    assert_eq!(altar.artifact_experience(ArtifactClass::Special), 0);
    assert_eq!(
        kind("t:fort"),
        ObjectKind::HillFort(HillFort {
            gold: [0, 25, 50, 75, 100, 100, 100]
        })
    );
    assert_eq!(
        kind("t:factory"),
        ObjectKind::WarMachineFactory(WarMachineFactory {
            machines: vec![(
                core.creature_by_key("t:cart").unwrap(),
                core.artifact_by_key("t:cart_art").unwrap()
            )],
        })
    );
    for k in ["t:post", "t:altar", "t:fort", "t:factory"] {
        let kind = kind(k);
        // The markets' visit since E08 T133a2, the Altar's since T134a, the Hill Fort's since
        // T134b1, the factory's since T134b2; neither ownable nor weekly.
        assert!(kind.effect().is_some(), "{k}");
        assert!(!kind.ownable() && !kind.weekly());
        assert_eq!(kind.timed_mark(), None);
    }
    // The rates in the core's resource order, whatever the tables' order.
    let rates = part.params().market().unwrap();
    let (wood, gold, gem) = (
        ResourceId::from_raw(0),
        ResourceId::from_raw(1),
        ResourceId::from_raw(2),
    );
    assert_eq!(rates.counts(), 2);
    assert_eq!(rates.cost(), gold);
    assert_eq!(
        rates.resource_lot(2, wood, gold),
        Some(Lot { give: 1, take: 75 })
    );
    assert_eq!(
        rates.resource_lot(1, gem, wood),
        Some(Lot { give: 5, take: 1 })
    );
    assert_eq!(rates.artifact_price(2, 2000, gold), Some(5000));
    // The factions' screens; none for a faction no pack names.
    let screens = |k: &str| part.altar_screens(core.faction_by_key(k).unwrap());
    let both = AltarScreens {
        artifacts: true,
        creatures: true,
    };
    assert_eq!(
        screens("t:castle"),
        Some(AltarScreens {
            artifacts: true,
            creatures: false
        })
    );
    assert_eq!(screens("t:keep"), Some(both));
    assert_eq!(
        screens("u:fen"),
        Some(AltarScreens {
            artifacts: false,
            creatures: true
        })
    );
    assert_eq!(part.altar_screens(FactionId::from_raw(3)), None);
}

// covers: ADV-MKT-001, ADV-ALT-002, ADV-ALT-003, ADV-HFT-002, ADV-WMF-001
#[test]
fn the_kinds_parameters_are_checked() {
    let market = |screens: &[MarketScreen], count, stock: &[ArtifactClass]| {
        ObjectKindDef::Market(Market {
            screens: screens.to_vec(),
            count,
            stock: stock.to_vec(),
        })
    };
    use ArtifactClass::{Special, Treasure};
    use MarketScreen::{Buy, Sell, Trade};
    let screens = "a market's screens are none or name one twice";
    assert_eq!(refused("post", market(&[], 1, &[])), screens);
    assert_eq!(refused("post", market(&[Trade, Trade], 1, &[])), screens);
    assert_eq!(
        refused("post", market(&[Trade], 0, &[])),
        "a market's count is 0"
    );
    let stock = "a market's stock is not exactly a buying one's";
    assert_eq!(refused("post", market(&[Sell], 1, &[Treasure])), stock);
    assert_eq!(refused("post", market(&[Buy], 1, &[])), stock);
    assert_eq!(
        refused("post", market(&[Buy], 1, &[Treasure, Special])),
        "a market's stock names the special class"
    );
    // E08 T133a1: a bit per entry in the sold entries.
    assert_eq!(
        refused("post", market(&[Buy], 1, &[Treasure; 9])),
        "a market's stock holds more than 8 entries"
    );
    assert_eq!(
        refused("post", market(&[Buy], 3, &[Treasure])),
        "a market's count is past [params.market]'s rows"
    );
    match trade_err(|p| pack(p, "t").params.as_mut().unwrap().market = None) {
        AdventureCompileError::InvalidObjectType { key, reason } => {
            assert_eq!(
                (key.as_str(), reason),
                ("t:post", "a market without [params.market]")
            );
        }
        other => panic!("{other:?}"),
    }
    // E08 T134b1: a Hill Fort's gold is the market parameters' cost resource.
    let no_market = |p: &mut Vec<AdventurePack>| {
        let t = pack(p, "t");
        t.params.as_mut().unwrap().market = None;
        t.object_types.retain(|ty| ty.key.as_str() != "t:post");
    };
    match trade_err(no_market) {
        AdventureCompileError::InvalidObjectType { key, reason } => {
            assert_eq!(
                (key.as_str(), reason),
                ("t:fort", "a hill fort without [params.market] (its gold)")
            );
        }
        other => panic!("{other:?}"),
    }
    let altar = |f: fn(&mut AltarDef)| {
        let mut def = altar();
        f(&mut def);
        ObjectKindDef::Altar(def)
    };
    assert_eq!(
        refused("altar", altar(|a| a.keeps.push(key("t:book")))),
        "an altar keeps an artifact twice"
    );
    assert_eq!(
        refused("altar", altar(|a| a.places = 0)),
        "an altar takes no artifact"
    );
    assert_eq!(
        refused("altar", altar(|a| a.creature_divisor = 0)),
        "an altar's creature divisor is 0"
    );
    assert_eq!(
        trade_err(|p| set(p, "altar", altar(|a| a.keeps = vec![key("t:shield")]))),
        AdventureCompileError::UnknownArtifact("t:shield".into())
    );
    assert_eq!(
        trade_err(|p| set(p, "altar", altar(|a| a.keeps = vec![key("u:book")]))),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:book".into(),
            pack: "t".into(),
        }
    );
    let percents = "a Hill Fort's gold percent is above 100 or not a multiple of 25";
    for gold in [
        [0, 25, 50, 75, 100, 100, 125],
        [0, 20, 50, 75, 100, 100, 100],
    ] {
        assert_eq!(
            refused("fort", ObjectKindDef::HillFort(HillFort { gold })),
            percents
        );
    }
    let machines = |list: &[&str]| {
        ObjectKindDef::WarMachineFactory(WarMachineFactoryDef {
            machines: list.iter().map(|k| key(k)).collect(),
        })
    };
    let none = "a factory sells no machine or one twice";
    assert_eq!(refused("factory", machines(&[])), none);
    assert_eq!(refused("factory", machines(&["t:cart", "t:cart"])), none);
    let one = "a factory's machine is not a war machine with one artifact";
    for list in [["t:imp"], ["t:ram"], ["t:tent"]] {
        assert_eq!(refused("factory", machines(&list)), one, "{list:?}");
    }
    assert_eq!(
        trade_err(|p| set(p, "factory", machines(&["t:catapult"]))),
        AdventureCompileError::UnknownCreature("t:catapult".into())
    );
    // Each is entered.
    for name in ["post", "altar", "fort", "factory"] {
        let error = trade_err(|p| {
            let ty = pack(p, "t")
                .object_types
                .iter_mut()
                .find(|t| t.key.as_str() == format!("t:{name}"))
                .unwrap();
            ty.visit = Visit::Neighbour;
        });
        assert_eq!(
            error,
            AdventureCompileError::InvalidObjectType {
                key: format!("t:{name}"),
                reason: "a trade or service is not entered",
            }
        );
    }
}

// covers: ADV-MKT-002
#[test]
fn the_market_parameters_are_checked() {
    let invalid = |f: fn(&mut MarketParamsDef)| {
        trade_err(|p| {
            let params = pack(p, "t").params.as_mut().unwrap();
            f(params.market.as_mut().unwrap());
        })
    };
    let reason = |f: fn(&mut MarketParamsDef)| match invalid(f) {
        AdventureCompileError::InvalidParams(reason) => reason,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        reason(|m| m.values.push((key("t:wood"), 1))),
        "a market value table names a resource twice"
    );
    assert_eq!(
        reason(|m| {
            m.artifact_values.pop();
        }),
        "a market value table misses a core resource"
    );
    assert_eq!(
        reason(|m| {
            m.creatures.pop();
        }),
        "the market's factor lists differ in length"
    );
    assert_eq!(
        reason(|m| {
            m.artifacts.pop();
        }),
        "the market's factor lists differ in length"
    );
    assert_eq!(reason(|m| m.values[0].1 = 0), "a market value is 0");
    assert_eq!(reason(|m| m.resources[1] = 0), "a market factor is 0");
    assert_eq!(
        reason(|m| {
            m.resources.clear();
            m.artifacts.clear();
            m.creatures.clear();
        }),
        "the market has no resource or row, or more than 255 rows"
    );
    assert_eq!(
        invalid(|m| m.values[0].0 = key("t:ore")),
        AdventureCompileError::UnknownResource("t:ore".into())
    );
    // E08 T133b (ADV-Q60 (a)): the cost resource is a core resource.
    assert_eq!(
        invalid(|m| m.cost = key("t:ore")),
        AdventureCompileError::UnknownResource("t:ore".into())
    );
    // `t` does not depend on `u`: a gem valued in `t`'s parameters is out of reach.
    let outside = trade_err(|p| {
        let t = pack(p, "t");
        let mut market = market_params();
        market.values.push((key("v:pearl"), 1));
        t.params.as_mut().unwrap().market = Some(market);
    });
    assert_eq!(
        outside,
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "v:pearl".into(),
            pack: "t".into(),
        }
    );
}

// covers: ADV-ALT-002, ADV-ALT-003
#[test]
fn an_altar_refuses_a_backpack_or_an_army_its_answer_cannot_name() {
    // A backpack of 64 at most (the answer's set of places, decision 386).
    let with_backpack = |backpack| {
        trade_core_with(|t| {
            t.skills = vec![SecondarySkillDef {
                key: key("t:wisdom"),
                name: "Wisdom".into(),
                levels: [vec![], vec![], vec![]],
            }];
            t.hero_params = Some(HeroParamsDef {
                primary_min: [0, 0, 1, 1],
                primary_max: 99,
                backpack,
                learn_level: 2,
                experience: vec![0, 1000],
                max_level: 74,
                wisdom: key("t:wisdom"),
                schools: vec![],
                mana_ceiling: 999,
                base_mana_regain: 1,
                regain_specialist_bonus: 1,
            });
        })
    };
    let reason =
        |core: &Ruleset, packs: &[AdventurePack]| match AdventureRuleset::compile(core, packs) {
            Err(AdventureCompileError::InvalidObjectType { key, reason }) => (key, reason),
            other => panic!("{other:?}"),
        };
    assert!(AdventureRuleset::compile(&with_backpack(64), &trade_packs()).is_ok());
    assert_eq!(
        reason(&with_backpack(65), &trade_packs()),
        (
            "t:altar".into(),
            "an altar under a backpack of more than 64"
        )
    );
    // Seven army slots at most (the answer's amounts by slot).
    let mut packs = trade_packs();
    pack(&mut packs, "t").params.as_mut().unwrap().army_slots = NonZeroU8::new(8).unwrap();
    assert_eq!(
        reason(&trade_core(), &packs),
        ("t:altar".into(), "an altar under more than 7 army slots")
    );
    // Without an Altar neither bounds anything.
    pack(&mut packs, "t")
        .object_types
        .retain(|t| t.key.as_str() != "t:altar");
    assert!(compile_trades(&packs).is_ok());
}

// covers: ADV-ALT-001
#[test]
fn every_faction_has_its_altar_entry_beside_an_altar() {
    let faction_error = |f: fn(&mut Vec<AdventurePack>)| match trade_err(f) {
        AdventureCompileError::AltarFaction { faction, reason } => (faction, reason),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        faction_error(|p| pack(p, "u").altar_factions.clear()),
        ("u:fen".into(), "it has no entry, and the part has an altar")
    );
    assert_eq!(
        faction_error(|p| pack(p, "u").altar_factions[0].creatures = false),
        ("u:fen".into(), "its heroes sacrifice nothing")
    );
    assert_eq!(
        faction_error(|p| pack(p, "u")
            .altar_factions
            .push(entry("t:castle", true, true))),
        ("t:castle".into(), "it has a second entry")
    );
    assert_eq!(
        trade_err(|p| pack(p, "u").altar_factions[0].faction = key("u:bog")),
        AdventureCompileError::UnknownFaction("u:bog".into())
    );
    assert_eq!(
        trade_err(|p| pack(p, "t").altar_factions[0].faction = key("u:fen")),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:fen".into(),
            pack: "t".into(),
        }
    );
    // Without an Altar a faction may go without.
    let mut packs = trade_packs();
    pack(&mut packs, "u").altar_factions.clear();
    pack(&mut packs, "t")
        .object_types
        .retain(|t| t.key.as_str() != "t:altar");
    assert!(compile_trades(&packs).is_ok());
    // The errors explain themselves.
    assert_eq!(
        AdventureCompileError::UnknownArtifact("t:x".into()).to_string(),
        "unknown artifact `t:x`"
    );
    assert_eq!(
        AdventureCompileError::AltarFaction {
            faction: "t:castle".into(),
            reason: "why",
        }
        .to_string(),
        "altar entry of `t:castle`: why"
    );
}

/// `t`'s market parameters.
fn market_of(p: &mut [AdventurePack]) -> Option<MarketParamsDef> {
    pack(p, "t").params.as_ref().unwrap().market.clone()
}

/// An edit of [`trade_packs`].
type Edit = dyn Fn(&mut Vec<AdventurePack>);

// covers: ADV-MKT-001, ADV-MKT-002, ADV-ALT-001, ADV-ALT-002, ADV-HFT-002, ADV-WMF-001
#[test]
fn every_trade_parameter_is_in_the_hash() {
    let hash = |f: &dyn Fn(&mut Vec<AdventurePack>)| {
        let mut packs = trade_packs();
        f(&mut packs);
        compile_trades(&packs).unwrap().content_hash()
    };
    let edits: Vec<Box<Edit>> = vec![
        Box::new(|_| {}),
        Box::new(|p| {
            let mut post = trading_post();
            post.screens = vec![MarketScreen::Sell, MarketScreen::Transfer];
            set(p, "post", ObjectKindDef::Market(post));
        }),
        Box::new(|p| {
            let mut post = trading_post();
            post.count = 1;
            set(p, "post", ObjectKindDef::Market(post));
        }),
        Box::new(|p| {
            let post = Market {
                screens: vec![MarketScreen::Buy],
                count: 2,
                stock: vec![ArtifactClass::Major],
            };
            set(p, "post", ObjectKindDef::Market(post));
        }),
        Box::new(|p| {
            let mut a = altar();
            a.experience[4] = 1;
            set(p, "altar", ObjectKindDef::Altar(a));
        }),
        Box::new(|p| {
            let mut a = altar();
            a.keeps = vec![key("t:sword")];
            set(p, "altar", ObjectKindDef::Altar(a));
        }),
        Box::new(|p| {
            let mut a = altar();
            a.places = 22;
            set(p, "altar", ObjectKindDef::Altar(a));
        }),
        Box::new(|p| {
            let mut a = altar();
            a.creature_factor = 4;
            set(p, "altar", ObjectKindDef::Altar(a));
        }),
        Box::new(|p| {
            let mut a = altar();
            a.creature_divisor = 41;
            set(p, "altar", ObjectKindDef::Altar(a));
        }),
        Box::new(|p| {
            let gold = [0, 25, 50, 75, 100, 100, 75];
            set(p, "fort", ObjectKindDef::HillFort(HillFort { gold }));
        }),
        Box::new(|p| {
            pack(p, "t")
                .object_types
                .retain(|t| t.key.as_str() != "t:factory");
        }),
        Box::new(|p| {
            let mut m = market_of(p).unwrap();
            m.cost = key("t:wood");
            pack(p, "t").params.as_mut().unwrap().market = Some(m);
        }),
        Box::new(|p| {
            let mut m = market_of(p).unwrap();
            m.values[0].1 = 251;
            pack(p, "t").params.as_mut().unwrap().market = Some(m);
        }),
        Box::new(|p| {
            let mut m = market_of(p).unwrap();
            m.artifact_values[0].1 = 251;
            pack(p, "t").params.as_mut().unwrap().market = Some(m);
        }),
        Box::new(|p| {
            let mut m = market_of(p).unwrap();
            m.creatures[1] = 71;
            pack(p, "t").params.as_mut().unwrap().market = Some(m);
        }),
        Box::new(|p| {
            // Without the parameters neither the market nor the Hill Fort (its gold, E08
            // T134b1).
            pack(p, "t").params.as_mut().unwrap().market = None;
            pack(p, "t")
                .object_types
                .retain(|t| !["t:post", "t:fort"].contains(&t.key.as_str()));
        }),
        Box::new(|p| pack(p, "t").altar_factions[0].creatures = true),
        Box::new(|p| pack(p, "t").altar_factions[1].artifacts = false),
    ];
    let hashes = edits.iter().map(|f| hash(f.as_ref())).collect::<Vec<_>>();
    assert_eq!(
        hashes.iter().collect::<BTreeSet<_>>().len(),
        hashes.len(),
        "{hashes:x?}"
    );
}
