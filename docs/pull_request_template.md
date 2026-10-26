<!-- Definition of Done (docs/05-testing.md). Tick every box or say why it does not apply. -->

## What and why

## Checklist

- [ ] Behavior implemented in the correct bounded context, in the ubiquitous language
- [ ] Rule catalog updated (new/changed rules with labelled sources); code has `rule:` markers, tests have `covers:` markers
- [ ] Unit tests + relevant property/replay tests; 100% coverage for touched logic code
- [ ] Quirks considered: known original deviations have a ledger entry and a flag where applicable
- [ ] Security considered: commands validated against actor and state; no hidden info leaked (docs/06-security.md)
- [ ] Performance considered: no allocation in hot paths; `cargo xtask bench-check` passes or the baseline update is explained
- [ ] CI's `cargo xtask verify` green on the pull request; locally the scoped verify, with `--with-original-data` where it applies (list any N/A steps; during an outage of CI: the full local verify's result, commit and date)
- [ ] ADR added/updated for architectural decisions; OPEN-QUESTIONS updated for open ones
- [ ] Lane: `<lane>` (branch `<lane>/<task>`); `check-lane` passes, or the PR is labelled `cross-lane` with the user's consent
- [ ] Lane HANDOFF, epic file and lane QUEUE updated (BACKLOG only when an epic starts or ends)
