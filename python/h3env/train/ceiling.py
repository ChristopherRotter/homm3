"""The teacher's ceiling (E25 T19, increment 6, AI-Q15; `docs/epics/E25-log.md`): how often the
fair system actor repeats its own choice on the same state, the bound on any clone's agreement
with it (E25 decision 2 of increment 6).

    uv run --group train python -m h3env.train.ceiling --config configs/distill/e25-clone.toml \\
        --draws 8 --decisions 25600 --network runs/e25-clone-1/best.pt \\
        --json records/distill/ceiling-1.json

The states are the cloning bench's held-out set's (`h3env.train.clone`): the bench's `run`
config's training distribution on the seed after the bench's `seed`, every external seat the
fair actor (`ppo.TEACHER`). At each state its choice is drawn `draws` times
(`BattleBatch.choice` draws from the side's seat stream and advances it, so each call on the
unchanged state is a fresh draw) and the last draw is played, so the episodes are the fair
actor's own (not the bench's: its streams advance once a decision). Every state of an ended
episode counts until `decisions` are scored.

For each state and each draw j (the "teacher's" choice, as one draw is in the bench), by the
category of draw j's entry (`distill.CATEGORIES`):

- `mode`: whether the most frequent entry of the other draws is draw j's (a tie credits each
  tied entry its share): a clone that always plays the teacher's most likely entry scores this,
  estimated from `draws - 1` draws (a little below the exact mode's);
- `pair`: the share of the other draws equal to draw j: the agreement of a clone that samples the
  teacher's own distribution;
- `entropy`: the entropy of the state's `draws` entries, plug-in and Miller-Madow corrected (the
  cross-entropy floor of any clone of the entry head, in nats);
- for a cast draw, the cast heads' `mode` along the teacher's choice: the spell among the other
  draws of the same entry, the target among those of the same spell, the second among those of the
  same target (a state without such a draw is not counted for that head).

Each `--network` (a run's network file or a bench's student) is scored on the same states and
draws: its most likely legal entry against draw j (`agreement`, the bench's measure) and its
cross-entropy on draw j's entry. The report is printed and, with `--json`, written as a record.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import math
import time
from collections import Counter
from collections.abc import Sequence
from pathlib import Path
from typing import Any

import numpy as np
import torch

from .. import _ffi
from .clone import Episodes
from .distill import CATEGORIES, Bench, category, load_bench
from .evaluate import load
from .policy import CASTS, masked
from .ppo import TEACHER
from .run import ConfigError, atomic_write_text, load_config, machine, source

#: The cast heads, in the order the teacher's choice fixes them.
HEADS = ("spell", "target", "second")


def mode_share(others: Sequence[Any], mine: Any) -> float:
    """The share of the most frequent values of `others` that `mine` is: 1 if it is the only
    most frequent one, 1/n if it is one of n tied, else 0 (`others` not empty)."""
    counts = Counter(others)
    top = max(counts.values())
    tied = sum(1 for c in counts.values() if c == top)
    return 1.0 / tied if counts.get(mine, 0) == top else 0.0


def entropies(values: Sequence[Any]) -> tuple[float, float]:
    """The plug-in entropy of `values` (nats) and its Miller-Madow correction."""
    n = len(values)
    counts = Counter(values)
    h = -sum(c / n * math.log(c / n) for c in counts.values())
    return h, h + (len(counts) - 1) / (2 * n)


class Sums:
    """The ceiling's sums by category (the module's documentation)."""

    def __init__(self, networks: Sequence[str]) -> None:
        k = len(CATEGORIES)
        self.count = np.zeros(k)
        self.mode = np.zeros(k)
        self.pair = np.zeros(k)
        self.entropy = np.zeros(k)
        self.entropy_mm = np.zeros(k)
        self.distinct = np.zeros(k)
        self.heads = {h: [0.0, 0] for h in HEADS}
        self.casts = 0
        self.agree = {n: np.zeros(k) for n in networks}
        self.ce = {n: np.zeros(k) for n in networks}

    def state(self, draws: np.ndarray, cats: np.ndarray) -> None:
        """Adds one state's draws (`draws`: the k actions' fields, `cats`: each draw's category)."""
        entries = draws["entry"].tolist()
        k = len(entries)
        h, h_mm = entropies(entries)
        distinct = len(set(entries))
        for j, (entry, c) in enumerate(zip(entries, cats.tolist(), strict=True)):
            others = entries[:j] + entries[j + 1 :]
            self.count[c] += 1
            self.mode[c] += mode_share(others, entry)
            self.pair[c] += others.count(entry) / (k - 1)
            self.entropy[c] += h
            self.entropy_mm[c] += h_mm
            self.distinct[c] += distinct
            if entry not in CASTS:
                continue
            self.casts += 1
            same = [i for i in range(k) if i != j and entries[i] == entry]
            for head in HEADS:
                if not same:
                    break
                mine = int(draws[head][j])
                values = [int(draws[head][i]) for i in same]
                row = self.heads[head]
                row[0] += mode_share(values, mine)
                row[1] += 1
                same = [i for i, v in zip(same, values, strict=True) if v == mine]

    def report(self) -> dict[str, Any]:
        """The sums as shares and means, overall and by category."""
        n = self.count

        def split(x: np.ndarray) -> dict[str, Any]:
            return {
                "all": float(x.sum() / max(n.sum(), 1)),
                "by_category": {
                    c: (float(x[i] / n[i]) if n[i] else None) for i, c in enumerate(CATEGORIES)
                },
            }

        return {
            "draws_scored": int(n.sum()),
            "categories": dict(zip(CATEGORIES, n.astype(int).tolist(), strict=True)),
            "mode": split(self.mode),
            "pair": split(self.pair),
            "entropy": split(self.entropy),
            "entropy_miller_madow": split(self.entropy_mm),
            "distinct_entries": split(self.distinct),
            "cast_draws": self.casts,
            "cast_heads": {
                h: {"mode": (s / c if c else None), "draws": c} for h, (s, c) in self.heads.items()
            },
            "networks": {
                name: {"agreement": split(self.agree[name]), "cross_entropy": split(self.ce[name])}
                for name in self.agree
            },
        }


def run(
    bench: Bench,
    draws: int,
    decisions: int,
    networks: Sequence[str] = (),
    config_text: str | None = None,
    log=print,
) -> dict[str, Any]:
    """The ceiling on `bench`'s held-out distribution (the module's documentation); `config_text`
    the run config's text, else read from the bench's `run`."""
    if draws < 2:
        raise ConfigError("the ceiling needs at least 2 draws a state")
    if not bench.run and config_text is None:
        raise ConfigError("the ceiling needs the bench's `run`, the distribution it plays")
    text = Path(bench.run).read_text(encoding="utf-8") if config_text is None else config_text
    base = dataclasses.replace(load_config(text), seed=bench.seed, system_share=0.0)
    seed = bench.seed + 1
    nets = {name: load(name) for name in networks}
    for net in nets.values():
        net.eval()
    sums = Sums(list(nets))
    ep = Episodes(base, seed, bench.envs, bench.threads, rows=1)
    b = ep.batch
    #: Each env's states of its current episode: (draws, categories, obs, entry mask).
    open_: list[list[tuple]] = [[] for _ in range(bench.envs)]
    scored, episodes, lost, start = 0, 0, 0, time.perf_counter()
    while scored < decisions:
        to_move = b.to_move.copy()
        seen = np.zeros((draws, len(to_move)), b.actions.dtype)
        ok = to_move >= 0
        for j in range(draws):
            encoded, n = b.choice(TEACHER)
            lost += n
            seen[j] = b.actions
            ok &= encoded == 1
        gone = (to_move >= 0) & ~ok
        if gone.any():
            chosen = b.actions.copy()
            b.random_actions(ep.rng)
            chosen[gone] = b.actions[gone]
            b.actions[:] = chosen
        rows = np.nonzero(ok)[0]
        if len(rows):
            obs = torch.from_numpy(b.obs[rows].copy())
            entry = seen["entry"][:, rows].astype(np.int64)
            cats = np.stack([category(obs, torch.from_numpy(e)).numpy() for e in entry])
            mask = b.entry_mask[rows].astype(bool)
            for at, env in enumerate(rows.tolist()):
                open_[env].append((seen[:, env].copy(), cats[:, at], obs[at], mask[at]))
        b.step()
        for env in np.nonzero(b.done)[0].tolist():
            states, open_[env] = open_[env], []
            episodes += 1
            ep.held_out.check(b.episode_seed[[env]])
            if scored >= decisions:
                continue
            states = states[: decisions - scored]
            for d, c, _, _ in states:
                sums.state(d, c)
            if nets:
                score_networks(sums, nets, states)
            scored += len(states)
    seconds = time.perf_counter() - start
    ep.close()
    versions = _ffi.versions()
    out = {
        "teacher": {
            "system": TEACHER,
            "system_policy": versions["system_policy"],
            "run": bench.run,
            "kinds": base.kinds,
            "seed": seed,
        },
        "draws": draws,
        "decisions": scored,
        "episodes": episodes,
        "lost": lost,
        "seconds": seconds,
        "networks": list(networks),
        "identity": {
            "started": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            **source(),
            **versions,
            "torch": torch.__version__,
            **machine(),
        },
        "ceiling": sums.report(),
    }
    log(table(out))
    return out


def score_networks(sums: Sums, nets: dict[str, Any], states: list[tuple]) -> None:
    """Each network's most likely legal entry and cross-entropy against every draw of `states`."""
    obs = torch.stack([s[2] for s in states])
    mask = torch.from_numpy(np.stack([s[3] for s in states]))
    entries = np.stack([s[0]["entry"] for s in states]).astype(np.int64)  # [states, draws]
    cats = np.stack([s[1] for s in states])
    with torch.no_grad():
        for name, net in nets.items():
            logits = net.entry_logits(net.encode(obs, mask))
            logp = torch.log_softmax(masked(logits, mask), dim=1)
            best = logp.argmax(1).numpy()
            picked = -logp.gather(1, torch.from_numpy(entries)).numpy()
            for j in range(entries.shape[1]):
                c = cats[:, j]
                sums.agree[name] += np.bincount(
                    c, weights=(best == entries[:, j]), minlength=len(CATEGORIES)
                )
                sums.ce[name] += np.bincount(c, weights=picked[:, j], minlength=len(CATEGORIES))


def table(out: dict[str, Any]) -> str:
    """The report as a short text table: a row per category, the overall row last."""
    c = out["ceiling"]
    names = list(c["networks"])
    head = f"{'category':<12}{'draws':>8}{'mode':>7}{'pair':>7}{'H':>7}{'H mm':>7}"
    head += "".join(f"{'agree ' + Path(n).parent.name:>18}{'CE':>7}" for n in names)
    lines = [
        f"the fair actor's ceiling: {out['decisions']:,} states of {out['episodes']:,} episodes, "
        f"{out['draws']} draws each, {out['seconds']:.0f} s (lost {out['lost']})",
        head,
    ]
    for cat in (*CATEGORIES, "all"):
        n = c["draws_scored"] if cat == "all" else c["categories"][cat]
        if not n:
            continue

        def v(x: dict[str, Any], cat: str = cat) -> float:
            return x["all"] if cat == "all" else x["by_category"][cat]

        row = f"{cat:<12}{n:>8}{v(c['mode']):>7.3f}{v(c['pair']):>7.3f}"
        row += f"{v(c['entropy']):>7.3f}{v(c['entropy_miller_madow']):>7.3f}"
        for name in names:
            x = c["networks"][name]
            row += f"{v(x['agreement']):>18.3f}{v(x['cross_entropy']):>7.3f}"
        lines.append(row)
    heads = ", ".join(
        f"{h} {x['mode']:.3f} ({x['draws']})"
        for h, x in c["cast_heads"].items()
        if x["mode"] is not None
    )
    lines.append(f"cast heads' mode along the teacher's choice: {heads or 'none'}")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="python -m h3env.train.ceiling", description=__doc__)
    parser.add_argument("--config", type=Path, required=True, help="the cloning bench's TOML file")
    parser.add_argument("--draws", type=int, default=8, help="the fair actor's draws a state")
    parser.add_argument("--decisions", type=int, default=25_600, help="the states scored")
    parser.add_argument("--network", action="append", default=[], help="a network to score")
    parser.add_argument("--set", action="append", default=[], metavar="KEY=VALUE")
    parser.add_argument("--json", type=Path, help="write the report here")
    args = parser.parse_args(argv)
    bench = load_bench(args.config.read_text(encoding="utf-8"), args.set)
    out = run(bench, args.draws, args.decisions, args.network)
    if args.json:
        atomic_write_text(args.json, json.dumps(out, indent=1) + "\n")


if __name__ == "__main__":
    main()
