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
# The code counts as verified when it was verified in this sandbox (target/sandbox/verified.txt),
# or GitHub records it for HEAD or one of its last ten first-parent commits, for one of
# origin/master's last ten, or for a pull request one of those belongs to: CI's success status
# `verified (ubuntu-24.04)` with the description `code <fingerprint>` (engine only), or the line
# `verify-once: verified (sandbox) code <fingerprint> …` (`clients verified (sandbox)` for the
# clients) in such a pull request's description. Then only the light gates run: for the engine
# the fast checks that read the left-out paths (check-rules, check-stubs, check-docs,
# check-exe-map), for the clients none. `--base <ref>` (a pull request's verify: `--base
# origin/master`): a branch whose changes since its merge base with <ref> leave the fingerprinted
# paths alone brings no code of its own and also runs only the light gates, as CI does for such a
# pull request (the base's own verify covers its code). Otherwise the full run:
# scripts/sandbox-verify.sh <verify options> (target/sandbox/verify.log) or
# clients/scripts/check.sh (target/sandbox/clients-check.log). After a full run that passed, or a
# fingerprint found, on committed files: the sandbox notes it, and the line goes into the
# description of the branch's open pull request (replacing an earlier one of its kind), or is
# printed for the pull request's verify line when none is open yet. (The sandbox's GitHub proxy
# refuses commit statuses and custom refs, 2026-10-15; pull requests it allows.) `--force` always
# runs in full. Lookups and the record use `gh api` (REST); without gh only the sandbox's notes.
# Exit code: the run's.
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

notes="$logs/verified.txt"
line="verify-once: $record_context code $fp"

# Whether commit $1 carries CI's status for this fingerprint, or one of its pull requests the line.
recorded_on() {
    if [ "$kind" = engine ] && [ -n "$(gh api "repos/$repo/commits/$1/statuses?per_page=100" \
        --jq ".[] | select(.state == \"success\" and .context == \"${contexts[0]}\"
            and .description == \"code $fp\") | .id" 2> /dev/null)" ]; then
        return 0
    fi
    gh api "repos/$repo/commits/$1/pulls" --jq '.[].body // ""' 2> /dev/null | grep -qF "$line"
}

# Where this fingerprint was verified, if anywhere.
verified_on() {
    if grep -qxF "$kind $fp" "$notes" 2> /dev/null; then
        echo "this sandbox"
        return 0
    fi
    [ "$have_gh" = true ] || return 1
    local c
    declare -A seen=()
    for c in $(git rev-list --first-parent -n 10 HEAD) \
        $(git rev-list --first-parent -n 10 origin/master 2> /dev/null); do
        [ -n "${seen[$c]:-}" ] && continue
        seen[$c]=1
        recorded_on "$c" && { echo "$c"; return 0; }
    done
    return 1
}

record() {
    local head branch pr body
    head=$(git rev-parse HEAD)
    if [ "$dirty" = true ]; then
        echo "verify-once: not recorded: the fingerprinted paths have uncommitted changes"
        return 0
    fi
    grep -qxF "$kind $fp" "$notes" 2> /dev/null || echo "$kind $fp" >> "$notes"
    branch=$(git branch --show-current)
    pr=
    if [ "$have_gh" = true ] && [ -n "$branch" ]; then
        pr=$(gh api "repos/$repo/pulls?state=open&head=${repo%%/*}:$branch" --jq '.[0].number // empty' \
            2> /dev/null)
    fi
    if [ -z "$pr" ]; then
        echo "verify-once: no open pull request for this branch: put this line into its verify line:"
        echo "$line on $head"
        return 0
    fi
    body=$(gh api "repos/$repo/pulls/$pr" --jq '.body // ""')
    { printf '%s\n' "$body" | grep -vF "verify-once: $record_context code "; echo "$line on $head"; } \
        > "$logs/pr-body.md"
    if gh api -X PATCH "repos/$repo/pulls/$pr" -F body=@"$logs/pr-body.md" > /dev/null; then
        echo "verify-once: recorded in #$pr: $line on $head"
    else
        echo "verify-once: not recorded: #$pr's description could not be changed; put this line in:"
        echo "$line on $head"
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
elif found=$(verified_on); then
    light "verified on $found"
    code=$?
    [ $code -eq 0 ] && record
    exit $code
else
    why="not verified yet"
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
