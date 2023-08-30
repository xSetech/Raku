// SPDX-License-Identifier: GPL-3.0-or-later

//! Kernel for a game submitted to the N64Brew Summer Game Jam, 2023

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn __start() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// eof
