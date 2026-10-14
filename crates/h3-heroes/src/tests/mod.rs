extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactSlotId, ArtifactTypeId, CompileError,
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, FactionDef, HeroClassDef, HeroEffect,
    HeroParamsDef, HeroTypeDef, HeroTypeId, PackManifest, ProfileDef, ResourceDef, Ruleset,
    SecondarySkillDef, SkillId, SpellCategory, SpellDef, SpellGrant, SpellId, SpellSchoolDef,
    StartingStackDef,
};
use proptest::prelude::*;

use super::*;

fn key(s: &str) -> ContentKey {
    ContentKey::parse(s).unwrap()
}

fn creature(k: &str, flags: &[&str]) -> CreatureDef {
    CreatureDef {
        key: key(k),
        speed: if flags.is_empty() { 5 } else { 0 },
        info: CreatureInfoDef {
            flags: flags.iter().map(ToString::to_string).collect(),
            ..CreatureInfoDef::default()
        },
    }
}

fn slot(k: &str, count: u8, backpack: bool) -> ArtifactSlotDef {
    ArtifactSlotDef {
        key: key(k),
        name: "Slot".into(),
        count,
        backpack,
    }
}

fn artifact(k: &str, slot: Option<&str>) -> ArtifactDef {
    ArtifactDef {
        key: key(k),
        name: "Artifact".into(),
        class: ArtifactClass::Treasure,
        cost: vec![],
        slot: slot.map(key),
        war_machine: None,
        needs_skill: None,
        effects: vec![],
        parts: vec![],
    }
}

fn spell(k: &str, category: SpellCategory) -> SpellDef {
    SpellDef {
        key: key(k),
        name: "Spell".into(),
        category,
        level: u8::from(category != SpellCategory::Ability),
        schools: if category == SpellCategory::Ability {
            vec![]
        } else {
            vec![key("t:air")]
        },
        cost: [5, 4, 4, 4],
        power: 10,
        effect: [10, 10, 20, 30],
        granted_by_name_only: false,
    }
}

const SKILLS: [&str; 10] = [
    "t:archery",
    "t:tactics",
    "t:wisdom",
    "t:luck",
    "t:leadership",
    "t:offense",
    "t:armorer",
    "t:artillery",
    "t:logistics",
    "t:first_aid",
];

/// Slots: head, two rings, three misc positions, a war-machine slot, a second machine slot
/// whose artifact places nothing (like the Catapult) and a book slot; the last three never go
/// to the backpack. Artifacts: `t:circle` is a combination of band, loop and helm worn as a ring
/// (it locks the other ring and the head); `t:kit` places the Ballista, `t:sling` the Catapult
/// (E08 T19a2a); the Grail has no slot.
fn pack() -> ContentPack {
    let mut kit = artifact("t:kit", Some("t:machine"));
    kit.war_machine = Some(key("t:ballista"));
    let mut sling = artifact("t:sling", Some("t:engine"));
    sling.war_machine = Some(key("t:catapult"));
    let mut circle = artifact("t:circle", Some("t:ring"));
    circle.parts = vec![key("t:band"), key("t:loop"), key("t:helm")];
    // The spell artifacts (E04b T8a): a scroll, an air tome, a crown of every level-1 spell and a
    // wand that names the portal and raises the bolt to Expert.
    let mut scroll = artifact("t:scroll", Some("t:misc"));
    scroll.effects = vec![HeroEffect::Scroll];
    let mut tome = artifact("t:tome", Some("t:misc"));
    tome.effects = vec![HeroEffect::Spells {
        grant: SpellGrant::School(key("t:air")),
    }];
    let mut crown = artifact("t:crown", Some("t:misc"));
    crown.effects = vec![HeroEffect::Spells {
        grant: SpellGrant::Level(1),
    }];
    let mut wand = artifact("t:wand", Some("t:misc"));
    wand.effects = vec![
        HeroEffect::Spells {
            grant: SpellGrant::Spell(key("t:titan")),
        },
        HeroEffect::SpellMastery {
            spell: key("t:bolt"),
            level: 3,
        },
    ];
    ContentPack {
        resources: vec![ResourceDef {
            key: key("t:gold"),
            name: "Gold".into(),
        }],
        factions: vec![FactionDef {
            key: key("t:town"),
            name: "Town".into(),
            native_terrain: None,
        }],
        creatures: vec![
            creature("t:peasant", &[]),
            creature("t:ballista", &["war_machine", "shooter"]),
            creature("t:catapult", &["war_machine", "catapult"]),
        ],
        spell_schools: vec![SpellSchoolDef {
            key: key("t:air"),
            name: "Air".into(),
            protection_rank: 0,
        }],
        spells: vec![
            spell("t:bolt", SpellCategory::Combat),
            spell("t:portal", SpellCategory::Adventure),
            spell("t:gaze", SpellCategory::Ability),
            // Only a grant naming it gives it (Titan's Lightning Bolt, HER-SPL-002).
            SpellDef {
                granted_by_name_only: true,
                ..spell("t:titan", SpellCategory::Combat)
            },
        ],
        skills: SKILLS
            .iter()
            .map(|k| SecondarySkillDef {
                key: key(k),
                name: "Skill".into(),
                levels: [vec![], vec![], vec![]],
            })
            .collect(),
        hero_classes: vec![HeroClassDef {
            key: key("t:knight"),
            name: "Knight".into(),
            faction: key("t:town"),
            primary: [2, 2, 1, 1],
            primary_gain: [[35, 45, 10, 10], [30, 30, 20, 20]],
            skill_weights: vec![],
        }],
        heroes: vec![HeroTypeDef {
            key: key("t:orrin"),
            name: "Orrin".into(),
            class: key("t:knight"),
            skills: vec![],
            spell_book: false,
            spells: vec![],
            army: vec![StartingStackDef {
                creature: key("t:peasant"),
                count: [1, 2],
            }],
            war_machines: vec![],
            specialty: vec![],
        }],
        artifact_slots: vec![
            slot("t:head", 1, true),
            slot("t:ring", 2, true),
            slot("t:misc", 3, true),
            slot("t:machine", 1, false),
            slot("t:engine", 1, false),
            slot("t:book", 1, false),
        ],
        artifacts: vec![
            artifact("t:helm", Some("t:head")),
            artifact("t:band", Some("t:ring")),
            artifact("t:loop", Some("t:ring")),
            circle,
            artifact("t:eye", Some("t:misc")),
            kit,
            artifact("t:hook", Some("t:engine")),
            artifact("t:book", Some("t:book")),
            artifact("t:grail", None),
            scroll,
            tome,
            crown,
            wand,
            sling,
        ],
        profiles: vec![
            ProfileDef {
                key: key("t:faithful"),
                quirks: vec![],
            },
            // The original's single-precision rates (quirk Q-0005).
            ProfileDef {
                key: key("t:original"),
                quirks: vec![("quirk.battle.single_precision_rates".into(), true)],
            },
        ],
        hero_params: Some(HeroParamsDef {
            primary_min: [0, 0, 1, 1],
            primary_max: 99,
            backpack: 3,
            learn_level: 2,
        }),
        ..ContentPack::new(PackManifest {
            id: "t".into(),
            version: "1".into(),
            depends: vec![],
        })
    }
}

fn compile(p: ContentPack) -> Result<Ruleset, CompileError> {
    Ruleset::compile(&[p], "t:faithful")
}

fn ruleset() -> Ruleset {
    compile(pack()).unwrap()
}

fn art(r: &Ruleset, k: &str) -> ArtifactInstance {
    r.artifact_by_key(k).unwrap().into()
}

fn skill(r: &Ruleset, k: &str) -> SkillId {
    r.skill_by_key(k).unwrap()
}

fn spell_id(r: &Ruleset, k: &str) -> SpellId {
    r.spell_by_key(k).unwrap()
}

fn slot_id(r: &Ruleset, k: &str) -> ArtifactSlotId {
    r.artifact_slot_by_key(k).unwrap()
}

/// Valid details that use every feature: the combination fills both rings and the head, two
/// copies of one misc artifact, every machine slot, the book, a backpack with an artifact
/// without slot, an adventure and a combat spell, and Knowledge 0 (below the minimum).
fn details(r: &Ruleset) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key("t:orrin").unwrap(),
        level: 1,
        experience: 0,
        primary: [99, 0, 1, 0],
        skills: vec![(skill(r, "t:archery"), 1), (skill(r, "t:tactics"), 3)],
        worn: vec![
            art(r, "t:circle"),
            art(r, "t:eye"),
            art(r, "t:eye"),
            art(r, "t:kit"),
            art(r, "t:hook"),
            art(r, "t:book"),
        ],
        backpack: vec![art(r, "t:helm"), art(r, "t:grail"), art(r, "t:eye")],
        spells: vec![spell_id(r, "t:portal"), spell_id(r, "t:bolt")],
        mana: u16::MAX,
    }
}

// covers: HER-DET-001, HER-SEC-001, HER-ART-001, HER-ART-002, HER-SPL-001
#[test]
fn valid_details_pass() {
    let r = ruleset();
    let d = details(&r);
    assert_eq!(validate(&d, &r), Ok(()));
    // Positions are interchangeable: the order worn does not matter.
    let mut reversed = d.clone();
    reversed.worn.reverse();
    assert_eq!(validate(&reversed, &r), Ok(()));
    // No skills, artifacts or spells at all.
    let bare = HeroDetails {
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        ..d
    };
    assert_eq!(validate(&bare, &r), Ok(()));
}

/// A change to the valid details and the error it causes.
type Case = (fn(&Ruleset, &mut HeroDetails), DetailsError);

// covers: HER-DET-001, HER-SEC-001, HER-ART-001, HER-ART-002, HER-ART-003, HER-SPL-001
#[test]
fn invalid_details_are_rejected() {
    let r = ruleset();
    let unknown_art = ArtifactTypeId::from_raw(99);
    let unknown_skill = SkillId::from_raw(99);
    let unknown_spell = SpellId::from_raw(99);
    let cases: Vec<Case> = vec![
        (
            |_, d| d.hero_type = HeroTypeId::from_raw(99),
            DetailsError::UnknownHeroType,
        ),
        (|_, d| d.level = 0, DetailsError::LevelZero),
        (
            |_, d| d.primary[2] = 100,
            DetailsError::PrimaryAboveLimit(PrimarySkill::Power),
        ),
        (
            |r, d| d.skills = SKILLS.iter().map(|k| (skill(r, k), 1)).collect(),
            DetailsError::TooManySkills,
        ),
        (
            |_, d| d.skills.push((SkillId::from_raw(99), 1)),
            DetailsError::UnknownSkill(unknown_skill),
        ),
        (
            |r, d| d.skills.push((skill(r, "t:archery"), 2)),
            DetailsError::DuplicateSkill(SkillId::from_raw(0)),
        ),
        (
            |r, d| d.skills.push((skill(r, "t:wisdom"), 0)),
            DetailsError::SkillLevel(SkillId::from_raw(2)),
        ),
        (
            |r, d| d.skills.push((skill(r, "t:wisdom"), 4)),
            DetailsError::SkillLevel(SkillId::from_raw(2)),
        ),
        (
            |_, d| d.worn.push(ArtifactTypeId::from_raw(99).into()),
            DetailsError::UnknownArtifact(unknown_art),
        ),
        (
            |r, d| d.worn.push(art(r, "t:grail")),
            DetailsError::NotWearable(ArtifactTypeId::from_raw(8)),
        ),
        // The combination locks the head and the second ring.
        (
            |r, d| d.worn.push(art(r, "t:helm")),
            DetailsError::SlotFull(ArtifactSlotId::from_raw(0)),
        ),
        (
            |r, d| d.worn.push(art(r, "t:band")),
            DetailsError::SlotFull(ArtifactSlotId::from_raw(1)),
        ),
        (
            |r, d| d.worn.extend([art(r, "t:eye"), art(r, "t:eye")]),
            DetailsError::SlotFull(ArtifactSlotId::from_raw(2)),
        ),
        (
            |r, d| d.worn.push(art(r, "t:kit")),
            DetailsError::SlotFull(ArtifactSlotId::from_raw(3)),
        ),
        (
            |r, d| d.backpack.push(art(r, "t:eye")),
            DetailsError::BackpackFull,
        ),
        (
            |_, d| d.backpack[0] = ArtifactTypeId::from_raw(99).into(),
            DetailsError::UnknownArtifact(unknown_art),
        ),
        (
            |r, d| d.backpack[0] = art(r, "t:kit"),
            DetailsError::NotForBackpack(ArtifactTypeId::from_raw(5)),
        ),
        (
            |r, d| d.backpack[0] = art(r, "t:hook"),
            DetailsError::NotForBackpack(ArtifactTypeId::from_raw(6)),
        ),
        (
            |r, d| d.backpack[0] = art(r, "t:book"),
            DetailsError::NotForBackpack(ArtifactTypeId::from_raw(7)),
        ),
        (
            |_, d| d.spells.push(SpellId::from_raw(99)),
            DetailsError::UnknownSpell(unknown_spell),
        ),
        (
            |r, d| d.spells.push(spell_id(r, "t:portal")),
            DetailsError::DuplicateSpell(SpellId::from_raw(1)),
        ),
        (
            |r, d| d.spells.push(spell_id(r, "t:gaze")),
            DetailsError::NotAHeroSpell(SpellId::from_raw(2)),
        ),
        // A scroll holds a spell of the ruleset that is not an ability; nothing else holds one
        // (HER-ART-003), worn or carried.
        (
            |r, d| d.worn[1] = art(r, "t:scroll"),
            DetailsError::ScrollWithoutSpell(ArtifactTypeId::from_raw(9)),
        ),
        (
            |r, d| d.worn[1].spell = Some(spell_id(r, "t:bolt")),
            DetailsError::SpellOnArtifact(ArtifactTypeId::from_raw(4)),
        ),
        (
            |r, d| {
                d.worn[1] =
                    ArtifactInstance::scroll(art(r, "t:scroll").artifact, SpellId::from_raw(99));
            },
            DetailsError::UnknownSpell(unknown_spell),
        ),
        (
            |r, d| {
                d.worn[1] =
                    ArtifactInstance::scroll(art(r, "t:scroll").artifact, spell_id(r, "t:gaze"));
            },
            DetailsError::NotAHeroSpell(SpellId::from_raw(2)),
        ),
        (
            |r, d| d.backpack[0] = art(r, "t:scroll"),
            DetailsError::ScrollWithoutSpell(ArtifactTypeId::from_raw(9)),
        ),
    ];
    for (change, error) in cases {
        let mut d = details(&r);
        change(&r, &mut d);
        assert_eq!(validate(&d, &r), Err(error), "{error}");
    }
    // The handles the cases name are the ones of the test pack.
    assert_eq!(slot_id(&r, "t:machine"), ArtifactSlotId::from_raw(3));
    assert_eq!(art(&r, "t:grail").artifact, ArtifactTypeId::from_raw(8));
    assert_eq!(spell_id(&r, "t:gaze"), SpellId::from_raw(2));
    // A ruleset without hero parameters cannot validate.
    let mut p = pack();
    p.hero_params = None;
    let without = compile(p).unwrap();
    assert_eq!(
        validate(&details(&without), &without),
        Err(DetailsError::NoHeroParams)
    );
}

// covers: HER-WAR-001
#[test]
fn war_machines_come_from_the_worn_artifacts() {
    let r = ruleset();
    let ballista = r.creature_by_key("t:ballista").unwrap();
    let mut d = details(&r);
    assert_eq!(war_machines(&d, &r), [ballista]);
    // The machine slot's artifact in the backpack (invalid anyway) or an unknown artifact
    // brings nothing; an artifact without a war machine (the Catapult's kind) neither.
    d.worn = vec![ArtifactTypeId::from_raw(99).into(), art(&r, "t:hook")];
    d.backpack = vec![art(&r, "t:kit")];
    assert!(war_machines(&d, &r).is_empty());
}

#[test]
fn primary_skills_and_errors_have_names() {
    let names: Vec<(usize, &str)> = PrimarySkill::ALL
        .iter()
        .map(|s| (s.index(), s.name()))
        .collect();
    assert_eq!(
        names,
        [
            (0, "attack"),
            (1, "defense"),
            (2, "power"),
            (3, "knowledge")
        ]
    );
    let (art, slot) = (ArtifactTypeId::from_raw(7), ArtifactSlotId::from_raw(2));
    let (skill, spell) = (SkillId::from_raw(3), SpellId::from_raw(4));
    let cases = [
        (
            DetailsError::NoHeroParams,
            "the ruleset has no hero parameters",
        ),
        (DetailsError::UnknownHeroType, "unknown hero type"),
        (DetailsError::LevelZero, "the level must be at least 1"),
        (
            DetailsError::PrimaryAboveLimit(PrimarySkill::Defense),
            "defense above the ruleset's limit",
        ),
        (DetailsError::TooManySkills, "too many secondary skills"),
        (
            DetailsError::UnknownSkill(skill),
            "unknown secondary skill 3",
        ),
        (
            DetailsError::DuplicateSkill(skill),
            "secondary skill 3 listed twice",
        ),
        (
            DetailsError::SkillLevel(skill),
            "secondary skill 3 needs level 1..=3",
        ),
        (DetailsError::UnknownArtifact(art), "unknown artifact 7"),
        (DetailsError::NotWearable(art), "artifact 7 has no slot"),
        (
            DetailsError::SlotFull(slot),
            "artifact slot 2 has too few positions",
        ),
        (
            DetailsError::BackpackFull,
            "too many artifacts in the backpack",
        ),
        (
            DetailsError::NotForBackpack(art),
            "artifact 7 cannot be carried in the backpack",
        ),
        (DetailsError::UnknownSpell(spell), "unknown spell 4"),
        (DetailsError::DuplicateSpell(spell), "spell 4 listed twice"),
        (
            DetailsError::NotAHeroSpell(spell),
            "spell 4 is a creature ability",
        ),
        (
            DetailsError::ScrollWithoutSpell(art),
            "scroll 7 holds no spell",
        ),
        (
            DetailsError::SpellOnArtifact(art),
            "artifact 7 is no scroll and holds no spell",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}

/// Positions per slot that `worn` needs, counted independently of `validate`.
fn positions(r: &Ruleset, worn: &[ArtifactInstance]) -> Vec<usize> {
    r.artifact_slots()
        .iter()
        .enumerate()
        .map(|(i, _)| {
            worn.iter()
                .filter_map(|a| r.artifact(a.artifact))
                .map(|a| {
                    usize::from(a.slot.is_some_and(|s| s.index() == i))
                        + a.locks.iter().filter(|s| s.index() == i).count()
                })
                .sum()
        })
        .collect()
}

proptest! {
    // covers: HER-ART-001, HER-ART-002, HER-WAR-001
    #[test]
    fn validation_is_total_and_order_independent(
        // Mostly valid references and values, so that both outcomes are common.
        worn in proptest::collection::vec(0u16..10, 0..8),
        backpack in proptest::collection::vec(0u16..10, 0..5),
        skills in proptest::collection::vec((0u16..10, 1u8..=3), 0..3),
        spells in proptest::collection::vec(0u16..2, 0..2),
        level in 0u8..20,
        primary in proptest::array::uniform4(0u8..=100),
    ) {
        let r = ruleset();
        let d = HeroDetails {
            hero_type: HeroTypeId::from_raw(0),
            level,
            experience: 0,
            primary,
            skills: skills.iter().map(|&(s, l)| (SkillId::from_raw(s), l)).collect(),
            worn: worn.iter().map(|&a| ArtifactTypeId::from_raw(a).into()).collect(),
            backpack: backpack.iter().map(|&a| ArtifactTypeId::from_raw(a).into()).collect(),
            spells: spells.iter().map(|&s| SpellId::from_raw(s)).collect(),
            mana: 0,
        };
        let result = validate(&d, &r);
        let mut reversed = d.clone();
        reversed.worn.reverse();
        reversed.backpack.reverse();
        prop_assert_eq!(validate(&reversed, &r).is_ok(), result.is_ok());
        if result.is_ok() {
            let used = positions(&r, &d.worn);
            for (n, slot) in used.iter().zip(r.artifact_slots()) {
                prop_assert!(*n <= usize::from(slot.count));
            }
            prop_assert!(d.backpack.len() <= 3);
            // At most one war machine per machine slot.
            prop_assert!(war_machines(&d, &r).len() <= 1);
        }
    }
}

mod change;
mod derive;
