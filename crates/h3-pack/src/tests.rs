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
         learn_level = 2\n"
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
