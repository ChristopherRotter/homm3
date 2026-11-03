//! What a viewer's own towns tell it of another's (E09 increment 1's design note §12): so far the
//! Tavern its towns count for the garrison's level (the level and the views: E09 T8c).

use crate::derive::Roles;
use crate::record::Town;
use crate::ruleset::TownRuleset;

/// Whether the town counts for its owner's Tavern level: a **standing** building of role
/// `tavern` (the Tavern, or Castle's Brotherhood of the Sword that supersedes it; E09 survey
/// §12.3: `player_tavern_count` 0x4CCE30). The viewer's level is E09 T8c's (design §12).
// rule: TWN-VEW-002
pub fn has_tavern(town: &Town, part: &TownRuleset) -> bool {
    Roles::of(town, part)
        .standing_roles()
        .any(|(_, r)| r.tavern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::*;

    #[test]
    fn a_standing_tavern_counts() {
        // covers: TWN-VEW-002
        let part = part();
        let tavern = |built: &[usize]| has_tavern(&town(built), &part);
        assert!(tavern(&[TAVERN]));
        // The Brotherhood supersedes the Tavern and counts in its place, alone too.
        assert!(tavern(&[TAVERN, BROTHERHOOD]));
        assert!(tavern(&[BROTHERHOOD]));
        assert!(!tavern(&[VILLAGE_HALL, FORT]));
        assert!(!has_tavern(&town_of(WILD, &[TAVERN]), &part));
        assert!(!has_tavern(&town(&[TAVERN]), &empty_part()));
    }
}
