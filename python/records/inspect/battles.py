"""The battles one by one (E25 T2b, a one-off script, not a module): every counted battle of a
network on the report set v1 against the given seats, as `h3env.train.inspect` records it, one
JSON line per battle (opponent, kind, side, outcome and the battle's counters: decisions per
category, rounds, truncation, reward), then the tactics phase's decisions per battle. From
`python/`:

    uv run --group train python records/inspect/battles.py runs/<run>/best.pt <out.jsonl> \
        system,greedy,aggressor
    uv run python records/inspect/battles.py --summary <out.jsonl> ...

The draws are the device's (the CPU's here; `inspect`'s note). The findings in
`docs/epics/E25-log.md` T2b.
"""

import json
import os
import sys
from collections import defaultdict


def record(checkpoint: str, out: str, opponents: list[str]) -> None:
    from h3env.train.evaluate import load, report, sampled
    from h3env.train.inspect import Recorder

    policy = load(checkpoint, "cpu")
    policy.eval()
    make_agent = sampled(policy)
    with open(out, "w", encoding="utf-8") as f:
        for opponent in opponents:
            recorders: list[Recorder] = []

            def make(seed: int, side: str, recorders: list[Recorder] = recorders) -> Recorder:
                recorders.append(Recorder(make_agent(seed, side), side))
                return recorders[-1]

            row = report(make, opponent=opponent, threads=os.cpu_count() or 1, preview=True)
            for r in recorders:
                r.finish()
                side = ("attacker", "defender")[r.side]
                for kind, won, stats, _ in r.battles:
                    line = {"opponent": opponent, "kind": kind, "side": side, "won": won}
                    line.update({k: v for k, v in stats.items() if isinstance(v, (int, float))})
                    f.write(json.dumps(line) + "\n")
            print(opponent, row["win_rate"], row["value"], row["truncated"], flush=True)


def _pct(xs: list[int], q: float) -> int:
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))] if xs else 0


def summary(path: str) -> None:
    """Per opponent: the battles with a tactics phase for the agent, its decisions there (mean,
    percentiles, how many above 50 and 200, how many cut at the step limit) and their share of
    all the agent's decisions; the battles of 50 rounds or more."""
    by = defaultdict(list)
    for line in open(path, encoding="utf-8"):
        r = json.loads(line)
        by[r["opponent"]].append(r)
    print("==", path)
    for opponent, rows in by.items():
        t = [r.get("tactics", 0) for r in rows if r.get("tactics", 0) > 0]
        decisions = sum(r.get("decisions", 0) for r in rows)
        print(
            f" {opponent:9} with tactics {len(t)} of {len(rows)}: mean"
            f" {sum(t) / max(len(t), 1):.1f} p50 {_pct(t, 0.5)} p90 {_pct(t, 0.9)} p99"
            f" {_pct(t, 0.99)} | >50"
            f" {sum(x > 50 for x in t)} >200 {sum(x > 200 for x in t)} cut"
            f" {sum(x >= 2000 for x in t)} | share {100 * sum(t) / max(decisions, 1):.1f} % |"
            f" 50 rounds or more {sum(r.get('rounds', 0) >= 50 for r in rows)}"
        )


if __name__ == "__main__":
    if sys.argv[1] == "--summary":
        for p in sys.argv[2:]:
            summary(p)
    else:
        record(sys.argv[1], sys.argv[2], sys.argv[3].split(","))
