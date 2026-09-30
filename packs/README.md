# Content packs

Human-editable content packs (ADR-0006), parsed by `h3-pack` and compiled into a `Ruleset` by
`h3-content`. Keys are namespaced `pack:name`.

| Pack | Purpose |
|---|---|
| `synthetic/` | Invented creatures with neutral names for tests, benchmarks, replays and the walking skeleton. **Not** original game data (docs/05-testing.md "Test data policy"). |

Original game content is never committed; it will be imported from the user's own installation
(E02, OPEN-QUESTIONS Q9).
