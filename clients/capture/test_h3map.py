"""Checks of h3map.py's and h3lod.py's parts that need no game, no Wine and no display (E24 M4
T6, the reviewer's R6-1, R6-4, R6-5, R6-7, R6-11, R6-12, R6-13): the staging's checks and
writes, `frames`' kept live parts, the engine's binaries, the kept texts' alphabet. From
clients/capture/: `python3 -m unittest test_h3map` (numpy, as the harness)."""

import codecs
import contextlib
import io
import json
import os
import re
import tempfile
import time
import tomllib
import unittest
from pathlib import Path
from unittest import mock

import h3drive
import h3lod
import h3map
import h3mem

CLIENTS = Path(__file__).resolve().parents[1]


class ShippedBonus(unittest.TestCase):
    """R6-1: a shipped map's (and an underground's) `bonus = "gold"` is set before Begin, as a
    writer's map's is, not dropped."""

    def calls(self, **kwargs):
        calls = []
        game = mock.Mock(proc=None)
        game.wait.side_effect = lambda name, *_: calls.append(("wait", name))
        game.click.side_effect = lambda x, y, *_: calls.append(("click", x, y))
        game.matches.return_value = True
        with mock.patch.object(h3drive, "gold_bonus",
                               side_effect=lambda g, players: calls.append(("gold", players))), \
                mock.patch.object(h3map.h3adv, "dismiss"):
            h3map.to_shipped(game, **kwargs)
        return calls

    def test_the_gold_bonus_is_set_between_the_list_and_begin(self):
        calls = self.calls(gold=3)
        at = calls.index(("wait", "scenario_list"))
        self.assertEqual(calls[at + 1:at + 3], [("gold", 3), ("click", 497, 553)])

    def test_no_bonus_without_gold(self):
        self.assertNotIn("gold", [c[0] for c in self.calls()])

    def test_started_hands_the_gold_to_a_shipped_start(self):
        with tempfile.TemporaryDirectory() as tmp, \
                mock.patch.object(h3drive, "Game"), \
                mock.patch.object(h3map, "to_shipped") as shipped:
            h3map.started(Path(tmp) / "map.h3m", Path(tmp), shipped=True, gold=2)
        self.assertEqual(shipped.call_args.kwargs, {"gold": 2})


class CheckSituation(unittest.TestCase):
    """R6-1, R6-4, R6-5, R6-13: what `check_situation` refuses before the game starts."""

    SHIPPED = {"map": "A.h3m", "view": [0, 0, 0], "explored": "start"}

    def refused(self, sit, sequence=False):
        with self.assertRaises(ValueError) as err:
            h3map.check_situation("s.toml", sit, sequence=sequence)
        return str(err.exception)

    def test_a_shipped_bonus_needs_its_players(self):
        self.assertIn("players", self.refused({**self.SHIPPED, "bonus": "gold"}))
        self.assertIn("players", self.refused({**self.SHIPPED, "bonus": "gold", "players": 0}))
        self.assertIn("bonus", self.refused({**self.SHIPPED, "bonus": "random", "players": 2}))
        self.assertIn("without `bonus`", self.refused({**self.SHIPPED, "players": 2}))
        h3map.check_situation("s.toml", {**self.SHIPPED, "bonus": "gold", "players": 3})

    def test_reselect_needs_a_select(self):
        self.assertIn("reselect", self.refused({**self.SHIPPED, "reselect": [1, 2]}))
        h3map.check_situation("s.toml", {**self.SHIPPED, "select": {"town": [3, 4]},
                                         "reselect": [1, 2]})

    def test_diplomacy_twice_is_refused(self):
        hero = {"at": [1, 2], "diplomacy": 2, "secondary": [[4, 1]]}
        self.assertIn("Diplomacy", self.refused({"view": [0, 0, 0], "heroes": [hero]}))
        hero["secondary"] = [[7, 1]]
        h3map.check_situation("s.toml", {"view": [0, 0, 0], "heroes": [hero]})

    def test_a_sequence_s_situation_has_no_pointer(self):
        sit = {**self.SHIPPED, "pointer": [216, 472]}
        self.assertIn("pointer", self.refused(sit, sequence=True))
        h3map.check_situation("s.toml", sit)

    def test_a_sequence_refuses_one_before_the_game_starts(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "p.toml").write_text('map = "A.h3m"\nview = [0, 0, 0]\n'
                                              'pointer = [216, 472]\n')
            with mock.patch.object(h3map, "SITUATIONS", Path(tmp)), \
                    mock.patch.object(h3map.subprocess, "run"), \
                    mock.patch.object(h3map, "started") as started, \
                    self.assertRaisesRegex(ValueError, "pointer"):
                h3map.sequence({"id": "p", "situation": "p", "ms": 10}, Path(tmp) / "out")
        started.assert_not_called()

    def test_every_committed_situation_passes(self):
        for path in sorted((CLIENTS / "map/situations").glob("*.toml")):
            if path.name != "set.toml":
                h3map.check_situation(path, tomllib.loads(path.read_text()))
        for e in tomllib.loads((CLIENTS / "map/sequences.toml").read_text())["sequence"]:
            path = CLIENTS / "map/situations" / f"{e['situation']}.toml"
            h3map.check_situation(path, tomllib.loads(path.read_text()), sequence=True)


class FakeMem(h3mem.Mem):
    """A game's memory as bytes by address, one hero of Red's on every tile."""

    def __init__(self):
        self.bytes = {}

    def game(self):
        return 0

    def write_bytes(self, address, data):
        self.bytes.update(zip(range(address, address + len(data)), data))

    def hero_on(self, x, y, z=0):
        return 0

    def hero(self, index):
        return {"index": index, "name": "hero"}


class Lists(unittest.TestCase):
    """R6-5: a hero's `secondary` clears every level; its `diplomacy` is written after it."""

    def test_diplomacy_survives_the_secondary_skills(self):
        mem = FakeMem()
        hero = {"at": [1, 2], "diplomacy": 3, "secondary": [[7, 1], [21, 2]]}
        h3map.lists(None, mem, {"heroes": [hero]})
        record = h3mem.HEROES
        self.assertEqual(mem.bytes[record + h3mem.DIPLOMACY], 3)
        self.assertEqual(mem.bytes[record + h3mem.SECONDARY + 7], 1)
        self.assertEqual(mem.bytes[record + h3mem.SECONDARY + 21], 2)


class HotSeat(unittest.TestCase):
    """E24 M5 T5 (design §5.31): a sequence's hot seat, its start and its timeline's lines."""

    def test_a_sequence_s_hot_seat_is_checked(self):
        entry = {"id": "s", "hot_seat": ["Red", "Blue"]}
        self.assertEqual(h3map.hot_seat_of(entry),
                         {"names": ["Red", "Blue"], "duration": 10, "start": False})
        self.assertEqual(h3map.hot_seat_of({**entry, "duration": 0, "start": True}),
                         {"names": ["Red", "Blue"], "duration": 0, "start": True})
        self.assertIsNone(h3map.hot_seat_of({"id": "s"}))
        for bad in [{"id": "s", "duration": 3}, {"id": "s", "start": True},
                    {**entry, "hot_seat": ["Red"]}, {**entry, "hot_seat": ["Red", "Pale Blue"]},
                    {**entry, "hot_seat": "Red"}, {**entry, "duration": 11},
                    {**entry, "duration": True}, {**entry, "start": 1},
                    {**entry, "start": True, "status": [1, 100]}]:
            with self.assertRaises(ValueError, msg=bad):
                h3map.hot_seat_of(bad)
        options, inputs = h3map.sequence_inputs({**entry, "situation": "x", "ms": 5,
                                                 "duration": 4, "start": False})
        self.assertEqual((options, inputs), ([], {}))

    def setup_calls(self, **kwargs):
        calls = []
        game = mock.Mock(proc=None)
        game.wait.side_effect = lambda name, *_: calls.append(("wait", name))
        game.click.side_effect = lambda x, y, *_: calls.append(("click", x, y))
        game.seed.side_effect = lambda value: calls.append(("seed", value))
        with mock.patch.object(h3drive, "run"), \
                mock.patch.object(h3drive, "gold_bonus",
                                  side_effect=lambda g, players: calls.append(("gold", players))), \
                mock.patch.object(h3map.h3adv, "dismiss"):
            h3map.h3adv.to_hot_seat(game, ["Red", "Blue"], **kwargs)
        return calls[calls.index(("wait", "scenario_list")) + 2:]

    def test_the_duration_and_the_seed_are_set_before_begin(self):
        left, begin = ("click", 66, 565), ("click", 497, 553)
        self.assertEqual(self.setup_calls(duration=7, seed=5),
                         [("click", 513, 518), left, left, left, ("seed", 5), begin,
                          ("wait", "turn_dialog")])
        # The bonus opened the advanced options already; a number of rows, as to_scenario's.
        self.assertEqual(self.setup_calls(gold=2, duration=9),
                         [("gold", 2), left, begin, ("wait", "turn_dialog")])
        self.assertEqual(self.setup_calls(gold=True, duration=10), [("gold", 2), begin,
                                                                    ("wait", "turn_dialog")])
        self.assertEqual(self.setup_calls(duration=10, begin=False), [])
        with self.assertRaises(ValueError):
            self.setup_calls(duration=11)

    def test_started_hands_the_hot_seat_over_and_lets_its_sounds_play_out(self):
        with tempfile.TemporaryDirectory() as tmp, \
                mock.patch.object(h3drive, "Game"), \
                mock.patch.object(h3drive, "sounds_settled") as settled, \
                mock.patch.object(h3map.h3adv, "to_hot_seat") as seat:
            h3map.started(Path(tmp) / "map.h3m", Path(tmp), seed=3, sound=True, gold=2,
                          hot_seat={"names": ["A", "B"], "duration": 0})
            self.assertEqual(seat.call_args.kwargs,
                             {"gold": 2, "seed": 3, "duration": 0, "begin": True})
            settled.assert_called_once()
            h3map.started(Path(tmp) / "map.h3m", Path(tmp), sound=True,
                          hot_seat={"names": ["A", "B"], "begin": False})
            settled.assert_called_once()

    def test_the_humans_and_the_duration_begun_are_checked(self):
        mem = mock.Mock()
        mem.is_human.side_effect = lambda c: c < 2
        mem.begin_humans.return_value = 2
        mem.turn_duration.return_value = 10
        sit = {"heroes": [{"at": [1, 1]}, {"at": [2, 2], "owner": 2}, {"at": [3, 3], "owner": 1}]}
        h3map.check_hot_seat(mem, sit, {"names": ["A", "B"]})
        with self.assertRaisesRegex(RuntimeError, "duration"):
            h3map.check_hot_seat(mem, sit, {"names": ["A", "B"], "duration": 0})
        with self.assertRaisesRegex(RuntimeError, "humans"):
            h3map.check_hot_seat(mem, sit, {"names": ["A", "B", "C"]})
        with self.assertRaisesRegex(ValueError, "colours"):
            h3map.check_hot_seat(mem, {"heroes": sit["heroes"][:2]}, {"names": list("ABC")})

    def test_a_start_stages_nothing(self):
        self.assertEqual(h3map.start_staged({"view": [0, 0, 0], "explored": "start",
                                             "heroes": [{"at": [1, 1], "class": 2}]}), [])
        self.assertEqual(h3map.start_staged({
            "date": [1, 1, 7], "heroes": [{"at": [1, 1], "points": 3}],
            "boats": [{"at": [2, 2], "facing": 1}], "towns": [{"at": [3, 3], "built": True}]}),
            ["built", "date", "facing", "points"])
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "s.toml").write_text('view = [0, 0, 0]\nselect = { hero = [1, 1] }\n')
            with mock.patch.object(h3map, "started") as started, \
                    self.assertRaisesRegex(ValueError, "stages nothing"):
                h3map.stage_start(Path(tmp) / "s.toml", Path(tmp) / "out", hot_seat={})
        started.assert_not_called()

    def test_the_timeline_s_hot_seat_lines_where_they_change(self):
        base = {"view": [0, 0, 0], "objects": [0], "cycles": [0, 0], "cursor": [1, 0, 700, 300],
                "arrows": []}
        seat = lambda t, shown, covered, timer: {  # noqa: E731
            **base, "t": t, "shown": [shown], "covered": [covered], "timer": timer}
        lines = h3map.map_timeline([seat(0, 0, 0, [1, 1]), seat(1, 0, 0, [1, 0]),
                                    seat(2, 1, 1, [0, 0]), seat(3, 1, 0, [0, 0])])
        self.assertEqual([line for line in lines if line.split()[1] in
                          ("shown", "covered", "timer")],
                         ["1 timer 1 0", "2 shown 1", "2 covered 1", "2 timer 0 0",
                          "3 covered 0"])
        self.assertFalse([line for line in h3map.map_timeline([{**base, "t": 0}])
                          if line.split()[1] in ("shown", "covered", "timer")])

    def test_the_clock_s_lines(self):
        # E24 M5 T5 part four (design §5.32): a warning in its millisecond, the chat where it
        # changes (never at 0).
        base = {"view": [0, 0, 0], "objects": [0], "cycles": [0, 0], "cursor": [1, 0, 700, 300],
                "arrows": []}
        lines = h3map.map_timeline([
            {**base, "t": 0, "chat": [0]},
            {**base, "t": 30001, "chat": [1, -1], "warning": [20000]},
            {**base, "t": 30060, "chat": [1, 30060]},
            {**base, "t": 50040, "chat": [0]}])
        self.assertEqual([line for line in lines if line.split()[1] in ("chat", "warning")],
                         ["30001 chat 1 -1", "30001 warning 20000", "30060 chat 1 30060",
                          "50040 chat 0"])

    def test_the_chat_s_new_lines(self):
        a, b, c = ["a", 1], ["b", 1], ["c", 0]
        self.assertEqual(h3map.chat_added([], [a]), [a])
        self.assertEqual(h3map.chat_added([a, b], [b, c]), [c])
        self.assertEqual(h3map.chat_added([a, b], [b]), [])
        self.assertEqual(h3map.chat_added([a], [a, a]), [a])
        self.assertEqual(h3map.chat_added([a, b], [a, b]), [])

    def test_the_hot_seat_sequences_pass_their_checks(self):
        # Waiting in `to-record.toml` until the viewer plays them (part three moves them).
        files = [CLIENTS / "map/sequences.toml", CLIENTS / "map/to-record.toml"]
        entries = [e for f in files if f.exists()
                   for e in tomllib.loads(f.read_text())["sequence"] if "hot_seat" in e]
        self.assertTrue(entries)
        for e in entries:
            h3map.sequence_inputs(e)
            path = CLIENTS / "map/situations" / f"{e['situation']}.toml"
            sit = tomllib.loads(path.read_text())
            h3map.check_situation(path, sit, sequence=True)
            if h3map.hot_seat_of(e)["start"]:
                self.assertEqual(h3map.start_staged(sit), [], e["id"])


class KeptLiveParts(unittest.TestCase):
    """R6-7: `frames` keeps a live part in `timelines/` only for a sequence of `sequences.toml`;
    a sound run's too since E24 M5 T5 part three (the hot seat's, which `drawn` visits)."""

    def test_a_scratch_file_s_stays_with_the_run(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            timelines, committed, scratch = tmp / "timelines", tmp / "seq.toml", tmp / "x.toml"
            timelines.mkdir()
            committed.write_text('[[sequence]]\nid = "a"\nsituation = "s"\nms = 5\n\n'
                                 '[[sequence]]\nid = "b"\nsituation = "s"\nms = 5\nsound = true\n')
            scratch.write_text('[[sequence]]\nid = "c"\nsituation = "s"\nms = 5\n')

            def ran(entry, here, sparse=False):
                here.mkdir(parents=True, exist_ok=True)
                (here / "timeline.txt").write_text("0 view 0 0 0\n")
                (here / "live.toml").write_text("k = 0\n")
                return [], 0

            with mock.patch.object(h3map, "SEQUENCES", committed), \
                    mock.patch.object(h3map, "TIMELINES", timelines), \
                    mock.patch.object(h3map, "sequence", side_effect=ran), \
                    contextlib.redirect_stdout(io.StringIO()) as said:
                h3map.run_frames(["--no-check", "--out", str(tmp / "out")])
                h3map.run_frames(["--sequences", str(scratch), "--no-check", "--out",
                                  str(tmp / "out")])
            self.assertEqual(sorted(p.name for p in timelines.iterdir()),
                             ["a-live.toml", "b-live.toml"])
            self.assertIn("c: not of seq.toml", said.getvalue())


class EngineBinaries(unittest.TestCase):
    """R6-12: the engine's binaries checked once, a missing one said clearly, a stale one warned
    of; a failure fails its own start or sequence only."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.tmp = Path(tmp.name)
        self.engine, self.sources = self.tmp / "debug", self.tmp / "crates"
        self.engine.mkdir()
        (self.sources / "a/src").mkdir(parents=True)
        (self.sources / "a/src/lib.rs").write_text("")
        patch = mock.patch.dict(os.environ, {"H3_ENGINE": str(self.engine)})
        patch.start()
        self.addCleanup(patch.stop)

    def test_a_missing_binary_is_named_with_the_build(self):
        (self.engine / "h3").write_text("")
        with self.assertRaisesRegex(RuntimeError, r"`h3-server` not in .*cargo build --locked"):
            h3map.check_engine(self.sources)

    def test_a_binary_older_than_a_source_is_warned_of(self):
        for name in h3map.ENGINE_BINARIES:
            (self.engine / name).write_text("")
        with contextlib.redirect_stderr(io.StringIO()) as said:
            h3map.check_engine(self.sources)
        self.assertEqual(said.getvalue(), "")
        later = time.time() + 60
        os.utime(self.sources / "a/src/lib.rs", (later, later))
        with contextlib.redirect_stderr(io.StringIO()) as said:
            h3map.check_engine(self.sources)
        self.assertIn("older than", said.getvalue())

    def test_engine_goes_on_past_a_failed_start(self):
        situations = self.tmp / "situations"
        (situations / "kept").mkdir(parents=True)
        sit = {"view": [0, 0, 0]}
        lines = []
        for ident in ("one", "two"):
            (situations / f"{ident}.toml").write_text("view = [0, 0, 0]\n")
            (situations / "kept" / f"{ident}.toml").write_text(
                f'situation = "{h3map.situation_digest(sit)}"\n')
            lines.append(f'[[situation]]\nid = "{ident}"\nengine = {{}}\n')
        (situations / "set.toml").write_text("\n".join(lines))
        with mock.patch.object(h3map, "SITUATIONS", situations), \
                mock.patch.object(h3map, "check_engine"), \
                mock.patch.object(h3map, "engine_map", side_effect=FileNotFoundError("h3")) as m, \
                contextlib.redirect_stdout(io.StringIO()) as said:
            self.assertEqual(h3map.run_engine(["--out", str(self.tmp / "out")]), 1)
        self.assertEqual(m.call_count, 2)
        self.assertIn("two: FAILED (FileNotFoundError", said.getvalue())

    def sequences(self):
        timelines, situations = self.tmp / "timelines", self.tmp / "situations"
        timelines.mkdir()
        situations.mkdir()
        (situations / "s.toml").write_text("view = [0, 0, 0]\n")
        digest = h3map.situation_digest({"view": [0, 0, 0]})
        lines = []
        for ident in ("one", "two"):
            (timelines / f"{ident}-live.toml").write_text(f'situation = "{digest}"\n')
            lines.append(f'[[sequence]]\nid = "{ident}"\nsituation = "s"\nms = 5\n'
                         'script = ["0 end-turn"]\n')
        (self.tmp / "seq.toml").write_text("\n".join(lines))
        return [mock.patch.object(h3map, "SEQUENCES", self.tmp / "seq.toml"),
                mock.patch.object(h3map, "TIMELINES", timelines),
                mock.patch.object(h3map, "SITUATIONS", situations)]

    def test_drawn_says_the_missing_binaries_once_and_draws_no_scripted_one(self):
        with contextlib.ExitStack() as stack:
            for patch in self.sequences():
                stack.enter_context(patch)
            ours = stack.enter_context(mock.patch.object(h3map, "ours_frames"))
            said = stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            failed, broken = h3map.drawn_sequences([], self.tmp / "out", False)
        self.assertEqual((failed, broken), (2, 2))
        ours.assert_not_called()
        self.assertEqual(said.getvalue().count("not in"), 1)

    def test_drawn_goes_on_past_a_missing_binary(self):
        with contextlib.ExitStack() as stack:
            for patch in self.sequences():
                stack.enter_context(patch)
            stack.enter_context(mock.patch.object(h3map, "check_engine"))
            stack.enter_context(mock.patch.object(h3map, "scripted", return_value=[]))
            ours = stack.enter_context(mock.patch.object(
                h3map, "ours_frames", side_effect=[FileNotFoundError("h3"), "x"]))
            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            failed, broken = h3map.drawn_sequences([], self.tmp / "out", False)
        self.assertEqual(ours.call_count, 2)
        self.assertEqual((failed, broken), (2, 2))


class KeptTexts(unittest.TestCase):
    """R6-11: what `h3lod.label` keeps between labels is what `h3-map-view`'s `tests/texts.rs`
    accepts, character for character."""

    def rust_kept(self):
        text = (CLIENTS / "crates/h3-map-view/tests/texts.rs").read_text()
        literal = re.search(r'^const KEPT: &str = "(.*)";$', text, re.M).group(1)
        return set(codecs.decode(literal, "unicode_escape"))

    def test_kept_is_the_test_s_set(self):
        rust = self.rust_kept()
        python = {chr(c) for c in range(0x3100) if re.fullmatch(h3lod.KEPT, chr(c))}
        self.assertEqual(python, rust)
        self.assertIsNone(h3lod.LABELLED.fullmatch("[ADVEVENT:٣]"))  # an Arabic-Indic 3

    def test_a_no_break_space_is_another_piece(self):
        self.assertIsNone(h3lod.LABELLED.fullmatch("12\xa0"))
        with mock.patch.object(h3lod, "pieces", return_value=[]):
            self.assertEqual(h3lod.by_pieces("\xa0"), "[other: 1 characters]")
            self.assertEqual(h3lod.by_pieces(" 12, 3"), " 12, 3")

    def test_every_kept_text_is_labelled_as_it_stands(self):
        count = 0
        for path in sorted((CLIENTS / "map/timelines").glob("*-live.toml")):
            part = tomllib.loads(path.read_text())
            texts = [t for m in part.get("message", []) for t in m.get("texts", [])]
            texts += [p[2] for m in part.get("message", []) for p in m["pictures"] if len(p) > 2]
            texts += [t for u in part.get("level_up", []) for t in u.get("texts", [])]
            texts += [f["text"] for f in part.get("found", [])]
            for text in texts:
                self.assertTrue(h3lod.LABELLED.fullmatch(text), f"{path.name}: {json.dumps(text)}")
            count += len(texts)
        self.assertGreaterEqual(count, 353)


if __name__ == "__main__":
    unittest.main()
