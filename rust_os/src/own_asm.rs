use core::arch::asm;

/// Initialises the mode stack pointers
#[inline(always)]
pub fn init_sps () {
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

// https://github.com/simon-auch/rust_GrandiOS/blob/master/GrandiOS/src/utils/exceptions/common_code.rs#L24
// https://web.archive.org/web/20200619135321/http://infocenter.arm.com/help/topic/com.arm.doc.dui0801b/BABBJEBD.html
#[macro_export]
macro_rules! trampolin {
    ($lr_offset:expr, $handler:ident) => (
        unsafe {
          asm!(
            concat!("sub r14, ", $lr_offset),
            "push  {{r0-r12, lr}}",
            // "sub    sp, 0x40", 
            // TODO: is below necessary and/or correct
            //make a bit of space on the stack for rust, since rust creates code like: "str r0, [pc, #4]" 
            // it expects the sp to be decremented before once. The 0x40 is a random guess and provides space for a few var$
          );
          {
              $handler();
          }
          asm!(
              // add    sp, 0x40
              "pop    {{r0-r12, pc}}",
          );
        }
    );
}
