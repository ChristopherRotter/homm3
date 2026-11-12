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
        for patch in (mock.patch.object(h3drive.subprocess, "run", side_effect=ran),
                      mock.patch.object(h3drive.subprocess, "Popen", side_effect=started),
                      mock.patch.object(h3drive.time, "sleep"),
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


if __name__ == "__main__":
    unittest.main()
