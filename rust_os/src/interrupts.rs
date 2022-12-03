#![allow(dead_code)]
use volatile_register::{RO, RW};


const AIC_ADDR: u32 = 0xFFFFF000;


// sys_clock: [u32;3],
// reserved1: u32,
// per_clock: [u32;3],
// reserved2: u32,
// unused: [u32;16],
// int_enable: WO<u32>,
// int_disable: WO<u32>,
// status: RO<u32>,
// mask: RO<u32>,

pub struct AIC {
    src_modes: [RW<u32>;32],
    src_vctrs: [RW<u32>;32],
    ivr: RO<u32>,
    fvr: RO<u32>,
    // unused0: [u32;6],
}

impl AIC {
    #[inline(always)]
    pub fn new() -> &'static mut AIC {
        unsafe {&mut *(AIC_ADDR as *mut AIC)}
    }
}