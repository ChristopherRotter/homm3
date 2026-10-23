"""The evaluation (E05 T8, `docs/epics/E05-log.md` §8.2; the held-out sets v1, E06 T3a,
`docs/epics/E06-design.md` §5): the agent against an engine seat (Random or a baseline bot) or
another network on held-out episodes of the encounter distribution, once as the attacker and once
as the defender of the same battles.

Each side's batch plays the distribution with the agent's seat external and the opponent's
internal (C ABI v29's seats), or both external with the two networks acting on their own rows
(`duel`, E06 T3b), from a fixed batch seed; the episodes counted are the first
`ceil(episodes / envs)` of each env, so every evaluation plays the same battles however long they
last. The held-out sets: v0 (E05's, `EVAL_SEED`: 1 000 episodes over 100 envs, the default kinds;
kept for comparisons), the report set v1 (`report`: per kind with a player on both sides, batch
seed `REPORT_V1_SEED + k`, the kind one-hot, the first episode of each of 500 envs; the ladder's
and the targets') and the gate set v1 (`GATE_V1_SEED`: 500 envs × 8 episodes of the default
kinds; `gate.play_gate`). The neutral set v1 (`report(held_out="neutral-v1")`, E06 T7c, design
§12) is built as the report set from the neutral kinds (batch seed `NEUTRAL_V1_SEED + k`, weighed
by the census's shares) with one side only: the agent attacks, the system actor defends (decision
7's neutral target). `h3 ai eval --set` plays the bots on the same episodes (its constants pinned
to these by an `h3-cli` test); its tables are `python/records/bots-report-v1.json` and
`bots-neutral-v1.json` (`bots_table`). Every held-out seed (`held_out_seeds`) is checked against
training's by the trainer. An episode the engine's seat ends before the agent could act is skipped
by the batch (design §1): it is reported as `skipped`, not counted (none among v0's episodes, E05
T9b).
"""

from __future__ import annotations

import json
import math
import time
from collections.abc import Callable, Iterable, Mapping
from pathlib import Path
from typing import Any

import numpy as np

from .. import _ffi
from ..batch import BattleBatch, episode_seed

#: The held-out set v0's batch seed (E05's evaluation; `h3 ai eval --set v0`'s); a training run's
#: seed must differ (checked through the episodes' seeds, `held_out_seeds`).
EVAL_SEED = 0x5EED_E7A1_0000_0001

#: The report set v1's base seed: kind k's batch seed is `REPORT_V1_SEED + k` (design §5).
REPORT_V1_SEED = 0x5EED_E7A1_0001_0000

#: The report set v1's envs per kind, the first episode of each.
REPORT_V1_ENVS = 500

#: The report set v1's kinds (a player on both sides) and their weights in the overall rates: the
#: distribution's default weights (100, 50, 25 in `encounters-v1.toml`).
REPORT_V1_WEIGHTS = {
    _ffi.ENCOUNTER_HERO: 4,
    _ffi.ENCOUNTER_SIEGE_HERO: 2,
    _ffi.ENCOUNTER_SIEGE_GARRISON: 1,
}

#: The gate set v1 (design §5, T3b's gate): its batch seed, envs and episodes per env.
GATE_V1_SEED = 0x5EED_E7A1_0002_0000
GATE_V1_ENVS = 500
GATE_V1_EPISODES = 8

#: The neutral set v1's base seed: kind k's batch seed is `NEUTRAL_V1_SEED + k` (E06 T7c).
NEUTRAL_V1_SEED = 0x5EED_E7A1_0003_0000

#: The neutral set v1's envs per kind, the first episode of each.
NEUTRAL_V1_ENVS = 500

#: The neutral set v1's kinds (the system actor defends) and their weights in the overall rates:
#: the census's shares (A1, `share` in `encounters-v1.toml`; the distribution draws them at 0).
NEUTRAL_V1_WEIGHTS = {
    _ffi.ENCOUNTER_WANDERING: 780,
    _ffi.ENCOUNTER_GUARDS: 20,
    _ffi.ENCOUNTER_SIEGE_NEUTRAL: 25,
}

#: The kinds whose defender is the system actor: one external seat, the attacker (design §12).
NEUTRAL_KINDS = frozenset(NEUTRAL_V1_WEIGHTS)

#: The held-out sets the ladder plays (`h3 ai eval --set`'s names).
SETS = ("report-v1", "neutral-v1", "v0")

#: The sets stratified by kind (`report`): their base seed, envs per kind, kinds' weights and the
#: sides the agent plays.
STRATIFIED = {
    "report-v1": (REPORT_V1_SEED, REPORT_V1_ENVS, REPORT_V1_WEIGHTS, ("attacker", "defender")),
    "neutral-v1": (NEUTRAL_V1_SEED, NEUTRAL_V1_ENVS, NEUTRAL_V1_WEIGHTS, ("attacker",)),
}

#: The neutral set's one opponent, the system actor (`h3 ai eval --set neutral-v1`'s defender).
NEUTRAL_OPPONENT = "system"

#: Every held-out set's batch seeds: a run training on one of them is refused (`run.resolve`), and
#: so is a ladder of a network trained on one.
HELD_OUT_BATCH_SEEDS = frozenset(
    {
        EVAL_SEED,
        GATE_V1_SEED,
        *(REPORT_V1_SEED + k for k in REPORT_V1_WEIGHTS),
        *(NEUTRAL_V1_SEED + k for k in NEUTRAL_V1_WEIGHTS),
    }
)

#: The two-sided 95 % quantile of the normal distribution.
Z95 = 1.96

#: The bots' tables on the stratified sets (`h3 ai eval --set <set> --json …`).
BOTS_TABLES = {
    name: Path(__file__).resolve().parents[2] / "records" / f"bots-{name}.json"
    for name in STRATIFIED
}

#: The bots' table on the report set v1.
BOTS_TABLE = BOTS_TABLES["report-v1"]

#: The schema of `h3 ai eval --json` the tables are written in (2: the fixed defender and the
#: system actor's policy version, E06 T7c).
BOTS_TABLE_SCHEMA = 2

#: The agent's side and the winning result per side.
SIDES = (("attacker", _ffi.RESULT_ATTACKER), ("defender", _ffi.RESULT_DEFENDER))

#: The engine's seats an agent can be evaluated against (C ABI v30's bots, E05 T7; the system
#: actor, v32, E06 T7c).
OPPONENTS = ("random", "aggressor", "greedy", "system")

#: An agent: fills `batch.actions` for every row from the batch's current outputs.
Agent = Callable[[BattleBatch], None]


class Stopped(Exception):
    """A stop requested (Ctrl+C, `run.Stop`) during an evaluation, a gate or a ladder: its games
    are discarded, it has no result."""


def episodes_per_env(episodes: int, envs: int) -> int:
    return math.ceil(episodes / envs)


def episode_seeds(episodes: int, envs: int, seed: int = EVAL_SEED) -> set[int]:
    """The seeds of the evaluation's episodes (the same for both sides)."""
    k = episodes_per_env(episodes, envs)
    return {episode_seed(seed, e, i) for e in range(envs) for i in range(k)}


def report_seeds(per_kind: int = REPORT_V1_ENVS, held_out: str = "report-v1") -> set[int]:
    """The seeds of a stratified set's episodes (the report set v1's by default), the first
    `per_kind` of each kind."""
    seed, _, weights, _ = STRATIFIED[held_out]
    return {episode_seed(seed + k, e, 0) for k in weights for e in range(per_kind)}


def gate_seeds() -> set[int]:
    """The seeds of the gate set v1's episodes."""
    return episode_seeds(GATE_V1_ENVS * GATE_V1_EPISODES, GATE_V1_ENVS, GATE_V1_SEED)


def held_out_seeds(episodes: int = 1000, envs: int = 100, seed: int = EVAL_SEED) -> set[int]:
    """Every held-out episode's seed (design §5): the evaluation's (v0, as `episode_seeds`), the
    report and neutral sets v1's and the gate set v1's."""
    return (
        episode_seeds(episodes, envs, seed)
        | report_seeds()
        | report_seeds(held_out="neutral-v1")
        | gate_seeds()
    )


def random_agent(rng: np.random.Generator) -> Agent:
    """Random from Python (as the engine's: a uniform entry, then uniform heads): Random's own rate
    against Random on the same episodes."""

    def agent(batch: BattleBatch) -> None:
        batch.random_actions(rng)

    return agent


def duel(learner, opponent, side: str, seed: int) -> Agent:
    """Two networks in one batch whose seats are both external: `learner` acts on the rows where
    `side` is to move, `opponent` on the others, each sampling from its own generator (seeded
    `seed` and `seed + 1`, alike for every network: common random numbers)."""
    import torch

    mine = 0 if side == "attacker" else 1
    generators = [
        torch.Generator(device=next(net.parameters()).device).manual_seed(seed + k)
        for k, net in enumerate((learner, opponent))
    ]

    def agent(batch: BattleBatch) -> None:
        to_move = batch.to_move
        for k, (net, who) in enumerate(((learner, mine), (opponent, 1 - mine))):
            rows = np.nonzero(to_move == who)[0]
            if len(rows):
                _fill(batch, net.act(batch, generators[k], rows=rows), rows)

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
    opponent: str = "random",
    sides: Iterable[str] = ("attacker", "defender"),
    stop=None,
) -> dict:
    """Plays the evaluation's episodes with `agent` on each of `sides` against `opponent` (an
    engine seat: Random by default, or a baseline bot; `external`: both seats external, `agent`
    plays both, as `duel`'s); returns the rates per side, per kind and, with both sides, overall
    (both sides weighed equally), the agent's mean reward v1, the truncations and the episodes the
    batch skipped (ended before the agent could act: counted in no rate). Raises `Stopped` when
    `stop` (`run.Stop`) was requested before one of its batch calls."""
    if opponent not in (*OPPONENTS, "external"):
        raise ValueError(f"unknown opponent {opponent!r} (one of {OPPONENTS})")
    k = episodes_per_env(episodes, envs)
    kinds = None if kinds is None else list(kinds)
    report: dict = {"episodes_per_side": k * envs}
    sides = set(sides)
    for name, winning in SIDES:
        if name not in sides:
            continue
        attacker, defender = (
            ("external", opponent) if name == "attacker" else (opponent, "external")
        )
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
        # The episodes among each env's first k that the batch skipped: those before the first one
        # reset shows, then those between an ended episode and the next one shown.
        skipped = int(np.minimum(batch.episode, k).sum())
        wins = truncated = 0
        reward = 0.0
        by_kind: dict[str, list[int]] = {}
        try:
            # A generous bound on the calls: Random's battles last under 1 000 decisions (§8.1).
            for _ in range(k * (max_steps or 10_000) + 1):
                live = (batch.episode < k) & (batch.to_move >= 0)
                if not live.any():
                    break
                if stop is not None and stop.requested:
                    raise Stopped
                before = batch.episode.astype(np.int64)
                kind = batch.encounter.copy()
                agent(batch)
                batch.step()
                after = np.minimum(batch.episode, k).astype(np.int64)
                skipped += int((after - np.minimum(before + batch.done, k)).sum())
                for e in np.nonzero(batch.done.astype(bool) & (before < k))[0]:
                    won = int(batch.result[e]) == winning
                    wins += won
                    truncated += int(batch.truncated[e])
                    reward += float(batch.reward[e, side])
                    counted[e] += 1
                    row = by_kind.setdefault(_ffi.ENCOUNTER_KINDS[int(kind[e])], [0, 0])
                    row[0] += won
                    row[1] += 1
        finally:
            batch.close()
        played = int(counted.sum())
        report[name] = {
            "episodes": played,
            "skipped": skipped,
            "win_rate": wins / max(played, 1),
            "reward": reward / max(played, 1),
            "truncated": truncated / max(played, 1),
            "by_kind": {
                kind: {"episodes": n, "win_rate": w / n} for kind, (w, n) in sorted(by_kind.items())
            },
        }
    if len(sides) == 2:
        report["win_rate"] = (report["attacker"]["win_rate"] + report["defender"]["win_rate"]) / 2
    return report


def report(
    make_agent: Callable[[int, str], Agent],
    *,
    opponent: str = "random",
    per_kind: int = REPORT_V1_ENVS,
    held_out: str = "report-v1",
    rng_seed: int = 0,
    threads: int = 0,
    max_steps: int = 2000,
    currency: str = "ai_value",
    mana_price: int = 10,
    stop=None,
) -> dict:
    """Plays a stratified set (design §5: the report set v1 by default; the neutral set v1, whose
    one opponent is the system actor, the agent attacking), its first `per_kind` episodes of each
    kind, against `opponent` (`play`'s), an agent per (kind, side) from `make_agent(seed, side)`
    (its draws seeded alike for every network: common random numbers); returns `stratified`'s
    rates, per kind each side's `play` report, and the games, skipped episodes, truncations and
    seconds."""
    if held_out not in STRATIFIED:
        raise ValueError(f"not a stratified set: {held_out!r} (one of {tuple(STRATIFIED)})")
    seed, envs, weights, sides = STRATIFIED[held_out]
    if held_out == "neutral-v1" and opponent != NEUTRAL_OPPONENT:
        raise ValueError(f"the neutral set v1's opponent is the system actor, not {opponent!r}")
    if not 1 <= per_kind <= envs:
        raise ValueError(f"the {held_out} set has 1 to {envs} episodes per kind")
    start = time.monotonic()
    by_kind = {}
    for k in weights:
        one_hot = [int(i == k) for i in range(len(_ffi.ENCOUNTER_KINDS))]
        by_kind[_ffi.ENCOUNTER_KINDS[k]] = {
            side: play(
                make_agent(rng_seed + 2 * (2 * k + j), side),
                episodes=per_kind,
                envs=per_kind,
                threads=threads,
                seed=seed + k,
                max_steps=max_steps,
                kinds=one_hot,
                currency=currency,
                mana_price=mana_price,
                opponent=opponent,
                sides=(side,),
                stop=stop,
            )[side]
            for j, side in enumerate(sides)
        }
    out = stratified(by_kind, held_out)
    sides = [s for kind in by_kind.values() for s in kind.values()]
    games = sum(s["episodes"] for s in sides)
    out.update(
        games=games,
        skipped=sum(s["skipped"] for s in sides),
        truncated=sum(s["truncated"] * s["episodes"] for s in sides) / max(games, 1),
        seconds=time.monotonic() - start,
        by_kind=by_kind,
    )
    return out


def stratified(by_kind: dict[str, dict], held_out: str = "report-v1") -> dict:
    """A stratified set's overall rates from each kind's sides (`play`'s per-side reports): the
    win rate and the value efficiency weighed by the set's weights (the report set's
    `REPORT_V1_WEIGHTS`), its sides equally, the win rate's 95 % interval from the stratified
    variance `Σ Σ w² r (1 − r) / n` over the sides' count squared (design §5), and each kind's
    win rate (its sides)."""
    _, _, weights, sides = STRATIFIED[held_out]
    total = sum(weights.values())
    m = len(sides)
    rate = value = variance = 0.0
    kinds = {}
    for k, weight in weights.items():
        name = _ffi.ENCOUNTER_KINDS[k]
        w = weight / total
        for side in sides:
            s = by_kind[name][side]
            r = s["win_rate"]
            rate += w * r / m
            value += w * s["reward"] / m
            variance += w * w * r * (1 - r) / max(s["episodes"], 1) / (m * m)
        kinds[name] = sum(by_kind[name][side]["win_rate"] for side in sides) / m
    return {"win_rate": rate, "interval": Z95 * math.sqrt(variance), "value": value, "kinds": kinds}


def bots_table(held_out: str = "report-v1", path: Path | None = None) -> dict:
    """The bots' table on a stratified set (`h3 ai eval --set <set> --json`, committed in
    `BOTS_TABLES`, or `path`): its schema and strata are the set's (checked), its rates by
    `(policy, opponent)` under `by_pair`."""
    path = Path(path or BOTS_TABLES[held_out])
    table = json.loads(path.read_text())
    seed, envs, weights, _ = STRATIFIED[held_out]
    strata = [(s["name"], s["seed"], s["envs"]) for s in table["strata"]]
    expected = [(_ffi.ENCOUNTER_KINDS[k], f"{seed + k:#018x}", envs) for k in weights]
    if table.get("schema") != BOTS_TABLE_SCHEMA or table["set"] != held_out or strata != expected:
        raise ValueError(f"{path}: not the {held_out} set's table (schema {BOTS_TABLE_SCHEMA})")
    table["by_pair"] = {(r["policy"], r["opponent"]): r for r in table["rates"]}
    return table


def _fill(batch: BattleBatch, choice, rows: np.ndarray | None = None) -> None:
    """`choice`'s heads into the batch's actions: every row's, or `rows`' (`choice` theirs)."""
    a = batch.actions
    where = slice(None) if rows is None else rows
    a["entry"][where] = choice.entry
    a["spell"][where] = choice.spell
    a["target"][where] = choice.target
    a["second"][where] = choice.second


def ladder(
    policy,
    *,
    opponents: Iterable[str] = OPPONENTS,
    networks: Mapping[str, Any] | None = None,
    held_out: str = "report-v1",
    per_kind: int = REPORT_V1_ENVS,
    rng_seed: int = 0,
    stop=None,
    **settings,
) -> dict[str, dict]:
    """The sampled policy against each engine seat in `opponents` (E05 T7) and each network of
    `networks` (by name; both seats external, E06 T3b) on a held-out set: the report set v1's first
    `per_kind` episodes of each kind (`report`; E06 T3a), v0's episodes `settings` give (as
    `play`'s; the engine seats only), or the neutral set v1's first `per_kind` per kind against
    its one opponent, the system actor (E06 T7c; the row `system`, whatever `opponents` names, no
    networks): the network's row beside `h3 ai eval`'s table on the same episodes. Each network's
    mode is restored after."""
    import torch

    if held_out not in SETS:
        raise ValueError(f"unknown held-out set {held_out!r} (one of {SETS})")
    networks = dict(networks or {})
    if held_out != "report-v1" and networks:
        raise ValueError(f"the held-out set {held_out} has no network opponents")
    if held_out == "neutral-v1":
        opponents = (NEUTRAL_OPPONENT,)
    nets = [policy, *networks.values()]
    modes = [net.training for net in nets]
    for net in nets:
        net.eval()
    device = next(policy.parameters()).device

    def sampled(seed: int, side: str = "attacker") -> Agent:
        generator = torch.Generator(device=device).manual_seed(seed)

        def agent(batch: BattleBatch) -> None:
            _fill(batch, policy.act(batch, generator=generator))

        return agent

    out = {}
    try:
        for opponent in opponents:
            if held_out == "v0":
                out[opponent] = play(sampled(rng_seed), opponent=opponent, stop=stop, **settings)
            else:
                out[opponent] = report(
                    sampled,
                    opponent=opponent,
                    per_kind=per_kind,
                    held_out=held_out,
                    rng_seed=rng_seed,
                    stop=stop,
                    **settings,
                )
        for name, net in networks.items():
            out[name] = report(
                lambda seed, side, net=net: duel(policy, net, side, seed),
                opponent="external",
                per_kind=per_kind,
                rng_seed=rng_seed,
                stop=stop,
                **settings,
            )
    finally:
        for net, mode in zip(nets, modes, strict=True):
            net.train(mode)
    return out


def compact(row: dict) -> dict:
    """A stratified set's row (`report`) as a record keeps it: the rates, each kind's per side
    (attacking, defending; the neutral set's attacking only), the games, skipped episodes,
    truncations and seconds."""
    return {
        "win_rate": row["win_rate"],
        "interval": row["interval"],
        "value": row["value"],
        "kinds": row["kinds"],
        "sides": {
            kind: [side["win_rate"] for side in sides.values()]
            for kind, sides in row["by_kind"].items()
        },
        "games": row["games"],
        "skipped": row["skipped"],
        "truncated": row["truncated"],
        "seconds": row["seconds"],
    }


def trained_seed(path) -> int:
    """The batch seed a checkpoint's run trained with (its episodes are `episode_seed(seed, …)`)."""
    import torch

    return int(torch.load(path, map_location="cpu", weights_only=True)["config"]["seed"])


def load(path, device: str = "cpu"):
    """The network of a checkpoint (`checkpoint.pt`, `best.pt`), of the kind and size the run's
    config names (`policy.build`)."""
    import torch

    from .policy import build

    saved = torch.load(path, map_location=device, weights_only=True)
    probe = BattleBatch(1)
    policy = build(saved["config"], probe)
    probe.close()
    policy.load_state_dict(saved["policy"])
    return policy.to(device)
