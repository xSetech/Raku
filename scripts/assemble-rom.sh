#!/usr/bin/env bash

set -e
set -u
set -o pipefail

if [[ ! -e kernel/ ]]; then
    echo "fail: ./kernel/ not found!" >&2
    echo "note: call this script from the repo root" >&2
    exit 1
fi

# Built outputs:
#   target/mips-ultra64-rcp/...
#   target/mips-ultra64-cpu/...

echo "Generating ROM header..."
./scripts/generate-rom-header.py

echo "Extracting and extending boot loader..."
llvm-objcopy --dump-section .text=target/bootloader.bin ./target/mips-ultra64-cpu/release/bootloader
./scripts/extend-boot-section.py target/bootloader.bin target/mips-ultra64-cpu/release/game

echo "Assembling ROM..."

# TODO:
#   - Name ROM file based on the project name
#   - Include ROM version from the workspace TOML in the file name
if [[ -e target/rom.z64 ]]; then
    mv -v target/rom.z64 target/rom.z64.bck  # backup existing ROM, if it exists
fi

cp target/header.bin target/rom.z64
cat target/bootloader.bin >> target/rom.z64
cat ./target/mips-ultra64-cpu/release/game >> target/rom.z64
# TODO
# cat ./target/mips-ultra64-rcp/release/rcp-kernel >> target/rom.z64

echo "Assembled ROM: target/rom.z64"

# eof
