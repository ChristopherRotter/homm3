//! E08 T132a2: increment 14's trades and services in the pack format and the original packs
//! (increment 14's design note §1 to §6; ADV-MKT-001, -002, ADV-ALT-001 to -003, ADV-HFT-001,
//! -002, ADV-WMF-001).

use h3_adventure::{
    Altar, AltarDef, AltarFactionDef, AltarScreens, HillFort, Lot, Market, MarketParamsDef,
    MarketScreen, WarMachineFactory, WarMachineFactoryDef,
};

use super::*;

/// One object type with this kind table, entered.
fn kind(table: &str) -> ObjectKindDef {
    let text = format!(
        "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"enter\"\nfrom_above = false\n{table}"
    );
    parse(&text).unwrap().object_types.remove(0).kind
}

// covers: ADV-MKT-001, ADV-ALT-002, ADV-HFT-002, ADV-WMF-001
#[test]
fn parses_the_trade_kinds() {
    let k = key_of;
    assert_eq!(
        kind("market = { screens = [\"trade\", \"transfer\"], count = 5 }"),
        ObjectKindDef::Market(Market {
            screens: vec![MarketScreen::Trade, MarketScreen::Transfer],
            count: 5,
            stock: vec![],
        })
    );
    assert_eq!(
        kind(
            "market = { screens = [\"sell\", \"buy\"], count = 1, stock = [\"relic\", \"special\"] }"
        ),
        ObjectKindDef::Market(Market {
            screens: vec![MarketScreen::Sell, MarketScreen::Buy],
            count: 1,
            stock: vec![ArtifactClass::Relic, ArtifactClass::Special],
        })
    );
    // The experience by class, 0 for a class left out; the kept artifacts in the pack's
    // namespace or as written.
    assert_eq!(
        kind(
            "[object_type.altar]\nexperience = { treasure = 1, minor = 2, major = 3, relic = 4, \
             special = 5 }\nkeeps = [\"book\", \"other:cat\"]\nplaces = 23\ncreature_factor = 5\n\
             creature_divisor = 40"
        ),
        ObjectKindDef::Altar(AltarDef {
            experience: [1, 2, 3, 4, 5],
            keeps: vec![k("p:book"), k("other:cat")],
            places: 23,
            creature_factor: 5,
            creature_divisor: 40,
        })
    );
    assert_eq!(
        kind(
            "[object_type.altar]\nexperience = { minor = 7 }\nplaces = 1\ncreature_factor = 0\n\
             creature_divisor = 0"
        ),
        ObjectKindDef::Altar(AltarDef {
            experience: [0, 7, 0, 0, 0],
            keeps: vec![],
            places: 1,
            creature_factor: 0,
            creature_divisor: 0,
        })
    );
    assert_eq!(
        kind("hill_fort = { gold = [0, 25, 50, 75, 100, 100, 100] }"),
        ObjectKindDef::HillFort(HillFort {
            gold: [0, 25, 50, 75, 100, 100, 100],
        })
    );
    assert_eq!(
        kind("war_machine_factory = { machines = [\"ballista\", \"other:cart\"] }"),
        ObjectKindDef::WarMachineFactory(WarMachineFactoryDef {
            machines: vec![k("p:ballista"), k("other:cart")],
        })
    );
    // Malformed tables and keys.
    for table in [
        "market = { screens = [\"steal\"], count = 5 }",
        "market = { screens = [], count = 256 }",
        "hill_fort = { gold = [0, 25] }",
        "war_machine_factory = {}",
        "[object_type.altar]\nexperience = { gold = 1 }\nplaces = 1\ncreature_factor = 5\n\
         creature_divisor = 40",
    ] {
        assert!(
            matches!(kind_error(table), AdventurePackError::Syntax(_)),
            "{table}"
        );
    }
    for table in [
        "[object_type.altar]\nexperience = {}\nkeeps = [\"Bad\"]\nplaces = 1\n\
         creature_factor = 5\ncreature_divisor = 40",
        "war_machine_factory = { machines = [\"Bad\"] }",
    ] {
        assert!(
            matches!(kind_error(table), AdventurePackError::Key(_)),
            "{table}"
        );
    }
}

// covers: ADV-MKT-002, ADV-ALT-001
#[test]
fn parses_the_market_parameters_and_the_altar_factions() {
    let k = key_of;
    let params = |table: &str| {
        let text = format!(
            "[params]\nnative_cost = 100\nland_points = [1]\nscouting_radius = 1\n\
             max_map_size = 1\nmax_levels = 1\nmax_players = 1\narmy_slots = 1\n\
             max_stack_count = 4095\nstarting_stack_chances = []\n\
             fly_cost = [1, 1, 1, 1]\nwater_walk_cost = [1, 1, 1, 1]\n\
             sea_points = [1, 1, 1, 1]\nsea_ground = [1, 1]\nflag_radius = 1\n\
             [params.calendar]\nweek_days = 1\nmonth_weeks = 1\n{table}"
        );
        parse(&text).map(|pack| pack.params.unwrap().market)
    };
    assert_eq!(params(""), Ok(None));
    assert_eq!(
        params(
            "[params.market]\nvalues = { wood = 250, \"other:gold\" = 1 }\nartifact_values = { wood = \
             125 }\nresources = [10, 15]\nartifacts = [20]\ncreatures = []"
        ),
        Ok(Some(MarketParamsDef {
            values: vec![(k("other:gold"), 1), (k("p:wood"), 250)],
            artifact_values: vec![(k("p:wood"), 125)],
            resources: vec![10, 15],
            artifacts: vec![20],
            creatures: vec![],
        }))
    );
    for (table, key_error) in [
        ("values = { Bad = 1 }\nartifact_values = {}", true),
        ("values = {}\nartifact_values = { Bad = 1 }", true),
        ("values = { wood = 65536 }\nartifact_values = {}", false),
        ("values = {}", false),
    ] {
        let error = params(&format!(
            "[params.market]\n{table}\nresources = []\nartifacts = []\ncreatures = []"
        ))
        .unwrap_err();
        assert_eq!(
            matches!(error, AdventurePackError::Key(_)),
            key_error,
            "{table}: {error:?}"
        );
        assert!(key_error || matches!(error, AdventurePackError::Syntax(_)));
    }
    // The factions' entries: artifacts, creatures or both, false if left out.
    let factions = |text: &str| parse(text).map(|pack| pack.altar_factions);
    assert_eq!(
        factions(
            "[[altar_faction]]\nfaction = \"castle\"\nartifacts = true\n\n\
             [[altar_faction]]\nfaction = \"other:fen\"\ncreatures = true\n\n\
             [[altar_faction]]\nfaction = \"keep\""
        ),
        Ok(vec![
            AltarFactionDef {
                faction: k("p:castle"),
                artifacts: true,
                creatures: false,
            },
            AltarFactionDef {
                faction: k("other:fen"),
                artifacts: false,
                creatures: true,
            },
            AltarFactionDef {
                faction: k("p:keep"),
                artifacts: false,
                creatures: false,
            },
        ])
    );
    assert!(matches!(
        factions("[[altar_faction]]\nfaction = \"Bad\""),
        Err(AdventurePackError::Key(_))
    ));
    assert!(matches!(
        factions("[[altar_faction]]\nfaction = \"castle\"\nboth = true"),
        Err(AdventurePackError::Syntax(_))
    ));
}

// covers: ADV-MKT-001, ADV-MKT-002, ADV-ALT-001, ADV-ALT-002, ADV-ALT-003, ADV-HFT-002, ADV-WMF-001
#[test]
fn the_packs_type_the_trades_and_services_of_increment_14() {
    let (base_core, base) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let (core, sod) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    let ty = |part: &AdventureRuleset, key: &str| {
        part.object_type(part.object_type_by_key(key).unwrap())
            .unwrap()
            .clone()
    };
    // Every one entered, none from above; the Altar and the Hill Fort mark the team's kind.
    for (key, mark) in [
        ("base:trading_post", false),
        ("base:black_market", false),
        ("base:altar_of_sacrifice", true),
        ("base:hill_fort", true),
        ("base:war_machine_factory", false),
        ("ab:freelancers_guild", false),
    ] {
        let t = ty(&sod, key);
        assert_eq!(
            (t.visit, t.from_above, t.kind_mark),
            (Visit::Enter, false, mark),
            "{key}"
        );
    }
    assert!(base.object_type_by_key("ab:freelancers_guild").is_none());
    let market = |key: &str| match ty(&sod, key).kind {
        ObjectKind::Market(market) => market,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        market("base:trading_post"),
        Market {
            screens: vec![MarketScreen::Trade, MarketScreen::Transfer],
            count: 5,
            stock: vec![],
        }
    );
    use ArtifactClass::{Major, Minor, Treasure};
    assert_eq!(
        market("base:black_market"),
        Market {
            screens: vec![MarketScreen::Buy],
            count: 5,
            stock: vec![Treasure, Treasure, Treasure, Minor, Minor, Minor, Major],
        }
    );
    assert_eq!(market("ab:freelancers_guild").screens, [MarketScreen::Sell]);
    let artifact = |key: &str| core.artifact_by_key(key).unwrap();
    let creature = |key: &str| core.creature_by_key(key).unwrap();
    assert_eq!(
        ty(&sod, "base:altar_of_sacrifice").kind,
        ObjectKind::Altar(Altar {
            experience: [1000, 1500, 3000, 6000, 0],
            keeps: vec![artifact("base:spell_book"), artifact("base:catapult")],
            places: 23,
            creature_factor: 5,
            creature_divisor: NonZeroU32::new(40).unwrap(),
        })
    );
    assert_eq!(
        ty(&sod, "base:hill_fort").kind,
        ObjectKind::HillFort(HillFort {
            gold: [0, 25, 50, 75, 100, 100, 100],
        })
    );
    assert_eq!(
        ty(&sod, "base:war_machine_factory").kind,
        ObjectKind::WarMachineFactory(WarMachineFactory {
            machines: vec![
                (creature("base:ballista"), artifact("base:ballista")),
                (
                    creature("base:first_aid_tent"),
                    artifact("base:first_aid_tent")
                ),
                (creature("base:ammo_cart"), artifact("base:ammo_cart")),
            ],
        })
    );
    // The factions: base's eight, ab's Conflux; good heroes artifacts, evil creatures, neutral
    // both (batch E14's E14-7).
    let screens = |core: &Ruleset, part: &AdventureRuleset, key: &str| {
        part.altar_screens(core.faction_by_key(key).unwrap())
            .map(|s| (s.artifacts, s.creatures))
    };
    for (key, artifacts, creatures) in [
        ("base:castle", true, false),
        ("base:rampart", true, false),
        ("base:tower", true, false),
        ("base:inferno", false, true),
        ("base:necropolis", false, true),
        ("base:dungeon", false, true),
        ("base:stronghold", true, true),
        ("base:fortress", true, true),
        ("ab:conflux", true, true),
    ] {
        assert_eq!(
            screens(&core, &sod, key),
            Some((artifacts, creatures)),
            "{key}"
        );
    }
    assert_eq!(
        screens(&base_core, &base, "base:castle"),
        Some((true, false))
    );
    assert_eq!(base.altar_screens(h3_content::FactionId::from_raw(8)), None);
    // The rates at five Marketplaces (ADV-MKT-002; batch E14's E14-1, E14-5): a wood for 75 gold,
    // 833 gold for a wood, 3 mercury for a gem, a Pixie (25 gold) for 17 gold, an artifact of
    // 2 000 for 5 000 gold.
    let rates = sod.params().market().unwrap();
    let resource = |key: &str| {
        let index = core.resources().iter().position(|r| r.key.as_str() == key);
        ResourceId::from_index(index.unwrap()).unwrap()
    };
    let (wood, gold) = (resource("base:wood"), resource("base:gold"));
    assert_eq!(rates.counts(), 10);
    assert_eq!(
        rates.resource_lot(5, wood, gold),
        Some(Lot { give: 1, take: 75 })
    );
    assert_eq!(
        rates.resource_lot(5, gold, wood),
        Some(Lot { give: 833, take: 1 })
    );
    assert_eq!(
        rates.resource_lot(5, resource("base:mercury"), resource("base:gems")),
        Some(Lot { give: 3, take: 1 })
    );
    assert_eq!(
        rates.creature_lot(5, 25, gold),
        Some(Lot { give: 1, take: 17 })
    );
    assert_eq!(rates.artifact_price(5, 2000, gold), Some(5000));
    assert_eq!(
        rates.factor_bits(5),
        Some([0x3E99_999A, 0x3ECC_CCCD, 0x3F33_3333])
    );
    assert_eq!(
        sod.altar_screens(core.faction_by_key("ab:conflux").unwrap()),
        Some(AltarScreens {
            artifacts: true,
            creatures: true,
        })
    );
}
