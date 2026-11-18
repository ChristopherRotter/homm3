//! The taverns (E09 increment 4, the Tavern; T37a, design note `E09-design.md` §61 to §63, §68,
//! §75; survey `E09-tavern.md` §2): each player's two tavern heroes, drawn by the tavern weights
//! from the taverns' stream (TWN-TAV-001), the game's start's and every week's fills after the
//! week's release (TWN-TAV-002; GAM-CAL-002), the tavern heroes over a turn (TWN-TAV-005), and
//! what their owner sees of them (`GameObservation::tavern`, [`GameEvent::TavernOffered`] and
//! [`GameEvent::TavernPartSeen`]; decisions 134, 135).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{FactionId, Formation, HeroClassId, HeroType, HeroTypeId, Ruleset, SkillId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{ArtifactInstance, ArtifactPlace, HeroDetails, Position};
use h3_kernel::Rng;
use h3_towns::TownRuleset;

use super::FactionSet;
use super::pool::{HeroPool, Tavern};
use crate::command::{GameEvent, Sink};
use crate::setup::{hero_details, starting_army, wear_machines};
use crate::state::Progression;
use crate::{GameRuleset, GameState, HeroDetailsSetup};

/// The backpack's places a tavern hero's screen shows (decision 134): the original's tavern
/// opens the offer's hero screen as a held popup (`tavern_dialog_proc` 0x5D7D7D, `hero_screen_show`
/// with its held flag, 0x4E1969), which scrolls nothing, so the backpack's first five places show
/// and the rest only by its count.
pub const TAVERN_BACKPACK_SHOWN: usize = 5;

/// What a tavern hero's owner sees of it without a list (decisions 134, 135; design note §75):
/// what the tavern dialog and the offer's held hero screen show. Its learned spells (the spell
/// book is a screen of its own), the levels at which Wisdom and a school were last offered and its
/// points never show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TavernHero {
    /// The hero.
    pub id: HeroId,
    /// Its type: identity and class.
    pub hero_type: HeroTypeId,
    /// Its level.
    pub level: u8,
    /// Its experience.
    pub experience: u32,
    /// Its base primary skills (attack, defense, power, knowledge).
    pub primary: [u8; 4],
    /// Its spell points.
    pub mana: u16,
    /// How many artifacts its backpack carries (the dialog's artifact count with the worn ones;
    /// the screen shows the first [`TAVERN_BACKPACK_SHOWN`]).
    pub carried: u16,
    /// Its army's formation in battle.
    pub formation: Formation,
    /// Its tactics flag.
    pub tactics: bool,
}

/// A part of a tavern hero's view that is one of a list (decision 135): told after its
/// [`GameEvent::TavernOffered`], one per part in this order: the army's stacks by slot, the
/// secondary skills in order, the worn artifacts by position, then the backpack's first
/// [`TAVERN_BACKPACK_SHOWN`] places.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TavernPart {
    /// A stack of its army.
    Stack(ArmyStack),
    /// A secondary skill with its level (1 Basic to 3 Expert).
    Skill {
        /// The skill.
        skill: SkillId,
        /// Its level.
        level: u8,
    },
    /// An artifact at its place: a position, or one of the backpack's first places.
    Artifact {
        /// Where it is.
        place: ArtifactPlace,
        /// The artifact.
        artifact: ArtifactInstance,
    },
}

/// A tavern hero as its owner sees it (TWN-TAV-002; decisions 134, 135; design note §75): the
/// hero screen's parts, the lists in [`TavernPart`]'s order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TavernHeroView {
    /// Its parts without a list.
    pub hero: TavernHero,
    /// Its army, by slot.
    pub army: Vec<ArmyStack>,
    /// Its secondary skills with their levels, in order.
    pub skills: Vec<(SkillId, u8)>,
    /// Its worn artifacts by position, then the backpack's first [`TAVERN_BACKPACK_SHOWN`].
    pub artifacts: Vec<(ArtifactPlace, ArtifactInstance)>,
}

impl TavernHeroView {
    /// The view of a tavern hero of `id` with `details` and the army, formation and tactics flag
    /// the pool keeps (`army`, `formation`, `tactics`).
    fn new(
        id: HeroId,
        details: &HeroDetails,
        (army, formation, tactics): (&[ArmyStack], Formation, bool),
    ) -> Self {
        let worn =
            details.worn.iter().enumerate().filter_map(|(at, item)| {
                item.map(|item| (ArtifactPlace::Worn(Position(at)), item))
            });
        let carried = details
            .backpack
            .iter()
            .take(TAVERN_BACKPACK_SHOWN)
            .enumerate()
            .map(|(index, &item)| (ArtifactPlace::Backpack(index), item));
        Self {
            hero: TavernHero {
                id,
                hero_type: details.hero_type,
                level: details.level,
                experience: details.experience,
                primary: details.primary,
                mana: details.mana,
                carried: u16::try_from(details.backpack.len()).unwrap_or(u16::MAX),
                formation,
                tactics,
            },
            army: army.to_vec(),
            skills: details.skills.clone(),
            artifacts: worn.chain(carried).collect(),
        }
    }

    /// Its parts in their order ([`TavernPart`]).
    pub fn parts(&self) -> impl Iterator<Item = TavernPart> + '_ {
        let stacks = self.army.iter().map(|&stack| TavernPart::Stack(stack));
        let skills = self
            .skills
            .iter()
            .map(|&(skill, level)| TavernPart::Skill { skill, level });
        let artifacts = self
            .artifacts
            .iter()
            .map(|&(place, artifact)| TavernPart::Artifact { place, artifact });
        stacks.chain(skills).chain(artifacts)
    }
}

/// The parts of the game the taverns write ([`GameState::taverns`]).
pub(crate) struct Parts<'s> {
    /// The hero pool with the players' tavern parts.
    pub(crate) pool: &'s mut Arc<HeroPool>,
    /// The store of heroes.
    pub(crate) heroes: &'s mut Arc<Vec<Arc<HeroDetails>>>,
    /// The heroes' level seeds.
    pub(crate) progression: &'s mut Arc<Progression>,
    /// The taverns' stream (`StreamId::TAVERNS`, decision 115).
    pub(crate) draws: &'s mut Rng,
}

/// The draw (TWN-TAV-001; `tavern_hero_draw` 0x4BB5E0, survey `E09-tavern.md` §2.2; design note
/// §62): a pure function of the pool, the core, the town part and its arguments, the player's
/// `colour` and `faction`, the class to `avoid` and whether only the faction's classes count
/// (`own_only`), from `draws`. Per hero class in the core's order its count, the types of the
/// class that are free and whose mask holds the colour, and its weight, the faction's record's
/// tavern weight of the class (every class 1 for a player without a faction or a record, the
/// agent's call); none counted: no type and no draw. A class without a count weighs 0; the class
/// to avoid weighs 0 while another class has a count; with `own_only`, when a class of the
/// faction still weighs anything, every other class weighs 0. A total weight of 0 (not reachable
/// with the shipped weights, each 5 or 6): no type and no draw. Else a class by one draw of 1 to
/// the total over the classes in order (the first whose running sum reaches it), then the n-th
/// counted type of the class in type order, n one draw of 1 to its count. The SoD edition's
/// Conflux rule is dead in this build (survey §2.2 step 3); no class is forced. The caller marks
/// the type offered.
// rule: TWN-TAV-001
pub(crate) fn draw(
    pool: &HeroPool,
    (core, part): (&Ruleset, &TownRuleset),
    (colour, faction): (u8, Option<FactionId>),
    (avoid, own_only): (Option<HeroClassId>, bool),
    draws: &mut Rng,
) -> Option<HeroTypeId> {
    let classes = core.hero_classes();
    let counts = pool.class_counts(classes.len(), colour);
    if counts.iter().all(|&count| count == 0) {
        return None;
    }
    let record = faction.and_then(|faction| part.town(faction));
    let mut weights: Vec<u32> = counts
        .iter()
        .enumerate()
        .map(|(class, &count)| match (count, record) {
            (0, _) => 0,
            (_, None) => 1,
            (_, Some(record)) => u32::from(record.tavern.get(class).copied().unwrap_or(0)),
        })
        .collect();
    if let Some(avoid) = avoid.map(HeroClassId::index) {
        let another = counts
            .iter()
            .enumerate()
            .any(|(class, &count)| class != avoid && count > 0);
        if another && let Some(weight) = weights.get_mut(avoid) {
            *weight = 0;
        }
    }
    if own_only {
        let own = |class: usize| faction.is_some_and(|f| classes[class].faction == f);
        if weights
            .iter()
            .enumerate()
            .any(|(class, &w)| own(class) && w > 0)
        {
            for (class, weight) in weights.iter_mut().enumerate() {
                if !own(class) {
                    *weight = 0;
                }
            }
        }
    }
    let total: u32 = weights.iter().sum();
    if total == 0 {
        return None;
    }
    let mut left = draws.range_inclusive(1, total);
    let class = weights.iter().position(|&weight| {
        let reached = left <= weight;
        left -= weight.min(left);
        reached
    });
    // The total's draw reaches a class with a weight, which has a count.
    class.and_then(|class| {
        let nth = draws.range_inclusive(1, counts[class]);
        let index = usize::try_from(nth - 1).unwrap_or(usize::MAX);
        pool.nth_free(class, colour, index)
    })
}

/// The week's release ([`HeroPool::release`]), the pool written only when a slot is released.
fn release(pool: &mut Arc<HeroPool>) {
    if pool.releases() {
        Arc::make_mut(pool).release();
    }
}

/// The fill (TWN-TAV-002; `tavern_new_heroes` 0x4C8450, survey §2.3; design note §63), per player
/// in player order (every player: none is defeated before E10), of `colours` by id: first, at the
/// game's start, the player's faction from `undrawn` (a `Random` choice no start read, TWN-SET-001:
/// uniform over its factions, the agent's call), then slot 0 then slot 1, each empty one: the
/// draw ([`draw`]: the other slot's hero's class to avoid, none when it is empty; own faction only
/// for slot 0); a type found is offered and the slot holds it; its record made when it has none
/// (the next hero id, GAM-HER-001's defaults with the default experience, then its level seed,
/// GAM-HER-002; design note §61); its backpack equipped (ADR-0013 point 14.5) and its spell points
/// full (HER-MAN-001); then its full starting army (GAM-SET-001: a rolled war machine's artifact
/// worn at the first free position that takes it, GAM-HER-001), replacing its away army (an empty
/// one for a type without a starting army). The original's order, so the draws are, per filled
/// slot: the class, the type, a new record's experience and level seed, the army. Its points come
/// at its owner's turn start ([`start_turn`]).
// rule: TWN-TAV-002, GAM-HER-001, GAM-HER-002
fn fill(parts: Parts<'_>, rules: &GameRuleset, colours: &[u8], undrawn: &[Option<FactionSet>]) {
    let (core, towns) = (rules.core(), rules.towns());
    let params = rules.adventure().params();
    let Parts {
        pool,
        heroes,
        progression,
        draws,
    } = parts;
    for (index, &colour) in colours.iter().enumerate() {
        let player = PlayerId::new(u8::try_from(index).unwrap_or(u8::MAX));
        let mut faction = pool.tavern(player).copied().unwrap_or_default().faction;
        let set = undrawn.get(index).copied().flatten();
        if let Some(set) = set.filter(|_| faction.is_none()) {
            let drawn = usize::try_from(draws.below(set.len())).unwrap_or(usize::MAX);
            faction = set.iter().nth(drawn);
            let tavern = Arc::make_mut(pool).tavern_mut(player);
            tavern
                .into_iter()
                .for_each(|tavern| tavern.faction = faction);
        }
        for slot in 0..2 {
            let tavern = pool.tavern(player).copied().unwrap_or_default();
            if tavern.slots[slot].is_some() {
                continue;
            }
            let avoid = tavern.slots[1 - slot]
                .and_then(|other| core.hero_type(other))
                .map(|ty| ty.class);
            let picked = draw(
                pool,
                (core, towns),
                (colour, faction),
                (avoid, slot == 0),
                draws,
            );
            let Some((hero_type, ty)) = picked.and_then(|t| core.hero_type(t).map(|ty| (t, ty)))
            else {
                continue;
            };
            let pool = Arc::make_mut(pool);
            pool.offer(hero_type);
            let held = pool.tavern_mut(player);
            held.into_iter()
                .for_each(|tavern| tavern.slots[slot] = Some(hero_type));
            let store = Arc::make_mut(heroes);
            let made = pool.record(hero_type).is_none();
            let id = record_of(pool, (store, progression), (core, hero_type, ty), draws);
            // A record's id indexes the store (the build's, or pushed above).
            let details = Arc::make_mut(&mut store[usize::from(id.raw())]);
            let changes = h3_heroes::backpack_equipped(details, core);
            for &change in &changes {
                h3_heroes::apply_change(details, core, change);
            }
            // A new record's defaults have them full already (GAM-HER-001).
            if !(made && changes.is_empty()) {
                details.mana = h3_heroes::max_mana(details, core);
            }
            let (army, machines) = starting_army(ty, params, draws).unwrap_or_default();
            wear_machines(details, core, &machines);
            // Full unless a war machine's artifact raised the maximum after them (none of the
            // packs' does).
            let full = machines.is_empty() || details.mana >= h3_heroes::max_mana(details, core);
            pool.set_army(hero_type, army, full);
        }
    }
}

/// The record of `hero_type` (`ty` its type), an offered type (design note §61, decision 109):
/// its record in the pool, else a new one made now, the next hero id with GAM-HER-001's defaults
/// and the default experience, then its level seed (GAM-HER-002), drawn from `draws` in that
/// order (a fill's from the taverns' stream, TWN-TAV-002; a replacement's from the hirer's,
/// TWN-TAV-004).
// rule: GAM-HER-001, GAM-HER-002
pub(super) fn record_of(
    pool: &mut HeroPool,
    (store, progression): (&mut Vec<Arc<HeroDetails>>, &mut Arc<Progression>),
    (core, hero_type, ty): (&Ruleset, HeroTypeId, &HeroType),
    draws: &mut Rng,
) -> HeroId {
    if let Some(id) = pool.record(hero_type) {
        return id;
    }
    let id = HeroId::new(u16::try_from(store.len()).unwrap_or(u16::MAX));
    let custom = HeroDetailsSetup::default();
    let details = hero_details(core, &custom, (hero_type, ty), &[], draws);
    Arc::make_mut(progression).record(draws);
    store.push(Arc::new(details));
    pool.set_record(hero_type, id);
    id
}

/// The game's start's fill (TWN-TAV-002; `new_game_heroes` 0x4C00C0's taverns, after the main
/// towns' heroes; design note §63): the same release (nothing is held yet) and fill, from the
/// taverns' stream, in the build after the setup stream's last draw and the adventure's start of
/// the game, before the first group's turn starts. `undrawn` by player id: the factions of a
/// `Random` choice no start read, drawn at its fill. Nothing is told: it is the first
/// observation's.
// rule: TWN-TAV-002
pub(crate) fn at_start(state: &mut GameState, rules: &GameRuleset, undrawn: &[Option<FactionSet>]) {
    let colours = colours(state);
    let parts = state.taverns();
    release(parts.pool);
    fill(parts, rules, &colours, undrawn);
}

/// The week's release and fill (TWN-TAV-002; GAM-CAL-002; design note §63), on a week's first day
/// after the towns' growth, before the adventure's week step, from the taverns' stream; then to
/// each player each slot whose view changed, in player then slot order: a
/// [`GameEvent::TavernOffered`] with its parts ([`tell`]). The views are read only for a sink that
/// keeps events.
// rule: TWN-TAV-002
pub(crate) fn weekly<S: Sink>(state: &mut GameState, rules: &GameRuleset, sink: &mut S) {
    let before = S::RECORDS.then(|| all_views(state));
    let colours = colours(state);
    let parts = state.taverns();
    release(parts.pool);
    fill(parts, rules, &colours, &[]);
    if let Some(before) = before {
        tell(state, &before, sink);
    }
}

/// Tells each slot whose view differs from `before` (each player's two by player id), in player
/// then slot order: its [`GameEvent::TavernOffered`], then a [`GameEvent::TavernPartSeen`] per
/// part ([`TavernHeroView::parts`]), all the player's alone.
pub(super) fn tell<S: Sink>(
    state: &GameState,
    before: &[[Option<TavernHeroView>; 2]],
    sink: &mut S,
) {
    for ((player, old), new) in (0..=u8::MAX)
        .map(PlayerId::new)
        .zip(before)
        .zip(all_views(state))
    {
        for (slot, (old, new)) in (0_u8..).zip(old.iter().zip(new)) {
            if *old == new {
                continue;
            }
            sink.emit(GameEvent::TavernOffered {
                player,
                slot,
                hero: new.as_ref().map(|view| view.hero),
            });
            for part in new.iter().flat_map(TavernHeroView::parts) {
                sink.emit(GameEvent::TavernPartSeen { player, slot, part });
            }
        }
    }
}

/// The players' colours, by id.
fn colours(state: &GameState) -> Vec<u8> {
    (0..=u8::MAX)
        .take(state.players())
        .map(|p| state.colour(PlayerId::new(p)).unwrap_or(u8::MAX))
        .collect()
}

/// Every player's two tavern heroes' views, by player id.
pub(super) fn all_views(state: &GameState) -> Vec<[Option<TavernHeroView>; 2]> {
    (0..=u8::MAX)
        .take(state.players())
        .map(|p| views(state, PlayerId::new(p)))
        .collect()
}

/// `player`'s two tavern heroes as it sees them, by slot (design note §75; decision 134): none
/// for an empty slot or a player not of the game.
// rule: TWN-TAV-002
pub(crate) fn views(state: &GameState, player: PlayerId) -> [Option<TavernHeroView>; 2] {
    let pool = state.pool();
    let slots = pool.tavern(player).map_or([None; 2], |t| t.slots);
    slots.map(|slot| {
        // A slot's type has its record and away record (the fill's).
        let record = slot.and_then(|t| pool.record(t).zip(pool.away(t)));
        record.and_then(|(id, away)| {
            let details = state.heroes().get(usize::from(id.raw()));
            details.map(|d| TavernHeroView::new(id, d, (&away.army, away.formation, away.tactics)))
        })
    })
}

/// The tavern heroes at `player`'s turn start (TWN-TAV-005; `next_player_turn` 0x4C6FE0, survey
/// §2.3; design note §68), after its heroes on the map and its garrison heroes (none draws): each
/// one's day's maximum and points left its land points, `h3_adventure::day_points` over its
/// details and its away army (ADR-0013 point 13; no week marks), for each whose army or record
/// changed since its points were set ([`super::pool::Away::stale`]: nothing else changes what the
/// derivation reads off the map, so the others' are its value already).
// rule: TWN-TAV-005
pub(crate) fn start_turn(state: &mut GameState, rules: &GameRuleset, player: PlayerId) {
    let (core, part) = (rules.core(), rules.adventure());
    let pool = state.pool();
    let slots = pool.tavern(player).map_or([None; 2], |t| t.slots);
    let points = slots.map(|slot| {
        let stale = slot.and_then(|t| pool.away(t).filter(|away| away.stale).map(|a| (t, a)));
        stale.map(|(hero_type, away)| {
            let id = pool.record(hero_type);
            let details = id.and_then(|id| state.heroes().get(usize::from(id.raw())));
            let details = details.map(|d| &**d);
            (
                hero_type,
                h3_adventure::day_points(core, part, details, &away.army, 0),
            )
        })
    });
    if points.iter().all(Option::is_none) {
        return;
    }
    let pool = Arc::make_mut(state.taverns().pool);
    for (hero_type, points) in points.into_iter().flatten() {
        pool.away_mut(hero_type).into_iter().for_each(|away| {
            away.movement = points;
            away.max_movement = points;
            away.stale = false;
        });
    }
}

/// The returning marks cleared (TWN-TAV-005; design note §68): `player`'s two at its turn's end
/// (`next_player_turn` 0x4C705D, after the victory tests), every player's at a day's start (`None`;
/// decision 126, after the week's release: `new_day` 0x4C80CF to 0x4C80FD). The pool is written
/// only when a mark is set.
// rule: TWN-TAV-005
pub(crate) fn clear_marks(state: &mut GameState, player: Option<PlayerId>) {
    let marked = |(p, t): (usize, &Tavern)| {
        player.is_none_or(|player| player.index() == p) && t.returning.iter().any(|&r| r)
    };
    if !state.pool().taverns().iter().enumerate().any(marked) {
        return;
    }
    let pool = Arc::make_mut(state.taverns().pool);
    for p in (0..=u8::MAX).take(pool.taverns().len()).map(PlayerId::new) {
        let tavern = pool
            .tavern_mut(p)
            .filter(|_| player.is_none_or(|player| player == p));
        tavern
            .into_iter()
            .for_each(|tavern| tavern.returning = [false; 2]);
    }
}
