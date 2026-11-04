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
}

impl Open {
    /// The dialog `since` ms after it opened, its creature's first step `first` ms after it
    /// (`creature_dialog_proc` 0x5F4CFF: once the clock reaches the frame deadline, then every
    /// 100 ms; none while a popup is held, E17 M6 T2a2b1).
    pub fn at(&self, first: u32, since: u32) -> MessageBox {
        let mut steps = self.steps.clone();
        for _ in 0..Steps::made_around(first, &self.holds, since) {
            steps.step();
        }
        let mut window = self.modal.open.message.clone();
        for item in &mut window.dialog.items {
            if let h3_ui::ui::Item::Def(d) = item
                && d.id == h3_view::stats::CREATURE
            {
                (d.group, d.frame) = (steps.group, steps.frame);
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
    // bonuses (Bloodlust, Precision).
    let melee = stack.attack + i32::from(stack.melee_attack_bonus);
    let ranged = stack.attack + i32::from(stack.ranged_attack_bonus);
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
    let factor = observation
        .sides
        .get(usize::from(stack.side == Side::Defender))
        .and_then(|s| s.hero.as_ref())
        .filter(|_| stack.war_machine)
        .map_or(1, |h| u16::from(h.attack) + 1);
    let damage = [stack.damage_min, stack.damage_max].map(|d| i32::from(d / factor) * times);
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
    })
}
