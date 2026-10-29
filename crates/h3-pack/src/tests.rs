//! The core files' own tests. The detailed syntax tests patch both files of a pack and live in
//! `h3-pack-battle` (R01b3).

use super::*;

#[test]
fn the_core_files_parse_and_compile() {
    let r = compile(&[SYNTHETIC_CORE], "synthetic:faithful").unwrap();
    // Every creature of the file (the battle lane's synthetic pack grows with its epics).
    let creatures = SYNTHETIC_CORE.matches("\n[[creature]]").count();
    assert!(creatures >= 12);
    assert_eq!(r.creature_count(), creatures);
    let spearman = r.creature_by_key("synthetic:spearman").unwrap();
    assert_eq!(r.creature_info(spearman).unwrap().growth, 12);
    let both = compile(
        &[SYNTHETIC_EXT_CORE, SYNTHETIC_CORE],
        "synthetic:strict-fixed",
    )
    .unwrap();
    assert!(both.creature_by_key("synthetic-ext:lurker").is_some());
    let sod = compile(&[SOD_CORE, AB_CORE, BASE_CORE], "base:faithful").unwrap();
    assert!(sod.hero_type_by_key("base:orrin").is_some());
    assert!(matches!(
        compile(&[SOD_CORE, BASE_CORE], "base:faithful"),
        Err(PackError::Compile(_))
    ));
}

#[test]
fn core_files_are_strict() {
    let syntax = |text: &str| matches!(parse_pack(text), Err(PackError::Syntax(_)));
    assert!(syntax("[pack]\nid = \"x\"\n"));
    // The battle's tables and fields are not the core file's.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    for extra in [
        "[battle]\nfield_width = 17\n",
        "[[obstacle]]\nid = \"o\"\nterrains = []\nblocked = []\n",
        "[[creature]]\nid = \"c\"\nspeed = 1\nattack = 1\n",
        "[[creature_override]]\ntarget = \"x:c\"\nabilities = []\n",
        "[[spell]]\nid = \"s\"\ncategory = \"combat\"\nlevel = 1\ncost = [1, 1, 1, 1]\n\
         effect = [1, 1, 1, 1]\npolarity = \"neutral\"\n",
    ] {
        assert!(syntax(&format!("{manifest}{extra}")), "{extra}");
    }
    let pack = parse_pack(&format!(
        "{manifest}[[creature]]\nid = \"c\"\nspeed = 3\nfaction = \"y:f\"\nupgrades = [\"d\"]\n\
         [[creature_override]]\ntarget = \"x:c\"\nfixes = \"quirk.x\"\nfaction = \"\"\n\
         [[spell]]\nid = \"s\"\ncategory = \"adventure\"\nlevel = 2\ncost = [1, 1, 1, 1]\n\
         effect = [1, 2, 3, 4]\n"
    ))
    .unwrap();
    assert_eq!(pack.manifest.id, "x");
    assert_eq!(pack.creatures[0].speed, 3);
    assert_eq!(
        pack.creatures[0].info.faction.as_ref().unwrap().as_str(),
        "y:f"
    );
    assert_eq!(pack.creatures[0].info.upgrades[0].as_str(), "x:d");
    assert_eq!(pack.creature_overrides[0].patch.faction, Some(None));
    assert_eq!(pack.spells[0].category, SpellCategory::Adventure);
    let key = |text: &str| matches!(parse_pack(text), Err(PackError::Key(_)));
    assert!(key(&format!(
        "{manifest}[[resource]]\nid = \"Gold\"\nname = \"Gold\"\n"
    )));
    assert!(key(&format!(
        "{manifest}[[creature_override]]\ntarget = \"c\"\n"
    )));
}

#[test]
fn references_and_errors() {
    assert_eq!(reference("p", "a").unwrap().as_str(), "p:a");
    assert_eq!(reference("p", "q:a").unwrap().as_str(), "q:a");
    let list = references("p", &["a".into(), "q:b".into()]).unwrap();
    assert_eq!(list[1].as_str(), "q:b");
    assert!(reference("p", "A").is_err());
    assert_eq!(PackError::Syntax("x".into()).to_string(), "syntax: x");
    let key = ContentKey::parse("bad").unwrap_err();
    assert!(PackError::from(key).to_string().contains("not a valid key"));
    let e = PackError::from(CompileError::UnknownProfile("a:b".into()));
    assert_eq!(
        e.to_string(),
        CompileError::UnknownProfile("a:b".into()).to_string()
    );
}

#[test]
fn the_adventure_and_outcome_effects_parse() {
    // E08 T18a: the kinds the battle never reads; no harvested pack uses them yet.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let pack = parse_pack(&format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"necromancy\", bp = 1000 }}, {{ kind = \"join_bonus\", level = 1 }}]\n\
         advanced = [{{ kind = \"experience_gain\", bp = 1000 }}]\n\
         expert = [{{ kind = \"max_mana\", bp = 10000 }}]\n"
    ))
    .unwrap();
    let levels = &pack.skills[0].levels;
    assert_eq!(
        levels[0],
        [
            HeroEffect::Necromancy {
                bp: 1000,
                unless_skill: None
            },
            HeroEffect::JoinBonus { level: 1 }
        ]
    );
    assert_eq!(levels[1], [HeroEffect::ExperienceGain { bp: 1000 }]);
    assert_eq!(levels[2], [HeroEffect::MaxMana { bp: 10_000 }]);
    // The parameters are checked when the pack compiles.
    let bad = format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"join_bonus\", level = 4 }}]\n"
    );
    let error = compile(&[bad.as_str(), SYNTHETIC_CORE], "synthetic:faithful").unwrap_err();
    assert!(
        error.to_string().contains("join_bonus level must be 1..=3"),
        "{error}"
    );
}

#[test]
fn the_raised_creature_parses() {
    // E08 T19a2 (decision 30): `raises` names one creature, or one per level of a skill.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let skill = |basic: &str| {
        parse_pack(&format!(
            "{manifest}[[secondary_skill]]\nid = \"s\"\nbasic = [{basic}]\n"
        ))
    };
    let key = |text: &str| ContentKey::parse(text).unwrap();
    let pack = skill("{ kind = \"raises\", creature = \"bones\" }").unwrap();
    assert_eq!(
        pack.skills[0].levels[0],
        [HeroEffect::Raises {
            priority: 0,
            creature: RaisedCreature::Creature(key("x:bones")),
        }]
    );
    let pack = skill(
        "{ kind = \"raises\", priority = 1, skill = \"s\", \
         by_level = [\"bones\", \"y:walker\", \"wight\", \"lich\"] }",
    )
    .unwrap();
    assert_eq!(
        pack.skills[0].levels[0],
        [HeroEffect::Raises {
            priority: 1,
            creature: RaisedCreature::ByLevel {
                skill: key("x:s"),
                by_level: [
                    key("x:bones"),
                    key("y:walker"),
                    key("x:wight"),
                    key("x:lich")
                ],
            },
        }]
    );
    let four = "by_level = [\"a\", \"b\", \"c\", \"d\"]";
    for (bad, syntax) in [
        ("creature = \"a\", skill = \"s\"".to_owned(), true),
        ("skill = \"s\"".to_owned(), true),
        (four.to_owned(), true),
        (format!("creature = \"a\", skill = \"s\", {four}"), true),
        (
            "skill = \"s\", by_level = [\"a\", \"b\", \"c\"]".to_owned(),
            true,
        ),
        ("priority = 1".to_owned(), true),
        ("creature = \"A\"".to_owned(), false),
        (format!("skill = \"S\", {four}"), false),
        (
            "skill = \"s\", by_level = [\"a\", \"b\", \"C\", \"d\"]".to_owned(),
            false,
        ),
    ] {
        let error = skill(&format!("{{ kind = \"raises\", {bad} }}")).unwrap_err();
        assert_eq!(
            matches!(error, PackError::Syntax(_)),
            syntax,
            "{bad}: {error}"
        );
        assert_eq!(
            matches!(error, PackError::Key(_)),
            !syntax,
            "{bad}: {error}"
        );
    }
}

#[test]
fn the_ballistics_effect_parses() {
    // E04c T6 (BAT-Q43): the hero's Ballistics level, the battle's Catapult control.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let pack = parse_pack(&format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         expert = [{{ kind = \"ballistics\", level = 3 }}]\n"
    ))
    .unwrap();
    assert_eq!(
        pack.skills[0].levels[2],
        [HeroEffect::Ballistics { level: 3 }]
    );
    let bad = format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"ballistics\", level = 0 }}]\n"
    );
    let error = compile(&[bad.as_str(), SYNTHETIC_CORE], "synthetic:faithful").unwrap_err();
    assert!(
        error.to_string().contains("ballistics level must be 1..=3"),
        "{error}"
    );
}

#[test]
fn wisdom_and_the_skill_condition_of_a_share_parse() {
    // E08 T19a2b (pack format v40): `learn_level`, the hero parameter it adds to, and the
    // `necromancy` share that counts only without a skill (the Cloak of the Undead King).
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let pack = parse_pack(&format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"learn_level\", levels = 1 }}]\n\
         [[artifact]]\nid = \"a\"\nname = \"A\"\nclass = \"relic\"\n\
         effects = [{{ kind = \"necromancy\", bp = 3000, unless_skill = \"s\" }}]\n\
         [hero_params]\nprimary_min = [0, 0, 1, 1]\nprimary_max = 99\nbackpack = 64\n\
         learn_level = 2\nexperience = [0, 10]\nmax_level = 1\nwisdom = \"s\"\nschools = []\n\
         mana_ceiling = 999\n"
    ))
    .unwrap();
    assert_eq!(
        pack.skills[0].levels[0],
        [HeroEffect::LearnLevel { levels: 1 }]
    );
    assert_eq!(
        pack.artifacts[0].effects,
        [HeroEffect::Necromancy {
            bp: 3000,
            unless_skill: Some(ContentKey::parse("x:s").unwrap())
        }]
    );
    assert_eq!(pack.hero_params.as_ref().map(|p| p.learn_level), Some(2));
    // The hero parameters need the learnable level; a bad skill key is a key error.
    let without = format!(
        "{manifest}[hero_params]\nprimary_min = [0, 0, 1, 1]\nprimary_max = 99\nbackpack = 64\n"
    );
    assert!(matches!(parse_pack(&without), Err(PackError::Syntax(_))));
    let bad_key = format!(
        "{manifest}[[artifact]]\nid = \"a\"\nname = \"A\"\nclass = \"relic\"\n\
         effects = [{{ kind = \"necromancy\", bp = 3000, unless_skill = \"S\" }}]\n"
    );
    assert!(matches!(parse_pack(&bad_key), Err(PackError::Key(_))));
    // The parameters are checked when the pack compiles.
    let bad = format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"learn_level\", levels = 0 }}]\n"
    );
    let error = compile(&[bad.as_str(), SYNTHETIC_CORE], "synthetic:faithful").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("learn_level levels must be 1..=5"),
        "{error}"
    );
}

#[test]
fn the_movement_and_scouting_kinds_parse() {
    // E08 T35b (pack format v46, increment 5): the six kinds the adventure's derivations read.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let pack = parse_pack(&format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"pathfinding\", level = 1 }}, {{ kind = \"land_movement\", bp = 1000 }}]\n\
         expert = [{{ kind = \"scouting_radius\", tiles = 3 }}]\n\
         [[artifact]]\nid = \"a\"\nname = \"A\"\nclass = \"relic\"\n\
         effects = [{{ kind = \"land_movement_points\", points = 600 }}, \
         {{ kind = \"flying\", level = 3 }}, {{ kind = \"water_walking\", level = 0 }}]\n"
    ))
    .unwrap();
    assert_eq!(
        pack.skills[0].levels,
        [
            vec![
                HeroEffect::Pathfinding { level: 1 },
                HeroEffect::LandMovement { bp: 1000 }
            ],
            vec![],
            vec![HeroEffect::ScoutingRadius { tiles: 3 }],
        ]
    );
    assert_eq!(
        pack.artifacts[0].effects,
        [
            HeroEffect::LandMovementPoints { points: 600 },
            HeroEffect::Flying { level: 3 },
            HeroEffect::WaterWalking { level: 0 },
        ]
    );
    // An unknown field is a syntax error; the parameters are checked when the pack compiles.
    let unknown = format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"flying\", level = 1, cost = 100 }}]\n"
    );
    assert!(matches!(parse_pack(&unknown), Err(PackError::Syntax(_))));
    for (effect, message) in [
        (
            "kind = \"pathfinding\", level = 4",
            "pathfinding level must be 1..=3",
        ),
        (
            "kind = \"land_movement\", bp = 0",
            "land_movement bp must be at least 1",
        ),
        (
            "kind = \"land_movement_points\", points = 0",
            "land_movement_points points must be at least 1",
        ),
        (
            "kind = \"water_walking\", level = 4",
            "flying and water_walking levels must be 0..=3",
        ),
        (
            "kind = \"scouting_radius\", tiles = 0",
            "scouting_radius tiles must be at least 1",
        ),
    ] {
        let bad = format!("{manifest}[[secondary_skill]]\nid = \"s\"\nbasic = [{{ {effect} }}]\n");
        let error = compile(&[bad.as_str(), SYNTHETIC_CORE], "synthetic:faithful").unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
fn the_meeting_and_spell_book_kinds_parse() {
    // E08 T45 (pack format v47, increment 6): Scholar at a meeting, Titan's Thunder's book.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let pack = parse_pack(&format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"meeting_spells\", level = 2 }}]\n\
         [[artifact]]\nid = \"a\"\nname = \"A\"\nclass = \"relic\"\n\
         effects = [{{ kind = \"gives_spell_book\" }}]\n"
    ))
    .unwrap();
    assert_eq!(
        pack.skills[0].levels,
        [vec![HeroEffect::MeetingSpells { level: 2 }], vec![], vec![]]
    );
    assert_eq!(pack.artifacts[0].effects, [HeroEffect::GivesSpellBook]);
    // An unknown field is a syntax error, also on the kind without parameters.
    for effect in [
        "kind = \"meeting_spells\", level = 2, spells = 3",
        "kind = \"gives_spell_book\", level = 1",
    ] {
        let unknown =
            format!("{manifest}[[secondary_skill]]\nid = \"s\"\nbasic = [{{ {effect} }}]\n");
        assert!(
            matches!(parse_pack(&unknown), Err(PackError::Syntax(_))),
            "{effect}"
        );
    }
    for level in [0, 6] {
        let bad = format!(
            "{manifest}[[secondary_skill]]\nid = \"s\"\n\
             basic = [{{ kind = \"meeting_spells\", level = {level} }}]\n"
        );
        let error = compile(&[bad.as_str(), SYNTHETIC_CORE], "synthetic:faithful").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("meeting_spells level must be 1..=5"),
            "{error}"
        );
    }
}

/// E08 T45 (pack format v47): Scholar's levels from the gap-fill in every expansion, Titan's
/// Thunder's book with SoD (HER-SPL-007, HER-ART-007).
#[test]
fn the_harvested_scholar_and_titans_thunder() {
    for packs in [
        &[SOD_CORE, AB_CORE, BASE_CORE][..],
        &[AB_CORE, BASE_CORE],
        &[BASE_CORE],
    ] {
        let r = compile(packs, "base:faithful").unwrap();
        let scholar = &r
            .skill(r.skill_by_key("base:scholar").unwrap())
            .unwrap()
            .levels;
        assert_eq!(
            scholar.clone(),
            [2, 3, 4].map(|level| vec![HeroEffect::MeetingSpells { level }])
        );
    }
    let r = compile(&[SOD_CORE, AB_CORE, BASE_CORE], "base:faithful").unwrap();
    let thunder = r
        .artifact(r.artifact_by_key("sod:titans_thunder").unwrap())
        .unwrap();
    assert!(
        thunder.effects.contains(&HeroEffect::GivesSpellBook),
        "{thunder:?}"
    );
}

#[test]
fn the_sea_kinds_parse() {
    // E08 T70 (pack format v49, increment 8): Navigation, the sea points, free boarding.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let pack = parse_pack(&format!(
        "{manifest}[[secondary_skill]]\nid = \"s\"\n\
         basic = [{{ kind = \"navigation\", level = 1 }}]\n\
         [[artifact]]\nid = \"a\"\nname = \"A\"\nclass = \"relic\"\n\
         effects = [{{ kind = \"sea_movement_points\", points = 500 }}, \
         {{ kind = \"free_boarding\" }}]\n"
    ))
    .unwrap();
    assert_eq!(
        pack.skills[0].levels,
        [vec![HeroEffect::Navigation { level: 1 }], vec![], vec![]]
    );
    assert_eq!(
        pack.artifacts[0].effects,
        [
            HeroEffect::SeaMovementPoints { points: 500 },
            HeroEffect::FreeBoarding
        ]
    );
    // An unknown field is a syntax error, also on the kind without parameters.
    for effect in [
        "kind = \"navigation\", level = 2, bp = 3",
        "kind = \"free_boarding\", points = 1",
    ] {
        let unknown =
            format!("{manifest}[[secondary_skill]]\nid = \"s\"\nbasic = [{{ {effect} }}]\n");
        assert!(
            matches!(parse_pack(&unknown), Err(PackError::Syntax(_))),
            "{effect}"
        );
    }
    for (effect, message) in [
        (
            "kind = \"navigation\", level = 0",
            "navigation level must be 1..=3",
        ),
        (
            "kind = \"navigation\", level = 4",
            "navigation level must be 1..=3",
        ),
        (
            "kind = \"sea_movement_points\", points = 0",
            "sea_movement_points points must be at least 1",
        ),
    ] {
        let bad = format!("{manifest}[[secondary_skill]]\nid = \"s\"\nbasic = [{{ {effect} }}]\n");
        let error = compile(&[bad.as_str(), SYNTHETIC_CORE], "synthetic:faithful").unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
}

/// E08 T70 (pack format v49): Navigation's levels and the sea artifacts' points from the gap-fill
/// in every expansion, the Admiral's Hat's free boarding with SoD (HER-ADV-009).
#[test]
fn the_harvested_navigation_and_sea_artifacts() {
    for packs in [
        &[SOD_CORE, AB_CORE, BASE_CORE][..],
        &[AB_CORE, BASE_CORE],
        &[BASE_CORE],
    ] {
        let r = compile(packs, "base:faithful").unwrap();
        let navigation = &r
            .skill(r.skill_by_key("base:navigation").unwrap())
            .unwrap()
            .levels;
        assert_eq!(
            navigation.clone(),
            [1, 2, 3].map(|level| vec![HeroEffect::Navigation { level }])
        );
        for (key, points) in [
            ("base:necklace_of_ocean_guidance", 1000),
            ("base:sea_captains_hat", 500),
        ] {
            let artifact = r.artifact(r.artifact_by_key(key).unwrap()).unwrap();
            assert!(
                artifact
                    .effects
                    .contains(&HeroEffect::SeaMovementPoints { points }),
                "{artifact:?}"
            );
            assert!(
                !artifact.effects.contains(&HeroEffect::FreeBoarding),
                "{artifact:?}"
            );
        }
    }
    let r = compile(&[SOD_CORE, AB_CORE, BASE_CORE], "base:faithful").unwrap();
    let hat = r
        .artifact(r.artifact_by_key("sod:admirals_hat").unwrap())
        .unwrap();
    // The combination's own effect; its points come through its parts (HER-ADV-009).
    assert_eq!(hat.effects, [HeroEffect::FreeBoarding], "{hat:?}");
}

#[test]
fn the_mana_ceiling_parses_and_is_required() {
    // E08 T59 (pack format v48): the most spell points a reward record leaves a hero.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let params = "[hero_params]\nprimary_min = [0, 0, 1, 1]\nprimary_max = 99\nbackpack = 64\n\
                  learn_level = 2\nexperience = [0, 10]\nmax_level = 1\nwisdom = \"s\"\n\
                  schools = []\n";
    let pack = parse_pack(&format!("{manifest}{params}mana_ceiling = 999\n")).unwrap();
    assert_eq!(pack.hero_params.map(|p| p.mana_ceiling), Some(999));
    // Without it, or above a spell point's range, the parameters do not parse.
    for tail in ["", "mana_ceiling = 65536\n"] {
        assert!(matches!(
            parse_pack(&format!("{manifest}{params}{tail}")),
            Err(PackError::Syntax(_))
        ));
    }
    // The harvested base pack holds the original's 999.
    let base = compile(&[BASE_CORE], "base:faithful").unwrap();
    assert_eq!(base.hero_params().map(|p| p.mana_ceiling), Some(999));
}

#[test]
fn the_harvested_flags_of_increment_7() {
    // E08 T59: A-21's and A-25's flags (`ADV-QUIRK-21`, `-22`), on in `faithful` and off in
    // `strict-fixed` with every expansion.
    use h3_content::{FULL_BACKPACK_REFUSES, ROE_WITCH_HUT_MASK};
    for packs in [
        &[SOD_CORE, AB_CORE, BASE_CORE][..],
        &[AB_CORE, BASE_CORE],
        &[BASE_CORE],
    ] {
        for (profile, on) in [("base:faithful", true), ("base:strict-fixed", false)] {
            let q = compile(packs, profile).unwrap().quirks();
            assert_eq!(
                [ROE_WITCH_HUT_MASK, FULL_BACKPACK_REFUSES].map(|f| q.is_enabled(f)),
                [on; 2],
                "{profile}"
            );
        }
    }
}

#[test]
fn the_level_data_and_class_overrides_parse() {
    // E08 T26b (pack format v44): the experience of the first levels, the highest level, the
    // skills offered when due, the class's intervals and a gated class override.
    let manifest = "[pack]\nid = \"x\"\nversion = \"1\"\n";
    let class = "[[hero_class]]\nid = \"c\"\nfaction = \"f\"\nprimary = [1, 1, 1, 1]\n\
                 primary_gain = [[25, 25, 25, 25], [25, 25, 25, 25]]\n";
    let params = "[hero_params]\nprimary_min = [0, 0, 1, 1]\nprimary_max = 99\nbackpack = 64\n\
                  learn_level = 2\nexperience = [0, 1000, 2000]\nmax_level = 74\n\
                  mana_ceiling = 999\n";
    let pack = parse_pack(&format!(
        "{manifest}{params}wisdom = \"w\"\nschools = [\"base:fire\", \"air\"]\n\
         {class}offer_intervals = [6, 4]\n\
         [[hero_class_override]]\ntarget = \"x:c\"\nfixes = \"quirk.content.q\"\n\
         offer_intervals = [3, 3]\n"
    ))
    .unwrap();
    let p = pack.hero_params.as_ref().unwrap();
    assert_eq!(
        (p.experience.as_slice(), p.max_level),
        (&[0, 1000, 2000][..], 74)
    );
    assert_eq!(p.wisdom.as_str(), "x:w");
    assert_eq!(
        p.schools.iter().map(ContentKey::as_str).collect::<Vec<_>>(),
        ["base:fire", "x:air"]
    );
    assert_eq!(pack.hero_classes[0].offer_intervals, [6, 4]);
    let o = &pack.hero_class_overrides[0];
    assert_eq!(
        (o.target.as_str(), o.fixes.as_deref()),
        ("x:c", Some("quirk.content.q"))
    );
    assert_eq!(o.patch.offer_intervals, Some([3, 3]));
    // A class needs its intervals; the parameters need the level data; keys are checked.
    let syntax = |text: &str| matches!(parse_pack(text), Err(PackError::Syntax(_)));
    assert!(syntax(&format!("{manifest}{class}")));
    assert!(syntax(&format!("{manifest}{params}wisdom = \"w\"\n")));
    let key = |text: &str| matches!(parse_pack(text), Err(PackError::Key(_)));
    assert!(key(&format!(
        "{manifest}{params}wisdom = \"W\"\nschools = []\n"
    )));
    assert!(key(&format!(
        "{manifest}{params}wisdom = \"w\"\nschools = [\"A\"]\n"
    )));
    assert!(key(&format!(
        "{manifest}[[hero_class_override]]\ntarget = \"c\"\noffer_intervals = [3, 3]\n"
    )));
}

/// E08 T26b: the harvested packs' level data (HER-EXP-002, HER-LVL-004) and the Elementalist's
/// intervals, the executable's in `faithful` and the magic classes' in `strict-fixed`
/// (ADV-QUIRK-10), with and without SoD.
#[test]
fn the_harvested_movement_and_scouting_effects() {
    // E08 T35b (pack format v46): the three skills' and six artifacts' effects from the gap-fill,
    // and the flags of increment 5, on in `faithful` and off in `strict-fixed` (the third, flag
    // 30, retired in E08 T41: ADV-Q22).
    use h3_content::{DROP_IGNORES_ARTIFACTS, RADIUS_WAITS_FOR_A_STEP};
    for packs in [
        &[SOD_CORE, AB_CORE, BASE_CORE][..],
        &[AB_CORE, BASE_CORE],
        &[BASE_CORE],
    ] {
        for (profile, on) in [("base:faithful", true), ("base:strict-fixed", false)] {
            let r = compile(packs, profile).unwrap();
            let skill = |k: &str| {
                let levels = &r.skill(r.skill_by_key(k).unwrap()).unwrap().levels;
                levels.clone().map(|l| l.into_iter().next().unwrap())
            };
            assert_eq!(
                skill("base:pathfinding"),
                [1, 2, 3].map(|level| HeroEffect::Pathfinding { level })
            );
            assert_eq!(
                skill("base:logistics"),
                [1000, 2000, 3000].map(|bp| HeroEffect::LandMovement { bp })
            );
            assert_eq!(
                skill("base:scouting"),
                [1, 2, 3].map(|tiles| HeroEffect::ScoutingRadius { tiles })
            );
            let artifact = |k: &str| {
                r.artifact(r.artifact_by_key(k).unwrap())
                    .unwrap()
                    .effects
                    .clone()
            };
            assert_eq!(
                [
                    "base:boots_of_speed",
                    "base:equestrians_gloves",
                    "base:angel_wings",
                    "base:boots_of_levitation",
                    "base:speculum",
                    "base:spyglass"
                ]
                .map(artifact),
                [
                    vec![HeroEffect::LandMovementPoints { points: 600 }],
                    vec![HeroEffect::LandMovementPoints { points: 300 }],
                    vec![HeroEffect::Flying { level: 3 }],
                    vec![HeroEffect::WaterWalking { level: 3 }],
                    vec![HeroEffect::ScoutingRadius { tiles: 1 }],
                    vec![HeroEffect::ScoutingRadius { tiles: 1 }],
                ]
            );
            let q = r.quirks();
            assert_eq!(
                [DROP_IGNORES_ARTIFACTS, RADIUS_WAITS_FOR_A_STEP].map(|f| q.is_enabled(f)),
                [on; 2],
                "{profile}"
            );
        }
    }
}

#[test]
fn the_harvested_level_data_and_the_elementalists_intervals() {
    for packs in [&[SOD_CORE, AB_CORE, BASE_CORE][..], &[AB_CORE, BASE_CORE]] {
        for (profile, elementalist) in [("base:faithful", [6, 4]), ("base:strict-fixed", [3, 3])] {
            let r = compile(packs, profile).unwrap();
            let p = r.hero_params().unwrap();
            assert_eq!((p.max_level, p.experience.len()), (74, 75));
            assert_eq!(
                [
                    p.experience[1],
                    p.experience[11],
                    p.experience[12],
                    p.experience[74]
                ],
                [1000, 20_600, 24_320, 1_810_034_207]
            );
            let skill = |k: &str| r.skill_by_key(k).unwrap();
            assert_eq!(p.wisdom, skill("base:wisdom"));
            assert_eq!(
                p.schools,
                [
                    "base:fire_magic",
                    "base:air_magic",
                    "base:water_magic",
                    "base:earth_magic"
                ]
                .map(skill)
            );
            let intervals = |k: &str| {
                r.hero_classes()
                    .iter()
                    .find(|c| c.key.as_str() == k)
                    .unwrap()
                    .offer_intervals
            };
            assert_eq!(
                [
                    "base:knight",
                    "base:cleric",
                    "base:witch",
                    "ab:planeswalker",
                    "ab:elementalist"
                ]
                .map(intervals),
                [[6, 4], [3, 3], [3, 3], [6, 4], elementalist],
                "{profile}"
            );
            let short = r
                .hero_classes()
                .iter()
                .filter(|c| c.offer_intervals == [3, 3])
                .count();
            assert_eq!(short, if elementalist == [3, 3] { 9 } else { 8 });
        }
    }
}
