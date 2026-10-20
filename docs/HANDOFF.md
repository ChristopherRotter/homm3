# HANDOFF

Read first, then the lane's HANDOFF (`AGENTS.md`): the lane index and every lane's traps; a lane's state lives in its own HANDOFF only. Rules: `docs/08-work-process.md` ("Lanes", "Keeping the start-of-session reading small"); `check-docs` keeps it under 6 000 bytes; change it in a `shared/` pull request.

## Lanes

| Lane | Owns (`lanes.toml`) | State, traps, tasks |
|---|---|---|
| battle | `h3-battle`, `h3-content-battle`, `h3-pack-battle`, `h3-cli`, the formats/tools crates, packs, fixtures, experiments | [HANDOFF](lanes/battle/HANDOFF.md), [QUEUE](lanes/battle/QUEUE.md) |
| adventure | `h3-adventure`, `h3-game`, `h3-map`, `h3-pack-adventure`, `h3-heroes`; `src/adventure/` in `h3-sim` and `h3-cli`, `fixtures/replays/adventure/`, `docs/formats/h3m.md` | [HANDOFF](lanes/adventure/HANDOFF.md), [QUEUE](lanes/adventure/QUEUE.md) |
| ai | `h3-sim`, `h3-ffi`, `python/` | [HANDOFF](lanes/ai/HANDOFF.md), [QUEUE](lanes/ai/QUEUE.md) |
| server | `h3-protocol`, `h3-server` | [HANDOFF](lanes/server/HANDOFF.md), [QUEUE](lanes/server/QUEUE.md) |
| client | `clients/` (own Cargo workspace) | [HANDOFF](lanes/client/HANDOFF.md), [QUEUE](lanes/client/QUEUE.md) |

Shared (own pull request, merged first): `h3-kernel`, `h3-contracts`, the core `h3-content` and `h3-pack` (R01), workspace and gate files (`Cargo.toml`, `arch.toml`, `lanes.toml`, `xtask`, CI), ADRs, process docs, `BACKLOG.md`, this file. Common: `OPEN-QUESTIONS.md`, `docs/stubs.toml` (a block per lane), both `Cargo.lock`s, `docs/exe/map.toml`, `structs.toml` (a line per entry), each lane's `QUEUE.md` inbox.

- **GitHub and CI:** `ChristopherRotter/homm3` (private, free plan: without minutes or billing runs fail unstarted and the full local verify stands in; CI never blocks work, `05-testing.md` "CI"). `ci.yml` runs `check-lane` and the full verify for pull requests and `master` (once per code, not for docs only) and weekly `cargo deny check advisories`. Changes reach `master` by pull request only, **opened when its branch is ready to merge**; each push to an open one runs CI: merge `master` into it after `master`'s latest run finished.

## Traps for every lane

- **Battle types on the wire** (ADR-0018 point 5): a new `Command`, `EventKind`, `CommandError`, `CastTarget`, `DamageRoll` or `ObstacleKind` variant, or a field of `Observation`'s structs, breaks `h3-protocol`'s build until mapped (same PR, `cross-lane`, or server lane first).
- **Hero-level properties** (a hero's available spells, a spell's mastery) are derived once in `h3-heroes` (ADR-0013 point 2, BAT-Q19): the adventure map calls the same functions, never a copy.

**Sandbox and sync**
- Original files: stage the user's `original-data/` through the device link (`08-work-process.md` "Cloud sandbox"; one LOD per call), then `scripts/sandbox-setup.sh`; without the link a session only prepares (Q48). Fetch `master` before a push (a thin pack).
- **Stop hook "… no remote branch"** after a push: nothing is lost, don't report it; fix: `08-work-process.md` "Cloud sandbox" 1.
- Git on the user's machine: delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock`), `-c user.name -c user.email`; no GitHub credentials there. Cloud sessions open pull requests by the REST API (GraphQL is blocked).
- The VM can restart (files stay, background jobs die): check `ps` first. A build's `Bus error` or `No space`: the disk allowance is spent (`08-work-process.md` "Cloud sandbox" 5).
- `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background (the coverage tools: `scripts/llvm-tools.sh`) and poll its log; the inner loop is `verify --scope <lane> --quick`.
- The background verify tests whatever is in the clone: try ad-hoc tests in a `git worktree` with a `CARGO_TARGET_DIR` of its own (never shared: `xtask` takes its root from its build; remove a finished one). Kill by pid: `pkill -f` matches the tool's own shell.
- bench-check at 0 Ir: valgrind 3.22 found no symbols in the lld-linked `h3` (the gate links with `-z norelro`).

**Sources**
- The wiki is reachable only through WebFetch (summarised) and needs the user's approval per session: ask for verbatim quotes, use `index.php?title=<Page>&action=raw` and `index.php?search=<term>&fulltext=1`; ask for the `swh` template's SoD argument, the summariser otherwise reports HotA.
- VCMI 1.7.5: `git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git` works through the proxy.
- Experiments: agents play batches on the client lane's harness (`experiments/README.md`).
- Code reading (Q29, Q57): stage `Heroes3.exe`, `pip install capstone pefile`, then `scripts/exe.py lookup|fn|switch|callers|refs|grep` with the address map (`docs/exe/README.md`); every address cited in docs or code needs a map entry (`check-exe-map`; `scripts/exe.py cited`). `fn` prints a long function only sliced (`--around`, `--from/--to`): never read `map.toml` or `h3api.toml` whole. It prints each float constant's width; the x87 runs at 53 bits (§7.10).

**Coverage, lints, formatting**
- LLVM counts the implicit `else` of `if let`, the error arm of `?` and rare closures: no defensive dead branches; use infallible accessors and combinators; saturate, do not return `Option`. Lines only proptests reach make the gate flaky: give each a fixed test (`cargo llvm-cov -p <crate> -- --skip <proptest>`).
- Misses on non-code lines: stale binaries (`cargo llvm-cov clean --workspace`).
- clippy: `should_implement_trait` rejects methods named `div`/`shr`/`mul`/`add`/`sub`; `type_complexity` fires on arrays of fn pointers in tests (use a type alias). `cargo fmt` reflows code: match scripted patches against the formatted text.
- Dev-dependencies obey the layers too (`check-arch`): domain crates build test rulesets in code.
- serde ignores unknown keys of a unit variant in an internally tagged enum (use struct variants); TOML serialization needs table fields after plain values.
