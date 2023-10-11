// SPDX-License-Identifier: GPL-3.0-or-later

//! Hardware register representation

use core::marker::PhantomData;
use core::ptr;

/// A hardware register
#[repr(C)]
pub struct Register(u32);
impl Register {

    /// Volatile load of word from register
    #[inline(always)]
    pub unsafe fn lw(&self) -> u32 {
        ptr::read_volatile(&self.0)
    }

    /// Volatile store of word to register
    #[inline(always)]
    pub unsafe fn sw(&mut self, word: u32) {
        ptr::write_volatile(&mut self.0, word)
    }

}

/// Read-only word-sized hardware register.
/// The word that's read maps to a bitfield, T.
pub struct RO<T>
    where
        T: From<u32> + Copy,
{
    reg: Register,
    _t_readable: PhantomData<T>,
}

impl<T> RO<T>
    where
        T: From<u32> + Copy,
{

    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe {
            self.reg.lw().into()
        }
    }

}

/// Write-only word-sized hardware register.
/// The word that's written maps to a bitfield, T.
pub struct WO<T>
    where
        T: Into<u32> + Copy,
{
    reg: Register,
    _t_writable: PhantomData<T>,
}

impl<T> WO<T>
    where
        T: Into<u32> + Copy,
{

    #[inline(always)]
    pub unsafe fn write(&mut self, value: T) {
        self.reg.sw(
            value.into()
        )
    }

}

/// Readable and writable word-sized hardware register.
/// The word that's read maps to a bitfield, TR.
/// The word that's written maps to a bitfield, TW.
pub struct RW<TR, TW>
    where
        TR: From<u32> + Copy,
        TW: Into<u32> + Copy,
{
    reg: Register,
    _t_readable: PhantomData<TR>,
    _t_writable: PhantomData<TW>,
}

impl<TR, TW> RW<TR, TW>
    where
        TR: From<u32> + Copy,
        TW: Into<u32> + Copy,
{

    #[inline(always)]
    pub fn read(&self) -> TR {
        unsafe { self.reg.lw().into() }
    }

    #[inline(always)]
    pub unsafe fn write(&mut self, value: TW) {
        self.reg.sw(
            value.into()
        )
    }

}

// eof
