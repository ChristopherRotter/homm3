"""The entity network (E06 T5a, `docs/epics/E06-design.md` §11): the layout it reads is the
library's, its logits stand in the encodings' order, an empty slot is not seen, the stacks' slot
order does not matter to its context; the visualization command. They need the group `train`."""

from __future__ import annotations

import numpy as np
import pytest
from test_train import mixed_batch

from h3env import BattleBatch, _ffi
from h3env.train import layout as L

torch = pytest.importorskip("torch")

from h3env.train.entity import COLUMNS, FIXED, PER_STACK, ROWS, Entity  # noqa: E402
from h3env.train.network import main, mermaid, parts  # noqa: E402
from h3env.train.policy import Mlp  # noqa: E402


def small() -> Entity:
    torch.manual_seed(0)
    return Entity(width=16, layers=1, heads=2, embed=8)


def test_the_layout_is_the_librarys() -> None:
    # The sizes and versions (the literals are pinned against the Rust encodings by `h3-sim`'s
    # tests), the entries the binding names, and on real battles the stack columns the network
    # reads: the legal attacks, shots and heals name present stacks, one present stack is active
    # where a side is to move, and an on-field stack stands inside the field.
    versions = _ffi.versions()
    assert (versions["observation_encoding"], versions["action_encoding"]) == (
        L.OBSERVATION_ENCODING,
        L.ACTION_ENCODING,
    )
    assert _ffi.sizes() == (L.ACTION_COUNT, L.SPELL_COUNT, L.TARGET_COUNT, L.OBSERVATION_LEN)
    assert (_ffi.RETREAT, _ffi.SURRENDER) == (L.END_TACTICS + 1, L.END_TACTICS + 2)
    assert (_ffi.HERO_CAST, _ffi.CREATURE_CAST) == (L.END_TACTICS + 3, L.END_TACTICS + 4)
    assert L.WALL_SHOT_BASE == _ffi.CREATURE_CAST + 1
    batch = mixed_batch()
    rng = np.random.default_rng(0)
    seen = 0
    for _ in range(40):
        for env in np.nonzero(batch.to_move >= 0)[0]:
            stacks = batch.obs[env, L.STACKS :].reshape(L.STACK_CAPACITY, L.STACK_FEATURES)
            present = stacks[:, L.PRESENT] == 1.0
            legal = np.nonzero(batch.entry_mask[env])[0]
            for lo, hi, per in (
                (L.ATTACK_BASE, L.SHOOT_BASE, L.ATTACK_SLOTS),
                (L.SHOOT_BASE, L.HEAL_BASE, 2),
                (L.HEAL_BASE, L.END_TACTICS, 1),
            ):
                named = (legal[(legal >= lo) & (legal < hi)] - lo) // per
                assert present[named].all()
                seen += len(named)
            active = np.nonzero(stacks[:, L.ACTIVE] == 1.0)[0]
            assert len(active) <= 1 and present[active].all()
            # Every position within the embeddings' ranges (no field is wider or taller).
            assert (stacks[present, L.X] < COLUMNS).all() and (stacks[present, L.Y] < ROWS).all()
            assert (stacks[~present] == 0.0).all()
        a = batch.actions
        casts = [_ffi.HERO_CAST, _ffi.CREATURE_CAST]
        for env, mask in enumerate(batch.entry_mask):
            legal = np.setdiff1d(np.nonzero(mask)[0], casts)
            a["entry"][env] = rng.choice(legal) if len(legal) else 0
        a["spell"] = a["target"] = a["second"] = 0
        batch.step()
    assert seen > 0
    batch.close()


def test_the_logits_stand_in_the_encodings_order() -> None:
    # Each part's scores tagged by where they come from; the entry logits place them as action
    # encoding v13 does (`h3_sim::action`), the target logits as its target heads.
    net = small()
    rows = 2

    def tag(base: float, shape):
        def hook(module, args, out):
            return base + torch.arange(int(np.prod(shape)), dtype=out.dtype).view(shape).expand(
                rows, *shape
            )

        return hook

    net.fixed_head.register_forward_hook(tag(10_000, (FIXED,)))
    net.move_head.register_forward_hook(tag(20_000, (L.MAX_FIELD_HEXES, 1)))
    net.stack_head.register_forward_hook(tag(30_000, (L.STACK_CAPACITY, PER_STACK)))
    net.target_none.register_forward_hook(tag(40_000, (1,)))
    net.target_hex.register_forward_hook(tag(50_000, (L.MAX_FIELD_HEXES, 1)))
    net.target_stack.register_forward_hook(tag(60_000, (L.STACK_CAPACITY, 1)))
    obs = torch.zeros(rows, L.OBSERVATION_LEN)
    h = net.encode(obs)
    entry = net.entry_logits(h)[0]
    assert entry.shape == (L.ACTION_COUNT,)
    s = 7
    assert entry[0] == 10_000 and entry[1] == 10_001
    assert entry[L.MOVE_BASE + 30] == 20_030
    assert entry[L.ATTACK_BASE + 24 * s + 5] == 30_000 + PER_STACK * s + 5
    assert entry[L.SHOOT_BASE + 2 * s + 1] == 30_000 + PER_STACK * s + 24 + 1
    assert entry[L.HEAL_BASE + s] == 30_000 + PER_STACK * s + 26
    assert entry[L.END_TACTICS] == 10_002 and entry[_ffi.RETREAT] == 10_003
    assert entry[_ffi.HERO_CAST] == 10_005 and entry[_ffi.CREATURE_CAST] == 10_006
    assert entry[L.WALL_SHOT_BASE + 7] == 10_014
    cast = net.cast_of(torch.full((rows,), int(_ffi.HERO_CAST)))
    zero = torch.zeros(rows, dtype=torch.long)
    target = net.target_logits(h, cast, zero)[0]
    assert target.shape == (L.TARGET_COUNT,)
    assert target[0] == 40_000 and target[1 + 30] == 50_030
    assert target[L.TARGET_STACK_BASE + s] == 60_000 + s
    second = net.second_logits(h, cast, zero, zero)
    assert second.shape == (rows, L.TARGET_COUNT)


def test_an_empty_slot_is_not_seen_and_the_slots_order_does_not_matter() -> None:
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    batch.close()
    net = small().eval()
    stacks = obs[:, L.STACKS :].view(-1, L.STACK_CAPACITY, L.STACK_FEATURES)
    present = stacks[..., L.PRESENT] == 1.0
    row = int(torch.nonzero(present.sum(1) >= 2)[0, 0])
    empty = int(torch.nonzero(~present[row])[0, 0])
    with torch.no_grad():
        context, tokens, hexes = net.encode(obs[row : row + 1])
        # Noise in an empty slot (its present flag still 0) changes nothing the heads read
        # but that slot's own token.
        noisy = obs[row : row + 1].clone()
        view = noisy[:, L.STACKS :].view(1, L.STACK_CAPACITY, L.STACK_FEATURES)
        view[0, empty, 1:] = torch.rand(L.STACK_FEATURES - 1) * 50
        c2, t2, h2 = net.encode(noisy)
        assert torch.allclose(context, c2, atol=1e-6) and torch.equal(hexes, h2)
        others = [i for i in range(L.STACK_CAPACITY) if i != empty]
        assert torch.allclose(tokens[0, others], t2[0, others], atol=1e-6)
        # Two present stacks swap their slots: the context stays, their tokens swap.
        a, b = torch.nonzero(present[row])[:2, 0].tolist()
        swapped = obs[row : row + 1].clone()
        view = swapped[:, L.STACKS :].view(1, L.STACK_CAPACITY, L.STACK_FEATURES)
        view[0, [a, b]] = view[0, [b, a]].clone()
        c3, t3, _ = net.encode(swapped)
        assert torch.allclose(context, c3, atol=1e-5)
        assert torch.allclose(tokens[0, [a, b]], t3[0, [b, a]], atol=1e-5)


def test_the_entity_network_refuses_other_encodings(monkeypatch) -> None:
    batch = BattleBatch(1)
    other = {"observation_encoding": 22, "action_encoding": 13}
    monkeypatch.setattr(_ffi, "versions", lambda: other)
    with pytest.raises(ValueError, match="observation encoding v21"):
        Entity.for_batch(batch)
    batch.close()
    with pytest.raises(ValueError, match="no multiple"):
        Entity(width=10, heads=4)


def test_the_architecture_shows_its_parts_costs_and_diagram(tmp_path, capsys) -> None:
    batch = mixed_batch()
    obs = torch.from_numpy(batch.obs.copy())
    mlp = Mlp.for_batch(batch, hidden=32, embed=8)
    batch.close()
    by_name = {p.name: p for p in parts(mlp, obs)}
    # The MLP's trunk: 8 882 × 32 + 32 × 32 multiply-adds a decision, its shape the hidden width.
    assert by_name["trunk"].macs == L.OBSERVATION_LEN * 32 + 32 * 32
    assert by_name["trunk"].shape == (8, 32) and by_name["trunk"].cast_macs == 0
    assert by_name["spell_head"].macs == 0 and by_name["spell_head"].cast_macs == 34 * 128
    assert sum(p.parameters for p in by_name.values()) == sum(p.numel() for p in mlp.parameters())
    entity = {p.name: p for p in parts(small(), obs)}
    assert entity["blocks"].shape == (8, L.STACK_CAPACITY + 3, 16)
    # A block's attention: 2 · 45² · 16 beside its linear layers.
    linear = 16 * 48 + 16 * 16 + 16 * 32 + 32 * 16
    assert entity["blocks"].macs == pytest.approx(45 * linear + 2 * 45 * 45 * 16)
    assert entity["(own)"].parameters == L.MAX_FIELD_HEXES * 16
    assert "flowchart LR" in mermaid(mlp) and "transformer block" in mermaid(small())
    out = tmp_path / "network.md"
    main(
        [
            "--set",
            "network.kind=entity",
            "--set",
            "network.width=16",
            "--rows",
            "4",
            "--time",
            "--repeats",
            "1",
            "--mermaid",
            str(out),
        ]
    )
    printed = capsys.readouterr().out
    assert "| **total** |" in printed and "training pass" in printed
    text = out.read_text(encoding="utf-8")
    assert text.startswith("# The entity network of the defaults") and "```mermaid" in text
