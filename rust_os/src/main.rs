#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

mod exceptions;
mod memory_controller;
mod serial;
use serial::read;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

#[link_section = ".init"]
#[no_mangle]
extern "C" fn _start() {
    println!("Starting up");
    
    // let handler = exceptions::ExceptionHandler::new();
    loop {
        let c: u8 = read();
        println!("You typed {}, dec: {c}, hex {c:X}, pointer {:p}", c as char, &c);
    }
}