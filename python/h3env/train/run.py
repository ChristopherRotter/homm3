"""A training run's lifecycle, free of torch (E06 T2a, T2b1; ADR-0026, `docs/epics/E06-design.md`
§1 to §5): the config and its TOML file, the schedule in samples, atomic writes, the held-out
check, the run's directory with its typed metrics and its record, and what a resume needs from
them (the changes it allows, the segments' batch seeds, the metrics cut back to the save, Ctrl+C).

A config file states only what differs from the defaults here; its tables group the keys (each
key belongs to one table, named in `Config`'s field metadata):

    seed = 3
    samples = 20_000_000

    [ppo]
    lr = 1e-4

`--set ppo.lr=1e-4` overrides one key (a TOML value, else a bare string). The record holds the
resolved config, so a run never depends on a later commit's defaults.
"""

from __future__ import annotations

import dataclasses
import json
import os
import platform
import signal
import subprocess
import sys
import threading
import time
import tomllib
from collections.abc import Callable, Iterable
from dataclasses import dataclass, field
from pathlib import Path
from typing import IO, Any

from .evaluate import HELD_OUT_BATCH_SEEDS, OPPONENTS, PLAYER_KINDS, REPORT_V1_ENVS

#: The version of `record.json`'s layout (`check_record`); 2: its segments (T2b1); 3: the
#: engine's identity (the encodings' and encounters' versions, the batch's entries, T2c); 4: the
#: gates, promotions and ladders instead of the evaluations, the parent (T3b); 5: the league
#: instead of the pool, its member table per gate (T4a).
RECORD_SCHEMA = 5

#: The config's tables of the older schemas committed records keep (`check_record`): 4 before the
#: league (`t3b-1`).
OLDER_TABLES = {4: ("env", "network", "ppo", "pool", "gate", "ladder", "checkpoint", "device")}

#: The identity's keys a record needs.
IDENTITY = (
    "commit",
    "dirty",
    "abi",
    "observation_encoding",
    "action_encoding",
    "encounters",
    "entries",
    "torch",
    "device",
)


#: The networks `[network] kind` names (`policy.build`).
NETWORK_KINDS = ("mlp", "entity")


def _in(table: str) -> dict:
    return {"table": table}


@dataclass
class Config:
    """Every setting of a run (`docs/epics/E06-design.md` §1.2): its record's `config`, by table.
    Cadences that change the training count samples (learner decisions that trained, §2)."""

    #: The record's id ("": the run directory's name).
    name: str = ""
    seed: int = 1
    #: The budget: the run stops at the first update boundary at or past this many samples.
    samples: int = 20_000_000
    envs: int = field(default=256, metadata=_in("env"))
    threads: int = field(default_factory=lambda: os.cpu_count() or 1, metadata=_in("env"))
    max_steps: int = field(default=2000, metadata=_in("env"))
    #: The family's kind weights (`H3_ENCOUNTER_*` order); None: the distribution's defaults.
    kinds: list[int] | None = field(default=None, metadata=_in("env"))
    #: The training's reward (the mana-price experiment, decision 10): the gate reads results
    #: only, the ladders score at reward v1's (`evaluate.CURRENCY`, `MANA_PRICE`).
    currency: str = field(default="ai_value", metadata=_in("env"))
    mana_price: int = field(default=10, metadata=_in("env"))
    #: The network (design §11): "mlp" (E05's) or "entity"; a saved file without the key is an
    #: MLP's.
    kind: str = field(default="mlp", metadata=_in("network"))
    #: The MLP's layers.
    hidden: int = field(default=512, metadata=_in("network"))
    #: Both kinds' spell and target embeddings.
    embed: int = field(default=32, metadata=_in("network"))
    #: The entity network's token width, transformer blocks and attention heads.
    width: int = field(default=128, metadata=_in("network"))
    layers: int = field(default=2, metadata=_in("network"))
    heads: int = field(default=4, metadata=_in("network"))
    #: Whether the MLP's entry head reads the batch's kill preview (AI-Q7 (b) part 2, E06 T6d:
    #: each legal strike's previewed kills, `policy`'s documentation); every batch the network
    #: plays then writes it.
    preview: bool = field(default=False, metadata=_in("network"))
    #: Whether the MLP's trunk reads the turn order (observation v22's last block, each stack
    #: slot's places; the turn-order probe, E06-log); else v21's prefix, as before v22.
    order: bool = field(default=False, metadata=_in("network"))
    rollout: int = field(default=128, metadata=_in("ppo"))
    #: 1.0 since E06 T5a (AI-Q9: γ 0.995 let the winner stall, `t4-1` against `t4-2`).
    gamma: float = field(default=1.0, metadata=_in("ppo"))
    lam: float = field(default=0.95, metadata=_in("ppo"))
    lr: float = field(default=2.5e-4, metadata=_in("ppo"))
    #: "linear" from `lr` to `lr * lr_floor` over `lr_horizon` samples, then the floor; or
    #: "constant".
    lr_schedule: str = field(default="linear", metadata=_in("ppo"))
    lr_floor: float = field(default=0.05, metadata=_in("ppo"))
    #: 0: the budget at the run's start (`resolve`).
    lr_horizon: int = field(default=0, metadata=_in("ppo"))
    epochs: int = field(default=4, metadata=_in("ppo"))
    minibatches: int = field(default=8, metadata=_in("ppo"))
    clip: float = field(default=0.2, metadata=_in("ppo"))
    value_coef: float = field(default=0.5, metadata=_in("ppo"))
    entropy_coef: float = field(default=0.01, metadata=_in("ppo"))
    max_grad_norm: float = field(default=0.5, metadata=_in("ppo"))
    #: What a battle cut at `max_steps` costs each learning side (E06 T5c, the user's amendment
    #: of decision 10): its last transition gets `-truncation_penalty` and ends there, instead of
    #: bootstrapping from the final observation (0: the bootstrap, as before T5c). Reward v1 lies
    #: in about (-1, 1), so 1.0 makes stalling to the limit worse than any decided battle.
    truncation_penalty: float = field(default=1.0, metadata=_in("ppo"))
    #: The league (design §8, `league`): the share of self-play episodes (else a live member plays
    #: one side, drawn per episode).
    self_play: float = field(default=0.5, metadata=_in("league"))
    #: The share of the player kinds' episodes (a hero battle, the two sieges a player defends)
    #: against the system actor (E06 T6b, decision 9 amended; design §12): it plays a uniformly
    #: drawn side inside the batch, the learner the other, and no member is drawn; the rest are
    #: self-play or a member's as above. In thousandths; above 0 it needs `[env] kinds` (the
    #: batch's entries split them).
    system_share: float = field(default=0.0, metadata=_in("league"))
    #: Samples between the learner's snapshots (the first after this many; 0: none, only the
    #: promoted networks join).
    snapshot_every: int = field(default=1_000_000, metadata=_in("league"))
    #: The members kept (each a network on the device); a new one retires the most beaten.
    max_members: int = field(default=16, metadata=_in("league"))
    #: The members a rollout's new episodes draw from (bounding a call's forward passes).
    live: int = field(default=4, metadata=_in("league"))
    #: Samples between the live set's draws (0: at every rollout).
    live_every: int = field(default=250_000, metadata=_in("league"))
    #: PFSP's weight of a member, `(1 − p)^pfsp_power` (0: uniform).
    pfsp_power: float = field(default=2.0, metadata=_in("league"))
    #: The decay of the learner's score sums against each member, per update.
    decay: float = field(default=0.99, metadata=_in("league"))
    #: The games (not decayed) a member must have played before it may retire.
    retire_games: int = field(default=200, metadata=_in("league"))
    #: Samples between gates (design §6): the learner against the best on the gate set v1, an
    #: SPRT promotes it; 0: only at the end. A run also gates at its end if it trained since its
    #: last gate.
    gate_every: int = field(default=10_000_000, metadata=_in("gate"))
    #: The SPRT's hypotheses (Elo differences) and error rates, and its cap in pairs (the cap
    #: rejects as inconclusive; at most the gate set's 4 000).
    elo0: float = field(default=0.0, metadata=_in("gate"))
    elo1: float = field(default=20.0, metadata=_in("gate"))
    alpha: float = field(default=0.05, metadata=_in("gate"))
    beta: float = field(default=0.05, metadata=_in("gate"))
    max_pairs: int = field(default=4000, metadata=_in("gate"))
    #: The engine seats each promoted network's ladder plays (design §7), on the report set v1
    #: (the system actor among them since E06 T7c).
    ladder_opponents: list[str] = field(
        default_factory=lambda: list(OPPONENTS), metadata=_in("ladder")
    )
    #: Whether each ladder also plays the neutral set v1 (as many per kind), attacking the system
    #: actor (E06 T7c: decision 7's neutral target).
    ladder_neutral: bool = field(default=True, metadata=_in("ladder"))
    #: Episodes per kind of a promoted network's ladder (the report set's first; at most 500).
    ladder_episodes: int = field(default=250, metadata=_in("ladder"))
    #: Episodes per kind of the final best's ladder at the run's end (0: none; 500 for a claim
    #: of a target).
    ladder_final_episodes: int = field(default=500, metadata=_in("ladder"))
    #: The promoted networks before it each ladder also plays (both seats external).
    ladder_promoted: int = field(default=1, metadata=_in("ladder"))
    #: Minutes of wall time between checkpoints, taken at the next update boundary (0: at every
    #: one); also before and after each gate and ladder, at a stop and at the end (design §3.2).
    checkpoint_minutes: float = field(default=5.0, metadata=_in("checkpoint"))
    device: str = field(default="auto", metadata=_in("device"))
    torch_threads: int = field(default=0, metadata=_in("device"))


#: Each table's keys, in `Config`'s order ("" for the top level).
TABLES: dict[str, tuple[str, ...]] = {}
for _f in dataclasses.fields(Config):
    _t = _f.metadata.get("table", "")
    TABLES[_t] = (*TABLES.get(_t, ()), _f.name)
_FIELDS = {f.name: f for f in dataclasses.fields(Config)}

#: The keys a resume may change (design §1.3): the budget, the machine's share, the checkpoints'
#: cadence. Every other key changes the training: that is a new run.
RESUMABLE = ("samples", "threads", "device", "torch_threads", "checkpoint_minutes")


def key_path(key: str) -> str:
    """`table.key` as a config file and `--set` name it (`key` at the top level)."""
    table = _FIELDS[key].metadata.get("table", "")
    return f"{table}.{key}" if table else key


class ConfigError(ValueError):
    """A config file or `--set` that names an unknown key or gives a value of the wrong type."""


_KINDS = {bool: "true or false", int: "an integer", float: "a number", str: "a string"}


def _check_type(key: str, value: Any) -> Any:
    default = getattr(Config(), key)
    if key == "ladder_opponents":
        if (
            isinstance(value, list)
            and value
            and all(v in OPPONENTS for v in value)
            and len(set(value)) == len(value)
        ):
            return value
        raise ConfigError(f"`ladder_opponents` takes distinct names of {OPPONENTS}, not {value!r}")
    if key == "kinds":
        if value is None or (
            isinstance(value, list)
            and len(value) == 6
            and all(isinstance(x, int) and not isinstance(x, bool) and x >= 0 for x in value)
        ):
            return value
        raise ConfigError(f"`kinds` takes six weights (H3_ENCOUNTER_* order), not {value!r}")
    if isinstance(default, bool):
        ok = isinstance(value, bool)
    elif isinstance(default, int):
        ok = isinstance(value, int) and not isinstance(value, bool)
    elif isinstance(default, float):
        ok = isinstance(value, (int, float)) and not isinstance(value, bool)
        value = float(value) if ok else value
    else:
        ok = isinstance(value, str)
    if not ok:
        raise ConfigError(f"`{key}` takes {_KINDS[type(default)]}, not {value!r}")
    return value


def _assign(values: dict[str, Any], table: str, key: str, value: Any) -> None:
    if key not in TABLES.get(table, ()):
        where = f"[{table}]" if table else "the top level"
        home = _FIELDS[key].metadata.get("table", "") if key in _FIELDS else None
        hint = (
            "" if home is None else f" (it belongs to {f'[{home}]' if home else 'the top level'})"
        )
        raise ConfigError(f"unknown key `{key}` in {where}{hint}")
    values[key] = _check_type(key, value)


def _parse_value(text: str) -> Any:
    try:
        return tomllib.loads(f"v = {text}")["v"]
    except tomllib.TOMLDecodeError:
        return text


def load_config(text: str = "", sets: Iterable[str] = ()) -> Config:
    """The config of a TOML file's `text` (unknown tables and keys refused), then each `--set
    table.key=value` (`key=value` for the top level), resolved (`resolve`)."""
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError as e:
        raise ConfigError(f"the config file is not TOML: {e}") from None
    values: dict[str, Any] = {}
    for key, value in data.items():
        if isinstance(value, dict):
            if key not in TABLES or not key:
                raise ConfigError(f"unknown table [{key}]")
            for k, v in value.items():
                _assign(values, key, k, v)
        else:
            _assign(values, "", key, value)
    for item in sets:
        _set(values, item)
    return resolve(Config(**values))


def _set(values: dict[str, Any], item: str) -> None:
    path, sep, raw = item.partition("=")
    if not sep:
        raise ConfigError(f"--set takes table.key=value, not {item!r}")
    table, _, key = path.strip().rpartition(".")
    _assign(values, table, key, _parse_value(raw.strip()))


def resumed(saved: Config, sets: Iterable[str] = ()) -> tuple[Config, dict[str, list[Any]]]:
    """The config a resume continues with (design §1.3): the checkpoint's with each `--set`
    applied, and what changed (`{key: [old, new]}`, for the segment's record). A change outside
    `RESUMABLE` is refused with its keys named. The learning rate's horizon stays the one fixed
    at the run's start, whatever the budget becomes."""
    values = dataclasses.asdict(saved)
    for item in sets:
        _set(values, item)
    config = resolve(Config(**values))
    changed = {
        k: [getattr(saved, k), getattr(config, k)]
        for k in _FIELDS
        if getattr(saved, k) != getattr(config, k)
    }
    refused = [key_path(k) for k in changed if k not in RESUMABLE]
    if refused:
        allowed = ", ".join(key_path(k) for k in RESUMABLE)
        raise ConfigError(
            f"a resume changes only {allowed}; {', '.join(refused)} would change the training: "
            "start a new run"
        )
    return config, changed


def resolve(config: Config) -> Config:
    """`lr_horizon` 0 fixed at the budget; the values checked."""
    c = config
    if c.lr_horizon == 0:
        c = dataclasses.replace(c, lr_horizon=c.samples)
    problems = []
    for key in ("samples", "envs", "max_steps", "rollout", "epochs", "minibatches", "hidden"):
        if getattr(c, key) <= 0:
            problems.append(f"`{key}` must be positive")
    if c.kind not in NETWORK_KINDS:
        problems.append(f"`kind` is one of {', '.join(NETWORK_KINDS)}")
    elif c.preview and c.kind != "mlp":
        problems.append('`preview` is the MLP\'s (`kind = "mlp"`)')
    elif c.order and c.kind != "mlp":
        problems.append('`order` is the MLP\'s (`kind = "mlp"`)')
    if min(c.embed, c.width, c.layers, c.heads) <= 0 or c.width % c.heads:
        problems.append(
            "`embed`, `width`, `layers` and `heads` are positive, `width` a multiple of `heads`"
        )
    for key in ("max_members", "live"):
        if getattr(c, key) <= 0:
            problems.append(f"`{key}` must be positive")
    for key in (
        "threads",
        "snapshot_every",
        "live_every",
        "retire_games",
        "gate_every",
        "torch_threads",
        "ladder_promoted",
    ):
        if getattr(c, key) < 0:
            problems.append(f"`{key}` must not be negative")
    if c.checkpoint_minutes < 0 or c.pfsp_power < 0 or c.truncation_penalty < 0:
        problems.append(
            "`checkpoint_minutes`, `pfsp_power` and `truncation_penalty` must not be negative"
        )
    if c.lr_schedule not in ("linear", "constant"):
        problems.append("`lr_schedule` is linear or constant")
    if not 0.0 <= c.lr_floor <= 1.0 or not 0.0 <= c.self_play <= 1.0 or not 0.0 < c.decay <= 1.0:
        problems.append("`lr_floor` and `self_play` lie in [0, 1], `decay` in (0, 1]")
    if not 0.0 <= c.system_share <= 1.0:
        problems.append("`system_share` lies in [0, 1]")
    elif c.system_share and c.kinds is None:
        problems.append("`system_share` needs `[env] kinds` (the player kinds' weights to split)")
    elif c.system_share and not any(c.kinds[k] for k in PLAYER_KINDS):
        problems.append("`system_share` needs a player kind (hero, siege_hero, siege_garrison)")
    if c.seed in HELD_OUT_BATCH_SEEDS:
        problems.append("`seed` is a held-out set's batch seed (v0's, the v1 sets')")
    if not (0.0 < c.alpha < 0.5 and 0.0 < c.beta < 0.5 and c.elo0 < c.elo1):
        problems.append("the gate's `alpha` and `beta` lie in (0, 0.5), `elo0` below `elo1`")
    if not 1 <= c.max_pairs <= 4000:
        problems.append("`max_pairs` lies in 1 to 4000 (the gate set's pairs)")
    if not 1 <= c.ladder_episodes <= REPORT_V1_ENVS or not 0 <= c.ladder_final_episodes <= 500:
        problems.append("`ladder_episodes` lies in 1 to 500, `ladder_final_episodes` in 0 to 500")
    if problems:
        raise ConfigError("; ".join(problems))
    return c


def by_table(config: Config) -> dict[str, Any]:
    """The config as the record shows it: the top-level keys, then one dict per table."""
    out: dict[str, Any] = {}
    for table, keys in TABLES.items():
        values = {k: getattr(config, k) for k in keys}
        if table:
            out[table] = values
        else:
            out.update(values)
    return out


def learning_rate(config: Config, samples: int) -> float:
    """The learning rate after `samples` (§2): linear to the floor over the horizon, then flat."""
    if config.lr_schedule == "constant":
        return config.lr
    progress = samples / max(config.lr_horizon, 1)
    return config.lr * max(1.0 - progress, config.lr_floor)


class Cadence:
    """Due once at each multiple of `every` samples (0: never): an update that passes several
    counts once."""

    def __init__(self, every: int) -> None:
        self.every = every
        self.next = every

    def due(self, samples: int) -> bool:
        if self.every <= 0 or samples < self.next:
            return False
        self.next = (samples // self.every + 1) * self.every
        return True


_MASK64 = (1 << 64) - 1
_GOLDEN = 0x9E3779B97F4A7C15


def splitmix64(state: int) -> int:
    """SplitMix64's next output from `state` (Steele, Lea and Flood 2014; the generator's state
    advances by the golden gamma before the mix)."""
    z = (state + _GOLDEN) & _MASK64
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & _MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & _MASK64
    return z ^ (z >> 31)


def segment_seed(seed: int, segment: int) -> int:
    """Segment `segment`'s batch seed (design §3.3): the run's `seed` for the first, else the
    `segment`-th output of SplitMix64 seeded with `seed`, so a resumed run plays episodes it has
    not trained on. Every episode's seed is still checked against the held-out set's."""
    if segment == 0:
        return seed
    return splitmix64((seed + (segment - 1) * _GOLDEN) & _MASK64)


class Stop:
    """A requested stop (design §3.2): while installed (`with stop:`, in the main thread), the
    first SIGINT (Ctrl+C; Windows delivers it to Python too) sets `requested`, which the run reads
    at each batch call of a rollout and at each update boundary; the second raises
    `KeyboardInterrupt` at once (losing what the first would have saved)."""

    def __init__(self) -> None:
        self.requested = False
        self._previous: Any = None
        self._installed = False

    def request(self) -> None:
        self.requested = True

    def _handle(self, signum: int, frame: Any) -> None:
        if self.requested:
            raise KeyboardInterrupt
        self.requested = True

    def __enter__(self) -> Stop:
        if threading.current_thread() is threading.main_thread():
            self._previous = signal.signal(signal.SIGINT, self._handle)
            self._installed = True
        return self

    def __exit__(self, *exc: Any) -> None:
        if self._installed:
            previous = self._previous
            signal.signal(
                signal.SIGINT, signal.default_int_handler if previous is None else previous
            )
            self._installed = False


def atomic_write(path: Path, write: Callable[[IO[bytes]], None]) -> bool:
    """`write`s into `<path>.tmp` beside `path`, syncs it, then replaces `path` (atomic on one
    volume); returns whether it did. Windows refuses the replace while another process holds
    `path` open (a ladder reading `best.pt`): five tries over about a second, then the old file
    stays, a warning goes to standard error and the run goes on (design §3; the next save
    tries again)."""
    path = Path(path)
    tmp = path.with_name(path.name + ".tmp")
    with open(tmp, "wb") as f:
        write(f)
        f.flush()
        os.fsync(f.fileno())
    for attempt in range(5):
        try:
            os.replace(tmp, path)
            return True
        except PermissionError as e:
            if attempt == 4:
                tmp.unlink(missing_ok=True)
                print(f"warning: {path} kept, its replace refused: {e}", file=sys.stderr)
                return False
            time.sleep(0.25)
    return False


def atomic_write_text(path: Path, text: str) -> None:
    atomic_write(path, lambda f: f.write(text.encode("utf-8")))


class HeldOutError(RuntimeError):
    """A training episode's seed is a held-out one: the run stops (design §5)."""


class HeldOut:
    """The held-out episodes' seeds, fixed; each training episode's seed is checked against them
    (no growing set of training seeds; disjoint unless two 64-bit hashes meet)."""

    def __init__(self, seeds: Iterable[int]) -> None:
        self.seeds = frozenset(int(s) for s in seeds)

    def check(self, seeds: Iterable[int]) -> None:
        hit = self.seeds.intersection(int(s) for s in seeds)
        if hit:
            raise HeldOutError(f"training reached held-out episodes (seeds {sorted(hit)[:3]})")


def _git(*args: str) -> str | None:
    try:
        out = subprocess.run(["git", *args], capture_output=True, text=True, check=True, timeout=10)
        return out.stdout
    except (OSError, subprocess.SubprocessError):
        return None


def source() -> dict[str, Any]:
    """The commit the run starts from and whether tracked files differ from it."""
    head = _git("rev-parse", "HEAD")
    status = _git("status", "--porcelain", "--untracked-files=no")
    return {
        "commit": head.strip() if head else "unknown",
        "dirty": None if status is None else bool(status.strip()),
    }


def _cpu() -> str:
    if sys.platform == "linux":
        try:
            for line in Path("/proc/cpuinfo").read_text().splitlines():
                if line.startswith("model name"):
                    return line.split(":", 1)[1].strip()
        except OSError:
            pass
    return platform.processor() or platform.machine()


def machine() -> dict[str, Any]:
    import numpy

    return {
        "os": platform.platform(),
        "cpu": _cpu(),
        "cores": os.cpu_count(),
        "python": platform.python_version(),
        "numpy": numpy.__version__,
    }


#: The update records' fields the record's curve averages per gate interval.
CURVE = (
    "samples_per_s",
    "attacker_won",
    "truncated",
    "league_score",
    "frozen_passes",
    "policy_loss",
    "value_loss",
    "entropy",
    "kl",
    "clip",
    "explained_variance",
)


def _segment(record: dict[str, Any]) -> dict[str, Any]:
    """A segment's entry in the record, opened by its `start` or `segment` record."""
    identity = record.get("identity", {})
    at = {"samples": record["samples"], "updates": record["update"], "minutes": record["minutes"]}
    return {
        "segment": record.get("segment", 0),
        "seed": record.get("seed"),
        **{k: identity.get(k) for k in ("started", "commit", "dirty", "device")},
        "changed": record.get("changed", {}),
        "start": at,
        "end": dict(at),
        #: "budget" (its `end`), "stop" (its `stop`), "lost" (killed or crashed: the next
        #: segment found no record of its end), None (running).
        "ended": None,
        #: Its records after its last checkpoint, cut at the next segment's start.
        "dropped": 0,
    }


#: A gate record's fields the record keeps (`league`: the member table at the gate, schema 5).
GATE = ("best", "result", "pairs", "counts", "llr", "score", "elo", "games", "seconds", "league")


class Record:
    """The run's summary (`record.json`, design §4), built from its typed metrics records in
    order (so `metrics.jsonl` replayed rebuilds it: a resume does, `Run`)."""

    def __init__(self, name: str, config: Config, identity: dict[str, Any]) -> None:
        self.name = name
        self.config = config
        self.identity = identity
        #: The network the run started from (`--init-from`), or None.
        self.parent: dict[str, Any] | None = None
        self.progress: dict[str, Any] = {
            "samples": 0,
            "updates": 0,
            "episodes": 0,
            "minutes": 0.0,
            #: The gates' and ladders' wall time (their share of the run's, design §9).
            "evaluation_minutes": 0.0,
            "ended": None,
        }
        self.segments: list[dict[str, Any]] = []
        self.curve: list[dict[str, Any]] = []
        self.gates: list[dict[str, Any]] = []
        self.promotions: list[dict[str, Any]] = []
        self.ladders: list[dict[str, Any]] = []
        self._sums: dict[str, float] = {}
        self._count = 0
        #: Per encounter kind since the last gate: the attacker's wins, truncations, episodes.
        self._kinds: dict[str, list[float]] = {}
        #: Per side of the learner against the system actor since the last gate: its score sum
        #: and episodes (E06 T6b).
        self._system: dict[str, list[float]] = {}

    def _ended(self, record: dict[str, Any], how: str) -> None:
        self.progress["ended"] = how
        self.progress["minutes"] = record["minutes"]
        if self.segments:
            self.segments[-1]["ended"] = how
            self.segments[-1]["end"]["minutes"] = record["minutes"]

    def observe(self, record: dict[str, Any]) -> None:
        kind = record["type"]
        at = {k: record[k] for k in ("samples", "update", "minutes") if k in record}
        if kind == "start":
            self.identity = record.get("identity", self.identity)
            self.parent = record.get("parent")
            self.segments = [_segment(record)]
        elif kind == "segment":
            last = self.segments[-1]
            if last["ended"] is None:
                last["ended"] = "lost"
            last["dropped"] = record["dropped"]
            self.segments.append(_segment(record))
            self.progress["ended"] = None
        elif kind == "update":
            for k in ("samples", "episodes", "minutes"):
                self.progress[k] = record[k]
            for name, row in record.get("by_kind", {}).items():
                sums = self._kinds.setdefault(name, [0.0, 0.0, 0])
                sums[0] += row["attacker_won"] * row["episodes"]
                sums[1] += row["truncated"] * row["episodes"]
                sums[2] += row["episodes"]
            for name, row in record.get("system", {}).items():
                sums = self._system.setdefault(name, [0.0, 0])
                sums[0] += row["score"] * row["episodes"]
                sums[1] += row["episodes"]
            self.progress["updates"] = record["update"]
            if self.segments:
                end = self.segments[-1]["end"]
                end["samples"], end["updates"] = record["samples"], record["update"]
                end["minutes"] = record["minutes"]
            for k in CURVE:
                value = record.get(k)
                if value is not None and value == value:  # NaN (no variance) counts nothing
                    self._sums[k] = self._sums.get(k, 0.0) + float(value)
            self._count += 1
        elif kind == "gate":
            if self._count:
                point = {k: v / self._count for k, v in self._sums.items()}
                if self._kinds:
                    point["by_kind"] = {
                        name: {
                            "episodes": n,
                            "attacker_won": won / n,
                            "truncated": cut / n,
                        }
                        for name, (won, cut, n) in self._kinds.items()
                    }
                if self._system:
                    point["system"] = {
                        name: {"episodes": n, "score": score / n}
                        for name, (score, n) in self._system.items()
                    }
                self.curve.append(
                    {"samples": record["samples"], "updates": record["update"], **point}
                )
            self._sums, self._count, self._kinds, self._system = {}, 0, {}, {}
            self.gates.append({**at, **{k: record[k] for k in GATE}})
            self.progress["evaluation_minutes"] += record["seconds"] / 60
        elif kind == "promotion":
            self.promotions.append({**at, "index": record["index"], "how": record["how"]})
        elif kind == "ladder":
            keep = ("win_rate", "interval", "value", "kinds", "truncated")
            neutral = record.get("neutral")
            self.ladders.append(
                {
                    **at,
                    **{k: record[k] for k in ("index", "final", "per_kind", "seconds")},
                    "opponents": {
                        name: {k: row[k] for k in keep} for name, row in record["opponents"].items()
                    },
                    "neutral": None if neutral is None else {k: neutral[k] for k in keep},
                }
            )
            self.progress["evaluation_minutes"] += record["seconds"] / 60
        elif kind == "stop":
            self._ended(record, "stop")
        elif kind == "end":
            self._ended(record, record["reason"])

    def summary(self) -> dict[str, Any]:
        best = self.promotions[-1] if self.promotions else None
        return {
            "schema": RECORD_SCHEMA,
            "id": self.name,
            "identity": self.identity,
            "parent": self.parent,
            "config": by_table(self.config),
            "progress": dict(self.progress),
            "segments": self.segments,
            "curve": self.curve,
            "gates": self.gates,
            "promotions": self.promotions,
            "ladders": self.ladders,
            "best": None if best is None else {"index": best["index"], "samples": best["samples"]},
        }


def check_record(record: Any) -> list[str]:
    """What a committed record (`python/records/<run>.json`) lacks for its schema; empty if
    nothing."""
    if not isinstance(record, dict):
        return ["not a JSON object"]
    schema = record.get("schema")
    if schema != RECORD_SCHEMA and schema not in OLDER_TABLES:
        return [
            f"schema {schema!r} is not {RECORD_SCHEMA} (nor an older one, {list(OLDER_TABLES)})"
        ]
    tables = OLDER_TABLES.get(schema, tuple(t for t in TABLES if t))
    problems = []
    shapes = {
        "id": str,
        "identity": dict,
        "config": dict,
        "progress": dict,
        "segments": list,
        "curve": list,
        "gates": list,
        "promotions": list,
        "ladders": list,
    }
    for key, kind in shapes.items():
        if not isinstance(record.get(key), kind):
            problems.append(f"`{key}` is not a {kind.__name__}")
    if problems:
        return problems
    for key in IDENTITY:
        if key not in record["identity"]:
            problems.append(f"`identity.{key}` is missing")
    for key in ("seed", "samples", *tables):
        if key not in record["config"]:
            problems.append(f"`config.{key}` is missing")
    for key in ("samples", "updates", "episodes", "minutes", "evaluation_minutes", "ended"):
        if key not in record["progress"]:
            problems.append(f"`progress.{key}` is missing")
    if not record["id"]:
        problems.append("`id` is empty")
    if not record["segments"]:
        problems.append("`segments` is empty")
    for i, s in enumerate(record["segments"]):
        if not isinstance(s, dict) or not {"segment", "seed", "ended", "start", "end"} <= set(s):
            problems.append(f"segment {i} lacks `segment`, `seed`, `ended`, `start` or `end`")
    if "parent" not in record:
        problems.append("`parent` is missing")
    gate = {"samples", "result", "pairs", "llr", "score"} | ({"league"} if schema >= 5 else set())
    for name, needs in (
        ("gates", gate),
        ("promotions", {"samples", "index", "how"}),
        ("ladders", {"samples", "index", "final", "opponents"}),
    ):
        for i, entry in enumerate(record[name]):
            if not isinstance(entry, dict) or not needs <= set(entry):
                problems.append(f"{name} {i} lacks one of {sorted(needs)}")
    return problems


class RunExistsError(FileExistsError):
    """The directory already holds a run's metrics: a new run would mix its records with them."""


class ResumeError(RuntimeError):
    """A run that cannot be resumed: no checkpoint, another checkpoint format, metrics shorter
    than at the save, or a finished run whose budget the resume does not extend."""


def check_new(out: Path | None) -> None:
    """Refuses a directory whose `metrics.jsonl` holds records (`--resume` continues one)."""
    if out is not None:
        metrics = Path(out) / "metrics.jsonl"
        if metrics.exists() and metrics.stat().st_size > 0:
            raise RunExistsError(
                f"{out} already holds a run (its metrics.jsonl): choose another, or --resume it"
            )


class Run:
    """A run's directory (design §1.1): `config.toml` (the file as given), `metrics.jsonl` (one
    typed record per line, flushed), `record.json` (rewritten atomically by `write_record`).
    Without a directory it keeps the record in memory only.

    With `resume` (`(length, seconds)` from a checkpoint, design §3.3) it reopens a run's
    directory instead: `metrics.jsonl` is cut back to the `length` bytes it had at the save (the
    records after it, `dropped`, were of updates the resumed run plays again), the record is
    rebuilt from what stays, and the run's clock continues from `seconds`."""

    def __init__(
        self,
        out: Path | None,
        config: Config,
        identity: dict[str, Any],
        text: str = "",
        log: Callable[[dict[str, Any]], None] | None = None,
        resume: tuple[int, float] | None = None,
    ) -> None:
        self.out = None if out is None else Path(out)
        name = config.name or (self.out.name if self.out is not None else "")
        self.record = Record(name, config, identity)
        self.log = log
        self.metrics: IO[bytes] | None = None
        #: `metrics.jsonl`'s length in bytes (a checkpoint's position in it).
        self.length = 0
        self.dropped = 0
        self.start = time.perf_counter()
        self.offset = 0.0 if resume is None else resume[1]
        if self.out is None:
            return
        if resume is None:
            check_new(self.out)
            self.out.mkdir(parents=True, exist_ok=True)
        for stale in self.out.glob("*.tmp"):
            stale.unlink()
        path = self.out / "metrics.jsonl"
        if resume is None:
            atomic_write_text(self.out / "config.toml", text)
        else:
            self._cut(path, resume[0])
        self.metrics = path.open("ab")
        self.length = path.stat().st_size

    def _cut(self, path: Path, length: int) -> None:
        data = path.read_bytes() if path.exists() else b""
        if len(data) < length:
            raise ResumeError(
                f"{path} holds {len(data)} bytes, fewer than the {length} at the checkpoint"
            )
        self.dropped = sum(1 for line in data[length:].split(b"\n") if line.strip())
        for line in data[:length].decode("utf-8").splitlines():
            self.record.observe(json.loads(line))
        with path.open("r+b") as f:
            f.truncate(length)

    def seconds(self) -> float:
        """The run's wall time: its segments' up to their last checkpoints, and this one's."""
        return self.offset + time.perf_counter() - self.start

    def minutes(self) -> float:
        return self.seconds() / 60

    def emit(self, record: dict[str, Any]) -> None:
        """Appends one typed record (`type` first) and shows it to the log."""
        self.record.observe(record)
        if self.metrics is not None:
            line = (json.dumps(record) + "\n").encode("utf-8")
            self.metrics.write(line)
            self.metrics.flush()
            self.length += len(line)
        if self.log is not None:
            self.log(record)

    def sync(self) -> int:
        """Puts the metrics on disk (before a checkpoint names their length) and returns that
        length."""
        if self.metrics is not None:
            self.metrics.flush()
            os.fsync(self.metrics.fileno())
        return self.length

    def write_record(self) -> None:
        if self.out is not None:
            text = json.dumps(self.record.summary(), indent=1) + "\n"
            atomic_write_text(self.out / "record.json", text)

    def close(self) -> None:
        if self.metrics is not None:
            self.metrics.close()
            self.metrics = None
