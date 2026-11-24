//! A spell's mastery on a ground, a cast's cost on the map and who may cast there (E08 T141b;
//! HER-SPL-003 amended, HER-SPL-008; ADR-0013 point 17).

use std::vec;

use h3_content::{HeroEffectDef, SchoolId};

use super::*;

/// The test pack with a second school and magic: Wisdom stands for Air Magic (Basic to Expert
/// air), the band gives Advanced fire, the book is a spell book; `t:flame` is a fire spell whose
/// Expert cost is 0, `t:storm` one of air and fire.
fn spells_ruleset() -> Ruleset {
    let mut p = pack();
    p.spell_schools.push(SpellSchoolDef {
        key: key("t:fire"),
        name: "Fire".into(),
        protection_rank: 0,
    });
    for s in &mut p.skills {
        if s.key.to_string() == "t:wisdom" {
            s.levels = [1, 2, 3].map(|level| {
                vec![HeroEffectDef::SchoolMastery {
                    school: key("t:air"),
                    level,
                }]
            });
        }
    }
    for a in &mut p.artifacts {
        match a.key.to_string().as_str() {
            "t:book" => a.effects = vec![HeroEffectDef::SpellBook],
            "t:band" => {
                a.effects = vec![HeroEffectDef::SchoolMastery {
                    school: key("t:fire"),
                    level: 2,
                }];
            }
            _ => {}
        }
    }
    p.spells.extend([
        SpellDef {
            schools: vec![key("t:fire")],
            cost: [9, 8, 6, 0],
            ..spell("t:flame", SpellCategory::Adventure)
        },
        SpellDef {
            schools: vec![key("t:air"), key("t:fire")],
            cost: [20, 15, 12, 10],
            ..spell("t:storm", SpellCategory::Combat)
        },
    ]);
    compile(p).unwrap()
}

fn school(r: &Ruleset, k: &str) -> SchoolId {
    let at = r
        .spell_schools()
        .iter()
        .position(|s| s.key.to_string() == k)
        .unwrap();
    SchoolId::from_index(at).unwrap()
}

/// Valid details with nothing worn or carried, the portal learned and the given skills.
fn plain(r: &Ruleset, skills: &[(&str, u8)]) -> HeroDetails {
    HeroDetails {
        skills: skills
            .iter()
            .map(|&(k, level)| (skill(r, k), level))
            .collect(),
        worn: bare(r),
        backpack: vec![],
        spells: vec![spell_id(r, "t:portal")],
        ..details(r)
    }
}

// covers: HER-SPL-003
#[test]
fn a_raised_ground_gives_its_spells_expert() {
    let r = spells_ruleset();
    let id = |k| spell_id(&r, k);
    let (air, fire) = (school(&r, "t:air"), school(&r, "t:fire"));
    // Basic air (Wisdom 1) and the wand (the bolt at Expert, HER-SPL-003's artifact raise).
    let d = HeroDetails {
        worn: wearing(&r, &[art(&r, "t:wand")]),
        ..plain(&r, &[("t:wisdom", 1)])
    };
    let on = |raise| spell_masteries_on(&d, &r, raise).unwrap();
    // Ordinary ground: the schools' and the wand's; no fire, so the flame is not listed.
    let ordinary = vec![
        (id("t:bolt"), 3),
        (id("t:portal"), 1),
        (id("t:titan"), 1),
        (id("t:storm"), 1),
    ];
    assert_eq!(on(SchoolRaise::None), ordinary);
    assert_eq!(spell_masteries(&d, &r), Ok(ordinary.clone()));
    assert_eq!(SchoolRaise::default(), SchoolRaise::None);
    // No school raised is ordinary ground.
    assert_eq!(on(SchoolRaise::Schools(SchoolSet::default())), ordinary);
    // Fire raised (Fiery Fields): every spell with a fire school at Expert, the storm by one of
    // its two; the air spells keep their Basic.
    let fiery: SchoolSet = [fire].into_iter().collect();
    assert!(fiery.contains(fire) && !fiery.contains(air));
    assert_eq!(
        on(SchoolRaise::Schools(fiery)),
        [
            (id("t:bolt"), 3),
            (id("t:portal"), 1),
            (id("t:titan"), 1),
            (id("t:flame"), 3),
            (id("t:storm"), 3),
        ]
    );
    // Air raised (Magic Clouds): a hero's Basic air spells at Expert, the flame still none.
    assert_eq!(
        on(SchoolRaise::Schools([air].into_iter().collect())),
        [
            (id("t:bolt"), 3),
            (id("t:portal"), 3),
            (id("t:titan"), 3),
            (id("t:storm"), 3),
        ]
    );
    // Every school (Magic Plains, the original's field 1): every spell at Expert, the ability
    // without a school too, and without any school mastery of the hero's.
    let nobody = plain(&r, &[]);
    assert_eq!(spell_masteries(&nobody, &r), Ok(vec![]));
    let all: Vec<(SpellId, u8)> = (0..r.spells().len())
        .map(|i| (SpellId::from_index(i).unwrap(), 3))
        .collect();
    assert_eq!(spell_masteries_on(&nobody, &r, SchoolRaise::All), Ok(all));
    // Invalid details are reported.
    let wrong = HeroDetails { level: 0, ..d };
    assert_eq!(
        spell_masteries_on(&wrong, &r, SchoolRaise::All),
        Err(DetailsError::LevelZero)
    );
}

// covers: HER-SPL-008
#[test]
fn a_map_cast_costs_its_mastery_column_at_least_one() {
    let r = spells_ruleset();
    let id = |k| spell_id(&r, k);
    let fiery = SchoolRaise::Schools([school(&r, "t:fire")].into_iter().collect());
    let nobody = plain(&r, &[]);
    let cost = |d: &HeroDetails, k, raise| map_spell_cost(d, &r, id(k), raise);
    // Without a mastery the first column; a raised ground's Expert column, at least 1 (the
    // flame's Expert costs 0).
    assert_eq!(cost(&nobody, "t:flame", SchoolRaise::None), Ok(9));
    assert_eq!(cost(&nobody, "t:flame", fiery), Ok(1));
    assert_eq!(cost(&nobody, "t:flame", SchoolRaise::All), Ok(1));
    // The band's Advanced fire: the flame's 6, the storm's 12 (its better school); Expert air
    // (Wisdom 3) the storm's 10, whether the hero has the spell or not.
    let banded = HeroDetails {
        worn: wearing(&r, &[art(&r, "t:band")]),
        ..nobody.clone()
    };
    assert_eq!(cost(&banded, "t:flame", SchoolRaise::None), Ok(6));
    assert_eq!(cost(&banded, "t:storm", SchoolRaise::None), Ok(12));
    let expert = plain(&r, &[("t:wisdom", 3)]);
    assert_eq!(cost(&expert, "t:storm", SchoolRaise::None), Ok(10));
    // The wand's Expert bolt: 4; the portal at Expert air: 4, at none: 5.
    let wand = HeroDetails {
        worn: wearing(&r, &[art(&r, "t:wand")]),
        ..nobody.clone()
    };
    assert_eq!(cost(&wand, "t:bolt", SchoolRaise::None), Ok(4));
    assert_eq!(cost(&nobody, "t:portal", SchoolRaise::None), Ok(5));
    assert_eq!(cost(&expert, "t:portal", SchoolRaise::None), Ok(4));
    // A spell the ruleset lacks; invalid details first.
    let unknown = SpellId::from_raw(99);
    assert_eq!(
        map_spell_cost(&nobody, &r, unknown, SchoolRaise::None),
        Err(DetailsError::UnknownSpell(unknown))
    );
    let wrong = HeroDetails { level: 0, ..nobody };
    assert_eq!(
        map_spell_cost(&wrong, &r, unknown, SchoolRaise::None),
        Err(DetailsError::LevelZero)
    );
}

// covers: HER-SPL-008
#[test]
fn a_hero_casts_on_the_map_what_it_has_without_a_book() {
    let r = spells_ruleset();
    let id = |k| spell_id(&r, k);
    let scroll_of = |k| ArtifactInstance::scroll(art(&r, "t:scroll").artifact, id(k));
    let nobody = plain(&r, &[]);
    let casts = |worn: &[ArtifactInstance], k| {
        let d = HeroDetails {
            worn: wearing(&r, worn),
            ..nobody.clone()
        };
        can_cast_on_map(&d, &r, id(k)).unwrap()
    };
    // A learned spell; one it does not have.
    assert!(casts(&[], "t:portal"));
    assert!(!casts(&[], "t:flame"));
    // A worn scroll's spell without a book (row E15-15), where the battle has no spell book; a
    // tome's spells alike; a book changes nothing.
    assert!(casts(&[scroll_of("t:flame")], "t:flame"));
    let scrolled = HeroDetails {
        worn: wearing(&r, &[scroll_of("t:flame")]),
        ..nobody.clone()
    };
    let hero = derive_battle_hero(&scrolled, BattleBonuses::NONE, &r).unwrap();
    assert_eq!(hero.magic.spell_book, None);
    assert!(casts(&[art(&r, "t:tome")], "t:bolt"));
    assert!(casts(&[art(&r, "t:book")], "t:portal"));
    assert!(!casts(&[art(&r, "t:book")], "t:bolt"));
    // A scroll in the backpack gives nothing.
    let carried = HeroDetails {
        backpack: vec![scroll_of("t:flame")],
        ..nobody.clone()
    };
    assert_eq!(can_cast_on_map(&carried, &r, id("t:flame")), Ok(false));
    // Invalid details are reported.
    let wrong = HeroDetails { level: 0, ..nobody };
    assert_eq!(
        can_cast_on_map(&wrong, &r, id("t:portal")),
        Err(DetailsError::LevelZero)
    );
}
