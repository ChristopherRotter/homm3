//! A stack's stats dialog in the viewer (E17 M6 T2a1; `docs/epics/E17-design.md` §12.1): what it
//! shows of a stack, from a seat's observation, its rules and the asset tables, for the capture
//! set's stills (`h3_view::stats` the dialog itself). *Provisional* (M6 T2a1's review list): the
//! numbers the executable reads from its own records are the observation's where it carries
//! them, the rest derived here as noted.

use std::collections::BTreeMap;

use h3_content_battle::BattleRuleset;
use h3_kernel::Handle;
use h3_protocol::wire::{Manifest, Observation, Side, Stack};
use h3_towns::TownRuleset;
use h3_ui::text::{Args, Catalogue};
use h3_view::message::MessageBox;
use h3_view::stats::{
    Account, Building, Described, Effect, Field, HeroList, Hints, Memory, Modal, Pair, Stats,
    StatsLayout, Steps, Texts,
};
use h3_view::tables::Tables;

/// The creatures the dialog tells by number (their original ids): the Ballista's damage, the war
/// machines' steps, the Faerie Dragon's cast button, the arrow tower's shots.
const BALLISTA: u32 = 146;
const ARROW_TOWER: u32 = 149;
const FAERIE_DRAGON: u32 = 134;
const WAR_MACHINES: core::ops::RangeInclusive<u32> = 145..=148;
/// The spells it tells by number (their original ids): those that change morale and luck, and
/// Forgetfulness, which at Advanced stops a shot.
const MIRTH: u32 = 49;
const SORROW: u32 = 50;
const FORTUNE: u32 = 51;
const MISFORTUNE: u32 = 52;
const FORGETFULNESS: u32 = 61;
/// The creatures morale's and luck's descriptions name (their original ids; design §12.4): the
/// Angels and Archangels in the army, the other army's Bone and Ghost Dragons and Devils and Arch
/// Devils, the Minotaurs' and the Halflings' minimums.
const ANGELS: [u32; 2] = [13, 12];
const DRAGONS: [u32; 2] = [69, 68];
const DEVILS: [u32; 2] = [55, 54];
const MINOTAURS: [u32; 2] = [78, 79];
const HALFLINGS: u32 = 138;
/// An army's slots (the original's seven); war machines and created stacks take others.
const ARMY_SLOTS: u8 = 7;

/// What the dialog needs to name and number a stack: the record's manifest (the handles' keys),
/// the rules, the asset tables and the catalogue.
pub struct Sources<'a> {
    /// The record's manifest.
    pub manifest: &'a Manifest,
    /// The rules the battle is played by.
    pub ruleset: &'a BattleRuleset,
    /// The asset tables.
    pub tables: &'a Tables,
    /// The catalogue.
    pub catalogue: &'a Catalogue,
    /// The town part the battle's packs compile to (E17 M6 T3c3: a siege's buildings and a
    /// hero's Grail building by their keys); none: the descriptions name no building.
    pub towns: Option<&'a TownRuleset>,
}

/// The dialog a left press (or the key T) opened on a stack: the modal dialog with its OK's
/// state, status line and cursor (E17 M6 T2a2a), its creature's first steps, and the point the
/// mouse was at (the battle's cursor and status line keep it: §12.1); or the one the right
/// button shows while held (E17 M6 T2b1, design §12.5: `held`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The dialog and what its input did to it.
    pub modal: Modal,
    /// Its creature as the dialog opened.
    pub steps: Steps,
    /// Where the mouse was as it opened.
    pub looked: [i32; 2],
    /// The right button's popups held over it (E17 M6 T2a2b1), each `[start, end]` in ms since
    /// it opened (`u32::MAX` while held): its creature does not step meanwhile
    /// ([`Steps::made_around`]).
    pub holds: Vec<[u32; 2]>,
    /// The right button's dialog over the battle (E17 M6 T2b1; `stack_stats_dialog` with its
    /// second argument 1, `dialog_show_held` 0x530D30): without its OK and cast button, shown
    /// while the button is held (`popup_run_held` 0x602A40: the cursor hidden, every input but
    /// its ends dropped, no procedure run, so no hint and no step), then closed as the modal one.
    pub held: bool,
    /// Its creature as last drawn ([`Self::at`]).
    pub drawn: Drawn,
}

/// The creature of an open dialog as last drawn (the reviewers' R1-2, R2-2): the steps made by
/// then and where they left it, so that each frame steps on from the last one rather than from
/// the opening (the steps are the same whatever the holds: only their count depends on them). A
/// cache: dialogs are equal whatever theirs hold.
#[derive(Debug, Clone, Default)]
pub struct Drawn(core::cell::RefCell<Option<(u32, Steps)>>);

impl PartialEq for Drawn {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Drawn {}

impl Open {
    /// The dialog `since` ms after it opened, its creature's first step `first` ms after it
    /// (`creature_dialog_proc` 0x5F4CFF: once the clock reaches the frame deadline, then every
    /// 100 ms; none while a popup is held, E17 M6 T2a2b1): stepped on from the frame drawn last,
    /// from the opening for an earlier one.
    pub fn at(&self, first: u32, since: u32) -> MessageBox {
        let made = Steps::made_around(first, &self.holds, since);
        let mut drawn = self.drawn.0.borrow_mut();
        let (mut count, mut steps) = match drawn.take() {
            Some((count, steps)) if count <= made => (count, steps),
            _ => (0, self.steps.clone()),
        };
        while count < made {
            steps.step();
            count += 1;
        }
        let (group, frame) = (steps.group, steps.frame);
        *drawn = Some((made, steps));
        drop(drawn);
        let mut window = self.modal.open.message.clone();
        for item in &mut window.dialog.items {
            if let h3_ui::ui::Item::Def(d) = item
                && d.id == h3_view::stats::CREATURE
            {
                (d.group, d.frame) = (group, frame);
            }
        }
        window
    }
}

/// The cursor over the open dialog at the mouse's `point` (E17 M6 T2a2a): the default cursor's
/// frame its moves set (the spell book's cursor DEF, `crdeflt.def`), until then the combat
/// cursor's pointer the press or the key set as it opened (0x476E97, 0x475D72).
pub fn cursor(
    layout: &h3_view::layout::BattleLayout,
    modal: &Modal,
    point: [i32; 2],
) -> h3_view::scene::Draw {
    match modal.cursor {
        Some(frame) => h3_view::scene::Draw::Cursor {
            def: layout.book.cursor.clone(),
            frame,
            x: point[0],
            y: point[1],
            hot: usize::try_from(frame)
                .ok()
                .and_then(|f| layout.book.hot.get(f).copied())
                .unwrap_or_default(),
        },
        None => h3_view::mouse::cursor(layout, layout.mouse.panel_frame, point),
    }
}

/// The original id of a spell handle (none: not in the manifest or the tables).
fn spell_original(sources: &Sources<'_>, handle: u16) -> Option<u32> {
    let key = sources.manifest.keys.spells.get(usize::from(handle))?;
    sources
        .tables
        .spells
        .get(key)
        .map(|s| u32::from(s.original))
}

/// Whether `stack` can shoot now (`stack_can_shoot_now` 0x4428F0): a Ballista or an arrow tower
/// always; else a shooter with shots, no enemy beside it unless it shoots then too, not forgetful
/// at Advanced or Expert.
fn can_shoot_now(
    sources: &Sources<'_>,
    observation: &Observation,
    stack: &Stack,
    original: u32,
) -> bool {
    if original == BALLISTA || original == ARROW_TOWER {
        return true;
    }
    if !stack.shooter || stack.shots == 0 {
        return false;
    }
    let forgetful = stack
        .effects
        .iter()
        .any(|e| spell_original(sources, e.spell) == Some(FORGETFULNESS) && e.mastery >= 2);
    if forgetful {
        return false;
    }
    if stack.shoots_when_adjacent {
        return true;
    }
    let own = hexes(stack);
    let blocked = observation.stacks.iter().any(|other| {
        other.alive
            && other.controller != stack.controller
            && hexes(other)
                .iter()
                .any(|&h| own.iter().any(|&o| beside(o, h)))
    });
    !blocked
}

/// A stack's hexes: its position, and a two-hex stack's other (the attacker's to the right, the
/// defender's to the left).
fn hexes(stack: &Stack) -> Vec<u8> {
    let mut out = vec![stack.hex];
    if stack.two_hex {
        let other = if stack.side == Side::Attacker {
            stack.hex.checked_add(1)
        } else {
            stack.hex.checked_sub(1)
        };
        out.extend(other);
    }
    out
}

/// Whether hexes `a` and `b` are neighbours.
fn beside(a: u8, b: u8) -> bool {
    (0..6).any(|d| h3_view::mouse::neighbour(a, d) == Some(b))
}

/// What the dialog shows of stack `id` in `observation` (`stack_stats_dialog_create` 0x5F3360),
/// `modal` for a left press's; none when the observation, the rules or the tables lack it.
pub fn of(sources: &Sources<'_>, observation: &Observation, id: u8, modal: bool) -> Option<Stats> {
    let stack = observation.stacks.iter().find(|s| s.id == id)?;
    let handle = Handle::from_raw(stack.creature);
    let creature = sources.ruleset.creature(handle)?;
    let core = &**sources.ruleset;
    let info = core.creature(handle)?;
    let key = sources
        .manifest
        .keys
        .creatures
        .get(usize::from(stack.creature))?;
    let entry = sources.tables.creatures.get(key)?;
    let original = u32::from(entry.original);
    let count = i64::from(stack.count);
    let name = sources
        .catalogue
        .text(&format!("creature.{key}"), &Args::new().count(count));
    let ability = sources
        .catalogue
        .text(&format!("creature-ability.{key}"), &Args::new());
    // The attack: the ranged one when it can shoot now, the higher of it and the melee one
    // (0x5F345B); the observation's attack holds its timed effects but the melee and ranged
    // bonuses (Bloodlust, Precision); saturated, a server's attack being any `i32` (the
    // reviewer's R1-1).
    let melee = stack
        .attack
        .saturating_add(i32::from(stack.melee_attack_bonus));
    let ranged = stack
        .attack
        .saturating_add(i32::from(stack.ranged_attack_bonus));
    let attack = if can_shoot_now(sources, observation, stack, original) {
        ranged.max(melee)
    } else {
        melee
    };
    let controller = usize::from(stack.controller == Side::Defender);
    let hero = observation
        .sides
        .get(controller)
        .and_then(|s| s.hero.as_ref());
    // A Ballista under a hero: each end times its attack, clamped to 0..99, plus 1 (0x5F58FD).
    // The wire's war machine's range holds its side's hero's factor already (the engine's
    // BAT-WAR-005: its attack plus 1, unclamped), the executable's stack the creature's own:
    // divided out first (E17 M6 T2b1: T2a1 multiplied the Ballista's twice, `held-machine`).
    let times = match hero {
        Some(h) if original == BALLISTA => i32::from(h.attack.min(99)) + 1,
        _ => 1,
    };
    let damage = own_damage(observation, stack).map(|d| d * times);
    // Morale and luck: the stack's own, Mirth's to Misfortune's changes in them (the engine's
    // materialised values, as the executable's +0x4E8 and +0x4EC; E17 M6 T2a2b2: T2a1 added the
    // timed effects a second time), before the roll levels' clamp as the executable's (CLT-Q40's
    // carry, E17 M6 T3c1: the clamped luck, 0 and up, showed the Devils' -1 as neutral).
    let morale = i32::from(stack.unclamped_morale);
    let luck = i32::from(stack.unclamped_luck);
    let effects = stack
        .effects
        .iter()
        .filter_map(|e| {
            let key = sources.manifest.keys.spells.get(usize::from(e.spell))?;
            Some(Effect {
                spell: spell_original(sources, e.spell)?,
                name: sources
                    .catalogue
                    .text(&format!("spell.{key}"), &Args::new()),
                rounds: e.rounds,
            })
        })
        .collect();
    // The cast button (0x5F3974): a Faerie Dragon of the side acting now, with casts left,
    // outside the tactics phase, in the modal dialog.
    let acting = observation
        .active
        .and_then(|a| observation.stacks.iter().find(|s| s.id == a))
        .map(|s| s.controller);
    let cast = modal
        && original == FAERIE_DRAGON
        && acting == Some(stack.controller)
        && stack.casts_left > 0
        && observation.tactics.is_none();
    let war_machine = WAR_MACHINES.contains(&original);
    let speed_now = if stack.war_machine {
        0
    } else {
        i32::from(stack.speed)
    };
    Some(Stats {
        name,
        def: entry.def.clone(),
        town: info.faction.and_then(|f| u8::try_from(f.index()).ok()),
        count: stack.count,
        attack: Pair {
            own: i32::from(creature.attack),
            now: attack,
        },
        defense: Pair {
            own: i32::from(creature.defense),
            now: stack.defense,
        },
        shots: stack.shooter.then(|| Pair {
            own: i32::from(creature.shots),
            now: i32::from(stack.shots),
        }),
        damage,
        health: Pair {
            own: i32::try_from(creature.hit_points).unwrap_or(i32::MAX),
            now: i32::try_from(stack.max_hp).unwrap_or(i32::MAX),
        },
        health_left: i32::try_from(stack.top_hp).unwrap_or(i32::MAX),
        speed: Pair {
            own: i32::from(info.speed),
            now: if war_machine { 0 } else { speed_now },
        },
        morale,
        luck,
        effects,
        account: Account::default(),
        ability: Some(ability),
        cast,
        modal,
    })
}

/// `stack`'s own damage range (the executable's +0xD0 and +0xD4: a war machine's its record's).
/// The wire's war machine's range holds its side's hero's factor (the engine's BAT-WAR-005: its
/// attack plus 1, unclamped), divided out here.
fn own_damage(observation: &Observation, stack: &Stack) -> [i32; 2] {
    let factor = observation
        .sides
        .get(usize::from(stack.side == Side::Defender))
        .and_then(|s| s.hero.as_ref())
        .filter(|_| stack.war_machine)
        .map_or(1, |h| u16::from(h.attack) + 1);
    [stack.damage_min, stack.damage_max].map(|d| i32::from(d / factor))
}

/// What stack `id`'s army info window shows in `observation` (E17 M6 T4b2b2, design §12.15
/// "Their fill"; `army_info_stack_set` 0x46DC30): the stats dialog's numbers ([`of`]), so the two
/// agree, the stack's own damage range for the original's Dmg row (CLT-Q48), the background in
/// the colours of the stack's side's hero's player (+0xF4's: its owner's, where the dialog takes
/// its controller's; oddity 8), else the viewing player's `viewing`. None without the stack, its
/// creature's rules or tables.
pub fn army_info(
    sources: &Sources<'_>,
    observation: &Observation,
    id: u8,
    viewing: u8,
) -> Option<h3_view::army_info::StackInfo> {
    let stats = of(sources, observation, id, false)?;
    let stack = observation.stacks.iter().find(|s| s.id == id)?;
    let key = sources
        .manifest
        .keys
        .creatures
        .get(usize::from(stack.creature))?;
    let creature = u32::from(sources.tables.creatures.get(key)?.original);
    let index = usize::from(stack.side == Side::Defender);
    let side = &observation.sides[index];
    let player = match side.hero {
        Some(_) => side
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0)),
        None => viewing,
    };
    let own = own_damage(observation, stack);
    Some(h3_view::army_info::StackInfo::of(
        &stats,
        (creature, player),
        own,
    ))
}

/// What morale's and luck's descriptions tell of `stack`'s sources (E17 M6 T2a2b2, design §12.4),
/// as far as `observation` shows them (CLT-Q40): the special field, the creature's town and
/// flags, the armies of the side whose sources they list (its controller's in the client's
/// profile, its owner's in `Bugs::Original`, as the executable's +0xF4: CLT-Q41 (2)), the
/// creatures' minimums and the timed spells' changes; none when the rules or the tables lack it.
fn account(
    sources: &Sources<'_>,
    observation: &Observation,
    stack: &Stack,
    bugs: h3_render::Bugs,
) -> Option<Account> {
    let core = &**sources.ruleset;
    let info = core.creature(Handle::from_raw(stack.creature))?;
    let key_of = |s: &Stack| sources.manifest.keys.creatures.get(usize::from(s.creature));
    let original_of = |s: &Stack| {
        key_of(s)
            .and_then(|k| sources.tables.creatures.get(k))
            .map(|e| u32::from(e.original))
    };
    // A creature's plural (the creature table's +0x18): its name for more than one.
    let plural = |s: &Stack| {
        key_of(s).map(|k| {
            sources
                .catalogue
                .text(&format!("creature.{k}"), &Args::new().count(2))
        })
    };
    let field = field(sources.manifest, observation);
    let listed = match bugs {
        h3_render::Bugs::Fixed => stack.controller,
        h3_render::Bugs::Original => stack.side,
    };
    // A side's army (+0x54C4): its slots' stacks as the battle began, dead ones too.
    let army = |side: Side| {
        observation.stacks.iter().filter(move |s| {
            s.side == side
                && s.slot < ARMY_SLOTS
                && !(s.created || s.raised || s.clone || s.war_machine)
        })
    };
    let other = if listed == Side::Attacker {
        Side::Defender
    } else {
        Side::Attacker
    };
    // The first of `kinds` the army of `side` holds: its plural.
    let first_of = |side: Side, kinds: &[u32]| {
        kinds.iter().find_map(|&kind| {
            army(side)
                .find(|s| original_of(s) == Some(kind))
                .and_then(plural)
        })
    };
    // The Angelic Alliance's join (an `AlignmentGroup` on the listed side's hero): its factions
    // count as one alignment (E17 M6 T3c2, design §12.10).
    let joined: Vec<usize> = observation.morale_and_luck[usize::from(listed == Side::Defender)]
        .iter()
        .filter_map(|e| match e {
            h3_protocol::wire::MoraleLuck::AlignmentGroup { factions, .. } => Some(factions),
            _ => None,
        })
        .flatten()
        .map(|&f| usize::from(f))
        .collect();
    // Each creature by the engine's faction. *Provisional* (the reviewer's R2-7): in a map of
    // kind 0 (`[game]+0x1F698`, RoE) the executable takes the four elementals (112 to 115) as
    // neutral here, for Clover Field's line and for the creature's town (design §12.4); the
    // engine carries no map kind, so the client follows its faction.
    let alignments: std::collections::BTreeSet<Option<usize>> = army(listed)
        .filter_map(|s| core.creature(Handle::from_raw(s.creature)))
        .map(|i| {
            i.faction.map(|f| {
                if joined.contains(&f.index()) {
                    joined.iter().copied().min().unwrap_or(f.index())
                } else {
                    f.index()
                }
            })
        })
        .collect();
    let undead = army(listed)
        .filter_map(|s| core.creature(Handle::from_raw(s.creature)))
        .any(|i| i.flags.contains(h3_content::CreatureFlag::Undead));
    let original = original_of(stack);
    let by = |spell: u32| -> i32 {
        stack
            .effects
            .iter()
            .filter(|e| spell_original(sources, e.spell) == Some(spell))
            .map(|e| i32::from(e.value))
            .sum()
    };
    Some(Account {
        field,
        town: info.faction.and_then(|f| u8::try_from(f.index()).ok()),
        no_morale: info.flags.contains(h3_content::CreatureFlag::NoMorale),
        alignments: u32::try_from(alignments.len()).unwrap_or(u32::MAX),
        undead,
        angels: first_of(listed, &ANGELS),
        dragons: first_of(other, &DRAGONS),
        devils: first_of(other, &DEVILS),
        morale_minimum: original
            .filter(|o| MINOTAURS.contains(o))
            .and_then(|_| plural(stack)),
        luck_minimum: original
            .filter(|&o| o == HALFLINGS)
            .and_then(|_| plural(stack)),
        morale_spells: by(MIRTH) - by(SORROW),
        luck_spells: by(FORTUNE) - by(MISFORTUNE),
        hourglass: cap_artifact(sources, observation, listed, Kind::Luck),
        oppression: cap_artifact(sources, observation, listed, Kind::Morale),
        hero_morale: with_grail(
            hero_list(sources.manifest, observation, listed, Kind::Morale),
            sources,
            (observation, listed, Kind::Morale),
        ),
        hero_luck: with_grail(
            hero_list(sources.manifest, observation, listed, Kind::Luck),
            sources,
            (observation, listed, Kind::Luck),
        ),
        siege_morale: siege_buildings(sources, observation, listed, Kind::Morale),
        siege_luck: siege_buildings(sources, observation, listed, Kind::Luck),
    })
}

/// The name of the artifact whose cap on `kind` reaches the stacks of `side` (either hero's:
/// the Hourglass of the Evil Hour's `LuckAtMost`, the Spirit of Oppression's `MoraleAtMost`;
/// E17 M6 T3c2, design §12.10): its name from the catalogue by its original id (ARTRAITS).
fn cap_artifact(
    sources: &Sources<'_>,
    observation: &Observation,
    side: Side,
    kind: Kind,
) -> Option<String> {
    use h3_protocol::wire::{Affects, MoraleLuck, Source};
    for (index, mine) in [(0usize, Side::Attacker), (1, Side::Defender)] {
        for effect in &observation.morale_and_luck[index] {
            let (affects, source) = match (effect, kind) {
                (
                    MoraleLuck::MoraleAtMost {
                        affects, source, ..
                    },
                    Kind::Morale,
                )
                | (
                    MoraleLuck::LuckAtMost {
                        affects, source, ..
                    },
                    Kind::Luck,
                ) => (affects, source),
                _ => continue,
            };
            let reaches = match affects {
                Affects::Both => true,
                Affects::Own => mine == side,
                Affects::Enemy => mine != side,
            };
            if !reaches {
                continue;
            }
            let original = match source {
                Source::Artifact { artifact } => sources
                    .manifest
                    .keys
                    .artifacts
                    .get(usize::from(*artifact))
                    .and_then(|k| k.rsplit(':').next())
                    .and_then(|name| CAP_ARTIFACTS.iter().find(|(key, _)| *key == name))
                    .map(|&(_, id)| id),
                _ => None,
            };
            if let Some(id) = original {
                return Some(
                    sources
                        .catalogue
                        .text(&format!("artifact.{id}"), &Args::new()),
                );
            }
        }
    }
    None
}

/// The artifacts that cap morale or luck, by their keys' names in the packs, with their original
/// ids: the Spirit of Oppression and the Hourglass of the Evil Hour.
const CAP_ARTIFACTS: [(&str, u32); 2] = [
    ("spirit_of_oppression", 84),
    ("hourglass_of_the_evil_hour", 85),
];

/// The key of building `index` in faction handle `faction`'s town record (E17 M6 T3c3): the
/// wire's building indices are the engine's (`h3_towns::BuildingId`), so the packs' town part
/// names them; none without it.
fn building_key<'a>(sources: &Sources<'a>, faction: u16, index: u8) -> Option<&'a str> {
    let record = sources
        .towns?
        .town(h3_content::FactionId::from_raw(faction))?;
    let building = record.buildings.get(usize::from(index))?;
    Some(building.key.as_str())
}

/// The siege town's buildings that change the defender's `kind` (E17 M6 T3c2, design §12.4): the
/// town is passed to the defender's stacks only (`listed` the side whose sources the description
/// lists); the Tavern (morale), a Castle's Brotherhood of the Sword (morale), a Rampart's
/// Fountain of Fortune (luck), each from `Siege.bonuses` by its key in the town record (T3c3), in
/// the original's order. The observation shows nothing else of a town's buildings (CLT-Q40).
fn siege_buildings(
    sources: &Sources<'_>,
    observation: &Observation,
    listed: Side,
    kind: Kind,
) -> Vec<Building> {
    let Some(siege) = observation
        .siege
        .as_ref()
        .filter(|_| listed == Side::Defender)
    else {
        return Vec::new();
    };
    let changing: Vec<&str> = siege
        .bonuses
        .iter()
        .filter(|b| match kind {
            Kind::Morale => b.morale != 0,
            Kind::Luck => b.luck != 0,
        })
        .filter_map(|b| building_key(sources, siege.faction, b.building))
        .collect();
    let order: &[(&str, Building)] = match kind {
        Kind::Morale => &[
            ("tavern", Building::Tavern),
            ("brotherhood_of_the_sword", Building::Brotherhood),
        ],
        Kind::Luck => &[("fountain_of_fortune", Building::Fountain)],
    };
    order
        .iter()
        .filter(|(key, _)| changing.contains(key))
        .map(|&(_, building)| building)
        .collect()
}

/// `list` with the owner's Grail building (E17 M6 T3c2, design §12.9) its changes name: a
/// `Source::Building` of side `listed`'s hero (the first town of its owner with the Grail's
/// building standing, as the engine names it) that is a Castle's Colossus for morale or a
/// Rampart's Spirit Guardian for luck, by its key in the town record (T3c3); its +2 is in
/// `list.total` already.
fn with_grail(
    mut list: HeroList,
    sources: &Sources<'_>,
    (observation, listed, kind): (&Observation, Side, Kind),
) -> HeroList {
    use h3_protocol::wire::{MoraleLuck, Source};
    let effects = &observation.morale_and_luck[usize::from(listed == Side::Defender)];
    for effect in effects {
        let source = match (effect, kind) {
            (MoraleLuck::Morale { source, .. }, Kind::Morale)
            | (MoraleLuck::Luck { source, .. }, Kind::Luck) => source,
            _ => continue,
        };
        let Source::Building { faction, index } = *source else {
            continue;
        };
        list.grail = match (building_key(sources, faction, index), kind) {
            (Some("colossus"), Kind::Morale) => Some(Building::Colossus),
            (Some("spirit_guardian"), Kind::Luck) => Some(Building::SpiritGuardian),
            _ => list.grail,
        };
    }
    list
}

/// Which of a hero's lists.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Morale,
    Luck,
}

/// The artifacts the hero's lists name (design §12.9), by their keys' names in the packs, with
/// their original ids.
const LISTED_ARTIFACTS: [(&str, u32); 8] = [
    ("still_eye_of_the_dragon", 45),
    ("clover_of_fortune", 46),
    ("cards_of_prophecy", 47),
    ("ladybird_of_luck", 48),
    ("badge_of_courage", 49),
    ("crest_of_valor", 50),
    ("glyph_of_gallantry", 51),
    ("pendant_of_courage", 108),
];

/// Side `side`'s hero's morale or luck list (`kind`; E17 M6 T3c1, design §12.9) from
/// `observation`'s `morale_and_luck` (CLT-Q40's carry): its changes that reach its own stacks
/// summed, the listed artifacts and the skill (Leadership, Luck) they name by their handles' keys
/// in `manifest`; empty without a hero.
fn hero_list(manifest: &Manifest, observation: &Observation, side: Side, kind: Kind) -> HeroList {
    use h3_protocol::wire::{Affects, MoraleLuck, Source};
    let name = |keys: &[String], handle: u16| {
        keys.get(usize::from(handle))
            .and_then(|k| k.rsplit(':').next())
            .map(str::to_owned)
    };
    let skill_name = match kind {
        Kind::Morale => "leadership",
        Kind::Luck => "luck",
    };
    let mut list = HeroList::default();
    let effects = &observation.morale_and_luck[usize::from(side == Side::Defender)];
    for effect in effects {
        let (amount, affects, source) = match (effect, kind) {
            (
                &MoraleLuck::Morale {
                    amount,
                    affects,
                    source,
                },
                Kind::Morale,
            )
            | (
                &MoraleLuck::Luck {
                    amount,
                    affects,
                    source,
                },
                Kind::Luck,
            ) => (amount, affects, source),
            _ => continue,
        };
        if !matches!(affects, Affects::Own | Affects::Both) {
            continue;
        }
        list.total += i32::from(amount);
        match source {
            Source::Skill { skill, level } => {
                if name(&manifest.keys.skills, skill).as_deref() == Some(skill_name) {
                    list.skill = level;
                }
            }
            Source::Artifact { artifact } => {
                let original = name(&manifest.keys.artifacts, artifact).and_then(|n| {
                    LISTED_ARTIFACTS
                        .iter()
                        .find(|(key, _)| *key == n)
                        .map(|&(_, id)| id)
                });
                if let Some(id) = original.filter(|id| !list.artifacts.contains(id)) {
                    list.artifacts.push(id);
                }
            }
            _ => {}
        }
    }
    list
}

/// The special field `observation`'s battle is fought on, by its battlefield's key in
/// `manifest` (the original's +0x53C0: 2 Cursed Ground, 3 Holy Ground, 4 Evil Fog, 5 Clover
/// Field; E17 M6 T2a2b2): morale's and luck's descriptions, the heroes' info windows (T3b2).
pub fn field(manifest: &Manifest, observation: &Observation) -> Field {
    observation
        .battlefield
        .and_then(|h| manifest.keys.battlefields.get(usize::from(h)))
        .map_or(Field::Other, |key| match key.rsplit(':').next() {
            Some("cursed_ground") => Field::Cursed,
            Some("holy_ground") => Field::Holy,
            Some("evil_fog") => Field::EvilFog,
            Some("clover_field") => Field::Clover,
            _ => Field::Other,
        })
}

/// The player whose colours the dialog's background takes (0x5F3550): the owner of the hero who
/// controls the stack, else the viewing player (`viewing`).
pub fn player(observation: &Observation, stack: &Stack, viewing: u8) -> u8 {
    let index = usize::from(stack.controller == Side::Defender);
    let side = &observation.sides[index];
    match side.hero {
        Some(_) => side
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0)),
        None => viewing,
    }
}

/// The dialog a left press (or the key T) opens on stack `id` (its place next to the stack's hex,
/// its creature's steps from the frame counts `frames` of its DEF's groups; `seed` the viewer's
/// generator's), the mouse at `looked`, or with `held` the one the right button shows while held
/// (E17 M6 T2b1: no OK, no cast button, its creature standing); drawn and hinted in `memory`'s
/// profile, which its opening may make forget the item last hovered; morale's and luck's
/// descriptions with the pictures' sizes `sizes` (their DEFs the layout's `morale_described` and
/// `luck_described`).
pub fn open(
    sources: &Sources<'_>,
    observation: &Observation,
    (id, looked, held): (u8, [i32; 2], bool),
    (layout, anchor, sizes): (&StatsLayout, [i32; 2], [[i32; 2]; 2]),
    (frames, viewing, seed): (BTreeMap<u32, u32>, u8, u32),
    memory: &mut Memory,
) -> Option<Open> {
    let mut stats = of(sources, observation, id, !held)?;
    // The cast button only for the acting stack in the client's profile (CLT-Q38 (4), answered
    // at E17 M6 T2a2a's start: its cast is the acting stack's own drawn spell); the original's
    // shows it for any Faerie Dragon of the side acting, for the comparisons.
    if memory.bugs() == h3_render::Bugs::Fixed && observation.active != Some(id) {
        stats.cast = false;
    }
    let stack = observation.stacks.iter().find(|s| s.id == id)?;
    let key = sources
        .manifest
        .keys
        .creatures
        .get(usize::from(stack.creature))?;
    let original = sources
        .tables
        .creatures
        .get(key)
        .map(|e| u32::from(e.original));
    let war_machine = original.is_some_and(|o| WAR_MACHINES.contains(&o));
    let texts = Texts::of(sources.catalogue);
    let bugs = memory.bugs();
    stats.account = account(sources, observation, stack, bugs)?;
    let window = h3_view::stats::layout(
        layout,
        &texts,
        &stats,
        (
            h3_view::stats::place(anchor),
            player(observation, stack, viewing),
        ),
        bugs,
    );
    let pictures: Described = [
        (layout.morale_described.clone(), sizes[0]),
        (layout.luck_described.clone(), sizes[1]),
    ];
    let hints = Hints::of(sources.catalogue, &stats, &pictures, bugs);
    Some(Open {
        modal: Modal::new(window, hints, memory),
        steps: Steps::new(frames, war_machine, seed),
        looked,
        // The held one's procedure never runs (`popup_run_held`): held from its opening on.
        holds: if held {
            vec![[0, u32::MAX]]
        } else {
            Vec::new()
        },
        held,
        drawn: Drawn::default(),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use h3_protocol::wire::Tactics;
    use h3_render::Bugs;

    /// `stats-dragon`'s battle at its opening (the Faerie Dragons to act, Red human, beside
    /// Marksmen; the Wolf Riders and Goblins against them): its ruleset, the attacker's own
    /// observation and its record's manifest.
    pub(crate) fn opening() -> (BattleRuleset, Observation, Manifest) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (ruleset, setup) = crate::scenario::battle(
            &dir.join("situations/stats-dragon.toml"),
            &dir.join("../packs"),
            1,
        )
        .unwrap();
        let mut live = crate::live::Live::new(
            &ruleset,
            setup,
            1,
            ([true, false], crate::live::Computer::Greedy),
        )
        .unwrap();
        let observation = live.own_view().unwrap();
        let record = live.record().unwrap();
        let manifest = h3_view::record::parse(&record).unwrap().header.manifest;
        (ruleset, observation, manifest)
    }

    pub(crate) fn tables() -> Tables {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        crate::assets::load(&dir.join("assets"), crate::assets::ALL).unwrap()
    }

    /// The handle of the creature or spell `key` in `keys` (the manifest's).
    pub(crate) fn handle(keys: &[String], key: &str) -> u16 {
        let index = keys.iter().position(|k| k == key).unwrap();
        u16::try_from(index).unwrap()
    }

    /// The id of the stack whose creature is `key`.
    pub(crate) fn stack_of(manifest: &Manifest, o: &Observation, key: &str) -> u8 {
        let creature = handle(&manifest.keys.creatures, key);
        (o.stacks.iter())
            .find(|s| s.creature == creature)
            .unwrap()
            .id
    }

    fn stack(o: &mut Observation, id: u8) -> &mut Stack {
        o.stacks.iter_mut().find(|s| s.id == id).unwrap()
    }

    /// A timed effect of spell handle `spell` at `mastery` with `value`.
    fn effect(spell: u16, mastery: u8, value: u16) -> h3_protocol::wire::Effect {
        h3_protocol::wire::Effect {
            spell,
            rounds: Some(2),
            mastery,
            value,
            attack: 0,
            defense: 0,
            speed: 0,
        }
    }

    #[test]
    fn the_attack_saturates_on_a_server_s_values() {
        // The reviewer's R1-1: a server's attack at the `i32`'s top with its bonuses.
        let (ruleset, mut o, manifest) = opening();
        let (tables, catalogue) = (tables(), Catalogue::default());
        let sources = Sources {
            manifest: &manifest,
            ruleset: &ruleset,
            tables: &tables,
            catalogue: &catalogue,
            towns: None,
        };
        let marksmen = stack_of(&manifest, &o, "base:marksman");
        let s = stack(&mut o, marksmen);
        (s.attack, s.melee_attack_bonus, s.ranged_attack_bonus) = (i32::MAX, 200, 200);
        assert_eq!(
            of(&sources, &o, marksmen, true).unwrap().attack.now,
            i32::MAX
        );
        let info = army_info(&sources, &o, marksmen, 0).unwrap();
        assert_eq!(info.attack.now, i32::MAX);
    }

    #[test]
    fn the_attack_shown_is_the_ranged_one_while_the_stack_can_shoot() {
        // The reviewer's R1-4 (`stack_can_shoot_now` 0x4428F0): the higher of the ranged and the
        // melee attack while it can shoot, else the melee one.
        let (ruleset, mut o, manifest) = opening();
        let (tables, catalogue) = (tables(), Catalogue::default());
        let sources = Sources {
            manifest: &manifest,
            ruleset: &ruleset,
            tables: &tables,
            catalogue: &catalogue,
            towns: None,
        };
        let marksmen = stack_of(&manifest, &o, "base:marksman");
        let goblins = stack_of(&manifest, &o, "base:goblin");
        let forgetfulness = handle(&manifest.keys.spells, "base:forgetfulness");
        let s = stack(&mut o, marksmen);
        (s.attack, s.melee_attack_bonus, s.ranged_attack_bonus) = (10, 0, 3);
        (s.shooter, s.shots, s.shoots_when_adjacent) = (true, 5, false);
        let hex = s.hex;
        let attack = |o: &Observation| of(&sources, o, marksmen, true).unwrap().attack.now;
        // Free to shoot: the ranged one; the melee one when it is higher.
        assert_eq!(attack(&o), 13);
        stack(&mut o, marksmen).melee_attack_bonus = 5;
        assert_eq!(attack(&o), 15);
        stack(&mut o, marksmen).melee_attack_bonus = 0;
        // No shots: the melee one.
        stack(&mut o, marksmen).shots = 0;
        assert_eq!(attack(&o), 10);
        stack(&mut o, marksmen).shots = 5;
        // Forgetful at Advanced or Expert: the melee one; at Basic it shoots.
        stack(&mut o, marksmen).effects = vec![effect(forgetfulness, 2, 0)];
        assert_eq!(attack(&o), 10);
        stack(&mut o, marksmen).effects = vec![effect(forgetfulness, 1, 0)];
        assert_eq!(attack(&o), 13);
        stack(&mut o, marksmen).effects.clear();
        // An enemy beside it: the melee one, unless it shoots then too; a dead one blocks nothing.
        let near = |h: u8| (0..6).filter_map(move |d| h3_view::mouse::neighbour(h, d));
        let next = near(hex).next().unwrap();
        let g = stack(&mut o, goblins);
        (g.hex, g.two_hex) = (next, false);
        assert_eq!(attack(&o), 10);
        stack(&mut o, marksmen).shoots_when_adjacent = true;
        assert_eq!(attack(&o), 13);
        stack(&mut o, marksmen).shoots_when_adjacent = false;
        stack(&mut o, goblins).alive = false;
        assert_eq!(attack(&o), 13);
        stack(&mut o, goblins).alive = true;
        // A two-hex enemy whose other hex (the defender's to the left) is beside it: blocked.
        let right = near(hex)
            .find(|&b| {
                b.checked_add(1)
                    .is_some_and(|r| r != hex && !beside(hex, r))
            })
            .unwrap();
        let g = stack(&mut o, goblins);
        (g.hex, g.two_hex) = (right + 1, true);
        assert_eq!(attack(&o), 10);
        stack(&mut o, goblins).two_hex = false;
        assert_eq!(attack(&o), 13);
        // Its own other hex (the attacker's to the right) beside an enemy: blocked.
        let far = near(hex + 1)
            .find(|&h| h != hex && !beside(hex, h))
            .unwrap();
        stack(&mut o, goblins).hex = far;
        assert_eq!(attack(&o), 13);
        stack(&mut o, marksmen).two_hex = true;
        assert_eq!(attack(&o), 10);
        // A Ballista shoots whatever stands beside it: the ranged one.
        let ballista = handle(&manifest.keys.creatures, "base:ballista");
        let s = stack(&mut o, marksmen);
        (s.creature, s.shooter, s.shots) = (ballista, false, 0);
        assert_eq!(attack(&o), 13);
    }

    #[test]
    fn the_cast_button_is_a_faerie_dragon_s_of_the_side_acting() {
        // The reviewer's R1-4 (0x5F3974): a Faerie Dragon of the side acting now, with casts
        // left, outside the tactics phase, in the modal dialog; the client's profile only the
        // acting stack's own (CLT-Q38 (4), `open`).
        let (ruleset, mut o, manifest) = opening();
        let (tables, catalogue) = (tables(), Catalogue::default());
        let sources = Sources {
            manifest: &manifest,
            ruleset: &ruleset,
            tables: &tables,
            catalogue: &catalogue,
            towns: None,
        };
        let dragons = stack_of(&manifest, &o, "ab:faerie_dragon");
        let marksmen = stack_of(&manifest, &o, "base:marksman");
        let goblins = stack_of(&manifest, &o, "base:goblin");
        o.active = Some(dragons);
        stack(&mut o, dragons).casts_left = 1;
        let cast = |o: &Observation, modal: bool| of(&sources, o, dragons, modal).unwrap().cast;
        assert!(cast(&o, true));
        assert!(!cast(&o, false));
        assert!(!of(&sources, &o, marksmen, true).unwrap().cast);
        // The Marksmen acting: still the side's; the other side's stack: not.
        o.active = Some(marksmen);
        assert!(cast(&o, true));
        o.active = Some(goblins);
        assert!(!cast(&o, true));
        o.active = None;
        assert!(!cast(&o, true));
        o.active = Some(dragons);
        stack(&mut o, dragons).casts_left = 0;
        assert!(!cast(&o, true));
        stack(&mut o, dragons).casts_left = 1;
        o.tactics = Some(Tactics {
            side: Side::Attacker,
            columns: 3,
        });
        assert!(!cast(&o, true));
        o.tactics = None;
        // The dialog opened on them while the Marksmen act: the original's shows it, the
        // client's not.
        o.active = Some(marksmen);
        let shown = |bugs: Bugs| {
            let layout =
                h3_view::layout::parse_battle(include_str!("../../../../../layouts/battle.toml"))
                    .unwrap();
            let mut memory = Memory::new(bugs);
            let open = open(
                &sources,
                &o,
                (dragons, [0, 0], false),
                (&layout.stats, [300, 300], [[0, 0]; 2]),
                (BTreeMap::new(), 0, 1),
                &mut memory,
            )
            .unwrap();
            let items = &open.modal.open.message.dialog.items;
            items.iter().any(|i| i.id() == h3_view::stats::CAST)
        };
        assert!(shown(Bugs::Original));
        assert!(!shown(Bugs::Fixed));
    }

    #[test]
    fn the_descriptions_list_the_side_by_profile_and_its_army() {
        // The reviewer's R2-6 (design §12.4, CLT-Q41 (2)): the Goblins hypnotized by the
        // attacker list its army in the client's profile, their own side's in the original's;
        // an army its slots' stacks, dead ones kept, war machines and stacks made in the battle
        // left out; the other army's Devils; the alignments, undead, the minimums, the spells.
        let (ruleset, mut o, manifest) = opening();
        let (tables, catalogue) = (tables(), Catalogue::default());
        let sources = Sources {
            manifest: &manifest,
            ruleset: &ruleset,
            tables: &tables,
            catalogue: &catalogue,
            towns: None,
        };
        let creature = |key: &str| handle(&manifest.keys.creatures, key);
        let spell = |key: &str| handle(&manifest.keys.spells, key);
        let plural = |key: &str| catalogue.text(&format!("creature.{key}"), &Args::new().count(2));
        let dragons = stack_of(&manifest, &o, "ab:faerie_dragon");
        let marksmen = stack_of(&manifest, &o, "base:marksman");
        let wolves = stack_of(&manifest, &o, "base:wolf_rider");
        let goblins = stack_of(&manifest, &o, "base:goblin");
        // The attacker's: Arch Devils (dead, kept) for the dragons, Marksmen; an Archangel war
        // machine and created Angels left out. The defender's: Skeletons, the Goblins.
        let s = stack(&mut o, dragons);
        (s.creature, s.alive) = (creature("base:arch_devil"), false);
        let template = stack(&mut o, marksmen).clone();
        let next = o.stacks.iter().map(|s| s.id).max().unwrap() + 1;
        o.stacks.push(Stack {
            id: next,
            creature: creature("base:archangel"),
            slot: 3,
            war_machine: true,
            ..template.clone()
        });
        o.stacks.push(Stack {
            id: next + 1,
            creature: creature("base:angel"),
            slot: 4,
            created: true,
            ..template.clone()
        });
        o.stacks.push(Stack {
            id: next + 2,
            creature: creature("base:angel"),
            slot: 7,
            ..template
        });
        stack(&mut o, wolves).creature = creature("base:skeleton");
        let g = stack(&mut o, goblins);
        g.controller = Side::Attacker;
        g.effects = vec![
            effect(spell("base:mirth"), 1, 3),
            effect(spell("base:sorrow"), 1, 1),
            effect(spell("base:misfortune"), 1, 2),
        ];
        let goblin = o.stacks.iter().find(|s| s.id == goblins).unwrap().clone();
        let fixed = account(&sources, &o, &goblin, Bugs::Fixed).unwrap();
        assert_eq!(fixed.alignments, 2);
        assert!(!fixed.undead);
        assert_eq!((fixed.angels, fixed.devils), (None, None));
        let original = account(&sources, &o, &goblin, Bugs::Original).unwrap();
        assert_eq!(original.alignments, 2);
        assert!(original.undead);
        assert_eq!(original.devils, Some(plural("base:arch_devil")));
        assert_eq!(original.angels, None);
        // The stack's own: its town, its spells.
        assert_eq!((fixed.town, original.town), (Some(6), Some(6)));
        assert_eq!((fixed.morale_spells, fixed.luck_spells), (2, -2));
        assert_eq!((fixed.morale_minimum, fixed.luck_minimum), (None, None));
        // Angels in a slot of the army: listed; Devils beside Arch Devils: the Arch Devils first.
        stack(&mut o, next + 1).created = false;
        let marksman = o.stacks.iter().find(|s| s.id == marksmen).unwrap().clone();
        let listed = account(&sources, &o, &marksman, Bugs::Fixed).unwrap();
        assert_eq!(listed.angels, Some(plural("base:angel")));
        assert_eq!(listed.devils, None);
        assert_eq!(listed.alignments, 2);
        // Undead in the defender's army, the Goblins its own still though the attacker's to
        // command.
        let skeleton = o.stacks.iter().find(|s| s.id == wolves).unwrap().clone();
        let defending = account(&sources, &o, &skeleton, Bugs::Fixed).unwrap();
        assert!(defending.undead);
        assert_eq!(defending.alignments, 2);
        assert_eq!(defending.devils, Some(plural("base:arch_devil")));
        // The minimums: the Minotaurs' morale, the Halflings' luck.
        let s = stack(&mut o, goblins);
        s.creature = creature("base:minotaur");
        let minotaur = s.clone();
        let minimum = account(&sources, &o, &minotaur, Bugs::Fixed).unwrap();
        assert_eq!(minimum.morale_minimum, Some(plural("base:minotaur")));
        let s = stack(&mut o, goblins);
        s.creature = creature("ab:halfling");
        let halfling = s.clone();
        let minimum = account(&sources, &o, &halfling, Bugs::Fixed).unwrap();
        assert_eq!(minimum.luck_minimum, Some(plural("ab:halfling")));
    }

    #[test]
    fn the_creature_steps_on_from_the_frame_drawn_last() {
        // The reviewers' R1-2 and R2-2: each frame steps on from the last one drawn, as from the
        // opening; an earlier frame, or a popup held later, as from the opening too.
        let (ruleset, o, manifest) = opening();
        let (tables, catalogue) = (tables(), Catalogue::default());
        let sources = Sources {
            manifest: &manifest,
            ruleset: &ruleset,
            tables: &tables,
            catalogue: &catalogue,
            towns: None,
        };
        let layout =
            h3_view::layout::parse_battle(include_str!("../../../../../layouts/battle.toml"))
                .unwrap();
        let marksmen = stack_of(&manifest, &o, "base:marksman");
        let frames: BTreeMap<u32, u32> = [(0, 8), (1, 6), (2, 4), (3, 5), (4, 7), (20, 2), (21, 2)]
            .into_iter()
            .collect();
        let mut memory = Memory::new(Bugs::Fixed);
        let mut open = open(
            &sources,
            &o,
            (marksmen, [0, 0], false),
            (&layout.stats, [300, 300], [[0, 0]; 2]),
            (frames, 0, 7),
            &mut memory,
        )
        .unwrap();
        let creature = |window: &MessageBox| {
            window.dialog.items.iter().find_map(|item| match item {
                h3_ui::ui::Item::Def(d) if d.id == h3_view::stats::CREATURE => {
                    Some((d.group, d.frame))
                }
                _ => None,
            })
        };
        let fresh = |open: &Open, since: u32| {
            let fresh = Open {
                drawn: Drawn::default(),
                ..open.clone()
            };
            creature(&fresh.at(100, since))
        };
        let mut seen = std::collections::BTreeSet::new();
        for since in [0, 100, 350, 350, 2_000, 900, 60_000, 60_050, 61_000] {
            let shown = creature(&open.at(100, since));
            assert_eq!(shown, fresh(&open, since), "{since}");
            seen.insert(shown);
        }
        assert!(seen.len() > 3);
        open.holds.push([61_500, 70_000]);
        for since in [61_600, 69_000, 70_000, 75_000, 1_000] {
            assert_eq!(
                creature(&open.at(100, since)),
                fresh(&open, since),
                "{since}"
            );
        }
        // A cache: equal whatever it holds.
        assert_eq!(
            open,
            Open {
                drawn: Drawn::default(),
                ..open.clone()
            }
        );
    }
}
