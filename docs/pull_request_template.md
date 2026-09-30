<!-- Definition of Done (docs/05-testing.md). Tick every box or say why it does not apply. -->

## What and why

## Checklist

- [ ] Behavior implemented in the correct bounded context, in the ubiquitous language
- [ ] Rule catalog updated (new/changed rules with labelled sources); code has `rule:` markers, tests have `covers:` markers
- [ ] Unit tests + relevant property/replay tests; 100% coverage for touched logic code
- [ ] Quirks considered: known original deviations have a ledger entry and a flag where applicable
- [ ] Security considered: commands validated against actor and state; no hidden info leaked (docs/06-security.md)
- [ ] Performance considered: no allocation in hot paths; `cargo xtask bench-check` passes or the baseline update is explained
- [ ] `cargo xtask verify` green (list any N/A steps and the platform)
- [ ] ADR added/updated for architectural decisions; OPEN-QUESTIONS updated for open ones
- [ ] docs/HANDOFF.md, epic file and BACKLOG updated
