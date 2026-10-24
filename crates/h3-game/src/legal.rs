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

use h3_adventure::{ArmyRef, ArmySlot, Reach};
use h3_battle::LegalCommands;
use h3_content::Formation;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{
    ArmyWorth, ArtifactMove, ArtifactPlace, Position, check_assemble, check_disassemble,
    check_move, original_choice,
};

use crate::command::{Actor, Answer, GameCommand};
use crate::manage::Manage;
use crate::{GameRuleset, GameState, OpenBattle};

/// The commands `actor` may issue now but its moves: with an open battle the battle's legal
/// commands for its side while that side is to act, and none while the other side is
/// (`h3_battle::legal_commands`; GAM-BAT-001, E08 T19a); with an open offer its answers, `Accept`
/// then `Decline`, in its turn or not, and nothing else (GAM-DEC-001); `Decline` alone when the
/// offer no longer stands (ADR-0017 point 10: a border guard another player of the group passed,
/// E08 T15a); with an open level-up its answers, `First` then `Second`, in its turn or not, and
/// nothing else, and none for any other actor while the game waits for it (GAM-DEC-002, E08
/// T30); otherwise `EndTurn`. Empty for a player whose turn is not
/// running or who is not in the game, by `apply`'s own checks (ADR-0017's consequence). The moves
/// are [`legal_moves`].
// rule: GAM-DEC-001, GAM-DEC-002, GAM-BAT-001
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
    if let Some(offer) = state.decision(player) {
        let accept = state.adventure().offer_stands(offer);
        return [Answer::Accept, Answer::Decline]
            .into_iter()
            .filter(|&answer| accept || answer == Answer::Decline)
            .map(GameCommand::Answer)
            .collect();
    }
    if state.level_up(player).is_some() {
        return vec![
            GameCommand::Answer(Answer::First),
            GameCommand::Answer(Answer::Second),
        ];
    }
    if state.check_acting(player).is_err() {
        return Vec::new();
    }
    vec![GameCommand::EndTurn]
}

/// The system actor's default answer to `player`'s first open decision (ADR-0017 points 2 and 6;
/// E08 T30), which the hosting layer gives when the player does not answer (the server lane's
/// M5): `Decline` for an offer; for a level-up the offer the original's choice takes (HER-LVL-006,
/// the original's when the dialog's time runs out), with the setup's army worth and the map's
/// terrain shares, as at load (GAM-SET-002), until E10 gives the engine the computer player's
/// state its flag-set path reads (ADR-0017 point 6, ADV-Q17: `ADV-STUB-8`); none for a battle (a
/// battle's default is the server lane's) or without an open decision.
// rule: GAM-DEC-002
pub fn default_answer(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Option<Answer> {
    if state.decision(player).is_some() {
        return Some(Answer::Decline);
    }
    state.open_level_up(player).and_then(|(view, level_up)| {
        // stub: ADV-STUB-8
        let shares = state.adventure().map().terrain_shares();
        state.hero(view.hero).map(|details| {
            let worth = ArmyWorth::SETUP;
            Answer::of(original_choice(
                details,
                rules.core(),
                &level_up,
                worth,
                shares,
            ))
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
/// one to a listed tile or one back to the hero's own. A longer path may rarely reach a tile the
/// query does not list ([`Reach`]); every tile one step away is listed. A client may so compute
/// its own paths: `apply` checks each step against the state and the actor.
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

/// The management commands `actor` may give on its hero `hero` now but the splits (GAM-MAN-002;
/// E08 T51a, ADR-0017 point 6: listed per hero, as its moves are), each as `apply` accepts it: its
/// army's commands ([`h3_adventure::AdventureState::army_commands`]: each stack's moves onto the
/// army's other slots, then its dismissals), both formations and both values of the flag, each
/// artifact's moves onto every place of the hero that takes it (`h3_heroes::check_move`: the
/// sources and the targets in position order, then by backpack index, a target up to the
/// backpack's length), then the assemblies and the disassemblies by position. Empty when the
/// actor's turn is not running or it has an open decision, by `apply`'s own checks, or the hero
/// is not its on the map. A split's counts are [`split_range`]'s.
// rule: GAM-MAN-002
pub fn legal_management(
    state: &GameState,
    rules: &GameRuleset,
    actor: Actor,
    hero: HeroId,
) -> Vec<Manage> {
    let own = match actor {
        Actor::Player(player) => state
            .check_ready(player)
            .ok()
            .and_then(|()| state.own_details(player, hero).ok())
            .map(|details| (player, details)),
        Actor::System => None,
    };
    let Some((player, details)) = own else {
        return Vec::new();
    };
    let core = rules.core();
    let army = state
        .adventure()
        .army_commands(rules.adventure(), player, (ArmyRef::Hero(hero), None), None)
        .into_iter()
        .map(Manage::Army);
    let formations =
        [Formation::Loose, Formation::Tight].map(|formation| Manage::Formation { hero, formation });
    let flags = [true, false].map(|on| Manage::Tactics { hero, on });
    let positions = (0..details.worn.len()).map(Position);
    let worn = positions.clone().map(ArtifactPlace::Worn);
    let carried = |end: usize| (0..end).map(ArtifactPlace::Backpack);
    let targets = worn.clone().chain(carried(details.backpack.len() + 1));
    let moves = worn
        .chain(carried(details.backpack.len()))
        .flat_map(|from| targets.clone().map(move |to| ArtifactMove { from, to }))
        .filter(|&moved| check_move(details, None, core, moved).is_ok())
        .map(|moved| Manage::Artifact {
            from: (hero, moved.from),
            to: (hero, moved.to),
        });
    let assemblies = positions
        .clone()
        .filter(|&at| check_assemble(details, core, at).is_ok())
        .map(|at| Manage::Assemble { hero, at });
    let disassemblies = positions
        .filter(|&at| check_disassemble(details, core, at).is_ok())
        .map(|at| Manage::Disassemble { hero, at });
    army.chain(formations)
        .chain(flags)
        .chain(moves)
        .chain(assemblies)
        .chain(disassemblies)
        .collect()
}

/// The counts a split of `actor`'s stack at `from` with `to` may leave in `to` (GAM-MAN-002,
/// ADV-ARM-003, ADV-ARM-007; E08 T51a): the adventure's
/// [`h3_adventure::AdventureState::split_range`]. `None` when the actor may not give the split
/// now (its turn, its open decision: `apply`'s checks), the two slots are not of one army
/// (outside a meeting and an army dialog), or no split is possible there.
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
    state.check_ready(player).ok()?;
    (from.army == to.army).then_some(())?;
    state
        .adventure()
        .split_range(rules.adventure(), player, (from, to), None)
}
