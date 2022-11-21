
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

/// | Modus | Stackpointer Adresse |
/// |-------|----------------------|
/// | svc (reset) | 0x0021 00FF |
/// | und (undef. instr.) | 0x0021 01FF |
/// | abt (abort) | 0x0021 02FF |
/// | irq | 0x0021 03FF |
/// | fiq | 0x0021 04FF |
fn init_sps () {
  
}