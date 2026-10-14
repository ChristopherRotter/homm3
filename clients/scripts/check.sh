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

# Runs a command quietly: its last lines when it passes, all of its output when it fails.
run() {
    local tail_lines=$1 out
    shift
    if out=$("$@" 2>&1); then
        [ "$tail_lines" -gt 0 ] && printf '%s\n' "$out" | tail -n "$tail_lines"
        return 0
    fi
    printf '%s\n' "$out"
    echo "== failed: $*" >&2
    exit 1
}

# `--locked` everywhere: a change to the root Cargo.toml's workspace dependencies must not
# re-resolve clients/Cargo.lock silently.
step fmt
run 0 cargo fmt --check
step clippy
run 0 cargo clippy --locked --all-targets -- -D warnings
step test
run 1 cargo test --locked --quiet
step deny
run 1 cargo deny --locked --all-features check

# The deny policy stays the engine's until the user's licence decision at M2 (ADR-0020 point 4).
step "deny policy"
run 1 python3 -I -c '
import sys, tomllib
a, b = (tomllib.load(open(p, "rb")) for p in sys.argv[1:])
if a.get("licenses") != b.get("licenses") or a.get("sources") != b.get("sources"):
    sys.exit("clients/deny.toml: [licenses] or [sources] differ from the root deny.toml")
print("deny policy: as the engine'"'"'s")
' deny.toml ../deny.toml

# Coverage: 100 % of lines and regions in the pure crates' libraries (docs/05-testing.md). The
# binaries under `src/bin/` are adapters (files, clock, printing), left out by design; an engine
# crate outside this workspace is never in the report. Exclusions (*provisional*, E17 M0 T2; each
# with its reason, as the engine's coverage-exclusions.toml):
#   /src/bin/         the adapters
#   /crates/h3-sdl/   the platform adapter: every SDL call (E17 M2 T5a, ADR-0024); `h3-view
#                     window --check` exercises it
ignore='/src/bin/|/crates/h3-sdl/'
if [ "$coverage" = true ]; then
    step "coverage (excluded: $ignore)"
    run 1 cargo llvm-cov --locked --quiet --workspace --all-targets \
        --ignore-filename-regex "$ignore" --fail-under-lines 100 --fail-under-regions 100 \
        --show-missing-lines --summary-only
fi

# The CI job's path filter must name every engine crate this workspace builds, in both of its
# lists (push and pull_request), so that a change to one of them runs the job (E17 M0 T2). The
# metadata goes through stdin: as an argument it outgrows the system's limit (E17 M1 T2).
workflow=../.github/workflows/clients.yml
step "ci paths"
if [ -f "$workflow" ]; then
    metadata=$(cargo metadata --locked --format-version 1)
    run 1 python3 -I -c '
import json, os, sys
workflow = open(sys.argv[1]).read()
root = os.path.realpath("..")
missing = []
for p in json.load(sys.stdin)["packages"]:
    if p["source"] is not None:
        continue
    d = os.path.relpath(os.path.dirname(os.path.realpath(p["manifest_path"])), root)
    if not d.startswith("clients" + os.sep) and workflow.count(f"\"{d}/**\"") != 2:
        missing.append(d)
if missing:
    sys.exit("clients.yml does not watch (in both lists): " + ", ".join(sorted(missing)))
print("ci paths: ok")
' "$workflow" <<< "$metadata"
else
    echo "ci paths: skipped (no .github/workflows/clients.yml)"
fi
echo "== all gates passed"
