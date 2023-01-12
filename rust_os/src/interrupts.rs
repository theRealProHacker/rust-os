use volatile_register::{RO, RW, WO};

const AIC_ADDR: u32 = 0xFFFFF000;

#[allow(dead_code)]
pub enum SrcType {
    LowLevelSens,
    NegativeEdgeTriggered,
    HighLevelSens,
    PositiveEdgeTriggered
}

pub struct AIC {
    // p. 251
    pub src_modes: [RW<u32>;32],
    pub src_vctrs: [RW<u32>;32],
    pub ivr: RO<u32>,
    pub fvr: RO<u32>,
    _isr: u32,
    _ipr: u32,
    _imr: u32,
    _unused0: [u32;3],
    pub enable: WO<u32>,
    _unused1: [u32;3],
    pub eoicr: WO<u32>,
}

impl AIC {
    #[inline(always)]
    pub fn new() -> &'static mut AIC {
        unsafe {&mut *(AIC_ADDR as *mut AIC)}
    }

    #[inline(always)]
    pub fn enable_interrupt(&mut self, index: u8) {
        super::own_asm::demask_interrupts();
        unsafe {
            self.enable.write(1<<index);
        }
    }

    /// Setzt den handler an [index] mit Priorität [prio] und Source Typ [src_type]
    /// index muss zwischen 0 und 31 sein.
    /// prio muss zwischen 0 und 7 sein
    #[inline(always)]
    pub fn set_handler(&mut self, index: usize, handler: u32, prio: u32, src_type: SrcType) -> &mut Self {
        unsafe{
            self.src_modes[index].write(prio | ((src_type as u32)<<5));
            self.src_vctrs[index].write(handler);
        }
        self.enable_interrupt(index as u8);
        self
    }

    #[inline(always)]
    pub fn end_of_interrupt(&mut self) {
        unsafe{self.eoicr.write(1)}
    }
}