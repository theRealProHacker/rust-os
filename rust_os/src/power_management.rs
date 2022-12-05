use volatile_register::{WO, RO};

pub struct PMC {
    // p. 276
    pub sys_clock: [WO<u32>;3],
    _reserved1: u32,
    _per_clock: [u32;3],
    _reserved2: u32,
    _unused: [u32;16],
    _int_enable: WO<u32>,
    _int_disable: WO<u32>,
    _status: RO<u32>,
    _mask: RO<u32>,
}

const PMC_ADDR: u32 = 0xFFFF_FC00;

impl PMC {
    #[inline(always)]
    pub fn new() -> &'static mut PMC {
        unsafe {&mut *(PMC_ADDR as *mut PMC)}
    }
    #[inline(always)]
    pub fn enable_sys_clock(&mut self) {
        unsafe{self.sys_clock[0].write(1);}
    }
}