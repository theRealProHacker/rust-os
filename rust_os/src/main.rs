#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

mod exceptions;
mod memory_controller;
mod serial;
use core::{ptr::{write_volatile, read_volatile}, arch::asm};

use serial::read;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

extern "C" fn data_abort_handler() -> ! {
  // Wir müssen nichts machen, da wir nie zurückspringen
  println!("Data Abort");
  loop {}
}

extern "C" fn undef_handler() -> ! {
  // Wir müssen nichts machen, da wir nie zurückspringen
  println!("Undefined Instruction");
  loop {}
}

extern "C" fn swi_handler() -> ! {
  // Wir müssen nichts machen, da wir nie zurückspringen
  println!("Software Interrupt");
  loop {}
}

#[inline(always)]
extern "C" fn raise_data_abort() {
  let _ : u32 = unsafe { read_volatile(
    0x400000 as *mut u32) 
  };
}

#[inline(always)]
extern "C" fn raise_swi() {
  unsafe {
    asm!(
      "swi 0"
    )
  }
}

#[inline(always)]
extern "C" fn raise_undef() {
  unsafe {
    asm!(
      "ldr r0 #0xFFFFFFFFFFFF"
    )
  }
}


#[link_section = ".init"]
#[no_mangle]
extern "C" fn _start() {
    println!("Starting up");
    memory_controller::remap();
    let exceptions = exceptions::ExceptionTable::new();
    unsafe {
      exceptions.data_abort_handler.write(data_abort_handler as u32);
      exceptions.undef_handler.write(undef_handler as u32);
      exceptions.swi_handler.write(swi_handler as u32);
    };
    loop {
        let c: u8 = read();
        println!("You typed {}, dec: {c}, hex {c:X}, pointer {:p}", c as char, &c);
        raise_swi();
    }
}