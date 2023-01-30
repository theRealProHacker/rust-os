/*
Sys-Calls are pretty simple.
You just put your arguments arcording to aapcs without the stack (so max 4 args).
Then you call swi with the correct code.
*/

// we use some types and an extern function from the os lib
use crate::exceptions::SWICode::*;
use crate::thread;
use crate::Registers;

macro_rules! sys_call {
    ($name:ident$args:tt -> $t:ty as $code:expr) => {
        #[naked]
        #[allow(improper_ctypes_definitions)]
        pub extern "aapcs" fn $name$args -> $t {
            use core::arch::asm;
            unsafe {
                asm!(
                "swi #{code}",
                "mov pc, lr",
                code = const $code as u32,
                options(noreturn))
            }
        }
    };
}

macro_rules! sys_calls {
    ($name:ident$args:tt -> $t:ty as $code:expr) => {
        sys_call!{$name$args -> $t as $code}
    };
    ($name:ident$args:tt -> $t:ty as $code:expr,$( $_name:ident$_args:tt -> $_t:ty as $_code:expr ),+) => {
        sys_call!{$name$args -> $t as $code}
        sys_calls!{$( $_name$_args -> $_t as $_code ),+}
    };
}

sys_calls! {
    exit() -> ! as Exit,
    _fork(regs: u32) -> thread::ID as Fork,
    _sleep(time: u32) -> () as Sleep,
    put_char(c: char) -> () as PutChar,
    read_char() -> char as ReadChar
}
/*
exit: Exit the current thread
fork:
    Create a new thread with the given Registers
    The return value indicates failure/success
    0 means failure
    any other number is the id of the created thread
sleep: Lets the current thread sleep for the given number of ms
put_char: Displays a char to the main serial output
read_char: Waits for a new char from the main serial input
*/

pub fn fork(regs: &Registers) -> usize {
    _fork(regs as *const Registers as u32)
}

/// Sleeps for the given ms
pub fn sleep(time: u32) {
    if time < 20 {
        todo!("Just sleep blockingly")
    }
    _sleep(time)
}
