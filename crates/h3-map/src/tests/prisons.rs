//! The experiment writer's Prison and Market of Time (E08 T138: batch E15's maps; increment 15's
//! design note `docs/epics/E08-design.md` §7, survey `docs/rules/surveys/E08-spells.md` §1): a
//! prison (class 62) with its prisoner's body, a placed hero's without an owner, read back as
//! written in SoD and RoE, its type left out of the map's allowed heroes; the Market of Time
//! (class 50) without a body, the one shipped map's template; and their import until T141a types
//! them (a prisoner's type in the setup's prisoners, each object a blocker).

use super::import::{ROE, SOD};
use super::limits;
use crate::model::{Body, RawMap};
use crate::writer as w;
use crate::{import, read};

/// Red's prisoner-to-be: Gelu (hero type 155 is past RoE's 128, so the RoE map takes Orrin, 0),
/// with every part of a placed hero's body (spells and primary skills in SoD alone).
fn prisoner(hero_type: u8, sod: bool) -> w::Hero {
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[0] = Some(7); // Centaur's Axe: in the head slot, the writer checks no slot
    worn[17] = Some(0); // a spell book
    w::Hero {
        tile: (0, 0),
        owner: 3,
        hero_type,
        class: 17,
        name: Some("Freed".into()),
        experience: Some(5000),
        skills: Some(vec![(1, 2), (13, 3)]),
        army: Some([Some((1, 12)), None, Some((13, 3)), None, None, None, None]),
        tight_formation: true,
        artifacts: Some(w::HeroArtifacts {
            worn,
            backpack: vec![9],
        }),
        spells: sod.then(|| {
            let mut bits = [0u8; 9];
            bits[0] = 0b0110; // Scuttle Boat (1), Summon Boat (2)
            bits[1] = 0b1000; // Dimension Door (8)
            bits
        }),
        primary: sod.then_some([2, 3, 4, 5]),
    }
}

/// A 36 × 36 grass map, Red and Blue playing: Red's hero at (2, 30), a prison at (10, 10) and a
/// Market of Time at (20, 10).
fn prisons_map(hero_type: u8, sod: bool) -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    w::Map {
        name: "Prisons".into(),
        description: "E08 T138".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![],
        roads: vec![],
        players,
        objects: vec![
            super::economy::hero((2, 30), 0, 1),
            w::MapObject::Prison(w::Prison {
                tile: (10, 10),
                prisoner: prisoner(hero_type, sod),
            }),
            w::MapObject::Plain(w::Plain {
                tile: (20, 10),
                template: w::ObjectTemplate::market_of_time(),
            }),
        ],
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: Vec::new(),
        banned_spells: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

fn read_back(bytes: &[u8]) -> RawMap {
    read(&w::gzip(bytes), &limits()).unwrap()
}

fn template_of(m: &RawMap, index: usize) -> &crate::model::Template {
    &m.templates[usize::try_from(m.objects[index].template).unwrap()]
}

#[test]
fn a_prison_reads_back_with_its_prisoner_s_body() {
    for (format, hero_type, sod) in [(w::Format::Sod, 155, true), (w::Format::RoE, 0, false)] {
        let m = read_back(&w::write_in(&prisons_map(hero_type, sod), format).unwrap());
        let prison = &m.objects[1];
        assert_eq!((prison.pos.x, prison.pos.y, prison.pos.z), (10, 10, 0));
        let t = template_of(&m, 1);
        assert_eq!((t.class, t.subclass), (62, 0), "{format:?}");
        assert_eq!(t.def.as_bytes(), b"AVXprsn0.def");
        // Objects.txt's line: the anchor and the tile west of it blocked, the anchor visited.
        assert_eq!((t.passable[5], t.visitable[5]), (0x3f, 0x80));
        assert!(t.passable[..5].iter().all(|&b| b == 0xff));
        assert!(t.visitable[..5].iter().all(|&b| b == 0));
        let Body::Hero(body) = &prison.body else {
            panic!("not a hero's body: {:?}", prison.body);
        };
        // No owner, whatever the prisoner's record says; the body as a placed hero's.
        assert_eq!(body.owner, 0xff);
        assert_eq!(body.hero_type, hero_type);
        assert_eq!(
            body.name.as_ref().map(|n| n.as_bytes()),
            Some(&b"Freed"[..])
        );
        assert_eq!(body.experience, Some(5000));
        assert_eq!(body.secondary_skills, Some(vec![(1, 2), (13, 3)]));
        let army = body.army.as_ref().unwrap();
        assert_eq!(army[0].map(|s| (s.creature, s.count)), Some((1, 12)));
        assert_eq!(army[2].map(|s| (s.creature, s.count)), Some((13, 3)));
        assert_eq!(body.formation, 1);
        let artifacts = body.artifacts.as_ref().unwrap();
        assert_eq!((artifacts.worn[0], artifacts.worn[17]), (Some(7), Some(0)));
        assert_eq!(artifacts.backpack, [Some(9)]);
        assert_eq!(body.patrol, 0xff);
        if sod {
            assert_eq!(
                body.spells.as_ref().map(|b| b.0.clone()),
                Some(vec![6, 8, 0, 0, 0, 0, 0, 0, 0])
            );
            assert_eq!(body.primary_skills, Some([2, 3, 4, 5]));
        } else {
            assert_eq!((&body.spells, body.primary_skills), (&None, None));
        }
        // The prisoner's type is not in the map's pool (the editor's maps; survey §1); a placed
        // hero's still is.
        assert!(!m.allowed_heroes.get(usize::from(hero_type)), "{format:?}");
        assert!(m.allowed_heroes.get(1));
        // The prisoner is no player's hero: Red's main hero is its placed one, Blue has none.
        let main = |p: usize| {
            let details = m.players[p].details.as_ref().unwrap();
            details.main_hero.as_ref().map(|h| h.hero_type)
        };
        assert_eq!((main(0), main(1)), (Some(1), None), "{format:?}");
    }
}

#[test]
fn the_writer_checks_a_prisoner_s_body_as_a_placed_hero_s() {
    let refused = |edit: fn(&mut w::Hero), format| {
        let mut map = prisons_map(0, format == w::Format::Sod);
        let w::MapObject::Prison(prison) = &mut map.objects[1] else {
            unreachable!()
        };
        edit(&mut prison.prisoner);
        w::write_in(&map, format)
    };
    let out_of_range = |what, value| Err(w::WriteError::OutOfRange { what, value });
    // Its owner and tile are not written: a player that does not play is no refusal.
    assert!(refused(|h| h.owner = 7, w::Format::Sod).is_ok());
    assert_eq!(
        refused(|h| h.hero_type = 156, w::Format::Sod),
        out_of_range("hero type", 156)
    );
    assert_eq!(
        refused(|h| h.skills = Some(vec![(28, 1)]), w::Format::Sod),
        out_of_range("secondary skill", 28)
    );
    assert_eq!(
        refused(|h| h.spells = Some([0; 9]), w::Format::RoE),
        out_of_range("hero spells in an RoE map", 0)
    );
}

#[test]
fn the_market_of_time_reads_back_without_a_body() {
    assert!(w::PLAIN_OBJECTS.contains(&w::MARKET_OF_TIME_OBJECT));
    for (format, sod) in [(w::Format::Sod, true), (w::Format::RoE, false)] {
        let m = read_back(&w::write_in(&prisons_map(0, sod), format).unwrap());
        let market = &m.objects[2];
        assert_eq!((market.pos.x, market.pos.y), (20, 10));
        assert_eq!(market.body, Body::None);
        // `Titans Winter`'s template (survey §1.2): the anchor and the tile west of it blocked,
        // the anchor entered.
        let t = template_of(&m, 2);
        assert_eq!((t.class, t.subclass), (50, 0));
        assert_eq!(t.def.as_bytes(), b"AVXmktt0.def");
        assert_eq!(
            (t.passable[5], t.visitable[5], t.terrains),
            (0x3f, 0x80, 0x1ff)
        );
    }
    let mut map = prisons_map(0, true);
    let w::MapObject::Plain(market) = &mut map.objects[2] else {
        unreachable!()
    };
    market.template.subclass = 1;
    assert_eq!(
        w::write(&map),
        Err(w::WriteError::OutOfRange {
            what: "Market of Time subclass",
            value: 1
        })
    );
}

#[test]
fn the_import_takes_the_prisoner_s_type_and_both_objects_as_blockers_until_t141a() {
    for (format, rules, hero_type, sod) in [
        (w::Format::Sod, &*SOD, 155, true),
        (w::Format::RoE, &*ROE, 0, false),
    ] {
        let raw = read_back(&w::write_in(&prisons_map(hero_type, sod), format).unwrap());
        let setup = import(&raw, rules).unwrap();
        setup.validate(rules).unwrap();
        let core = rules.core();
        let prisoner = core.hero_type_by_key(import::codes::HERO_TYPES[usize::from(hero_type)]);
        assert_eq!(
            setup.prisoners,
            prisoner.into_iter().collect::<Vec<_>>(),
            "{format:?}"
        );
        assert!(!setup.allowed_heroes.contains(&prisoner.unwrap()));
        // Both untyped (no kind yet: T141a), each blocking its two tiles.
        assert_eq!(setup.map.objects.len(), 2);
        for object in &setup.map.objects {
            assert_eq!(object.object_type, None);
        }
    }
}
