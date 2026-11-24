"""Checks of h3rmg.py's phase oracle readers that need no game, no Wine and no display (E14
T1b2): a log and two dumps written as xdd.c's section 10 writes them. From clients/capture/:
`python3 -m unittest test_h3rmg`."""

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


if __name__ == "__main__":
    unittest.main()
