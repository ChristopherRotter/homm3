# ADR-0020 amendment (draft): the original's redraw artifacts are not reproduced; its drawing bugs are fixed

- Status: Draft, decided by the user on 2026-10-14 (E17 M3 T1a; point 3 on 2026-10-15, M3 T4a);
  goes into ADR-0020 through a `shared/` pull request at the next checkpoint.
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
   artifact only when it is the viewer's pixel at the same moment with one to four more
   applications of the original's shadow (design §3.3), or one the viewer drew there at an
   earlier moment of the sequence with at least one more (M3 T1c); any other difference fails as
   before. Stale count boxes stay listed differences until the count boxes are drawn (M3 T5),
   which classifies them.

3. A bug in the original's drawing code (the first: its text drawer loses the glyph at an
   unbreakable word's break and aligns the line by the whole word, design §3.6) is fixed in what
   the client draws. The comparisons with the original's captures reproduce it, so that they
   stay exact: a development switch (`h3_render::Bugs`) the comparing commands set, neither a
   player's option nor a quirk profile. It reaches the timeline too, not only the renderer: a
   creature caster's casting group, and the stale frame the original's partial redraws leave of
   a caster its spell missed (M3 T7c-1), are such bugs (CLT-Q8, the user: "dev-only for the
   captures, fixed otherwise"), not artifacts of point 1. Every oddity of the original is first
   an open question (the user, 2026-10-22).

## Consequences

- No stateful renderer and no replay of the original's redraw order per animation kind.
- A recorded frame lists its artifacts apart from its differences; a change in either fails.
- A fixed drawing bug keeps the original's behaviour beside it, for the comparisons only; each
  one is named in the design notes.
- The rest of ADR-0020 point 9 stands: layout, resolution, timing and frame rate as the original.
