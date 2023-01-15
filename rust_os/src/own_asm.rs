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
      "BIC {reg}, #(1<<7)",
      "MSR CPSR, {reg}",
      reg = out(reg) _
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
