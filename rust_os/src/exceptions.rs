#![allow(missing_fragment_specifier)]
// #![allow(dead_code)]
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
  #[inline(always)]
  pub fn new() -> &'static mut Self {
    unsafe {&mut *(IVT as *mut Self)}
  }

  #[inline(always)]
  pub fn init(&mut self)->&mut Self{
    unsafe{
      // Hier müssen wir in die Register den assembly code reinschreiben, 
      // der in den handler springt, der in den handler-Registern steht
      // https://armconverter.com/?code=ldr%20pc,%20%5Bpc,%20%230x14%5D
      // Wir haben little-endian und big-endian versucht, aber nur big hat funktioniert
      const ASM_AS_BYTES: u32 = 0xE59FF014;
      // AT91_interrupts.pdf p.2
      // https://armconverter.com/?code=ldr%20pc,%5Bpc,%23-0xF20%5D
      self.irq.write(0xE51FFF20);
      self.undef.write(ASM_AS_BYTES);
      self.swi.write(ASM_AS_BYTES);
      // self.prefetch.write(asm_as_bytes); Not needed
      self.data_abort.write(ASM_AS_BYTES);
    }
    self
  }

  // Jetzt kann man handler in die handler-Register reinschreiben
}

/// Initialises the mode stack pointers
#[inline(always)]
pub fn init_sps () {
  // Zuerst gehen wir in einen Modus. Dann setzen wir den Stackpointer auf den oben gennanten
  /*
    const USR: u32 = 0x10;
    const FIQ: u32 = 0x11;
    const IRQ: u32 = 0x12;
    const SVC: u32 = 0x13;
    const ABT: u32 = 0x17;
    const UND: u32 = 0x1B;
    const SYS: u32 = 0x1F;
  
  */
  unsafe {
    asm!(
      // Wir kopieren erst CPSR nach r1, um dann atomare Änderungen am CPSR ausführen zu können
      "mrs {r1}, CPSR",
      "mrs {backup}, CPSR",
      // Um nur die letzten 5 bits zu ändern, können wir zuerst die bits clearen (mit 1..100000 verunden)
      // und danach mit dem Modus (z.B. 0..010000) verodern
      // software interrupt
      "bic {r1}, #0x1F",
      "orr {r1}, #0x13",
      "msr CPSR, {r1}",
      // und jetzt können wir den Stackpointer in dem Modus setzen
      "mov sp, {swi_sp}",
      // undefined
      "bic {r1}, #0x1F",
      "orr {r1}, #0x1B",
      "msr CPSR, {r1}",
      "mov sp, {und_sp}",
      // abort
      "bic {r1}, #0x1F",
      "orr {r1}, #0x17",
      "msr CPSR, {r1}",
      "mov sp, {abt_sp}",
      // irq
      "bic {r1}, #0x1F",
      "orr {r1}, #0x12",
      "msr CPSR, {r1}",
      "mov sp, {irq_sp}",
      // back to backup
      "msr CPSR, {backup}",
      r1 = out(reg) _,
      backup = out(reg) _,
      swi_sp = in (reg) 0x2100FFu32,
      und_sp = in (reg) 0x2101FFu32,
      abt_sp = in (reg) 0x2102FFu32,
      irq_sp = in (reg) 0x2103FFu32,
    )
  };
}