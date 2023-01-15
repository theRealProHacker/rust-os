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
            asm!("mcr	p15, 0, r0, c7, c0, 4")
        }
    }
}

#[inline(always)]
pub fn exit() -> ! {
    unsafe { asm!("swi #0", options(noreturn)) }
}
