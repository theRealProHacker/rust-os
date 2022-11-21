#![no_std]
#![no_main]
#![feature(generic_arg_infer)]
#![feature(naked_functions)]

mod exceptions;
mod memory_controller;
mod serial;
use core::{ptr::{read_volatile, write_volatile}, arch::asm};

use serial::read;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

// Diese funktionieren leider nicht.
extern "C" fn data_abort_handler() -> ! {
  // Wir müssen nichts machen, da wir nie zurückspringen
  serial::Serial::new().write(b'd');
  loop {}
}

extern "C" fn undef_handler() -> ! {
  // Wir müssen nichts machen, da wir nie zurückspringen
  serial::Serial::new().write(b'u');
  loop {}
}

extern "C" fn swi_handler() -> ! {
  // Wir müssen nichts machen, da wir nie zurückspringen
  unsafe {
    write_volatile(&mut *(0xFFFFF214 as *mut _), b'u' as u32);
    // asm!(
    //   "str {r}, [{addr}]",
    //   r = in(reg) b'u',
    //   addr = in(reg) 0xFFFFF214u32,
    // );
  }
  loop {}
}

#[inline(always)]
fn raise_data_abort() {
  let _ : u32 = unsafe { read_volatile(
    0x400000 as *mut u32) 
  };
}

#[inline(always)]
fn raise_swi() {
  unsafe {
    asm!(
      "swi 0"
    )
  }
}

#[inline(always)]
fn raise_undef() {
  unsafe {
    asm!(
      ".word 0xFFFFFFFF"
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
    // exceptions::init_sps();
    loop {
        let c: u8 = read();
        println!("You typed {}, dec: {c}, hex {c:X}, pointer {:p}", c as char, &c);
        raise_swi();
    }
}