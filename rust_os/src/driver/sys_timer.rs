//! Der System-Timer-Driver

use volatile_register::{RO, RW, WO};

const ST_ADDR: u32 = 0xFFFF_FD00;

pub struct SysTimer {
    // p. 296
    pub ctrl: u32,
    pub interval_mode: RW<u32>,
    _unused: [u32; 2],
    pub status: RO<u32>,
    pub int_enable: WO<u32>,
    pub int_disable: WO<u32>,
    pub int_mask: RO<u32>,
}

impl SysTimer {
    #[inline(always)]
    pub fn new() -> &'static mut SysTimer {
        unsafe { &mut *(ST_ADDR as *mut SysTimer) }
    }

    #[inline(always)]
    pub fn init(&mut self) -> &mut Self {
        unsafe {
            self.int_enable.write(1);
        }
        self
    }

    /// Sets the interval of the period clock
    #[inline(always)]
    pub fn set_interval(&mut self, interval: u16) {
        // Clocked at 32768 Hz -> 32768 cycles = 1s
        // not affected by power management and slow clock mode
        unsafe { self.interval_mode.write(interval as u32) }
    }
}
