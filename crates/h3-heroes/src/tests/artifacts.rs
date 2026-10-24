//! Tests of an artifact's moves, the combinations, Titan's Thunder's Spell Book and Scholar at a
//! meeting (E08 T49b; HER-ART-005 to -007, HER-SPL-007).

use std::collections::BTreeMap;
use std::string::ToString;
use std::vec;

use h3_content::{HeroEffectDef, SecondarySkillDef, SpellCategory, SpellDef};

use super::*;

/// The test pack with what the artifacts' rules read: `t:book` gives a spell book; `t:thunder`, a
/// misc artifact, adds one when equipped (`gives_spell_book`), as the combination `t:circle`
/// does; `t:well`, a misc combination of `t:eye` and `t:tome`; `t:prop`, an artifact of the
/// Ballista's slot that places nothing (it never moves); Wisdom adds 1/2/3 spell levels, the new
/// skill `t:scholar` teaches up to 2/3/4 at a meeting, as `t:satchel` (misc) up to 3; spells
/// `t:s2` to `t:s5` of levels 2 to 5.
fn artifacts_pack() -> ContentPack {
    let mut p = pack();
    for a in &mut p.artifacts {
        match a.key.to_string().as_str() {
            "t:book" => a.effects = vec![HeroEffect::SpellBook],
            "t:circle" => a.effects = vec![HeroEffect::GivesSpellBook],
            _ => {}
        }
    }
    let mut thunder = artifact("t:thunder", Some("t:misc"));
    thunder.effects = vec![HeroEffect::GivesSpellBook];
    let mut well = artifact("t:well", Some("t:misc"));
    well.parts = vec![key("t:eye"), key("t:tome")];
    let mut satchel = artifact("t:satchel", Some("t:misc"));
    satchel.effects = vec![HeroEffect::MeetingSpells { level: 3 }];
    p.artifacts.extend([
        thunder,
        well,
        artifact("t:prop", Some("t:machine")),
        satchel,
    ]);
    for s in &mut p.skills {
        if s.key.to_string() == "t:wisdom" {
            s.levels = [1, 2, 3].map(|levels| vec![HeroEffectDef::LearnLevel { levels }]);
        }
    }
    p.skills.push(SecondarySkillDef {
        key: key("t:scholar"),
        name: "Scholar".into(),
        levels: [2, 3, 4].map(|level| vec![HeroEffectDef::MeetingSpells { level }]),
    });
    for level in 2..=5 {
        p.spells.push(SpellDef {
            level,
            ..spell(&std::format!("t:s{level}"), SpellCategory::Combat)
        });
    }
    p
}

fn artifacts_ruleset() -> Ruleset {
    compile(artifacts_pack()).unwrap()
}

fn w(i: usize) -> ArtifactPlace {
    ArtifactPlace::Worn(Position(i))
}

fn b(i: usize) -> ArtifactPlace {
    ArtifactPlace::Backpack(i)
}

fn mv(from: ArtifactPlace, to: ArtifactPlace) -> ArtifactMove {
    ArtifactMove { from, to }
}

fn place(artifact: ArtifactInstance, at: ArtifactPlace) -> HeroChange {
    HeroChange::PlaceArtifact { artifact, at }
}

fn remove(at: ArtifactPlace) -> HeroChange {
    HeroChange::RemoveArtifact(at)
}

/// A hero for moves (the positions of [`POSITIONS`]): the circle on ring 1 (head 0 and ring 2
/// locked), the eye on misc 3, the tome on misc 4 (misc 5 free), the kit on 6, the hook on 7, the
/// book on 8; the backpack full with the helm, the Grail and the crown.
fn mover(r: &Ruleset) -> HeroDetails {
    let worn = ["t:circle", "t:eye", "t:tome", "t:kit", "t:hook", "t:book"].map(|k| art(r, k));
    HeroDetails {
        worn: wearing(r, &worn),
        backpack: ["t:helm", "t:grail", "t:crown"].map(|k| art(r, k)).to_vec(),
        ..details(r)
    }
}

/// A second hero: `worn` at their first positions, an empty backpack, no skills or spells.
fn other(r: &Ruleset, worn: &[&str]) -> HeroDetails {
    let worn: Vec<ArtifactInstance> = worn.iter().map(|k| art(r, k)).collect();
    HeroDetails {
        skills: vec![],
        worn: wearing(r, &worn),
        backpack: vec![],
        spells: vec![],
        ..details(r)
    }
}

/// Applies changes in order, each passing its check, and checks the result valid.
fn apply_all(r: &Ruleset, d: &mut HeroDetails, changes: &[HeroChange]) {
    for &change in changes {
        assert_eq!(check_change(d, r, change), Ok(()), "{change:?}");
        apply_change(d, r, change);
    }
    assert_eq!(validate(d, r), Ok(()));
}

/// The details after a move [`check_move`] passes, its changes applied change by change.
fn moved(
    r: &Ruleset,
    from: &HeroDetails,
    to: Option<&HeroDetails>,
    move_: ArtifactMove,
) -> (HeroDetails, Option<HeroDetails>) {
    assert_eq!(check_move(from, to, r, move_), Ok(()), "{move_:?}");
    let changes = move_changes(from, to, r, move_).unwrap();
    let mut a = from.clone();
    apply_all(r, &mut a, &changes.from);
    let b = to.cloned().map(|mut b| {
        apply_all(r, &mut b, &changes.to);
        b
    });
    (a, b)
}

/// The move refused with `error` by both functions.
fn refused(
    r: &Ruleset,
    from: &HeroDetails,
    to: Option<&HeroDetails>,
    move_: ArtifactMove,
    error: MoveError,
) {
    assert_eq!(check_move(from, to, r, move_), Err(error), "{move_:?}");
    assert_eq!(move_changes(from, to, r, move_), Err(error), "{move_:?}");
}

/// The artifact types of a position.
fn kind(d: &HeroDetails, i: usize) -> Option<ArtifactTypeId> {
    d.worn[i].map(|item| item.artifact)
}

// covers: HER-ART-005
#[test]
fn an_artifact_moves_within_one_hero() {
    let r = artifacts_ruleset();
    let d = mover(&r);
    assert_eq!(validate(&d, &r), Ok(()));
    let a = |k| art(&r, k);
    // To a free position of its slot.
    let changes = move_changes(&d, None, &r, mv(w(3), w(5))).unwrap();
    assert_eq!(
        changes,
        MoveChanges {
            from: vec![remove(w(3)), place(a("t:eye"), w(5))],
            to: vec![],
        }
    );
    let (after, _) = moved(&r, &d, None, mv(w(3), w(5)));
    assert_eq!(
        (kind(&after, 3), kind(&after, 5)),
        (None, Some(a("t:eye").artifact))
    );
    // Onto an occupied one: the two swap.
    let changes = move_changes(&d, None, &r, mv(w(3), w(4))).unwrap();
    assert_eq!(
        changes.from,
        [
            remove(w(3)),
            remove(w(4)),
            place(a("t:eye"), w(4)),
            place(a("t:tome"), w(3)),
        ]
    );
    let (after, _) = moved(&r, &d, None, mv(w(3), w(4)));
    assert_eq!(
        (kind(&after, 3), kind(&after, 4)),
        (Some(a("t:tome").artifact), Some(a("t:eye").artifact))
    );
    // The combination to the other ring: its locks are a count, so they follow.
    let (after, _) = moved(&r, &d, None, mv(w(1), w(2)));
    assert_eq!(
        (kind(&after, 1), kind(&after, 2)),
        (None, Some(a("t:circle").artifact))
    );
    // From the backpack onto an occupied position: the occupant goes to the backpack's place.
    let (after, _) = moved(&r, &d, None, mv(b(2), w(4)));
    assert_eq!(kind(&after, 4), Some(a("t:crown").artifact));
    assert_eq!(after.backpack, [a("t:helm"), a("t:grail"), a("t:tome")]);
    let (after, _) = moved(&r, &d, None, mv(b(2), w(5)));
    assert_eq!(after.backpack, [a("t:helm"), a("t:grail")]);
    // The backpack's arrows (batch E6's E6-10): the first to the end, the last to the front.
    let (after, _) = moved(&r, &d, None, mv(b(0), b(2)));
    assert_eq!(after.backpack, [a("t:grail"), a("t:crown"), a("t:helm")]);
    let (after, _) = moved(&r, &d, None, mv(b(2), b(0)));
    assert_eq!(after.backpack, [a("t:crown"), a("t:helm"), a("t:grail")]);
    // Into the backpack, at an index up to its length, while it holds fewer than its most.
    refused(&r, &d, None, mv(w(3), b(0)), MoveError::CannotPlace(b(0)));
    let mut room = d.clone();
    room.backpack.pop();
    let (after, _) = moved(&r, &room, None, mv(w(3), b(1)));
    assert_eq!(after.backpack, [a("t:helm"), a("t:eye"), a("t:grail")]);
    refused(
        &r,
        &room,
        None,
        mv(w(3), b(3)),
        MoveError::CannotPlace(b(3)),
    );
    refused(&r, &d, None, mv(b(0), b(3)), MoveError::CannotPlace(b(3)));
    // The combination in the backpack frees its locks: the helm goes on the head.
    let (freed, _) = moved(&r, &room, None, mv(w(1), b(0)));
    let (after, _) = moved(&r, &freed, None, mv(b(1), w(0)));
    assert_eq!(kind(&after, 0), Some(a("t:helm").artifact));
}

// covers: HER-ART-005
#[test]
fn what_never_moves_and_where_an_artifact_cannot_go() {
    let r = artifacts_ruleset();
    let d = mover(&r);
    let mut room = d.clone();
    room.backpack.pop();
    let id = |k| art(&r, k).artifact;
    // The spell book and the Catapult's slot's artifact never move.
    refused(
        &r,
        &room,
        None,
        mv(w(8), b(0)),
        MoveError::Unmovable(id("t:book")),
    );
    refused(
        &r,
        &d,
        None,
        mv(w(7), w(7)),
        MoveError::Unmovable(id("t:hook")),
    );
    let mut sling = d.clone();
    sling.worn[7] = Some(art(&r, "t:sling"));
    refused(
        &r,
        &sling,
        None,
        mv(w(7), w(6)),
        MoveError::Unmovable(id("t:sling")),
    );
    // Nothing at the source.
    for from in [w(5), w(99), b(3)] {
        refused(&r, &d, None, mv(from, w(5)), MoveError::NoArtifact(from));
    }
    // The same place.
    for place in [w(3), b(1)] {
        refused(&r, &d, None, mv(place, place), MoveError::SamePlace);
    }
    // A position of another slot, a locked one, one past the last.
    for to in [w(0), w(2), w(6), w(99)] {
        refused(&r, &d, None, mv(w(3), to), MoveError::CannotPlace(to));
    }
    refused(&r, &d, None, mv(b(0), w(0)), MoveError::CannotPlace(w(0)));
    // The Grail has no position; a war machine never goes to the backpack nor another slot.
    for to in [w(5), w(9)] {
        refused(&r, &d, None, mv(b(1), to), MoveError::CannotPlace(to));
    }
    refused(
        &r,
        &room,
        None,
        mv(w(6), b(0)),
        MoveError::CannotPlace(b(0)),
    );
    refused(&r, &d, None, mv(w(6), w(7)), MoveError::CannotPlace(w(7)));
}

// covers: HER-ART-005
#[test]
fn an_artifact_moves_between_two_heroes() {
    let r = artifacts_ruleset();
    let d = mover(&r);
    let a = |k| art(&r, k);
    let eyed = other(&r, &["t:eye"]);
    // To a free position: removed from one, placed on the other.
    let changes = move_changes(&d, Some(&eyed), &r, mv(w(4), w(4))).unwrap();
    assert_eq!(
        changes,
        MoveChanges {
            from: vec![remove(w(4))],
            to: vec![place(a("t:tome"), w(4))],
        }
    );
    // Onto an occupied one: the other's artifact comes back to the source's place.
    let changes = move_changes(&d, Some(&eyed), &r, mv(w(4), w(3))).unwrap();
    assert_eq!(
        changes,
        MoveChanges {
            from: vec![remove(w(4)), place(a("t:eye"), w(4))],
            to: vec![remove(w(3)), place(a("t:tome"), w(3))],
        }
    );
    let (mine, theirs) = moved(&r, &d, Some(&eyed), mv(w(4), w(3)));
    assert_eq!(kind(&mine, 4), Some(a("t:eye").artifact));
    assert_eq!(kind(&theirs.unwrap(), 3), Some(a("t:tome").artifact));
    // From a backpack onto an occupied position: the occupant into the source's backpack place.
    let (mine, theirs) = moved(&r, &d, Some(&eyed), mv(b(2), w(3)));
    assert_eq!(mine.backpack, [a("t:helm"), a("t:grail"), a("t:eye")]);
    assert_eq!(kind(&theirs.unwrap(), 3), Some(a("t:crown").artifact));
    // The receiver's locks: the circle on a hero wearing two rings does not fit.
    let ringed = other(&r, &["t:band", "t:loop", "t:prop"]);
    refused(
        &r,
        &d,
        Some(&ringed),
        mv(w(1), w(1)),
        MoveError::CannotPlace(w(1)),
    );
    // The mover's locks: the band for the circle, which cannot come back beside the loop.
    refused(
        &r,
        &ringed,
        Some(&d),
        mv(w(1), w(1)),
        MoveError::CannotReturn(w(1)),
    );
    // The same place of two heroes is a move.
    let (mine, theirs) = moved(&r, &d, Some(&eyed), mv(w(6), w(6)));
    assert_eq!(
        (kind(&mine, 6), kind(&theirs.unwrap(), 6)),
        (None, Some(a("t:kit").artifact))
    );
    // A war machine onto the other's machine: the same machine swaps, an unmovable one stays.
    let kitted = other(&r, &["t:kit"]);
    let (mine, theirs) = moved(&r, &d, Some(&kitted), mv(w(6), w(6)));
    assert_eq!(
        (mine.worn[6], theirs.unwrap().worn[6]),
        (Some(a("t:kit")), Some(a("t:kit")))
    );
    refused(
        &r,
        &d,
        Some(&ringed),
        mv(w(6), w(6)),
        MoveError::Unmovable(a("t:prop").artifact),
    );
    refused(
        &r,
        &d,
        Some(&eyed),
        mv(w(6), b(0)),
        MoveError::CannotPlace(b(0)),
    );
    // The Grail from backpack to backpack; a full backpack takes nothing.
    let changes = move_changes(&d, Some(&eyed), &r, mv(b(1), b(0))).unwrap();
    assert_eq!(changes.to, [place(a("t:grail"), b(0))]);
    refused(
        &r,
        &eyed,
        Some(&d),
        mv(w(3), b(3)),
        MoveError::CannotPlace(b(3)),
    );
}

// covers: HER-ART-007
#[test]
fn equipping_titans_thunder_adds_a_spell_book_that_stays() {
    let r = artifacts_ruleset();
    let a = |k| art(&r, k);
    let thunder = a("t:thunder");
    let bookless = other(&r, &["t:eye"]);
    // Received and worn: the book on the book position.
    let mut d = bookless.clone();
    apply_all(&r, &mut d, &[HeroChange::AddArtifact(thunder)]);
    assert_eq!((d.worn[4], d.worn[8]), (Some(thunder), Some(a("t:book"))));
    // Taken off: the book stays.
    apply_all(&r, &mut d, &[remove(w(4))]);
    assert_eq!(d.worn[8], Some(a("t:book")));
    // Placed at a position, moved onto one, from another hero: each equip adds it; a hero that
    // has one keeps it alone.
    let mut d = bookless.clone();
    apply_all(&r, &mut d, &[place(thunder, w(5))]);
    assert_eq!(d.worn[8], Some(a("t:book")));
    let mut carrier = other(&r, &[]);
    carrier.backpack = vec![thunder];
    let (_, theirs) = moved(&r, &carrier, Some(&bookless), mv(b(0), w(5)));
    assert_eq!(theirs.unwrap().worn[8], Some(a("t:book")));
    let (mine, _) = moved(&r, &carrier, None, mv(b(0), w(3)));
    assert_eq!(mine.worn[8], Some(a("t:book")));
    // Into the backpack, no book: not equipped.
    let mut full = other(&r, &["t:eye", "t:tome", "t:crown"]);
    apply_all(&r, &mut full, &[HeroChange::AddArtifact(thunder)]);
    assert_eq!(
        (full.backpack.as_slice(), full.worn[8]),
        (&[thunder][..], None)
    );
    let mut d = bookless.clone();
    apply_all(&r, &mut d, &[place(thunder, b(0))]);
    assert_eq!(d.worn[8], None);
    // Another artifact adds none; a ruleset without a spell book artifact neither.
    let mut d = bookless.clone();
    apply_all(&r, &mut d, &[HeroChange::AddArtifact(a("t:tome"))]);
    assert_eq!(d.worn[8], None);
    let mut p = artifacts_pack();
    p.artifacts
        .iter_mut()
        .filter(|a| a.key.to_string() == "t:book")
        .for_each(|a| a.effects.clear());
    let plain = compile(p).unwrap();
    let mut worn = wearing(&plain, &[]);
    assert!(equip(&mut worn, &plain, thunder, Position(3)));
    assert_eq!((worn[3], worn[8]), (Some(thunder), None));
    // `equip` at a position that refuses changes nothing.
    let mut worn = wearing(&r, &[]);
    assert!(!equip(&mut worn, &r, thunder, Position(0)));
    assert_eq!(worn, wearing(&r, &[]));
}

// covers: HER-ART-006
#[test]
fn a_combination_assembles_from_its_worn_parts() {
    let r = artifacts_ruleset();
    let a = |k| art(&r, k);
    // The band, the loop and the helm make the circle, at the first position that takes it.
    let d = other(&r, &["t:helm", "t:band", "t:loop"]);
    assert_eq!(
        check_assemble(&d, &r, Position(2)),
        Ok(a("t:circle").artifact)
    );
    let changes = assemble_changes(&d, &r, Position(2)).unwrap();
    assert_eq!(
        changes,
        [
            remove(w(2)),
            remove(w(1)),
            remove(w(0)),
            HeroChange::AddArtifact(a("t:circle")),
        ]
    );
    let mut after = d.clone();
    apply_all(&r, &mut after, &changes);
    assert_eq!(after.worn, wearing(&r, &[a("t:circle"), a("t:book")]));
    // One copy of each other part, from the highest position: the well from the tome and the
    // eye on misc 5, at the first misc position free (not the clicked one).
    let d = other(&r, &["t:eye", "t:tome", "t:eye"]);
    let changes = assemble_changes(&d, &r, Position(5)).unwrap();
    assert_eq!(
        changes,
        [
            remove(w(5)),
            remove(w(4)),
            HeroChange::AddArtifact(a("t:well"))
        ]
    );
    let mut after = d.clone();
    apply_all(&r, &mut after, &changes);
    assert_eq!(
        (kind(&after, 3), kind(&after, 4), kind(&after, 5)),
        (Some(a("t:eye").artifact), Some(a("t:well").artifact), None)
    );
    // A part missing, or only in the backpack; no part; nothing there.
    let mut short = other(&r, &["t:band", "t:loop"]);
    assert_eq!(
        check_assemble(&short, &r, Position(1)),
        Err(CombineError::Incomplete(Position(1)))
    );
    short.backpack = vec![a("t:helm")];
    assert_eq!(
        assemble_changes(&short, &r, Position(1)),
        Err(CombineError::Incomplete(Position(1)))
    );
    let d = mover(&r);
    assert_eq!(
        check_assemble(&d, &r, Position(6)),
        Err(CombineError::Incomplete(Position(6)))
    );
    assert_eq!(
        check_assemble(&d, &r, Position(5)),
        Err(CombineError::NoArtifact(Position(5)))
    );
}

// covers: HER-ART-006, HER-ART-007
#[test]
fn a_worn_combination_disassembles_and_the_book_stays() {
    let r = artifacts_ruleset();
    let a = |k| art(&r, k);
    // The circle (it gives a book) assembled on a hero without one, then taken apart: the parts
    // in artifact order (helm, band, loop) at their first positions; the book stays.
    let mut d = other(&r, &["t:helm", "t:band", "t:loop"]);
    d.worn[8] = None;
    let changes = assemble_changes(&d, &r, Position(1)).unwrap();
    apply_all(&r, &mut d, &changes);
    assert_eq!(
        (kind(&d, 1), d.worn[8]),
        (Some(a("t:circle").artifact), Some(a("t:book")))
    );
    assert_eq!(
        check_disassemble(&d, &r, Position(1)),
        Ok(a("t:circle").artifact)
    );
    let changes = disassemble_changes(&d, &r, Position(1)).unwrap();
    assert_eq!(
        changes,
        [
            remove(w(1)),
            HeroChange::AddArtifact(a("t:helm")),
            HeroChange::AddArtifact(a("t:band")),
            HeroChange::AddArtifact(a("t:loop")),
        ]
    );
    apply_all(&r, &mut d, &changes);
    assert_eq!(
        d.worn,
        wearing(&r, &[a("t:helm"), a("t:band"), a("t:loop"), a("t:book")])
    );
    // Not a combination; nothing there.
    assert_eq!(
        disassemble_changes(&d, &r, Position(1)),
        Err(CombineError::NotCombination(Position(1)))
    );
    assert_eq!(
        check_disassemble(&d, &r, Position(5)),
        Err(CombineError::NoArtifact(Position(5)))
    );
}

// covers: HER-ART-005, HER-ART-006
#[test]
fn the_errors_read_well() {
    let texts = [
        (
            MoveError::NoArtifact(w(3)).to_string(),
            "no artifact at position 3",
        ),
        (
            MoveError::Unmovable(ArtifactTypeId::from_raw(7)).to_string(),
            "artifact 7 never moves",
        ),
        (
            MoveError::SamePlace.to_string(),
            "the artifact is there already",
        ),
        (
            MoveError::CannotPlace(b(2)).to_string(),
            "the artifact cannot go in the backpack at 2",
        ),
        (
            MoveError::CannotReturn(w(1)).to_string(),
            "the target's artifact cannot go back at position 1",
        ),
        (
            CombineError::NoArtifact(Position(4)).to_string(),
            "no artifact worn at 4",
        ),
        (
            CombineError::Incomplete(Position(4)).to_string(),
            "the artifact at 4 completes no combination",
        ),
        (
            CombineError::NotCombination(Position(4)).to_string(),
            "the artifact at 4 is no combination",
        ),
    ];
    for (text, expected) in texts {
        assert_eq!(text, expected);
    }
}

/// A scholar hero: the skills at their levels, the book worn or not, these spells learned.
fn scholar(r: &Ruleset, skills: &[(&str, u8)], book: bool, spells: &[&str]) -> HeroDetails {
    let worn: &[&str] = if book { &["t:book"] } else { &[] };
    HeroDetails {
        skills: skills.iter().map(|&(k, l)| (skill(r, k), l)).collect(),
        spells: spells.iter().map(|k| spell_id(r, k)).collect(),
        ..other(r, worn)
    }
}

// covers: HER-SPL-007
#[test]
fn scholar_teaches_the_learned_spells_up_to_each_limit() {
    let r = artifacts_ruleset();
    let ids = |ks: &[&str]| -> Vec<SpellId> { ks.iter().map(|k| spell_id(&r, k)).collect() };
    // Batch E6's E6-7 in small: Expert Wisdom and Expert Scholar (limit 4) against no Wisdom
    // (limit 2); each learns what the other has learned and it has not, in spell order.
    let a = scholar(
        &r,
        &[("t:wisdom", 3), ("t:scholar", 3)],
        true,
        &["t:s3", "t:s2", "t:bolt"],
    );
    let b = scholar(&r, &[], true, &["t:s5", "t:s4", "t:portal", "t:bolt"]);
    assert_eq!(
        meeting_spells(&a, &b, &r),
        (ids(&["t:portal", "t:s4"]), ids(&["t:s2"]))
    );
    assert_eq!(
        meeting_spells(&b, &a, &r),
        (ids(&["t:s2"]), ids(&["t:portal", "t:s4"]))
    );
    // The greater level of the two: Basic Scholar (2) on one, Advanced (3) on the other.
    let c = scholar(&r, &[("t:wisdom", 3), ("t:scholar", 1)], true, &["t:bolt"]);
    let d = scholar(
        &r,
        &[("t:wisdom", 3), ("t:scholar", 2)],
        true,
        &["t:s3", "t:s4"],
    );
    assert_eq!(
        meeting_spells(&c, &d, &r),
        (ids(&["t:s3"]), ids(&["t:bolt"]))
    );
    // An artifact's level counts; the spells of artifacts and scrolls are not taught.
    let mut e = scholar(&r, &[("t:wisdom", 3)], true, &[]);
    apply_all(&r, &mut e, &[HeroChange::AddArtifact(art(&r, "t:satchel"))]);
    apply_all(&r, &mut e, &[HeroChange::AddArtifact(art(&r, "t:crown"))]);
    assert_eq!(meeting_spells(&e, &d, &r), (ids(&["t:s3"]), ids(&[])));
    // No Scholar, or either without a worn book: nothing.
    let plain = scholar(&r, &[("t:wisdom", 3)], true, &["t:s2"]);
    assert_eq!(meeting_spells(&plain, &b, &r), (vec![], vec![]));
    let bookless = scholar(&r, &[], false, &["t:s2"]);
    assert_eq!(meeting_spells(&a, &bookless, &r), (vec![], vec![]));
    assert_eq!(meeting_spells(&bookless, &a, &r), (vec![], vec![]));
}

/// A source or target place for the property: positions 0 to 9, backpack 0 to 4.
fn any_place() -> impl Strategy<Value = ArtifactPlace> {
    prop_oneof![(0usize..10).prop_map(w), (0usize..5).prop_map(b)]
}

/// How many of each artifact type two heroes hold, worn or carried, but the books.
fn holdings(r: &Ruleset, heroes: &[&HeroDetails]) -> BTreeMap<u16, usize> {
    let book = art(r, "t:book").artifact;
    let mut counts = BTreeMap::new();
    for d in heroes {
        let items = d.worn.iter().flatten().chain(&d.backpack);
        for item in items.filter(|i| i.artifact != book) {
            *counts.entry(item.artifact.raw()).or_insert(0) += 1;
        }
    }
    counts
}

proptest! {
    // covers: HER-ART-005
    #[test]
    fn a_checked_move_applies_change_by_change(
        from in any_place(),
        to in any_place(),
        two in any::<bool>(),
        room in any::<bool>(),
    ) {
        let r = artifacts_ruleset();
        let mut d = mover(&r);
        if room {
            d.backpack.pop();
        }
        let e = other(&r, &["t:band", "t:eye", "t:kit"]);
        let target = two.then_some(&e);
        let move_ = mv(from, to);
        match move_changes(&d, target, &r, move_) {
            Ok(changes) => {
                prop_assert_eq!(check_move(&d, target, &r, move_), Ok(()));
                let (mine, theirs) = moved(&r, &d, target, move_);
                let before: Vec<&HeroDetails> = [Some(&d), target].into_iter().flatten().collect();
                let after: Vec<&HeroDetails> =
                    [Some(&mine), theirs.as_ref()].into_iter().flatten().collect();
                prop_assert_eq!(holdings(&r, &before), holdings(&r, &after));
                prop_assert!(changes.from.len() + changes.to.len() >= 2);
            }
            Err(error) => prop_assert_eq!(check_move(&d, target, &r, move_), Err(error)),
        }
    }
}
