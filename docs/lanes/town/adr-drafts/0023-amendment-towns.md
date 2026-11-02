# ADR-0023 amendment (draft): the siege's facts from the town, the moat by the packs

- Status: Draft (E09 T2a, 2026-11-02), with the draft ADR-0028 beside it; the user's decisions 14,
  15 (TWN-Q2) and 20 (TWN-Q3). Accepted at T2b's checkpoint (`TWN-Q4`), it goes into ADR-0023
  through the `shared/` pull request that creates the crates. No seam changes: `Siege` keeps its
  fields.
- Amends: point 2 (who derives the facts, and the moat's source), point 5 (the natural
  obstacles), the Consequences' version line.
- Working detail: the survey `docs/rules/surveys/E09-towns.md` §9.4 (each field's reading); the
  design note `docs/epics/E09-design.md` (T2b).

## Context

ADR-0023 has "the town context, E09" derive `Siege` and lets "the game's version" (the original's
`mapKind`) decide the moat; its Consequences leave how a game gets its version to the adventure
lane, "probably the map's format: an experiment candidate". The survey read the version's source
(the scenario dialog's, survey §9.4); the user ruled that the enabled expansions deliver the
rules, never the map's version (decision 14, E08 decision 104), and dropped the batch row for it
(decision 15).

## Decision (amendment)

1. **Point 2, the derivation:** `h3-towns` derives `Siege` from the town (its faction and built
   buildings) and its faction's town record (ADR-0028 points 5 and 7), for every battle at a
   town, by the readings of survey §9.4: `faction` the town's; `fort` the highest of its Fort,
   Citadel and Castle; `structures` its buildings as the original counts them for its turrets;
   `escape_tunnel` a Stronghold's tunnel built; `morale` and `luck` its buildings' bonuses (the
   Tavern, the Brotherhood of the Sword, the Fountain of Fortune). Point 3's defending hero's town
   bonuses are derived there too and applied by the game. Scenarios and the CLI still give
   `Siege` directly.
2. **Point 2, the moat:** "the game's version decides" becomes **the enabled packs decide**: from
   a Citadel `moat` is the faction's record's kind, below it `Absent`. The base pack's records
   give RoE's and AB's (harmless; the Stronghold none), the SoD pack overrides them (damaging;
   the Stronghold's moat). A Tower's value keeps deciding its land mines' damage (point 2, as the
   battle reads it since E04c T5; the original's own moat flag is absent there, survey §9.4). Under the
   base and AB packs the game plays RoE's or AB's moat on every map, under SoD's on every map; the
   original's version from its scenario dialog (survey §9.4) is the recorded deviation of
   decision 14.
3. **Point 5, the natural obstacles** before the seam: "none in a siege under SoD rules, the
   usual ones for a Stronghold under RoE or AB rules" becomes the faction's record's flag (the
   base pack's Stronghold draws them, the SoD pack's override none), drawn by the game as a
   battle's obstacles (ADR-0017 point 4).

## Consequences

- "How a game gets its version is the adventure lane's (... an experiment candidate)" is
  withdrawn: no version exists (decision 14; batch TWN-1's row 5 dropped, decision 15).
- "The town context's derivation (E09) gets a rule list from the survey's §1": the rules are
  `TWN-*` ids with E04c's survey §1 and E09's §9.4 as sources.
- `Moat`'s doc comment in `h3-contracts` ("the game's version decides it") follows in the
  `shared/` pull request, the battle's `siege::setup` comment by a note in the battle lane's
  inbox; the battle's rules do not change.
- The battle part's siege records (walls, moat hexes and damage per faction) stay the battle's;
  the town part holds only whether a moat exists and does damage, so the two never disagree on
  what the battle needs.

## Alternatives considered

- The game's version from the map's format, or the original's scenario dialog's: the user's
  rule (decision 14) reads the packs.
- The moat's kind in the battle part's siege record, or an edition derived from the packs'
  names (TWN-Q3, decision 20).
