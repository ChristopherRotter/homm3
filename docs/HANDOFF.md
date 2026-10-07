# HANDOFF

Every session reads this file first, then its lane's `docs/lanes/<lane>/HANDOFF.md` (state, lane traps, the last three sessions) and the lane's reading list in `AGENTS.md`. This file is the lane index plus the traps of every lane. Rules: `docs/08-work-process.md` ("Lanes", "Keeping the start-of-session reading small"); `cargo xtask check-docs` keeps it under 6 000 bytes. It is a shared path: change it in a `shared/` pull request.

## Lanes

| Lane | Owns (`lanes.toml`) | State | Files |
|---|---|---|---|
| battle | `h3-battle`, `h3-heroes`, `h3-content`, `h3-pack`, `h3-sim`, `h3-ffi`, `h3-cli`, the formats/tools crates, packs, fixtures, experiments | active: E04b increment 3 | [HANDOFF](lanes/battle/HANDOFF.md), [QUEUE](lanes/battle/QUEUE.md) |
| adventure | `h3-adventure` (new) | refinement session next (E07/E08) | [HANDOFF](lanes/adventure/HANDOFF.md), [QUEUE](lanes/adventure/QUEUE.md) |
| server | `h3-protocol`, `h3-server` (new) | refinement session next (E16) | [HANDOFF](lanes/server/HANDOFF.md), [QUEUE](lanes/server/QUEUE.md) |
| client | `clients/` (own Cargo workspace) | waits for the user's asset notes | [HANDOFF](lanes/client/HANDOFF.md), [QUEUE](lanes/client/QUEUE.md) |

Shared (own pull request, merged first): `h3-kernel`, `h3-contracts`, workspace and gate files (`Cargo.toml`, `arch.toml`, `lanes.toml`, `xtask`, CI), ADRs, process docs, `BACKLOG.md`, this file. Common (any lane): `OPEN-QUESTIONS.md`, `docs/stubs.toml`.

- **GitHub and CI:** `ChristopherRotter/homm3` (private). `.github/workflows/ci.yml` runs the full `cargo xtask verify` on Linux for pull requests, `master` and weekly; `lanes.yml` runs `cargo xtask check-lane` on pull requests. Changes reach `master` by pull request only.
- **Phase 0** (2026-10-07, `shared/lanes-phase-0`): lanes, `check-lane`, `verify --scope`/`--summary`, per-workload baselines, lane docs. The battle lane rebases onto it at its next checkpoint: battle's HANDOFF now lives in `docs/lanes/battle/`.

## Traps for every lane

**Sandbox and sync**
- Original files in a cloud session: stage them from the user's repo folder (`D:\Users\Christopher\Source\Repos\homm3\original-data`, read access through the device link; one large LOD per staging call), then `scripts/sandbox-setup.sh`. Without the link the session only prepares (Q48). Sync is `git push` (fetch `master` first so the push is thin).
- Git on the user's machine: get delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock` otherwise); it has no GitHub credentials, so it cannot push.
- The sandbox VM can restart mid-session (files stay, background jobs die): check `ps` and the logs before waiting on a background job.
- `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background and poll its log, or use `verify --summary` (full log in `target/verify/verify.log`). The inner loop is `verify --scope <lane> --quick`; CI runs the full verify.
- The background verify tests whatever is in the clone: try ad-hoc tests in a `git worktree` with its own `CARGO_TARGET_DIR`. `pkill -f <pattern>` also kills the tool's own shell when the pattern is in the command line: kill by pid.
- bench-check at 0 Ir: valgrind 3.22 read no symbols from the lld-linked `h3` (the gate links it with `-z norelro`).

**Sources**
- The wiki is reachable only through WebFetch (summarised) and needs the user's approval per session: ask for verbatim quotes, use `index.php?title=<Page>&action=raw` and `index.php?search=<term>&fulltext=1`; ask for the `swh` template's SoD argument, the summariser otherwise reports HotA.
- VCMI 1.7.5: `git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git` works through the proxy.
- Code reading (Q29): stage `Heroes3.exe`, `pip install capstone pefile`, disassemble `.text` with `skipdata` into one text file and grep it; known layouts in `docs/formats/installation.md` §7. Check each float constant's width (`qword` double or `dword` f32); the x87 runs at 53 bits (§7.10).

**Coverage, lints, formatting**
- LLVM counts the implicit `else` of `if let`, the error arm of `?` and rarely reached closures: no defensive dead branches; use infallible accessors and combinators, saturate instead of returning `Option`, make property tests deterministic in what they cover.
- Coverage misses on non-code lines mean stale binaries: `cargo llvm-cov clean --workspace`.
- clippy: `should_implement_trait` rejects methods named `div`/`shr`/`mul`/`add`/`sub`; `type_complexity` fires on arrays of fn pointers in tests (use a type alias). `cargo fmt` reflows code: match scripted patches against the formatted text.
- Dev-dependencies obey the layers too (`check-arch`): domain crates build test rulesets in code.
- serde ignores unknown keys of a unit variant in an internally tagged enum (use struct variants); TOML serialization needs table fields after plain values.
