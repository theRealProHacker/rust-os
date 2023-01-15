use core::ptr::read_volatile;

use volatile_register::WO;
use core::arch::asm;
use crate::{println, get_reg, trampolin, print};

const IVT_ADDR: u32 = 0;

#[repr(C)]
pub struct IVT {
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
  // Wir können nicht unendlich weit springen, deswegen ein Offset von 5*4 = 20(dec) = 1*16+4*1 = 14(hex)
  pub undef_handler: WO<extern "aapcs" fn()>,
  pub swi_handler: WO<extern "aapcs" fn()>,
  prefetch_handler: WO<extern "aapcs" fn()>,
  pub data_abort_handler: WO<extern "aapcs" fn()>,
}

impl IVT {
  #[inline(always)]
  pub fn new() -> &'static mut Self {
    unsafe {&mut *(IVT_ADDR as *mut Self)}
  }

  #[inline(always)]
  pub fn init(&mut self)->&mut Self{
    unsafe{
      // AT91_interrupts.pdf p.2
      // https://armconverter.com/?code=ldr%20pc,%5Bpc,%23-0xF20%5D
      self.irq.write(0xE51FFF20);
      // Hier müssen wir in die Register den assembly code reinschreiben, 
      // der in den handler springt, der in den handler-Registern steht
      // https://armconverter.com/?code=ldr%20pc,%20%5Bpc,%20%230x14%5D
      const ASM_AS_BYTES: u32 = 0xE59FF014;
      self.undef.write(ASM_AS_BYTES);
      self.swi.write(ASM_AS_BYTES);
      // self.prefetch.write(asm_as_bytes); Not needed
      self.data_abort.write(ASM_AS_BYTES);
    }
    unsafe {
      self.data_abort_handler.write(dab_handler);
      self.undef_handler.write(und_handler);
      self.swi_handler.write(swi_handler);
    }
    self
  }
}

// handler
extern "aapcs" fn dab_handler() {
  print!("Data abort");
  let a: u32;
  get_reg!(a=lr);
  let content = unsafe {
    read_volatile((a-8) as *const [u32;16])
  };
  println!(" at {} with context: {content:?}", a-8);
  loop{}
}

extern "aapcs" fn und_handler() {
  print!("Undefined Instruction");
  let a: u32;
  get_reg!(a=lr);
  let content = unsafe {
    read_volatile((a-8) as *const [u32;16])
  };
  println!(" at {} with context: {content:?}", a-8);
  loop{}
}

extern "aapcs" fn swi_handler() {
  trampolin!(0, _swi_handler);
}

fn _swi_handler() {
  print!("Software interrupt\n");
}
