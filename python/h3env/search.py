"""Decision-time search over determinized worlds (E06 T8, `docs/epics/E06-design.md` §14; ADR-0025
point 12a): Gumbel AlphaZero at the root and the completed-Q improved policy below it (Danihelka
et al., "Policy improvement by planning with Gumbel", ICLR 2022), with a trained network's heads
as the prior and its value at a leaf.

At each of its decisions the agent builds W *worlds* of the battle from its side's knowledge
(`BattleBatch.determinize`: the other side's hero guessed, its hidden obstacles and a drawn
spell drawn again, the stream re-seeded) and runs N simulations over them. The tree is open-loop:
keyed by action histories, each simulation takes a world (round robin), forks it with a fresh
stream from the agent's own and replays its path; a node keeps its first evaluation's prior and
selects among the actions legal in the simulation's world. Inside the tree the network plays both
sides, each from its own observation of the world. Values are the side to move's: reward v1 at a
terminal, the value head at a leaf.

The root takes the Gumbel-top-k of the legal entries (a cast's heads by their Gumbel-perturbed
argmax in order) and sequential halving over them; a node below it every entry legal at its first
evaluation (a cast's heads the network's most likely there). Every env the agent moves is
searched at once: a simulation round evaluates one leaf per env in one network call. A chosen
action the true battle refuses (a cast whose target the world's guess made legal) gives way to
the network's own most likely action.

`python -m h3env.search --time` times a decision; `--report --best <best.pt>` plays the search
and the sampled policy on the same held-out episodes (E06 T8c).
"""

from __future__ import annotations

import argparse
import math
import time
from dataclasses import dataclass, field

import numpy as np
import torch

from . import _ffi
from ._ffi import H3Error
from .batch import BattleBatch
from .tools import Env
from .train.policy import CASTS, Policy, masked

#: The completed-Q transform's constants (the paper's `c_visit`, `c_scale`).
C_VISIT = 50.0
C_SCALE = 0.1


def considered_visits(k: int, simulations: int) -> list[int]:
    """Sequential halving's schedule (the paper's, as `mctx` builds it): for each simulation the
    visit count the root's next action has among the `k` considered, halving them each phase."""
    if k <= 1:
        return list(range(simulations))
    phases = math.ceil(math.log2(k))
    sequence: list[int] = []
    visits = [0] * k
    considered = k
    while len(sequence) < simulations:
        extra = max(1, simulations // (phases * considered))
        for _ in range(extra):
            sequence.extend(visits[:considered])
            for i in range(considered):
                visits[i] += 1
        considered = max(2, considered // 2)
    return sequence[:simulations]


def scaled(q: np.ndarray) -> np.ndarray:
    """A value of reward v1 (zero-sum, about −1 to 1) on [0, 1]."""
    return np.clip((q + 1.0) / 2.0, 0.0, 1.0)


def softmax(x: np.ndarray) -> np.ndarray:
    z = np.exp(x - x.max())
    return z / z.sum()


@dataclass
class Node:
    """A node of the open-loop tree: the side to move, its candidate actions (`[m, 4]`: entry,
    spell, target, second) with the prior's log-probabilities, the network's value for that side
    and each action's visits, summed values (that side's) and child."""

    side: int
    actions: np.ndarray
    logits: np.ndarray
    value: float
    visits: np.ndarray = field(init=False)
    total: np.ndarray = field(init=False)
    children: list = field(init=False)

    def __post_init__(self) -> None:
        m = len(self.actions)
        self.visits = np.zeros(m, np.int64)
        self.total = np.zeros(m, np.float64)
        self.children = [None] * m

    def completed(self, legal: np.ndarray) -> np.ndarray:
        """The completed Q-values over the `legal` candidates: a visited action's mean, the
        others the mixed value (the paper's `v_mix`)."""
        n = self.visits
        q = self.total / np.maximum(n, 1)
        visited = (n > 0) & legal
        pi = softmax(np.where(legal, self.logits, -np.inf))
        mixed = self.value
        if visited.any():
            weight = pi[visited].sum()
            mixed = (self.value + n[legal].sum() / weight * (pi[visited] * q[visited]).sum()) / (
                1 + n[legal].sum()
            )
        return np.where(visited, q, mixed)

    def sigma(self, legal: np.ndarray) -> np.ndarray:
        """The completed-Q transform `σ(q̂)` over the `legal` candidates."""
        top = self.visits[legal].max() if legal.any() else 0
        return (C_VISIT + top) * C_SCALE * scaled(self.completed(legal))

    def select(self, legal: np.ndarray) -> int:
        """Below the root: the deterministic selection of the completed-Q improved policy among
        the `legal` candidates (the paper's eq. 14)."""
        logits = np.where(legal, self.logits, -np.inf)
        improved = softmax(logits + self.sigma(legal))
        n = self.visits
        score = improved - n / (1 + n[legal].sum())
        return int(np.argmax(np.where(legal, score, -np.inf)))

    def backup(self, i: int, value: float, side: int) -> None:
        """Adds a simulation's `value` (for `side`) to candidate `i`."""
        self.visits[i] += 1
        self.total[i] += value if side == self.side else -value


@dataclass
class Leaf:
    """A position to evaluate: its world, the side to move, the path to it, whether it is a
    search's root, and whether it becomes a node (else only its value is backed up)."""

    world: Env
    side: int
    path: list
    search: Search
    root: bool = False
    store: bool = True


@dataclass
class Search:
    """One env's decision: its worlds, side, root, the root's Gumbels and schedule."""

    env: int
    side: int
    worlds: list
    root: Node | None = None
    gumbel: np.ndarray | None = None
    sequence: list = field(default_factory=list)

    def root_action(self, simulation: int) -> int:
        """The root's candidate for `simulation`: among those of the scheduled visit count, the
        largest `g + logits + σ(q̂)`."""
        root = self.root
        legal = np.ones(len(root.actions), bool)
        target = self.sequence[simulation]
        score = self.gumbel + root.logits + root.sigma(legal)
        return int(np.argmax(np.where(root.visits == target, score, -np.inf)))

    def choice(self) -> np.ndarray:
        """The root's choice: among the most visited, the largest `g + logits + σ(q̂)`."""
        root = self.root
        legal = np.ones(len(root.actions), bool)
        score = self.gumbel + root.logits + root.sigma(legal)
        top = root.visits.max()
        return root.actions[int(np.argmax(np.where(root.visits == top, score, -np.inf)))]


class Searcher:
    """The search with `policy` (no kill preview: a single env writes none), `simulations` per
    decision over `worlds` worlds, `k` actions considered at the root; its draws from `rng`."""

    def __init__(
        self, policy: Policy, simulations: int, worlds: int, k: int, rng: np.random.Generator
    ) -> None:
        if policy.reads_preview:
            raise ValueError("the search's network reads no kill preview")
        if min(simulations, worlds, k) < 1:
            raise ValueError("simulations, worlds and k are positive")
        self.policy = policy
        self.simulations = simulations
        self.n_worlds = worlds
        self.k = k
        self.rng = rng
        self.device = next(policy.parameters()).device
        self.stats = {"decisions": 0, "simulations": 0, "evaluations": 0, "fallbacks": 0}

    def seed(self) -> int:
        return int(self.rng.integers(0, 2**63))

    @torch.no_grad()
    def decide(self, batch: BattleBatch, rows: np.ndarray) -> np.ndarray:
        """The actions (`_ffi.ACTION_DTYPE`, one per row of `rows`) the search chooses for the
        envs `rows` of `batch`, each for its side to move."""
        searches = []
        for e in rows:
            side = int(batch.to_move[e])
            worlds = [batch.determinize(int(e), side, self.seed()) for _ in range(self.n_worlds)]
            searches.append(Search(int(e), side, worlds))
        try:
            self.expand([Leaf(s.worlds[0], s.side, [], s, root=True) for s in searches])
            for s in searches:
                self.plant(s, batch.entry_mask[s.env].astype(bool))
            for simulation in range(self.simulations):
                leaves = []
                for s in searches:
                    if simulation < len(s.sequence):
                        leaf = self.simulate(s, simulation)
                        if leaf is not None:
                            leaves.append(leaf)
                self.expand(leaves)
            out = np.zeros(len(rows), _ffi.ACTION_DTYPE)
            for i, s in enumerate(searches):
                out[i] = tuple(int(x) for x in s.choice())
            self.stats["decisions"] += len(rows)
        finally:
            for s in searches:
                for w in s.worlds:
                    w.close()
        return out

    def plant(self, s: Search, legal: np.ndarray) -> None:
        """The root's Gumbel-top-k candidates among the entries the true battle allows (`legal`;
        the world's may differ, BAT-Q26) and their schedule (a single one needs no search)."""
        root = s.root
        allowed = legal[root.actions[:, 0]]
        if allowed.any():
            root.actions, root.logits = root.actions[allowed], root.logits[allowed]
            root.logits = root.logits - np.logaddexp.reduce(root.logits)
        m = len(root.actions)
        g = self.rng.gumbel(size=m)
        k = min(self.k, m)
        keep = np.argsort(-(g + root.logits), kind="stable")[:k]
        root.actions, root.logits = root.actions[keep], root.logits[keep]
        root.visits, root.total = root.visits[keep], root.total[keep]
        root.children = [None] * k
        s.gumbel = g[keep]
        s.sequence = considered_visits(k, self.simulations) if k > 1 else []

    def simulate(self, s: Search, simulation: int) -> Leaf | None:
        """One simulation of search `s`: a fork of its next world with a fresh stream, its path
        stepped from the root; the leaf to evaluate, or `None` once its value is backed up (a
        terminal, or an action the world refuses)."""
        self.stats["simulations"] += 1
        world = s.worlds[simulation % len(s.worlds)].fork()
        world.reseed(self.seed())
        node, i = s.root, s.root_action(simulation)
        path = [(node, i)]
        side = s.side
        while True:
            entry, spell, target, second = (int(x) for x in node.actions[i])
            try:
                result = world.step(node.side, entry, spell, target, second)
            except H3Error:
                # Refused in this world (a cast's heads another world made legal): the node's
                # own value, so the schedule moves on.
                world.close()
                self.back(path, node.value, node.side)
                return None
            side = world.to_move()
            if result.done or side < 0:
                rewards = (result.reward_attacker, result.reward_defender)
                world.close()
                for n, j in path:
                    n.backup(j, rewards[n.side], n.side)
                return None
            child = node.children[i]
            if child is None:
                return Leaf(world, side, path, s)
            if child.side != side:
                # Another side moves here in this world (good morale, a stack that died): its
                # value alone, the tree's node stays the first world's.
                return Leaf(world, side, path, s, store=False)
            legal = world.legal_mask(side)[child.actions[:, 0]].astype(bool)
            if not legal.any():
                world.close()
                self.back(path, child.value, child.side)
                return None
            node, i = child, child.select(legal)
            path.append((node, i))

    @staticmethod
    def back(path: list, value: float, side: int) -> None:
        for n, j in path:
            n.backup(j, value, side)

    def expand(self, leaves: list[Leaf]) -> None:
        """Evaluates `leaves` in one network call (and, for their legal casts, one per head) and
        backs each one's value up its path."""
        if not leaves:
            return
        self.stats["evaluations"] += len(leaves)
        policy = self.policy
        obs = np.stack([leaf.world.observe(leaf.side) for leaf in leaves])
        masks = np.stack([leaf.world.legal_mask(leaf.side) for leaf in leaves])
        h = policy.encode(torch.from_numpy(obs).to(self.device))
        values = policy.value(h).cpu().numpy().astype(np.float64)
        legal = torch.from_numpy(masks).to(self.device).bool()
        logp = torch.log_softmax(masked(policy.entry_logits(h), legal), dim=1).cpu().numpy()
        heads = self.cast_heads(leaves, h, masks)
        for r, leaf in enumerate(leaves):
            entries = np.nonzero(masks[r])[0].astype(np.uint32)
            actions = np.zeros((len(entries), 4), np.uint32)
            actions[:, 0] = entries
            for c, cast in enumerate(CASTS):
                at = np.nonzero(entries == cast)[0]
                if len(at):
                    actions[at[0], 1:] = heads[c][r]
            node = Node(leaf.side, actions, logp[r, entries].astype(np.float64), float(values[r]))
            if leaf.root:
                leaf.search.root = node
                continue
            if leaf.store:
                parent, i = leaf.path[-1]
                parent.children[i] = node
            self.back(leaf.path, node.value, node.side)
            leaf.world.close()

    def cast_heads(self, leaves: list[Leaf], h, masks: np.ndarray) -> list[np.ndarray]:
        """Per cast entry (hero, creature), each leaf's most likely heads where it is legal
        (`[rows, 3]`: spell, target, second; zeros elsewhere)."""
        out = []
        for cast in CASTS:
            heads = np.zeros((len(leaves), 3), np.uint32)
            rows = np.nonzero(masks[:, cast])[0]
            if len(rows):
                heads[rows] = self.heads_of([leaves[r] for r in rows], h, rows, int(cast))
            out.append(heads)
        return out

    def heads_of(self, leaves: list[Leaf], h, rows: np.ndarray, cast: int) -> np.ndarray:
        """`leaves`' (the rows `rows` of `h`) most likely heads of cast entry `cast`, each head
        masked by its world's head call."""
        policy, device = self.policy, self.device
        hr = tuple(t[torch.from_numpy(rows).to(device)] for t in h)
        one_hot = Policy.cast_of(torch.full((len(rows),), cast, device=device))

        def best(logits: torch.Tensor, masks: list[np.ndarray]) -> torch.Tensor:
            m = torch.from_numpy(np.stack(masks)).to(device).bool()
            return masked(logits, m).argmax(dim=1)

        spell = best(
            policy.spell_logits(hr, one_hot),
            [leaf.world.spell_mask(leaf.side, cast) for leaf in leaves],
        )
        s = spell.cpu().numpy()
        target = best(
            policy.target_logits(hr, one_hot, spell),
            [leaf.world.target_mask(leaf.side, cast, int(s[j])) for j, leaf in enumerate(leaves)],
        )
        t = target.cpu().numpy()
        second = best(
            policy.second_logits(hr, one_hot, spell, target),
            [
                leaf.world.second_mask(leaf.side, cast, int(s[j]), int(t[j]))
                for j, leaf in enumerate(leaves)
            ],
        )
        return np.stack([s, t, second.cpu().numpy()], axis=1).astype(np.uint32)


def legal_in(batch: BattleBatch, rows: np.ndarray, actions: np.ndarray) -> np.ndarray:
    """Which of `actions` (for `rows`) the true battles accept: the entry legal and, for a cast,
    each head (the batch's head calls)."""
    ok = batch.entry_mask[rows, actions["entry"]].astype(bool)
    cast = np.isin(actions["entry"], CASTS)
    if cast.any():
        entries = np.zeros(batch.envs, np.uint32)
        spells = np.zeros(batch.envs, np.uint16)
        targets = np.zeros(batch.envs, np.uint16)
        entries[rows] = actions["entry"]
        spells[rows] = actions["spell"]
        targets[rows] = actions["target"]
        spell_ok = batch.spell_mask(entries)[rows, actions["spell"]].astype(bool)
        target_ok = batch.target_mask(entries, spells)[rows, actions["target"]].astype(bool)
        second_ok = batch.second_mask(entries, spells, targets)[rows, actions["second"]]
        ok &= ~cast | (spell_ok & target_ok & second_ok.astype(bool))
    return ok


def search(policy: Policy, simulations: int = 32, worlds: int = 8, k: int = 16, limit: int = 1):
    """The search as `evaluate.report`'s agents (`make_agent(seed, side)`, as `evaluate.sampled`):
    each searches the envs it moves that have played fewer than `limit` episodes (`report`'s
    battles: one each), the others take a random legal action (their battles count for nothing);
    its draws (Gumbels, worlds, streams) from a generator seeded `seed`. `make.searchers` keeps
    each agent's `Searcher` (its `stats`)."""
    searchers = []

    def make(seed: int, side: str = "attacker"):
        rng = np.random.default_rng(seed)
        searcher = Searcher(policy, simulations, worlds, k, rng)
        searchers.append(searcher)

        def agent(batch: BattleBatch) -> None:
            batch.random_actions(rng)
            rows = np.nonzero((batch.to_move >= 0) & (batch.episode < limit))[0]
            if not len(rows):
                return
            chosen = searcher.decide(batch, rows)
            ok = legal_in(batch, rows, chosen)
            if not ok.all():
                searcher.stats["fallbacks"] += int((~ok).sum())
                bad = rows[~ok]
                generator = torch.Generator(device=searcher.device).manual_seed(seed)
                own = policy.act(batch, generator, greedy=True, rows=bad)
                chosen["entry"][~ok] = own.entry
                chosen["spell"][~ok] = own.spell
                chosen["target"][~ok] = own.target
                chosen["second"][~ok] = own.second
            batch.actions[rows] = chosen

        return agent

    make.searchers = searchers
    return make


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="python -m h3env.search",
        description="Time a search decision (--time) or measure the search against the sampled "
        "policy on a held-out set (--report, E06 T8c).",
    )
    mode = p.add_mutually_exclusive_group(required=True)
    mode.add_argument("--time", action="store_true", help="time decisions of a batch")
    mode.add_argument("--report", action="store_true", help="the search and the sampled policy")
    p.add_argument("--best", help="a saved network (best.pt; --report needs one)")
    p.add_argument("--device", default="cpu", help="the network's device (cuda on a GPU)")
    p.add_argument("--simulations", type=int, default=16)
    p.add_argument("--worlds", type=int, default=8)
    p.add_argument("--k", type=int, default=16, help="actions considered at the root")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--envs", type=int, default=16, help="--time: the batch's envs")
    p.add_argument("--decisions", type=int, default=2, help="--time: batch calls to time")
    p.add_argument("--opponent", default="system", help="--report: an engine seat")
    p.add_argument("--set", default="report-v1", help="--report: report-v1 or neutral-v1")
    p.add_argument("--per-kind", type=int, default=500, help="--report: episodes per kind")
    p.add_argument("--no-sampled", action="store_true", help="--report: the search alone")
    p.add_argument("--json", help="--report: write the results here")
    return p


def network(args) -> Policy:
    """The network `--best` names (on `--device`), else an untrained MLP."""
    if args.best:
        from .train.evaluate import load

        return load(args.best, device=args.device).eval()
    from .train.policy import Mlp

    probe = BattleBatch(1)
    torch.manual_seed(args.seed)
    policy = Mlp.for_batch(probe).to(args.device).eval()
    probe.close()
    return policy


def time_decisions(args) -> dict:
    """`--time`: `--decisions` calls of a batch of `--envs` encounters with a hero to guess, every
    env searched; the cost per decision and per simulation."""
    policy = network(args)
    batch = BattleBatch(args.envs, seed=args.seed, max_steps=2000)
    batch.add_encounters(kinds=[0, 0, 2, 1, 1, 0])
    batch.reset()
    make = search(policy, args.simulations, args.worlds, args.k, limit=10**9)
    agent = make(args.seed)
    start = time.perf_counter()
    for _ in range(args.decisions):
        agent(batch)
        batch.step()
    seconds = time.perf_counter() - start
    batch.close()
    stats = dict(make.searchers[-1].stats)
    out = {
        **stats,
        "seconds": seconds,
        "ms_per_decision": 1000 * seconds / max(stats["decisions"], 1),
        "us_per_simulation": 1e6 * seconds / max(stats["simulations"], 1),
    }
    print(
        f"{stats['decisions']} decisions, {stats['simulations']} simulations, "
        f"{stats['evaluations']} evaluations in {seconds:.2f} s: "
        f"{out['ms_per_decision']:.1f} ms a decision, {out['us_per_simulation']:.0f} µs a "
        "simulation"
    )
    return out


def measure(args) -> dict:
    """`--report` (E06 T8c): the search with `--best` and the sampled policy (unless
    `--no-sampled`) on the same episodes of a held-out set (`evaluate.report`), against
    `--opponent`; their rates, values, intervals, seconds and the search's costs."""
    from .train.evaluate import report, sampled

    if not args.best:
        raise SystemExit("--report needs --best (a saved network)")
    policy = network(args)
    settings = dict(opponent=args.opponent, per_kind=args.per_kind, held_out=args.set)
    make = search(policy, args.simulations, args.worlds, args.k)
    out = {
        "settings": {
            "best": args.best,
            "simulations": args.simulations,
            "worlds": args.worlds,
            "k": args.k,
            **settings,
        },
        "search": report(make, **settings),
    }
    stats = {key: sum(s.stats[key] for s in make.searchers) for key in make.searchers[0].stats}
    out["search"]["costs"] = {
        **stats,
        "ms_per_decision": 1000 * out["search"]["seconds"] / max(stats["decisions"], 1),
    }
    if not args.no_sampled:
        out["sampled"] = report(sampled(policy), **settings)
    for name in ("search", "sampled"):
        if name in out:
            r = out[name]
            print(
                f"{name}: win rate {r['win_rate']:.3f} ± {r['interval']:.3f}, value "
                f"{r['value']:+.3f}, {r['games']} games in {r['seconds']:.0f} s"
            )
    if args.json:
        import json
        from pathlib import Path

        Path(args.json).write_text(json.dumps(out, indent=1) + "\n")
    return out


def main(argv: list[str] | None = None) -> dict:
    args = parser().parse_args(argv)
    return time_decisions(args) if args.time else measure(args)


if __name__ == "__main__":
    main()
