// SPDX-License-Identifier: GPL-3.0-or-later

//! Raku Bootloader
//!
//! This program confirms RDRAM is setup, copies a kernel entry point into RAM,
//! and jumps to the entry point. The kernel is expected to be stored in an ELF
//! file located in the cartridge ROM at an offset known ahead of time.
//!
//! Note the target of the jump to the kernel entry point will be in the KSEG1
//! segment; the kernel is responsible for CPU caches and the TLB. The kernel
//! must also signal to the PIF that console startup has completed.
//!
//! This program is typically called the "IPL3". It's loaded from the cartridge
//! ROM by console firmware into RSP DMEM as part of console startup. Despite
//! being loaded into a memory bank on the RSP, it's actually executed from the
//! CPU (VR4300). Reminder that the PC will be an address in the KSEG1 memory
//! segment, which is directly mapped (no TLB) and uncached, offset by 0x40.
//!
//! For more information about how the console starts, read here:
//! - https://n64brew.dev/wiki/PIF-NUS#Console_startup

#![no_std]
#![no_main]

#![feature(asm_experimental_arch)]
#![feature(pointer_byte_offsets)]
#![feature(asm_const)]

use core::arch::asm;
use core::panic::PanicInfo;

/// Base addresses (in the KSEG1 segment) of various memory-mapped hardware interfaces.
///
/// Documentation:
///     - https://n64brew.dev/wiki/Memory_map#Physical_Memory_Map
///
const MIPS_INTERFACE_BASE: usize = 0xA4300000;
// const RDRAM_INTERFACE_BASE: usize = 0xA4700000;
const CARTRIDGE_ROM_VADDR_BASE: usize = 0xB0000000;
// const CARTRIDGE_ROM_PI_ADDR_BASE: usize = 0x10000000;

/*
    32-bit ELF offsets and constants

    Documentation:
        - https://refspecs.linuxfoundation.org/elf/gabi4+/ch4.eheader.html
        - https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#File_header
*/

/// 32-bit ELF file header offset and constants
const ELF_32_OFFSET_OF_E_SHOFF: usize = 0x20;
const ELF_32_OFFSET_OF_E_SHNUM: usize = 0x30;
const ELF_32_E_SHENTSIZE: usize = 0x28;

/// 32-bit ELF section header entry offset and constants
const ELF_32_OFFSET_OF_SH_SIZE: usize = 0x14;
const ELF_32_OFFSET_OF_SH_ADDR: usize = 0x0C;
const ELF_32_OFFSET_OF_SH_OFFSET: usize = 0x10;

/*
    Addresses and offsets below regarding the kernel ELF are chosen.
    Remember to update the linker and Python scripts if they change.
*/

/// Address of the entry point of the kernel (the __start symbol / function).
/// Kernel ELF sections necessary to run the kernel are copied here directly
/// from the cartridge ROM.
const KERNEL_ENTRY_ADDRESS: usize = 0xA0000000;

/// Offset in the ROM where content of the kernel ELF starts.
///
/// Note, two words precede this value in the ROM. The first word is the size of
/// the kernel ELF file (unused in this program). The second word is reserved
/// for future use (e.g. another file), but mainly pads the kernel file so that
/// it's 8-byte aligned. See the copy function below for why that's important.
///
/// Note also a discussion about libdragon's approach for ELFs/objects:
/// https://discord.com/channels/205520502922543113/1144749130309189823
///
const KERNEL_ELF_OFFSET_IN_ROM: usize = 0x1008;  // req: 8-byte alignment

/// Disable all interrupts to cautiously prevent disruption of the boot process.
/// Interrupt handlers will be setup by the kernel shortly after entry.
///
/// Documentation:
///     - https://n64brew.dev/wiki/File:VR4300-Users-Manual.pdf
///     - https://n64brew.dev/wiki/MIPS_Interface#0x0430_000C_-_MI_MASK
///
#[inline(always)]
fn mask_interrupts() {

    // Disable interrupts controlled by the "System Control Coprocessor" (CP0). This
    // is done by reading the CP0 Status register (r12), zeroing the interrupt masks
    // and global interrupt-enable bit, and writing back the modified register value.
    // See p.170 of the VR4300 manual.
    let mut cp0_status_reg_value: u32;
    unsafe {
        asm!(
            "mfc0 $13, $12",  // "Move From Coprocessor 0"
            out("$13") cp0_status_reg_value,
        );
    }

    // Bits 0 (global interrupt enable bit) and 8-15 (specific peripheral
    // interrupt masks) are cleared here, while unrelated and currently set bits
    // of the status are preserved and written back to CP0 r12.
    //
    //                 offset=                    15   11   7    3
    cp0_status_reg_value &= 0b1111_1111_1111_1111_0000_0000_1111_1110;
    unsafe {
        asm!(
            "mtc0 $13, $12",  // "Move To Coprocessor 0"
            in("$13") cp0_status_reg_value,
        );
    }

    // Disable interrupts managed by the MIPS/CPU interface by writing a value
    // to the "mask" register. Each "clear mask" flag corresponding to each type
    // of interrupt in the value is set to 1. The flags are located at bits 0,
    // 2, 4, 6, 8, and 10.
    let mi: *mut u32 = MIPS_INTERFACE_BASE as *mut u32;
    unsafe {
        *mi.offset(3) = 0b0000_0000_0000_0000_0000_0101_0101_0101;
    }

}

/// Confirm RDRAM is initialized (TODO)
///
/// Documentation:
///     - https://n64brew.dev/wiki/RDRAM#Initialization_Sequence
///
#[inline(always)]
fn init_rdram() {
    return;
}

/// Copy some amount of bytes from cartridge ROM to RDRAM.
///
/// This copies data word by word from the ROM's memory-mapped address space.
/// Kernel data are required to be at least word-aligned in the ROM (for least
/// surprising behavior). The wiki pages on the PI and memory map document what
/// kind of surprises can come from unaligned accesses.
///
/// Documentation:
///     - https://n64brew.dev/wiki/Peripheral_Interface
///
#[inline(always)]
fn copy_from_rom_to_ram(from_offset_in_rom: usize, to_vaddr: usize, num_bytes: usize) {
    let vaddr_src: *mut u32 = (CARTRIDGE_ROM_VADDR_BASE + from_offset_in_rom) as *mut u32;
    let vaddr_dst: *mut u32 = to_vaddr as *mut u32;
    let num_words: usize = num_bytes / 4;
    for idx in 0..num_words as isize {
        unsafe {
            *vaddr_dst.offset(idx) = *vaddr_src.offset(idx);
        }
    }
}

/// Load a kernel from the cartridge ROM.
///
/// To reduce complexity of the bootloader, the kernel ELF file in ROM isn't
/// fully parsed. Compile-time constant offsets are used to read the section
/// header and metadata about each section (i.e. size, virtual address, and
/// offset within the file). A section is loaded into RDRAM if it has non-zero
/// size and a virtual address in RDRAM KSEG1.
///
#[inline(always)]
fn load_kernel() {

    // Read, from the ELF header, the offset of the section header table within the file
    let section_header_table_offset: usize = unsafe {
        *((CARTRIDGE_ROM_VADDR_BASE + KERNEL_ELF_OFFSET_IN_ROM + ELF_32_OFFSET_OF_E_SHOFF) as *mut u32) as usize
    };

    // Read, from the ELF header, the number of sections within the section header table
    let section_header_table_entries: usize = unsafe {
        *((CARTRIDGE_ROM_VADDR_BASE + KERNEL_ELF_OFFSET_IN_ROM + ELF_32_OFFSET_OF_E_SHNUM) as *mut u16) as usize
    };

    for sht_entry_idx in 0..section_header_table_entries {
        let sht_entry_offset: usize = ELF_32_E_SHENTSIZE * sht_entry_idx;
        let sht_entry_ptr: *mut u32 = (
            CARTRIDGE_ROM_VADDR_BASE + KERNEL_ELF_OFFSET_IN_ROM + section_header_table_offset + sht_entry_offset
        ) as *mut u32;

        // Read, from the section header table entry, the size of the section in bytes.
        // Skip the section if the size is zero (i.e. nothing to copy to RAM).
        let section_size: usize = unsafe { *(sht_entry_ptr.byte_offset(ELF_32_OFFSET_OF_SH_SIZE as isize)) } as usize;
        if section_size == 0 {
            continue;
        }

        // Read, from the section header table entry, the virtual address of the section.
        // Skip the section if the linker did not place it in RDRAM KSEG1.
        let section_vaddr: usize = unsafe { *(sht_entry_ptr.byte_offset(ELF_32_OFFSET_OF_SH_ADDR as isize)) } as usize;
        if section_vaddr & 0xA0000000 != 0xA0000000 {
            continue;  // skip, not in KSEG1
        }
        if section_vaddr & 0x1FFFFFFF > 0x03EFFFFF {
            continue;  // skip, not in RDRAM
        }

        // Read, from the section header table entry, the offset of the section within the file.
        let section_offset_in_elf: usize = unsafe { *(sht_entry_ptr.byte_offset(ELF_32_OFFSET_OF_SH_OFFSET as isize)) } as usize;
        let section_offset_in_rom: usize = KERNEL_ELF_OFFSET_IN_ROM + section_offset_in_elf;

        copy_from_rom_to_ram(
            section_offset_in_rom,
            section_vaddr,
            section_size,
        );
    }

}

/// Set the stack pointer and enter the loaded kernel.
///
/// Note, one approach could have been to cast an address as a function pointer
/// and call into the entry point, as such:
///
/// ```no_run
/// let enter_kernel: extern "C" fn() -> ! = unsafe {
///     core::intrinsics::transmute(KERNEL_ENTRY_ADDRESS as *const ())
/// };
/// enter_kernel();
/// ```
///
/// The code will compile to the same jump instruction, but the compiler sees
/// the function call and prepares the stack for it by adding a prologue and
/// epilogue around __start()– unnecessary code.
///
#[inline(always)]
fn goto_kernel() -> ! {
    unsafe {
        asm!(
            "la $29, 0xA03ffff0",               // $sp ($29) <- top of 4th MB of RAM w/ 16-byte alignment
            "la $31, {kernel_entry_address}",   // $ra
            "j {kernel_entry_address}",
            kernel_entry_address = const KERNEL_ENTRY_ADDRESS,
            options(noreturn),
        );
    }
}

/// Entry point of the bootloader
///
/// IPL2 places this program in RSP DMEM (0x04000000) and jumps to offset 0x40.
/// Note that a linker script in this crate places this function exactly at the
/// target of the jump (0xA4000040).
///
#[no_mangle]
pub extern "C" fn __start() -> ! {
    mask_interrupts();
    init_rdram();
    load_kernel();
    goto_kernel();
}

/// Required implementation for runtime panics
///
/// Important note, due to the required simplicity of this bootloader, and the
/// lack of facilities to abort execution at any time, it's important to confirm
/// panic() is impossible to reach in the call graph.
///
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// eof
