# Content packs

Human-editable content packs (ADR-0006), parsed by `h3-pack` and compiled into a `Ruleset` by
`h3-content`. Keys are namespaced `pack:name`.

| Pack | Purpose |
|---|---|
| `synthetic/` | Invented creatures with neutral names for tests, benchmarks, replays and the walking skeleton. **Not** original game data (docs/05-testing.md "Test data policy"). |

Original game content is never committed; it will be imported from the user's own installation
(E02; import-only per OPEN-QUESTIONS Q9: copies of original files live in the gitignored
`original-data/` directory, the installation itself is never written to).
