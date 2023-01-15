/// Einfach nur die Funktion, um das IVT schreiben zu enablen
use core::ptr::{write_volatile, read_volatile};

// p. 128
const MEMORY_CONTROLLER: u32 = 0xFFFFFF00;

/// MC remap
#[inline(always)]
pub fn remap() {
    unsafe {
        // Wir setzen das erste Bit
        write_volatile(MEMORY_CONTROLLER as *mut u32, 1);
    }
}

pub fn get_abort_adress() -> u32 {
    unsafe {
        read_volatile((MEMORY_CONTROLLER+8) as *mut u32)
    }
}