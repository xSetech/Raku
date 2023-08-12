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

echo "Generating ROM header..."
./scripts/generate-rom-header.py

echo "Extracting and extending boot loader..."
llvm-objcopy --dump-section .text=target/bootloader.bin ./target/mips-ultra64-cpu/release/bootloader
./scripts/extend-boot-section.py target/bootloader.bin

echo "Assembling ROM..."
cat target/header.bin target/bootloader.bin ./target/mips-ultra64-cpu/release/cpu-kernel ./target/mips-ultra64-rcp/release/rcp-kernel > target/rom.z64

echo "Assembled ROM: target/rom.z64"

# eof
