#!/usr/bin/env python3
"""Print catalog entries by id: rules.py [--short] ID...  (searches battle.toml and heroes.toml)."""
import sys, re, tomllib
root = "/home/claude/homm3/docs/rules/catalog/"
short = "--short" in sys.argv
ids = [a for a in sys.argv[1:] if not a.startswith("--")]
rules = {}
for f in ("battle.toml", "heroes.toml", "kernel.toml"):
    with open(root + f, "rb") as fh:
        for r in tomllib.load(fh).get("rule", []):
            rules[r["id"]] = r
for i in ids:
    r = rules.get(i)
    if not r:
        print(f"{i}: not found"); continue
    if short:
        print(f"{i} [{r.get('status')}] {r.get('title')}"); continue
    print(f"== {i} [{r.get('status')}] {r.get('title')}")
    for k in ("summary", "quirks", "numeric", "notes", "note", "scope"):
        if k in r: print(f"  {k}: {r[k]}")
    for s in r.get("source", []):
        print(f"  - source {s.get('kind')} ({s.get('confidence','?')}): {s.get('ref')}")
        if s.get("url"): print(f"      url: {s['url']}")
        if s.get("note"): print(f"      note: {s['note']}")
    print()
