"""Each seat's bookkeeping for self-play (E05 T6a, `docs/epics/E05-design.md` §3, §5).

In a batch both sides of an env may be the caller's (self-play), and they do not alternate: a
side may act several times in a row (morale, the wait phase, a stack per turn). A side's
transition opens when it acts and closes at its next decision or at the episode's end, carrying
the rewards of every call in between (reward v1 is terminal: only the call that decides a battle
pays, and it pays both sides). `SeatBook` keeps that per env and side with numpy arrays, so a
trainer can keep one trajectory per (env, side). A side that never acted in an episode has no
transition, so nothing carries its reward.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from .batch import BattleBatch


@dataclass
class Closed:
    """The transitions a call closed, [N, 2] each (env, side)."""

    #: A transition of this env's side closed in the call.
    closed: np.ndarray
    #: Its reward: the sum of the side's rewards since it acted (0 where nothing closed).
    reward: np.ndarray
    #: It closed because the episode ended (decided or truncated); else at the side's next
    #: decision, whose observation is the batch's current `obs` row.
    terminal: np.ndarray
    #: The episode was truncated: a trainer may bootstrap from `final_obs[env, side]`.
    truncated: np.ndarray


class SeatBook:
    """The open transition of each (env, side) between its action and its next decision.

    Per call: `act(batch)` before `step` (the sides about to act open a transition), then
    `settle(batch)` after it (returns what closed). A rejected action must not be settled: the
    batch raises on one by default (`BattleBatch.step(check=True)`).
    """

    def __init__(self, envs: int) -> None:
        self.open = np.zeros((envs, 2), bool)
        self.returns = np.zeros((envs, 2), np.float32)
        self._rows = np.arange(envs)

    def act(self, batch: BattleBatch) -> None:
        """Opens a transition for each env's acting side (an env that gave up has none)."""
        acting = batch.to_move >= 0
        rows = self._rows[acting]
        sides = batch.to_move[acting].astype(np.intp)
        self.open[rows, sides] = True
        self.returns[rows, sides] = 0.0

    def settle(self, batch: BattleBatch) -> Closed:
        """Adds the call's rewards to the open transitions and closes those that ended: both sides'
        at an episode's end, the acting side's at its next decision."""
        self.returns += np.where(self.open, batch.reward, 0.0)
        done = batch.done.astype(bool)
        terminal = self.open & done[:, None]
        next_decision = np.zeros_like(self.open)
        acting = ~done & (batch.to_move >= 0)
        rows = self._rows[acting]
        sides = batch.to_move[acting].astype(np.intp)
        next_decision[rows, sides] = self.open[rows, sides]
        closed = terminal | next_decision
        out = Closed(
            closed=closed,
            reward=np.where(closed, self.returns, 0.0).astype(np.float32),
            terminal=terminal,
            truncated=terminal & batch.truncated.astype(bool)[:, None],
        )
        self.open &= ~closed
        self.returns[closed] = 0.0
        return out
