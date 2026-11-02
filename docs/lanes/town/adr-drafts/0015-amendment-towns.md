# ADR-0015 amendment (draft): the town crates, the town part, the import of towns and starting heroes

- Status: Draft (E09 T2a, 2026-11-02), with the draft ADR-0028 beside it; the user's decisions 2,
  3, 6, 8, 9, 12 to 14 (TWN-Q1, TWN-Q2) and 17 to 19 (TWN-Q3). Accepted at T2b's checkpoint
  (`TWN-Q4`), it goes into ADR-0015 through the `shared/` pull request that creates the crates.
- Amends: point 1 (the crates), point 2 (the ruleset parts), point 4 (the import's placeholders
  and refusals), point 5 (the contract ids).
- Working detail: the survey `docs/rules/surveys/E09-towns.md` §1, §2, §4, §5; the design note
  `docs/epics/E09-design.md` (T2b).

## Context

ADR-0015 lists four crates and the parts beside the core, and makes the import refuse what an
increment cannot represent: a random hero and a hero generated at a main town since E08's
increment 1, a hero on a town's blocked tile since (`Unsupported::HeroInTown`), and it keeps a
town as an untyped blocker. The survey finds every shipped map importable once towns, the main
town's hero, the random hero and a hero in a town exist (finding 2); the user put all four, the
players' faction choice and the starting-hero pick in E09's increment 1 (decisions 6, 8, 9).

## Decision (amendment)

1. **Point 1, the crates:** two more, `h3-towns` (domain, logic) and `h3-pack-towns` (adapter),
   as ADR-0028 point 1; `h3-game` depends on `h3-towns`. The import's towns are `h3-map`'s
   `src/import/town*`, the town lane's (decision 3).
2. **Point 2, the parts:** a third context part, the **town part** (`TownRuleset`; ADR-0028
   point 5). A pack folder holds `core.toml`, `battle.toml`, `adventure.toml` and `towns.toml`;
   `GameRuleset` bundles the core and the three parts, and a replay's header and the server's
   manifest record each part's hash.
3. **Point 4, the import, from E09's increment 1.** It still draws nothing and checks no game
   invariant:
   - **Towns** (classes 77 and 98) become two setups: `h3-towns`' town setup (a fixed faction, or
     a random town's rule with its alignment and owner; the built and forbidden buildings as the
     map gives them, or the standard set with the map's Fort flag; the events; the name) and the
     adventure's town object (its template, owner and garrison; ADR-0016's amendment), joined by
     the town's `TownId`. A random stack in a garrison stays a placeholder of its level and
     upgrade. An alignment byte from 8 to 254 is an invalid map, a typed error (T-1, decision 12),
     as an owner colour nobody plays.
   - **Players:** each player's setup gains its **allowed factions** (the map's set), its
     **faction choice** (the map's default: the one allowed faction, else Random), its
     **starting-hero pick** (none from the map), its **main town** (the header's, as the town's
     entrance tile, finding 3) with whether it **generates a hero**, and its **random-hero flag**.
     Whoever seats the players may change the choice and the pick before the game is built, as
     the human flag (decision 6: the seam with E10's scenario setup); the game validates them
     (ADR-0017's amendment).
   - **The random hero** (class 70) is a hero placeholder with its owner and the rest of its
     body as a placed hero's (decision 8); **the main town's hero** is the game's when the main
     town generates one. Neither is `Unsupported` any longer.
   - **A hero on a town's blocked tile** right of its entrance is moved onto the entrance (the
     original's `placed_heroes_setup`, VCMI's "H3M format quirk"; TD-6 by the reading, decision
     13); elsewhere on a town's blocked tiles it stays `Unsupported::HeroInTown` (none on the
     shipped maps).
   - **The allowed heroes** are the map's set, an RoE-format map's with the Conflux heroes when
     the ruleset has them (survey §4.2; by the packs, decision 14) and never the campaign
     heroes; SoD's disposed heroes give their colour masks.
   - No version is read for a rule: the moat and Conflux follow the packs (decision 14; ADR-0023's
     amendment).
4. **Point 5, the contract ids:** `TownId` beside `PlayerId` and `HeroId` (ADR-0028 point 3).

## Consequences

- The map import's version bumps (the adventure lane's, `cross-lane`); exit criterion 1's census
  of the imported maps goes into `docs/formats/h3m.md`.
- The arbitrary-bytes property test covers the towns' and players' new fields with the rest.
- `h3-map`'s writer (the experiment maps' towns of a fixed faction, E04c T5b) gains what batch
  TWN-1 needs (T3), in `src/writer/town*`.

## Alternatives considered

- The import resolving a random town or drawing a starting hero: the import draws nothing
  (point 4), so a placeholder stays one until the game's build.
- A hero on any of a town's blocked tiles moved (TD-6): only the reading's case, decision 13.
