"""Checks of h3rmg.py's phase oracle readers that need no game, no Wine and no display (E14
T1b2, T3a): a log, dumps and a memory record written as xdd.c's section 10 writes them. From
clients/capture/: `python3 -m unittest test_h3rmg`."""

import hashlib
import struct
import tempfile
import unittest
from pathlib import Path

import h3rmg


def dump(path, step, cells, objects=(), extras=()):
    """A dump of format 1 (xdd.c's 10.): a 2 x 1 x 1 map's `cells` (12 words each)."""
    raw = b"H3RMGCEL" + struct.pack("<5i", 1, step, 2, 1, 1)
    raw += b"".join(struct.pack("<12I", *c) for c in cells)
    raw += struct.pack("<i", len(objects)) + b"".join(struct.pack("<6i", *o) for o in objects)
    raw += struct.pack("<i", len(extras)) + b"".join(struct.pack("<3i", *e) for e in extras)
    path.write_bytes(raw)


CELL = (0, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0, 0, 0x7FBC7FBC, 0xFFFF7FBC, 0x548,
        0xA000000, 0)


class PhaseOracle(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.tmp = Path(tmp.name)

    def test_log_by_phase(self):
        log = self.tmp / "rmg1.draws"
        log.write_text("seed 5\ngenerate\n549975 7\nstep 1\n532A5B 1\n53BB5F 2\n53BB5F 3\n"
                       "step 3\n5B38D3 4\ngenerated 1\nwrite\nend 5 0\n")
        phases, elsewhere = h3rmg.phase_log(log)
        self.assertEqual(elsewhere, 0)
        self.assertEqual([p for p, _ in phases],
                         ["setup", "template", 1, 3, "generated", "write"])
        self.assertEqual(phases[2][1], [(0x532A5B, 1), (0x53BB5F, 2), (0x53BB5F, 3)])
        self.assertEqual(h3rmg.runs_of(phases[2][1]), "532A5B 53BB5F×2")

    def test_log_whose_end_disagrees(self):
        log = self.tmp / "rmg1.draws"
        log.write_text("seed 5\ngenerate\n549975 7\nend 2 0\n")
        with self.assertRaises(RuntimeError):
            h3rmg.phase_log(log)

    def test_cells_differing_in_memory_bits_only(self):
        a, b = self.tmp / "a.cells", self.tmp / "b.cells"
        dump(a, 0, [CELL, CELL])
        # +0x14 (word 5) and +0x2C's bits 1-31 (word 11) as the heap left them.
        dump(b, 0, [CELL[:5] + (0x1234,) + CELL[6:11] + (0xB3B8B3B8,), CELL])
        found, memory = h3rmg.cells_differences(a, b)
        self.assertEqual(found, {"+0x14": 1, "+0x2C": 1})
        self.assertTrue(memory)

    def test_cells_differing_in_a_set_bit(self):
        a, b = self.tmp / "a.cells", self.tmp / "b.cells"
        dump(a, 6, [CELL, CELL], objects=[(3, 5, 0, 1, 0, 0)])
        # +0x2C's bit 0 (a pending Border Guard) and an object's place are no memory bits.
        dump(b, 6, [CELL[:11] + (1,), CELL], objects=[(3, 5, 0, 1, 0, 0)])
        found, memory = h3rmg.cells_differences(a, b)
        self.assertEqual(found, {"+0x2C": 1})
        self.assertFalse(memory)
        dump(b, 6, [CELL, CELL], objects=[(3, 5, 0, 0, 0, 0)])
        self.assertEqual(h3rmg.cells_differences(a, b), ({"objects": 1}, False))

    def test_draws_by_phase_digested_as_logged(self):
        log = self.tmp / "rmg1.draws"
        log.write_text("seed 5\ngenerate\n549975 7\nstep 1\n532A5B 1\n53BB5F 2\nstep 9\n"
                       "generated 1\nwrite\n54ABA6 3\nend 4 0\n")
        phases, _ = h3rmg.phase_log(log)
        steps = h3rmg.phase_digests(phases)
        self.assertEqual([(k, n) for k, n, _ in steps],
                         [("setup", 0), ("template", 1), ("1", 2), ("9", 0), ("generated", 0),
                          ("write", 1)])
        # A phase's digest is its lines' as the log prints them, an empty one the empty text's.
        self.assertEqual(steps[2][2], hashlib.sha256(b"532A5B 1\n53BB5F 2\n").hexdigest()[:12])
        self.assertEqual(steps[0][2], hashlib.sha256(b"").hexdigest()[:12])

    def test_step_6s_plan_ends_at_its_first_connection(self):
        plan = [(0x543EB8, 2), (0x54403E, 9), (0x5B38B9, 4), (0x5B38D3, 1)]
        phases = [(5, [(0x544DEC, 1)]), (6, plan + [(0x54146B, 0), (0x5B38D3, 3)]), (8, [])]
        text = "".join(f"{s:X} {v}\n" for s, v in plan).encode()
        lanes = "".join(f"{s:X} {v}\n" for s, v in plan[:2]).encode()
        self.assertEqual(h3rmg.plan_prefix(phases),
                         ((2, hashlib.sha256(lanes).hexdigest()[:12]),
                          (4, hashlib.sha256(text).hexdigest()[:12]), 0x54146B))
        self.assertEqual(h3rmg.plan_line("c", phases).split()[1:],
                         [f"lanes=2:{hashlib.sha256(lanes).hexdigest()[:12]}", "draws=4",
                          f"digest={hashlib.sha256(text).hexdigest()[:12]}", "next=54146B"])
        # A step 6 of the plan alone (no connection draws) ends with none.
        self.assertEqual(h3rmg.plan_prefix([(6, plan)])[2], None)
        self.assertTrue(h3rmg.plan_line("c", [(6, [])]).endswith(
            "lanes=0:e3b0c44298fc draws=0 digest=e3b0c44298fc next=none"))
        # A plan cut by a site of neither kind, or a log without step 6, is refused.
        with self.assertRaises(RuntimeError):
            h3rmg.plan_prefix([(6, plan + [(0x540C61, 0)])])
        with self.assertRaises(RuntimeError):
            h3rmg.plan_prefix([(5, plan)])

    def test_memory_record(self):
        zone = bytearray(0xD4)
        zone[0x41], zone[0x44], zone[0x84] = 1, 1, 0xCC
        zone[0xC:0x10] = b"\x07\x00\x00\x00"
        generator = bytearray(0x44)
        generator[8:12] = b"\xFF\xFF\xFF\xFF"
        path = self.tmp / "rmg1.memory"
        path.write_text(f"cw seed 027F\ncw 1 027F\nzone 53BDA1 1 {bytes(0xD4).hex().upper()} "
                        f"{bytes(0x44).hex()}\ncw 2 027F\nzone 53E149 2 {zone.hex().upper()} "
                        f"{generator.hex().upper()}\nend\n")
        words, zones = h3rmg.read_memory(path)
        self.assertEqual(h3rmg.control_words(words), "027F")
        self.assertEqual([(site, step) for site, step, _, _ in zones],
                         [(0x53BDA1, 1), (0x53E149, 2)])
        f = h3rmg.zone_fields(*(zones[1][k] for k in (0, 2, 3)))
        # +0x41 and +0x44 set: bits 0 and 3; +0x84's 0xCC is "not 0"; the limits are not kept.
        self.assertEqual((f["kind"], f["set"], f["mask"]), ("temporary", 2, "009"))
        self.assertEqual((f["same"], f["m84"], f["m94"]), (0, 1, 0))
        self.assertNotIn("g8", f)  # a heap value two fresh starts can disagree on
        self.assertEqual(h3rmg.zone_lines("c", {"zones": [f]}),
                         ["c 1 temporary set=2 mask=009 same=0 m84=1 m94=0"])
        self.assertEqual(h3rmg.zones_digest([]), "none")
        self.assertEqual(h3rmg.control_words({"seed": 0x27F, "1": 0x37F}), "seed:027F,1:037F")
        path.write_text("cw seed 027F\n")
        with self.assertRaises(RuntimeError):
            h3rmg.read_memory(path)

    def test_zone_bytes_fail_only_a_fresh_start(self):
        def facts(zones):
            return {"h3m": "x", "phases": {"draws": 1, "log": "l", "cw": "027F",
                                           "zones": zones, "cells": [(0, "c")]}}
        a = facts([{"kind": "temporary", "set": 0}])
        b = facts([{"kind": "temporary", "set": 1}])
        self.assertEqual(h3rmg.compared(a, a), ("same", False))
        self.assertEqual(h3rmg.compared(a, b), ("same; zone bytes DIFFER (1 zones)", True))
        self.assertEqual(h3rmg.compared(a, b, fresh=False),
                         ("same; zone bytes differ (1 zones)", False))


if __name__ == "__main__":
    unittest.main()
