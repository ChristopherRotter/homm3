"""Ports H3API's names onto this build of Heroes3.exe: `scripts/exe.py port-h3api DIR`.

H3API (https://github.com/RoseKavalier/H3API, MIT) reverse-engineered the 3.2 build of the
executable. The user's GOG Complete build is a close relative whose code moved by a constant
amount per region (0x2E0 around 0x44xxxx, -0x310 around 0x5Axxxx, -0x1590 around 0x50Bxxx), while
most of `.data` stayed in place. Without the 3.2 binary, a function is matched when

1. the shifted address is a function start in this build (a call target, a code pointer or a
   tail jump, `scripts/exe.py`),
2. the shift is shared by H3API's other functions nearby (their shifted addresses are starts too:
   a random shift hits a start for about one in twenty 16-byte slots), and
3. the return pops what H3API's calling convention and argument count say (`ret 4 * n`).

Everything ported is a guess (docs/exe/README.md, "Confidence"); the evidence column says how
good the match is. The output is regenerated, never edited by hand.
"""

from __future__ import annotations

import bisect
import datetime
import re
import subprocess
from collections import defaultdict
from pathlib import Path

CALL_RE = re.compile(r"\b(THISCALL|FASTCALL|STDCALL|CDECL)_(\d+)\(")
DEF_RE = re.compile(r"_H3API_\s+[^;{}()]*?([~\w]+(?:::[~\w]+)*)\s*\(")
STRUCT_RE = re.compile(r"\bstruct\s+(H3\w+)\s*(?::[^{;]*)?\{")
SIZE_RE = re.compile(r"_H3API_SIZE_\((0x[0-9A-Fa-f]+)\)")
FIELD_RE = re.compile(
    r"/\*\*\s*@brief\s*\[([0-9A-Fa-f]+)\]\s*(.*?)\*/\s*([^;{}/]+?);", re.S
)
INFO_RE = re.compile(r"_H3API_GET_INFO_\((0x[0-9A-Fa-f]+)(?:\s*\+\s*(\d+))?\s*,\s*(\w+)\)")
MAX_SHIFT = 0x8000
NEAREST = 3  # neighbours on each side that vote for a shift
STRONG = 3  # votes that make a shift a region's anchor
WEAK_SPREAD = 0x400  # how far a weak own shift may differ from its region's

MIT = """\
# H3API is Copyright (c) 2021 RoseKavalier and licensed under the MIT License:
#
# Permission is hereby granted, free of charge, to any person obtaining a copy of this software
# and associated documentation files (the "Software"), to deal in the Software without
# restriction, including without limitation the rights to use, copy, modify, merge, publish,
# distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
# Software is furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all copies or
# substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
# BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
# DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"""


def split_args(text: str, start: int) -> tuple[list[str], int]:
    """The comma-separated arguments of the call whose `(` is just before `start`."""
    depth, args, current, i = 1, [], [], start
    while i < len(text) and depth:
        c = text[i]
        if c in "(<[":
            depth += 1
        elif c in ")>]":
            depth -= 1
            if depth == 0:
                break
        if c == "," and depth == 1:
            args.append("".join(current).strip())
            current = []
        else:
            current.append(c)
        i += 1
    args.append("".join(current).strip())
    return args, i


def enclosing_struct(text: str, pos: int) -> str | None:
    best = None
    for m in STRUCT_RE.finditer(text, 0, pos):
        best = m.group(1)
    return best


def parse_functions(root: Path) -> dict[int, dict]:
    found: dict[int, dict] = {}
    for path in sorted(root.glob("include/**/*")):
        if path.suffix not in (".cpp", ".hpp", ".inl"):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for m in CALL_RE.finditer(text):
            args, _ = split_args(text, m.end())
            if len(args) < 2 or not re.fullmatch(r"0x[0-9A-Fa-f]+", args[1]):
                continue
            va = int(args[1], 16)
            name = None
            for d in DEF_RE.finditer(text, 0, m.start()):
                name = d.group(1)
            if name is None:
                continue
            if "::" not in name:
                owner = enclosing_struct(text, m.start()) if path.suffix == ".hpp" else None
                name = f"{owner}::{name}" if owner else name
            entry = {
                "h3api_va": va,
                "name": name,
                "conv": m.group(1).lower(),
                "argc": int(m.group(2)),
                "ret": " ".join(args[0].split()),
                "params": ", ".join(" ".join(a.split()) for a in args[2:]),
                "file": str(path.relative_to(root / "include")),
            }
            if va in found and found[va]["name"] != name:
                aliases = found[va].setdefault("aliases", [])
                if name not in aliases:
                    aliases.append(name)
                continue
            found.setdefault(va, entry)
    return found


def parse_structs(root: Path) -> list[dict]:
    structs = []
    for path in sorted(root.glob("include/**/*.hpp")):
        text = path.read_text(encoding="utf-8", errors="replace")
        for m in STRUCT_RE.finditer(text):
            open_brace = m.end() - 1
            depth, i = 0, open_brace
            while i < len(text):
                if text[i] == "{":
                    depth += 1
                elif text[i] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            body = text[open_brace + 1 : i]
            size = SIZE_RE.search(body)
            if not size:
                continue
            # Fields at the struct's own level only: drop nested bodies (unions, inner structs).
            flat, depth = [], 0
            for c in body:
                if c == "{":
                    depth += 1
                if depth == 0:
                    flat.append(c)
                if c == "}":
                    depth -= 1
            fields = []
            for f in FIELD_RE.finditer("".join(flat)):
                decl = " ".join(f.group(3).split())
                if "(" in decl or decl.startswith(("using", "typedef", "return")):
                    continue
                ident = re.search(r"([A-Za-z_]\w*)\s*(\[[^\]]*\])*\s*(:\s*\d+)?$", decl)
                if not ident:
                    continue
                fname = ident.group(1)
                ftype = decl[: ident.start()].strip() + (ident.group(2) or "")
                if ident.group(3):
                    ftype += f" {ident.group(3).strip()}"
                fields.append({
                    "offset": int(f.group(1), 16),
                    "name": fname,
                    "type": ftype,
                    "summary": " ".join(f.group(2).split()),
                })
            if fields:
                structs.append({
                    "name": m.group(1),
                    "size": int(size.group(1), 16),
                    "file": str(path.relative_to(root / "include")),
                    "fields": fields,
                })
    return structs


def parse_data(root: Path) -> list[dict]:
    found = []
    for path in sorted(root.glob("include/**/*.hpp")):
        text = path.read_text(encoding="utf-8", errors="replace")
        for m in INFO_RE.finditer(text):
            found.append({
                "h3api_va": int(m.group(1), 16),
                "operand": int(m.group(2) or 0),
                "name": m.group(3),
                "file": str(path.relative_to(root / "include")),
            })
    return found


def expected_pop(conv: str, argc: int) -> int:
    if conv == "thiscall":
        return 4 * max(0, argc - 1)
    if conv == "fastcall":
        return 4 * max(0, argc - 2)
    if conv == "stdcall":
        return 4 * argc
    return 0


def pops(code, start: int) -> set[int]:
    out = set()
    for va, _, mnem, op in code.instructions(start, code.function_end(start)):
        if mnem == "ret":
            out.add(int(op, 0) if op else 0)
    return out


def match_functions(found: dict[int, dict], code) -> tuple[list[dict], list[dict]]:
    """Pick each function's shift: its own if its nearest neighbours share it, else theirs."""
    starts = sorted(code.detected)
    start_set = set(starts)
    addrs = sorted(found)
    pop_cache: dict[int, set[int]] = {}

    def ret_state(a: int, va: int) -> str:
        if va not in pop_cache:
            pop_cache[va] = pops(code, va)
        have = pop_cache[va]
        want = expected_pop(found[a]["conv"], found[a]["argc"])
        if not have:
            return "no ret"
        if want in have:
            return "ret ok"
        return "ret " + "/".join(f"0x{p:X}" for p in sorted(have)) + f" not 0x{want:X}"

    def fits(a: int, d: int) -> bool:
        return a + d in start_set and not ret_state(a, a + d).startswith("ret 0x")

    def nearest(i: int) -> list[int]:
        return addrs[max(0, i - NEAREST) : i] + addrs[i + 1 : i + 1 + NEAREST]

    own: dict[int, tuple[int, int]] = {}  # h3api va -> (shift, support)
    for i, a in enumerate(addrs):
        lo = bisect.bisect_left(starts, a - MAX_SHIFT)
        hi = bisect.bisect_right(starts, a + MAX_SHIFT)
        best = None
        for s in starts[lo:hi]:
            d = s - a
            if not fits(a, d):
                continue
            support = sum(1 for b in nearest(i) if fits(b, d))
            key = (support, -abs(d))
            if best is None or key > best[0]:
                best = (key, d, support)
        if best is not None:
            own[a] = (best[1], best[2])
    strong = sorted(a for a, (_, support) in own.items() if support >= STRONG)
    ported, unmatched = [], []
    taken: dict[int, int] = {}
    for a in addrs:
        f = found[a]
        options = []  # (shift, how)
        if a in own and own[a][1] >= STRONG:
            options.append((own[a][0], f"{own[a][1]} of the {2 * NEAREST} nearest share it"))
        j = bisect.bisect_left(strong, a)
        for b in sorted(strong[max(0, j - 1) : j + 1], key=lambda b: abs(b - a)):
            if b != a:
                options.append((own[b][0], f"as 0x{b:06X} nearby"))
        if a in own and own[a][1] < STRONG:
            # A weak shift of its own is believed only near its region's: functions can move
            # between builds (H3API's Rand(low, high) 0x50C7C0 is 0x50B230 here, read in §7.5).
            near = [own[b][0] for b in strong[max(0, j - 1) : j + 1]]
            if any(abs(own[a][0] - d) <= WEAK_SPREAD for d in near):
                options.append(
                    (own[a][0], f"weak: own shift, {own[a][1]} of the {2 * NEAREST} nearest share it")
                )
        chosen = None
        for d, how in options:
            if a + d in start_set:
                state = ret_state(a, a + d)
                if not state.startswith("ret 0x"):
                    chosen = (d, how, state)
                    break
                chosen = chosen or (d, how, state)
        if chosen is None:
            unmatched.append({**f, "reason": "no function start at any nearby shift"})
            continue
        d, how, state = chosen
        va = a + d
        sign = "-" if d < 0 else "+"
        evidence = f"shift {sign}0x{abs(d):X} ({how}), {state}"
        if va in taken:
            unmatched.append({**f, "reason": f"0x{va:06X} already taken by 0x{taken[va]:06X}"})
            continue
        taken[va] = a
        ported.append({**f, "va": va, "evidence": evidence})
    return ported, unmatched


def this_loads(code, img, sites: list[int]) -> dict[int, int]:
    """Where the code loads `this` (ecx) from a global before each call site."""
    counts: dict[int, int] = defaultdict(int)
    for site in sites:
        i = code.index(site)
        for va, _, mnem, op in reversed(code.ins[max(0, i - 8) : i]):
            if mnem == "mov" and op.startswith("ecx, "):
                m = re.fullmatch(r"ecx, dword ptr \[(0x[0-9a-f]+)\]", op)
                if m and not img.is_code(int(m.group(1), 16)):
                    counts[int(m.group(1), 16)] += 1
                break
            if mnem in ("call", "ret", "jmp") or op.startswith("ecx") and mnem not in ("push", "cmp", "test"):
                break
    return counts


def match_data(found: list[dict], img, code, ported: list[dict]) -> tuple[list[dict], list[dict]]:
    """Operands in code move with their function's region; a global pointer is found where the
    callers of its class's methods load `this`, or by the shift of the nearest such global."""
    shifts = sorted((f["h3api_va"], f["va"] - f["h3api_va"]) for f in ported)
    keys = [s[0] for s in shifts]
    methods: dict[str, list[int]] = defaultdict(list)
    for f in ported:
        if f["conv"] == "thiscall" and "::" in f["name"]:
            methods[f["name"].split("::")[0]].append(f["va"])
    out, unmatched, pending = [], [], []
    for f in found:
        a = f["h3api_va"]
        if img.is_code(a):
            j = bisect.bisect_left(keys, a)
            near = sorted(shifts[max(0, j - 2) : j + 2], key=lambda s: abs(s[0] - a))
            hit = None
            for _, d in near:
                site = a + d
                ins = code.ins[code.index(site)]
                value = img.unpack("<I", site + f["operand"])
                if ins[0] == site and value is not None and img.section_of(value) in (".rdata", ".data"):
                    hit = (site, value)
                    break
            if hit is None:
                unmatched.append({**f, "reason": "no instruction with a data operand at the nearby shifts"})
                continue
            site, value = hit
            out.append({
                "va": value, "h3api_va": None, "name": f["name"],
                "what": "table or object (H3API takes its address from an instruction)",
                "evidence": f"the operand at 0x{site:06X}+{f['operand']} (H3API 0x{a:06X}, shifted like its region)",
            })
            continue
        sites = [s for va in methods.get(f["name"], []) for s in code.calls.get(va, [])]
        loads = {g: n for g, n in this_loads(code, img, sites).items() if abs(g - a) <= 0x1000}
        if loads:
            g, n = max(loads.items(), key=lambda kv: kv[1])
            out.append({
                "va": g, "h3api_va": a, "name": f["name"],
                "what": f"pointer to the {f['name']}",
                "evidence": f"`this` of {n} of {len(sites)} calls to its methods is loaded from here",
            })
        else:
            pending.append(f)
    anchors = sorted((d["h3api_va"], d["va"] - d["h3api_va"]) for d in out if d["h3api_va"])
    for f in pending:
        a = f["h3api_va"]
        by_distance = sorted(anchors, key=lambda x: abs(x[0] - a))
        # The nearest anchor's shift first, then the same address (the low .data did not move):
        # a pointer holding a static table settles it.
        candidates = [
            (d, f"shift {d:+#x} as the pointer at 0x{b + d:06X}") for b, d in by_distance[:2]
        ] + [(0, "same address")]
        best = None
        for d, how in candidates:
            refs = len(code.refs.get(a + d, []))
            value = img.unpack("<I", a + d) or 0
            static = img.section_of(value) in (".rdata", ".data")
            if refs and (best is None or (static and not best[3])):
                best = (d, how, refs, static, value)
        if best is None:
            unmatched.append({**f, "reason": "nothing refers to it at the nearby shifts"})
            continue
        d, how, refs, static, value = best
        held = f"holds 0x{value:06X} (a static table)" if static else "zero until run time"
        out.append({
            "va": a + d, "h3api_va": a, "name": f["name"],
            "what": f"pointer to the {f['name']} (or its array)",
            "evidence": f"{how}; {refs} code references; {held}",
        })
    seen, unique = set(), []
    for d in out:
        if d["va"] not in seen:
            seen.add(d["va"])
            unique.append(d)
    return unique, unmatched


def q(text: str) -> str:
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def port(root: Path, img, code, out_path: Path) -> None:
    if not (root / "include/H3API.hpp").is_file():
        raise SystemExit(f"{root} is no H3API clone (git clone --depth 1 "
                         "https://github.com/RoseKavalier/H3API.git)")
    commit = subprocess.run(["git", "-C", str(root), "log", "-1", "--format=%H %cs"],
                            capture_output=True, text=True).stdout.split()
    found = parse_functions(root)
    ported, unmatched = match_functions(found, code)
    data, data_unmatched = match_data(parse_data(root), img, code, ported)
    structs = parse_structs(root)
    lines = [
        "# Names from H3API ported onto this build of Heroes3.exe. GENERATED by",
        "# `scripts/exe.py port-h3api <H3API clone>`; do not edit (docs/exe/README.md).",
        "#",
        "# Every name here is a GUESS: H3API reverse-engineered the 3.2 build, and the port",
        "# matches its addresses onto the GOG Complete 4.0 build by region shift, function starts",
        "# and the return's stack cleanup (scripts/exe_h3api.py). A reading that confirms a name",
        "# goes into docs/exe/map.toml (ours), never here. Struct layouts are H3API's, unchecked.",
        "#",
        MIT.rstrip(),
        "",
        "# va: this build; h3api_va: H3API's (3.2); conv and argc: H3API's calling convention and",
        "# argument count (thiscall counts `this`); evidence: why the match is believed.",
        "function = [",
    ]
    for f in sorted(ported, key=lambda f: f["va"]):
        aliases = f", aliases = [{', '.join(q(a) for a in f['aliases'])}]" if "aliases" in f else ""
        lines.append(
            f"  {{ va = 0x{f['va']:06X}, h3api_va = 0x{f['h3api_va']:06X}, name = {q(f['name'])}, "
            f"conv = {q(f['conv'])}, argc = {f['argc']}, ret = {q(f['ret'])}, "
            f"params = {q(f['params'])}, evidence = {q(f['evidence'])}{aliases} }},"
        )
    lines += ["]", "", "# Pointers and tables H3API names (_H3API_GET_INFO_).", "data = ["]
    for d in sorted(data, key=lambda d: d["va"]):
        lines.append(
            f"  {{ va = 0x{d['va']:06X}, name = {q(d['name'])}, what = {q(d['what'])}, "
            + (f"h3api_va = 0x{d['h3api_va']:06X}, " if d["h3api_va"] else "")
            + f"evidence = {q(d['evidence'])} }},"
        )
    lines += ["]", "", "# H3API functions and data not matched in this build (improve the matcher or",
              "# find them by reading).", "unmatched = ["]
    for f in sorted(unmatched + data_unmatched, key=lambda f: f["h3api_va"]):
        lines.append(f"  {{ h3api_va = 0x{f['h3api_va']:06X}, name = {q(f['name'])}, "
                     f"reason = {q(f['reason'])} }},")
    lines += [
        "]",
        "",
        "[source]",
        'repo = "https://github.com/RoseKavalier/H3API"',
        f"commit = {q(commit[0] if commit else 'unknown')}",
        f"committed = {q(commit[1] if len(commit) > 1 else 'unknown')}",
        f"ported = {q(datetime.date.today().isoformat())}",
        'license = "MIT"',
        f"functions_parsed = {len(found)}",
        f"functions_ported = {len(ported)}",
        f"data_ported = {len(data)}",
        f"structs = {len(structs)}",
        "",
    ]
    for s in structs:
        lines += [
            "[[struct]]",
            f"name = {q(s['name'])}",
            f"size = 0x{s['size']:X}",
            f"file = {q(s['file'])}",
            "fields = [",
        ]
        for fld in s["fields"]:
            lines.append(
                f"  {{ offset = 0x{fld['offset']:X}, name = {q(fld['name'])}, type = {q(fld['type'])}, "
                f"summary = {q(fld['summary'])} }},"
            )
        lines += ["]", ""]
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines))
    print(f"H3API: {len(found)} functions parsed, {len(ported)} ported, "
          f"{len([u for u in unmatched])} unmatched; {len(data)} data items, "
          f"{len(structs)} structs -> {out_path}")
