use core::arch::asm;

#[macro_export]
macro_rules! get_reg {
    ($var:ident=$reg:ident) => (
        unsafe {
          asm!(
            concat!("mov {reg}, ", stringify!($reg)),
            reg = out(reg) $var,
          );
        }
    );
}

#[inline(always)]
pub fn demask_interrupts() {
  unsafe{
    asm!(
    "MRS {reg}, CPSR",
    "BIC {reg}, {seventh_bit}",
    "MSR CPSR, {reg}",
    reg = out(reg) _,
    seventh_bit = const 1 << 7 as u32
    );
  }
}

// #[inline(always)]
// pub fn mask_interrupts() {
//   unsafe{
//     asm!(
//     "MRS {r1}, CPSR",
//     "ORR {r1}, {seventh_bit}",
//     "MSR CPSR, {r1}",
//     r1 = out(reg) _,
//     seventh_bit = const 1 << 7 as u32
//     );
//   }
// }

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
  const STACK_BOTTOM: u32 = 0x24000000;
  const STACK_SIZE: u32 = 64*1024;
  unsafe {
    asm!(
      "mrs {backup}, CPSR",
      // sys und usr mode
      "mov sp, {sys_sp}",
      // software interrupt
      "bic {r1}, {backup}, #0x1F",
      "orr {r1}, #0x13",
      "msr CPSR, {r1}",
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
      sys_sp = in (reg) STACK_BOTTOM,
      swi_sp = in (reg) STACK_BOTTOM-STACK_SIZE*1,
      und_sp = in (reg) STACK_BOTTOM-STACK_SIZE*2,
      abt_sp = in (reg) STACK_BOTTOM-STACK_SIZE*3,
      irq_sp = in (reg) STACK_BOTTOM-STACK_SIZE*4,
      options(nomem) 
    )
  };
}

#[macro_export]
macro_rules! trampolin {
    ($lr_offset:expr, $handler:ident) => (
        unsafe {
          asm!(
            concat!("sub lr, ", $lr_offset),
            "push  {{r0-r12, lr}}",
          );
          $handler();
          asm!(
            "pop    {{r0-r12, pc}}",
          );
        }
    );
}
