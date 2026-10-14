//! Tests of the hero changes (E08 T19a2a): their check and application, where a received artifact
//! goes, what a defeated hero yields, and the artifact a destroyed war machine takes.

use std::string::ToString;
use std::vec;
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
    // Experience adds, saturating; spell points are set.
    let gained = applied(&r, d.clone(), HeroChange::GainExperience(70));
    assert_eq!(gained.experience, 70);
    let gained = applied(&r, gained, HeroChange::GainExperience(u32::MAX));
    assert_eq!(gained.experience, u32::MAX);
    assert_eq!(
        applied(&r, d.clone(), HeroChange::SetSpellPoints(12)).mana,
        12
    );
    // A removed artifact must be there; the later ones move up.
    let worn = applied(
        &r,
        d.clone(),
        HeroChange::RemoveArtifact(ArtifactPlace::Worn(1)),
    );
    assert_eq!(worn.worn, [&d.worn[..1], &d.worn[2..]].concat());
    let carried = applied(
        &r,
        d.clone(),
        HeroChange::RemoveArtifact(ArtifactPlace::Backpack(0)),
    );
    assert_eq!(carried.backpack, d.backpack[1..]);
    for place in [ArtifactPlace::Worn(6), ArtifactPlace::Backpack(3)] {
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
        applied(&r, roomy, HeroChange::AddArtifact(bolt))
            .worn
            .last(),
        Some(&bolt)
    );
}

// covers: HER-CHG-002
#[test]
fn a_received_artifact_is_worn_if_it_fits_else_carried() {
    let r = ruleset();
    let d = details(&r);
    let eye = art(&r, "t:eye");
    // Two of three misc positions are taken: a third eye is worn; a fourth finds the backpack
    // full (three of three).
    let d = applied(&r, d, HeroChange::AddArtifact(eye));
    assert_eq!(d.worn.last(), Some(&eye));
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
    bare.worn = vec![art(&r, "t:helm")];
    let circle = art(&r, "t:circle");
    let carried = applied(&r, bare.clone(), HeroChange::AddArtifact(circle));
    assert_eq!(carried.backpack.last(), Some(&circle));
    bare.worn.clear();
    let worn = applied(&r, bare, HeroChange::AddArtifact(circle));
    assert_eq!(worn.worn, [circle]);
}

// covers: HER-ART-004
#[test]
fn a_defeated_hero_yields_its_worn_artifacts_then_its_backpack_from_the_end() {
    let r = ruleset();
    let mut d = details(&r);
    // An unknown artifact (which validation refuses) is never yielded.
    d.worn.insert(0, ArtifactTypeId::from_raw(99).into());
    let mut yielded = Vec::new();
    while let Some((place, item)) = next_capture(&d, &r) {
        yielded.push((place, item));
        apply_change(&mut d, &r, HeroChange::RemoveArtifact(place));
    }
    let worn = |k| (ArtifactPlace::Worn(1), art(&r, k));
    let carried = |i, k| (ArtifactPlace::Backpack(i), art(&r, k));
    assert_eq!(
        yielded,
        [
            worn("t:circle"),
            worn("t:eye"),
            worn("t:eye"),
            carried(2, "t:eye"),
            carried(1, "t:grail"),
            carried(0, "t:helm"),
        ]
    );
    // The war machines, the engine and the book stay.
    let kept = ["t:kit", "t:hook", "t:book"].map(|k| art(&r, k));
    assert_eq!(d.worn[1..], kept);
}

// covers: HER-WAR-002
#[test]
fn a_destroyed_war_machine_takes_its_artifact_but_the_catapult() {
    let r = ruleset();
    let machine = |k| r.creature_by_key(k).unwrap();
    let mut d = details(&r);
    assert_eq!(
        lost_war_machine(&d, &r, machine("t:ballista")),
        Some(ArtifactPlace::Worn(3))
    );
    // No artifact places it; the Catapult stays.
    assert_eq!(lost_war_machine(&d, &r, machine("t:catapult")), None);
    d.worn[4] = art(&r, "t:sling");
    assert_eq!(validate(&d, &r), Ok(()));
    assert_eq!(lost_war_machine(&d, &r, machine("t:catapult")), None);
    d.worn.remove(3);
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
            ChangeError::NoArtifact(ArtifactPlace::Worn(2)),
            "no artifact worn at 2",
        ),
        (
            ChangeError::NoArtifact(ArtifactPlace::Backpack(5)),
            "no artifact in the backpack at 5",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}
