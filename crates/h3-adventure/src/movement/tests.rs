//! Tests of a day's movement points (E08 T6b).

use h3_content::CreatureTypeId;
use h3_contracts::ArmyStack;

use super::*;
use crate::testing::rules;

const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

fn stack(slot: u8, creature: CreatureTypeId) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count: 1,
    }
}

// covers: ADV-MOV-001
#[test]
fn the_slowest_creature_picks_the_land_tables_entry() {
    let (core, part) = rules();
    // Gogs (speed 2) slow the imps (speed 5) down to the table's third entry.
    assert_eq!(
        day_points(&core, &part, &[stack(0, IMP), stack(4, GOG)]),
        1430
    );
    assert_eq!(day_points(&core, &part, &[stack(1, GOG)]), 1430);
    // Speed 5 is past the table: its last entry.
    assert_eq!(day_points(&core, &part, &[stack(0, IMP)]), 1500);
    // Neither an empty army nor an unknown creature slows an army down.
    assert_eq!(day_points(&core, &part, &[]), 1500);
    let ghost = stack(2, CreatureTypeId::from_raw(9));
    assert_eq!(day_points(&core, &part, &[ghost, stack(0, IMP)]), 1500);
}
