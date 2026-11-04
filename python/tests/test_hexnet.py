"""The hex networks (E25 T7a, `h3env.train.hexnet`) and the MLP's depth: the kernel covers a
hex's neighbours, a stack is drawn on its hexes, an empty slot is not seen and the slots' order
does not matter to the context, the field's size holds on real battles, the visualization counts
the convolutions; the normalisation keeps the field and the defaults are the networks before it,
the transformer blocks' feed-forward width (E25 T24); E25 T25's keys (`mask_input`, `order`,
`attack_hex`) on the structured kinds. They need the group `train`."""

from __future__ import annotations

from dataclasses import asdict

import numpy as np
import pytest
from test_train import mixed_batch

from h3env import _ffi
from h3env.train import layout as L
from h3env.train.run import Config, ConfigError, load_config

torch = pytest.importorskip("torch")

from h3env.train.entity import Entity  # noqa: E402
from h3env.train.geometry import attack_hexes, attack_table  # noqa: E402
from h3env.train.hexnet import (  # noqa: E402
    FIELD_HEIGHT,
    FIELD_WIDTH,
    HEX_KERNEL,
    CellNorm,
    Grid,
    HexConv,
    HexConv2d,
    HexFormer,
    neighbours,
)
from h3env.train.network import hex_mermaid, parts  # noqa: E402
from h3env.train.policy import Mlp, build, symlog  # noqa: E402

KINDS = ["hexconv", "hexconv-norm", "hexformer"]


def small(kind: str):
    torch.manual_seed(0)
    if kind == "hexconv":
        return HexConv(width=16, layers=2, embed=8)
    if kind == "hexconv-norm":
        net = HexConv(width=16, layers=2, embed=8, norm="layer")
        # Gains and biases away from 1 and 0, as training moves them.
        with torch.no_grad():
            for m in net.modules():
                if isinstance(m, CellNorm):
                    m.weight.uniform_(0.5, 1.5)
                    m.bias.uniform_(-0.5, 0.5)
        return net
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


@pytest.mark.parametrize("kind", KINDS)
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


@pytest.mark.parametrize("kind", KINDS)
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


def test_the_normalisation_keeps_the_cells_off_the_field_at_zero() -> None:
    # A LayerNorm's bias would move a cell off the field; the normalised network sets them back,
    # so every block's output, and the trunk's, is 0 there as without it.
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    batch.close()
    net = small("hexconv-norm").eval()
    seen = []
    hooks = [b.register_forward_hook(lambda m, a, out: seen.append(out)) for b in net.blocks]
    with torch.no_grad():
        net.encode(obs)
    for hook in hooks:
        hook.remove()
    off = 1.0 - net.cells
    assert len(seen) == 2 and all((out * off == 0).all() for out in seen)
    assert all((out * net.cells != 0).any() for out in seen)
    names = list(net.state_dict())
    assert {"blocks.0.norm1.weight", "blocks.0.norm2.bias", "norm_out.weight"} <= set(names)


def t7a_block(block, x: torch.Tensor, cells: torch.Tensor, real: int) -> torch.Tensor:
    """`ConvBlock.forward` as E25 T7a wrote it (no normalisation), verbatim."""
    h = block.conv1(torch.relu(x)) * cells
    mean = h.sum((2, 3)) / real
    top = (h - (1.0 - cells) * 1e9).amax((2, 3))
    h = torch.relu(h + block.pool(torch.cat([mean, top], dim=1))[:, :, None, None])
    return x + block.conv2(h) * cells


def test_the_defaults_are_the_networks_before_t24() -> None:
    # A network file before E25 T24 has no `norm` or `feed_forward`: it builds the network it
    # was (the same parameters by name and shape) and plays as it did (the hex convolution's
    # block bit for bit as T7a's); the transformer blocks' feed-forward layer twice the width.
    batch = mixed_batch()
    for kind in ("hexconv", "hexformer", "entity"):
        config = asdict(Config(kind=kind, width=16, layers=2, heads=2, embed=8))
        old = {k: v for k, v in config.items() if k not in ("norm", "feed_forward")}
        torch.manual_seed(0)
        net = build(old, batch)
        torch.manual_seed(0)
        same = build(config, batch)
        shapes = {k: v.shape for k, v in net.state_dict().items()}
        assert shapes == {k: v.shape for k, v in same.state_dict().items()}
        if kind == "hexconv":
            assert not any("norm" in k for k in shapes)
        else:
            assert shapes["blocks.0.ff.0.weight"] == (32, 16)
            assert shapes["blocks.0.ff.2.weight"] == (16, 32)
    batch.close()
    net = small("hexconv").train()
    with torch.no_grad():
        for p in net.parameters():
            p.add_(torch.randn_like(p) * 0.1)
    x = torch.randn(5, 16, net.grid.height, net.grid.columns) * net.cells
    for block in net.blocks:
        assert torch.equal(block(x, net.cells, net.grid.hexes), t7a_block(block, x, net.cells, 187))
    with pytest.raises(ValueError, match="no normalisation"):
        HexConv(width=8, layers=1, embed=8, norm="batch")


def test_the_feed_forward_width_and_the_new_keys_config() -> None:
    batch = mixed_batch()
    for kind in ("hexformer", "entity"):
        config = asdict(Config(kind=kind, width=16, layers=1, heads=2, embed=8, feed_forward=4))
        net = build(config, batch)
        assert net.blocks[0].ff[0].out_features == 64 and net.blocks[0].ff[2].in_features == 64
    assert isinstance(net, Entity)
    conv = build(asdict(Config(kind="hexconv", width=16, layers=1, embed=8, norm="layer")), batch)
    choice = conv.act(batch)
    assert (batch.entry_mask[np.arange(batch.envs), choice.entry] == 1).all()
    batch.close()
    assert load_config("[network]\nkind = 'hexconv'\nnorm = 'layer'\n").norm == "layer"
    for text, match in (
        ("[network]\nnorm = 'layer'\n", "`norm` is the hex convolution network's"),
        ("[network]\nkind = 'hexformer'\nnorm = 'layer'\n", "`norm` is the hex convolution"),
        ("[network]\nkind = 'hexconv'\nnorm = 'batch'\n", "`norm` is one of none, layer"),
        ("[network]\nfeed_forward = 0\n", "`feed_forward` must be positive"),
    ):
        with pytest.raises(ConfigError, match=match):
            load_config(text)


def test_the_defaults_are_the_networks_before_t25() -> None:
    # A network file before E25 T25 has no `mask_input`: it builds the network it was (the same
    # parameters by name and shape) and encodes as it did, its entry mask handed or not (the
    # MLP's trunk bit for bit as `Mlp.encode` before T25).
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    mask = torch.from_numpy(batch.entry_mask.copy()).bool()
    for kind in ("mlp", "entity", "hexconv", "hexformer"):
        config = asdict(Config(kind=kind, hidden=32, width=16, layers=2, heads=2, embed=8))
        old = {k: v for k, v in config.items() if k not in ("mask_input", "attack_hex")}
        torch.manual_seed(0)
        net = build(old, batch).eval()
        torch.manual_seed(0)
        same = build(config, batch)
        shapes = {k: v.shape for k, v in net.state_dict().items()}
        assert shapes == {k: v.shape for k, v in same.state_dict().items()}
        assert not net.reads_mask
        with torch.no_grad():
            plain, given = net.encode(obs), net.encode(obs, mask)
            assert all(torch.equal(a, b) for a, b in zip(plain, given, strict=True))
            assert len(given) == (1 if kind == "mlp" else 3)
            assert torch.equal(net.values(obs), net.values(obs, mask))
            if kind == "mlp":
                assert shapes["trunk.0.weight"] == (32, L.ORDER)
                assert torch.equal(given[0], net.trunk(symlog(obs[:, : L.ORDER])))
    batch.close()


def test_the_t25_keys_config() -> None:
    # E25 T25: `mask_input` and `order` are every kind's, `attack_hex` the structured kinds'
    # (`run.validate` refuses it on the MLP, `policy.build` too).
    for kind in STRUCTURED:
        text = f"[network]\nkind = '{kind}'\nmask_input = true\norder = true\nattack_hex = true\n"
        config = load_config(text)
        assert config.mask_input and config.order and config.attack_hex
    with pytest.raises(ConfigError, match="`attack_hex` is the structured kinds'"):
        load_config("[network]\nattack_hex = true\n")
    batch = mixed_batch()
    with pytest.raises(ValueError, match="`attack_hex` is the structured kinds'"):
        build(asdict(Config(hidden=32, embed=8, attack_hex=True)), batch)
    batch.close()
    assert load_config("[network]\nmask_input = true\norder = true\n").mask_input
    assert not load_config("").attack_hex


STRUCTURED = ["entity", "hexconv", "hexformer"]
#: Each E25 T25 key's new modules.
NEW_MODULES = {
    "mask_input": {"mask_hex_in", "mask_stack_in", "mask_fixed_in"},
    "order": {"order_in"},
    "attack_hex": {"attack_hex_head"},
}


def keyed(kind: str, batch, **keys):
    torch.manual_seed(0)
    config = Config(kind=kind, width=16, layers=1, heads=2, embed=8, **keys)
    return build(asdict(config), batch).eval()


def legal_attacks(mask: torch.Tensor) -> torch.Tensor:
    return mask[:, L.ATTACK_BASE : L.SHOOT_BASE].reshape(-1, L.STACK_CAPACITY, L.ATTACK_SLOTS)


@pytest.mark.parametrize("key", list(NEW_MODULES))
@pytest.mark.parametrize("kind", STRUCTURED)
def test_a_t25_key_adds_its_modules_to_the_network_without_it(kind, key) -> None:
    # A key's modules come after the others: with the same seed the network with it has the
    # other's weights, and with its new weights at 0 computes the other's outputs; with them as
    # made it computes others, finite. `mask_input` and `attack_hex` read the entry mask and
    # refuse an encoding without it.
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    mask = torch.from_numpy(batch.entry_mask.copy()).bool()
    plain, net = keyed(kind, batch), keyed(kind, batch, **{key: True})
    batch.close()
    old, new = plain.state_dict(), net.state_dict()
    assert set(old) <= set(new) and all(torch.equal(v, new[k]) for k, v in old.items())
    assert {name.split(".")[0] for name in set(new) - set(old)} == NEW_MODULES[key]
    assert net.reads_mask == (key != "order") and net.reads_order == (key == "order")
    if net.reads_mask:
        for call in (net.encode, net.values):
            with pytest.raises(ValueError, match="reads the legal entry mask"):
                call(obs)
    with torch.no_grad():
        h, base = net.encode(obs, mask), plain.encode(obs, mask)
        assert len(h) == (4 if key == "attack_hex" else 3) and len(base) == 3
        logits = net.entry_logits(h)
        assert torch.isfinite(logits).all()
        assert not torch.allclose(logits, plain.entry_logits(base))
        for name in NEW_MODULES[key]:
            for p in getattr(net, name).parameters():
                p.zero_()
        h = net.encode(obs, mask)
        assert all(torch.equal(a, b) for a, b in zip(h[:3], base, strict=True))
        assert torch.equal(net.entry_logits(h), plain.entry_logits(base))
        assert torch.equal(net.values(obs, mask), plain.values(obs, mask))


@pytest.mark.parametrize("kind", STRUCTURED)
def test_the_mask_input_reads_the_moves_the_attacks_and_the_fixed_entries(kind) -> None:
    # Flipping a legal move, a legal attack (its slot's entries and its from-hex's plane) or a
    # fixed entry (End tactics) of one row moves that row's encoding only.
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    mask = torch.from_numpy(batch.entry_mask.copy()).bool()
    net = keyed(kind, batch, mask_input=True)
    batch.close()
    move = torch.nonzero(mask[:, L.MOVE_BASE : L.ATTACK_BASE])[0] + torch.tensor([0, L.MOVE_BASE])
    attack = torch.nonzero(mask[:, L.ATTACK_BASE : L.SHOOT_BASE])[0]
    attack = attack + torch.tensor([0, L.ATTACK_BASE])
    row = int(torch.nonzero(mask[:, L.END_TACTICS])[0, 0]) if mask[:, L.END_TACTICS].any() else 0
    with torch.no_grad():
        context = net.encode(obs, mask)[0]
        for r, entry in (move.tolist(), attack.tolist(), [row, L.END_TACTICS]):
            other = mask.clone()
            other[r, entry] = ~other[r, entry]
            moved = net.encode(obs, other)[0]
            assert not torch.equal(moved[r], context[r])
            rest = torch.arange(len(obs)) != r
            assert torch.equal(moved[rest], context[rest])


@pytest.mark.parametrize("kind", STRUCTURED)
def test_the_stack_tokens_read_the_turn_order_with_order(kind) -> None:
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    mask = torch.from_numpy(batch.entry_mask.copy()).bool()
    plain, net = keyed(kind, batch), keyed(kind, batch, order=True)
    batch.close()
    moved = obs.clone()
    moved[:, L.ORDER :] += 5.0
    with torch.no_grad():
        assert not torch.equal(net.encode(obs, mask)[1], net.encode(moved, mask)[1])
        assert all(
            torch.equal(a, b)
            for a, b in zip(plain.encode(obs, mask), plain.encode(moved, mask), strict=True)
        )


@pytest.mark.parametrize("kind", STRUCTURED)
def test_a_melee_entry_scores_the_hex_it_attacks_from(kind) -> None:
    # `attack_hex`: the encoding's fourth tensor is the legal attack entries' from-hexes
    # (`geometry.attack_hexes` where the entry is legal, -1 elsewhere). The head's term is added
    # to the legal entries only (its output bias moves exactly them), and a from-hex's vector
    # moves exactly the legal entries of its row made from it.
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    mask = torch.from_numpy(batch.entry_mask.copy()).bool()
    net = keyed(kind, batch, attack_hex=True)
    batch.close()
    table = torch.from_numpy(attack_table(*net_field(net)))
    expected = attack_hexes(obs, table, net_field(net)[0])
    expected = torch.where(legal_attacks(mask), expected, torch.full_like(expected, -1))
    attacks = slice(L.ATTACK_BASE, L.SHOOT_BASE)
    with torch.no_grad():
        h = net.encode(obs, mask)
        legal_from = h[3]
        assert torch.equal(legal_from, expected) and (legal_from >= 0).sum() > 0
        before = net.entry_logits(h)
        net.attack_hex_head.out.bias += 1.0
        after = net.entry_logits(h)
        net.attack_hex_head.out.bias -= 1.0
        legal = (legal_from >= 0).flatten(1).to(before.dtype)
        assert torch.allclose(after[:, attacks] - before[:, attacks], legal, atol=1e-5)
        assert torch.equal(after[:, : L.ATTACK_BASE], before[:, : L.ATTACK_BASE])
        assert torch.equal(after[:, L.SHOOT_BASE :], before[:, L.SHOOT_BASE :])
        row, slot, k = (int(i) for i in torch.nonzero(legal_from >= 0)[0])
        at = int(legal_from[row, slot, k])
        hexes = h[2].clone()
        hexes[row, at] += torch.randn(hexes.shape[2])
        moved = net.entry_logits((h[0], h[1], hexes, h[3]))
        changed = (moved[:, attacks] != before[:, attacks]).view_as(legal_from)
        same_hex = torch.zeros_like(changed)
        same_hex[row] = legal_from[row] == at
        assert changed[row, slot, k] and torch.equal(changed, same_hex)
        # An illegal entry's term is 0 and its logit finite.
        assert torch.isfinite(moved).all()


def net_field(net) -> tuple[int, int]:
    if isinstance(net, Entity):
        return FIELD_WIDTH, FIELD_HEIGHT
    return net.grid.width, net.grid.height
