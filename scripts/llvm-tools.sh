# Sourced by scripts/sandbox-verify.sh and scripts/verify-once.sh: sets LLVM_COV and
# LLVM_PROFDATA for cargo-llvm-cov in a cloud sandbox (docs/08-work-process.md, "Cloud sandbox").
#
# An explicit setting of both wins. Otherwise the pinned toolchain's own `llvm-tools` component
# (rustup's llvm-tools-preview, added when missing): the same LLVM as rustc, which reads the
# coverage profiles rustc writes (2026-10-20: rustc 1.97 is LLVM 22, the image's apt has no
# LLVM 20 installed). Last, the apt package llvm-20 (/usr/lib/llvm-20, the setup before
# 2026-10-20). Neither found: both stay unset and cargo-llvm-cov looks for itself.
llvm_tools_env() {
    if [ -n "${LLVM_COV:-}" ] && [ -n "${LLVM_PROFDATA:-}" ]; then
        return 0
    fi
    local bin
    bin="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
    if [ ! -x "$bin/llvm-profdata" ]; then
        rustup component add llvm-tools-preview > /dev/null 2>&1 || true
    fi
    if [ -x "$bin/llvm-cov" ] && [ -x "$bin/llvm-profdata" ]; then
        export LLVM_COV="$bin/llvm-cov" LLVM_PROFDATA="$bin/llvm-profdata"
    elif [ -x /usr/lib/llvm-20/bin/llvm-cov ]; then
        export LLVM_COV=/usr/lib/llvm-20/bin/llvm-cov LLVM_PROFDATA=/usr/lib/llvm-20/bin/llvm-profdata
    fi
}
