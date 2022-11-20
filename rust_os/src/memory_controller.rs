/// Einfach nur die Funktion, um das ROM schreiben zu enablen
use core::ptr::write_volatile;



const MEMORY_CONTROLLER: u32 = 0xFFFFFF00;

/// remap ermöglicht uns in den ROM zu schreiben
pub fn remap() {
    unsafe{
        // Wir setzen das erste Bit
        write_volatile(MEMORY_CONTROLLER as *mut u32, 1);
    }
}
