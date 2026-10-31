//! The battle's result window from the battle's end (E17 M5 T4b1; `docs/epics/E17-design.md`
//! §7.20): what `h3_view::result` lays out, taken from the engine's outcome (`BattleOutcome`: the
//! result, every stack's start and end), the record's opening (the heroes, a town, the human
//! sides), the asset tables (the portraits, the creatures' original ids), the ruleset (hit points,
//! AI values) and the catalogue. Shared by the window's battle (`crate::battle`) and the capture
//! set's stills and sequences (`crate::spellbook`, `crate::sequences`).

use std::collections::BTreeMap;

use h3_content::{CreatureTypeId, Ruleset};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, BattleOutcome, BattleResult, BattleSide, StackOrigin, StackRecord};
use h3_protocol::wire::{Manifest, Observation};
use h3_render::Bugs;
use h3_ui::text::{Args, Catalogue};
use h3_view::layout::BattleLayout;
use h3_view::message::MessageBox;
use h3_view::result::{self, Face, GaveUp, Side, Texts};
use h3_view::sound::Tune;
use h3_view::tables::Tables;

/// What the window is made from besides the battle: the record's manifest, the asset tables, the
/// ruleset, the catalogue and the layout.
pub struct Sources<'a> {
    /// The record's manifest (its keys).
    pub manifest: &'a Manifest,
    /// The asset tables.
    pub tables: &'a Tables,
    /// The battle's ruleset when its creatures' hit points and AI values are the battle's: a
    /// battle fought here's, a server's game's compiled here when the content check finds it the
    /// same (E17 M5 T8b2, R5b-2); none: the window reads no creature's values ([`window`]).
    pub ruleset: Option<&'a BattleRuleset>,
    /// The catalogue.
    pub catalogue: &'a Catalogue,
    /// The battle's layout.
    pub layout: &'a BattleLayout,
    /// The profile: the client's tells only the side that gave up that it fled or surrendered
    /// (CLT-Q35, [`window`]).
    pub bugs: Bugs,
}

/// The result window of the battle that opened as `opening` (the record's first snapshot) and
/// ended as `outcome`, shown as `battle_end` 0x477470 shows it: the winner's side when its player
/// is a human here (`opening`'s sides' `human`), else the other one; its background in the
/// colours of `player` (the viewing player's); with the tune it plays, which picks its clip too
/// (E17 M5 T4b2b, [`tune`]). The winner is [`result::winner`]'s: none after a side retreated or
/// surrendered against an opponent without army survivors (BAT-END-001; [`survivors`]).
///
/// Always a window (E17 M5 T8b2, R5b-1 and R5b-5): what the client lacks is drawn empty. A hero
/// whose type the manifest does not name is still a hero (its +500 counts, the text names it) with
/// an empty name and no portrait; a hero type without a portrait in the tables has none; a side
/// without a hero and without a creature to show (no record but the arrow towers) shows no face.
/// Without the ruleset's values (`sources.ruleset` none: a server's game whose content differs
/// or that the packs here do not compile) the experience is 0 and a side without a hero shows no
/// creature (*provisional*: the server's values are not known here).
pub fn window(
    sources: &Sources<'_>,
    opening: &Observation,
    outcome: &BattleOutcome,
    player: u8,
) -> (MessageBox, Tune) {
    let values = sources.ruleset.map(|ruleset| {
        let core: &Ruleset = ruleset;
        move |c: CreatureTypeId| {
            (
                ruleset.creature(c).map_or(0, |t| t.hit_points),
                core.creature(c).map_or(0, |t| t.ai_value),
            )
        }
    });
    built(sources, values, opening, outcome, player)
}

/// [`window`] with each creature's hit points and AI value by `values` (none: not known).
fn built(
    sources: &Sources<'_>,
    values: Option<impl Fn(CreatureTypeId) -> (u32, u32)>,
    opening: &Observation,
    outcome: &BattleOutcome,
    player: u8,
) -> (MessageBox, Tune) {
    let (gave_up, giver) = match outcome.result {
        BattleResult::Retreat(side) => (Some(GaveUp::Retreat), Some(side.index())),
        BattleResult::Surrender(side) => (Some(GaveUp::Surrender), Some(side.index())),
        BattleResult::Victory(_) | BattleResult::Draw => (None, None),
    };
    let winner = result::winner(
        outcome.result.winner().map(BattleSide::index),
        gave_up.is_some(),
        survivors(&outcome.stacks),
    );
    let human = [0, 1].map(|i| opening.sides.get(i).is_some_and(|s| s.human));
    let shown = result::shown_side(winner, human);
    let won = winner == Some(shown);
    // How the battle was given up as the window tells it: the shown side's enemy when it won, the
    // shown side itself when it lost. With no winner named (BAT-END-001) the original tells the
    // defender's hero fled or surrendered whichever side did (the flags are either side's); the
    // client tells it only of the side that did, else its army fell (CLT-Q35, answered (b) at E17
    // M5 T8c), its tune and clip by the same branch; the original's for the comparisons.
    let told = gave_up.filter(|_| won || sources.bugs == Bugs::Original || giver == Some(shown));
    let town = opening.siege.is_some();
    let catalogue = sources.catalogue;
    let heroes = [0, 1].map(|i| hero_key(sources, opening, i));
    // A hero's name: its type's (a map's own name for a hero is not read); empty when the
    // manifest names no such type.
    let names = heroes.map(|hero| {
        hero.map(|key| {
            key.map_or_else(String::new, |key| {
                catalogue.text(&format!("hero.{key}"), &Args::new())
            })
        })
    });
    let stacks = |i: usize| outcome.stacks.get(i).map_or(&[][..], Vec::as_slice);
    let experience = match &values {
        Some(values) if won => {
            let loser = 1 - shown;
            experience(
                stacks(loser),
                |c| values(c).0,
                (
                    names[loser].is_some(),
                    gave_up.is_some(),
                    town && shown == 0,
                ),
            )
        }
        _ => 0,
    };
    let text = result::text(catalogue, won, told, names[shown].as_deref(), experience);
    let original = |c: CreatureTypeId| {
        let key = sources.manifest.keys.creatures.get(usize::from(c.raw()))?;
        sources
            .tables
            .creatures
            .get(key)
            .map(|e| u32::from(e.original))
    };
    let face = |index: usize| match heroes[index] {
        // Its type's large portrait (*provisional*: the hero's own portrait, its +0x34 the
        // executable reads, is its type's unless a map sets another, which the client does not
        // know).
        Some(key) => Face::Hero {
            name: names[index].clone().unwrap_or_default(),
            portrait: key.and_then(|key| sources.tables.heroes.get(key)?.portrait.clone()),
        },
        None => {
            let found = values
                .as_ref()
                .and_then(|values| strongest(stacks(index), original, |c| values(c).1));
            let Some((creature, several)) = found else {
                return Face::Creature {
                    name: String::new(),
                    original: None,
                };
            };
            let count = if several { 2 } else { 1 };
            let name = sources
                .manifest
                .keys
                .creatures
                .get(usize::from(creature.raw()))
                .map_or_else(String::new, |key| {
                    catalogue.text(&format!("creature.{key}"), &Args::new().count(count))
                });
            Face::Creature {
                name,
                original: original(creature),
            }
        }
    };
    let sides = [0, 1].map(|index| Side {
        face: face(index),
        victorious: winner == Some(index),
        losses: losses(stacks(index), original),
    });
    let texts = Texts::of(catalogue, text);
    let window = result::layout(&sources.layout.result, &texts, &sides, player);
    (window, tune(won, shown, town, told))
}

/// The window's tune, and so its clip (`h3_view::clip::of`), by the branches that pick its text
/// (0x470B68 to 0x470CFF; `h3_view::sound::end_tune` picks the same from the record's outcome):
/// the shown side won, [`Tune::DefendCastle`] for a town's defender, else [`Tune::Win`]; it lost,
/// [`Tune::LoseCastle`] for a town's defender, else the way a side gave up, else [`Tune::Lose`].
pub fn tune(won: bool, shown: usize, town: bool, gave_up: Option<GaveUp>) -> Tune {
    let defender_at_town = town && shown == 1;
    match (won, gave_up) {
        (true, _) if defender_at_town => Tune::DefendCastle,
        (true, _) => Tune::Win,
        (false, _) if defender_at_town => Tune::LoseCastle,
        (false, Some(GaveUp::Surrender)) => Tune::Surrender,
        (false, Some(GaveUp::Retreat)) => Tune::Retreat,
        (false, None) => Tune::Lose,
    }
}

/// Side `index`'s hero, if it has one: its type's key (none: the manifest names no such type).
fn hero_key<'a>(
    sources: &Sources<'a>,
    opening: &Observation,
    index: usize,
) -> Option<Option<&'a str>> {
    let hero = opening.sides.get(index)?.hero.as_ref()?;
    let keys = &sources.manifest.keys.hero_types;
    Some(keys.get(usize::from(hero.hero_type)).map(String::as_str))
}

/// The records the window reads of a side: its first 20 (the executable's loops run over 20 of a
/// side's 21 stack slots).
const RECORDS: usize = 20;
/// The arrow tower (149), which a side's strongest creature never is (0x4705BD).
const ARROW_TOWER: u32 = 149;
/// The points a beaten hero gives (0x46A3DF), taken back after a retreat or a surrender
/// (0x46A3FC), and given to an attacker that took a town (0x46A416).
const HERO_POINTS: i64 = 500;

/// A side's losses (0x470D89 to 0x470E48): over its records' (`stacks`, the engine's, in stack
/// order) creatures lost (`start − end`), each creature's sum in the order it first lost some, by
/// its original id (`original`; a creature without one is left out).
#[must_use]
pub fn losses(
    stacks: &[StackRecord],
    original: impl Fn(CreatureTypeId) -> Option<u32>,
) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for record in stacks.iter().take(RECORDS) {
        let lost = record.start.saturating_sub(record.end);
        let Some(id) = original(record.creature).filter(|_| lost > 0) else {
            continue;
        };
        match out.iter_mut().find(|(o, _)| *o == id) {
            Some((_, sum)) => *sum = sum.saturating_add(lost),
            None => out.push((id, lost)),
        }
    }
    out
}

/// A side without a hero's face (0x4705A2 to 0x470627): of its records but the arrow towers, the
/// first with the highest AI value (`ai_value`), its name plural when there are several
/// (`more`), as the executable reads them; its creature and whether there are several.
#[must_use]
pub fn strongest(
    stacks: &[StackRecord],
    original: impl Fn(CreatureTypeId) -> Option<u32>,
    ai_value: impl Fn(CreatureTypeId) -> u32,
) -> Option<(CreatureTypeId, bool)> {
    let mut best: Option<(CreatureTypeId, u32)> = None;
    let mut count = 0;
    for record in stacks.iter().take(RECORDS) {
        if original(record.creature) == Some(ARROW_TOWER) {
            continue;
        }
        count += 1;
        let value = ai_value(record.creature);
        if best.is_none_or(|(_, b)| value > b) {
            best = Some((record.creature, value));
        }
    }
    best.map(|(creature, _)| (creature, count > 1))
}

/// The points the result's text says the winner's hero receives (`battle_experience` 0x46A350):
/// the hit points (`hit_points`) of the creatures the loser's army and raised stacks lost
/// (`loser`, its records; not its war machines nor the stacks the battle created, which the
/// engine's `StackOrigin` tells as the executable's flags do), 500 more when the loser had a
/// hero (`beaten_hero`) unless a side gave up (`gave_up`), 500 more when the attacker took a town
/// (`town_taken`). *Not built:* the winner's Learning (×, `crt_ftol`): the client's heroes have
/// none (*provisional*: the scenarios name no Learning).
#[must_use]
pub fn experience(
    loser: &[StackRecord],
    hit_points: impl Fn(CreatureTypeId) -> u32,
    (beaten_hero, gave_up, town_taken): (bool, bool, bool),
) -> u32 {
    let killed: i64 = loser
        .iter()
        .take(RECORDS)
        .filter(|s| matches!(s.origin, StackOrigin::Army(_) | StackOrigin::Raised(_)))
        .map(|s| i64::from(s.start.saturating_sub(s.end)) * i64::from(hit_points(s.creature)))
        .sum();
    let mut points = killed;
    if beaten_hero {
        points += HERO_POINTS;
    }
    if gave_up {
        points -= HERO_POINTS;
    }
    if town_taken {
        points += HERO_POINTS;
    }
    u32::try_from(points.max(0)).unwrap_or(u32::MAX)
}

/// Whether each side has army survivors at the battle's end, from its records (`stacks`): an army
/// stack or one raised from corpses alive (BAT-END-001; what the engine's `survivors` hold, which
/// a served outcome leaves empty: [`served`]).
#[must_use]
pub fn survivors(stacks: &[Vec<StackRecord>; 2]) -> [bool; 2] {
    stacks.each_ref().map(|side| {
        side.iter()
            .any(|s| s.end > 0 && matches!(s.origin, StackOrigin::Army(_) | StackOrigin::Raised(_)))
    })
}

/// The outcome of a battle on a server as a seat's record shows it (E17 M5 T6b1; no engine runs
/// here): the game's end `result` (a forfeit the other side's victory; a void game a draw), and
/// each side's records ([`records`]). What the result window reads only (*provisional*: the
/// survivors, the heroes' outcomes and the surrender's cost are left empty).
pub fn served(
    opening: &Observation,
    end: &Observation,
    result: h3_protocol::wire::Outcome,
) -> BattleOutcome {
    use h3_protocol::wire::{Outcome, Side as Wire};
    let side = |s: Wire| match s {
        Wire::Attacker => BattleSide::Attacker,
        Wire::Defender => BattleSide::Defender,
    };
    let result = match result {
        Outcome::Victory { side: s } => BattleResult::Victory(side(s)),
        Outcome::Retreat { side: s } => BattleResult::Retreat(side(s)),
        Outcome::Surrender { side: s } => BattleResult::Surrender(side(s)),
        Outcome::Forfeit { side: s } => BattleResult::Victory(side(s).opponent()),
        Outcome::Draw {} | Outcome::NoShow {} | Outcome::Unverified {} => BattleResult::Draw,
    };
    BattleOutcome {
        result,
        survivors: [Army::default(), Army::default()],
        war_machines: [Vec::new(), Vec::new()],
        stacks: records(opening, end),
        surrender_cost: None,
        heroes: [None, None],
        rounds: end.round,
    }
}

/// Each side's records as a seat's record shows the battle's `end` (the seat's,
/// `h3_view::remote::seen`), its stacks in id order: where each came from, its creatures at the
/// `opening` (the first of its stacks with its id and creature; else its `base_count`: when the
/// battle created it) and at the end (none when it is dead).
pub fn records(opening: &Observation, end: &Observation) -> [Vec<StackRecord>; 2] {
    use h3_protocol::wire::Side as Wire;
    // The opening's counts by id and creature, read once (E17 M5 T8b2, R5b-1: a hostile record's
    // many stacks cost no search per stack).
    let mut started = BTreeMap::new();
    for s in &opening.stacks {
        started.entry((s.id, s.creature)).or_insert(s.count);
    }
    let mut stacks: [Vec<StackRecord>; 2] = [Vec::new(), Vec::new()];
    for stack in &end.stacks {
        let origin = if stack.war_machine {
            StackOrigin::WarMachine
        } else if stack.raised {
            StackOrigin::Raised(stack.slot)
        } else if stack.created {
            StackOrigin::Created
        } else {
            StackOrigin::Army(stack.slot)
        };
        let start = started
            .get(&(stack.id, stack.creature))
            .copied()
            .unwrap_or(stack.base_count);
        stacks[usize::from(stack.side == Wire::Defender)].push(StackRecord {
            origin,
            creature: CreatureTypeId::from_raw(stack.creature),
            start,
            end: if stack.alive { stack.count } else { 0 },
        });
    }
    stacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_kernel::Handle;
    use h3_protocol::wire::{Hero, KeyTables, Seat, Siege};
    use h3_ui::text::pack::Pack;
    use h3_ui::ui::Item;
    use h3_view::result::{
        ATTACKER_NAME, ATTACKER_PORTRAIT, DEFENDER_NAME, DEFENDER_PORTRAIT, TEXT,
    };
    use std::path::Path;

    /// The asset tables: Orrin's portrait, Sandro none; the Pikeman, the Dragon and the arrow
    /// tower, the Ghost none.
    const TABLE: &str = r#"
[table]
id = "t"
depends = []

[battle]
groups = { moving = 0, mouse_over = 1, standing = 2, hit = 3, defend = 4, death = 5, strike = [11, 12, 13], shoot = [14, 15, 16], breath = [17, 18, 19], move_start = 20, move_end = 21, turn_left = [7, 10], turn_right = [9, 8] }
hero_groups = { standing = 0, idle = 1, defeat = 2, victory = 3, cast = 4 }
flags = ["CmFlagL.def", "CmFlagR.def"]
magic = []
heal_sound = "Regener"

[[hero]]
key = "orrin"
original = 0
def = "CH00.DEF"
portrait = "HPL000Kn.pcx"

[[hero]]
key = "sandro"
original = 1
def = "CH01.DEF"

[[creature]]
key = "pikeman"
original = 0
def = "CPKMAN.DEF"
times = [9000, 575, 500, 115]

[[creature]]
key = "dragon"
original = 26
def = "CBDRGN.DEF"
times = [9000, 575, 500, 115]

[[creature]]
key = "tower"
original = 149
def = "CTOWER.DEF"
times = [9000, 575, 500, 115]
"#;

    const PACK: &str = r#"{
    "result": { "experience": " {{hero}} +{{experience}}" },
    "hero": { "t:orrin": "Orrin", "t:sandro": "Sandro" },
    "creature": {
        "t:pikeman_one": "Pikeman", "t:pikeman_other": "Pikemen",
        "t:dragon_one": "Dragon", "t:dragon_other": "Dragons"
    }
}"#;

    const PIKEMAN: u16 = 0;
    const DRAGON: u16 = 1;
    const TOWER: u16 = 2;
    const GHOST: u16 = 3;

    /// What the window is made from: the opening of the synthetic duel (its two stacks, no
    /// heroes), the manifest with the tables' keys and the Ghost's, which they lack.
    struct Fixture {
        manifest: Manifest,
        tables: Tables,
        catalogue: Catalogue,
        layout: BattleLayout,
        opening: Observation,
    }

    fn fixture() -> Fixture {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let text = crate::scenario::record_of(
            &root.join("../../../packs/synthetic/scenarios/duel.toml"),
            &root.join("../../../packs"),
            1,
            Seat::Spectator,
        )
        .unwrap();
        let record = h3_view::record::parse(&text).unwrap();
        let list = |keys: &[&str]| keys.iter().map(|k| (*k).to_string()).collect();
        let mut manifest = record.header.manifest.clone();
        manifest.keys = KeyTables {
            creatures: list(&["t:pikeman", "t:dragon", "t:tower", "t:ghost"]),
            hero_types: list(&["t:orrin", "t:sandro"]),
            ..KeyTables::default()
        };
        let layout = root.join("../../layouts/battle.toml");
        Fixture {
            manifest,
            tables: h3_view::tables::load(&[h3_view::tables::parse(TABLE).unwrap()], "t").unwrap(),
            catalogue: Catalogue {
                packs: vec![Pack::from_json("en", PACK).unwrap()],
            },
            layout: h3_view::layout::parse_battle(&std::fs::read_to_string(layout).unwrap())
                .unwrap(),
            opening: record.first_snapshot().unwrap().clone(),
        }
    }

    /// A hero of type `hero_type`.
    fn hero(hero_type: u16) -> Hero {
        Hero {
            hero_type,
            attack: 0,
            defense: 0,
            power: 0,
            knowledge: 0,
            mana: 0,
            max_mana: 0,
            casts: 0,
            commands_ballista: false,
            commands_tent: false,
            commands_catapult: false,
            spell_book: false,
            sex: None,
        }
    }

    /// The opening with each side's hero (by type) and whether a human plays it.
    fn opening(f: &Fixture, heroes: [Option<u16>; 2], human: [bool; 2]) -> Observation {
        let mut opening = f.opening.clone();
        for (i, side) in opening.sides.iter_mut().enumerate() {
            side.hero = heroes[i].map(hero);
            side.human = human[i];
        }
        opening
    }

    fn outcome(result: BattleResult, stacks: [Vec<StackRecord>; 2]) -> BattleOutcome {
        BattleOutcome {
            result,
            survivors: [Army::default(), Army::default()],
            war_machines: [Vec::new(), Vec::new()],
            stacks,
            surrender_cost: None,
            heroes: [None, None],
            rounds: 1,
        }
    }

    /// Each creature's hit points and AI value: the tower's the highest.
    fn values(c: CreatureTypeId) -> (u32, u32) {
        match c.raw() {
            PIKEMAN => (10, 100),
            DRAGON => (180, 5000),
            TOWER => (1, 9999),
            _ => (0, 0),
        }
    }

    /// The window `f` builds of `opening` and `outcome` with the creatures' values or without, as
    /// the comparisons build it (the original's profile).
    fn shown(
        f: &Fixture,
        opening: &Observation,
        outcome: &BattleOutcome,
        known: bool,
    ) -> (MessageBox, Tune) {
        shown_in(f, opening, outcome, known, Bugs::Original)
    }

    /// [`shown`] in the profile `bugs`.
    fn shown_in(
        f: &Fixture,
        opening: &Observation,
        outcome: &BattleOutcome,
        known: bool,
        bugs: Bugs,
    ) -> (MessageBox, Tune) {
        let sources = Sources {
            manifest: &f.manifest,
            tables: &f.tables,
            ruleset: None,
            catalogue: &f.catalogue,
            layout: &f.layout,
            bugs,
        };
        let values = known.then_some(values);
        built(&sources, values, opening, outcome, 0)
    }

    fn text(window: &MessageBox, id: u16) -> &str {
        window.shown.texts.get(&id).map_or("", String::as_str)
    }

    /// A portrait: a picture's name, or a DEF item's DEF, frame and mirror.
    fn portrait(window: &MessageBox, id: u16) -> (Option<&str>, u32, bool) {
        match window.dialog.item(id) {
            Some(Item::Picture(p)) => (Some(p.picture.as_str()), 0, false),
            Some(Item::Def(d)) => (d.def.as_deref(), d.frame, d.mirrored),
            _ => panic!("no portrait {id}"),
        }
    }

    /// The two sides' words.
    fn words(window: &MessageBox) -> [&str; 2] {
        [text(window, 0x103), text(window, 0x104)]
    }

    #[test]
    fn a_side_without_a_hero_shows_its_strongest_creature() {
        // R5b-7: plural by its records, dead ones counted, the towers not; `TwCrPort.def` at the
        // creature's original id + 2, the defender's mirrored.
        let f = fixture();
        let defender = vec![
            record(StackOrigin::Army(0), PIKEMAN, 10, 0),
            record(StackOrigin::Army(1), DRAGON, 2, 0),
            record(StackOrigin::WarMachine, TOWER, 1, 1),
        ];
        let attacker = vec![record(StackOrigin::Army(0), PIKEMAN, 5, 5)];
        let won = outcome(
            BattleResult::Victory(BattleSide::Attacker),
            [attacker.clone(), defender.clone()],
        );
        let (window, tune) = shown(&f, &opening(&f, [Some(0), None], [true, false]), &won, true);
        assert_eq!(text(&window, DEFENDER_NAME), "Dragons");
        assert_eq!(
            portrait(&window, DEFENDER_PORTRAIT),
            (Some("TwCrPort.def"), 28, true)
        );
        assert_eq!(
            portrait(&window, ATTACKER_PORTRAIT),
            (Some("HPL000Kn.pcx"), 0, false)
        );
        assert_eq!(words(&window), ["result.victorious", "result.defeated"]);
        // 10 × 10 + 2 × 180; the tower gives nothing, no hero 500.
        assert_eq!(text(&window, TEXT), "result.won Orrin +460");
        assert_eq!(tune, Tune::Win);
        // One record but the tower: singular. The attacker's own creature is not mirrored.
        let alone = outcome(
            BattleResult::Victory(BattleSide::Attacker),
            [attacker, defender[1..].to_vec()],
        );
        let (window, _) = shown(&f, &opening(&f, [None, None], [true, false]), &alone, true);
        assert_eq!(text(&window, DEFENDER_NAME), "Dragon");
        assert_eq!(text(&window, ATTACKER_NAME), "Pikeman");
        assert_eq!(
            portrait(&window, ATTACKER_PORTRAIT),
            (Some("TwCrPort.def"), 2, false)
        );
        // Without a hero the text names none.
        assert_eq!(text(&window, TEXT), "result.won");
    }

    #[test]
    fn what_the_client_lacks_is_drawn_empty() {
        // R5b-1 and R5b-5: always a window.
        let f = fixture();
        let stacks = [
            vec![record(StackOrigin::Army(0), GHOST, 3, 3)],
            vec![
                record(StackOrigin::WarMachine, TOWER, 1, 0),
                record(StackOrigin::Army(0), PIKEMAN, 10, 4),
            ],
        ];
        let won = outcome(BattleResult::Victory(BattleSide::Attacker), stacks.clone());
        // A creature the tables lack: its name, no portrait.
        let (window, _) = shown(&f, &opening(&f, [None, Some(1)], [true, false]), &won, true);
        assert_eq!(text(&window, ATTACKER_NAME), "creature.t:ghost");
        assert_eq!(portrait(&window, ATTACKER_PORTRAIT), (None, 0, false));
        // A hero type without a portrait: its name, no portrait.
        assert_eq!(text(&window, DEFENDER_NAME), "Sandro");
        assert_eq!(portrait(&window, DEFENDER_PORTRAIT), (None, 0, true));
        // A hero type the manifest does not name: still a hero, its 500 counted (6 × 10 + 500).
        let (window, _) = shown(
            &f,
            &opening(&f, [Some(0), Some(7)], [true, false]),
            &won,
            true,
        );
        assert_eq!(text(&window, DEFENDER_NAME), "");
        assert_eq!(portrait(&window, DEFENDER_PORTRAIT), (None, 0, true));
        assert_eq!(text(&window, TEXT), "result.won Orrin +560");
        // No creature to show (the towers only): no face; a creature the manifest does not name:
        // no name.
        let towers = outcome(
            BattleResult::Victory(BattleSide::Attacker),
            [
                vec![record(StackOrigin::Army(0), 9, 1, 1)],
                stacks[1][..1].to_vec(),
            ],
        );
        let (window, _) = shown(&f, &opening(&f, [None, None], [true, false]), &towers, true);
        assert_eq!(text(&window, DEFENDER_NAME), "");
        assert_eq!(portrait(&window, DEFENDER_PORTRAIT), (None, 0, true));
        assert_eq!(text(&window, ATTACKER_NAME), "");
        // Without the creatures' values (R5b-2): no experience, no strongest creature.
        let (window, _) = shown(
            &f,
            &opening(&f, [Some(0), None], [true, false]),
            &won,
            false,
        );
        assert_eq!(text(&window, TEXT), "result.won Orrin +0");
        assert_eq!(text(&window, DEFENDER_NAME), "");
        assert_eq!(portrait(&window, DEFENDER_PORTRAIT), (None, 0, true));
    }

    #[test]
    fn the_experience_is_the_shown_winner_s_and_a_town_s_for_its_attacker() {
        let f = fixture();
        let stacks = [
            vec![record(StackOrigin::Army(0), PIKEMAN, 5, 0)],
            vec![record(StackOrigin::Army(0), PIKEMAN, 10, 4)],
        ];
        let heroes = [Some(0), Some(1)];
        let attacker_won = outcome(BattleResult::Victory(BattleSide::Attacker), stacks.clone());
        let defender_won = outcome(BattleResult::Victory(BattleSide::Defender), stacks);
        let mut town = opening(&f, heroes, [true, true]);
        town.siege = Some(Siege {
            faction: 0,
            fort: 1,
            moat: 0,
            gate_open: false,
            walls: Vec::new(),
            moat_hexes: Vec::new(),
            parts: Vec::new(),
        });
        // 6 × 10 + the hero's 500, 500 more for the attacker at a town.
        let field = opening(&f, heroes, [true, true]);
        let (window, _) = shown(&f, &field, &attacker_won, true);
        assert_eq!(text(&window, TEXT), "result.won Orrin +560");
        let (window, tune) = shown(&f, &town, &attacker_won, true);
        assert_eq!(text(&window, TEXT), "result.won Orrin +1060");
        assert_eq!(tune, Tune::Win);
        // The town's defender won: no town's 500 (5 × 10 + 500).
        let (window, tune) = shown(&f, &town, &defender_won, true);
        assert_eq!(text(&window, TEXT), "result.won Sandro +550");
        assert_eq!(tune, Tune::DefendCastle);
        // The shown side lost (the winner's player not a human here): no experience.
        let (window, tune) = shown(&f, &opening(&f, heroes, [false, true]), &attacker_won, true);
        assert_eq!(text(&window, TEXT), "result.lost");
        assert_eq!(words(&window), ["result.victorious", "result.defeated"]);
        assert_eq!(tune, Tune::Lose);
    }

    #[test]
    fn a_side_that_gave_up_against_created_stacks_only_leaves_no_winner() {
        // R5b-3, BAT-END-001: both defeated, the defender's view, no experience.
        let f = fixture();
        let attacker = vec![record(StackOrigin::Army(0), PIKEMAN, 5, 5)];
        let created = vec![
            record(StackOrigin::Army(0), PIKEMAN, 10, 0),
            record(StackOrigin::Created, DRAGON, 1, 1),
        ];
        let human = opening(&f, [Some(0), Some(1)], [true, true]);
        let fled = outcome(
            BattleResult::Retreat(BattleSide::Attacker),
            [attacker.clone(), created.clone()],
        );
        let (window, tune) = shown(&f, &human, &fled, true);
        assert_eq!(words(&window), ["result.defeated", "result.defeated"]);
        assert_eq!(text(&window, TEXT), "result.lost.fled");
        assert_eq!(tune, Tune::Retreat);
        // The original tells the defender's hero fled though the attacker did; the client tells
        // it lost (CLT-Q35), with the tune and the clip of that branch.
        let (window, tune) = shown_in(&f, &human, &fled, true, Bugs::Fixed);
        assert_eq!(words(&window), ["result.defeated", "result.defeated"]);
        assert_eq!(text(&window, TEXT), "result.lost");
        assert_eq!(tune, Tune::Lose);
        // The defender who fled is told so in both.
        let left = outcome(
            BattleResult::Retreat(BattleSide::Defender),
            [
                vec![record(StackOrigin::Created, DRAGON, 1, 1)],
                vec![record(StackOrigin::Army(0), PIKEMAN, 5, 5)],
            ],
        );
        for bugs in [Bugs::Original, Bugs::Fixed] {
            let (window, tune) = shown_in(&f, &human, &left, true, bugs);
            assert_eq!(text(&window, TEXT), "result.lost.fled");
            assert_eq!(tune, Tune::Retreat);
        }
        // A raised stack is an army survivor.
        let mut raised = created;
        raised[1].origin = StackOrigin::Raised(1);
        let fled = outcome(
            BattleResult::Retreat(BattleSide::Attacker),
            [attacker, raised],
        );
        let (window, tune) = shown(&f, &human, &fled, true);
        assert_eq!(words(&window), ["result.defeated", "result.victorious"]);
        assert_eq!(text(&window, TEXT), "result.won.fled Sandro +0");
        assert_eq!(tune, Tune::Win);
    }

    #[test]
    fn a_served_outcome_is_read_from_the_record_s_opening_and_end() {
        use h3_protocol::wire::{Outcome, Side as Wire};
        let f = fixture();
        let mut opening = f.opening.clone();
        let (a, d) = (opening.stacks[0].clone(), opening.stacks[1].clone());
        assert_eq!((a.side, d.side), (Wire::Attacker, Wire::Defender));
        assert!(a.id != d.id && a.creature != d.creature && a.alive);
        // A hostile record's repeated stack: the first counts.
        let mut again = a.clone();
        again.count = 99;
        opening.stacks.push(again);
        let mut end = opening.clone();
        end.round = 4;
        let mut stacks = vec![a.clone(), d.clone()];
        stacks[0].count = 3;
        // Dead: its end 0 whatever its count; another creature than the opening's under its id:
        // its start its base count.
        stacks[1].alive = false;
        stacks[1].creature = a.creature;
        stacks[1].base_count = 17;
        for (id, side, flags, slot) in [
            (5, Wire::Attacker, (false, false, true), 255),
            (6, Wire::Defender, (true, false, false), 16),
            (7, Wire::Defender, (false, true, false), 2),
        ] {
            let mut s = a.clone();
            s.id = id;
            s.side = side;
            (s.war_machine, s.raised, s.created) = flags;
            s.slot = slot;
            s.base_count = 4;
            s.count = 2;
            stacks.push(s);
        }
        end.stacks = stacks;
        let read = served(
            &opening,
            &end,
            Outcome::Victory {
                side: Wire::Attacker,
            },
        );
        let creature = |s: &h3_protocol::wire::Stack| Handle::from_raw(s.creature);
        assert_eq!(
            read.stacks,
            [
                vec![
                    StackRecord {
                        origin: StackOrigin::Army(a.slot),
                        creature: creature(&a),
                        start: a.count,
                        end: 3,
                    },
                    StackRecord {
                        origin: StackOrigin::Created,
                        creature: creature(&a),
                        start: 4,
                        end: 2,
                    },
                ],
                vec![
                    StackRecord {
                        origin: StackOrigin::Army(d.slot),
                        creature: creature(&a),
                        start: 17,
                        end: 0,
                    },
                    StackRecord {
                        origin: StackOrigin::WarMachine,
                        creature: creature(&a),
                        start: 4,
                        end: 2,
                    },
                    StackRecord {
                        origin: StackOrigin::Raised(2),
                        creature: creature(&a),
                        start: 4,
                        end: 2,
                    },
                ],
            ]
        );
        assert_eq!(read.rounds, 4);
        assert_eq!(survivors(&read.stacks), [true, true]);
        assert_eq!(read.result, BattleResult::Victory(BattleSide::Attacker));
        let result = |outcome| served(&opening, &end, outcome).result;
        assert_eq!(
            result(Outcome::Forfeit {
                side: Wire::Attacker
            }),
            BattleResult::Victory(BattleSide::Defender)
        );
        assert_eq!(
            result(Outcome::Retreat {
                side: Wire::Defender
            }),
            BattleResult::Retreat(BattleSide::Defender)
        );
        assert_eq!(
            result(Outcome::Surrender {
                side: Wire::Attacker
            }),
            BattleResult::Surrender(BattleSide::Attacker)
        );
        for drawn in [Outcome::Draw {}, Outcome::NoShow {}, Outcome::Unverified {}] {
            assert_eq!(result(drawn), BattleResult::Draw);
        }
    }

    #[test]
    fn survivors_are_army_or_raised_stacks_alive() {
        let side = |origin, end| vec![record(origin, PIKEMAN, 5, end)];
        assert_eq!(
            survivors(&[
                side(StackOrigin::Army(0), 1),
                side(StackOrigin::Raised(0), 1)
            ]),
            [true, true]
        );
        assert_eq!(
            survivors(&[side(StackOrigin::Army(0), 0), side(StackOrigin::Created, 1)]),
            [false, false]
        );
        assert_eq!(
            survivors(&[side(StackOrigin::WarMachine, 1), Vec::new()]),
            [false, false]
        );
    }

    fn record(origin: StackOrigin, creature: u16, start: u32, end: u32) -> StackRecord {
        StackRecord {
            origin,
            creature: Handle::from_raw(creature),
            start,
            end,
        }
    }

    #[test]
    fn the_windows_tune_is_the_one_the_battles_end_plays() {
        use h3_protocol::wire::{Outcome, Side as Seat};
        let seat = |i: usize| {
            if i == 0 {
                Seat::Attacker
            } else {
                Seat::Defender
            }
        };
        let pairs = [[true, true], [true, false], [false, true], [false, false]];
        for (human, survivors) in pairs.into_iter().flat_map(|h| pairs.map(|s| (h, s))) {
            for town in [false, true] {
                for winner in [None, Some(0), Some(1)] {
                    for gave_up in [None, Some(GaveUp::Retreat), Some(GaveUp::Surrender)] {
                        let outcome = match (winner, gave_up) {
                            (None, _) => Outcome::Draw {},
                            (Some(w), None) => Outcome::Victory { side: seat(w) },
                            (Some(w), Some(GaveUp::Retreat)) => {
                                Outcome::Retreat { side: seat(1 - w) }
                            }
                            (Some(w), Some(GaveUp::Surrender)) => {
                                Outcome::Surrender { side: seat(1 - w) }
                            }
                        };
                        let gave_up = winner.and(gave_up);
                        let named = result::winner(winner, gave_up.is_some(), survivors);
                        let shown = result::shown_side(named, human);
                        let won = named == Some(shown);
                        let giver = winner.map(|w| 1 - w);
                        for bugs in [Bugs::Original, Bugs::Fixed] {
                            // As `built` tells it (CLT-Q35).
                            let told = gave_up
                                .filter(|_| won || bugs == Bugs::Original || giver == Some(shown));
                            assert_eq!(
                                tune(won, shown, town, told),
                                h3_view::sound::end_tune(outcome, (human, town), survivors, bugs),
                                "{human:?} {survivors:?} {town} {winner:?} {gave_up:?} {bugs:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn losses_sum_by_creature_in_the_order_they_first_lost() {
        let stacks = vec![
            record(StackOrigin::Army(0), 5, 10, 10),
            record(StackOrigin::Army(1), 3, 10, 4),
            record(StackOrigin::Army(2), 5, 8, 0),
            record(StackOrigin::WarMachine, 7, 1, 0),
            record(StackOrigin::Army(3), 3, 2, 1),
            record(StackOrigin::Created, 9, 4, 0),
        ];
        let original = |c: CreatureTypeId| (c.raw() != 9).then_some(u32::from(c.raw()) * 10);
        assert_eq!(losses(&stacks, original), vec![(30, 7), (50, 8), (70, 1)]);
    }

    #[test]
    fn the_strongest_creature_is_the_first_of_the_highest_value_without_the_towers() {
        let stacks = vec![
            record(StackOrigin::Army(0), 1, 10, 0),
            record(StackOrigin::Army(1), 2, 10, 0),
            record(StackOrigin::Army(2), 3, 10, 0),
            record(StackOrigin::WarMachine, 149, 1, 1),
        ];
        let original = |c: CreatureTypeId| Some(u32::from(c.raw()));
        let value = |c: CreatureTypeId| match c.raw() {
            2 | 3 => 50,
            149 => 1000,
            _ => 10,
        };
        assert_eq!(
            strongest(&stacks, original, value),
            Some((Handle::from_raw(2), true))
        );
        assert_eq!(
            strongest(&stacks[..1], original, value),
            Some((Handle::from_raw(1), false))
        );
        assert_eq!(strongest(&[], original, value), None);
    }

    #[test]
    fn experience_counts_the_loser_s_army_and_the_heroes() {
        let loser = vec![
            record(StackOrigin::Army(0), 1, 10, 0),
            record(StackOrigin::Raised(1), 2, 4, 1),
            record(StackOrigin::WarMachine, 3, 1, 0),
            record(StackOrigin::Created, 4, 5, 0),
        ];
        let hp = |c: CreatureTypeId| u32::from(c.raw()) * 10;
        // 10 × 10 + 3 × 20; the machine and the created stack give nothing.
        assert_eq!(experience(&loser, hp, (false, false, false)), 160);
        assert_eq!(experience(&loser, hp, (true, false, false)), 660);
        assert_eq!(experience(&loser, hp, (true, true, false)), 160);
        assert_eq!(experience(&loser, hp, (true, false, true)), 1160);
        assert_eq!(experience(&[], hp, (false, true, false)), 0);
    }
}
