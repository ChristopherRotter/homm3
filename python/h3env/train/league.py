"""The league (E06 T4a, ADR-0026 point 8, `docs/epics/E06-design.md` §8): prioritized fictitious
self-play (AlphaStar's PFSP) over the learner's past, free of torch (the trainer holds the
members' networks, their files sit in the run's directory).

Its members are a snapshot of the learner every `snapshot_every` samples and every promoted
network. Each carries the learner's score against it, `p = (score + 1) / (games + 2)`: the
learner's training episodes against it, a win 1, a draw or a truncation ½, a loss 0, both sums
decayed by `decay` per update (recent form; a new member at ½). The live set: `live` members drawn
without replacement with weights `(1 − p)^pfsp_power` (power 2: "hard", the members the learner
still loses to), drawn again every `live_every` samples (0: at every rollout) and whenever it
holds a retired member or fewer than it could. An episode is self-play with `self_play`, else
against a live member by the same weights. Episodes in flight keep their opponent, so for a while
after a draw a call may pass the previous set's networks too. At most `max_members`: a new member
replaces the most beaten one (the highest `p`) among those with at least `retire_games` games
(not decayed), or among all when none has; a tie retires the oldest.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any

import numpy as np

from .run import Cadence, Config

#: A member's kinds: a snapshot of the learner (its file in `league/`), a promoted network (its
#: file the lineage's, `promoted/<n>.pt`); in T7 the system actor.
KINDS = ("snapshot", "promoted")


@dataclass
class Member:
    """One member: its id (from 0, in order of birth), kind, file (relative to the run's
    directory), birth (samples, update), and the learner's score against it."""

    id: int
    kind: str
    file: str
    samples: int
    update: int
    #: The learner's decayed score sum against it and its decayed games.
    score: float = 0.0
    games: float = 0.0
    #: Its games, not decayed (retirement's threshold).
    played: int = 0

    @property
    def p(self) -> float:
        """The learner's estimated score against it."""
        return (self.score + 1.0) / (self.games + 2.0)


class League:
    """The members, the live set and the draws' generator (`state`, `restore`: in the
    checkpoint; the networks are not)."""

    def __init__(self, config: Config, seed: int) -> None:
        self.config = config
        self.members: list[Member] = []
        self.next_id = 0
        #: The live members' ids, in the order drawn.
        self.live: list[int] = []
        self.draws = Cadence(config.live_every)
        self.rng = np.random.default_rng(seed)

    def member(self, id: int) -> Member | None:
        return next((m for m in self.members if m.id == id), None)

    def weights(self, members: list[Member]) -> np.ndarray:
        """PFSP's weights `(1 − p)^pfsp_power`, normalised (each `p` below 1, so none is 0)."""
        w = np.array([(1.0 - m.p) ** self.config.pfsp_power for m in members])
        return w / w.sum()

    def join(self, kind: str, file: str, samples: int, update: int) -> tuple[Member, Member | None]:
        """A new member; when the league is full the most beaten one retires first (returned)."""
        retired = None
        if len(self.members) >= self.config.max_members:
            seasoned = [m for m in self.members if m.played >= self.config.retire_games]
            # The highest p, the oldest of equals: a unique order.
            retired = max(seasoned or self.members, key=lambda m: (m.p, -m.id))
            self.members.remove(retired)
            if retired.id in self.live:
                self.live.remove(retired.id)
        member = Member(self.next_id, kind, file, samples, update)
        self.next_id += 1
        self.members.append(member)
        return member, retired

    def draw(self, samples: int) -> bool:
        """At a rollout's start: draws the live set when due (or stale); returns whether it did."""
        c = self.config
        could = min(c.live, len(self.members))
        due = self.draws.due(samples)
        if not (c.live_every == 0 or due or len(self.live) < could):
            return False
        if could == 0:
            return False
        picked = self.rng.choice(
            len(self.members), size=could, replace=False, p=self.weights(self.members)
        )
        self.live = [self.members[int(i)].id for i in picked]
        return True

    def opponent(self) -> Member | None:
        """A new episode's opponent: None for self-play, else a live member by PFSP's weights."""
        if not self.live or self.rng.random() < self.config.self_play:
            return None
        live = [m for m in self.members if m.id in self.live]
        return live[int(self.rng.choice(len(live), p=self.weights(live)))]

    def side(self) -> int:
        """The side a member plays in a new episode (uniform)."""
        return int(self.rng.integers(2))

    def settle(self, results: dict[int, list[float]]) -> None:
        """An update's results (member id: [the learner's score sum, games]): every member's sums
        decay, then each member still in the league adds its own."""
        d = self.config.decay
        for m in self.members:
            m.score *= d
            m.games *= d
            score, games = results.get(m.id, (0.0, 0))
            m.score += score
            m.games += games
            m.played += int(games)

    def table(self) -> list[dict[str, Any]]:
        """The member table (the record's, per gate)."""
        return [
            {
                "id": m.id,
                "kind": m.kind,
                "samples": m.samples,
                "p": round(m.p, 4),
                "games": round(m.games, 1),
                "played": m.played,
                "live": m.id in self.live,
            }
            for m in self.members
        ]

    def state(self) -> dict[str, Any]:
        return {
            "members": [asdict(m) for m in self.members],
            "next_id": self.next_id,
            "live": list(self.live),
            "next_draw": self.draws.next,
            "rng": self.rng.bit_generator.state,
        }

    def restore(self, state: dict[str, Any]) -> None:
        self.members = [Member(**m) for m in state["members"]]
        self.next_id = state["next_id"]
        self.live = list(state["live"])
        self.draws.next = state["next_draw"]
        self.rng.bit_generator.state = state["rng"]
