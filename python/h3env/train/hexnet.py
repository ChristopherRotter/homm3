"""The networks that read the battlefield as hexes (E25 T7a, AI-Q11; `docs/epics/E25-log.md`):
observation v22 read as a field of hexes by its layout (`layout`), with no encoding change, and
the entity network's heads (`entity.Pointed`).

Both kinds draw the battle on the field the same way (`HexNet.draw`): each hex's vector is its
planes through a linear layer plus a learned per-hex embedding, plus the token of each stack
slot that stands on it (`symlog` of its features through a linear layer plus its creature's
embedding; a two-hex stack on its position, its rear hex, and on its other hex, right of it for
the attacker's stack and left for the defender's, `h3_battle::geometry::footprint`), plus the
global vector (the globals, both public hero parts and the viewer's private part, each through a
linear layer) on every hex. A slot without a stack is drawn nowhere and an off-field stack (a
town's turret) only keeps its token. The kinds differ in the trunk over the hexes:

- `HexConv` (`kind = "hexconv"`): the field in axial coordinates (`Grid`: a hex's column
  `x - ceil(y / 2)`, shifted to start at 0; 17 x 11 hexes are 22 x 11 cells, 187 of them real),
  where a 3 x 3 kernel without its corners (+1, +1) and (-1, -1) covers exactly a hex and its six
  neighbours (`HEX_KERNEL`), one ordinary convolution per layer. `layers` residual blocks of two
  such convolutions of `width` channels, the second with a bias from the real cells' mean and
  maximum (global pooling), the branch initialised at 0; the cells off the field stay 0; no batch
  statistics (a frozen copy plays as it trained).
- `HexFormer` (`kind = "hexformer"`): a global token and the field's hexes as tokens, `layers`
  pre-norm transformer blocks (`entity.Block`'s) whose attention adds a learned bias per head
  and axial offset between two hexes (the same for every row).

The heads read the trunk's output: a hex's vector (zero for an index off the field), a stack's
vector (its token plus the trunk's vector at its position, through a linear layer), the context
(the global vector or token, the hexes' mean and maximum, the active stack's and the present
stacks' mean). The field's size is the network's config (`[network] field_width`,
`field_height`; the base ruleset's battlefield, `packs/base/battle.toml`): a stack outside it is
off the field.
"""

from __future__ import annotations

import numpy as np
import torch
from torch import nn
from torch.nn import functional as F

from ..batch import BattleBatch
from . import layout as L
from .entity import CREATURES, Block, Pointed, check_layout
from .policy import Encoding, symlog

#: The base ruleset's battlefield (`[battle] field_width`, `field_height`), the default.
FIELD_WIDTH = 17
FIELD_HEIGHT = 11

#: The hexes a 3 x 3 kernel covers on the axial grid (rows: the hex row's offset -1 to 1; columns:
#: the axial column's): a hex and its six neighbours, `(dq, dr)` (+1, +1) and (-1, -1) not.
HEX_KERNEL = torch.tensor([[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]])


def neighbours(width: int, height: int, hex_: int) -> list[int]:
    """The hexes next to `hex_` on a `width` x `height` field, as `h3_battle::geometry::Field::
    neighbors` finds them (even rows shifted right: their upper and lower neighbours are at x and
    x + 1, an odd row's at x - 1 and x); for the tests."""
    x, y = hex_ % width, hex_ // width
    shift = 0 if y % 2 == 0 else -1
    out = []
    for nx, ny in (
        (x + 1, y),
        (x + shift + 1, y + 1),
        (x + shift, y + 1),
        (x - 1, y),
        (x + shift, y - 1),
        (x + shift + 1, y - 1),
    ):
        if 0 <= nx < width and 0 <= ny < height:
            out.append(ny * width + nx)
    return out


class Grid:
    """A `width` x `height` field's hexes on the axial grid: hex `y * width + x` is the cell of
    row `y`, column `x - ceil(y / 2) + shift` (`shift` = `ceil((height - 1) / 2)`, so the columns
    start at 0)."""

    def __init__(self, width: int, height: int) -> None:
        if width <= 0 or height <= 0 or width * height > L.MAX_FIELD_HEXES:
            raise ValueError(
                f"a {width} x {height} field does not fit the encoding's {L.MAX_FIELD_HEXES} hexes"
            )
        self.width, self.height = width, height
        self.hexes = width * height
        shift = height // 2
        self.columns = width + shift
        x = np.arange(self.hexes) % width
        y = np.arange(self.hexes) // width
        #: Each hex's cell (row-major over `height` x `columns`).
        self.cell = y * self.columns + x - (y + 1) // 2 + shift
        #: Each hex's axial column and row.
        self.q = x - (y + 1) // 2 + shift
        self.r = y


class HexNet(Pointed):
    """What both hex kinds share: the drawing of the battle on the field and the read-out to the
    entity network's heads (module docstring). A kind implements `trunk`."""

    def __init__(self, width: int, embed: int, field_width: int, field_height: int) -> None:
        super().__init__()
        self.width = width
        self.grid = Grid(field_width, field_height)
        hexes = self.grid.hexes
        self.stack_in = nn.Linear(L.STACK_FEATURES, width)
        self.creature_embed = nn.Embedding(CREATURES, width)
        self.global_in = nn.Linear(L.OWN_HERO, width)
        self.own_in = nn.Linear(L.OWN_HERO_FEATURES, width)
        self.hex_in = nn.Linear(L.HEX_PLANES, width)
        self.hex_embed = nn.Parameter(torch.zeros(hexes, width))
        nn.init.normal_(self.hex_embed, std=0.02)
        self.read = nn.Linear(width, width)
        self.context = nn.Linear(5 * width, width)

    def draw(self, obs: torch.Tensor):
        """The rows' hex vectors [rows, hexes, width], the global vector [rows, width], the stack
        slots' tokens [rows, 42, width], each slot's hex (its position; -1 off the field or
        without a stack) and its presence and activity [rows, 42] (bool)."""
        rows = obs.shape[0]
        g = self.grid
        x = symlog(obs)
        raw = obs[:, L.STACKS : L.ORDER].view(rows, L.STACK_CAPACITY, L.STACK_FEATURES)
        present = raw[..., L.PRESENT] > 0.5
        tokens = self.stack_in(
            x[:, L.STACKS : L.ORDER].view(rows, L.STACK_CAPACITY, L.STACK_FEATURES)
        ) + self.creature_embed(raw[..., L.CREATURE].long().clamp(0, CREATURES - 1))
        sx, sy = raw[..., L.X].long(), raw[..., L.Y].long()
        inside = present & (sx >= 0) & (sx < g.width) & (sy >= 0) & (sy < g.height)
        position = torch.where(inside, sy * g.width + sx, torch.full_like(sx, -1))
        # A two-hex stack's other hex: right of its position for the attacker's, left for the
        # defender's (its side: its own flag against the viewer's seat).
        viewer_attacks = obs[:, L.VIEWER_ATTACKER : L.VIEWER_ATTACKER + 1] > 0.5
        attacker = (raw[..., L.OWN] > 0.5) == viewer_attacks
        fx = sx + torch.where(attacker, 1, -1)
        other = inside & (raw[..., L.TWO_HEX] > 0.5) & (fx >= 0) & (fx < g.width)
        planes = x[:, L.FIELD : L.STACKS].view(rows, L.HEX_PLANES, L.MAX_FIELD_HEXES)
        glob = self.global_in(x[:, : L.OWN_HERO]) + self.own_in(x[:, L.OWN_HERO : L.FIELD])
        field = self.hex_in(planes[:, :, : g.hexes].transpose(1, 2)) + self.hex_embed
        field = field + glob.unsqueeze(1)
        base = torch.arange(rows, device=obs.device).unsqueeze(1) * g.hexes
        flat = field.reshape(rows * g.hexes, self.width)
        for at, on in ((position, inside), (sy * g.width + fx, other)):
            flat = flat.index_add(0, (base + at)[on], tokens[on])
        active = (raw[..., L.ACTIVE] > 0.5) & present
        return flat.view(rows, g.hexes, self.width), glob, tokens, position, present, active

    def trunk(self, field: torch.Tensor, glob: torch.Tensor):
        """The hex vectors [rows, hexes, width] and the global vector [rows, width] after the
        kind's layers."""
        raise NotImplementedError

    def encode(self, obs: torch.Tensor) -> Encoding:
        """The context [rows, width], the stacks' vectors [rows, 42, width] and the hexes'
        vectors [rows, 255, width] (zero off the field)."""
        field, glob, tokens, position, present, active = self.draw(obs)
        hexes, glob = self.trunk(field, glob)
        rows = hexes.shape[0]
        at = position.clamp(min=0)
        under = hexes.gather(1, at.unsqueeze(2).expand(-1, -1, self.width))
        under = under * (position >= 0).unsqueeze(2).to(hexes.dtype)
        stacks = tokens + torch.relu(self.read(under))
        weights = present.to(hexes.dtype).unsqueeze(2)
        mean = (stacks * weights).sum(1) / weights.sum(1).clamp(min=1.0)
        context = torch.relu(
            self.context(
                torch.cat(
                    [
                        glob,
                        hexes.mean(1),
                        hexes.amax(1),
                        (stacks * active.to(hexes.dtype).unsqueeze(2)).sum(1),
                        mean,
                    ],
                    dim=1,
                )
            )
        )
        padded = hexes.new_zeros(rows, L.MAX_FIELD_HEXES, self.width)
        padded[:, : self.grid.hexes] = hexes
        return context, stacks, padded


class HexConv2d(nn.Conv2d):
    """A 3 x 3 convolution on the axial grid whose kernel covers a hex and its six neighbours
    (`HEX_KERNEL`)."""

    def __init__(self, width: int) -> None:
        super().__init__(width, width, 3, padding=1)
        self.register_buffer("kernel_mask", HEX_KERNEL.clone(), persistent=False)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return F.conv2d(x, self.weight * self.kernel_mask, self.bias, padding=1)


class ConvBlock(nn.Module):
    """A residual block of two hex convolutions, the second's input biased by the first's mean
    and maximum over the real cells (global pooling); the branch starts at 0."""

    def __init__(self, width: int) -> None:
        super().__init__()
        self.conv1 = HexConv2d(width)
        self.pool = nn.Linear(2 * width, width)
        self.conv2 = HexConv2d(width)
        nn.init.zeros_(self.conv2.weight)
        nn.init.zeros_(self.conv2.bias)

    def forward(self, x: torch.Tensor, cells: torch.Tensor, real: int) -> torch.Tensor:
        """`x` [rows, width, rows of the grid, its columns]; `cells` [1, 1, …] 1 on the field's
        cells, 0 off it; `real` their number."""
        h = self.conv1(torch.relu(x)) * cells
        mean = h.sum((2, 3)) / real
        top = (h - (1.0 - cells) * 1e9).amax((2, 3))
        h = torch.relu(h + self.pool(torch.cat([mean, top], dim=1))[:, :, None, None])
        return x + self.conv2(h) * cells


class HexConv(HexNet):
    """The hex convolution network (module docstring)."""

    def __init__(
        self,
        width: int = 64,
        layers: int = 6,
        embed: int = 32,
        field_width: int = FIELD_WIDTH,
        field_height: int = FIELD_HEIGHT,
    ) -> None:
        super().__init__(width, embed, field_width, field_height)
        g = self.grid
        cells = torch.zeros(g.height * g.columns)
        cells[torch.from_numpy(g.cell)] = 1.0
        self.register_buffer("cells", cells.view(1, 1, g.height, g.columns), persistent=False)
        self.register_buffer("cell_index", torch.from_numpy(g.cell).long(), persistent=False)
        self.blocks = nn.ModuleList(ConvBlock(width) for _ in range(layers))
        self.glob_out = nn.Linear(width, width)
        self.make_heads(width, width, embed)

    def trunk(self, field: torch.Tensor, glob: torch.Tensor):
        rows = field.shape[0]
        g = self.grid
        grid = field.new_zeros(rows, g.height * g.columns, self.width)
        grid[:, self.cell_index] = field
        x = grid.transpose(1, 2).reshape(rows, self.width, g.height, g.columns)
        for block in self.blocks:
            x = block(x, self.cells, g.hexes)
        hexes = torch.relu(x).flatten(2).transpose(1, 2)[:, self.cell_index]
        return hexes, torch.relu(self.glob_out(glob))


class BiasBlock(Block):
    """`entity.Block` with an additive attention bias [heads, tokens, tokens] (the same for
    every row) instead of a mask."""

    def forward(self, x: torch.Tensor, bias: torch.Tensor) -> torch.Tensor:
        rows, tokens, width = x.shape
        qkv = self.qkv(self.norm1(x)).view(rows, tokens, 3, self.heads, width // self.heads)
        q, k, v = qkv.permute(2, 0, 3, 1, 4)
        a = F.scaled_dot_product_attention(q, k, v, attn_mask=bias.unsqueeze(0))
        x = x + self.proj(a.transpose(1, 2).reshape(rows, tokens, width))
        return x + self.ff(self.norm2(x))


class HexFormer(HexNet):
    """The hex transformer (module docstring)."""

    def __init__(
        self,
        width: int = 64,
        layers: int = 4,
        heads: int = 4,
        embed: int = 32,
        field_width: int = FIELD_WIDTH,
        field_height: int = FIELD_HEIGHT,
    ) -> None:
        super().__init__(width, embed, field_width, field_height)
        if width % heads:
            raise ValueError(f"the width {width} is no multiple of the heads {heads}")
        g = self.grid
        span_q, span_r = 2 * g.columns - 1, 2 * g.height - 1
        dq = g.q[None, :] - g.q[:, None] + g.columns - 1
        dr = g.r[None, :] - g.r[:, None] + g.height - 1
        self.register_buffer("offset", torch.from_numpy(dq * span_r + dr).long(), persistent=False)
        self.blocks = nn.ModuleList(BiasBlock(width, heads) for _ in range(layers))
        self.offset_bias = nn.Parameter(torch.zeros(layers, heads, span_q * span_r))
        self.norm = nn.LayerNorm(width)
        self.make_heads(width, width, embed)

    def trunk(self, field: torch.Tensor, glob: torch.Tensor):
        x = torch.cat([glob.unsqueeze(1), field], dim=1)
        hexes = self.grid.hexes
        for block, table in zip(self.blocks, self.offset_bias, strict=True):
            bias = table.new_zeros(table.shape[0], hexes + 1, hexes + 1)
            bias[:, 1:, 1:] = table[:, self.offset]
            x = block(x, bias)
        x = self.norm(x)
        return x[:, 1:], x[:, 0]


def for_batch(kind: str, batch: BattleBatch, config: dict) -> HexNet:
    """The hex network `kind` names for `batch` (whose encodings must be the layout's), from a
    run's `[network]` settings (`config`)."""
    check_layout(batch, f"{kind} network")
    field = (
        int(config.get("field_width", FIELD_WIDTH)),
        int(config.get("field_height", FIELD_HEIGHT)),
    )
    if kind == "hexconv":
        return HexConv(config["width"], config["layers"], config["embed"], *field)
    return HexFormer(config["width"], config["layers"], config["heads"], config["embed"], *field)
