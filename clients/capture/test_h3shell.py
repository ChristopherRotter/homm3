"""Checks of h3shell.py's parts that need no game, no Wine and no display (E27 M1 T2). From
clients/capture/: `python3 -m unittest test_h3shell` (numpy, as the harness)."""

import re
import tempfile

import numpy as np
import tomllib
import unittest
from pathlib import Path
from unittest import mock

import h3drive
import h3shell

KEY = h3drive.SETTINGS_KEY
SETUP_REG = (
    "REGEDIT4\r\n\r\n"
    f"[{KEY}]\r\n"
    '"AppPath"="D:\\\\"\r\n"Show Intro"=dword:00000000\r\n"Main Game Full Screen"=dword:00000001\r\n'
    "\r\n[HKEY_CURRENT_USER\\Software\\Wine\\Drivers]\r\n\"Audio\"=\"\"\r\n"
)


def known_verbs():
    """The steps `Run.do` takes: its `verb == "…"` and `verb in (…)` tests."""
    src = Path(h3shell.__file__).read_text()
    known = set(re.findall(r'verb == "([a-z-]+)"', src))
    for group in re.findall(r"verb in \(([^)]*)\)", src):
        known |= set(re.findall(r'"([a-z-]+)"', group))
    return known


class Settings(unittest.TestCase):
    """A run's settings: the key removed, setup.sh's values, the base's, the recipe's last."""

    def written(self, values):
        with tempfile.TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            (Path(tmp) / "h3.reg").write_bytes(SETUP_REG.encode("cp1252"))
            with mock.patch.object(h3drive, "GAME_DIR", game):
                return h3shell.registry_file(values).decode("cp1252")

    def test_the_key_is_removed_before_it_is_written(self):
        text = self.written({})
        self.assertLess(text.index(f"[-{KEY}]"), text.index(f"[{KEY}]"))
        self.assertTrue(text.startswith("REGEDIT4\r\n\r\n"))

    def test_a_recipe_value_comes_after_setups_so_it_wins(self):
        text = self.written({"Show Intro": 1})
        section = text[text.index(f"[{KEY}]"):].split("\r\n\r\n", 1)[0]
        lines = section.split("\r\n")
        self.assertEqual(lines[-1], '"Show Intro"=dword:00000001')
        self.assertLess(lines.index('"Show Intro"=dword:00000000'), len(lines) - 1)
        # The base settings come before the recipe's, which may override them.
        self.assertIn('"First Time"=dword:00000000', lines)
        self.assertIn('"Bink Video"=dword:00000001', lines)

    def test_the_recipe_overrides_a_base_setting(self):
        section = self.written({"Bink Video": 0}).split(f"[{KEY}]", 1)[1].split("\r\n\r\n")[0]
        self.assertEqual(section.rstrip("\r\n").split("\r\n")[-1], '"Bink Video"=dword:00000000')

    def test_the_other_keys_stay(self):
        self.assertIn("[HKEY_CURRENT_USER\\Software\\Wine\\Drivers]", self.written({}))


class Kept(unittest.TestCase):
    def test_a_shot_and_a_clip(self):
        self.assertEqual(h3shell.kept_line("m", ("menu", "shot", 10, "ab")), "m menu t10 ab")
        clip = ("logo", "clip", 0, (3, 0, 140, "cd"))
        self.assertEqual(h3shell.kept_line("s", clip), "s logo clip 3@0-140 cd")
        entry = h3shell.clip_entry([(0, "aa"), (40, "bb")])
        self.assertEqual(entry[:3], (2, 0, 40))

    def test_a_sample(self):
        sample = ("sound1", "sound", 120, (2, 87, 1))
        self.assertEqual(h3shell.kept_line("m", sample), "m sound1 t120 ch2 87ms x1")

    def test_a_voice(self):
        # E27 M2 T5b2: a clip's sound, its end empty while it still played.
        self.assertEqual(h3shell.kept_line("r", ("voice1", "voice", 10, ("smacker", 2000, 70))),
                         "r voice1 smacker t10-2000 v70")
        self.assertEqual(h3shell.kept_line("r", ("voice2", "voice", 5493, ("bink", None, 127))),
                         "r voice2 bink t5493- v127")

    def test_the_clips_sound_log_gives_each_voice_once(self):
        # xdd.dll's 7.: a Smacker clip's buffer (its Play from the service, its Stop and Release
        # at SmackClose), a Bink clip's sample (its first load, a pause at a skip, its release),
        # one closed before it played (no voice); the times from the frozen start.
        at = h3shell.FROZEN_AT
        log = [f"{at} use 0 0", f"{at} create 45056 88200", f"{at} timer 75 65561",
               f"{at} pan -7 0", f"{at} volume -517 0", f"{at + 10} play 0 1",
               f"{at + 5000} play 0 1", f"{at + 5343} timer-killed 1 0",
               f"{at + 5343} stop 40348 0", f"{at + 5343} release 0 0",
               f"{at + 5493} bink-minimum 2048 22050", f"{at + 5493} bink-allocate 0 0",
               f"{at + 5493} bink-volume 127 0", f"{at + 5493} bink-load 0 2048",
               f"{at + 5493} bink-play 0 88200", f"{at + 5560} bink-load 1 2048",
               f"{at + 12000} bink-stop 0 0", f"{at + 12000} bink-release 0 0",
               f"{at + 13000} create 45056 88200", f"{at + 13000} stop 0 0",
               f"{at + 13000} release 0 0", "a torn line"]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "clipsound"
            path.write_text("".join(line + "\n" for line in log))
            self.assertEqual(h3shell.clip_voices(path),
                             [("smacker", 10, 5343, 70), ("bink", 5493, 12000, 127)])
            self.assertEqual(h3shell.clip_voices(Path(tmp) / "none"), [])

    def test_a_volume_index_inverts_smackw32s_table(self):
        self.assertEqual(h3shell.volume_index(-517), 70)
        self.assertEqual(h3shell.volume_index(0), 127)
        self.assertEqual(h3shell.volume_index(-6), 126)
        self.assertEqual(h3shell.volume_index(-10000), 0)
        self.assertEqual(h3shell.volume_index(1), "unknown1")

    def test_a_mask_leaves_its_box_out_of_the_digest(self):
        a = np.zeros((600, 800), dtype=np.uint16)
        b = a.copy()
        b[100, 400] = 0xFFFF
        self.assertNotEqual(h3shell.digest(a), h3shell.digest(b))
        box = h3shell.mask_box("180,96,620,124")
        self.assertEqual(h3shell.digest(a, [box]), h3shell.digest(b, [box]))
        self.assertEqual(b[100, 400], 0xFFFF)  # the frame itself left as it was
        for bad in ("1,2,3", "5,0,5,9", "0,9,5,9", "a,0,5,9"):
            with self.assertRaises(ValueError, msg=bad):
                h3shell.mask_box(bad)

    def test_the_masks_follow_the_steps(self):
        steps = ["shot:a", "mask:0,0,1,1", "shot:b", "clip:c:100", "mask:2,2,3,3", "shot:d",
                 "unmask", "clip-until:e:main_menu"]
        self.assertEqual(h3shell.shot_masks(steps), {
            "a": [], "b": [(0, 0, 1, 1)], "c": [(0, 0, 1, 1)],
            "d": [(0, 0, 1, 1), (2, 2, 3, 3)], "e": []})

    def test_a_viewer_line_matches_either_run_of_the_original(self):
        self.assertEqual(h3shell.originals("m a t10 ab"), ["m a t10 ab"])
        self.assertEqual(h3shell.originals("s clips clip 3@0-9 ab differs clip 3@0-8 cd"),
                         ["s clips clip 3@0-9 ab", "s clips clip 3@0-8 cd"])
        self.assertEqual(h3shell.compared("m", "a", None, "m a t10 ab"), "not drawn")
        self.assertEqual(h3shell.compared("m", "a", "m a t10 ab", "m a t10 ab"), "exact")
        self.assertEqual(h3shell.compared("m", "a", "m a t20 ab", "m a t10 ab"),
                         "the original's pixels at t20, not t10")
        self.assertTrue(h3shell.compared("m", "zz-none", "m a t10 cd", "m a t10 ab")
                        .startswith("not the original's"))

    def test_two_runs_that_differ_keep_both(self):
        a, b = ("menu", "shot", 10, "ab"), ("menu", "shot", 20, "ab")
        self.assertEqual(h3shell.kept_line("m", a, b), "m menu t10 ab differs t20 ab")
        self.assertEqual(h3shell.kept_line("m", a, a), "m menu t10 ab")

    def test_every_recipe_parses_and_its_ids_are_unique(self):
        recipes = h3shell.all_recipes()
        self.assertTrue(recipes)
        verbs = known_verbs()
        for r in recipes:
            for step in r["steps"]:
                self.assertIn(step.split(":")[0], verbs, f"{r['id']}: {step}")
            names = [s.split(":")[1] for s in r["steps"]
                     if s.split(":")[0] in ("shot", "clip", "clip-until")]
            self.assertEqual(len(names), len(set(names)), f"{r['id']}: a shot's name twice")

    def test_every_verb_is_in_the_steps_doc(self):
        for verb in known_verbs():
            self.assertRegex(h3shell.STEPS, rf"(?m)(^  |\| ){re.escape(verb)}\b", verb)

    def test_every_screen_a_recipe_waits_for_has_probes(self):
        known = h3shell.screens()
        for r in h3shell.all_recipes():
            for step in r["steps"]:
                verb, _, arg = step.partition(":")
                if verb in ("until", "check"):
                    for name in arg.split(":")[0].split("|"):
                        self.assertIn(name, known, f"{r['id']}: {step}")
                elif verb == "clip-until":
                    for name in arg.split(":")[1].split("|"):
                        self.assertIn(name, known, f"{r['id']}: {step}")

    def test_the_viewers_record_names_only_recipes(self):
        ids = {r["id"] for r in h3shell.all_recipes()}
        for ident, _ in h3shell.read_drawn():
            self.assertIn(ident, ids)

    def test_the_kept_files_name_only_recipes_of_their_group(self):
        for path in h3shell.KEPT.glob("*.txt"):
            ids = {r["id"] for r in h3shell.load_group(path.stem)}
            for line in path.read_text().splitlines():
                if line and not line.startswith("#"):
                    self.assertIn(line.split(" ", 1)[0], ids, f"{path.name}: {line}")

    def test_the_groups_screens_files_parse(self):
        for path in h3shell.SHELL.glob("*.screens.toml"):
            for name, entry in tomllib.loads(path.read_text()).items():
                if name != "button":
                    self.assertEqual(len(entry["probes"]), 12, f"{path.name}: {name}")


if __name__ == "__main__":
    unittest.main()
