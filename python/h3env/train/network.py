"""A network's architecture at a glance (E06 T5a, `docs/epics/E06-log.md` §11):

    uv run --group train python -m h3env.train.network --config configs/e06-t5.toml
    uv run --group train python -m h3env.train.network --set network.kind=mlp --time
    uv run --group train python -m h3env.train.network --from runs/t4-2/best.pt \
        --mermaid runs/t4-2/network.md
    uv run --group train python -m h3env.train.network --bench configs/distill/e25-grid.toml \
        --time --rows 512 --device cuda

The network a config names (`--config`, `--set`, as a run's) or a saved file holds (`--from`),
run on `--rows` observations of the encounter distribution: each part's parameters, output shape
and multiply-adds per decision (the linear layers' and the attention's; the cast heads apart, as
only a cast runs them), the totals, and with `--time` the time per decision of a call's forward
pass, of a training pass (forward and backward of `evaluate`) and of a sample-pass (the training
pass with an AdamW step, a bench's or PPO's minibatch step; on CUDA its peak memory too, E25
T24). `--bench` puts every student of a bench's file in one table instead (its parameters,
multiply-adds and with `--time` its timing; E25 T24). `--mermaid` writes the data flow as a
Mermaid diagram with the table (Markdown, which GitHub renders). No dependency beyond torch.
"""

from __future__ import annotations

import argparse
import time
from collections.abc import Callable
from dataclasses import asdict, dataclass
from pathlib import Path

import numpy as np
import torch
from torch import nn

from ..batch import BattleBatch
from .distill import STUDENT_KEYS, Bench, bench_device, load_bench, optimizer, student_config
from .entity import Block, Entity
from .hexnet import HexConv, HexNet
from .policy import CASTS, Mlp, Policy, build
from .run import Config, load_config


@dataclass
class Part:
    """One top-level part of a network: its parameters, its first output's shape (rows first)
    and its multiply-adds per row on a decision's pass and on a cast's heads."""

    name: str
    kind: str
    parameters: int
    shape: tuple[int, ...] | None = None
    macs: float = 0.0
    cast_macs: float = 0.0


def parts(
    net: Policy,
    obs: torch.Tensor,
    mask: torch.Tensor | None = None,
    preview: torch.Tensor | None = None,
) -> list[Part]:
    """`net`'s top-level parts (its direct children, then its own parameters as one part) on the
    rows `obs`: the decision's pass (`encode`, `value`, `entry_logits`, and for a network that
    reads the kill preview `with_preview` on the rows' entry `mask` and `preview`), then the cast
    heads with every row a hero's cast of spell 0 at no target."""
    rows = obs.shape[0]
    table: dict[str, Part] = {}
    counting: list[bool] = [False]

    def leaf_macs(module: nn.Module, args, out) -> float:
        if isinstance(module, nn.Linear):
            return module.in_features * module.out_features * out.numel() / module.out_features
        if isinstance(module, Block):
            x = args[0]
            return 2.0 * x.shape[1] * x.shape[1] * x.shape[2] * x.shape[0]
        if isinstance(module, nn.Conv2d):
            # A hex convolution's kernel counts its seven hexes (`hexnet.HEX_KERNEL`).
            mask = getattr(module, "kernel_mask", None)
            taps = float(mask.sum()) if mask is not None else float(np.prod(module.kernel_size))
            return out.numel() * module.in_channels / module.groups * taps
        return 0.0

    hooks = []
    for name, child in net.named_children():
        part = table[name] = Part(
            name, type(child).__name__, sum(p.numel() for p in child.parameters())
        )

        def record(module, args, out, part=part):
            if part.shape is None:
                part.shape = tuple((out[0] if isinstance(out, tuple) else out).shape)

        def count(module, args, out, part=part):
            macs = leaf_macs(module, args, out) / rows
            if counting[0]:
                part.cast_macs += macs
            else:
                part.macs += macs

        # A list of blocks is never called itself: its last block's output is the list's.
        shown = child[-1] if isinstance(child, nn.ModuleList) else child
        hooks.append(shown.register_forward_hook(record))
        for module in child.modules():
            hooks.append(module.register_forward_hook(count))
    own = sum(p.numel() for p in net.parameters(recurse=False))
    with torch.no_grad():
        h = net.encode(obs)
        net.value(h)
        logits = net.entry_logits(h)
        if net.reads_preview:
            net.with_preview(logits, h, obs, preview, mask)
        counting[0] = True
        entry = torch.full((rows,), int(CASTS[0]), dtype=torch.long, device=obs.device)
        cast = net.cast_of(entry)
        zero = torch.zeros(rows, dtype=torch.long, device=obs.device)
        net.spell_logits(h, cast)
        net.target_logits(h, cast, zero)
        net.second_logits(h, cast, zero, zero)
    for hook in hooks:
        hook.remove()
    out = list(table.values())
    if own:
        out.append(Part("(own)", "Parameter", own))
    return out


def table(rows: list[Part]) -> str:
    """The parts as a Markdown table with the totals: the multiply-adds of every decision, and a
    cast's extra ones (its three heads)."""
    lines = [
        "| part | kind | parameters | output | multiply-adds | a cast's |",
        "|---|---|---:|---|---:|---:|",
    ]
    for p in rows:
        shape = "" if p.shape is None else " × ".join(str(d) for d in p.shape[1:]) or "1"
        lines.append(
            f"| `{p.name}` | {p.kind} | {p.parameters:,} | {shape} | {p.macs:,.0f} | "
            f"{p.cast_macs:,.0f} |"
        )
    lines.append(
        f"| **total** | | **{sum(p.parameters for p in rows):,}** | | "
        f"**{sum(p.macs for p in rows):,.0f}** | **{sum(p.cast_macs for p in rows):,.0f}** |"
    )
    return "\n".join(lines)


def mermaid(net: Policy) -> str:
    """The data flow of `net` as a Mermaid flowchart."""
    if isinstance(net, Mlp):
        first, second = net.trunk[0], net.trunk[2]
        more = len(net.trunk) // 2 - 1
        return "\n".join(
            [
                "flowchart LR",
                f'  obs["observation v21<br/>{first.in_features:,}"] --> sl["symlog"]',
                f'  sl --> t1["Linear {first.in_features:,} → {first.out_features}<br/>ReLU"]',
                f'  t1 --> t2["{more} × Linear {second.in_features} → {second.out_features}'
                '<br/>ReLU"]',
                '  t2 --> value["value"]',
                f'  t2 --> entry["entry logits {net.entry_head.out_features:,}"]',
                '  t2 --> spell["spell logits"]',
                '  cast(["the cast"]) --> spell',
                '  spell --> se(["spell embedding"]) --> target["target logits"]',
                "  t2 --> target",
                '  target --> te(["target embedding"]) --> second["second target logits"]',
                "  t2 --> second",
                "  se --> second",
            ]
        )
    if isinstance(net, HexNet):
        return hex_mermaid(net)
    assert isinstance(net, Entity)
    w, n = net.width, len(net.blocks)
    return "\n".join(
        [
            "flowchart LR",
            '  obs["observation v21"] --> sl["symlog"]',
            '  sl --> stacks["42 stack slots × 99"]',
            '  sl --> glob["globals + public heroes"]',
            '  sl --> own["own hero"]',
            '  sl --> planes["16 hex planes × 255"]',
            f'  stacks --> st["stack tokens {w}<br/>+ creature, column, row embeddings"]',
            f'  glob --> gt["global token {w}"]',
            f'  own --> ot["own-hero token {w}"]',
            '  planes --> hx["hex vectors 255 × 16<br/>+ per-hex embedding"]',
            f'  hx --> ft["field token {w}"]',
            f'  st & gt & ot & ft --> enc["{n} × transformer block<br/>'
            f'{net.blocks[0].heads} heads, empty slots masked"]',
            '  enc --> ctx["context: global, active stack,<br/>mean of the stacks"]',
            '  ctx --> value["value"]',
            '  ctx --> fixed["15 fixed entries"]',
            '  hx --> move["move pointer: 255"]',
            "  ctx --> move",
            '  enc --> sp["stack pointer: 42 × (24 attacks,<br/>2 shots, heal)"]',
            "  ctx --> sp",
            '  fixed & move & sp --> entry["entry logits 1 404"]',
            '  ctx --> spell["spell logits"]',
            '  cast(["the cast"]) --> spell',
            '  spell --> se(["spell embedding"]) --> target["target pointer:<br/>'
            'none, 255 hexes, 42 stacks"]',
            "  ctx & hx & enc --> target",
            '  target --> te(["target embedding"]) --> second["second target pointer"]',
            "  ctx & hx & enc & se --> second",
        ]
    )


def hex_mermaid(net: HexNet) -> str:
    """The data flow of a hex network (`hexnet`) as a Mermaid flowchart."""
    w, n, g = net.width, len(net.blocks), net.grid
    if isinstance(net, HexConv):
        trunk = (
            f'  field --> enc["{n} × residual block: 2 hex convolutions<br/>'
            f'{g.height} × {g.columns} axial cells, global pooling"]'
        )
    else:
        trunk = (
            f'  field & glob --> enc["{n} × transformer block, {net.blocks[0].heads} heads<br/>'
            f'{g.hexes} hexes + global token, bias per axial offset"]'
        )
    return "\n".join(
        [
            "flowchart LR",
            '  obs["observation v21"] --> sl["symlog"]',
            '  sl --> stacks["42 stack slots × 99"]',
            '  sl --> glob["globals, heroes, own hero"]',
            '  sl --> planes["16 hex planes × 255"]',
            f'  stacks --> st["stack tokens {w}<br/>+ creature embedding"]',
            f'  planes --> hx["hex vectors {g.hexes} × {w}<br/>+ per-hex embedding"]',
            '  st & hx & glob --> field["the field: stacks on their hexes,<br/>'
            'the global vector on every hex"]',
            trunk,
            '  enc --> sv["stack vectors: token + the hex under it"]',
            "  st --> sv",
            '  enc & sv & glob --> ctx["context: global, hexes mean and max,<br/>'
            'active stack, mean of the stacks"]',
            '  ctx --> value["value"]',
            '  ctx --> fixed["15 fixed entries"]',
            '  enc --> move["move pointer: 255"]',
            "  ctx --> move",
            '  sv --> sp["stack pointer: 42 × (24 attacks,<br/>2 shots, heal)"]',
            "  ctx --> sp",
            '  fixed & move & sp --> entry["entry logits 1 404"]',
            '  ctx --> spell["spell logits"]',
            '  cast(["the cast"]) --> spell',
            '  spell --> se(["spell embedding"]) --> target["target pointer:<br/>'
            'none, 255 hexes, 42 stacks"]',
            "  ctx & enc & sv --> target",
            '  target --> te(["target embedding"]) --> second["second target pointer"]',
            "  ctx & enc & sv & se --> second",
        ]
    )


def timing(net: Policy, batch: BattleBatch, repeats: int) -> dict[str, float | None]:
    """Microseconds per decision of a call's forward pass (`encode`, `value`, `entry_logits` on
    the batch's rows), of a training pass (`evaluate` and its backward) and of a sample-pass (a
    training pass, the gradient's clipping and an AdamW step at learning rate 0, so the network
    stays as it was: a bench's or PPO's minibatch step, E25 T24), each the median of `repeats`
    after one warm-up; and on CUDA the peak memory of a sample-pass above what was allocated
    before it (`peak_mb`; None on the CPU)."""
    device = next(net.parameters()).device
    cuda = device.type == "cuda"
    obs = torch.from_numpy(batch.obs).to(device)
    mask = torch.from_numpy(batch.entry_mask).to(device).bool()
    preview = None if batch.preview is None else torch.from_numpy(batch.preview).to(device)
    rows = obs.shape[0]
    entry = mask.float().argmax(dim=1)
    cast = torch.zeros(rows, dtype=torch.bool, device=device)
    zero = torch.zeros(rows, dtype=torch.long, device=device)
    heads = (mask, zero, mask, zero, mask, zero)
    adam = optimizer(net, 0.0)

    def forward() -> None:
        with torch.no_grad():
            h = net.encode(obs)
            net.value(h)
            net.with_preview(net.entry_logits(h), h, obs, preview, mask)

    def train() -> None:
        logp, ent, value = net.evaluate(obs, mask, entry, cast, heads, preview)
        (logp.sum() + ent.sum() + value.sum()).backward()
        net.zero_grad(set_to_none=True)

    def sample_pass() -> None:
        logp, ent, value = net.evaluate(obs, mask, entry, cast, heads, preview)
        adam.zero_grad()
        (logp.sum() + ent.sum() + value.sum()).backward()
        torch.nn.utils.clip_grad_norm_(net.parameters(), 0.5)
        adam.step()

    out: dict[str, float | None] = {"peak_mb": None}
    for name, step in (("forward", forward), ("training", train), ("sample_pass", sample_pass)):
        seconds = []
        for i in range(repeats + 1):
            if cuda:
                torch.cuda.synchronize()
                base = torch.cuda.memory_allocated()
                torch.cuda.reset_peak_memory_stats()
            start = time.perf_counter()
            step()
            if cuda:
                torch.cuda.synchronize()
            if i:
                seconds.append(time.perf_counter() - start)
            if cuda and name == "sample_pass":
                peak = (torch.cuda.max_memory_allocated() - base) / 2**20
                out["peak_mb"] = max(out["peak_mb"] or 0.0, peak)
        out[name] = float(np.median(seconds)) * 1e6 / rows
    net.zero_grad(set_to_none=True)
    return out


def bench_table(
    bench: Bench,
    base: Config,
    batch: BattleBatch,
    device: torch.device,
    repeats: int | None,
    log: Callable[[str], None] = print,
) -> str:
    """Every student of `bench` (its `[network]` keys over `base`) on the batch's rows as a
    Markdown table, each line logged as it comes: its parameters and multiply-adds a decision,
    and with `repeats` its `timing` (E25 T24: a bench's candidates side by side, their cost before
    a run)."""
    obs = torch.from_numpy(batch.obs).to(device)
    mask = torch.from_numpy(batch.entry_mask).to(device).bool()
    lines = [
        "| student | network | parameters | multiply-adds | forward µs | training µs "
        "| sample-pass µs | peak MB |",
        "|---|---|---:|---:|---:|---:|---:|---:|",
    ]
    for line in lines:
        log(line)
    for name, keys in bench.students.items():
        torch.manual_seed(0)
        net = build(asdict(student_config(base, keys)), batch).to(device)
        counted = parts(net, obs, mask)
        t = timing(net, batch, repeats) if repeats else {}

        def us(key: str, t=t) -> str:
            x = t.get(key)
            return "-" if x is None else f"{x:,.1f}"

        network = ", ".join(f"{k} {v}" for k, v in keys.items() if k not in STUDENT_KEYS)
        line = (
            f"| {name} | {network} | {sum(p.parameters for p in counted):,} | "
            f"{sum(p.macs for p in counted):,.0f} | {us('forward')} | {us('training')} | "
            f"{us('sample_pass')} | {us('peak_mb')} |"
        )
        log(line)
        lines.append(line)
        del net
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="python -m h3env.train.network", description=__doc__)
    parser.add_argument("--config", type=Path, default=None, help="a run's TOML config file")
    parser.add_argument("--set", action="append", default=[], metavar="TABLE.KEY=VALUE")
    parser.add_argument("--from", dest="saved", type=Path, default=None, metavar="FILE")
    parser.add_argument(
        "--bench",
        type=Path,
        default=None,
        metavar="FILE",
        help="a bench's TOML file: every student of it in one table (over its `run` config, "
        "else over --config and --set)",
    )
    parser.add_argument(
        "--rows", type=int, default=256, help="the observations (a call's envs, a minibatch's rows)"
    )
    parser.add_argument(
        "--time", action="store_true", help="time a forward, a training pass and a sample-pass"
    )
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--device", default="cpu")
    parser.add_argument("--mermaid", type=Path, default=None, metavar="FILE.md")
    args = parser.parse_args(argv)
    device = bench_device(args.device)
    if args.bench is not None:
        if args.saved is not None or args.mermaid is not None:
            parser.error("--bench times a bench's students: no --from, no --mermaid")
        bench = load_bench(args.bench.read_text(encoding="utf-8"))
        path = Path(bench.run) if bench.run else args.config
        base = load_config(path.read_text(encoding="utf-8") if path else "", args.set)
        batch = BattleBatch(args.rows, seed=base.seed)
        try:
            batch.add_encounters(kinds=base.kinds)
            batch.reset()
            print(
                f"# The students of {args.bench} over {path or 'the defaults'} ({args.rows} "
                f"observations on {device}, {torch.get_num_threads()} threads)\n"
            )
            bench_table(bench, base, batch, device, args.repeats if args.time else None)
        finally:
            batch.close()
        return
    if args.saved is not None:
        config = torch.load(args.saved, map_location="cpu", weights_only=True)["config"]
        source = str(args.saved)
    else:
        text = args.config.read_text(encoding="utf-8") if args.config else ""
        config = asdict(load_config(text, args.set))
        source = str(args.config or "the defaults")
    batch = BattleBatch(args.rows, seed=config["seed"], preview=config.get("preview", False))
    try:
        batch.add_encounters(kinds=config.get("kinds"))
        batch.reset()
        net = build(config, batch).to(device)
        if args.saved is not None:
            saved = torch.load(args.saved, map_location=device, weights_only=True)
            net.load_state_dict(saved["policy"])
        net.eval()
        obs = torch.from_numpy(batch.obs).to(device)
        kind = config.get("kind", "mlp")
        title = f"The {kind} network of {source} ({args.rows} observations)"
        mask = torch.from_numpy(batch.entry_mask).to(device).bool()
        preview = None if batch.preview is None else torch.from_numpy(batch.preview).to(device)
        rows = parts(net, obs, mask, preview)
        lines = [f"# {title}", "", table(rows)]
        if args.time:
            t = timing(net, batch, args.repeats)
            peak = "" if t["peak_mb"] is None else f", its peak {t['peak_mb']:,.0f} MB"
            lines += [
                "",
                f"Per decision on {device}: forward {t['forward']:.1f} µs, "
                f"training pass {t['training']:.1f} µs, sample-pass {t['sample_pass']:.1f} µs"
                f"{peak} ({torch.get_num_threads()} threads).",
            ]
    finally:
        batch.close()
    text = "\n".join(lines)
    print(text)
    if args.mermaid is not None:
        args.mermaid.write_text(f"{text}\n\n```mermaid\n{mermaid(net)}\n```\n", encoding="utf-8")
        print(f"\nwrote {args.mermaid}")


if __name__ == "__main__":
    main()
