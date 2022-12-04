use volatile_register::{RO, RW, WO};


const AIC_ADDR: u32 = 0xFFFFF000;


pub struct AIC {
    pub src_modes: [RW<u32>;32],
    pub src_vctrs: [RW<u32>;32],
    pub ivr: RO<u32>,
    pub fvr: RO<u32>,
    _unused: [u32;6], 
    pub enable: WO<u32>,
}

impl AIC {
    #[inline(always)]
    pub fn new() -> &'static mut AIC {
        unsafe {&mut *(AIC_ADDR as *mut AIC)}
    }

    #[inline(always)]
    pub fn init(&mut self) -> &mut Self {
        self.enable_interrupt(1);
        self
    }

    #[inline(always)]
    pub fn enable_interrupt(&mut self, index: u8) {
        unsafe {self.enable.write(1<<index)}
    }
}