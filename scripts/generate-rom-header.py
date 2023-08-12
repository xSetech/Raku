#!/usr/bin/env python3
""" Generate a ROM header blob

See https://n64brew.dev/wiki/ROM_Header
"""

import struct

with open("target/header.bin", "wb") as f:

    # PI BSD DOM1 Configuration Flags
    f.write(struct.pack(">I", 0x80371240))

    # Clock Rate
    f.write(struct.pack(">I", 0x0000000F))

    # Boot Address
    f.write(struct.pack(">I", 0x00000000))

    # "Release Address", or runtime version
    f.write(struct.pack(">I", 0x00000001))

    # Check Code
    f.write(struct.pack(">Q", 0))

    # Reserved
    f.write(struct.pack(">Q", 0))

    # Game Title
    f.write(struct.pack("20s", b"Summer".ljust(20)))

    # Reserved
    f.write(struct.pack("7s", b""))

    # Game Code (Destination Code, Unique Code, Category Code)
    f.write(struct.pack("4s", b"NSuE"))

    # ROM Version
    f.write(struct.pack(">1c", b"\x00"))

# eof
