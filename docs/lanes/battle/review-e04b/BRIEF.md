# E04b independent review (task T11b): common brief for the rule reviewers

You are one of several independent reviewers. The repository `/home/claude/homm3` is a headless,
deterministic rules engine for Heroes of Might and Magic III (Rust). Epic E04b added battle magic:
69 rules `BAT-MAG-001..069`, some `HER-*` rules and 26 quirks. The epic closes after this review.
You did not write any of it. Your value is a fresh, sceptical read: the implementers' summaries,
notes and tests may be wrong, and nobody has checked them independently yet. Each reviewer has a
slice of rules; yours is in your task prompt.

## What reviewing a rule means

For every rule in your slice:

1. **Read the rule.** `python3 <SCRATCH>/review/rules.py BAT-MAG-009 BAT-MAG-017` prints entries
   (summary, quirks, every source). Do not read `docs/rules/catalog/battle.toml` whole (310 KB).
2. **Rule text against the code.** Find the code with `grep -rn "<ID>" crates --include=*.rs`
   (`// rule: <ID>` marks implementing code, `// covers: <ID>` marks tests). Read the implementing
   functions completely and follow their calls; do not stop at the marked line. Check every clause
   of the summary: is it implemented as stated? Does the code do something with a gameplay effect
   that the summary does not say?
3. **Rule and code against the sources.**
   - *Code readings of the original executable* (`kind = "original-data"`, "Heroes3.exe code
     reading") are recorded in `docs/formats/installation.md` §7.8–§7.24 (line ranges in your task
     prompt; read only your sections, in slices). Check note against summary against Rust code.
   - *Re-read the executable yourself* where a mistake is most likely or most costly: the order of
     checks, which stack, side or hero a test reads, boundaries (`>` or `>=`, off-by-one in
     durations and counts), rounding and truncation, signed against unsigned compares, float
     constants' width. The tool: `python3 scripts/exe.py lookup|fn|switch|callers|refs|grep` (read
     `docs/exe/README.md`, "Reading code" and "Confidence and reliability", first; `fn` prints a
     long function only in slices: `--around`, `--from/--to`). Never read `docs/exe/map.toml` or
     `docs/exe/h3api.toml` whole. You cannot re-read everything: put rules first whose status is
     `implemented` with no `observation` source (they rest on one reading alone), and every quirk in
     your slice (a quirk needs a second, independent look at the deciding paths before it may keep
     its flag: `docs/04-rules-and-sources.md` §4, "Confirm before flagging"). Say in your report
     which addresses you re-read.
   - *VCMI 1.7.5* is cloned at `/home/claude/vcmi`. Where a rule leans on VCMI ("VCMI agrees", or
     VCMI is the only agreeing source for a reading: `docs/04-rules-and-sources.md` §2, "Consensus
     before trust"), open the cited file and check that VCMI really does what the entry says.
   - *Observations*: `experiments/results/E04b-batch-2a.md` … `2f.md`. For a `verified` rule, check
     that the cited row really shows what the rule claims, and that a test reproduces it with the
     same numbers.
   - The wiki's statements are quoted in the survey `docs/rules/surveys/E04b-magic.md` and in the
     catalog sources. You have no wiki access (see below).
4. **Tests.** Do the tests pin each clause that matters in play, including its boundary? Name the
   gaps that matter; do not list every untested line (coverage is at 100 % by gate).
5. **On the way**, note robustness problems (a panic on reachable input: indexing, `unwrap`,
   arithmetic overflow with extreme counts or power — overflow checks are on in every profile; a
   loop without a bound) and security problems (a command trusting payload data; hidden information
   reaching the other side through an event, an error or the legal commands) in your rules' code.

Each quirk in your slice (`docs/rules/quirks-ledger.md`; find its entry by id): the original's
behaviour is confirmed as described (by which source: experiment, re-read, VCMI), both variants of
the flag are implemented and each has a test that fails for the other variant, and the class
(`preserve`, `fix-optional`, `exploit-fix`) fits `docs/04-rules-and-sources.md` §4.

## Rules for working

- **Do not change any file in the repository.** Do not commit. Your only output is your report.
- **Do not run `cargo` or any build.** The machine has two cores and the build directory is in
  use by the verify. If a claim needs a test run to settle, write the test as a sketch into the
  finding and mark it `needs-run`; the coordinator runs it.
- **No WebFetch or WebSearch** (the wiki needs the user's approval for each session). List wiki
  checks you want under "Wiki checks wanted".
- **No sub-agents.** No disassembly listings in your report (licence rule, ADR-0009): describe the
  original's code in your own words with addresses.
- Read big files in slices. Use `grep -n` to find, then read ranges.
- Helpful: the lane's traps in `docs/lanes/battle/HANDOFF.md` ("Standing traps"), the epic
  `docs/epics/E04b-battle-magic.md`, and each task's record in `docs/epics/E04b-log.md` (181 KB:
  grep for your rule ids, read only those records). The seam types are in `crates/h3-contracts`,
  content in `crates/h3-content`, the Heroes context in `crates/h3-heroes`, the battle in
  `crates/h3-battle` (tests in `src/tests/`), packs in `packs/` (generated; the synthetic test pack
  is in `crates/h3-battle/src/tests/support.rs`).

## What counts as a finding

A finding is a concrete, checkable claim, one of:

- `behaviour-bug`: the code does not do what the best source says (give the scenario);
- `rule-text`: the summary is wrong, stale or leaves out something the code does that matters;
- `source`: a source is misquoted, mislabelled, does not support the claim, or a "VCMI agrees" is
  untrue; a reading that rests on nothing but itself where the rules of `docs/04` ask for more;
- `test-gap`: a clause with gameplay effect that no test pins;
- `robustness`, `security`: as above;
- `question`: the sources conflict with real gameplay effect and you cannot settle it.

A rule with no finding is a normal result; say so. Do not report style, naming or wording taste.
Do not report anything you have not traced in the code yourself. If summary and code differ, say
which one the source supports. If you are unsure, say so and name the check that would settle it.
The coordinator verifies every finding before acting on it, so exact evidence matters more than
the number of findings. Do not soften or drop a finding because it is inconvenient.

## Your report

Write it to `<SCRATCH>/review/<your name>.md` (the name is in your task prompt), in this form:

```
# Review <name>: <slice>

## Coverage
| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
(one line per rule and quirk; verdict: ok | F-<n> | partly checked: <what is left>)

## Findings
### F-<name>-<n>: <the claim in one line>
- rules: …
- kind: behaviour-bug | rule-text | source | test-gap | robustness | security | question
- severity: high (wrong outcome in normal play) | medium (an edge case or a rare setup) | low (docs or tests only)
- confidence: high | medium | low, and what would raise it
- evidence: file:line references; what the source says (section, address, VCMI file:line, batch
  row); a concrete scenario: setup → what the source says must happen → what the code does
- fix or test sketch: short

## Wiki checks wanted
## Not reviewed, left open
```

Your final message to the coordinator: at most 12 lines. How many rules and quirks you checked
fully, the findings by severity with their one-line claims, and anything you could not do.

`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.
