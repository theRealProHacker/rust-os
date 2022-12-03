#![allow(dead_code)]

// pub struct PMC {

// }

const PMC_ADDR: u32 = 0x0;

use core::ptr::write_volatile;

#[inline(always)]
pub fn enable_sys_clock() {
    unsafe {write_volatile(PMC_ADDR as *mut u32, 1)}
}