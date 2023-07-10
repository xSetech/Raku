# 🏖️ Summer

This repo contains work in progress homebrew for the Nintendo 64
intended as a submission to the summer-themed N64brew Game Jam 2023.

## Repo Layout

The repo contains three crates that target two different architectures.

### 3 Crates

1. `cpu-kernel`, a kernel that runs on the CPU.
2. `rcp-kernel`, a kernel that runs on the RCP.
3. `lib-kernel`, a library supporting both kernels.

Most of the code critical for actual gameplay lives in `lib-kernel`.

The other two crates exist to make compiling for each architecture a bit easier
when using Cargo as the build system.

### 2 Microarchitectures

The Nintendo 64 contains two MIPS processors with different microarchitectures.

1. The CPU, which is basically a VR4300.
2. The RCP, which is... a lobotimized R4000 with a 32-bit ISA extended for
   vector math and video output.

## Building

### Requirements

- Rust, patched to support the N32 MIPS ABI.
- A typical Unix terminal, like Bash or zsh
- ... maybe LLVM w/ LLD?

### Overview and How-to

An end-to-end build happens in two steps:

1. The two kernels are built via `cargo build -Z build-std=core` in each crate directory.
2. A ROM (.z64) is formed from the built kernels via `assemble-rom.sh`

You can run all build everything end-to-end by running `build.sh`

## License

This work is available under the GNU General Public License v3.0 or later, see LICENSE.md
