// SPDX-License-Identifier: GPL-3.0-or-later

//! Raku CPU kernel
//!
//! This is currently just a demo of drawing to the framebuffer.

#![no_std]
#![no_main]

#![feature(asm_experimental_arch)]
#![feature(int_roundings)]

use core::arch::asm;
use core::panic::PanicInfo;

use lib_kernel::{
    VIDEO_INTERFACE_CONTROL,
    FRAMEBUFFER_1_ADDRESS,
};

const RDP_INTERFACE: usize = 0xA4100000;
const RDP_COMMANDS: usize = 0xA0200000;

/// Initialize video output (NTSC 320x240)
fn vi_draw() {
    let vi = VIDEO_INTERFACE_CONTROL as *mut u32;
    unsafe {
        *vi.offset(0x00) = 0b00000000000000000011001100000011 as u32;
        *vi.offset(0x01) = FRAMEBUFFER_1_ADDRESS as u32;
        *vi.offset(0x02) = 320 as u32;
        *vi.offset(0x05) = 0x03E52239 as u32;
        *vi.offset(0x06) = 525 as u32;
        *vi.offset(0x07) = 0x00000C15 as u32;
        *vi.offset(0x08) = 0x0C150C15 as u32;
        *vi.offset(0x09) = 0x006C02EC as u32;  // VI_H_VIDEO
        *vi.offset(0x0A) = 0x002501FF as u32;  // VI_V_VIDEO
        *vi.offset(0x0B) = 0x000E0204 as u32;
        *vi.offset(0x0C) = 0b00000000000000000000001000000000 as u32;  // x scale, 2.10
        *vi.offset(0x0D) = 0b00000000000000000000010000000000 as u32;  // y scale
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
fn rdp_draw(step: u32, square: &mut [u16; 5]) {
    let rdp_commands = RDP_COMMANDS as *mut u64;

    let mut color: u32 = 0;

    /* Generating a 32-bit color (RGBA) from a counter

    Steps:
        1. blue
        2. blue/green
        3. green
        4. green/red
        5. red
        6. red/blue

    256 colors per step, 6 steps = 1536
    */

    let color_cycle_lg: u32 = step % (256 * 6);
    let color_cycle_sm: u32 = color_cycle_lg.div_floor(256) + 1;

    let color_value: u8 = (color_cycle_lg % 256) as u8;
    let alpha_value: u8 = 0x01;

    let r: u8;
    let g: u8;
    let b: u8;
    let a: u8;

    match color_cycle_sm {
        1 => {
            r = 0;
            g = 0;
            b = color_value;
            a = alpha_value;
        },
        2 => {
            r = 0;
            g = color_value;
            b = 0xFF;
            a = alpha_value;
        },
        3 => {
            r = 0;
            g = 0xFF;
            b = 0xFF - color_value;
            a = alpha_value;
        },
        4 => {
            r = color_value;
            g = 0xFF;
            b = 0;
            a = alpha_value;
        },
        5 => {
            r = 0xFF;
            g = 0xFF - color_value;
            b = 0;
            a = alpha_value;
        },
        6 => {
            r = 0xFF - color_value;
            g = 0;
            b = 0;
            a = alpha_value;
        },
        _ => {
            r = 0;
            g = 0;
            b = 0;
            a = 0;
        }
    }

    color |= (r as u32) << 8 * 3;
    color |= (g as u32) << 8 * 2;
    color |= (b as u32) << 8 * 1;
    color |= (a as u32) << 8 * 0;

    /* Positioning the square */

    let width: u16 = 320;
    let height: u16 = 240;

    let x_size: u16 = 0x10;
    let y_size: u16 = 0x10;

    let x_max: u16 = width - x_size;
    let y_max: u16 = height - y_size;

    // 0 down, 1 right, 2 up, 3 left
    let mut direction: u16 = square[0];
    let mut x_offset: u16 = square[1];
    let mut y_offset: u16 = square[2];
    let mut x_padding: u16 = square[3];
    let mut y_padding: u16 = square[4];

    let x_padding_max: u16 = width / 2;
    let y_padding_max: u16 = height / 2;

    // Running around the framebuffer
    match direction {
        0 => {
            y_offset += 1;
            if y_offset >= (y_max - y_padding) {
                direction += 1;
            }
        },
        1 => {
            x_offset += 1;
            if x_offset >= (x_max - x_padding) {
                direction += 1;
            }
        },
        2 => {
            y_offset -= 1;
            if y_offset <= y_padding {
                direction += 1;
            }
        },
        3 => {
            x_offset -= 1;
            if x_offset <= x_padding {
                direction = 0;
                y_padding += y_size / 2;
                x_padding += x_size / 2;
            }
        },
        _ => {
            direction = 0;
            x_offset = 0;
            y_offset = 0;
        }
    }

    if x_padding > x_padding_max {
        x_padding = 0;
    }
    if y_padding > y_padding_max {
        y_padding = 0;
    }

    // Coordinates of the rectangle
    let x_hi: u16 = x_offset;
    let y_hi: u16 = y_offset;
    let x_lo: u16 = x_offset + x_size;  // "lo" is higher in value
    let y_lo: u16 = y_offset + y_size;

    // Save state of direction and offsets
    square[0] = direction;
    square[1] = x_offset;
    square[2] = y_offset;
    square[3] = x_padding;
    square[4] = y_padding;

    // Construct RDP commands to draw a rectangle
    unsafe {

        // "Set Fill Color"
        *rdp_commands.offset(0) = ((0b0011_0111_0000_0000_0000_0000_0000_0000 as u64) << 32) | (color as u64);

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

    // Fill the framebuffer once
    let fb = FRAMEBUFFER_1_ADDRESS as *mut u32;
    for o in 0..=(320 * 240) {
        unsafe {
            *fb.offset(o) = 0x20004000;
        }
    }

    vi_draw();
    rdp_setup();

    // Direction, X offset, Y offset, X padding, Y padding
    let mut square_state: [u16; 5] = [0; 5];

    let mut counter: u32 = 0;
    loop {
        rdp_draw(counter, &mut square_state);
        counter = counter.wrapping_add(1);
        vi_draw();

        // An artificial delay in absence of a timer interrupt...
        for _ in 0..0x0FFF {
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

// eof
