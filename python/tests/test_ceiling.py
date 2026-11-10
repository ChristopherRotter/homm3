"""The teacher's ceiling (E25 T19, `h3env.train.ceiling`): the shares and entropies of a state's
draws, a network scored on them, a run end to end on the fair system actor's episodes and the
committed records. They need the group `train`."""

from __future__ import annotations

import json
import math
from dataclasses import asdict
from pathlib import Path

import numpy as np
import pytest

from h3env.train.run import ConfigError, load_config

torch = pytest.importorskip("torch")

from h3env.batch import BattleBatch  # noqa: E402
from h3env.train import ceiling  # noqa: E402
from h3env.train.ceiling import Sums, entropies, mode_share, run, score_networks  # noqa: E402
from h3env.train.distill import CATEGORIES, Bench  # noqa: E402
from h3env.train.policy import CASTS, build  # noqa: E402

#: A small run on every kind, as `test_clone`'s (`e25-neutral.toml`'s shape).
RUN = """
seed = 1

[env]
envs = 8
threads = 2
max_steps = 300
kinds = [1, 1, 2, 1, 1, 1]

[network]
hidden = 32
embed = 8
tactics_rounds = 3

[ppo]
rollout = 16
shaping = true

[device]
device = "cpu"
"""

MOVE = CATEGORIES.index("move")
CAST = CATEGORIES.index("cast")


def actions(entries, **heads) -> np.ndarray:
    """The draws of one state: `entries` and the cast heads' values, in the batch's dtype."""
    probe = BattleBatch(1)
    out = np.zeros(len(entries), probe.actions.dtype)
    probe.close()
    out["entry"] = entries
    for name, values in heads.items():
        out[name] = values
    return out


def test_the_mode_share_credits_ties_and_the_entropies_correct_the_plug_in() -> None:
    assert mode_share([3, 3, 4], 3) == 1.0 and mode_share([3, 3, 4], 4) == 0.0
    assert mode_share([3, 4], 4) == 0.5 and mode_share([3, 4, 5], 6) == 0.0
    assert entropies([2, 2, 2]) == (0.0, 0.0)
    h, mm = entropies([1, 1, 1, 2])
    assert h == pytest.approx(-(0.75 * math.log(0.75) + 0.25 * math.log(0.25)))
    assert mm == pytest.approx(h + 1 / 8)


def test_a_states_draws_count_by_their_own_category() -> None:
    sums = Sums([])
    # Three draws of one move and one of another: each draw against the other three.
    sums.state(actions([5, 5, 5, 7]), np.full(4, MOVE))
    r = sums.report()
    assert r["draws_scored"] == 4 and r["categories"]["move"] == 4
    assert r["mode"]["by_category"]["move"] == pytest.approx(3 / 4)
    assert r["pair"]["all"] == pytest.approx((3 * 2 / 3) / 4)
    assert r["distinct_entries"]["all"] == 2.0 and r["cast_draws"] == 0
    assert r["mode"]["by_category"]["cast"] is None
    # A cast: the spell's mode among the draws of the same entry, the target's among those of
    # the same spell; a lone entry scores no head.
    sums = Sums([])
    cast, other = CASTS[0], CASTS[1]
    sums.state(
        actions([cast, cast, cast, other], spell=[1, 1, 2, 1], target=[3, 3, 3, 3]),
        np.full(4, CAST),
    )
    heads = sums.report()["cast_heads"]
    assert sums.casts == 4
    # Spell: draws 0 and 1 (each 1 against [1, 2]: a tie, ½), draw 2 (2 against [1, 1]: 0).
    assert heads["spell"]["draws"] == 3 and heads["spell"]["mode"] == pytest.approx(1 / 3)
    # Target: draws 0 and 1 (each other's spell and target), draw 2 has no draw of its spell.
    assert heads["target"]["draws"] == 2 and heads["target"]["mode"] == 1.0


def test_a_network_is_scored_against_every_draw() -> None:
    config = load_config(RUN)
    probe = BattleBatch(1)
    net = build(asdict(config), probe).eval()
    obs_len, entries = probe.observation_len, probe.action_count
    probe.close()
    with torch.no_grad():
        net.entry_head.bias[0] = 50.0
    mask = np.ones(entries, bool)
    states = [
        (actions([0, 0, 1]), np.full(3, MOVE), torch.zeros(obs_len), mask),
        (actions([1, 1, 1]), np.full(3, MOVE), torch.zeros(obs_len), mask),
    ]
    sums = Sums(["net"])
    score_networks(sums, {"net": net}, states)
    for d, c, _, _ in states:
        sums.state(d, c)
    x = sums.report()["networks"]["net"]
    assert x["agreement"]["by_category"]["move"] == pytest.approx(2 / 6)
    assert x["cross_entropy"]["all"] > 10.0
    # E25 T30: by the draw's category, the category of the network's entry (round 0 here: tactics).
    assert x["confusion"] == {"move": {"tactics": 1.0}}


def test_a_ceiling_runs_on_the_fair_actors_episodes(tmp_path: Path) -> None:
    bench = Bench(seed=7, envs=8, threads=2, students={"mlp": {"kind": "mlp"}})
    with pytest.raises(ConfigError, match="the bench's `run`"):
        run(bench, 3, 40)
    with pytest.raises(ConfigError, match="at least 2 draws"):
        run(bench, 1, 40, config_text=RUN)
    config = load_config(RUN)
    probe = BattleBatch(1)
    net = build(asdict(config), probe)
    probe.close()
    path = tmp_path / "net.pt"
    torch.save({"config": asdict(config), "policy": net.state_dict()}, path)
    lines: list[str] = []
    out = run(bench, 3, 120, [str(path)], config_text=RUN, log=lines.append)
    c = out["ceiling"]
    assert out["decisions"] == 120 and out["lost"] == 0 and out["episodes"] > 0
    assert c["draws_scored"] == 360 and sum(c["categories"].values()) == 360
    for key in ("mode", "pair"):
        assert 0.0 <= c[key]["all"] <= 1.0
    assert c["entropy"]["all"] <= c["entropy_miller_madow"]["all"]
    assert 0.0 <= c["networks"][str(path)]["agreement"]["all"] <= 1.0
    for row in c["networks"][str(path)]["confusion"].values():
        assert sum(row.values()) == pytest.approx(1.0)
    assert any("'s choice by the teacher's category" in line for line in lines[0].splitlines())
    assert out["teacher"]["system"] == "system" and out["teacher"]["seed"] == 8
    assert "the fair actor's ceiling" in lines[0]
    # The same bench draws the same states: a second run repeats the first.
    again = run(bench, 3, 120, [str(path)], config_text=RUN, log=lines.append)
    assert again["ceiling"] == c


def test_every_committed_ceiling_record_is_complete() -> None:
    # `records/distill/ceiling/<n>.json` (E25 T19): `--json`'s report.
    records = sorted((Path(__file__).parents[1] / "records" / "distill" / "ceiling").glob("*.json"))
    assert records
    for path in records:
        out = json.loads(path.read_text(encoding="utf-8"))
        assert {"teacher", "draws", "decisions", "identity", "ceiling"} <= set(out)
        assert out["teacher"]["system"] == ceiling.TEACHER
        c = out["ceiling"]
        assert set(c["categories"]) == set(CATEGORIES)
        assert c["draws_scored"] == out["draws"] * out["decisions"]
        assert set(c["networks"]) == set(out["networks"])
