"""Where a network and the fair system actor choose differently (E25 T37 (b), a one-off script,
not a module): the cloning bench's held-out states (as `h3env.train.ceiling` plays them: the fair
actor's own episodes on the bench's `run` config, the seed after the bench's), the fair actor's
most frequent entry of `draws` draws (the teacher's) against the network's most likely legal entry,
and for each pair of categories the features of the two choices that the fair actor's inputs
read (`docs/epics/E25-log.md` T37 (a)). From `python/`:

    uv run --group train python records/inspect/disagree.py runs/e25-structure-1/best.pt \\
        records/inspect/e25-structure-1-disagree.json --decisions 12800

The features are this script's approximations from observation v22 (no engine query): a hex's
**threat** counts the opponent's creature stacks (alive, with turns, not war machines, not
shooters that can shoot now) whose next turn reaches a hex next to the active stack standing
there (a breadth-first walk of at most its speed over hexes without an obstacle, a wall or a
stack; a flyer any free hex within its speed; a two-hex stack walks as one hex and strikes from
both), **threat value** their damage potential (count times mean damage); **adjacent** the
opponent's stacks next to it, **blocks** those of them that can shoot now (a shooter next to an
enemy cannot), **nearest** the hex distance to the opponent's nearest creature stack, **hazard**
an opponent's Fire Wall or a moat hex, **free** the free hexes around it (the tactics phase's
shooters take the hex with the fewest, BAT-AI-023). A shot's or a strike's **target** features:
its previewed kills' hit points (the batch's kill preview: the mean of the least and the
greatest kills times the creature's hit points) and its damage potential. Two tests of what
each chooser reads, against a uniform legal choice in the same states (`moves`, `strikes`): how
often a move takes the least threatened legal hex where the threat varies, and how often a strike
or a shot takes the most previewed kills or the largest potential among several legal targets.
The findings in `docs/epics/E25-log.md` T37 (b).
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import math
import time
from collections import Counter, defaultdict
from collections.abc import Sequence
from pathlib import Path

import numpy as np
import torch

from h3env import _ffi
from h3env.batch import BattleBatch
from h3env.train import layout as L
from h3env.train.distill import CATEGORIES, category, load_bench
from h3env.train.evaluate import held_out_seeds, load
from h3env.train.geometry import Field, attack_hexes, attack_table
from h3env.train.policy import masked
from h3env.train.ppo import _SEATS, TEACHER, episode_entries
from h3env.train.run import HeldOut, load_config

W, H = 17, 11
FIELD = Field(W, H)
HEXES = W * H
# Stack columns (observation v22, `observation.rs`'s order) the script reads beyond `layout`.
DMIN, DMAX, SPEED, SHOTS, FLIES = 9, 10, 11, 21, 25
FORGETFUL, NO_SHOOTING, NO_TURNS = 67, 68, 69
# Battlefield planes: natural obstacle, the opponent's Fire Wall, wall, moat.
P_BLOCKED, P_FIRE_WALL, P_WALL, P_MOAT = 0, 9, 14, 15

NEIGHBOURS = [
    [n for d in range(6) if (n := FIELD.neighbor(h, d)) is not None] for h in range(HEXES)
]
_Q = np.array([h % W - math.ceil((h // W) / 2) for h in range(HEXES)])
_R = np.array([h // W for h in range(HEXES)])
DIST = (
    np.abs(_Q[:, None] - _Q[None])
    + np.abs(_R[:, None] - _R[None])
    + np.abs((_Q[:, None] + _R[:, None]) - (_Q[None] + _R[None]))
) // 2
#: Hexes a stack may stand on: not the side columns.
STANDABLE = np.array([0 < h % W < W - 1 for h in range(HEXES)])

FEATURES = ("threat", "threat_value", "adjacent", "blocks", "nearest", "hazard", "free")
TARGET_FEATURES = ("kill_hp", "potential")


class State:
    """What the features read of one observation row."""

    def __init__(self, obs: np.ndarray) -> None:
        s = obs[L.STACKS : L.ORDER].reshape(L.STACK_CAPACITY, L.STACK_FEATURES)
        planes = obs[L.FIELD : L.STACKS].reshape(L.HEX_PLANES, L.MAX_FIELD_HEXES)[:, :HEXES]
        self.s = s
        self.attacker = obs[L.VIEWER_ATTACKER] > 0
        live = (s[:, L.PRESENT] > 0) & (s[:, L.ALIVE] > 0) & (s[:, L.TURRET] == 0)
        self.live = live
        self.pos = (s[:, L.X] + W * s[:, L.Y]).astype(int)
        self.own = s[:, L.OWN] > 0
        self.two = s[:, L.TWO_HEX] > 0
        occupied = np.zeros(HEXES, bool)
        self.cover: list[tuple[int, ...]] = []
        for i in range(L.STACK_CAPACITY):
            if not live[i] or not 0 <= self.pos[i] < HEXES:
                self.cover.append(())
                continue
            side = 0 if self.own[i] == self.attacker else 1
            hexes = FIELD.stack_footprint(int(self.pos[i]), bool(self.two[i]), side).hexes()
            self.cover.append(hexes)
            occupied[list(hexes)] = True
        blocked = (planes[P_BLOCKED] > 0) | (planes[P_WALL] > 0) | ~STANDABLE
        self.free = ~blocked & ~occupied
        self.hazard = (planes[P_FIRE_WALL] > 0) | (planes[P_MOAT] > 0)
        self.active = int(np.argmax(s[:, L.ACTIVE] > 0)) if (s[:, L.ACTIVE] > 0).any() else -1
        creature = live & (s[:, L.WAR_MACHINE] == 0)
        self.enemy = creature & ~self.own
        self.potential = s[:, L.COUNT] * (s[:, DMIN] + s[:, DMAX]) / 2
        # Can shoot now: a shooter with shots, not forgetful or prevented, no enemy next to it.
        adjacent_enemy = np.zeros(L.STACK_CAPACITY, bool)
        for i in np.nonzero(live)[0]:
            ring = self.ring(i)
            adjacent_enemy[i] = any(
                self.own[j] != self.own[i] and live[j] and set(self.cover[j]) & ring
                for j in range(L.STACK_CAPACITY)
            )
        self.shoots = (
            live
            & (s[:, L.SHOOTER] > 0)
            & (s[:, SHOTS] > 0)
            & (s[:, FORGETFUL] == 0)
            & (s[:, NO_SHOOTING] == 0)
            & ~adjacent_enemy
        )
        self._reach: dict[int, set[int]] = {}

    def ring(self, i: int, cover: Sequence[int] | None = None) -> set[int]:
        """The hexes next to stack `i`'s footprint (or `cover`), the footprint left out."""
        hexes = self.cover[i] if cover is None else cover
        return {n for h in hexes for n in NEIGHBOURS[h]} - set(hexes)

    def strike_hexes(self, i: int) -> set[int]:
        """The hexes stack `i` can strike on its next turn: next to a position it reaches."""
        if i not in self._reach:
            start = self.pos[i]
            speed = int(self.s[i, SPEED])
            if self.s[i, FLIES] > 0:
                positions = {
                    h for h in range(HEXES) if self.free[h] and DIST[start, h] <= speed
                } | {start}
            else:
                positions, frontier = {start}, [start]
                for _ in range(speed):
                    nxt = []
                    for h in frontier:
                        for n in NEIGHBOURS[h]:
                            if n not in positions and self.free[n]:
                                positions.add(n)
                                nxt.append(n)
                    frontier = nxt
            hexes = set(self.cover[i])
            for p in positions:
                hexes.add(p)
                if self.two[i]:
                    side = 0 if self.own[i] == self.attacker else 1
                    hexes.update(FIELD.stack_footprint(p, True, side).hexes())
            self._reach[i] = {n for h in hexes for n in NEIGHBOURS[h]}
        return self._reach[i]

    def threat_at(self, hex_: int) -> int:
        """The threat on the active stack standing on `hex_` (`features`' first)."""
        side = 0 if self.attacker else 1
        cover = set(FIELD.stack_footprint(hex_, bool(self.two[self.active]), side).hexes())
        return sum(
            1
            for j in np.nonzero(self.enemy)[0]
            if not self.shoots[j] and self.s[j, NO_TURNS] == 0 and cover & self.strike_hexes(j)
        )

    def features(self, hex_: int) -> dict[str, float]:
        """The active stack standing on `hex_` (its position)."""
        a = self.active
        side = 0 if self.attacker else 1
        cover = FIELD.stack_footprint(hex_, bool(self.two[a]), side).hexes()
        ring = self.ring(a, cover)
        threat, value, adjacent, blocks = 0, 0.0, 0, 0
        nearest = 99
        for j in np.nonzero(self.enemy)[0]:
            near = bool(set(self.cover[j]) & ring)
            adjacent += near
            blocks += near and bool(self.shoots[j])
            nearest = min(nearest, min(int(DIST[h, c]) for h in cover for c in self.cover[j]))
            if self.shoots[j] or self.s[j, NO_TURNS] > 0:
                continue
            if set(cover) & self.strike_hexes(j):
                threat += 1
                value += float(self.potential[j])
        free = sum(1 for n in ring if self.free[n] or n in self.cover[a])
        return {
            "threat": threat,
            "threat_value": value,
            "adjacent": adjacent,
            "blocks": blocks,
            "nearest": nearest,
            "hazard": float(any(self.hazard[h] for h in cover)),
            "free": free,
        }


def hex_of(entry: int, state: State, attack: np.ndarray) -> int | None:
    """The hex a choice leaves the active stack on: a move's, a strike's from-hex, else its own."""
    if L.MOVE_BASE <= entry < L.ATTACK_BASE:
        h = entry - L.MOVE_BASE
        return h if h < HEXES else None
    if L.ATTACK_BASE <= entry < L.SHOOT_BASE:
        slot, k = divmod(entry - L.ATTACK_BASE, L.ATTACK_SLOTS)
        h = int(attack[slot, k])
        return h if 0 <= h < HEXES else None
    p = state.pos[state.active]
    return int(p) if 0 <= p < HEXES else None


def target_of(entry: int) -> int | None:
    """A strike's or a shot's target stack slot."""
    if L.ATTACK_BASE <= entry < L.SHOOT_BASE:
        return (entry - L.ATTACK_BASE) // L.ATTACK_SLOTS
    if L.SHOOT_BASE <= entry < L.HEAL_BASE:
        return (entry - L.SHOOT_BASE) // 2
    return None


def target_features(entry: int, state: State, preview: np.ndarray) -> dict[str, float]:
    t = target_of(entry)
    k = entry - L.ATTACK_BASE
    kills = preview[k].mean() if 0 <= k < len(preview) else 0.0
    return {
        "kill_hp": float(kills * state.s[t, L.MAX_HP]),
        "potential": float(state.potential[t]),
    }


class Sums:
    """Per group, the teacher's and the network's features and the comparisons' counts."""

    def __init__(self) -> None:
        self.pairs: Counter = Counter()
        self.groups: dict[str, dict[str, list[tuple[float, float]]]] = defaultdict(
            lambda: defaultdict(list)
        )

    def add(self, group: str, mine: dict[str, float], theirs: dict[str, float]) -> None:
        for k, v in mine.items():
            self.groups[group][k].append((theirs[k], v))

    def report(self) -> dict:
        out: dict = {"pairs": {f"{a}>{b}": n for (a, b), n in sorted(self.pairs.items())}}
        groups = {}
        for g, feats in sorted(self.groups.items()):
            row = {}
            for k, xs in feats.items():
                a = np.array(xs)
                t, n = a[:, 0], a[:, 1]
                row[k] = {
                    "n": len(a),
                    "teacher": float(t.mean()),
                    "network": float(n.mean()),
                    "teacher_lower": float((t < n).mean()),
                    "teacher_higher": float((t > n).mean()),
                }
            groups[g] = row
        out["groups"] = groups
        return out


def group_of(t_cat: str, n_cat: str, t: int, n: int) -> str | None:
    """The comparison a disagreement belongs to, or None (counted only)."""
    if t_cat == n_cat == "melee":
        same_target = target_of(t) == target_of(n)
        return "melee, same target" if same_target else "melee, other target"
    if t_cat == n_cat and t_cat in ("move", "shot", "tactics"):
        return t_cat
    if {t_cat, n_cat} <= {"wait_defend", "move", "melee"}:
        return f"{t_cat} > {n_cat}"
    return None


def run(checkpoint: str, config: Path, draws: int, decisions: int, log=print) -> dict:
    bench = load_bench(config.read_text(encoding="utf-8"))
    text = Path(bench.run).read_text(encoding="utf-8")
    c = dataclasses.replace(load_config(text), seed=bench.seed, system_share=0.0)
    seed = bench.seed + 1
    net = load(checkpoint, "cpu")
    net.eval()
    b = BattleBatch(
        bench.envs,
        threads=bench.threads,
        seed=seed,
        max_steps=c.max_steps,
        currency=c.currency,
        mana_price=c.mana_price,
        preview=True,
    )
    for weight, kinds, side in episode_entries(c):
        b.add_encounters(weight, kinds=kinds, **({} if side < 0 else _SEATS[side]))
    held = HeldOut(held_out_seeds())
    b.reset()
    held.check(b.episode_seed)
    rng = np.random.default_rng(seed)
    table = torch.from_numpy(attack_table())
    sums = Sums()
    #: Per chooser, each move's (its hex's threat, the least and the mean over the legal hexes).
    moves: dict[str, list] = {"teacher": [], "network": []}
    #: Per (chooser, category), each strike's (most kills, chance, most potential, chance, targets
    #: differ) among the legal entries of its category.
    strikes: dict[tuple[str, str], list] = defaultdict(list)
    scored, agree, start, report = 0, 0, time.perf_counter(), 2000
    while scored < decisions:
        to_move = b.to_move.copy()
        seen = np.zeros((draws, len(to_move)), np.int64)
        ok = to_move >= 0
        for j in range(draws):
            encoded, _ = b.choice(TEACHER)
            seen[j] = b.actions["entry"]
            ok &= encoded == 1
        last = b.actions.copy()
        rows = np.nonzero(ok)[0]
        if len(rows):
            obs_np = b.obs[rows].copy()
            obs = torch.from_numpy(obs_np)
            mask = torch.from_numpy(b.entry_mask[rows].astype(bool))
            preview = b.preview[rows].copy()
            with torch.no_grad():
                logits = net.entry_logits(net.encode(obs, mask))
                mine = masked(logits, mask).argmax(1)
            teacher = np.array(
                [Counter(seen[:, r].tolist()).most_common(1)[0][0] for r in rows], np.int64
            )
            t_cats = category(obs, torch.from_numpy(teacher)).numpy()
            n_cats = category(obs, mine).numpy()
            mine = mine.numpy()
            attack = attack_hexes(obs, table).numpy()
            for at in range(len(rows)):
                t, n = int(teacher[at]), int(mine[at])
                tc, nc = CATEGORIES[t_cats[at]], CATEGORIES[n_cats[at]]
                scored += 1
                legal = None
                for who, e, cat in (("teacher", t, tc), ("network", n, nc)):
                    if cat != "move":
                        continue
                    if legal is None:
                        st = State(obs_np[at])
                        hexes = np.nonzero(mask[at, L.MOVE_BASE : L.MOVE_BASE + HEXES].numpy())[0]
                        legal = {int(h): st.threat_at(int(h)) for h in hexes}
                    if len(legal) > 1 and e - L.MOVE_BASE in legal:
                        moves[who].append(
                            (
                                legal[e - L.MOVE_BASE],
                                min(legal.values()),
                                np.mean(list(legal.values())),
                                np.mean([v == min(legal.values()) for v in legal.values()]),
                            )
                        )
                for who, e, cat in (("teacher", t, tc), ("network", n, nc)):
                    if cat not in ("shot", "melee"):
                        continue
                    lo, hi = (
                        (L.SHOOT_BASE, L.HEAL_BASE)
                        if cat == "shot"
                        else (L.ATTACK_BASE, L.SHOOT_BASE)
                    )
                    entries = np.nonzero(mask[at, lo:hi].numpy())[0] + lo
                    if len(entries) < 2:
                        continue
                    k = entries - L.ATTACK_BASE
                    st_ = State(obs_np[at])
                    slots = np.array([target_of(int(x)) for x in entries])
                    kill = preview[at][k].mean(1) * st_.s[slots, L.MAX_HP]
                    pot = st_.potential[slots]
                    i = int(np.nonzero(entries == e)[0][0])
                    strikes[(who, cat)].append(
                        (
                            kill[i] == kill.max(),
                            np.mean(kill == kill.max()),
                            pot[i] == pot.max(),
                            np.mean(pot == pot.max()),
                            len(set(slots.tolist())) > 1,
                        )
                    )
                if t == n:
                    agree += 1
                    continue
                sums.pairs[(tc, nc)] += 1
                g = group_of(tc, nc, t, n)
                if g is None:
                    continue
                st = State(obs_np[at])
                if st.active < 0:
                    continue
                th, nh = hex_of(t, st, attack[at]), hex_of(n, st, attack[at])
                if g == "melee, same target" and th == nh:
                    continue
                if th is not None and nh is not None and g != "shot":
                    sums.add(g, st.features(nh), st.features(th))
                if g in ("shot", "melee, other target"):
                    sums.add(
                        g + " (targets)",
                        target_features(n, st, preview[at]),
                        target_features(t, st, preview[at]),
                    )
        b.actions[:] = last
        gone = (to_move >= 0) & ~ok
        if gone.any():
            b.random_actions(rng)
            last[gone] = b.actions[gone]
            b.actions[:] = last
        b.step()
        for env in np.nonzero(b.done)[0].tolist():
            held.check(b.episode_seed[[env]])
        if scored >= report:
            report += 2000
            log(f"{scored} states, {time.perf_counter() - start:.0f} s", flush=True)
    out = {
        "network": checkpoint,
        "network_sha256": hashlib.sha256(Path(checkpoint).read_bytes()).hexdigest(),
        "identity": {**_ffi.versions(), "torch": torch.__version__},
        "bench": str(config),
        "seed": seed,
        "draws": draws,
        "decisions": scored,
        "agreement": agree / max(scored, 1),
        "seconds": time.perf_counter() - start,
        **sums.report(),
        "strikes": {
            f"{who} {cat}": {
                "n": len(xs),
                "most_kills": float(np.mean([x[0] for x in xs])),
                "most_kills_uniform": float(np.mean([x[1] for x in xs])),
                "most_potential": float(np.mean([x[2] for x in xs])),
                "most_potential_uniform": float(np.mean([x[3] for x in xs])),
            }
            for (who, cat), xs0 in sorted(strikes.items())
            if (xs := [x for x in xs0 if x[4]])
        },
        "moves": {
            who: {
                "n": len(xs),
                "chosen": float(np.mean([x[0] for x in xs])),
                "least": float(np.mean([x[1] for x in xs])),
                "mean_legal": float(np.mean([x[2] for x in xs])),
                "at_least": float(np.mean([x[0] == x[1] for x in xs])),
                "at_least_when_some_higher": float(
                    np.mean([x[0] == x[1] for x in xs if x[2] > x[1]] or [0.0])
                ),
                "some_higher": float(np.mean([x[2] > x[1] for x in xs])),
                "chance_when_some_higher": float(
                    np.mean([x[3] for x in xs if x[2] > x[1]] or [0.0])
                ),
            }
            for who, xs in moves.items()
            if xs
        },
    }
    log(lines(out))
    return out


def lines(out: dict) -> str:
    rows = [
        f"{out['decisions']} states, the teacher's mode entry agreed {out['agreement']:.3f}",
        "disagreements (teacher > network): "
        + ", ".join(
            f"{k} {v}" for k, v in sorted(out["pairs"].items(), key=lambda kv: -kv[1])[:12]
        ),
    ]
    for who, x in out.get("strikes", {}).items():
        rows.append(
            f"{who} (several targets): {x['n']}, the most kills {x['most_kills']:.3f} (uniform "
            f"{x['most_kills_uniform']:.3f}), the most potential {x['most_potential']:.3f} "
            f"(uniform {x['most_potential_uniform']:.3f})"
        )
    for who, x in out.get("moves", {}).items():
        rows.append(
            f"{who}'s moves: {x['n']}, threat chosen {x['chosen']:.2f}, "
            f"least legal {x['least']:.2f}, "
            f"mean legal {x['mean_legal']:.2f}, at the least {x['at_least']:.3f} "
            f"({x['at_least_when_some_higher']:.3f} where the threat varies, {x['some_higher']:.3f}"
            f" of them; a uniform legal hex {x['chance_when_some_higher']:.3f})"
        )
    for g, feats in out["groups"].items():
        rows.append(f"== {g}")
        for k, x in feats.items():
            rows.append(
                f"  {k:<13} n {x['n']:>6}  teacher {x['teacher']:>9.2f}  network "
                f"{x['network']:>9.2f}  teacher lower {x['teacher_lower']:.3f}  higher "
                f"{x['teacher_higher']:.3f}"
            )
    return "\n".join(rows)


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("checkpoint")
    p.add_argument("out", type=Path)
    p.add_argument("--config", type=Path, default=Path("configs/distill/e25-structure.toml"))
    p.add_argument("--draws", type=int, default=8)
    p.add_argument("--decisions", type=int, default=12_800)
    a = p.parse_args()
    out = run(a.checkpoint, a.config, a.draws, a.decisions)
    a.out.write_text(json.dumps(out, indent=1) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
