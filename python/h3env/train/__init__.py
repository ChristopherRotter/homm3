"""The training (E05 T8, `docs/epics/E05-log.md` §8; E06, ADR-0026): an own PPO over the batch's
factorized, conditioned heads, trained by self-play on the encounter distribution v1, promoted by
an SPRT gate against its best and measured by ladders against the engine's bots on held-out
episodes (the sampled policy).

    uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-1

It needs the group `train` (PyTorch); `import h3env` itself stays free of torch, and so does
`h3env.train.run` (the config, the records): the torch modules load on first use.
"""

from __future__ import annotations

import importlib

from .run import Config

#: The names this package exports from its torch modules, loaded on first use (PEP 562).
_LAZY = {
    "EVAL_SEED": "evaluate",
    "ladder": "evaluate",
    "gate": "gate",
    "Choice": "policy",
    "Policy": "policy",
    "Rollout": "ppo",
    "gae": "ppo",
    "train": "ppo",
    "resume": "ppo",
}

__all__ = ["Config", *_LAZY]


def __getattr__(name: str):
    module = _LAZY.get(name)
    if module is None:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
    return getattr(importlib.import_module(f".{module}", __name__), name)
