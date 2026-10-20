"""A training run's lifecycle, free of torch (E06 T2a; ADR-0026, `docs/epics/E06-design.md` §1,
§2, §4, §5.3): the config and its TOML file, the schedule in samples, atomic writes, the held-out
check, the run's directory with its typed metrics and its record.

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
import subprocess
import sys
import time
import tomllib
from collections.abc import Callable, Iterable
from dataclasses import dataclass, field
from pathlib import Path
from typing import IO, Any

#: The version of `record.json`'s layout (`check_record`).
RECORD_SCHEMA = 1


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
    currency: str = field(default="ai_value", metadata=_in("env"))
    mana_price: int = field(default=10, metadata=_in("env"))
    hidden: int = field(default=512, metadata=_in("network"))
    embed: int = field(default=32, metadata=_in("network"))
    rollout: int = field(default=128, metadata=_in("ppo"))
    gamma: float = field(default=0.995, metadata=_in("ppo"))
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
    #: The share of episodes in which a frozen earlier network (from the pool) plays one side,
    #: drawn per episode (self-play otherwise; `E05-log.md` §8.2; the league replaces it, T4).
    pool_share: float = field(default=0.5, metadata=_in("pool"))
    #: Snapshots the pool holds; when full, a new one replaces a random one (ages stay spread).
    pool_size: int = field(default=8, metadata=_in("pool"))
    #: Samples between snapshots (the first after this many).
    pool_every: int = field(default=600_000, metadata=_in("pool"))
    #: Samples between evaluations against Random (and one at the end); 0: only at the end. Until
    #: the gate and the ladder (T3) replace it.
    eval_every: int = field(default=5_000_000, metadata=_in("evaluation"))
    eval_episodes: int = field(default=1000, metadata=_in("evaluation"))
    eval_envs: int = field(default=100, metadata=_in("evaluation"))
    #: The held-out episodes' batch seed (E05's `EVAL_SEED`, the held-out set v0).
    eval_seed: int = field(default=0x5EED_E7A1_0000_0001, metadata=_in("evaluation"))
    #: Also evaluate greedy play (the criterion is the sampled policy's).
    eval_greedy: bool = field(default=True, metadata=_in("evaluation"))
    #: Evaluate the untrained network and Random itself on the same episodes first.
    eval_start: bool = field(default=True, metadata=_in("evaluation"))
    device: str = field(default="auto", metadata=_in("device"))
    torch_threads: int = field(default=0, metadata=_in("device"))


#: Each table's keys, in `Config`'s order ("" for the top level).
TABLES: dict[str, tuple[str, ...]] = {}
for _f in dataclasses.fields(Config):
    _t = _f.metadata.get("table", "")
    TABLES[_t] = (*TABLES.get(_t, ()), _f.name)
_FIELDS = {f.name: f for f in dataclasses.fields(Config)}


class ConfigError(ValueError):
    """A config file or `--set` that names an unknown key or gives a value of the wrong type."""


_KINDS = {bool: "true or false", int: "an integer", float: "a number", str: "a string"}


def _check_type(key: str, value: Any) -> Any:
    default = getattr(Config(), key)
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
        path, sep, raw = item.partition("=")
        if not sep:
            raise ConfigError(f"--set takes table.key=value, not {item!r}")
        table, _, key = path.strip().rpartition(".")
        _assign(values, table, key, _parse_value(raw.strip()))
    return resolve(Config(**values))


def resolve(config: Config) -> Config:
    """`lr_horizon` 0 fixed at the budget; the values checked."""
    c = config
    if c.lr_horizon == 0:
        c = dataclasses.replace(c, lr_horizon=c.samples)
    problems = []
    for key in ("samples", "envs", "max_steps", "rollout", "epochs", "minibatches", "hidden"):
        if getattr(c, key) <= 0:
            problems.append(f"`{key}` must be positive")
    for key in ("threads", "pool_every", "eval_every", "torch_threads"):
        if getattr(c, key) < 0:
            problems.append(f"`{key}` must not be negative")
    if c.lr_schedule not in ("linear", "constant"):
        problems.append("`lr_schedule` is linear or constant")
    if not 0.0 <= c.lr_floor <= 1.0 or not 0.0 <= c.pool_share <= 1.0:
        problems.append("`lr_floor` and `pool_share` lie in [0, 1]")
    if c.seed == c.eval_seed:
        problems.append("`seed` is the held-out episodes' batch seed (`eval_seed`)")
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


def atomic_write(path: Path, write: Callable[[IO[bytes]], None]) -> None:
    """`write`s into `<path>.tmp` beside `path`, syncs it, then replaces `path` (atomic on one
    volume). Windows refuses the replace while another process holds `path` open: five tries
    over about a second, then the error."""
    path = Path(path)
    tmp = path.with_name(path.name + ".tmp")
    with open(tmp, "wb") as f:
        write(f)
        f.flush()
        os.fsync(f.fileno())
    for attempt in range(5):
        try:
            os.replace(tmp, path)
            return
        except PermissionError:
            if attempt == 4:
                raise
            time.sleep(0.25)


def atomic_write_text(path: Path, text: str) -> None:
    atomic_write(path, lambda f: f.write(text.encode("utf-8")))


class HeldOutError(RuntimeError):
    """A training episode's seed is a held-out one: the run stops (design §5.3)."""


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


#: The update records' fields the record's curve averages per evaluation interval.
CURVE = (
    "samples_per_s",
    "attacker_won",
    "truncated",
    "pool_won",
    "policy_loss",
    "value_loss",
    "entropy",
    "kl",
    "clip",
    "explained_variance",
)


def _rates(result: dict) -> dict[str, Any]:
    return {
        "win_rate": result["win_rate"],
        "attacker": result["attacker"]["win_rate"],
        "defender": result["defender"]["win_rate"],
    }


class Record:
    """The run's summary (`record.json`, design §4.2), built from its typed metrics records in
    order (so `metrics.jsonl` replayed rebuilds it)."""

    def __init__(self, name: str, config: Config, identity: dict[str, Any]) -> None:
        self.name = name
        self.config = config
        self.identity = identity
        self.progress: dict[str, Any] = {
            "samples": 0,
            "updates": 0,
            "episodes": 0,
            "minutes": 0.0,
            "ended": None,
        }
        self.curve: list[dict[str, Any]] = []
        self.evaluations: list[dict[str, Any]] = []
        self._sums: dict[str, float] = {}
        self._count = 0

    def observe(self, record: dict[str, Any]) -> None:
        kind = record["type"]
        if kind == "update":
            for k in ("samples", "episodes", "minutes"):
                self.progress[k] = record[k]
            self.progress["updates"] = record["update"]
            for k in CURVE:
                value = record.get(k)
                if value is not None and value == value:  # NaN (no variance) counts nothing
                    self._sums[k] = self._sums.get(k, 0.0) + float(value)
            self._count += 1
        elif kind == "eval" and not record.get("repeated"):
            if self._count:
                point = {k: v / self._count for k, v in self._sums.items()}
                self.curve.append(
                    {"samples": record["samples"], "updates": record["update"], **point}
                )
            self._sums, self._count = {}, 0
            entry: dict[str, Any] = {
                "samples": record["samples"],
                "update": record["update"],
                "minutes": record["minutes"],
                "sampled": record["sampled"],
            }
            for name in ("greedy", "random"):
                if name in record:
                    entry[name] = _rates(record[name])
            self.evaluations.append(entry)
        elif kind == "end":
            self.progress["ended"] = record["reason"]
            self.progress["minutes"] = record["minutes"]

    def summary(self) -> dict[str, Any]:
        best = max(self.evaluations, key=lambda e: e["sampled"]["win_rate"], default=None)
        return {
            "schema": RECORD_SCHEMA,
            "id": self.name,
            "identity": self.identity,
            "config": by_table(self.config),
            "progress": dict(self.progress),
            "curve": self.curve,
            "evaluations": self.evaluations,
            "best": None
            if best is None
            else {"samples": best["samples"], "win_rate": best["sampled"]["win_rate"]},
        }


def check_record(record: Any) -> list[str]:
    """What a committed record (`python/records/<run>.json`) lacks for its schema; empty if
    nothing."""
    if not isinstance(record, dict):
        return ["not a JSON object"]
    if record.get("schema") != RECORD_SCHEMA:
        return [f"schema {record.get('schema')!r} is not {RECORD_SCHEMA}"]
    problems = []
    shapes = {
        "id": str,
        "identity": dict,
        "config": dict,
        "progress": dict,
        "curve": list,
        "evaluations": list,
    }
    for key, kind in shapes.items():
        if not isinstance(record.get(key), kind):
            problems.append(f"`{key}` is not a {kind.__name__}")
    if problems:
        return problems
    for key in ("commit", "dirty", "abi", "torch", "device"):
        if key not in record["identity"]:
            problems.append(f"`identity.{key}` is missing")
    for key in ("seed", "samples", *(t for t in TABLES if t)):
        if key not in record["config"]:
            problems.append(f"`config.{key}` is missing")
    for key in ("samples", "updates", "episodes", "minutes", "ended"):
        if key not in record["progress"]:
            problems.append(f"`progress.{key}` is missing")
    if not record["id"]:
        problems.append("`id` is empty")
    for i, e in enumerate(record["evaluations"]):
        if not isinstance(e, dict) or "samples" not in e or "win_rate" not in e.get("sampled", {}):
            problems.append(f"evaluation {i} lacks `samples` or `sampled.win_rate`")
    return problems


class Run:
    """A run's directory (design §1.1): `config.toml` (the file as given), `metrics.jsonl` (one
    typed record per line, flushed), `record.json` (rewritten atomically by `write_record`).
    Without a directory it keeps the record in memory only."""

    def __init__(
        self,
        out: Path | None,
        config: Config,
        identity: dict[str, Any],
        text: str = "",
        log: Callable[[dict[str, Any]], None] | None = None,
    ) -> None:
        self.out = None if out is None else Path(out)
        name = config.name or (self.out.name if self.out is not None else "")
        self.record = Record(name, config, identity)
        self.log = log
        self.metrics: IO[str] | None = None
        self.start = time.perf_counter()
        if self.out is not None:
            self.out.mkdir(parents=True, exist_ok=True)
            for stale in self.out.glob("*.tmp"):
                stale.unlink()
            atomic_write_text(self.out / "config.toml", text)
            self.metrics = (self.out / "metrics.jsonl").open("a", encoding="utf-8")

    def minutes(self) -> float:
        return (time.perf_counter() - self.start) / 60

    def emit(self, record: dict[str, Any]) -> None:
        """Appends one typed record (`type` first) and shows it to the log."""
        self.record.observe(record)
        if self.metrics is not None:
            self.metrics.write(json.dumps(record) + "\n")
            self.metrics.flush()
        if self.log is not None:
            self.log(record)

    def write_record(self) -> None:
        if self.out is not None:
            text = json.dumps(self.record.summary(), indent=1) + "\n"
            atomic_write_text(self.out / "record.json", text)

    def close(self) -> None:
        if self.metrics is not None:
            self.metrics.close()
            self.metrics = None
