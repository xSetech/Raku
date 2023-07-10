#!/usr/bin/env bash

set -e
set -u
set -o pipefail

if [[ ! -e cpu-kernel/ ]]; then
    echo "fail: ./cpu-kernel/ not found!" >&2
    echo "note: call this script from the repo root" >&2
    exit 1
fi

# Built outputs:
#   target/mips-ultra64-rcp/...
#   target/mips-ultra64-cpu/...

echo "Assembling ROM..."

# TODO
#
# How this might work:
#
#   - Craft the ROM header, place up to 0x40
#   - Remap IPL3 from RCP kernel ELF up to 0x1000
#   - Concatenate the two kernels, RCP kernel first
#
echo "ROM assembly unimplemented!" >&2
exit 1

# eof
