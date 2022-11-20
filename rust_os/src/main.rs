#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

mod exceptions;
mod memory_controller;
mod serial;
use core::ptr::write_volatile;

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

fn raise_data_abort() {
  unsafe {
    write_volatile(0x20_1001 as *mut _,  0);
  }
}



#[link_section = ".init"]
#[no_mangle]
extern "C" fn _start() {
    println!("Starting up");
    memory_controller::remap();
    let exception_handler = exceptions::ExceptionHandler::new();
    unsafe {
      exception_handler.data_abort_handler.write(data_abort_handler as u32);
      exception_handler.undef_handler.write(undef_handler as u32);
      exception_handler.swi_handler.write(swi_handler as u32);
    };
    loop {
        let c: u8 = read();
        println!("You typed {}, dec: {c}, hex {c:X}, pointer {:p}", c as char, &c);
    }
}