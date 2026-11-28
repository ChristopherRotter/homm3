//! The legal commands (ADR-0017 point 1, ADR-0004 §7; E08 T8b2a): what an actor may issue now,
//! built from the checks `apply` makes; while an actor has an open decision, its answers only
//! (ADR-0017 point 6, GAM-DEC-001; E08 T14b; a level-up's since T30, GAM-DEC-002), or its
//! battle's commands (T19a); and the system actor's default answer ([`default_answer`], T30).
//! Moves are not listed one by one: a hero with many points
//! reaches thousands of tiles, each with its own path. They are given per hero as the
//! reachable-tiles query ([`legal_moves`]), a path built only for the destination chosen, as the
//! battle gives casts by their targets. Consistency with `apply` is property-tested on imported
//! maps (`h3-map`'s tests). The management commands are listed per hero too ([`legal_management`],
//! a split's counts by [`split_range`]; E08 T51a, ADR-0017 point 6): a flat list would grow every
//! call in a player's turn by up to a few thousand commands.

use alloc::vec;
use alloc::vec::Vec;
use core::ops::RangeInclusive;

use h3_adventure::{AdventureCommand, ArmyRef, ArmySlot, ChoiceKind, Pos, Reach};
use h3_battle::LegalCommands;
use h3_content::{CreatureTypeId, Formation, SpellId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::{
    ArmyWorth, ArtifactMove, ArtifactPlace, HeroDetails, Position, SlotState, check_assemble,
    check_disassemble, check_move, original_choice,
};

use crate::command::{
    Actor, AltarAnswer, Answer, ArenaChoice, ChestChoice, Consent, GameCommand, HillFortAnswer,
    HireAnswer, MarketAnswer, SchoolChoice, SkillOffer, TavernAnswer, UniversityAnswer,
};
use crate::manage::Manage;
use crate::state::Together;
use crate::{GameRuleset, GameState, OpenBattle};

/// The commands `actor` may issue now but its moves: with an open battle the battle's legal
/// commands for its side while that side is to act, and none while the other side is
/// (`h3_battle::legal_commands`; GAM-BAT-001, E08 T19a); otherwise with an open decision its
/// answers alone, in its turn or not ([`answers`]: GAM-DEC-001 to GAM-DEC-006, GAM-DEC-008), and
/// none for any
/// other actor while the game waits for a level-up (GAM-DEC-002, E08 T30); otherwise `EndTurn`,
/// then since E08 T84b2 a `BuildBoat` for each Shipyard of the player's or an ally's whose boat
/// tile is free while the treasury can pay its price, in object order (ADV-SHY-003:
/// `AdventureState::buildable_shipyards`), then since E08 T105b2 a `Revisit` for each of the
/// player's heroes on the visitable tile of an entered object whose visit is built, in the order
/// of their ids (ADV-TEL-008: `AdventureState::revisitable`; at 0 points too, so a turn may
/// revisit without end, increment 11's design note §18), then since E09 T18b2b a town's build
/// (`GameCommand::Town`) for each town the player owns, in `TownId` order, and each building its
/// hall offers whose price the treasury holds, in the record's order (TWN-BLD-001:
/// `GameState::town_offer`'s payable set; none for a town built in today, TWN-BLD-004), each
/// town's builds followed by its recruitments (E09 T27), its Tavern's hires by slot (E09 T38a2,
/// TWN-TAV-003) and its hero moves between its bars (E09 T28a), then since E08 T144a1 its heroes'
/// casts on the map without a target (`GameCommand::Cast`, GAM-SPL-001: per hero in the
/// player's list, Summon Boat, Fly and Water Walk where the hero may cast them, in the part's
/// order; increment 15's design note §17, E08 decision 422). Scuttle Boat's and Dimension Door's
/// casts are given per hero by their targets ([`legal_cast_targets`]), as the moves are. Empty for
/// a player whose turn is not running or who is not in the game, by `apply`'s own checks
/// (ADR-0017's consequence). The moves are [`legal_moves`]; a meeting's, an army dialog's and an
/// exchange's management commands [`legal_management`]'s.
// rule: GAM-DEC-001, GAM-DEC-002, GAM-DEC-003, GAM-DEC-004, GAM-DEC-005, GAM-DEC-006, GAM-BAT-001
// rule: GAM-DEC-008, GAM-DEC-009, ADV-SHY-003, ADV-TEL-008, TWN-BLD-001, TWN-TAV-003
// rule: GAM-DEC-010, GAM-DEC-011, GAM-DEC-012, GAM-DEC-013, GAM-SPL-001
pub fn legal_commands(state: &GameState, rules: &GameRuleset, actor: Actor) -> Vec<GameCommand> {
    if state.check_held(actor).is_err() {
        return Vec::new();
    }
    let player = match actor {
        // E08 T20b2: the system actor's are the unowned side's of the first battle it waits for.
        Actor::System => {
            return state
                .unowned_battle()
                .map(|battle| battle_commands(battle, rules, None))
                .unwrap_or_default();
        }
        Actor::Player(player) => player,
    };
    if let Some(battle) = state.battle(player) {
        return battle_commands(battle, rules, Some(player));
    }
    if state.has_decision(player) {
        return answers(state, rules, player)
            .into_iter()
            .map(GameCommand::Answer)
            .collect();
    }
    if state.check_acting(player).is_err() {
        return Vec::new();
    }
    let boats = state
        .adventure()
        .buildable_shipyards(rules.adventure(), player)
        .filter(|&(_, _, price)| state.can_pay(player, price))
        .map(|(object_type, anchor, _)| {
            GameCommand::Adventure(AdventureCommand::BuildBoat {
                object_type,
                anchor,
            })
        });
    let revisits = state
        .adventure()
        .revisitable(rules.adventure(), player)
        .map(|hero| GameCommand::Adventure(AdventureCommand::Revisit { hero }));
    // E09 T38a2: folded into a list sized for a turn's usual commands (the end, a town's builds,
    // recruitments and hires), so that the chain's parts run without the nested `next` and the
    // list without its first regrowths (`legal-towns-36`).
    let mut listed = Vec::with_capacity(LISTED);
    core::iter::once(GameCommand::EndTurn)
        .chain(boats)
        .chain(revisits)
        .chain(crate::town::legal_builds(state, rules, player))
        .for_each(|command| listed.push(command));
    state.legal_casts(rules, player, &mut listed);
    listed
}

/// The room [`legal_commands`] reserves for a player's commands in its turn (E09 T38a2).
const LISTED: usize = 16;

/// The answers to `player`'s first open decision but a battle (ADR-0017 point 6; E08 T63a: typed
/// by the decision's kind), in this order: an offer's `Accept` while it stands (ADR-0017 point
/// 10: not a border guard another player of the group passed, E08 T15a), then `Decline`
/// (GAM-DEC-001); a level-up's `First`, then `Second` (GAM-DEC-002); a meeting's or an army
/// dialog's `Close` (GAM-DEC-003, GAM-DEC-004); a chest's `Gold`, then `Experience`; the Arena's
/// `Attack`, then `Defense`; a School's `First`, `Second`, then `Leave` (GAM-DEC-005); a
/// University's `Buy` of each slot open for the hero while the treasury can pay its price, in
/// slot order, then `Leave` (GAM-DEC-006); a hire dialog's `Buy` of the most of each creature in
/// the source's order whose most is at least 1 (`GameState::hire_mosts`), then `Close`
/// (GAM-DEC-008, E08 T95b; since E10 T3b1 an ally's dwelling's `Close` alone, ADV-DWL-004); a map
/// Tavern's `Hire` of each slot whose hire passes, in slot order,
/// then `Close` (GAM-DEC-010, E09 T39a); since E08 T135a (increment 14's design note §18) a
/// representative answer per action, then `Leave`: a market's trades of one lot, sales of one lot
/// for the cost resource and purchases in it (GAM-DEC-011, `GameState::market_answers`: no
/// transfer), an Altar's two alls (GAM-DEC-012, `GameState::altar_answers`), a Hill Fort's
/// upgrades of state 1, then `All` (GAM-DEC-013, `GameState::hill_fort_answers`); every other
/// answer their views allow is legal for `apply`.
fn answers(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<Answer> {
    if let Some(offer) = state.decision(player) {
        let accept = state.adventure().offer_stands(offer);
        return [Consent::Accept, Consent::Decline]
            .into_iter()
            .filter(|&consent| accept || consent == Consent::Decline)
            .map(Answer::Offer)
            .collect();
    }
    if state.level_up(player).is_some() {
        return vec![
            Answer::LevelUp(SkillOffer::First),
            Answer::LevelUp(SkillOffer::Second),
        ];
    }
    if let Some(choice) = state.choice(player) {
        return match choice.kind {
            ChoiceKind::Chest { .. } => vec![
                Answer::Chest(ChestChoice::Gold),
                Answer::Chest(ChestChoice::Experience),
            ],
            ChoiceKind::Primary { leave: false, .. } => vec![
                Answer::Arena(ArenaChoice::Attack),
                Answer::Arena(ArenaChoice::Defense),
            ],
            ChoiceKind::Primary { leave: true, .. } => vec![
                Answer::School(SchoolChoice::First),
                Answer::School(SchoolChoice::Second),
                Answer::School(SchoolChoice::Leave),
            ],
        };
    }
    if let Some(visit) = state.university(player) {
        let pays = state.can_pay(player, &visit.price);
        return (0..=u8::MAX)
            .zip(&visit.skills)
            .filter(|&(_, &skill)| {
                pays && state.slot_state(rules, visit.hero, skill) == SlotState::Open
            })
            .map(|(slot, _)| Answer::University(UniversityAnswer::Buy(slot)))
            .chain([Answer::University(UniversityAnswer::Leave)])
            .collect();
    }
    if let Some(dialog) = state.hire_dialog(player) {
        // rule: ADV-DWL-004
        // E10 T3b1: an ally's dwelling's dialog is read-only, `Close` alone.
        if state.adventure().hire_read_only(dialog) {
            return vec![Answer::Hire(HireAnswer::Close)];
        }
        return state
            .hire_mosts(rules, player, dialog)
            .into_iter()
            .filter(|&(_, most)| most > 0)
            .map(|(creature, amount)| Answer::Hire(HireAnswer::Buy { creature, amount }))
            .chain([Answer::Hire(HireAnswer::Close)])
            .collect();
    }
    // rule: GAM-DEC-010
    if let Some(visit) = state.tavern(player) {
        return crate::town::hire::map_hires(state, rules, (player, visit))
            .chain([Answer::Tavern(TavernAnswer::Close)])
            .collect();
    }
    // rule: GAM-DEC-011
    if let Some(visit) = state.market(player) {
        return state
            .market_answers(rules, player, visit)
            .into_iter()
            .chain([MarketAnswer::Leave])
            .map(Answer::Market)
            .collect();
    }
    // rule: GAM-DEC-012
    if let Some(visit) = state.altar(player) {
        return state
            .altar_answers(rules, visit)
            .into_iter()
            .chain([AltarAnswer::Leave])
            .map(Answer::Altar)
            .collect();
    }
    // rule: GAM-DEC-013
    if let Some(visit) = state.hill_fort(player) {
        return state
            .hill_fort_answers(rules, player, visit)
            .into_iter()
            .chain([HillFortAnswer::Leave])
            .map(Answer::HillFort)
            .collect();
    }
    // The other decisions but a battle: a meeting, an army dialog or an exchange.
    vec![Answer::Close]
}

/// The system actor's default answer to `player`'s first open decision (ADR-0017 points 2 and 6;
/// E08 T30, T63a), which the hosting layer gives when the player does not answer (the server
/// lane's M5): an offer's `Decline`; a meeting's and an army dialog's `Close` (E08 T51b), an
/// exchange's (E08 T114a2); a
/// chest's `Gold` (the original's plain OK), the Arena's `Attack` (its Cancel), a School's and a
/// University's `Leave`, a hire dialog's `Close` (the original's Close, E08 T95b), a map Tavern's
/// `Close` (E09 T39a), a market's `Leave` (E08 T133b: nothing lost), an Altar's `Leave` (E08
/// T134a: nothing lies on the altar, A-48 fixed), a Hill Fort's `Leave` (E08 T134b1); for a
/// level-up
/// the offer the original's choice takes (HER-LVL-006, the original's when the dialog's time runs
/// out), with the setup's army worth and the map's terrain shares, as at load (GAM-SET-002), until
/// E10 gives the engine the computer player's state its flag-set path reads (ADR-0017 point 6,
/// ADV-Q17: `ADV-STUB-8`); none for a battle (a battle's default is the server lane's) or without
/// an open decision.
// rule: GAM-DEC-002, GAM-DEC-005, GAM-DEC-006, GAM-DEC-008, GAM-DEC-010, GAM-DEC-011
// rule: GAM-DEC-012, GAM-DEC-013
pub fn default_answer(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Option<Answer> {
    if state.decision(player).is_some() {
        return Some(Answer::Offer(Consent::Decline));
    }
    if let Some(choice) = state.choice(player) {
        return Some(match choice.kind {
            ChoiceKind::Chest { .. } => Answer::Chest(ChestChoice::Gold),
            ChoiceKind::Primary { leave: false, .. } => Answer::Arena(ArenaChoice::Attack),
            ChoiceKind::Primary { leave: true, .. } => Answer::School(SchoolChoice::Leave),
        });
    }
    if state.university(player).is_some() {
        return Some(Answer::University(UniversityAnswer::Leave));
    }
    if state.hire_dialog(player).is_some() {
        return Some(Answer::Hire(HireAnswer::Close));
    }
    if state.tavern(player).is_some() {
        return Some(Answer::Tavern(TavernAnswer::Close));
    }
    if state.market(player).is_some() {
        return Some(Answer::Market(MarketAnswer::Leave));
    }
    if state.altar(player).is_some() {
        return Some(Answer::Altar(AltarAnswer::Leave));
    }
    if state.hill_fort(player).is_some() {
        return Some(Answer::HillFort(HillFortAnswer::Leave));
    }
    if !matches!(state.together(player), Some(Together::Alone) | None) {
        return Some(Answer::Close);
    }
    state.open_level_up(player).and_then(|(view, level_up)| {
        // stub: ADV-STUB-8
        let shares = state.adventure().map().terrain_shares();
        state.hero(view.hero).map(|details| {
            let worth = ArmyWorth::SETUP;
            Answer::LevelUp(SkillOffer::of(original_choice(
                details,
                rules.core(),
                &level_up,
                worth,
                shares,
            )))
        })
    })
}

/// The legal commands of `battle` for the side `owner` owns (none: an unowned side) while it is
/// to act; none while the other side is.
fn battle_commands(
    battle: &OpenBattle,
    rules: &GameRuleset,
    owner: Option<PlayerId>,
) -> Vec<GameCommand> {
    let mut legal = LegalCommands::new();
    if let Some(side) = battle
        .state()
        .side_to_act()
        .filter(|side| battle.owners()[side.index()] == owner)
    {
        h3_battle::legal_commands(battle.state(), rules.battle(), side, &mut legal);
    }
    legal
        .iter()
        .map(|&command| GameCommand::Battle(command))
        .collect()
}

/// Where `actor` may move `hero` now: the hero's reachable-tiles query
/// ([`h3_adventure::AdventureState::reach`]: the tiles its owner has explored, ADV-MOV-008), each
/// tile with the points the hero would have left there and the query's path to it, which a
/// `MoveHero` takes as it is. `None` when the actor's turn is not running or it has an open
/// decision, by `apply`'s own checks (GAM-DEC-001), or the hero is not the actor's.
///
/// The query keeps one path per tile; `apply` accepts every path the hero can pay, also another
/// one to a listed tile or one back to the hero's own (for a hero that starts on the entrance
/// of a town its team does not own, TWN-SET-010's heroes, the step back onto it is that town's
/// visit: its siege from the tile before when its garrison defends it, E09 T8b2a). A longer path
/// may rarely reach a tile the query does not list ([`Reach`]); every tile one step away is
/// listed. A client may so compute its own paths: `apply` checks each step against the state and
/// the actor.
pub fn legal_moves(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    hero: HeroId,
) -> Option<Reach> {
    let Actor::Player(player) = actor else {
        return None;
    };
    state.check_ready(player).ok()?;
    let adventure = state.adventure();
    adventure
        .hero(hero)
        .filter(|h| h.owner == player)
        .and_then(|h| adventure.reach(rules.core(), rules.adventure(), state.heroes(), h.id))
}

/// Where `actor` may cast `spell`, Scuttle Boat or Dimension Door, by its hero `hero` now
/// (GAM-SPL-001; E08 T144a1, E08 decision 422): the targets of the spell's window, in row order,
/// each a `GameCommand::Cast { hero, spell, target: Some(target) }` that `apply` accepts (a
/// charged failure too: under `strict-fixed` the hidden tiles of the window, under `faithful`
/// the door's wrong kind; increment 15's design note §11). Given per hero as its moves are
/// ([`legal_moves`]): up to 323 targets each, which [`legal_commands`] does not list. Empty when
/// the actor's turn is not running or it has an open decision, by `apply`'s own checks
/// (GAM-DEC-001), for a hero not the actor's, a spell without a target, or one the hero may not
/// cast there now.
// rule: GAM-SPL-001
pub fn legal_cast_targets(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    (hero, spell): (HeroId, SpellId),
) -> Vec<Pos> {
    let Actor::Player(player) = actor else {
        return Vec::new();
    };
    if state.check_ready(player).is_err() {
        return Vec::new();
    }
    state.cast_targets(rules, player, (hero, spell))
}

/// The management commands `actor` may give on its hero `hero` now but the splits (GAM-MAN-002;
/// E08 T51a, T51b; ADR-0017 point 6: listed per hero, as its moves are), each as `apply` accepts
/// it: its army's commands ([`h3_adventure::AdventureState::army_commands`]: each stack's moves
/// onto the army's other slots, then onto the other army's slots, a meeting's other hero's or an
/// army dialog's group's or an exchange's garrison's, then its dismissals; with an army dialog or
/// an exchange open the group's or the garrison's own after them, by the one owner rule
/// (ADV-GAR-003; E08 T114a2): its stacks' moves onto its other slots and onto the hero's, its
/// dismissals), both
/// formations and both values of the flag, each artifact's moves onto every place of the hero
/// and then, with a meeting open, of the other hero that takes it (`h3_heroes::check_move`: the
/// sources and the targets in position order, then by backpack index, a target up to the
/// backpack's length), then the assemblies and the disassemblies by position, then since E09
/// T38b2 outside a decision the hero's dismissal when it passes (GAM-MAN-003: on the map, not a
/// town's visitor, not the player's one hero on the map while it owns no town). Empty when the
/// actor's turn is not running, it has another open decision or a meeting, an army dialog or an
/// exchange that does not hold the hero, by `apply`'s own checks, or the hero is not its on the map. A split's
/// counts are [`split_range`]'s.
// rule: GAM-MAN-002, GAM-MAN-003
pub fn legal_management(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    hero: HeroId,
) -> Vec<Manage> {
    let own = match actor {
        Actor::Player(player) => state
            .check_managing(player)
            .ok()
            .filter(|together| together.admits_heroes(core::iter::once(hero)).is_ok())
            .and_then(|together| {
                let details = state.own_details(player, hero).ok()?;
                Some((player, together, details))
            }),
        Actor::System => None,
    };
    let Some((player, together, details)) = own else {
        return Vec::new();
    };
    let (core, part) = (rules.core(), rules.adventure());
    let group = state.army_dialog(player).map(|(_, group)| group);
    let other = match together {
        Together::Meeting([visitor, visited]) => {
            Some(if visitor == hero { visited } else { visitor })
        }
        Together::Alone | Together::Dialog(_) | Together::Exchange { .. } => None,
    };
    // The army held with the hero without a hero of its own: an army dialog's group, an
    // exchange's stationary army (E08 T114a2).
    let stationary = match together {
        Together::Dialog(_) => Some(ArmyRef::Group),
        Together::Exchange { army, .. } => Some(army),
        Together::Alone | Together::Meeting(_) => None,
    };
    // E09 T28a (TWN-ROW-001): outside a decision, a visitor of an own town and its upper bar, a
    // garrison hero and its town's visitor.
    let bars = match together {
        Together::Alone => town_partner(state, player, hero),
        Together::Meeting(_) | Together::Dialog(_) | Together::Exchange { .. } => None,
    };
    let toward = other.map(ArmyRef::Hero).or(stationary).or(bars);
    let own_army = ArmyRef::Hero(hero);
    let adventure = state.adventure();
    let army = adventure.army_commands(part, player, (own_army, toward), group);
    let grouped = stationary.map_or_else(Vec::new, |stationary| {
        adventure.army_commands(part, player, (stationary, Some(own_army)), group)
    });
    let formations =
        [Formation::Loose, Formation::Tight].map(|formation| Manage::Formation { hero, formation });
    let flags = [true, false].map(|on| Manage::Tactics { hero, on });
    let positions = (0..details.worn.len()).map(Position);
    // A hero's places: its positions, then its backpack's indices up to its length and `extra`.
    let places = |details: &HeroDetails, extra: usize| {
        (0..details.worn.len())
            .map(|p| ArtifactPlace::Worn(Position(p)))
            .chain((0..details.backpack.len() + extra).map(ArtifactPlace::Backpack))
    };
    // A meeting's other hero takes artifacts too (HER-ART-005); since E10 T3b1 an ally's into
    // its empty positions and its backpack alone (GAM-TEM-003), and nothing is taken from it: its
    // own commands are none (`own_details` refuses it above).
    let ally = state.meeting_ally(player, together);
    let receiver = other.and_then(|id| {
        let details = match ally {
            Some(ally) if ally == id => state.hero(ally),
            _ => state.own_details(player, id).ok(),
        };
        details.map(|d| (id, d))
    });
    let gift = move |id: HeroId, other: &HeroDetails, to: ArtifactPlace| {
        ally != Some(id)
            || match to {
                ArtifactPlace::Worn(Position(at)) => {
                    other.worn.get(at).is_some_and(Option::is_none)
                }
                ArtifactPlace::Backpack(_) => true,
            }
    };
    let targets =
        places(details, 1)
            .map(|to| (hero, to, None))
            .chain(receiver.into_iter().flat_map(move |(id, other)| {
                places(other, 1)
                    .filter(move |&to| gift(id, other, to))
                    .map(move |to| (id, to, Some(other)))
            }));
    let moves = places(details, 0)
        .flat_map(|from| {
            targets
                .clone()
                .map(move |(id, to, other)| (from, (id, to), other))
        })
        .filter(|&(from, (_, to), other)| {
            check_move(details, other, core, ArtifactMove { from, to }).is_ok()
        })
        .map(|(from, to, _)| Manage::Artifact {
            from: (hero, from),
            to,
        });
    let assemblies = positions
        .clone()
        .filter(|&at| check_assemble(details, core, at).is_ok())
        .map(|at| Manage::Assemble { hero, at });
    let disassemblies = positions
        .filter(|&at| check_disassemble(details, core, at).is_ok())
        .map(|at| Manage::Disassemble { hero, at });
    // E09 T38b2 (GAM-MAN-003): outside a decision, a hero on the map that may be dismissed.
    let dismissal = (together == Together::Alone)
        .then(|| adventure.hero(hero))
        .flatten()
        .filter(|on| crate::town::leave::check_dismissal(adventure, player, on).is_ok())
        .map(|_| Manage::Dismiss { hero });
    army.into_iter()
        .chain(grouped)
        .map(Manage::Army)
        .chain(formations)
        .chain(flags)
        .chain(moves)
        .chain(assemblies)
        .chain(disassemblies)
        .chain(dismissal)
        .collect()
}

/// The army of a town's other bar that `player`'s hero `hero` meets outside a decision
/// (TWN-ROW-001; E09 T28a, design §52): a visitor of an own town, the town's upper bar (its
/// garrison hero's army, else its garrison); a garrison hero, its town's visitor's army when the
/// visitor is the player's. None for any other hero.
fn town_partner(state: &GameState, player: PlayerId, hero: HeroId) -> Option<ArmyRef> {
    let adventure = state.adventure();
    if let Some((town, _)) = adventure.garrisoned(hero) {
        let visitor = adventure.town_visitor(town)?;
        let own = adventure.hero(visitor).is_some_and(|h| h.owner == player);
        return own.then_some(ArmyRef::Hero(visitor));
    }
    let map = adventure.map();
    let town = adventure
        .hero(hero)
        .and_then(|on| map.town_at(on.pos))
        .filter(|&town| map.town_object(town).and_then(|i| adventure.owner(i)) == Some(player))?;
    Some(
        adventure
            .town_hero(town)
            .map_or(ArmyRef::Garrison(town), |held| ArmyRef::Hero(held.id)),
    )
}

/// The management commands `actor` may give on the upper bar of its town `town` now but the
/// splits (GAM-MAN-002 amended, TWN-ROW-001; E09 T28a, design §52), each as `apply` accepts it:
/// with a garrison hero its whole management ([`legal_management`] with that hero: its army's
/// moves within it and onto its town's visitor's, its dismissals, formation, flag and
/// artifacts); without one the town's garrison's moves onto its other slots and onto the
/// visitor's army, if the player's hero visits, then its dismissals
/// ([`h3_adventure::AdventureState::army_commands`], the one owner rule included). Empty when the
/// actor's turn is not running, it has an open decision, the game waits for another player's
/// level-up, or the town is not its.
// rule: GAM-MAN-002, TWN-ROW-001
pub fn legal_town_management(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    town: TownId,
) -> Vec<Manage> {
    let Actor::Player(player) = actor else {
        return Vec::new();
    };
    let adventure = state.adventure();
    if let Some(held) = adventure.town_hero(town) {
        return legal_management(state, rules, actor, held.id);
    }
    let alone = state.check_managing(player).ok() == Some(Together::Alone);
    if !alone {
        return Vec::new();
    }
    let visitor = adventure
        .town_visitor(town)
        .filter(|&hero| adventure.hero(hero).is_some_and(|h| h.owner == player));
    let armies = (ArmyRef::Garrison(town), visitor.map(ArmyRef::Hero));
    adventure
        .army_commands(rules.adventure(), player, armies, None)
        .into_iter()
        .map(Manage::Army)
        .collect()
}

/// The counts a split of `actor`'s stack at `from` with `to` may leave in `to` (GAM-MAN-002,
/// ADV-ARM-003, ADV-ARM-007; E08 T51a, T51b): the adventure's
/// [`h3_adventure::AdventureState::split_range`], with the open army dialog's group. `None` when
/// the actor may not give the split now (its turn, its open decision: `apply`'s checks), the two
/// slots' armies are not together (one army outside a decision, a meeting's two heroes, an army
/// dialog's hero and group, an exchange's hero and garrison), or no split is possible there (the
/// one owner rule included, ADV-GAR-003).
// rule: GAM-MAN-002
pub fn split_range(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    (from, to): (ArmySlot, ArmySlot),
) -> Option<RangeInclusive<u32>> {
    let Actor::Player(player) = actor else {
        return None;
    };
    let armies = (from.army, (to.army != from.army).then_some(to.army));
    let together = state.check_managing(player).ok()?;
    state.admits_army(player, together, armies).ok()?;
    let group = state.army_dialog(player).map(|(_, group)| group);
    state
        .adventure()
        .split_range(rules.adventure(), player, (from, to), group)
}

/// The amounts of `creature` `player` may buy now in its open hire dialog (GAM-DEC-008,
/// ADV-HIR-002; E08 T95b, increment 10's design note §13), as [`split_range`] gives a split's
/// counts: from 1 to the most (`GameState::hire_mosts`: the least of the source's count, the
/// treasury divided by the creature's cost for each resource, and the room in the army), each a
/// `Buy` `apply` accepts. Empty when the game waits for another player's level-up, the player's
/// first open decision is no hire dialog, or its source does not sell the creature.
// rule: GAM-DEC-008
pub fn hire_range(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    creature: CreatureTypeId,
) -> RangeInclusive<u32> {
    let most = state
        .check_held(Actor::Player(player))
        .ok()
        .and_then(|()| state.hire_dialog(player))
        .and_then(|dialog| {
            state
                .hire_mosts(rules, player, dialog)
                .into_iter()
                .find(|&(sold, _)| sold == creature)
        })
        .map_or(0, |(_, most)| most);
    1..=most
}
