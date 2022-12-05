use volatile_register::WO;

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
  pub undef_handler: WO<u32>,
  pub swi_handler: WO<u32>,
  prefetch_handler: WO<u32>,
  pub data_abort_handler: WO<u32>,
}

impl IVT {
  #[inline(always)]
  pub fn new() -> &'static mut Self {
    unsafe {&mut *(IVT_ADDR as *mut Self)}
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
