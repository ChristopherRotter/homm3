//! E05 T3d: the batch through the C ABI against the Rust batch, the shared rulesets, the shipped
//! packs, and every function's errors; E06 T2c: each entry's engine identity.

use std::sync::Arc;

use h3_battle::CommandError;
use h3_kernel::{Rng, StreamId};
use h3_sim::action::{CREATURE_CAST, HERO_CAST, RETREAT};

use super::*;
use crate::H3_RESULT_NONE;

const DUEL: &str = h3_pack_battle::SYNTHETIC_DUEL;
const SKIRMISH: &str = h3_pack_battle::SYNTHETIC_SKIRMISH;
const FULL: &str = h3_pack_battle::SYNTHETIC_FULL;
const SIEGE: &str = h3_pack_battle::SYNTHETIC_SIEGE;
const TOWN: &str = include_str!("../scenario/town.toml");

const EXTERNAL: H3Seats = H3Seats {
    attacker: H3_SEAT_EXTERNAL,
    defender: H3_SEAT_EXTERNAL,
};

/// The arrays a binding holds for a batch of `n` envs.
struct Arrays {
    obs: Vec<f32>,
    mask: Vec<u8>,
    to_move: Vec<i8>,
    reward: Vec<f32>,
    done: Vec<u8>,
    truncated: Vec<u8>,
    result: Vec<i8>,
    final_obs: Vec<f32>,
    episode: Vec<u32>,
    scenario: Vec<u32>,
    seed: Vec<u64>,
    status: Vec<i32>,
    encounter: Vec<i8>,
}

impl Arrays {
    fn new(n: usize) -> Self {
        Self {
            obs: vec![0.0; n * OBSERVATION_LEN],
            mask: vec![0; n * ACTION_COUNT],
            to_move: vec![0; n],
            reward: vec![0.0; 2 * n],
            done: vec![0; n],
            truncated: vec![0; n],
            result: vec![0; n],
            final_obs: vec![0.0; n * 2 * OBSERVATION_LEN],
            episode: vec![0; n],
            scenario: vec![0; n],
            seed: vec![0; n],
            status: vec![0; n],
            encounter: vec![0; n],
        }
    }

    fn buffers(&mut self) -> H3BatchBuffers {
        H3BatchBuffers {
            obs: self.obs.as_mut_ptr(),
            entry_mask: self.mask.as_mut_ptr(),
            to_move: self.to_move.as_mut_ptr(),
            reward: self.reward.as_mut_ptr(),
            done: self.done.as_mut_ptr(),
            truncated: self.truncated.as_mut_ptr(),
            result: self.result.as_mut_ptr(),
            final_obs: self.final_obs.as_mut_ptr(),
            episode: self.episode.as_mut_ptr(),
            scenario: self.scenario.as_mut_ptr(),
            episode_seed: self.seed.as_mut_ptr(),
            status: self.status.as_mut_ptr(),
            encounter: self.encounter.as_mut_ptr(),
        }
    }
}

fn config(envs: u32, threads: u32, seed: u64, max_steps: u32) -> H3BatchConfig {
    H3BatchConfig {
        envs,
        threads,
        seed,
        max_steps,
        reward: H3RewardConfig {
            currency: H3_CURRENCY_GOLD,
            mana_price: 7,
        },
    }
}

/// `h3_batch_create`, or its message.
fn create(config: &H3BatchConfig) -> Result<*mut H3Batch, String> {
    let mut error = [0 as c_char; 128];
    let b = unsafe { h3_batch_create(config, error.as_mut_ptr(), error.len()) };
    if b.is_null() {
        let bytes: Vec<u8> = error
            .iter()
            .take_while(|c| **c != 0)
            .map(|c| *c as u8)
            .collect();
        Err(String::from_utf8(bytes).unwrap())
    } else {
        Ok(b)
    }
}

/// `h3_batch_add_scenario` with its code and message.
fn add(b: *mut H3Batch, text: &str, weight: u32, seats: H3Seats) -> (i32, String) {
    let mut error = [0 as c_char; 256];
    let code = unsafe {
        h3_batch_add_scenario(
            b,
            text.as_ptr().cast(),
            text.len(),
            weight,
            seats,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    let bytes: Vec<u8> = error
        .iter()
        .take_while(|c| **c != 0)
        .map(|c| *c as u8)
        .collect();
    (code, String::from_utf8(bytes).unwrap())
}

/// A uniformly random index among the set bytes of `mask` (0 if none is set).
fn pick(mask: &[u8], rng: &mut Rng) -> usize {
    let legal: Vec<usize> = (0..mask.len()).filter(|&i| mask[i] == 1).collect();
    if legal.is_empty() {
        return 0;
    }
    legal[rng.below(legal.len() as u32) as usize]
}

/// Random legal actions from the ABI's masks, a cast's heads one call after another.
fn actions(b: *mut H3Batch, arrays: &Arrays, n: usize, rng: &mut Rng) -> Vec<H3Action> {
    let entries: Vec<u32> = arrays
        .mask
        .chunks(ACTION_COUNT)
        .map(|m| pick(m, rng) as u32)
        .collect();
    let mut spells = vec![0u8; n * SPELL_COUNT];
    let mut targets = vec![0u8; n * TARGET_COUNT];
    let (spell, target, second);
    unsafe {
        let e = entries.as_ptr();
        assert_eq!(
            h3_batch_spell_mask(b, e, spells.as_mut_ptr(), spells.len()),
            H3_OK
        );
        spell = spells
            .chunks(SPELL_COUNT)
            .map(|m| pick(m, rng) as u16)
            .collect::<Vec<_>>();
        let s = spell.as_ptr();
        assert_eq!(
            h3_batch_target_mask(b, e, s, targets.as_mut_ptr(), targets.len()),
            H3_OK
        );
        target = targets
            .chunks(TARGET_COUNT)
            .map(|m| pick(m, rng) as u16)
            .collect::<Vec<_>>();
        let t = target.as_ptr();
        assert_eq!(
            h3_batch_second_mask(b, e, s, t, targets.as_mut_ptr(), targets.len()),
            H3_OK
        );
        second = targets
            .chunks(TARGET_COUNT)
            .map(|m| pick(m, rng) as u16)
            .collect::<Vec<_>>();
    }
    (0..n)
        .map(|i| H3Action {
            entry: entries[i],
            spell: spell[i],
            target: target[i],
            second: second[i],
        })
        .collect()
}

/// The same scenario for the Rust batch.
fn rust_scenario(text: &str, weight: u32, seats: [Seat; 2]) -> h3_sim::Scenario {
    let random = h3_pack_battle::parse_scenario(text)
        .unwrap()
        .random_obstacles;
    let (ruleset, setup) = h3_pack_battle::synthetic(text).unwrap();
    h3_sim::Scenario::new(Arc::new(ruleset), &setup, random, weight, seats).unwrap()
}

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.to_bits()).collect()
}

/// The Rust batch's outputs.
struct RustArrays {
    obs: Vec<f32>,
    mask: Vec<u8>,
    final_obs: Vec<f32>,
    reports: Vec<EnvReport>,
}

impl RustArrays {
    fn new(n: usize) -> Self {
        Self {
            obs: vec![0.0; n * OBSERVATION_LEN],
            mask: vec![0; n * ACTION_COUNT],
            final_obs: vec![0.0; n * 2 * OBSERVATION_LEN],
            reports: vec![EnvReport::default(); n],
        }
    }

    fn outputs(&mut self) -> Outputs<'_> {
        Outputs {
            obs: Some(&mut self.obs),
            entry_mask: Some(&mut self.mask),
            final_obs: Some(&mut self.final_obs),
            reports: Some(&mut self.reports),
        }
    }
}

/// The ABI's arrays hold what the Rust batch's outputs hold.
fn same(arrays: &Arrays, rust: &RustArrays) {
    let (final_obs, reports) = (&rust.final_obs, &rust.reports);
    assert_eq!(bits(&arrays.obs), bits(&rust.obs));
    assert_eq!(arrays.mask, rust.mask);
    let rows = 2 * OBSERVATION_LEN;
    for (i, r) in reports.iter().enumerate() {
        assert_eq!(arrays.to_move[i] as i32, side_code(r.to_move), "env {i}");
        assert_eq!(bits(&arrays.reward[2 * i..2 * i + 2]), bits(&r.reward));
        assert_eq!(arrays.done[i], u8::from(r.done));
        assert_eq!(arrays.truncated[i], u8::from(r.truncated));
        assert_eq!(arrays.result[i] as i32, result_code(r.result));
        assert_eq!(arrays.episode[i], r.episode);
        assert_eq!(arrays.scenario[i], r.scenario);
        assert_eq!(arrays.seed[i], r.episode_seed);
        assert_eq!(arrays.status[i], status_code(r.error));
        if r.truncated {
            let range = i * rows..(i + 1) * rows;
            assert_eq!(
                bits(&arrays.final_obs[range.clone()]),
                bits(&final_obs[range])
            );
        }
    }
}

#[test]
fn the_abi_batch_is_the_rust_batch() {
    let n = 5;
    let c = config(n as u32, 2, 21, 150);
    let b = create(&c).unwrap();
    let random = H3Seats {
        attacker: H3_SEAT_EXTERNAL,
        defender: H3_SEAT_RANDOM,
    };
    let flipped = H3Seats {
        attacker: H3_SEAT_RANDOM,
        defender: H3_SEAT_EXTERNAL,
    };
    assert_eq!(add(b, DUEL, 2, EXTERNAL).0, H3_OK);
    assert_eq!(add(b, SKIRMISH, 1, random).0, H3_OK);
    assert_eq!(add(b, SIEGE, 1, flipped).0, H3_OK);
    let rust_seats = [
        [Seat::External, Seat::External],
        [Seat::External, Seat::Internal(Policy::Random)],
        [Seat::Internal(Policy::Random), Seat::External],
    ];
    let mut rust = BattleBatch::new(BatchConfig {
        envs: c.envs,
        threads: 0,
        seed: c.seed,
        max_steps: c.max_steps,
        reward: RewardConfig {
            currency: Currency::Gold,
            mana_price: 7,
        },
    })
    .unwrap();
    for ((text, weight), seats) in [(DUEL, 2), (SKIRMISH, 1), (SIEGE, 1)]
        .iter()
        .zip(rust_seats)
    {
        rust.add_scenario(rust_scenario(text, *weight, seats))
            .unwrap();
    }
    let mut arrays = Arrays::new(n);
    let mut out = RustArrays::new(n);
    let buffers = arrays.buffers();
    assert_eq!(unsafe { h3_batch_reset(b, &buffers) }, H3_OK);
    rust.reset(&mut out.outputs()).unwrap();
    same(&arrays, &out);
    let mut rng = Rng::for_stream(4, StreamId::AGENT);
    let (mut ended, mut rewarded, mut truncated) = (0, 0, 0);
    for call in 0..300 {
        let chosen = actions(b, &arrays, n, &mut rng);
        let buffers = arrays.buffers();
        assert_eq!(
            unsafe { h3_batch_step(b, chosen.as_ptr(), &buffers) },
            H3_OK,
            "{call}"
        );
        let chosen: Vec<Action> = chosen.into_iter().map(Action::from).collect();
        rust.step(&chosen, &mut out.outputs()).unwrap();
        same(&arrays, &out);
        for i in 0..n {
            assert_eq!(arrays.reward[2 * i], -arrays.reward[2 * i + 1]);
            ended += u32::from(arrays.done[i]);
            rewarded += u32::from(arrays.reward[2 * i] != 0.0);
            truncated += u32::from(arrays.truncated[i]);
            if arrays.done[i] == 0 {
                assert_eq!(arrays.result[i] as i32, H3_RESULT_NONE);
            }
        }
    }
    assert!(
        ended > 3 && rewarded > 0 && truncated > 0,
        "{ended} {rewarded} {truncated}"
    );
    assert_eq!(
        h3_batch_episode_seed(c.seed, 3, 2),
        episode_seed(c.seed, 3, 2)
    );
    unsafe { h3_batch_destroy(b) };
}

#[test]
fn scenarios_on_the_same_packs_share_a_ruleset() {
    let b = create(&config(2, 0, 1, 0)).unwrap();
    assert_eq!(add(b, DUEL, 1, EXTERNAL).0, H3_OK);
    assert_eq!(add(b, FULL, 1, EXTERNAL).0, H3_OK);
    assert_eq!(add(b, TOWN, 1, EXTERNAL).0, H3_OK);
    let h = unsafe { &*b };
    let s = h.batch.scenarios();
    assert!(Arc::ptr_eq(s[0].ruleset(), s[1].ruleset()));
    assert!(!Arc::ptr_eq(s[0].ruleset(), s[2].ruleset()));
    assert_eq!(h.rulesets.len(), 2);
    // A siege on the shipped packs plays through the batch.
    let mut arrays = Arrays::new(2);
    let mut rng = Rng::from_seed(3);
    let buffers = arrays.buffers();
    assert_eq!(unsafe { h3_batch_reset(b, &buffers) }, H3_OK);
    for _ in 0..40 {
        let chosen = actions(b, &arrays, 2, &mut rng);
        let buffers = arrays.buffers();
        assert_eq!(
            unsafe { h3_batch_step(b, chosen.as_ptr(), &buffers) },
            H3_OK
        );
    }
    unsafe { h3_batch_destroy(b) };
}

#[test]
fn errors_are_codes_not_panics() {
    let null: *mut H3Batch = std::ptr::null_mut();
    let mut error = [0 as c_char; 64];
    // Creation.
    assert!(unsafe { h3_batch_create(std::ptr::null(), error.as_mut_ptr(), 64) }.is_null());
    assert!(unsafe { h3_batch_create(std::ptr::null(), null.cast(), 0) }.is_null());
    assert_eq!(
        create(&config(0, 0, 1, 0)).unwrap_err(),
        "a batch needs at least one env"
    );
    // v31: an absurd N is refused, not an abort on its allocation.
    assert_eq!(
        create(&config(u32::MAX, 0, 1, 0)).unwrap_err(),
        format!(
            "{} envs: a batch holds at most {H3_BATCH_MAX_ENVS}",
            u32::MAX
        )
    );
    let mut odd = config(1, 0, 1, 0);
    odd.reward.currency = 9;
    assert_eq!(create(&odd).unwrap_err(), "unknown currency");
    unsafe { h3_batch_destroy(null) };

    // Scenarios.
    let b = create(&config(2, 1, 1, 0)).unwrap();
    assert_eq!(add(null, DUEL, 1, EXTERNAL).0, H3_ERR_NULL);
    let code = unsafe {
        h3_batch_add_scenario(b, std::ptr::null(), 0, 1, EXTERNAL, error.as_mut_ptr(), 64)
    };
    assert_eq!(code, H3_ERR_NULL);
    let bad = [0xFFu8, 0xFE];
    let code =
        unsafe { h3_batch_add_scenario(b, bad.as_ptr().cast(), 2, 1, EXTERNAL, null.cast(), 0) };
    assert_eq!(code, H3_ERR_ARGUMENT);
    let unknown = H3Seats {
        attacker: 7,
        defender: H3_SEAT_EXTERNAL,
    };
    assert_eq!(
        add(b, DUEL, 1, unknown),
        (H3_ERR_ARGUMENT, "unknown seat".to_owned())
    );
    let internal = H3Seats {
        attacker: H3_SEAT_RANDOM,
        defender: H3_SEAT_RANDOM,
    };
    assert_eq!(add(b, DUEL, 1, internal).0, H3_ERR_ARGUMENT);
    assert_eq!(add(b, DUEL, 0, EXTERNAL).0, H3_ERR_ARGUMENT);
    let (code, message) = add(b, "[scenario]\nid = 1\n", 1, EXTERNAL);
    assert!(
        code == H3_ERR_ARGUMENT && message.starts_with("scenario:"),
        "{message}"
    );
    let elsewhere = DUEL.replace("[\"synthetic\"]", "[\"mod\"]");
    let (code, message) = add(b, &elsewhere, 1, EXTERNAL);
    assert!(
        code == H3_ERR_ARGUMENT && message.starts_with("unknown pack"),
        "{message}"
    );

    // Calls before a scenario or a reset.
    let mut arrays = Arrays::new(2);
    let buffers = arrays.buffers();
    let zeros = [0u32; 2];
    let heads = [0u16; 2];
    let mut out = vec![0u8; 2 * SPELL_COUNT.max(TARGET_COUNT)];
    let action = [H3Action::default(); 2];
    unsafe {
        assert_eq!(h3_batch_reset(null, &buffers), H3_ERR_NULL);
        assert_eq!(h3_batch_reset(b, &buffers), H3_ERR_NOT_READY);
        assert_eq!(
            h3_batch_step(b, action.as_ptr(), &buffers),
            H3_ERR_NOT_READY
        );
        assert_eq!(h3_batch_step(null, action.as_ptr(), &buffers), H3_ERR_NULL);
        assert_eq!(h3_batch_step(b, std::ptr::null(), &buffers), H3_ERR_NULL);
        let (z, h, o, len) = (zeros.as_ptr(), heads.as_ptr(), out.as_mut_ptr(), out.len());
        assert_eq!(h3_batch_spell_mask(b, z, o, len), H3_ERR_NOT_READY);
        assert_eq!(h3_batch_target_mask(b, z, h, o, len), H3_ERR_NOT_READY);
        assert_eq!(h3_batch_second_mask(b, z, h, h, o, len), H3_ERR_NOT_READY);
        assert_eq!(h3_batch_spell_mask(null, z, o, len), H3_ERR_NULL);
        assert_eq!(
            h3_batch_spell_mask(b, std::ptr::null(), o, len),
            H3_ERR_NULL
        );
        assert_eq!(h3_batch_spell_mask(b, z, null.cast(), len), H3_ERR_NULL);
        assert_eq!(
            h3_batch_target_mask(b, z, std::ptr::null(), o, len),
            H3_ERR_NULL
        );
        assert_eq!(
            h3_batch_second_mask(b, z, h, std::ptr::null(), o, len),
            H3_ERR_NULL
        );
        assert_eq!(h3_batch_spell_mask(b, z, o, 1), H3_ERR_ARGUMENT);
        assert_eq!(h3_batch_target_mask(b, z, h, o, 1), H3_ERR_ARGUMENT);
        assert_eq!(h3_batch_second_mask(b, z, h, h, o, 1), H3_ERR_ARGUMENT);
    }

    // Weights that overflow; then a refused action leaves its env unchanged while the other steps.
    assert_eq!(add(b, SKIRMISH, u32::MAX, EXTERNAL).0, H3_OK);
    let (code, message) = add(b, SKIRMISH, 1, EXTERNAL);
    assert!(
        code == H3_ERR_ARGUMENT && message.contains("u32::MAX"),
        "{message}"
    );
    unsafe {
        assert_eq!(h3_batch_reset(b, std::ptr::null()), H3_OK);
        assert_eq!(h3_batch_reset(b, &buffers), H3_OK);
    }
    let mut rng = Rng::from_seed(2);
    for (refused, entry) in [(0, u32::MAX), (1, RETREAT as u32)] {
        let mut chosen = actions(b, &arrays, 2, &mut rng);
        chosen[refused] = H3Action {
            entry,
            ..H3Action::default()
        };
        let row = refused * OBSERVATION_LEN..(refused + 1) * OBSERVATION_LEN;
        let obs = arrays.obs[row.clone()].to_vec();
        let mask = arrays.mask[refused * ACTION_COUNT..(refused + 1) * ACTION_COUNT].to_vec();
        let kept = (
            arrays.to_move[refused],
            arrays.episode[refused],
            arrays.seed[refused],
        );
        let buffers = arrays.buffers();
        assert_eq!(
            unsafe { h3_batch_step(b, chosen.as_ptr(), &buffers) },
            H3_ERR_REJECTED
        );
        let mut status = [H3_OK; 2];
        status[refused] = H3_ERR_INVALID_ACTION;
        assert_eq!(arrays.status, status);
        assert_eq!(arrays.obs[row], obs[..]);
        assert_eq!(
            arrays.mask[refused * ACTION_COUNT..(refused + 1) * ACTION_COUNT],
            mask[..]
        );
        let now = (
            arrays.to_move[refused],
            arrays.episode[refused],
            arrays.seed[refused],
        );
        assert_eq!(now, kept);
    }
    let chosen = actions(b, &arrays, 2, &mut rng);
    assert_eq!(
        unsafe { h3_batch_step(b, chosen.as_ptr(), std::ptr::null()) },
        H3_OK
    );
    unsafe { h3_batch_destroy(b) };

    // The codes of the rest.
    assert_eq!(
        status_code(Some(StepError::EpisodeOver)),
        H3_ERR_EPISODE_OVER
    );
    assert_eq!(
        status_code(Some(StepError::Rejected(CommandError::NotYourTurn))),
        H3_ERR_REJECTED
    );
    let short = BatchError::Buffer {
        name: "obs",
        len: 0,
        needed: 1,
    };
    assert_eq!(error_code(&short), H3_ERR_ARGUMENT);
    assert_eq!(error_code(&BatchError::NotReset), H3_ERR_NOT_READY);
    assert_eq!(H3_ERR_PANIC, -99);
    assert!([HERO_CAST, CREATURE_CAST].iter().all(|&c| c < ACTION_COUNT));
}

/// `h3_batch_add_encounters` with its code and message, each encounter's own seats.
fn add_encounters(b: *mut H3Batch, weight: u32, kinds: Option<[u32; 6]>) -> (i32, String) {
    add_encounters_seated(b, weight, kinds, None)
}

/// `h3_batch_add_encounters` with its code and message.
fn add_encounters_seated(
    b: *mut H3Batch,
    weight: u32,
    kinds: Option<[u32; 6]>,
    seats: Option<H3Seats>,
) -> (i32, String) {
    let mut error = [0 as c_char; 256];
    let kinds_ptr = kinds.as_ref().map_or(std::ptr::null(), |k| k.as_ptr());
    let seats_ptr = seats.as_ref().map_or(std::ptr::null(), std::ptr::from_ref);
    let code = unsafe {
        h3_batch_add_encounters(
            b,
            weight,
            kinds_ptr,
            seats_ptr,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    let bytes: Vec<u8> = error
        .iter()
        .take_while(|c| **c != 0)
        .map(|c| *c as u8)
        .collect();
    (code, String::from_utf8(bytes).unwrap())
}

/// E05 T5b2: the encounter distribution through the C ABI: its kinds in the reports, its
/// ruleset shared with a scenario on the original packs, its weights and errors.
/// `h3_batch_scenario_info` of entry `index`, with its code.
fn info(b: *const H3Batch, index: u32) -> (i32, H3ScenarioInfo) {
    let mut out = H3ScenarioInfo::default();
    let code = unsafe { h3_batch_scenario_info(b, index, &mut out) };
    (code, out)
}

#[test]
fn each_entry_names_its_rulesets() {
    // E06 T2c (v31): a training run's record names the content its batch plays.
    assert_eq!(h3_encounters_version(), h3_sim::ENCOUNTERS_VERSION);
    let null: *const H3Batch = std::ptr::null();
    assert_eq!(unsafe { h3_batch_scenario_count(null) }, 0);
    assert_eq!(info(null, 0).0, H3_ERR_NULL);
    let b = create(&config(2, 0, 1, 0)).unwrap();
    let nowhere = std::ptr::null_mut();
    assert_eq!(
        unsafe { h3_batch_scenario_info(b, 0, nowhere) },
        H3_ERR_NULL
    );
    assert_eq!(info(b, 0).0, H3_ERR_ARGUMENT);
    assert_eq!(add(b, DUEL, 2, EXTERNAL).0, H3_OK);
    assert_eq!(add(b, TOWN, 1, EXTERNAL).0, H3_OK);
    assert_eq!(add_encounters(b, 3, None).0, H3_OK);
    assert_eq!(unsafe { h3_batch_scenario_count(b) }, 3);
    let infos: Vec<H3ScenarioInfo> = (0..3)
        .map(|i| {
            let (code, out) = info(b, i);
            assert_eq!(code, H3_OK);
            out
        })
        .collect();
    assert_eq!(info(b, 3).0, H3_ERR_ARGUMENT);
    let scenarios = unsafe { &*b }.batch.scenarios();
    for (s, i) in scenarios.iter().zip(&infos) {
        assert_eq!(i.ruleset_hash, s.ruleset().core_hash());
        assert_eq!(i.battle_ruleset_hash, s.ruleset().content_hash());
    }
    let (duel, town, encounters) = (infos[0], infos[1], infos[2]);
    assert_eq!(
        (duel.weight, duel.encounters, duel.adventure_ruleset_hash),
        (2, 0, 0)
    );
    // The town and the encounters run on the original packs' one ruleset, the encounters with
    // their adventure part too.
    assert_ne!(duel.battle_ruleset_hash, town.battle_ruleset_hash);
    assert_eq!(town.battle_ruleset_hash, encounters.battle_ruleset_hash);
    assert_eq!((town.encounters, town.adventure_ruleset_hash), (0, 0));
    assert_eq!((encounters.weight, encounters.encounters), (3, 1));
    let adventure = scenarios[2].sampler().unwrap().adventure().content_hash();
    assert_eq!(encounters.adventure_ruleset_hash, adventure);
    assert_ne!(adventure, 0);
    unsafe { h3_batch_destroy(b) };
}

#[test]
fn encounters_are_an_episode_source() {
    let n = 12;
    let b = create(&config(n, 2, 4, 400)).unwrap();
    assert_eq!(add_encounters(std::ptr::null_mut(), 1, None).0, H3_ERR_NULL);
    let (code, message) = add_encounters(b, 0, None);
    assert_eq!(code, H3_ERR_ARGUMENT);
    assert!(message.contains("weight"), "{message}");
    let (code, message) = add_encounters(b, 1, Some([0; 6]));
    assert_eq!(code, H3_ERR_ARGUMENT);
    assert!(message.starts_with("encounters: "), "{message}");
    // A scenario on the original packs first: the encounters share its ruleset.
    assert_eq!(add(b, TOWN, 1, EXTERNAL).0, H3_OK);
    assert_eq!(add_encounters(b, 3, None), (H3_OK, String::new()));
    // Only hero battles, then the neutral kinds as a curriculum.
    assert_eq!(add_encounters(b, 2, Some([0, 0, 1, 0, 0, 0])).0, H3_OK);
    assert_eq!(add_encounters(b, 2, Some([1, 1, 0, 0, 0, 1])).0, H3_OK);
    let h = unsafe { &*b };
    let scenarios = h.batch.scenarios();
    assert_eq!(scenarios.len(), 4);
    assert!(
        scenarios[1..]
            .iter()
            .all(|s| Arc::ptr_eq(s.ruleset(), scenarios[0].ruleset()))
    );
    let mut arrays = Arrays::new(n as usize);
    let buffers = arrays.buffers();
    assert_eq!(unsafe { h3_batch_reset(b, &buffers) }, H3_OK);
    let mut rng = Rng::for_stream(4, StreamId::AGENT);
    let mut seen = [false; 6];
    for _ in 0..60 {
        for i in 0..n as usize {
            let kind = arrays.encounter[i];
            match arrays.scenario[i] {
                0 => assert_eq!(i32::from(kind), H3_ENCOUNTER_NONE),
                1 => assert!(
                    [
                        H3_ENCOUNTER_HERO,
                        H3_ENCOUNTER_SIEGE_HERO,
                        H3_ENCOUNTER_SIEGE_GARRISON
                    ]
                    .contains(&i32::from(kind))
                ),
                2 => assert_eq!(i32::from(kind), H3_ENCOUNTER_HERO),
                _ => assert!(
                    [
                        H3_ENCOUNTER_WANDERING,
                        H3_ENCOUNTER_GUARDS,
                        H3_ENCOUNTER_SIEGE_NEUTRAL
                    ]
                    .contains(&i32::from(kind))
                ),
            }
            if kind >= 0 {
                seen[kind as usize] = true;
            }
        }
        let actions: Vec<H3Action> = (0..n as usize)
            .map(|i| {
                let mask = &arrays.mask[i * ACTION_COUNT..(i + 1) * ACTION_COUNT];
                let legal: Vec<usize> = (0..ACTION_COUNT)
                    .filter(|&e| mask[e] == 1 && e != CREATURE_CAST && e != HERO_CAST)
                    .collect();
                let entry = legal[rng.below(legal.len() as u32) as usize];
                H3Action {
                    entry: entry as u32,
                    spell: 0,
                    target: 0,
                    second: 0,
                }
            })
            .collect();
        assert_eq!(
            unsafe { h3_batch_step(b, actions.as_ptr(), &buffers) },
            H3_OK
        );
        // A neutral defender is played inside the call: the caller always acts as the attacker.
        for i in 0..n as usize {
            if arrays.scenario[i] == 3 {
                assert_eq!(arrays.to_move[i], 0);
            }
        }
    }
    assert!(
        seen[H3_ENCOUNTER_HERO as usize] && seen[H3_ENCOUNTER_WANDERING as usize],
        "{seen:?}"
    );
    unsafe { h3_batch_destroy(b) };
}

/// E05 T8b (v29): seats for the encounter distribution replace each encounter's own for every
/// episode of the entry: a random attacker leaves the caller the defender, a neutral kind's
/// included; unknown or both internal seats are refused with the batch unchanged.
#[test]
fn encounter_seats_through_the_abi() {
    let n = 8;
    let b = create(&config(n, 0, 6, 300)).unwrap();
    let random = |attacker, defender| Some(H3Seats { attacker, defender });
    let (code, message) = add_encounters_seated(b, 1, None, random(H3_SEAT_EXTERNAL, 7));
    assert!(
        code == H3_ERR_ARGUMENT && message == "unknown seat",
        "{message}"
    );
    let (code, message) = add_encounters_seated(b, 1, None, random(H3_SEAT_RANDOM, H3_SEAT_RANDOM));
    assert_eq!(code, H3_ERR_ARGUMENT, "{message}");
    assert!(unsafe { &*b }.batch.scenarios().is_empty());
    let defender = random(H3_SEAT_RANDOM, H3_SEAT_EXTERNAL);
    assert_eq!(add_encounters_seated(b, 1, None, defender).0, H3_OK);
    assert_eq!(
        add_encounters_seated(b, 1, Some([1, 0, 0, 0, 0, 0]), defender).0,
        H3_OK
    );
    let mut arrays = Arrays::new(n as usize);
    let buffers = arrays.buffers();
    assert_eq!(unsafe { h3_batch_reset(b, &buffers) }, H3_OK);
    let mut rng = Rng::for_stream(6, StreamId::AGENT);
    let mut wandering = false;
    for _ in 0..80 {
        assert!(
            arrays.to_move.iter().all(|&side| side == 1),
            "{:?}",
            arrays.to_move
        );
        wandering |= arrays.encounter.contains(&(H3_ENCOUNTER_WANDERING as i8));
        let actions = actions(b, &arrays, n as usize, &mut rng);
        assert_eq!(
            unsafe { h3_batch_step(b, actions.as_ptr(), &buffers) },
            H3_OK
        );
    }
    assert!(wandering);
    unsafe { h3_batch_destroy(b) };
}

/// E05 T7 (v30): the baseline bots as internal seats: a scenario against `H3_SEAT_GREEDY` or
/// `H3_SEAT_AGGRESSOR` leaves the caller only its own side to move, and battles end.
#[test]
fn bot_seats_play_inside_the_call() {
    let n = 6;
    let b = create(&config(n, 0, 3, 300)).unwrap();
    for seats in [
        H3Seats {
            attacker: H3_SEAT_EXTERNAL,
            defender: H3_SEAT_GREEDY,
        },
        H3Seats {
            attacker: H3_SEAT_AGGRESSOR,
            defender: H3_SEAT_EXTERNAL,
        },
    ] {
        assert_eq!(add(b, SKIRMISH, 1, seats).0, H3_OK);
    }
    let both = H3Seats {
        attacker: H3_SEAT_GREEDY,
        defender: H3_SEAT_AGGRESSOR,
    };
    assert_eq!(add(b, SKIRMISH, 1, both).0, H3_ERR_ARGUMENT);
    let mut arrays = Arrays::new(n as usize);
    let buffers = arrays.buffers();
    assert_eq!(unsafe { h3_batch_reset(b, &buffers) }, H3_OK);
    let mut rng = Rng::for_stream(3, StreamId::AGENT);
    let mut ended = 0;
    for _ in 0..200 {
        for i in 0..n as usize {
            let external = if arrays.scenario[i] == 0 { 0 } else { 1 };
            assert_eq!(arrays.to_move[i], external, "env {i}");
        }
        let actions = actions(b, &arrays, n as usize, &mut rng);
        assert_eq!(
            unsafe { h3_batch_step(b, actions.as_ptr(), &buffers) },
            H3_OK
        );
        ended += arrays.done.iter().filter(|&&d| d == 1).count();
    }
    assert!(ended > 0);
    unsafe { h3_batch_destroy(b) };
}
