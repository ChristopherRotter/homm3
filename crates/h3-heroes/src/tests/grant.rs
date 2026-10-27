//! Tests of a reward record's hero half and of what a teacher may teach (E08 T60b; design §3, §4):
//! `grant` in the original's order, whether a hero can receive an artifact under A-25's flag in
//! both profiles (`ADV-QUIRK-22`), and the teachers' functions.

use std::vec;
use std::vec::Vec;

use h3_content::{FULL_BACKPACK_REFUSES, HeroEffectDef};

use super::*;

/// The test pack with the teachers' skills and both profiles: Wisdom adds 1/2/3 spell levels,
/// Offense stands for Learning (5/10/15 %), Luck for Diplomacy (a join bonus of 1/2/3); `t:book`
/// gives a spell book; spells of levels 3 and 5; the class weighs archery 5, wisdom 3, logistics
/// 2 and luck 0 (the others absent). `t:faithful` has A-25's flag on, `t:strict` off.
fn reward_pack() -> ContentPack {
    let mut p = pack();
    for s in &mut p.skills {
        match s.key.to_string().as_str() {
            "t:wisdom" => {
                s.levels = [1, 2, 3].map(|levels| vec![HeroEffectDef::LearnLevel { levels }]);
            }
            "t:offense" => {
                s.levels = [500, 1000, 1500].map(|bp| vec![HeroEffectDef::ExperienceGain { bp }]);
            }
            "t:luck" => {
                s.levels = [1, 2, 3].map(|level| vec![HeroEffectDef::JoinBonus { level }]);
            }
            _ => {}
        }
    }
    for a in &mut p.artifacts {
        if a.key.to_string() == "t:book" {
            a.effects = vec![HeroEffect::SpellBook];
        }
    }
    p.spells.push(SpellDef {
        level: 3,
        ..spell("t:shield", SpellCategory::Combat)
    });
    p.spells.push(SpellDef {
        level: 5,
        ..spell("t:storm", SpellCategory::Combat)
    });
    p.hero_classes[0].skill_weights = [
        ("t:archery", 5),
        ("t:wisdom", 3),
        ("t:logistics", 2),
        ("t:luck", 0),
    ]
    .map(|(k, weight)| (key(k), weight))
    .into();
    p.profiles = vec![
        ProfileDef {
            key: key("t:faithful"),
            quirks: vec![("quirk.adventure.full_backpack_refuses".into(), true)],
        },
        ProfileDef {
            key: key("t:strict"),
            quirks: vec![],
        },
    ];
    p
}

/// The reward pack in `faithful` (A-25's flag on).
fn faithful() -> Ruleset {
    compile(reward_pack()).unwrap()
}

/// The reward pack in `strict` (A-25's flag off).
fn strict() -> Ruleset {
    Ruleset::compile(&[reward_pack()], "t:strict").unwrap()
}

/// `d` with every change of `granted` applied in order, each passing its check first; the
/// details stay valid.
fn applied(r: &Ruleset, mut d: HeroDetails, granted: &Granted) -> HeroDetails {
    for &change in &granted.changes {
        assert_eq!(check_change(&d, r, change), Ok(()), "{change:?}");
        apply_change(&mut d, r, change);
    }
    assert_eq!(validate(&d, r), Ok(()));
    d
}

/// The record that gives only `artifacts`.
fn artifacts(artifacts: Vec<ArtifactTypeId>) -> HeroReward {
    HeroReward {
        artifacts,
        ..HeroReward::default()
    }
}

// covers: HER-CHG-003
#[test]
fn a_record_is_granted_in_the_original_order() {
    let r = faithful();
    let mut d = details(&r);
    d.backpack.clear();
    d.mana = 100;
    let (wisdom, offense, shield, storm, bolt) = (
        skill(&r, "t:wisdom"),
        skill(&r, "t:offense"),
        spell_id(&r, "t:shield"),
        spell_id(&r, "t:storm"),
        spell_id(&r, "t:bolt"),
    );
    let eye = art(&r, "t:eye").artifact;
    let reward = HeroReward {
        experience: 1000,
        primary: [-5, 0, 3, 127],
        skills: vec![(offense, 3), (wisdom, 1)],
        mana: 500,
        morale: -1,
        luck: 2,
        artifacts: vec![eye],
        spells: vec![shield, storm, bolt],
    };
    let granted = grant(&d, &r, &reward);
    assert_eq!(
        granted.changes,
        [
            HeroChange::GainPrimary([0, 0, 3, 127]),
            HeroChange::RaiseSkill {
                skill: offense,
                level: 3
            },
            HeroChange::RaiseSkill {
                skill: wisdom,
                level: 1
            },
            HeroChange::SetSpellPoints(600),
            HeroChange::Bonus {
                morale: -1,
                luck: 2,
                mark: None
            },
            HeroChange::AddArtifact(eye.into()),
            HeroChange::LearnSpell(shield),
        ]
    );
    // D-39: the Expert Learning the record gives does not grow its own experience (E7-14), which
    // the game gives last: it is no change.
    assert_eq!(granted.experience, 1000);
    // Step 3's Wisdom counts for the spells (level 3 learned, level 5 not); a known one is
    // skipped.
    assert_eq!(granted.skipped, [storm, bolt]);
    assert!(granted.lost.is_empty());
    let after = applied(&r, d.clone(), &granted);
    assert_eq!(after.primary, [99, 0, 4, 99]);
    assert_eq!(after.mana, 600, "E7-13: above the maximum");
    assert_eq!(after.experience, d.experience);
    // An empty record grants nothing.
    assert_eq!(grant(&d, &r, &HeroReward::default()), Granted::default());
}

// covers: HER-CHG-003
#[test]
fn the_experience_is_signed_and_counts_the_learning_known_before() {
    let r = faithful();
    let mut d = details(&r);
    let offense = skill(&r, "t:offense");
    let experience = |d: &HeroDetails, amount| {
        let reward = HeroReward {
            experience: amount,
            ..HeroReward::default()
        };
        let granted = grant(d, &r, &reward);
        assert!(granted.changes.is_empty());
        granted.experience
    };
    assert_eq!(experience(&d, 0), 0);
    assert_eq!(experience(&d, 0x7FFF_FFFF), 0x7FFF_FFFF);
    // The map's u32 read as signed: 2³¹ and above give none.
    assert_eq!(experience(&d, 0x8000_0000), 0);
    assert_eq!(experience(&d, u32::MAX), 0);
    // Learning known before the record grows it.
    d.skills.push((offense, 1));
    assert_eq!(experience(&d, 1000), 1050);
}

// covers: HER-CHG-003
#[test]
fn skills_rise_never_fall_and_a_new_one_needs_room() {
    let r = faithful();
    let mut d = details(&r);
    let (archery, tactics, wisdom) = (
        skill(&r, "t:archery"),
        skill(&r, "t:tactics"),
        skill(&r, "t:wisdom"),
    );
    let raised = |d: &HeroDetails, skills: Vec<(SkillId, u8)>| {
        let reward = HeroReward {
            skills,
            ..HeroReward::default()
        };
        grant(d, &r, &reward).changes
    };
    // D-37 (E7-12): Basic + Advanced gives Advanced; Expert + Basic stays Expert.
    let raise = |skill, level| HeroChange::RaiseSkill { skill, level };
    assert_eq!(raised(&d, vec![(archery, 2)]), [raise(archery, 2)]);
    assert!(raised(&d, vec![(tactics, 1)]).is_empty());
    // A pair the ruleset refuses gives nothing; a skill twice rises once.
    let unknown = SkillId::from_raw(99);
    assert!(raised(&d, vec![(unknown, 1), (wisdom, 0), (wisdom, 4)]).is_empty());
    assert_eq!(
        raised(&d, vec![(wisdom, 2), (wisdom, 1)]),
        [raise(wisdom, 2)]
    );
    // Eight skills: a new one is not learned, a known one still rises.
    d.skills = SKILLS[..8].iter().map(|k| (skill(&r, k), 1)).collect();
    let new = skill(&r, SKILLS[8]);
    assert_eq!(
        raised(&d, vec![(new, 1), (archery, 3)]),
        [raise(archery, 3)]
    );
}

// covers: HER-CHG-003
#[test]
fn the_mana_is_held_between_zero_and_the_ceiling() {
    let r = faithful();
    let mut d = details(&r);
    let mut points = |mana, delta| {
        d.mana = mana;
        let reward = HeroReward {
            mana: delta,
            ..HeroReward::default()
        };
        grant(&d, &r, &reward).changes
    };
    let set = HeroChange::SetSpellPoints;
    assert_eq!(points(900, 500), [set(999)]);
    // D-38: a hero above the ceiling falls to it; a loss stops at 0.
    assert_eq!(points(u16::MAX, 1), [set(999)]);
    assert_eq!(points(100, -500), [set(0)]);
    assert_eq!(points(100, i32::MIN), [set(0)]);
    assert!(points(u16::MAX, 0).is_empty(), "no mana: no change");
    // Without the hero parameters the ceiling is 0.
    let mut p = reward_pack();
    p.hero_params = None;
    let bare = compile(p).unwrap();
    let reward = HeroReward {
        mana: 5,
        ..HeroReward::default()
    };
    assert_eq!(grant(&details(&r), &bare, &reward).changes, [set(0)]);
}

// covers: HER-CHG-003
#[test]
fn the_bonuses_and_primary_skills_are_given_only_when_set() {
    let r = faithful();
    let d = details(&r);
    let only = |reward: HeroReward| grant(&d, &r, &reward).changes;
    assert!(
        only(HeroReward {
            primary: [-1, 0, -128, 0],
            ..HeroReward::default()
        })
        .is_empty()
    );
    assert_eq!(
        only(HeroReward {
            morale: 0,
            luck: -3,
            ..HeroReward::default()
        }),
        [HeroChange::Bonus {
            morale: 0,
            luck: -3,
            mark: None
        }]
    );
}

// covers: HER-CHG-003
#[test]
fn a_spell_needs_the_book_the_record_may_give() {
    let r = faithful();
    let mut d = details(&r);
    let book = art(&r, "t:book");
    take_off(&mut d.worn, book);
    d.backpack.clear();
    d.spells.clear();
    let bolt = spell_id(&r, "t:bolt");
    let reward = HeroReward {
        artifacts: vec![book.artifact],
        spells: vec![bolt],
        ..HeroReward::default()
    };
    // Step 6's book counts for step 7.
    let granted = grant(&d, &r, &reward);
    assert_eq!(
        granted.changes,
        [HeroChange::AddArtifact(book), HeroChange::LearnSpell(bolt)]
    );
    applied(&r, d.clone(), &granted);
    // Without it the spell is skipped; an artifact the ruleset refuses as received (a scroll
    // without its spell, one it lacks) is lost.
    let scroll = art(&r, "t:scroll").artifact;
    let unknown = ArtifactTypeId::from_raw(99);
    let reward = HeroReward {
        artifacts: vec![scroll, unknown],
        spells: vec![bolt],
        ..HeroReward::default()
    };
    let granted = grant(&d, &r, &reward);
    assert!(granted.changes.is_empty());
    assert_eq!(granted.lost, [scroll, unknown]);
    assert_eq!(granted.skipped, [bolt]);
}

// covers: HER-CHG-003
#[test]
fn a_full_backpack_refuses_an_artifact_in_faithful_only() {
    // ADV-QUIRK-22 (A-25, E7-5, E7-6, E7-10, E7-19): the test pack's backpack holds its most, 3,
    // and misc position 5 is free.
    let (faithful, strict) = (faithful(), strict());
    for r in [&faithful, &strict] {
        let flag = r.quirks().is_enabled(FULL_BACKPACK_REFUSES);
        let d = details(r);
        let (eye, helm, kit, scroll) = (
            art(r, "t:eye"),
            art(r, "t:helm"),
            art(r, "t:kit"),
            ArtifactInstance::scroll(art(r, "t:scroll").artifact, spell_id(r, "t:bolt")),
        );
        // A free position: received only with the flag off.
        assert_eq!(can_receive_artifact(&d, r, eye), !flag);
        assert_eq!(can_receive_artifact(&d, r, scroll), !flag);
        // No free position (the head locked by the combination) and a full backpack: neither.
        assert!(!can_receive_artifact(&d, r, helm));
        let granted = grant(&d, r, &artifacts(vec![eye.artifact, helm.artifact]));
        if flag {
            assert!(granted.changes.is_empty());
            assert_eq!(granted.lost, [eye.artifact, helm.artifact]);
        } else {
            assert_eq!(granted.changes, [HeroChange::AddArtifact(eye)]);
            assert_eq!(granted.lost, [helm.artifact]);
            assert_eq!(applied(r, d.clone(), &granted).worn[5], Some(eye));
        }
        // Below the most both profiles place it; a war machine whose position is taken never.
        let mut room = d.clone();
        room.backpack.pop();
        assert!(can_receive_artifact(&room, r, eye));
        assert!(can_receive_artifact(&room, r, helm));
        assert!(!can_receive_artifact(&room, r, kit));
        // Each received artifact counts for the next: the second helm finds the backpack full.
        let granted = grant(&room, r, &artifacts(vec![helm.artifact, helm.artifact]));
        assert_eq!(granted.changes, [HeroChange::AddArtifact(helm)]);
        assert_eq!(granted.lost, [helm.artifact]);
    }
}

// covers: HER-CHG-004
#[test]
fn a_teachers_spell_needs_a_book_and_the_level_and_may_ignore_known_ones() {
    let r = faithful();
    let mut d = details(&r);
    let (bolt, shield, gaze) = (
        spell_id(&r, "t:bolt"),
        spell_id(&r, "t:shield"),
        spell_id(&r, "t:gaze"),
    );
    // Known: refused when it counts (a Shrine, a Scholar, a box), passed when not (the Pyramid).
    assert!(!can_learn_spell(&d, &r, bolt, true));
    assert!(can_learn_spell(&d, &r, bolt, false));
    assert!(!learns_spell(&d, &r, bolt));
    // Level 3 needs Basic Wisdom; an ability and a spell the ruleset lacks never.
    assert!(!can_learn_spell(&d, &r, shield, false));
    d.skills.push((skill(&r, "t:wisdom"), 1));
    assert!(can_learn_spell(&d, &r, shield, true));
    assert!(!can_learn_spell(&d, &r, gaze, false));
    assert!(!can_learn_spell(&d, &r, SpellId::from_raw(99), false));
    // No spell book worn: nothing.
    take_off(&mut d.worn, art(&r, "t:book"));
    assert!(!can_learn_spell(&d, &r, shield, true));
    assert!(!can_learn_spell(&d, &r, bolt, false));
}

// covers: HER-CHG-004
#[test]
fn a_witch_hut_and_a_university_teach_by_their_tests() {
    let r = faithful();
    let mut d = details(&r);
    let k = |name| skill(&r, name);
    let unknown = SkillId::from_raw(99);
    // The hut: not known, below the most skills; no class test.
    assert!(!hut_teaches(&d, &r, k("t:archery")));
    assert!(hut_teaches(&d, &r, k("t:luck")));
    assert!(hut_teaches(&d, &r, k("t:first_aid")));
    assert!(!hut_teaches(&d, &r, unknown));
    // The University: known, barred by the class's weight 0 (or none), else open.
    assert_eq!(university_slot(&d, &r, k("t:archery")), SlotState::Known);
    assert_eq!(university_slot(&d, &r, k("t:wisdom")), SlotState::Open);
    for barred in [k("t:luck"), k("t:first_aid"), unknown] {
        assert_eq!(university_slot(&d, &r, barred), SlotState::Barred);
    }
    // Eight skills: no room for a new one, before the class's test.
    d.skills = SKILLS[..8].iter().map(|s| (k(s), 1)).collect();
    assert!(!hut_teaches(&d, &r, k("t:logistics")));
    assert_eq!(university_slot(&d, &r, k("t:logistics")), SlotState::NoRoom);
    assert_eq!(university_slot(&d, &r, k("t:first_aid")), SlotState::NoRoom);
    assert_eq!(university_slot(&d, &r, k("t:wisdom")), SlotState::Known);
    // A hero type the ruleset lacks has no class: every new skill barred.
    let mut stranger = details(&r);
    stranger.hero_type = HeroTypeId::from_raw(9);
    assert_eq!(
        university_slot(&stranger, &r, k("t:wisdom")),
        SlotState::Barred
    );
}

// covers: HER-CHG-004
#[test]
fn a_scholar_teaches_its_gift_or_its_fallback() {
    let r = faithful();
    let mut d = details(&r);
    d.spells.clear();
    let k = |name| skill(&r, name);
    let fallback = PrimarySkill::Knowledge;
    let reward = |d: &HeroDetails, gift| scholar_reward(d, &r, gift, fallback);
    let plus = HeroChange::GainPrimary([0, 0, 0, 1]);
    let raise = |skill, level| HeroChange::RaiseSkill { skill, level };
    // A primary skill: +1 to it.
    assert_eq!(
        reward(&d, ScholarGift::Primary(PrimarySkill::Defense)),
        HeroChange::GainPrimary([0, 1, 0, 0])
    );
    // A skill: a known one a level up below Expert, a new one at Basic; else the fallback.
    assert_eq!(
        reward(&d, ScholarGift::Skill(k("t:archery"))),
        raise(k("t:archery"), 2)
    );
    assert_eq!(reward(&d, ScholarGift::Skill(k("t:tactics"))), plus);
    assert_eq!(
        reward(&d, ScholarGift::Skill(k("t:luck"))),
        raise(k("t:luck"), 1)
    );
    assert_eq!(reward(&d, ScholarGift::Skill(SkillId::from_raw(99))), plus);
    // A spell it can learn, else the fallback (the level, a known one).
    let (bolt, shield) = (spell_id(&r, "t:bolt"), spell_id(&r, "t:shield"));
    assert_eq!(
        reward(&d, ScholarGift::Spell(bolt)),
        HeroChange::LearnSpell(bolt)
    );
    assert_eq!(reward(&d, ScholarGift::Spell(shield)), plus);
    d.spells = vec![bolt];
    assert_eq!(reward(&d, ScholarGift::Spell(bolt)), plus);
    // Eight skills: a new one gives the fallback, a known one still rises.
    d.skills = SKILLS[..8].iter().map(|s| (k(s), 1)).collect();
    assert_eq!(reward(&d, ScholarGift::Skill(k("t:logistics"))), plus);
    assert_eq!(
        reward(&d, ScholarGift::Skill(k("t:wisdom"))),
        raise(k("t:wisdom"), 2)
    );
}

// covers: HER-CHG-004
#[test]
fn the_library_admits_by_level_and_diplomacy() {
    let r = faithful();
    let mut d = details(&r);
    let diplomacy = skill(&r, "t:luck");
    // Level 10, or 8, 6, 4 with Basic, Advanced, Expert Diplomacy.
    for (skill_level, admitted) in [(0, 10), (1, 8), (2, 6), (3, 4)] {
        d.skills.retain(|&(s, _)| s != diplomacy);
        if skill_level > 0 {
            d.skills.push((diplomacy, skill_level));
        }
        for level in [admitted - 1, admitted] {
            d.level = level;
            assert_eq!(library_admits(&d, &r, 10), level == admitted, "{d:?}");
        }
    }
}

// covers: HER-CHG-004, HER-EXP-001
#[test]
fn a_teachers_experience_grows_by_learning_and_the_tree_gives_one_level() {
    let r = faithful();
    let mut d = details(&r);
    let offense = skill(&r, "t:offense");
    // The Learning Stone's 1 000 and a chest's 1 500 − 500, by the Learning factor.
    assert_eq!(learning_experience(&d, &r, 1000), 1000);
    d.skills.push((offense, 3));
    assert_eq!(learning_experience(&d, &r, 1500 - 500), 1150);
    // The Tree: the gap to the next level's threshold, without Learning (1 000, 1 000, 1 200).
    let params = r.hero_params().unwrap();
    for (level, gap) in [(1, 1000), (2, 1000), (3, 1200)] {
        d.level = level;
        d.experience = xp(&r, level) + 500;
        let gap_given = next_level_experience(&d, &r);
        assert_eq!(gap_given, gap);
        assert_eq!(
            level_of(params, d.experience + gap_given),
            level + 1,
            "one level"
        );
    }
    // The highest level: the gap to the bound's level; without the hero parameters none.
    d.level = params.max_level;
    let top = experience_for(params, params.max_level);
    assert_eq!(
        next_level_experience(&d, &r),
        experience_for(params, params.max_level + 1) - top
    );
    let mut p = reward_pack();
    p.hero_params = None;
    assert_eq!(next_level_experience(&d, &compile(p).unwrap()), 0);
}
