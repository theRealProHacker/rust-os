use crate::{thread, Registers};

use super::syscalls::{exit, fork, put_char, read_char, sleep};

#[allow(improper_ctypes_definitions)]
extern "aapcs" fn child(c: char) {
    for _ in 0..20 {
        put_char(c);
        sleep(5000);
    }
    exit()
}

#[no_mangle]
extern "aapcs" fn main_thread() {
    loop {
        let char = read_char();
        let regs = &(thread!(child(char as u8, 4)));
        if fork(regs) == 0 {
            exit()
        }
    }
}
