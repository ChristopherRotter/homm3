/*
 * h3 engine C ABI, version 32 (see crates/h3-ffi/src/lib.rs and ADR-0011; the batch: ADR-0025).
 * SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Handles are opaque. Functions return H3_OK (0) or a negative error code; they never unwind.
 * Sides: 0 = attacker, 1 = defender. A handle must not be used from two threads at once.
 *
 * Trusted tooling (ADR-0011, ADR-0025 point 12): h3_env_fork, h3_env_state_hash and the episode
 * seeds (H3BatchBuffers.episode_seed, h3_batch_episode_seed) carry the full state or the battle's
 * future draws. They are for training tools, replays and tests: never an agent's input, never
 * forwarded to a client. An agent sees the projected observations only (h3_env_observe, obs).
 */
#ifndef H3_H
#define H3_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define H3_OK 0
#define H3_ERR_NULL (-1)
#define H3_ERR_ARGUMENT (-2)
#define H3_ERR_INVALID_ACTION (-3)
#define H3_ERR_REJECTED (-4)
#define H3_ERR_EPISODE_OVER (-5)
#define H3_ERR_NOT_READY (-6) /* a batch without a scenario, or not reset yet */
#define H3_ERR_PANIC (-99)

/* H3StepResult.result */
#define H3_RESULT_NONE (-1)    /* not decided (running or truncated) */
#define H3_RESULT_ATTACKER 0   /* the attacker won */
#define H3_RESULT_DEFENDER 1   /* the defender won */
#define H3_RESULT_DRAW 2       /* both armies wiped out at once */
#define H3_RESULT_ATTACKER_RETREATED 3   /* the attacker retreated: the defender wins */
#define H3_RESULT_DEFENDER_RETREATED 4   /* the defender retreated: the attacker wins */
#define H3_RESULT_ATTACKER_SURRENDERED 5 /* the attacker surrendered: the defender wins */
#define H3_RESULT_DEFENDER_SURRENDERED 6 /* the defender surrendered: the attacker wins */

typedef struct H3Env H3Env;

/* An action: an entry of the action encoding and, for a cast, the spell and target heads (all 0
 * for every other entry). Targets: 0 = none, 1 + h = hex h, 256 + s = stack slot s; the second
 * target is Teleport's destination hex, 0 for every other spell. Choose the heads in order, each
 * from its mask: h3_env_legal_mask, h3_env_spell_mask, h3_env_target_mask, h3_env_second_mask. */
typedef struct H3Action {
    uint32_t entry;
    uint16_t spell;
    uint16_t target;
    uint16_t second;
} H3Action;

typedef struct H3StepResult {
    uint8_t done;
    uint8_t truncated;
    int32_t result; /* H3_RESULT_* */
    float reward_attacker;
    float reward_defender;
} H3StepResult;

uint32_t h3_abi_version(void);
uint32_t h3_action_count(void); /* entries */
uint32_t h3_spell_count(void);  /* values of H3Action.spell */
uint32_t h3_target_count(void); /* values of H3Action.target and .second */
uint32_t h3_observation_len(void);
uint32_t h3_observation_encoding_version(void); /* v31 */
uint32_t h3_action_encoding_version(void);      /* v31 */

/* scenario: 0 = duel, 1 = skirmish, 2 = full armies; max_steps 0 = unlimited. NULL on error. */
H3Env *h3_env_create_synthetic(uint32_t scenario, uint64_t seed, uint32_t max_steps);
/* Any battle (a field, a special battlefield, a garrison, a town under siege) from a scenario's
 * text: toml[0..len], UTF-8, not NUL-terminated (the format of packs/synthetic/scenarios/), on
 * the built-in packs it names ("synthetic", "synthetic-ext", "base", "ab", "sod"); max_steps 0 =
 * unlimited. NULL on error, its message written into error[0..error_len] as a NUL-terminated
 * string cut to fit (nothing if error is NULL or error_len is 0); untouched on success. */
H3Env *h3_env_create_scenario(const char *toml, size_t len, uint64_t seed, uint32_t max_steps,
                              char *error, size_t error_len);
void h3_env_destroy(H3Env *env);
H3Env *h3_env_fork(const H3Env *env); /* trusted tooling: the full state */
int32_t h3_env_reset(H3Env *env, uint64_t seed);
int32_t h3_env_to_move(const H3Env *env);
int32_t h3_env_legal_mask(H3Env *env, int32_t actor, uint8_t *out, size_t len);
int32_t h3_env_spell_mask(H3Env *env, int32_t actor, uint32_t entry, uint8_t *out, size_t len);
int32_t h3_env_target_mask(H3Env *env, int32_t actor, uint32_t entry, uint16_t spell, uint8_t *out,
                           size_t len);
int32_t h3_env_second_mask(H3Env *env, int32_t actor, uint32_t entry, uint16_t spell,
                           uint16_t target, uint8_t *out, size_t len);
int32_t h3_env_observe(const H3Env *env, int32_t viewer, float *out, size_t len);
int32_t h3_env_step(H3Env *env, int32_t actor, H3Action action, H3StepResult *out);
uint64_t h3_env_state_hash(const H3Env *env); /* trusted tooling: the full state's hash */

/* The training batch (v27, ADR-0025): N envs per handle, stepped with one call on T worker
 * threads, each env's next episode started in the call that ends one (its outputs then show the
 * new episode, and done/result/reward/final_obs the one that ended). Every acting side the batch
 * reports is an external seat; internal seats play inside the call. Both sides play as human
 * players' (ADR-0025 point 6); Retreat and Surrender are never legal and refused (point 7). */
#define H3_SEAT_EXTERNAL 0 /* the caller plays the side */
#define H3_SEAT_RANDOM 1   /* uniformly random legal commands inside the call (tests) */
#define H3_SEAT_AGGRESSOR 2 /* v30: the baseline bot Aggressor inside the call (E05 T7) */
#define H3_SEAT_GREEDY 3    /* v30: the baseline bot Greedy inside the call (E05 T7) */
#define H3_SEAT_SYSTEM 4    /* v32: the system actor, the original's computer, inside the call
                              (E06 T7); a neutral kind's defender in the encounter distribution */
/* The encounter distribution's kinds (v28): H3BatchBuffers.encounter and the weights of
 * h3_batch_add_encounters, in this order. */
#define H3_ENCOUNTER_NONE (-1)          /* the episode's scenario is no encounter distribution */
#define H3_ENCOUNTER_WANDERING 0        /* a hero attacks a wandering monster */
#define H3_ENCOUNTER_GUARDS 1           /* a hero attacks the guards of a pile or a garrison */
#define H3_ENCOUNTER_HERO 2             /* a hero attacks a hero in the field */
#define H3_ENCOUNTER_SIEGE_HERO 3       /* a hero attacks a town a hero defends */
#define H3_ENCOUNTER_SIEGE_GARRISON 4   /* a hero attacks a player's town without a hero */
#define H3_ENCOUNTER_SIEGE_NEUTRAL 5    /* a hero attacks a neutral town */
#define H3_BATCH_MAX_ENVS 65536 /* v31: H3BatchConfig.envs at most (h3_sim::MAX_ENVS) */
#define H3_CURRENCY_AI_VALUE 0 /* the creatures' ai_value */
#define H3_CURRENCY_GOLD 1     /* the creatures' cost in the ruleset's money */

typedef struct H3Batch H3Batch;

/* Reward v1: (K_o - K_s - mana_price * (m_s - m_o)) / S, at the call that decides a battle. */
typedef struct H3RewardConfig {
    uint32_t currency;   /* H3_CURRENCY_* */
    uint32_t mana_price; /* per spell point, in the currency (default 10) */
} H3RewardConfig;

typedef struct H3BatchConfig {
    uint32_t envs;      /* 1 <= N <= H3_BATCH_MAX_ENVS (v31: more is refused) */
    uint32_t threads;   /* T (at most N are started); 0 = the caller's thread */
    uint64_t seed;      /* every env's episode seeds derive from it */
    uint32_t max_steps; /* accepted steps of both seats per episode; 0 = unlimited */
    H3RewardConfig reward;
} H3BatchConfig;

typedef struct H3Seats {
    int32_t attacker; /* H3_SEAT_*; at least one side external */
    int32_t defender;
} H3Seats;

/* An episode source entry's engine identity (v31, E06 T2c): a training run's record names it. */
typedef struct H3ScenarioInfo {
    uint64_t ruleset_hash;           /* the core ruleset's content hash (a replay header's) */
    uint64_t battle_ruleset_hash;    /* the battle ruleset's (a replay header's) */
    uint64_t adventure_ruleset_hash; /* the encounter distribution's adventure ruleset's; 0 for a
                                        scenario */
    uint32_t weight;                 /* its weight among the entries */
    uint32_t encounters;             /* 1: h3_batch_add_encounters' entry, 0: a scenario */
} H3ScenarioInfo;

/* The caller's arrays, N rows each; a NULL pointer skips that output. They must not overlap. */
typedef struct H3BatchBuffers {
    float *obs;             /* [N, h3_observation_len()]: the acting side's observation */
    uint8_t *entry_mask;    /* [N, h3_action_count()] */
    int8_t *to_move;        /* the acting side, -1 for an env that gave up */
    float *reward;          /* [N, 2]: this call's reward (attacker, defender) */
    uint8_t *done;          /* an episode ended in this call (decided or truncated) */
    uint8_t *truncated;
    int8_t *result;         /* H3_RESULT_* of the episode that ended, else H3_RESULT_NONE */
    float *final_obs;       /* [N, 2, h3_observation_len()]: both sides' last observations,
                               written only for the envs truncated in this call */
    uint32_t *episode;      /* the env's episode counter (0 the first) */
    uint32_t *scenario;     /* the index of its scenario, in the order added */
    uint64_t *episode_seed; /* its seed (trusted tooling): a single env of the episode's
                               scenario with both sides human and this seed plays the same
                               battle (an encounter's: Rust's Scenario::episode only) */
    int32_t *status;        /* H3_OK, or why the env refused this call's action (unchanged) */
    int8_t *encounter;      /* its episode's H3_ENCOUNTER_* kind (v28) */
} H3BatchBuffers;

/* NULL on error, its message written into error[0..error_len] as for h3_env_create_scenario. */
H3Batch *h3_batch_create(const H3BatchConfig *config, char *error, size_t error_len);
void h3_batch_destroy(H3Batch *batch);
/* Adds a scenario's text (as for h3_env_create_scenario) with weight >= 1 and its seats;
 * scenarios on the same packs and profile share one ruleset. H3_ERR_ARGUMENT with a message on
 * error (the batch unchanged). */
int32_t h3_batch_add_scenario(H3Batch *batch, const char *toml, size_t len, uint32_t weight,
                              H3Seats seats, char *error, size_t error_len);
/* Adds the encounter distribution v1 (v28, E05 T5b) with weight >= 1: each episode's battle and
 * seats drawn from its seed (a neutral defender played inside the call), on the original packs
 * (base, ab, sod, base:faithful; their ruleset shared with the batch's scenarios on them).
 * kinds: NULL for the distribution's weights (the neutral kinds 0), or six weights in
 * H3_ENCOUNTER_* order, not all 0. seats (v29): NULL for each encounter's own, else these for
 * every episode of the entry (at least one external). H3_ERR_ARGUMENT with a message on error
 * (batch unchanged). */
int32_t h3_batch_add_encounters(H3Batch *batch, uint32_t weight, const uint32_t *kinds,
                                const H3Seats *seats, char *error, size_t error_len);
/* Every env's first episode (a later call replays the same episodes). */
int32_t h3_batch_reset(H3Batch *batch, const H3BatchBuffers *out);
/* The head masks for every env at once (len >= N * h3_spell_count() or h3_target_count()),
 * conditioned on each env's earlier choices; "0 only" for an entry that is no cast. The N-value
 * input arrays must not overlap out. */
int32_t h3_batch_spell_mask(H3Batch *batch, const uint32_t *entries, uint8_t *out, size_t len);
int32_t h3_batch_target_mask(H3Batch *batch, const uint32_t *entries, const uint16_t *spells,
                             uint8_t *out, size_t len);
int32_t h3_batch_second_mask(H3Batch *batch, const uint32_t *entries, const uint16_t *spells,
                             const uint16_t *targets, uint8_t *out, size_t len);
/* actions[0..N]; H3_ERR_REJECTED if an env refused its action (see status), the others step. */
int32_t h3_batch_step(H3Batch *batch, const H3Action *actions, const H3BatchBuffers *out);
/* The seed of an env's episode (0 the first), as H3BatchBuffers.episode_seed (trusted tooling). */
uint64_t h3_batch_episode_seed(uint64_t batch_seed, uint32_t env, uint32_t episode);
/* v31 (E06 T2c): the entries in the order added (0 for NULL), each one's identity (H3_ERR_NULL,
 * H3_ERR_ARGUMENT for an index past them), and the encounter distribution's version. */
uint32_t h3_batch_scenario_count(const H3Batch *batch);
int32_t h3_batch_scenario_info(const H3Batch *batch, uint32_t index, H3ScenarioInfo *out);
uint32_t h3_encounters_version(void);
/* v32 (E06 T7): the behaviour version of H3_SEAT_SYSTEM's policy (ADR-0027 point 12). */
uint32_t h3_system_policy_version(void);

#ifdef __cplusplus
}
#endif

#endif /* H3_H */
