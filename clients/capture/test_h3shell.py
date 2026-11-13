"""Checks of h3shell.py's parts that need no game, no Wine and no display (E27 M1 T2). From
clients/capture/: `python3 -m unittest test_h3shell` (numpy, as the harness)."""

import re
import tempfile
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
