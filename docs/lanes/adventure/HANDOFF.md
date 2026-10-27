# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–147, ADV-Q26 to ADV-Q30): T54 to T58b done (the battle seam's facts; survey `E08-rewards.md`; the drafts and design `E08-design.md`, eight rounds, decision 139; batch E7 scored, A-23 withdrawn; A-25 a quirk with a flag, decision 145); T59 done (#497, `shared/`: the ADRs amended, `HeroParams::mana_ceiling`, flags 37 and 38, the glossary); **T60 done** in two (decision 147): T60a (#500: `HeroDetails::bonuses`, the four changes, HER-ADV-008, game state hash v15), **T60b** (`adventure/e08-t60b`: `grant`, `can_receive_artifact` under flag 38, the teachers; HER-CHG-003, -004; no version). Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** T60b's `adventure/e08-t60b` (see the newest session).
- **Next:** merge T60b's pull request; then T61 (design §7 to §10: the import, kinds, draws and state; its cut confirmed at its start).
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v15 (T60a), adventure state hash v12 (T54), adventure ruleset part v11 (T50), map import v11 (`MAP_IMPORT_VERSION`, T54), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; next 23), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; `HER-CHG-003`, `-004` built by T60b; increment 7's planned in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 26; A-23 withdrawn); decisions next 148. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-27 — increment 7, round 4: T60b, `grant` and the teachers

Baseline c637652 (#500 merged as 40151d3, CI green on it; the sandbox's light gates passed). T60b whole (decision 147; no split needed). `h3-heroes`: `reward.rs` (`grant`, `HeroReward`, `Granted`, `can_receive_artifact` under flag 38), `teach.rs` (the teachers, `SlotState`, `ScholarGift`), `can_learn_spell` beside `learns_spell` (which delegates; `h3-game` untouched); HER-CHG-003, HER-CHG-004 (new id); `ADV-QUIRK-22`'s test in both profiles; T61 queued.

### 2026-10-27 — increment 7, round 4: T60a, the battle bonuses and the four changes

Baseline 46f7d62 (#497, #498 merged; CI green on 1c99f41; the sandbox's light gates passed; the user's repo folder fast-forwarded, one stale lock of the session's own fetch removed). The user's cut: T60 in two, T60a now (decision 147), the other lanes' `HeroDetails` literals one line each (`cross-lane`, consented). `BattleBonuses` on `HeroDetails`; `GainPrimary`, `RaiseSkill`, `Bonus`, `ClearBonuses` (`ChangeError::SkillLevel`, `Mark`); the bonuses in the own side's morale and luck (HER-ADV-008); game state hash v15; the twelve replays re-recorded, their commands unchanged; the battle and AI lanes' inboxes; T60b queued.

### 2026-10-27 — increment 7, round 3: T59, the shared part

Baseline 18f3cb1 (#496 merged; CI on `master` green; the sandbox's light gates passed; one stale lock in the user's `.git` removed). The user's cut: T59 whole, one `shared/` pull request (decision 146). The three drafts applied to ADR-0013, -0016 and -0017, A-25's flag written into ADR-0013 point 15.4 and ADR-0016 point 4 (the drafts predated decision 145); `HeroParams::mana_ceiling` (pack format v48, core ruleset hash v49), the harvester's `exe-mana-ceiling`; `ADV-QUIRK-21`, `-22` (flags 37, 38) in the ledger; the glossary; the 22 replays' headers and the AI lane's bots' tables; the epic file trimmed (the done tasks' cuts to the archive); T60 queued.
