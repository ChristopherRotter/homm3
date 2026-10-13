# E08 design note: increment 2

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendments (E08 T11b; accepted at ADV-Q10, applied by #115). The task that builds a
section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-objects.md`. The versions increment 2 bumped are listed in `E08-log.md`
(T14b's second half, T15a).

## Border gates (T15b)

T15a built the keys (`AdventureState::has_key`, `keys`; a byte of player bits per colour in
`state/flags.rs`), the observatories and the guards with their offer (`OfferKind::Pass`,
`offer_stands`); `ObjectKind::effect` gives `None` for a gate, so a step onto one is still
`Visit`. Left for T15b: the gate entered and crossed like a free tile by a key holder's heroes
(it does not end a move) and visited from a neighbour by the others, so the visit mode becomes
per player in `visit_at`, `check_step` and the query together (inbox, T14 review; the query's
`Search` has the owner), and setup's refusal for a hero on a gate's tile (`StandError::Visited`
follows the type's mode today).
