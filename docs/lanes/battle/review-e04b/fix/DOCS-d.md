# Docs list D: the rule texts after the review (catalog, quirks ledger, installation.md, the address map)

Worker name: `d`. Worktree: `/home/claude/wt-d`. Branch: `battle/e04b-t11b-d` (starts at the
joined branch `battle/e04b-t11b`, head `64fe5cf`, which has all 21 fixes of the first round).
`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.

The independent review of epic E04b (battle magic; a deterministic Heroes of Might and Magic III
rules engine in Rust) is done; its code fixes are in. The fix workers were not allowed to touch
`docs/`. You bring the rule documentation in line with the code and with what the review found.
You change **only** these files:

- `docs/rules/catalog/*.toml` (the rule catalog; `README.md` there explains the fields),
- `docs/rules/quirks-ledger.md`,
- `docs/formats/installation.md` (§7: the code reading of the executable),
- `docs/exe/map.toml` and `docs/exe/structs.toml` (the address map; `docs/exe/README.md`).

No code, no other doc. `docs/OPEN-QUESTIONS.md`, `docs/experiments/README.md`, the surveys and the
lane's `QUEUE.md` are worker `q`'s (it runs at the same time in `/home/claude/wt-q`); the epic
files, the log and the HANDOFF are the coordinator's.

## Read first

- `AGENTS.md` ("Non-negotiables": rule ids, sources, license hygiene) and
  `docs/04-rules-and-sources.md` (§2 the sources and "Consensus before trust", §3 the catalog,
  §4 quirks). `docs/exe/README.md` for how a code reading is recorded: our own words and numbers
  with addresses, **never a disassembly listing or decompiled code** (ADR-0009).
- `<SCRATCH>/review/DIGEST.md`: the map of all 87 findings (part 1: which fix item covers which;
  part 2: the open ones; part 4: cross-report notes). Read it once in full.
- The fix workers' notes: `<SCRATCH>/review/fix-notes-a.md` (from "Rule texts, ledger entries and
  `installation.md` sentences that are now wrong (A1 to A11)" to the end) and
  `<SCRATCH>/review/fix-notes-b.md` (from "Rule texts, ledger entries and `installation.md` that
  are now wrong, with the wording proposed" to the end, including "The address map"). Each
  proposal gives the rule id, the old clause and the new clause.
- For each finding you work on, its section in the report (`<SCRATCH>/review/<report>.md`).

## Part 1: the 21 fixes' texts (from the fix notes)

Apply every wording proposal of both notes files: rule summaries and sources in the catalog,
ledger entries, `installation.md` sentences, and the map's summaries (including the new map
entries the notes ask for, e.g. 0x44B2D0 and 0x44AE60 for B4, and the stale summaries they list,
e.g. 0x442880). The proposals were written by the people who changed the code, but check each
against the code in your worktree before you apply it (the functions are named in the notes): if
a proposal and the code disagree, the code wins and you write the clause yourself; list such cases
in your notes. State each rule as it is now, in the present tense; put no history into a summary
("was", "since the review", "no longer"): history goes to the epic's log, which the coordinator
writes from your notes.

Decisions of the coordinator that the notes left open: the Harpies' return (B9) is part of the
strike's action, so the battle's end is tested once after the return (say so in the rule); a
berserk candidate of the automatic Ballista stays "any stack" (A10); without a casting hero there
is no caster stage in the spell damage (B6).

## Part 2: the review's doc findings

Each is a `DOC` entry in the digest's part 2; the report section has the exact text. Do all of:
F-r1-7, F-r1-8, F-r1-9, F-r1-11, F-r2a-5, F-r2b-stack-spells-4, F-r2b-stack-spells-5,
F-r3-control-battlefield-9 (the texts: the ledger's "intended behaviour" of BAT-QUIRK-1, §7.16,
the VCMI sources of BAT-MAG-031 and BAT-MAG-026), F-r4a-7, F-r4b-strike-magic-7,
F-r4b-strike-magic-5 (the VCMI sources on BAT-MAG-040 to 047, each saying where VCMI differs from
the reading the engine follows), F-r5a-5, F-r5b-mana-defences-6, F-r6-limits-opening-3,
F-r6-limits-opening-4, F-r6-limits-opening-5 (the three text lines only), and the doc parts of
covered findings that the digest's part 4 lists under "Left over from covered findings".

Rules for sources:
- A VCMI source you add or correct must be checked in `/home/claude/vcmi` (VCMI 1.7.5): open the
  file, find the code, and write what it does. `url` in the form the catalog uses for 1.7.5,
  `retrieved = "2026-10-12"`. If the reviewer's citation does not hold, write what you found and
  note it.
- There is no wiki access in this session. Add or change no wiki source; where a finding asks
  for a wiki check, leave the rule's wiki sources as they are (worker `q` records the check as
  wanted).
- Where VCMI contradicts the reading a rule rests on, the VCMI source says so plainly ("VCMI: …;
  not followed, the code reading decides: BAT-Q28") and keeps `kind = "vcmi"`.
- `status` fields: change none. If a fix or a finding makes a rule's `verified` status doubtful
  (the observation no longer covers what the summary says), list the rule in your notes.

## Part 3: the readings the engine does not follow yet

The review re-read the executable on points where the engine still does something else, because
the reading stands against VCMI or the wiki, or needs the user's decision. The engine is unchanged
on these; worker `q` writes the open questions. So that the readings are not lost (the reports are
not merged), record each in `docs/formats/installation.md`, in the § that already describes that
routine or topic (find it; do not start a new § unless none fits), as a short dated paragraph
"Review re-read (E04b T11b, 2026-10-12)": what the code does, with the addresses the report gives,
in our own words; what the report did not read or marks as unsure; and the closing clause "the
engine does not follow this yet: BAT-Q<n>". Add map entries for routines the paragraph names that
the map lacks. The points and their question ids:

| finding | question |
|---|---|
| F-r1-1 (Chain Lightning searches the attacker's stacks first) | BAT-Q28 |
| F-r1-6 (the chain tests later candidates as cast by the opposite side) | BAT-Q28 |
| F-r1-12 = F-r5a-3 (a creature's area spell or chain takes the acting side's hero's damage bonuses) | BAT-Q28 |
| F-r2a-4 = F-r4b-strike-magic-4 (the Defend bonus is fixed at Defend, inside the defense field) | BAT-Q28 |
| F-r2a-6 (Dispel's aim test wants a duration above 0) | BAT-Q28 |
| F-r2b-stack-spells-2 (the retaliation at a striker the Fire Shield's reflection killed) | BAT-Q28 |
| F-r3-control-battlefield-5 (a two-hex walker leaving a pit along its row is stopped again) | BAT-Q28 |
| F-r4a-2 (a dead stack's round state is frozen until it is raised and a new round starts) | BAT-Q28 |
| F-r4a-5 (a raise aimed at a two-hex corpse raises the top corpse on its position hex) | BAT-Q28 |
| F-r4a-6 and F-r4a-9 (the surrender price: creatures raised for the battle only; halving first) | BAT-Q28 |
| F-r5b-mana-defences-4 (the wait phase orders every pending stack by the phase) | BAT-Q28 |
| F-r5b-mana-defences-5 (a freeze does not count as its side's action) | BAT-Q28 |
| F-r6-limits-opening-2 (the Unicorns' aura under the Orb of Vulnerability; the user decided BAT-Q27 with the wiki, VCMI and batch 2e) | BAT-Q28 |
| F-r5a-1 (Magic Mirror never redirects a creature's spell) | BAT-Q29 |
| F-r3-control-battlefield-4 (no attack is offered from a seen pit) | BAT-Q30 |
| F-r3-control-battlefield-11 = F-sec-1 part a (a berserk stack's turn follows its own side's hidden pits) | BAT-Q31 |
| F-r5a-2 and F-r5a-4 (the Faerie Dragon's F-key path: any hex for an area spell; no "magic allowed" test) | BAT-Q32 |
| F-r5b-mana-defences-3 (a side no human commands escapes one bad-morale freeze in four) | BAT-Q33 |

For BAT-QUIRK-9 and BAT-QUIRK-2 in the ledger: the user's decision on them is pending, so do not
rewrite or retire the entries; add one dated line to each ("Review (E04b T11b, 2026-10-12): the
re-read finds …; open as BAT-Q29" / "BAT-Q30") and correct `installation.md` §7.21's and the
map's statement that the redirect reaches a creature's cast (it is a statement about the
executable that the re-read refutes; say what the re-read found and that the engine still
redirects: BAT-Q29). Likewise add a ledger line beside BAT-QUIRK-4 for the berserk walker
(BAT-Q31), and the two original bugs the engine leaves out that F-r4b-strike-magic-7 misses in the
ledger (the bound Harpy, BAT-Q9; the binder on a stack its strike killed, BAT-Q8), in the ledger's
form for bugs that are never reproduced.

Worker `c` is fixing three more points in code right now (r1 F-3: a spell without a target that
would affect nobody is refused; r3 F-8: pit and mine draws avoid only living stacks and corpses;
r4a F-4: an expired created stack's record is not reused). Leave the texts of those three to the
coordinator.

## How to work

- The catalog's summaries are long single-line TOML strings: edit the clause, keep the rest of the
  line byte for byte. Use exact-string edits; never rewrite a whole summary from memory.
- Checks, from your worktree, one cargo command at a time (two code workers build on this
  machine; worker `q` shares your target directory and its cargo calls wait for yours):
  `export CARGO_TARGET_DIR=/home/claude/target-docs CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=1`
  then `cargo xtask check-rules`, `cargo xtask check-docs`, `cargo xtask check-exe-map`. All three
  must pass before each commit. A build outlasts a tool call: start it with
  `nohup … > <log> 2>&1 &` and poll. Logs go to `<SCRATCH>/wd/`.
- Commit in coherent groups (for example per fix list, per report): message
  `battle: E04b T11b review — docs: <what> (<finding ids or fix items>)`, a body that says which
  files and rules, and these two last lines exactly:
  ```
  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01YL987pakGrLu1G5DK369iw
  ```
  Push after each commit: `git push -q -u origin battle/e04b-t11b-d`.
- Keep `<SCRATCH>/review/docs-notes-d.md` current as you go (a restart must lose little): per part,
  what is done; proposals you changed and why; VCMI citations that did not hold; rules whose
  `verified` status is now doubtful; anything you could not place.

Your final message: at most 20 lines: the three parts with done / not done and commit ids, the
state of the three checks, the open points for the coordinator.
