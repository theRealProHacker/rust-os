#![allow(dead_code)]

// pub struct PMC {
    // p. 276
    // sys_clock: [u32;3],
    // reserved1: u32,
    // per_clock: [u32;3],
    // reserved2: u32,
    // unused: [u32;16],
    // int_enable: WO<u32>,
    // int_disable: WO<u32>,
    // status: RO<u32>,
    // mask: RO<u32>,

// }

const PMC_ADDR: u32 = 0xFFFF_FC00;

use core::ptr::write_volatile;

#[inline(always)]
pub fn enable_sys_clock() {
    unsafe {write_volatile(PMC_ADDR as *mut u32, 1)}
}