use super::*;
use alloc::string::ToString;
use alloc::vec;

/// The lists' art of the base table (design §4.5).
pub(crate) fn art() -> ListArt {
    let text = include_str!("../../../../map/assets/base.toml");
    crate::tables::parse(text).unwrap().map.unwrap().lists
}

fn hero(portrait: u8, points: i32, mana: i32) -> KingdomHero {
    KingdomHero {
        asleep: false,
        movable: true,
        portrait,
        points,
        mana,
    }
}

#[test]
fn the_movement_bar_by_the_points_left() {
    // hero_movement_frame 0x4E5330: 0 at or below 0, a frame per 100 points below 2 300, then 23,
    // 24 from 2 500, 25 from 2 800 (VCMI 1.7.5: points / 100 up to 25).
    let bar = art().movement;
    let frames: Vec<u32> = [
        -5, 0, 1, 99, 100, 1500, 2299, 2300, 2499, 2500, 2799, 2800, 9999,
    ]
    .into_iter()
    .map(|v| bar.frame(v))
    .collect();
    assert_eq!(frames, [0, 0, 0, 0, 1, 15, 22, 23, 23, 24, 24, 25, 25]);
    assert_eq!(bar.frames(), 26);
}

#[test]
fn the_mana_bar_by_the_mana() {
    // hero_mana_frame 0x4E5380: (mana + 4) / 5 below 116, 23 to 144, 24 to 169, then 25 (VCMI
    // 1.7.5: mana / 5 up to 25).
    let bar = art().mana;
    let frames: Vec<u32> = [0, 1, 5, 6, 10, 114, 115, 116, 144, 145, 169, 170, 500]
        .into_iter()
        .map(|v| bar.frame(v))
        .collect();
    assert_eq!(frames, [0, 1, 1, 2, 2, 23, 23, 23, 23, 24, 24, 25, 25]);
    assert_eq!(bar.frames(), 26);
}

#[test]
fn a_town_s_picture_by_its_faction_fort_and_day() {
    // town_list_frame 0x5BD700 with 1: 2 + 2 x faction with a fort, 20 + 2 x faction without, one
    // more when it built today.
    let towns = art().towns;
    let frame = |faction, fort, built| {
        towns.frame(KingdomTown {
            faction,
            fort,
            built,
        })
    };
    assert_eq!(frame(0, true, false), 2);
    assert_eq!(frame(0, true, true), 3);
    assert_eq!(frame(1, false, false), 22);
    assert_eq!(frame(8, true, true), 19);
    assert_eq!(frame(8, false, true), 37);
    assert_eq!((towns.empty, towns.selected), (0, 1));
    assert_eq!(towns.frames(9), 38);
    assert_eq!(towns.frames(0), 2);
}

#[test]
fn the_tables_lists_checked() {
    let good = art();
    assert_eq!(good.check(), Ok(()));
    assert_eq!(good.portraits.len(), 163);
    let broken = |change: &dyn Fn(&mut ListArt)| {
        let mut art = good.clone();
        change(&mut art);
        art.check().unwrap_err()
    };
    assert_eq!(broken(&|a| a.portraits.clear()), "no portraits");
    assert!(broken(&|a| a.movement.step = 0).starts_with("movement: a step of 0"));
    assert!(broken(&|a| a.mana.round = 5).starts_with("mana: a step of 5 rounded by 5"));
    assert!(broken(&|a| a.mana.round = -1).starts_with("mana: a step"));
    for top in [vec![], vec![0, 5], vec![5, 5], vec![9, 5]] {
        let error = broken(&|a| a.movement.top = top.clone());
        assert!(error.contains("not ascending"), "{top:?}: {error}");
    }
    assert_eq!(
        broken(&|a| a.movement.from = 21),
        "movement: the frame below 2300 passes `from` (21)"
    );
}

#[test]
fn the_hero_list_s_slots_from_its_top() {
    // hero_list_slot 0x403560: the heroes from the list's first on, then empty slots.
    let art = art();
    let mut kingdom = Kingdom {
        heroes: vec![hero(0, 2800, 0), hero(162, 1500, 5), hero(7, 0, 116)],
        ..Kingdom::default()
    };
    let slots = hero_slots(&art, &kingdom).unwrap();
    let shown: Vec<(Option<&str>, u32, u32)> = slots
        .iter()
        .map(|s| (s.picture.as_deref(), s.movement, s.mana))
        .collect();
    assert_eq!(
        shown,
        [
            (Some("HPS000Kn.PCX"), 25, 0),
            (Some("HPS136Wi.PCX"), 15, 1),
            (Some("HPS007Kn.PCX"), 0, 23),
            (None, 0, 0),
            (None, 0, 0),
        ]
    );
    kingdom.tops[0] = 2;
    let slots = hero_slots(&art, &kingdom).unwrap();
    assert_eq!(slots[0].picture.as_deref(), Some("HPS007Kn.PCX"));
    assert_eq!(slots[1].picture, None);
    // A top past every index: empty slots.
    kingdom.tops[0] = usize::MAX;
    assert!(hero_slots(&art, &kingdom).unwrap()[0].picture.is_none());
    // A portrait past the table.
    kingdom.tops[0] = 0;
    kingdom.heroes[1].portrait = 163;
    assert_eq!(hero_slots(&art, &kingdom), None);
}

#[test]
fn the_town_list_s_slots_from_its_top() {
    let art = art();
    let town = |faction, fort| KingdomTown {
        faction,
        fort,
        built: false,
    };
    let mut kingdom = Kingdom {
        towns: vec![town(0, true), town(5, false), town(8, true)],
        ..Kingdom::default()
    };
    assert_eq!(town_slots(&art, &kingdom), [2, 30, 18, 0, 0]);
    kingdom.tops[1] = 1;
    assert_eq!(town_slots(&art, &kingdom), [30, 18, 0, 0, 0]);
    kingdom.tops[1] = usize::MAX;
    assert_eq!(town_slots(&art, &kingdom), [0; SLOTS]);
}

#[test]
fn the_selection_s_slot() {
    let mut kingdom = Kingdom {
        heroes: vec![hero(0, 0, 0); 8],
        towns: vec![KingdomTown::default(); 7],
        tops: [3, 2],
        ..Kingdom::default()
    };
    let slot = |kingdom: &Kingdom| selected_slot(kingdom);
    assert_eq!(slot(&kingdom), None);
    kingdom.selected = Selected::Hero(7);
    assert_eq!(slot(&kingdom), Some((Selected::Hero(7), 4)));
    // Above or below the slots shown: none.
    kingdom.selected = Selected::Hero(2);
    assert_eq!(slot(&kingdom), None);
    kingdom.tops[0] = 0;
    kingdom.selected = Selected::Hero(5);
    assert_eq!(slot(&kingdom), None);
    kingdom.selected = Selected::Town(2);
    assert_eq!(slot(&kingdom), Some((Selected::Town(2), 0)));
    kingdom.selected = Selected::Town(1);
    assert_eq!(slot(&kingdom), None);
    assert_eq!("HPS000Kn.PCX".to_string(), art().portraits[0]);
}
