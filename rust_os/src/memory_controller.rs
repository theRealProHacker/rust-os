/// Einfach nur die Funktion, um das IVT schreiben zu enablen
use core::ptr::write_volatile;


// S. 128 v 701
const MEMORY_CONTROLLER: u32 = 0xFFFFFF00;

/// MC remap
#[inline(always)]
pub fn remap() {
    unsafe{
        // Wir setzen das erste Bit
        write_volatile(MEMORY_CONTROLLER as *mut u32, 1);
    }
}
