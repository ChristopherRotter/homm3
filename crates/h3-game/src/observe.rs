//! What a viewer may know of the whole game (ADR-0017 points 1 and 2, E08 T8b1;
//! `docs/06-security.md`, "Information hiding"): the game's public facts (the day, since E08 T85b the
//! month's kind, the players' colours, whose turn is running), the viewer's own treasury (E08 T13a; other players' never
//! show), since E08 T85b the timed events that fired at its turn start today, and open decision (T14b; since T30 an offer or a level-up, since T51b a meeting or an
//! army dialog, since T63a a choice or a University, since T95b a hire dialog), the heroes it sees with their types and its own heroes'
//! details (T18b, GAM-HER-001), each context's observation for the viewer (since T19a its
//! battle's, as its side sees it; since E09 T8c1 the towns it sees, TWN-VEW-001 to -003), and the
//! projection of the game's events. The seed, the streams and the state hash never show
//! (ADR-0017 point 5).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureObservation, AdventureReveal, BitGrid, Choice, HeroView, HireTarget, ObjectTypeId,
    Offer, Pos, UniversityVisit, Viewers,
};
use h3_content::{CreatureTypeId, HeroTypeId, ResourceId, SkillId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, PrimarySkill, SlotState};

use crate::command::GameEvent;
use crate::town::view::{TownView, town_views, views_of};
use crate::{GameRuleset, GameState, MonthKind};

/// Who an observation or an event's projection is for (ADR-0017 point 2). The spectator, who
/// sees public facts only, comes with the server lane as an additive variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewer {
    /// A player: what it may know.
    Player(PlayerId),
}

/// What one viewer may know of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameObservation {
    /// The player this observation is for.
    pub viewer: PlayerId,
    /// The day, from 1.
    pub day: u32,
    /// The month's kind (GAM-CAL-003, E08 T85b): public, as the original's month's message.
    pub month: MonthKind,
    /// The players, by id.
    pub players: Vec<PlayerView>,
    /// The players of the group whose turn is running.
    pub turn_group: Vec<PlayerId>,
    /// The viewer's treasury, by resource in the core's order (GAM-RES-001); empty for a viewer
    /// that is not a player of the game.
    pub treasury: Vec<u32>,
    /// The map's timed events that fired at the viewer's turn start today, by their messages in
    /// the map's order (GAM-CAL-004, E08 T85b: [`GameState::events_today`]); empty before its
    /// turn starts. Day 1's fire inside the build, so this is how a client learns of them. Other
    /// players' never show.
    pub events_today: Vec<u16>,
    /// The viewer's first open decision but a battle (GAM-DEC-001, E08 T14b; GAM-DEC-002, T30;
    /// GAM-DEC-003, GAM-DEC-004, T51b; GAM-DEC-005, GAM-DEC-006, T63a; GAM-DEC-008, T95b): an
    /// offer, its object named by type and anchor, with its price, a level-up with its offers, a
    /// meeting's two heroes, an army dialog's hero and group, a choice, a University with its
    /// slots, or a hire dialog with its creatures; other
    /// players' never show. While it is open the viewer's only legal commands are
    /// its answers, and in a meeting or an army dialog the management commands on what it holds.
    pub decision: Option<DecisionView>,
    /// The heroes the viewer sees on the adventure map (its own and the others its explored tiles
    /// show), by id: each hero's type (and so its class), and the details of its own only
    /// (GAM-HER-001, E08 T18b; survey `E08-battles.md` §9).
    pub heroes: Vec<HeroInfo>,
    /// The adventure map as the viewer sees it.
    pub adventure: AdventureObservation,
    /// The towns the viewer sees, in `TownId` order (TWN-VEW-001 to -003, E09 T8c1): each town a
    /// tile of which it has explored, its object among [`Self::adventure`]'s
    /// ([`h3_adventure::ObjectView::town`]), with its public facts, its garrison by the viewer's
    /// level and, for its owner's team, the popup's top level's facts.
    pub towns: Vec<TownView>,
    /// The battle of the viewer's first open decision, as the viewer's side sees it
    /// (`h3_battle::observe`, ADR-0013 point 9; GAM-BAT-001, E08 T19a); other players' battles
    /// never show.
    pub battle: Option<h3_battle::Observation>,
}

/// An open decision as its owner sees it (E08 T30): a battle is [`GameObservation::battle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionView {
    /// An offer (GAM-DEC-001), answered `Offer`.
    Offer(Offer),
    /// A level-up with two offers (GAM-DEC-002), answered `LevelUp`.
    LevelUp(LevelUpView),
    /// A meeting of two of the viewer's heroes (GAM-DEC-003, E08 T51b), closed with `Close`;
    /// meanwhile the viewer manages both, also between them.
    Meeting {
        /// The visitor, then the visited hero.
        heroes: [HeroId; 2],
    },
    /// An army dialog (GAM-DEC-004, E08 T51b), closed with `Close`, which loses the group's
    /// rest; meanwhile the viewer manages the hero and the group, also between them. Since E08
    /// T84a an own mine's Leave Guards dialog (ADV-MIN-002): its group the mine's guards, which
    /// the `Close` keeps there.
    ArmyDialog {
        /// The hero the group joined, or the visiting hero on the mine's entrance.
        hero: HeroId,
        /// The group's stacks, by slot.
        group: Vec<ArmyStack>,
        /// A Leave Guards dialog's mine, by type and anchor (E08 T84a); none for a join's group.
        mine: Option<(ObjectTypeId, Pos)>,
    },
    /// A visit's choice (GAM-DEC-005, E08 T63a): a treasure chest's gold and experience
    /// (answered `Chest`), the Arena's two skills and their gain (`Arena`), or a School's two
    /// skills, gain and price (`School`).
    Choice(Choice),
    /// A University (GAM-DEC-006, E08 T63a): its skills in the draw's order with their price,
    /// and each slot's state for the hero (`h3_heroes::university_slot`: an open one may be
    /// bought while the treasury can pay), answered `University`.
    University {
        /// The University.
        visit: UniversityVisit,
        /// Each skill's slot state for the hero, in the skills' order.
        slots: Vec<SlotState>,
    },
    /// A hire dialog (GAM-DEC-008, E08 T95b): its army, its source and the source's creatures
    /// with their counts and costs, answered `Hire`.
    Hire(HireView),
}

impl DecisionView {
    /// The offer, if the decision is one.
    pub const fn offer(&self) -> Option<&Offer> {
        match self {
            Self::Offer(offer) => Some(offer),
            _ => None,
        }
    }

    /// The level-up, if the decision is one.
    pub const fn level_up(&self) -> Option<&LevelUpView> {
        match self {
            Self::LevelUp(level_up) => Some(level_up),
            _ => None,
        }
    }
}

/// A hire dialog as its owner sees it (GAM-DEC-008, E08 T95b; increment 10's design note §13):
/// what the original's `H3RecruitMgr` shows but the most, which the owner's own treasury gives (a
/// client draws the slider). The source is named by type and anchor, never by index (ADR-0016
/// point 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HireView {
    /// The army the bought creatures join: the visiting hero's.
    pub target: HireTarget,
    /// The source's type: the dwelling's or the Refugee Camp's.
    pub object_type: ObjectTypeId,
    /// The source's anchor.
    pub anchor: Pos,
    /// The source's creatures in its order (a dwelling's kind's; a camp's one).
    pub creatures: Vec<HireCreature>,
}

/// A creature a hire dialog sells, as its owner sees it ([`HireView`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HireCreature {
    /// The creature.
    pub creature: CreatureTypeId,
    /// How many the source holds now.
    pub count: u32,
    /// The cost of one, by resource in the core's order (each resource it costs more than 0 of).
    pub cost: Vec<(ResourceId, u32)>,
}

/// A level-up with two offers as its owner sees it (GAM-DEC-002, E08 T30): what the original's
/// dialog shows. The hero's level seed never shows (ADR-0017 point 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelUpView {
    /// The hero.
    pub hero: HeroId,
    /// The new level.
    pub level: u8,
    /// The primary skill it raises by 1.
    pub primary: PrimarySkill,
    /// The offers, the dialog's left (`First`) and right (`Second`).
    pub offers: [SkillId; 2],
}

/// What a viewer knows of a hero beyond the adventure map's view of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroInfo {
    /// The hero.
    pub id: HeroId,
    /// Its hero type: identity and class, public as the original's portrait.
    pub hero_type: HeroTypeId,
    /// Its details (experience, primary and secondary skills, artifacts, spells, spell points),
    /// for its owner only.
    pub details: Option<Arc<HeroDetails>>,
}

/// A player as every viewer sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerView {
    /// Its colour (0 red to 7 pink).
    pub colour: u8,
    /// Whether its turn is running: its group's turn has started and it has not ended its turn.
    pub acting: bool,
}

/// What `viewer` may know of the game: the public facts and the adventure map as it sees it
/// (ADV-FOG-004 to ADV-FOG-006). A viewer that is not a player of the game sees the public facts
/// and no tile.
pub fn observe(state: &GameState, rules: &GameRuleset, viewer: Viewer) -> GameObservation {
    let Viewer::Player(player) = viewer;
    let adventure = state
        .adventure()
        .observe(rules.core(), rules.adventure(), player);
    let towns = town_views(state, rules, &adventure);
    GameObservation {
        viewer: player,
        day: state.day(),
        month: state.month(),
        players: (0..=u8::MAX)
            .take(state.players())
            .map(PlayerId::new)
            .map(|p| PlayerView {
                colour: state.colour(p).unwrap_or(u8::MAX),
                acting: state.is_acting(p),
            })
            .collect(),
        turn_group: state.turn_group().to_vec(),
        treasury: state
            .treasury(player)
            .map_or_else(Vec::new, <[u32]>::to_vec),
        events_today: state.events_today(player),
        decision: state
            .decision(player)
            .cloned()
            .map(DecisionView::Offer)
            .or_else(|| state.level_up(player).map(DecisionView::LevelUp))
            .or_else(|| {
                let heroes = state.meeting(player)?;
                Some(DecisionView::Meeting { heroes })
            })
            .or_else(|| {
                let (hero, group) = state.army_dialog(player)?;
                Some(DecisionView::ArmyDialog {
                    hero,
                    group: group.to_vec(),
                    mine: state.leave_guards(player),
                })
            })
            .or_else(|| state.choice(player).cloned().map(DecisionView::Choice))
            .or_else(|| {
                let visit = state.university(player)?;
                Some(DecisionView::University {
                    slots: visit
                        .skills
                        .iter()
                        .map(|&skill| state.slot_state(rules, visit.hero, skill))
                        .collect(),
                    visit: visit.clone(),
                })
            })
            .or_else(|| {
                let dialog = state.hire_dialog(player)?;
                Some(DecisionView::Hire(state.hire_view(rules, dialog)))
            }),
        heroes: hero_infos(state, &adventure.heroes),
        adventure,
        towns,
        battle: state.battle(player).and_then(|open| {
            open.side_of(player)
                .map(|side| h3_battle::observe(open.state(), rules.battle(), side))
        }),
    }
}

/// The heroes of `views` with their types and, for the viewer's own, their details
/// ([`GameObservation::heroes`], GAM-HER-001), in their order.
fn hero_infos(state: &GameState, views: &[HeroView]) -> Vec<HeroInfo> {
    views
        .iter()
        .flat_map(|view| {
            // Every hero on the map has its details in the store, at its id.
            let details = state.heroes().get(usize::from(view.id.raw()));
            details.map(|details| HeroInfo {
                id: view.id,
                hero_type: details.hero_type,
                details: view.own.as_ref().map(|_| Arc::clone(details)),
            })
        })
        .collect()
}

/// What a viewer sees now that it did not see through an earlier explored grid of its own
/// ([`reveal`]; SRV-Q26 (a), the server lane's E16 M5 T5c): [`GameObservation`]'s parts that the
/// grids' difference decides, each as [`observe`] shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameReveal {
    /// The adventure map's: the tiles whose sight changed, the heroes, objects and empty boats
    /// the viewer sees now and did not, the towns it newly explored.
    pub adventure: AdventureReveal,
    /// The heroes of [`AdventureReveal::heroes`] with their types, in their order, as
    /// [`GameObservation::heroes`] holds them (none is the viewer's own, so none has details).
    pub heroes: Vec<HeroInfo>,
    /// The viewer's views of [`AdventureReveal::towns`], in their order, as
    /// [`GameObservation::towns`] holds them (TWN-VEW-001 to -004).
    pub towns: Vec<TownView>,
}

/// What `viewer` sees now that it did not see through `since`, an earlier explored grid of its
/// own (SRV-Q26 (a); `h3_adventure::AdventureState::reveal`): the tiles whose sight changed with
/// their views, the heroes (with their types), typed objects and empty boats it sees now and did
/// not, and its views of the towns a tile of which it explored since, each as [`observe`] shows
/// it. A server keeps each viewer's grid (`state.adventure().explored(player)`, cheap to keep:
/// its copies share their blocks) and asks when the viewer's explored count grew; the viewer's
/// view folded with its events and this answer is its observation again. What it saw through
/// `since` stays the events' to keep current.
// rule: ADV-FOG-004, ADV-FOG-005, ADV-FOG-006, ADV-FOG-008, TWN-VEW-001
pub fn reveal(
    state: &GameState,
    rules: &GameRuleset,
    viewer: Viewer,
    since: &BitGrid,
) -> GameReveal {
    let Viewer::Player(player) = viewer;
    let adventure = state
        .adventure()
        .reveal(rules.core(), rules.adventure(), player, since);
    GameReveal {
        heroes: hero_infos(state, &adventure.heroes),
        towns: views_of(state, rules, player, &adventure.towns),
        adventure,
    }
}

/// The event as `viewer` may see it, or `None` if it is hidden from it: the game's own events are
/// public but a treasury's change, a timed event's firing (E08 T85b), an offer's, a hero's change and its level-ups, which show to
/// their owner only (E08 T13b, T14b, T19a2a, T30; a management command's since T51a, a meeting's
/// and an army dialog's since T51b, a choice's and a University's since T63a, a hire dialog's
/// since T95b; a town's build since E09 T18b1; a town's view, told to each of its viewers, since
/// E09 T18b2a), a battle's, which show to the owners that see them (E08 T19a); an adventure or
/// a battle event shows to
/// the players who see it, naming the viewer alone among them
/// ([`h3_adventure::AdventureEvent::project`]).
pub fn project(event: &GameEvent, viewer: Viewer) -> Option<GameEvent> {
    let Viewer::Player(player) = viewer;
    let alone = Viewers::NONE.with(player);
    match *event {
        GameEvent::Adventure(event) => event.project(player).map(GameEvent::Adventure),
        GameEvent::BattleStarted {
            heroes,
            tile,
            seen_by,
        } => seen_by
            .contains(player)
            .then_some(GameEvent::BattleStarted {
                heroes,
                tile,
                seen_by: alone,
            }),
        GameEvent::Battle { event, seen_by } => {
            seen_by.contains(player).then_some(GameEvent::Battle {
                event,
                seen_by: alone,
            })
        }
        GameEvent::BattleEnded {
            heroes,
            result,
            seen_by,
        } => seen_by.contains(player).then_some(GameEvent::BattleEnded {
            heroes,
            result,
            seen_by: alone,
        }),
        GameEvent::ResourceGained { player: owner, .. }
        | GameEvent::ResourceLost { player: owner, .. }
        | GameEvent::ResourcePaid { player: owner, .. }
        | GameEvent::TownBuilt { player: owner, .. }
        | GameEvent::TownEventFired { player: owner, .. }
        | GameEvent::TownChanged { player: owner, .. }
        | GameEvent::TownStackSeen { player: owner, .. }
        | GameEvent::TownIncomeSeen { player: owner, .. }
        | GameEvent::HeroChanged { player: owner, .. }
        | GameEvent::CreaturesRaised { player: owner, .. }
        | GameEvent::OfferOpened { player: owner, .. }
        | GameEvent::OfferRefused { player: owner, .. }
        | GameEvent::OfferAnswered { player: owner, .. }
        | GameEvent::LevelUpOpened { player: owner, .. }
        | GameEvent::LevelGained { player: owner, .. }
        | GameEvent::StackChanged { player: owner, .. }
        | GameEvent::FormationSet { player: owner, .. }
        | GameEvent::TacticsSet { player: owner, .. }
        | GameEvent::MeetingOpened { player: owner, .. }
        | GameEvent::ArmyDialogOpened { player: owner, .. }
        | GameEvent::LeaveGuardsOpened { player: owner, .. }
        | GameEvent::ChoiceOpened { player: owner, .. }
        | GameEvent::UniversityOpened { player: owner, .. }
        | GameEvent::HireOpened { player: owner, .. }
        | GameEvent::DecisionClosed { player: owner }
        | GameEvent::GroupLost { player: owner, .. }
        | GameEvent::TimedEventFired { player: owner, .. } => (owner == player).then_some(*event),
        GameEvent::TurnEnded { .. }
        | GameEvent::DayStarted { .. }
        | GameEvent::WeekStarted { .. }
        | GameEvent::MonthStarted { .. }
        | GameEvent::TurnStarted { .. } => Some(*event),
    }
}
