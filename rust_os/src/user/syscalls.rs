/*
Sys-Calls are pretty simple.
You just put your arguments arcording to aapcs without the stack (so max 4 args).
Then you call swi with the correct code.
*/

#![allow(dead_code)]

use core::arch::asm;
// we use some types and an extern function from the os lib
use crate::exceptions::SWICode::*;
use crate::thread;
use crate::Registers;

extern "aapcs" {
    /// Exits the current thread. Defined in util.rs
    pub fn exit() -> !;
}

#[macro_export]
macro_rules! sys_call {
    ($code:expr) => {
        unsafe { asm!(
            "swi #{code}",
            "mov pc, lr",
            code = const $code as u32,
            options(noreturn))
        }
    };
}

/// Spawns a new thread given the threads registers
#[allow(improper_ctypes_definitions)]
#[naked]
pub extern "aapcs" fn fork(regs: &mut Registers) -> Option<thread::ID> {
    sys_call!(Fork)
}

#[naked]
extern "aapcs" fn _sleep(time: u32) {
    sys_call!(Sleep)
}

/// Sleeps for the given ms
pub fn sleep(time: u32) {
    if time < 20 {
        todo!("Just sleep blockingly")
    }
    _sleep(time)
}

#[naked]
extern "aapcs" fn _put_char(c: u8) {
    sys_call!(PutChar)
}

/// Puts a char to the serial output
pub fn put_char(c: char) {
    _put_char(c as u8)
}

#[naked]
pub extern "aapcs" fn _read_char() -> u8 {
    sys_call!(ReadChar)
}

/// Reads a char from the serial input
pub fn read_char() -> char {
    _read_char() as char
}
