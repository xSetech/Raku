#!/usr/bin/env python3
""" Extend a given boot section to 0x1000 - 0x40 bytes
"""

from pathlib import Path
import sys

# The literal .boot section from the compiled ELF
BOOT_SECTION = Path(sys.argv[1])

# 0x1000 bytes, minus space for the ROM header
BOOT_LENGTH: int = 0x1000 - 0x40

if not BOOT_SECTION.exists():
    print(f"error: boot section doesn't exist: {BOOT_SECTION}", file=sys.stderr)
    sys.exit(1)

with BOOT_SECTION.open("rb") as f:
    content: bytes = f.read()
    if len(content) > BOOT_LENGTH:
        print(f"error: {BOOT_SECTION} is too large: {len(content)} bytes vs {BOOT_LENGTH} max", file=sys.stderr)
        sys.exit(1)

padding: int = BOOT_LENGTH - len(content)
if padding != 0:
    print(f"Padding {BOOT_SECTION} with {padding} bytes")
    with BOOT_SECTION.open("wb") as f:
        f.write(content + (b"\x00" * padding))

# eof
