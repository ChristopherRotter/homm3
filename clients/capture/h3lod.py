#!/usr/bin/env python3
"""The original's text tables for the capture harness (E17 H1, *provisional*): a LOD archive's
directory and entries (docs/formats/installation.md §2), its `.TXT` tables parsed as h3-formats'
`TextTable` does (§4: records of TAB-separated fields, a field that starts with a quote runs to the
next quote before TAB, CR, LF or the end), and a table's records as patterns of the `printf`
texts the game formats from them. Read from the run directory's copy at run time, never kept:
the harness names a record by its number, never by its text (ADR-0009).

  h3lod.py find <table> <text>   the records of a table (ADVEVENT.TXT) a text was formatted from

Library use: `table(name)` (records of fields, header rows included, from 0), `matches(name,
text)` (the records whose first field formats to `text`, or two neighbours' together: "89+90"),
`identify(text)` (the most specific match in ADVEVENT.TXT, GENRLTXT.TXT or, since E08 T68b,
RANDSIGN.TXT, the signs' and Ocean Bottles' default texts, since E08 T79a MINEEVNT.TXT, the mines'
flag messages: "ADVEVENT:86", "RANDSIGN:24", "MINEEVNT:0").
"""

import re
import struct
import sys
import zlib
from functools import lru_cache
from pathlib import Path

DATA = Path(__file__).resolve().parent.parent.parent / "original-data" / "wine" / "game" / "Data"
# The archives in the order the game searches them for a bitmap's name (its row 3: the base
# archive first, the AB one for the names it lacks; design §3.1, 0x559320). Until E17 M3 T6c-2 the
# harness took the AB copies first, whose GENRLTXT differs from record 79 on and Help.txt from 96.
ARCHIVES = ("H3bitmap.lod", "H3ab_bmp.lod")


@lru_cache(maxsize=None)
def directory(archive):
    """{lower-case name: (offset, size, compressed size)} of a LOD archive."""
    data = (DATA / archive).read_bytes()
    if data[:4] != b"LOD\0":
        raise ValueError(f"{archive} is not a LOD archive")
    count = struct.unpack_from("<I", data, 8)[0]
    entries = {}
    for i in range(count):
        name, offset, size, _kind, packed = struct.unpack_from("<16sIIII", data, 0x5C + 32 * i)
        entries[name.split(b"\0")[0].decode("latin-1").lower()] = (offset, size, packed)
    return entries


def entry(name):
    """An entry's bytes, from the first archive that holds it."""
    for archive in ARCHIVES:
        found = directory(archive).get(name.lower())
        if found:
            offset, size, packed = found
            with open(DATA / archive, "rb") as f:
                f.seek(offset)
                raw = f.read(packed or size)
            return zlib.decompress(raw) if packed else raw
    raise KeyError(f"no {name} in {ARCHIVES}")


def parse(data):
    """A text table's records of fields (Windows-1252, h3-formats' TextTable rules)."""
    text = data.decode("cp1252", errors="replace")
    records, record, field, i, start = [], [], [], 0, True
    while i < len(text):
        c = text[i]
        if start and c == '"':
            j = i + 1
            while True:
                if j >= len(text):
                    raise ValueError(f"record {len(records) + 1}: unterminated quoted field")
                if text[j] == '"' and (j + 1 == len(text) or text[j + 1] in "\t\r\n"):
                    break
                field.append(text[j])
                j += 1
            i, start = j + 1, False
            continue
        start = False
        if c == "\t":
            record.append("".join(field))
            field, start = [], True
        elif c == "\r" and text[i + 1:i + 2] == "\n":
            pass
        elif c == "\n":
            record.append("".join(field))
            records.append(record)
            record, field, start = [], [], True
        else:
            field.append(c)
        i += 1
    if not start or record or field:
        record.append("".join(field))
        records.append(record)
    return records


@lru_cache(maxsize=None)
def table(name):
    return parse(entry(name))


def pattern(text):
    """A regular expression for what `printf` makes of a record's text: each conversion (`%s`,
    `%d`, ...) any text, `%%` a percent sign, white space any white space, since E08 T68b a
    doubled quote one or two, since E08 T79a a brace optional."""
    parts = re.split(r"(%%|%[-+ #0]*\d*(?:\.\d+)?[a-zA-Z]|\s+)", text.strip())
    out = []
    for part in parts:
        if part == "%%":
            out.append("%")
        elif part.startswith("%") and len(part) > 1:
            out.append(".*?")
        elif part and part.isspace():
            out.append(r"\s+")
        elif part:
            # A doubled quote (a quoted field's quote, ADVEVENT 126: the Survivor's "no room";
            # E08 T68b) shows as one or two; since E08 T79a a title's braces (MINEEVNT's
            # `{Sawmill}`) may be gone from the text a dialog holds.
            out.append(re.escape(part).replace('""', '"{1,2}').replace(r"\{", r"\{?")
                       .replace(r"\}", r"\}?"))
    return re.compile("".join(out), re.S)


@lru_cache(maxsize=None)
def patterns(name):
    """Each record's pattern, then each two neighbours' together (a message the game writes from
    two records, ADVEVENT's 89 and 90)."""
    texts = [(str(n), r[0]) for n, r in enumerate(table(name)) if r and r[0].strip()]
    rows = table(name)
    pairs = [(f"{n}+{n + 1}", rows[n][0] + rows[n + 1][0]) for n in range(len(rows) - 1)
             if rows[n] and rows[n + 1] and rows[n][0].strip() and rows[n + 1][0].strip()]
    return [(n, pattern(t)) for n, t in texts + pairs]


def matches(name, text):
    """The records (from 0; "89+90" for two together) of table `name` whose first field formats
    to `text`."""
    text = text.strip()
    return [n for n, p in patterns(name) if p.fullmatch(text)]


# Since E08 T68b RANDSIGN.TXT (batch E8b's E8-9: an Ocean Bottle without a text shows the default
# text of its column mod 25, sign_default_texts 0x696C74; plain texts, no conversions). Since E08
# T79a MINEEVNT.TXT (batch E9a: a mine's flag message, by the record's resource; the first run
# named it GENRLTXT:4, its picture's caption `%d/day`).
TABLES = ("ADVEVENT.TXT", "GENRLTXT.TXT", "RANDSIGN.TXT", "MINEEVNT.TXT")


def literal(name, record):
    """How much of a record's text (or two neighbours' together: "89+90") is literal, its
    conversions left out."""
    rows = table(name)
    text = "".join(rows[int(n)][0] for n in record.split("+"))
    return len(re.sub(r"%[-+ #0]*\d*(?:\.\d+)?[a-zA-Z]|\s+", "", text))


def identify(text):
    """`<TABLE>:<record>` of the most specific record that formats `text` or a run of its lines (a
    dialog's text items, h3mem.dialog_text: a picture's caption is an item of its own), None for
    none: the most literal text; at a tie the whole text before a part, the first table's, the
    first record. Since E08 T32b2: the first match of the whole text before, and `%s the %s`
    (GENRLTXT 16) formats every text with " the " in it."""
    lines = text.strip().split("\n")
    runs = sorted({"\n".join(lines[i:j]) for i in range(len(lines))
                   for j in range(i + 1, len(lines) + 1)}, key=len, reverse=True)
    found = [(name, n, run) for run in runs for name in TABLES for n in matches(name, run)]
    if not found:
        return None
    name, record, _ = max(found, key=lambda f: (literal(f[0], f[1]), len(f[2]), -found.index(f)))
    return f"{name.split('.')[0]}:{record}"


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "find":
        print(matches(sys.argv[2], sys.argv[3]))
    else:
        sys.exit(__doc__)
