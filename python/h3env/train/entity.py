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

Three opt-in keys of every structured kind (this network and `hexnet`'s; E25 T25, each off by
default: a network without them has the parameters and computes the outputs it did before):

- `[network] mask_input` (`Extra`, `mask_features`): the rows' legal entry mask as inputs. Two hex
  planes, the legal moves and the hexes some legal attack is made from (`geometry.attack_hexes`),
  through a linear layer into each hex's vector (here inside its sum before the ReLU); each stack
  slot's 27 entries (its 24 attacks, 2 shots, heal) as 0/1 through a linear layer into its token;
  the 15 fixed entries (`FIXED`'s) through a linear layer into the global token.
- `[network] order`: each stack slot's three turn-order places (`layout.ORDER`, after `symlog`)
  through a linear layer into its token.
- `[network] attack_hex` (`AttackHex`): a melee entry also scores the hex its attack is made
  from: each legal attack entry's logit gains `out(ReLU(W_s stack + W_h hex + W_q context))` of
  its target stack's vector, its from-hex's vector and the context, the output layer initialised
  near 0. The encoding then carries the legal attack entries' from-hexes as a fourth tensor.

With `mask_input` or `attack_hex` the network reads the entry mask (`reads_mask`); the attack
positions are those of the config's field (`[network] field_width`, `field_height`).
"""

from __future__ import annotations

from typing import NamedTuple

import torch
from torch import nn
from torch.nn import functional as F

from .. import _ffi
from ..batch import BattleBatch
from . import layout as L
from .geometry import FIELD_HEIGHT, FIELD_WIDTH, attack_hexes, attack_table
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
#: The hex planes of the entry mask (`mask_features`): the legal moves, the legal attacks'
#: from-hexes.
MASK_PLANES = 2


def mask_features(mask: torch.Tensor, legal_from: torch.Tensor, hexes: int):
    """The entry mask as the structured kinds read it (`[network] mask_input`, E25 T25; module
    docstring), from `mask` [rows, ACTION_COUNT] as 0/1 of the inputs' dtype and the legal attack
    entries' from-hexes `legal_from` [rows, 42, 24] (-1 elsewhere): the first `hexes` hexes' two
    planes [rows, hexes, 2] (the move to it is legal, some legal attack is made from it), each
    stack slot's entries [rows, 42, 27] (`PER_STACK`'s: its attacks, shots, heal) and the fixed
    entries [rows, 15] (`FIXED`'s: Wait, Defend, End tactics, Retreat, Surrender, the casts, the
    wall shots)."""
    rows, slots = mask.shape[0], L.STACK_CAPACITY
    moves = mask[:, L.MOVE_BASE : L.MOVE_BASE + hexes]
    # Hex h + 1 of a scratch row is 1 when a legal attack is made from h; column 0 takes the -1s.
    froms = mask.new_zeros(rows, L.MAX_FIELD_HEXES + 1)
    froms = froms.scatter(1, (legal_from + 1).flatten(1), 1.0)[:, 1 : hexes + 1]
    per_slot = torch.cat(
        [
            mask[:, L.ATTACK_BASE : L.SHOOT_BASE].reshape(rows, slots, L.ATTACK_SLOTS),
            mask[:, L.SHOOT_BASE : L.HEAL_BASE].reshape(rows, slots, 2),
            mask[:, L.HEAL_BASE : L.END_TACTICS].unsqueeze(2),
        ],
        dim=2,
    )
    fixed = torch.cat([mask[:, :2], mask[:, L.END_TACTICS :]], dim=1)
    return torch.stack([moves, froms], dim=2), per_slot, fixed


class Extra(NamedTuple):
    """The opt-in inputs' terms (E25 T25; `Pointed.extra`), None where a key is off: added to
    the hexes' vectors [rows, hexes, hex width], to the stack slots' tokens [rows, 42, width], to
    the global vector [rows, width]; the legal attack entries' from-hexes [rows, 42, 24] (-1
    elsewhere), the encoding's fourth tensor with `attack_hex`."""

    hexes: torch.Tensor | None = None
    stacks: torch.Tensor | None = None
    glob: torch.Tensor | None = None
    legal_from: torch.Tensor | None = None


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


class AttackHex(nn.Module):
    """The from-hex score of the melee entries (`[network] attack_hex`, E25 T25): a legal attack
    entry's `out(ReLU(W_s stack + W_h hex + W_q context))` of its target slot's vector, its
    from-hex's vector and the context, computed for the legal entries only (as `Mlp.with_preview`
    is) and 0 for the others."""

    def __init__(self, width: int, hex_width: int, hidden: int) -> None:
        super().__init__()
        self.stack = nn.Linear(width, hidden)
        self.hex = nn.Linear(hex_width, hidden, bias=False)
        self.query = nn.Linear(width, hidden, bias=False)
        self.out = init(nn.Linear(hidden, 1), 0.01)

    def forward(
        self,
        context: torch.Tensor,
        stacks: torch.Tensor,
        hexes: torch.Tensor,
        legal_from: torch.Tensor,
    ) -> torch.Tensor:
        """The attack entries' terms [rows, 42 * 24] (`legal_from` [rows, 42, 24]: each legal
        entry's from-hex, -1 elsewhere)."""
        row, slot, k = torch.nonzero(legal_from >= 0, as_tuple=True)
        at = legal_from[row, slot, k]
        hidden = self.stack(stacks[row, slot]) + self.hex(hexes[row, at])
        score = self.out(torch.relu(hidden + self.query(context)[row])).squeeze(1)
        out = score.new_zeros(legal_from.shape)
        return out.index_put((row, slot, k), score).flatten(1)


class Block(nn.Module):
    """A pre-norm transformer block: self-attention over the tokens a row's mask lets through,
    then a feed-forward layer of `feed_forward` times the width (twice before E25 T24), each
    added to its input."""

    def __init__(self, width: int, heads: int, feed_forward: int = 2) -> None:
        super().__init__()
        self.heads = heads
        self.norm1 = nn.LayerNorm(width)
        self.qkv = nn.Linear(width, 3 * width)
        self.proj = nn.Linear(width, width)
        self.norm2 = nn.LayerNorm(width)
        inner = feed_forward * width
        self.ff = nn.Sequential(nn.Linear(width, inner), nn.ReLU(), nn.Linear(inner, width))

    def forward(self, x: torch.Tensor, mask: torch.Tensor) -> torch.Tensor:
        """`x` [rows, tokens, width]; `mask` [rows, tokens] bool, the tokens attended to."""
        rows, tokens, width = x.shape
        qkv = self.qkv(self.norm1(x)).view(rows, tokens, 3, self.heads, width // self.heads)
        q, k, v = qkv.permute(2, 0, 3, 1, 4)
        a = F.scaled_dot_product_attention(q, k, v, attn_mask=mask[:, None, None, :])
        x = x + self.proj(a.transpose(1, 2).reshape(rows, tokens, width))
        return x + self.ff(self.norm2(x))


class Pointed(Policy):
    """The heads of the networks that read the battle as stacks and hexes (the entity network,
    `hexnet`'s kinds): from an encoding of the context [rows, width], the stack slots' vectors
    [rows, 42, width] and the hexes' vectors [rows, 255, hex width] the value and the fixed
    entries come from the context, a move points at its hex's vector, an attack, a shot or a heal
    at its stack's, a cast's target at "none", a hex or a stack (module docstring). A kind makes
    its trunk, then `make_heads` (their names and order are the entity network's: its files
    load), then `make_extra` (the opt-in keys' modules, E25 T25)."""

    #: Whether the encoding reads the entry mask as inputs (`[network] mask_input`).
    mask_input = False
    #: Whether the stack tokens read the turn order (`[network] order`).
    reads_order = False
    #: Whether the melee entries score their from-hex (`[network] attack_hex`; `AttackHex`).
    attack_hex = False

    def make_heads(self, width: int, hex_width: int, embed: int) -> None:
        """Makes the heads for vectors of `width` (the context, the stacks) and `hex_width`."""
        self.value_head = init(nn.Linear(width, 1), 1.0)
        self.fixed_head = init(nn.Linear(width, FIXED), 0.01)
        self.move_head = Pointer(hex_width, width, width)
        self.stack_head = Pointer(width, width, width, PER_STACK)
        self.spell_head = init(nn.Linear(width + 2, L.SPELL_COUNT), 0.01)
        self.spell_embed = nn.Embedding(L.SPELL_COUNT, embed)
        query = width + 2 + embed
        self.target_none = init(nn.Linear(query, 1), 0.01)
        self.target_hex = Pointer(hex_width, query, width)
        self.target_stack = Pointer(width, query, width)
        self.target_embed = nn.Embedding(L.TARGET_COUNT, embed)
        self.second_none = init(nn.Linear(query + embed, 1), 0.01)
        self.second_hex = Pointer(hex_width, query + embed, width)
        self.second_stack = Pointer(width, query + embed, width)

    def make_extra(
        self,
        width: int,
        hex_width: int,
        mask_input: bool,
        order: bool,
        attack_hex: bool,
        field: tuple[int, int],
    ) -> None:
        """The opt-in keys' modules (module docstring), made after the others (so a network with
        a key starts with the others' weights of the one without): `mask_input`'s linear layers
        into the hexes' vectors (`hex_width`), the stacks' tokens and the global (`width`),
        `order`'s into the stacks' tokens, `attack_hex`'s head; the attack positions' table of
        the `field` (columns, rows) where the mask is read, a buffer saved files lack."""
        self.mask_input, self.reads_order, self.attack_hex = mask_input, order, attack_hex
        self.reads_mask = mask_input or attack_hex
        if self.reads_mask:
            self.field_width = field[0]
            table = torch.from_numpy(attack_table(*field))
            self.register_buffer("attack_table", table, persistent=False)
        if mask_input:
            self.mask_hex_in = nn.Linear(MASK_PLANES, hex_width)
            self.mask_stack_in = nn.Linear(PER_STACK, width)
            self.mask_fixed_in = nn.Linear(FIXED, width)
        if order:
            self.order_in = nn.Linear(L.ORDER_FEATURES, width)
        if attack_hex:
            self.attack_hex_head = AttackHex(width, hex_width, width)

    def extra(
        self, obs: torch.Tensor, x: torch.Tensor, mask: torch.Tensor | None, hexes: int
    ) -> Extra:
        """The opt-in keys' terms (`Extra`) for the rows `obs` (`x` their `symlog`, `mask` their
        entry mask; the first `hexes` hexes' vectors)."""
        if not (self.reads_mask or self.reads_order):
            return Extra()
        rows = obs.shape[0]
        stacks = glob = planes = legal_from = None
        if self.reads_order:
            order = x[:, L.ORDER : L.OBSERVATION_LEN].reshape(rows, L.STACK_CAPACITY, -1)
            stacks = self.order_in(order)
        if self.reads_mask:
            inputs = self.mask_inputs(mask, x.dtype)
            attack_from = attack_hexes(obs, self.attack_table, self.field_width)
            legal = inputs[:, L.ATTACK_BASE : L.SHOOT_BASE].reshape(attack_from.shape) > 0.5
            legal_from = torch.where(legal, attack_from, torch.full_like(attack_from, -1))
            if self.mask_input:
                hex_planes, per_slot, fixed = mask_features(inputs, legal_from, hexes)
                planes = self.mask_hex_in(hex_planes)
                slot = self.mask_stack_in(per_slot)
                stacks = slot if stacks is None else stacks + slot
                glob = self.mask_fixed_in(fixed)
        return Extra(planes, stacks, glob, legal_from if self.attack_hex else None)

    def value(self, encoding: Encoding) -> torch.Tensor:
        return self.value_head(encoding[0]).squeeze(1)

    def entry_logits(self, encoding: Encoding) -> torch.Tensor:
        """The entries in action encoding v13's order (`layout`)."""
        context, stacks, hexes = encoding[:3]
        fixed = self.fixed_head(context)
        per_stack = self.stack_head(stacks, context)
        attacks = per_stack[..., : L.ATTACK_SLOTS].flatten(1)
        if self.attack_hex:
            attacks = attacks + self.attack_hex_head(context, stacks, hexes, encoding[3])
        return torch.cat(
            [
                fixed[:, :2],
                self.move_head(hexes, context).squeeze(2),
                attacks,
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
        stacks, hexes = encoding[1:3]
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


def check_layout(batch: BattleBatch, name: str) -> None:
    """Refuses a batch whose encodings are not the layout's (`name` the network's, for the
    message)."""
    versions = _ffi.versions()
    sizes = (batch.observation_len, batch.action_count, batch.spell_count, batch.target_count)
    expected = (L.OBSERVATION_LEN, L.ACTION_COUNT, L.SPELL_COUNT, L.TARGET_COUNT)
    if (
        versions["observation_encoding"] != L.OBSERVATION_ENCODING
        or versions["action_encoding"] != L.ACTION_ENCODING
        or sizes != expected
    ):
        raise ValueError(
            f"the {name} reads observation encoding "
            f"v{L.OBSERVATION_ENCODING} and action encoding v{L.ACTION_ENCODING} "
            f"{expected}; the library has v{versions['observation_encoding']}, "
            f"v{versions['action_encoding']} {sizes}"
        )


class Entity(Pointed):
    """The entity network of design §11, with E25 T25's opt-in keys (module docstring)."""

    def __init__(
        self,
        width: int = 128,
        layers: int = 2,
        heads: int = 4,
        embed: int = 32,
        feed_forward: int = 2,
        mask_input: bool = False,
        order: bool = False,
        attack_hex: bool = False,
        field: tuple[int, int] = (FIELD_WIDTH, FIELD_HEIGHT),
    ) -> None:
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
        self.blocks = nn.ModuleList(Block(width, heads, feed_forward) for _ in range(layers))
        self.norm = nn.LayerNorm(width)
        self.context = nn.Linear(3 * width, width)
        self.make_heads(width, HEX_WIDTH, embed)
        self.make_extra(width, HEX_WIDTH, mask_input, order, attack_hex, field)

    @classmethod
    def for_batch(
        cls,
        batch: BattleBatch,
        width: int = 128,
        layers: int = 2,
        heads: int = 4,
        embed: int = 32,
        feed_forward: int = 2,
        mask_input: bool = False,
        order: bool = False,
        attack_hex: bool = False,
        field: tuple[int, int] = (FIELD_WIDTH, FIELD_HEIGHT),
    ) -> Entity:
        """The network for `batch`, whose encodings must be the layout's (the opt-in keys and
        the field of the attack positions as `Pointed.make_extra`'s)."""
        check_layout(batch, "entity network")
        return cls(width, layers, heads, embed, feed_forward, mask_input, order, attack_hex, field)

    def encode(self, obs: torch.Tensor, mask: torch.Tensor | None = None) -> Encoding:
        """The context [rows, width], the stacks' tokens [rows, 42, width] and the hexes'
        vectors [rows, 255, HEX_WIDTH], with `attack_hex` the legal attack entries' from-hexes
        [rows, 42, 24]; `mask` (the entry mask) is read with `mask_input` or `attack_hex`."""
        rows = obs.shape[0]
        x = symlog(obs)
        extra = self.extra(obs, x, mask, L.MAX_FIELD_HEXES)
        raw = obs[:, L.STACKS : L.ORDER].view(rows, L.STACK_CAPACITY, L.STACK_FEATURES)
        present = raw[..., L.PRESENT] > 0.5
        stacks = (
            self.stack_in(x[:, L.STACKS : L.ORDER].view(rows, L.STACK_CAPACITY, L.STACK_FEATURES))
            + self.creature_embed(raw[..., L.CREATURE].long().clamp(0, CREATURES - 1))
            + self.column_embed(raw[..., L.X].long().clamp(0, COLUMNS - 1))
            + self.row_embed(raw[..., L.Y].long().clamp(0, ROWS - 1))
        )
        if extra.stacks is not None:
            stacks = stacks + extra.stacks
        planes = x[:, L.FIELD : L.STACKS].view(rows, L.HEX_PLANES, L.MAX_FIELD_HEXES)
        hexes = self.hex_in(planes.transpose(1, 2)) + self.hex_embed
        if extra.hexes is not None:
            hexes = hexes + extra.hexes
        hexes = torch.relu(hexes)
        glob = self.global_in(x[:, : L.OWN_HERO])
        if extra.glob is not None:
            glob = glob + extra.glob
        tokens = torch.cat(
            [
                glob.unsqueeze(1),
                self.own_in(x[:, L.OWN_HERO : L.FIELD]).unsqueeze(1),
                self.field_in(hexes.flatten(1)).unsqueeze(1),
                stacks,
            ],
            dim=1,
        )
        attended = torch.cat([present.new_ones(rows, LEADING), present], dim=1)
        for block in self.blocks:
            tokens = block(tokens, attended)
        tokens = self.norm(tokens)
        stacks = tokens[:, LEADING:]
        weights = present.to(tokens.dtype).unsqueeze(2)
        active = ((raw[..., L.ACTIVE] > 0.5) & present).to(tokens.dtype).unsqueeze(2)
        mean = (stacks * weights).sum(1) / weights.sum(1).clamp(min=1.0)
        context = torch.relu(
            self.context(torch.cat([tokens[:, 0], (stacks * active).sum(1), mean], dim=1))
        )
        if extra.legal_from is None:
            return context, stacks, hexes
        return context, stacks, hexes, extra.legal_from
