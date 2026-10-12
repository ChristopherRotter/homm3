#!/usr/bin/env bash
# The client workspace's gates (E17 M0 T2): what the CI job `.github/workflows/clients.yml` runs,
# and the full local check, in CI's place while CI cannot run (docs/05-testing.md, "CI").
#
#   clients/scripts/check.sh [--no-coverage]
#
# Needs cargo-deny and cargo-llvm-cov (`cargo install cargo-deny cargo-llvm-cov --locked`) and the
# LLVM tools (`rustup component add llvm-tools-preview`, or LLVM_COV/LLVM_PROFDATA as
# scripts/sandbox-verify.sh sets them). Prints one line per gate; stops at the first failure.
set -euo pipefail
cd "$(dirname "$0")/.."

coverage=true
for arg in "$@"; do
    case "$arg" in
        --no-coverage) coverage=false ;;
        *) echo "usage: clients/scripts/check.sh [--no-coverage]" >&2; exit 2 ;;
    esac
done

step() { echo "== $*"; }

step fmt
cargo fmt --check
step clippy
cargo clippy --quiet --all-targets -- -D warnings
step test
cargo test --quiet
step deny
cargo deny --all-features check 2>&1 | tail -n 3

# Coverage: 100 % of lines and regions in the pure crates' libraries (docs/05-testing.md). The
# binaries under `src/bin/` are adapters (files, clock, printing), left out by design; an engine
# crate outside this workspace is never in the report. Exclusions (*provisional*, E17 M0 T2; each
# with its reason, as the engine's coverage-exclusions.toml):
#   /src/bin/  the adapters
ignore='/src/bin/'
if [ "$coverage" = true ]; then
    step "coverage (excluded: $ignore)"
    cargo llvm-cov --quiet --workspace --all-targets --ignore-filename-regex "$ignore" \
        --fail-under-lines 100 --fail-under-regions 100 --show-missing-lines --summary-only \
        | tail -n 4
fi

# The CI job's path filter must name every engine crate this workspace builds, so that a change to
# one of them runs the job (E17 M0 T2).
workflow=../.github/workflows/clients.yml
if [ -f "$workflow" ]; then
    step "ci paths"
    cargo metadata --format-version 1 --locked | python3 -I -c '
import json, os, sys
workflow = open(sys.argv[1]).read()
root = os.path.realpath("..")
missing = []
for p in json.load(sys.stdin)["packages"]:
    if p["source"] is not None:
        continue
    d = os.path.relpath(os.path.dirname(os.path.realpath(p["manifest_path"])), root)
    if not d.startswith("clients" + os.sep) and f"\"{d}/**\"" not in workflow:
        missing.append(d)
if missing:
    sys.exit("clients.yml does not watch: " + ", ".join(sorted(missing)))
print("ci paths: ok")
' "$workflow"
fi
echo "== all gates passed"
