"""The behaviour analysis (E06 T6a, `docs/epics/E06-log.md`): what a trained network does against
the engine's seats on a held-out set, and how it fares, read from the batch's outputs alone:

    uv run --group train python -m h3env.train.inspect --checkpoint runs/t7-1/best.pt

The sampled policy plays as on the ladder (`evaluate.report` with `evaluate.sampled`: the same
episodes and, on the same device, the same draws) against each opponent (the system actor and
Greedy by default) with the batch's kill preview on (C ABI v33; the play is the same), and each
battle is sorted by its kind, the agent's side and its outcome (won; lost, a truncation too). Per
group (`summary`):

- **decisions:** their share per category (`CATEGORIES`: wait, defend, move, melee, shot, the
  hero's cast, a stack's cast, a commanded turret's, the tactics phase's, the rest: heals, wall
  shots), their number per battle and the battle's rounds;
- **the hero's casts:** the rounds the hero could cast in (a decision with its cast legal) and the
  share it cast in, its spells, the mana both heroes spent (the maximum less the last
  observation's: the encounters start each hero at its maximum, so a cast before the agent's
  first decision counts, one after its last does not);
- **shooting:** what the agent chose when a shot was legal (a stack's, not a turret's);
- **targeting:** each melee strike's and shot's previewed kills (the mean of the least and the
  greatest, as hit points: kills × the target's hit points per creature) against the best legal
  strike's, and the strikes killing nothing while one could; how often it struck when a strike
  was legal;
- **trades:** per *clean* melee strike or shot (the next observation shows its effects alone: no
  opponent stack's turn and no new round between) the hit points it dealt and those it cost
  (retaliation, Fire Shield), and whether its target lost at least the least kills previewed (a
  check of the reading); melee strikes into a target with retaliations left (some attackers
  suffer none); who struck first (the first interval between two observations with hit points
  lost: the agent's when its own strike opened it).

Hit points are the creatures' (no war machines, no turrets): a stack's `(count - 1) × hp + top
hp`. The opponent's decisions are not seen (its seat plays inside the batch); its hero's casts
show in its mana.
"""

from __future__ import annotations

import argparse
import json
import os
import tomllib
from collections import Counter
from collections.abc import Callable, Iterable
from pathlib import Path

import numpy as np

from .. import _ffi
from ..batch import BattleBatch
from . import layout as L
from .evaluate import (
    HELD_OUT_BATCH_SEEDS,
    NEUTRAL_OPPONENT,
    REPORT_V1_ENVS,
    STRATIFIED,
    Agent,
    compact,
    report,
)

#: The decision categories, in the report's order (`category`).
CATEGORIES = (
    "wait",
    "defend",
    "move",
    "melee",
    "shot",
    "hero_cast",
    "creature_cast",
    "turret",
    "tactics",
    "other",
)
MELEE, SHOT, HERO_CAST, CREATURE_CAST, TURRET, TACTICS = (
    CATEGORIES.index(c)
    for c in ("melee", "shot", "hero_cast", "creature_cast", "turret", "tactics")
)
#: The categories that may strike (for who struck first).
STRUCK = (MELEE, SHOT, HERO_CAST, CREATURE_CAST)

#: The opponents the analysis plays by default.
OPPONENTS = ("system", "greedy")

#: The packs the encounters load, in load order: a spell's handle indexes their spells.
PACKS = ("base", "ab", "sod")


def _entry_categories() -> np.ndarray:
    c = np.full(L.ACTION_COUNT, CATEGORIES.index("other"), np.int64)
    c[0] = CATEGORIES.index("wait")
    c[1] = CATEGORIES.index("defend")
    c[L.MOVE_BASE : L.ATTACK_BASE] = CATEGORIES.index("move")
    c[L.ATTACK_BASE : L.SHOOT_BASE] = MELEE
    c[L.SHOOT_BASE : L.HEAL_BASE] = SHOT
    c[_ffi.HERO_CAST] = HERO_CAST
    c[_ffi.CREATURE_CAST] = CREATURE_CAST
    return c


_CATEGORY = _entry_categories()

#: The target stack slot of each previewed entry (from `L.ATTACK_BASE`: the attacks, the shots).
_PREVIEW_SLOT = np.concatenate(
    [
        np.arange(L.ATTACK_SLOTS * L.STACK_CAPACITY) // L.ATTACK_SLOTS,
        np.arange(2 * L.STACK_CAPACITY) // 2,
    ]
)


def category(
    entries: np.ndarray, rounds: np.ndarray, turret: np.ndarray | None = None
) -> np.ndarray:
    """Each decision's index in `CATEGORIES`: its entry's; a commanded turret's (the active stack
    is one: its shot is BAT-SIE-018's, which the kill preview does not follow), the tactics
    phase's in round 0."""
    c = _CATEGORY[np.asarray(entries, np.int64)]
    if turret is not None:
        c = np.where(turret, TURRET, c)
    return np.where(rounds == 0, TACTICS, c)


def view(obs: np.ndarray) -> dict[str, np.ndarray]:
    """What the analysis reads of observations [m, len]: per row the round, the creatures' hit
    points on each side now and at the start (`base count × hp`), each hero's presence, mana and
    maximum (the viewer's, the opponent's), whether the active stack is a turret; per stack slot
    its count, hit points per creature, retaliations left and the opponent's turn flags (0 for no
    opponent stack)."""
    s = obs[:, L.STACKS : L.OBSERVATION_LEN].reshape(len(obs), L.STACK_CAPACITY, L.STACK_FEATURES)
    present = s[..., L.PRESENT] > 0
    army = present & (s[..., L.WAR_MACHINE] == 0) & (s[..., L.TURRET] == 0)
    own = s[..., L.OWN] > 0
    count, max_hp = s[..., L.COUNT], s[..., L.MAX_HP]
    hp = np.where(army & (count > 0), (count - 1) * max_hp + s[..., L.TOP_HP], 0.0)
    start = np.where(army, s[..., L.BASE_COUNT] * max_hp, 0.0)
    turns = 8 + s[..., L.ACTED] + 2 * s[..., L.WAITED] + 4 * s[..., L.STARTED]
    heroes = obs[:, L.HEROES : L.OWN_HERO].reshape(len(obs), 2, L.HERO_PUBLIC_FEATURES)
    return {
        "round": obs[:, L.ROUND],
        "turret": ((s[..., L.ACTIVE] > 0) & (s[..., L.TURRET] > 0)).any(1),
        "hp": np.stack([(hp * own).sum(1), (hp * ~own).sum(1)], 1),
        "start": np.stack([(start * own).sum(1), (start * ~own).sum(1)], 1),
        "hero": heroes[..., 0] > 0,
        "mana": heroes[..., L.HERO_MANA],
        "max_mana": heroes[..., L.HERO_MAX_MANA],
        "count": count,
        "max_hp": max_hp,
        "retaliations": s[..., L.RETALIATIONS_LEFT],
        "turns": np.where(present & ~own, turns, 0.0),
    }


class _Battle:
    """A counted episode in progress: its numbers (`Counter`), the hero's spells, the rounds it
    could cast and cast in, who struck first, the last observation's heroes and the last
    decision (settled at the next call)."""

    def __init__(self, episode: int, kind: int) -> None:
        self.episode = episode
        self.kind = _ffi.ENCOUNTER_KINDS[kind] if kind >= 0 else "scenario"
        self.stats: Counter = Counter()
        self.spells: Counter = Counter()
        self.castable: set[int] = set()
        self.cast: set[int] = set()
        self.first: str | None = None
        self.round = 0
        self.heroes = None
        self.pending = None


class Recorder:
    """An agent for one `evaluate.play` batch: plays `agent` and records the decisions of the
    episodes the play counts (each env's first `episodes`) as `side`, settles each decision at
    the next call (its effects) and each battle once it ends (`finish`: the last call's). The
    finished battles are `battles`: (kind, won, numbers, spells)."""

    def __init__(self, agent: Agent, side: str, episodes: int = 1) -> None:
        self.agent = agent
        self.side = 0 if side == "attacker" else 1
        self.winning = (_ffi.RESULT_ATTACKER, _ffi.RESULT_DEFENDER)[self.side]
        self.episodes = episodes
        self.open: dict[int, _Battle] = {}
        self.battles: list[tuple[str, bool, Counter, Counter]] = []
        self.batch: BattleBatch | None = None

    def __call__(self, batch: BattleBatch) -> None:
        if batch.preview is None:
            raise ValueError("the analysis reads the batch's kill preview (preview=True)")
        self.batch = batch
        rows = np.nonzero((batch.episode < self.episodes) & (batch.to_move >= 0))[0]
        seen = view(batch.obs[rows])
        self._settle(batch, rows, seen)
        self.agent(batch)
        self._record(batch, rows, seen)

    def finish(self) -> None:
        """Settles the battles the play's last call ended (the agent is not called after it); a
        battle still open (the play's bound reached) is left out."""
        if self.batch is not None:
            self._settle(self.batch, np.zeros(0, np.int64), None)

    def _settle(self, batch: BattleBatch, rows: np.ndarray, seen) -> None:
        at = {int(e): i for i, e in enumerate(rows)}
        for e, battle in list(self.open.items()):
            if batch.done[e]:
                del self.open[e]
                self._close(battle, batch, e)
            elif e in at:
                self._interval(battle, seen, at[e])

    def _interval(self, battle: _Battle, seen, i: int) -> None:
        """The effects of the battle's last decision up to this observation of it."""
        round_, hp, turns, c, slot, count, least = battle.pending
        dealt = hp[1] - seen["hp"][i, 1]
        taken = hp[0] - seen["hp"][i, 0]
        if battle.first is None and (dealt > 0 or taken > 0):
            battle.first = "agent" if c in STRUCK and dealt > 0 else "opponent"
        clean = seen["round"][i] == round_ and np.array_equal(seen["turns"][i], turns)
        if clean and c in (MELEE, SHOT):
            name = CATEGORIES[c]
            st = battle.stats
            st[f"clean_{name}"] += 1
            st[f"clean_{name}_dealt"] += float(dealt)
            st[f"clean_{name}_taken"] += float(taken)
            st["clean_least_killed"] += int(count - seen["count"][i, slot] >= least)

    def _record(self, batch: BattleBatch, rows: np.ndarray, seen) -> None:
        if not len(rows):
            return
        a = batch.actions
        entries = a["entry"][rows].astype(np.int64)
        rounds = seen["round"]
        cats = category(entries, rounds, seen["turret"])
        mask = batch.entry_mask[rows]
        # A turret's shots are left out of the strikes (the preview does not follow them).
        strikes = mask[:, L.ATTACK_BASE : L.HEAL_BASE].astype(bool) & ~seen["turret"][:, None]
        shot_legal = strikes[:, L.SHOOT_BASE - L.ATTACK_BASE :].any(1)
        castable = mask[:, _ffi.HERO_CAST] > 0
        preview = batch.preview[rows]
        kill_hp = preview.mean(2) * seen["max_hp"][:, _PREVIEW_SLOT]
        best = np.where(strikes, kill_hp, 0.0).max(1)
        k = np.clip(entries - L.ATTACK_BASE, 0, len(_PREVIEW_SLOT) - 1)
        m = np.arange(len(rows))
        chosen, least, slot = kill_hp[m, k], preview[m, k, 0], _PREVIEW_SLOT[k]
        for i, e in enumerate(rows.tolist()):
            battle = self.open.get(e)
            if battle is None:
                battle = self.open[e] = _Battle(int(batch.episode[e]), int(batch.encounter[e]))
                # Hit points lost before the agent's first decision: the opponent struck first
                # (or the agent's automatic war machines did).
                lost = seen["start"][i] - seen["hp"][i]
                battle.first = "opponent" if lost[0] > 0 else "agent" if lost[1] > 0 else None
            c, r, st = int(cats[i]), int(rounds[i]), battle.stats
            st["decisions"] += 1
            st[CATEGORIES[c]] += 1
            battle.round = max(battle.round, r)
            if castable[i]:
                battle.castable.add(r)
            if c == HERO_CAST:
                battle.cast.add(r)
                battle.spells[int(a["spell"][e])] += 1
            if shot_legal[i]:
                st["shot_legal"] += 1
                st[f"shot_legal_{CATEGORIES[c]}"] += 1
            if strikes[i].any():
                st["strike_legal"] += 1
                st["strike_legal_struck"] += c in (MELEE, SHOT)
            if c in (MELEE, SHOT):
                st["strikes"] += 1
                st["strike_hp"] += float(chosen[i])
                st["best_hp"] += float(best[i])
                st["at_best"] += bool(chosen[i] >= best[i])
                st["no_kill"] += bool(chosen[i] == 0 and best[i] > 0)
            if c == MELEE:
                st["melee_into_retaliation"] += bool(seen["retaliations"][i, slot[i]] > 0)
            battle.pending = (
                rounds[i],
                seen["hp"][i].copy(),
                seen["turns"][i].copy(),
                c,
                int(slot[i]),
                float(seen["count"][i, slot[i]]),
                float(least[i]),
            )
            battle.heroes = (seen["hero"][i], seen["mana"][i], seen["max_mana"][i])

    def _close(self, battle: _Battle, batch: BattleBatch, e: int) -> None:
        """The battle ended in the last call: its outcome, rounds, casts, mana and first strike."""
        st = battle.stats
        if battle.first is None:
            # Decided within the last interval: by the agent's strike if it struck last.
            c = battle.pending[3]
            battle.first = "agent" if c in STRUCK else "opponent"
        won = int(batch.result[e]) == self.winning
        st["battles"] = 1
        st["won"] = int(won)
        st["truncated"] = int(batch.truncated[e])
        st["reward"] = float(batch.reward[e, self.side])
        st["rounds"] = battle.round
        st[f"first_{battle.first}"] = 1
        hero, mana, max_mana = battle.heroes
        for j, who in enumerate(("hero", "opponent_hero")):
            if hero[j]:
                st[who] = 1
                st[f"{who}_mana_spent"] = float(max_mana[j] - mana[j])
                st[f"{who}_max_mana"] = float(max_mana[j])
        if hero[0]:
            st["castable_rounds"] = len(battle.castable)
            st["cast_rounds"] = len(battle.cast)
        self.battles.append((battle.kind, won, st, battle.spells))


def spell_names(packs: Iterable[str] = PACKS) -> list[str]:
    """The spells' names by handle: the packs' spells in load order (`BattleRuleset::spells`)."""
    root = Path(__file__).resolve().parents[3] / "packs"
    names = []
    for pack in packs:
        path = root / pack / "core.toml"
        if path.exists():
            names += [s["name"] for s in tomllib.loads(path.read_text("utf-8")).get("spell", [])]
    return names


def _share(a: float, b: float) -> float | None:
    return a / b if b else None


def summary(stats: Counter, spells: Counter, names: list[str] | None = None) -> dict:
    """A group's numbers (summed over its battles) as the report shows them: shares and means
    (None where nothing was counted)."""
    n, d, strikes = stats["battles"], stats["decisions"], stats["strikes"]
    hero, other = stats["hero"], stats["opponent_hero"]
    clean = {
        name: {
            "strikes": stats[f"clean_{name}"],
            "dealt": _share(stats[f"clean_{name}_dealt"], stats[f"clean_{name}"]),
            "taken": _share(stats[f"clean_{name}_taken"], stats[f"clean_{name}"]),
        }
        for name in ("melee", "shot")
    }
    names = names or []
    return {
        "battles": n,
        "won": _share(stats["won"], n),
        "truncated": _share(stats["truncated"], n),
        "reward": _share(stats["reward"], n),
        "rounds": _share(stats["rounds"], n),
        "decisions": _share(d, n),
        "mix": {c: _share(stats[c], d) for c in CATEGORIES},
        "hero": {
            "battles": hero,
            "castable_rounds": _share(stats["castable_rounds"], hero),
            "cast": _share(stats["cast_rounds"], stats["castable_rounds"]),
            "mana_spent": _share(stats["hero_mana_spent"], hero),
            "mana_share": _share(stats["hero_mana_spent"], stats["hero_max_mana"]),
            "spells": {
                (names[s] if s < len(names) else str(s)): k for s, k in spells.most_common()
            },
        },
        "opponent_hero": {
            "battles": other,
            "mana_spent": _share(stats["opponent_hero_mana_spent"], other),
            "mana_share": _share(
                stats["opponent_hero_mana_spent"], stats["opponent_hero_max_mana"]
            ),
        },
        "shooting": {
            "decisions": stats["shot_legal"],
            **{
                c: _share(stats[f"shot_legal_{c}"], stats["shot_legal"])
                for c in CATEGORIES
                if stats[f"shot_legal_{c}"]
            },
        },
        "targeting": {
            "strike_legal": stats["strike_legal"],
            "struck": _share(stats["strike_legal_struck"], stats["strike_legal"]),
            "strikes": strikes,
            "at_best": _share(stats["at_best"], strikes),
            "kills": _share(stats["strike_hp"], stats["best_hp"]),
            "no_kill": _share(stats["no_kill"], strikes),
        },
        "trades": {
            "melee_into_retaliation": _share(stats["melee_into_retaliation"], stats["melee"]),
            **clean,
            "least_killed": _share(
                stats["clean_least_killed"], stats["clean_melee"] + stats["clean_shot"]
            ),
            "first": _share(stats["first_agent"], n),
        },
    }


def groups(recorders: Iterable[Recorder], names: list[str] | None = None) -> dict:
    """The recorders' battles by kind, the agent's side and its outcome (`won`, `lost` and both:
    `all`), each `summary`d, and every battle's under `all`."""
    sums: dict[tuple[str, str, str], tuple[Counter, Counter]] = {}
    for r in recorders:
        side = ("attacker", "defender")[r.side]
        for kind, won, stats, spells in r.battles:
            for key in ((kind, side, "won" if won else "lost"), (kind, side, "all"), ("all",) * 3):
                total = sums.setdefault(key, (Counter(), Counter()))
                total[0].update(stats)
                total[1].update(spells)
    out: dict = {}
    for (kind, side, outcome), (stats, spells) in sums.items():
        row = summary(stats, spells, names)
        if kind == "all":
            out["all"] = row
        else:
            out.setdefault(kind, {}).setdefault(side, {})[outcome] = row
    return out


def inspect(
    make_agent: Callable[[int, str], Agent],
    *,
    opponents: Iterable[str] = OPPONENTS,
    held_out: str = "report-v1",
    per_kind: int = REPORT_V1_ENVS,
    rng_seed: int = 0,
    threads: int = 0,
    max_steps: int = 2000,
) -> dict[str, dict]:
    """Plays a stratified set as the ladder does (`evaluate.report`, agents from
    `make_agent(seed, side)`) against each of `opponents` (the neutral set's one: the system
    actor) with the kill preview on, recording every counted battle; returns per opponent the
    ladder's row (`evaluate.compact`) and the battles' `groups`."""
    if held_out not in STRATIFIED:
        raise ValueError(f"not a stratified set: {held_out!r} (one of {tuple(STRATIFIED)})")
    if held_out == "neutral-v1":
        opponents = (NEUTRAL_OPPONENT,)
    names = spell_names()
    out = {}
    for opponent in opponents:
        recorders: list[Recorder] = []

        def make(seed: int, side: str, recorders: list[Recorder] = recorders) -> Agent:
            recorders.append(Recorder(make_agent(seed, side), side))
            return recorders[-1]

        row = report(
            make,
            opponent=opponent,
            per_kind=per_kind,
            held_out=held_out,
            rng_seed=rng_seed,
            threads=threads,
            max_steps=max_steps,
            preview=True,
        )
        for r in recorders:
            r.finish()
        out[opponent] = {"ladder": compact(row), "groups": groups(recorders, names)}
    return out


def main(argv: list[str] | None = None) -> None:
    """`python -m h3env.train.inspect --checkpoint runs/t7-1/best.pt`: a trained network's
    behaviour against the system actor and Greedy on a held-out set, two tables per opponent."""
    from .evaluate import load, sampled, trained_seed

    parser = argparse.ArgumentParser(prog="python -m h3env.train.inspect", description=main.__doc__)
    parser.add_argument("--checkpoint", required=True)
    parser.add_argument("--set", dest="held_out", choices=tuple(STRATIFIED), default="report-v1")
    parser.add_argument("--opponents", default=",".join(OPPONENTS))
    parser.add_argument("--episodes", type=int, default=REPORT_V1_ENVS, help="per kind")
    parser.add_argument("--seed", type=int, default=0, help="the draws' (the ladder's: 0)")
    parser.add_argument("--threads", type=int, default=os.cpu_count() or 1)
    parser.add_argument("--max-steps", type=int, default=2000)
    parser.add_argument("--device", default="cpu")
    parser.add_argument("--json", help="also write the results to this file")
    args = parser.parse_args(argv)
    trained = trained_seed(args.checkpoint)
    if trained in HELD_OUT_BATCH_SEEDS:
        parser.error(f"the checkpoint was trained on a held-out set's episodes (seed {trained:#x})")
    policy = load(args.checkpoint, args.device)
    policy.eval()
    results = inspect(
        sampled(policy),
        opponents=args.opponents.split(","),
        held_out=args.held_out,
        per_kind=args.episodes,
        rng_seed=args.seed,
        threads=args.threads,
        max_steps=args.max_steps,
    )
    for opponent, result in results.items():
        print("\n".join(lines(opponent, result)), flush=True)
    if args.json:
        Path(args.json).write_text(json.dumps(results, indent=1) + "\n", encoding="utf-8")


def _cell(value: float | None, scale: float = 100.0, width: int = 5) -> str:
    return f"{'-':>{width}}" if value is None else f"{value * scale:>{width}.0f}"


def _rows(groups_: dict):
    for kind, sides in groups_.items():
        if kind == "all":
            continue
        for side, outcomes in sides.items():
            for outcome in ("won", "lost"):
                if outcome in outcomes:
                    yield f"{kind} {side[:3]} {outcome}", outcomes[outcome]
    yield "all", groups_["all"]


def lines(opponent: str, result: dict) -> list[str]:
    """An opponent's report for the console: the ladder's line, then two tables over the groups
    (shares in percent): the decisions, then the casts, shots, targeting and trades."""
    lad = result["ladder"]
    kinds = ", ".join(f"{kind} {rate:.3f}" for kind, rate in lad["kinds"].items())
    out = [
        f"against {opponent}: win rate {lad['win_rate']:.3f} ± {lad['interval']:.3f} ({kinds}), "
        f"value {lad['value']:+.3f}, {lad['games']} games",
        f"{'group':26} {'n':>4} {'rnds':>5} {'dec':>5} "
        + " ".join(f"{c[:5]:>5}" for c in CATEGORIES),
    ]
    for name, g in _rows(result["groups"]):
        out.append(
            f"{name:26} {g['battles']:>4} {_cell(g['rounds'], 1)} {_cell(g['decisions'], 1)} "
            + " ".join(_cell(g["mix"][c]) for c in CATEGORIES)
        )
    out.append(
        f"{'group':26} {'cr/b':>5} {'cast':>5} {'mana':>5} {'opp':>5} {'shot':>5} {'shoot':>5} "
        f"{'str':>5} {'best':>5} {'kills':>5} {'none':>5} {'ret':>5} {'m.dlt':>6} {'m.tkn':>6} "
        f"{'s.dlt':>6} {'first':>5}"
    )
    for name, g in _rows(result["groups"]):
        h, t, tr = g["hero"], g["targeting"], g["trades"]
        out.append(
            f"{name:26} {_cell(h['castable_rounds'], 1)} {_cell(h['cast'])} "
            f"{_cell(h['mana_spent'], 1)} {_cell(g['opponent_hero']['mana_spent'], 1)} "
            f"{g['shooting']['decisions']:>5} {_cell(g['shooting'].get('shot'))} "
            f"{_cell(t['struck'])} {_cell(t['at_best'])} {_cell(t['kills'])} "
            f"{_cell(t['no_kill'])} {_cell(tr['melee_into_retaliation'])} "
            f"{_cell(tr['melee']['dealt'], 1, 6)} {_cell(tr['melee']['taken'], 1, 6)} "
            f"{_cell(tr['shot']['dealt'], 1, 6)} {_cell(tr['first'])}"
        )
    spells = result["groups"]["all"]["hero"]["spells"]
    if spells:
        out.append("hero's spells: " + ", ".join(f"{s} {k}" for s, k in list(spells.items())[:12]))
    return out


if __name__ == "__main__":
    main()
