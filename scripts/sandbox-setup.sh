#!/usr/bin/env bash
# One-time setup of an agent's cloud sandbox (docs/08-work-process.md, "Cloud sandbox").
#
# Run from a clone of the user's repo, after the user's original-data files were staged:
#   scripts/sandbox-setup.sh [--lane <lane>] [STAGED_REPO_DIR]
# STAGED_REPO_DIR is where the staging tool put the user's repo files
# (default /mnt/user-data/uploads/homm3). --lane names the session's lane: its baseline is the
# gates it builds on (below).
#
# 0. Makes the clone track every branch name: a single-branch clone (`--depth 1`) stores no
#    `origin/<branch>` for a pushed lane branch, and the session's stop hook then reports pushed
#    commits as unpushed (docs/08-work-process.md, "Cloud sandbox"). Fetches stay explicit.
# 1. Links the image's `stable` toolchain as the pinned one (static.rust-lang.org is blocked).
# 2. Copies the staged original-data files into this clone's gitignored original-data/.
# 3. In the background: installs llvm-20 (coverage), cargo-deny and cargo-llvm-cov (their release
#    binaries from GitHub, pinned below; `cargo install` if the download fails), capstone and
#    pefile (scripts/exe.py, docs/exe/README.md), then the baseline through
#    scripts/verify-once.sh, which runs the full gates only for code no run has verified yet
#    (docs/05-testing.md, "Each code state once"): `engine -- --with-original-data`, or for the
#    client lane `clients` (with SDL's build dependencies, clients.yml's list). Logs in
#    target/sandbox/.
# Prints one line per step; the full output stays in the logs.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
lane=
staged=/mnt/user-data/uploads/homm3
while [ $# -gt 0 ]; do
    case "$1" in
        --lane) lane="${2:?--lane needs a lane}"; shift ;;
        *) staged="$1" ;;
    esac
    shift
done
logs="$root/target/sandbox"
mkdir -p "$logs"

# 0. Git: every branch name tracked, the current branch's remote ref fetched if it exists.
git -C "$root" config --replace-all remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
branch="$(git -C "$root" branch --show-current)"
if [ -n "$branch" ] && git -C "$root" ls-remote --exit-code --heads origin "$branch" > /dev/null 2>&1; then
    git -C "$root" fetch -q origin "$branch"
fi
echo "git: every branch tracked (origin/${branch:-?} after its first push)"

# 1. Toolchain.
channel="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$root/rust-toolchain.toml")"
triple="x86_64-unknown-linux-gnu"
toolchains="$HOME/.rustup/toolchains"
if [ ! -e "$toolchains/$channel-$triple" ]; then
    stable="$("$toolchains/stable-$triple/bin/rustc" --version | awk '{print $2}')"
    if [ "$stable" != "$channel" ]; then
        echo "toolchain: the image's stable is $stable, rust-toolchain.toml pins $channel" >&2
        exit 1
    fi
    ln -s "$toolchains/stable-$triple" "$toolchains/$channel-$triple"
fi
echo "toolchain: $(cd "$root" && rustc --version)"

# 2. Original data (read-only copies; never commit them, ADR-0009).
missing=()
for f in install/Heroes3.exe install/Data/H3bitmap.lod install/Data/H3ab_bmp.lod \
    extracted/H3bitmap/Objects.txt; do
    if [ -f "$staged/original-data/$f" ]; then
        mkdir -p "$(dirname "$root/original-data/$f")"
        cp "$staged/original-data/$f" "$root/original-data/$f"
    else
        missing+=("$f")
    fi
done
# The shipped maps, for the map reader's check (E08 T3; optional: the adventure lane stages them).
if [ -d "$staged/original-data/install/Maps" ]; then
    mkdir -p "$root/original-data/install/Maps"
    cp "$staged/original-data/install/Maps/"* "$root/original-data/install/Maps/"
fi
if [ ${#missing[@]} -eq 0 ]; then
    echo "original-data: copied"
else
    echo "original-data: not staged: ${missing[*]} (verify will run without --with-original-data)"
fi

# 3. Tools, then the baseline, in the background. The tools' release binaries (x86_64 Linux,
#    static): seconds, where `cargo install` compiles for about five minutes.
deny_url=https://github.com/EmbarkStudios/cargo-deny/releases/download/0.20.2/cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz
llvm_cov_url=https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.9.1/cargo-llvm-cov-x86_64-unknown-linux-musl.tar.gz
packages=(llvm-20)
if [ "$lane" = client ]; then
    baseline=(clients)
    log=target/sandbox/clients-check.log
    packages+=(cmake libx11-dev libxext-dev libxcursor-dev libxfixes-dev libxi-dev libxrandr-dev
        libxss-dev libxtst-dev libasound2-dev libpulse-dev)
else
    baseline=(engine)
    [ ${#missing[@]} -eq 0 ] && baseline+=(-- --with-original-data)
    log=target/sandbox/verify.log
fi
nohup bash -c "
    (apt-get update -qq && apt-get install -y -qq ${packages[*]}) > '$logs/apt.log' 2>&1 &
    apt=\$!
    bin=\$HOME/.cargo/bin
    for tool in 'cargo-deny $deny_url' 'cargo-llvm-cov $llvm_cov_url'; do
        set -- \$tool
        command -v \$1 > /dev/null && continue
        tmp=\$(mktemp -d)
        if curl -sSfL \"\$2\" | tar -xz -C \"\$tmp\" \
            && install -m 755 \"\$(find \"\$tmp\" -type f -name \$1 | head -n 1)\" \"\$bin/\$1\"; then
            echo \"\$1: release binary\"
        else
            cargo install --locked \$1
        fi
        rm -rf \"\$tmp\"
    done > '$logs/cargo-install.log' 2>&1
    pip install --break-system-packages -q capstone pefile > '$logs/pip.log' 2>&1
    wait \$apt
    '$root/scripts/verify-once.sh' ${baseline[*]}
" > "$logs/setup.log" 2>&1 &
echo "tools and baseline: running in the background (scripts/verify-once.sh ${baseline[*]}; the" \
    "run's line in target/sandbox/setup.log, its log $log)"
