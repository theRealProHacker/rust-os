use crate::registers::Registers;

use super::syscalls::{fork, put_char, read_char, sleep};

fn thread_function(c: char) {
    for _ in 0..20 {
        put_char(c);
        sleep(3000);
    }
}

fn main_thread() {
    loop {
        let char = read_char();
        let regs = &mut Registers::empty();
        regs.r0 = char as u8 as u32;
        regs.pc = thread_function as u32;
        fork(regs);
    }
}
