use super::*;

#[test]
fn days_count_beyond_the_points_left() {
    assert_eq!(days(300, 300, 1500), 0);
    assert_eq!(days(301, 300, 1500), 1);
    assert_eq!(days(1800, 300, 1500), 1);
    assert_eq!(days(1801, 300, 1500), 2);
    assert_eq!(days(9999, 0, 1500), 3);
    assert_eq!(days(5, 0, 0), 1);
}
