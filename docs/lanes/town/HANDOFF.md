# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–160, `TWN-Q15` to `-Q29`): T31 to T40 done
  (T40b whole, decision 159: the plans' map Taverns and Dens with the oracles taught the hire,
  the taverns map `e09-taverns-36.h3m` with six scripted replays, the golden counts 104, the
  `week-start-taverns-36` bench, the guild's query timed). **Next:** T41 (the audit against the
  twelve exit criteria, increment 4's closure with the README's row in a `shared/` pull request,
  increment 5 refined; `QUEUE.md`). The guild and the services are increment 5, travel and the
  Grail 6. Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6`, `-13` and
  `-15` (increment 5), `TWN-STUB-2` (the victory conditions), `TWN-STUB-16` (E10's loss
  condition), `-17` (increment 6), `-18` (E13).
- **Open pull requests:** T40b's (#1053, `town/e09-t40b`, `cross-lane`), opened at the session's end;
  watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v8: T39b1), both in `h3-towns`; the adventure lane's
  (game state hash v34, the adventure part v22, map import v26 and replay format v19, all by
  T39a; the part v23 since E08 T132a2; adventure state hash v26 by E08's T124b1), the env's (v13 by T39a) and the protocol's
  (v74 by T39b2) are bumped `cross-lane` (each task's record in `E09-log.md`; the
  plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24),
  quirk flags 43 and `StreamId::TAVERNS` (8) by T35b.
- **Ids** (next): questions `TWN-Q30`; decisions 161; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b; their tests by T38b1, T39b1); stubs `TWN-STUB-21` (`-11`, `-12` planned by §34; `-16`
  by T38b2, `-17`, `-18` by T39b1; `-20` lifted by T39b2); rules: TWN-TAV-001 to -009, GAM-MAN-003,
  GAM-DEC-010 and the adventure's ADV-VIS-007, -008 built; experiment batch `TWN-5`; disputes `TD-25`; quirk candidates `T-18`.
  ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-19 — T40b: the taverns map and its replays, the plans' map Tavern and Den

Baseline dc92305 (#1048 merged; master's code verified by CI, the setup's light gates passed).
The user's computer was connected at the start: 63 files of `original-data/` and the four
manuals with three help files staged, SHA-256 checked. The user kept T40b whole and consented
to `cross-lane` (decisions 159, 160, `TWN-Q29`). Built: design §78's As built (T40b): the plans'
`Plan::taverns` with `tavern_answers`, `check_map_hire` and the decision in `check_decisions`
and `managed`; `e09-taverns-36.h3m` and six `taverns-*-s1` replays with their tests; the golden
counts 104; `week-start-taverns-36`. PR #1053.

### 2026-11-19 — T40a: the census of the taverns, the pool's properties

Baseline 38d422b (#1040 merged; CI's run on `master` standing in: the VM restarted at once and
the setup's baseline died). The user's computer was connected at the start only: the client
lane's 64 files and the four manuals with three help files staged, SHA-256 checked; its tools
went away minutes in (staging had finished). The user cut T40 in two, the census first, consented to
`cross-lane` and chose public read-only readers of the pool (decisions 156 to 158, `TWN-Q28`).
Built: design §78's As built (T40a). PR #1048.

### 2026-11-19 — T39b2: the Den of Thieves' visit, the guild on the wire

Baseline 8930a1b (#1034 merged; CI's run on `master` standing in: a VM restart killed the setup's
baseline mid-build). The user's computer was connected at the start only: 64 files (the client
lane's list) and the four manuals with three help files staged, SHA-256 checked. The user kept
T39b2 whole (decision 154, `TWN-Q27`, as recommended). Built: design §74's and §75's As built
(`AdventureOutcome::thieves_guild`; `town::thieves::tell_den`, `ThievesGuildSeen` and
`GuildPartSeen` per `GuildPart`; protocol v74: `GameQuery::ThievesGuild`, the wire's guild types
and `ThievesGuild::see`, the events mapped and not folded); the census: no golden map lays out a
Den. ADR-0017 point 2's wording by `shared/e09-t39b2` (#1039).
