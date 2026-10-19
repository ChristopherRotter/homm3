"""The smoke training (E05 T8, `docs/epics/E05-design.md` §8): an own PPO over the batch's
factorized, conditioned heads, trained by self-play on the encounter distribution v1 and evaluated
against the engine's Random on held-out episodes.

    uv run --group train python -m h3env.train --minutes 60 --out runs/smoke

It needs the group `train` (PyTorch); `import h3env` itself stays free of torch.
"""

from .evaluate import EVAL_SEED, evaluate
from .policy import Choice, Policy
from .ppo import Config, Rollout, gae, train

__all__ = ["EVAL_SEED", "Choice", "Config", "Policy", "Rollout", "evaluate", "gae", "train"]
