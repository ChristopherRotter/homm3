//! Tests of the rewards' hero changes (E08 T60a; increment 7's design note §1, §2): primary skills
//! gained, a skill raised to a level. The battle bonuses are the hero on the map's since E08 T115a
//! (`h3-adventure`'s tests).

use std::string::ToString;

use super::*;

/// `d` with the change applied, after checking it passes and keeps the details valid.
fn applied(r: &Ruleset, mut d: HeroDetails, change: HeroChange) -> HeroDetails {
    assert_eq!(check_change(&d, r, change), Ok(()), "{change:?}");
    apply_change(&mut d, r, change);
    assert_eq!(validate(&d, r), Ok(()), "{change:?}");
    d
}

/// The change refused with `error`; applying it changes nothing.
fn refused(r: &Ruleset, d: &HeroDetails, change: HeroChange, error: ChangeError) {
    assert_eq!(check_change(d, r, change), Err(error), "{change:?}");
    let mut after = d.clone();
    apply_change(&mut after, r, change);
    assert_eq!(&after, d, "{change:?}");
}

// covers: HER-CHG-001
#[test]
fn primary_skills_gained_saturate_at_the_limit() {
    let r = ruleset();
    let d = details(&r);
    // [99, 0, 1, 0] with the ruleset's most 99 (A-9's class: no wrap).
    let gained = applied(&r, d.clone(), HeroChange::GainPrimary([5, 2, 0, 200]));
    assert_eq!(gained.primary, [99, 2, 1, 99]);
    assert_eq!(
        applied(&r, d.clone(), HeroChange::GainPrimary([0; 4])),
        d,
        "nothing gained"
    );
}

// covers: HER-CHG-001
#[test]
fn a_skill_rises_to_its_level_and_never_falls() {
    let r = ruleset();
    let d = details(&r);
    let (archery, tactics, wisdom) = (
        skill(&r, "t:archery"),
        skill(&r, "t:tactics"),
        skill(&r, "t:wisdom"),
    );
    let raise = |skill, level| HeroChange::RaiseSkill { skill, level };
    // A known skill below the level rises to it in its place; one at or above it stays (D-37).
    let raised = applied(&r, d.clone(), raise(archery, 3));
    assert_eq!(raised.skills, [(archery, 3), (tactics, 3)]);
    assert_eq!(applied(&r, d.clone(), raise(tactics, 1)), d);
    assert_eq!(applied(&r, d.clone(), raise(archery, 1)), d);
    // A new one is learned at the level, after the others.
    let learned = applied(&r, d.clone(), raise(wisdom, 2));
    assert_eq!(learned.skills, [(archery, 1), (tactics, 3), (wisdom, 2)]);
    // Refused: a skill the ruleset lacks, a level not Basic to Expert.
    let unknown = SkillId::from_raw(99);
    refused(&r, &d, raise(unknown, 1), ChangeError::CannotLearn(unknown));
    for level in [0, 4] {
        refused(
            &r,
            &d,
            raise(wisdom, level),
            ChangeError::SkillLevel(wisdom),
        );
        refused(
            &r,
            &d,
            raise(archery, level),
            ChangeError::SkillLevel(archery),
        );
    }
    // With the most skills (eight) a new one is refused; a known one still rises.
    let mut full = d.clone();
    full.skills = SKILLS[..8].iter().map(|k| (skill(&r, k), 1)).collect();
    let new = skill(&r, SKILLS[8]);
    refused(&r, &full, raise(new, 1), ChangeError::CannotLearn(new));
    let raised = applied(&r, full, raise(archery, 2));
    assert_eq!(raised.skills[0], (archery, 2));
}

#[test]
fn the_new_change_errors_have_texts() {
    let texts = [(
        ChangeError::SkillLevel(SkillId::from_raw(3)),
        "secondary skill 3 cannot rise to that level",
    )];
    for (error, text) in texts {
        assert_eq!(error.to_string(), text);
    }
}
