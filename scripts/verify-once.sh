#!/usr/bin/env bash
# Verifies each state of the code once, in a sandbox as CI does (user, 2026-10-14;
# docs/05-testing.md, "Each code state once").
#
#   scripts/verify-once.sh engine  [--force] [--base <ref>] [-- <verify options>]
#   scripts/verify-once.sh clients [--force] [--base <ref>]
#
# The code's fingerprint hashes the tracked files the heavy gates read:
# - engine: every tracked file but docs/, experiments/, clients/ and *.md (ci.yml's "Plan" step
#   computes the same: keep the two identical);
# - clients: the paths .github/workflows/clients.yml watches, *.md left out.
# The code counts as verified when a success status with the description `code <fingerprint>` and
# the context `verified (ubuntu-24.04)` (CI's; engine only) or this script's (`verified (sandbox)`,
# `clients verified (sandbox)`) sits on HEAD or one of its last ten first-parent commits, on one of
# origin/master's last ten, or on the head of a pull request one of those merged. Then only the
# light gates run: for the engine the fast checks that read the left-out paths (check-rules,
# check-stubs, check-docs, check-exe-map), for the clients none. `--base <ref>` (a pull request's
# verify: `--base origin/master`): a branch whose changes since its merge base with <ref> leave the
# fingerprinted paths alone brings no code of its own and also runs only the light gates, as CI
# does for such a pull request (the base's own verify covers its code). Otherwise the full run:
# scripts/sandbox-verify.sh <verify options> (target/sandbox/verify.log) or
# clients/scripts/check.sh (target/sandbox/clients-check.log). After a full run or a fingerprint
# found, the status goes on HEAD when the fingerprinted paths are committed and GitHub has HEAD,
# so that the next lookup finds it there. `--force` always runs the full verify. Lookups and the
# record use `gh api` (REST); without gh a full run, no record. Exit code: the run's.
set -uo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
logs="$root/target/sandbox"
mkdir -p "$logs"

usage() {
    echo "usage: scripts/verify-once.sh engine|clients [--force] [--base <ref>] [-- <verify options>]" >&2
    exit 2
}

kind="${1:-}"
[ "$kind" = engine ] || [ "$kind" = clients ] || usage
shift
force=false
base=
extra=()
while [ $# -gt 0 ]; do
    case "$1" in
        --force) force=true ;;
        --base) [ $# -ge 2 ] || usage; base="$2"; shift ;;
        --) shift; extra=("$@"); break ;;
        *) usage ;;
    esac
    shift
done

if [ "$kind" = engine ]; then
    spec=(. ':!docs/' ':!experiments/' ':!clients/' ':!*.md')
    contexts=("verified (ubuntu-24.04)" "verified (sandbox)")
    log="$logs/verify.log"
else
    # The push trigger's path list of clients.yml (its pull_request list is the same).
    mapfile -t spec < <(python3 -I - "$root/.github/workflows/clients.yml" <<'PY'
import sys
lines = open(sys.argv[1]).read().splitlines()
start = next(i for i, l in enumerate(lines) if l.strip() == "paths:")
for line in lines[start + 1:]:
    item = line.strip()
    if not item.startswith("- "):
        break
    path = item[2:].strip().strip('"')
    if not path.startswith("!"):
        print(path[:-3] if path.endswith("/**") else path)
PY
    )
    [ ${#spec[@]} -gt 0 ] || { echo "verify-once: no paths in clients.yml" >&2; exit 2; }
    spec+=(':!*.md')
    contexts=("clients verified (sandbox)")
    log="$logs/clients-check.log"
fi
record_context="${contexts[-1]}"

fp=$(git ls-files -s -- "${spec[@]}" | sha256sum | cut -c1-40)
dirty=false
[ -n "$(git status --porcelain -- "${spec[@]}")" ] && dirty=true
repo="${GH_REPO:-$(git remote get-url origin | sed -E 's#^.*github\.com[:/]##; s#\.git$##')}"
have_gh=false
command -v gh > /dev/null && have_gh=true

# Whether commit $1 carries a success status for this fingerprint.
recorded_on() {
    local filter="" c
    for c in "${contexts[@]}"; do
        filter+="${filter:+ or }.context == \"$c\""
    done
    [ -n "$(gh api "repos/$repo/commits/$1/statuses?per_page=100" --jq ".[] | select(.state ==
        \"success\" and .description == \"code $fp\" and ($filter)) | .id" 2> /dev/null)" ]
}

# The commit whose statuses record this fingerprint, if any.
verified_on() {
    local c h
    declare -A seen=()
    local masters
    masters=$(git rev-list --first-parent -n 10 origin/master 2> /dev/null)
    for c in $(git rev-list --first-parent -n 10 HEAD) $masters; do
        [ -n "${seen[$c]:-}" ] && continue
        seen[$c]=1
        recorded_on "$c" && { echo "$c"; return 0; }
    done
    for c in $masters; do
        for h in $(gh api "repos/$repo/commits/$c/pulls" --jq '.[].head.sha' 2> /dev/null); do
            [ -n "${seen[$h]:-}" ] && continue
            seen[$h]=1
            recorded_on "$h" && { echo "$h"; return 0; }
        done
    done
    return 1
}

record() {
    local head
    head=$(git rev-parse HEAD)
    if [ "$dirty" = true ]; then
        echo "verify-once: not recorded: the fingerprinted paths have uncommitted changes"
    elif [ "$have_gh" = false ]; then
        echo "verify-once: not recorded: no gh"
    elif ! gh api "repos/$repo/commits/$head" --jq .sha > /dev/null 2>&1; then
        echo "verify-once: not recorded: GitHub does not have $head (push it, then run again)"
    elif gh api "repos/$repo/statuses/$head" -f state=success -f context="$record_context" \
        -f description="code $fp" > /dev/null; then
        echo "verify-once: recorded \`$record_context\` (code $fp) on $head"
    else
        echo "verify-once: not recorded: the status call failed"
    fi
}

light() {
    echo "verify-once: $kind code $fp: $1; only the light gates"
    if [ "$kind" = clients ]; then
        echo "EXIT=0" > "$log"
        return 0
    fi
    local code=0 gate
    {
        for gate in check-rules check-stubs check-docs check-exe-map; do
            echo "==> $gate"
            cargo xtask "$gate" || code=1
        done
        echo "EXIT=$code"
    } > "$log" 2>&1
    grep -E '^EXIT=' "$log"
    return $code
}

found=
why=
if [ "$force" = true ]; then
    why="--force"
elif [ "$dirty" = true ]; then
    why="uncommitted changes in the fingerprinted paths"
elif [ -n "$base" ] && mb=$(git merge-base "$base" HEAD 2> /dev/null) \
    && git diff --quiet "$mb" -- "${spec[@]}"; then
    light "no changes to the fingerprinted paths since the merge base with $base"
    exit $?
elif [ "$have_gh" = false ]; then
    why="no gh to look up the fingerprint"
elif found=$(verified_on); then
    light "verified on $found"
    code=$?
    [ $code -eq 0 ] && [ "$found" != "$(git rev-parse HEAD)" ] && record
    exit $code
else
    why="no verified commit carries the fingerprint"
fi

echo "verify-once: $kind code $fp: $why; the full run (log: $log)"
if [ "$kind" = engine ]; then
    "$root/scripts/sandbox-verify.sh" "${extra[@]}"
    code=$?
else
    export LLVM_COV="${LLVM_COV:-/usr/lib/llvm-20/bin/llvm-cov}"
    export LLVM_PROFDATA="${LLVM_PROFDATA:-/usr/lib/llvm-20/bin/llvm-profdata}"
    "$root/clients/scripts/check.sh" > "$log" 2>&1
    code=$?
    echo "EXIT=$code" >> "$log"
fi
grep -E '^EXIT=' "$log" | tail -n 1
[ $code -eq 0 ] && record
exit $code
