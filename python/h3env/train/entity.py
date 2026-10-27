"""The entity network (E06 T5a, `docs/epics/E06-log.md` §11; decision 5): observation v21 read
as entities by its layout (`layout`), with no encoding change.

Tokens of `width` features: one per stack slot (its features under `symlog` through a linear
layer, plus learned embeddings of its creature's handle and its column and row; an empty slot is
masked out of attention), the global token (the globals and both public hero parts), the own-hero
token (the viewer's private part) and the field token (each hex's planes through a shared layer
with a learned per-hex embedding, the hex vectors flattened through a linear layer). `layers`
pre-norm transformer blocks run over them; the context is the global token's output, the active
stack's and the mean of the present stacks'. The heads point: a move at its hex's vector, an
attack, a shot or a heal at its stack's token, a target at "none", a hex or a stack, each scored
against a query (`Pointer`).
"""

from __future__ import annotations

import torch
from torch import nn
from torch.nn import functional as F

from .. import _ffi
from ..batch import BattleBatch
from . import layout as L
from .policy import Encoding, Policy, init, symlog

#: The embeddings' sizes: creature handles (a larger one shares the last), columns and rows (an
#: off-field turret's included).
CREATURES = 256
COLUMNS = 32
ROWS = 16
#: Features of a hex's vector.
HEX_WIDTH = 16
#: The entries scored from the context: Wait, Defend, End tactics, Retreat, Surrender, the two
#: casts, the wall shots.
FIXED = 2 + 5 + L.MAX_WALL_TARGETS
#: The entries scored per stack: the attacks from each attack position, the two shots, the heal.
PER_STACK = L.ATTACK_SLOTS + 2 + 1
#: The tokens before the stacks': the global, the own hero's, the field's.
LEADING = 3


class Pointer(nn.Module):
    """Additive attention scores (Bahdanau): `out` logits per key from `ReLU(W_k key + W_q
    query)` through a linear layer."""

    def __init__(self, key: int, query: int, hidden: int, out: int = 1) -> None:
        super().__init__()
        self.key = nn.Linear(key, hidden)
        self.query = nn.Linear(query, hidden, bias=False)
        self.out = init(nn.Linear(hidden, out), 0.01)

    def forward(self, keys: torch.Tensor, query: torch.Tensor) -> torch.Tensor:
        """Logits [rows, keys, out] of `keys` [rows, keys, key] against `query` [rows, query]."""
        return self.out(torch.relu(self.key(keys) + self.query(query).unsqueeze(1)))


class Block(nn.Module):
    """A pre-norm transformer block: self-attention over the tokens a row's mask lets through,
    then a feed-forward layer of twice the width, each added to its input."""

    def __init__(self, width: int, heads: int) -> None:
        super().__init__()
        self.heads = heads
        self.norm1 = nn.LayerNorm(width)
        self.qkv = nn.Linear(width, 3 * width)
        self.proj = nn.Linear(width, width)
        self.norm2 = nn.LayerNorm(width)
        self.ff = nn.Sequential(nn.Linear(width, 2 * width), nn.ReLU(), nn.Linear(2 * width, width))

    def forward(self, x: torch.Tensor, mask: torch.Tensor) -> torch.Tensor:
        """`x` [rows, tokens, width]; `mask` [rows, tokens] bool, the tokens attended to."""
        rows, tokens, width = x.shape
        qkv = self.qkv(self.norm1(x)).view(rows, tokens, 3, self.heads, width // self.heads)
        q, k, v = qkv.permute(2, 0, 3, 1, 4)
        a = F.scaled_dot_product_attention(q, k, v, attn_mask=mask[:, None, None, :])
        x = x + self.proj(a.transpose(1, 2).reshape(rows, tokens, width))
        return x + self.ff(self.norm2(x))


class Entity(Policy):
    """The entity network of design §11 (module docstring)."""

    def __init__(self, width: int = 128, layers: int = 2, heads: int = 4, embed: int = 32) -> None:
        super().__init__()
        if width % heads:
            raise ValueError(f"the width {width} is no multiple of the heads {heads}")
        self.width = width
        self.stack_in = nn.Linear(L.STACK_FEATURES, width)
        self.creature_embed = nn.Embedding(CREATURES, width)
        self.column_embed = nn.Embedding(COLUMNS, width)
        self.row_embed = nn.Embedding(ROWS, width)
        self.global_in = nn.Linear(L.OWN_HERO, width)
        self.own_in = nn.Linear(L.OWN_HERO_FEATURES, width)
        self.hex_in = nn.Linear(L.HEX_PLANES, HEX_WIDTH)
        self.hex_embed = nn.Parameter(torch.zeros(L.MAX_FIELD_HEXES, HEX_WIDTH))
        nn.init.normal_(self.hex_embed, std=0.02)
        self.field_in = nn.Linear(L.MAX_FIELD_HEXES * HEX_WIDTH, width)
        self.blocks = nn.ModuleList(Block(width, heads) for _ in range(layers))
        self.norm = nn.LayerNorm(width)
        self.context = nn.Linear(3 * width, width)
        self.value_head = init(nn.Linear(width, 1), 1.0)
        self.fixed_head = init(nn.Linear(width, FIXED), 0.01)
        self.move_head = Pointer(HEX_WIDTH, width, width)
        self.stack_head = Pointer(width, width, width, PER_STACK)
        self.spell_head = init(nn.Linear(width + 2, L.SPELL_COUNT), 0.01)
        self.spell_embed = nn.Embedding(L.SPELL_COUNT, embed)
        query = width + 2 + embed
        self.target_none = init(nn.Linear(query, 1), 0.01)
        self.target_hex = Pointer(HEX_WIDTH, query, width)
        self.target_stack = Pointer(width, query, width)
        self.target_embed = nn.Embedding(L.TARGET_COUNT, embed)
        self.second_none = init(nn.Linear(query + embed, 1), 0.01)
        self.second_hex = Pointer(HEX_WIDTH, query + embed, width)
        self.second_stack = Pointer(width, query + embed, width)

    @classmethod
    def for_batch(
        cls,
        batch: BattleBatch,
        width: int = 128,
        layers: int = 2,
        heads: int = 4,
        embed: int = 32,
    ) -> Entity:
        """The network for `batch`, whose encodings must be the layout's."""
        versions = _ffi.versions()
        sizes = (batch.observation_len, batch.action_count, batch.spell_count, batch.target_count)
        expected = (L.OBSERVATION_LEN, L.ACTION_COUNT, L.SPELL_COUNT, L.TARGET_COUNT)
        if (
            versions["observation_encoding"] != L.OBSERVATION_ENCODING
            or versions["action_encoding"] != L.ACTION_ENCODING
            or sizes != expected
        ):
            raise ValueError(
                "the entity network reads observation encoding "
                f"v{L.OBSERVATION_ENCODING} and action encoding v{L.ACTION_ENCODING} "
                f"{expected}; the library has v{versions['observation_encoding']}, "
                f"v{versions['action_encoding']} {sizes}"
            )
        return cls(width, layers, heads, embed)

    def encode(self, obs: torch.Tensor) -> Encoding:
        """The context [rows, width], the stacks' tokens [rows, 42, width] and the hexes'
        vectors [rows, 255, HEX_WIDTH]."""
        rows = obs.shape[0]
        x = symlog(obs)
        raw = obs[:, L.STACKS : L.ORDER].view(rows, L.STACK_CAPACITY, L.STACK_FEATURES)
        present = raw[..., L.PRESENT] > 0.5
        stacks = (
            self.stack_in(x[:, L.STACKS : L.ORDER].view(rows, L.STACK_CAPACITY, L.STACK_FEATURES))
            + self.creature_embed(raw[..., L.CREATURE].long().clamp(0, CREATURES - 1))
            + self.column_embed(raw[..., L.X].long().clamp(0, COLUMNS - 1))
            + self.row_embed(raw[..., L.Y].long().clamp(0, ROWS - 1))
        )
        planes = x[:, L.FIELD : L.STACKS].view(rows, L.HEX_PLANES, L.MAX_FIELD_HEXES)
        hexes = torch.relu(self.hex_in(planes.transpose(1, 2)) + self.hex_embed)
        tokens = torch.cat(
            [
                self.global_in(x[:, : L.OWN_HERO]).unsqueeze(1),
                self.own_in(x[:, L.OWN_HERO : L.FIELD]).unsqueeze(1),
                self.field_in(hexes.flatten(1)).unsqueeze(1),
                stacks,
            ],
            dim=1,
        )
        mask = torch.cat([present.new_ones(rows, LEADING), present], dim=1)
        for block in self.blocks:
            tokens = block(tokens, mask)
        tokens = self.norm(tokens)
        stacks = tokens[:, LEADING:]
        weights = present.to(tokens.dtype).unsqueeze(2)
        active = ((raw[..., L.ACTIVE] > 0.5) & present).to(tokens.dtype).unsqueeze(2)
        mean = (stacks * weights).sum(1) / weights.sum(1).clamp(min=1.0)
        context = torch.relu(
            self.context(torch.cat([tokens[:, 0], (stacks * active).sum(1), mean], dim=1))
        )
        return context, stacks, hexes

    def value(self, encoding: Encoding) -> torch.Tensor:
        return self.value_head(encoding[0]).squeeze(1)

    def entry_logits(self, encoding: Encoding) -> torch.Tensor:
        """The entries in action encoding v13's order (`layout`)."""
        context, stacks, hexes = encoding
        fixed = self.fixed_head(context)
        per_stack = self.stack_head(stacks, context)
        return torch.cat(
            [
                fixed[:, :2],
                self.move_head(hexes, context).squeeze(2),
                per_stack[..., : L.ATTACK_SLOTS].flatten(1),
                per_stack[..., L.ATTACK_SLOTS : L.ATTACK_SLOTS + 2].flatten(1),
                per_stack[..., -1],
                fixed[:, 2:],
            ],
            dim=1,
        )

    def spell_logits(self, encoding: Encoding, cast: torch.Tensor) -> torch.Tensor:
        return self.spell_head(torch.cat([encoding[0], cast], dim=1))

    def _targets(self, encoding: Encoding, query: torch.Tensor, none, hex_, stack) -> torch.Tensor:
        """A target head's logits: none, every hex, every stack slot."""
        _, stacks, hexes = encoding
        return torch.cat(
            [none(query), hex_(hexes, query).squeeze(2), stack(stacks, query).squeeze(2)], dim=1
        )

    def target_logits(self, encoding, cast, spell):
        query = torch.cat([encoding[0], cast, self.spell_embed(spell)], dim=1)
        return self._targets(encoding, query, self.target_none, self.target_hex, self.target_stack)

    def second_logits(self, encoding, cast, spell, target):
        query = torch.cat(
            [encoding[0], cast, self.spell_embed(spell), self.target_embed(target)], dim=1
        )
        return self._targets(encoding, query, self.second_none, self.second_hex, self.second_stack)
