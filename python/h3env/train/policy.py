"""The smoke run's network (E05 T8, `docs/epics/E05-design.md` §8.3): an MLP over the flat
observation v21 with a value head and the factorized action's heads, each cast head conditioned
on the earlier choices and masked by the batch's head calls.

A row's action is its entry and, only where the entry is a cast, the spell, the target and the
second target, chosen one head after another; its log-probability is the sum over the heads it
chose. Masked logits take the dtype's lowest value, never `-inf`, so a row with a single legal
choice (or none: an env that gave up) stays finite.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import torch
from torch import nn

from .. import _ffi
from ..batch import BattleBatch

#: The two cast entries, in the order of the cast one-hot the later heads take.
CASTS = np.array([_ffi.HERO_CAST, _ffi.CREATURE_CAST], np.uint32)


def symlog(x: torch.Tensor) -> torch.Tensor:
    """`sign(x) * ln(1 + |x|)`: the observation's counts and hit points (up to 6e5) on a log
    scale, without running statistics."""
    return torch.sign(x) * torch.log1p(torch.abs(x))


def masked(logits: torch.Tensor, mask: torch.Tensor) -> torch.Tensor:
    """`logits` with every entry outside `mask` (bool, same shape) at the dtype's lowest value."""
    return logits.masked_fill(~mask, torch.finfo(logits.dtype).min)


def pick(logits: torch.Tensor, generator: torch.Generator | None, greedy: bool) -> torch.Tensor:
    """One index per row: the most likely (`greedy`), else a draw (Gumbel-max with `generator`)."""
    if greedy:
        return logits.argmax(dim=1)
    u = torch.rand(logits.shape, generator=generator, device=logits.device)
    gumbel = -torch.log(-torch.log(u.clamp(1e-20, 1.0 - 1e-7)))
    return (logits + gumbel).argmax(dim=1)


def chosen(logits: torch.Tensor, index: torch.Tensor) -> torch.Tensor:
    """The log-probability of `index` under each row's (masked) `logits`."""
    return torch.log_softmax(logits, dim=1).gather(1, index.unsqueeze(1)).squeeze(1)


def entropy(logits: torch.Tensor) -> torch.Tensor:
    """Each row's entropy under its masked `logits` (a masked entry adds 0, not NaN)."""
    logp = torch.log_softmax(logits, dim=1)
    return -(logp.exp() * logp).sum(dim=1)


def init(layer: nn.Linear, gain: float) -> nn.Linear:
    """Orthogonal weights with `gain`, zero bias (CleanRL's PPO)."""
    nn.init.orthogonal_(layer.weight, gain)
    nn.init.zeros_(layer.bias)
    return layer


@dataclass
class Choice:
    """The actions `Policy.act` chose for a batch's rows, as numpy arrays [N] (masks [N, K])."""

    entry: np.ndarray
    spell: np.ndarray
    target: np.ndarray
    second: np.ndarray
    #: The row chose a cast: its later heads were chosen (else they are 0).
    cast: np.ndarray
    logprob: np.ndarray
    value: np.ndarray
    #: The head masks of the cast rows (zero rows elsewhere).
    spell_mask: np.ndarray
    target_mask: np.ndarray
    second_mask: np.ndarray


class Policy(nn.Module):
    """The network: `symlog`, a trunk of two `hidden`-unit layers, the value and entry heads on it;
    the spell head takes the trunk and which cast, the target head also the spell's embedding, the
    second head also the target's."""

    def __init__(
        self,
        observation_len: int,
        actions: int,
        spells: int,
        targets: int,
        hidden: int = 512,
        embed: int = 32,
    ) -> None:
        super().__init__()
        root2 = float(np.sqrt(2.0))
        self.trunk = nn.Sequential(
            init(nn.Linear(observation_len, hidden), root2),
            nn.ReLU(),
            init(nn.Linear(hidden, hidden), root2),
            nn.ReLU(),
        )
        self.value_head = init(nn.Linear(hidden, 1), 1.0)
        self.entry_head = init(nn.Linear(hidden, actions), 0.01)
        self.spell_head = init(nn.Linear(hidden + 2, spells), 0.01)
        self.spell_embed = nn.Embedding(spells, embed)
        self.target_head = init(nn.Linear(hidden + 2 + embed, targets), 0.01)
        self.target_embed = nn.Embedding(targets, embed)
        self.second_head = init(nn.Linear(hidden + 2 + 2 * embed, targets), 0.01)

    @classmethod
    def for_batch(cls, batch: BattleBatch, hidden: int = 512, embed: int = 32) -> Policy:
        """A network sized for `batch`'s encodings."""
        return cls(
            batch.observation_len,
            batch.action_count,
            batch.spell_count,
            batch.target_count,
            hidden,
            embed,
        )

    def features(self, obs: torch.Tensor) -> torch.Tensor:
        return self.trunk(symlog(obs))

    def value(self, features: torch.Tensor) -> torch.Tensor:
        return self.value_head(features).squeeze(1)

    def _cast(self, entry: torch.Tensor) -> torch.Tensor:
        """Which cast each row chose, one-hot [rows, 2] (hero, creature)."""
        hero = entry == int(CASTS[0])
        return torch.stack([hero, ~hero], dim=1).to(torch.float32)

    def _spell_logits(self, h: torch.Tensor, cast: torch.Tensor) -> torch.Tensor:
        return self.spell_head(torch.cat([h, cast], dim=1))

    def _target_logits(self, h: torch.Tensor, cast: torch.Tensor, spell: torch.Tensor):
        return self.target_head(torch.cat([h, cast, self.spell_embed(spell)], dim=1))

    def _second_logits(self, h, cast, spell, target):
        x = torch.cat([h, cast, self.spell_embed(spell), self.target_embed(target)], dim=1)
        return self.second_head(x)

    @torch.no_grad()
    def act(
        self,
        batch: BattleBatch,
        generator: torch.Generator | None = None,
        greedy: bool = False,
    ) -> Choice:
        """Chooses an action for every row of `batch` from its current observation and masks: the
        entry, then for the rows that chose a cast its heads through the batch's head calls."""
        device = next(self.parameters()).device
        n = batch.envs
        obs = torch.from_numpy(batch.obs).to(device)
        h = self.features(obs)
        value = self.value(h)
        entry_mask = torch.from_numpy(batch.entry_mask).to(device).bool()
        logits = masked(self.entry_head(h), entry_mask)
        entry = pick(logits, generator, greedy)
        logprob = chosen(logits, entry)
        entries = entry.cpu().numpy().astype(np.uint32)
        cast = np.isin(entries, CASTS)
        spell = np.zeros(n, np.uint16)
        target = np.zeros(n, np.uint16)
        second = np.zeros(n, np.uint16)
        spell_mask = np.zeros((n, batch.spell_count), np.uint8)
        target_mask = np.zeros((n, batch.target_count), np.uint8)
        second_mask = np.zeros((n, batch.target_count), np.uint8)
        if cast.any():
            rows_np = np.nonzero(cast)[0]
            rows = torch.from_numpy(rows_np).to(device)
            hc, c = h[rows], self._cast(entry[rows])

            def head(mask_np: np.ndarray, out_mask: np.ndarray, logits_of) -> torch.Tensor:
                out_mask[rows_np] = mask_np[rows_np]
                m = torch.from_numpy(out_mask[rows_np]).to(device).bool()
                head_logits = masked(logits_of(), m)
                index = pick(head_logits, generator, greedy)
                logprob[rows] += chosen(head_logits, index)
                return index

            s = head(batch.spell_mask(entries), spell_mask, lambda: self._spell_logits(hc, c))
            spell[rows_np] = s.cpu().numpy()
            t = head(
                batch.target_mask(entries, spell),
                target_mask,
                lambda: self._target_logits(hc, c, s),
            )
            target[rows_np] = t.cpu().numpy()
            u = head(
                batch.second_mask(entries, spell, target),
                second_mask,
                lambda: self._second_logits(hc, c, s, t),
            )
            second[rows_np] = u.cpu().numpy()
        return Choice(
            entry=entries,
            spell=spell,
            target=target,
            second=second,
            cast=cast,
            logprob=logprob.cpu().numpy(),
            value=value.cpu().numpy(),
            spell_mask=spell_mask,
            target_mask=target_mask,
            second_mask=second_mask,
        )

    def evaluate(
        self,
        obs: torch.Tensor,
        entry_mask: torch.Tensor,
        entry: torch.Tensor,
        cast: torch.Tensor,
        heads: tuple[torch.Tensor, ...],
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """The log-probability, entropy and value of chosen actions under the current weights.

        `heads` = (spell_mask, spell, target_mask, target, second_mask, second), rows as `obs`
        (masks bool); only the `cast` rows' heads count. The entropy is the entry's plus, for a
        cast, its heads' along the chosen path."""
        spell_mask, spell, target_mask, target, second_mask, second = heads
        h = self.features(obs)
        value = self.value(h)
        logits = masked(self.entry_head(h), entry_mask)
        logprob = chosen(logits, entry)
        ent = entropy(logits)
        rows = torch.nonzero(cast).squeeze(1)
        if rows.numel():
            hc, c = h[rows], self._cast(entry[rows])
            s, t, u = spell[rows], target[rows], second[rows]
            extra_logp = torch.zeros_like(logprob)
            extra_ent = torch.zeros_like(ent)
            for head_logits, mask, index in (
                (self._spell_logits(hc, c), spell_mask, s),
                (self._target_logits(hc, c, s), target_mask, t),
                (self._second_logits(hc, c, s, t), second_mask, u),
            ):
                head_logits = masked(head_logits, mask[rows])
                extra_logp = extra_logp.index_add(0, rows, chosen(head_logits, index))
                extra_ent = extra_ent.index_add(0, rows, entropy(head_logits))
            logprob = logprob + extra_logp
            ent = ent + extra_ent
        return logprob, ent, value
