# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed**. **Increment 7: rewards on the map** (decisions 121–148, ADV-Q26 to ADV-Q30): T54 to T58b done (the battle seam's facts; survey `E08-rewards.md`; the drafts and design `E08-design.md`; batch E7 scored, A-23 withdrawn; A-25 a quirk with a flag); T59 done (#497, `shared/`); T60 done (T60a #500, T60b #504); **T61 in two** (decision 148): **T61a** (`adventure/e08-t61a`: the twelve kinds in `adventure.toml`, the 31 classes imported with what they hold, `Holds`, `RewardRecord`, the guards of artifacts, scrolls, boxes and events, the monster's artifact, A-24 refused), then T61b. Records: `E08-log.md`; increment 6's in `E08-archive.md`.
- **Open pull requests:** T61a's `adventure/e08-t61a` (see the newest session).
- **Next:** merge T61a's pull request; then T61b (the bans, GAM-SET-003, with the AI lane's two `GameSetup` literals `cross-lane`; the draws, design §9; the state, §10): first `--with-original-data` with `maps.tar` staged, since T61a's import was not run against the shipped maps.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v15 (T60a), adventure state hash v13 (T61a), adventure ruleset part v12 (T61a), map import v12 (`MAP_IMPORT_VERSION`, T61a), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 7 (last: T59, with ADR-0013).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 31), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired; 21, 22 A-21, A-25, flags 37, 38; next 23), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`; `HER-CHG-003`, `-004` built by T60b; increment 7's planned in survey §12 and design §17); the E08 surveys' disputes `D-<n>` (next 41) and reading quirk candidates `A-<n>` (next 26; A-23 withdrawn); decisions next 149. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-27 — increment 7, round 5: T61a, the import and the kinds

Baseline 94a5039 (#504 merged, CI green; the sandbox's light gates passed). The user's cut: T61 in two as designed (decision 148). `h3-adventure`: `ruleset/rewards.rs` and twelve kinds (ruleset part v12), `Holds`/`RewardRecord`/`EventSetup` on the setup, `state/holds.rs` (`SetupError::Holds`), guards on more kinds, `Monster::artifact` (state hash v13); `h3-map`: the 31 classes and their bodies (import v12, A-24); `h3_heroes::validate_artifact`; the twelve replays re-recorded; the AI lane's inbox (the census's guarded piles); `cross-lane` (consented): three default lines in the AI lane's `h3-sim` adventure tests. Not run: `--with-original-data` (the user's computer was not reachable); the bans moved to T61b.

### 2026-10-27 — increment 7, round 4: T60b, `grant` and the teachers

Baseline c637652 (#500 merged as 40151d3, CI green on it; the sandbox's light gates passed). T60b whole (decision 147; no split needed). `h3-heroes`: `reward.rs` (`grant`, `HeroReward`, `Granted`, `can_receive_artifact` under flag 38), `teach.rs` (the teachers, `SlotState`, `ScholarGift`), `can_learn_spell` beside `learns_spell` (which delegates; `h3-game` untouched); HER-CHG-003, HER-CHG-004 (new id); `ADV-QUIRK-22`'s test in both profiles; T61 queued.

### 2026-10-27 — increment 7, round 4: T60a, the battle bonuses and the four changes

Baseline 46f7d62 (#497, #498 merged; CI green on 1c99f41; the sandbox's light gates passed; the user's repo folder fast-forwarded, one stale lock of the session's own fetch removed). The user's cut: T60 in two, T60a now (decision 147), the other lanes' `HeroDetails` literals one line each (`cross-lane`, consented). `BattleBonuses` on `HeroDetails`; `GainPrimary`, `RaiseSkill`, `Bonus`, `ClearBonuses` (`ChangeError::SkillLevel`, `Mark`); the bonuses in the own side's morale and luck (HER-ADV-008); game state hash v15; the twelve replays re-recorded, their commands unchanged; the battle and AI lanes' inboxes; T60b queued.
