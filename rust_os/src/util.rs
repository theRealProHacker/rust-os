use core::arch::{arm::__nop, asm};

#[allow(dead_code)]
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

/// Exits the currently running thread
#[naked]
#[no_mangle]
pub extern "aapcs" fn exit() -> ! {
    unsafe { asm!("swi #0", options(noreturn)) }
}

#[macro_export]
macro_rules! get_reg {
    ($var:ident=$reg:ident) => (
        let $var: u32;
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
        let $var;
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

// Note: most copied from beispiel_4
#[macro_export]
macro_rules! trampoline {
    ($handler:ident, $lr_offset:expr) => {
        unsafe {
            asm!(
                // push everything onto the stack
                concat!("sub	lr, ", $lr_offset),
                "stmfd sp!, {lr}",
                // Aufgrund des S-Bits ist kein Writeback möglich, also Platz auf Stack manuell reservieren
                "sub	sp, #(15*4)",
                "stmia sp, {r0-r14}^",
                // pass the stack pointer
                "mov r0, sp",
                concat!("bl ", stringify!($handler)),
                // Zuvor gesicherte Register wieder herstellen (R0-R12, R13-R14 im User-Modus).
                // Laut Doku sollte in der Instruktion nach LDM^ aufkeines der umgeschalteten Register zugegriffen werden.
                "ldmia	sp, {r0-r14}^
                nop
                add	sp, sp, #(15*4)
            
                /* Rücksprung durch Laden des PC mit S-Bit */ 
                ldmfd	sp!, {pc}^",
                options(noreturn, raw)
            )
        }
    };
}
