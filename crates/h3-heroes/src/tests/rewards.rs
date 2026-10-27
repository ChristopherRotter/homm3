//! Tests of the rewards' hero changes and the battle bonuses (E08 T60a; design §1, §2): primary
//! skills gained, a skill raised to a level, a bonus with its once-mark, the bonuses cleared.

use std::string::ToString;
use std::vec::Vec;

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

const fn bonus(morale: i8, luck: i8, mark: Option<u8>) -> HeroChange {
    HeroChange::Bonus { morale, luck, mark }
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

// covers: HER-ADV-008, HER-CHG-001
#[test]
fn bonuses_add_up_saturate_and_once_marks_gate_them() {
    let r = ruleset();
    let d = details(&r);
    assert_eq!(d.bonuses, BattleBonuses::default());
    // The Warrior's Tomb's −3 morale under its mark: a second Tomb adds nothing (E7-11).
    let tomb = applied(&r, d.clone(), bonus(-3, 0, Some(0)));
    assert_eq!(
        tomb.bonuses,
        BattleBonuses {
            morale: -3,
            luck: 0,
            marks: 0b1
        }
    );
    assert!(tomb.bonuses.has_mark(0) && !tomb.bonuses.has_mark(1));
    let again = applied(&r, tomb.clone(), bonus(-3, 0, Some(0)));
    assert_eq!(again, tomb);
    // A looted Pyramid's −2 luck under another mark, once (E7-21).
    let pyramid = applied(&r, again, bonus(0, -2, Some(1)));
    let pyramid = applied(&r, pyramid, bonus(0, -2, Some(1)));
    assert_eq!(
        pyramid.bonuses,
        BattleBonuses {
            morale: -3,
            luck: -2,
            marks: 0b11
        }
    );
    // A box's and an event's add up without a mark, saturating both ways (A-9's class).
    let boxes = applied(&r, pyramid, bonus(100, 127, None));
    let boxes = applied(&r, boxes, bonus(100, 127, None));
    assert_eq!((boxes.bonuses.morale, boxes.bonuses.luck), (127, 127));
    let low = applied(&r, d.clone(), bonus(-100, -100, None));
    let low = applied(&r, low, bonus(-100, -100, None));
    assert_eq!((low.bonuses.morale, low.bonuses.luck), (-128, -128));
    // A mark of no kind is refused; it is never set.
    refused(&r, &d, bonus(1, 1, Some(8)), ChangeError::Mark(8));
    assert!(
        !BattleBonuses {
            marks: 0xFF,
            ..BattleBonuses::default()
        }
        .has_mark(8)
    );
    // The battle's end clears them all.
    let cleared = applied(&r, boxes, HeroChange::ClearBonuses);
    assert_eq!(cleared.bonuses, BattleBonuses::default());
    // Any value is valid.
    let mut any = d;
    any.bonuses = BattleBonuses {
        morale: i8::MIN,
        luck: i8::MAX,
        marks: 0xFF,
    };
    assert_eq!(validate(&any, &r), Ok(()));
}

#[test]
fn the_new_change_errors_have_texts() {
    let texts = [
        (
            ChangeError::SkillLevel(SkillId::from_raw(3)),
            "secondary skill 3 cannot rise to that level",
        ),
        (ChangeError::Mark(9), "once-mark 9 is of no kind"),
    ];
    for (error, text) in texts {
        assert_eq!(error.to_string(), text);
    }
}
