//! The core files' own tests. The detailed syntax tests patch both files of a pack and live in
//! `h3-pack-battle` (R01b3).

use super::*;

#[test]
fn the_core_files_parse_and_compile() {
    let r = compile(&[SYNTHETIC_CORE], "synthetic:faithful").unwrap();
    assert_eq!(r.creature_count(), 12);
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
            HeroEffect::Necromancy { bp: 1000 },
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
