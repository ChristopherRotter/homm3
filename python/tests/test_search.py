"""The search over determinized worlds (E06 T8b, design §14; ADR-0025 point 12a): the worlds
through the binding, the schedule and selection rules, and the search agent's choices."""

from __future__ import annotations

import numpy as np
import pytest

from h3env import BattleBatch, H3Error, _ffi

torch = pytest.importorskip("torch")

from h3env.search import Node, Searcher, considered_visits, search  # noqa: E402
from h3env.train.evaluate import play  # noqa: E402
from h3env.train.policy import Mlp  # noqa: E402

#: The player kinds with a hero to guess (the hero battles and the sieges a hero defends).
HERO_KINDS = [0, 0, 2, 1, 0, 0]


def batch(envs: int = 6, seed: int = 4) -> BattleBatch:
    b = BattleBatch(envs, seed=seed, max_steps=600)
    b.add_encounters(kinds=HERO_KINDS)
    b.reset()
    return b


def network(b: BattleBatch) -> Mlp:
    torch.manual_seed(0)
    return Mlp.for_batch(b, hidden=32, embed=8).eval()


def test_a_world_shows_its_side_the_battle_and_holds_its_own_stream() -> None:
    b = batch()
    for e in range(b.envs):
        side = int(b.to_move[e])
        world = b.determinize(e, side, 5)
        assert np.array_equal(world.observe(side), b.obs[e])
        assert world.state_hash() == b.determinize(e, side, 5).state_hash()
        assert world.state_hash() != b.determinize(e, side, 6).state_hash()
        before = world.state_hash()
        world.reseed(9)
        assert world.state_hash() != before
        world.close()
    with pytest.raises(H3Error, match="no world"):
        b.determinize(b.envs, 0, 5)
    b.close()


def test_sequential_halving_halves_the_considered_actions() -> None:
    # Each phase gives every considered action as many visits, then keeps the better half.
    assert considered_visits(4, 8) == [0, 0, 0, 0, 1, 1, 2, 2]
    assert considered_visits(16, 16) == [0] * 16
    assert considered_visits(1, 3) == [0, 1, 2]
    assert considered_visits(2, 5) == [0, 0, 1, 1, 2]


def test_the_improved_policy_follows_the_completed_values() -> None:
    actions = np.zeros((3, 4), np.uint32)
    node = Node(0, actions, np.log(np.array([0.5, 0.3, 0.2])), value=0.0)
    legal = np.ones(3, bool)
    # Unvisited, the completed values are the node's value: the prior's best first.
    assert np.allclose(node.completed(legal), 0.0) and node.select(legal) == 0
    node.backup(0, -1.0, 0)
    node.backup(1, 1.0, 0)
    # A visited action's mean; the others the mixed value; a value for the other side counts
    # against this node's.
    q = node.completed(legal)
    assert q[0] == -1.0 and q[1] == 1.0 and -1.0 < q[2] < 1.0
    assert node.select(legal) == 1
    node.backup(1, 1.0, 1)
    assert node.total[1] == 0.0 and node.visits[1] == 2
    # An action illegal in the simulation's world is never chosen.
    assert node.select(np.array([True, False, True])) != 1


def test_small_value_differences_decide_once_rescaled() -> None:
    # A battle's values differ by hundredths between actions: rescaled by their least and
    # greatest, the best one still outweighs a prior that prefers another (E06-log T8c).
    actions = np.zeros((3, 4), np.uint32)
    node = Node(0, actions, np.log(np.array([0.5, 0.3, 0.2])), value=0.0)
    legal = np.ones(3, bool)
    for i, value in enumerate((0.0, 0.01, 0.0)):
        node.backup(i, value, 0)
    sigma = node.sigma(legal)
    assert np.allclose(sigma, [0.0, (50 + 1) * 0.1, 0.0])
    assert node.select(legal) == 1
    # Equal values add nothing; the illegal candidates do not set the scale.
    flat = Node(0, actions, np.log(np.array([0.5, 0.3, 0.2])), value=0.0)
    for i in range(3):
        flat.backup(i, 0.2, 0)
    assert np.allclose(flat.sigma(legal), 0.0) and flat.select(legal) == 0
    flat.backup(2, -5.0, 0)
    assert np.allclose(flat.sigma(np.array([True, True, False]))[:2], 0.0)


def test_a_search_chooses_legal_actions_and_a_seed_gives_the_same() -> None:
    choices = []
    for _ in range(2):
        b = batch()
        make = search(network(b), simulations=6, worlds=3, k=4, limit=10)
        agent = make(11)
        seen = []
        for _ in range(4):
            agent(b)
            seen.append(b.actions.copy())
            b.step()
            assert (b.status == _ffi.OK).all()
        stats = make.searchers[-1].stats
        assert stats["decisions"] == 4 * b.envs and stats["simulations"] > 0
        assert stats["evaluations"] >= stats["decisions"]
        choices.append(seen)
        b.close()
    assert all(np.array_equal(a, b) for a, b in zip(*choices, strict=True))


def test_a_search_agent_plays_its_battles_to_the_end() -> None:
    b = batch()
    net = network(b)
    b.close()
    agent = search(net, simulations=4, worlds=2, k=4)(3)
    result = play(agent, episodes=2, envs=2, kinds=HERO_KINDS, max_steps=600)
    assert result["attacker"]["episodes"] + result["attacker"]["skipped"] == 2
    assert result["defender"]["episodes"] + result["defender"]["skipped"] == 2


def test_the_search_refuses_a_preview_network_and_empty_budgets() -> None:
    b = BattleBatch(2, preview=True)
    preview = Mlp.for_batch(b, hidden=8, embed=4, preview=True)
    b.close()
    rng = np.random.default_rng(0)
    with pytest.raises(ValueError, match="kill preview"):
        Searcher(preview, 4, 2, 4, rng)
    with pytest.raises(ValueError, match="positive"):
        Searcher(Mlp.for_batch(BattleBatch(2), hidden=8, embed=4), 0, 2, 4, rng)


def test_the_report_plays_the_search_and_the_sampled_policy_on_the_same_episodes(tmp_path) -> None:
    # E06 T8c's measurement (`--report`): a saved network (as `best.pt` holds one, `t7-1`'s
    # settings: an MLP without the turn order, which reads v21's prefix) searched and sampled.
    import json
    from dataclasses import asdict

    from h3env.search import main
    from h3env.train import Config
    from h3env.train.policy import build

    config = asdict(Config(hidden=16, embed=4))
    b = BattleBatch(1)
    torch.manual_seed(0)
    net = build(config, b)
    b.close()
    best = tmp_path / "best.pt"
    torch.save({"config": config, "policy": net.state_dict()}, best)
    out = tmp_path / "search.json"
    args = ["--report", "--best", str(best), "--per-kind", "1", "--simulations", "3"]
    result = main([*args, "--worlds", "2", "--k", "2", "--json", str(out)])
    assert json.loads(out.read_text())["settings"]["simulations"] == 3
    search_, sampled_ = result["search"], result["sampled"]
    assert search_["games"] == sampled_["games"] > 0
    assert set(search_["kinds"]) == set(sampled_["kinds"])
    costs = search_["costs"]
    assert costs["decisions"] > 0 and costs["simulations"] > 0 and costs["fallbacks"] >= 0
    with pytest.raises(SystemExit):
        main(["--report", "--per-kind", "1"])
