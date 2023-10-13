// SPDX-License-Identifier: GPL-3.0-or-later

//! Core the game
//!
//! This is currently just a simple visual demo, and an incomplete one at that.
//! There are many todo items...
//!

#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::dev::rdp::interface::RDPInterface;
use kernel::dev::rdp::commands as rdp_commands;

use n64_pac::vi;

/// Addresses of two 640x480 32-bit RGBA frame buffers
///
/// With double buffering, two operations are occurring at any time:
///     - Reading, from a frame buffer by the video interface
///     - Writing, to another frame buffer by the RCP
///
/// If both frame buffers are placed on the same physical memory chip, these
/// writes and reads will conflict and cause increased memory latency. For the
/// resolution and bit depth chosen, this is impossible to avoid within 4MB (4x
/// 1MB chips). To mitigate, each buffer is placed so that the tail end of one
/// and the tip of the other extend into the 3rd MB.
///
/// See: https://www.youtube.com/watch?v=jbr-EwCqSfs&t=915s
///
const FRAME_BUFFER_1_VADDR: usize = 0xA0100000;  // ..0xA022C000
const FRAME_BUFFER_2_VADDR: usize = 0xA02D4000;  // ..0xA0400000

static mut DISPLAY_LIST: [u64; 9] = [0; 9];

/// Initializes the video interface (NTSC, 640x480, 32-bit color)
///
/// Documentation:
///     - https://n64brew.dev/wiki/Video_Interface
///
#[inline(never)]
fn init_vi() {
    unsafe {
        vi::set_origin(FRAME_BUFFER_1_VADDR as u32);
        vi::set_ctrl(
            vi::CtrlReg(0)
                .with_dither_filter_enable(false)
                .with_pixel_advance(0b0011)
                .with_aa_mode(vi::AntiAliasMode::Disabled)
                .with_serrate(false)
                .with_divot_enable(false)
                .with_gamma_enable(false)
                .with_gamma_dither_enable(false)
                .with_depth(vi::ColorDepth::BPP32)
        );
        vi::set_width(640);
        vi::set_v_intr(0x3FF);
        vi::set_burst(
            vi::BurstReg(0)
                .with_burst_start(62)
                .with_vsync_width(5)
                .with_burst_width(34)
                .with_hsync_width(57)
        );
        vi::set_v_sync(0x20D);
        vi::set_h_sync(
            vi::HSyncReg(0)
                .with_leap(0)
                .with_h_sync(0xC15)
        );
        vi::set_h_sync_leap(
            vi::HSyncLeapReg(0)
                .with_leap_a(0xC15)
                .with_leap_b(0xC15)
        );
        vi::set_h_video(
            vi::HVideoReg(0)
                .with_h_start(0x06C)
                .with_h_end(0x2EC)
        );
        vi::set_v_video(
            vi::VVideoReg(0)
                .with_v_start(0x025)
                .with_v_end(0x1FF)
        );
        vi::set_v_burst(
            vi::VBurstReg(0)
                .with_v_burst_start(0x00E)
                .with_v_burst_end(0x204)
        );
        vi::set_x_scale(
            vi::XScaleReg(0)
                .with_x_offset(0)
                .with_x_scale(0b010000000000)
        );
        vi::set_y_scale(
            vi::YScaleReg(0)
                .with_y_offset(0)
                .with_y_scale(0b100000000000)
        );

    }
}

/// Blank the frame buffers ("fb1" & "fb2")
///
#[inline(never)]
fn init_fbs() {

    // A test pattern that was useful in debugging is written to the canvas at
    // its four corners. These will most likely be invisible on real hardware
    // due to "danger" (invisible) zones.
    let fb1 = FRAME_BUFFER_1_VADDR as *mut u32;
    unsafe {

        for px in 0..(640 * 480) as isize {
            if px % 2 == 0 {
                *fb1.offset(px) = 0xFF000000;
            } else {
                *fb1.offset(px) = 0x00FF0000;
            }
        }

        for row in 0..480 as isize {
            let offset: isize = 640 * row;
            *fb1.offset(offset + row) = 0xFFFFFF00;
            *fb1.offset(offset + 639 - row) = 0xFFFFFF00;
            *fb1.offset(offset + 480) = 0x0000FF00;
            *fb1.offset(offset + 639 - 480) = 0x0000FF00;
            if row % 32 == 0 {
                for col in 0..640 as isize {
                    let value = *fb1.offset(offset + col);
                    *fb1.offset(offset + col) = value | 0x0000FF00;
                }
            }
        }

    }

    // RDP display list to clear the frame buffers
    let clear_fbs_display_list = &[

        rdp_commands::set_other_modes::SetOtherModes(0)
            .with_opcode(rdp_commands::RDPCommands::SET_OTHER_MODES.opcode())
            .with_atomic_primitive_enable(true)
            .with_cycle_type(rdp_commands::set_other_modes::CycleType::Fill)
            .into(),

        rdp_commands::set_scissor::SetScissor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_SCISSOR.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left(0)
            .with_x_lower_right(640)
            .with_y_lower_right(480)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x99009900)
            .into(),

        rdp_commands::set_color_image::SetColorImage(0)
            .with_opcode(rdp_commands::RDPCommands::SET_COLOR_IMAGE.opcode())
            .with_address(FRAME_BUFFER_1_VADDR as u32)
            .with_model(rdp_commands::set_color_image::CanvasColorModel::RGBA)
            .with_pixel_size(rdp_commands::set_color_image::CanvasPixelSize::WORD)
            .with_width(640 - 1)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left(1 << 2)
            .with_x_lower_right(640 << 2)
            .with_y_lower_right(480 << 2)
            .into(),

        rdp_commands::set_color_image::SetColorImage(0)
            .with_opcode(rdp_commands::RDPCommands::SET_COLOR_IMAGE.opcode())
            .with_address(FRAME_BUFFER_2_VADDR as u32)
            .with_model(rdp_commands::set_color_image::CanvasColorModel::RGBA)
            .with_pixel_size(rdp_commands::set_color_image::CanvasPixelSize::WORD)
            .with_width(640 - 1)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left(1 << 2)
            .with_x_lower_right(640 << 2)
            .with_y_lower_right(480 << 2)
            .into(),

        rdp_commands::full_sync::FullSync(0)
            .with_opcode(rdp_commands::RDPCommands::FULL_SYNC.opcode())
            .into(),

        0,

    ];

    // The display list sent to the RDP is formed by copying elements from the
    // slice above into a statically defined array in module-scope. This works
    // around a problem where arrays allocated in the stack frame do not write
    // their values to memory if the compiler can't observe use of the values.
    //
    // There are workarounds like the code below, but the long term follow-up
    // here is really to just get a heap allocator and never pass pointers from
    // the stack. So, this workaround will go away very soon.
    //
    unsafe {
        DISPLAY_LIST.clone_from_slice(clear_fbs_display_list)
    }

    // Submit the commands to the RDP; see the wiki for details.
    let rdpi = RDPInterface::new();
    let ptrs = clear_fbs_display_list.as_ptr_range();
    unsafe {
        rdpi.dp_start.write(ptrs.start as u32);
        // rdpi.dp_end.write(ptrs.end as u32)
    }

}

#[no_mangle]
pub extern "C" fn __start() -> ! {
    init_vi();
    init_fbs();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// eof
