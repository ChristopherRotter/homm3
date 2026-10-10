/* C ABI smoke test (E01): create an environment, play random legal actions to the end (choosing
 * a cast's heads from their masks, E04b T2c), fork, reset, destroy. Built and run by
 * `cargo xtask ffi-smoke`. Prints the final state hash. */
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

/* A uniformly random index among the set bytes of mask[0..n] (n if none is set). */
static uint32_t pick_index(const uint8_t *mask, uint32_t n, uint64_t *lcg) {
    uint32_t legal = 0;
    for (uint32_t i = 0; i < n; i++) legal += mask[i];
    if (legal == 0) return n;
    *lcg = *lcg * 6364136223846793005ULL + 1442695040888963407ULL;
    uint32_t pick = (uint32_t)(*lcg >> 33) % legal;
    for (uint32_t i = 0; i < n; i++) {
        if (mask[i] && pick-- == 0) return i;
    }
    return n;
}

int main(void) {
    CHECK(h3_abi_version() == 16);
    uint32_t n_actions = h3_action_count();
    uint32_t n_spells = h3_spell_count();
    uint32_t n_targets = h3_target_count();
    uint32_t n_obs = h3_observation_len();
    uint8_t *mask = malloc(n_actions);
    uint8_t *spells = malloc(n_spells);
    uint8_t *targets = malloc(n_targets);
    float *obs = malloc(sizeof(float) * n_obs);
    CHECK(mask && spells && targets && obs);

    /* The full scenario: both sides are led by heroes with spell books. */
    H3Env *env = h3_env_create_synthetic(2, 42, 1000);
    CHECK(env != NULL);
    uint64_t lcg = 12345;
    H3StepResult result = {0};
    int steps = 0;
    int casts = 0;
    int stack_casts = 0;
    while (h3_env_to_move(env) >= 0) {
        int32_t side = h3_env_to_move(env);
        CHECK(h3_env_legal_mask(env, side, mask, n_actions) == H3_OK);
        CHECK(h3_env_observe(env, side, obs, n_obs) == H3_OK);
        /* Fight the battle out: never retreat or surrender (the entries before the two casts). */
        mask[n_actions - 4] = 0;
        mask[n_actions - 3] = 0;
        H3Action action = {0};
        action.entry = pick_index(mask, n_actions, &lcg);
        CHECK(action.entry < n_actions);
        CHECK(h3_env_spell_mask(env, side, action.entry, spells, n_spells) == H3_OK);
        action.spell = (uint16_t)pick_index(spells, n_spells, &lcg);
        CHECK(action.spell < n_spells);
        CHECK(h3_env_target_mask(env, side, action.entry, action.spell, targets, n_targets) ==
              H3_OK);
        action.target = (uint16_t)pick_index(targets, n_targets, &lcg);
        CHECK(action.target < n_targets);
        CHECK(h3_env_second_mask(env, side, action.entry, action.spell, action.target, targets,
                                 n_targets) == H3_OK);
        /* None (index 0) but for Teleport's destination (E04b T5a). */
        action.second = (uint16_t)pick_index(targets, n_targets, &lcg);
        CHECK(action.second < n_targets);
        casts += action.entry == n_actions - 2;
        /* The active stack's cast (E04b T7b1): the Wisps' Hasten. */
        stack_casts += action.entry == n_actions - 1;
        CHECK(h3_env_step(env, side, action, &result) == H3_OK);
        steps++;
    }
    CHECK(result.done == 1);
    CHECK(casts > 0);
    CHECK(result.result == H3_RESULT_NONE ? result.truncated == 1
                                          : result.result >= H3_RESULT_ATTACKER &&
                                                result.result <= H3_RESULT_DEFENDER_SURRENDERED);
    H3Action defend = {1, 0, 0, 0};
    CHECK(h3_env_step(env, 0, defend, &result) == H3_ERR_EPISODE_OVER);
    CHECK(h3_env_step(NULL, 0, defend, &result) == H3_ERR_NULL);

    uint64_t hash = h3_env_state_hash(env);
    H3Env *fork = h3_env_fork(env);
    CHECK(fork != NULL && h3_env_state_hash(fork) == hash);
    CHECK(h3_env_reset(env, 42) == H3_OK);
    h3_env_destroy(fork);
    h3_env_destroy(env);
    free(mask);
    free(spells);
    free(targets);
    free(obs);
    printf("ffi smoke ok: %d steps, %d casts, %d stack casts, result %d, final hash %016llx\n",
           steps, casts, stack_casts, (int)result.result, (unsigned long long)hash);
    return 0;
}
