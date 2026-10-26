//! Tests of the store of heroes (GAM-HER-001, E08 T18b): the defaults and the setup's
//! customisations, the war machines a starting army gives, full spell points, validation by the
//! hero model, the hash and the observation.

use std::sync::Arc;

use h3_content::SkillId;
use h3_heroes::{ArtifactInstance, DetailsError, HeroDetails, PrimarySkill, wearing};
use h3_kernel::hash::hash_of;

use super::*;

const GUNNER: HeroTypeId = HeroTypeId::from_raw(3);
const SAGE: HeroTypeId = HeroTypeId::from_raw(4);
const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const ONE: HeroId = HeroId::new(1);

fn art(rules: &GameRuleset, k: &str) -> ArtifactInstance {
    rules.core().artifact_by_key(k).unwrap().into()
}

/// Blue's hero's details when it is of `hero_type`, with the setup edited by `edit`.
fn details(
    rules: &GameRuleset,
    hero_type: HeroTypeId,
    seed: u64,
    edit: impl FnOnce(&mut GameHeroSetup),
) -> HeroDetails {
    let mut setup = setup();
    setup.heroes[1].hero_type = hero_type;
    edit(&mut setup.heroes[1]);
    GameState::new(rules, &setup, seed)
        .unwrap()
        .hero(ONE)
        .unwrap()
        .clone()
}

// covers: GAM-HER-001, HER-MAN-001
#[test]
fn a_hero_without_customisations_gets_its_types_and_classs_defaults() {
    let rules = rules();
    let sage = details(&rules, SAGE, 3, |_| {});
    assert_eq!((sage.hero_type, sage.level), (SAGE, 1));
    assert_eq!(sage.primary, [1, 1, 1, 1], "the class's");
    let wits = rules.core().skill_by_key("t:wits").unwrap();
    assert_eq!(sage.skills, [(wits, 3)]);
    // Positions: the book 0, the machine 1, the rings 2 and 3.
    assert_eq!(
        sage.worn,
        [Some(art(&rules, "t:book")), None, None, None],
        "its type starts with a spell book"
    );
    assert!(sage.backpack.is_empty() && sage.spells.is_empty());
    // Spell points full: Knowledge 1 and Expert Wits (+100 %).
    assert_eq!(sage.mana, 20);
    let orrin = details(&rules, ORRIN, 3, |_| {});
    assert!(orrin.worn.iter().all(Option::is_none) && orrin.skills.is_empty());
    assert_eq!(orrin.mana, 10);
    // The experience is 40 to 90, drawn from the setup stream: the same seed, the same draw.
    let drawn: Vec<u32> = (0..200)
        .map(|seed| details(&rules, ORRIN, seed, |_| {}).experience)
        .collect();
    assert!(drawn.iter().all(|e| (40..=90).contains(e)));
    assert!(drawn.contains(&40) && drawn.contains(&90));
    assert_eq!(details(&rules, ORRIN, 7, |_| {}).experience, drawn[7]);
}

// covers: GAM-HER-001, GAM-SET-001
#[test]
fn a_war_machine_its_starting_army_gives_is_worn_as_its_artifact() {
    // The gunner's slots: imp, the cart, gog; the cart's roll gives its artifact.
    let cart = |chances: Vec<u8>, worn: Option<Vec<ArtifactInstance>>| {
        let rules = rules_with(chances, 2);
        details(&rules, GUNNER, 3, |hero| {
            hero.details.artifacts = worn.map(|worn| HeroArtifactsSetup {
                worn: wearing(rules.core(), &worn),
                backpack: vec![],
            });
        })
        .worn
    };
    let rules = rules();
    let cart_art = art(&rules, "t:cart_art");
    assert_eq!(
        cart(vec![100, 100], None),
        [None, Some(cart_art), None, None]
    );
    assert_eq!(cart(vec![0, 100], None), [None; 4]);
    // A custom army draws nothing: no machine.
    let custom = details(&rules, GUNNER, 3, |hero| {
        hero.army = HeroArmy::Custom(vec![ArmyStack {
            slot: 0,
            creature: IMP,
            count: 1,
        }]);
    });
    assert!(custom.worn.iter().all(Option::is_none));
    // A machine whose slot a worn artifact takes is not given twice.
    assert_eq!(
        cart(vec![100, 100], Some(vec![cart_art])),
        [None, Some(cart_art), None, None],
        "the slot is taken"
    );
}

// covers: GAM-HER-001, HER-MAN-001
#[test]
fn the_setups_customisations_replace_the_defaults() {
    let rules = rules();
    let wits = rules.core().skill_by_key("t:wits").unwrap();
    let charm = art(&rules, "t:charm");
    let custom = details(&rules, SAGE, 3, |hero| {
        hero.details = HeroDetailsSetup {
            experience: Some(999),
            primary: Some([2, 3, 4, 8]),
            skills: Some(vec![(wits, 1)]),
            artifacts: Some(HeroArtifactsSetup {
                worn: wearing(rules.core(), &[]),
                backpack: vec![charm],
            }),
            spells: Some(vec![]),
            sex: None,
        };
    });
    assert_eq!(custom.experience, 999);
    assert_eq!(custom.primary, [2, 3, 4, 8]);
    assert_eq!(custom.skills, [(wits, 1)]);
    // Custom artifacts replace the type's spell book.
    assert!(custom.worn.iter().all(Option::is_none));
    assert_eq!(custom.backpack, [charm]);
    // Knowledge 8 and Basic Wits: 80 × 1.25.
    assert_eq!(custom.mana, 100);
}

// covers: GAM-HER-001
#[test]
fn the_hero_model_validates_the_details() {
    let mut setup = setup();
    setup.heroes[1].details.primary = Some([0, 0, 0, 100]);
    assert_eq!(
        setup.validate(&rules()),
        Err(GameSetupError::Hero(
            ONE,
            DetailsError::PrimaryAboveLimit(PrimarySkill::Knowledge)
        ))
    );
}

// covers: GAM-HER-001
#[test]
fn the_store_is_in_the_hash_and_shared_by_forks() {
    let rules = rules();
    let mut setup = setup();
    setup.heroes[0].details.experience = Some(100);
    let state = GameState::new(&rules, &setup, 3).unwrap();
    assert_eq!(state.heroes().len(), 2);
    assert_eq!(state.hero(HeroId::new(0)).unwrap().experience, 100);
    assert_eq!(state.hero(HeroId::new(2)), None);
    let fork = state.clone();
    assert!(Arc::ptr_eq(&fork.heroes()[0], &state.heroes()[0]));
    assert_eq!(fork.state_hash(), state.state_hash());
    assert_eq!(state.state_hash(), hash_of(&state));
    let changes: Vec<fn(&mut GameHeroSetup)> = vec![
        |h| h.details.experience = Some(101),
        |h| h.details.primary = Some([1, 1, 1, 2]),
        |h| h.details.skills = Some(vec![(SkillId::from_raw(0), 1)]),
        |h| {
            h.details.artifacts = Some(HeroArtifactsSetup {
                // The book on its position 0 of four.
                worn: vec![
                    Some(ArtifactInstance::from(
                        h3_content::ArtifactTypeId::from_raw(0),
                    )),
                    None,
                    None,
                    None,
                ],
                backpack: vec![],
            });
        },
        |h| {
            h.details.artifacts = Some(HeroArtifactsSetup {
                worn: vec![None; 4],
                backpack: vec![ArtifactInstance::from(
                    h3_content::ArtifactTypeId::from_raw(2),
                )],
            });
        },
        |h| {
            h.details.artifacts = Some(HeroArtifactsSetup {
                worn: vec![None; 4],
                backpack: vec![ArtifactInstance::scroll(
                    h3_content::ArtifactTypeId::from_raw(3),
                    h3_content::SpellId::from_raw(0),
                )],
            });
        },
        |h| h.details.spells = Some(vec![h3_content::SpellId::from_raw(0)]),
        |h| h.hero_type = SAGE,
        |h| h.formation = Formation::Tight,
    ];
    for (i, change) in changes.iter().enumerate() {
        let mut other = setup.clone();
        change(&mut other.heroes[0]);
        let other = GameState::new(&rules, &other, 3).unwrap();
        assert_ne!(other.state_hash(), state.state_hash(), "change {i}");
    }
}

// covers: GAM-HER-001
#[test]
fn a_player_sees_its_heroes_details_and_the_others_types() {
    let rules = rules();
    let mut setup = setup();
    setup.heroes[1].hero_type = SAGE;
    let state = GameState::new(&rules, &setup, 3).unwrap();
    let red = observe(&state, &rules, Viewer::Player(RED));
    let blue = observe(&state, &rules, Viewer::Player(BLUE));
    let info = |o: &GameObservation, id: u16| {
        o.heroes
            .iter()
            .find(|h| h.id == HeroId::new(id))
            .unwrap()
            .clone()
    };
    assert_eq!(red.heroes.len(), 2);
    assert_eq!(info(&red, 0).details.as_deref(), state.hero(HeroId::new(0)));
    assert_eq!(info(&red, 1).hero_type, SAGE);
    assert_eq!(info(&red, 1).details, None);
    assert_eq!(info(&blue, 1).details.as_deref(), state.hero(ONE));
    assert_eq!(info(&blue, 0).details, None);
}
