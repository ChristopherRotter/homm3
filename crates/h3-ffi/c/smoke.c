/* C ABI smoke test (E01): create an environment, play random legal actions to the end (choosing
 * a cast's heads from their masks, E04b T2c), fork, reset, destroy; since E05 T2 also a siege
 * created from a scenario's text, and a scenario's error message. Built and run by
 * `cargo xtask ffi-smoke`. Prints the final state hashes. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

/* The wall-shot entries at the end of the action space (action encoding v13, E04c T6). */
#define WALL_SHOTS 8

/* A siege on the synthetic pack as a scenario's text (E05 T2, the format of
 * packs/synthetic/scenarios/): a hero with Advanced Ballistics aims her Catapult (BAT-SIE-011)
 * at a citadel with a damaging moat. */
static const char SIEGE[] =
    "[scenario]\n"
    "id = \"smoke-siege\"\n"
    "packs = [\"synthetic\"]\n"
    "profile = \"synthetic:faithful\"\n"
    "terrain = \"synthetic:meadow\"\n"
    "[scenario.siege]\n"
    "faction = \"synthetic:vale\"\n"
    "fort = \"citadel\"\n"
    "moat = \"damaging\"\n"
    "[scenario.attacker_hero]\n"
    "hero_type = \"synthetic:brea\"\n"
    "catapult = 2\n"
    "[[attacker]]\n"
    "slot = 0\n"
    "creature = \"synthetic:spearman\"\n"
    "count = 30\n"
    "[[attacker]]\n"
    "slot = 1\n"
    "creature = \"synthetic:hurler\"\n"
    "count = 6\n"
    "[[defender]]\n"
    "slot = 0\n"
    "creature = \"synthetic:spearman\"\n"
    "count = 20\n";

typedef struct Played {
    int steps;
    int casts;       /* a hero's casts */
    int stack_casts; /* a stack's casts */
    int wall_shots;  /* the Catapult's aimed shots */
    H3StepResult result;
} Played;

/* Plays env to its end with random legal actions; 0 on success. */
static int play(H3Env *env, Played *out) {
    uint32_t n_actions = h3_action_count();
    uint32_t n_spells = h3_spell_count();
    uint32_t n_targets = h3_target_count();
    uint32_t n_obs = h3_observation_len();
    uint8_t *mask = malloc(n_actions);
    uint8_t *spells = malloc(n_spells);
    uint8_t *targets = malloc(n_targets);
    float *obs = malloc(sizeof(float) * n_obs);
    CHECK(mask && spells && targets && obs);
    uint64_t lcg = 12345;
    memset(out, 0, sizeof *out);
    while (h3_env_to_move(env) >= 0) {
        int32_t side = h3_env_to_move(env);
        CHECK(h3_env_legal_mask(env, side, mask, n_actions) == H3_OK);
        CHECK(h3_env_observe(env, side, obs, n_obs) == H3_OK);
        /* Fight the battle out: never retreat or surrender (the entries before the two casts,
           which come before the eight wall shots since action encoding v13). */
        mask[n_actions - WALL_SHOTS - 4] = 0;
        mask[n_actions - WALL_SHOTS - 3] = 0;
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
        out->casts += action.entry == n_actions - WALL_SHOTS - 2;
        /* The active stack's cast (E04b T7b1): the Wisps' Hasten. */
        out->stack_casts += action.entry == n_actions - WALL_SHOTS - 1;
        out->wall_shots += action.entry >= n_actions - WALL_SHOTS;
        CHECK(h3_env_step(env, side, action, &out->result) == H3_OK);
        out->steps++;
    }
    CHECK(out->result.done == 1);
    CHECK(out->result.result == H3_RESULT_NONE
              ? out->result.truncated == 1
              : out->result.result >= H3_RESULT_ATTACKER &&
                    out->result.result <= H3_RESULT_DEFENDER_SURRENDERED);
    free(mask);
    free(spells);
    free(targets);
    free(obs);
    return 0;
}

int main(void) {
    CHECK(h3_abi_version() == 26);

    /* The full scenario: both sides are led by heroes with spell books. */
    H3Env *env = h3_env_create_synthetic(2, 42, 1000);
    CHECK(env != NULL);
    Played full;
    CHECK(play(env, &full) == 0);
    CHECK(full.casts > 0);
    H3StepResult result = {0};
    H3Action defend = {1, 0, 0, 0};
    CHECK(h3_env_step(env, 0, defend, &result) == H3_ERR_EPISODE_OVER);
    CHECK(h3_env_step(NULL, 0, defend, &result) == H3_ERR_NULL);

    uint64_t hash = h3_env_state_hash(env);
    H3Env *fork = h3_env_fork(env);
    CHECK(fork != NULL && h3_env_state_hash(fork) == hash);
    CHECK(h3_env_reset(env, 42) == H3_OK);
    h3_env_destroy(fork);
    h3_env_destroy(env);

    /* A siege from a scenario's text (E05 T2, F-robust-2). */
    char error[256] = {0};
    H3Env *siege = h3_env_create_scenario(SIEGE, strlen(SIEGE), 7, 1000, error, sizeof error);
    CHECK(siege != NULL);
    Played town;
    CHECK(play(siege, &town) == 0);
    CHECK(town.wall_shots > 0);
    uint64_t siege_hash = h3_env_state_hash(siege);
    h3_env_destroy(siege);

    /* An error comes back as a message. */
    const char *bad = "[scenario";
    CHECK(h3_env_create_scenario(bad, strlen(bad), 7, 0, error, sizeof error) == NULL);
    CHECK(strncmp(error, "scenario: ", 10) == 0);

    printf("ffi smoke ok: %d steps, %d casts, %d stack casts, result %d, final hash %016llx; "
           "siege: %d steps, %d wall shots, result %d, final hash %016llx\n",
           full.steps, full.casts, full.stack_casts, (int)full.result.result,
           (unsigned long long)hash, town.steps, town.wall_shots, (int)town.result.result,
           (unsigned long long)siege_hash);
    return 0;
}
