# Contributing

Thank you for your interest. This project is an engine for a game with a very precise original, so
the rules below are stricter than usual: they keep the engine **faithful, deterministic and
legally clean**. Most of the code so far was written by AI coding agents under a human maintainer;
human contributions follow the same rules, and nothing here assumes you use an agent (if you do,
point it at [AGENTS.md](AGENTS.md)).

## Before you start

- **Open an issue first** for anything beyond a typo or a small fix: say what you want to change
  and why. Changes to a seam type, an ADR-level decision, a public crate boundary, a dependency or
  the license need the maintainer's agreement before code is written (the list is in
  [AGENTS.md](AGENTS.md), "When to stop and ask").
- Read the [vision](docs/01-vision.md) (goals and non-goals), the
  [architecture](docs/02-architecture.md) and the [domain language](docs/03-domain.md), and the
  [ADRs](docs/adr/README.md) that touch your change. ADRs are binding.
- Find your area: the work is split into lanes (battle, adventure, ai, server, client,
  map-client); [`lanes.toml`](lanes.toml) says who owns which path and each lane's state is in
  `docs/lanes/<lane>/HANDOFF.md`. Keep a pull request inside one lane where you can; a change that
  spans lanes or touches shared paths (kernel, contracts, `Cargo.toml`, ADRs, process docs) is
  split into a small shared change that merges first, or labelled `cross-lane` by the maintainer.

## The rules that matter most

1. **No original game material.** Never commit game files, graphics, sound, maps, flavour text or
   anything extracted from a copy of the game beyond identifiers and numbers. `original-data/` is
   git-ignored and `cargo xtask check-arch` fails if anything under it is tracked. Tests use
   synthetic content ([`packs/synthetic`](packs/synthetic)).
2. **Licensing.** The project is GPL-2.0-or-later. By contributing you agree that your work is
   licensed under it. You may port or adapt code from [VCMI](https://github.com/vcmi/vcmi)
   (GPL-2.0-or-later) with an origin comment and a `kind = "vcmi"` rule source; ported code must
   still obey the rules below. Do not copy from sources with an incompatible or unclear license,
   and keep dependencies GPLv2-compatible (`deny.toml`).
3. **Game logic is pure and deterministic.** In the logic crates: no I/O, clock, threads or
   globals, no `HashMap`/`HashSet`, no sorts without a unique tie-breaker, floating point only
   through the audited `kernel::num` surface ([ADR-0003](docs/adr/0003-determinism-and-numeric-model.md)).
   Same ruleset, state, seed and commands must give bit-identical results on every platform.
4. **Validated and server-authoritative.** Never trust payload-supplied ownership, prices or
   amounts; a rejected command leaves the state untouched.
5. **Every rule has an ID, a source and a test.** Add the rule to the catalog in
   [`docs/rules/catalog/`](docs/rules/catalog) with a labelled source (original data, a reading of
   the executable, wiki page with URL, VCMI behaviour, observation), and mark the code and test
   (`rule:` / `covers:`). Where the original has a bug, follow the quirks ledger and the profiles
   ([docs/04-rules-and-sources.md](docs/04-rules-and-sources.md)): if you are unsure whether
   something is a bug the engine should keep, ask.
6. **Coverage and gates.** Logic crates keep 100 % line and region coverage (a reviewed exclusion
   list exists), plus property tests and determinism replays.

## Building and checking

```sh
cargo xtask verify --quick                    # the inner loop (never counts as green)
cargo xtask verify --scope <lane> --summary   # a lane's loop, one line per gate (exit 2 is normal)
cargo xtask verify                            # every gate, before you open a pull request
cargo xtask check-lane --base origin/master   # your pull request only touches its lane's paths
```

A full verify takes a while (roughly twenty minutes warm on two cores, much longer from a cold
build). Tool requirements are in the [README](README.md#quick-start); the gates are described in
[docs/05-testing.md](docs/05-testing.md). Formatting is `cargo fmt`, lints are `cargo clippy` with
the workspace policy (`clippy.toml`).

Golden replays, benchmark baselines and ruleset-hash headers change with behaviour: regenerate
them deliberately, in the same pull request, with the reason in the commit message
([fixtures/README.md](fixtures/README.md), [benches/README.md](benches/README.md)).

## Pull requests

- Branch names carry the lane: `<lane>/<task>` or `shared/<topic>`. Small, coherent commits with
  messages in the project's language (for example `battle: …`, `fix(game): …`).
- Open the pull request when the branch is ready to merge; fill in the checklist in
  [`docs/pull_request_template.md`](docs/pull_request_template.md) (Definition of Done).
- Keep the documentation current in the same change: the rule catalog, the quirks ledger, ADRs
  (a decision is an ADR draft in `docs/lanes/<lane>/adr-drafts/` until the maintainer accepts it),
  and the lane's HANDOFF/queue if you work in a lane. How docs are organised so they stay short:
  [docs/08-work-process.md](docs/08-work-process.md), "Keeping the start-of-session reading small".
- Incomplete features must be visible: register them in `docs/stubs.toml` (no silent `todo!()` or
  untracked `#[ignore]`; `cargo xtask check-stubs`).
- Continuous integration may be unavailable for a while (limited Actions minutes). Then the
  maintainer relies on your local `cargo xtask verify` result, which you paste into the pull
  request with the commit and date.

## Reporting problems

- **Fidelity bugs** (the engine differs from the original): say which rule, what the original does,
  and how you know (a screenshot or saved game of the original, the wiki page, a VCMI behaviour);
  an observation in the original game is the strongest evidence. A staged experiment can settle it
  ([docs/experiments/README.md](docs/experiments/README.md)).
- **Security issues** (a command that bypasses validation, a hidden-information leak, a server
  vulnerability): please do not open a public issue; contact the maintainer privately via the
  GitHub profile of the repository owner.
- Anything else: an issue with the command you ran, the output and your platform.

Be kind and assume good faith; harassment is not tolerated, and the maintainer may remove comments
or contributors that cross the line.
