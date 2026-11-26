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

/// Maximum number of quirk flags (they are stored as a 128-bit set; 64 until E14 T2c, whose
/// registry reached 54 positions with ADR-0029's 15 flags still to come).
pub const MAX_QUIRKS: usize = 128;

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
    "quirk.town.creature_month_exact",
    "quirk.town.neutral_plague_full_growth",
    "quirk.adventure.sirens_again_after_battle",
    "quirk.town.leave_points_over_cleared_army",
    "quirk.town.guild_artifacts_by_backpacks",
    "quirk.town.aurora_ignores_map_bans",
    "quirk.adventure.flight_reads_hidden",
    "quirk.adventure.door_reads_hidden",
    "quirk.adventure.scuttle_reads_hidden",
    "quirk.adventure.summon_boat_by_steps",
    "quirk.adventure.spell_recast_refused",
    "quirk.adventure.door_failure_charged",
    "quirk.content.rmg_8xm8_zone_ids",
    "quirk.rmg.roe_monster_town_byte",
    "quirk.rmg.unowned_start_seventh",
    "quirk.town.visitor_only_trades",
    "quirk.town.university_ignores_map_bans",
    "quirk.rmg.zone_memory_model",
    "quirk.rmg.terrain_towns_count",
    "quirk.rmg.partition_units",
    "quirk.rmg.cosmetic",
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

/// TWN-QUIRK-10 (T-13, TD-13, E09 decision 69; survey `E09-growth.md` §3, batch TWN-3's
/// TWN-3-1): a creature's month doubles only the town counts whose creature is the month's
/// (`new_month` 0x4C8F5E's loop), so a level's upgrade or base creature beside it keeps its
/// count. Off: the level's counts whichever of its two creatures is the month's (VCMI 1.7.5's
/// rule). Class `fix-optional`; read by `h3-towns`' month from E09 T25b1, which the game's day
/// runs from E09 T26a1.
pub const CREATURE_MONTH_EXACT: QuirkFlag = QuirkFlag(39);

/// TWN-QUIRK-9 (T-12, TD-14, E09 decision 70; survey `E09-growth.md` §3, batch TWN-3's
/// TWN-3-3): the Plague takes a neutral town's whole growth g from a count its week grew by half
/// of it, then halves, so the town keeps max(0, before − ⌈g ÷ 2⌉) ÷ 2 of its count before the
/// week. Off: every town keeps half of its count before the week. Class `fix-optional`; read by
/// `h3-towns`' month from E09 T25b1, which the game's day runs from E09 T26a1.
pub const NEUTRAL_PLAGUE_FULL_GROWTH: QuirkFlag = QuirkFlag(40);

/// ADV-QUIRK-46 (A-44, E08 decisions 301, 308; survey `E08-garrisons-bonuses.md` §10.5, batch
/// E12's E12-8): the Sirens' mark (the original's status bit 20) is a battle bonus's, which
/// `hero_battle_bonuses_clear` clears at the hero's next battle's end, so the hero visits any
/// Sirens again after each battle and loses the creatures again. Off: the battle's end keeps the
/// mark, so a hero visits Sirens once (the one mark for every Sirens, decision 308). Class
/// `fix-optional`; read by the adventure's battle clear from E08 T115b.
pub const SIRENS_AGAIN_AFTER_BATTLE: QuirkFlag = QuirkFlag(41);

/// TWN-QUIRK-12 (T-16, E09 decision 116; design `E09-design.md` §67, batch TWN-4's TWN-4-4): a
/// hero that leaves the game without returning (a loss, a draw, the capture's leave, the
/// dismissal) takes as its points left and the day's maximum the land maximum the original
/// computes after clearing its army (`hero_leave_game` 0x4D9EC0 refills the points before it
/// places the one creature, so `hero_max_land_movement` 0x4E4990 starts from the empty slots'
/// speed 20, the land table's most). Off: the land maximum of the one creature it keeps. Seen
/// only when the type is drawn as a replacement and hired in the same turn. Class
/// `fix-optional`; read by `h3-game`'s leave from E09 T38b.
pub const LEAVE_POINTS_OVER_CLEARED_ARMY: QuirkFlag = QuirkFlag(42);

/// TWN-QUIRK-13 (T-17, E09 decision 117; survey `E09-tavern.md` §13, batch TWN-4's TWN-4-7):
/// the Thieves' Guild's artifacts row counts each player's heroes' backpacks alone
/// (`hero_worn_count` 0x4D9070 with argument 1 skips every worn position). Off: the worn and the
/// backpack's artifacts (VCMI's). Information only, no state touched. Class `fix-optional`; read
/// by `h3-game`'s Thieves' Guild from E09 T39b.
pub const GUILD_ARTIFACTS_BY_BACKPACKS: QuirkFlag = QuirkFlag(43);

/// TWN-QUIRK-14 (T-18, TD-26, E09 decision 169; survey `E09-guild.md` §5, batch TWN-5's
/// TWN-5-2): the Conflux's Aurora Borealis teaches every spell its town does not ban (but those
/// granted by name only) whatever the scenario bans: `town_heroes_learn_spells` 0x5BE030 tests
/// the town's banned set and never the map-wide bans (0x5BE10C to 0x5BE172). Off: the scenario's
/// bans are taken out of the Aurora set too (VCMI's map-wide test, with the town's bans kept).
/// Class `fix-optional`; read by `h3-towns`' Aurora set from E09 T48a.
pub const AURORA_IGNORES_MAP_BANS: QuirkFlag = QuirkFlag(44);

/// ADV-QUIRK-15 (A-13, reclassified fix-optional by E08 decision 399; survey `E08-spells.md`
/// §7.3, §13, batch E15's E15-13): the original's pathfinder lets a flying hero pass over a tile
/// its player has not explored, never stop there, and refuses it only where the hidden tile is
/// rock (`pathfinder_expand` 0x4B2300), so the flight's reach and prices read the hidden truth.
/// Off: the compromise (increment 15's design note §11.1): an unexplored tile is assumed flyable
/// at a pass's price, never a stop; the move walks step by step, each step revealing, and ends on
/// the last stop node reached before a step the revealed tile forbids, charged only up to it (E08
/// decision 407). Walkers and water-walkers never enter an unexplored tile either way. Class
/// `fix-optional`; read by the adventure's passage from E08 T142.
pub const FLIGHT_READS_HIDDEN: QuirkFlag = QuirkFlag(45);

/// ADV-QUIRK-51 (A-53, E08 decision 399; survey `E08-spells.md` §8.2, batch E15's E15-8): the
/// original's Dimension Door target dialog offers a hidden tile of its window when the tile is
/// truly free and none where it is blocked (`door_target_proc` 0x491900 reads the hidden truth);
/// a click on a hidden blocked tile is no cast. Off: every hidden tile of the window is a target,
/// the truth read at the cast (a wrong one fails, charged, and is revealed to the caster's team).
/// Class `fix-optional`; read by the door from E08 T143b.
pub const DOOR_READS_HIDDEN: QuirkFlag = QuirkFlag(46);

/// ADV-QUIRK-52 (A-50, E08 decision 402; survey `E08-spells.md` §6, batch E15's E15-10): the
/// original's Scuttle Boat dialog offers a hidden tile only where an empty boat truly lies, and
/// sinks it revealing nothing. Off: every hidden tile of the window is a target, a hidden empty
/// boat drawn for and sunk, any other hidden tile a charged failure, the tile revealed to the
/// caster's team in every outcome. Class `fix-optional`; read by Scuttle Boat from E08 T143a.
pub const SCUTTLE_READS_HIDDEN: QuirkFlag = QuirkFlag(47);

/// ADV-QUIRK-53 (D-75, E08 decision 400; survey `E08-spells.md` §5, batch E15's E15-9a and
/// E15-9b): after the hero's last empty boat, Summon Boat takes the least |dx| + |dy| among the
/// empty boats of the caster's player or of nobody, levels ignored, a tie to the later index (the
/// reading). Off: the least dx² + dy² on the hero's level, a tie to the lower index. Class
/// `fix-optional`; read by Summon Boat from E08 T143a.
pub const SUMMON_BOAT_BY_STEPS: QuirkFlag = QuirkFlag(48);

/// ADV-QUIRK-54 (A-51, E08 decision 403; survey `E08-spells.md` §7.1, batch E15's E15-11): Fly is
/// refused while it is on or Angel Wings are worn, and Water Walk on land while it is on or Boots
/// of Levitation are worn, free and silent (the original's "nothing"), so a re-cast at a higher
/// mastery never raises the level. Off: refused only when the new level would not exceed the
/// current one (the artifacts' 3 included). Class `fix-optional`; read by Fly and Water Walk from
/// E08 T142.
pub const SPELL_RECAST_REFUSED: QuirkFlag = QuirkFlag(49);

/// ADV-QUIRK-55 (A-52, E08 decision 403; survey `E08-spells.md` §8.1, batch E15's E15-7): an
/// explored Dimension Door target of the wrong kind (land from a boat, water from land) is
/// offered and its cast fails, charged (GENRLTXT 71: the mana, the points, a cast of the day).
/// Off: such an explored target is refused, changing nothing; a hidden one still fails charged.
/// Class `fix-optional`; read by the door from E08 T143b.
pub const DOOR_FAILURE_CHARGED: QuirkFlag = QuirkFlag(50);

/// RMG-QUIRK-22 (data; survey `E14-generator.md` §3.1): template 8XM8's six connection rows whose
/// first zone is a decimal (5.9 to 7.9) keep the ids `c_atoi` reads, 5, 6, 6, 7, 7, 7, so human
/// start 8 has no connection. Off: a gated override of the rmg part gives them 6, 6, 7, 7, 8, 8.
/// Read by the rmg part's compiler only, as every `quirk.content` flag (E14 T2b).
pub const RMG_8XM8_ZONE_IDS: QuirkFlag = QuirkFlag(51);

/// RMG-QUIRK-11 (survey `E14-generator.md` §2.1): for an RoE map a template zone's monster towns
/// lose Fortress (`rmg_zone` +0x9D, column 65) where the town types lose Conflux, the ninth. Off:
/// RoE's zones lose Conflux's monster town (+0x9E) instead. Read by `h3-rmg`'s templates (E14
/// T2c).
pub const RMG_ROE_MONSTER_TOWN_BYTE: QuirkFlag = QuirkFlag(52);

/// RMG-QUIRK-24 (survey `E14-generator.md` §2.1, E14 T2c): a start zone without Ownership
/// (position -1) marks its start one entry before the starts' array, which is the human starts'
/// eighth: a human start at position 7 that no zone has (AB's 2SM2i(2) and 2SM0k). Off: such a
/// zone marks no start. Read by `h3-rmg`'s positions.
pub const RMG_UNOWNED_START_SEVENTH: QuirkFlag = QuirkFlag(53);

/// TWN-QUIRK-15 (T-28, TD-28, E09 decision 202; survey `E09-trades.md` §2, §4.2, batch TWN-6's
/// TWN-6-5): the Artifact Merchants, the Freelancer's Guild, the Blacksmith and the Ballista Yard
/// serve the town's visiting hero only: without one they show GENRLTXT 274 and open nothing
/// (0x5E9D28, 0x5E9EAA, 0x5D1D4D to 0x5D1D8F), a garrison hero alone is never served, where the
/// spell book, the University and the Transformer take it. Off: the window's hero is the visitor,
/// else the garrison hero. Class `fix-optional`; read by the town's trades from E09 T58a.
pub const VISITOR_ONLY_TRADES: QuirkFlag = QuirkFlag(54);

/// TWN-QUIRK-16 (T-23, TD-30, E09 decision 204; survey `E09-trades.md` §6.3, batch TWN-6's
/// TWN-6-7): the Conflux's Magic University offers and teaches its four magic schools whatever
/// the scenario bans: neither `university_slot_state` 0x5F0790, `university_slot_click`
/// 0x5F0AC0, `university_learn` 0x5F0F60 nor `hero_learn_secondary_skill` 0x4E22D0 reads the
/// map's banned skills. Off: the scenario's banned skills leave the four when it opens (VCMI's).
/// Class `fix-optional`; read by the town's University from E09 T59b.
pub const UNIVERSITY_IGNORES_MAP_BANS: QuirkFlag = QuirkFlag(55);

/// RMG-QUIRK-1 (survey `E14-generator.md` §2.2, §3.2; registered at E14 T3c with increment 3's
/// flags, decision 10): step 2's zone generators whose town-type bytes nobody writes (the
/// temporary zone of each level's fit tests, the water zones) count and draw their town type from
/// the oracle's model of the bytes a fresh start's first map holds (E14 T3a's memory record). Off:
/// the bytes zeroed, no town type set, no draw. Read by step 2 (E14 T3d).
pub const RMG_ZONE_MEMORY_MODEL: QuirkFlag = QuirkFlag(56);

/// RMG-QUIRK-12 (survey `E14-generator.md` §1.5, §2.2; registered at E14 T3c with increment 3's
/// flags, decision 10): step 1's town type (+8) of a zone without one draws among all four entries
/// of its terrain's row of `rmg_terrain_towns` 0x682450, the -1 terminator and the Castle padding
/// included. Off: among the terrain's own town types before the -1, without Conflux for RoE. Read
/// by step 1 (E14 T3c).
pub const RMG_TERRAIN_TOWNS_COUNT: QuirkFlag = QuirkFlag(57);

/// RMG-QUIRK-14 (survey `E14-generator.md` §2.2; registered at E14 T3c with increment 3's flags,
/// decision 10): step 2's `rmg_zone_position_fits` 0x53AD60 compares map distances with the
/// existing zones' base sizes in template units, the temporary zone carrying a scaled size. Off:
/// every size scaled to the map. Read by step 2 (E14 T3d).
pub const RMG_PARTITION_UNITS: QuirkFlag = QuirkFlag(58);

/// RMG-QUIRK-21 (survey `E14-generator.md` §2.5, §2.7.2 to §2.7.4; registered at E14 T3c with
/// increment 3's flags, decision 10), one flag for all: the painters' pictures (the neighbours'
/// else-if, the inverted sand corner, the clear's short right column), rivers under objects and
/// obstacles, the description's town of the colour's index and its humans counting the computers.
/// Off: the pictures as the neighbours give them, rivers on free cells only, the description as
/// chosen. Read by the painters and the writer from E14 T3e.
pub const RMG_COSMETIC: QuirkFlag = QuirkFlag(59);

/// The prefix of the flags of data quirks: a gated override names one of them (`fixes`), and
/// rule code never reads them (ADR-0007, amendment E04b T8e).
pub const DATA_QUIRK_PREFIX: &str = "quirk.content.";

/// The set of enabled quirk flags of a ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    bits: u128,
}

impl Quirks {
    /// Whether `flag` is enabled.
    pub const fn is_enabled(self, flag: QuirkFlag) -> bool {
        self.bits & (1u128 << flag.0) != 0
    }

    /// Returns the set with `flag` set to `on`.
    pub const fn with(self, flag: QuirkFlag, on: bool) -> Self {
        let mask = 1u128 << flag.0;
        Self {
            bits: if on {
                self.bits | mask
            } else {
                self.bits & !mask
            },
        }
    }

    /// The raw bitset (for hashing and replay headers).
    pub const fn bits(self) -> u128 {
        self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: QuirkRegistry = QuirkRegistry::new(&["quirk.battle.a", "quirk.battle.b"]);

    #[test]
    fn registry_lookup_and_bitset() {
        assert_eq!(REGISTRY.len(), 60);
        assert_eq!(
            [
                "quirk.adventure.flight_reads_hidden",
                "quirk.adventure.door_reads_hidden",
                "quirk.adventure.scuttle_reads_hidden",
                "quirk.adventure.summon_boat_by_steps",
                "quirk.adventure.spell_recast_refused",
                "quirk.adventure.door_failure_charged",
                "quirk.content.rmg_8xm8_zone_ids",
                "quirk.rmg.roe_monster_town_byte",
                "quirk.rmg.unowned_start_seventh",
                "quirk.town.visitor_only_trades",
                "quirk.town.university_ignores_map_bans",
                "quirk.rmg.zone_memory_model",
                "quirk.rmg.terrain_towns_count",
                "quirk.rmg.partition_units",
                "quirk.rmg.cosmetic",
            ]
            .map(|name| REGISTRY.lookup(name)),
            [
                FLIGHT_READS_HIDDEN,
                DOOR_READS_HIDDEN,
                SCUTTLE_READS_HIDDEN,
                SUMMON_BOAT_BY_STEPS,
                SPELL_RECAST_REFUSED,
                DOOR_FAILURE_CHARGED,
                RMG_8XM8_ZONE_IDS,
                RMG_ROE_MONSTER_TOWN_BYTE,
                RMG_UNOWNED_START_SEVENTH,
                VISITOR_ONLY_TRADES,
                UNIVERSITY_IGNORES_MAP_BANS,
                RMG_ZONE_MEMORY_MODEL,
                RMG_TERRAIN_TOWNS_COUNT,
                RMG_PARTITION_UNITS,
                RMG_COSMETIC,
            ]
            .map(Some)
        );
        assert_eq!(
            [
                REGISTRY.lookup("quirk.town.creature_month_exact"),
                REGISTRY.lookup("quirk.town.neutral_plague_full_growth"),
            ],
            [Some(CREATURE_MONTH_EXACT), Some(NEUTRAL_PLAGUE_FULL_GROWTH)]
        );
        assert_eq!(
            [
                REGISTRY.lookup("quirk.town.leave_points_over_cleared_army"),
                REGISTRY.lookup("quirk.town.guild_artifacts_by_backpacks"),
            ],
            [
                Some(LEAVE_POINTS_OVER_CLEARED_ARMY),
                Some(GUILD_ARTIFACTS_BY_BACKPACKS)
            ]
        );
        assert_eq!(
            REGISTRY.lookup("quirk.town.aurora_ignores_map_bans"),
            Some(AURORA_IGNORES_MAP_BANS)
        );
        assert_eq!(
            [
                REGISTRY.lookup("quirk.adventure.roe_witch_hut_mask"),
                REGISTRY.lookup("quirk.adventure.full_backpack_refuses"),
            ],
            [Some(ROE_WITCH_HUT_MASK), Some(FULL_BACKPACK_REFUSES)]
        );
        assert_eq!(
            REGISTRY.lookup("quirk.adventure.sirens_again_after_battle"),
            Some(SIRENS_AGAIN_AFTER_BATTLE)
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
        assert_eq!(REGISTRY.len(), 60);
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
                .eq((0..60).filter(|i| ![9, 10, 14, 30].contains(i)))
        );
        // A registry without retired flags has them all.
        assert!(TEST.flags().map(QuirkFlag::index).eq(0..2));
    }
}
