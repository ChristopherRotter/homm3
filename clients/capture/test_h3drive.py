"""Checks of h3drive.py's parts that need no game, no Wine and no display (E17 M7 T4): every
command it would run is recorded instead. From clients/capture/: `python3 -m unittest
test_h3drive` (numpy, as the harness)."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import h3drive


class HeldInputs(unittest.TestCase):
    """E17 M7 T4 (R1-2): a key or a button an action holds is never left down on the display."""

    def setUp(self):
        self.calls = []
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.tmp = Path(tmp.name)

        def ran(args, **_):
            self.calls.append(list(args))
            return subprocess.CompletedProcess(args, 0)

        def started(args, **_):
            self.calls.append(["started", *args])
            return mock.MagicMock()

        environ = {k: v for k, v in os.environ.items() if k not in ("H3_RUN", "WINEPREFIX")}
        # The disk check reads the run directory's file system (`DiskCheck` below checks it).
        for patch in (mock.patch.object(h3drive.subprocess, "run", side_effect=ran),
                      mock.patch.object(h3drive.subprocess, "Popen", side_effect=started),
                      mock.patch.object(h3drive.time, "sleep"),
                      mock.patch.object(h3drive, "disk_check_fault", return_value=None),
                      mock.patch.dict(os.environ, environ, clear=True)):
            patch.start()
            self.addCleanup(patch.stop)

    def after(self, command):
        """The commands run after the last `command`."""
        last = max(i for i, c in enumerate(self.calls) if c == command)
        return self.calls[last + 1:]

    def test_a_run_that_fails_while_a_key_and_a_button_are_down_lets_them_go(self):
        game = h3drive.Game(self.tmp / "out")
        h3drive.act(game, "key-down:d")
        h3drive.act(game, "right:100,100")
        h3drive.act(game, "down:700,570")
        # The capture raised here: `run_set`'s `finally` stops the game.
        game.stop()
        self.assertEqual(self.after(["wineserver", "-k"]),
                         [["xdotool", "keyup", "d"], ["xdotool", "mouseup", "3"],
                          ["xdotool", "mouseup", "1"]])
        self.assertEqual((game.keys_held, game.right_held, game.left_held), (set(), False, False))

    def test_a_run_that_ends_as_it_should_lets_go_of_nothing_more(self):
        game = h3drive.Game(self.tmp / "out")
        h3drive.act(game, "key-down:d")
        h3drive.act(game, "key-up:d")
        game.stop()
        self.assertEqual(self.after(["wineserver", "-k"]), [])

    def test_a_game_starts_with_the_panel_s_hotkeys_and_the_buttons_up(self):
        # A killed run's `finally` never ran: the next game's start lets go of all of them before
        # the game starts.
        game_dir = self.tmp / "game"
        prefix = self.tmp / "prefix"
        (prefix / "dosdevices").mkdir(parents=True)
        (prefix / "dosdevices" / "d:").symlink_to(game_dir)
        os.environ["WINEPREFIX"] = str(prefix)
        with mock.patch.object(h3drive, "GAME_DIR", game_dir.resolve()), \
                mock.patch.object(h3drive, "sound_settings"):
            h3drive.Game(self.tmp / "out").start()
        released = [["xdotool", "keyup", k] for k in h3drive.PANEL_HOTKEYS]
        released += [["xdotool", "mouseup", "3"], ["xdotool", "mouseup", "1"]]
        self.assertEqual(self.after(["wineserver", "-k"]),
                         [*released, ["started", "wine", "Heroes3.exe"]])


class DiskCheck(unittest.TestCase):
    """E24 M4 T6 (the reviewer's R6-6): the game's disk check reads the free bytes in 32 bits,
    signed, and ends the game below 5 MiB, so a run outside [5 MiB, 2 GiB) modulo 4 GiB fails
    before the game starts, saying why."""

    GIB, MIB = 2 ** 30, 2 ** 20

    def fault(self, free, frsize=4096):
        st = mock.Mock(f_bavail=free // frsize, f_frsize=frsize)
        environ = {k: v for k, v in os.environ.items() if k != "H3_NO_DISK_CHECK"}
        with mock.patch.object(h3drive.os, "statvfs", return_value=st), \
                mock.patch.dict(os.environ, environ, clear=True):
            return h3drive.disk_check_fault("/run")

    def test_the_band_is_5_mib_to_2_gib_modulo_4_gib(self):
        inside = [5 * self.MIB, self.GIB, 2 * self.GIB - 4096, 4 * self.GIB + 5 * self.MIB,
                  20 * self.GIB + self.GIB]
        outside = [0, 5 * self.MIB - 4096, 2 * self.GIB, 3 * self.GIB, 4 * self.GIB - 4096,
                   4 * self.GIB, 8 * self.GIB + self.MIB]
        self.assertEqual([self.fault(f) for f in inside], [None] * len(inside))
        self.assertTrue(all(self.fault(f) for f in outside))

    def test_the_message_says_why_and_how_large_a_ballast(self):
        said = self.fault(23 * self.GIB + 512 * self.MIB)  # read as -512 MiB
        self.assertIn("read as -536870912", said)
        self.assertIn(f"ballast file of {2 * self.GIB + 512 * self.MIB} bytes", said)
        self.assertIn("free some space", self.fault(3 * self.MIB))

    def test_h3_no_disk_check_leaves_it_out(self):
        with mock.patch.dict(os.environ, {"H3_NO_DISK_CHECK": "1"}), \
                mock.patch.object(h3drive.os, "statvfs") as statvfs:
            self.assertIsNone(h3drive.disk_check_fault("/run"))
        statvfs.assert_not_called()

    def test_a_game_outside_the_band_never_starts(self):
        with tempfile.TemporaryDirectory() as tmp:
            game_dir, prefix = Path(tmp) / "game", Path(tmp) / "prefix"
            (prefix / "dosdevices").mkdir(parents=True)
            (prefix / "dosdevices" / "d:").symlink_to(game_dir)
            environ = {"WINEPREFIX": str(prefix), "HOME": tmp}
            with mock.patch.object(h3drive, "GAME_DIR", game_dir.resolve()), \
                    mock.patch.object(h3drive, "disk_check_fault", return_value="the check"), \
                    mock.patch.object(h3drive.subprocess, "run") as run, \
                    mock.patch.object(h3drive.subprocess, "Popen") as popen, \
                    mock.patch.dict(os.environ, environ, clear=True):
                with self.assertRaisesRegex(RuntimeError, "the check"):
                    h3drive.Game(Path(tmp) / "out").start()
        run.assert_not_called()
        popen.assert_not_called()


class WineEnvironment(unittest.TestCase):
    """E27 M3 T3b2, E24 M4 T2b2: Wine 9's 32-bit loader crashes at its start where TMPDIR is set
    (`free(): invalid pointer`), so the harness's commands run without it."""

    def test_tmpdir_never_reaches_wine(self):
        environ = {"TMPDIR": "/tmp", "HOME": "/root"}
        with mock.patch.dict(os.environ, environ, clear=True), \
                mock.patch.object(h3drive, "instance_fault", return_value=None):
            e = h3drive.env()
        self.assertNotIn("TMPDIR", e)
        self.assertEqual((e["HOME"], e["WINEPREFIX"]), ("/root", "/root/.wine-h3"))


if __name__ == "__main__":
    unittest.main()
