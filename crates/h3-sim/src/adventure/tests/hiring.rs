//! The env with an open hire dialog (E08 T95b, action encoding v7): the entries `HIRE_BASE + c`
//! buy the most of the dialog's creature in place `c` of its source's order, each legal while
//! that creature's `Buy` is listed (its most is 1 or more), tile 0 alone; `DECLINE` closes it; an
//! entry of no listed purchase decodes to a `Buy` of 0, which the game refuses; a recorded game of
//! a hire dialog's answers verifies (replay format v10).

use h3_content::CreatureTypeId;
use h3_game::{Answer, DecisionView, HireAnswer, hire_range};

use super::decisions::legal_entries;
use super::*;

const RED: PlayerId = PlayerId::new(0);

/// A pack of two dwellings without guards (every creature below the packs' guard level 5): `t:one`
/// sells archers (level 2) alone; `t:several` pikemen (level 1, free at the visit), archers and
/// griffins (level 3).
const T_CORE: &str = "[pack]\nid = \"t\"\nversion = \"1\"\ndepends = [\"sod\"]\n";
const T_ADVENTURE: &str = r#"
[[object_type]]
id = "one"
name = "One"
visit = "enter"
from_above = false
dwelling = { creatures = ["base:archer"], blocked = [[0, 0]], visitable = [[0, 0]] }

[[object_type]]
id = "several"
name = "Several"
visit = "enter"
from_above = false
dwelling = { creatures = ["base:pikeman", "base:archer", "base:griffin"], blocked = [[0, 0]], visitable = [[0, 0]] }
"#;

/// The original packs with [`T_ADVENTURE`]'s pack, at the faithful profile.
fn hiring_rules() -> GameRuleset {
    let profile = "base:faithful";
    let [base, ab, sod] = h3_pack_adventure::ORIGINAL_PACKS;
    let (core, part) =
        h3_pack_adventure::compile(&[base, ab, sod, (T_CORE, T_ADVENTURE)], profile).unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
            (T_CORE, ""),
        ],
        profile,
    )
    .unwrap();
    let [base, ab, sod] = h3_pack_towns::ORIGINAL_PACKS;
    let towns = h3_pack_towns::compile(&[base, ab, sod, (T_CORE, "")], profile).unwrap();
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .unwrap()
}

static HIRING: LazyLock<GameRuleset> = LazyLock::new(hiring_rules);

fn creature(key: &str) -> CreatureTypeId {
    HIRING.core().creature_by_key(key).unwrap()
}

/// The duel with [`beside`]'s dwellings of `key`, seed 1: the env and the commands red's hero in
/// slot 0 plays to enter its own and accept the hire question (the free creatures, then the hire
/// dialog), which the env has played.
fn hired(key: &str) -> (AdventureEnv, Vec<GameCommand>) {
    let setup = beside(&HIRING, h3_game::DEFAULT_DIFFICULTY, key);
    let at = charts(&setup)[0];
    let mut env = AdventureEnv::new(HIRING.clone(), setup, 1, 0).unwrap();
    let tile = tile_index(env.state(), at).unwrap();
    let mut commands = Vec::new();
    for action in [Action::move_to(0, tile), Action::accept()] {
        let (_, command) = env.step_decoded(RED, action, &mut Vec::new()).unwrap();
        commands.push(command);
    }
    (env, commands)
}

/// The creatures of red's open hire dialog, as its view shows them.
fn dialog(env: &AdventureEnv) -> Vec<CreatureTypeId> {
    match env.observation(RED).decision {
        Some(DecisionView::Hire(view)) => view.creatures.iter().map(|c| c.creature).collect(),
        other => panic!("no hire dialog: {other:?}"),
    }
}

/// The `Buy` of the most of `creature` the game lists for red (its `hire_range`'s end).
fn most(env: &AdventureEnv, creature: CreatureTypeId) -> GameCommand {
    let amount = *hire_range(env.state(), env.rules(), RED, creature).end();
    GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount }))
}

/// Each hire entry of red: legal exactly when its place's creature has a listed `Buy`, tile 0
/// alone then (none otherwise), decoding to that `Buy` of the most (to a `Buy` of 0 otherwise,
/// which the game refuses, the state untouched), the listed `Buy` encoding back to it.
fn check_hire_entries(env: &AdventureEnv) {
    let (state, rules) = (env.state(), env.rules());
    let listed = legal_commands(state, rules, Actor::Player(RED));
    let sold = dialog(env);
    let mut tiles = Vec::new();
    for place in 0..HIRE_SLOTS {
        let action = Action::hire(place);
        let entry = HIRE_BASE + place;
        let command = action::decode(state, rules, RED, action).unwrap();
        let buy = sold
            .get(place)
            .map(|&c| most(env, c))
            .filter(|b| listed.contains(b));
        let legal = legal_entries(env, RED).contains(&entry);
        assert_eq!(legal, buy.is_some(), "{place}");
        env.tile_mask(RED, entry, &mut tiles);
        let masked: Vec<usize> = (0..tiles.len()).filter(|&t| tiles[t]).collect();
        assert_eq!(masked, if legal { vec![0] } else { vec![] }, "{place}");
        assert_eq!(
            action::decode(state, rules, RED, Action::map(entry, 1)),
            None
        );
        if let Some(buy) = buy {
            assert_eq!(command, buy, "{place}");
            assert_eq!(action::encode(state, rules, RED, &command), Some(action));
            continue;
        }
        let creature = sold.get(place).copied().unwrap_or_default();
        let zero = GameCommand::Answer(Answer::Hire(HireAnswer::Buy {
            creature,
            amount: 0,
        }));
        assert_eq!(command, zero, "{place}");
        assert_eq!(action::encode(state, rules, RED, &zero), None);
        let mut refused = env.fork();
        assert!(matches!(
            refused.step(RED, action),
            Err(StepError::Rejected(_))
        ));
        assert_eq!(refused.state_hash(), env.state_hash());
    }
}

// covers: GAM-DEC-008
/// E08 T95b (encoding v7): a dialog of several creatures, the free pikemen gone (their most 0,
/// their entry masked), the archers' and griffins' entries legal, the fourth place past the
/// stock masked; a purchase keeps it open, `DECLINE` closes it. A dialog of one creature: its
/// purchase closes it. With no dialog every hire entry is masked and refused.
#[test]
fn a_hire_dialog_is_answered_through_the_env() {
    let (pikeman, archer, griffin) = (
        creature("base:pikeman"),
        creature("base:archer"),
        creature("base:griffin"),
    );
    let (mut env, _) = hired("t:several");
    assert_eq!(dialog(&env), [pikeman, archer, griffin]);
    assert_eq!(
        legal_entries(&env, RED),
        [DECLINE, HIRE_BASE + 1, HIRE_BASE + 2]
    );
    check_hire_entries(&env);
    let (state, rules) = (env.state(), env.rules());
    // `DECLINE` is the dialog's `Close`; a purchase of less than the most has no action.
    let close = GameCommand::Answer(Answer::Hire(HireAnswer::Close));
    assert_eq!(answer_entry(Answer::Hire(HireAnswer::Close)), Some(DECLINE));
    assert_eq!(
        action::decode(state, rules, RED, Action::decline()),
        Some(close.clone())
    );
    assert_eq!(
        action::encode(state, rules, RED, &close),
        Some(Action::decline())
    );
    let GameCommand::Answer(archers @ Answer::Hire(HireAnswer::Buy { amount, .. })) =
        most(&env, archer)
    else {
        unreachable!()
    };
    assert!(amount > 1, "{amount}");
    assert_eq!(answer_entry(archers), None);
    let fewer = GameCommand::Answer(Answer::Hire(HireAnswer::Buy {
        creature: archer,
        amount: 1,
    }));
    assert_eq!(action::encode(state, rules, RED, &fewer), None);
    // The archers' most: the dialog stays open, their entry masked now.
    env.step(RED, Action::hire(1)).unwrap();
    assert_eq!(dialog(&env), [pikeman, archer, griffin]);
    assert!(!legal_entries(&env, RED).contains(&(HIRE_BASE + 1)));
    check_hire_entries(&env);
    env.step(RED, Action::decline()).unwrap();
    assert!(!env.state().has_decision(RED));
    // No dialog: each hire entry masked, a `Buy` of 0 of the placeholder, refused.
    let mut tiles = Vec::new();
    for place in 0..HIRE_SLOTS {
        assert!(!legal_entries(&env, RED).contains(&(HIRE_BASE + place)));
        env.tile_mask(RED, HIRE_BASE + place, &mut tiles);
        assert!(!tiles.contains(&true));
        let (state, rules) = (env.state(), env.rules());
        let command = action::decode(state, rules, RED, Action::hire(place));
        let zero = Answer::Hire(HireAnswer::Buy {
            creature: CreatureTypeId::default(),
            amount: 0,
        });
        assert_eq!(command, Some(GameCommand::Answer(zero)));
        let refused = env.step(RED, Action::hire(place));
        assert!(
            matches!(refused, Err(StepError::Rejected(_))),
            "{refused:?}"
        );
    }
    // One creature: its entry alone with `DECLINE`; its purchase closes the dialog.
    let (mut env, _) = hired("t:one");
    assert_eq!(dialog(&env), [archer]);
    assert_eq!(legal_entries(&env, RED), [DECLINE, HIRE_BASE]);
    check_hire_entries(&env);
    env.step(RED, Action::hire(0)).unwrap();
    assert!(!env.state().has_decision(RED));
}

/// E08 T95b (replay format v10): a recorded game of a hire dialog's answers, a purchase and its
/// `Close`, verifies.
#[test]
fn a_replay_records_a_hire_dialogs_answers() {
    assert_eq!(replay::REPLAY_FORMAT_VERSION, 10);
    let (mut env, mut commands) = hired("t:several");
    for action in [Action::hire(2), Action::decline()] {
        let (_, command) = env.step_decoded(RED, action, &mut Vec::new()).unwrap();
        commands.push(command);
    }
    assert!(matches!(
        commands[2],
        GameCommand::Answer(Answer::Hire(HireAnswer::Buy { .. }))
    ));
    let setup = beside(&HIRING, h3_game::DEFAULT_DIFFICULTY, "t:several");
    let actors = commands.into_iter().map(|c| (Actor::Player(RED), c));
    let recorded =
        replay::Replay::record_commands(&HIRING, (&setup, duel_map()), 1, actors.collect(), 1)
            .unwrap();
    assert_eq!(recorded.final_hash, env.state_hash());
    recorded.verify(&HIRING, &duel_map(), &setup).unwrap();
}
