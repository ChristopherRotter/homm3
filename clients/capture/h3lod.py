#!/usr/bin/env python3
"""The original's text tables for the capture harness (E17 H1, *provisional*): a LOD archive's
directory and entries (docs/formats/installation.md §2), its `.TXT` tables parsed as h3-formats'
`TextTable` does (§4: records of TAB-separated fields, a field that starts with a quote runs to the
next quote before TAB, CR, LF or the end), and a table's records as patterns of the `printf`
texts the game formats from them. Read from the run directory's copy at run time, never kept:
the harness names a record by its number, never by its text (ADR-0009).

  h3lod.py find <table> <text>   the records of a table (ADVEVENT.TXT) a text was formatted from
  h3lod.py label <text>          the text as a kept file holds it (`label`)

Library use: `table(name)` (records of fields, header rows included, from 0), `matches(name,
text)` (the records whose first field formats to `text`, or two neighbours' together: "89+90"),
`identify(text)` (the most specific match in ADVEVENT.TXT, GENRLTXT.TXT or, since E08 T68b,
RANDSIGN.TXT, the signs' and Ocean Bottles' default texts, since E08 T79a MINEEVNT.TXT, the mines'
flag messages: "ADVEVENT:86", "RANDSIGN:24", "MINEEVNT:0"); since E24 M4's labels task
`label(text, own)`, a text as the records it was made of for a file kept in git ("[ADVEVENT:86]",
"+2 [PRISKILL:0]", "[other: 3 characters]").
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


def building_names(faction):
    """E09 T14b (batch TWN-2): a faction's (0 Castle to 8 Conflux) building names by building id,
    as the town screen and the hall show them: 0 to 14 and 16 from BLDGNEUT.TXT (a record an id),
    15 and 17 to 26 from BLDGSPEC.TXT (11 records a faction: 17 to 26, then the Resource Silo),
    30 to 43 from DWELLING.TXT (14 records a faction: the seven dwellings, then their upgrades);
    an empty name left out. For naming ids in a record, never written out."""
    neutral, special, dwelling = (table(n) for n in ("BLDGNEUT.TXT", "BLDGSPEC.TXT",
                                                     "DWELLING.TXT"))
    names = {b: neutral[b][0] for b in (*range(15), 16)}
    names.update({17 + j: special[11 * faction + j][0] for j in range(10)})
    names[15] = special[11 * faction + 10][0]
    names.update({30 + k: dwelling[14 * faction + k][0] for k in range(14)})
    return {b: n.strip() for b, n in sorted(names.items()) if n.strip()}


# E24 M4, the labels task (ADR-0009): a text the harness keeps in git (a live part's boxes, their
# captions, the level-up's and a find's texts) written as the records it was made of, never as
# the original's text. `label` gives `[ADVEVENT:86]` for a text one record (or two neighbours',
# "89+90") formats, its conversions each a number or names; else the text piece by piece: a name
# of NAME_FIELDS (`[CRTRAITS:15.1]`, the row's field 1: a plural) or a record of FORMAT_TABLES
# without conversions (`[GENRLTXT:5]`), numbers and punctuation kept, any other piece `[other: n
# characters]`; `[the map's text]` for a text the situation gives itself. LABELLED is what a kept
# text may hold (the client test `h3-map-view`'s `tests/texts.rs` checks the kept files by the
# same rule). KEPT is that test's own set, character for character (E24 M4 T6, the reviewer's
# R6-11: `\s` and `\d` took any Unicode space and digit, a no-break space kept here and refused
# there).
FORMAT_TABLES = ("ADVEVENT.TXT", "GENRLTXT.TXT", "ARRAYTXT.TXT", "ARTEVENT.TXT", "RANDSIGN.TXT",
                 "MINEEVNT.TXT")
NAME_FIELDS = {"ARTRAITS.TXT": (0,), "CRTRAITS.TXT": (0, 1), "SPTRAITS.TXT": (0,),
               "SSTRAITS.TXT": (0,), "SKILLLEV.TXT": (0,), "PRISKILL.TXT": (0,),
               "HOTRAITS.TXT": (0,), "HCTRAITS.TXT": (0,), "OBJNAMES.TXT": (0,),
               "CRGEN1.TXT": (0,), "CRGEN4.TXT": (0,), "CRBANKS.TXT": (0,), "RESTYPES.TXT": (0,),
               "PLCOLORS.TXT": (0,), "TOWNNAME.TXT": (0,), "TERRNAME.TXT": (0,),
               "MINENAME.TXT": (0,)}
KEPT = r"[ \t\r\n0-9+\-,.:;()%/'\"!?]"
LABELLED = re.compile(rf"(?:\[(?:[A-Z][A-Z0-9]*:\d+(?:\.\d+)?(?:\+\d+)?|other: \d+ characters"
                      rf"|the map's text)\]|{KEPT})*", re.ASCII)  # `\d` the test's ASCII digits
CONVERSION = r"%[-+ #0]*\d*(?:\.\d+)?[a-zA-Z]"


def groups_pattern(text):
    """`pattern`'s expression with each conversion a group of its own."""
    parts = re.split(rf"(%%|{CONVERSION}|\s+)", text.strip())
    out = []
    for part in parts:
        if part == "%%":
            out.append("%")
        elif part.startswith("%") and len(part) > 1:
            out.append("(.*?)")
        elif part and part.isspace():
            out.append(r"\s+")
        elif part:
            out.append(re.escape(part).replace('""', '"{1,2}').replace(r"\{", r"\{?")
                       .replace(r"\}", r"\}?"))
    return re.compile("".join(out), re.S)


@lru_cache(maxsize=None)
def formats():
    """Every record of FORMAT_TABLES with a conversion (and each two neighbours' together, as
    `patterns`): (label, literal length, expression with groups)."""
    out = []
    for name in FORMAT_TABLES:
        rows = table(name)
        texts = [(str(n), r[0]) for n, r in enumerate(rows) if r and r[0].strip()]
        texts += [(f"{n}+{n + 1}", rows[n][0] + rows[n + 1][0]) for n in range(len(rows) - 1)
                  if rows[n] and rows[n + 1] and rows[n][0].strip() and rows[n + 1][0].strip()]
        out += [(f"{name.split('.')[0]}:{n}", literal(name, n), groups_pattern(t))
                for n, t in texts if re.search(CONVERSION, t.replace("%%", ""))]
    return out


def piece_pattern(text, fold=False):
    """A piece's expression: its words with any white space between (a doubled quote and a
    brace as `pattern`'s), whole words only; with `fold` in any case."""
    words = text.split()
    body = r"\s+".join(re.escape(w).replace('""', '"{1,2}').replace(r"\{", r"\{?")
                       .replace(r"\}", r"\}?") for w in words)
    head = r"(?<![A-Za-z0-9])" if words[0][0].isalnum() else ""
    tail = r"(?![A-Za-z0-9])" if words[-1][-1].isalnum() else ""
    return re.compile(head + body + tail, re.I if fold else 0)


@lru_cache(maxsize=None)
def pieces(fold=False):
    """The names of NAME_FIELDS and the records of FORMAT_TABLES without a conversion, longest
    first: (label, expression; with `fold` in any case)."""
    out = []
    for name, fields in NAME_FIELDS.items():
        for n, record in enumerate(table(name)):
            for f in fields:
                if f < len(record) and record[f].strip():
                    out.append((f"{name.split('.')[0]}:{n}" + (f".{f}" if f else ""),
                                record[f].strip()))
    for name in FORMAT_TABLES:
        for n, record in enumerate(table(name)):
            if record and record[0].strip() and not re.search(CONVERSION,
                                                              record[0].replace("%%", "")):
                out.append((f"{name.split('.')[0]}:{n}", record[0].strip()))
    out.sort(key=lambda p: -len(p[1]))
    return [(lab, piece_pattern(text, fold)) for lab, text in out]


def by_pieces(text, fold=False):
    """`text` piece by piece: the leftmost, then longest, piece of `pieces` (with `fold` in any
    case) a label each, the rest kept where it is numbers and punctuation (KEPT), else `[other: n
    characters]`."""
    out, rest = [], text
    while rest:
        hits = [(m.start(), -(m.end() - m.start()), lab, m) for lab, p in pieces(fold)
                if (m := p.search(rest))]
        if not hits:
            break
        _, _, lab, m = min(hits, key=lambda h: (h[0], h[1]))
        out += [rest[:m.start()], f"[{lab}]"]
        rest = rest[m.end():]
    out.append(rest)
    # A piece of white space alone that KEPT does not hold (a no-break space) counts whole.
    return "".join(part if part.startswith("[") or re.fullmatch(f"{KEPT}*", part)
                   else f"[other: {len(part.strip()) or len(part)} characters]" for part in out)


def label(text, own=()):
    """A kept text by the records it was made of (above): "" for none; `text` itself when it is
    numbers and punctuation, or labels already (LABELLED: a kept file labelled again stays as it
    is); `[the map's text]` when it is one of `own` (the situation's own
    texts, white space aside); `[TABLE:record]` for the most literal record of FORMAT_TABLES
    that formats it whose conversions hold only numbers and names (`by_pieces` of them together,
    in any case: the game writes a resource's name in lower case, ADVEVENT 113, with no other
    piece); else `by_pieces`."""
    if LABELLED.fullmatch(text):
        return text
    plain = " ".join(text.split())
    if any(plain == " ".join(o.split()) for o in own):
        return "[the map's text]"
    found = []
    for lab, length, p in formats():
        m = p.fullmatch(text.strip())
        if m and "[other:" not in by_pieces(" ".join(m.groups()), fold=True):
            found.append((length, -len(found), lab))
    if found:
        return f"[{max(found)[2]}]"
    return by_pieces(text)


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "find":
        print(matches(sys.argv[2], sys.argv[3]))
    elif len(sys.argv) == 3 and sys.argv[1] == "label":
        print(label(sys.argv[2]))
    else:
        sys.exit(__doc__)
