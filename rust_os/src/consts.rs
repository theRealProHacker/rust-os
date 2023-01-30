//! All kind of constants
//!
//! Includes our memory layout, the number of possible threads, stack sizes and interrupt time slices

/*
Our memory layout is pretty simple:
0x2000_0000
    _start and stuff
0x2200_0000
    user memory
0x2400_0000 - Kernel Size
    kernel stacks
0x2400_0000
*/

// Kernel stuff
pub const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64 kB
pub const KERNEL_MEM: usize = 0x2400_0000;

// User stuff
// If THREAD_NUMBER is changed, also change in thread.rs
pub const THREAD_NUMBER: usize = 16;
// points to top
pub const USER_MEM: usize = 0x2400_0000 - 5 * KERNEL_STACK_SIZE;
// align this to 4
pub const USER_STACK_SIZE: usize = ((USER_MEM - 0x2200_0000) / THREAD_NUMBER) / 4 * 4;

// Time slicing
pub const TIME_SLICE: u32 = 32768;
pub const MS_PER_SLICE: u32 = TIME_SLICE * 1000 / 32768;

// Execution Modes (unfortunately actual Rust enums are pretty terrible)
pub const USR_MODE: u32 = 0x10;
// pub const FIQ_MODE: u32 = 0x11;
pub const IRQ_MODE: u32 = 0x12;
pub const SVC_MODE: u32 = 0x13;
pub const ABT_MODE: u32 = 0x17;
pub const UND_MODE: u32 = 0x1b;
pub const SYS_MODE: u32 = 0x1f;

pub const MODE_RESET: u32 = SYS_MODE;

#[inline(always)]
pub fn show_mode(mode: u32) -> &'static str {
    match mode {
        USR_MODE => "User",
        IRQ_MODE => "Interrupt",
        SVC_MODE => "Supervisor",
        ABT_MODE => "Abort",
        UND_MODE => "Undefined Instruction",
        SYS_MODE => "System",
        _ => "Unknown",
    }
}
