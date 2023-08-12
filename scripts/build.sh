#!/usr/bin/env bash

set -e
set -u
set -o pipefail

if [[ ! -e cpu-kernel/ ]]; then
    echo "fail: ./cpu-kernel/ not found!" >&2
    echo "note: call this script from the repo root" >&2
    exit 1
fi

export CARGO_PROFILE="${CARGO_PROFILE:=dev}"

# Step 1: Build the kernels

echo "Building the bootloader..."
cd bootloader/
cargo build -Z build-std=core --color always --profile ${CARGO_PROFILE}
cd - >/dev/null

echo "Building the CPU kernel..."
cd cpu-kernel/
cargo build -Z build-std=core --color always --profile ${CARGO_PROFILE}
cd - >/dev/null

echo "Building the RCP kernel..."
cd rcp-kernel/
cargo build -Z build-std=core --color always --profile ${CARGO_PROFILE}
cd - >/dev/null

# Step 2: Assemble the ROM

./scripts/assemble-rom.sh

# eof
