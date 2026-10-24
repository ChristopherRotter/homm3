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
            offer_intervals: [6, 4],
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
            experience: vec![
                0, 1000, 2000, 3200, 4600, 6200, 8000, 10000, 12200, 14700, 17500, 20600,
            ],
            max_level: 74,
            wisdom: key("t:wisdom"),
            schools: vec![],
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

/// The experience of `level` (HER-EXP-002): details at a level carry at least it.
fn xp(r: &Ruleset, level: u8) -> u32 {
    experience_for(r.hero_params().unwrap(), level)
}

fn slot_id(r: &Ruleset, k: &str) -> ArtifactSlotId {
    r.artifact_slot_by_key(k).unwrap()
}

/// The test pack's positions by slot key: head 0, rings 1 and 2, misc 3 to 5, the machine 6, the
/// engine 7, the book 8.
const POSITIONS: [&str; 9] = [
    "t:head",
    "t:ring",
    "t:ring",
    "t:misc",
    "t:misc",
    "t:misc",
    "t:machine",
    "t:engine",
    "t:book",
];

/// Nothing worn: every position of the test pack empty.
fn bare(r: &Ruleset) -> Vec<Option<ArtifactInstance>> {
    wearing(r, &[])
}

/// Clears every position holding `item`.
fn take_off(worn: &mut [Option<ArtifactInstance>], item: ArtifactInstance) {
    worn.iter_mut()
        .filter(|a| **a == Some(item))
        .for_each(|a| *a = None);
}

/// Puts `item` on the first position that takes it (one must).
fn put_on(r: &Ruleset, worn: &mut [Option<ArtifactInstance>], item: ArtifactInstance) {
    let artifact = r.artifact(item.artifact).unwrap();
    let at = positions::first_free(worn, r, artifact).unwrap();
    worn[at.0] = Some(item);
}

/// Valid details that use every feature: the combination fills both rings and the head, two
/// copies of one misc artifact, every machine slot, the book, a backpack with an artifact
/// without slot, an adventure and a combat spell, and Knowledge 0 (below the minimum). The nine
/// positions ([`POSITIONS`]): the combination on ring 1 (head 0 and ring 2 locked), the eyes on
/// misc 3 and 4 (misc 5 free), the kit on 6, the hook on 7, the book on 8.
fn details(r: &Ruleset) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key("t:orrin").unwrap(),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [99, 0, 1, 0],
        skills: vec![(skill(r, "t:archery"), 1), (skill(r, "t:tactics"), 3)],
        worn: wearing(
            r,
            &[
                art(r, "t:circle"),
                art(r, "t:eye"),
                art(r, "t:eye"),
                art(r, "t:kit"),
                art(r, "t:hook"),
                art(r, "t:book"),
            ],
        ),
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
    let at = |i: usize| d.worn[i].map(|item| item.artifact);
    assert_eq!(
        (
            at(0),
            at(1),
            at(2),
            at(3),
            at(4),
            at(5),
            at(6),
            at(7),
            at(8)
        ),
        (
            None,
            Some(art(&r, "t:circle").artifact),
            None,
            Some(art(&r, "t:eye").artifact),
            Some(art(&r, "t:eye").artifact),
            None,
            Some(art(&r, "t:kit").artifact),
            Some(art(&r, "t:hook").artifact),
            Some(art(&r, "t:book").artifact),
        )
    );
    let slots: Vec<ArtifactSlotId> = POSITIONS.iter().map(|k| slot_id(&r, k)).collect();
    assert_eq!(positions(&r), slots);
    assert_eq!(Position(4).to_string(), "4");
    // Another free position of the same slot is as good (HER-ART-001 counts per slot).
    let mut moved = d.clone();
    moved.worn.swap(1, 2);
    moved.worn.swap(4, 5);
    assert_eq!(validate(&moved, &r), Ok(()));
    // No skills, artifacts or spells at all.
    let bare = HeroDetails {
        skills: vec![],
        worn: bare(&r),
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
        // One entry per position.
        (|_, d| _ = d.worn.pop(), DetailsError::Positions),
        (|_, d| d.worn.push(None), DetailsError::Positions),
        (
            |_, d| d.worn[5] = Some(ArtifactTypeId::from_raw(99).into()),
            DetailsError::UnknownArtifact(unknown_art),
        ),
        (
            |r, d| d.worn[5] = Some(art(r, "t:grail")),
            DetailsError::NotWearable(ArtifactTypeId::from_raw(8)),
        ),
        // An artifact on another slot's position, a free one or its own slot's taken.
        (
            |r, d| d.worn[5] = Some(art(r, "t:helm")),
            DetailsError::WrongPosition(ArtifactTypeId::from_raw(0)),
        ),
        (
            |r, d| d.worn[7] = Some(art(r, "t:kit")),
            DetailsError::WrongPosition(ArtifactTypeId::from_raw(5)),
        ),
        // The combination locks the head and the second ring: their positions are empty but
        // taken.
        (
            |r, d| d.worn[0] = Some(art(r, "t:helm")),
            DetailsError::SlotFull(ArtifactSlotId::from_raw(0)),
        ),
        (
            |r, d| d.worn[2] = Some(art(r, "t:band")),
            DetailsError::SlotFull(ArtifactSlotId::from_raw(1)),
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
            |r, d| d.worn[3] = Some(art(r, "t:scroll")),
            DetailsError::ScrollWithoutSpell(ArtifactTypeId::from_raw(9)),
        ),
        (
            |r, d| {
                d.worn[3] = Some(ArtifactInstance::scroll(
                    art(r, "t:eye").artifact,
                    spell_id(r, "t:bolt"),
                ))
            },
            DetailsError::SpellOnArtifact(ArtifactTypeId::from_raw(4)),
        ),
        (
            |r, d| {
                d.worn[3] = Some(ArtifactInstance::scroll(
                    art(r, "t:scroll").artifact,
                    SpellId::from_raw(99),
                ));
            },
            DetailsError::UnknownSpell(unknown_spell),
        ),
        (
            |r, d| {
                d.worn[3] = Some(ArtifactInstance::scroll(
                    art(r, "t:scroll").artifact,
                    spell_id(r, "t:gaze"),
                ));
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
    d.worn = bare(&r);
    d.worn[6] = Some(ArtifactTypeId::from_raw(99).into());
    d.worn[7] = Some(art(&r, "t:hook"));
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
        (
            DetailsError::Positions,
            "the worn artifacts do not match the positions",
        ),
        (DetailsError::NotWearable(art), "artifact 7 has no slot"),
        (
            DetailsError::WrongPosition(art),
            "artifact 7 is on another slot's position",
        ),
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
fn slot_use(r: &Ruleset, worn: &[Option<ArtifactInstance>]) -> Vec<usize> {
    r.artifact_slots()
        .iter()
        .enumerate()
        .map(|(i, _)| {
            worn.iter()
                .flatten()
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
    // covers: HER-ART-001, HER-ART-002, HER-WAR-001, HER-CHG-002
    #[test]
    fn validation_is_total_and_positions_of_a_slot_interchangeable(
        // Mostly valid references and values, so that both outcomes are common: a list placed
        // at first positions, then a few positions swapped.
        listed in proptest::collection::vec(0u16..12, 0..8),
        swaps in proptest::collection::vec((0usize..9, 0usize..9), 0..3),
        backpack in proptest::collection::vec(0u16..10, 0..5),
        skills in proptest::collection::vec((0u16..10, 1u8..=3), 0..3),
        spells in proptest::collection::vec(0u16..2, 0..2),
        level in 0u8..20,
        primary in proptest::array::uniform4(0u8..=100),
    ) {
        let r = ruleset();
        let items: Vec<ArtifactInstance> =
            listed.iter().map(|&a| ArtifactTypeId::from_raw(a).into()).collect();
        let mut worn = wearing(&r, &items);
        // A placed list fits: only a scroll without its spell fails it.
        let placed = HeroDetails { worn: worn.clone(), ..details(&r) };
        let fits = matches!(
            validate(&HeroDetails { backpack: vec![], ..placed }, &r),
            Ok(()) | Err(DetailsError::ScrollWithoutSpell(_))
        );
        prop_assert!(fits);
        for &(i, j) in &swaps {
            worn.swap(i, j);
        }
        let d = HeroDetails {
            hero_type: HeroTypeId::from_raw(0),
            level,
            experience: 0,
            offered: Default::default(),
            primary,
            skills: skills.iter().map(|&(s, l)| (SkillId::from_raw(s), l)).collect(),
            worn,
            backpack: backpack.iter().map(|&a| ArtifactTypeId::from_raw(a).into()).collect(),
            spells: spells.iter().map(|&s| SpellId::from_raw(s)).collect(),
            mana: 0,
        };
        let result = validate(&d, &r);
        let slots = positions(&r);
        // Positions of one slot are interchangeable.
        for (i, j) in [(1, 2), (3, 5)] {
            let mut swapped = d.clone();
            swapped.worn.swap(i, j);
            prop_assert_eq!(slots[i], slots[j]);
            prop_assert_eq!(validate(&swapped, &r).is_ok(), result.is_ok());
        }
        if result.is_ok() {
            let used = slot_use(&r, &d.worn);
            for (n, slot) in used.iter().zip(r.artifact_slots()) {
                prop_assert!(*n <= usize::from(slot.count));
            }
            for (item, &slot) in d.worn.iter().zip(&slots) {
                let own = item.and_then(|item| r.artifact(item.artifact)).and_then(|a| a.slot);
                prop_assert!(item.is_none() || own == Some(slot));
            }
            prop_assert!(d.backpack.len() <= 3);
            // At most one war machine per machine slot.
            prop_assert!(war_machines(&d, &r).len() <= 1);
        }
    }
}

mod artifacts;
mod change;
mod choice;
mod derive;
mod level;
mod movement;
mod outcome;
