/* C ABI smoke test (E01): create an environment, play random legal actions to the end, fork,
 * reset, destroy. Built and run by `cargo xtask ffi-smoke`. Prints the final state hash. */
#include <stdio.h>
#include <stdlib.h>

#include "h3.h"

#define CHECK(cond)                                                          \
    do {                                                                     \
        if (!(cond)) {                                                       \
            fprintf(stderr, "check failed: %s (line %d)\n", #cond, __LINE__); \
            return 1;                                                        \
        }                                                                    \
    } while (0)

int main(void) {
    CHECK(h3_abi_version() == 3);
    uint32_t n_actions = h3_action_count();
    uint32_t n_obs = h3_observation_len();
    uint8_t *mask = malloc(n_actions);
    float *obs = malloc(sizeof(float) * n_obs);
    CHECK(mask && obs);

    H3Env *env = h3_env_create_synthetic(1, 42, 1000);
    CHECK(env != NULL);
    uint64_t lcg = 12345;
    H3StepResult result = {0};
    int steps = 0;
    while (h3_env_to_move(env) >= 0) {
        int32_t side = h3_env_to_move(env);
        CHECK(h3_env_legal_mask(env, side, mask, n_actions) == H3_OK);
        CHECK(h3_env_observe(env, side, obs, n_obs) == H3_OK);
        uint32_t legal = 0;
        for (uint32_t i = 0; i < n_actions; i++) legal += mask[i];
        CHECK(legal > 0);
        lcg = lcg * 6364136223846793005ULL + 1442695040888963407ULL;
        uint32_t pick = (uint32_t)(lcg >> 33) % legal;
        uint32_t action = 0;
        for (uint32_t i = 0; i < n_actions; i++) {
            if (mask[i] && pick-- == 0) {
                action = i;
                break;
            }
        }
        CHECK(h3_env_step(env, side, action, &result) == H3_OK);
        steps++;
    }
    CHECK(result.done == 1);
    CHECK(result.result == H3_RESULT_NONE ? result.truncated == 1
                                          : result.result >= H3_RESULT_ATTACKER &&
                                                result.result <= H3_RESULT_DRAW);
    CHECK(h3_env_step(env, 0, 1, &result) == H3_ERR_EPISODE_OVER);
    CHECK(h3_env_step(NULL, 0, 1, &result) == H3_ERR_NULL);

    uint64_t hash = h3_env_state_hash(env);
    H3Env *fork = h3_env_fork(env);
    CHECK(fork != NULL && h3_env_state_hash(fork) == hash);
    CHECK(h3_env_reset(env, 42) == H3_OK);
    h3_env_destroy(fork);
    h3_env_destroy(env);
    free(mask);
    free(obs);
    printf("ffi smoke ok: %d steps, result %d, final hash %016llx\n", steps, (int)result.result,
           (unsigned long long)hash);
    return 0;
}
