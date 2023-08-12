//! Bootloader

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::arch::asm;
use core::panic::PanicInfo;

use lib_kernel::{
    VIDEO_INTERFACE_CONTROL,
    FRAMEBUFFER_1_ADDRESS,
};

const RDP_INTERFACE: usize = 0xA4100000;
const RDP_COMMANDS: usize = 0xA0200000;

/// Initialize video output (320x240)
fn vi_draw() {
    let vi = VIDEO_INTERFACE_CONTROL as *mut u32;
    unsafe {
        *vi.offset(0x00) = 0b00000000000000000011001100000011 as u32;
        *vi.offset(0x01) = FRAMEBUFFER_1_ADDRESS as u32;
        *vi.offset(0x02) = 320 as u32;
        *vi.offset(0x05) = 0x03E52239 as u32;
        *vi.offset(0x06) = 525 as u32;
        *vi.offset(0x07) = 0x00150C15 as u32;
        *vi.offset(0x08) = 0x0C150C15 as u32;
        *vi.offset(0x09) = 0x006C02EC as u32;
        *vi.offset(0x0A) = 0x00201FF as u32;
        *vi.offset(0x0B) = 0x000E0204 as u32;
        *vi.offset(0x0C) = 0b00000000000000000000001000000000 as u32;
        *vi.offset(0x0D) = 0b00000000000000000000010000000000 as u32;
    }
}

/// Busy loop wait for RDP to signal completion of command processing
fn rdp_submit_and_wait() {
    let rdp_interface = RDP_INTERFACE as *mut u32;
    let rdp_commands = RDP_COMMANDS as *mut u64;
    unsafe {
        *rdp_interface.offset(0) = (rdp_commands.offset(0) as u32) & 0x00FFFFFF;  // DP_START
        *rdp_interface.offset(1) = (rdp_commands.offset(4) as u32) & 0x00FFFFFF;  // DP_END
        loop {
            // Wait for the RDP busy-bit to clear...
            if (*rdp_interface.offset(2) & 0b0000_0000_0000_0000_0000_0000_0100_0000) == 0 {
                return;
            }
        }
    }
}

/// Send RDP commands to setup drawing at the framebuffer
fn rdp_setup() {
    let rdp_commands = RDP_COMMANDS as *mut u64;

    unsafe {

        // "Set Other Modes"
        //                                63   59   55   51   47   43   39   35   31   27   23   19   15   11   7    3  0
        *rdp_commands.offset(0) = 0b0010_1111_1011_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;

        // "Set Color Image"
        *rdp_commands.offset(1) = ((0b0011_1111_0001_1000_0000_0001_0011_1111 as u64) << 32) | ((FRAMEBUFFER_1_ADDRESS & 0x00FFFFFF) as u64);

        // "Set Scissor"
        *rdp_commands.offset(2) = 0b0010_1101_0000_0000_0000_0000_0000_0000_0000_0000_0101_0000_0000_0011_1100_0000;

        // "Full Sync"
        *rdp_commands.offset(3) = 0b0010_1001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;

        // NOP
        *rdp_commands.offset(4) = 0;

    }

    rdp_submit_and_wait();

}

/// Draw stuff using the RDP
fn rdp_draw(step: u32) {
    let rdp_commands = RDP_COMMANDS as *mut u64;

    // Generate a 32-bit color (RGBA)
    let color: u64 = step as u64;

    let x: u8 = (step & 0x5F) as u8;
    let y: u8 = ((step & 0x5F00) >> 8) as u8;

    let x_lo: u8 = x | 0x60;
    let y_lo: u8 = y | 0x60;

    let x_hi: u8 = x;
    let y_hi: u8 = y;

    // Construct RDP commands to draw a rectangle
    unsafe {

        // "Set Fill Color"
        *rdp_commands.offset(0) = ((0b0011_0111_0000_0000_0000_0000_0000_0000 as u64) << 32) | color;

        // "Fill Rectangle"
        let lower: u32 = ((x_lo as u32) << 12+2) | ((y_lo as u32) << 2);
        let upper: u32 = ((x_hi as u32) << 12+2) | ((y_hi as u32) << 2);
        *rdp_commands.offset(1) = (((0b0011_0110_0000_0000_0000_0000_0000_0000 | lower) as u64) << 32) | upper as u64;

        // "Full Sync"
        *rdp_commands.offset(2) = 0b0010_1001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;

        // NOP
        *rdp_commands.offset(3) = 0;

    }

    // Submit RDP commands
    rdp_submit_and_wait();

}

#[no_mangle]
pub extern "C" fn __start() -> ! {

    // Fill a framebuffer
    let fb = FRAMEBUFFER_1_ADDRESS as *mut u32;
    for o in 0..=(320 * 240) {
        unsafe {
            *fb.offset(o) = 0x20004000;
        }
    }

    // Initialize video output (320x240)
    vi_draw();
    rdp_setup();

    let mut step: u32 = 0xCE4F2817;
    loop {

        rdp_draw(step);
        step = (step.rotate_left(3) ^ step).saturating_add(3);

        // Delay...
        for _ in 0..0xFFFF {
            unsafe {
                asm!(r#"nop"#);
            }
        }

    }


}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
