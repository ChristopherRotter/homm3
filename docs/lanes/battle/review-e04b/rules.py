#!/usr/bin/env python3
"""Print rule entries of the catalog as readable text.

  rules.py BAT-MAG-009 BAT-MAG-017      # an inclusive id range (same prefix)
  rules.py --ids HER-SPL-002,HER-BAT-015 # single ids
  rules.py --brief BAT-MAG-009 BAT-MAG-017   # id, status, title only
"""
import sys
import tomllib
from pathlib import Path

ROOT = Path("/home/claude/homm3/docs/rules/catalog")


def load():
    rules = {}
    for f in sorted(ROOT.glob("*.toml")):
        for r in tomllib.load(open(f, "rb")).get("rule", []):
            rules[r["id"]] = r
    return rules


def show(r, brief):
    if brief:
        print(f'{r["id"]}  [{r["status"]}]  {r["title"]}')
        return
    print(f'=== {r["id"]}  [{r["status"]}]  {r["title"]}')
    print(f'numeric: {r.get("numeric")}   quirks: {r.get("quirks", [])}')
    print(f'summary: {r["summary"]}')
    for s in r.get("source", []):
        head = f'  - source {s["kind"]} (confidence {s.get("confidence")}, retrieved {s.get("retrieved")})'
        print(head)
        print(f'      ref: {s.get("ref")}')
        if s.get("url"):
            print(f'      url: {s["url"]}')
        if s.get("note"):
            print(f'      note: {s["note"]}')
    print()


def main():
    args = sys.argv[1:]
    brief = "--brief" in args
    args = [a for a in args if a != "--brief"]
    rules = load()
    if args and args[0] == "--ids":
        ids = args[1].split(",")
    else:
        lo, hi = args
        prefix = lo.rsplit("-", 1)[0]
        a, b = int(lo.rsplit("-", 1)[1]), int(hi.rsplit("-", 1)[1])
        ids = [f"{prefix}-{n:03d}" for n in range(a, b + 1)]
    for i in ids:
        if i in rules:
            show(rules[i], brief)
        else:
            print(f"=== {i}: not in the catalog\n")


if __name__ == "__main__":
    main()
