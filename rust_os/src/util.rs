//! All kinds of utilities that have not yet found a right place
//!
//! e.g. Makros, simple inlined assembly instructions and the Registers struct

use core::arch::{arm::__nop, asm};

#[inline(always)]
pub fn nop() {
    unsafe { __nop() }
}

/// Waits the given number of cpu cycles
#[allow(dead_code)]
#[inline(always)]
pub fn wait(x: u32) {
    for _ in 0..x {
        nop()
    }
}

/// Just does nothing
#[inline(always)]
pub fn idle() -> ! {
    loop {
        nop()
    }
}

/// Exits the currently running thread
#[naked]
pub extern "aapcs" fn exit() -> ! {
    unsafe { asm!("swi #0", options(noreturn)) }
}

#[macro_export]
macro_rules! get_reg {
    ($var:ident=$reg:ident) => (
        let $var: u32;
        unsafe {
          asm!(concat!("mov {}, ", stringify!($reg)), out(reg) $var);
        }
    );
}

#[macro_export]
macro_rules! get_psr {
    ($var:ident=$psr:ident) => (
        let $var: u32;
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

// Note: most of this is stolen from beispiel_4
#[macro_export]
macro_rules! trampoline {
    ($name:ident=>$handler:ident@$lr_offset:expr) => {
        #[naked]
        pub extern "aapcs" fn $name() {
            unsafe {
                asm!(
                    // push everything onto the stack
                    concat!("sub lr, ", $lr_offset),
                    "stmfd sp!, {{lr}}",
                    // Aufgrund des S-Bits ist kein Writeback möglich, also Platz auf Stack manuell reservieren
                    "sub	sp, #(15*4)",
                    "stmia sp, {{r0-r14}}^",
                    // pass the stack pointer
                    "mov r0, sp",
                    "bl {handler}",
                    // Zuvor gesicherte Register wieder herstellen (R0-R12, R13-R14 im User-Modus).
                    // Laut Doku sollte in der Instruktion nach LDM^ auf
                    // keines der umgeschalteten Register zugegriffen werden.
                    "ldmia	sp, {{r0-r14}}^
                    nop
                    add	sp, sp, #(15*4)
                
                    /* Rücksprung durch Laden des PC mit S-Bit */ 
                    ldmfd	sp!, {{pc}}^",
                    handler = sym $handler,
                    options(noreturn)
                )
            }
        }
    };
}

/// A register struct
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Registers {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
}

impl Registers {
    /// Generates empty registers initialized at 0
    pub fn empty() -> Registers {
        Registers {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            sp: 0,
            lr: 0,
            pc: 0,
        }
    }
}

/// This macro helps to generate the correct registers.
/// Given a function-call like input it will make registers that if executed as a thread will
/// give the same result as if the given function was actually called
#[macro_export]
macro_rules! thread {
    ($func:ident()) => {{
        let mut regs = Registers::empty();
        regs.pc = $func as u32;
        regs
    }};
    ($func:ident($arg1:expr)) => {{
        let mut regs = Registers::empty();
        regs.pc = $func as u32;
        regs.r0 = $arg1 as u32;
        regs
    }};
    ($func:ident($arg1:expr,$arg2:expr)) => {{
        let mut regs = Registers::empty();
        regs.pc = $func as u32;
        regs.r0 = $arg1 as u32;
        regs.r1 = $arg2 as u32;
        regs
    }};
    ($func:ident($arg1:expr, $arg2:expr, $arg3:expr)) => {{
        let mut regs = Registers::empty();
        regs.pc = $func as u32;
        regs.r0 = $arg1 as u32;
        regs.r1 = $arg2 as u32;
        regs.r2 = $arg3 as u32;
        regs
    }};
    ($func:ident($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr)) => {{
        let mut regs = Registers::empty();
        regs.pc = $func as u32;
        regs.r0 = $arg1 as u32;
        regs.r1 = $arg2 as u32;
        regs.r2 = $arg3 as u32;
        regs.r3 = $arg4 as u32;
        regs
    }};
}
