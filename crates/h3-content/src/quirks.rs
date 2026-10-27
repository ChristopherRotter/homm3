//! Quirk flags (ADR-0007, docs/rules/quirks-ledger.md).
//!
//! A quirk flag toggles a known deviation of the original game. Flags that change behavior need
//! code that reads them, so the set of flags is a closed registry in code ([`REGISTRY`]); profiles
//! (data) choose which flags are on. `exploit-fix` quirks never get a flag.
//!
//! A flag's index is its position in the registry and never changes: a ruleset hashes its flags
//! as bits. A flag whose quirk turned out not to be one is retired: its position stays in the
//! registry as an entry without a name (`RETIRED`), which no profile can name, no code reads and
//! no later flag takes.

/// Maximum number of quirk flags (they are stored as a bitset).
pub const MAX_QUIRKS: usize = 64;

/// A registered quirk flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuirkFlag(u8);

impl QuirkFlag {
    /// The flag's bit index.
    pub const fn index(self) -> u8 {
        self.0
    }
}

/// The entry of a registry position whose flag was retired: it has no name, so
/// [`QuirkRegistry::lookup`] never finds it and [`QuirkRegistry::flags`] leaves it out.
const RETIRED: &str = "";

/// The closed list of quirk flags known to the engine: names like `quirk.battle.<name>`, with the
/// flag's index equal to its position.
#[derive(Debug, Clone, Copy)]
pub struct QuirkRegistry {
    names: &'static [&'static str],
}

impl QuirkRegistry {
    /// A registry over `names` (at most [`MAX_QUIRKS`] entries are addressable). An empty name
    /// keeps the position of a retired flag.
    pub const fn new(names: &'static [&'static str]) -> Self {
        Self { names }
    }

    /// Looks a flag up by name. No name finds a retired position, the empty one included.
    pub fn lookup(&self, name: &str) -> Option<QuirkFlag> {
        self.names
            .iter()
            .take(MAX_QUIRKS)
            .position(|n| *n != RETIRED && *n == name)
            .and_then(|i| u8::try_from(i).ok())
            .map(QuirkFlag)
    }

    /// The flags that exist, by index: every position but the retired ones.
    pub fn flags(&self) -> impl Iterator<Item = QuirkFlag> + '_ {
        (0u8..)
            .zip(self.names.iter().take(MAX_QUIRKS))
            .filter(|&(_, name)| *name != RETIRED)
            .map(|(index, _)| QuirkFlag(index))
    }

    /// Number of positions: the flags that exist and the retired ones.
    pub fn len(&self) -> usize {
        self.names.len().min(MAX_QUIRKS)
    }

    /// Whether the registry has no position.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The engine's quirk registry. Add entries together with the ledger entry, the rule and its
/// tests; a flag's index is its position here and must never change. A flag is retired by
/// putting `RETIRED` in its place, with a note of what it was.
pub const REGISTRY: QuirkRegistry = QuirkRegistry::new(&[
    "quirk.battle.no_luck_without_hero",
    "quirk.battle.single_precision_rates",
    "quirk.battle.unlimited_ballista_shots",
    "quirk.battle.first_protection",
    "quirk.battle.counterstrike_extra_round",
    "quirk.battle.frenzy_reduced_defense",
    "quirk.battle.fire_shield_first_stage",
    "quirk.battle.early_mass_version",
    "quirk.battle.mass_cure_skips_hypnotized",
    // 9: `quirk.battle.natives_sight_at_start` (BAT-QUIRK-1), retired on 2026-10-13 (the user,
    // BAT-Q34): the natives' sight fixed at the battle's start is the original's rule, and no
    // source describes the variant without it (`installation.md` §7.16).
    RETIRED,
    // 10: `quirk.battle.attack_from_seen_pit` (BAT-QUIRK-2), retired on 2026-10-12 (the user,
    // BAT-Q30): the original offers no attack from a pit the stack's side sees, so there is no
    // such quirk (`installation.md` §7.16).
    RETIRED,
    "quirk.battle.obstacle_anchors",
    "quirk.battle.draws_avoid_hidden",
    "quirk.battle.sacrifice_tests_as_resurrection",
    // 14: `quirk.battle.creature_mirror_untested` (BAT-QUIRK-9), retired on 2026-10-12 (the user,
    // BAT-Q29): the original's Magic Mirror never sends a creature's spell away, so there is no
    // such quirk (`installation.md` §7.21).
    RETIRED,
    "quirk.battle.clone_on_its_original",
    "quirk.battle.periodic_first_stack",
    "quirk.content.roland_crusader_specialty",
    "quirk.content.elemental_levels",
    "quirk.adventure.diagonal_check_ignores_roads",
    "quirk.adventure.diagonal_check_ignores_native",
    "quirk.adventure.native_skips_leading_neutrals",
    "quirk.adventure.fringe_shows_objects",
    "quirk.adventure.obstacles_from_tile",
    "quirk.adventure.split_from_tile",
    "quirk.battle.no_defender_ballista_at_fort",
    "quirk.battle.berserk_walks_to_position_hex",
    "quirk.adventure.level_up_from_seed",
    "quirk.content.elementalist_offer_intervals",
    "quirk.battle.original_ai_slips",
    // 30: `quirk.adventure.flying_inland_price` (ADV-QUIRK-16, D-24), retired on 2026-10-22 (E08
    // T41; ADV-Q22, E08 decision 85): the original prices a flying step from every land tile no
    // object blocks at the lesser of the fly and walking prices, so there is no such quirk (survey
    // `E08-hero-movement.md` §2.1 point 5, batch E5's E5-4).
    RETIRED,
    "quirk.adventure.drop_ignores_artifacts",
    "quirk.adventure.radius_waits_for_a_step",
    "quirk.battle.computer_morale_let_off",
    "quirk.battle.breath_skips_side_columns",
    "quirk.battle.retaliation_at_a_dead_striker",
    "quirk.battle.raise_by_rear_hex",
    "quirk.adventure.roe_witch_hut_mask",
    "quirk.adventure.full_backpack_refuses",
]);

/// Q-0003: creatures of a side without a hero never get lucky strikes (BAT-LCK-002).
pub const NO_LUCK_WITHOUT_HERO: QuirkFlag = QuirkFlag(0);

/// Q-0005: the original's rounding: damage in the executable's truncated stages and x87
/// arithmetic, rates as the code stores them (BAT-DMG-002 and others).
pub const SINGLE_PRECISION_RATES: QuirkFlag = QuirkFlag(1);

/// Q-0004: the shooting war machine (the Ballista) shoots on without shots left (BAT-WAR-002).
pub const UNLIMITED_BALLISTA_SHOTS: QuirkFlag = QuirkFlag(2);

/// Q-0007: a damage spell of several schools meets the target's first protection by school
/// rank, not its strongest (BAT-MAG-013).
pub const FIRST_PROTECTION: QuirkFlag = QuirkFlag(3);

/// Q-0008: Counterstrike's extra retaliations are given at the round start in which the spell
/// runs out, as the reset comes before the countdown (BAT-MAG-015).
pub const COUNTERSTRIKE_EXTRA_ROUND: QuirkFlag = QuirkFlag(4);

/// Q-0009 (S-16): Frenzy's attack bonus counts the frenzied stack's defense as the struck
/// stack sees it, reduced by a Behemoth's share (BAT-MAG-019).
pub const FRENZY_REDUCED_DEFENSE: QuirkFlag = QuirkFlag(5);

/// Q-0010: Fire Shield reflects a share of the strike's first damage stage (before the struck
/// stack's defense and protections), not of the damage dealt (BAT-MAG-018).
pub const FIRE_SHIELD_FIRST_STAGE: QuirkFlag = QuirkFlag(6);

/// Q-0011 (S-10): a spell whose mass version the original starts earlier than its clean
/// version (`SpellMagic::fixed_mass_from`) has it from that earlier mastery: Forgetfulness from
/// Advanced (BAT-MAG-023).
pub const EARLY_MASS_VERSION: QuirkFlag = QuirkFlag(7);

/// Q-0012 (S-13): a mass version skips hypnotized stacks also when its dispel would end their
/// Hypnotize (Expert Cure), which the clean version lets reach the caster's hypnotized stacks
/// (BAT-MAG-026). Not the mass version of a spell that only dispels: Expert Dispel reaches
/// hypnotized stacks with and without the flag, as the original's does (BAT-MAG-016).
pub const MASS_CURE_SKIPS_HYPNOTIZED: QuirkFlag = QuirkFlag(8);

/// BAT-QUIRK-3: a removable natural obstacle's anchor hex is part of it even where the obstacle
/// does not block it: it takes no Fire Wall, Force Field, pit or mine, and Remove Obstacle aimed at
/// it removes the obstacle (BAT-GEO-003, BAT-MAG-034). Off: such an anchor hex is a free hex.
pub const OBSTACLE_ANCHORS: QuirkFlag = QuirkFlag(11);

/// BAT-QUIRK-4: Quicksand's and Land Mine's random hexes avoid every obstacle, the other side's
/// hidden ones included, which tells the caster where none lies (BAT-MAG-032). Off: they avoid
/// only the obstacles the caster's side sees, and may land on a hidden one of the other side.
pub const DRAWS_AVOID_HIDDEN: QuirkFlag = QuirkFlag(12);

/// BAT-QUIRK-5: Sacrifice's target (the stack it raises) is tested as Resurrection's would be:
/// with Resurrection's level against immunities and Anti-Magic, its conditions, and its hit
/// points at the caster's mastery, which must raise at least one creature (BAT-MAG-036;
/// `installation.md` §7.18). Off: the target is tested with Sacrifice's own level and
/// conditions, without a hit-point minimum.
pub const SACRIFICE_TESTS_AS_RESURRECTION: QuirkFlag = QuirkFlag(13);

/// BAT-QUIRK-7: Clone's place search counts the original's own hexes as free, so a crowded
/// two-hex original can get its clone's place on itself: then no clone is made and the original
/// itself becomes a summoned stack with the clone's rounds, which vanishes when they run out and
/// is lost from its army after the battle (BAT-MAG-038; `installation.md` §7.19). Off: the
/// original's hexes are taken. A place whose other hex is the original's is skipped either way
/// (BAT-QUIRK-8).
pub const CLONE_ON_ITS_ORIGINAL: QuirkFlag = QuirkFlag(15);

/// BAT-QUIRK-13 (survey S-12): a periodic caster (the Enchanter) counts a spell when it is useful
/// on the first stack of the side it aims at, the side's lowest stack id (the original tests that
/// record once per record; BAT-MAG-053, `installation.md` §7.21), so a dead first stack rules out
/// every spell aimed at its side. Off: a spell counts when it is useful on any stack of that side.
pub const PERIODIC_FIRST_STACK: QuirkFlag = QuirkFlag(16);

/// Q-0006 (data): Roland's specialty names the Crusader, so it raises only Crusaders (the
/// executable's specialty table). Off: a gated override names the Swordsman, which raises both.
/// Read by the compiler only, as every `quirk.content` flag (ADR-0007, amendment E04b T8e).
pub const ROLAND_CRUSADER_SPECIALTY: QuirkFlag = QuirkFlag(17);

/// BAT-QUIRK-19 (data, survey S-05): the Ice, Magma and Energy Elementals have the executable's
/// levels 4, 3 and 5. Off: a gated override gives each its base creature's level, 3, 5 and 4.
/// Read by the compiler only.
pub const ELEMENTAL_LEVELS: QuirkFlag = QuirkFlag(18);

/// ADV-QUIRK-2: the diagonal exception compares the hero's points left with the source tile's
/// terrain cost even on a road, though the step it prices then costs the road's (ADV-MOV-006;
/// code 0x4B1740). Off: with the road's cost when both tiles have a road.
pub const DIAGONAL_CHECK_IGNORES_ROADS: QuirkFlag = QuirkFlag(19);

/// ADV-QUIRK-3: the diagonal exception compares the hero's points left with the terrain's
/// penalised cost even for an army native to it (ADV-MOV-006; code 0x4B1740). Off: with the
/// native cost.
pub const DIAGONAL_CHECK_IGNORES_NATIVE: QuirkFlag = QuirkFlag(20);

/// ADV-QUIRK-4: an army's native terrain skips neutral stacks in slots before its first faction
/// stack (ADV-MOV-004; code 0x44C590). Off: a neutral stack anywhere leaves the army without one.
pub const NATIVE_SKIPS_LEADING_NEUTRALS: QuirkFlag = QuirkFlag(21);

/// ADV-QUIRK-1: the fog's fringe, the hidden tiles touching an explored one, shows what objects
/// cover there and the heroes standing there, besides its terrain, roads and rivers (ADV-FOG-005;
/// the user's look in SoD, 2026-10-11). Off: the fringe shows terrain, roads and rivers only.
pub const FRINGE_SHOWS_OBJECTS: QuirkFlag = QuirkFlag(22);

/// ADV-QUIRK-5: a battle on the adventure map draws its obstacles from a generator seeded by its
/// tile, the original's 110 291 x + 167 801 y + 81 569 (GAM-BAT-001; code 0x463A08), so every
/// battle on a tile gets the same obstacles. Off: from the attacking player's stream.
pub const OBSTACLES_FROM_TILE: QuirkFlag = QuirkFlag(23);

/// ADV-QUIRK-6: a wandering monster's split into stacks takes its two draws from the C runtime's
/// generator seeded by its tile, the original's 248 071 x + 276 589 y + 310 111 z + 155 303
/// (ADV-MON-009; code 0x4AC580), so a visit splits the same way every time. Off: from the
/// attacking player's stream.
pub const SPLIT_FROM_TILE: QuirkFlag = QuirkFlag(24);

/// BAT-QUIRK-23: a battle at a town with a fort sets up no Ballista for the defender, though its
/// hero has one (BAT-SIE-029; the original's 0x463600, `installation.md` §7.27). Off: the
/// defender's Ballista is set up at every town, whatever its fort.
pub const NO_DEFENDER_BALLISTA_AT_FORT: QuirkFlag = QuirkFlag(25);

/// BAT-QUIRK-24 (survey S-11): a berserk walker's walk to its target ends only next to the
/// target's position hex, though the target is chosen as reached by either of its hexes
/// (BAT-MAG-025; the original's pathfinder 0x4B3400, `installation.md` §7.41): a two-hex target
/// reached only by its other hex makes the walker go round to its position, or do nothing. Off:
/// the walk ends next to either of the target's hexes.
pub const BERSERK_WALKS_TO_POSITION_HEX: QuirkFlag = QuirkFlag(26);

/// ADV-QUIRK-9: a level-up's draws come from the C runtime's generator reseeded from the hero's
/// seed byte and the new level, as the original's (`hero_level_up` 0x4DA720; survey
/// `E08-progression.md` §2.2), so a hero's level-ups follow from its seed. Off: from the owner's
/// stream. Read by the game from E08 T29 (GAM-HER-002).
pub const LEVEL_UP_FROM_SEED: QuirkFlag = QuirkFlag(27);

/// ADV-QUIRK-10 (data, A-10): the Elementalist gets Wisdom and the magic schools on the might
/// classes' intervals, 6 and 4 (the original's list of the shorter intervals, 0x4DAD74, ends with
/// the RoE classes). Off: a gated override gives it the magic classes' 3 and 3. Read by the
/// compiler only.
pub const ELEMENTALIST_OFFER_INTERVALS: QuirkFlag = QuirkFlag(28);

/// BAT-QUIRK-25 (survey `E23-battle-ai.md` §4): the original computer's slips in its own
/// estimates (a wrong record, argument, spell, side, mastery or test compared with the rule it
/// estimates; ADR-0027 point 9, BAT-Q55): on, the battle AI estimates as the original reads;
/// off, each estimate as the rule it estimates. It decides the computer's choices only, never a
/// rule's outcome. Read by `h3-battle`'s `computer` numbers and `h3-battle-ai` from E23 T2b.
pub const ORIGINAL_AI_SLIPS: QuirkFlag = QuirkFlag(29);

/// ADV-QUIRK-17 (A-12): the drop of the points left to 0 after a step prices the cheapest step
/// from the new tile without the artifacts' flying and water walking (`cheapest_step_from`
/// 0x4B1880 reads the hero's spell fields only; survey `E08-hero-movement.md` §2.5). Off: with
/// the hero's flying and water-walking levels. Read by the adventure from E08 T41.
pub const DROP_IGNORES_ARTIFACTS: QuirkFlag = QuirkFlag(31);

/// ADV-QUIRK-18 (D-26, E08 decision 77): a scouting radius grown in the day shows only at the
/// hero's next step: the new day reveals around a player's heroes only after a Cover of Darkness
/// hid tiles (`cover_of_darkness_hide` 0x4C7BA0; survey `E08-hero-movement.md` §4.2). Off: every
/// hero also reveals around it to its owner at each new day's start. Read by the adventure and
/// the game from E08 T40.
pub const RADIUS_WAITS_FOR_A_STEP: QuirkFlag = QuirkFlag(32);

/// BAT-QUIRK-27 (BAT-Q33, survey S-25): a stack of a side whose player is not human (a neutral
/// side, a computer player's; `BattleSetup.human`, ADR-0023 point 7) is let off a bad-morale
/// freeze one time in four (the original's second draw, 0x464BDB). Off: one roll for every side
/// (BAT-MOR-004). Read by the battle from E23 T7.
pub const COMPUTER_MORALE_LET_OFF: QuirkFlag = QuirkFlag(33);

/// BAT-QUIRK-28 (BAT-Q62, batch 2g BR, BC): a breath attack's second hex is found through the
/// original's neighbour table, which has no neighbour in the side columns (0x4640A0, melee_strike
/// 0x441739), so a breath never reaches a hex there: a war machine's rear hex in column 0 or 16
/// is not struck. Off: the stack behind the attacked hex anywhere (BAT-ATK-003, VCMI's pattern).
/// Read by the battle from the inbox session's batch 2g.
pub const BREATH_SKIPS_SIDE_COLUMNS: QuirkFlag = QuirkFlag(34);

/// BAT-QUIRK-29 (survey S-23, BAT-Q28 row 4, batch 2g FS): the struck stack retaliates at a
/// striker its Fire Shield's reflection killed and spends its retaliation, which strikes nothing
/// (0x441DD4 to 0x441E01; fixed in HotA 1.7.1): the round's next attacker is not struck back. Off:
/// no retaliation at a dead striker, none spent (BAT-RET-001). Read by the battle from batch 2g.
pub const RETALIATION_AT_A_DEAD_STRIKER: QuirkFlag = QuirkFlag(35);

/// BAT-QUIRK-30 (survey S-24, BAT-Q28 row 10, batch 2g RS): a hero's raising spell aimed at a
/// two-hex corpse raises the top corpse on that corpse's rear hex (0x5A1917; fixed in HotA
/// 1.5.4), so a two-hex corpse is found by its rear hex alone and a later corpse there hides it.
/// Off: by either hex (BAT-MAG-035). Read by the battle from batch 2g.
pub const RAISE_BY_REAR_HEX: QuirkFlag = QuirkFlag(36);

/// ADV-QUIRK-21 (A-21, E08 decision 130; survey `E08-rewards.md` §9, D-33): a Witch Hut on an RoE
/// map, whose file holds no skill mask, draws from the original's fixed mask 0xEFBF (0x5034BD):
/// the skills 0 to 15 but Leadership and Necromancy, never Water Magic to First Aid. Off: every
/// skill but Leadership and Necromancy (the editor's default, VCMI's). Read by the adventure's
/// build from E08 T61.
pub const ROE_WITCH_HUT_MASK: QuirkFlag = QuirkFlag(37);

/// ADV-QUIRK-22 (A-25, E08 decision 145; survey `E08-rewards.md` §15): a hero whose backpack
/// holds its most (64) receives no artifact, whatever its free positions: the visits test the
/// count before `hero_give_artifact` places anything, so the artifact is lost or turned into
/// gold or resources (ADR-0013 point 15.4). Off: the artifact goes to the first position that
/// takes it, else the backpack (HER-CHG-002); only one without room is refused. Read by
/// `h3-heroes` and the adventure's visits from E08 T60.
pub const FULL_BACKPACK_REFUSES: QuirkFlag = QuirkFlag(38);

/// The prefix of the flags of data quirks: a gated override names one of them (`fixes`), and
/// rule code never reads them (ADR-0007, amendment E04b T8e).
pub const DATA_QUIRK_PREFIX: &str = "quirk.content.";

/// The set of enabled quirk flags of a ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    bits: u64,
}

impl Quirks {
    /// Whether `flag` is enabled.
    pub const fn is_enabled(self, flag: QuirkFlag) -> bool {
        self.bits & (1u64 << flag.0) != 0
    }

    /// Returns the set with `flag` set to `on`.
    pub const fn with(self, flag: QuirkFlag, on: bool) -> Self {
        let mask = 1u64 << flag.0;
        Self {
            bits: if on {
                self.bits | mask
            } else {
                self.bits & !mask
            },
        }
    }

    /// The raw bitset (for hashing and replay headers).
    pub const fn bits(self) -> u64 {
        self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: QuirkRegistry = QuirkRegistry::new(&["quirk.battle.a", "quirk.battle.b"]);

    #[test]
    fn registry_lookup_and_bitset() {
        assert_eq!(REGISTRY.len(), 39);
        assert_eq!(
            [
                REGISTRY.lookup("quirk.adventure.roe_witch_hut_mask"),
                REGISTRY.lookup("quirk.adventure.full_backpack_refuses"),
            ],
            [Some(ROE_WITCH_HUT_MASK), Some(FULL_BACKPACK_REFUSES)]
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.computer_morale_let_off"),
            Some(COMPUTER_MORALE_LET_OFF)
        );
        assert_eq!(
            [
                REGISTRY.lookup("quirk.battle.breath_skips_side_columns"),
                REGISTRY.lookup("quirk.battle.retaliation_at_a_dead_striker"),
                REGISTRY.lookup("quirk.battle.raise_by_rear_hex"),
            ],
            [
                Some(BREATH_SKIPS_SIDE_COLUMNS),
                Some(RETALIATION_AT_A_DEAD_STRIKER),
                Some(RAISE_BY_REAR_HEX)
            ]
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.drop_ignores_artifacts"),
            Some(DROP_IGNORES_ARTIFACTS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.radius_waits_for_a_step"),
            Some(RADIUS_WAITS_FOR_A_STEP)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.original_ai_slips"),
            Some(ORIGINAL_AI_SLIPS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.level_up_from_seed"),
            Some(LEVEL_UP_FROM_SEED)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.content.elementalist_offer_intervals"),
            Some(ELEMENTALIST_OFFER_INTERVALS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.berserk_walks_to_position_hex"),
            Some(BERSERK_WALKS_TO_POSITION_HEX)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.no_defender_ballista_at_fort"),
            Some(NO_DEFENDER_BALLISTA_AT_FORT)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.split_from_tile"),
            Some(SPLIT_FROM_TILE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.obstacles_from_tile"),
            Some(OBSTACLES_FROM_TILE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.fringe_shows_objects"),
            Some(FRINGE_SHOWS_OBJECTS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.diagonal_check_ignores_roads"),
            Some(DIAGONAL_CHECK_IGNORES_ROADS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.diagonal_check_ignores_native"),
            Some(DIAGONAL_CHECK_IGNORES_NATIVE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.native_skips_leading_neutrals"),
            Some(NATIVE_SKIPS_LEADING_NEUTRALS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.content.roland_crusader_specialty"),
            Some(ROLAND_CRUSADER_SPECIALTY)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.content.elemental_levels"),
            Some(ELEMENTAL_LEVELS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.periodic_first_stack"),
            Some(PERIODIC_FIRST_STACK)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.clone_on_its_original"),
            Some(CLONE_ON_ITS_ORIGINAL)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.no_luck_without_hero"),
            Some(NO_LUCK_WITHOUT_HERO)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.single_precision_rates"),
            Some(SINGLE_PRECISION_RATES)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.unlimited_ballista_shots"),
            Some(UNLIMITED_BALLISTA_SHOTS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.first_protection"),
            Some(FIRST_PROTECTION)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.counterstrike_extra_round"),
            Some(COUNTERSTRIKE_EXTRA_ROUND)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.frenzy_reduced_defense"),
            Some(FRENZY_REDUCED_DEFENSE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.fire_shield_first_stage"),
            Some(FIRE_SHIELD_FIRST_STAGE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.early_mass_version"),
            Some(EARLY_MASS_VERSION)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.mass_cure_skips_hypnotized"),
            Some(MASS_CURE_SKIPS_HYPNOTIZED)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.obstacle_anchors"),
            Some(OBSTACLE_ANCHORS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.draws_avoid_hidden"),
            Some(DRAWS_AVOID_HIDDEN)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.sacrifice_tests_as_resurrection"),
            Some(SACRIFICE_TESTS_AS_RESURRECTION)
        );
        // `new` is a const fn; call it at run time too so coverage sees it.
        let runtime = QuirkRegistry::new(&["quirk.battle.a"]);
        assert_eq!(
            runtime.lookup("quirk.battle.a").map(QuirkFlag::index),
            Some(0)
        );
        assert_eq!(TEST.len(), 2);
        assert!(!TEST.is_empty());
        let b = TEST.lookup("quirk.battle.b").unwrap();
        assert_eq!(b.index(), 1);
        assert_eq!(TEST.lookup("quirk.battle.c"), None);
        let q = Quirks::default().with(b, true);
        assert!(q.is_enabled(b));
        assert_eq!(q.bits(), 2);
        assert!(!q.with(b, false).is_enabled(b));
    }

    #[test]
    fn a_retired_flag_keeps_its_position_and_has_no_name() {
        // Positions 10 and 14 were `quirk.battle.attack_from_seen_pit` (BAT-QUIRK-2, BAT-Q30)
        // and `quirk.battle.creature_mirror_untested` (BAT-QUIRK-9, BAT-Q29), both retired on
        // 2026-10-12, position 9 `quirk.battle.natives_sight_at_start` (BAT-QUIRK-1, BAT-Q34),
        // retired on 2026-10-13, position 30 `quirk.adventure.flying_inland_price`
        // (ADV-QUIRK-16, ADV-Q22), retired on 2026-10-22. Neither their old names nor the empty
        // name of their entries find them.
        for name in [
            "quirk.battle.natives_sight_at_start",
            "quirk.battle.attack_from_seen_pit",
            "quirk.battle.creature_mirror_untested",
            "quirk.adventure.flying_inland_price",
            "",
        ] {
            assert_eq!(REGISTRY.lookup(name), None, "{name:?}");
        }
        // The positions stay taken, so the flags after them keep their indices
        // (`registry_lookup_and_bitset`), and the flags that exist are every other position.
        assert_eq!(REGISTRY.len(), 39);
        assert_eq!(
            [
                REGISTRY.names[9],
                REGISTRY.names[10],
                REGISTRY.names[14],
                REGISTRY.names[30]
            ],
            [RETIRED; 4]
        );
        assert!(
            REGISTRY
                .flags()
                .map(QuirkFlag::index)
                .eq((0..39).filter(|i| ![9, 10, 14, 30].contains(i)))
        );
        // A registry without retired flags has them all.
        assert!(TEST.flags().map(QuirkFlag::index).eq(0..2));
    }
}
