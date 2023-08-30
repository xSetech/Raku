#!/usr/bin/env python3
""" Extend a given boot section to 0x1000 - 0x40 bytes and append kernel ELF size
"""

from pathlib import Path
from struct import pack
import sys

# Extracted .text section from the compiled bootloader
BOOTLOADER = Path(sys.argv[1])
KERNEL_ELF = Path(sys.argv[2])

# 0x1000 bytes, minus space for the ROM header
BOOT_LENGTH: int = 0x1000 - 0x40

if not BOOTLOADER.exists():
    print(f"error: boot section doesn't exist: {BOOTLOADER}", file=sys.stderr)
    sys.exit(1)

if not KERNEL_ELF.exists():
    print(f"error: kernel elf doesn't exist: {KERNEL_ELF}", file=sys.stderr)
    sys.exit(1)

with BOOTLOADER.open("rb") as f:
    content: bytes = f.read()
    if len(content) > BOOT_LENGTH:
        print(f"error: {BOOTLOADER} is too large: {len(content)} bytes vs {BOOT_LENGTH} max", file=sys.stderr)
        sys.exit(1)

padding: int = BOOT_LENGTH - len(content)
if padding != 0:
    print(f"Padding {BOOTLOADER} with {padding} bytes")
    with BOOTLOADER.open("wb") as f:
        f.write(content + (b"\x00" * padding))

kernel_elf_size: int = KERNEL_ELF.stat().st_size
kernel_elf_size_word: bytes = pack(">I", kernel_elf_size)  # be
print(f"Size of kernel is {kernel_elf_size} bytes (0x{kernel_elf_size:08X})")
with BOOTLOADER.open("ab") as f:
    f.write(kernel_elf_size_word)

    # Padding for 8-byte alignment; see cpu kernel linker script
    f.write(b"\x00\x00\x00\x00")

# eof
