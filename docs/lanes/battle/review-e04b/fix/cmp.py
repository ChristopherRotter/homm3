#!/usr/bin/env python3
"""usage: cmp.py <dir A or 'golden'> <dir B>: compares the replays of two recordings."""
import sys, tomllib, pathlib
S = pathlib.Path('/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad/b-logs/rep')
G = pathlib.Path('/home/claude/wt-b/fixtures/replays')
def d(x): return G if x == 'golden' else S / x
a, b = d(sys.argv[1]), d(sys.argv[2])
def find(doc, key):
    # the key may sit in a table
    if key in doc: return doc[key]
    for v in doc.values():
        if isinstance(v, dict) and key in v: return v[key]
    return None
for f in sorted(b.glob('*.toml')):
    x = tomllib.loads((a / f.name).read_text()); y = tomllib.loads(f.read_text())
    cx, cy = find(x, 'commands'), find(y, 'commands')
    hx, hy = find(x, 'checkpoints'), find(y, 'checkpoints')
    fx, fy = find(x, 'final_hash'), find(y, 'final_hash')
    first_cmd = next((i + 1 for i, (p, q) in enumerate(zip(cx, cy)) if p != q), None)
    if first_cmd is None and len(cx) != len(cy): first_cmd = min(len(cx), len(cy)) + 1
    first_cp = next(((i + 1) * 25 for i, (p, q) in enumerate(zip(hx, hy)) if p != q), None)
    wx, wy = find(x, 'winner'), find(y, 'winner')
    if cx == cy and hx == hy and fx == fy:
        print(f'{f.name}: identical')
    else:
        print(f'{f.name}: commands {len(cx)} -> {len(cy)}, first differing command: {first_cmd}; first differing checkpoint: after {first_cp}; final hash {"same" if fx == fy else "differs"}; winner {wx} -> {wy}')
