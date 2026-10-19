"""The smoke run's evaluation (E05 T8, `docs/epics/E05-design.md` §8.2): the agent against the
engine's Random on held-out episodes of the encounter family, once as the attacker and once as the
defender of the same battles.

Each side's batch plays the family with the agent's seat external and Random's internal (C ABI
v29's seats), from the fixed batch seed `EVAL_SEED`; the episodes counted are the first
`ceil(episodes / envs)` of each env, so every evaluation plays the same battles however long they
last. Their seeds (`episode_seeds`) are checked against training's by the caller.
"""

from __future__ import annotations

import math
from collections.abc import Callable, Iterable

import numpy as np

from .. import _ffi
from ..batch import BattleBatch, episode_seed

#: The evaluation batches' seed; a training run's seed must differ (checked through the episodes'
#: seeds, `episode_seeds`).
EVAL_SEED = 0x5EED_E7A1_0000_0001

#: The agent's seat and the winning result per side.
SIDES = (
    ("attacker", ("external", "random"), _ffi.RESULT_ATTACKER),
    ("defender", ("random", "external"), _ffi.RESULT_DEFENDER),
)

#: An agent: fills `batch.actions` for every row from the batch's current outputs.
Agent = Callable[[BattleBatch], None]


def episodes_per_env(episodes: int, envs: int) -> int:
    return math.ceil(episodes / envs)


def episode_seeds(episodes: int, envs: int, seed: int = EVAL_SEED) -> set[int]:
    """The seeds of the evaluation's episodes (the same for both sides)."""
    k = episodes_per_env(episodes, envs)
    return {episode_seed(seed, e, i) for e in range(envs) for i in range(k)}


def random_agent(rng: np.random.Generator) -> Agent:
    """Random from Python (as the engine's: a uniform entry, then uniform heads): Random's own rate
    against Random on the same episodes."""

    def agent(batch: BattleBatch) -> None:
        batch.random_actions(rng)

    return agent


def play(
    agent: Agent,
    *,
    episodes: int = 1000,
    envs: int = 100,
    threads: int = 0,
    seed: int = EVAL_SEED,
    max_steps: int = 2000,
    kinds: Iterable[int] | None = None,
    currency: str = "ai_value",
    mana_price: int = 10,
) -> dict:
    """Plays the evaluation's episodes with `agent` on each side against Random; returns the rates
    per side, per kind and overall (both sides weighed equally), the agent's mean reward v1 and
    the truncations."""
    k = episodes_per_env(episodes, envs)
    kinds = None if kinds is None else list(kinds)
    report: dict = {"episodes_per_side": k * envs}
    wins_total = 0
    for name, (attacker, defender), winning in SIDES:
        batch = BattleBatch(
            envs,
            threads=threads,
            seed=seed,
            max_steps=max_steps,
            currency=currency,
            mana_price=mana_price,
            final_obs=False,
        )
        batch.add_encounters(kinds=kinds, attacker=attacker, defender=defender)
        batch.reset()
        side = 0 if name == "attacker" else 1
        counted = np.zeros(envs, np.int64)
        wins = truncated = 0
        reward = 0.0
        by_kind: dict[str, list[int]] = {}
        # A generous bound on the calls: Random's battles last under 1 000 decisions (§8.1).
        for _ in range(k * (max_steps or 10_000) + 1):
            live = (counted < k) & (batch.to_move >= 0)
            if not live.any():
                break
            before = batch.episode.copy()
            kind = batch.encounter.copy()
            agent(batch)
            batch.step()
            for e in np.nonzero(batch.done.astype(bool) & (before < k))[0]:
                won = int(batch.result[e]) == winning
                wins += won
                truncated += int(batch.truncated[e])
                reward += float(batch.reward[e, side])
                counted[e] += 1
                row = by_kind.setdefault(_ffi.ENCOUNTER_KINDS[int(kind[e])], [0, 0])
                row[0] += won
                row[1] += 1
        batch.close()
        played = int(counted.sum())
        wins_total += wins
        report[name] = {
            "episodes": played,
            "win_rate": wins / max(played, 1),
            "reward": reward / max(played, 1),
            "truncated": truncated / max(played, 1),
            "by_kind": {
                kind: {"episodes": n, "win_rate": w / n} for kind, (w, n) in sorted(by_kind.items())
            },
        }
    report["win_rate"] = (report["attacker"]["win_rate"] + report["defender"]["win_rate"]) / 2
    return report


def evaluate(policy, config, rng_seed: int = 0, baseline: bool = False) -> dict:
    """The policy's evaluation with `config`'s settings (`ppo.Config`): the sampled policy (exit
    criterion 7's; its draws from a generator seeded with `rng_seed`, so every evaluation plays
    alike), greedy (with `config.eval_greedy`) and, with `baseline`, Random's own rates on the
    same episodes. Greedy play can repeat itself against Random until the step limit (T8b's
    sandbox run: up to 17 % truncated), hence the sampled policy as the criterion (the user,
    2026-10-19)."""
    import torch

    policy.eval()

    def greedy(batch: BattleBatch) -> None:
        c = policy.act(batch, greedy=True)
        _fill(batch, c)

    generator = torch.Generator(device=next(policy.parameters()).device).manual_seed(rng_seed)

    def sampled(batch: BattleBatch) -> None:
        _fill(batch, policy.act(batch, generator=generator))

    settings = dict(
        episodes=config.eval_episodes,
        envs=config.eval_envs,
        threads=config.threads,
        seed=config.eval_seed,
        max_steps=config.max_steps,
        kinds=config.kinds,
        currency=config.currency,
        mana_price=config.mana_price,
    )
    out = {"sampled": play(sampled, **settings)}
    if config.eval_greedy:
        out["greedy"] = play(greedy, **settings)
    if baseline:
        out["random"] = play(random_agent(np.random.default_rng(rng_seed)), **settings)
    policy.train()
    return out


def _fill(batch: BattleBatch, choice) -> None:
    a = batch.actions
    a["entry"] = choice.entry
    a["spell"] = choice.spell
    a["target"] = choice.target
    a["second"] = choice.second
