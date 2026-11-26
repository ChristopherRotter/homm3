use super::*;

#[test]
fn the_same_seed_draws_the_same() {
    let mut a = Picks::new(7);
    let mut b = Picks::new(7);
    let drawn: Vec<usize> = (0..20).map(|_| a.below(7)).collect();
    assert_eq!(drawn, (0..20).map(|_| b.below(7)).collect::<Vec<_>>());
    assert!(drawn.iter().all(|&d| d < 7));
    // Every value drawn in twenty, and another seed draws otherwise.
    assert!((0..7).all(|v| drawn.contains(&v)));
    let mut c = Picks::new(8);
    assert_ne!(drawn, (0..20).map(|_| c.below(7)).collect::<Vec<_>>());
    assert_eq!(Picks::new(1).below(0), 0);
}

#[test]
fn an_unnamed_town_s_name_by_the_seed() {
    // Without a seed the faction's first; with one, among its sixteen, the same each time, and
    // sixteen towns of a faction all named apart.
    assert_eq!(town_name(2, 5, None), 32);
    let names: Vec<u16> = (0..16).map(|town| town_name(2, town, Some(99))).collect();
    assert!(names.iter().all(|n| (32..48).contains(n)));
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 16);
    assert_eq!(town_name(2, 3, Some(99)), names[3]);
    assert_eq!(town_name(2, 19, Some(99)), names[3]);
}
