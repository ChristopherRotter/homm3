# Docs list Q: the review's open questions, quirk candidates, experiment candidates and queued work

Worker name: `q`. Worktree: `/home/claude/wt-q`. Branch: `battle/e04b-t11b-q` (starts at the
joined branch `battle/e04b-t11b`, head `64fe5cf`).
`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.

The independent review of epic E04b (battle magic; a deterministic Heroes of Might and Magic III
rules engine in Rust) left 54 findings that the code fixes do not cover. The coordinator decided
where each goes. You write the registries so that nothing is lost when the epic closes. You change
**only** these files:

- `docs/OPEN-QUESTIONS.md` (only the block "Battle lane (`BAT-Q<n>`)" under "Open"),
- `docs/experiments/README.md` (the paragraph of candidates for the next batch),
- `docs/rules/surveys/E04b-magic.md` (a new last section for the review's quirk candidates),
- `docs/lanes/battle/QUEUE.md` (only the "Inbox").

No code, no other doc. The catalog, the quirks ledger, `docs/formats/installation.md` and the
address map are worker `d`'s (it runs at the same time in `/home/claude/wt-d`).

## Read first

- `docs/04-rules-and-sources.md` §2 ("Consensus before trust", the conflict rule) and §4
  ("Confirm before flagging"): they are why the entries below exist and what each must contain.
- `docs/OPEN-QUESTIONS.md`: the head and two answered battle entries (BAT-Q22, BAT-Q21) for the
  form of an entry; `docs/08-work-process.md`, "Keeping the start-of-session reading small" (every
  session reads the "Open" section, so it must stay short).
- `<SCRATCH>/review/DIGEST.md` in full: part 1 maps all 87 findings, part 2 describes the open
  ones, part 3 merges the "wiki checks wanted" and "not reviewed" lists, part 4 groups the
  experiments by set-up.
- For each entry you write, the finding's own section in `<SCRATCH>/review/<report>.md`: take
  addresses, numbers and scenarios from the report, not from the digest.

## The decisions (the coordinator's; do not change them)

The engine's behaviour stays as it is on every point below until the user decides or an
experiment settles it. Worker `d` records each reading in `docs/formats/installation.md` (in the §
of its topic), so an entry here cites "`installation.md` §7.x" by topic and stays short; it does
not repeat the address-level detail beyond the one or two deciding addresses.

### Open questions (new entries, in this order, as `#### BAT-Q<n> — title`)

- **BAT-Q28 — the review's readings against VCMI or the wiki (E04b T11b).** One entry with a
  table, one row per point: the reading (one clause, deciding address), the other sources (VCMI
  file; the wiki as far as the reports know it: they had no wiki access, so say "wiki not
  checked" where that is so), what the engine does today, the effect in play (one clause), the
  experiment row that would settle it. Then: options (keep today's behaviour until the experiment;
  follow the reading now), the recommendation (keep today's behaviour and stage one batch; the
  reading's side is never implemented alone, §2), what is blocked (nothing). Rows:
  1. F-r1-1: Chain Lightning searches the attacker's stacks first.
  2. F-r1-12 = F-r5a-3: a creature's area spell or chain takes the acting side's hero's damage
     bonuses.
  3. F-r2a-4 = F-r4b-strike-magic-4: the Defend bonus is fixed when the stack defends and sits
     inside the defense (Acid Breath tests and lowers it).
  4. F-r2b-stack-spells-2: the struck stack retaliates at a striker that the Fire Shield's
     reflection killed, and spends its retaliation (if confirmed, an original oddity: a quirk's
     class would be the user's).
  5. F-r3-control-battlefield-5: a two-hex walker leaving a pit along its row is stopped again.
  6. F-r4a-2: a dead stack's round state is frozen until it is raised and a new round starts.
  7. F-r4a-6 with F-r4a-9: the surrender price leaves out creatures raised for the battle only;
     it halves before the discount.
  8. F-r5b-mana-defences-4: in the wait phase every pending stack is ordered by the phase, and
     Wait is not offered (no comparison with VCMI or the wiki was made: say so).
  9. F-r5b-mana-defences-5: a bad-morale or Fear freeze does not count as its side's action in
     the tie-break.
  10. F-r4a-5: a hero's raise aimed at a two-hex corpse raises the top corpse on that corpse's
      position hex (the reading stands alone; an original oddity if confirmed).
  11. F-r1-6 and F-r6-limits-opening-2: the Orb of Vulnerability: which hero's Orb a chain's
      later candidate meets; the Unicorns' aura under the Orb (the user decided BAT-Q27 with the
      wiki, VCMI and batch 2e's 12 of 12; a second reading stands against it: re-test with a
      control).
  12. F-r2a-6: Dispel below Expert cannot be aimed at a stack whose only effect is Acid Breath
      (a useless cast is legal in the engine; a fix needs a spell flag in the pack format).
  13. F-r4b-strike-magic-5, F-r5b-mana-defences-6, F-r5a-5, F-r1-11, F-r2b-stack-spells-4,
      F-r6-limits-opening-3: points where the engine follows the code reading and VCMI says
      otherwise (the ten of F-r4b-strike-magic-5 and the others, one line each: reading / VCMI):
      these need no decision now; they are listed so that the conflict is on record and the wiki
      checks and statistics rows can settle them.
- **BAT-Q29 — BAT-QUIRK-9: Magic Mirror never redirects a creature's spell (F-r5a-1).** The
  reading and VCMI agree; the engine redirects in both profiles and the quirk's flag
  `quirk.battle.creature_mirror_untested` describes a behaviour the original does not have.
  Options: (a) only a hero's spell is reflected, the flag is retired (recommended: the reading
  carries the rule, VCMI agrees); (b) keep today's behaviour until the Faerie Dragon experiment.
  The user was asked on 2026-10-12 (no answer yet).
- **BAT-Q30 — BAT-QUIRK-2: the original offers no attack from a seen pit
  (F-r3-control-battlefield-4).** Options: (a) retire the flag, such attacks are illegal in every
  profile (recommended); (b) swap the defaults; (c) keep. The user was asked on 2026-10-12.
- **BAT-Q31 — a berserk stack's automatic turn follows its own side's hidden pits
  (F-r3-control-battlefield-11, F-sec-1 part a).** The original does the same by r3's reading; the
  security review did not read it. Options: (a) keep as the original and note it in the ledger
  beside BAT-QUIRK-4 (recommended; the hidden-obstacle property test leaves Berserk out and names
  this entry); (b) automatic turns plan with the pits both sides see (the first round's A1 added
  that reach for the automatic Ballista) and a pit a walk stops in is revealed. The user was asked
  on 2026-10-12.
- **BAT-Q32 — the Faerie Dragon's F-key path (F-r5a-2, F-r5a-4).** With F the drawn spell is aimed
  as a hero's: an area spell at any hex outside the side columns (VCMI aims at a location too);
  two corners are unsettled (the side's hero has cast this round; no hero). That path has no
  "magic allowed" test, so the Orb of Inhibition would not stop it; batch 2e's OI row tried only
  the mouse cursor (this reopens BAT-Q21 point 1). Recommendation: one Faerie Dragon session
  settles both, with BAT-Q28 row 2 and BAT-Q29's check.
- **BAT-Q33 — who commands a side: computer and neutral sides escape one bad-morale freeze in
  four (F-r5b-mana-defences-3).** The reading (two reviewers read the same bytes), the flag's
  meaning unsure, wiki and VCMI know no such rule. A seam question: the battle does not know
  whether a human commands a side. Options and what each touches, as the report gives them;
  recommendation: keep one roll for every side until an experiment of a few hundred activations;
  decide the seam with E05 or E08.

Each entry: at most about 1 500 bytes, except BAT-Q28 (at most about 6 000). The whole addition to
the "Open" section must stay below 14 000 bytes. Replace the block's "(no open questions)".

### Quirk candidates that wait for confirmation (the survey)

By §4 ("Confirm before flagging") an original oddity is implemented only after a second,
independent confirmation; until then it waits in its survey with its experiment. Add a last
section to `docs/rules/surveys/E04b-magic.md`, "The review's quirk candidates (E04b T11b,
2026-10-12)", with one short entry each (what the reading says with the deciding address, what
would confirm it, the suggested class, the engine's clean behaviour today), continuing the
survey's `S-` numbering (find the highest id in use across `docs/rules/surveys/`; the E04b survey
uses S-09 to S-21): the retaliation at a dead striker (BAT-Q28 row 4), the two-hex corpse
(BAT-Q28 row 10), the bad-morale escape (BAT-Q33), and from the reports' "Not reviewed, left
open": an obstacle removed early is removed again when its rounds run out (r3: a crash or a later
obstacle vanishing; exploit-fix if confirmed), the soft-lock of a stack created in the wait phase
and blinded before its turn (r5b; the engine does not have it).

### Experiment candidates (`docs/experiments/README.md`)

Append one paragraph to the candidates: "From the E04b review (T11b, 2026-10-12)", grouped by
set-up as the digest's part 4 does (one Faerie Dragon session; one row each; the statistics rows;
the Orb of Vulnerability rows), each with its question or survey id. Add the merged "Wiki checks
wanted" as one compact sentence list ("Wiki checks the review asks for (no wiki access in the
review): …") with the rule or question each belongs to. The candidates paragraph already asks
"whether the interface offers an attack from a seen pit" (BAT-QUIRK-2): say the review's reading
answers it (BAT-Q30) rather than adding it again.

### Queued work (`docs/lanes/battle/QUEUE.md`, "Inbox")

The inbox is a flat dated list; its entry "2026-10-11 (E04b T11a, the closure's triage): E04b's
loose ends by destination" has one sub-bullet per destination (E04c, E05, E08–E10, "Battle, no
task yet"). Add one entry in the same form right after it, "2026-10-12 (E04b T11b, the review):
**the review's leftovers by destination.**", with sub-bullets for the destinations that apply, and
in them, each point in one clause:

- the behaviour items of the digest's part 3 "Not reviewed, left open" that are not quirk
  candidates above (the Ammo Cart's test, the automatic First Aid Tent's choice of patient,
  Hypnotize ending in mid-turn, good morale rolled before the stack re-enters its hexes, no turn
  start in the wait phase, the `rebirth` creature killed on its way, an active Phoenix killed by
  its own hero's cast, a redirected Blind on the active stack, the larger of two Fire Shields,
  stage 1's hero step at rate 0, the rolls at a chance of 0 or 100);
- the items for E04c (sieges, Cursed Ground, Magic Plains, the anti-magic garrison);
- left over from covered findings (digest part 4): a bound on the `SpellDamage` rates in
  `hero::check`; a long `fixed` duration on a turn-stopping change; `action::encode` and
  `stack_spell` without a viewer (F-sec-2's "same root");
- for the server lane (it has its own queue, which you must not edit: one line here that says
  "for the server lane"): the wire has no query for a creature's cast targets; the sample scenario
  hosts `synthetic:faithful` with BAT-QUIRK-4 and BAT-QUIRK-2 on; hidden draws share the public
  rolls' stream (F-sec-5: a keyed stream for hosted games);
- "batch 2g: the review's readings (BAT-Q28 to BAT-Q33, the survey's new candidates)" as battle
  work with no task.

`cargo xtask check-docs` enforces size budgets: keep the additions short, and if a budget fails,
shorten your own lines (never raise a limit, never delete someone else's entry).

## How to work

- Checks, from your worktree, one cargo command at a time (worker `d` shares the target directory;
  a cargo call waits while the other runs):
  `export CARGO_TARGET_DIR=/home/claude/target-docs CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=1`
  then `cargo xtask check-docs` and `cargo xtask check-rules`. Both must pass before each commit.
  A build outlasts a tool call: start it with `nohup … > <log> 2>&1 &` and poll. Logs go to
  `<SCRATCH>/wq/`.
- Write for a reader who has not seen the review: full sentences, the terms the catalog uses, no
  finding ids alone without what they mean (give the id in brackets after the point). No
  disassembly listings (ADR-0009): own words and addresses.
- One commit per file or coherent group: message
  `battle: E04b T11b review — docs: <what>`, a body, and these two last lines exactly:
  ```
  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01YL987pakGrLu1G5DK369iw
  ```
  Push after each commit: `git push -q -u origin battle/e04b-t11b-q`.
- Keep `<SCRATCH>/review/docs-notes-q.md` current: what is done, and every finding or list item
  you could not place (with why).

Your final message: at most 15 lines: the four files with done / not done and commit ids, the
sizes of the additions, the state of the checks, anything you could not place.
