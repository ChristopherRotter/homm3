# Town lane — QUEUE archive

Inbox notes that left the queue, verbatim, with where their work went (`docs/08-work-process.md`,
"Keeping the start-of-session reading small"). Read only when a doc points here.

## 2026-11-02 — E09 T2b: carried by the design note `docs/epics/E09-design.md`

Each note's work is a section of the design note and a task of the cut (decision 22): the main
town's hero and a hero in a town §7, §14 (T7a, T7b); the town classes typed §8 (T8a); the income
§6, §16 (T6, T8c); ADR-0023's derivation and the experiment maps' towns §6, §15, §20 (T3a, T6,
T8b; the moat by the packs, decisions 14 and 20); the attacker's Catapult settled by ADR-0023
point 6 (§15); Conflux and the moat by expansion by decisions 14 and 20, Conflux's record in the
AB pack (§3, T5b).

- (adventure archive, 2026-10-10, T5a/T6a) A hero placed on a town's blocked tile is the town's
  visiting hero (VCMI `CGameState::placeHeroesInTowns`), and heroes generated at a main town (129
  maps then, 128 now): both `Unsupported` until towns exist. Increment 1.
- (adventure archive, 2026-10-13, T14a) Untyped blockers have no from-above setting, so a hero on
  a town's gate leaves upward: type the town classes. Increment 1.
- (adventure archive, 2026-10-30, T77a) `town_gold_income` 0x5BF600 is a town's income (read at
  T1b2a: survey `E09-towns.md` §11). Increment 1.
- (ADR-0023, Consequences) The town context's derivation of `Siege` gets a rule list from survey
  `E04c-sieges` §1; the game's version (the moat) follows the enabled packs (decision 14, E08
  decision 104; no experiment); experiment maps need towns (the writer has towns of a fixed
  faction). Increment 1.
- (adventure archive, 2026-10-14, T18b) VCMI gives every hero a catapult; the original was not
  read; it matters in sieges. Increment 1 (the siege's war machines).
- (E11, Q58) Conflux as a town and the moat by expansion are E09's.
