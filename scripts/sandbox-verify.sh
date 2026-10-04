#!/usr/bin/env bash
# `cargo xtask verify` in an agent's cloud sandbox (docs/08-work-process.md, "Cloud sandbox").
#
#   scripts/sandbox-verify.sh [verify options]      # e.g. --with-original-data
#
# Runs in the foreground with the LLVM 20 coverage tools and writes target/sandbox/verify.log,
# ending with "EXIT=<code>". A full run takes 2-10 minutes, longer than an agent's tool call:
# start it with `nohup scripts/sandbox-verify.sh ... > /dev/null 2>&1 &` and poll
# `tail -n 3 target/sandbox/verify.log`. If the determinism step dies with SIGILL, the sandbox
# moved to another CPU: this script then deletes target/native once and runs verify again.
set -uo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
log="$root/target/sandbox/verify.log"
mkdir -p "$(dirname "$log")"
export LLVM_COV=/usr/lib/llvm-20/bin/llvm-cov
export LLVM_PROFDATA=/usr/lib/llvm-20/bin/llvm-profdata

cd "$root"
cargo xtask verify "$@" > "$log" 2>&1
code=$?
if [ $code -ne 0 ] && grep -q "SIGILL" "$log"; then
    rm -rf "$root/target/native"
    cargo xtask verify "$@" > "$log" 2>&1
    code=$?
fi
echo "EXIT=$code" >> "$log"
exit $code
