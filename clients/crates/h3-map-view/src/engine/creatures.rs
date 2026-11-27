//! The creatures' numbers (E24 M5 T4c, design §5.29, *provisional*): a creature on the wire (an
//! army's stack, a mine's guards, a monster drawn, a week's or a month's creature, a visit's
//! creatures joined or refused) is the engine's content handle, by the packs' order (`base` the
//! original's 0 to 117, then its war machines 145 to 149 as 118 to 122; `ab` from 123 on), but the
//! view's tables, the catalogue's `creature.<n>`, the portraits and the monsters' templates are by
//! the original's number. [`Creatures`] turns a handle into that number by its key in the engine's
//! rules (`creature_key`) and the original's number of each key ([`NUMBERS`]); [`super::Loaded`]
//! makes it of the record's rules once, and every play reading of a wire creature reads it (a
//! visit's events numbered as `engine::played` keeps them: [`Creatures::event`]). The test
//! tooling's paths (a sequence's, a situation's, a capture's, `Ask::of`) read the map file's or the
//! original's numbers already.

use alloc::vec::Vec;

use h3_game::GameRuleset;
use h3_kernel::Handle;
use h3_protocol::wire::game::{AdventureEvent, GameEvent};

/// No creature of the original's: a handle the rules do not hold.
pub const NONE: u16 = u16::MAX;

/// The original's number of each of the packs' creatures by its key, ascending, the numbers no
/// creature has left out (122, 124, 126, 128): the engine's import's table (`h3_map`'s
/// `import::codes::CREATURES`, the map's creature ids to their keys; crate-private there, so the
/// view keeps its copy, *provisional*), read the other way; `tests/engine.rs` checks each entry a
/// map file holds (to 144) against the import, the war machines' by their keys.
const NUMBERS: [(u16, &str); 146] = [
    (0, "base:pikeman"),
    (1, "base:halberdier"),
    (2, "base:archer"),
    (3, "base:marksman"),
    (4, "base:griffin"),
    (5, "base:royal_griffin"),
    (6, "base:swordsman"),
    (7, "base:crusader"),
    (8, "base:monk"),
    (9, "base:zealot"),
    (10, "base:cavalier"),
    (11, "base:champion"),
    (12, "base:angel"),
    (13, "base:archangel"),
    (14, "base:centaur"),
    (15, "base:centaur_captain"),
    (16, "base:dwarf"),
    (17, "base:battle_dwarf"),
    (18, "base:wood_elf"),
    (19, "base:grand_elf"),
    (20, "base:pegasus"),
    (21, "base:silver_pegasus"),
    (22, "base:dendroid_guard"),
    (23, "base:dendroid_soldier"),
    (24, "base:unicorn"),
    (25, "base:war_unicorn"),
    (26, "base:green_dragon"),
    (27, "base:gold_dragon"),
    (28, "base:gremlin"),
    (29, "base:master_gremlin"),
    (30, "base:stone_gargoyle"),
    (31, "base:obsidian_gargoyle"),
    (32, "base:stone_golem"),
    (33, "base:iron_golem"),
    (34, "base:mage"),
    (35, "base:arch_mage"),
    (36, "base:genie"),
    (37, "base:master_genie"),
    (38, "base:naga"),
    (39, "base:naga_queen"),
    (40, "base:giant"),
    (41, "base:titan"),
    (42, "base:imp"),
    (43, "base:familiar"),
    (44, "base:gog"),
    (45, "base:magog"),
    (46, "base:hell_hound"),
    (47, "base:cerberus"),
    (48, "base:demon"),
    (49, "base:horned_demon"),
    (50, "base:pit_fiend"),
    (51, "base:pit_lord"),
    (52, "base:efreeti"),
    (53, "base:efreet_sultan"),
    (54, "base:devil"),
    (55, "base:arch_devil"),
    (56, "base:skeleton"),
    (57, "base:skeleton_warrior"),
    (58, "base:walking_dead"),
    (59, "base:zombie"),
    (60, "base:wight"),
    (61, "base:wraith"),
    (62, "base:vampire"),
    (63, "base:vampire_lord"),
    (64, "base:lich"),
    (65, "base:power_lich"),
    (66, "base:black_knight"),
    (67, "base:dread_knight"),
    (68, "base:bone_dragon"),
    (69, "base:ghost_dragon"),
    (70, "base:troglodyte"),
    (71, "base:infernal_troglodyte"),
    (72, "base:harpy"),
    (73, "base:harpy_hag"),
    (74, "base:beholder"),
    (75, "base:evil_eye"),
    (76, "base:medusa"),
    (77, "base:medusa_queen"),
    (78, "base:minotaur"),
    (79, "base:minotaur_king"),
    (80, "base:manticore"),
    (81, "base:scorpicore"),
    (82, "base:red_dragon"),
    (83, "base:black_dragon"),
    (84, "base:goblin"),
    (85, "base:hobgoblin"),
    (86, "base:wolf_rider"),
    (87, "base:wolf_raider"),
    (88, "base:orc"),
    (89, "base:orc_chieftain"),
    (90, "base:ogre"),
    (91, "base:ogre_mage"),
    (92, "base:roc"),
    (93, "base:thunderbird"),
    (94, "base:cyclops"),
    (95, "base:cyclops_king"),
    (96, "base:behemoth"),
    (97, "base:ancient_behemoth"),
    (98, "base:gnoll"),
    (99, "base:gnoll_marauder"),
    (100, "base:lizardman"),
    (101, "base:lizard_warrior"),
    (102, "base:gorgon"),
    (103, "base:mighty_gorgon"),
    (104, "base:serpent_fly"),
    (105, "base:dragon_fly"),
    (106, "base:basilisk"),
    (107, "base:greater_basilisk"),
    (108, "base:wyvern"),
    (109, "base:wyvern_monarch"),
    (110, "base:hydra"),
    (111, "base:chaos_hydra"),
    (112, "base:air_elemental"),
    (113, "base:earth_elemental"),
    (114, "base:fire_elemental"),
    (115, "base:water_elemental"),
    (116, "base:gold_golem"),
    (117, "base:diamond_golem"),
    (118, "ab:pixie"),
    (119, "ab:sprite"),
    (120, "ab:psychic_elemental"),
    (121, "ab:magic_elemental"),
    (123, "ab:ice_elemental"),
    (125, "ab:magma_elemental"),
    (127, "ab:storm_elemental"),
    (129, "ab:energy_elemental"),
    (130, "ab:firebird"),
    (131, "ab:phoenix"),
    (132, "ab:azure_dragon"),
    (133, "ab:crystal_dragon"),
    (134, "ab:faerie_dragon"),
    (135, "ab:rust_dragon"),
    (136, "ab:enchanter"),
    (137, "ab:sharpshooter"),
    (138, "ab:halfling"),
    (139, "ab:peasant"),
    (140, "ab:boar"),
    (141, "ab:mummy"),
    (142, "ab:nomad"),
    (143, "ab:rogue"),
    (144, "ab:troll"),
    (145, "base:catapult"),
    (146, "base:ballista"),
    (147, "base:first_aid_tent"),
    (148, "base:ammo_cart"),
    (149, "base:arrow_tower"),
];

/// The original's number of each creature handle of a game's rules (the module's documentation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Creatures {
    /// By handle.
    numbers: Vec<u16>,
}

impl Creatures {
    /// The numbers of `rules`' creatures: each handle's key's ([`NONE`] for a key the original
    /// numbers not).
    #[must_use]
    pub fn new(rules: &GameRuleset) -> Self {
        let core = rules.core();
        let numbers = (0..core.creature_count())
            .map(|index| {
                Handle::from_index(index)
                    .and_then(|handle| core.creature_key(handle))
                    .and_then(|key| NUMBERS.iter().find(|&&(_, k)| k == key.as_str()))
                    .map_or(NONE, |&(number, _)| number)
            })
            .collect();
        Self { numbers }
    }

    /// The original's number of the creature `handle` ([`NONE`] for a handle the rules do not
    /// hold).
    #[must_use]
    pub fn number(&self, handle: u16) -> u16 {
        self.numbers
            .get(usize::from(handle))
            .copied()
            .unwrap_or(NONE)
    }

    /// `event` as a visit keeps it (`engine::Visit`'s events): the creatures its readings name
    /// (`super::visits`: a hero's stack changed, an army dialog's stack, creatures joined, refused
    /// for want of room, none to recruit) numbered; every other event as the record's.
    #[must_use]
    pub(super) fn event(&self, event: &GameEvent) -> GameEvent {
        let mut event = event.clone();
        match &mut event {
            GameEvent::StackChanged {
                stack: Some(stack), ..
            }
            | GameEvent::ArmyDialogOpened { stack, .. } => {
                stack.creature = self.number(stack.creature);
            }
            GameEvent::Adventure {
                event:
                    AdventureEvent::CreaturesJoined { creature, .. }
                    | AdventureEvent::JoinRefused { creature, .. }
                    | AdventureEvent::NoneToRecruit { creature, .. },
            } => *creature = self.number(*creature),
            _ => {}
        }
        event
    }
}
