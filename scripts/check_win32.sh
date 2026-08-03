#!/bin/sh
# Cross-check dispel-gui-win32 for the Windows target using zig as the C compiler.
# Usage: scripts/check_win32.sh [cargo args...]
cd "$(dirname "$0")/.."
export CC="zig cc"
export CFLAGS="--target=x86_64-windows-gnu"
exec rtk cargo check --target x86_64-pc-windows-gnu -p dispel-gui-win32 "$@"
