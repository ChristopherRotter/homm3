//! Tests of the hero changes (E08 T19a2a): their check and application, where a received artifact
//! goes, what a defeated hero yields, and the artifact a destroyed war machine takes; since E08
//! T49a by position, the first position that takes an artifact, and an artifact placed.

use std::string::ToString;
use std::vec::Vec;

use super::*;

/// `details(r)` with the change applied, after checking it passes and keeps the details valid.
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
fn changes_are_checked_and_applied() {
    let r = ruleset();
    let d = details(&r);
    // Experience adds, at most the bound (HER-EXP-002); spell points are set.
    let gained = applied(&r, d.clone(), HeroChange::GainExperience(70));
    assert_eq!(gained.experience, 70);
    let gained = applied(&r, gained, HeroChange::GainExperience(u32::MAX));
    assert_eq!(gained.experience, 1_810_034_206);
    assert_eq!(
        applied(&r, d.clone(), HeroChange::SetSpellPoints(12)).mana,
        12
    );
    // A removed artifact must be there: its position is left empty, the backpack's later ones
    // move up.
    let worn = applied(
        &r,
        d.clone(),
        HeroChange::RemoveArtifact(ArtifactPlace::Worn(Position(1))),
    );
    let mut left = d.worn.clone();
    left[1] = None;
    assert_eq!(worn.worn, left);
    let carried = applied(
        &r,
        d.clone(),
        HeroChange::RemoveArtifact(ArtifactPlace::Backpack(0)),
    );
    assert_eq!(carried.backpack, d.backpack[1..]);
    for place in [
        ArtifactPlace::Worn(Position(5)),
        ArtifactPlace::Worn(Position(9)),
        ArtifactPlace::Backpack(3),
    ] {
        let change = HeroChange::RemoveArtifact(place);
        refused(&r, &d, change, ChangeError::NoArtifact(place));
    }
    // A received artifact must be one of the ruleset, holding a spell exactly if it is a scroll.
    let mut roomy = d.clone();
    roomy.backpack.clear();
    let unknown = ArtifactTypeId::from_raw(99);
    let error = ChangeError::Artifact(DetailsError::UnknownArtifact(unknown));
    refused(&r, &roomy, HeroChange::AddArtifact(unknown.into()), error);
    let scroll = art(&r, "t:scroll");
    let error = ChangeError::Artifact(DetailsError::ScrollWithoutSpell(scroll.artifact));
    refused(&r, &roomy, HeroChange::AddArtifact(scroll), error);
    let bolt = ArtifactInstance::scroll(scroll.artifact, spell_id(&r, "t:bolt"));
    assert_eq!(
        applied(&r, roomy, HeroChange::AddArtifact(bolt)).worn[5],
        Some(bolt)
    );
}

// covers: HER-CHG-002
#[test]
fn a_received_artifact_is_worn_if_it_fits_else_carried() {
    let r = ruleset();
    let d = details(&r);
    let eye = art(&r, "t:eye");
    // Two of three misc positions are taken: a third eye is worn on the third (5); a fourth finds
    // the backpack full (three of three).
    let d = applied(&r, d, HeroChange::AddArtifact(eye));
    assert_eq!(d.worn[5], Some(eye));
    refused(
        &r,
        &d,
        HeroChange::AddArtifact(eye),
        ChangeError::NoRoom(eye.artifact),
    );
    // With room in the backpack it goes to its end.
    let mut d = d;
    d.backpack.remove(0);
    let carried = applied(&r, d.clone(), HeroChange::AddArtifact(eye));
    assert_eq!(carried.worn, d.worn);
    assert_eq!(carried.backpack.last(), Some(&eye));
    // A ring finds both rings taken (the combination locks the second), the Grail has no slot:
    // both are carried. A war machine is never carried.
    for k in ["t:band", "t:grail"] {
        let carried = applied(&r, d.clone(), HeroChange::AddArtifact(art(&r, k)));
        assert_eq!(carried.backpack.last(), Some(&art(&r, k)), "{k}");
    }
    let kit = art(&r, "t:kit");
    refused(
        &r,
        &d,
        HeroChange::AddArtifact(kit),
        ChangeError::NoRoom(kit.artifact),
    );
    // A combination is worn only if the slots it locks are free too.
    let mut bare = d.clone();
    bare.worn = super::bare(&r);
    bare.worn[0] = Some(art(&r, "t:helm"));
    let circle = art(&r, "t:circle");
    let carried = applied(&r, bare.clone(), HeroChange::AddArtifact(circle));
    assert_eq!(carried.backpack.last(), Some(&circle));
    bare.worn[0] = None;
    let worn = applied(&r, bare.clone(), HeroChange::AddArtifact(circle));
    assert_eq!(worn.worn[1], Some(circle));
    // The lowest empty position of the slot: with the second ring taken, the first.
    bare.worn[2] = Some(art(&r, "t:band"));
    let looped = applied(&r, bare, HeroChange::AddArtifact(art(&r, "t:loop")));
    assert_eq!(looped.worn[1], Some(art(&r, "t:loop")));
}

// covers: HER-CHG-002, HER-ART-001
#[test]
fn the_first_position_that_takes_an_artifact() {
    let r = ruleset();
    let d = details(&r);
    let first = |k| first_position(&d, &r, art(&r, k).artifact);
    // The third misc position is free; the head and the second ring are locked by the
    // combination; the Grail has no slot; the machine's position is taken.
    assert_eq!(first("t:eye"), Some(Position(5)));
    for k in ["t:helm", "t:band", "t:grail", "t:kit"] {
        assert_eq!(first(k), None, "{k}");
    }
    assert_eq!(first_position(&d, &r, ArtifactTypeId::from_raw(99)), None);
    // A given position takes an artifact when it is empty, of its slot and the locks fit.
    let eye = art(&r, "t:eye").artifact;
    assert!(takes(&d.worn, &r, eye, Position(5)));
    assert!(!takes(&d.worn, &r, eye, Position(4)));
    assert!(!takes(&d.worn, &r, art(&r, "t:helm").artifact, Position(0)));
    assert!(!takes(
        &d.worn,
        &r,
        ArtifactTypeId::from_raw(99),
        Position(5)
    ));
    // A list placed: each at its first position in list order, one that none takes left out
    // (the fourth eye, the combination after the helm, an unknown artifact).
    let [eye, helm, circle] = ["t:eye", "t:helm", "t:circle"].map(|k| art(&r, k));
    let unknown = ArtifactTypeId::from_raw(99).into();
    let worn = wearing(&r, &[eye, helm, eye, unknown, eye, eye, circle]);
    let mut expected = bare(&r);
    expected[0] = Some(helm);
    expected[3..6].copy_from_slice(&[Some(eye); 3]);
    assert_eq!(worn, expected);
}

// covers: HER-CHG-001, HER-ART-001, HER-ART-002
#[test]
fn an_artifact_is_placed_at_a_given_place() {
    let r = ruleset();
    let d = details(&r);
    let [eye, helm, band, grail, kit] =
        ["t:eye", "t:helm", "t:band", "t:grail", "t:kit"].map(|k| art(&r, k));
    let place = |artifact, at| HeroChange::PlaceArtifact { artifact, at };
    let worn = |i| ArtifactPlace::Worn(Position(i));
    // An empty position of its slot.
    assert_eq!(
        applied(&r, d.clone(), place(eye, worn(5))).worn[5],
        Some(eye)
    );
    // Taken, another slot's, locked by the combination, past the positions, no slot.
    for (artifact, at) in [
        (eye, worn(3)),
        (eye, worn(0)),
        (helm, worn(0)),
        (band, worn(2)),
        (eye, worn(9)),
        (grail, worn(5)),
    ] {
        let change = place(artifact, at);
        refused(&r, &d, change, ChangeError::CannotPlace(at));
    }
    let unknown = ArtifactTypeId::from_raw(99);
    let error = ChangeError::Artifact(DetailsError::UnknownArtifact(unknown));
    refused(&r, &d, place(unknown.into(), worn(5)), error);
    // The backpack: full (three of three); with room, inserted at an index up to its length, never
    // a war machine.
    let full = ArtifactPlace::Backpack(0);
    refused(&r, &d, place(eye, full), ChangeError::CannotPlace(full));
    let mut roomy = d.clone();
    roomy.backpack.remove(2);
    let inserted = applied(&r, roomy.clone(), place(eye, ArtifactPlace::Backpack(1)));
    assert_eq!(inserted.backpack, [helm, eye, grail]);
    let appended = applied(&r, roomy.clone(), place(grail, ArtifactPlace::Backpack(2)));
    assert_eq!(appended.backpack, [helm, grail, grail]);
    for (artifact, at) in [(eye, 3), (kit, 0)] {
        let at = ArtifactPlace::Backpack(at);
        refused(
            &r,
            &roomy,
            place(artifact, at),
            ChangeError::CannotPlace(at),
        );
    }
}

// covers: HER-ART-004
#[test]
fn a_defeated_hero_yields_its_worn_artifacts_then_its_backpack_from_the_end() {
    let r = ruleset();
    let mut d = details(&r);
    // An unknown artifact (which validation refuses) is never yielded.
    d.worn[5] = Some(ArtifactTypeId::from_raw(99).into());
    let mut yielded = Vec::new();
    while let Some((place, item)) = next_capture(&d, &r) {
        yielded.push((place, item));
        apply_change(&mut d, &r, HeroChange::RemoveArtifact(place));
    }
    let worn = |i, k| (ArtifactPlace::Worn(Position(i)), art(&r, k));
    let carried = |i, k| (ArtifactPlace::Backpack(i), art(&r, k));
    // In position order: the combination on the first ring, the eyes on misc 3 and 4.
    assert_eq!(
        yielded,
        [
            worn(1, "t:circle"),
            worn(3, "t:eye"),
            worn(4, "t:eye"),
            carried(2, "t:eye"),
            carried(1, "t:grail"),
            carried(0, "t:helm"),
        ]
    );
    // The war machines, the engine and the book stay.
    let kept = ["t:kit", "t:hook", "t:book"].map(|k| Some(art(&r, k)));
    assert_eq!(d.worn[6..], kept);
    assert!(d.worn[..5].iter().all(Option::is_none));
}

// covers: HER-WAR-002
#[test]
fn a_destroyed_war_machine_takes_its_artifact_but_the_catapult() {
    let r = ruleset();
    let machine = |k| r.creature_by_key(k).unwrap();
    let mut d = details(&r);
    assert_eq!(
        lost_war_machine(&d, &r, machine("t:ballista")),
        Some(ArtifactPlace::Worn(Position(6)))
    );
    // No artifact places it; the Catapult stays.
    assert_eq!(lost_war_machine(&d, &r, machine("t:catapult")), None);
    d.worn[7] = Some(art(&r, "t:sling"));
    assert_eq!(validate(&d, &r), Ok(()));
    assert_eq!(lost_war_machine(&d, &r, machine("t:catapult")), None);
    d.worn[6] = None;
    assert_eq!(lost_war_machine(&d, &r, machine("t:ballista")), None);
}

#[test]
fn change_errors_read_well() {
    let r = ruleset();
    let eye = art(&r, "t:eye").artifact;
    let cases = [
        (
            ChangeError::Artifact(DetailsError::UnknownArtifact(eye)),
            "the artifact does not fit the ruleset: unknown artifact 4",
        ),
        (ChangeError::NoRoom(eye), "no room for artifact 4"),
        (
            ChangeError::NoArtifact(ArtifactPlace::Worn(Position(2))),
            "no artifact worn at 2",
        ),
        (
            ChangeError::NoArtifact(ArtifactPlace::Backpack(5)),
            "no artifact in the backpack at 5",
        ),
        (
            ChangeError::CannotPlace(ArtifactPlace::Worn(Position(3))),
            "the artifact cannot be worn at 3",
        ),
        (
            ChangeError::CannotPlace(ArtifactPlace::Backpack(4)),
            "the artifact cannot go into the backpack at 4",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}

// covers: HER-CHG-001
#[test]
fn a_learned_spell_must_be_a_new_hero_spell() {
    // E08 T19a2b: Eagle Eye's picks (HER-SPL-006).
    let r = ruleset();
    let d = details(&r);
    let titan = spell_id(&r, "t:titan");
    let learned = applied(&r, d.clone(), HeroChange::LearnSpell(titan));
    assert_eq!(learned.spells, [&d.spells[..], &[titan]].concat());
    let bolt = spell_id(&r, "t:bolt");
    refused(
        &r,
        &d,
        HeroChange::LearnSpell(bolt),
        ChangeError::KnownSpell(bolt),
    );
    let gaze = spell_id(&r, "t:gaze");
    let error = ChangeError::Spell(DetailsError::NotAHeroSpell(gaze));
    refused(&r, &d, HeroChange::LearnSpell(gaze), error);
    let unknown = SpellId::from_raw(99);
    let error = ChangeError::Spell(DetailsError::UnknownSpell(unknown));
    refused(&r, &d, HeroChange::LearnSpell(unknown), error);
    assert_eq!(
        ChangeError::KnownSpell(bolt).to_string(),
        "spell 0 is learned already"
    );
    assert!(
        error
            .to_string()
            .starts_with("the spell does not fit the ruleset: ")
    );
}
