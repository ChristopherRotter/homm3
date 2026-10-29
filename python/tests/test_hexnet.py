"""The hex networks (E25 T7a, `h3env.train.hexnet`) and the MLP's depth: the kernel covers a
hex's neighbours, a stack is drawn on its hexes, an empty slot is not seen and the slots' order
does not matter to the context, the field's size holds on real battles, the visualization counts
the convolutions. They need the group `train`."""

from __future__ import annotations

from dataclasses import asdict

import numpy as np
import pytest
from test_train import mixed_batch

from h3env import _ffi
from h3env.train import layout as L
from h3env.train.run import Config, ConfigError, load_config

torch = pytest.importorskip("torch")

from h3env.train.hexnet import (  # noqa: E402
    FIELD_HEIGHT,
    FIELD_WIDTH,
    HEX_KERNEL,
    Grid,
    HexConv,
    HexConv2d,
    HexFormer,
    neighbours,
)
from h3env.train.network import hex_mermaid, parts  # noqa: E402
from h3env.train.policy import Mlp, build  # noqa: E402


def small(kind: str):
    torch.manual_seed(0)
    if kind == "hexconv":
        return HexConv(width=16, layers=2, embed=8)
    return HexFormer(width=16, layers=2, heads=2, embed=8)


def test_the_kernel_covers_a_hex_and_its_six_neighbours() -> None:
    # A hex convolution of all-ones weights spreads a hex's one into exactly the hexes that
    # `h3_battle::geometry::Field::neighbors` names (ported by `neighbours`), on every hex of the
    # field; the cells off the field are only padding.
    g = Grid(FIELD_WIDTH, FIELD_HEIGHT)
    assert (g.columns, g.hexes) == (22, 187) and len(set(g.cell.tolist())) == g.hexes
    assert g.cell.min() >= 0 and g.cell.max() < g.height * g.columns
    conv = HexConv2d(1)
    with torch.no_grad():
        conv.weight.fill_(1.0)
        conv.bias.zero_()
    hex_of = {int(c): h for h, c in enumerate(g.cell)}
    for h in range(g.hexes):
        x = torch.zeros(1, 1, g.height * g.columns)
        x[0, 0, g.cell[h]] = 1.0
        with torch.no_grad():
            y = conv(x.view(1, 1, g.height, g.columns)).flatten()
        reached = {hex_of[int(c)] for c in torch.nonzero(y).flatten() if int(c) in hex_of}
        assert reached == {h, *neighbours(FIELD_WIDTH, FIELD_HEIGHT, h)}, h
    assert HEX_KERNEL.sum() == 7
    # An interior hex on an even and an odd row has six neighbours.
    assert len(neighbours(17, 11, 2 * 17 + 5)) == len(neighbours(17, 11, 3 * 17 + 5)) == 6
    with pytest.raises(ValueError, match="does not fit"):
        Grid(17, 16)


def stack_obs(x: int, y: int, two_hex: bool, own: bool, viewer_attacks: bool) -> torch.Tensor:
    """An observation of one stack at (x, y) and nothing else."""
    obs = torch.zeros(1, L.OBSERVATION_LEN)
    obs[0, L.VIEWER_ATTACKER] = float(viewer_attacks)
    s = obs[0, L.STACKS : L.STACKS + L.STACK_FEATURES]
    s[L.PRESENT] = s[L.ALIVE] = 1.0
    s[L.OWN] = float(own)
    s[L.COUNT] = 10.0
    s[L.X], s[L.Y] = float(x), float(y)
    s[L.TWO_HEX] = float(two_hex)
    return obs


@pytest.mark.parametrize("kind", ["hexconv", "hexformer"])
def test_a_stack_is_drawn_on_its_hexes(kind) -> None:
    # The drawing differs from the empty field's only on the stack's hexes: its position, and for
    # a two-hex stack the hex in front of it, right for the attacker's, left for the defender's
    # (its own flag against the viewer's seat).
    net = small(kind).eval()
    x, y = 6, 4
    at = y * FIELD_WIDTH + x
    with torch.no_grad():
        for two_hex, own, viewer, hexes in (
            (False, True, True, {at}),
            (True, True, True, {at, at + 1}),
            (True, False, True, {at, at - 1}),
            (True, True, False, {at, at - 1}),
            (True, False, False, {at, at + 1}),
        ):
            obs = stack_obs(x, y, two_hex, own, viewer)
            empty = obs.clone()
            empty[0, L.STACKS + L.PRESENT] = 0.0
            drawn = net.draw(obs)[0] - net.draw(empty)[0]
            changed = set(torch.nonzero(drawn[0].abs().sum(1) > 1e-6).flatten().tolist())
            assert changed == hexes, (two_hex, own, viewer)
        # An off-field stack (a turret's position) keeps its token and is drawn nowhere.
        obs = stack_obs(x, FIELD_HEIGHT + 2, False, True, True)
        field, _, tokens, position, present, _ = net.draw(obs)
        assert position[0, 0] == -1 and present[0, 0] and tokens[0, 0].abs().sum() > 0
        empty = obs.clone()
        empty[0, L.STACKS + L.PRESENT] = 0.0
        assert torch.allclose(field, net.draw(empty)[0])


@pytest.mark.parametrize("kind", ["hexconv", "hexformer"])
def test_an_empty_slot_is_not_seen_and_the_slots_order_does_not_matter(kind) -> None:
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    batch.close()
    net = small(kind).eval()
    stacks = obs[:, L.STACKS : L.ORDER].view(-1, L.STACK_CAPACITY, L.STACK_FEATURES)
    present = stacks[..., L.PRESENT] == 1.0
    row = int(torch.nonzero(present.sum(1) >= 2)[0, 0])
    empty = int(torch.nonzero(~present[row])[0, 0])
    with torch.no_grad():
        context, vectors, hexes = net.encode(obs[row : row + 1])
        assert hexes.shape == (1, L.MAX_FIELD_HEXES, 16)
        assert (hexes[0, FIELD_WIDTH * FIELD_HEIGHT :] == 0).all()
        noisy = obs[row : row + 1].clone()
        view = noisy[:, L.STACKS : L.ORDER].view(1, L.STACK_CAPACITY, L.STACK_FEATURES)
        view[0, empty, 1:] = torch.rand(L.STACK_FEATURES - 1) * 50
        c2, v2, h2 = net.encode(noisy)
        assert torch.allclose(context, c2, atol=1e-6) and torch.allclose(hexes, h2, atol=1e-6)
        others = [i for i in range(L.STACK_CAPACITY) if i != empty]
        assert torch.allclose(vectors[0, others], v2[0, others], atol=1e-6)
        a, b = torch.nonzero(present[row])[:2, 0].tolist()
        swapped = obs[row : row + 1].clone()
        view = swapped[:, L.STACKS : L.ORDER].view(1, L.STACK_CAPACITY, L.STACK_FEATURES)
        view[0, [a, b]] = view[0, [b, a]].clone()
        c3, v3, _ = net.encode(swapped)
        assert torch.allclose(context, c3, atol=1e-5)
        assert torch.allclose(vectors[0, [a, b]], v3[0, [b, a]], atol=1e-5)


def test_the_field_size_holds_on_real_battles() -> None:
    # The base ruleset's 17 x 11 field: every legal move and every hex a plane marks lies on it,
    # every present stack stands on it but a town's turret, which stands off it.
    batch = mixed_batch()
    rng = np.random.default_rng(0)
    hexes = FIELD_WIDTH * FIELD_HEIGHT
    on, off = 0, 0
    for _ in range(40):
        moves = np.nonzero(batch.entry_mask[:, L.MOVE_BASE : L.ATTACK_BASE])[1]
        assert (moves < hexes).all()
        planes = batch.obs[:, L.FIELD : L.STACKS].reshape(-1, L.HEX_PLANES, L.MAX_FIELD_HEXES)
        assert (planes[:, :, hexes:] == 0).all()
        stacks = batch.obs[:, L.STACKS : L.ORDER].reshape(-1, L.STACK_CAPACITY, L.STACK_FEATURES)
        present = stacks[..., L.PRESENT] == 1.0
        inside = (stacks[..., L.X] < FIELD_WIDTH) & (stacks[..., L.Y] < FIELD_HEIGHT)
        turret = stacks[..., L.TURRET] == 1.0
        assert (inside | turret)[present].all()
        on += int((present & inside).sum())
        off += int((present & ~inside).sum())
        a = batch.actions
        casts = [_ffi.HERO_CAST, _ffi.CREATURE_CAST]
        for env, mask in enumerate(batch.entry_mask):
            legal = np.setdiff1d(np.nonzero(mask)[0], casts)
            a["entry"][env] = rng.choice(legal) if len(legal) else 0
        a["spell"] = a["target"] = a["second"] = 0
        batch.step()
    assert on > 0
    batch.close()


def test_the_mlps_depth_and_the_kinds_config() -> None:
    batch = mixed_batch()
    # Two layers by default and for a saved MLP without the key (every MLP before E25 T7a):
    # the same parameters by name, so their files load.
    two = Mlp.for_batch(batch, hidden=32, embed=8)
    config = asdict(Config(hidden=32, embed=8))
    del config["layers"]
    assert list(build(config, batch).state_dict()) == list(two.state_dict())
    deep = build(asdict(Config(hidden=32, embed=8, layers=4)), batch)
    linear = [m for m in deep.trunk if isinstance(m, torch.nn.Linear)]
    assert len(linear) == 4 and linear[-1].in_features == 32
    for kind, cls in (("hexconv", HexConv), ("hexformer", HexFormer)):
        net = build(asdict(Config(kind=kind, width=16, layers=1, heads=2, embed=8)), batch)
        assert isinstance(net, cls) and net.grid.hexes == FIELD_WIDTH * FIELD_HEIGHT
        choice = net.act(batch)
        assert (batch.entry_mask[np.arange(batch.envs), choice.entry] == 1).all()
    batch.close()
    assert load_config("[network]\nkind = 'hexformer'\n").kind == "hexformer"
    with pytest.raises(ConfigError, match="`field_width` must be positive"):
        load_config("[network]\nfield_width = 0\n")


def test_the_visualization_counts_the_convolutions() -> None:
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    batch.close()
    net = small("hexconv")
    by_name = {p.name: p for p in parts(net, obs)}
    # Per block two convolutions over the 11 x 22 cells of 7 taps, and the pooling's layer.
    cells = 11 * 22
    assert by_name["blocks"].macs == pytest.approx(2 * (2 * cells * 16 * 16 * 7 + 32 * 16))
    assert sum(p.parameters for p in by_name.values()) == sum(p.numel() for p in net.parameters())
    assert "hex convolutions" in hex_mermaid(net)
    assert "bias per axial offset" in hex_mermaid(small("hexformer"))
