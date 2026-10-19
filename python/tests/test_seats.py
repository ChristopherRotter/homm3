"""Each seat's transitions in self-play (E05 T6a): every action opens one, each closes once, at
the side's next decision or the episode's end, and the episode's rewards reach both sides."""

from __future__ import annotations

import numpy as np
from conftest import scenario

from h3env import BattleBatch, SeatBook


def play(batch: BattleBatch, calls: int, seed: int):
    book = SeatBook(batch.envs)
    rng = np.random.default_rng(seed)
    opened = closed = 0
    for _ in range(calls):
        acting = batch.to_move.copy()
        reopened = (book.open[np.arange(batch.envs), acting.clip(0)] & (acting >= 0)).sum()
        assert reopened == 0, "a side acts while its transition is open"
        book.act(batch)
        opened += int((acting >= 0).sum())
        batch.random_actions(rng)
        batch.step()
        out = book.settle(batch)
        closed += int(out.closed.sum())
        yield book, out
    assert opened == closed + int(book.open.sum())


def test_self_play_closes_every_transition_once() -> None:
    batch = BattleBatch(8, threads=2, seed=6, max_steps=150)
    batch.add_scenario(scenario("skirmish"))
    terminal_rewards = 0
    batch.reset()
    for book, out in play(batch, 400, 1):
        done = batch.done.astype(bool)
        # The acting side's transition closes at once unless the other side moves next.
        for i in np.nonzero(~done)[0]:
            side = batch.to_move[i]
            assert out.closed[i].sum() <= 1
            if out.closed[i].any():
                assert out.closed[i, side] and not out.terminal[i].any()
        # Reward v1 is terminal: a transition that closes before the end carries 0.
        assert (out.reward[~out.terminal] == 0).all()
        # At an episode's end every open transition closes, and in self-play the two sides'
        # rewards cancel when both acted.
        for i in np.nonzero(done)[0]:
            assert not book.open[i].any()
            assert out.terminal[i].any()
            if out.terminal[i].all():
                assert out.reward[i, 0] == -out.reward[i, 1]
                terminal_rewards += int(out.reward[i, 0] != 0)
            assert (out.truncated[i] == (out.terminal[i] & bool(batch.truncated[i]))).all()
    assert terminal_rewards > 0


def test_an_internal_seat_has_no_transitions() -> None:
    batch = BattleBatch(4, seed=8, max_steps=200)
    batch.add_scenario(scenario("full"), defender="random")
    batch.reset()
    for book, out in play(batch, 200, 2):
        assert not book.open[:, 1].any() and not out.closed[:, 1].any()
