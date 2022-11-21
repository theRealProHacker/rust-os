#![allow(missing_fragment_specifier)]
use core::arch::asm;

use volatile_register::WO;

const IVT: u32 = 0;

#[repr(C)]
pub struct ExceptionTable {
  // Eigentlich RW, aber wir sollten hier nicht lesen
  reset: WO<u32>,
  undef: WO<u32>,
  swi: WO<u32>,
  prefetch: WO<u32>,
  data_abort: WO<u32>,
  reserved: WO<u32>,
  irq: WO<u32>,
  fiq: WO<u32>,
  // Hier sind die pointer zu den echten handlern
  // Wir können nicht unendlich weit springen, deswegen ein Offset von 5*4 = 20(10) = 16+4 = 14(16)
  // pub undef_handler: WO<extern fn()>, Das wäre optimal, haben wir aber nicht zum Laufen bekommen. 
  pub undef_handler: WO<u32>,
  pub swi_handler: WO<u32>,
  prefetch_handler: WO<u32>,
  pub data_abort_handler: WO<u32>,
}

impl ExceptionTable {
  pub fn new() -> &'static mut Self {
    unsafe {(&mut *(IVT as *mut Self)).init()}
  }

  fn init(&mut self)->&mut Self{
    unsafe{
      // Hier müssen wir in die Register den assembly code reinschreiben, der in den handler springt, der in den handler-Registern steht
      // https://armconverter.com/?code=ldr%20pc,%20%5Bpc,%20%230x14%5D
      // Wir haben little-endian und big-endian versucht, aber nur big hat funktioniert
      const ASM_AS_BYTES: u32 = 0xE59FF014;
      self.undef.write(ASM_AS_BYTES);
      self.swi.write(ASM_AS_BYTES);
      // self.prefetch.write(asm_as_bytes); Not needed
      self.data_abort.write(ASM_AS_BYTES);
    }
    self
  }

  // Jetzt kann man handler in die handler-Register reinschreiben
}

const USR: u32 = 0x10;
const FIQ: u32 = 0x11;
const IRQ: u32 = 0x12;
const SVC: u32 = 0x13;
const ABT: u32 = 0x17;
const UND: u32 = 0x1B;
const SYS: u32 = 0x1F;

macro_rules! set_reg {
  // Wir clearen und dann oren wir
  ($reg:tt, $value: tt) => (
    asm!(
      "bic {set_reg}, {set_reg}, #0x1F",
      concat!("orr ", stringify!($reg), ", {tmp_reg}"),
      tmp_reg = in(reg) $value,
      set_reg = out(reg) _,
    )
  )
}

/// | Modus | Stackpointer Adresse |
/// |-------|----------------------|
/// | svc (reset) | 0x0021 00FF |
/// | und (undef. instr.) | 0x0021 01FF |
/// | abt (abort) | 0x0021 02FF |
/// | irq | 0x0021 03FF |
/// | fiq | 0x0021 04FF |
pub fn init_sps () {
  // Zuerst gehen wir in einen Modus. Dann setzen wir den Stackpointer auf den oben gennanten
  unsafe {
    // set_reg!(CPSR, SVC);
    // asm!();
    asm!(
      "mrs     r0, CPSR",  //auslaesen vom status register
        "bic     r0, r0, #0x1F", //set all mode bits to zero
        "orr     r1, r0, #0x11", //ARM_MODE_FIQ
        "msr     CPSR, r1", 
        "mov     sp, #0x400", //set stack pointer for fiq mode
        "orr     r1, r0, #0x12", //ARM_MODE_IRQ
        "msr     CPSR, r12", 
        "mov     sp, #0x800", //set stack pointer for irq mode
        "orr     r1, r0, #0x13", //ARM_MODE_ABORT
        "msr     CPSR, r1", 
        "mov     sp, #0xC00", //set stack pointer for abort mode
        "orr     r1, r0, #0x17", //ARM_MODE_supervisor
        "msr     CPSR, r1", 
        "mov     sp, #0x1000", //set stack pointer for supervisor mode
        "orr     r1, r0, #0x1B", //ARM_MODE_UNDEFINED
        "msr     CPSR, r1", 
        "mov     sp, #0x1400", //set stack pointer for undefined mode
        "orr     r1, r0, #0x1F", //ARM_MODE_SYS
        "msr     CPSR, r1", 
        "mov     sp, #0x4000", //set stack pointer for system/user mode
    ); 
  };
}