use core::arch::{arm::__nop, asm};

#[inline(always)]
pub fn wait(x: u32) {
    for _ in 0..x {
        unsafe {
            __nop();
        }
    }
}

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            // wait for interrupt
            // asm!("mcr	p15, 0, r0, c7, c0, 4")
            asm!("nop")
        }
    }
}

#[inline(never)]
pub fn exit() -> ! {
    unsafe { asm!("swi #0", options(noreturn)) }
}

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

#[macro_export]
macro_rules! get_psr {
    ($var:ident=$psr:ident) => (
        unsafe {
          asm!(
            concat!("mrs {help}, ", stringify!($psr)),
            help = out(reg) $var,
          );
        }
    );
}

#[macro_export]
macro_rules! set_psr {
    ($psr:ident=$var:ident) => (
        unsafe {
          asm!(
            concat!("msr ", stringify!($psr),", {help}"),
            help = in(reg) $var,
          );
        }
    );
}

#[inline(always)]
pub fn demask_interrupts() {
    unsafe {
        asm!(
          "MRS {reg}, CPSR",
          "BIC {reg}, #(1<<7)",
          "MSR CPSR, {reg}",
          reg = out(reg) _
        );
    }
}

#[inline(always)]
pub fn mask_interrupts() {
    unsafe {
        asm!(
        "MRS {r1}, CPSR",
        "ORR {r1}, #(1<<7)",
        "MSR CPSR, {r1}",
        r1 = out(reg) _,
        );
    }
}
