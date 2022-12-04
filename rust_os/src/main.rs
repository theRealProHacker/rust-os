#![no_std]
#![no_main]

use crate::exceptions::ExceptionTable;

mod exceptions;
mod memory_controller;
mod serial;
mod interrupts;
mod sys_timer;
mod power_management;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}


#[link_section = ".init"]
#[no_mangle]
extern "C" fn _start() {
  exceptions::init_sps();
  ExceptionTable::new().init();
  interrupts::AIC::new().init();
  serial::Serial::new().init();
  println!("Starting up");
  memory_controller::remap();
  let sys_timer = sys_timer::SysTimer::new().init();
  sys_timer.set_interval(32768);
  loop {
      let c: u8 = serial::read();
      println!("You typed {}, dec: {c}, hex {c:X}, pointer {:p}", c as char, &c);
  }
}