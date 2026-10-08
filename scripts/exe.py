#!/usr/bin/env python3
"""Code reading in `Heroes3.exe` with the shared address map (docs/exe/README.md).

The tool disassembles the user's executable on the fly and annotates it with the names in
`docs/exe/map.toml` (our readings), `docs/exe/structs.toml` (record layouts) and
`docs/exe/h3api.toml` (names ported from H3API, guesses). Queries print one function, its callers
or callees, the references to a table, a search over the instructions, or data values, so an
agent reads what it needs instead of grepping a 30 MB listing.

Setup: Python 3.11+, `pip install capstone pefile` (add `--break-system-packages` in a sandbox),
and the executable at `original-data/install/Heroes3.exe` (staged from the user's repo folder,
`docs/08-work-process.md`, "Cloud sandbox"). Nothing this tool prints may be committed: findings
go into the map and the docs in our own words and numbers (OPEN-QUESTIONS Q29, Q57).

Commands (ADDR is hexadecimal, `0x5A8090` or `5a8090`; NAME is a map or H3API name):

  fn ADDR|NAME               the function containing ADDR, annotated
  callers ADDR|NAME          call sites of a function, grouped by the calling function
  callees ADDR|NAME          the calls a function makes, in order
  refs ADDR|NAME             every instruction referring to an address (or into a table's range)
  grep REGEX [--fn X]        instructions matching REGEX (lower case text), with their function
  lookup ADDR|NAME|TEXT      what the map knows about an address, or entries matching a text
  data ADDR [--type T] [--count N] [--stride S]
                             values in the image (T: u8 i8 u16 i16 u32 i32 f32 f64 ptr str)
  listing [--out PATH]       the whole annotated listing (default original-data/exe/heroes3.asm)
  cited                      addresses cited in docs/ and crates/ that no map entry covers
  check [--fix-sizes]        the map against the executable (starts, sizes, kinds)
  stats                      how much of the executable the map names
  port-h3api DIR             regenerate docs/exe/h3api.toml from an H3API clone (README)
"""

from __future__ import annotations

import argparse
import bisect
import hashlib
import re
import signal
import struct
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXE = ROOT / "original-data/install/Heroes3.exe"
# docs/formats/installation.md §1: the GOG Complete edition's executable (SoD 4.0, 2000-09-08).
SHA256 = "aac39e8cf7ff46fb009057bd53716f4be46a88a49b857165b8d19e04df9d3d7d"
MAP = ROOT / "docs/exe/map.toml"
STRUCTS = ROOT / "docs/exe/structs.toml"
H3API = ROOT / "docs/exe/h3api.toml"
OUT = ROOT / "original-data/exe"

KINDS = ("function", "site", "table", "global")
CONFIDENCES = ("read", "inferred", "guess")
PAD = (0x90, 0xCC)
HEX_RE = re.compile(r"0x[0-9a-f]+")
CITE_RE = re.compile(r"(?<![0-9A-Za-z_])0x([0-9A-Fa-f]{6})(?![0-9A-Za-z_])")
JMP_TABLE_RE = re.compile(r"dword ptr \[(e[a-z]{2})\*4 \+ (0x[0-9a-f]+)\]")
BYTE_TABLE_RE = re.compile(r"(e[a-d]x|[a-d]l), byte ptr \[(e[a-z]{2}) \+ (0x[0-9a-f]+)\]")
CMP_RE = re.compile(r"(e[a-z]{2}|[a-z]{2}), (0x[0-9a-f]+|\d+)$")
FLOAT_MNEMS = re.compile(r"^(f[a-z]+|(add|sub|mul|div|mov|comi|ucomi|cvt[a-z0-9]*)s[sd])$")


def die(message: str) -> None:
    print(f"exe.py: {message}", file=sys.stderr)
    sys.exit(1)


def parse_va(text: str) -> int | None:
    t = text.strip().lower().removeprefix("0x")
    if re.fullmatch(r"[0-9a-f]{5,8}", t):
        return int(t, 16)
    return None


def hx(va: int) -> str:
    return f"0x{va:06X}"


# --------------------------------------------------------------------------------------------
# The image


class Image:
    """The executable's sections, imports and typed reads (no I/O after loading)."""

    def __init__(self, path: Path):
        try:
            import pefile
        except ImportError:
            die("needs `pip install capstone pefile`")
        if not path.is_file():
            die(f"{path.relative_to(ROOT)} is missing: stage Heroes3.exe (docs/exe/README.md)")
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        if digest != SHA256:
            die(f"{path.name} has SHA-256 {digest}; the map is for {SHA256} only")
        pe = pefile.PE(data=data)
        self.base = pe.OPTIONAL_HEADER.ImageBase
        self.entry = self.base + pe.OPTIONAL_HEADER.AddressOfEntryPoint
        self.end = self.base + pe.OPTIONAL_HEADER.SizeOfImage
        self.sections: list[tuple[str, int, int, bytes]] = []
        for s in pe.sections:
            name = s.Name.rstrip(b"\0").decode()
            raw = s.get_data()[: s.Misc_VirtualSize]
            self.sections.append((name, self.base + s.VirtualAddress, s.Misc_VirtualSize, raw))
        text = self.section(".text")
        self.text_start, self.text_end = text[1], text[1] + text[2]
        self.code = text[3]
        self.imports: dict[int, str] = {}
        for entry in getattr(pe, "DIRECTORY_ENTRY_IMPORT", []):
            dll = entry.dll.decode().removesuffix(".dll").removesuffix(".DLL")
            for imp in entry.imports:
                name = imp.name.decode() if imp.name else f"#{imp.ordinal}"
                self.imports[imp.address] = f"{dll}!{name}"

    def section(self, name: str) -> tuple[str, int, int, bytes]:
        for s in self.sections:
            if s[0] == name:
                return s
        die(f"no section {name}")
        raise AssertionError

    def section_of(self, va: int) -> str | None:
        for name, start, size, _ in self.sections:
            if start <= va < start + size:
                return name
        return None

    def is_code(self, va: int) -> bool:
        return self.text_start <= va < self.text_end

    def read(self, va: int, n: int) -> bytes | None:
        for _, start, size, raw in self.sections:
            if start <= va and va + n <= start + size:
                chunk = raw[va - start : va - start + n]
                return chunk + bytes(n - len(chunk))  # uninitialised data reads as zero
        return None

    def unpack(self, fmt: str, va: int):
        raw = self.read(va, struct.calcsize(fmt))
        return None if raw is None else struct.unpack(fmt, raw)[0]

    def cstring(self, va: int, limit: int = 72) -> str | None:
        raw = self.read(va, limit + 1) or self.read(va, 8)
        if not raw:
            return None
        end = raw.find(b"\0")
        text = raw if end < 0 else raw[:end]
        if len(text) < 3 or not all(32 <= b < 127 or b in (9, 10, 13) for b in text):
            return None
        shown = text[:limit].decode("ascii").encode("unicode_escape").decode()
        return f'"{shown}"' + ("..." if end < 0 or end > limit else "")


# --------------------------------------------------------------------------------------------
# The code: instructions, jump tables, function starts, references


class Code:
    """A linear sweep of `.text` that decodes switch tables as data, plus derived indexes."""

    def __init__(self, img: Image):
        try:
            import capstone
        except ImportError:
            die("needs `pip install capstone pefile`")
        self.img = img
        md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
        md.skipdata = True
        first = list(md.disasm_lite(img.code, img.text_start))
        self.tables = self._find_tables(first)  # va -> (kind "dd"/"db", count)
        # Second sweep: the code between the tables.
        self.ins: list[tuple[int, int, str, str]] = []
        cursor = img.text_start
        for va in sorted(self.tables):
            kind, count = self.tables[va]
            if va < cursor:
                continue
            self._sweep(md, cursor, va)
            if kind == "dd":
                for k in range(count):
                    at = va + 4 * k
                    self.ins.append((at, 4, "dd", f"0x{img.unpack('<I', at):x}"))
                cursor = va + 4 * count
            else:
                for k in range(0, count, 16):
                    chunk = img.read(va + k, min(16, count - k)) or b""
                    self.ins.append((va + k, len(chunk), "db", ",".join(map(str, chunk))))
                cursor = va + count
        self._sweep(md, cursor, img.text_end)
        self.addrs = [i[0] for i in self.ins]
        self.refs: dict[int, list[int]] = defaultdict(list)  # target -> sites
        self.calls: dict[int, list[int]] = defaultdict(list)  # callee -> call sites
        starts = {img.entry}
        jumps: set[int] = set()
        for va, _, mnem, op in self.ins:
            if mnem in ("db", "dd", ".byte"):
                if mnem == "dd":
                    self.refs[int(op, 16)].append(va)
                continue
            for h in HEX_RE.findall(op):
                target = int(h, 16)
                if img.text_start <= target < img.end:
                    self.refs[target].append(va)
            if mnem == "call" and op.startswith("0x"):
                target = int(op, 16)
                if img.is_code(target):
                    starts.add(target)
                    self.calls[target].append(va)
            elif mnem == "jmp" and op.startswith("0x"):
                jumps.add(int(op, 16))
        # Code addresses taken as values (vtables, callbacks) or reached by a tail jump count as
        # starts when they look like one: 16-aligned after padding or a return.
        case_labels = {
            int(op, 16) for _, _, m, op in self.ins if m == "dd"
        }
        candidates = set(jumps)
        for name, start, size, raw in img.sections:
            if name in (".rdata", ".data"):
                for off in range(0, len(raw) - 3, 4):
                    candidates.add(struct.unpack_from("<I", raw, off)[0])
        for va, _, mnem, op in self.ins:
            if mnem in ("push", "mov") and op.split(", ")[-1].startswith("0x"):
                candidates.add(int(op.split(", ")[-1], 16))
        for c in candidates:
            if img.is_code(c) and c not in case_labels and self._looks_like_start(c):
                starts.add(c)
        self.detected = starts
        self.starts = sorted(starts)

    @staticmethod
    def _cmp_count(window: list[tuple[int, int, str, str]]) -> int | None:
        for _, _, mnem, op in reversed(window):
            if mnem == "cmp":
                m = CMP_RE.search(op)
                if m:
                    return int(m.group(2), 0) + 1
        return None

    def _find_tables(self, ins: list[tuple[int, int, str, str]]) -> dict[int, tuple[str, int]]:
        img = self.img
        tables: dict[int, tuple[str, int]] = {}
        for i, (va, _, mnem, op) in enumerate(ins):
            if mnem != "jmp":
                continue
            m = JMP_TABLE_RE.search(op)
            if not m:
                continue
            reg, table = m.group(1), int(m.group(2), 16)
            if not img.is_code(table):
                continue
            window = ins[max(0, i - 12) : i]
            count = None
            for j in range(len(window) - 1, -1, -1):
                bm = BYTE_TABLE_RE.search(window[j][3])
                dest = bm and ("e" + bm.group(1)[0] + "x" if bm.group(1).endswith("l") else bm.group(1))
                if window[j][2] in ("movzx", "mov") and bm and dest == reg:
                    btable = int(bm.group(3), 16)
                    n = self._cmp_count(window[:j])
                    if n and img.is_code(btable) and n <= 1024:
                        values = img.read(btable, n) or b""
                        tables[btable] = ("db", n)
                        count = max(values) + 1 if values else None
                    break
            if count is None:
                count = self._cmp_count(window)
            if count is None or count > 1024:
                count = 0
                while count < 256:
                    v = img.unpack("<I", table + 4 * count)
                    if v is None or not img.is_code(v) or abs(v - va) > 0x10000:
                        break
                    count += 1
            if count:
                tables[table] = ("dd", count)
        return tables

    def _sweep(self, md, start: int, end: int) -> None:
        if end <= start:
            return
        code = self.img.code[start - self.img.text_start : end - self.img.text_start]
        self.ins.extend(md.disasm_lite(code, start))

    def _looks_like_start(self, va: int) -> bool:
        if va % 16:
            return False
        i = bisect.bisect_left(self.addrs, va)
        if i >= len(self.addrs) or self.addrs[i] != va:
            return False
        prev = self.img.code[va - self.img.text_start - 1]
        ret_n = self.img.code[va - self.img.text_start - 3] == 0xC2
        return prev in PAD or prev == 0xC3 or ret_n

    def add_starts(self, extra) -> None:
        self.starts = sorted(set(self.starts) | set(extra))

    def index(self, va: int) -> int:
        """Index of the instruction covering va (the last one starting at or before it)."""
        return max(0, bisect.bisect_right(self.addrs, va) - 1)

    def is_instruction(self, va: int) -> bool:
        i = bisect.bisect_left(self.addrs, va)
        return i < len(self.addrs) and self.addrs[i] == va and self.ins[i][2] not in ("db", "dd")

    def function_start(self, va: int) -> int | None:
        i = bisect.bisect_right(self.starts, va) - 1
        return self.starts[i] if i >= 0 and self.img.is_code(va) else None

    def function_end(self, start: int) -> int:
        """End of the function at `start`: the next start, less trailing padding."""
        i = bisect.bisect_right(self.starts, start)
        end = self.starts[i] if i < len(self.starts) else self.img.text_end
        while end > start + 1 and self.img.code[end - 1 - self.img.text_start] in PAD:
            end -= 1
        return end

    def instructions(self, start: int, end: int):
        i = bisect.bisect_left(self.addrs, start)
        while i < len(self.ins) and self.ins[i][0] < end:
            yield self.ins[i]
            i += 1


# --------------------------------------------------------------------------------------------
# The map


def load_toml(path: Path) -> dict:
    if not path.is_file():
        return {}
    with path.open("rb") as f:
        return tomllib.load(f)


class Names:
    """The map (ours), the structs and the H3API guesses, as lookups."""

    def __init__(self):
        self.entries: list[dict] = load_toml(MAP).get("entry", [])
        self.structs: list[dict] = load_toml(STRUCTS).get("struct", [])
        h3 = load_toml(H3API)
        self.h3api: dict[int, dict] = {f["va"]: f for f in h3.get("function", [])}
        self.h3api_data: dict[int, dict] = {d["va"]: d for d in h3.get("data", [])}
        self.h3api_structs: list[dict] = h3.get("struct", [])
        self.by_va = {e["va"]: e for e in self.entries}
        self.by_name = {e["name"]: e for e in self.entries}
        self.h3api_by_name: dict[str, dict] = {}
        for f in self.h3api.values():
            self.h3api_by_name.setdefault(f["name"], f)
            self.h3api_by_name.setdefault(f["name"].split("::")[-1], f)
        self.ranges = sorted(
            (e["va"], e["va"] + max(1, e.get("size", 4 if e["kind"] == "global" else 1)), e)
            for e in self.entries
            if e["kind"] != "site"
        )

    def resolve(self, text: str) -> int:
        va = parse_va(text)
        if va is not None:
            return va
        if text in self.by_name:
            return self.by_name[text]["va"]
        if text in self.h3api_by_name:
            return self.h3api_by_name[text]["va"]
        near = [e["name"] for e in self.entries if text.lower() in e["name"].lower()]
        near += [f["name"] for f in self.h3api.values() if text.lower() in f["name"].lower()]
        hint = f"; did you mean: {', '.join(near[:8])}" if near else ""
        die(f"unknown address or name `{text}`{hint}")
        raise AssertionError

    def covering(self, va: int) -> dict | None:
        """The innermost map entry (not a site) whose range covers va."""
        best = None
        i = bisect.bisect_right(self.ranges, (va, float("inf"))) - 1
        while i >= 0:
            start, end, e = self.ranges[i]
            if start <= va < end and (best is None or start > best["va"]):
                best = e
            if va - start > 0x40000:
                break
            i -= 1
        return best


class Annotator:
    def __init__(self, img: Image, code: Code, names: Names):
        self.img, self.code, self.names = img, code, names
        sizes = {}
        for e in names.entries:
            if e["kind"] == "function":
                sizes[e["va"]] = e
        code.add_starts(sizes)

    def name_of(self, va: int, exact_only: bool = False) -> str | None:
        n = self.names
        if va in n.by_va:
            e = n.by_va[va]
            return e["name"] if e["confidence"] == "read" else f"{e['name']} ({e['confidence']})"
        if va in n.h3api:
            return f"h3api:{n.h3api[va]['name']}?"
        if va in n.h3api_data:
            return f"h3api:{n.h3api_data[va]['name']}?"
        if va in self.img.imports:
            return self.img.imports[va]
        if exact_only:
            return None
        if self.img.is_code(va):
            start = self.code.function_start(va)
            if start is None:
                return None
            base = self.name_of(start, exact_only=True) or f"sub_{start:06X}"
            return base if start == va else f"{base}+0x{va - start:X}"
        e = n.covering(va)
        if e is not None:
            return e["name"] if e["va"] == va else f"{e['name']}+0x{va - e['va']:X}"
        return None

    def notes(self, ins: tuple[int, int, str, str]) -> list[str]:
        va, _, mnem, op = ins
        out = []
        if mnem in ("db", "dd"):
            if mnem == "dd":
                out.append(self.name_of(int(op, 16)) or "")
            return [o for o in out if o]
        for h in HEX_RE.findall(op):
            target = int(h, 16)
            if not (self.img.text_start <= target < self.img.end):
                continue
            if mnem.startswith("j") and op == h and self.img.is_code(target):
                if self.code.function_start(target) == self.code.function_start(va):
                    continue
            label = self.name_of(target)
            value = None
            if not self.img.is_code(target):
                if FLOAT_MNEMS.match(mnem) and ("ptr [" + h) in op:
                    if "qword ptr" in op:
                        value = f"= {self.img.unpack('<d', target)!r} (f64)"
                    elif "dword ptr" in op:
                        value = f"= {self.img.unpack('<f', target)!r} (f32)"
                    elif "tbyte ptr" in op:
                        value = "(f80)"
                elif op.endswith(h) or "push" == mnem:
                    value = self.img.cstring(target)
            text = " ".join(t for t in (label, value) if t)
            if text:
                out.append(text)
        return out

    def line(self, ins: tuple[int, int, str, str]) -> str:
        va, _, mnem, op = ins
        text = f"{hx(va)}  {mnem} {op}".rstrip()
        notes = self.notes(ins)
        site = self.names.by_va.get(va)
        prefix = ""
        if site is not None and site["kind"] == "site":
            prefix = f"  ;; site {site['name']}: {site['summary']}\n"
        if notes:
            text = f"{text:<52} ; {' | '.join(notes)}"
        return prefix + text

    def header(self, start: int, end: int) -> list[str]:
        lines = [f"; ---- {hx(start)}..{hx(end)} ({end - start} bytes)"]
        e = self.names.by_va.get(start)
        if e is not None:
            lines.append(f"; {e['name']} [{e['kind']}, {e['confidence']}]: {e['summary']}")
            for key in ("args", "returns", "refs", "h3api"):
                if key in e:
                    value = ", ".join(e[key]) if isinstance(e[key], list) else e[key]
                    lines.append(f";   {key}: {value}")
        h = self.names.h3api.get(start)
        if h is not None:
            lines.append(
                f"; h3api (guess): {h['name']}({h.get('params', '')}) {h.get('conv', '')} "
                f"-> {h.get('ret', '')} [{h.get('evidence', '')}]"
            )
        callers = self.code.calls.get(start, [])
        lines.append(f"; called from {len(callers)} site(s)")
        return lines


def setup():
    img = Image(EXE)
    code = Code(img)
    names = Names()
    return img, code, names, Annotator(img, code, names)


# --------------------------------------------------------------------------------------------
# Commands


def cmd_fn(args) -> None:
    img, code, names, ann = setup()
    va = names.resolve(args.target)
    start = code.function_start(va)
    if start is None:
        die(f"{hx(va)} is not in .text (try `lookup` or `data`)")
    end = code.function_end(start)
    print("\n".join(ann.header(start, end)))
    for ins in code.instructions(start, end):
        print(ann.line(ins))


def cmd_callers(args) -> None:
    img, code, names, ann = setup()
    va = names.resolve(args.target)
    print(f"; callers of {hx(va)} {ann.name_of(va) or ''}")
    groups: dict[int, list[int]] = defaultdict(list)
    for site in code.calls.get(va, []):
        groups[code.function_start(site) or 0].append(site)
    for start in sorted(groups):
        sites = ", ".join(hx(s) for s in groups[start])
        print(f"{hx(start)} {ann.name_of(start) or ''}: {sites}")
    other = [s for s in code.refs.get(va, []) if s not in code.calls.get(va, [])]
    if other:
        print(f"; also referenced (jump, value or table) at: {', '.join(hx(s) for s in other)}")


def cmd_callees(args) -> None:
    img, code, names, ann = setup()
    va = names.resolve(args.target)
    start = code.function_start(va)
    if start is None:
        die(f"{hx(va)} is not in .text")
    seen = {}
    for ins in code.instructions(start, code.function_end(start)):
        if ins[2] == "call":
            target = ins[3]
            seen.setdefault(target, ins[0])
    print(f"; calls made by {hx(start)} {ann.name_of(start) or ''}")
    for target, site in seen.items():
        t = int(target, 16) if target.startswith("0x") else None
        label = ann.name_of(t) if t is not None else " | ".join(ann.notes((site, 0, "call", target)))
        print(f"{hx(site)}  call {target:<28} ; {label or ''}")


def cmd_refs(args) -> None:
    img, code, names, ann = setup()
    va = names.resolve(args.target)
    e = names.by_va.get(va)
    end = va + (e.get("size", 1) if e is not None and e["kind"] != "site" else 1)
    print(f"; references to {hx(va)}..{hx(end)} {ann.name_of(va) or ''}")
    for target in sorted(t for t in code.refs if va <= t < end):
        for site in code.refs[target]:
            i = code.index(site)
            fn = code.function_start(site)
            where = ann.name_of(fn) if fn is not None else ""
            print(f"{ann.line(code.ins[i])}   [in {where}]")


def cmd_grep(args) -> None:
    img, code, names, ann = setup()
    pattern = re.compile(args.regex)
    if args.fn:
        start = code.function_start(names.resolve(args.fn))
        if start is None:
            die("--fn is not in .text")
        pool = code.instructions(start, code.function_end(start))
    else:
        pool = iter(code.ins)
    shown = 0
    for ins in pool:
        if pattern.search(f"{ins[2]} {ins[3]}"):
            fn = code.function_start(ins[0])
            print(f"{ann.line(ins)}   [in {ann.name_of(fn) if fn else '?'}]")
            shown += 1
            if shown >= args.limit:
                print(f"; stopped at --limit {args.limit}")
                break


def cmd_lookup(args) -> None:
    names = Names()
    va = parse_va(args.query)
    if va is None and args.query not in names.by_name and args.query not in names.h3api_by_name:
        q = args.query.lower()
        for e in names.entries:
            if q in e["name"].lower() or q in e["summary"].lower():
                print(f"{hx(e['va'])} {e['name']} [{e['kind']}, {e['confidence']}]: {e['summary']}")
        for f in [*names.h3api.values(), *names.h3api_data.values()]:
            if q in f["name"].lower():
                print(f"{hx(f['va'])} h3api:{f['name']} (guess) [{f.get('evidence', '')}]")
        for s in names.structs + names.h3api_structs:
            for fld in s.get("fields", []):
                if q in fld["name"].lower() or q in fld.get("summary", "").lower():
                    print(f"{s['name']}+0x{fld['offset']:X} {fld['name']}: {fld.get('summary', '')}")
        return
    img, code, names, ann = setup()
    va = names.resolve(args.query)
    section = img.section_of(va) or "outside the image"
    print(f"{hx(va)} in {section}: {ann.name_of(va) or 'unnamed'}")
    e = names.by_va.get(va) or names.covering(va)
    if e is not None:
        for key, value in e.items():
            print(f"  {key} = {hx(value) if key in ('va', 'size') else value}")
    if img.is_code(va):
        start = code.function_start(va)
        if start is not None:
            end = code.function_end(start)
            print(f"  function {hx(start)}..{hx(end)} {ann.name_of(start)}, "
                  f"{len(code.calls.get(start, []))} caller site(s)")
    for guess in (names.h3api.get(va), names.h3api_data.get(va)):
        if guess is not None:
            print(f"  h3api (guess): {guess}")
    print(f"  referenced at {len(code.refs.get(va, []))} site(s)")


DATA_TYPES = {
    "u8": "<B", "i8": "<b", "u16": "<H", "i16": "<h", "u32": "<I", "i32": "<i",
    "f32": "<f", "f64": "<d", "ptr": "<I",
}


def cmd_data(args) -> None:
    img, code, names, ann = setup()
    va = names.resolve(args.target)
    if args.type == "str":
        print(f"{hx(va)} {img.cstring(va, 200)}")
        return
    fmt = DATA_TYPES[args.type]
    width = struct.calcsize(fmt)
    stride = args.stride or width
    values = []
    for k in range(args.count):
        v = img.unpack(fmt, va + k * stride)
        if v is None:
            break
        if args.type == "ptr":
            label = ann.name_of(v) or img.cstring(v) or ""
            values.append(f"{hx(va + k * stride)}: 0x{v:06X} {label}".rstrip())
        else:
            values.append(f"{hx(va + k * stride)}: {v!r}")
    print("\n".join(values))


def cmd_listing(args) -> None:
    img, code, names, ann = setup()
    out = Path(args.out) if args.out else OUT / "heroes3.asm"
    out.parent.mkdir(parents=True, exist_ok=True)
    starts = set(code.starts)
    with out.open("w") as f:
        f.write("; Heroes3.exe annotated by scripts/exe.py. Never commit this file (Q29).\n")
        for ins in code.ins:
            if ins[0] in starts:
                f.write("\n" + "\n".join(ann.header(ins[0], code.function_end(ins[0]))) + "\n")
            f.write(ann.line(ins) + "\n")
    print(f"wrote {out} ({len(code.ins)} lines, {len(starts)} functions)")


def cited_addresses() -> dict[int, list[str]]:
    """Image addresses cited under docs/ and crates/ (as `cargo xtask check-exe-map` scans)."""
    image = load_toml(MAP).get("image", {})
    low, high = image.get("text_start", 0x401000), image.get("end", 0x6B6000)
    ignore = set(image.get("ignore", []))
    found: dict[int, list[str]] = defaultdict(list)
    for top in ("docs", "crates"):
        for path in sorted((ROOT / top).rglob("*")):
            name = path.relative_to(ROOT).as_posix()
            if (path.suffix not in (".md", ".toml", ".rs") or name.startswith("docs/exe/")
                    or "/target/" in name or not path.is_file()):
                continue
            for m in CITE_RE.finditer(path.read_text(encoding="utf-8", errors="replace")):
                va = int(m.group(1), 16)
                if low <= va < high and va not in ignore and name not in found[va]:
                    found[va].append(name)
    return found


def cmd_cited(args) -> None:
    img, code, names, ann = setup()
    cited = cited_addresses()
    missing = [va for va in sorted(cited) if names.covering(va) is None and va not in names.by_va]
    by_fn: dict[int, list[int]] = defaultdict(list)
    for va in missing:
        by_fn[code.function_start(va) or va].append(va)
    print(f"; {len(cited)} cited addresses, {len(missing)} not covered by the map "
          f"(in {len(by_fn)} functions or data items)")
    for start in sorted(by_fn):
        where = "code" if img.is_code(start) else img.section_of(start)
        end = code.function_end(start) if img.is_code(start) else start
        sites = ", ".join(hx(v) for v in by_fn[start])
        files = sorted({f for v in by_fn[start] for f in cited[v]})
        print(f"{hx(start)} [{where}, size 0x{end - start:X}] {ann.name_of(start) or ''}: "
              f"{sites}  ({', '.join(files)})")


def cmd_check(args) -> None:
    img, code, names, ann = setup()
    problems, fixes = [], {}
    for e in names.entries:
        va, kind = e["va"], e["kind"]
        if kind in ("function", "site") and not code.is_instruction(va):
            problems.append(f"{hx(va)} {e['name']}: not an instruction start")
        if kind == "function":
            if va not in code.detected:
                problems.append(f"{hx(va)} {e['name']}: no call, pointer or jump reaches it "
                                f"(fine if read so; it is a start because the map says so)")
            size = code.function_end(va) - va
            if e.get("size") != size:
                fixes[va] = size
        if kind in ("table", "global") and img.section_of(va) is None:
            problems.append(f"{hx(va)} {e['name']}: outside the image")
    for va, size in sorted(fixes.items()):
        e = names.by_va[va]
        problems.append(f"{hx(va)} {e['name']}: size {e.get('size')} should be 0x{size:X}")
    if args.fix_sizes and fixes:
        fix_sizes(fixes)
        print(f"fixed {len(fixes)} size(s) in {MAP.relative_to(ROOT)}")
    print("\n".join(problems) if problems else "check: the map agrees with the executable")


def fix_sizes(fixes: dict[int, int]) -> None:
    """Set `size = ...` in the one-line entries at the given addresses, keeping the rest."""
    out = []
    for line in MAP.read_text().split("\n"):
        m = re.match(r"\{ va = (0x[0-9A-Fa-f]+),", line)
        va = int(m.group(1), 16) if m else None
        if va in fixes:
            size = f"size = 0x{fixes[va]:X}"
            if re.search(r"\bsize = 0x[0-9A-Fa-f]+", line):
                line = re.sub(r"\bsize = 0x[0-9A-Fa-f]+", size, line)
            else:
                line = re.sub(r'(name = "[^"]*", )', rf"\1{size}, ", line, count=1)
        out.append(line)
    MAP.write_text("\n".join(out))


def cmd_stats(args) -> None:
    img, code, names, ann = setup()
    funcs = code.starts
    ours = [e for e in names.entries if e["kind"] == "function"]
    by_conf = defaultdict(int)
    for e in names.entries:
        by_conf[(e["kind"], e["confidence"])] += 1
    h3 = [va for va in names.h3api if va in set(funcs)]
    print(f"instructions {len(code.ins)}, switch tables {len(code.tables)}, functions {len(funcs)}")
    print(f"map entries {len(names.entries)}: " +
          ", ".join(f"{k} {c}: {n}" for (k, c), n in sorted(by_conf.items())))
    print(f"functions named by the map {len(ours)}, by H3API only "
          f"{len(set(h3) - {e['va'] for e in ours})}, unnamed "
          f"{len(set(funcs) - set(h3) - {e['va'] for e in ours})}")
    print(f"structs {len(names.structs)} (ours), {len(names.h3api_structs)} (H3API)")


def cmd_port_h3api(args) -> None:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import exe_h3api  # noqa: E402

    img, code, names, ann = setup()
    exe_h3api.port(Path(args.dir), img, code, H3API)


def main() -> None:
    if hasattr(signal, "SIGPIPE"):
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)  # quiet when piped into `head`
    p = argparse.ArgumentParser(prog="scripts/exe.py", description=__doc__.split("\n\n")[0])
    sub = p.add_subparsers(dest="command", required=True)
    for name, fn in (("fn", cmd_fn), ("callers", cmd_callers), ("callees", cmd_callees),
                     ("refs", cmd_refs)):
        s = sub.add_parser(name)
        s.add_argument("target")
        s.set_defaults(run=fn)
    s = sub.add_parser("grep")
    s.add_argument("regex")
    s.add_argument("--fn")
    s.add_argument("--limit", type=int, default=200)
    s.set_defaults(run=cmd_grep)
    s = sub.add_parser("lookup")
    s.add_argument("query")
    s.set_defaults(run=cmd_lookup)
    s = sub.add_parser("data")
    s.add_argument("target")
    s.add_argument("--type", default="i32", choices=[*DATA_TYPES, "str"])
    s.add_argument("--count", type=int, default=16)
    s.add_argument("--stride", type=int, default=0)
    s.set_defaults(run=cmd_data)
    s = sub.add_parser("listing")
    s.add_argument("--out")
    s.set_defaults(run=cmd_listing)
    for name, fn in (("cited", cmd_cited), ("stats", cmd_stats)):
        sub.add_parser(name).set_defaults(run=fn)
    s = sub.add_parser("check")
    s.add_argument("--fix-sizes", action="store_true")
    s.set_defaults(run=cmd_check)
    s = sub.add_parser("port-h3api")
    s.add_argument("dir")
    s.set_defaults(run=cmd_port_h3api)
    args = p.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
