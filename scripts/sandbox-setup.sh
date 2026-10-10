#!/usr/bin/env bash
# One-time setup of an agent's cloud sandbox (docs/08-work-process.md, "Cloud sandbox").
#
# Run from a clone of the user's repo, after the user's original-data files were staged:
#   scripts/sandbox-setup.sh [STAGED_REPO_DIR]
# STAGED_REPO_DIR is where the staging tool put the user's repo files
# (default /mnt/user-data/uploads/homm3).
#
# 1. Links the image's `stable` toolchain as the pinned one (static.rust-lang.org is blocked).
# 2. Copies the staged original-data files into this clone's gitignored original-data/.
# 3. In the background: installs llvm-20 (coverage), cargo-deny and cargo-llvm-cov, capstone and
#    pefile (scripts/exe.py, docs/exe/README.md), then runs
#    scripts/sandbox-verify.sh --with-original-data. Logs in target/sandbox/.
# Prints one line per step; the full output stays in the logs.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
staged="${1:-/mnt/user-data/uploads/homm3}"
logs="$root/target/sandbox"
mkdir -p "$logs"

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

# 3. Tools, then the baseline verify, in the background.
verify_args=()
[ ${#missing[@]} -eq 0 ] && verify_args=(--with-original-data)
nohup bash -c "
    (apt-get update -qq && apt-get install -y -qq llvm-20) > '$logs/apt.log' 2>&1 &
    apt=\$!
    for tool in cargo-deny cargo-llvm-cov; do
        command -v \$tool > /dev/null || cargo install --locked \$tool
    done > '$logs/cargo-install.log' 2>&1
    pip install --break-system-packages -q capstone pefile > '$logs/pip.log' 2>&1
    wait \$apt
    '$root/scripts/sandbox-verify.sh' ${verify_args[*]}
" > "$logs/setup.log" 2>&1 &
echo "tools and verify: running in the background; the baseline verify writes target/sandbox/verify.log"
