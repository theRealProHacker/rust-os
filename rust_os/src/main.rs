#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use core::arch::global_asm;
use core::arch::asm;
use volatile_register::WO;

global_asm!(include_str!("start.S"));

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

const PIOB: u32 = 0xfffff600;
const YELLOW_LED: u32  = 1 << 27;

// Hier nutzen wir eine C-ähnliche Implementation mit Structs
// Die Register werden als Write-Only abstrahiert, so wie es auch in der Dokumentation steht. 
#[repr(C)]
struct LED {
  pub per: WO<u32>,
  unused0: [u32;3],
  pub oer: WO<u32>,
  unused1: [u32;7],
  pub sodr: WO<u32>,
  pub codr: WO<u32>,
}

#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags))
    };
}

fn delay(x: u32) {
  for _ in 1..x {
    nop();
  }
}

#[no_mangle]
extern "C" fn yellow_on() {
  unsafe {
    // Hier machen wir etwas "Verbotenes". Wir casten den Adressbereich ab PIOB zu einem LED-Struct
    let led: &mut LED = &mut *(PIOB as *mut LED);
    /* Initialisieren */
    led.per.write(YELLOW_LED);
    led.oer.write(YELLOW_LED);
    /* Anschalten */
    loop {
      led.sodr.write(YELLOW_LED);
      delay(100_000);
      led.codr.write(YELLOW_LED);
      delay(100_000);
    }
  }
}

// Printf Überlegung
// Wir nehmen an, dass wir eine Funktion writeString haben, die auf die serielle Schnittstelle schreibt.
// Es gibt eigentlich ein crate, das genau printf ohne libc implementiert. Die Frage ist, ob wir das einfach so,
// aber natürlich mit Quellenangabe, benutzen dürfen.
// Stattdessen gehe ich jetzt durch willcrichton.net (Link im README) durch.
// Die Hauptidee ist LinkedLists zu benutzen, um sowohl das Format als auch die angegebenen Variablen zu speichern
// printf soll ein Makro sein, damit es unbegrenztt viele Argumente verschiedenen Types annehmen kann.
// fn printf(s: &str) {

// }
