# ADR-0010: Testing gates and rule traceability

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

The user requires 100% coverage for everything except truly edge code, later BDD acceptance tests, and a mechanism that prevents agents from forgetting required tests.

## Decision

1. Single entry point `cargo xtask verify` (fmt, clippy policy, deny, check-arch, check-rules, check-stubs, tests incl. property and replay, coverage gate, benchmark regression), required in CI.
2. **100% line and region coverage** on core (logic) crates via LLVM source-based coverage; exclusions only in a single reviewed `coverage-exclusions.toml` and never for domain logic.
3. **Rule traceability:** rule IDs in a catalog; `rule:` markers in code, `covers:` markers in tests; a rule cannot be `implemented` without a covering test.
4. **No silent debt:** stub registry; `todo!`/`unimplemented!`/`#[ignore]`/TODO in logic code require a registered stub.
5. Mandatory property tests per context for atomicity, legality consistency, ownership isolation, observation hygiene, clone equivalence, determinism.
6. Determinism replay fixtures with golden hashes per platform; fuzzing for untrusted-input parsers; periodic mutation testing; benchmark regression thresholds.
7. BDD acceptance tests come later and must reference rule IDs.
8. The Definition of Done in `05-testing.md` is part of every task.

## Consequences

Testing cannot be forgotten without a failing build. Cost: more ceremony per change, and a strict gate that occasionally needs a justified exclusion or baseline update.

## Alternatives considered

Coverage target without traceability (coverage can be satisfied by meaningless tests); traceability without coverage (misses untested branches); convention-only checklists (agents forget).
