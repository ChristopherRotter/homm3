/*
 * h3 engine C ABI, version 14 (see crates/h3-ffi/src/lib.rs and ADR-0011).
 * SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Handles are opaque. Functions return H3_OK (0) or a negative error code; they never unwind.
 * Sides: 0 = attacker, 1 = defender. A handle must not be used from two threads at once.
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

/* scenario: 0 = duel, 1 = skirmish, 2 = full armies; max_steps 0 = unlimited. NULL on error. */
H3Env *h3_env_create_synthetic(uint32_t scenario, uint64_t seed, uint32_t max_steps);
void h3_env_destroy(H3Env *env);
H3Env *h3_env_fork(const H3Env *env);
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
uint64_t h3_env_state_hash(const H3Env *env);

#ifdef __cplusplus
}
#endif

#endif /* H3_H */
