#!/usr/bin/env bash

set -e
set -u
set -o pipefail

if [[ ! -e kernel/ ]]; then
    echo "fail: ./kernel/ not found!" >&2
    echo "note: call this script from the repo root" >&2
    exit 1
fi

export CARGO_PROFILE="${CARGO_PROFILE:=dev}"

# Step 1: Build the kernels

echo "Building the bootloader..."
cd bootloader/
cargo build -Z build-std=core --color always --profile ${CARGO_PROFILE}
cd - >/dev/null

echo "Building the game engine..."
cd game/
cargo build -Z build-std=core --color always --profile ${CARGO_PROFILE}
cd - >/dev/null

# Step 2: Assemble the ROM

./scripts/assemble-rom.sh

# eof
