# ADR-0020 amendment (draft): the original's redraw artifacts are not reproduced

- Status: Draft, decided by the user on 2026-10-14 (E17 M3 T1a); goes into ADR-0020 through a
  `shared/` pull request at the next checkpoint.
- Amends: ADR-0020 point 9 (fidelity first: frames compared exactly).
- Detail: `docs/epics/E17-design.md` §7.11 (the reading), `docs/lanes/client/QUEUE.md` (M3 T1c).

## Context

The original redraws the battlefield by dirty rectangles: each frame restores a rectangle and
draws every item that meets it whole, over pixels it did not restore, and shows only that
rectangle (design §7.11). Two visible artifacts follow: shadows darkened again where an item is
drawn over its own earlier image, and the saved screen's stale count boxes under a moving stack.
The viewer composes each frame whole, so it differs from the original exactly there (M3 T1a: the
melee's walk-by, 30 of `behind`'s 194 frames).

## Decision

1. The client draws each frame whole, in every profile. It does not emulate the original's
   partial redraws or reproduce their artifacts: they are not part of `faithful`.
2. The reference check stays exact for everything else: a differing pixel counts as a redraw
   artifact only when it is the viewer's pixel with one to four more applications of the
   original's shadow (design §3.3); any other difference fails as before. Stale count boxes stay
   masked until the count boxes are drawn (M3 T5), which classifies them the same way.

## Consequences

- No stateful renderer and no replay of the original's redraw order per animation kind.
- A recorded frame lists its artifacts apart from its differences; a change in either fails.
- The rest of ADR-0020 point 9 stands: layout, resolution, timing and frame rate as the original.
