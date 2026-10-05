# E04b log: done tasks in full

The full record of E04b's finished tasks (what was built, refinements, agent decisions marked
"change freely", measurements). **Not start-of-session reading:** the epic file keeps one line per
done task; read the matching section here when a task builds on an earlier one or a decision is
questioned. Rules live in the catalog, quirks in the ledger, decisions in ADR-0014; this log only
records how and why the tasks went.

## Done tasks

### T1 — Magic survey and ADR-0014 (2026-10-05, one session with the refinement)

- **Survey** `docs/rules/surveys/E04b-magic.md`, just in time (Q41): casting by a hero, effect kinds
  and targeting, durations and end conditions, counters, the resolution order, stacks created in
  battle, hidden obstacles, creature magic, quirks S-09 to S-21 by task, disputes M-1 to M-6,
  batch 2a candidates X-1 to X-6. Sources: VCMI 1.7.5 (cloned in the sandbox), the wiki (fetched
  with the user's approval through a summarising tool; it quotes at most about 125 characters and
  several pages are thin, so resistance stacking and the duration countdown stay open), the RoE
  manual pages on the wiki, original data.
- **Original data:** the spell table's flag word decoded by matching the 81 values against known
  behaviour and VCMI (`docs/formats/installation.md` §7.6; bit `0x1000` unclear, dispute M-1); the
  AB and SoD `SPTRAITS.TXT` differ only in row 57. Code reading for Q42 (§7.7): 21 stack records
  per side, Clone and Summon Elemental refused at 20 stacks, the elemental-type rule. Tools:
  `capstone` and `pefile` in the sandbox, `skipdata` sweep of `.text` (HANDOFF trap).
- **ADR-0014** proposed, revised once and accepted (Q42); ADR-0004 point 8 amended (growable lists
  in battle). Design note `docs/epics/E04b-design.md` with proposals for T2a–T2d, T3, T5, T6.
- **User decisions:** a hero's cast is optional, at most once per round if permitted; a counter
  spell replaces its pair's effect, the later wins; timed effects in one growable list per battle;
  21 slots per side with the limit of 20; commands may resolve unexpectedly for the limited
  observer (Q42).
- **Agent decisions (change freely):** rule area `BAT-MAG`; the confidence labels of the decoded
  flags; the candidate list for batch 2a; the proposals in the design note (sizes, names).
